//! # `app::status::rasterstop` — saying why zooming in stopped
//!
//! `OPERATOR_REQUESTS.md` **O186**:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/status/rasterstop.md`.

use crate::app::state::OpenDoc;
use crate::text::status as t;

/// Named region: the raster-stop sentence, when the zoom is at a learned
/// ceiling.
///
/// The whole of this module's obligation is that the sentence is **on screen and
/// legible**, and a driven check can only assert that about a rect the
/// application published. A `ui-verify` check that merely found the string in a
/// trace would assert that the shell *decided* to say it — which the code below
/// already proves.
pub(super) const REGION: &str = "status-group:raster-stop"; // ui-text-exempt: trace region name, never displayed

/// How close to the ceiling counts as *at* it.
const NEAR: f32 = 1e-3;

/// Whether the view is sitting at this page's measured raster ceiling.
///
/// Split out from [`show`] so it can be unit-tested without a `Ui`, and so the
/// condition is stated once: **a ceiling has been learned for this page at this
/// epoch, and the current zoom is at or above what it permits.**
///
/// # Why the conversion happens here and not in the store
///
/// [`crate::render::ceiling::RasterCeiling`] holds a **raster scale** — device
/// pixels per PDF point — where `doc.view.zoom` is a zoom, so every reader has to
/// convert. That is the right arrangement: a ceiling kept as a *zoom* would be
/// wrong by the density ratio on a window dragged between two monitors,
/// silently, and only on the machine it was not measured on.
///
/// ⚠ The conversion is [`crate::viewer::zoom_for_raster_scale`] and not a
/// division by the display density, because the operator's render quality is in
/// the scale too. Getting that wrong moves this sentence away from the zoom the
/// clamp actually stops at, which is the one place it must agree — O218.
///
/// ⚠ The density goes through [`crate::viewer::sane_pixels_per_point`] rather
/// than being trusted, inside that helper. A nonsense density would otherwise
/// make `permitted` **infinite**, so this predicate would answer `false` forever
/// — the disclosure switching itself off in precisely the condition it exists
/// for, leaving the operator back at a control that stops responding in silence.
///
/// `pixels_per_point.max(f32::MIN_POSITIVE)` reads like that guard and is not
/// one: `f32::max` returns the *other* operand when one is `NaN`, so a `NaN`
/// density survives as `f32::MIN_POSITIVE` and the division produces infinity
/// anyway.
#[must_use]
pub(super) fn at_the_ceiling(doc: &OpenDoc, pixels_per_point: f32) -> bool {
    let page = doc.view.page_index;
    let Some(scale) = doc.raster_ceiling.for_page(page, doc.page_epochs.get(page)) else {
        return false;
    };
    if !scale.is_finite() || scale <= 0.0 {
        return false;
    }
    let permitted =
        crate::viewer::zoom_for_raster_scale(scale, pixels_per_point, doc.prefs.render_quality);
    doc.view.zoom.is_finite() && doc.view.zoom >= permitted * (1.0 - NEAR)
}

/// Draw the sentence, if the view is at a learned ceiling.
///
/// Drawn through [`super::disclosure::disclosure_line`] rather than by hand,
/// which buys the properties this surface requires and a hand-rolled `ui.label`
/// would each have to re-earn: elision to the same fraction of the remaining
/// width as every other left-half sentence, the whole text on hover, and **no
/// growth in the bar's height** — a second row on this panel re-opens R128, the
/// fit-zoom feedback loop.
pub(super) fn show(ui: &mut egui::Ui, doc: &OpenDoc) {
    if !at_the_ceiling(doc, ui.ctx().pixels_per_point()) {
        return;
    }
    super::disclosure::disclosure_line(ui, REGION, t::raster_stop_status_line());
}

#[cfg(test)]
mod tests {
    use super::*;
    // The real fixture through the real opener, not a hand-built `OpenDoc`:
    // `open_fixture` makes the same calls `PdfcerApp::open_path` makes in the same
    // order, which is what makes `page_epochs` and `view.page_index` below mean
    // what they mean in the running application.
    use crate::app::state::{FOUR_PAGES, OpenDoc, open_fixture};

