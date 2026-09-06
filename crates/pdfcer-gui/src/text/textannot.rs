//! # `text::textannot` — every word the text-annotation dialog shows
//!
//! Copy for the three markup kinds that carry words: text box, sticky note and
//! stamp.
//!
//! ## ★ The one distinction every string here has to preserve
//!
//! **A text box is painted on the page. A sticky note is not.**
//!
//! That is the difference between a callout somebody reading a printed drawing
//! will see and a note only a PDF reader shows, and an operator who gets it
//! backwards has either published a private remark or hidden a public one.
//! Neither is recoverable by noticing later — the file is what it is.
//!
//! So the two kinds do not share a sentence anywhere below, even where the
//! control is identical. A shared string would be one an author could reword
//! for one kind and silently change for the other.
//!
//! ## ★ It is no longer only the dialog's copy, and that is deliberate
//!
//! Everything above `cancel` is the authoring dialog. Everything below it is
//! about **editing a note on a text box that is already placed** — a surface
//! this module did not originally serve.
//!
//! They live together because they are the same claim. *Painted on the page*
//! is what the dialog promises when the box is placed, and it is exactly why
//! the words cannot be corrected afterwards; splitting the two apart would put
//! the promise in one file and its consequence in another, where an author
//! could soften either without touching the other. The section banner below
//! carries the measurement.

use crate::canvas::textannot::TextAnnotKind;
use pdfcer_core::annot_author::StampName;

/// The window's title.
#[must_use]
pub const fn title(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => "Text box",
        TextAnnotKind::Sticky => "Sticky note",
        TextAnnotKind::Stamp => "Stamp",
    }
}

/// The sentence under the title.
///
/// ★ Each says **where the words end up**, which is the distinction this
/// module exists to preserve. The text box's says "on the page"; the sticky's
/// says the opposite in as many words, because an operator who believes a
/// sticky prints will use it for something that needed to.
#[must_use]
pub const fn intro(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => {
            "This is written onto the page, inside the box you drew, and it \
             prints. It wraps to fit the box."
        }
        TextAnnotKind::Sticky => {
            "This is a note attached to the page, not written on it. A marker \
             shows where it is and the words open when someone clicks it — so \
             it does NOT print."
        }
        TextAnnotKind::Stamp => {
            "A standard stamp, drawn into the box you dragged. It is written \
             onto the page and it prints."
        }
    }
}

/// The text field's placeholder.
#[must_use]
pub const fn hint(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => "What should it say?",
        TextAnnotKind::Sticky => "What is the note?",
        // Unreachable — a stamp draws the gallery instead of a field — and
        // answered rather than left to a `todo!()`, because a panic in a
        // dialog is a worse outcome than a placeholder nobody sees.
        TextAnnotKind::Stamp => "",
    }
}

/// What the operator should know about the field, under it.
///
/// Both name the engine limits stated when the burn-in landed: the face is
/// Base-14 Latin, so anything outside it becomes a question mark. The text
/// box's also names wrapping, because that is what makes the box's width a
/// choice rather than a formality.
#[must_use]
pub const fn bound(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => {
            "Wraps to the box, in a standard Latin font — other alphabets come \
             out as question marks. Drag a wider box if it does not fit."
        }
        TextAnnotKind::Sticky => {
            "Shown in a standard Latin font when the note is opened; other \
             alphabets come out as question marks."
        }
        TextAnnotKind::Stamp => "",
    }
}

/// One stamp's name, as an operator reads it.
///
/// ★ Title case, not the `/Name` spelling. The PDF carries `/NotApproved`;
/// nobody says that out loud, and a gallery that listed it would be showing
/// the operator the file format rather than the choice. The engine's own
/// appearance paints its own label — this is only how the option is *offered*.
///
/// The catch-all is required because `StampName` is `#[non_exhaustive]`, and
/// it returns the empty string rather than a guess: a stamp this catalog has
/// no prose for is one `crate::canvas::textannot::STAMPS` does not list, so
/// the arm is unreachable from the gallery and inventing a label would be
/// prose pdfcer made up about somebody else's addition.
#[must_use]
pub const fn stamp_label(stamp: StampName) -> &'static str {
    match stamp {
        StampName::Approved => "Approved",
        StampName::NotApproved => "Not approved",
        StampName::Draft => "Draft",
        StampName::Final => "Final",
        StampName::ForComment => "For comment",
        StampName::AsIs => "As is",
        StampName::Expired => "Expired",
        _ => "",
    }
}

