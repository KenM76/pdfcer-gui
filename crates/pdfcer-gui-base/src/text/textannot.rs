//! # `text::textannot` — every word the text-annotation dialog shows
//!
//! Copy for the three markup kinds that carry words: text box, sticky note and
//! stamp.
//!
//! ## The one distinction every string here has to preserve
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
//! ## It is no longer only the dialog's copy, and that is deliberate
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
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/textannot.md`.

use crate::wordmarkup::{StampSize, TextAnnotKind};
use pdfcer_core::annot_author::{StampName, StickyIcon};

/// The window's title.
#[must_use]
pub const fn title(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => "Text box",
        TextAnnotKind::Sticky => "Sticky note",
        TextAnnotKind::Stamp => "Stamp",
        TextAnnotKind::Attachment => "Attach a file",
        TextAnnotKind::Caret => "Insert text",
        TextAnnotKind::Sound => "Attach a sound",
        TextAnnotKind::Screen => "Place a media clip",
    }
}

/// The sentence under the title.
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
        TextAnnotKind::Attachment => {
            "The file is stored inside this PDF and a marker shows where. \
             Anyone with the PDF can open or save the file from the marker. \
             The marker prints; the file does not."
        }
        TextAnnotKind::Caret => {
            "A caret marks where the words should go. They are a proposal in a \
             comment: the page's own text does not change."
        }
        TextAnnotKind::Sound => {
            "The recording is stored inside this PDF and an icon shows where; \
             clicking the icon plays it. PDF 2.0 deprecates sound comments, so \
             some readers may not play them. The icon prints; the sound does not."
        }
        TextAnnotKind::Screen => {
            "The clip is stored inside this PDF and plays in the region you \
             drew. A frame with a play symbol shows there and prints; the clip \
             does not. Many readers, including most browsers, do not play clips."
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
        TextAnnotKind::Attachment => "Description (optional)",
        TextAnnotKind::Caret => "Words to insert",
        TextAnnotKind::Sound | TextAnnotKind::Screen => "Description (optional)",
    }
}

/// What the operator should know about the field, under it.
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
        TextAnnotKind::Attachment => {
            "Stored with the file, and shown by PDF readers as the attachment's \
             description."
        }
        TextAnnotKind::Caret => {
            "Shown when the comment is opened, in a standard Latin font; other \
             alphabets come out as question marks."
        }
        TextAnnotKind::Sound => {
            "Shown when the comment is opened, in a standard Latin font; other \
             alphabets come out as question marks."
        }
        TextAnnotKind::Screen => {
            "Stored as the region's description; readers that list a page's \
             media show it."
        }
    }
}

/// One stamp's name, as an operator reads it.
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

/// One sticky-note icon's name, as an operator reads it.
#[must_use]
pub fn sticky_icon_label(icon: &StickyIcon) -> &'static str {
    match icon {
        StickyIcon::Comment => "Comment",
        StickyIcon::Key => "Key",
        StickyIcon::Note => "Note",
        StickyIcon::Help => "Help",
        StickyIcon::NewParagraph => "New paragraph",
        StickyIcon::Paragraph => "Paragraph",
        StickyIcon::Insert => "Insert",
        // **A name §12.5.6.4 permits and pdfcer does not model** —
        // `StickyIcon::Other`, new in `pdfcer-core` `Pass 253.5`.
        //
        // This function cannot name it, and that is a property of its return
        // type rather than an omission: a `&'static str` cannot carry bytes out
        // of the operator's file. The surface that CAN —
        // `text::panels::textannotstyle::markup_icon_foreign_named` — returns
        // an owned `String` and prints the name in quotes.
        //
        // ⇒ Every caller that may be handed one must therefore match on
        // `Other` **before** reaching here. The properties panel's chooser
        // does. A caller that does not will show this word, which is why it
        // says *what pdfcer knows about it* rather than pretending to a name.
        //
        // `#[non_exhaustive]` is NOT what this arm is for. `StickyIcon` is a
        // closed enum; this is a real variant with a real meaning, and a
        // wildcard here would also silently swallow an eighth standard icon if
        // §12.5.6.4 ever grew one.
        StickyIcon::Other(_) => "Another icon",
    }
}

/// The label over the sticky note's icon chooser.
#[must_use]
pub const fn sticky_icon_heading() -> &'static str {
    "Icon"
}

/// **What the icon actually changes** — said under every chooser that
/// offers one, on both surfaces.
#[must_use]
pub const fn sticky_icon_bound() -> &'static str {
    "The icon is recorded in the file and other PDF readers draw it. pdfcer \
     draws its own note marker for all of them, so this will not change how \
     the note looks here."
}

