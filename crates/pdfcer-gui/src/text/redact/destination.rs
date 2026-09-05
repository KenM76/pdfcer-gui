//! # `text::redact::destination` — where the redacted document goes
//!
//! ★★★ **Added 2026-09-04, on the operator's explicit instruction**, and the
//! only group of strings in this catalog that exists because a ruling was
//! reversed rather than because a control was added. His words:
//!
//! > *"why does it have to save to a new file right away? Why can't it just
//! > wait on saving until I choose to save over the existing file or save as a
//! > new file?"*
//!
//! Until that day the apply had exactly one destination — a new file — and
//! `crate::dialogs::redact::commit` recorded the absence of any alternative as
//! a design property: *"There is no 'save over the original' branch to find,
//! because there is none to write."* `crate::dialogs::redact::Destination`
//! carries the whole of why that was overruled and what survives of it.
//!
//! ## Its own file, and the seam it was cut along
//!
//! `text/redact.rs` reached 1536 lines with this group in it, over rule R2's
//! 1500-line ceiling. The seam is not arbitrary and it is not "the last thing
//! added": these ten strings are the only ones in the catalog that describe a
//! **destination** rather than a **removal**, they are consumed by one region
//! of one dialog, and every other sentence in the parent file would read the
//! same if they did not exist. A split along "what was added most recently"
//! would have put `confirm_button` here and `confirm_button_replace` there.
//!
//! Re-exported by the parent with `pub use`, so every call site spells these
//! exactly as it did before — `crate::text::redact::destination_replace(..)` —
//! and the split is invisible to consumers, which is the property that makes it
//! a mechanical change rather than a rename.
//!
//! ## ★ The wording rule this group adds to the three it inherits
//!
//! `crate::text::redact`'s rules 1–3 bind here unchanged. This group adds a
//! fourth of its own, and every string below obeys it:
//!
//! > **Name the file.** *"Replace the original"* is a sentence about a role.
//! > *"Replace sheet-01.pdf"* is a sentence about a file, and the operator is
//! > about to destroy a file. Every string here that refers to the document
//! > being replaced takes its name as an argument, including the button label —
//! > which is how `crate::dialogs::redact::file_name_of` came to exist, so that
//! > one file is spelled one way across the choice, the acknowledgement, the
//! > button and the outcome.

// ---------------------------------------------------------------------------
// ★★★ THE DEFERRED DESTINATION — 2026-09-04, evening
//
// The half of O125 that could not be written in the morning. The engine
// shipped `EditSession::apply_redactions` the same afternoon (`Pass 250.1`),
// so applying can now land in the open document and Save decides where it goes
// — which is what he actually asked for:
//
//   "Why can't it just wait on saving until I choose to save over the existing
//    file or save as a new file?"
//
// These strings say the thing the other two groups cannot: **nothing has been
// written**. That is the claim an operator most needs and is least likely to
// assume, because every redaction tool he has ever used produced a file.
// ---------------------------------------------------------------------------

/// The deferred destination, and **the default since 2026-09-04**.
///
/// ★ It replaced [`destination_new_file`] as the default, and the swap is the
/// safer direction rather than the more convenient one: a new file is a write,
/// and this is not one. Nothing on disk changes until the operator saves, so
/// the default answer is now the only one of the three that cannot lose
/// anything.
///
/// Worded as *"this document"* rather than *"the session"* or *"in memory"* —
/// he is looking at a document, and the two nouns that would be accurate are
/// both ours rather than his.
#[must_use]
pub fn destination_open_document() -> &'static str {
    "This document — Save decides where it goes, and when"
}

/// ★★ Why the deferred destination is safe, and the one consequence that is
/// not.
///
/// Three facts in the order they matter: nothing is written now, the file on
/// disk is untouched until he saves, and **the undo history goes**. The third
/// is the price of this route and it is stated in the same breath as the
/// benefit rather than left for [`undo_will_be_cleared`] to carry alone — a
/// tooltip an operator reads while *choosing* is a better place for a
/// consequence than a sentence he reads after choosing.
#[must_use]
pub fn destination_open_document_tooltip() -> &'static str {
    "Nothing is written now. The marked content is removed from the document in front of you, and the file on disk is left exactly as it was until you use Save or Save As — the same as any other edit. The one difference: applying clears the undo history, so nothing before it can be stepped back."
}

