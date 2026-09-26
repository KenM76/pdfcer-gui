//! # `text::panels::attachments` — every string the Attachments panel shows
//!
//! One area of the catalog described in [`crate::text`]'s header, covering
//! [`crate::panels::attachments`] and the three apply arms in
//! [`crate::app::actions::attachments`] that report what those verbs did.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/panels/attachments.md`.

use pdfcer_core::attachments::{AttachmentNotes, DeclaredSizeCheck, NameHazard};

// ---------------------------------------------------------------------------
// The listing
// ---------------------------------------------------------------------------

/// How many files this document carries, as the panel's first line.
#[must_use]
pub fn count(total: usize) -> String {
    if total == 1 {
        "1 attached file.".to_owned()
    } else {
        format!("{total} attached files.")
    }
}

/// Shown when the document carries nothing.
#[must_use]
pub const fn empty() -> &'static str {
    "This document carries no attached files."
}

/// A row for an attachment nothing named.
#[must_use]
pub const fn unnamed() -> &'static str {
    "(unnamed)"
}

/// Where a document-level attachment lives.
#[must_use]
pub const fn where_document() -> &'static str {
    "Attached to the document"
}

/// Where a page-level attachment lives, and the consequence of that.
#[must_use]
pub fn where_page(page_number: usize) -> String {
    format!("On page {page_number} — deleting that page takes this file with it")
}

/// The media type the document claims for the payload.
#[must_use]
pub fn kind_claimed(mime: &str) -> String {
    format!("Type, as the document declares it: {mime}")
}

/// The size line for a row.
#[must_use]
pub fn size(declared: Option<u64>, check: DeclaredSizeCheck) -> String {
    match check {
        // The bare figure, with no qualifying clause, and that is the whole
        // difference between this arm and every other one below: pdfcer counted
        // the bytes and they matched what the document declared, so there is
        // nothing left to hedge. A sentence here would be hedging a fact.
        DeclaredSizeCheck::Agrees { bytes } => human_bytes(bytes),
        DeclaredSizeCheck::Disagrees { declared, actual } => format!(
            "The document says {} and pdfcer counted {}.",
            human_bytes(declared),
            human_bytes(actual)
        ),
        DeclaredSizeCheck::Unverified => match declared {
            Some(bytes) => format!(
                "{} as declared — compressed, so pdfcer has not checked it yet.",
                human_bytes(bytes)
            ),
            None => "Compressed; size unchecked.".to_owned(),
        },
        DeclaredSizeCheck::NoStream => {
            "This entry names a file that is not inside the PDF, so there is nothing to save."
                .to_owned()
        }
        // `NotDeclared`, and any variant a later engine adds. `DeclaredSizeCheck`
        // is `#[non_exhaustive]`, so the catch-all must be the answer that claims
        // the LEAST — "the document did not say" is true of an unknown variant in
        // a way that any size sentence would not be.
        _ => "Size not stated by the document.".to_owned(),
    }
}

/// The created/modified line for a row, or `None` when the document said
/// neither.
#[must_use]
pub fn dates(created: Option<&str>, modified: Option<&str>) -> Option<String> {
    match (created, modified) {
        (Some(c), Some(m)) => Some(format!("Created {c} · modified {m}")),
        (Some(c), None) => Some(format!("Created {c}")),
        (None, Some(m)) => Some(format!("Modified {m}")),
        (None, None) => None,
    }
}

/// Why a date can look like machine output.
#[must_use]
pub const fn date_tooltip() -> &'static str {
    "Shown exactly as the file wrote it. pdfcer does not reformat a date it has not parsed."
}

// ---------------------------------------------------------------------------
// What pdfcer could not vouch for — the per-row disclosures
// ---------------------------------------------------------------------------

/// The name shown is pdfcer's best reading of bytes it could not fully decode.
#[must_use]
pub const fn name_is_approximate() -> &'static str {
    "This name is approximate — some of its characters could not be decoded."
}

/// The document gave no filename at all, so the index key is standing in.
#[must_use]
pub const fn name_is_the_index_key() -> &'static str {
    "The document gave this file no name; what is shown is its index key."
}

/// There is a filespec but no bytes behind it.
#[must_use]
pub const fn no_bytes() -> &'static str {
    "This entry points at a file kept outside the PDF, so pdfcer has nothing to save out."
}

