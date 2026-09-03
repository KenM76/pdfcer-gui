//! # `text::panels::attachments` — every string the Attachments panel shows
//!
//! One area of the catalog described in [`crate::text`]'s header, covering
//! [`crate::panels::attachments`] and the three apply arms in
//! [`crate::app::actions::attachments`] that report what those verbs did.
//!
//! ## What this surface is about, in one paragraph
//!
//! A PDF may carry **whole files inside itself** — ISO 32000-1 §7.11.4.1
//! *embedded file streams*, reached either from the catalogue's
//! `/Names /EmbeddedFiles` name tree (document-level, belongs to the file) or
//! from a `/FileAttachment` annotation on one page (§12.5.6.15, pinned to a
//! rectangle and destroyed when that page is deleted). Neither kind is visible
//! anywhere on the page. That single fact decides most of the wording below:
//! **every sentence here is a disclosure**, because there is nothing on the
//! canvas an operator could have looked at instead.
//!
//! ## ★★★ The three sentences that are NOT optional
//!
//! Each one exists because `pdfcer-core` states an obligation in its own doc
//! comment and a shell that skipped it would be shipping a lie the operator
//! cannot detect:
//!
//! | sentence | why it must be said |
//! |---|---|
//! | [`removed`] | `detach_file`: *"This is NOT a redaction verb and must not be described as one … Shells are expected to say so rather than let 'delete' imply erasure."* Under the default incremental save (§7.5.6) the bytes are still in the file, recoverable from the previous revision. |
//! | [`may_be_encrypted`] | `AttachmentNotes::may_be_encrypted`: since PDF 1.5 an embedded file can be encrypted **in an otherwise unencrypted document** (`/EFF` naming a `DefEmbeddedFile` crypt filter, §7.6.5), so the intuitive guard is wrong *silently* — the filter chain runs, produces bytes, and those bytes are garbage that looks like a successful extraction. |
//! | [`name_was_changed`] | `sanitize_attachment_name`: the name in a document is attacker-controlled and unconstrained — `..\..\Windows\System32\evil.exe`, `invoice.pdf\0.exe`, `CON.txt` are all authorable — so pdfcer writes a different file name than the row shows, and *"pdfcer renamed this file"* with no reason is the sneaky behaviour rule 4 forbids. |
//!
//! ## ★ Why the size is worded as a measurement and never as a verdict
//!
//! `/Params /Size` is **optional** and §7.11.4 attaches no `shall` to it, so a
//! document whose declaration disagrees with its bytes is not thereby
//! non-conforming — `pdfcer-core` records this as ambiguity **EF-A2** and its
//! own `DeclaredSizeCheck::Disagrees` doc says to word it *"the document says
//! 999999 and pdfcer counted 10"*, not *"this document is broken"*. [`size`]
//! follows that to the letter, and [`DeclaredSizeCheck::is_contradicted`]'s
//! name — *contradicted*, not *invalid* — is the same decision one layer down.
//!
//! ## ★ Why the dates are printed exactly as the file wrote them
//!
//! `Attachment::created` and `Attachment::modified` are **raw and unparsed** by
//! design: `pdfcer-core` has no shared §7.9.4 date type yet and says outright
//! that inventing a private one *"would guarantee two parsers that disagree the
//! day a second caller wants dates."* So `D:20240117093000Z` is what a row
//! shows, and [`date_tooltip`] is where an operator finds out why — the same
//! decision, in the same words, that [`super::comments::comment_row_byline`]
//! made for `/M` on an annotation.
//!
//! [`DeclaredSizeCheck::is_contradicted`]: pdfcer_core::attachments::DeclaredSizeCheck::is_contradicted

use pdfcer_core::attachments::{AttachmentNotes, DeclaredSizeCheck, NameHazard};

// ---------------------------------------------------------------------------
// The listing
// ---------------------------------------------------------------------------

/// How many files this document carries, as the panel's first line.
///
/// Singular is spelled out rather than reached by a plural rule, matching
/// [`super::bookmarks_count`]'s shape: *"1 attached files"* is the tell that a
/// program is filling in a template.
#[must_use]
pub fn count(total: usize) -> String {
    if total == 1 {
        "1 attached file.".to_owned()
    } else {
        format!("{total} attached files.")
    }
}