    fn open_doc() -> OpenDoc {
        open_fixture(FOUR_PAGES)
    }

    /// The overwhelmingly common case: no page has ever refused a render, so the
    /// bar says nothing.
    #[test]
    fn a_document_that_has_refused_nothing_says_nothing() {
        let doc = open_doc();
        assert!(!at_the_ceiling(&doc, 1.0));
    }

    /// **At the ceiling it speaks; below it is silent.**
    #[test]
    fn the_sentence_appears_at_the_ceiling_and_not_below_it() {
        let mut doc = open_doc();
        let page = doc.view.page_index;
        // 40,000 refused, so 30,000 is learned (BACKOFF = 0.75).
        doc.raster_ceiling
            .learn(page, 40_000.0, doc.page_epochs.get(page));
        let learned = 30_000.0_f32;

        doc.view.zoom = learned;
        assert!(at_the_ceiling(&doc, 1.0), "at the ceiling it must speak");

        doc.view.zoom = learned * 0.5;
        assert!(
            !at_the_ceiling(&doc, 1.0),
            "half way there it must be silent"
        );
    }

    /// **It retires itself when he zooms out** — the whole reason this module
    /// needs no store and no retirement rule.
    #[test]
    fn zooming_out_retires_the_sentence_with_nothing_remembering_to() {
        let mut doc = open_doc();
        let page = doc.view.page_index;
        doc.raster_ceiling
            .learn(page, 40_000.0, doc.page_epochs.get(page));
        doc.view.zoom = 30_000.0;
        assert!(at_the_ceiling(&doc, 1.0));

        // One ladder step down is enough; nothing is cleared.
        doc.view.zoom = 20_000.0;
        assert!(!at_the_ceiling(&doc, 1.0));
    }

    /// **An edit to the page retires the sentence**, because the ceiling is keyed
    /// on the page epoch and an edit moves it.
    #[test]
    fn an_edit_to_the_page_retires_the_sentence() {
        let mut doc = open_doc();
        let page = doc.view.page_index;
        doc.raster_ceiling
            .learn(page, 40_000.0, doc.page_epochs.get(page));
        doc.view.zoom = 30_000.0;
        assert!(at_the_ceiling(&doc, 1.0));

        doc.page_epochs.bump(page);
        assert!(
            !at_the_ceiling(&doc, 1.0),
            "a measurement of the page before the edit says nothing about it after"
        );
    }

    /// **The stored value is a raster SCALE**, so the zoom that triggers the
    /// sentence halves when the display density doubles.
    #[test]
    fn the_density_conversion_is_applied_here_too() {
        let mut doc = open_doc();
        let page = doc.view.page_index;
        doc.raster_ceiling
            .learn(page, 40_000.0, doc.page_epochs.get(page));
        // Learned scale 30,000. At 2x density that permits a zoom of 15,000.
        doc.view.zoom = 15_000.0;
        assert!(
            at_the_ceiling(&doc, 2.0),
            "15,000x at 2x density is the ceiling"
        );
        assert!(
            !at_the_ceiling(&doc, 1.0),
            "the same zoom at 1x density is only half way there"
        );
    }

    /// ⚠ **A nonsense display density must not switch the disclosure off.**
    #[test]
    fn a_nonsense_display_density_does_not_silence_the_disclosure() {
        let mut doc = open_doc();
        let page = doc.view.page_index;
        doc.raster_ceiling
            .learn(page, 40_000.0, doc.page_epochs.get(page));
        let nonsense = [0.0_f32, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY];

        // Learned scale 30,000, which at the fallback density of 1.0 permits a
        // zoom of 30,000 — so every row below must behave like `ppp = 1.0`.
        doc.view.zoom = 30_000.0;
        for bad in nonsense {
            assert!(
                at_the_ceiling(&doc, bad),
                "a density of {bad} must not silence the sentence"
            );
        }

        // And a long way below the ceiling it must stay quiet, whatever the
        // density claimed to be.
        doc.view.zoom = 100.0;
        for bad in nonsense {
            assert!(
                !at_the_ceiling(&doc, bad),
                "a density of {bad} must not conjure the sentence at 100x"
            );
        }
    }
}