/// What the gallery does not offer, said once under it.
///
/// ★ A disclosure rather than a limitation apologised for. Acrobat's *dynamic*
/// stamps bake a name and a timestamp into the appearance; pdfcer has no
/// identity to put in one, and the note-text exchange settled that this shell
/// invents no placeholder for `/T`. So a stamp claiming to be signed by
/// somebody would be a claim pdfcer cannot support — and saying so is cheaper
/// than an operator looking for the feature.
#[must_use]
pub const fn stamp_bound() -> &'static str {
    "These are the standard stamps. pdfcer does not add a name or a date to \
     them, because it does not know who you are."
}

/// The commit control.
#[must_use]
pub const fn accept() -> &'static str {
    "Add"
}

/// Why Add is greyed.
///
/// Names the kind, because "type something first" is unhelpful next to a
/// gallery and impossible next to a stamp — the message only ever appears for
/// the two kinds that take typing, and it says which one it is about.
#[must_use]
pub const fn accept_disabled(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => "Type what the box should say first.",
        TextAnnotKind::Sticky => "Type the note first.",
        // Unreachable: a stamp is always ready. Answered rather than panicking,
        // as `hint` is.
        TextAnnotKind::Stamp => "",
    }
}

/// The abandon control.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

// ===========================================================================
// EDITING a note that already exists — the other end of the same distinction
// ===========================================================================
//
// ★★★ **A `/FreeText`'s `/Contents` and the words painted in it are kept in
// step BY THE ENGINE now, in the same command and the same undo entry. The
// disclosure this section carried for one morning is deleted; the shape of the
// correction is kept, because the shape is the useful part.**
//
// # What was measured here on 2026-09-06, and it was correct when written
//
// At `pdfcer-core` v0.42.0 (`821ab47`, the pin this shell then took):
//
// 1. `annot_author::free_text` writes the operator's words TWICE — into the
//    `/AP` `/N` appearance stream that is what the page actually shows, and
//    into `/Contents`. At the moment a text box is placed the two are the same
//    string, so the divergence had no visible first moment.
// 2. `EditSession::set_markup_note` committed **one object** — the annotation
//    dictionary. The `/AP` stream is a separate object and was not in the
//    command.
// 3. R43, in the engine's own words: *"pdfcer paints from `/AP` or not at
//    all."*
//
// ⇒ Editing a placed text box's note rewrote `/Contents` and left the page
// showing the words the box was made with. Not a bug in the call — the call
// did exactly what it documented — but an operator typing into a box and
// watching the page not change had been told nothing.
//
// It could not be closed from this side. Regenerating an appearance needs the
// annotation read back into a `TextAnnotSpec`, and there was no reader for the
// text-bearing family anywhere in the crate. So this shell did the only two
// things open to it: it **disclosed**, off-canvas and twice — a line in the
// note editor before the write, and the first sentence of the status line
// after it — and it **filed**
// `request_a_text_boxs_painted_words_cannot_be_rewritten.md`, naming
// `annot_author::text_spec_from_dict` as the single unblocker.
//
// # ★★ What replaced it, the same afternoon
//
// `pdfcer-core` `95a936e` (`Pass 258.1`) shipped that reader and did not stop
// there: **`set_markup_note` re-bakes the appearance itself**, opt-out-free,
// inside the same command, so the two halves share one undo entry and can
// never be undone apart. Read, not quoted:
//
// * `edit.rs:26137` — `pub fn set_markup_note`.
// * `edit.rs:26272-26289` — the `/FreeText` arm; on a hit the command carries
//   **two** `ObjectWrite`s, the dictionary and the `/AP` stream, and
//   `appearance_rebaked` is `rebake.is_some()`.
// * `edit.rs:27000` — `rebake_free_text_appearance`, which reads the ORIGINAL
//   dictionary into a spec, re-bakes and commits.
//
// So the before-the-write hint is **deleted**. There is nothing left for it to
// warn about, and it could not warn about what remains: whether an appearance
// is foreign is measured *inside* the verb, by baking and comparing bytes, so
// no editor drawn before the call can know it. The two status-line sentences
// survive, narrowed to the one case that is still real, and they key on the
// engine's own answer instead of on a subtype this shell classified.
//
// ★ This deletion is the standing rule, not a tidy-up: *delete the workaround
// when the cause is removed.* A mechanism with no caller rots, and a
// limitation sentence has an hours-long shelf life. A shell still warning an
// operator about a defect the engine closed the same afternoon is lying to
// them, and it is the kind of lie nobody notices because it reads as caution.
//
// # ★★★ The half that survives, and why the subtype test is no longer enough
//
// [`pdfcer_core::edit::MarkupNoteChange::appearance_rebaked`] is `false` on
// three quite different occasions and **only one of them owes an operator a
// sentence**:
//
// | `/Subtype` | `appearance_rebaked` | what it means | what is said |
// |---|---|---|---|
// | `Text` (sticky) | `false` | a sticky paints no words — a reader's popup shows them — so `/Contents` is the whole of the content | nothing |
// | `Stamp` | `false` | `/Contents` is a comment *about* the stamp; `annot_author::stamp` never writes the key at all | nothing |
// | `FreeText`, appearance pdfcer's own | **`true`** | the words on the page moved with the note, in one undo entry | nothing — the page itself shows it |
// | `FreeText`, appearance FOREIGN | `false` | a designer's box with a shadow, a gradient or an image in it keeps it, rather than being replaced by pdfcer's plainer rendering. **The note still commits** | **this row, and only this row** |
//
// ⇒ The guard is therefore `paints_its_note(subtype) && !appearance_rebaked`,
// and both halves are load-bearing in opposite directions:
//
// - Drop the **subtype** half and the sentence fires on every sticky note in
//   the document — the failure the engine names itself, *a disclosure that
//   fires on the overwhelmingly common path is one an operator learns to
//   skip*, and skipping it costs the case that is real.
// - Drop the **`appearance_rebaked`** half and the deleted lie is back: every
//   text-box note edit told the page did not move, on a build where it did.

