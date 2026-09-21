//! # `app::status::rasterstop` — saying why zooming in stopped
//!
//! `OPERATOR_REQUESTS.md` **O186**:
//!
//! > *"If this error is caused by some other limitation that will always happen,
//! > zoom should stop at the limit and not end up showing an error - the canvas
//! > will just stop zooming in and can still function. the error can still be
//! > shown on the bottom bar so the user has some idea as to why zooming stopped
//! > short of 1 trillion percent."*
//!
//! The clauses of that ask are split across modules:
//!
//! | clause | where |
//! |---|---|
//! | a refusal becomes a **remembered** limit | [`crate::render::ceiling::RasterCeiling`] |
//! | the zoom is **pulled back** to it, once | `crate::render::settle::absorb`'s `learn_raster_ceiling` |
//! | the limit **binds every later gesture** | [`crate::viewer::zoom_ceiling`]'s fourth parameter |
//! | *"the error can still be shown on the bottom bar"* | **here** |
//!
//! ★ Without this module the others produce a `+` button and a Ctrl+wheel that
//! stop responding with nothing anywhere saying why — a silently-inert control,
//! which is the defect class this shell exists to refuse. A clamp the operator
//! cannot account for is worse than the error sentence it replaced, because an
//! error is at least a report.
//!
//! ## A pure function of state — no store, no retirement rule
//!
//! The other sentences on the bar's left half each need a store and a rule for
//! when to forget, and each such rule is a small machine that can be wrong. This
//! one needs neither, because of what the sentence *means*: **"the zoom is at
//! this page's measured ceiling right now"** is a question about the current
//! frame and nothing else. So it retires itself on a zoom out (the predicate
//! simply stops holding on the next frame), on a page turn (the ceiling is keyed
//! on the page index, and a page that has refused nothing has none), and on an
//! edit (the ceiling is keyed on
//! [`crate::app::state::pageepoch::PageEpochs`]). It cannot be stale against the
//! wrong document, because there is no state here to be stale.
//!
//! It therefore **reappears** if he zooms back in to the ceiling, and that is
//! correct: it is a readout of a condition, the same species as the zoom
//! percentage beside it, not a notification of an event.
//!
//! ## Rule 4 — the off-canvas half, and there is no on-canvas half
//!
//! pdfcer declined a zoom the operator asked for and nothing on the page looks
//! different, so it owes a report. The report is one small line in the status bar
//! and that is the whole of it: no badge, no tint, no marker, nothing drawn into
//! the page view, nothing positioned relative to the document.
//!
//! ## Why it is not folded into [`super::decline`]
//!
//! That module rules that a region zoom clamped by the *derived*
//! `max_zoom_for_page` ceiling is a **partial grant** rather than a decline,
//! because the region is still framed and centred and the framing verb raises
//! `Action::ZoomTo` carrying the clamped number, so the readout states the truth
//! unaided. Two differences put a *learned* ceiling outside that ruling, either
//! one decisive:
//!
//! 1. **The readout cannot explain a learned ceiling.** He asked to go further
//!    *in* and the number did not move at all, and a readout identical before and
//!    after a gesture explains nothing about the gesture.
//! 2. **A decline is about one command; this is about a direction.** Every
//!    zoom-in gesture from now on does nothing on this page — the same species of
//!    fact as [`super::filter`]'s empty-filter note, *why every gesture will do
//!    nothing* rather than *why that one did*, which is why it is drawn
//!    immediately after that note and before the decline.

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
///
/// A fraction rather than an absolute, because the ceiling varies by better than
/// an order of magnitude across page geometries — measured at scale 284,964 on
/// an E-size sheet against 8,053,069 on a business card — so any fixed epsilon
/// would be meaningless at one end and a gate at the other.
///
/// ★ Generous at one part in a thousand rather than tight at `f32::EPSILON`, and
/// the asymmetry is deliberate. The clamp lands on the ceiling by way of
/// [`crate::viewer::clamp_zoom`] and a division by `pixels_per_point`, so the
/// stored value and the live value are the same number arrived at by two routes
/// and need not be bit-identical. **The two errors do not cost the same**: too
/// tight and the sentence silently fails to appear in exactly the state it exists
/// for — a silence nobody reports, because the operator has no way to know a
/// sentence was owed — while too loose shows it a thousandth of a percent early,
/// which is true enough to be unnoticeable.
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
/// ★ `pixels_per_point.max(f32::MIN_POSITIVE)` reads like that guard and is not
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
    ///
    /// The test that makes the module safe rather than the one that makes it
    /// work. `RasterCeiling` is empty for every document except the handful
    /// zoomed past a rasterizer wall, so a predicate that answered `true` by
    /// accident would put a permanent, false sentence on every file's bar.
    #[test]
    fn a_document_that_has_refused_nothing_says_nothing() {
        let doc = open_doc();
        assert!(!at_the_ceiling(&doc, 1.0));
    }

    /// **At the ceiling it speaks; below it is silent.**
    ///
    /// Both halves are asserted together because either alone is satisfied by a
    /// constant: a predicate hard-wired to `true` passes the first and one
    /// hard-wired to `false` passes the second.
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

    /// ★ **It retires itself when he zooms out** — the whole reason this module
    /// needs no store and no retirement rule.
    ///
    /// Asserted rather than argued: "it cannot go stale" is a claim about a
    /// mechanism that does not exist, and the only way to keep such a claim true
    /// is to measure the condition it rests on.
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
    ///
    /// The invalidation rule [`crate::render::ceiling::RasterCeiling`] owns,
    /// asserted from this side of the boundary: a reader asking with a stale
    /// epoch — or with none — would keep the sentence on a page whose content has
    /// been replaced, where the old measurement says nothing.
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
    ///
    /// The disclosure-side twin of
    /// `viewer::ceiling::tests::the_learned_ceiling_is_a_raster_scale_and_not_a_zoom`.
    /// Both are needed: if only one reader divided, the shell would clamp at one
    /// zoom and explain itself at another, and on a high-DPI screen the sentence
    /// would be a factor of two away from the number beside it.
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
    ///
    /// A bad density makes the permitted zoom infinite, so the predicate answers
    /// `false` forever: the sentence vanishes in exactly the state it exists for,
    /// and the operator is back at a control that stops responding in silence.
    /// That failure mode is **invisible** — nobody reports a sentence they were
    /// never told was owed.
    ///
    /// Every row is asserted unconditionally, with no `||` anywhere: an `||`
    /// between a measurement and an excuse (`at_the_ceiling(&doc, bad) ||
    /// bad.is_nan()`) measures neither, and passes whatever the `NaN` case does.
    ///
    /// ★ **Both directions, and the second half is not redundant.** A bad density
    /// must not silence the sentence *and* must not conjure it; the loops falsify
    /// different clauses of the guard:
    ///
    /// * the first fails under `.max(f32::MIN_POSITIVE)`, because the permitted
    ///   zoom becomes infinite and nothing ever reaches it;
    /// * the second fails under a guard that checks only `> 0.0` and forgets
    ///   `is_finite`, because an infinite density makes the permitted zoom **zero**
    ///   and the sentence then appears at every zoom on the page — worse than its
    ///   absence, since a line that is always on stops being read.
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
