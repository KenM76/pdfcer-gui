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
// ★★★ **A `/FreeText`'s `/Contents` and the words painted in it start
// IDENTICAL and then silently diverge, and only this module is in a position
// to say so.**
//
// The module header's rule — *a text box is painted on the page, a sticky note
// is not* — was written about the authoring dialog, where the operator is
// choosing which kind to place. It has a second, sharper consequence that was
// unstated until it was measured on 2026-09-06, and the strings below are
// that consequence.
//
// # What was measured, in the engine, at the pin this build takes
//
// `pdfcer-core` v0.42.0 (`821ab47`, the `Cargo.lock` pin — read, not quoted):
//
// 1. `annot_author::free_text` (`annot_author.rs:3082`) writes the operator's
//    words TWICE: into the `/AP` `/N` appearance stream that is what the page
//    actually shows, and into `/Contents` at `annot_author.rs:3131`. At the
//    moment a text box is placed the two are the same string.
// 2. `EditSession::set_markup_note` (`edit.rs:25827`) commits **one object** —
//    the annotation dictionary (`edit.rs:25930-25940`). The `/AP` stream is a
//    separate object and is not in the command.
// 3. R43, in the engine's own words at `edit.rs:25992`: *"pdfcer paints from
//    `/AP` or not at all."*
//
// ⇒ Editing a placed text box's note rewrites `/Contents` and leaves the page
// showing the words the box was made with. Not a bug in the call — the call
// does exactly what it documents — but an operator typing into a box and
// watching the page not change has been told nothing.
//
// # Why the engine cannot simply re-bake it, so nobody re-derives this
//
// `set_markup_style` (`edit.rs:26055`) DOES regenerate an appearance, and it
// refuses `FreeText` **by name** in its own error list (`edit.rs:26041`). The
// reason is one level down: regeneration reads the annotation back into a spec
// with `annot_author::spec_from_dict` (`annot_author.rs:461`), whose arms are
// the geometric family only. There is no `TextAnnotSpec` reader anywhere in
// the crate, and the engine says so itself, in the sentence it shows an
// operator who pastes one (`edit.rs:41443`):
//
// > *"pdfcer can author this kind of annotation but cannot yet read one back
// > off a page into the model the clipboard carries."*
//
// `regenerate_appearances` (`edit.rs:31424`) is not the escape hatch either —
// it needs an `/AcroForm` and walks fields, not annotations.
//
// # ★★ Why this DISCLOSES rather than refuses, which was the live choice
//
// Refusing the edit by name was weighed and rejected, and the reasoning is
// recorded because the opposite call would also have been defensible:
//
// - A `/FreeText`'s `/Contents` is **not only** the painted text. It is the
//   comment body this shell's own Comments panel lists and a reader's comment
//   list shows. `canvas::notepopup::model`'s own header already argued this,
//   before the divergence was measured: *"a reviewer correcting a typo needs
//   the second one."*
// - Refusing would put the Comments panel back to read-only for one subtype —
//   which is the exact defect `set_markup_note` was built to close. The engine
//   names it (`edit.rs:25767`): *"what they actually shipped was the fourth
//   option: their Comments panel is read-only."*
// - The act is recoverable: it is one undo entry, and `MarkupNoteChange`
//   carries the replaced words back.
//
// So the edit happens and renders exactly as it will save (R8b), and the
// operator is told — **twice, and off-canvas both times**: once in the editor
// before they type, once in the status line after it is written. Nothing is
// drawn onto the page; R8b's surviving half is that an inference the operator
// cannot see still owes a report, and one report before the fact is worth more
// than one after it.
//
// Filed at the engine as `request_a_text_boxs_painted_words_cannot_be_rewritten.md`.

/// Whether this `/Subtype` PAINTS its `/Contents` onto the page.
///
/// The single subtype test the three strings below share, so the question is
/// asked in one place rather than three. `subtype` is `pdfcer-core`'s own
/// `Annotation::subtype_label` output (`annot.rs:640`) — the raw `/Subtype`
/// name, not a label of ours.
///
/// # ★ Why the answer is `FreeText` and NOT the other two note-bearing kinds
///
/// Measured per kind rather than assumed for the family, and the family turns
/// out not to be uniform:
///
/// | `/Subtype` | `/Contents` painted? | measured at |
/// |---|---|---|
/// | `FreeText` | **yes** — it is the appearance's own input | `annot_author.rs:3131` |
/// | `Text` (sticky) | no — *"shown by the reader's popup, never painted on the page"* | `annot_author.rs:3197` |
/// | `Stamp` | no — `stamp()` writes `/Name` and never writes `/Contents` at all; the painted label comes from the name | `annot_author.rs:3298` |
///
/// ⇒ Editing a sticky note's or a stamp's note is **completely correct today**
/// and owes no disclosure. Saying otherwise would be a warning an operator
/// learns to ignore, which costs the one case that is real.
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

