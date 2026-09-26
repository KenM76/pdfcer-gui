//! **The verbs whose subject is a PREFERENCE, not a document.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/prefs.md`.

use crate::app::prefs::Prefs;

/// One operator preference, carried from the surface that changed it to the
/// file that remembers it.
///
/// See the module header for the four properties every member shares and for
/// why the *live* half is already applied by the time one of these is raised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefAction {
    /// **Persist the Find bar's *Zoom* control** — `OPERATOR_REQUESTS.md`
    /// **O163**.
    ///
    /// # Why this is not a [`crate::find::FindRequest`]
    ///
    /// `FindRequest`'s own doc states the rule its two variants share: *what
    /// has to go through the funnel is what needs the **document***, and both
    /// of them do. This needs the opposite — property 1 in the module header.
    /// `Action::Find`'s arm is inside a `Status::Open(doc)` match that would
    /// silently drop it, which is precisely the failure that property
    /// describes.
    FindZoom(bool),
    /// **Persist the Pages panel's previews tick and its time limit** —
    /// `OPERATOR_REQUESTS.md` **O187**: *"the draw page previews
    /// timeout needs to be remembered, and setting it to 0 should set it to
    /// infinity (never time out)"*.
    ///
    /// # Why one variant carries both, when they are two controls
    ///
    /// Because they are one **decision surface** — a checkbox and the box
    /// beside it — and `Prefs::save` is a whole-file write. Two variants would
    /// mean two file writes for the gesture *"turn previews off and set a
    /// limit for when I turn them back on"*, which is the sequence
    /// `crate::panels::pages::previews` explicitly designs for. Each raiser
    /// reads the value it did not change straight out of the cache in the same
    /// frame, so the module header's carry-never-re-read rule still holds for
    /// both halves.
    ///
    /// # It would survive inside the document guard today, and is above it
    /// anyway
    ///
    /// The Pages panel is only drawn with a document open, so unlike
    /// [`Self::FindZoom`] this one has no live counter-example. It is a
    /// preference regardless, because *which surface happens to raise a
    /// preference* is not a property of the preference — and a rule that held
    /// only while the panel needed a document is a rule waiting to be broken
    /// by a Settings entry for the same two values.
    PagePreviews {
        /// The tick, as it now stands.
        on: bool,
        /// The limit in milliseconds, as it now stands — **`0` meaning no
        /// limit**, the operator's own notation. Converted at exactly one
        /// place, [`crate::panels::pages::thumbnails::budget_from_millis`].
        budget_ms: u64,
    },
}

impl PrefAction {
    /// Write this preference through to `prefs` and save the file.
    ///
    /// Takes `&mut Prefs` rather than `&mut PdfcerApp` on purpose: the whole
    /// point of the family is that none of it can touch a document, and a
    /// signature that could would make that a convention rather than a fact.
    /// A future member that genuinely needed more than `Prefs` would be
    /// telling you it is not a member.
    ///
    /// The save failure is swallowed — property 4 in the module header.
    pub(super) fn apply(self, prefs: &mut Prefs) {
        match self {
            Self::FindZoom(on) => {
                prefs.find_zoom_on_jump = on;
                let _ = prefs.save();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("find-zoom-persisted on={on}")
                });
            }
            Self::PagePreviews { on, budget_ms } => {
                prefs.page_previews = on;
                prefs.page_preview_budget_ms = budget_ms;
                let _ = prefs.save();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    //
                    // `budget_ms` PLAIN, and `0` reaching the trace as `0`
                    // rather than as a word: a harness reads this field, and a
                    // Debug-formatted or prettified value in a field a machine
                    // parses has already produced one driven check in this repo
                    // that reported the opposite of the truth.
                    format!("page-previews-persisted on={on} budget_ms={budget_ms}")
                });
            }
        }
    }
}
