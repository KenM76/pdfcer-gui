//! # `panels::comments::note` — the note being typed, and the stamp that keeps
//! it honest
//!
//! One annotation's `/Contents` while the operator is editing it, and nothing
//! else. Split out of [`super`] rather than added to it because this is the
//! first piece of **inter-frame operator state** the Comments panel has ever
//! had, and that module's own header says in as many words that it had none:
//! *"it is a pure function of the document. Nothing in it is expanded, picked,
//! drafted or remembered."* That sentence is now false and is corrected there;
//! this file is what made it false, and it is worth its own file so the
//! argument for the correction is in one place.
//!
//! ## ★★★ Why a draft at all, rather than writing on every keystroke
//!
//! Because a keystroke is not an operator act. `EditSession::set_markup_note`
//! is **one undoable command**, so a live binding would raise one per letter
//! and `Ctrl+Z` would walk backwards through a sentence a character at a time
//! — the same argument `crate::panels::properties::geometry` makes for its
//! Apply button, and the same one `pdfcer-core` itself makes about looping a
//! singular verb.
//!
//! ⇒ So: type freely, press **Save note** once, get one undo entry.
//!
//! ## ★★★ The stamp is `(annotation, edit epoch)`, and the epoch is the member
//! that is easy to leave out
//!
//! The annotation half is obvious — a draft belongs to the row it was opened
//! on, and clicking a different row must not carry the words across. The epoch
//! half is the one that has already gone wrong once in this project, in
//! `GeometryDraft`, and the failure has the same shape here:
//!
//! > Open the editor on a highlight, type three sentences, press `Ctrl+Z`
//! > (undoing something unrelated), then press Save.
//!
//! Without the epoch that Save writes the operator's words onto whatever
//! annotation now holds that object id — and after an undo of an *add*, that
//! id may name nothing at all, or, worse, a different annotation entirely once
//! the writer reuses the number. With the epoch the draft is simply dropped
//! when the document moves under it, and the operator retypes, which is the
//! honest outcome: **the alternative is words landing somewhere nobody asked
//! for, silently.**
//!
//! ★ Dropped rather than *refused at Save time*, because a stale draft on
//! screen is a lie for however long it stays there — it shows text next to a
//! shape that no longer has that text — and the moment to stop lying is the
//! moment it goes stale, not the moment somebody presses a button.
//!
//! ## What this deliberately does NOT do
//!
//! - **It does not hold a second selection.** The draft names one annotation
//!   by `ObjId`; it does not decide what the canvas outlines, what the Format
//!   tab describes or what Delete acts on. `crate::panels::ObjectTreeUi::focus`
//!   carries the whole argument about why a panel growing its own selection is
//!   a defect waiting for two surfaces to disagree.
//! - **It does not know the author or the date.** Those are supplied at apply
//!   time from `crate::app::prefs` and `crate::app::clock`, because they are
//!   properties of the operator and the moment rather than of the draft. A
//!   draft that captured the clock when the editor opened would date a comment
//!   by when somebody started typing it.

use pdfcer_core::object::ObjId;

/// The note being typed, and what it belongs to.
///
/// `Default` is *no editor open*, which is the state the panel is in on every
/// frame except the ones where the operator is writing.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NoteDraft {
    /// What the draft describes: the annotation, and the **edit epoch** it was
    /// opened at. `None` means no editor is open.
    ///
    /// See the module header for why the epoch is a member and not an
    /// optimisation.
    stamp: Option<(ObjId, u64)>,
    /// The words, exactly as typed. A `String` rather than an `Option<String>`
    /// because [`Self::stamp`] already answers *is anything open*, and two
    /// fields that can each answer it is one of them going stale.
    text: String,
}

impl NoteDraft {
    /// **Open the editor on one annotation**, seeded with whatever note it
    /// already carries.
    ///
    /// Seeded rather than blank, because *edit* is the commoner act than
    /// *replace*: correcting a typo in a comment is the fourth of the four
    /// rows `pdfcer-core`'s own reply lists as what a review IS, and it would be
    /// retyping from scratch against an empty box.
    pub fn begin(&mut self, id: ObjId, epoch: u64, seed: &str) {
        self.stamp = Some((id, epoch));
        self.text = seed.to_owned();
    }

