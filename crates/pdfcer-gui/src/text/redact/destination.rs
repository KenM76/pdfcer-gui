//! # `text::redact::destination` — where the redacted document goes
//!
//!
//! > *"why does it have to save to a new file right away? Why can't it just
//! > wait on saving until I choose to save over the existing file or save as a
//! > new file?"*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/redact/destination.md`.

//
//
//
//   * `undo_will_be_cleared` said the undo history is destroyed. It is
//     preserved. It is replaced by `removal_happens_at_save`, which says the
//     thing that IS now true and is far more surprising.
//   * `applied_into_document` said the content had been removed from the
//     document and only the file was outstanding. Nothing has been removed —
//     the removal is ARMED — and `staged_into_document` says so.
//
// The claim that survives both rewrites, and it is the one this group
// exists for: **nothing has been written.** That is what an operator most needs
// and is least likely to assume, because every redaction tool he has ever used
// produced a file.
// ---------------------------------------------------------------------------

/// The deferred destination, and **the default since 2026-09-04**.
#[must_use]
pub fn destination_open_document() -> &'static str {
    "This document — the removal happens when you Save"
}

/// The *apply into the open document, now* destination.
#[must_use]
pub const fn destination_open_document_now() -> &'static str {
    "This document, now — the page changes immediately"
}

/// See [`destination_open_document_now`].
#[must_use]
pub const fn destination_open_document_now_tooltip() -> &'static str {
    "Removes the marked content from the open document straight away, so you can see the result. \
     No file is written — saving is still up to you. Undo is cleared, because the content is \
     genuinely gone."
}

/// The permanence statement for [`destination_open_document_now`].
#[must_use]
pub const fn permanence_statement_now() -> &'static str {
    "The marked content will be removed from the open document as soon as you press this. It \
     cannot be undone. No file is written — the document on disk is unchanged until you save."
}

/// The confirm button for [`destination_open_document_now`].
#[must_use]
pub const fn confirm_button_into_document_now() -> &'static str {
    "Remove it now"
}

/// Why the deferred destination is safe, and the one thing about it that
/// surprises people.
#[must_use]
pub fn destination_open_document_tooltip() -> &'static str {
    "Nothing is written and nothing is removed yet — the removal is set up now and carried out when you use Save or Save As. The page will not change in the meantime: you will still see the marks and the content underneath them, because nothing has happened to them. Undo still works, and you can call the whole thing off from this window."
}

/// **The one thing the operator must know before he presses the button,
/// stated ABOVE it.**
#[must_use]
pub fn removal_happens_at_save() -> &'static str {
    "Nothing is removed when you press this. The page will look exactly as it does now — the marks and the content underneath them stay on screen — and the content leaves the document at the moment you use Save or Save As. Until then it is still in the file on disk, and you can call this off with the button below."
}

/// **The confirm control's label for the deferred destination.**
#[must_use]
pub fn confirm_button_into_document() -> &'static str {
    "Set up the removal — it happens when I save"
}

/// **The permanence statement for the deferred destination.**
#[must_use]
pub fn permanence_statement_deferred() -> &'static str {
    "Applying sets the removal up now and carries it out when you save. Nothing is removed yet and nothing is written yet. When it does happen it is a full rewrite, not an edit: nothing can bring the removed content back — not Undo, not a previous revision, not any recovery tool. Until you save, the document you have open and the file on disk both still contain that content."
}

/// **The outcome sentence for the deferred destination.**
#[must_use]
pub fn staged_into_document(regions: u64, pages: usize, residuals: usize) -> String {
    if residuals == 0 {
        format!(
            "Set up — {regions} marked region(s) across {pages} page(s) will be removed when you save. Nothing has been removed yet and the page has not changed: use Save to write the redacted document over the file you opened, or Save As for a new one."
        )
    } else {
        format!(
            "⚠  Set up — {regions} marked region(s) will be removed when you save, but {residuals} item(s) could NOT be removed and will still be in the saved file. Do not treat it as fully redacted; see the report you acknowledged for what and why. Nothing has been removed yet and the page has not changed: use Save or Save As to write it."
        )
    }
}

