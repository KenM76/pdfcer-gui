//! # `text::panels::annotgeometry` — the words the **annotation** half of the
//! Properties panel's geometry section owns
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/panels/annotgeometry.md`.

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
/// # It names the remedy, and the remedy is not in pdfcer
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
/// # "Position and size", not "properties"
///
/// The flag governs more than geometry, but this hover is attached to four
/// geometry fields, and naming the whole of what the flag covers would invite
/// the operator to conclude that the colour swatches above it are also dead
/// when they may not be — that is a different surface's sentence to write.
///
/// # "Comment", not "annotation"
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

    /// The sentence names what the operator can DO, which is this catalog's
    /// standing rule for anything that says no. A refusal that only diagnoses
    /// leaves them looking for a control.
    ///
    /// **Falsified** by deleting the second sentence of [`locked`]: this went
    /// red on the `"Unlock it"` assertion, and green again when restored.
    #[test]
    fn the_refusal_names_a_remedy() {
        assert!(locked().contains("Unlock it"));
    }

    /// **Present tense, no past-tense verb about an edit.**
    #[test]
    fn it_does_not_claim_an_edit_already_happened() {
        let line = locked();
        assert!(
            !line.contains("was refused") && !line.contains("is unchanged"),
            "a hover on a greyed control must not report a past edit: {line}"
        );
    }

    /// The operator's word, not the specification's.
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
