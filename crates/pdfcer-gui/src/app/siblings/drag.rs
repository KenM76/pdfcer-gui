//! A document tab dragged off the strip. Dropped on another pdfcer-gui
//! window, the document moves there; dropped anywhere else — this window's
//! own canvas, the desktop — it tears off into a window of its own, as a
//! browser tab does. The window's only document has nowhere new to go and
//! stays.
//!
//! Positions cross windows in desktop pixels: each window publishes its
//! client area in them (`wire`'s `rect=`), and a drop point is converted to
//! them from this window's points. Points include the operator's interface
//! scale, so the conversion uses [`egui::Context::pixels_per_point`], which
//! does too.

use eframe::egui;

use super::PdfcerApp;
use crate::text::siblings::WindowMoveRefusal;

/// This window's client area in desktop pixels, `[left, top, right, bottom]`.
pub(super) fn client_rect_px(ctx: &egui::Context) -> Option<[i32; 4]> {
    let inner = ctx.input(|i| i.viewport().inner_rect)?;
    let ppp = ctx.pixels_per_point();
    #[allow(clippy::cast_possible_truncation)] // desktop coordinates fit an i32
    let px = |v: f32| (v * ppp).round() as i32;
    Some([
        px(inner.left()),
        px(inner.top()),
        px(inner.right()),
        px(inner.bottom()),
    ])
}

/// The desktop pixel under `at`, a point in this window.
fn desktop_px(ctx: &egui::Context, at: egui::Pos2) -> Option<egui::Pos2> {
    let inner = ctx.input(|i| i.viewport().inner_rect)?;
    Some(((inner.min + at.to_vec2()).to_vec2() * ctx.pixels_per_point()).to_pos2())
}

impl PdfcerApp {
    /// The tab in `slot` was dragged off the strip and released at `at`.
    pub(in crate::app) fn tab_dropped_outside(
        &mut self,
        ctx: &egui::Context,
        slot: usize,
        at: egui::Pos2,
    ) {
        let inside = ctx.content_rect().contains(at);
        let desktop = desktop_px(ctx, at);
        let target = desktop
            .filter(|_| !inside)
            .and_then(|p| self.siblings.peers.iter().find(|w| w.contains(p.x, p.y)))
            .cloned();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "window-tab-dropped slot={slot} at={:.0},{:.0} desktop={} onto={}",
                at.x,
                at.y,
                desktop.map_or_else(String::new, |p| format!("{:.0},{:.0}", p.x, p.y)),
                target.as_ref().map_or(0, |w| w.pid)
            )
        });
        if let Some(peer) = target {
            let Some(path) = self.movable(slot) else {
                crate::app::status::decline::record_window_move(WindowMoveRefusal::Unsaved);
                return;
            };
            self.siblings.send(ctx, &peer, &path, self.page_of(slot));
        } else if self.document_count() > 1 {
            self.move_to_new_window(slot);
        }
    }
}