/// Shown when the document carries nothing.
///
/// **Worded as a fact about the document, not as an absence of a feature.**
/// The overwhelming majority of PDFs have no attachments and are perfectly
/// ordinary; an operator reading this must not be left wondering whether pdfcer
/// failed to look.
#[must_use]
pub const fn empty() -> &'static str {
    "This document carries no attached files."
}

/// A row for an attachment nothing named.
///
/// `NameSource::None` is reachable — a filespec may carry no `/F`, `/UF`,
/// `/DOS`, `/Mac` or `/Unix`, and a page annotation has no name-tree key to
/// fall back on — and the row must still exist, because the operator can still
/// save the bytes out. A blank line where a name belongs reads as a rendering
/// fault.
#[must_use]
pub const fn unnamed() -> &'static str {
    "(unnamed)"
}

/// Where a document-level attachment lives.
///
/// The distinction this states is the one `pdfcer_core::attachments`' module
/// docs say *"bites hardest at save time and at page-delete time"*: this kind
/// belongs to the document and survives the deletion of every page.
#[must_use]
pub const fn where_document() -> &'static str {
    "Attached to the document"
}

/// Where a page-level attachment lives, and the consequence of that.
///
/// ★ The clause about page deletion is the whole reason this string is not
/// simply *"On page 3"*. A `/FileAttachment` annotation (§12.5.6.15) is
/// **destroyed when its page is deleted**, and this application can delete a
/// page from three different surfaces. An operator who has been told is one
/// who can decide; one who has not finds out from a file that used to have
/// their supplier's spreadsheet in it.
#[must_use]
pub fn where_page(page_number: usize) -> String {
    format!("On page {page_number} — deleting that page takes this file with it")
}

/// The media type the document claims for the payload.
///
/// ★★ *"claims"* is load-bearing and is not softened. `/Subtype` on an embedded
/// file stream is a **claim by the document about its own payload, never a
/// measurement** — `pdfcer-core` does not sniff the bytes, and `/text#2Fplain`
/// on a Windows executable is trivially authorable. A caller that presented
/// this as a safety signal would be turning an unverified assertion into an
/// assurance, which is exactly the shape of the mistake that gets somebody to
/// double-click.
#[must_use]
pub fn kind_claimed(mime: &str) -> String {
    format!("Type, as the document declares it: {mime}")
}

/// The size line for a row.
///
/// # ★ Four different sentences, because there are four different facts
///
/// Collapsing them would put a number on screen with no way to tell an
/// agreed measurement from an unchecked declaration — and it is the *third*
/// case that makes the collapse dishonest rather than merely lossy:
///
/// | state | what pdfcer actually knows |
/// |---|---|
/// | `NotDeclared` | the document said nothing. §7.11.4 makes `/Size` optional, so this is ordinary. |
/// | `NoStream` | there are no bytes at all — an external file reference (§7.11.3), legal and not extractable. |
/// | `Unverified` | a size was declared and the stream is **filtered**, so its raw byte count is not its decoded byte count. Comparing them would manufacture a false verdict in both directions. |
/// | `Agrees` / `Disagrees` | pdfcer counted. Only here is a comparison honest. |
///
/// The `Disagrees` wording states both numbers and passes no judgment; see
/// this module's header for why that is a requirement rather than a courtesy.
#[must_use]
pub fn size(declared: Option<u64>, check: DeclaredSizeCheck) -> String {
    match check {
        // ★ The bare figure, with no qualifying clause, and that is the whole
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
///
/// Printed verbatim. See the module header: `pdfcer-core` stores these raw
/// because it has no shared §7.9.4 date type, and a parser written here would
/// be a second one that disagrees with whichever is written next.
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
///
/// On hover rather than on the row, exactly as
/// [`super::comments::comment_row_modified_tooltip`] is and for its reason: it
/// answers a question most operators will never ask, and the ordinary value is
/// legible enough to compare two rows by.
#[must_use]
pub const fn date_tooltip() -> &'static str {
    "Shown exactly as the file wrote it. pdfcer does not reformat a date it has not parsed."
}

// ---------------------------------------------------------------------------
// What pdfcer could not vouch for — the per-row disclosures
// ---------------------------------------------------------------------------

/// The name shown is pdfcer's best reading of bytes it could not fully decode.
///
/// `Attachment::name_exact` is `false` when decoding needed at least one
/// U+FFFD substitution — an undefined PDFDocEncoding code, an odd trailing byte
/// after a UTF-16BE BOM, an unpaired surrogate. That is **pdfcer's own
/// lossiness**, and rule 4 requires disclosing it exactly as much as it
/// requires disclosing an inference about the document.
#[must_use]
pub const fn name_is_approximate() -> &'static str {
    "This name is approximate — some of its characters could not be decoded."
}

