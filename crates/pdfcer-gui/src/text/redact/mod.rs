//! # `text::redact` — every word the redaction surface says
//!
//! Consumed by [`crate::panels::redact`] (mark and review) and
//! [`crate::dialogs::redact`] (the apply transaction and its report).
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/redact/mod.md`.

// ---------------------------------------------------------------------------
// The panel — marking and review
// ---------------------------------------------------------------------------

/// The panel's own heading.
#[must_use]
pub fn panel_title() -> &'static str {
    "Redact"
}

/// The sentence at the top of the panel.
#[must_use]
pub fn panel_intro() -> &'static str {
    // 2026-09-04: *"writes a file"* rather than *"writes a NEW file"*. The
    // apply dialog now offers to replace the open document as well, so the old
    // wording promised a property the operator can switch off two clicks later
    // — and this sentence is read before that dialog is ever opened, which
    // makes it the worse place to be specific about the destination. The
    // permanence, which does not vary, stays exactly as emphatic as it was.
    //
    // 2026-09-05: *"produces a file"* rather than *"writes a file"*, and
    // *"once it is written"* on the permanence clause. The default destination
    // no longer writes at the moment of applying — it arms a save
    // (`Pass 250.2`) — so a sentence promising a file at that click is a
    // promise the operator can watch not happen. The permanence is unchanged
    // and stays exactly as emphatic: what varies is *when*, never *whether*.
    "Mark content, then apply to permanently remove it. Marking is reversible and changes nothing in the file; applying produces a file with the marked content gone, and once it is written that cannot be undone."
}

/// The heading over the marking controls.
#[must_use]
pub fn mark_heading() -> &'static str {
    "Mark content for removal"
}

/// The control that marks the whole of the page on screen.
#[must_use]
pub fn mark_whole_page() -> &'static str {
    "Mark whole page"
}

/// Its tooltip.
///
/// Names the reversibility explicitly, since marking an entire page in one
/// click is easy to do by accident.
#[must_use]
pub fn mark_whole_page_tooltip() -> &'static str {
    "Mark this entire page for redaction. Nothing is removed until you apply, and you can take the mark off again from the list below or with Undo."
}

/// Why *Mark whole page* is greyed — `OPERATOR_REQUESTS.md` O77.
#[must_use]
pub fn mark_whole_page_disabled() -> &'static str {
    "This document has no pages to mark."
}

/// The label beside the search field.
#[must_use]
pub fn search_label() -> &'static str {
    "Text:"
}

/// The control that searches and marks every hit.
#[must_use]
pub fn search_button() -> &'static str {
    "Find & mark"
}

/// Its tooltip, in both states.
///
/// The disabled form explains what would enable it, rather than leaving a dead
/// control unexplained.
#[must_use]
pub fn search_button_tooltip(has_query: bool) -> &'static str {
    if has_query {
        "Finds this text on every page and adds a mark over each match, for you to review before applying."
    } else {
        "Type the text you want marked first — there is nothing to search for."
    }
}

/// The label on the literal-versus-pattern switch.
#[must_use]
pub fn match_mode_label() -> &'static str {
    "Match:"
}

/// The literal half of that switch.
#[must_use]
pub fn match_literal() -> &'static str {
    "Exact text"
}

/// Its tooltip.
#[must_use]
pub fn match_literal_tooltip() -> &'static str {
    "Find this text exactly as typed, ignoring upper and lower case."
}

/// The pattern half of that switch.
#[must_use]
pub fn match_pattern() -> &'static str {
    "Pattern"
}

/// Its tooltip.
#[must_use]
pub fn match_pattern_tooltip() -> &'static str {
    "Find every run SHAPED like what you type — for example ###-##-#### marks every social-security number on every page in one action. Use this when you know the shape but not the values."
}

/// **The hint under the search field, in whichever mode is selected.**
#[must_use]
pub fn search_hint(pattern: bool) -> &'static str {
    if pattern {
        "# matches any digit, ? matches any single character, everything else is literal. So ###-###-#### finds phone numbers and A?-#### finds A1-2345. Marks are added for you to review before applying. This can only find text pdfcer can extract — on a scanned page with no text layer it will find nothing, which is not the same as there being nothing sensitive there."
    } else {
        "Finds this exact text on every page and adds a mark over each match, for you to review before applying. It can only find text pdfcer can extract — on a scanned page with no text layer it will find nothing, which is not the same as there being nothing sensitive there."
    }
}

/// **The census line above the mark list.**
#[must_use]
pub fn marks_count(count: usize) -> String {
    if count == 0 {
        "No redaction marks in this document. Nothing is marked, and nothing has been removed."
            .to_owned()
    } else {
        format!(
            "{count} pending redaction mark(s) — the content underneath them is STILL IN THIS FILE until you apply."
        )
    }
}

/// One row in the mark list: which page, and how big the marked region is.
#[must_use]
pub fn mark_row(page_number: usize, size: Option<(f64, f64)>) -> String {
    match size {
        Some((w, h)) => format!("Page {page_number} — region {w:.0} × {h:.0} pt"),
        None => format!("Page {page_number} — region (no stored size)"),
    }
}

/// The tooltip on a mark row.
///
/// Says what clicking it does; the row is a navigation control, not a
/// selection.
#[must_use]
pub fn mark_row_tooltip() -> &'static str {
    "Go to this page so you can see what the mark covers."
}

/// The control that takes one mark off.
#[must_use]
pub fn mark_remove() -> &'static str {
    "Remove"
}

/// Its tooltip.
///
/// The second half is the whole point: removing a MARK is not undoing a
/// redaction, because no redaction happened.
#[must_use]
pub fn mark_remove_tooltip() -> &'static str {
    "Remove this mark. It was never applied, so nothing in the document changes and nothing is recovered — the content it covers was there all along."
}

/// The control that opens the apply report.
///
/// The label promises a **report**, not an apply, because the click that opens
/// it must not feel like the click that commits.
#[must_use]
pub fn review_and_apply() -> &'static str {
    "Review & apply redactions…"
}

/// Its tooltip, in both states.
#[must_use]
pub fn review_and_apply_tooltip(can_apply: bool) -> &'static str {
    if can_apply {
        "Opens a report of exactly what will be permanently removed, and of anything pdfcer could not remove. Nothing is written until you confirm there."
    } else {
        "Mark at least one region first — there is nothing to apply."
    }
}

// ---------------------------------------------------------------------------
// The apply dialog — the report, the acknowledgements, the write
// ---------------------------------------------------------------------------

/// The dialog's title.
#[must_use]
pub fn apply_title() -> &'static str {
    "Apply redactions — permanent removal"
}

/// The heading above the report body.
#[must_use]
pub fn report_heading() -> &'static str {
    "What applying will do"
}

/// **The permanence statement — the first thing in the dialog body, never
/// abbreviated, never softened.**
///
/// It says what this operation does *in this shell*, which is not what a
/// generic redaction warning would say. Apply does not mutate the open
/// document: it writes a **file**, and leaves the session exactly as it is,
/// marks and all (`crate::redact` §3) — whichever destination was chosen, and
/// including the destination that overwrites the file the session was loaded
/// from. A sentence about "you cannot undo this once you save" would describe a
/// save that never happens.
///
/// The clause about the open document is not reassurance filler. It is the
/// answer to the question an operator asks immediately afterwards — *"so what
/// happened to the thing I was working on?"* — and getting it wrong in either
/// direction is expensive: believing the open document was redacted is the
/// worse error, and believing nothing happened at all is the one that makes
/// people press the button twice.
#[must_use]
/// 2026-09-04: it takes the destination, because two of its three claims
/// stopped being unconditionally true when the operator was given the choice of
/// replacing the open file. *"Writes a NEW file"* and *"the document you have
/// open is left exactly as it is now"* are both false on a replace, and this
/// sentence is drawn in the warning role at the top of the report — the one
/// sentence a reader who takes in nothing else takes in. A false claim there is
/// worse than no claim anywhere.
pub fn permanence_statement(replacing_the_open_file: bool) -> &'static str {
    if replacing_the_open_file {
        "Applying REPLACES the file you have open with a copy that has the marked content permanently removed. It is a full rewrite, not an edit: nothing in that file can bring the removed content back — not Undo, not a previous revision, not any recovery tool. The file you are replacing is the last copy of that content, so it will not exist anywhere afterwards."
    } else {
        "Applying writes a NEW file with the marked content permanently removed. It is a full rewrite, not an edit: nothing in that file can bring the removed content back — not Undo, not a previous revision, not any recovery tool. The document you have open is left exactly as it is now, marks and all."
    }
}

/// The heading for the affirmative half of the report.
#[must_use]
pub fn will_remove_heading() -> &'static str {
    "Will be permanently removed:"
}

/// **The removal summary — the measured centrepiece of the report.**
#[must_use]
pub fn removal_summary(regions: u64, pages: usize, glyphs: u64, streams: u64) -> String {
    format!(
        "{regions} marked region(s) across {pages} page(s): {glyphs} character(s) deleted from {streams} page content stream(s), and the marks themselves removed."
    )
}

/// The annotation line, shown only when the count is non-zero.
#[must_use]
pub fn annotations_removed(count: u64) -> String {
    format!(
        "{count} annotation object(s) will be removed in total: the redaction marks themselves, plus any annotation that overlapped a marked region — an annotation sitting over redacted content can carry a copy of it in its own appearance or text."
    )
}

/// The document-information line, shown only when the count is non-zero.
#[must_use]
pub fn info_scrubbed(count: u64) -> String {
    format!(
        "{count} document-information entr(y/ies) contained the redacted text and will be scrubbed of it."
    )
}

/// The object-stream line, shown only when the count is non-zero.
///
/// ISO 32000-1 §7.5.7: a removed object must not survive compressed inside a
/// container.
#[must_use]
pub fn containers_decomposed(containers: u64, promoted: u64) -> String {
    format!(
        "{containers} compressed object container(s) will be taken apart ({promoted} object(s) moved out of them), so no removed object can survive inside one."
    )
}

/// **The single-revision line — engine rule R35 in operator language.**
#[must_use]
pub fn single_revision_note() -> &'static str {
    // 2026-09-04: *"the file that is written"* rather than *"the new file"*,
    // because it may not be a new one — the operator can now choose to replace
    // the document he opened, and on that path this sentence carries MORE
    // weight rather than less: replacing a file that had five earlier revisions
    // with a single-revision rewrite is precisely the property that makes the
    // replacement safe, and is the opposite of what `file.save` does.
    "The file that is written will be a single revision. Any earlier revision of this document — which would still hold the un-redacted content — is not carried into it."
}

/// **The verification line — the ONLY place in this catalog permitted to
/// use the word "verified".**
#[must_use]
pub fn verified_line(strings_checked: usize) -> String {
    format!(
        "Verified: pdfcer searched the finished file for all {strings_checked} distinct piece(s) of removed text and found none of them — not in the page content, not in any other stream, not in the raw bytes."
    )
}

/// The verification line's honest companion when some removed strings were too
/// short for a whole-file byte search to mean anything.
#[must_use]
pub fn verification_limit_line(too_short: usize) -> String {
    format!(
        "{too_short} removed piece(s) were too short (under 4 characters) for any search to say anything useful — a single letter or digit is on every page of every document — so pdfcer could not verify those. The removal itself is still reported above."
    )
}

/// **The mark covers a raster image, and those pixels will be DESTROYED** —
/// `OPERATOR_REQUESTS.md` O103.
#[must_use]
pub fn mark_covers_image(images: usize) -> String {
    format!(
        "This region covers part of {images} image(s) — those pixels will be destroyed, not hidden. The mark is authored either way."
    )
}

/// The heading for the residual section — the part that makes the feature
/// honest.
#[must_use]
pub fn residual_heading() -> &'static str {
    "⚠  Read before continuing — the following will still be in the saved file:"
}

// ---------------------------------------------------------------------------
// The carriers — every place a copy of the removed text can hide
//
//
// `pub use` rather than a `carriers::` path at every call site, so the split is
// invisible to consumers and the catalog keeps one flat namespace.
// ---------------------------------------------------------------------------
mod carriers;
pub use carriers::{
    carrier_name, checked_clean_line, engine_notes_heading, engine_notes_lead, left_by_choice_line,
    residual_carrier_line, residual_sweep_line, sweep_scrubbed_line,
};

/// **One residual line for a removed string that still occurs somewhere in
/// the saved file while occurring in nothing the document draws.**
#[must_use]
pub fn raw_residual_line(text: &str, site: crate::redact::ResidualSite) -> String {
    use crate::redact::ResidualSite as S;
    // The drawn-content hit has its OWN sentence, because the shared one
    // below opens with "no longer appears in anything this document draws" —
    // the one thing that is false here. The operator's words, 2026-09-09:
    // *"the way the error is worded it sounds like it found matching text
    // somewhere else in the document — which it very well could since I
    // didn't select it or want it redacted."* So this sentence says where,
    // says it is probably his unselected text, and tells him what ticking the
    // box means under each reading.
    if site == S::DrawnContent {
        return format!(
            "⚠  The removed text “{text}” also appears in this document's drawn content OUTSIDE the area you marked — most likely another occurrence of the same words that you did not select. pdfcer cannot tell that apart from removed text left behind. If it is text you did not mean to remove, this is not a leak and you can continue; if you meant to remove every occurrence, cancel and mark those too."
        );
    }
    let place = match site {
        S::DrawnContent => unreachable!("answered above"),
        S::FontProgram => {
            "inside an embedded font program — the part of a font file that holds its own name, its copyright and the English descriptions of its lettering features"
        }
        S::ImageSamples => "inside the pixel data of an image",
        S::ObjectContainer => {
            "inside a compressed object container, which keeps its own copy of objects that were moved out of it"
        }
        S::Attachment => {
            "inside a file attached to this document — an attachment is a separate document and redaction does not reach into it"
        }
        S::Metadata => "inside a metadata stream",
        S::OtherStream => "inside a stream this build does not recognise",
        S::RawBytes => "in the file's raw bytes, outside every stream pdfcer could decode",
    };
    format!(
        "⚠  The removed text “{text}” no longer appears in anything this document draws, but that same byte sequence still occurs {place}. It may be an unrelated coincidence, or a copy in a carrier pdfcer does not recognise — pdfcer cannot tell which, so it is reported rather than claimed removed."
    )
}

/// **What happened to the raster images under the regions** — `pdfcer-core`
/// v0.26.0, `Pass 245.0`.
#[must_use]
pub fn images_destroyed(cleared: u64, removed: u64, over_covered: u64) -> String {
    let mut line = String::new();
    if removed > 0 {
        line.push_str(&format!(
            "{removed} image(s) were covered completely and have been removed from the file. "
        ));
    }
    if cleared > 0 {
        line.push_str(&format!(
            "{cleared} image(s) had the covered pixels destroyed and overwritten — not hidden, and not recoverable. "
        ));
    }
    if over_covered > 0 {
        line.push_str(&format!(
            "{over_covered} of them sit at an angle, so a rectangle slightly larger than the one you drew was cleared."
        ));
    }
    line.trim_end().to_owned()
}

/// **A shared image was copied so the other pages keep theirs** — the
/// disclosure the engine asks for when one image object is painted in more than
/// one place.
#[must_use]
pub fn images_shared_copied(count: u64) -> String {
    format!(
        "{count} of those image(s) are also used elsewhere in the document. Only the marked placement was changed — the same picture is still on the other pages, because you did not mark those."
    )
}

/// **Marks that were left in the document unapplied** — `RedactionReport::marks_retained`.
#[must_use]
pub fn marks_retained_line(count: u64) -> String {
    format!(
        "⚠  {count} region(s) were NOT redacted — the image under them could not be decoded, so pdfcer left the mark in place rather than pretend. What you marked there is still in the saved file. The unapplied mark is still on the page, so you can find it again."
    )
}

/// **Vector geometry crossing a region that could not be cut.**
#[must_use]
pub fn vector_paths_residual_line(count: u64) -> String {
    format!(
        "⚠  {count} drawn line(s) or shape(s) cross a redacted region and could not be cut, so their geometry is still in the saved file underneath the black box. On a drawing that outline can be as identifying as the text was."
    )
}

/// **The clip whose outline had to be kept after its ink was cut.**
#[must_use]
pub fn vector_clips_kept_line(count: u64) -> String {
    format!(
        "⚠  {count} shape(s) crossing a redacted region were also being used to crop what is drawn after them, so pdfcer removed their ink but had to keep their outline — changing it would have cropped the rest of the page. Nothing of them is visible; the outline is still in the file."
    )
}

/// **The drawn geometry that was cut out of the regions** — `pdfcer-core`
/// v0.27.0.
#[must_use]
pub fn vector_paths_cut_line(cut: u64, dropped: u64) -> String {
    if dropped > 0 {
        format!(
            "{cut} drawn line(s) or shape(s) crossing a region were cut at its edge so nothing of them is left inside it — {dropped} of those lay entirely within a region and were deleted."
        )
    } else {
        format!(
            "{cut} drawn line(s) or shape(s) crossing a region were cut at its edge, so nothing of them is left inside it."
        )
    }
}

/// One residual line when materialising the operator's unsaved edits had to
/// promote objects out of an object stream (engine rule R38).
#[must_use]
pub fn promotion_line(count: usize) -> String {
    format!(
        "⚠  {count} object(s) had to be moved out of a compressed container in order to write your unsaved edits, and the container keeps its own copy of their previous value. Page content is never stored that way, so this cannot hold redacted text — but it is a leftover of your edits and is reported rather than passed over."
    )
}

/// The scope reminder — what redaction does **not** touch.
///
/// Named so an operator does not read "redacted" as "sanitised".
#[must_use]
pub fn scope_reminder() -> &'static str {
    "This removes what your marks cover. It does not sweep the document for unrelated hidden data: metadata history, embedded files, scripts or hidden layers that no mark touches are not part of this operation."
}

/// **The extra acknowledgement, shown ONLY when the report has a residual
/// section.**
#[must_use]
pub fn residual_acknowledgement_checkbox() -> &'static str {
    "I have read the items above, I understand they will NOT be removed, and I still want to apply the redactions that can be completed."
}

/// The mandatory confirmation.
///
/// Its wording targets the exact misunderstanding this feature exists to
/// prevent.
#[must_use]
pub fn confirm_checkbox() -> &'static str {
    "I understand this permanently removes the underlying content, not just the visible marks."
}

// ---------------------------------------------------------------------------
// The destination — where the redacted document goes
//
// ---------------------------------------------------------------------------
// The text itself -- the half of the engine's report this shell only ever
// grepped
//
// `pub use` rather than a `removed::` path at every call site, matching the
// two splits above, so the catalog keeps one flat namespace.
// ---------------------------------------------------------------------------
mod removed;
pub use removed::{
    MAX_CHARS, MAX_ENTRIES, removed_text_entry, removed_text_heading, removed_text_lead,
    removed_text_more, removed_text_none, removed_text_undecodable,
};

mod destination;
pub use destination::{
    cancel_button_staged, cancel_button_staged_tooltip, confirm_button_into_document,
    confirm_button_into_document_now, confirm_button_replace, destination_heading,
    destination_new_file, destination_new_file_tooltip, destination_open_document,
    destination_open_document_now, destination_open_document_now_tooltip,
    destination_open_document_tooltip, destination_replace, destination_replace_tooltip,
    overwrite_acknowledgement_checkbox, permanence_statement_deferred, permanence_statement_now,
    removal_happens_at_save, saved_applying_redaction, staged_body, staged_heading,
    staged_into_document, staging_cancelled,
};

/// **The confirm control. The label IS the consequence** — never "OK", never
/// "Yes", never "Apply" alone.
#[must_use]
pub fn confirm_button() -> &'static str {
    "Permanently remove & save as…"
}

/// Why *Permanently remove & save as…* is greyed —
/// `OPERATOR_REQUESTS.md` O77's sweep.
#[must_use]
pub fn confirm_disabled(
    permanence_outstanding: bool,
    residuals_outstanding: bool,
    overwrite_outstanding: bool,
) -> &'static str {
    match (
        permanence_outstanding,
        residuals_outstanding,
        overwrite_outstanding,
    ) {
        (false, false, false) => "Ready.",
        (true, false, false) => "Tick the box above to confirm you have read what will be removed.",
        (false, true, false) => {
            "Tick the box above to confirm you have read what pdfcer could not prove it removed."
        }
        (false, false, true) => {
            "Tick the box above to confirm you understand the file you have open will be replaced."
        }
        (true, true, false) => {
            "Tick both boxes above to confirm you have read what will be removed."
        }
        (true, false, true) => {
            "Tick both boxes above — one confirms what will be removed, the other that the file you have open will be replaced."
        }
        (false, true, true) => {
            "Tick both boxes above — one confirms what pdfcer could not prove it removed, the other that the file you have open will be replaced."
        }
        (true, true, true) => {
            "Tick all three boxes above: what will be removed, what pdfcer could not prove it removed, and that the file you have open will be replaced."
        }
    }
}

/// The control that closes the dialog without applying.
///
/// Phrased as a deferral rather than a refusal, because cancelling here loses
/// nothing — the marks survive.
#[must_use]
pub fn cancel_button() -> &'static str {
    "Don't apply yet"
}

/// **The no-shortcut disclosure, in the dialog's footer.**
#[must_use]
pub fn no_shortcut_note() -> &'static str {
    "There is deliberately no keyboard shortcut for this button. It is the one action in pdfcer that ends in a change nothing can undo, so it takes a deliberate click."
}

/// The title on the system file-save dialog.
#[must_use]
pub fn save_dialog_title() -> &'static str {
    "Save redacted copy"
}

/// **The suffix appended to the original file's stem to suggest a name.**
#[must_use]
pub fn suggested_suffix() -> &'static str {
    "-redacted"
}

// ---------------------------------------------------------------------------
// Outcomes
// ---------------------------------------------------------------------------

/// The line shown once a clean redaction is on disk.
///
/// "Verified" is earned here — the absence proof ran on these exact bytes, and
/// ran again between the buffer and the write. The last clause corrects the
/// learned Undo expectation rather than leaving it to be assumed (rule 3).
#[must_use]
/// 2026-09-04: `replaced` is what the last clause turns on. *"The document
/// you still have open is unchanged"* is a reassurance when a copy was written
/// and a **falsehood** when the open file was the one replaced — and on that
/// path the operator is owed the opposite fact, which is stranger than it
/// sounds and which nothing else on screen would tell them: the window in front
/// of them still shows the marks and the content, because the session was not
/// touched, while the file those bytes came from no longer contains either.
pub fn applied_clean(file_name: &str, regions: u64, pages: usize, replaced: bool) -> String {
    if replaced {
        format!(
            "Redacted — {file_name} has been replaced, {regions} region(s) across {pages} page(s) removed, and verified absent from it. That file cannot be un-redacted. ⚠  The window you are looking at still shows the marks and the content underneath them, because this destination writes a file and leaves the document you have open exactly as it was — close it and open {file_name} again to see what is now in the file."
        )
    } else {
        format!(
            "Redacted and saved to {file_name} — {regions} region(s) across {pages} page(s) removed, and verified absent from the saved file. That file cannot be un-redacted; the document you still have open is unchanged."
        )
    }
}

/// The line shown once a redaction that had acknowledged residuals is on disk.
#[must_use]
pub fn applied_with_residuals(
    file_name: &str,
    regions: u64,
    residuals: usize,
    replaced: bool,
) -> String {
    if replaced {
        format!(
            "⚠  Redacted — {file_name} has been replaced and {regions} region(s) removed, but {residuals} item(s) could NOT be removed and are still in it. Do not treat it as fully redacted; see the report you acknowledged for what and why. The window you are looking at still shows the marks — close it and open {file_name} again to see what is now in the file."
        )
    } else {
        format!(
            "⚠  Redacted and saved to {file_name} — {regions} region(s) removed, but {residuals} item(s) could NOT be removed and are still in that file. Do not treat it as fully redacted; see the report you acknowledged for what and why."
        )
    }
}

/// The sentence for a refusal that happened before anything was written.
#[must_use]
pub fn refusal_message(refusal: &crate::redact::RedactApplyRefusal) -> String {
    use crate::redact::RedactApplyRefusal as R;
    match refusal {
        R::NothingToApply => "Nothing to apply — this document has no redaction marks.".to_owned(),
        // **The ask SHIPPED, and this arm was rewritten rather than
        // deleted** — 2026-09-11. The note it replaces ended *"delete this arm
        // when it ships"*, and following that literally would have been wrong.
        //
        // # What the old arm said, and what made it stop being true
        //
        // It was written on 2026-09-09 against the operator's own `SW41177
        // MATERIAL REQUIREMENTS.pdf` (Excel for Microsoft 365 writes
        // hybrid-reference files, §7.5.8.4). At that time the engine refused a
        // full rewrite of **any** hybrid file, so the sentence said so: *"this
        // file was saved in PDF's hybrid-reference form... pdfcer cannot yet
        // rewrite a hybrid file from scratch"*.
        //
        // `Pass 281.0` narrowed the refusal to a hybrid whose `/XRefStm`
        // **does not parse** — the file says it hides objects and pdfcer
        // cannot tell which, so either partition would be a guess. Ordinary
        // hybrid files, including his, now redact.
        //
        // So the old sentence became the failure mode it was written to
        // fix, with the subject moved one step: it named the file's FORM as
        // the obstacle when the obstacle is a specific damaged structure
        // inside it, and it told an operator whose file redacts fine that a
        // whole class of file does not. A sentence that survives the fix it
        // asked for is worse than the one it replaced, because the reason to
        // doubt it has been filed as closed.
        //
        // The arm SURVIVES because the concern that earned it survives. The
        // engine's own remedy for this error is *"use incremental save"*, and
        // a redaction is the one operation forbidden to take it (R35: an
        // incremental save leaves the un-redacted bytes in a prior revision).
        // An operator who reads the writer's advice and follows it produces
        // exactly the file the redaction existed to prevent. That is worth a
        // sentence of our own no matter how narrow the cause becomes.
        //
        // Selected on the VARIANT now, not on the words. See
        // `RedactApplyRefusal::broken_xref_stream`.
        R::FullRewriteUnavailable {
            reason,
            broken_xref_stream: true,
        } => format!(
            "Redaction refused — this file hides some of its objects in a compressed table (PDF calls this hybrid-reference form; some Microsoft Office exports use it), and that table is damaged, so pdfcer cannot tell which objects it hides. Rewriting the file from scratch is what a redaction needs, and pdfcer will not guess at half a file. Nothing was written. Do NOT save it the ordinary way and assume the marks took — an incremental save would leave the un-redacted content in the file's previous revision, where anyone could recover it. The fix that works today: print the sheet to a new PDF from another program, then redact that copy. The writer's reason: {reason}"
        ),
        R::FullRewriteUnavailable {
            reason,
            broken_xref_stream: false,
        } => format!(
            "Redaction refused — this document cannot be rewritten in full, and nothing was written. Applying a redaction requires rewriting the entire file as one revision: an incremental save would leave the un-redacted content sitting in the file's previous revision, where anyone could recover it, so pdfcer will not fall back to one. The writer's reason: {reason}"
        ),
        R::MaterialisedDocumentUnreadable { reason } => format!(
            "Redaction refused — pdfcer rewrote your unsaved edits but could not read the result back, so it could not apply the redactions to them. Nothing was written. This is a fault in pdfcer, not in your document. The parser's reason: {reason}"
        ),
        R::CoreRefused { reason } => {
            format!("Redaction refused, and nothing was written: {reason}")
        }
        R::VerificationFailed { survivors } => format!(
            "Redaction refused — between checking the result and writing it, pdfcer found {} piece(s) of removed text in drawn content that were not in the list you acknowledged. Nothing was written. Close this window and run Apply redactions again so the list is rebuilt.",
            survivors.len()
        ),
        // Not a failure, and the sentence must not read as one. This is the
        // ordinary state of a document whose removal is already armed, and the
        // only reason it arrives as a refusal at all is that the pipeline
        // refuses by name rather than letting the engine's `RedactionPending`
        // surface as "this document cannot be rewritten in full" — a true
        // sentence about the wrong subject. See `crate::redact` §1.0.2.
        R::AlreadyStaged => {
            "A removal is already set up for this document. Nothing has been removed yet; it happens when you save. Use Save or Save As to carry it out, or call it off from this window."
                .to_owned()
        }
    }
}

