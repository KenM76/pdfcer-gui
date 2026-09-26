//! # `app::state::heldpreview` — the preview that outlives the gesture
//!
//! `OPERATOR_REQUESTS.md` **O63**, third piece. The module owns one question:
//! **for how long is a picture of the document still true?** A released gesture
//! leaves two correct things racing — an edit that has already landed, and a
//! raster that has not caught up — and everything here is the rule for deciding
//! which of them the operator should be looking at.
//!
//! **Ken, 2026-08-30:** *"the live preview should remain while the update to the
//! pdf structure runs in the background."* Without the hold, releasing a drag
//! drops the preview while the raster underneath still shows the object where it
//! started; the object appears to **snap back** and then jump, which reads as
//! the program having refused the edit and changed its mind.
//!
//! Holding the picture is honest rather than optimistic: the edit **has**
//! happened, so the held shape is the true state of the document, drawn by the
//! only path that can produce it in under a second. That is why every clause
//! below keys on evidence that the commit landed, and why the one clause that
//! cannot get that evidence immediately is bounded by a quarter of a second
//! rather than trusted.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/state/heldpreview.md`.

use super::OpenDoc;

/// A live shape preview kept on screen while the page raster catches up.
///
/// See [`OpenDoc::held_preview`] for why this exists at all.
pub(crate) struct HeldPreview {
    /// The geometry, exactly as the gesture last drew it.
    pub shape: crate::canvas::shapes::ShapePreview,
    /// `edit_epoch` at the moment the gesture released — **before** the commit.
    ///
    /// The liveness test compares against this rather than against the epoch
    /// the commit produced, because the commit has not happened yet when this is
    /// stored: actions are drained *after* the frame that raised them.
    pub captured_at_epoch: u64,
    /// When it was captured, for the backstop.
    pub since: std::time::Instant,
}

/// How long a held preview may survive before it is dropped regardless.
const HELD_PREVIEW_MAX: std::time::Duration = std::time::Duration::from_secs(4);

/// How long a hold may sit with the edit epoch **unmoved** before it is dropped.
const HELD_PREVIEW_GRACE: std::time::Duration = std::time::Duration::from_millis(250);

impl OpenDoc {
    /// **The held preview, if it should still be on screen.**
    ///
    /// Three conditions, in order, and each rejects a different way for a hold
    /// to be wrong:
    ///
    /// | test | what it rejects |
    /// |---|---|
    /// | the epoch moved | a gesture that was **refused** — nothing committed, so there is nothing to preview |
    /// | the raster has not caught up | a picture that is already correct — drawing over it would be strictly worse than the real thing |
    /// | it is younger than [`HELD_PREVIEW_MAX`] | a raster that will never arrive, leaving a preview nobody can clear |
    ///
    /// There is a fourth state that deliberately draws: the frame **between**
    /// the release and the commit. Actions are drained after the frame that
    /// raised them, so for exactly one frame `edit_epoch` still equals
    /// `captured_at_epoch`. Rejecting that frame would blink the preview off and
    /// on again, which is the flicker this feature exists to remove.
    pub(crate) fn held_preview_to_draw(&self) -> Option<&crate::canvas::shapes::ShapePreview> {
        let held = self.held_preview.as_ref()?;
        if held.since.elapsed() > HELD_PREVIEW_MAX {
            return None;
        }
        // The one-frame window, bounded by TIME rather than by the epoch:
        // accepting an unmoved epoch unconditionally would also accept a
        // refusal, which never moves it at all. `moving::drag` declines to hold
        // for the refusals it can see; this covers the ones only the apply phase
        // can — the engine saying no after the Action was raised. See
        // [`HELD_PREVIEW_GRACE`].
        if self.edit_epoch == held.captured_at_epoch {
            return (held.since.elapsed() < HELD_PREVIEW_GRACE).then_some(&held.shape);
        }
        // The raster carrying the edit has landed. The document's own picture is
        // correct now, and it is better than this one in every way.
        //
        // Compared against the PAGE's own epoch, never against `edit_epoch`:
        // the two are issued by independent counters and diverge permanently
        // (see [`Self::page_is_catching_up`]). Comparing the wrong pair makes
        // this early return stop firing for the rest of the session, leaving a
        // held preview drawn over a raster that had already caught up.
        if self.page_texture_epoch == self.page_epochs.get(self.view.page_index) {
            return None;
        }
        Some(&held.shape)
    }

