//! # `text::panels::annotgeometry` — the words the **annotation** half of the
//! Properties panel's geometry section owns
//!
//! ## Why a module of its own, when the four field labels are next door
//!
//! Because the four field labels are **not** this module's, and that is the
//! finding worth writing down rather than the reason for the split.
//!
//! [`crate::panels::properties::geometry`] draws one section over two kinds of
//! subject — a page-content object and a markup annotation — and it draws them
//! with the **same heading, the same units note, the same four labels and the
//! same Apply button**. Those all live in [`super::properties`] and are called
//! by that section unchanged whichever subject it has. Duplicating them under
//! an annotation-shaped name would have produced a second copy of *"Points,
//! measured to the bottom-left corner. Y increases upward."*, which is the one
//! sentence in this whole feature that must not exist twice: it states the
//! coordinate convention, and two copies of a convention is how a panel ends up
//! presenting Y from the top in one half and from the bottom in the other.
//!
//! ⇒ **What is here is only what the annotation subject adds**, which is the
//! one reason a control can be unavailable over an annotation and cannot be
//! over a path: the *file* has refused, before the operator has typed anything.
//!
//! ## ★★ The register: a hover on a greyed control answers "why can't I", not
//! "what went wrong"
//!
//! The sentence below is read while the pointer rests on a control that is
//! visibly present and visibly dead. The operator has not acted yet — nothing
//! has failed, nothing is unchanged, and a sentence written in the past tense
//! (*"that change was refused"*) would describe an event that has not happened.
//! So it is present tense and it names the **agent**: the file marks it locked.
//! [`crate::text::markup::NodeEditRefusal::line`]'s `Locked` arm is the same fact
//! in the other tense, for the surface where the operator has already pulled a
//! grip and watched nothing move, and the two are deliberately worded
//! differently rather than shared.
//!
//! ## ★★★ What is deliberately NOT here, and it is the interesting half
//!
//! **A pre-press warning that a non-uniform resize may be refused.**
//!
//! `EditSession::resize_annotation` refuses to scale an annotation whose
//! appearance stream pdfcer did not draw, unless the scale is uniform or
//! `ResizeOptions::allow_appearance_distortion` takes the distortion knowingly
//! (`pdfcer-core` `edit.rs:24455`). It is a real refusal an operator can reach
//! from these fields by typing a Width and leaving Height alone.
//!
//! It gets **no string here**, because the condition is not one this panel can
//! evaluate. The engine decides it by rebuilding the annotation's appearance
//! from its own spec and **comparing bytes** — a question about the file that
//! cannot be answered without doing the work. A hover reading *"this may be
//! refused"* would therefore be a guess, and it would be the wrong guess for
//! every mark pdfcer drew itself, which is most of the marks on this operator's
//! sheets.
//!
//! ⇒ So the refusal is surfaced **after** the press and **by name**, through
//! `app::actions::annots::resize`'s existing `inspect_err` arm and
//! `app::status::decline::record_resize_not_rebuildable` — the identical
//! sentence the eight resize grips already produce for the identical engine
//! error. One refusal, one wording, one place. A typed Width that the engine
//! declines says exactly what a dragged one says, which is the property that
//! makes the typed route a second *input* rather than a second *feature*.

/// **Why the four fields and Apply are greyed over a locked annotation.**
///
/// §12.5.3 Table 165 bit 8 — `/F` `Locked` — is the file telling every user
/// interface that this annotation's properties may not be changed. pdfcer
/// honours it here by greying rather than by hiding, and that is R9's
/// distinction applied exactly: the capability is present, this *particular*
/// annotation is out of bounds, and selecting a different one restores the
/// fields. Hiding the section instead would have said "pdfcer cannot type
/// geometry", which is false and which the operator would have no way to
/// disprove.
///
/// # ★★ It names the remedy, and the remedy is not in pdfcer
///
/// This shell has no unlock verb — clearing `/F` bit 8 is an authoring act on
/// somebody else's decision, and nothing in `EditSession` offers it. A sentence
/// that stopped at *"this is locked"* would leave the operator hunting a pdfcer
/// menu that does not exist, so it says where the flag can be cleared instead.
/// That is the identical ruling [`crate::text::markup::NodeEditRefusal`]'s
/// `Locked` arm makes for the node editor, reached the same way and worded in
/// the present tense because this is read **before** an attempt rather than
/// after one.
///
/// # ★ "Position and size", not "properties"
///
/// The flag governs more than geometry, but this hover is attached to four
/// geometry fields, and naming the whole of what the flag covers would invite
/// the operator to conclude that the colour swatches above it are also dead
/// when they may not be — that is a different surface's sentence to write.
///
/// # ★ "Comment", not "annotation"
///
/// `crate::text`'s standing rule that a label is the operator's vocabulary.
/// The Comments panel, the ribbon's Comment tab and every disclosure in
/// `crate::text::markup` say *comment*; `annotation` is the word §12.5 uses and
/// the word this crate's identifiers use, and it appears in no sentence an
/// operator reads.
#[must_use]
pub const fn locked() -> &'static str {
    "The file marks this comment as locked, so its position and size cannot be \
     changed here. Unlock it in the program that made it."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ The sentence names what the operator can DO, which is this catalog's
    /// standing rule for anything that says no. A refusal that only diagnoses
    /// leaves them looking for a control.
    ///
    /// **Falsified** by deleting the second sentence of [`locked`]: this went
    /// red on the `"Unlock it"` assertion, and green again when restored.
    #[test]
    fn the_refusal_names_a_remedy() {
        assert!(locked().contains("Unlock it"));
    }

    /// ★★ **Present tense, no past-tense verb about an edit.**
    ///
    /// This is a hover on a control the operator has not pressed. A sentence
    /// saying an edit "was refused" would describe an event that has not
    /// happened — the exact confusion `super::properties`'
    /// `geometry_nothing_typed` avoids next door by naming the next act.
    ///
    /// **Falsified** by rewording [`locked`] to *"that change was refused"*:
    /// red, as it must be.
    #[test]
    fn it_does_not_claim_an_edit_already_happened() {
        let line = locked();
        assert!(
            !line.contains("was refused") && !line.contains("is unchanged"),
            "a hover on a greyed control must not report a past edit: {line}"
        );
    }

    /// ★ The operator's word, not the specification's.
    ///
    /// **Falsified** by swapping `comment` for `annotation` in [`locked`]: red.
    #[test]
    fn it_speaks_the_operators_vocabulary() {
        assert!(
            !locked().contains("annotation"),
            "`annotation` is §12.5's word and this crate's identifier; the \
             operator reads `comment`"
        );
    }
}
