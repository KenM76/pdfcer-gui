//! The outline a single click would place, drawn under the pointer.
//!
//! `OPERATOR_REQUESTS.md` **O203**: *"when placing form items, there should be
//! a live preview of their size and placement of what we will get if we just
//! click once to place them."*
//!
//! # The contract
//!
//! [`click_rect`] is the **one** place a click's rectangle is computed, and
//! [`clicking`](crate::canvas::clicking) calls it rather than building its own.
//! That is the whole reason this module exists as something other than a
//! painter: a preview computed beside the placement rather than from it is a
//! second derivation of the same geometry, and the two would agree on an
//! unrotated page and on nothing else. The one test here asserts they are the
//! same value, at every `/Rotate`, because a value cannot say which producer
//! made it.
//!
//! # Rule 4
//!
//! This is the **cursor**, not content. Nothing is written, nothing on the page
//! is marked, and the outline disappears the instant the click lands — what
//! replaces it is the field itself, rendered exactly as a saved-and-reopened
//! file will render it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/formfield/ghost.md`.

use egui::{Pos2, Ui};
use pdfcer_core::page_tree::{Page, Rect as PdfRect};

use super::FormFieldKind;
use crate::canvas::mapping::{self, PageMapping};

/// How wide the ghost's outline is drawn, in screen pixels.
const GHOST_PX: f32 = 1.5;

/// The `/Rect` a single click at `at` would give a field of `kind`.
#[must_use]
pub(in crate::canvas) fn click_rect(kind: FormFieldKind, at: Pos2, page: &Page) -> Option<PdfRect> {
    let (corner, _) = crate::canvas::markup::band::endpoints(at, at, page)?;
    let (w, h) = kind.default_size_pt();
    Some(PdfRect {
        llx: corner.0,
        lly: corner.1,
        urx: corner.0 + w,
        ury: corner.1 + h,
    })
}

/// Draw the outline a click would produce, following the pointer.
pub(in crate::canvas) fn preview(
    ui: &Ui,
    page: Option<&Page>,
    kind: FormFieldKind,
    map: &PageMapping,
    pointer: Option<Pos2>,
) {
    let (Some(page), Some(at)) = (page, pointer) else {
        return;
    };
    let Some(rect) = click_rect(kind, at, page) else {
        return;
    };
    let Some(canvas) = mapping::annot_canvas_rect([rect.llx, rect.lly, rect.urx, rect.ury], page)
    else {
        return;
    };
    let screen = map.rect_to_screen(canvas);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // Both rectangles on one line: the PDF `/Rect` the click would write
        // and the screen box the operator is looking at. A ghost that tracks
        // the pointer while promising the wrong `/Rect` and a ghost that
        // promises the right one while drawn in the wrong place are the two
        // ways this can fail, and a screenshot separates neither.
        format!(
            "form-ghost kind={kind:?} rect={:.2},{:.2},{:.2},{:.2} \
             screen={:.1},{:.1},{:.1},{:.1}",
            rect.llx,
            rect.lly,
            rect.urx,
            rect.ury,
            screen.min.x,
            screen.min.y,
            screen.max.x,
            screen.max.y,
        )
    });
    ui.painter().rect_stroke(
        screen,
        0.0,
        egui::Stroke::new(
            GHOST_PX,
            egui_shell::theme::Theme::canvas_selection_ink(ui.ctx()),
        ),
        egui::StrokeKind::Outside,
    );
}

#[cfg(test)]
mod tests {
    use super::{FormFieldKind, click_rect};
    use crate::canvas::mapping::annot_canvas_rect;
    use egui::Pos2;

    /// A letter page at a given `/Rotate`.
    fn projection_page(rotate: u16) -> pdfcer_core::page_tree::Page {
        pdfcer_core::page_tree::Page {
            id: pdfcer_core::object::ObjId::new(9, 0),
            resources: pdfcer_core::object::Dict::new(),
            media_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
            crop_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
            crop_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            bleed_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
            bleed_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            trim_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
            trim_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            art_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
            art_box_resolution: pdfcer_core::page_tree::BoxResolution::Defaulted,
            rotate,
            contents: Vec::new(),
            contents_unresolved: 0,
            resources_defaulted: false,
            contents_flattened: 0,
        }
    }

    /// Canvas units are points at scale 1, so a point-space tolerance reads as
    /// a fraction of a point on screen.
    const TOL: f32 = 0.01;

    /// **The outline drawn under the pointer is the box the click writes**,
    /// at every `/Rotate`.
    #[test]
    fn the_ghost_is_the_rect_the_click_places_at_every_rotation() {
        let at = Pos2::new(120.0, 300.0);
        for rotate in [0u16, 90, 180, 270] {
            let page = projection_page(rotate);
            for kind in FormFieldKind::ALL {
                let rect = click_rect(kind, at, &page).expect("a letter page projects");
                let (w, h) = kind.default_size_pt();

                assert!(
                    (rect.urx - rect.llx - w).abs() < f64::from(TOL)
                        && (rect.ury - rect.lly - h).abs() < f64::from(TOL),
                    "{kind:?} at /Rotate {rotate} sized {}x{}, wanted {w}x{h}",
                    rect.urx - rect.llx,
                    rect.ury - rect.lly,
                );

                let canvas = annot_canvas_rect([rect.llx, rect.lly, rect.urx, rect.ury], &page)
                    .expect("a real rect on a real page projects");

                let corners = [
                    Pos2::new(canvas.min.x, canvas.min.y),
                    Pos2::new(canvas.max.x, canvas.min.y),
                    Pos2::new(canvas.min.x, canvas.max.y),
                    Pos2::new(canvas.max.x, canvas.max.y),
                ];
                assert!(
                    corners
                        .iter()
                        .any(|c| (c.x - at.x).abs() < TOL && (c.y - at.y).abs() < TOL),
                    "{kind:?} at /Rotate {rotate}: the click at {at:?} is no corner of {canvas:?}"
                );

                let quarter_turned = rotate % 180 != 0;
                let (want_w, want_h) = if quarter_turned { (h, w) } else { (w, h) };
                assert!(
                    (f64::from(canvas.width()) - want_w).abs() < f64::from(TOL)
                        && (f64::from(canvas.height()) - want_h).abs() < f64::from(TOL),
                    "{kind:?} at /Rotate {rotate} drew {}x{}, wanted {want_w}x{want_h}",
                    canvas.width(),
                    canvas.height(),
                );
            }
        }
    }
}