/// What the gallery does not offer, said once under it.
#[must_use]
pub const fn stamp_bound() -> &'static str {
    "These are the standard stamps. pdfcer does not add a name or a date to \
     them, because it does not know who you are."
}

/// The label over the stamp's size chooser.
#[must_use]
pub const fn stamp_size_heading() -> &'static str {
    "Size"
}

/// One label size, as an operator reads it in the chooser.
#[must_use]
pub fn stamp_size_label(size: StampSize) -> String {
    match size {
        StampSize::FitTheBox => "Fit the box I drew".to_string(),
        StampSize::Points(pt) => format!("{pt} pt"),
    }
}

/// **What the size chooser does to the box**, said once under it.
#[must_use]
pub const fn stamp_size_bound() -> &'static str {
    "The stamp gets wider if the words need it, and never narrower than the box you drew."
}

/// The commit control.
#[must_use]
pub const fn accept() -> &'static str {
    "Add"
}

/// Why Add is greyed.
#[must_use]
pub const fn accept_disabled(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => "Type what the box should say first.",
        TextAnnotKind::Sticky => "Type the note first.",
        // Unreachable: a stamp is always ready. Answered rather than panicking,
        // as `hint` is.
        TextAnnotKind::Stamp => "",
        // Unreachable: the file is picked before the dialog opens.
        TextAnnotKind::Attachment | TextAnnotKind::Sound | TextAnnotKind::Screen => "",
        TextAnnotKind::Caret => "Type the words to insert, or tick New paragraph.",
    }
}

/// The label of the caret's new-paragraph choice.
#[must_use]
pub const fn caret_paragraph() -> &'static str {
    "New paragraph"
}

/// What the new-paragraph choice adds, said under it.
#[must_use]
pub const fn caret_paragraph_bound() -> &'static str {
    "Adds a paragraph mark (¶) above the caret, asking for a paragraph break there."
}

/// The disclosure after a caret is placed.
#[must_use]
pub fn caret_placed(page: usize) -> String {
    format!(
        "Insert-text comment added on page {}. The page's text is unchanged; \
         the words are in the comment.",
        page + 1
    )
}

/// The replace-text window's title.
#[must_use]
pub const fn replace_title() -> &'static str {
    "Replace text"
}

/// The sentence under the replace-text title.
#[must_use]
pub const fn replace_intro() -> &'static str {
    "The selected text is struck through and a caret follows it. The new words \
         are a proposal in a comment: the page's own text does not change."
}

/// The replace-text field's placeholder.
#[must_use]
pub const fn replace_hint() -> &'static str {
    "Replacement words"
}

/// Why Add is greyed in the replace-text window.
#[must_use]
pub const fn replace_accept_disabled() -> &'static str {
    "Type the replacement words first."
}

/// The disclosure after a replace-text comment is placed.
#[must_use]
pub fn replace_placed(page: usize) -> String {
    format!(
        "Replace-text comment added on page {}. The page's text is unchanged; \
         the replacement is in the comment.",
        page + 1
    )
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
// **A `/FreeText`'s `/Contents` and the words painted in it are kept in
// step BY THE ENGINE, in the same command and the same undo entry.**
//
// `annot_author::free_text` writes the operator's words TWICE — into the `/AP`
// `/N` appearance stream, which is what the page actually shows, and into
// `/Contents` — and R43 is the engine's own rule that *"pdfcer paints from
// `/AP` or not at all."* A verb that rewrote only the dictionary would leave
// the page showing the words the box was made with, and the divergence would
// have no visible first moment, because at authoring time the two strings are
// the same.
//
// `EditSession::set_markup_note` closes that itself, opt-out-free. On a
// `/FreeText` whose appearance pdfcer would have drawn, the command carries
// **two** `ObjectWrite`s — the dictionary and the `/AP` stream — and
// [`pdfcer_core::edit::MarkupNoteChange::appearance_rebaked`] reports whether
// the second happened. `rebake_free_text_appearance` is what reads the
// ORIGINAL dictionary back into a spec, re-bakes and commits, so the two
// halves share one undo entry and can never be undone apart.
//
// ⇒ There is therefore **no hint before the write**, and no useful one is
// available: whether an appearance is foreign is measured *inside* the verb,
// by baking and comparing bytes, so no editor drawn before the call can know
// it. The disclosure is off-canvas and after the fact, and it keys on the
// engine's own answer rather than on a subtype this shell classified.
//
// # Which `appearance_rebaked == false` owes a sentence, and why the
// subtype test is not enough
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
/// # Why the answer is `FreeText` and NOT the other two note-bearing kinds
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
/// The stamp row is the one worth reading twice. A stamp's `/Contents` is a
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
#[must_use]
pub fn note_clear_disclosure(subtype: &str, appearance_rebaked: bool) -> Option<&'static str> {
    (paints_its_note(subtype) && !appearance_rebaked).then_some(
        "The comment on this text box was removed. The words printed on the \
         page were NOT redrawn: this box was drawn by another program, so \
         pdfcer keeps its appearance exactly as it is rather than emptying it, \
         and the page still reads what it did before.",
    )
}