/// ★★★ **The undo consequence, stated ABOVE the confirm control.**
///
/// The engine's verb *finalizes*: `EditSession::apply_redactions` collapses the
/// session onto a clean redacted base with an empty undo stack, so every step
/// in the log goes — not only the redaction, and not only the steps that
/// touched the redacted region. Our own engine request offered to take it that
/// way *"and we will disclose it on screen before the operator commits"*; this
/// is that disclosure, and *before* is the whole of its value.
///
/// # ★ Why a count and why two forms
///
/// *"This will discard 14 undo steps"* and *"this will discard 1"* are
/// different decisions, and zero is a real, common and uninteresting state —
/// a document opened, marked, applied, with nothing else done to it. Saying
/// *"0 undo steps will be discarded"* there would spend the operator's
/// attention on a non-event and, worse, teach him to skim the sentence on the
/// runs where it says 14.
///
/// The zero form still says the redaction cannot be stepped back, because that
/// fact does not depend on the count and it is the fact rule 3 exists for.
#[must_use]
pub fn undo_will_be_cleared(steps: usize) -> String {
    if steps == 0 {
        "Applying cannot be stepped back: this document's undo history is cleared at the moment it is applied.".to_owned()
    } else {
        format!(
            "Applying cannot be stepped back, and it clears this document's whole undo history — {steps} step(s), including the ones that have nothing to do with the redaction. You can keep editing afterwards; you cannot step back past this point."
        )
    }
}

/// ★ **The confirm control's label for the deferred destination.**
///
/// No ellipsis, for [`confirm_button_replace`]'s reason inverted: there no
/// further question was coming because the file was already named, and here
/// none is coming because **no file is involved at all**. Promising a picker
/// with a punctuation mark would be a lie the operator acts on either way.
///
/// It says *"from this document"* rather than *"apply"*, because the whole
/// misunderstanding this feature exists to prevent is that applying does
/// something to the marks rather than to the content.
#[must_use]
pub fn confirm_button_into_document() -> &'static str {
    "Permanently remove from this document"
}

/// ★★ **The permanence statement for the deferred destination.**
///
/// A third form of [`super::permanence_statement`], and the only one of the
/// three whose first clause is about something that does **not** happen. The
/// middle clause — the impossibility of getting the content back — is worded
/// identically to its two siblings, deliberately: it is the part an operator
/// must not have to read twice to compare.
///
/// ★ It says the file on disk *"still contains that content until you save
/// over it"*, which is a rule-4 disclosure and not reassurance. An operator who
/// applies, does not save, and hands over the original file has redacted
/// nothing — and that is a genuinely reachable state on this destination and on
/// no other.
#[must_use]
pub fn permanence_statement_deferred() -> &'static str {
    "Applying removes the marked content from the document you have open, and writes nothing. It is a full rewrite of the document in memory, not an edit: nothing can bring the removed content back — not Undo, not a previous revision, not any recovery tool. The file on disk is untouched and still contains that content until you save over it with Save or Save As."
}

