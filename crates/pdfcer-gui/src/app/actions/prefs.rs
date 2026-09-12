//! ★★★ **The verbs whose subject is a PREFERENCE, not a document** — split out
//! of [`super::action`] and [`super::apply`] under **R2** on 2026-09-12, the day
//! **O187** made the family a family by giving it a second member.
//!
//! # What makes this a family rather than a size-driven cut
//!
//! Four properties, and every member has all four. A verb that has three of
//! them belongs somewhere else.
//!
//! 1. **It needs no open document.** Every other arm in [`super::apply`] acts
//!    on `Status::Open(doc)`, and the guard's *"no document: silently drop"* is
//!    the right answer for all of them. It is the wrong answer here, and
//!    wrong in a way that is almost impossible to report: the Find bar is
//!    reachable with nothing open, so a preference set there would stick most
//!    of the time and vanish the rest. A defect that works most of the time is
//!    the worst kind there is.
//!
//! 2. **It changes nothing an undo could reach.** No `EditSession`, no epoch
//!    bump, no raster invalidation, no [`super::funnel::vector_edit`]. The
//!    four-step protocol that every document change goes through has nothing
//!    to do here, which is why these arms `return` rather than falling into
//!    it.
//!
//! 3. **The live half has already been applied by the surface that raised
//!    it.** This is the property a reader most often gets backwards, so it is
//!    stated once here rather than twice in two variant docs. The *live* value
//!    and the *persisted* value have different owners:
//!
//!    | Preference | Live owner | Why the live half cannot wait a frame |
//!    |---|---|---|
//!    | Find's *Zoom* tick | [`crate::find::FindState`] | the very next *Next* would obey the old answer |
//!    | Pages previews + limit | [`crate::panels::pages::thumbnails::ThumbnailCache`] | the very next render would obey the old answer |
//!
//!    What neither surface can reach is `PdfcerApp::prefs` and the file behind
//!    it. **That is the only thing these actions are for.** They are a
//!    write-through, not an apply.
//!
//! 4. **It writes the file immediately**, and the failure is swallowed. The
//!    rule `app::actions::view`'s `smart_select` states and this module
//!    inherits: *one discrete operator decision is one write, now*. Losing a
//!    preference across a restart does not justify a modal in front of
//!    somebody who is in the middle of searching.
//!
//! # ⚠ Why the operand is carried and never re-read
//!
//! The arm runs **after** the frame that raised it. The widget that was ticked
//! may not exist any more — a panel can close, a dock tab can change, the Find
//! bar can be dismissed — so an arm that went looking for the control to ask
//! what it was set to would be reading a surface that has already gone. Every
//! action in this crate carries a complete statement of intent for that
//! reason; here it is not a style rule but the difference between a preference
//! that sticks and one that sticks when the panel happens to still be open.
//!
//! ★ [`PrefAction::PagePreviews`] takes this one step further and carries
//! **both** of its two values even when only one changed, because
//! `Prefs::save` is a whole-file write. See its own doc.
//!
//! # Where the other direction lives
//!
//! Nowhere near here, deliberately. The read-back happens **once, at
//! construction**, in `crate::app::PdfcerApp::new`, which seeds the live
//! owners from the file. A preference that were re-read per frame would let
//! the file win an argument the operator had already had with the control.

use crate::app::prefs::Prefs;

/// One operator preference, carried from the surface that changed it to the
/// file that remembers it.
///
/// See the module header for the four properties every member shares and for
/// why the *live* half is already applied by the time one of these is raised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefAction {
    /// ★ **Persist the Find bar's *Zoom* control** — `OPERATOR_REQUESTS.md`
    /// **O163**, 2026-09-09.
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
    /// ★★★ **Persist the Pages panel's previews tick and its time limit** —
    /// `OPERATOR_REQUESTS.md` **O187**, 2026-09-12: *"the draw page previews
    /// timeout needs to be remembered, and setting it to 0 should set it to
    /// infinity (never time out)"*.
    ///
    /// # ★ Why one variant carries both, when they are two controls
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
    /// # ⚠ It would survive inside the document guard today, and is above it
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
                    // ★ `budget_ms` PLAIN, and `0` reaching the trace as `0`
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