/// The document promised bytes and cannot produce them.
#[must_use]
pub const fn broken_stream() -> &'static str {
    "This attachment's bytes are missing from the file — the document points at them and they are not there."
}

/// The whole listing may be ciphertext.
#[must_use]
pub const fn may_be_encrypted() -> &'static str {
    "This document is encrypted, so anything saved out of it may be unreadable — pdfcer does not decrypt attachments yet."
}

/// Everything the listing had to skip, bound or degrade, as sentences.
#[must_use]
pub fn listing_notes(notes: &AttachmentNotes) -> Vec<String> {
    let mut said = Vec::new();
    if notes.may_be_encrypted {
        said.push(may_be_encrypted().to_owned());
    }
    if notes.truncated {
        said.push(
            "This document has more attachments than pdfcer lists; the rest are not shown."
                .to_owned(),
        );
    }
    if notes.name_tree_budget_exhausted {
        said.push(
            "pdfcer stopped walking this document's attachment index early — it is deeper or larger than pdfcer follows, so some entries are missing from this list."
                .to_owned(),
        );
    }
    if notes.name_tree_cycles > 0 {
        said.push(
            "This document's attachment index loops back on itself. pdfcer skipped the loop; the file is malformed."
                .to_owned(),
        );
    }
    if notes.malformed_tree_entries > 0 {
        said.push(entry_count(
            notes.malformed_tree_entries,
            "entry in this document's attachment index could not be read",
            "entries in this document's attachment index could not be read",
        ));
    }
    if notes.annotations_without_filespec > 0 {
        said.push(entry_count(
            notes.annotations_without_filespec,
            "page note claims to carry a file and does not name one",
            "page notes claim to carry a file and do not name one",
        ));
    }
    if notes.unresolvable_streams > 0 {
        said.push(entry_count(
            notes.unresolvable_streams,
            "attachment's bytes are missing from this file",
            "attachments' bytes are missing from this file",
        ));
    }
    if notes.page_tree_unwalkable {
        said.push(
            "pdfcer could not read this document's page tree, so it did not look for files attached to pages. The list above is complete only for the document itself."
                .to_owned(),
        );
    }
    said
}

/// `"1 <singular>."` or `"N <plural>."` — the shape every count in
/// [`listing_notes`] uses.
fn entry_count(n: usize, singular: &str, plural: &str) -> String {
    if n == 1 {
        format!("1 {singular}.")
    } else {
        format!("{n} {plural}.")
    }
}

// ---------------------------------------------------------------------------
// Attaching
// ---------------------------------------------------------------------------

/// The heading over the attach row.
#[must_use]
pub const fn attach_heading() -> &'static str {
    "Attach a file"
}

/// The hint text in the optional description field.
#[must_use]
pub const fn attach_description_hint() -> &'static str {
    "Description (optional)"
}

/// Why the description is worth typing, and why it can only be typed now.
#[must_use]
pub const fn attach_description_note() -> &'static str {
    "A note about what this file is. It can only be set now — pdfcer cannot edit a description afterwards."
}

/// The button that opens the picker.
#[must_use]
pub const fn attach_button() -> &'static str {
    "Attach file…"
}

/// What pressing it will do, on hover.
#[must_use]
pub const fn attach_tooltip() -> &'static str {
    "Embed a copy of a file inside this PDF. The original is not moved or changed."
}

/// The heading on the platform's file picker.
#[must_use]
pub const fn attach_dialog_title() -> &'static str {
    "Choose a file to attach"
}

/// **What attaching actually did**, said off-canvas because the page cannot
/// show it.
#[must_use]
pub fn attached(name: &str, bytes: u64) -> String {
    format!(
        "{name} is now embedded in this document ({}). It is a copy — the original file is untouched — and it appears on no page.",
        human_bytes(bytes)
    )
}

/// The one refusal this surface can provoke that the operator can understand.
#[must_use]
pub const fn attach_refused_multi_node_tree() -> &'static str {
    "pdfcer cannot add to this document's attachment index — it is stored in a form pdfcer would risk damaging. The files already attached are unharmed and can still be saved out."
}

/// The source file could not be read.
#[must_use]
pub fn attach_source_unreadable(detail: &str) -> String {
    format!("pdfcer could not read that file, so nothing was attached: {detail}")
}