/// ★★ **The outcome sentence for the deferred destination.**
///
/// [`super::applied_clean`]'s sibling for the route where no file exists yet,
/// and it is drawn in the edit-disclosure slot by the action funnel rather than
/// in a dialog — because this is an ordinary edit now, and an ordinary edit
/// reports where every other one does.
///
/// Rule 1 is kept mechanically: `residuals` picks between two genuinely
/// different sentences rather than putting a number into one, and the residual
/// form never borrows the clean form's *"verified"*.
///
/// ★ Both forms end with the same instruction — **save, or nothing has
/// happened to the file** — because that is the one thing this route can get
/// wrong that the write-now routes cannot.
#[must_use]
pub fn applied_into_document(
    regions: u64,
    pages: usize,
    residuals: usize,
    undo_steps_cleared: usize,
) -> String {
    let undo = if undo_steps_cleared == 0 {
        String::new()
    } else {
        format!(" {undo_steps_cleared} undo step(s) were discarded.")
    };
    if residuals == 0 {
        format!(
            "Redacted — {regions} region(s) across {pages} page(s) removed from this document, and verified absent from it.{undo} Nothing is on disk yet: use Save to write it over the file you opened, or Save As for a new one."
        )
    } else {
        format!(
            "⚠  Redacted — {regions} region(s) removed from this document, but {residuals} item(s) could NOT be removed and are still in it. Do not treat it as fully redacted; see the report you acknowledged for what and why.{undo} Nothing is on disk yet: use Save or Save As to write it."
        )
    }
}

/// **The destination choice's heading.**
///
/// ★★★ Added 2026-09-04 on the operator's instruction. Until that date the
/// apply had exactly one destination — a new file — and he asked, in the same
/// breath as the refusal complaint:
///
/// > *"why does it have to save to a new file right away? Why can't it just
/// > wait on saving until I choose to save over the existing file or save as a
/// > new file?"*
///
/// Worded as a question about **this document** rather than as a settings
/// label, because it is asked once, here, about one operation.
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
///
/// ★ It **names the file**. *"Replace the original"* is a sentence about a
/// role; *"Replace sheet-01.pdf"* is a sentence about a file, and the operator
/// is about to destroy a file.
#[must_use]
pub fn destination_replace(file_name: &str) -> String {
    format!("Replace {file_name} with the redacted document")
}

/// ★★ The consequence of replacing, stated where it is chosen rather than
/// after.
///
/// Two facts, in the order they matter: the file being replaced is the **only
/// remaining copy** of what is being removed, and nothing in pdfcer brings it
/// back. Neither is a scold and neither is a refusal — the operator asked for
/// this control, and a control that argues with the person using it is a
/// control that gets clicked past.
#[must_use]
pub fn destination_replace_tooltip() -> &'static str {
    // ★ *"not Undo, not …"* rather than *"Undo does not reach it"*, and not by
    // preference: `no_post_apply_sentence_mentions_undo_as_a_way_back` rejected
    // the first draft of this sentence on 2026-09-04. The rule-3 sweep accepts
    // the word only in an explicit negation, and matching
    // `permanence_statement`'s own phrasing is what makes the two sentences
    // read as one claim rather than two overlapping ones.
    "The file on disk is overwritten with the redacted document. It is the only remaining copy of the content you are removing, so once it is replaced that content is gone from your machine — not Undo, not an earlier revision of the file, not any recovery tool will bring it back."
}

/// ★★★ **The third acknowledgement: the FILE, not the content.**
///
/// Distinct from [`confirm_checkbox`] because it is a different fact.
/// That one is about the *content* — that applying removes what is underneath
/// rather than the marks on top. This one is about the *document*: that the
/// file the operator opened will not be there afterwards.
///
/// A person can perfectly well have understood the first and not noticed the
/// second, which is exactly the operator this box exists for. It is asked for
/// **only** while the replace destination is selected, so it never becomes a
/// box that is always there and therefore always ticked.
#[must_use]
pub fn overwrite_acknowledgement_checkbox(file_name: &str) -> String {
    format!(
        "I understand that {file_name} will be REPLACED by the redacted document, and that it is the only remaining copy of the content being removed."
    )
}

/// ★ **The confirm control's label when the destination is the open file.**
///
/// No ellipsis, and that is the point of the wording. On
/// [`confirm_button`] the ellipsis is *"a promise that a further question is
/// coming"* — the file picker. Here no further question is coming, so promising
/// one would be a lie told by a punctuation mark, and the operator would press
/// it expecting a chance to change their mind.
///
/// It names the file for [`destination_replace`]'s reason.
#[must_use]
pub fn confirm_button_replace(file_name: &str) -> String {
    format!("Permanently remove & replace {file_name} now")
}
