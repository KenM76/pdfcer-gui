//! # `text::redact::carriers` — the places a copy of the removed text can hide
//!
//! ★★★ **Added 2026-09-09**, when `pdfcer-core` at `369d4de` introduced
//! [`CarrierAction::CheckedClean`][cc] and three whole-file-sweep counters, and
//! the engine-API drift gate showed that this shell reads **none** of them.
//!
//! [cc]: pdfcer_core::redact::CarrierAction::CheckedClean
//!
//! ## What a *carrier* is, in one paragraph, because the word is doing work
//!
//! ISO 32000-1 §12.5.6.23 obliges a redaction to remove the marked content from
//! **all** of the document, not from the page. A PDF can hold the same run of
//! text in a dozen places that are not the page: the document-information
//! dictionary, an XMP packet, a form's XML description, the tagged-reading
//! tree, an attached file, an earlier saved revision. The engine calls each of
//! those a **carrier**, sweeps every one of them, and records a verdict per
//! carrier in [`RedactionReport::carriers`][cs]. This module is where those
//! verdicts become English.
//!
//! [cs]: pdfcer_core::redact::RedactionReport::carriers
//!
//! ## ★★★ The defect this module was written to close
//!
//! Before it, this shell read the carrier list at exactly two sites and both
//! were the same `==` filter against `DisclosedNotScrubbed`. That has three
//! consequences, and each is worse than the last:
//!
//! 1. **The carrier's raw engine key was printed to the operator.** The
//!    sentence he saw read *"⚠ struct_tree: present in this document…"*. The
//!    engine documents those keys as *"a short stable identifier"* — an
//!    identifier for a program, in a report written for a person.
//! 2. **`residual_sweep` got the generic sentence**, which is about a carrier
//!    that *holds* content. The sweep holds nothing; it is the engine's search
//!    of every other object in the file. The generic sentence was therefore not
//!    merely jargon but false, in the residual list, on the one surface where
//!    rule 1 forbids a comfortable sentence.
//! 3. **`CheckedClean` was invisible.** Because both readers are `==` filters
//!    and neither is a `match`, the new variant was not swallowed by a
//!    catch-all — it was never read at all, and nothing on screen changed by
//!    one character. The engine's own doc comment says what that costs:
//!
//!    > *"a shell that tells an operator 'nothing to do' when the truth is
//!    > 'checked, clean' has taken away the one thing that distinguishes a
//!    > diligence sweep from a no-op."*
//!
//!    ★ And this is not a theoretical variant. `carrier_info` reports
//!    `CheckedClean` at the engine's `redact.rs:2791` and the residual sweep at
//!    `redact.rs:3138` — the two commonest carriers on the operator's own
//!    files.
//!
//! ## The wording rule this group adds to the three it inherits
//!
//! [`crate::text::redact`]'s rules 1–3 bind here unchanged. This group adds a
//! fourth, and every string below obeys it:
//!
//! > **Say where, in his vocabulary, never in the engine's.** A carrier name is
//! > the whole actionable content of a residual line: *"the tagged-reading
//! > structure a screen reader follows"* is something an operator can decide
//! > about in a second, and *"struct_tree"* is something he can only click
//! > past. This is the same finding `raw_residual_line` was corrected for on
//! > 2026-09-04, one sentence over, after his report that the warning *"always
//! > finds text that wasn't redacted"*.
//!
//! ★ **The fallback is the raw key, deliberately.** `CarrierStatus::carrier` is
//! an open vocabulary — a future engine build may add one — and the choice at
//! that moment is between printing an unknown identifier and printing nothing.
//! Printing nothing would drop a disclosure the engine went to the trouble of
//! making. So an unknown carrier reads awkwardly and is still *there*, which is
//! the correct trade on this surface. The engine-API drift gate is what will
//! tell us a new key exists; it is how `CheckedClean` was found in the first
//! place.

// ---------------------------------------------------------------------------
// Naming a carrier
// ---------------------------------------------------------------------------

