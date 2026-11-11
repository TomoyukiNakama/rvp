//! Custom seek bar: thin track that thickens on hover, knob, and a time preview.

use egui::{Color32, CornerRadius, FontId, Rect, Sense, Stroke, Ui, pos2, vec2};

use super::widgets::format_time;
use crate::theme::color;

pub enum SeekEvent {
    /// The user is dragging; `exact` is false (cheap keyframe seek for scrubbing).
    Scrub(f64),
    /// Click or drag release: final, precise position.
    Commit(f64),
}

/// Draws the seek bar filling the available width. `position`/`duration` in seconds.
pub fn seek_bar(ui: &mut Ui, position: f64, duration: Option<f64>) -> Option<SeekEvent> {
    let height = 22.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click_and_drag());
    let Some(duration) = duration.filter(|d| *d > 0.0) else {
        paint_track(ui, rect, 0.0, false);
        return None;
    };

    let id = resp.id;
    let to_time = |x: f32| -> f64 {
        let t = ((x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        f64::from(t) * duration
    };

    let mut event = None;
    let dragging = resp.dragged();
    if let Some(p) = resp.interact_pointer_pos() {
        let t = to_time(p.x);
        if resp.drag_stopped() || (resp.clicked() && !dragging) {
            event = Some(SeekEvent::Commit(t));
            ui.ctx().data_mut(|d| d.remove::<f64>(id));
        } else if dragging {
            // Throttle scrubbing to distinct positions.
            let last = ui.ctx().data(|d| d.get_temp::<f64>(id));
            if last.is_none_or(|l| (l - t).abs() > duration / 2000.0) {
                ui.ctx().data_mut(|d| d.insert_temp(id, t));
                event = Some(SeekEvent::Scrub(t));
            }
        }
    }

    let shown = match (&event, dragging) {
        (Some(SeekEvent::Scrub(t)), _) => *t,
        (_, true) => ui.ctx().data(|d| d.get_temp::<f64>(id)).unwrap_or(position),
        _ => position,
    };
    let frac = (shown / duration).clamp(0.0, 1.0) as f32;
    let active = resp.hovered() || dragging;
    paint_track(ui, rect, frac, active);

    // Hover preview of the time under the cursor.
    if let Some(hover) = resp.hover_pos().or(dragging.then(|| resp.interact_pointer_pos()).flatten()) {
        let t = to_time(hover.x);
        let text = format_time(t);
        let font = FontId::proportional(12.5);
        let galley = ui.painter().layout_no_wrap(text, font, color::TEXT);
        let pad = vec2(8.0, 4.0);
        let size = galley.size() + pad * 2.0;
        let x = hover.x.clamp(rect.left() + size.x / 2.0, rect.right() - size.x / 2.0);
        let bubble = Rect::from_center_size(pos2(x, rect.top() - size.y / 2.0 - 6.0), size);
        let p = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, id.with("bubble")));
        p.rect(bubble, CornerRadius::same(7), color::SURFACE, Stroke::new(1.0, color::BORDER), egui::StrokeKind::Inside);
        p.galley(bubble.min + pad, galley, color::TEXT);
    }

    event
}

fn paint_track(ui: &Ui, rect: Rect, frac: f32, active: bool) {
    let t = ui.ctx().animate_bool_with_time(ui.id().with("seek_hover"), active, 0.12);
    let thickness = egui::lerp(4.0..=6.0, t);
    let track = Rect::from_center_size(rect.center(), vec2(rect.width(), thickness));
    let r = CornerRadius::same((thickness / 2.0).round() as u8);
    let p = ui.painter();
    p.rect_filled(track, r, color::TRACK);
    if frac > 0.0 {
        let mut filled = track;
        filled.set_right(track.left() + track.width() * frac);
        let c = if active { color::ACCENT_HOVER } else { color::ACCENT };
        p.rect_filled(filled, r, c);
    }
    if t > 0.0 {
        let center = pos2(track.left() + track.width() * frac, track.center().y);
        p.circle_filled(center, 7.0 * t, Color32::WHITE);
        p.circle_stroke(center, 7.0 * t, Stroke::new(1.0, Color32::from_black_alpha(60)));
    }
}
