//! # `app::pageonly` — the sentence that says how to leave a page-only window
//!
//! Full screen removes every control, so for a few seconds after it starts a line
//! over the top of the page names the way out. It takes no clicks.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/pageonly.md`.

use egui::{Align2, Area, Frame, Id, Order, Rect, Vec2};

use crate::text::window as t;

/// How long the sentence stays, in seconds.
const SHOWN_SECONDS: f64 = 3.0;

/// How far below the top of the window it sits, in points.
const TOP_GAP_PTS: f32 = 16.0;

/// The region a driven check measures.
const REGION: &str = "fullscreen-hint"; // ui-text-exempt: trace region name, never displayed

/// Memory key for the time page-only began, absent while it is off.
const STARTED: &str = "pdfcer.pageonly.started"; // ui-text-exempt: memory key, never displayed

/// Draw the sentence while `page_only` is young; call once per frame.
pub(crate) fn hint(ctx: &egui::Context, page_only: bool) {
    let id = Id::new(STARTED);
    if !page_only {
        ctx.data_mut(|d| d.remove_temp::<f64>(id));
        return;
    }
    let now = ctx.input(|i| i.time);
    let started = ctx.data(|d| d.get_temp::<f64>(id));
    let started = started.unwrap_or_else(|| {
        ctx.data_mut(|d| d.insert_temp(id, now));
        now
    });
    let chord = crate::app::window::fullscreen_chord(ctx);
    let line = t::fullscreen_hint(chord.as_deref());
    crate::diag::trace_changed("fullscreen-hint", || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("fullscreen-hint line={line:?}")
    });
    let left = SHOWN_SECONDS - (now - started);
    if left <= 0.0 {
        return;
    }
    let top = ctx.content_rect().center_top() + Vec2::new(0.0, TOP_GAP_PTS);
    let shown = Area::new(Id::new(REGION))
        .order(Order::Foreground)
        .interactable(false)
        .pivot(Align2::CENTER_TOP)
        .fixed_pos(top)
        .show(ctx, |ui| {
            Frame::popup(ui.style()).show(ui, |ui| ui.label(&line));
        });
    let rect: Rect = shown.response.rect;
    crate::diag::ui_rect(REGION, rect);
    ctx.request_repaint_after(std::time::Duration::from_secs_f64(left));
}