/// The document gave no filename at all, so the index key is standing in.
///
/// ★★ A name-tree key is **not** a filename and has no declared encoding.
/// Table 31 describes `/EmbeddedFiles` as mapping name strings to file
/// specifications and stops there — the sibling `/Renditions` row in the same
/// table *does* require Unicode, so the omission is deliberate — and §7.9.6
/// says outright that *"any encoding of the keys may be used as long as it is
/// self-consistent"*. Producers routinely mangle these with numeric suffixes
/// and portfolio folder prefixes, so a key shown as a name is a guess twice
/// over, and this sentence is how both are disclosed.
#[must_use]
pub const fn name_is_the_index_key() -> &'static str {
    "The document gave this file no name; what is shown is its index key."
}

/// There is a filespec but no bytes behind it.
///
/// **Not necessarily a defect.** §7.11.3 file specifications also describe
/// *external* files, which legitimately have nothing embedded — so this is
/// worded as a fact about what the row can do, not as damage.
#[must_use]
pub const fn no_bytes() -> &'static str {
    "This entry points at a file kept outside the PDF, so pdfcer has nothing to save out."
}

/// The document promised bytes and cannot produce them.
///
/// Distinct from [`no_bytes`], and the distinction is the whole reason both
/// exist: `AttachmentNotes::unresolvable_streams` is documented as *"always a
/// defect"* — an `/EF` entry that exists and does not resolve to a stream —
/// while `filespecs_without_stream` is ordinary. One sentence for both would
/// either call a legal document damaged or let real damage pass unremarked.
#[must_use]
pub const fn broken_stream() -> &'static str {
    "This attachment's bytes are missing from the file — the document points at them and they are not there."
}

/// The whole listing may be ciphertext.
///
/// See this module's header for why over-warning is the correct error here:
/// the flag is set from the presence of `/Encrypt` alone, which is cheap and
/// deliberately over-broad, and the failure it guards against is *silent* —
/// a successful-looking extraction of garbage.
#[must_use]
pub const fn may_be_encrypted() -> &'static str {
    "This document is encrypted, so anything saved out of it may be unreadable — pdfcer does not decrypt attachments yet."
}

/// Everything the listing had to skip, bound or degrade, as sentences.
///
/// # ★★ Why this is a function over the whole struct rather than a string per flag
///
/// Because the panel must show **all** of them, and the failure mode of a
/// string-per-flag catalog is a caller that renders four of the seven. The
/// disclosure obligation here is not per-flag; it is *"is this list
/// complete?"*, and that question has one answer assembled from the whole
/// struct. Written as a pure function so [`tests`] can hold it to that without
/// a `Ui`.
///
/// An all-default `AttachmentNotes` returns an **empty vector**, which is the
/// property the panel relies on to draw nothing: *"all-zero/false means the
/// listing is complete and everything parsed."*
///
/// ★ `page_tree_unwalkable` is reported even though the document-level list is
/// still complete, because the operator cannot tell the difference between
/// *"there are no page attachments"* and *"pdfcer could not go and look"* — and
/// those are the two answers that matter when a file has gone missing.
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
///
/// A helper rather than seven hand-written `if`s, because seven copies of a
/// plural rule is seven chances to ship *"1 entries"*, and that is the exact
/// tell this catalog's header calls out.
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
///
/// ★ *"optional"* is in the hint rather than in a sentence beside it, because
/// it is the answer to the only question the field raises and an operator who
/// reads it in the box has been answered before they wonder.
#[must_use]
pub const fn attach_description_hint() -> &'static str {
    "Description (optional)"
}

