//! # `canvas::backdrop` — the low-resolution page under the sharp one
//!
//!
//! > *"the screen should never be blank while waiting to render when zooming
//! > out — there should be at least a low resolution zoom of the newly panned
//! > or zoomed out area instead of just remaining blank while the higher
//! > definition render occurs."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/backdrop.md`.

use egui::Ui;

use crate::app::state::OpenDoc;

/// Paint the backdrop for `page`, and say whether anything was painted.
pub(super) fn paint(ui: &Ui, doc: &OpenDoc, page: usize, current: usize, rect: egui::Rect) -> bool {
    // Only for the CURRENT page. A strip neighbour has its own texture and
    // its own "no texture yet" state, which `render::strip::draw_page_state`
    // already answers honestly; giving it this page's pixels would be drawing
    // one page on another.
    if page != current {
        return false;
    }
    let Some(base) = doc
        .base_texture
        .as_ref()
        .filter(|t| t.key.page() == page)
        // Per-page (O74). The rule this enforces is unchanged and is rule
        // 4's — "a backdrop from before an edit would show content the document
        // no longer has" — but the question is now asked of the page the
        // backdrop is a picture of, rather than of the whole document.
        .filter(|_| doc.base_texture_epoch == doc.page_epochs.get(doc.view.page_index))
    else {
        return false;
    };
    egui::Image::from_texture(&base.texture).paint_at(ui, rect);
    true
}

/// Publish how much of the visible page actually has a picture on it.
pub(super) fn publish_coverage(
    ui: &Ui,
    doc: &OpenDoc,
    rect: egui::Rect,
    paint_rect: egui::Rect,
    backdrop: bool,
    textured: bool,
    is_current: bool,
) {
    if !is_current {
        return;
    }
    let want = rect.intersect(ui.clip_rect());
    let area = |r: egui::Rect| f64::from(r.width().max(0.0) * r.height().max(0.0));
    let fraction = |got: egui::Rect| {
        if area(want) > 0.0 {
            area(got) / area(want)
        } else {
            1.0
        }
    };
    // The REAL raster's reach, backdrop excluded, and it is the same
    // `paint_rect` `canvas::present` drew the texture at — not a recomputation
    // of where it should have gone. A second derivation is how an instrument
    // comes to report on a rectangle nothing was painted at.
    let sharp = if textured {
        fraction(paint_rect.intersect(want))
    } else {
        0.0
    };
    let covered = if backdrop { fraction(want) } else { sharp };
    crate::diag::trace_on_change("canvas-coverage", || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "covered={covered:.3} sharp={sharp:.3} textured={} backdrop={} zoom={:.3}",
            u8::from(textured),
            u8::from(backdrop),
            doc.view.zoom
        )
    });
    //
    // The disclosure in `app::status::disclosure::line_weights_disclosure`
    // reads the same field to choose which sentence the operator sees. This
    // line is the machine-readable twin, and it exists because the harness had
    // the same problem the operator does: `tools/ui-verify`'s `line_weights`
    // judged the mode by **counting dark pixels before and after**, which
    // cannot tell *the mode reached the renderer and had nothing to cap here*
    // from *the mode is not reaching the renderer at all*. Measured on
    // `a1-titleblock.pdf` at 394 %: the drawing lost **1.39 %** of its ink,
    // below the check's floor, and the check failed a working build.
    //
    // ⇒ **Never widen a tolerance when the measurement runs out — read a better
    // instrument.** This is the better instrument, and it is the engine's own
    // count rather than a second inference from pixels.
    //
    // `trace_on_change`, like the coverage line above it, so a still canvas
    // does not fill the trace. It is keyed on the whole formatted line, so a
    // count that goes 10 → 0 → 10 across a pan is three events and not one.
    crate::diag::trace_on_change("canvas-hairline", || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "thinned={} mode={}",
            doc.page_texture.as_ref().map_or_else(
                || "none".to_owned(),
                |t| t.diagnostics.strokes_hairlined.to_string()
            ),
            u8::from(!doc.view.line_weights),
        )
    });
}
