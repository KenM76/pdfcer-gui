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
// ★★★ THE DEFERRED DESTINATION — 2026-09-04 evening, REWRITTEN 2026-09-05
//
// The half of O125 that could not be written on the morning of 2026-09-04. The
// engine shipped `EditSession::apply_redactions` that afternoon (`Pass 250.1`)
// and this group said what that verb did: the removal landed in the open
// document, the page changed, and the whole undo log went with it.
//
// `Pass 250.2` (2026-09-05) changed the verb underneath these sentences, and
// the sentences are REWRITTEN rather than softened, because two of them were
// exactly backwards:
//
//   * `undo_will_be_cleared` said the undo history is destroyed. It is
//     preserved. It is replaced by `removal_happens_at_save`, which says the
//     thing that IS now true and is far more surprising.
//   * `applied_into_document` said the content had been removed from the
//     document and only the file was outstanding. Nothing has been removed —
//     the removal is ARMED — and `staged_into_document` says so.
//
// ★★★ The claim that survives both rewrites, and it is the one this group
// exists for: **nothing has been written.** That is what an operator most needs
// and is least likely to assume, because every redaction tool he has ever used
// produced a file.
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
///
/// ★★ 2026-09-05: *"the removal happens when you Save"* rather than *"Save
/// decides where it goes, and when"*. The old half-sentence was true of the
/// collapsing verb, where the removal had already happened and only its
/// destination was outstanding. On `Pass 250.2` the removal itself is what is
/// outstanding, and a label that talks only about *where* invites the operator
/// to believe the *what* is already done.
#[must_use]
pub fn destination_open_document() -> &'static str {
    "This document — the removal happens when you Save"
}

/// ★★ Why the deferred destination is safe, and the one thing about it that
/// surprises people.
///
/// **REWRITTEN 2026-09-05.** The sentence it replaces ended *"applying clears
/// the undo history, so nothing before it can be stepped back"*, which was the
/// price of `Pass 250.1`'s collapsing verb and is now false in the strongest
/// possible way: this route is the one that **keeps** the undo history, and
/// keeping it is the entire reason the engine shipped `Pass 250.2`.
///
/// Three facts, in the order an operator needs them: nothing is written and
/// nothing is removed *yet*; the page will not change, which is the surprise;
/// and undo still works, so this is reversible until it reaches a file.
#[must_use]
pub fn destination_open_document_tooltip() -> &'static str {
    "Nothing is written and nothing is removed yet — the removal is set up now and carried out when you use Save or Save As. The page will not change in the meantime: you will still see the marks and the content underneath them, because nothing has happened to them. Undo still works, and you can call the whole thing off from this window."
}

/// ★★★ **The one thing the operator must know before he presses the button,
/// stated ABOVE it.**
///
/// **This function replaces `undo_will_be_cleared(steps)`, 2026-09-05**, in the
/// same place, in the same warning role, drawn from the same region — and it
/// says the opposite thing, because the engine's verb changed.
///
/// What the old sentence carried was the *price* of the collapsing verb: the
/// whole undo log went, and the operator had accepted that on condition he was
/// told the step count first. There is no price to state any more. What has
/// taken its place is a **surprise**, and it is arguably the more important
/// disclosure of the two:
///
/// > He presses a button labelled *"Permanently remove from this document"* and
/// > **the page does not change.**
///
/// Every redaction tool he has used draws a black box the instant he confirms.
/// This one arms a save. Without this sentence the two readings available to
/// him are *"it did not work"* and *"it worked and the marks are just still
/// drawn"* — and the second is the one that ships a marked file.
///
/// # ★ Why no count, where the old one had two forms
///
/// The old sentence branched on the number of undo steps because *"this will
/// discard 14"* and *"this will discard 1"* are different decisions. Nothing
/// here is a quantity: the removal is armed or it is not, the page does not
/// change either way, and Save is the moment either way. A number would be
/// decoration on the one sentence in this dialog that must be read.
#[must_use]
pub fn removal_happens_at_save() -> &'static str {
    "Nothing is removed when you press this. The page will look exactly as it does now — the marks and the content underneath them stay on screen — and the content leaves the document at the moment you use Save or Save As. Until then it is still in the file on disk, and you can call this off with the button below."
}

/// ★ **The confirm control's label for the deferred destination.**
///
/// No ellipsis, for [`confirm_button_replace`]'s reason inverted: there no
/// further question was coming because the file was already named, and here
/// none is coming because **no file is involved at all**. Promising a picker
/// with a punctuation mark would be a lie the operator acts on either way.
///
/// ★★ 2026-09-05: *"Set up"* rather than *"Permanently remove from this
/// document"*. The old label was the consequence of the collapsing verb and is
/// now a description of something the button does not do — it arms a removal
/// that happens at the save — and on this dialog the standing rule is that
/// **the label IS the consequence**. A label claiming an immediate removal on a
/// control that performs none is the same defect as an ellipsis promising a
/// picker that never opens, one order of magnitude worse.
///
/// It still says *"the content"* rather than *"the marks"*, because the whole
/// misunderstanding this feature exists to prevent is that applying does
/// something to the marks rather than to the content.
#[must_use]
pub fn confirm_button_into_document() -> &'static str {
    "Set up the removal — it happens when I save"
}

