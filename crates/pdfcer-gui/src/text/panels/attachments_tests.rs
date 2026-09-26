//! Tests of `text::panels::attachments` that read the application crate.

#[cfg(test)]
mod tests {
    use crate::panels::objects::test_support::engine_fixture;
    use crate::text::panels::attachments::*;

    /// **A damaged document's listing speaks, and says more than one thing.**
    ///
    /// # Why this is a fixture test and not a table of hand-built structs
    ///
    /// It was written the other way first and **could not compile**:
    /// `AttachmentNotes` is `#[non_exhaustive]`, so no crate but `pdfcer-core`
    /// may construct one with a struct expression — including with functional
    /// update syntax, which is the trap, because `..Default::default()` looks
    /// like it should be exempt and is not.
    ///
    /// That is a better constraint than the one it replaced. The engine's
    /// `degenerate.pdf` is a document whose attachment index really does loop
    /// back on itself and really does carry entries that cannot be read, so
    /// what is asserted here is that the **shell's rendering of a real
    /// diagnostic** is non-empty and distinct — rather than that a hand-built
    /// value round-trips through a `match`.
    ///
    /// Distinctness is the half worth having: seven flags rendering the same
    /// sentence would pass a length check and tell an operator nothing about
    /// which of seven different things happened to their file.
    #[test]
    fn a_damaged_listing_speaks_and_its_sentences_are_distinct() {
        let path = engine_fixture("attachments/degenerate.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let (_, notes) = pdfcer_core::attachments::list_attachments_with_notes(&doc);

        let said = listing_notes(&notes);
        assert!(
            said.len() >= 2,
            "this fixture carries a cycle AND unreadable entries; if the panel \
             says fewer than two things, a flag is going unrendered: {said:?}"
        );
        let mut unique = said.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            said.len(),
            "two different faults must not read the same: {said:?}"
        );
        for sentence in &said {
            assert!(sentence.ends_with('.'), "a disclosure is prose: {sentence}");
        }
    }

    /// **An ordinary document's listing still says nothing**, checked against a
    /// real file rather than against a default value.
    #[test]
    fn a_well_formed_document_needs_no_caveat() {
        let path = engine_fixture("attachments/both-kinds.pdf");
        let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
        let (found, notes) = pdfcer_core::attachments::list_attachments_with_notes(&doc);
        assert_eq!(found.len(), 2, "the fixture carries one of each kind");
        assert!(
            listing_notes(&notes).is_empty(),
            "a well-formed document must carry no caveat: {:?}",
            listing_notes(&notes)
        );
    }
}
