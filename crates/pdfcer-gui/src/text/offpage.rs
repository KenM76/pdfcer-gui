//! # `text::offpage` — the words for **content that is in the file but not on
//! the sheet**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/offpage.md`.

use pdfcer_core::offpage::OffPage;

/// The window's title bar.
#[must_use]
pub const fn window_title() -> &'static str {
    "Content off the sheet"
}

/// The opening sentence, above everything.
#[must_use]
pub const fn intro() -> &'static str {
    "Anything drawn outside a page's boundary is still in the file. It does not \
     print and does not show on screen, but it is still copied, still found by a \
     text search, and still travels when the file is sent."
}

/// The headline when the scan found nothing at all.
///
/// "any page", not "this page": the scan is document-wide and a sentence that
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

/// **Why a picture crossing the edge is still on this list after a
/// clean** — the disclosure Rule 4 owes, added 2026-09-12.
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
#[must_use]
pub fn placement_word(how: OffPage) -> &'static str {
    match how {
        OffPage::Fully => "entirely off the sheet",
        OffPage::Partial => "crossing the edge",
        _ => "outside the boundary",
    }
}

/// The engine's stable object token turned into a word the operator uses.
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
#[must_use]
pub fn unreadable_row(page_index: usize, why: &str) -> String {
    let page = page_index + 1;
    format!("Page {page} could not be read, so it was not checked: {why}")
}

/// The line shown while the scan is still walking the document.
#[must_use]
pub fn scanning(done: usize, total: usize) -> String {
    format!("Checking page {} of {total}\u{2026}", done + 1)
}

/// The second sentence after a multi-page mark, about what Undo will do.
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
#[must_use]
pub fn marked_skipped(pages: usize) -> String {
    let sheets = if pages == 1 { "page" } else { "pages" };
    let were = if pages == 1 { "was" } else { "were" };
    format!("{pages} {sheets} could not be read and {were} left untouched.")
}

/// What pages the ENGINE refused contribute to the status line.
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

    /// The progress line names the page being WORKED ON, not the count
    /// already done.
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
    #[test]
    fn the_undo_note_carries_the_step_count() {
        let note = marked_undo_note(4);
        assert!(note.contains('4'), "got: {note}");
        assert!(note.contains("step"), "got: {note}");
    }

    /// The recovered string is what the row LEADS with.
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

    /// The two placements do not describe each other.
    #[test]
    fn the_two_placements_are_told_apart_by_their_own_words() {
        let fully = placement_word(OffPage::Fully);
        let partial = placement_word(OffPage::Partial);
        assert!(fully.contains("entirely"), "got: {fully}");
        assert!(!fully.contains("crossing"), "got: {fully}");
        assert!(partial.contains("crossing"), "got: {partial}");
        assert!(!partial.contains("entirely"), "got: {partial}");
    }

    /// Both counts in one heading when both are non-zero, and neither
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