/// **A carrier's engine key, in the operator's words.**
///
/// The fourteen keys are the vocabulary `pdfcer_core::redact::CarrierStatus`
/// documents at its own `redact.rs:270-272`, which is a superset of the twelve
/// any build currently emits: `thumbnails` and `overlapping_annotations` are
/// named by the engine's contract and are not produced by `369d4de`. They are
/// listed anyway, because the cost of an unused arm is nothing and the cost of
/// a missing one is the raw key on screen.
///
/// Returns the input unchanged for a key it does not know — see the module
/// header for why that beats returning nothing.
///
/// ★ Each phrase is a **noun phrase**, because every caller drops it into the
/// subject slot of a sentence it does not control (*"⚠ {name}: present in this
/// document…"*, *"pdfcer also checked … — {list} —"*). A phrase that read as a
/// clause would break both.
#[must_use]
pub fn carrier_name(carrier: &str) -> &str {
    match carrier {
        // Named for what he sees in the Properties dialog of every PDF reader
        // ever written, not for the trailer key it comes from.
        "info" => "the document properties — title, author, subject, keywords",
        "xmp" => "the document's embedded metadata record (XMP)",
        // ★ The sweep is not a place; it is the search of every other place.
        // `residual_sweep_line` exists because that difference matters in the
        // residual list. This phrase is for the clean census, where it is one
        // item in a list and reads correctly as one.
        "residual_sweep" => "every other object in the file",
        "images" => "the pictures on the page",
        "xfa" => "an XFA form — a form described by an XML document rather than by PDF fields",
        "struct_tree" => "the tagged-reading structure a screen reader follows",
        "attachments" => "files attached to this document",
        "ocg" => "the optional-content layers a viewer can switch on and off",
        "vector_paths" => "the drawn lines and filled shapes",
        "shadings" => "gradient fills",
        "object_streams" => "the compressed object containers",
        "prior_revisions" => "earlier saved revisions kept inside the file",
        "thumbnails" => "the page thumbnails stored in the file",
        "overlapping_annotations" => "annotations sitting over a marked region",
        other => other,
    }
}

// ---------------------------------------------------------------------------
// The residual lines
// ---------------------------------------------------------------------------

/// One residual line for a carrier the engine detected and could not scrub.
///
/// Moved here from `text/redact/mod.rs` on 2026-09-09 with its wording
/// corrected to name the carrier in English; the signature and every call site
/// are unchanged, because the parent re-exports it.
///
/// ★ Dispatches to [`residual_sweep_line`] for the one carrier the generic
/// sentence is **false** about. Selecting the sentence here rather than at the
/// call site is deliberate: the call site is a `.map` over the whole carrier
/// list and has no business knowing that one member of that list is a different
/// kind of thing. The catalog owns the words, and owns which words.
#[must_use]
pub fn residual_carrier_line(carrier: &str) -> String {
    if carrier == "residual_sweep" {
        return residual_sweep_line().to_owned();
    }
    format!(
        "⚠  {}: present in this document, and pdfcer cannot scrub it in this build. Whatever it holds will still be in the saved file — check it by hand.",
        carrier_name(carrier)
    )
}

/// ★★ **The residual sentence for the whole-file sweep, which is not a carrier
/// at all.**
///
/// The engine reports `residual_sweep` as `DisclosedNotScrubbed` for two
/// distinct reasons, and this sentence has to be true of both, because the
/// carrier list does not distinguish them:
///
/// | cause | engine site | what happened |
/// |---|---|---|
/// | the sweep never ran | `redact.rs:2962` | every removed piece is shorter than the engine's match floor, so searching for them would edit on a coincidence |
/// | the sweep ran and stopped short | `redact.rs:3126` | some stream objects hold the text and are not safe to blank — a font programme, an image, or text drawn through a subset font whose operand bytes are glyph codes rather than characters |
///
/// ★★★ **The second cause is the operator's own files.** `OPERATOR_REQUESTS.md`
/// O142's finding is that his CAD sheets draw text one glyph at a time through
/// subset fonts, which is exactly the shape the engine names here. So this is
/// not the rare arm; on his sheets it is the likely one.
///
/// ★ It ends by pointing at [`engine_notes_heading`]'s section, because that is
/// where the *object numbers* are — the engine puts them in a note, and a
/// sentence that says "some objects" while the numbers sit four inches below is
/// withholding the only part he can act on.
#[must_use]
pub fn residual_sweep_line() -> &'static str {
    "⚠  Copies elsewhere in the file: pdfcer searches every object in the document for the text it removed, and this search did not finish. Either no removed piece was long enough to search for without editing on a coincidence, or the text turned up inside objects pdfcer will not blank unread — a font programme, an image, or text drawn one glyph at a time through a subset font. pdfcer's own notes, at the foot of this report, say which. Check or remove them by hand."
}

// ---------------------------------------------------------------------------
// The two things that were never said at all
// ---------------------------------------------------------------------------