/// **The sentence for a save that could not be built because the staged
/// removal was refused.**
#[must_use]
pub fn save_refused_message(refusal: &crate::redact::RedactApplyRefusal) -> String {
    use crate::redact::RedactApplyRefusal as R;
    match refusal {
        R::NothingToApply => {
            "Nothing was saved. This document has a removal set up, and there are no marks left for it to remove — the marks were taken off after it was set up. Call the removal off from Review & apply, then save; while it is set up, this is the only kind of save pdfcer will do."
                .to_owned()
        }
        other => format!(
            "Nothing was saved: the removal this document has set up could not be carried out, so pdfcer refused rather than write a file with the content still in it. {}",
            refusal_message(other)
        ),
    }
}

/// The sentence for a write that was attempted and produced no file.
#[must_use]
pub fn write_failed(reason: &crate::redact::WriteRefusal) -> String {
    use crate::redact::WriteRefusal as W;
    match reason {
        // Unreachable from the dialog, whose confirm control is disabled until
        // the box is ticked — and worded rather than left to a panic, because
        // `crate::redact::write_to`'s gate is the mechanism and a control being
        // greyed is not.
        W::ResidualsNotAcknowledged { .. } => {
            "Nothing was written: the items pdfcer could not remove have not been acknowledged.".to_owned()
        }
        W::VerificationFailed { survivors } => format!(
            "Nothing was written. pdfcer checked the finished bytes one last time before writing them and found {} piece(s) of the supposedly-removed text still present. Do not use any file produced from this document until this is investigated.",
            survivors.len()
        ),
        W::FileSystem(_) => {
            "The redacted file could not be written. Check that the folder still exists and that you can write to it, then try again — nothing has been lost, and the marks are still in the document.".to_owned()
        }
        // Reachable only from *This document, now*. It names the ONE thing
        // that still works, because the operator has just been told a removal
        // both succeeded and cannot be used — and a sentence that stops there
        // reads as data loss when nothing has been lost at all.
        //
        // It says the open document is untouched, first. That is the question
        // an operator asks about their own screen before they ask about a file.
        W::RedactedDocumentUnreadable { .. } => {
            "The content was removed, but pdfcer could not read the result back to show it to you. Your open document has not been changed and nothing has been lost. Choose “A new file” instead — the same removal is written straight to disk on that route, without being read back.".to_owned()
        }
    }
}

