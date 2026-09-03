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
}