/// ★★★ **The diligence census: what pdfcer checked and found clean.**
///
/// This is [`CarrierAction::CheckedClean`][cc] reaching the screen, and the
/// argument for it is the engine's, quoted in the module header: *"checked,
/// clean"* and *"nothing to do"* are different facts, and only one of them is
/// evidence of diligence.
///
/// [cc]: pdfcer_core::redact::CarrierAction::CheckedClean
///
/// ★ It obeys rule 2 without needing the word. It does not say **verified** —
/// that word belongs to [`crate::text::redact::verified_line`] alone and is
/// earned by this shell's own byte sweep of the finished file. It says
/// *checked*, and *checked* is exactly what the engine did.
///
/// ★★ It obeys rule 1 too, and that is the subtler half: this sentence never
/// appears **instead of** a residual, only alongside one. The residual section
/// is derived from a different verdict and drawn from a different branch, so
/// there is no arrangement of this report in which a clean census can displace
/// a warning. That property is what makes it safe to write a reassuring
/// sentence on this surface at all.
///
/// Takes the names already in English — the caller maps through
/// [`carrier_name`] — so this function never has to know the key vocabulary.
#[must_use]
pub fn checked_clean_line(names: &[&str]) -> String {
    format!(
        "pdfcer also checked {} other place(s) a copy of the removed text could hide — {} — and found no trace of it in any of them.",
        names.len(),
        names.join("; ")
    )
}

/// ★★ **What the whole-file sweep found and will remove, beyond the pages.**
///
/// The three counters `pdfcer-core` `369d4de` added, in one sentence. They are
/// reported separately by the engine and stay separate here for the reason its
/// own doc comment gives: *"A single total would hide the fact that the second
/// number is the one nobody expected to be non-zero."*
///
/// * `entries` — stored text entries removed from dictionaries anywhere in the
///   file: a superseded copy of the document properties, a thread's information
///   dictionary, anything else whose strings quoted the removed text.
/// * `objects` — how many objects are edited in total. Always ≥ the number of
///   dictionaries, because it also counts metadata packets blanked and content
///   streams blanked.
/// * `content_streams` — of those, the ones that are **drawing instructions**.
///
/// ★★★ The third gets its own clause and no other treatment would do. Every
/// other member of the sweep removes a metadata string, which changes nothing
/// anybody looks at. This one changes what a page would paint if anything still
/// pointed at it — and the engine's own note on the field says a shell
/// disclosing *"pdfcer edited N objects"* should be able to say how many of them
/// were content. That is rule 1 in a different costume: a report that folds an
/// edit to drawing instructions into a count of metadata edits has quietly
/// picked which of the two is worth mentioning.
///
/// Future tense throughout, like every other sentence in the report body: this
/// is shown **before** the operator confirms, and nothing has happened yet.
#[must_use]
pub fn sweep_scrubbed_line(entries: u64, objects: u64, content_streams: u64) -> String {
    let mut out = format!(
        "Beyond the pages themselves, {objects} object(s) elsewhere in this file hold a copy of the removed text and will be scrubbed of it — {entries} stored text entr(y/ies)"
    );
    if content_streams > 0 {
        out.push_str(&format!(
            ", and {content_streams} abandoned drawing-instruction stream(s) whose text will be blanked. Those are the only part of this sweep that edits drawing instructions rather than stored text: nothing on any page points at them today, and what they would paint if anything did will change"
        ));
    }
    out.push('.');
    out
}

// ---------------------------------------------------------------------------
// The engine's own notes
// ---------------------------------------------------------------------------

/// The label on the collapsed section that holds
/// [`RedactionReport::notes`][notes].
///
/// [notes]: pdfcer_core::redact::RedactionReport::notes
///
/// ★★★ **Collapsed, and that is the whole design.** These notes are the
/// engine's own prose: they cite ISO 32000-1 by table number, they name objects
/// by number, and there can be a dozen of them on an ordinary sheet. Opened by
/// default they would bury the residual section under spec citations — and
/// `OPERATOR_REQUESTS.md` O160 is this operator telling us exactly what that
/// costs, about a different part of this same dialog: a warning nobody can act
/// on is a warning everybody learns to click past, which then costs the real
/// one its force.
///
/// ★★ **And they are not dropped, which is the other half.** Until 2026-09-09
/// `RedactionReport::notes` was read by nothing in this shell at all. The object
/// numbers the residual sweep could not scrub existed only there. A disclosure
/// the engine wrote and the shell discarded is worse than one it never wrote,
/// because the engine's authors believe it was delivered.
///
/// The count is in the label so the section advertises whether it is empty
/// before it is opened.
#[must_use]
pub fn engine_notes_heading(count: usize) -> String {
    format!("pdfcer's own notes on this removal ({count})")
}

/// The one line above the notes, saying whose voice they are in.
///
/// ★ Rule 1 territory: some of these notes ARE residuals — the retained-mark
/// note, the sweep's object list — and some are cosmetic. They are shown
/// unedited rather than summarised because a note pdfcer wrote about its own
/// uncertainty is the one thing this report must not paraphrase, and because
/// the sentences that matter most are already lifted out into the residual
/// section above by their own derivations. This section is the record; the
/// section above it is the warning.
#[must_use]
pub fn engine_notes_lead() -> &'static str {
    "In pdfcer's own words, unedited. These name objects by number and cite the PDF standard by clause; anything here that needed your attention is already stated above in plain English."
}