/// Why the description is worth typing, and why it can only be typed now.
///
/// ★★ The second half is a **capability disclosure**, not a nicety.
/// `EditSession::attach_file` takes the description at attach time and
/// `pdfcer-core` has no verb that edits one afterwards, so an operator who
/// leaves the box empty has made a decision they cannot revisit without
/// removing the file and attaching it again. R9 forbids drawing a control for
/// the edit that does not exist; it does not forbid saying so.
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

/// ★★★ **What attaching actually did**, said off-canvas because the page cannot
/// show it.
///
/// Three clauses, and each one is a thing the operator has no other way to
/// learn:
///
/// 1. **the file is embedded, and a copy** — the original is untouched, which
///    is the first thing anybody wonders and the thing that decides whether
///    they go and delete it;
/// 2. **it is not on any page** — a document-level attachment (§7.11.4.1
///    route 2) appears nowhere in the rendering, so an operator looking for a
///    visual confirmation will not find one and must not conclude the attach
///    failed;
/// 3. **the document has grown** — the bytes are now inside the PDF, and on a
///    large attachment that is the difference between a file that emails and
///    one that does not.
#[must_use]
pub fn attached(name: &str, bytes: u64) -> String {
    format!(
        "{name} is now embedded in this document ({}). It is a copy — the original file is untouched — and it appears on no page.",
        human_bytes(bytes)
    )
}

/// The one refusal this surface can provoke that the operator can understand.
///
/// # ★★ Why this refusal is surfaced and the other three are not
///
/// `attach_file` refuses four ways. Three of them —
/// `DocumentEncrypted`, the certification gate and
/// `ObjectCreationWouldExposeHiddenObjects` — are properties of the *document*
/// that every other authoring verb in this shell shares, and this shell's
/// settled answer for those is `super::super::apply::vector_edit`'s trace: they
/// are conditions an operator cannot fix from this panel, and wording them here
/// would put four sentences in the one status slot for states the Attachments
/// panel did not create.
///
/// `AttachmentTreeUnsupported` is different in kind, and that is the whole
/// argument for this string. It is **specific to this feature**, it is
/// **unreachable from any other surface**, and — the part that matters — the
/// press produces *nothing at all*: no row appears, no error appears, and an
/// operator has no way to distinguish it from a button that is broken.
///
/// It is worded as a limit of pdfcer rather than as a fault in the file, because
/// that is what it is: a `/Kids` name tree is entirely legal (§7.9.6), and the
/// engine's refusal is a refusal to risk *"a document whose EXISTING
/// attachments stop resolving"* by guessing at a `/Limits` repair.
#[must_use]
pub const fn attach_refused_multi_node_tree() -> &'static str {
    "pdfcer cannot add to this document's attachment index — it is stored in a form pdfcer would risk damaging. The files already attached are unharmed and can still be saved out."
}

/// The source file could not be read.
///
/// The detail is the operating system's own message, passed through: it names
/// the file and says whether it was a permission, a lock or a missing path,
/// and none of those is a distinction this catalog could redraw better.
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
///
/// ★ It names the **three objects** that go, because *"remove the row"* is what
/// a careless implementation would do and it is the worst possible outcome:
/// `detach_file`'s own docs say that removing only the tree entry leaves *"the
/// bytes in the file with nothing pointing at them: invisible to every reader,
/// still fully present on disk."* Saying what pdfcer does is how an operator can
/// tell this implementation from that one.
#[must_use]
pub const fn remove_tooltip() -> &'static str {
    "Remove this file from the document — the index entry, the file specification and the bytes, as one undoable step."
}

/// Why a page-level attachment has no Remove button here.
///
/// R9 says an absent capability renders nothing, and this is the sentence that
/// makes the absence legible rather than mysterious. It is not that pdfcer
/// cannot remove one; it is that a `/FileAttachment` is an **annotation**, is
/// listed in the Comments panel as one, and is removed as one —
/// `EditSession::detach_file` answers `AttachmentNotFound` for it by name,
/// precisely so a shell can say which of the two kinds the operator is looking
/// at.
#[must_use]
pub const fn remove_lives_with_the_note() -> &'static str {
    "This one is a note on a page. Remove it from the page, as a comment, rather than from here."
}