/// **The sentence after a save that actually performed the removal.**
#[must_use]
pub fn saved_applying_redaction(
    file_name: &str,
    regions: u64,
    pages: usize,
    residuals: usize,
) -> String {
    if residuals == 0 {
        format!(
            "Redacted and saved to {file_name} — {regions} region(s) across {pages} page(s) removed, and verified absent from the saved file. ⚠  The window still shows the marks and the content, because this removal happens at the write and leaves the document you have open alone. The removal is still set up, so every save of this document does it again until you call it off."
        )
    } else {
        format!(
            "⚠  Redacted and saved to {file_name} — {regions} region(s) removed, but {residuals} item(s) could NOT be removed and are still in that file. Do not treat it as fully redacted. The window still shows the marks and the content, because this removal happens at the write. The removal is still set up, so every save of this document does it again until you call it off."
        )
    }
}

/// **The sentence after the operator calls a staged removal off.**
#[must_use]
pub fn staging_cancelled(marks: usize) -> String {
    format!(
        "The removal is called off — nothing will be removed when you save, and ordinary saves work again. The {marks} mark(s) are still on the document and still cover content that is still in the file."
    )
}

// ---------------------------------------------------------------------------
// The STAGED phase — what the dialog says when it is reopened on a document
// whose removal is already armed.
//
// A phase of its own rather than a disabled report, because the two questions
// are different: an unstaged document asks *"shall I?"* and a staged one asks
// *"what did I already decide, and can I change my mind?"*. A report greyed out
// with a note under it answers the first question badly instead of the second
// one well.
// ---------------------------------------------------------------------------

/// The heading of the staged phase.
#[must_use]
pub fn staged_heading() -> &'static str {
    "A removal is already set up for this document"
}

/// **What the staged state actually is**, in the four facts that decide what
/// the operator does next.
#[must_use]
pub fn staged_body() -> &'static str {
    "The marked content has not been removed and the page has not changed. It comes out at the moment you use Save or Save As, which is the only way this document can be saved while the removal is set up. You can keep editing and undoing in the meantime, and you can call the removal off below — that leaves the marks in place and lets ordinary saves work again."
}

/// The control that un-stages a removal.
#[must_use]
pub fn cancel_button_staged() -> &'static str {
    "Call the removal off"
}

/// Its tooltip. Says what survives, because that is what the operator is
/// actually asking.
#[must_use]
pub fn cancel_button_staged_tooltip() -> &'static str {
    "Nothing is removed and nothing is written. The marks stay exactly where they are, so you can set the removal up again later, and ordinary saves start working again."
}

/// **The destination choice's heading.**
#[must_use]
pub fn destination_heading() -> &'static str {
    "Where should the redacted document go?"
}

/// The safe destination, and the default.
#[must_use]
pub fn destination_new_file() -> &'static str {
    "A new file — you choose the name"
}

/// Why the default is the default, without scolding the operator for leaving
/// it.
#[must_use]
pub fn destination_new_file_tooltip() -> &'static str {
    "The document you have open is left exactly as it is, so the content you are removing still exists in it until you decide otherwise."
}

/// The destination that replaces the source document.
#[must_use]
pub fn destination_replace(file_name: &str) -> String {
    format!("Replace {file_name} with the redacted document")
}

/// The consequence of replacing, stated where it is chosen rather than
/// after.
#[must_use]
pub fn destination_replace_tooltip() -> &'static str {
    // *"not Undo, not …"* rather than *"Undo does not reach it"*, and not by
    // preference: `no_post_apply_sentence_mentions_undo_as_a_way_back` rejected
    // the first draft of this sentence on 2026-09-04. The rule-3 sweep accepts
    // the word only in an explicit negation, and matching
    // `permanence_statement`'s own phrasing is what makes the two sentences
    // read as one claim rather than two overlapping ones.
    "The file on disk is overwritten with the redacted document. It is the only remaining copy of the content you are removing, so once it is replaced that content is gone from your machine — not Undo, not an earlier revision of the file, not any recovery tool will bring it back."
}

/// **The third acknowledgement: the FILE, not the content.**
#[must_use]
pub fn overwrite_acknowledgement_checkbox(file_name: &str) -> String {
    format!(
        "I understand that {file_name} will be REPLACED by the redacted document, and that it is the only remaining copy of the content being removed."
    )
}

/// **The confirm control's label when the destination is the open file.**
#[must_use]
pub fn confirm_button_replace(file_name: &str) -> String {
    format!("Permanently remove & replace {file_name} now")
}
