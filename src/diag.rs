//! Diagnostics for problems that are hard to reproduce:
//!
//! - every run logs to `<state dir>/rvp.log` (as well as stderr); the previous
//!   runs are kept as `rvp.log.1` … `rvp.log.3`, and a log stops growing at
//!   [`MAX_LOG_BYTES`];
//! - panics are written to the log with a backtrace;
//! - a watchdog thread notices when the UI thread stops producing frames even
//!   though a repaint was requested, and logs the app state together with
//!   backtraces of the UI and drag-and-drop threads.
//!
//! The state dir is `~/.local/state/rvp/` (Flatpak: `~/.var/app/<id>/.local/state/rvp/`).

use std::backtrace::Backtrace;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const MAX_LOG_BYTES: u64 = 20 * 1024 * 1024;
const KEEP_OLD_LOGS: u32 = 3;
/// How often the watchdog checks, and after how long without a frame it asks for one.
const CHECK_EVERY: Duration = Duration::from_secs(1);
const IDLE_PROBE_AFTER: Duration = Duration::from_secs(2);
/// No frame this long after the watchdog's repaint request = stalled.
const STALL_AFTER: Duration = Duration::from_secs(5);

pub fn log_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "rvp")?;
    Some(dirs.state_dir().unwrap_or(dirs.data_local_dir()).join("rvp.log"))
}

// ---- Log file ------------------------------------------------------------------

struct LogFile {
    file: Option<File>,
    written: u64,
}

static LOG_FILE: Mutex<LogFile> = Mutex::new(LogFile { file: None, written: 0 });

/// env_logger target: stderr + the log file.
struct Tee;

impl Write for Tee {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = std::io::stderr().write_all(buf);
        let mut log = LOG_FILE.lock().unwrap_or_else(|e| e.into_inner());
        if log.written < MAX_LOG_BYTES {
            let full = log.written + buf.len() as u64 >= MAX_LOG_BYTES;
            if let Some(f) = log.file.as_mut() {
                let _ = f.write_all(buf);
                if full {
                    let _ = f.write_all(b"[log size limit reached; further messages go to stderr only]\n");
                }
            }
            log.written += buf.len() as u64;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(f) = LOG_FILE.lock().unwrap_or_else(|e| e.into_inner()).file.as_mut() {
            f.flush()?;
        }
        Ok(())
    }
}

fn open_log_file() -> Option<(File, PathBuf)> {
    let path = log_path()?;
    std::fs::create_dir_all(path.parent()?).ok()?;
    let old = |n: u32| path.with_extension(format!("log.{n}"));
    for n in (1..KEEP_OLD_LOGS).rev() {
        let _ = std::fs::rename(old(n), old(n + 1));
    }
    let _ = std::fs::rename(&path, old(1));
    Some((File::create(&path).ok()?, path))
}

/// Sets up logging (stderr + file; `RUST_LOG` overrides the default filter) and the panic hook.
pub fn init_logging() {
    let file = open_log_file();
    let path = file.as_ref().map(|(_, p)| p.clone());
    LOG_FILE.lock().unwrap_or_else(|e| e.into_inner()).file = file.map(|(f, _)| f);

    // rvp at debug (drops, window state, playback events); everything else warnings only.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn,rvp=debug"))
        .format_timestamp_millis()
        .format(|buf, record| {
            let thread = std::thread::current();
            writeln!(
                buf,
                "[{} {:5} {} {}] {}",
                buf.timestamp_millis(),
                record.level(),
                thread.name().unwrap_or("?"),
                record.target(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(Tee)))
        .init();

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panic: {info}\n{}", Backtrace::force_capture());
        let _ = Tee.flush();
        default_hook(info);
    }));

    log::info!(
        "rvp {} starting (pid {}, flatpak: {}, log: {})",
        env!("CARGO_PKG_VERSION"),
        std::process::id(),
        std::env::var("FLATPAK_ID").unwrap_or_else(|_| "no".into()),
        path.map_or_else(|| "none".into(), |p| p.display().to_string()),
    );
    register_thread("main");
}

// ---- Watchdog ------------------------------------------------------------------

static START: OnceLock<Instant> = OnceLock::new();
/// Milliseconds since start of the last UI pass.
static LAST_PASS_MS: AtomicU64 = AtomicU64::new(0);
/// Short description of the app state, refreshed by the UI thread.
static STATE: Mutex<String> = Mutex::new(String::new());
/// Threads whose stacks are dumped on a stall: (name, kernel tid).
static THREADS: Mutex<Vec<(&'static str, i32)>> = Mutex::new(Vec::new());
/// Backtrace captured by the signal handler of the thread being dumped.
static DUMP: Mutex<Option<Backtrace>> = Mutex::new(None);
static DUMP_TID: AtomicI32 = AtomicI32::new(0);
static HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);

