//! File drag-and-drop on Wayland.
//!
//! winit 0.30 implements drag-and-drop only for X11. On Wayland we join
//! winit's connection (the same approach smithay-clipboard uses for the
//! clipboard), create our own `wl_data_device` on a private event queue, and
//! dispatch it on a background thread. Dropped `text/uri-list` payloads are
//! turned into paths and sent to the app.

use std::ffi::{OsString, c_void};
use std::io::Read;
use std::os::fd::AsFd;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::Mutex;

use anyhow::Context as _;
use crossbeam_channel::Sender;
use wayland_backend::client::Backend;
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_data_device::{self, WlDataDevice};
use wayland_client::protocol::wl_data_device_manager::{DndAction, WlDataDeviceManager};
use wayland_client::protocol::wl_data_offer::{self, WlDataOffer};
use wayland_client::protocol::{wl_registry, wl_seat::WlSeat};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle, event_created_child};

const URI_LIST: &str = "text/uri-list";

pub enum DndEvent {
    /// A drag carrying files entered (`true`) or left (`false`) the window.
    Hover(bool),
    Drop(Vec<PathBuf>),
}

struct State {
    tx: Sender<DndEvent>,
    repaint: egui::Context,
    /// Offer of the drag currently over our surface.
    current: Option<WlDataOffer>,
}

impl State {
    fn send(&self, ev: DndEvent) {
        let _ = self.tx.send(ev);
        self.repaint.request_repaint();
    }
}

#[derive(Default)]
struct OfferMimes(Mutex<Vec<String>>);

/// Starts listening for drops. `display` is winit's `wl_display`.
///
/// # Safety
/// `display` must be a valid `wl_display` that outlives the process' use of
/// this module (true for the main window's display).
pub unsafe fn start(display: NonNull<c_void>, tx: Sender<DndEvent>, repaint: egui::Context) -> anyhow::Result<()> {
    // SAFETY: guaranteed by the caller.
    let backend = unsafe { Backend::from_foreign_display(display.as_ptr().cast()) };
    let conn = Connection::from_backend(backend);
    let (globals, mut queue) = registry_queue_init::<State>(&conn).context("wayland registry")?;
    let qh = queue.handle();
    let manager: WlDataDeviceManager = globals.bind(&qh, 1..=3, ()).context("wl_data_device_manager")?;
    let seat: WlSeat = globals.bind(&qh, 1..=1, ()).context("wl_seat")?;
    let _device = manager.get_data_device(&seat, &qh, ());

    let mut state = State { tx, repaint, current: None };
    std::thread::Builder::new().name("rvp-dnd".into()).spawn(move || {
        // Keep the protocol objects alive for the lifetime of the thread.
        let _keep = (manager, seat, _device);
        loop {
            if let Err(e) = queue.blocking_dispatch(&mut state) {
                log::warn!("wayland dnd: dispatch failed: {e}");
                break;
            }
        }
    })?;
    log::info!("wayland drag-and-drop enabled");
    Ok(())
}

/// Parses a `text/uri-list` payload into local paths.
fn parse_uri_list(list: &str) -> Vec<PathBuf> {
    list.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.strip_prefix("file://"))
        // `file://host/path` → `/path` (host is normally empty or localhost).
        .map(|rest| &rest[rest.find('/').unwrap_or(0)..])
        .map(|p| PathBuf::from(OsString::from_vec(percent_decode(p))))
        .collect()
}

fn percent_decode(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(v) = u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("zz"), 16)
        {
            out.push(v);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

fn has_uri_list(offer: &WlDataOffer) -> bool {
    offer
        .data::<OfferMimes>()
        .is_some_and(|m| m.0.lock().unwrap_or_else(|e| e.into_inner()).iter().any(|t| t == URI_LIST))
}

fn read_offer(offer: &WlDataOffer, conn: &Connection) -> std::io::Result<String> {
    let (mut reader, writer) = std::io::pipe()?;
    offer.receive(URI_LIST.into(), writer.as_fd());
    drop(writer); // the source writes into its own copy of the fd
    conn.flush().map_err(std::io::Error::other)?;
    let mut text = String::new();
    reader.read_to_string(&mut text)?;
    Ok(text)
}

impl Dispatch<WlDataDevice, ()> for State {
    fn event(
        state: &mut Self,
        _device: &WlDataDevice,
        event: wl_data_device::Event,
        _: &(),
        conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_device::Event::Enter { serial, id, .. } => {
                if let Some(old) = state.current.take() {
                    old.destroy();
                }
                let Some(offer) = id else { return };
                if has_uri_list(&offer) {
                    offer.accept(serial, Some(URI_LIST.into()));
                    if offer.version() >= 3 {
                        offer.set_actions(DndAction::Copy, DndAction::Copy);
                    }
                    state.send(DndEvent::Hover(true));
                } else {
                    offer.accept(serial, None);
                }
                state.current = Some(offer);
            }
            wl_data_device::Event::Leave => {
                if let Some(offer) = state.current.take() {
                    offer.destroy();
                }
                state.send(DndEvent::Hover(false));
            }
            wl_data_device::Event::Drop => {
                let Some(offer) = state.current.take() else { return };
                state.send(DndEvent::Hover(false));
                if has_uri_list(&offer) {
                    match read_offer(&offer, conn) {
                        Ok(text) => {
                            let paths = parse_uri_list(&text);
                            if !paths.is_empty() {
                                state.send(DndEvent::Drop(paths));
                            }
                        }
                        Err(e) => log::warn!("wayland dnd: reading drop failed: {e}"),
                    }
                    if offer.version() >= 3 {
                        offer.finish();
                    }
                }
                offer.destroy();
            }
            wl_data_device::Event::Selection { id: Some(offer) } => {
                // Clipboard offers to *our* device are unused (egui has its own).
                offer.destroy();
            }
            _ => {}
        }
    }

    event_created_child!(State, WlDataDevice, [
        wl_data_device::EVT_DATA_OFFER_OPCODE => (WlDataOffer, OfferMimes::default()),
    ]);
}

impl Dispatch<WlDataOffer, OfferMimes> for State {
    fn event(
        _: &mut Self,
        _: &WlDataOffer,
        event: wl_data_offer::Event,
        mimes: &OfferMimes,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_data_offer::Event::Offer { mime_type } = event {
            mimes.0.lock().unwrap_or_else(|e| e.into_inner()).push(mime_type);
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSeat, ()> for State {
    fn event(_: &mut Self, _: &WlSeat, _: wayland_client::protocol::wl_seat::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<WlDataDeviceManager, ()> for State {
    fn event(
        _: &mut Self,
        _: &WlDataDeviceManager,
        _: wayland_client::protocol::wl_data_device_manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_list_parsing() {
        let list = "# comment\r\nfile:///home/me/My%20Videos/%E5%8B%95%E7%94%BB.mkv\r\nfile://localhost/tmp/a.mp4\r\nhttps://example.com/x.mp4\r\n";
        assert_eq!(
            parse_uri_list(list),
            vec![PathBuf::from("/home/me/My Videos/動画.mkv"), PathBuf::from("/tmp/a.mp4")]
        );
    }
}
