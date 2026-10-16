//! **The width handle of an open paragraph** — drag its right edge and the
//! paragraph re-wraps at that width (`OPERATOR_REQUESTS.md` O295).
//!
//! Contract: offered only while the text tool is armed and the draft is a
//! paragraph (`Anchor::Block`) on a drawn page. A drag that ends at a width
//! different from the block's own settles the draft — committing any typing
//! first — and raises `TextAction::Reflow` with `width` in PDF points. The
//! rubber band drawn during the drag is the cursor, not content (rule 4).
//!
//! Registered in the page layer after the page widgets and before the gesture
//! layer, as the guides are, so a press on the handle never reaches the
//! page's `Response` and cannot also move the caret or start a marquee.
//!
//! The width is read along the paragraph's own x axis in PDF space, so the
//! handle sits on whichever screen edge `urx` maps to on a rotated page.

use egui::{Id, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use super::{Anchor, read};
use crate::app::actions::Action;
use crate::app::actions::text::TextAction;
use crate::app::state::OpenDoc;
use crate::canvas::strip::PageView;
use crate::viewer;

/// Half the handle's reach either side of the edge, in screen points.
const REACH: f32 = 5.0;
/// The side of the drawn grip square, in screen points.
const GRIP: f32 = 7.0;
/// The narrowest column a drag may ask for, in PDF points — about two
/// characters of 10 pt body text, below which every word is its own line.
const MIN_WIDTH_PT: f64 = 18.0;
/// A drag that moves the edge less than this, in PDF points, is no request.
const DEAD_PT: f64 = 0.5;
/// egui temp-data key for the width the current drag would ask for.
const STATE: &str = "textedit.width-grip";

/// Offer the handle and, on release, raise the re-wrap. Returns whether the
/// handle took the pointer this frame.
pub(in crate::canvas) fn offer(
    ui: &Ui,
    doc: &OpenDoc,
    pages: &[PageView],
    actions: &mut Vec<Action>,
) -> bool {
    let Some(draft) = read(ui.ctx()) else {
        return false;
    };
    let Anchor::Block {
        block,
        llx,
        lly,
        urx,
        ury,
        recognised: false,
        ..
    } = draft.anchor
    else {
        return false;
    };
    let (Some(view), Some(sheet)) = (
        pages.iter().find(|v| v.page == draft.page),
        doc.pages.get(draft.page),
    ) else {
        return false;
    };
    let screen = |x: f64, y: f64| -> Option<Pos2> {
        #[allow(clippy::cast_possible_truncation)]
        let canvas = viewer::pdf_space_to_canvas(Pos2::new(x as f32, y as f32), sheet)?;
        Some(view.map.to_screen(canvas))
    };
    let (Some(top), Some(bottom)) = (screen(urx, ury), screen(urx, lly)) else {
        return false;
    };
    let edge = Rect::from_two_pos(top, bottom);
    let upright = edge.width() <= edge.height();
    let reach = if upright {
        edge.expand2(Vec2::new(REACH, 0.0))
    } else {
        edge.expand2(Vec2::new(0.0, REACH))
    };
    let id = Id::new((STATE, draft.page, block));
    let response = ui.interact(reach, id, Sense::drag());
    // ui-text-exempt: a diagnostic region name, never displayed.
    crate::diag::ui_rect("text.paragraph-width", reach);
    let cursor = if upright {
        egui::CursorIcon::ResizeHorizontal
    } else {
        egui::CursorIcon::ResizeVertical
    };
    if response.hovered() || response.dragged() {
        ui.ctx().set_cursor_icon(cursor);
    }
    let accent = egui_shell::theme::Theme::of(ui.ctx()).palette.accent;
    let painter = ui.painter();
    painter.rect_filled(
        Rect::from_center_size(edge.center(), Vec2::splat(GRIP)),
        1.0,
        accent,
    );

    let wanted = response
        .interact_pointer_pos()
        .and_then(|p| viewer::canvas_to_pdf_space(view.map.to_page(p), sheet))
        .map(|pdf| (f64::from(pdf.x) - llx).max(MIN_WIDTH_PT));
    let ctx = ui.ctx();
    if let Some(width) = wanted {
        ctx.data_mut(|d| d.insert_temp(Id::new(STATE), width));
    }
    // The release frame carries no pointer position; the last one stands.
    let width = wanted.or_else(|| ctx.data(|d| d.get_temp::<f64>(Id::new(STATE))));
    if response.dragged()
        && let Some(width) = width
        && let (Some(a), Some(b)) = (screen(llx, ury), screen(llx + width, lly))
    {
        painter.rect_stroke(
            Rect::from_two_pos(a, b),
            0.0,
            Stroke::new(1.5, accent),
            egui::StrokeKind::Middle,
        );
    }
    if response.drag_stopped() {
        ctx.data_mut(|d| d.remove::<f64>(Id::new(STATE)));
        if let Some(width) = width.filter(|w| (w - (urx - llx)).abs() > DEAD_PT) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "text-edit-width-dragged page={} block={block} width={width:.2} was={:.2}",
                    draft.page,
                    urx - llx
                )
            });
            super::settle(ctx, actions);
            actions.push(Action::Text(TextAction::Reflow {
                page: draft.page,
                block,
                width: Some(width),
            }));
        }
    }
    response.hovered() || response.dragged() || response.drag_stopped()
}
