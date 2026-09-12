//! # `text::offpage` — the words for **content that is in the file but not on
//! the sheet**
//!
//! The copy for [`crate::dialogs::offpage`], and the disclosure half of the
//! off-page feature O23 opened.
//!
//! ## ★★★ What this window is FOR, in the operator's own words
//!
//! O23 asked, on 2026-08-21, for objects outside the page box to be reachable
//! at all — they were invisible, unselectable and unrenderable. Three weeks
//! later he came back with the half that matters:
//!
//! > *"how do I view and edit objects that are off of the page? we added this
//! > feature but I didn't see how to enable it."*
//!
//! The **view** and **edit** halves shipped — a halo raster that paints past
//! the sheet, a pasteboard that lets the view centre on a point far off it, and
//! a cull that stopped throwing the page away when its sheet left the screen.
//! What none of them answer is the question that comes **first**:
//!
//! > **Which of my drawings have marks outside the sheet at all?**
//!
//! An operator cannot go and look at a thing whose existence they have no
//! reason to suspect. That is what this window is: a census, and a way to cut.
//!
//! ## ★★ Why this is a *Protect* control and not a *View* one
//!
//! Because off-page content is the classic PDF leak, and on a CAD drawing it is
//! not a hypothetical one. A sheet exported with the drawing border cropped
//! down, a superseded revision note parked outside the frame, a customer's name
//! on a title block that was moved off rather than deleted — **none of it
//! renders, and all of it is still in the file.** It is still extractable by any
//! text search, still copied by any text export, and still there when the file
//! is emailed.
//!
//! ⇒ So the sentences below lead with **what is still readable**, never with a
//! count. `pdfcer_core::offpage::OffPageObject::text` carries the recovered
//! string precisely so this module can say it, and the engine's own doc comment
//! makes the argument: *"there are 4 off-page objects"* invites a shrug, and
//! *"one of them reads `SUPERSEDED — DO NOT BUILD`"* does not.
//!
//! ## Rule 4 — this window marks nothing on the canvas
//!
//! Not one sentence here is drawn over the document, and pressing the mark
//! button produces `/Redact` annotations that render exactly as any other
//! `/Redact` annotation does. There is no provisional tint, no dashed halo, no
//! "off-page" badge. The disclosure is this window; the canvas is the document.
//!
//! ★ And the removal is a **mark**, not an apply. `edit.redact_apply` remains
//! the only place content is destroyed, which is the arm/mark/obliterate split
//! `crate::shell::commands::catalog::edit` argues at the redaction family's
//! registration. A control that scanned and deleted in one press would be a
//! fourth member of that family that skipped its middle step.
//!
//! ## ★ Why every function here takes primitives rather than the engine's types
//!
//! `pdfcer_core::offpage::PageScan` and `OffPageObject` are `#[non_exhaustive]`,
//! so nothing outside `pdfcer-core` can build one — which would make every test
//! below impossible to write, and a copy module whose sentences cannot be
//! asserted is a copy module that drifts. The one engine type that IS passed
//! through is [`OffPage`], because it is an enum: its variants can be named from
//! here, and describing "entirely off" as "crossing the edge" is exactly the
//! mistake a `bool` would invite.

use pdfcer_core::offpage::OffPage;

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Content off the sheet"
}

/// The opening sentence, above everything.
///
/// ★ It states the **consequence** before the subject, because the subject
/// ("objects outside the page box") is a thing an operator has no prior reason
/// to care about, and the consequence is the whole reason they should.
#[must_use]
pub const fn intro() -> &'static str {
    "Anything drawn outside a page's boundary is still in the file. It does not \
     print and does not show on screen, but it is still copied, still found by a \
     text search, and still travels when the file is sent."
}

/// The headline when the scan found nothing at all.
///
/// ★ "any page", not "this page": the scan is document-wide and a sentence that
/// sounded page-scoped would leave an operator wondering about the other forty.
#[must_use]
pub const fn nothing_found() -> &'static str {
    "Nothing is drawn outside any page boundary in this document."
}

/// The one-line summary above the per-page list.
///
/// Pages, not objects, close the sentence: the operator's next act is to go and
/// look at a page, and a bare object count does not tell them which one.
#[must_use]
pub fn summary(pages: usize, objects: usize) -> String {
    let sheets = if pages == 1 { "page" } else { "pages" };
    let marks = if objects == 1 { "object" } else { "objects" };
    format!("{objects} {marks} sit outside the boundary, on {pages} {sheets}.")
}

