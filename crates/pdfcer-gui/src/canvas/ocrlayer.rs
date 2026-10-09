//! # `canvas::ocrlayer` — the invisible text a scan carries, drawn
//!
//! A scanned page is a picture. When it has been through OCR it is a picture
//! **plus** a layer of real text, positioned over the marks it was recognised
//! from and drawn in text rendering mode 3 — present, searchable, copyable,
//! and never visible. This module draws it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/ocrlayer.md`.

use egui::{Color32, CornerRadius, Painter, Rect};

use crate::app::state::OpenDoc;
use crate::canvas::strip::PageView;

pub use pdfcer_gui_base::ocrlayerpref::DEFAULT_COLOUR;

/// Where the overlay's colour lives while the program runs. Memory key.
const COLOUR_KEY: &str = "pdfcer.ocr-layer.colour"; // ui-text-exempt: a memory key, never displayed

/// The colour the overlay is drawn in, as the operator left it.
#[must_use]
pub fn colour(ctx: &egui::Context) -> [u8; 3] {
    ctx.data(|d| d.get_temp::<[u8; 3]>(egui::Id::new(COLOUR_KEY)))
        .unwrap_or(DEFAULT_COLOUR)
}

/// Mirror the persisted colour into the live one, once per frame.
pub fn sync(ctx: &egui::Context, rgb: [u8; 3]) {
    if colour(ctx) != rgb {
        ctx.data_mut(|d| d.insert_temp(egui::Id::new(COLOUR_KEY), rgb));
    }
}

/// [`colour`] as egui sees it.
#[must_use]
pub fn colour32(ctx: &egui::Context) -> Color32 {
    let [r, g, b] = colour(ctx);
    // NOT A THEME COLOUR: the operator's own, set through `OPERATOR_REQUESTS.md`
    // O229 and persisted. A theme must never move it — restyling the
    // application would change a colour chosen to contrast with a particular
    // scan, which the theme has no knowledge of.
    Color32::from_rgb(r, g, b)
}

/// **Is this run part of the OCR layer?**
#[must_use]
pub fn is_ocr_run(run: &pdfcer_core::text_extract::TextRun) -> bool {
    run.glyphs.iter().any(|glyph| glyph.invisible)
}

/// **How much paint, from a slider position** — the one answer, and the one
/// the status bar must quote.
#[must_use]
pub fn painted_fraction(raw: f32) -> f32 {
    if raw.is_finite() {
        raw.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// How opaque the paper veil over the raster is, out of 255.
///
/// Rises with the slider: at the right-hand stop the veil is opaque and the
/// scan is gone, which is the design's *blank field behind the text*.
#[must_use]
pub fn veil_alpha(strength: f32) -> u8 {
    // `round` rather than a truncating cast: 1.0 must reach 255 exactly, or
    // the right-hand stop leaves one part in 255 of the scan showing through
    // and the "only the text layer is visible" end of O226 is not reached.
    (painted_fraction(strength) * 255.0).round() as u8
}

/// How opaque the overlay text is, out of 255.
#[must_use]
pub fn text_alpha(strength: f32) -> u8 {
    veil_alpha(strength)
}

/// The colour a fully-drawn veil leaves behind.
fn paper() -> Color32 {
    super::shapes::paper()
}

/// **Fade the page raster**, by painting paper over it.
pub(super) fn draw_veil(painter: &Painter, pages: &[PageView], strength: f32) {
    let alpha = veil_alpha(strength);
    if alpha == 0 {
        return;
    }
    let veil = super::overlay::at_alpha(paper(), alpha);
    for view in pages {
        if view.raster.is_none() {
            continue;
        }
        painter.rect_filled(view.paint_rect, CornerRadius::ZERO, veil);
    }
}

/// **Draw the recognised text**, in the operator's colour, through
/// `canvas::ocrink`'s engine render.
pub(super) fn draw_text(painter: &Painter, doc: &OpenDoc, pages: &[PageView], strength: f32) {
    let alpha = text_alpha(strength);
    if alpha == 0 {
        return;
    }
    let Some(page_index) = doc.page_text().map(|text| text.page_index) else {
        return;
    };
    // The page the CACHE describes, found among the pages drawn — never
    // `pages[0]` and never the acting page's map. See the header.
    let Some(view) = pages.iter().find(|view| view.page == page_index) else {
        return;
    };
    let held = edited_rect(painter.ctx(), doc, page_index);
    crate::diag::trace_changed("ocr-layer-held", || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("ocr-layer-held runs={}", usize::from(held.is_some()))
    });
    super::ocrink::paint(painter, doc, view, held, colour(painter.ctx()), alpha);
}

/// The canvas rectangle of the invisible run an open text edit is rewriting on
/// `page`, which `textedit::shaped` draws instead. `None` while that preview
/// has fallen back to the editor box.
fn edited_rect(ctx: &egui::Context, doc: &OpenDoc, page: usize) -> Option<Rect> {
    let draft = super::textedit::read(ctx)?;
    let super::textedit::Anchor::Run { run, .. } = draft.anchor else {
        return None;
    };
    if draft.page != page || !super::textedit::shaped::read(ctx, &draft)?.invisible() {
        return None;
    }
    // The provenance-bearing extraction, because the editor names runs by its
    // indices.
    let text = doc.provenance_page_text(page)?;
    let bbox = text.runs.get(run)?.bbox?;
    super::geometry::pdf_rect_to_canvas(
        (bbox.llx, bbox.lly, bbox.urx, bbox.ury),
        doc.pages.get(page)?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Both stops, and the fact that they are the two ends.**
    #[test]
    fn the_slider_stops_are_no_paint_and_full_paint() {
        assert_eq!(veil_alpha(0.0), 0, "the left stop must not veil the scan");
        assert_eq!(veil_alpha(1.0), 255, "the right stop must blank the scan");
        assert_eq!(text_alpha(0.0), 0, "the left stop must draw no text");
        assert_eq!(text_alpha(1.0), 255, "the right stop must draw text fully");
    }

    /// Out-of-range and corrupt positions paint nothing rather than paint
    /// wrongly — the argument is on [`painted_fraction`].
    #[test]
    fn a_position_outside_the_range_is_clamped_and_a_corrupt_one_paints_nothing() {
        assert_eq!(veil_alpha(-1.0), 0);
        assert_eq!(veil_alpha(2.0), 255);
        assert_eq!(veil_alpha(f32::NAN), 0);
        assert_eq!(veil_alpha(f32::INFINITY), 0);
    }

    /// The midpoint is genuinely between the two, not at one of them.
    ///
    /// A cast that truncated, or a scale that used 256, would still pass the
    /// stops test above; this is what tells them apart.
    #[test]
    fn the_middle_of_the_slider_is_half_of_each() {
        let mid = veil_alpha(0.5);
        assert!(
            (120..=135).contains(&mid),
            "half the slider should be about half the paint, got {mid}"
        );
    }
}