/// Shown in the note editor **before** the operator types, or `None` for a
/// subtype whose note edit is complete.
///
/// The before half. It is the one that can still change what the operator
/// does — a disclosure that arrives after the write can only explain.
#[must_use]
pub fn note_edit_hint(subtype: &str) -> Option<&'static str> {
    paints_its_note(subtype).then_some(
        "The words printed in this box cannot be changed once it is placed. \
         What you type here is the comment attached to it: the Comments panel \
         and a reader's comment list show it, and the box on the page keeps \
         the words it was made with.",
    )
}

/// The status-line disclosure **after** a note is written, or `None`.
///
/// ★ States the two halves separately and in that order — what did change,
/// then what did not — because an operator who reads only the first clause
/// must not come away believing the page moved.
#[must_use]
pub fn note_edit_disclosure(subtype: &str) -> Option<&'static str> {
    paints_its_note(subtype).then_some(
        "The comment on this text box was changed. The words printed on the \
         page were NOT: pdfcer cannot redraw a text box once it is placed, so \
         the box still reads what it did before.",
    )
}

/// The status-line disclosure after a note is **removed**, or `None`.
///
/// A separate string from [`note_edit_disclosure`] rather than a shared one,
/// for this module's standing reason: the two describe different acts, and a
/// shared sentence is one an author can reword for one and silently change for
/// the other. Removing is also the worse surprise — the comment is gone from
/// every list and the page is unchanged, so the operator has deleted the only
/// copy they could see and kept the one they cannot edit.
#[must_use]
pub fn note_clear_disclosure(subtype: &str) -> Option<&'static str> {
    paints_its_note(subtype).then_some(
        "The comment on this text box was removed. The words printed on the \
         page were NOT: pdfcer cannot redraw a text box once it is placed, so \
         the box still reads what it did before.",
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

    /// ★★★ **Only the text box owes the painted-words disclosure, and the
    /// other two owe it in neither direction.**
    ///
    /// The guard on the measurement in the section banner. It is written as a
    /// both-directions assertion rather than one `assert!(paints_its_note(
    /// "FreeText"))` because the expensive mistake is the *false positive*: a
    /// sticky note that started warning "the words printed on the page were
    /// NOT changed" would be telling the operator about a page that never
    /// showed those words, and a warning an operator learns to dismiss is
    /// worse than none — it costs the one case that is real.
    ///
    /// `Stamp` is in the list on its own footing. Its `/Contents` is a comment
    /// ABOUT the stamp; `annot_author::stamp` never writes that key at all, so
    /// there is nothing for an edit to make stale.
    #[test]
    fn only_a_text_box_paints_the_words_its_note_edit_changes() {
        assert!(
            paints_its_note("FreeText"),
            "a /FreeText's /Contents IS its appearance's input (annot_author.rs:3131)"
        );
        for quiet in ["Text", "Stamp", "Square", "Highlight", "Line", "Polygon"] {
            assert!(
                !paints_its_note(quiet),
                "/{quiet} does not paint its /Contents, so a note edit on one is complete \
                 and owes no disclosure"
            );
            assert!(note_edit_hint(quiet).is_none(), "/{quiet} was warned");
            assert!(note_edit_disclosure(quiet).is_none(), "/{quiet} was warned");
            assert!(
                note_clear_disclosure(quiet).is_none(),
                "/{quiet} was warned"
            );
        }
    }

    /// ★★ **Each disclosure says what did NOT change, not only what did.**
    ///
    /// The one property that makes these sentences worth drawing. A disclosure
    /// reading only *"the comment was changed"* is true, is what the operator
    /// already knows, and leaves the whole defect unsaid — so the negative
    /// clause is asserted rather than trusted to review.
    ///
    /// Checked on the word `NOT` in capitals: it is the only token in either
    /// string that cannot survive an author trimming the sentence down to its
    /// cheerful half.
    #[test]
    fn both_disclosures_state_the_half_that_did_not_move() {
        let edited = note_edit_disclosure("FreeText").expect("a text box discloses");
        let cleared = note_clear_disclosure("FreeText").expect("a text box discloses");
        for (what, s) in [("edit", edited), ("clear", cleared)] {
            assert!(
                s.contains("NOT"),
                "the {what} disclosure dropped its negative clause: {s:?}"
            );
            assert!(
                s.contains("printed on the page"),
                "the {what} disclosure must name the page, not just the box: {s:?}"
            );
        }
        assert_ne!(
            edited, cleared,
            "changing a note and removing one are different acts and must not share a string"
        );
    }

    /// The hint reaches the operator BEFORE the write, and says the same thing
    /// as the disclosure that follows it.
    ///
    /// Both halves matter. A hint that contradicted the status line would be
    /// two answers to one question; a hint that merely repeated it would be
    /// spent in the wrong place, since only the hint can still change what the
    /// operator does.
    #[test]
    fn the_editor_hint_warns_before_the_write_and_agrees_with_it() {
        let hint = note_edit_hint("FreeText").expect("a text box hints");
        assert!(
            hint.contains("cannot be changed"),
            "the hint must say the painted words are fixed: {hint:?}"
        );
        assert!(
            hint.contains("comment"),
            "the hint must say what the operator IS editing, or it is only a refusal: {hint:?}"
        );
        assert_ne!(
            Some(hint),
            note_edit_disclosure("FreeText"),
            "the before and after strings address different moments and are not one string"
        );
    }
}