/// The status-line disclosure after a mark is placed whose words contained
/// characters pdfcer could **not** write, or `None`.
#[must_use]
pub fn placed_unencodable(count: usize) -> Option<String> {
    (count > 0).then(|| {
        format!(
            "{count} character(s) in these words have no code in the font this mark is drawn \
             in, and were written as `?`. The rest of the mark is exactly as you typed it."
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two kinds' intros disagree about printing, out loud.**
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

    /// Every kind that takes typing has a hint and a bound, and every kind whose
    /// typing is required has a greyed reason. An attachment's description is
    /// optional, so its Add is never greyed.
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
            let required = typed
                && !matches!(
                    kind,
                    TextAnnotKind::Attachment | TextAnnotKind::Sound | TextAnnotKind::Screen
                );
            assert_eq!(
                !accept_disabled(*kind).is_empty(),
                required,
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
        use crate::wordmarkup::STAMPS;
        for s in STAMPS {
            assert!(!stamp_label(*s).is_empty(), "{s:?} has no label");
        }
        for (i, a) in STAMPS.iter().enumerate() {
            for b in STAMPS.iter().skip(i + 1) {
                assert_ne!(stamp_label(*a), stamp_label(*b));
            }
        }
    }

    /// ☑ **Every label size the chooser offers has a distinct label, and the
    /// numeric ones carry their unit.**
    #[test]
    fn every_offered_stamp_size_is_named_distinctly_and_carries_its_unit() {
        use crate::wordmarkup::STAMP_SIZES;
        for size in STAMP_SIZES {
            let label = stamp_size_label(*size);
            assert!(!label.is_empty(), "{size:?} has no label");
            match size {
                StampSize::Points(pt) => {
                    assert!(
                        label.contains("pt"),
                        "{pt} pt is offered without its unit: {label:?}"
                    );
                    assert!(
                        label.contains(&pt.to_string()),
                        "{pt} pt is offered without its number: {label:?}"
                    );
                }
                // The policy entry must NOT read as a number, which is the
                // whole reason it is a sentence. Asserted rather than trusted,
                // because "Auto" would pass every other clause here.
                StampSize::FitTheBox => assert!(
                    !label.chars().any(|c| c.is_ascii_digit()),
                    "the fit-the-box entry reads as a size: {label:?}"
                ),
            }
        }
        for (i, a) in STAMP_SIZES.iter().enumerate() {
            for b in STAMP_SIZES.iter().skip(i + 1) {
                assert_ne!(
                    stamp_size_label(*a),
                    stamp_size_label(*b),
                    "two entries in the size chooser read the same"
                );
            }
        }
    }

    /// **The size disclosure says the box GROWS and says it is never
    /// narrowed** — both halves, because either alone is a different promise.
    #[test]
    fn the_size_disclosure_states_both_directions() {
        let said = stamp_size_bound();
        assert!(
            said.contains("wider"),
            "the growth half is missing: {said:?}"
        );
        assert!(
            said.contains("never"),
            "the never-shrunk half is missing: {said:?}"
        );
        assert!(!stamp_size_heading().is_empty());
    }

    /// The three subtypes an operator can write a note onto, plus a handful of
    /// geometric ones, as the four-row table in the section banner asks them.
    ///
    /// Shared by the two tests below so the table is enumerated once. Adding a
    /// subtype here makes both of them ask about it.
    const QUIET: [&str; 6] = ["Text", "Stamp", "Square", "Highlight", "Line", "Polygon"];

    /// **Exactly one of the four rows owes the operator a sentence: a
    /// `/FreeText` whose appearance the engine did NOT re-bake.**
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
            "set_markup_note re-baked the appearance in the same command, so there \
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

    /// **Each disclosure says what did NOT change, and WHY it was left
    /// alone.**
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
