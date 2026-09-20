//! # `text::redact::removed` — the words themselves, not the count of them
//!
//! Consumed by [`crate::dialogs::redact::disclosures::removed_text`], drawn
//! under [`super::will_remove_heading`] beneath the counts.
//!
//! ## What this block exists to prevent
//!
//! `OPERATOR_REQUESTS.md` **O217**, fourth bullet: *"what will be removed is
//! visible before it is committed."* Redaction is the one verb in this program
//! where a selection that takes too much cannot be undone, and until this block
//! the apply report answered *how much* — regions, pages, characters, streams —
//! and never *what*. A number cannot be checked against an intention. The words
//! can.
//!
//! The case that makes it load-bearing rather than pleasant: the shell's unit
//! of text selection is the visual line, and `G032` records that the engine
//! groups a line without any horizontal-gap criterion, so a bill-of-materials
//! row welds its item number, part number, description and quantity into one
//! selectable thing. An operator who marks the quantity is marking the row. He
//! cannot see that on the canvas — R8b forbids marking the canvas, and rightly
//! — so the only place he can be told is here, and the only wording that tells
//! him is the text.
//!
//! ## The claim these sentences are allowed to make
//!
//! [`pdfcer_core::redact::RedactionReport::redacted_text`] is not a prediction.
//! [`crate::redact::prepare_redaction_apply`] performs the whole removal into
//! memory before this dialog draws, and the engine keeps the strings *because
//! the interpreter decoded the codes while removing them* — its own field doc
//! says they are kept "for the operator's review and for the absence-proof gate
//! to grep". This shell had built the grep and not the review.
//!
//! Two properties of that vector shape every sentence here, and both are
//! disclosed rather than smoothed over:
//!
//! - **One entry per marked region, concatenated.** A region that removed
//!   `"1"`, `"BRACKET"` and `"4"` is one entry reading `1BRACKET4`, not three.
//! - **Distinct.** Two regions that removed identical text collapse to one
//!   entry, so the entry count is a floor on the region count and never equals
//!   it by construction. An operator counting entries to check his marks would
//!   be counting the wrong thing, so [`removed_text_lead`] says so.

/// The most entries drawn before the list is cut short.
///
/// A backstop, not the normal case: a whole-page mark on a dense sheet can
/// concatenate a page per region, and a list nobody can reach the end of is a
/// list that gets skimmed — which is the failure this whole surface exists to
/// prevent. When it bites, [`removed_text_more`] says by how much; the cut is
/// never silent.
pub const MAX_ENTRIES: usize = 200;

/// The most characters drawn from any one entry.
///
/// Sized for the case this block was built for and one order above it: a welded
/// bill-of-materials row on the operator's own drawing runs to a few dozen
/// characters, so a row is never truncated and only a region that swallowed a
/// paragraph is. Counted in `char`s and cut on a `char` boundary, because a
/// byte cut inside a multi-byte sequence panics and the strings arriving here
/// are whatever the document's fonts decoded to.
pub const MAX_CHARS: usize = 240;

/// The heading over the list.
///
/// It says *text* rather than *words*: what a region removed is a run of
/// character codes, which may be a part-number fragment or a single digit, and
/// calling that a word invites the reader to expect prose and to distrust the
/// list when he does not get it.
#[must_use]
pub fn removed_text_heading() -> &'static str {
    "The text inside the marked regions, decoded as it was removed:"
}

/// The sentence under the heading that says what one entry is.
///
/// Both of `redacted_text`'s shape properties in one sentence, because an
/// operator who reads the list without them draws two wrong conclusions from
/// it: that each line is one piece of text on the page, and that the number of
/// lines is the number of marks.
#[must_use]
pub fn removed_text_lead() -> &'static str {
    "One line per marked region, run together in the order it was drawn. Regions that removed identical text appear once, so there are never more lines than marks and there are often fewer."
}

/// One entry, quoted, truncated on a `char` boundary if it is enormous.
///
/// Quoted so leading and trailing spaces are visible: a region that took
/// `" 4 "` and one that took `"4"` are different marks, and on a table row the
/// difference is whether the cell's padding went with it.
#[must_use]
pub fn removed_text_entry(text: &str) -> String {
    let count = text.chars().count();
    if count <= MAX_CHARS {
        return format!("\u{201c}{text}\u{201d}");
    }
    let shown: String = text.chars().take(MAX_CHARS).collect();
    let dropped = count - MAX_CHARS;
    format!(
        "\u{201c}{shown}\u{201d} \u{2014} and {dropped} more character(s) in this one region, not shown"
    )
}

/// The line that closes a list cut short by [`MAX_ENTRIES`].
#[must_use]
pub fn removed_text_more(hidden: usize) -> String {
    format!(
        "\u{2026} and {hidden} further region(s) whose text is not listed here. All of them will be removed; only the listing stops."
    )
}

/// Marks are about to be applied and none of them holds any text.
///
/// Not an omission and not an error: a mark over a drawing, a photograph or a
/// blank margin removes no character codes, and this is the sentence that keeps
/// *"pdfcer found nothing"* distinguishable from *"pdfcer did not look"*. R9's
/// rule — an unavailable capability renders nothing — does not apply, because
/// the answer here is a finding rather than an absent feature.
#[must_use]
pub fn removed_text_none() -> &'static str {
    "No text is inside the marked regions. Whatever they cover is drawn some other way \u{2014} line work, a picture, or blank paper \u{2014} and the counts above say what happens to it."
}

/// Character codes are going and the report carries no text for them.
///
/// The honest half of the block and the reason it is not an `if !is_empty()`.
/// Saying nothing here would let a silent empty list read as *"nothing textual
/// is in the marks"*, which is the opposite of what is true, and an inference
/// the operator cannot see owes a report under R8b whether or not the report is
/// comfortable.
///
/// ★ **It names no cause, deliberately.** At least two produce this state and
/// the shell cannot tell them apart from the report: a font whose encoding
/// cannot be inverted decodes to nothing, and a removed region whose mark the
/// engine could not attribute is skipped when `redacted_text` is assembled. The
/// sentence therefore states the observation — codes counted, no text reported
/// — and stops. A sentence naming one of two indistinguishable causes is a
/// guess wearing a report's voice, and this is the surface where that costs the
/// most.
#[must_use]
pub fn removed_text_undecodable(glyphs: u64) -> String {
    format!(
        "{glyphs} character(s) will be deleted from the page content and pdfcer can list none of them: the engine reported the removal without reporting any text for it. They are removed either way \u{2014} what is missing is the listing, not the removal."
    )
}