/// Whether this `/Subtype` PAINTS its `/Contents` onto the page.
///
/// The first half of the surviving disclosure's guard — the second is the
/// engine's `appearance_rebaked` — so the question is asked in one place
/// rather than two. `subtype` is `pdfcer-core`'s own `Annotation::subtype_label`
/// output, reaching this shell as
/// [`pdfcer_core::edit::MarkupNoteChange::subtype`]: the raw `/Subtype` name,
/// not a label of ours.
///
/// # ★ Why the answer is `FreeText` and NOT the other two note-bearing kinds
///
/// Measured per kind rather than assumed for the family, and the family turns
/// out not to be uniform:
///
/// | `/Subtype` | `/Contents` painted? | re-baked by `set_markup_note`? |
/// |---|---|---|
/// | `FreeText` | **yes** — it is the appearance's own input | yes, when the appearance on disk is one pdfcer would have drawn |
/// | `Text` (sticky) | no — *"shown by the reader's popup, never painted on the page"* | no, and nothing is stale |
/// | `Stamp` | no — `stamp()` writes `/Name` and never writes `/Contents` at all; the painted label comes from the name | no, and nothing is stale |
///
/// ⇒ Editing a sticky note's or a stamp's note is **complete** and owes no
/// disclosure, whatever `appearance_rebaked` says — for those two `false` is
/// the correct and final answer, not a failure. Saying otherwise would be a
/// warning an operator learns to ignore, which costs the one case that is real.
///
/// ★ The stamp row is the one worth reading twice. A stamp's `/Contents` is a
/// comment *about* the stamp, not the stamp's words — so it is not stale, it
/// was never the same thing. (Its painted label is separately unreachable: it
/// is baked into the `/AP` and stored under no key, so nothing can read it
/// back. That is a different gap and it is filed separately.)
#[must_use]
pub fn paints_its_note(subtype: &str) -> bool {
    subtype == "FreeText"
}

/// The status-line disclosure **after** a note is written, or `None` when the
/// edit was complete and nothing is owed.
///
/// `appearance_rebaked` is [`pdfcer_core::edit::MarkupNoteChange::appearance_rebaked`]
/// passed through unexamined — the engine's answer to *which half moved*, not
/// an inference of ours. See the section banner for the four-row table this
/// implements and for why `false` alone is not the condition.
///
/// ★ States the two halves separately and in that order — what did change,
/// then what did not — because an operator who reads only the first clause
/// must not come away believing the page moved. It also says **why** the
/// picture was left alone, since "it was not redrawn" without "because it is
/// not ours to redraw" reads as a failure rather than as preservation.
#[must_use]
pub fn note_edit_disclosure(subtype: &str, appearance_rebaked: bool) -> Option<&'static str> {
    (paints_its_note(subtype) && !appearance_rebaked).then_some(
        "The comment on this text box was changed. The words printed on the \
         page were NOT redrawn: this box was drawn by another program, so \
         pdfcer keeps its appearance exactly as it is rather than replacing it \
         with a plainer version, and the page still reads what it did before.",
    )
}