/// ★★ **The permanence statement for the deferred destination.**
///
/// A third form of [`super::permanence_statement`], and the only one of the
/// three whose first clause is about something that does **not** happen. The
/// middle clause — the impossibility of getting the content back — is worded
/// identically to its two siblings, deliberately: it is the part an operator
/// must not have to read twice to compare.
///
/// ★★★ **REWRITTEN 2026-09-05.** Its first clause used to read *"Applying
/// removes the marked content from the document you have open, and writes
/// nothing"*, which was true of the collapsing verb and is now false in the
/// most expensive direction available: it claims a removal that has not
/// happened, at the top of the report, in the warning role, which is the one
/// sentence a reader who takes in nothing else takes in.
///
/// ★ It still says the file on disk *"still contains that content"*, which is a
/// rule-4 disclosure and not reassurance. An operator who arms a redaction,
/// does not save, and hands over the original file has redacted nothing — and
/// that is a genuinely reachable state on this destination and on no other.
#[must_use]
pub fn permanence_statement_deferred() -> &'static str {
    "Applying sets the removal up now and carries it out when you save. Nothing is removed yet and nothing is written yet. When it does happen it is a full rewrite, not an edit: nothing can bring the removed content back — not Undo, not a previous revision, not any recovery tool. Until you save, the document you have open and the file on disk both still contain that content."
}

/// ★★ **The outcome sentence for the deferred destination.**
///
/// [`super::applied_clean`]'s sibling for the route where nothing has been
/// removed yet, and it is drawn in the edit-disclosure slot by the action
/// funnel rather than in a dialog — because arming a save is an ordinary edit
/// now, and an ordinary edit reports where every other one does.
///
/// ★★★ **RENAMED from `applied_into_document` and rewritten, 2026-09-05.** The
/// old sentence began *"Redacted — N region(s) … removed from this document,
/// and verified absent from it"* and both halves of that are now false: nothing
/// has been removed, and nothing has been verified, because the engine's
/// staging verb discards the bytes there would have been to sweep
/// (`crate::redact` §1.0.1). Saying *"verified"* here would have been the
/// catalog's rule 2 broken at the one place it is load-bearing.
///
/// Rule 1 is kept mechanically: `residuals` picks between two genuinely
/// different sentences rather than putting a number into one.
///
/// ★ Both forms say the same two things at the end — **the page has not
/// changed** and **Save is what does it** — because those are the two things
/// this route can get wrong that the write-now routes cannot.
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

/// ★★★ **The sentence after a save that actually performed the removal.**
///
/// New 2026-09-05, and it carries three facts an operator has no other way to
/// learn. It is recorded by `crate::app::save` on every one of the three save
/// verbs, and on `file.save` it is recorded **after** the ordinary
/// *"saved to …"* receipt, deliberately — the slot holds one disclosure, and
/// this is the one rule 4 says wins.
///
/// 1. **It happened.** The word *verified* is earned here and nowhere else on
///    this route: `crate::redact::save_applying_pending` swept the exact bytes
///    for the removed text before returning them, and `crate::app::save`
///    swept them again between the buffer and the syscall.
/// 2. **The window is now stale**, in the same way and for the same reason
///    [`super::applied_clean`]'s replace form is stale: the session was never
///    mutated, so the canvas goes on drawing the marks and the content while the
///    file no longer holds either. Getting this wrong in the reassuring
///    direction — *"the document is redacted"* — teaches him that a page which
///    still shows a name is a page whose name was removed.
/// 3. **It is still armed.** `save_applying_redaction` takes `&self` and does
///    not clear the flag, so the next save applies the removal again and the
///    ordinary save modes stay refused until he cancels. *"I saved it, so it is
///    done"* is the assumption that would otherwise stand, and it is wrong in
///    the direction that surprises him at the next `Ctrl+S`.
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
///
/// New 2026-09-05 with [`cancel_button_staged`]. Short, and it says the two
/// things a cancel has to say: the removal will not happen, and **the marks are
/// still there** — because taking the arming off is not taking the marks off,
/// and an operator who read this as *"never mind, that is dealt with"* would
/// have a marked document he believes is a clean one.
#[must_use]
pub fn staging_cancelled(marks: usize) -> String {
    format!(
        "The removal is called off — nothing will be removed when you save, and ordinary saves work again. The {marks} mark(s) are still on the document and still cover content that is still in the file."
    )
}

// ---------------------------------------------------------------------------
// ★ The STAGED phase — what the dialog says when it is reopened on a document
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
///
/// It deliberately does **not** repeat the report. The numbers were measured at
/// the moment of consent and the removal re-runs at the save over whatever the
/// document says then, so quoting them here would present a stale measurement
/// as a current one — which is the failure `crate::redact::StagedRedaction`'s
/// own doc comment calls a preview rather than a receipt.
#[must_use]
pub fn staged_body() -> &'static str {
    "The marked content has not been removed and the page has not changed. It comes out at the moment you use Save or Save As, which is the only way this document can be saved while the removal is set up. You can keep editing and undoing in the meantime, and you can call the removal off below — that leaves the marks in place and lets ordinary saves work again."
}

/// The control that un-stages a removal.
///
/// ★★★ It exists because **a stageable operation that cannot be un-staged is a
/// trap**, and the trap has teeth here: while a removal is armed the engine
/// refuses both ordinary save modes by name, so an operator who changed his
/// mind and had no way to say so could not save his document at all.
///
/// Worded as *"call off"* rather than *"cancel"*, because this window already
/// has a *Don't apply yet* control and two buttons whose labels both read as
/// *cancel* on one screen is how the wrong one gets pressed.
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
