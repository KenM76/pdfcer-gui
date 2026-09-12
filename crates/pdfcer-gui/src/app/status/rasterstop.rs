//! # `app::status::rasterstop` — saying why zooming in stopped
//!
//! `OPERATOR_REQUESTS.md` **O186**, and the clause that makes the rest of that
//! row honest:
//!
//! > *"If this error is caused by some other limitation that will always happen,
//! > zoom should stop at the limit and not end up showing an error - the canvas
//! > will just stop zooming in and can still function. the error can still be
//! > shown on the bottom bar so the user has some idea as to why zooming stopped
//! > short of 1 trillion percent."*
//!
//! Three of those sentences are implemented elsewhere:
//!
//! | clause | where |
//! |---|---|
//! | a refusal becomes a **remembered** limit | [`crate::render::ceiling::RasterCeiling`] |
//! | the zoom is **pulled back** to it, once | `crate::render::settle`'s `learn_raster_ceiling` |
//! | the limit **binds every later gesture** | [`crate::viewer::zoom_ceiling`]'s fourth parameter |
//! | *"the error can still be shown on the bottom bar"* | **here** |
//!
//! ## ★★★ Why this module is not optional, and is not decoration
//!
//! Without it, the three clauses above produce a `+` button and a Ctrl+wheel
//! that stop responding with **nothing anywhere saying why**. That is a
//! silently-inert control, and it is the exact defect class this project was
//! founded to stop: the old shell's Delete key stopped working after a canvas
//! click and reported nothing, and O24's own text predicted the same shape in
//! advance — *"the buttons stop working exactly where the setting starts
//! mattering"*.
//!
//! A clamp the operator cannot account for is worse than the error sentence it
//! replaced, because an error is at least a report. `zoom_ceiling`'s header says
//! so in as many words: the silence of the learned clause *"is honest only
//! because this module discloses it"*.
//!
//! ## ★★★ It is a pure function of state — no store, no retirement rule
//!
//! The other sentences on the bar's left half each needed a store and a rule for
//! when to forget: [`super::disclosure`]'s two are keyed on `edit_epoch`,
//! [`super::decline`]'s is a thread-local retired at the dispatcher, and
//! [`super::page_box`]'s clamp note is retired by the operator's next act. Each
//! of those rules is a small machine that can be wrong.
//!
//! This one needs none of them, and that is not a shortcut — it falls out of
//! what the sentence *means*. It says **"the zoom is at this page's measured
//! ceiling right now"**, which is a question about the current frame and nothing
//! else. So:
//!
//! * **it retires itself when he zooms out**, because the predicate stops
//!   holding on the very next frame — no code has to remember anything;
//! * **it retires itself when he turns to another page**, because the ceiling is
//!   keyed on the page index and a page that has refused nothing has none;
//! * **it retires itself when he edits the page**, because the ceiling is keyed
//!   on [`crate::app::state::pageepoch::PageEpochs`] and an edit moves the epoch
//!   past it;
//! * **it cannot ever be stale**, which is the failure mode every store-backed
//!   sentence on this bar has had to argue its way out of. There is no state
//!   here that could be shown against the wrong document, because there is no
//!   state here at all.
//!
//! ★ The one thing that follows from this and is worth stating plainly: the
//! sentence **reappears** if he zooms back in to the ceiling, and that is
//! correct. It is not a notification of an event; it is a readout of a
//! condition, the same species as the zoom percentage beside it.
//!
//! ## ★★ Rule 4 — this is the off-canvas half, and there is no on-canvas half
//!
//! pdfcer decided something the operator did not ask for (it declined a zoom he
//! requested) and cannot see (nothing on the page looks different), so it owes a
//! report. The report is **one small line in the status bar** and that is the
//! whole of it: no badge, no tint, no marker, nothing drawn into the page view,
//! and nothing positioned relative to the document. The operator's own reason
//! for that rule is the nagging in the old shell, and a "you have hit the
//! zoom limit" overlay on the canvas would be precisely it.
//!
//! ## Why it is not folded into [`super::decline`]
//!
//! That module's header already rules on the neighbouring case — a region zoom
//! clamped by the *derived* `max_zoom_for_page` ceiling is a **partial grant**
//! rather than a decline, because the region is still framed and centred, and
//! because the framing verb raises `Action::ZoomTo` carrying the clamped number
//! so the readout states the truth unaided.
//!
//! ★★ That ruling is correct and does **not** cover this. Two differences, and
//! either alone is decisive:
//!
//! 1. **The readout cannot explain a *learned* ceiling.** In the partial-grant
//!    case the operator asked for a region and got it, just smaller; the number
//!    in the readout is the answer to his question. Here he asked to go *further
//!    in* and the number did not move at all. A readout that is identical before
//!    and after a gesture explains nothing about the gesture.
//! 2. **A decline is about one command; this is about a direction.** Every
//!    zoom-in gesture from now on will do nothing on this page, which is the
//!    same species of fact as [`super::filter`]'s empty-filter note — *why every
//!    gesture will do nothing* — rather than *why that one did*. That is also
//!    why it is drawn immediately after that note and before the decline.