// ---------------------------------------------------------------------------
// Removing
// ---------------------------------------------------------------------------

/// The button that removes one document-level attachment.
#[must_use]
pub const fn remove_button() -> &'static str {
    "Remove"
}

/// What removing does, before the press.
#[must_use]
pub const fn remove_tooltip() -> &'static str {
    "Remove this file from the document — the index entry, the file specification and the bytes, as one undoable step."
}

/// Why a page-level attachment has no Remove button here.
#[must_use]
pub const fn remove_lives_with_the_note() -> &'static str {
    "This one is a note on a page. Remove it from the page, as a comment, rather than from here."
}

/// **What removing actually did** — including the part that is not what
/// the word suggests.
#[must_use]
pub fn removed(name: &str) -> String {
    format!(
        // The command is named in words rather than with the ribbon's ▸
        // separator: `crate::text::glyphs` records that U+25B8 is a codepoint
        // this build's font stack CANNOT DRAW, so a path written that way
        // reaches the operator as a substitution box in the middle of the one
        // sentence that has to be understood.
        "{name} is no longer attached. Its bytes are still recoverable from an earlier revision inside this file until you save a compacted copy, on the File tab."
    )
}

// ---------------------------------------------------------------------------
// Saving one out
// ---------------------------------------------------------------------------

/// The button that writes one attachment to disk.
#[must_use]
pub const fn save_button() -> &'static str {
    "Save a copy…"
}

/// What pressing it will do, on hover.
#[must_use]
pub const fn save_tooltip() -> &'static str {
    "Write this file to disk. pdfcer does not open or check what is in it — the type shown is only what the document claims."
}

/// The heading on the platform's save dialog.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Save the attached file as"
}

/// Where the copy went.
#[must_use]
pub fn saved(path: &str) -> String {
    format!("Saved to {path}.")
}

/// **pdfcer used a different name than the row shows, and here is why.**
#[must_use]
pub fn name_was_changed(from: &str, to: &str, hazards: &[NameHazard]) -> String {
    let mut said = format!("The document calls this file {from}; pdfcer saved it as {to}");
    let mut reasons: Vec<&str> = hazards.iter().copied().map(hazard).collect();
    reasons.dedup();
    if reasons.is_empty() {
        said.push('.');
    } else {
        said.push_str(" — ");
        said.push_str(&reasons.join("; "));
        said.push('.');
    }
    said
}

/// One hazard, in the operator's terms rather than the enum's.
fn hazard(hazard: NameHazard) -> &'static str {
    match hazard {
        NameHazard::PathSeparator => "the name was a path, not a file name",
        NameHazard::ParentTraversal => "it tried to climb out of the folder you chose",
        NameHazard::DriveOrStream => "it named a drive or a hidden data stream",
        NameHazard::ControlCharacter => {
            "it contained characters that can hide a file's real extension"
        }
        NameHazard::BidiOverride => "it contained characters that can reverse how a name reads",
        NameHazard::UndecodableBytes => "some of its characters could not be decoded at all",
        NameHazard::ReservedCharacter => "it contained characters a file name may not hold",
        NameHazard::ReservedDeviceName => "Windows reserves that name for a device",
        NameHazard::TrailingDotOrSpace => {
            "it ended in a dot or a space, which Windows silently strips"
        }
        NameHazard::Empty => "there was nothing usable left to call it",
        NameHazard::TooLong => "it was too long",
        _ => "it was not safe to use as a file name",
    }
}

/// The bytes could not be decoded out of the document.
#[must_use]
pub fn extract_failed(detail: &str) -> String {
    format!("pdfcer could not read that attachment out of the document: {detail}")
}

/// The file could not be written.
#[must_use]
pub fn save_failed(detail: &str) -> String {
    format!("pdfcer could not write that file: {detail}")
}

/// The row the operator pressed is no longer in the document.
#[must_use]
pub const fn gone() -> &'static str {
    "That attachment is no longer in this document, so nothing was saved."
}

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