/// The status-bar note appended to an ordinary save while marks are still
/// pending.
#[must_use]
pub fn save_kept_pending_marks(count: usize) -> String {
    format!(
        "That save kept {count} pending redaction mark(s) in the file — the marked content is still there. Marking does not remove anything; nothing is removed until you apply."
    )
}

// ===========================================================================
// Appearance — what an applied redaction LOOKS like
// ===========================================================================
//

/// The appearance group's heading.
#[must_use]
pub const fn appearance_heading() -> &'static str {
    "How a redaction will look"
}

/// What the heading means, said once so no control below has to repeat it.
#[must_use]
pub const fn appearance_intro() -> &'static str {
    "Marks are outlined in red while you review them. These settings are what \
     replaces the content when you apply, and they are recorded on each mark \
     as you make it — so changing them affects the next mark, not the ones \
     already in the list."
}

/// The fill control's label.
#[must_use]
pub const fn fill_label() -> &'static str {
    "Cover with"
}

/// One fill's name.
#[must_use]
pub const fn fill_option_label(fill: crate::panels::redact::appearance::Fill) -> &'static str {
    use crate::panels::redact::appearance::Fill as F;
    match fill {
        F::Black => "Black",
        F::White => "White",
        F::Custom(..) => "Colour…",
        F::Transparent => "Nothing",
    }
}