/// ★★★ **Why a picture crossing the edge is still on this list after a
/// clean** — the disclosure Rule 4 owes, added 2026-09-12.
///
/// # The fact, and it is the engine's, not this shell's
///
/// `pdfcer_core::offpage` says it outright: a `Partial` image **survives a
/// clean by design, and this scan still reports it.** `redact_image` clears
/// by snapping its cell grid **outward**, so it erases every sample that is
/// off the sheet and no sample that is not. What clearing cannot do is move
/// the placement — the picture's box still straddles the boundary — and this
/// scan classifies by GEOMETRY. So the picture is counted again on the next
/// run. The engine measured twelve such objects across seven of the
/// operator's 174 drawings and its own summary is the sentence to keep:
/// *the count is honest about the geometry and misleading about the ink.*
///
/// # Why this exists at all
///
/// Rule 4's surviving half — **an inference the operator cannot see still
/// owes an off-canvas report.** Clean the sheet, reopen this window, and the
/// picture is listed again. Without this sentence the only reading available
/// to the operator is *the clean failed*; they would be wrong, and nothing on
/// screen would say so.
///
/// ⚠ It marks nothing, tints nothing and flags nothing on the canvas. It is
/// a line in a results window, which is exactly where Rule 4 puts disclosure.
///
/// ★ **Pictures, never objects.** Line work and text do not behave this way —
/// their clean removes the geometry itself — so a sentence that swept them in
/// would be false about most of what this window reports. The caller counts
/// `kind == "image" && how == Partial` and shows nothing when that is zero,
/// because a paragraph explaining a residual this document does not have is
/// the kind of nagging the operator has already asked this project to stop.
#[must_use]
pub fn partial_image_note(pictures: usize) -> String {
    if pictures == 1 {
        "The picture crossing the edge stays on this list after a clean. Clearing \
         erases what is off the sheet but cannot move the picture, so its outline \
         still crosses the edge and this list counts by position. It is not a \
         failed clean."
            .to_owned()
    } else {
        format!(
            "The {pictures} pictures crossing the edge stay on this list after a \
             clean. Clearing erases what is off the sheet but cannot move a \
             picture, so its outline still crosses the edge and this list counts \
             by position. It is not a failed clean."
        )
    }
}

/// One page's heading in the list.
///
/// `page_index` is the engine's 0-based index and is displayed 1-based, because
/// every other page number in this shell is.
#[must_use]
pub fn page_heading(page_index: usize, fully: usize, partial: usize) -> String {
    let page = page_index + 1;
    match (fully, partial) {
        (0, _) => format!("Page {page} — {partial} crossing the edge"),
        (_, 0) => format!("Page {page} — {fully} entirely off the sheet"),
        _ => format!("Page {page} — {fully} entirely off, {partial} crossing the edge"),
    }
}

/// One object's row.
///
/// ★★ The recovered text comes FIRST when there is any, before the kind and
/// before anything else. See the module header: the string is the disclosure and
/// everything else is bookkeeping.
#[must_use]
pub fn object_row(kind: &str, how: OffPage, text: Option<&str>) -> String {
    let where_ = placement_word(how);
    let kind_word = kind_word(kind);
    match text.map(str::trim).filter(|t| !t.is_empty()) {
        Some(text) => format!("\u{201c}{text}\u{201d} — {kind_word}, {where_}"),
        None => format!("{kind_word} — {where_}"),
    }
}

/// How an object sits relative to its sheet, in words.
///
/// ★ The wildcard arm is not laziness. [`OffPage`] is `#[non_exhaustive]`, so a
/// third kind of overhang the engine adds tomorrow must neither stop this shell
/// compiling nor be described as one of the two it is not.
#[must_use]
pub fn placement_word(how: OffPage) -> &'static str {
    match how {
        OffPage::Fully => "entirely off the sheet",
        OffPage::Partial => "crossing the edge",
        _ => "outside the boundary",
    }
}

/// The engine's stable object token turned into a word the operator uses.
///
/// ★ A `match` rather than a capitalisation, so a token this shell has never met
/// renders as itself rather than as a mangled English word. The engine documents
/// `path`, `text` and `image`; anything else is new, and showing it verbatim
/// means a report about it names the real token.
#[must_use]
pub fn kind_word(kind: &str) -> &str {
    // ui-text-exempt: the left-hand side is the engine's stable token, never displayed.
    match kind {
        "path" => "Line work",
        "text" => "Text",
        "image" => "Picture",
        other => other,
    }
}

/// The note under a page whose content could not be read.
///
/// ★★★ *"No findings"* and *"I could not look"* must not print the same way —
/// the engine says so in `scan_document`'s own doc comment and returns the two
/// separately for exactly this reason. A page that will not decode is the one
/// place this window must not imply a clean bill.
#[must_use]
pub fn unreadable_row(page_index: usize, why: &str) -> String {
    let page = page_index + 1;
    format!("Page {page} could not be read, so it was not checked: {why}")
}