/// A byte count for a listing, in the file manager's units.
fn human_bytes(bytes: u64) -> String {
    super::byte_size(usize::try_from(bytes).unwrap_or(usize::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;

    /// **A complete listing says nothing**, which is what lets the panel draw
    /// no disclosure block at all.
    #[test]
    fn an_undamaged_listing_discloses_nothing() {
        assert!(listing_notes(&AttachmentNotes::default()).is_empty());
    }

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

    /// **A count of one is not spelled as a plural.**
    #[test]
    fn one_is_never_spelled_as_a_plural() {
        assert_eq!(count(1), "1 attached file.");
        assert!(count(2).contains("2 attached files"));
        assert_eq!(
            entry_count(1, "thing happened", "things happened"),
            "1 thing happened."
        );
        assert_eq!(
            entry_count(4, "thing happened", "things happened"),
            "4 things happened."
        );
        assert_eq!(
            entry_count(0, "thing happened", "things happened"),
            "0 things happened.",
            "zero takes the plural, as English does"
        );
    }

    /// **The removal sentence says the bytes survive, and names the remedy.**
    #[test]
    fn the_removal_sentence_does_not_let_remove_imply_erasure() {
        let said = removed("quote.xlsx");
        assert!(said.contains("quote.xlsx"), "{said}");
        assert!(said.contains("recoverable"), "{said}");
        assert!(said.contains("compacted"), "{said}");
    }

    /// **A size disagreement states both numbers and accuses nobody.**
    #[test]
    fn a_size_disagreement_is_reported_as_a_measurement() {
        let said = size(
            Some(999_999),
            DeclaredSizeCheck::Disagrees {
                declared: 999_999,
                actual: 10,
            },
        );
        assert!(said.contains("says") && said.contains("counted"), "{said}");
        for accusation in ["invalid", "corrupt", "non-conforming", "broken"] {
            assert!(
                !said.to_lowercase().contains(accusation),
                "a size mismatch is not a verdict on the document: {said}"
            );
        }
    }

    /// **An unverified size is not reported as an agreement.**
    #[test]
    fn an_unchecked_size_says_it_is_unchecked() {
        let said = size(Some(4096), DeclaredSizeCheck::Unverified);
        assert!(
            said.contains("declared") || said.contains("unchecked"),
            "{said}"
        );
        let none = size(None, DeclaredSizeCheck::NotDeclared);
        assert!(none.contains("not stated"), "{none}");
    }

    /// **A sanitised name names both spellings and says what was wrong.**
    #[test]
    fn a_renamed_save_says_both_names_and_the_reason() {
        let said = name_was_changed(
            "..\\..\\Windows\\System32\\evil.exe",
            "evil.exe",
            &[NameHazard::PathSeparator, NameHazard::ParentTraversal],
        );
        assert!(said.contains("evil.exe"), "{said}");
        assert!(said.contains("climb out"), "{said}");
        // No Rust identifier reaches the operator.
        assert!(!said.contains("ParentTraversal"), "{said}");
    }

    /// A sanitised name with no recorded hazard still forms a sentence.
    #[test]
    fn a_reasonless_rename_does_not_dangle() {
        let said = name_was_changed("a", "b", &[]);
        assert!(said.ends_with('.'), "{said}");
        assert!(!said.contains("—"), "{said}");
    }

    /// **Every disclosure is a sentence and every label is not.**
    ///
    /// The convention [`crate::text`] states, checked here because this module
    /// holds both kinds two lines apart and the wrong one is easy to copy.
    #[test]
    fn labels_are_names_and_disclosures_are_sentences() {
        for label in [
            attach_heading(),
            attach_button(),
            remove_button(),
            save_button(),
            attach_description_hint(),
            unnamed(),
            where_document(),
        ] {
            assert!(!label.ends_with('.'), "a label takes no full stop: {label}");
        }
        for prose in [
            empty(),
            name_is_approximate(),
            name_is_the_index_key(),
            no_bytes(),
            broken_stream(),
            may_be_encrypted(),
            attach_tooltip(),
            attach_description_note(),
            attach_refused_multi_node_tree(),
            remove_tooltip(),
            remove_lives_with_the_note(),
            save_tooltip(),
            date_tooltip(),
            gone(),
        ] {
            assert!(prose.ends_with('.'), "prose ends in a full stop: {prose}");
        }
    }

    /// **The page-level row states the consequence, not just the location.**
    #[test]
    fn a_page_row_warns_that_deleting_the_page_takes_the_file() {
        let said = where_page(3);
        assert!(said.contains('3'), "{said}");
        assert!(said.contains("deleting that page"), "{said}");
    }
}