fn now_ms() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// Marks the calling thread for stack dumps on a stall.
pub fn register_thread(name: &'static str) {
    // SAFETY: gettid has no preconditions.
    let tid = unsafe { libc::gettid() };
    THREADS.lock().unwrap_or_else(|e| e.into_inner()).push((name, tid));
}

/// Called by the UI thread at the start of every pass.
pub fn ui_pass() {
    LAST_PASS_MS.store(now_ms(), Ordering::Relaxed);
}

/// Updates the state shown in stall reports (cheap; call at most a few times a second).
pub fn set_state(state: String) {
    *STATE.lock().unwrap_or_else(|e| e.into_inner()) = state;
}

pub fn start_watchdog(ctx: egui::Context) {
    now_ms();
    ui_pass();
    let spawned = std::thread::Builder::new().name("rvp-watchdog".into()).spawn(move || {
        let mut stalled_since: Option<u64> = None;
        let mut probe_at: Option<u64> = None;
        loop {
            std::thread::sleep(CHECK_EVERY);
            let now = now_ms();
            let last = LAST_PASS_MS.load(Ordering::Relaxed);
            if let Some(since) = stalled_since {
                if last > since {
                    log::warn!("watchdog: UI recovered after {:.1} s without frames", (last - since) as f64 / 1000.0);
                    stalled_since = None;
                    probe_at = None;
                }
                continue;
            }
            match probe_at {
                // A frame came after the probe (or the UI is busy anyway): all fine.
                Some(p) if last >= p => probe_at = None,
                Some(p) if now - p >= STALL_AFTER.as_millis() as u64 => {
                    report_stall(now - last);
                    stalled_since = Some(last);
                    probe_at = None;
                }
                Some(_) => {}
                None if now - last >= IDLE_PROBE_AFTER.as_millis() as u64 => {
                    // Idle is normal (no repaints needed); make sure a repaint still works.
                    probe_at = Some(now);
                    ctx.request_repaint();
                }
                None => {}
            }
        }
    });
    if let Err(e) = spawned {
        log::warn!("watchdog not started: {e}");
    }
}

fn report_stall(since_ms: u64) {
    let state = STATE.lock().unwrap_or_else(|e| e.into_inner()).clone();
    log::error!(
        "watchdog: UI thread stalled — no frame for {:.1} s although a repaint was requested\n  state: {state}",
        since_ms as f64 / 1000.0
    );
    let threads = THREADS.lock().unwrap_or_else(|e| e.into_inner()).clone();
    for (name, tid) in threads {
        match dump_stack(tid) {
            Some(bt) => log::error!("watchdog: stack of thread {name} (tid {tid}):\n{bt}"),
            None => log::error!("watchdog: no stack for thread {name} (tid {tid})"),
        }
    }
    let _ = Tee.flush();
}

extern "C" fn dump_handler(_: libc::c_int) {
    // Only the thread being dumped records its stack. Capturing allocates, which is
    // not async-signal-safe, but the stalled threads are normally parked in a syscall.
    // SAFETY: gettid has no preconditions.
    if unsafe { libc::gettid() } == DUMP_TID.load(Ordering::Acquire)
        && let Ok(mut slot) = DUMP.try_lock()
    {
        *slot = Some(Backtrace::force_capture());
    }
}

/// Asks thread `tid` to capture its own backtrace (SIGUSR2); waits briefly for it.
fn dump_stack(tid: i32) -> Option<Backtrace> {
    if !HANDLER_INSTALLED.swap(true, Ordering::AcqRel) {
        // SAFETY: installs a handler for SIGUSR2, which nothing else in rvp uses.
        unsafe {
            let mut sa: libc::sigaction = std::mem::zeroed();
            sa.sa_sigaction = dump_handler as *const () as usize;
            sa.sa_flags = libc::SA_RESTART;
            libc::sigemptyset(&mut sa.sa_mask);
            libc::sigaction(libc::SIGUSR2, &sa, std::ptr::null_mut());
        }
    }
    *DUMP.lock().unwrap_or_else(|e| e.into_inner()) = None;
    DUMP_TID.store(tid, Ordering::Release);
    // SAFETY: plain syscall; a vanished tid just fails.
    let sent = unsafe { libc::syscall(libc::SYS_tgkill, libc::getpid(), tid, libc::SIGUSR2) } == 0;
    if !sent {
        return None;
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if let Some(bt) = DUMP.lock().unwrap_or_else(|e| e.into_inner()).take() {
            return Some(bt);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    None
}