use crate::app::state::OpenDoc;
use crate::text::status as t;

/// Named region: the raster-stop sentence, when the zoom is at a learned
/// ceiling.
///
/// Named for the same reason its siblings are, and for a reason sharper than
/// theirs: the whole of this module's obligation is that the sentence is **on
/// screen and legible**, and a driven check can only assert that about a rect
/// the application published. A `ui-verify` check that merely found the string
/// in a trace would be asserting that the shell *decided* to say it, which is
/// the thing already proved by the code above.
pub(super) const REGION: &str = "status-group:raster-stop"; // ui-text-exempt: trace region name, never displayed

/// How close to the ceiling counts as *at* it.
///
/// ★ A fraction rather than an absolute, because the ceiling spans four orders
/// of magnitude across page sizes — 284,964 on an E-size sheet against
/// 8,053,069 on a business card — so any fixed epsilon would be meaningless at
/// one end and a gate at the other.
///
/// ★★ Generous at one part in a thousand rather than tight at `f32::EPSILON`,
/// and that asymmetry is deliberate. The clamp lands on the ceiling by way of
/// [`crate::viewer::clamp_zoom`] and a division by `pixels_per_point`, so the
/// stored value and the live value are the same number arrived at by two routes
/// and need not be bit-identical. **The cost of the two errors is not
/// symmetric**: too tight and the sentence silently fails to appear in exactly
/// the state it exists for — a silence nobody reports, because the operator has
/// no way to know a sentence was owed — while too loose merely shows it a
/// thousandth of a percent early, which is true enough to be unnoticeable.
const NEAR: f32 = 1e-3;

/// Whether the view is sitting at this page's measured raster ceiling.
///
/// Split out from [`show`] so it can be unit-tested without a `Ui`, and so the
/// condition can be stated once: **a ceiling has been learned for this page at
/// this epoch, and the current zoom is at or above what it permits.**
///
/// # Why the conversion happens here and not in the store
///
/// [`crate::render::ceiling::RasterCeiling`] holds a **raster scale** — device
/// pixels per PDF point — and `doc.view.zoom` is a zoom. The two differ by
/// `pixels_per_point`, so every reader has to divide, and this is the second
/// such reader (the first is `zoom_ceiling`). That is the right arrangement and
/// its reasoning lives in the store: a ceiling kept as a *zoom* would be wrong
/// by the density ratio on a window dragged between two monitors, silently, and
/// only on the machine it was not measured on.
///
/// ⚠ `pixels_per_point` goes through [`crate::viewer::sane_pixels_per_point`]
/// rather than being trusted, and that shared guard exists because of this
/// function. A nonsense density makes `permitted` **infinite**, so this predicate
/// answers `false` forever — the disclosure switches itself off in precisely the
/// condition it exists for, and the operator is back to a control that stops
/// responding in silence.
///
/// ★ The first draft here wrote `pixels_per_point.max(f32::MIN_POSITIVE)`, which
/// reads like a guard and is not one: `f32::max` returns the *other* operand when
/// one is `NaN`, so a `NaN` density became `f32::MIN_POSITIVE` and the division
/// produced infinity anyway. It was caught by writing the `NaN` row of
/// `a_nonsense_display_density_does_not_silence_the_disclosure` below, not by
/// reading the code.
#[must_use]
pub(super) fn at_the_ceiling(doc: &OpenDoc, pixels_per_point: f32) -> bool {
    let page = doc.view.page_index;
    let Some(scale) = doc.raster_ceiling.for_page(page, doc.page_epochs.get(page)) else {
        return false;
    };
    if !scale.is_finite() || scale <= 0.0 {
        return false;
    }
    let permitted = scale / crate::viewer::sane_pixels_per_point(pixels_per_point);
    doc.view.zoom.is_finite() && doc.view.zoom >= permitted * (1.0 - NEAR)
}