/// The line shown while the scan is still walking the document.
///
/// ★★★ This window is the one place in this shell that does real work **after**
/// it has opened, one page per frame, because a whole-document decomposition of
/// the operator's benchmark drawing measures 469 ms per sheet and a 36-sheet set
/// would freeze the program for seventeen seconds. So the progress line is not
/// decoration: without it the window would sit there listing nothing while the
/// answer was still being computed, which reads exactly like "nothing found".
///
/// `done` is a count of pages already checked, so the page being worked on is
/// `done + 1` — the number is a position in the walk, not an index.
#[must_use]
pub fn scanning(done: usize, total: usize) -> String {
    format!("Checking page {} of {total}\u{2026}", done + 1)
}

/// The second sentence after a multi-page mark, about what Undo will do.
///
/// ★★★ Rule 4's surviving half. The engine records **one command per page**,
/// because there is no verb that authors redaction marks across a page range —
/// so one press of this window's button leaves the operator with N undo steps
/// rather than one, and a single `Ctrl+Z` takes back one sheet's marks and
/// leaves the rest. That is invisible: the marks all appeared at once, so
/// nothing on screen suggests they will not leave at once.
///
/// Returned only for a multi-page mark. On one page the sentence would be true
/// and useless, and a disclosure that fires when there is nothing to disclose is
/// how an operator learns to stop reading them.
#[must_use]
pub fn marked_undo_note(pages: usize) -> String {
    format!("Undo takes these back one page at a time \u{2014} {pages} steps.")
}

/// The button that puts `/Redact` marks over every off-page band.
#[must_use]
pub const fn mark_button() -> &'static str {
    "Mark it all for removal"
}

/// Hover text for the mark button, and the sentence that keeps the middle step
/// of arm/mark/obliterate visible.
#[must_use]
pub const fn mark_tooltip() -> &'static str {
    "Adds redaction marks over the area outside each page boundary. Nothing is \
     removed yet — review the marks, then use Apply redactions."
}

/// Why the mark button is greyed.
#[must_use]
pub const fn nothing_to_mark() -> &'static str {
    "There is nothing outside any page boundary to mark."
}

/// The close button.
#[must_use]
pub const fn close_button() -> &'static str {
    "Close"
}

/// What the status line says after the marks land.
///
/// ★ The count is of BANDS, not of objects, and the sentence says so. A page
/// with six off-page objects in one corner gets one band over that corner, and
/// an operator told "6 marks" who then counts four in the review list would be
/// right to distrust the program.
#[must_use]
pub fn marked_disclosure(bands: usize, pages: usize) -> String {
    let marks = if bands == 1 { "mark" } else { "marks" };
    let sheets = if pages == 1 { "page" } else { "pages" };
    format!(
        "Marked {bands} redaction {marks} across {pages} {sheets}. Nothing is removed \
         until you apply redactions."
    )
}

/// What pages the scan could not read contribute to the status line.
///
/// ★ Second sentence, never instead of the first: the mark SUCCEEDED, and the
/// residual belongs beside that rather than in place of it. Rule 4's ordering,
/// the same as `crate::text::redact::mark_covers_image`'s.
#[must_use]
pub fn marked_skipped(pages: usize) -> String {
    let sheets = if pages == 1 { "page" } else { "pages" };
    let were = if pages == 1 { "was" } else { "were" };
    format!("{pages} {sheets} could not be read and {were} left untouched.")
}

/// What pages the ENGINE refused contribute to the status line.
///
/// ★★ Distinct from [`marked_skipped`], which is about pages the **scan** could
/// not read, and the two can both be true of one press. This one means the scan
/// found off-page content on a sheet and `add_redaction` then declined to
/// author the mark — a certified document, an encrypted one, a degenerate band.
/// Collapsing them into one sentence would tell an operator to go and look at a
/// page for the wrong reason.
#[must_use]
pub fn marked_refused(pages: usize) -> String {
    let sheets = if pages == 1 { "page" } else { "pages" };
    let were = if pages == 1 { "was" } else { "were" };
    format!("{pages} {sheets} {were} refused and carry no mark.")
}

