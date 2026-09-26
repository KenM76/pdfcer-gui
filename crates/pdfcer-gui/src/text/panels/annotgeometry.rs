//! # `text::panels::annotgeometry` — the words the **annotation** half of the
//! Properties panel's geometry section owns
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/panels/annotgeometry.md`.

/// **Why the four fields and Apply are greyed over a locked annotation.**
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