/// What "Nothing" actually does, which is not what it sounds like.
#[must_use]
pub const fn fill_transparent_note() -> &'static str {
    "The content is still removed. Nothing is drawn over the gap, so the page \
     will not show where it was."
}

/// The caption field's label.
#[must_use]
pub const fn overlay_label() -> &'static str {
    "Write on it"
}

/// The caption field's placeholder-ish hint.
#[must_use]
pub const fn overlay_hint() -> &'static str {
    "Optional. Left empty, the box is plain."
}

/// The warning shown when a caption would be drawn in black on a dark fill.
#[must_use]
pub const fn overlay_illegible_warning() -> &'static str {
    "pdfcer draws this caption in black, and it will be hard to read on a dark \
     cover. Choose a lighter cover, or leave the caption off."
}

/// The justification control's label.
#[must_use]
pub const fn quadding_label() -> &'static str {
    "Line it up"
}

/// One justification's name.
#[must_use]
pub const fn quadding_option_label(q: pdfcer_core::vartext::Quadding) -> &'static str {
    use pdfcer_core::vartext::Quadding as Q;
    match q {
        Q::Left => "Left",
        Q::Center => "Centre",
        Q::Right => "Right",
    }
}

/// What the operator should know about the caption before they rely on it.
#[must_use]
pub const fn overlay_bound() -> &'static str {
    "Captions use a standard Latin font — other alphabets come out as question \
     marks — and pdfcer picks the size to fit the box, so a long caption on a \
     small mark ends up tiny."
}

