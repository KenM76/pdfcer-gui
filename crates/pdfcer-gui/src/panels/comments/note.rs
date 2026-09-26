//! # `panels::comments::note` — the note being typed, and the stamp that keeps
//! it honest
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/comments/note.md`.

use pdfcer_core::object::ObjId;

/// **Where the words go when Save is pressed** — the one thing that
/// distinguishes writing a note from answering one.
///
/// # Why the destination lives on the draft rather than beside it
///
/// Because a reply *is* the note editor with a different destination, and
/// building it as a second draft, a second `TextEdit`, a second Escape
/// handler and a second stale-epoch rule would have been four places for the
/// two to drift apart. `crate::panels::comments::editor` therefore draws one
/// box and asks this enum which verb the commit raises, which means the
/// **stale-draft rule, the Escape route and the seeding rule are written
/// once** and cannot come to differ between the two.
///
/// And it is on the **stamp**, beside the annotation and the epoch, rather
/// than a fourth loose field. Those three answer one question together —
/// *what is open, on what, as of when* — and a destination that could be read
/// while nothing was open is a destination that eventually is.
///
/// # Why an enum and not `is_reply: bool`
///
/// The two reach different engine verbs with different outcomes:
/// `set_markup_note` edits a dictionary that exists, `add_reply` creates an
/// annotation. `crate::app::actions::annot::AnnotAction::Reply` makes the same
/// argument for keeping them apart on the action bus, and the argument is the
/// same one rung down — a bool is a value a future `match` can forget to
/// consider, and the outcome of forgetting here is a reviewer's answer written
/// over the comment it was answering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftTarget {
    /// The words become the annotation's own `/Contents` —
    /// `EditSession::set_markup_note`, raised as `AnnotAction::SetNote`.
    Note,
    /// The words become a **new annotation** answering this one — `/IRT` plus
    /// `/RT /R`, `EditSession::add_reply`, raised as `AnnotAction::Reply`.
    ///
    /// The stamped `ObjId` is then the **parent**, not the thing being
    /// edited, and nothing about the parent is modified. That reading of the
    /// same field is exactly why the target is stored beside it rather than
    /// inferred at the button.
    Reply,
}

/// The note being typed, and what it belongs to.
///
/// `Default` is *no editor open*, which is the state the panel is in on every
/// frame except the ones where the operator is writing.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NoteDraft {
    /// What the draft describes: the annotation, the **edit epoch** it was
    /// opened at, and **where the words are going**. `None` means no editor is
    /// open.
    ///
    /// See the module header for why the epoch is a member and not an
    /// optimisation, and [`DraftTarget`] for why the destination is in here
    /// rather than beside it.
    stamp: Option<(ObjId, u64, DraftTarget)>,
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
        self.stamp = Some((id, epoch, DraftTarget::Note));
        self.text = seed.to_owned();
    }

    /// **Open the editor to ANSWER one annotation**, empty.
    ///
    /// Empty, and that is the opposite of [`Self::begin`]'s rule rather
    /// than an oversight in it. Seeding a note editor is right because *edit*
    /// is commoner than *replace* and the operator is correcting words that
    /// already exist. Seeding a **reply** with the parent's words would put
    /// somebody else's sentence in the operator's mouth: `add_reply` writes a
    /// new annotation with the operator's own `/T`, so anything left in the
    /// box goes out signed by them.
    ///
    /// ⇒ Two entry points rather than one with a `seed` the caller passes
    /// `""` to, because that spelling puts the rule at every call site instead
    /// of in the type — and there would be exactly one call site until the day
    /// there were two.
    pub fn begin_reply(&mut self, parent: ObjId, epoch: u64) {
        self.stamp = Some((parent, epoch, DraftTarget::Reply));
        self.text.clear();
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

    /// Whether the editor is open on this annotation **at this epoch**, for
    /// either destination.
    ///
    /// The epoch is compared rather than ignored so that a caller cannot
    /// accidentally draw a stale editor between the edit landing and the next
    /// [`Self::sync`]; in practice `sync` runs first, and this is the belt to
    /// its braces.
    ///
    /// The **destination is deliberately not compared**, and that is what
    /// makes the row draw one editor rather than two: a row whose reply box is
    /// open must not also offer *Add note* beside it, because two boxes on one
    /// row is two drafts and this type holds one. Ask [`Self::target`] when the
    /// answer matters.
    #[must_use]
    pub fn editing(&self, id: ObjId, epoch: u64) -> bool {
        matches!(self.stamp, Some((sid, sepoch, _)) if sid == id && sepoch == epoch)
    }

    /// **Where the open editor's words are going.** `None` when none is open.
    ///
    /// Asked by the row that is drawing the editor, so the labels, the hint,
    /// the signature disclosure and the verb the commit raises all come from
    /// one answer — the failure mode being a box captioned *Post reply* whose
    /// Save writes `/Contents`, which is invisible until somebody reads the
    /// file.
    #[must_use]
    pub fn target(&self) -> Option<DraftTarget> {
        self.stamp.map(|(_, _, target)| target)
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
        if let Some((_, stamped, _)) = self.stamp
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
/// # Why a struct rather than two fields on `PanelsState`
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
    /// **How many WRITING controls the panel drew on its last frame** —
    /// Delete buttons plus note editors — so a headless test can assert that a
    /// reading stance offers none.
    ///
    /// # Why an instrument and not an assertion on a pure function
    ///
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
    /// **What the reviewer has narrowed the list to**, and how it is
    /// ordered. Added 2026-09-05; see [`super::filter`].
    ///
    /// Deliberately **not** stamped with the edit epoch, unlike [`NoteDraft`],
    /// and for [`Self::scrolled_to`]'s reason: a draft holds *words that would
    /// be written into the document*, so a document that moved under it is
    /// invalidated. This holds only *which rows the operator asked to see*,
    /// which an edit cannot make wrong — and resetting it on every keystroke
    /// in a note would throw away the narrowing that made the note findable.
    ///
    /// It survives the panel being closed and reopened, which is correct: a
    /// reviewer who filtered to their own comments, went to look at the page
    /// and came back is still doing the same job. What keeps that honest is
    /// that the filtered list **says so** on every frame — see
    /// [`crate::text::panels::comments::comments_filtered`] — so a filter
    /// nobody remembers setting can never be a filter nobody can see.
    pub filter: super::filter::Filter,
    /// **The annotation this panel last scrolled to**, so it scrolls once
    /// per selection *change* rather than once per frame.
    ///
    /// Without it, `scroll_to_me` on the selected row would run every frame and
    /// **pin the list under the operator's own scrollbar** — they could not look
    /// at any other row while a shape was selected on the canvas, which is a
    /// surface fighting its user. With it, the scroll is a response to a
    /// gesture, which is what an operator reads it as.
    ///
    /// Deliberately **not** stamped with the edit epoch, unlike
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

    /// The property the epoch exists for. An edit landing while the operator
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

    /// **The destination is remembered, and the two openings differ.**
    #[test]
    fn a_reply_draft_and_a_note_draft_remember_which_they_are() {
        let mut draft = NoteDraft::default();
        draft.begin(id(7), 3, "the note");
        assert_eq!(draft.target(), Some(DraftTarget::Note));

        draft.begin_reply(id(7), 3);
        assert_eq!(draft.target(), Some(DraftTarget::Reply));

        // …and a closed draft has no destination at all, which is what stops a
        // caller reading one while nothing is open.
        draft.close();
        assert_eq!(draft.target(), None);
    }

    /// **A reply opens EMPTY**, where a note edit opens seeded.
    #[test]
    fn a_reply_starts_blank_and_a_note_edit_does_not() {
        let mut draft = NoteDraft::default();
        draft.begin(id(7), 3, "Check this radius");
        assert_eq!(draft.text(), "Check this radius");

        draft.begin_reply(id(7), 3);
        assert!(
            draft.text().is_empty(),
            "a reply seeded with the parent's words would publish them under \
             the operator's own byline, got {:?}",
            draft.text()
        );
    }

    /// **One row draws one editor, whichever destination it has.**
    #[test]
    fn a_reply_editor_counts_as_the_rows_one_open_editor() {
        let mut draft = NoteDraft::default();
        draft.begin_reply(id(7), 3);
        assert!(draft.editing(id(7), 3));
        // …and it still belongs to one row at one epoch.
        assert!(!draft.editing(id(8), 3));
        assert!(!draft.editing(id(7), 4));
    }

    /// **An edit under the operator drops a REPLY draft too.**
    #[test]
    fn an_edit_under_the_operator_drops_a_reply_draft_as_well() {
        let mut draft = NoteDraft::default();
        draft.begin_reply(id(7), 3);
        draft.text_mut().push_str("half an answer");
        draft.sync(4);
        assert!(!draft.editing(id(7), 4));
        assert_eq!(draft.target(), None);
        assert!(draft.text().is_empty());
    }
}