/// The status-line disclosure after a note is **removed**, or `None`.
///
/// A separate string from [`note_edit_disclosure`] rather than a shared one,
/// for this module's standing reason: the two describe different acts, and a
/// shared sentence is one an author can reword for one and silently change for
/// the other. Removing is also the worse surprise — the comment is gone from
/// every list and the page is unchanged, so the operator has deleted the only
/// copy they could see and kept the one they cannot.
#[must_use]
pub fn note_clear_disclosure(subtype: &str, appearance_rebaked: bool) -> Option<&'static str> {
    (paints_its_note(subtype) && !appearance_rebaked).then_some(
        "The comment on this text box was removed. The words printed on the \
         page were NOT redrawn: this box was drawn by another program, so \
         pdfcer keeps its appearance exactly as it is rather than emptying it, \
         and the page still reads what it did before.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **The two kinds' intros disagree about printing, out loud.**
    ///
    /// The one property this module exists to hold. A text box prints and a
    /// sticky does not, and an operator who has them backwards has either
    /// published a private remark or hidden a public one — neither recoverable
    /// by noticing afterwards.
    ///
    /// Asserted on the words rather than trusted to review, because the two
    /// controls are otherwise identical and a copy-paste between them would be
    /// invisible in a diff.
    #[test]
    fn the_printing_distinction_is_stated_in_both_directions() {
        let boxed = intro(TextAnnotKind::TextBox);
        let sticky = intro(TextAnnotKind::Sticky);
        assert!(
            boxed.contains("prints"),
            "the text box does not say it prints: {boxed:?}"
        );
        assert!(
            sticky.contains("does NOT print"),
            "the sticky does not say it stays off the page: {sticky:?}"
        );
        assert_ne!(boxed, sticky, "the two kinds share a sentence");
    }

    /// Every kind that takes typing has a hint, a bound and a greyed reason.
    ///
    /// And the stamp has none of the three, which is the assertion that stops
    /// a field being added to it later without anyone deciding to.
    #[test]
    fn only_the_typing_kinds_carry_field_copy() {
        for kind in TextAnnotKind::ALL {
            let typed = !kind.uses_gallery();
            assert_eq!(
                !hint(*kind).is_empty(),
                typed,
                "{kind:?}'s hint disagrees with whether it takes typing"
            );
            assert_eq!(!bound(*kind).is_empty(), typed, "{kind:?}'s bound");
            assert_eq!(
                !accept_disabled(*kind).is_empty(),
                typed,
                "{kind:?}'s disabled reason"
            );
            assert!(!title(*kind).is_empty());
            assert!(!intro(*kind).is_empty());
        }
    }

    /// Every stamp the gallery offers has a label, and they are distinct.
    ///
    /// A gallery entry with no label would draw an empty radio the operator
    /// could select and could not identify.
    #[test]
    fn every_offered_stamp_is_named_distinctly() {
        use crate::canvas::textannot::STAMPS;
        for s in STAMPS {
            assert!(!stamp_label(*s).is_empty(), "{s:?} has no label");
        }
        for (i, a) in STAMPS.iter().enumerate() {
            for b in STAMPS.iter().skip(i + 1) {
                assert_ne!(stamp_label(*a), stamp_label(*b));
            }
        }
    }

    /// The three subtypes an operator can write a note onto, plus a handful of
    /// geometric ones, as the four-row table in the section banner asks them.
    ///
    /// Shared by the two tests below so the table is enumerated once. Adding a
    /// subtype here makes both of them ask about it.
    const QUIET: [&str; 6] = ["Text", "Stamp", "Square", "Highlight", "Line", "Polygon"];

    /// ★★★ **Exactly one of the four rows owes the operator a sentence: a
    /// `/FreeText` whose appearance the engine did NOT re-bake.**
    ///
    /// The guard on the section banner's table, and it is deliberately built
    /// **positive first**. The engine's own methodology note, sent with
    /// `Pass 258.1` and aimed at this side of the wire:
    ///
    /// > Our first version of the foreign-appearance test asserted
    /// > `!appearance_rebaked` — and **passed with the entire re-bake
    /// > disabled**. A "not X" assertion is vacuous when the thing that would
    /// > produce X is absent.
    ///
    /// So the first four assertions here establish that these functions *can*
    /// speak, and every `is_none()` below them is a claim about a condition
    /// rather than about a function that never returns anything. Sabotaging
    /// either half of the guard — deleting the `paints_its_note` term or the
    /// `!appearance_rebaked` term — turns this test red, which is the property
    /// a vacuous version would not have.
    ///
    /// The two silences are owed in *opposite* directions and both are
    /// expensive:
    ///
    /// - a sticky or a stamp that started warning "the words printed on the
    ///   page were NOT redrawn" describes a page that never showed those
    ///   words, and a warning an operator learns to dismiss costs the one case
    ///   where it is true;
    /// - a re-baked text box that warned anyway would be the deleted lie
    ///   restored, on a build where the page demonstrably moved.
    #[test]
    fn only_a_foreign_text_box_appearance_owes_the_operator_a_sentence() {
        // ROW 4 — the disclosure. The positive control, first, because it is
        // what stops everything below it being vacuous.
        assert!(
            note_edit_disclosure("FreeText", false).is_some(),
            "a /FreeText whose appearance pdfcer did not author keeps that \
             appearance, so the note moved and the page did not"
        );
        assert!(
            note_clear_disclosure("FreeText", false).is_some(),
            "removing the note is the worse surprise of the two and must speak \
             at least as loudly"
        );

        // ROW 3 — same subtype, appearance re-baked. The page moved with the
        // words, in one undo entry, and the operator can see it.
        assert!(
            note_edit_disclosure("FreeText", true).is_none(),
            "set_markup_note re-baked the appearance (edit.rs:26272), so there \
             is no second half left to disclose"
        );
        assert!(
            note_clear_disclosure("FreeText", true).is_none(),
            "the box was emptied along with its note; warning otherwise is the \
             deleted limitation sentence coming back"
        );

        // ROWS 1 and 2 — a sticky and a stamp, in BOTH values of the flag,
        // because for them `false` is correct and final rather than a failure.
        for quiet in QUIET {
            assert!(
                !paints_its_note(quiet),
                "/{quiet} does not paint its /Contents, so a note edit on one \
                 is complete"
            );
            for rebaked in [false, true] {
                assert!(
                    note_edit_disclosure(quiet, rebaked).is_none(),
                    "/{quiet} was warned with appearance_rebaked={rebaked}; \
                     false is the correct and final answer for this subtype"
                );
                assert!(
                    note_clear_disclosure(quiet, rebaked).is_none(),
                    "/{quiet} was warned with appearance_rebaked={rebaked}"
                );
            }
        }

        assert!(
            paints_its_note("FreeText"),
            "a /FreeText's /Contents IS its appearance's input"
        );
    }

    /// ★★ **Each disclosure says what did NOT change, and WHY it was left
    /// alone.**
    ///
    /// The negative clause was the whole point of these sentences when they
    /// were written and it still is: a disclosure reading only *"the comment
    /// was changed"* is true, is what the operator already knows, and leaves
    /// the whole of the surviving half unsaid.
    ///
    /// ★ The *why* clause is new, and it is what stops the sentence reading as
    /// a defect report. On the old build the picture did not move because
    /// pdfcer could not move it; on this one it did not move because moving it
    /// would have thrown away a shadow, a gradient or an image that another
    /// program drew — preservation, not failure. Asserted on *"another
    /// program"* rather than on a verb, because the verb is rewordable
    /// (`keeps`, `leaves`, `preserves` are all honest) while the **cause** is
    /// not: drop that clause and the sentence reports a capability pdfcer is
    /// missing instead of a decision it made, which is the whole difference
    /// between this sentence and the one it replaced.
    ///
    /// Checked on `NOT` in capitals for the same reason: it is the only token
    /// that cannot survive a trim down to the cheerful half.
    #[test]
    fn both_disclosures_state_the_half_that_did_not_move_and_why() {
        let edited = note_edit_disclosure("FreeText", false).expect("a foreign box discloses");
        let cleared = note_clear_disclosure("FreeText", false).expect("a foreign box discloses");
        for (what, s) in [("edit", edited), ("clear", cleared)] {
            assert!(
                s.contains("NOT"),
                "the {what} disclosure dropped its negative clause: {s:?}"
            );
            assert!(
                s.contains("printed on the page"),
                "the {what} disclosure must name the page, not just the box: {s:?}"
            );
            assert!(
                s.contains("another program"),
                "the {what} disclosure must name WHY the appearance was left \
                 alone, or it reports a capability pdfcer is missing instead \
                 of a decision it made: {s:?}"
            );
            assert!(
                !s.contains("cannot redraw"),
                "the {what} disclosure still claims pdfcer cannot redraw a text \
                 box; set_markup_note has re-baked since 95a936e: {s:?}"
            );
        }
        assert_ne!(
            edited, cleared,
            "changing a note and removing one are different acts and must not share a string"
        );
    }
}