    /// Drop a held preview that has stopped being live.
    ///
    /// Separate from [`Self::held_preview_to_draw`] because that one takes
    /// `&self` — it is called from the painter, which holds the document
    /// immutably. This is called once a frame from `canvas::interact`, which
    /// does not, and it exists so a dead hold does not sit in memory carrying
    /// thousands of segments until the next gesture replaces it.
    pub(crate) fn retire_held_preview(&mut self) {
        if self.held_preview.is_some() && self.held_preview_to_draw().is_none() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "canvas-held-preview-retired".to_owned()
            });
            self.held_preview = None;
        }
    }

    /// **Hold this preview until the page catches up.**
    ///
    /// Called on release, with the geometry the gesture last drew. Replaces any
    /// previous hold outright: a second edit supersedes the first, and two
    /// previews on screen would be two claims about one document.
    pub(crate) fn hold_preview(&mut self, shape: crate::canvas::shapes::ShapePreview) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "canvas-held-preview epoch={} segments={}",
                self.edit_epoch,
                shape.segment_count()
            )
        });
        self.held_preview = Some(HeldPreview {
            shape,
            captured_at_epoch: self.edit_epoch,
            since: std::time::Instant::now(),
        });
    }
}

/// How far behind the picture must be before the program says so.
const CATCHING_UP_AFTER: std::time::Duration = std::time::Duration::from_millis(400);

impl OpenDoc {
    /// **Is the picture on screen behind the document, noticeably?**
    ///
    /// # What this answers, and why it is not the same question as the hold
    ///
    /// [`Self::held_preview_to_draw`] asks *"should this particular geometry
    /// still be drawn?"* and only ever has an answer for a **canvas gesture on
    /// a path**. This asks *"is the page the operator is looking at out of
    /// date?"* and has an answer for **every edit in the program** — a colour,
    /// a Bold press, a delete, a redaction mark, a page rotation, an undo.
    ///
    /// ⇒ That is the whole reason it exists. `OPERATOR_REQUESTS.md` O63 is
    /// *"live preview for everything we do"*, and a drawn preview is only
    /// possible where the shell holds the geometry. Where it does not, the
    /// honest substitute is not a worse picture — it is **saying that the
    /// picture is not the answer yet**, which is the third of the three options
    /// the operator chose between and the one with no failure mode.
    ///
    /// Deliberately silent under [`CATCHING_UP_AFTER`]: see that constant.
    ///
    /// **Both sides of the comparison must come from the same counter.**
    /// `edit_epoch` is incremented by the action modules; `page_texture_epoch`
    /// holds a `PageEpochs` value written by `render::settle`. They are issued
    /// independently, so an `EditScope::Page(other)` edit advances `edit_epoch`
    /// without advancing this page's entry and the two pass each other for good
    /// — deleting a page guarantees it, because `actions::pages` calls
    /// `bump_all` and a `resize` in the same breath. Comparing across the two
    /// counters therefore does not flicker, it **sticks**: *"the picture is
    /// catching up"* stays on the status bar for the rest of the session over a
    /// picture that is perfectly correct.
    ///
    /// A unit test that sets both fields by hand cannot see this — equal or
    /// adjacent values hold under either model. Only a test that edits a
    /// *different* page exercises the divergence.
    pub(crate) fn page_is_catching_up(&self) -> bool {
        self.page_texture_epoch != self.page_epochs.get(self.view.page_index)
            && self
                .last_edit_at
                .is_some_and(|at| at.elapsed() >= CATCHING_UP_AFTER)
    }
}