    /// Close the editor and abandon the words.
    ///
    /// Called from Cancel, from Escape, from a successful Save, and from
    /// [`Self::sync`] when the document moves. **One way to close**, so a
    /// future arm cannot leave the text behind while clearing the stamp.
    pub fn close(&mut self) {
        self.stamp = None;
        self.text.clear();
    }

    /// Whether the editor is open on this annotation **at this epoch**.
    ///
    /// The epoch is compared rather than ignored so that a caller cannot
    /// accidentally draw a stale editor between the edit landing and the next
    /// [`Self::sync`]; in practice `sync` runs first, and this is the belt to
    /// its braces.
    #[must_use]
    pub fn editing(&self, id: ObjId, epoch: u64) -> bool {
        self.stamp == Some((id, epoch))
    }

    /// The words, for a `TextEdit` to write into.
    pub fn text_mut(&mut self) -> &mut String {
        &mut self.text
    }

    /// The words, to send to the engine.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// **Drop the draft if the document has moved under it.**
    ///
    /// Called once per frame by the panel, before anything is drawn, with the
    /// document's current edit epoch. See the module header: a draft stamped at
    /// an older epoch describes a document that no longer exists, and the
    /// moment to stop showing it is the moment it goes stale.
    ///
    /// A no-op when nothing is open, which is almost every frame.
    pub fn sync(&mut self, epoch: u64) {
        if let Some((_, stamped)) = self.stamp
            && stamped != epoch
        {
            self.close();
        }
    }
}

/// **Everything the Comments panel remembers between frames.**
///
/// Two members, and the pairing is the point: both are *the operator's place in
/// this panel* rather than anything about the document.
///
/// # ★ Why a struct rather than two fields on `PanelsState`
///
/// Because `PanelsState` hands each panel **one** accessor, deliberately — a
/// panel reaches its own state and cannot reach another's. Two loose fields
/// would need two accessors and would let a future panel take one of them by
/// accident. `crate::panels::pages::PagesUi`, `redact::RedactUi` and
/// `bookmarks::BookmarksUi` are the same shape for the same reason.
#[derive(Debug, Default)]
pub struct CommentsUi {
    /// The note being typed. See [`NoteDraft`].
    pub draft: NoteDraft,
    /// ★★★ **How many WRITING controls the panel drew on its last frame** —
    /// Delete buttons plus note editors — so a headless test can assert that a
    /// reading stance offers none.
    ///
    /// # Why an instrument and not an assertion on a pure function
    ///
    /// On 2026-09-05 the Delete control and the note editor were both drawn,
    /// live and effective, in **Read** — the mode whose whole stated posture is
    /// *the document is not yours to alter*. Forty-six tests over this panel
    /// passed, the twenty-nine gates were green and the ribbon comparison
    /// exited 0. It was found by launching the release binary off screen and
    /// reading its trace.
    ///
    /// The lesson this project already had, applied one rung up: **a unit test
    /// that calls the verb cannot see the chain in front of it.** A test of a
    /// `should_offer_delete(caps)` predicate would have passed on the build
    /// that never asked it. So the observable is the *drawn* count, taken from
    /// the same pass that draws, and the test drives the real `body`.
    ///
    /// Reset at the top of every frame, so a stale count can never be read as a
    /// fresh one — the failure mode `NoteDraft`'s epoch stamp exists to prevent,
    /// in its cheapest form.
    pub writing_controls_drawn: u32,
    /// ★ **What the reviewer has narrowed the list to**, and how it is
    /// ordered. Added 2026-09-05; see [`super::filter`].
    ///
    /// Deliberately **not** stamped with the edit epoch, unlike [`NoteDraft`],
    /// and for [`Self::scrolled_to`]'s reason: a draft holds *words that would
    /// be written into the document*, so a document that moved under it is
    /// invalidated. This holds only *which rows the operator asked to see*,
    /// which an edit cannot make wrong — and resetting it on every keystroke
    /// in a note would throw away the narrowing that made the note findable.
    ///
    /// ★★ It survives the panel being closed and reopened, which is correct: a
    /// reviewer who filtered to their own comments, went to look at the page
    /// and came back is still doing the same job. What keeps that honest is
    /// that the filtered list **says so** on every frame — see
    /// [`crate::text::panels::comments::comments_filtered`] — so a filter
    /// nobody remembers setting can never be a filter nobody can see.
    pub filter: super::filter::Filter,
    /// ★★★ **The annotation this panel last scrolled to**, so it scrolls once
    /// per selection *change* rather than once per frame.
    ///
    /// Without it, `scroll_to_me` on the selected row would run every frame and
    /// **pin the list under the operator's own scrollbar** — they could not look
    /// at any other row while a shape was selected on the canvas, which is a
    /// surface fighting its user. With it, the scroll is a response to a
    /// gesture, which is what an operator reads it as.
    ///
    /// ★ Deliberately **not** stamped with the edit epoch, unlike
    /// [`NoteDraft`]'s key, and the difference is worth stating: a draft holds
    /// *words that would be written into the document*, so a document that moved
    /// under it invalidates it. This holds only *where the scrollbar is*, which
    /// no edit can make wrong.
    pub scrolled_to: Option<ObjId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The id used throughout. Generation 0 is the ordinary case for an
    /// annotation in a file pdfcer has not rewritten.
    fn id(num: u32) -> ObjId {
        ObjId { num, generation: 0 }
    }