/// ★★★ **What removing actually did** — including the part that is not what
/// the word suggests.
///
/// # This sentence is required by `pdfcer-core`, in its own words
///
/// > *"This is NOT a redaction verb and must not be described as one. If the
/// > attachment was sensitive, the operator needs a full rewrite … Shells are
/// > expected to say so rather than let 'delete' imply erasure."*
///
/// Under the default incremental save (§7.5.6) **every prior revision is still
/// in the file by design** — that is what makes existing signatures survive —
/// so the attachment's bytes remain recoverable from the earlier revision.
/// Only a full rewrite drops superseded revisions.
///
/// ★ And the second sentence **names the command that does it**. A disclosure
/// that states a hazard and leaves the operator to find the remedy has done
/// half the job; `file.save_compacted` is the full rewrite, it is on File ▸
/// Save, and it is one control away.
#[must_use]
pub fn removed(name: &str) -> String {
    format!(
        // ★ The command is named in words rather than with the ribbon's ▸
        // separator: `crate::icons::glyphs` records that U+25B8 is a codepoint
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
///
/// *"Save a copy"* rather than *"Extract"*: extraction is the engine's word for
/// decoding a stream, and an operator's word for what this does is saving a
/// copy. Nothing is taken out of the document.
#[must_use]
pub const fn save_button() -> &'static str {
    "Save a copy…"
}

/// What pressing it will do, on hover.
///
/// ★ The second clause is the disclosure the first invites: the bytes came from
/// inside a file that arrived from somewhere, and `pdfcer-core`'s own module
/// docs say it *"does not execute, open, or interpret them, and neither should
/// a caller without its own gate."* pdfcer writes the file and stops; opening it
/// is the operator's decision, and they should make it knowing that the
/// document's declared type is a claim rather than a check.
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

/// ★★★ **pdfcer used a different name than the row shows, and here is why.**
///
/// # Why this is a required disclosure and not a nicety
///
/// `sanitize_attachment_name`'s own docs record the design choice this string
/// completes. pdfcer reports the **raw** name in the listing, because *"a
/// forensic reader that quietly repairs its input is not a reader"* — the
/// operator investigating a suspicious file must see the traversal that made it
/// suspicious. And pdfcer refuses to *use* that raw name on a filesystem,
/// because the failure mode is silent, remote and severe.
///
/// Between those two correct decisions sits a gap: the row says one thing and
/// the file on disk is called another. This sentence is the bridge, and
/// `SafeName::hazards` exists — sorted and deduplicated, *"so a message can
/// list them deterministically"* — for exactly this call.
///
/// ★ The hazard names are translated to plain English rather than printed. An
/// operator seeing *"ParentTraversal"* has been shown a Rust identifier; one
/// seeing *"it tried to climb out of the folder you chose"* has been told what
/// happened to them.
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
///
/// `NameHazard` is `#[non_exhaustive]`, so the catch-all is required and is
/// deliberately the weakest claim available: *"it was not safe to use as a file
/// name"* is true of any hazard a later engine adds, where guessing at a
/// specific cause would not be.
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
///
/// The detail is `AttachmentError`'s own `Display`, which distinguishes the
/// four causes the engine went to the trouble of separating — an external
/// reference, a missing stream, an unservable span, and a filter chain that
/// failed or blew the decompression-bomb ceiling. Re-wording them here would
/// be a second vocabulary for facts the engine already states precisely.
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
///
/// Reachable, and by the ordinary route rather than an exotic one: the queue
/// drains **after** the frame, so an undo or a second removal raised earlier in
/// the same frame can take the row away before this action is applied.
/// Declining with a sentence beats declining in silence, and both beat acting
/// on whatever moved into its place.
#[must_use]
pub const fn gone() -> &'static str {
    "That attachment is no longer in this document, so nothing was saved."
}

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