/// What a mark-by-search says when part of the document could not be read.
#[must_use]
pub fn unreadable_warning(fonts: u64) -> String {
    if fonts == 1 {
        "One font in this document stores text that cannot be searched. Any matches inside it were NOT marked and are still in the file.".to_owned()
    } else {
        format!(
            "{fonts} fonts in this document store text that cannot be searched. Any matches inside them were NOT marked and are still in the file."
        )
    }
}

/// The hover behind [`unreadable_warning`].
#[must_use]
pub const fn unreadable_tooltip() -> &'static str {
    "Some PDFs store text as drawings with no record of which letters they are. It looks and prints normally, but nothing can search it — so searching for a word cannot find it, and marking every match will miss it. Check those areas by eye, or mark them by drawing a box."
}

/// **What was marked from a canvas selection.**
#[must_use]
pub fn marked_selection(objects: usize) -> String {
    if objects == 1 {
        "Marked for redaction. Nothing has been removed yet \u{2014} press Apply redactions \
         when you have checked the marks."
            .to_owned()
    } else {
        format!(
            "{objects} objects marked for redaction. Nothing has been removed yet \u{2014} \
             press Apply redactions when you have checked the marks."
        )
    }
}

/// The three wording rules, enforced — in their own file since 2026-09-04.
/// See [`tests`]'s header for the seam.
#[cfg(test)]
mod tests;