    /// A fresh draft is closed, which is what makes `Default` the right way to
    /// express "no editor open".
    #[test]
    fn a_fresh_draft_is_not_editing_anything() {
        let draft = NoteDraft::default();
        assert!(!draft.editing(id(7), 0));
        assert!(draft.text().is_empty());
    }

    /// Seeding is the point of `begin` — an editor that opened empty would
    /// make correcting a typo into retyping the sentence.
    #[test]
    fn beginning_an_edit_seeds_the_existing_words() {
        let mut draft = NoteDraft::default();
        draft.begin(id(7), 3, "Check this radius");
        assert!(draft.editing(id(7), 3));
        assert_eq!(draft.text(), "Check this radius");
    }

    /// ★ The property the epoch exists for. An edit landing while the operator
    /// is typing drops the draft rather than leaving words pointed at an
    /// object id that may now name something else.
    #[test]
    fn an_edit_under_the_operator_drops_the_draft() {
        let mut draft = NoteDraft::default();
        draft.begin(id(7), 3, "half a sentence");
        draft.sync(4);
        assert!(!draft.editing(id(7), 4));
        assert!(!draft.editing(id(7), 3));
        assert!(draft.text().is_empty());
    }

    /// The same epoch is not a change, and a draft that dropped itself every
    /// frame would be an editor nobody could type into.
    #[test]
    fn the_same_epoch_leaves_the_draft_alone() {
        let mut draft = NoteDraft::default();
        draft.begin(id(7), 3, "still here");
        draft.sync(3);
        assert!(draft.editing(id(7), 3));
        assert_eq!(draft.text(), "still here");
    }

    /// A draft belongs to one row. Opening the editor on another annotation
    /// must not carry the first one's words across — the words would be
    /// somebody else's comment, and Save would write them.
    #[test]
    fn opening_another_row_replaces_the_draft_rather_than_appending_to_it() {
        let mut draft = NoteDraft::default();
        draft.begin(id(7), 3, "on seven");
        draft.begin(id(9), 3, "on nine");
        assert!(!draft.editing(id(7), 3));
        assert!(draft.editing(id(9), 3));
        assert_eq!(draft.text(), "on nine");
    }

    /// Closing clears both members. Asserted rather than assumed because the
    /// failure mode is invisible: a stamp cleared with the text left behind
    /// seeds the *next* editor with the previous row's words.
    #[test]
    fn closing_clears_the_words_as_well_as_the_stamp() {
        let mut draft = NoteDraft::default();
        draft.begin(id(7), 3, "abandoned");
        draft.close();
        assert!(!draft.editing(id(7), 3));
        assert!(draft.text().is_empty());
    }
}
