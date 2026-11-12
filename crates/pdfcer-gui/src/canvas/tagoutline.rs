//! # `canvas::tagoutline` — the box around the content of the element picked
//! in the Tags panel
//!
//! A selection affordance, drawn like the current find hit: the theme's
//! selection ink, over every drawn page the element spans, each through its
//! own page's map. Nothing is drawn after an edit, because the boxes were
//! read against the page as it was.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/tagoutline.md`.

use egui::{CornerRadius, Painter, Stroke, StrokeKind};

use crate::app::state::OpenDoc;
use crate::canvas::strip::PageView;

/// The harness region of the outline on page `n` (1-based).
fn region(page: usize) -> String {
    format!("canvas.tag-outline.{page}") // ui-text-exempt: a harness region name, never displayed
}

/// Paint the picked element's boxes on each drawn page.
pub(super) fn draw(painter: &Painter, doc: &OpenDoc, pages: &[PageView]) {
    let Some(pick) = doc.tag_pick.as_ref() else {
        return;
    };
    let stroke = Stroke::new(
        2.0,
        egui_shell::theme::Theme::canvas_selection_ink(painter.ctx()),
    );
    for view in pages {
        let Some(info) = doc.pages.get(view.page) else {
            continue;
        };
        for r in pick.boxes_on(view.page, doc.edit_epoch) {
            let Some(canvas) =
                crate::canvas::geometry::pdf_rect_to_canvas((r.llx, r.lly, r.urx, r.ury), info)
            else {
                continue;
            };
            let screen = view.map.rect_to_screen(canvas);
            crate::diag::ui_rect(&region(view.page + 1), screen);
            painter.rect_stroke(
                screen.expand(2.0),
                CornerRadius::ZERO,
                stroke,
                StrokeKind::Outside,
            );
        }
    }
}
