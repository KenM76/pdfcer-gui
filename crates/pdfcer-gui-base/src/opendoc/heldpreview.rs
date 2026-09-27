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
//! Design and rationale: `docs/modules/pdfcer-gui-base/opendoc/heldpreview.md`.

use super::OpenDoc;

pub use crate::shapepreview::HeldPreview;

/// How long a held preview may survive before it is dropped regardless.
const HELD_PREVIEW_MAX: std::time::Duration = std::time::Duration::from_secs(4);

/// How long a hold may sit with the edit epoch **unmoved** before it is dropped.
const HELD_PREVIEW_GRACE: std::time::Duration = std::time::Duration::from_millis(250);

impl OpenDoc {
    /// **The held preview, if it should still be on screen.**
    pub fn held_preview_to_draw(&self) -> Option<&crate::shapepreview::ShapePreview> {
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
    pub fn retire_held_preview(&mut self) {
        if self.held_preview.is_some() && self.held_preview_to_draw().is_none() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "canvas-held-preview-retired".to_owned()
            });
            self.held_preview = None;
        }
    }

    /// **Hold this preview until the page catches up.**
    pub fn hold_preview(&mut self, shape: crate::shapepreview::ShapePreview) {
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
    pub fn page_is_catching_up(&self) -> bool {
        self.page_texture_epoch != self.page_epochs.get(self.view.page_index)
            && self
                .last_edit_at
                .is_some_and(|at| at.elapsed() >= CATCHING_UP_AFTER)
    }
}
