//! # `text::importtext` — what the import tells the operator afterwards
//!
//! The receipt for `file.import_text`, and its refusals.
//! `dialogs::import_text` holds the words the *window* says before the press;
//! this module holds the words that arrive after it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/importtext.md`.

/// **How many pages arrived, and where.**
#[must_use]
pub fn pages_created(pages: usize, pages_before: usize) -> String {
    let noun = if pages == 1 { "page" } else { "pages" };
    format!(
        "Imported {pages} {noun}. The document had {pages_before} before and has \
         {} now.",
        pages_before + pages
    )
}

/// **The undo promise cannot be kept** — `PlaceTextReport::coalesced` is false.
#[must_use]
pub fn many_undo_steps(entries: usize) -> String {
    format!(
        "This import was too large to undo in one step — it will take {entries} presses of Undo \
         to reverse it completely."
    )
}

/// **A paragraph he wrote as one block is now on two sheets.**
///
/// Rule 4's surviving half: the result looks exactly like a paragraph he wrote
/// that way, so nothing on the page can tell him this happened.
#[must_use]
pub fn paragraphs_split(count: usize) -> String {
    let noun = if count == 1 {
        "paragraph"
    } else {
        "paragraphs"
    };
    format!("{count} {noun} ran past the bottom of a page and continue on the next one.")
}

/// **Tabs became spaces**, so column alignment is gone.
#[must_use]
pub fn tabs_collapsed(count: usize) -> String {
    let noun = if count == 1 { "tab" } else { "tabs" };
    format!(
        "{count} {noun} became ordinary spaces, so anything lined up in columns will not line up \
         any more. Importing again in Courier keeps space-aligned columns straight."
    )
}

/// **Form feeds in the source became page breaks.**
#[must_use]
pub fn page_breaks(count: usize) -> String {
    let noun = if count == 1 {
        "page break"
    } else {
        "page breaks"
    };
    format!(
        "{count} {noun} already in the file were used as they were — text exported from pdfcer \
         carries them, so a file that came from here keeps its original pagination."
    )
}

/// **A word wider than the column.**
#[must_use]
pub fn overlong_words(count: usize) -> String {
    let noun = if count == 1 { "word is" } else { "words are" };
    format!(
        "{count} {noun} wider than the column and will run past the right margin. A smaller \
         font size or a narrower margin would fit them."
    )
}

/// **Non-printing bytes were removed.**
#[must_use]
pub fn control_chars(count: usize) -> String {
    let noun = if count == 1 {
        "character"
    } else {
        "characters"
    };
    format!(
        "{count} non-printing {noun} were removed — they have no shape in any PDF font, so they \
         could not have been drawn."
    )
}

/// **Characters the font could not write, dropped.**
#[must_use]
pub fn unmappable_dropped(count: usize) -> String {
    let noun = if count == 1 {
        "character"
    } else {
        "characters"
    };
    format!(
        "{count} {noun} could not be written in the chosen font and were left out of the \
         imported pages."
    )
}

/// **The engine's own self-check failed** — this is a defect report.
#[must_use]
pub fn overflowed(lines: usize) -> String {
    let noun = if lines == 1 { "line" } else { "lines" };
    format!(
        "{lines} {noun} did not fit the page and were drawn outside it. That is a fault in \
         pdfcer rather than in your file — the import worked, but it is worth reporting."
    )
}

// ---------------------------------------------------------------------------
// Refusals. Nothing was created; the document is exactly as it was.
// ---------------------------------------------------------------------------

/// The file could not be opened.
#[must_use]
pub fn unreadable_file(why: &str) -> String {
    format!("That file could not be opened, so nothing was imported. {why}")
}

/// The file is not UTF-8.
#[must_use]
pub fn not_utf8() -> String {
    "That file is not UTF-8 text, so nothing was imported. Re-save it as UTF-8 — most editors \
     offer that under Save As — and try again."
        .to_owned()
}

/// The margins leave no width.
#[must_use]
pub fn no_column() -> String {
    "The margins leave no room for text on that sheet, so nothing was imported. Choose a larger \
     sheet size or a smaller margin."
        .to_owned()
}

/// The margins leave no height for even one line.
#[must_use]
pub fn page_too_short() -> String {
    "The margins leave no room for even one line on that sheet, so nothing was imported. Choose \
     a larger sheet size, a smaller margin, or a smaller font size."
        .to_owned()
}

/// The font cannot write some of the text.
#[must_use]
pub fn unmappable_refused(base_font: &str, total: usize, listing: &str) -> String {
    let noun = if total == 1 {
        "character"
    } else {
        "characters"
    };
    format!(
        "{base_font} cannot write {total} {noun} in that file, so nothing was imported: \
         {listing}. Try importing in a different font, or replace those characters in the file."
    )
}

/// The document has no page to insert beside.
#[must_use]
pub fn no_page_to_insert_beside() -> String {
    "This document has no pages, so there is nowhere to put the imported ones. Imported pages go \
     before or after an existing page."
        .to_owned()
}

/// Everything else, carrying the engine's own words.
#[must_use]
pub fn refused(why: &str) -> String {
    format!("Nothing was imported and the document is exactly as it was. {why}")
}
