//! # `text::commands::arrange` — the four labels of **Markup ▸ Arrange**, the
//! controls that decide which mark is on top
//!
//! ## ★ Why this is a file of its own
//!
//! **R2**, and the same seam [`super::markupstyle`] took hours earlier the same
//! day: [`super`] is a large catalog that has already been split twice, and four
//! `CommandText`s written to this project's register — every choice carrying its
//! *why* — is more than its remaining headroom.
//!
//! The seam is one step further along [`super::annotate`]'s line, and the three
//! now form a sequence a reader can hold:
//!
//! | module | the commands that… | governed by |
//! |---|---|---|
//! | [`super::annotate`] | **place** a mark | `add_markup`, a gesture |
//! | [`super::markupstyle`] | **restyle** a mark already placed | `set_markup_style`, an `ObjId` |
//! | this one | **re-depth** a mark already placed | `reorder_annotations`, the page's whole `/Annots` |
//!
//! ## ★★★ The word the four labels must NOT use, and it is the one the file
//! format uses
//!
//! *Z-order.* Every one of these controls is about `/Annots` array order, which
//! §12.5.6 paints in sequence, and the vocabulary a programmer reaches for is
//! **z-order** or **stacking order**. No operator has ever called it that.
//!
//! What they say is **front** and **back**, and every reference application
//! agrees: Acrobat's comment context menu is *Bring to Front / Bring Forward /
//! Send Backward / Send to Back*; Illustrator, InDesign, PowerPoint, Visio and
//! Bluebeam all ship the identical four words in the identical order. That is
//! not four programs converging by accident — it is the vocabulary, and inventing
//! a fifth wording here would cost an operator the one thing they already knew
//! about this feature before they opened pdfcer.
//!
//! ⇒ So the labels are borrowed verbatim, and the tooltips do the explaining.
//!
//! ## ★★ What each tooltip has to say that the label cannot
//!
//! The labels are a **pair of pairs** — two ends and two single steps — and the
//! failure mode they invite is pressing *Bring forward* four times when *Bring
//! to front* was meant. So each tooltip names its own scale explicitly (*all the
//! way* versus *one place*) and, for the single steps, says what the step is
//! measured against.
//!
//! And every one of them says **on this page**, because that is the true scope
//! and it is not guessable: `/Annots` is per page (§7.7.3.4 — it is not
//! inheritable), so "in front of everything" means in front of everything on
//! this sheet and says nothing about the next one.
//!
//! ★ None of the four names a keyboard chord, per [`crate::text::shortcuts`]'
//! rule: the keys are in the manifest's keymap and the shortcuts window reads
//! them from there. A chord written into a tooltip is `DEFECTS.md` D5's shape —
//! a second statement of a fact that already has one.

use super::CommandText;

/// `markup.bring_to_front`
///
/// ★ "Front", not "top". Both are used in the trade, and *front* is what all
/// five reference applications say — and the one that survives the operator's
/// own vocabulary for a drawing, where *top* is a direction on the sheet.
#[must_use]
pub const fn markup_bring_to_front() -> CommandText {
    CommandText::new(
        "Bring to front",
        "Draw the selected mark over everything else on this page, all the way to the front.",
    )
}

/// `markup.bring_forward`
///
/// ★★ *"one place"*, and the tooltip says what a place is. The commonest
/// misreading of this control is that it moves the mark to the front by some
/// unspecified amount; naming the unit — one mark — is what makes the pair of
/// pairs legible without a diagram.
#[must_use]
pub const fn markup_bring_forward() -> CommandText {
    CommandText::new(
        "Bring forward",
        "Draw the selected mark over the next one up, moving it one place forward on this page.",
    )
}

/// `markup.send_backward`
#[must_use]
pub const fn markup_send_backward() -> CommandText {
    CommandText::new(
        "Send backward",
        "Draw the selected mark under the next one down, moving it one place back on this page.",
    )
}

/// `markup.send_to_back`
#[must_use]
pub const fn markup_send_to_back() -> CommandText {
    CommandText::new(
        "Send to back",
        "Draw the selected mark under everything else on this page, all the way to the back.",
    )
}