/// Draw the sentence, if the view is at a learned ceiling.
///
/// Drawn through [`super::disclosure::disclosure_line`] rather than by hand,
/// which is what buys the three properties this surface requires and which a
/// hand-rolled `ui.label` would each have to re-earn: it is elided to the same
/// fraction of the remaining width as every other left-half sentence, it carries
/// its whole text on hover, and **it cannot make the bar taller** — R128, the
/// fit-zoom feedback loop, which a second row on this panel would re-open.
pub(super) fn show(ui: &mut egui::Ui, doc: &OpenDoc) {
    if !at_the_ceiling(doc, ui.ctx().pixels_per_point()) {
        return;
    }
    super::disclosure::disclosure_line(ui, REGION, t::raster_stop_status_line());
}

#[cfg(test)]
mod tests {
    use super::*;
    // ★ The real four-page fixture through the real opener, not a hand-built
    // `OpenDoc`: `open_fixture` makes the same three calls `PdfcerApp::open_path`
    // makes in the same order, which is what makes `page_epochs` and
    // `view.page_index` below mean what they mean in the running application.
    use crate::app::state::{FOUR_PAGES, OpenDoc, open_fixture};

    fn open_doc() -> OpenDoc {
        open_fixture(FOUR_PAGES)
    }

    /// The overwhelmingly common case: no page has ever refused a render, so the
    /// bar says nothing.
    ///
    /// ★ This is the test that makes the module safe rather than the one that
    /// makes it work. `RasterCeiling` is empty for every document the operator
    /// will ever open except the handful that are zoomed past a rasterizer wall,
    /// so a predicate that answered `true` by accident would put a permanent,
    /// false sentence on the bar of every file in the building.
    #[test]
    fn a_document_that_has_refused_nothing_says_nothing() {
        let doc = open_doc();
        assert!(!at_the_ceiling(&doc, 1.0));
    }

    /// ★★ **At the ceiling it speaks; a hair below it is silent.**
    ///
    /// The two halves have to be asserted together, because either alone is
    /// satisfied by a constant: a predicate hard-wired to `true` passes the first
    /// and a predicate hard-wired to `false` passes the second.
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

    /// ★★★ **It retires itself when he zooms out** — the whole reason this
    /// module needs no store and no retirement rule.
    ///
    /// Asserted rather than argued, because "it cannot go stale" is a claim about
    /// a mechanism that does not exist, and the only way to keep such a claim
    /// true is to measure the condition it rests on.
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

    /// ★★ **An edit to the page retires the sentence**, because the ceiling is
    /// keyed on the page epoch and an edit moves it.
    ///
    /// The same invalidation rule [`crate::render::ceiling::RasterCeiling`] owns,
    /// asserted from this side of the boundary: a reader that asked with a stale
    /// epoch — or with no epoch — would keep the sentence on a page whose content
    /// has been replaced, where the old measurement says nothing.
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

    /// ★★ **The stored value is a raster SCALE**, so the zoom that triggers the
    /// sentence halves when the display density doubles.
    ///
    /// The twin of `viewer::ceiling::tests::the_learned_ceiling_is_a_raster_scale_and_not_a_zoom`,
    /// on the disclosure side. Both are needed: if only one reader divided, the
    /// shell would clamp at one zoom and explain itself at another, and the
    /// operator on a high-DPI screen would get a sentence that was a factor of
    /// two away from the number beside it.
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

    /// ⚠★★ **A nonsense display density must not switch the disclosure off** —
    /// the test that found a real defect in this module's first draft.
    ///
    /// A bad density makes the permitted zoom infinite, so the predicate answers
    /// `false` forever: the sentence vanishes in exactly the state it exists for,
    /// and the operator is back to a control that stops responding in silence.
    /// That is the worst available failure mode and it is **invisible** — nobody
    /// reports a sentence they were never told was owed.
    ///
    /// ★★ Every row is asserted unconditionally, with no `||` anywhere. The draft
    /// of this test read `at_the_ceiling(&doc, bad) || bad.is_nan()`, which passes
    /// whatever the `NaN` case does — and the `NaN` case was broken, because
    /// `f32::max` returns the non-`NaN` operand and the then-current guard was a
    /// `.max()`. An `||` between a measurement and an excuse measures neither.
    ///
    /// ★★★ **Both directions, and the second half is not redundant.** A bad
    /// density must not silence the sentence *and* must not conjure it. They
    /// falsify different clauses of the guard:
    ///
    /// * the first loop fails under `.max(f32::MIN_POSITIVE)`, because the
    ///   permitted zoom becomes infinite and nothing ever reaches it;
    /// * the second fails under a guard that checks only `> 0.0` and forgets
    ///   `is_finite`, because an infinite density makes the permitted zoom **zero**
    ///   and the sentence then appears at every zoom on the page — which is worse
    ///   than its absence, since a line that is always on is a line that stops
    ///   being read.
    ///
    /// Neither loop alone can tell those two wrong guards from the right one.
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