/// The refusal when the document's page tree will not walk.
///
/// The one failure that is about the **document** rather than a page — see
/// `scan_document`'s error contract.
#[must_use]
pub fn cannot_scan(why: &str) -> String {
    format!("This document's page list could not be read, so nothing was checked: {why}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ The progress line names the page being WORKED ON, not the count
    /// already done.
    ///
    /// Falsified per clause, on the boundary that matters: before any page has
    /// been checked the line must say page **1**, not page 0 and not page 2.
    /// A one-off here is the kind of defect nobody reports and everybody sees.
    #[test]
    fn the_progress_line_counts_from_one_and_never_overshoots() {
        assert!(
            scanning(0, 36).contains("page 1 of 36"),
            "the first frame must name page 1, got: {}",
            scanning(0, 36)
        );
        assert!(
            !scanning(0, 36).contains("page 0"),
            "a 0-based index must never reach the operator, got: {}",
            scanning(0, 36)
        );
        assert!(
            scanning(35, 36).contains("page 36 of 36"),
            "the last frame must name the last page, got: {}",
            scanning(35, 36)
        );
    }

    /// The undo note states the number of steps, because the number is the
    /// whole disclosure.
    ///
    /// "Undo takes these back one page at a time" without a count leaves the
    /// operator pressing `Ctrl+Z` an unknown number of times; the count is what
    /// turns the sentence into an instruction.
    #[test]
    fn the_undo_note_carries_the_step_count() {
        let note = marked_undo_note(4);
        assert!(note.contains('4'), "got: {note}");
        assert!(note.contains("step"), "got: {note}");
    }

    /// ★★★ The recovered string is what the row LEADS with.
    ///
    /// This is the module header's whole argument, asserted rather than
    /// described: an operator who reads "3 objects" shrugs, and one who reads
    /// the words on the off-page note does not.
    #[test]
    fn a_row_for_off_page_text_quotes_the_text_before_anything_else() {
        let row = object_row("text", OffPage::Fully, Some("SUPERSEDED"));
        assert!(
            row.starts_with('\u{201c}'),
            "the recovered text must open the row, got: {row}"
        );
        assert!(row.contains("SUPERSEDED"), "got: {row}");
    }

    /// Whitespace-only recovered text is not text.
    ///
    /// A text object whose glyphs mapped to spaces would otherwise render as a
    /// row that quotes nothing and reads as a bug.
    #[test]
    fn a_blank_recovered_string_falls_back_to_the_kind() {
        let row = object_row("text", OffPage::Partial, Some("   "));
        assert!(!row.contains('\u{201c}'), "got: {row}");
        assert!(row.starts_with("Text"), "got: {row}");
    }

    /// An engine token this shell has never met is shown verbatim.
    #[test]
    fn an_unknown_kind_token_is_shown_rather_than_mangled() {
        assert_eq!(kind_word("shading"), "shading");
    }

    /// ★★ The two placements do not describe each other.
    ///
    /// Falsified per clause: this asserts that *fully* does not carry the word
    /// the *partial* sentence turns on, and the reverse. An `||` across the pair
    /// would assert neither.
    #[test]
    fn the_two_placements_are_told_apart_by_their_own_words() {
        let fully = placement_word(OffPage::Fully);
        let partial = placement_word(OffPage::Partial);
        assert!(fully.contains("entirely"), "got: {fully}");
        assert!(!fully.contains("crossing"), "got: {fully}");
        assert!(partial.contains("crossing"), "got: {partial}");
        assert!(!partial.contains("entirely"), "got: {partial}");
    }

    /// ★ Both counts in one heading when both are non-zero, and neither
    /// mentioned when it is zero. A heading reading "0 entirely off" is noise on
    /// every page that has only edge-crossers, which is most of them.
    #[test]
    fn a_page_heading_names_only_the_counts_that_are_non_zero() {
        let heading = page_heading(3, 1, 0);
        assert!(heading.starts_with("Page 4"), "1-based, got: {heading}");
        assert!(heading.contains("1 entirely off"), "got: {heading}");
        assert!(!heading.contains("crossing"), "got: {heading}");

        let both = page_heading(0, 2, 5);
        assert!(both.contains("2 entirely off"), "got: {both}");
        assert!(both.contains("5 crossing"), "got: {both}");
    }

    /// An unreadable page is reported 1-based and carries the engine's reason.
    #[test]
    fn an_unreadable_page_is_named_and_the_reason_is_carried() {
        let row = unreadable_row(11, "content stream will not decode");
        assert!(row.starts_with("Page 12"), "got: {row}");
        assert!(row.contains("will not decode"), "got: {row}");
    }

    /// The mark disclosure counts BANDS, says the word, and says nothing is
    /// gone yet.
    #[test]
    fn the_mark_disclosure_says_marks_and_says_nothing_is_removed_yet() {
        let line = marked_disclosure(4, 2);
        assert!(line.contains("4 redaction marks"), "got: {line}");
        assert!(line.contains("until you apply"), "got: {line}");
        let one = marked_disclosure(1, 1);
        assert!(one.contains("1 redaction mark across 1 page"), "got: {one}");
    }

    /// The skipped-pages sentence agrees with itself on number.
    #[test]
    fn the_skipped_sentence_is_singular_for_one_page() {
        assert!(marked_skipped(1).contains("1 page could not be read and was"));
        assert!(marked_skipped(3).contains("3 pages could not be read and were"));
    }
}