/// A byte count for a listing, in the file manager's units.
///
/// Delegates to [`super::byte_size`], which carries the argument for base-1024
/// arithmetic with the colloquial `KB`/`MB` labels: this figure is read by
/// operators comparing it against what Explorer tells them about the file they
/// just attached, and matching that is worth more here than matching IEC.
///
/// The cast is saturating rather than lossy: a `u64` byte count larger than
/// `usize` cannot be produced by a file this application can read, and
/// saturating is the answer that stays a number instead of wrapping to a small
/// one on a 32-bit build.
fn human_bytes(bytes: u64) -> String {
    super::byte_size(usize::try_from(bytes).unwrap_or(usize::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panels::objects::test_support::engine_fixture;

    /// **A complete listing says nothing**, which is what lets the panel draw
    /// no disclosure block at all.
    ///
    /// `AttachmentNotes`' own doc states the contract — *"all-zero/false means
    /// the listing is complete and everything parsed"* — and a
    /// [`listing_notes`] that returned a reassurance instead of nothing would
    /// put a permanent sentence above every ordinary document's list.
    #[test]
    fn an_undamaged_listing_discloses_nothing() {
        assert!(listing_notes(&AttachmentNotes::default()).is_empty());
    }

    /// ★★ **A damaged document's listing speaks, and says more than one thing.**
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
    ///
    /// The companion to [`an_undamaged_listing_discloses_nothing`], and the one
    /// that would catch a flag pdfcer sets over-eagerly: a `Default` is a value
    /// nobody produced, and a listing that quietly reported *"pdfcer stopped
    /// reading early"* about every well-formed document would still pass that
    /// test.
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
    ///
    /// The tell this catalog's header names, checked on the panel's own first
    /// line and on the helper every counted note goes through — which is the
    /// point of that helper existing: seven hand-written plural rules would be
    /// seven chances to ship *"1 entries"*.
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

    /// ★★ **The removal sentence says the bytes survive, and names the remedy.**
    ///
    /// `detach_file`'s doc comment makes this a shell obligation in as many
    /// words, and the failure mode it guards against is an operator who removed
    /// a sensitive attachment, saved, and believes it is gone. Both halves are
    /// pinned: the fact, and the command that acts on it — a warning with no
    /// route out is half a disclosure.
    #[test]
    fn the_removal_sentence_does_not_let_remove_imply_erasure() {
        let said = removed("quote.xlsx");
        assert!(said.contains("quote.xlsx"), "{said}");
        assert!(said.contains("recoverable"), "{said}");
        assert!(said.contains("compacted"), "{said}");
    }

    /// **A size disagreement states both numbers and accuses nobody.**
    ///
    /// §7.11.4 attaches no `shall` to `/Size` (ambiguity EF-A2), so a
    /// disagreement is a measurement rather than a verdict. The words this
    /// forbids are the ones that would turn one into the other.
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

    /// ★ **An unverified size is not reported as an agreement.**
    ///
    /// The case `DeclaredSizeCheck` exists for: the stream is filtered, so its
    /// raw byte count is not its decoded byte count, and printing the
    /// declaration bare would present an unchecked claim as a measurement.
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

    /// ★★ **A sanitised name names both spellings and says what was wrong.**
    ///
    /// The gap this bridges is structural: the listing shows the raw name
    /// because a reader must not repair its evidence, and the filesystem gets
    /// the safe one because the failure mode is a file written outside the
    /// destination. Without this sentence the operator sees two different names
    /// and is told nothing.
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
    ///
    /// Reachable: `SafeName::changed` is true whenever the value differs from
    /// the input, and a future sanitiser step could change one without pushing
    /// a hazard. The failure this pins is the dangling em dash — a sentence
    /// that ends in punctuation waiting for a clause that never came.
    #[test]
    fn a_reasonless_rename_does_not_dangle() {
        let said = name_was_changed("a", "b", &[]);
        assert!(said.ends_with('.'), "{said}");
        assert!(!said.contains("—"), "{said}");
    }

    /// ★ **Every disclosure is a sentence and every label is not.**
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
    ///
    /// A `/FileAttachment` is destroyed with its page, this application can
    /// delete a page from three surfaces, and *"On page 3"* alone would leave
    /// an operator to discover that from a file that has lost something.
    #[test]
    fn a_page_row_warns_that_deleting_the_page_takes_the_file() {
        let said = where_page(3);
        assert!(said.contains('3'), "{said}");
        assert!(said.contains("deleting that page"), "{said}");
    }
}
