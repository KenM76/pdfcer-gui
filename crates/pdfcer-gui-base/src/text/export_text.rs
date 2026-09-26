//! # `text::export_text` — every word the Export-text window shows, and every
//! sentence a text export owes afterwards
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/export_text.md`.

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Export text"
}

/// The paragraph under the title.
#[must_use]
pub const fn intro() -> &'static str {
    "The words on the page are written to a plain text file. Only the words \
     travel — nothing about where they sat, what size they were, or which \
     font they were set in — so a drawing's title block arrives as lines of \
     text rather than as a block."
}

// ---------------------------------------------------------------------------
// Which pages
// ---------------------------------------------------------------------------

/// The heading over the page controls.
#[must_use]
pub const fn pages_heading() -> &'static str {
    "Pages"
}

/// *This page only*, naming the page so the choice is checkable.
#[must_use]
pub fn pages_current(page_number: usize) -> String {
    format!("This page only (page {page_number})")
}

/// *Every page*, with the count, so the operator knows what they are asking
/// for before a long document blocks the window.
#[must_use]
pub fn pages_all(count: usize) -> String {
    format!("Every page ({count})")
}

/// The typed-range radio.
#[must_use]
pub const fn pages_range() -> &'static str {
    "Pages"
}

/// What the range box accepts.
#[must_use]
pub const fn pages_range_hint() -> &'static str {
    "For example 3, or 1-4, or 5,1-2. Page numbers are the ones printed on the \
     paper."
}

/// The typed range names no page.
#[must_use]
pub fn pages_range_invalid(count: usize) -> String {
    format!("That names no page. This document has {count}.")
}

// ---------------------------------------------------------------------------
// Where one page ends and the next begins
// ---------------------------------------------------------------------------

/// The heading over the page-separator controls.
#[must_use]
pub const fn separator_heading() -> &'static str {
    "Between pages"
}

/// The default: the engine's own separator, U+000C.
#[must_use]
pub const fn separator_form_feed() -> &'static str {
    "A page break (the usual choice)"
}

/// The default separator's hint.
#[must_use]
pub const fn separator_form_feed_hint() -> &'static str {
    "A form-feed character, which is what every other program writing extracted \
     text uses and what Copy document text already puts on the clipboard. Most \
     text editors show it as a page break; a few show nothing at all."
}

/// The opt-in: a visible marker line pdfcer writes.
#[must_use]
pub const fn separator_marker() -> &'static str {
    "A line saying which page follows"
}

/// The marker's hint, and it discloses that this is pdfcer's own text.
#[must_use]
pub const fn separator_marker_hint() -> &'static str {
    "Easier to read, but these lines are pdfcer's own words — they are not in \
     the document, and nothing in the file will say so later."
}

/// The marker line itself, written into the exported file.
#[must_use]
pub fn page_marker(page_number: usize) -> String {
    format!("\n\n----- Page {page_number} -----\n\n")
}

// ---------------------------------------------------------------------------
// How the file is written
// ---------------------------------------------------------------------------

/// The heading over the file-format controls.
#[must_use]
pub const fn file_heading() -> &'static str {
    "The file"
}

/// The encoding, stated rather than assumed.
#[must_use]
pub const fn encoding_line() -> &'static str {
    "Written as UTF-8, so degree signs, diameter marks and anything else beyond \
     plain ASCII survive."
}

/// The byte-order-mark checkbox.
#[must_use]
pub const fn bom() -> &'static str {
    "Start the file with a byte-order mark"
}

/// What a BOM buys and what it costs.
#[must_use]
pub const fn bom_hint() -> &'static str {
    "Three extra bytes that tell older programs the file is UTF-8. Helpful for \
     Excel and for Windows tools that would otherwise guess; a nuisance for \
     anything that reads the file as data."
}

/// The line-endings checkbox.
#[must_use]
pub const fn line_endings_windows() -> &'static str {
    "Use Windows line endings"
}

/// What the line-endings choice changes, and what the default is.
#[must_use]
pub const fn line_endings_hint() -> &'static str {
    "Off writes the lines exactly as they were extracted, which is what Copy \
     document text puts on the clipboard. Turn it on for a program that shows \
     the whole file as one long line."
}

// ---------------------------------------------------------------------------
// The standing losses — said in the window, before the press
// ---------------------------------------------------------------------------

/// The heading over the standing losses.
#[must_use]
pub const fn loses_heading() -> &'static str {
    "What a text file cannot carry"
}

/// Layout. The loss an operator is most likely to be surprised by.
///
#[must_use]
pub const fn loses_layout() -> &'static str {
    "A table becomes a run of lines, and side-by-side columns can come out \
     interleaved a line at a time. The words are all there; the arrangement is \
     not, and the file gives no sign that it used to have one."
}

/// Line and word breaks are pdfcer's, not the document's.
#[must_use]
pub const fn loses_breaks() -> &'static str {
    "Where the lines and the spaces fall is pdfcer's reading of where the \
     letters sat, not something the document records. Two files that look \
     identical on paper can break differently here."
}

/// Style, position and colour do not travel.
#[must_use]
pub const fn loses_style() -> &'static str {
    "Nothing about the font, the size, the colour or the position is written. \
     Text drawn behind an image, text on a hidden layer and text in the title \
     block all come out looking the same."
}

// ---------------------------------------------------------------------------
// Commit
// ---------------------------------------------------------------------------

/// The Export button.
#[must_use]
pub const fn export_button() -> &'static str {
    "Export"
}

/// The Cancel button.
#[must_use]
pub const fn cancel_button() -> &'static str {
    "Cancel"
}

/// The native save dialog's title.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Export text"
}

// ---------------------------------------------------------------------------
// The receipt — off-canvas, after the fact (rule 4 / decision 059)
// ---------------------------------------------------------------------------

/// **Nothing on any requested page carries readable text, so nothing was
/// written.**
#[must_use]
pub fn no_text_at_all(pages: usize) -> String {
    let subject = if pages == 1 {
        "That page carries no text pdfcer can read".to_owned()
    } else {
        format!("None of those {pages} pages carries any text pdfcer can read")
    };
    // "Recognise text, on the File tab" rather than the ribbon-path
    // spelling with a `▸` in it. `crate::text::dropped` records the same
    // refusal and the reason binds hardest here: `text::glyphs` proves the
    // font stack cannot draw that codepoint, so it renders as a substitution
    // box — and a box is the worst possible thing to put in the one sentence
    // whose entire job is to tell the operator where to go.
    format!(
        "{subject}, so no file was written. A scanned or plotted drawing is a \
         picture of its words rather than words, and there is nothing in it to \
         export. Recognise text, on the File tab, reads a scan and adds the \
         words behind the image; after that this export will find them."
    )
}

/// The receipt's first line: the file, and what landed in it.
#[must_use]
pub fn wrote(path: &str, pages: usize, characters: usize) -> String {
    let page_word = if pages == 1 { "page" } else { "pages" };
    format!("{pages} {page_word} written to {path} — {characters} characters.")
}

/// The encoding actually used, when it was not the plain default.
#[must_use]
pub fn wrote_with(bom: bool, windows_line_endings: bool) -> Option<String> {
    match (bom, windows_line_endings) {
        (false, false) => None,
        (true, false) => Some("Written with a UTF-8 byte-order mark.".to_owned()),
        (false, true) => Some("Written with Windows line endings.".to_owned()),
        (true, true) => {
            Some("Written with a UTF-8 byte-order mark and Windows line endings.".to_owned())
        }
    }
}

/// The page markers were pdfcer's own words, and the file does not say so.
#[must_use]
pub fn marker_lines_added(count: usize) -> String {
    format!(
        "{count} page-marker line(s) were added by pdfcer. They are not in the \
         document and nothing in the file distinguishes them from text that is."
    )
}

/// Some of the pages asked for produced nothing, and they are named.
#[must_use]
pub fn pages_without_text(page_numbers: &[usize]) -> String {
    const SHOWN: usize = 8;
    let listed = page_numbers
        .iter()
        .take(SHOWN)
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let tail = page_numbers.len().saturating_sub(SHOWN);
    let list = if tail > 0 {
        format!("{listed} and {tail} more")
    } else {
        listed
    };
    let verb = if page_numbers.len() == 1 {
        "carries"
    } else {
        "carry"
    };
    format!(
        "Page(s) {list} {verb} no text pdfcer can read and are empty in the \
         file — most often a scanned sheet. Recognise text, on the File tab, \
         would give them words."
    )
}

/// Text that exists, renders perfectly, and was never recoverable as
/// Unicode.
#[must_use]
pub fn unreadable_fonts(identity: u64, type3: u64) -> String {
    let total = identity.saturating_add(type3);
    format!(
        "{total} font(s) in this document publish no way to turn their glyphs \
         back into characters, so text set in them is missing from the file \
         even though it draws correctly on screen. That is the PDF standard's \
         own answer for these fonts, not a pdfcer limit — no reader can recover \
         those words."
    )
}

/// Characters that fell through the whole decoding ladder.
#[must_use]
pub fn undecodable_characters(failures: u64, total: u64) -> String {
    format!(
        "{failures} of {total} characters could not be decoded and stand in the \
         file as the Unicode replacement character (U+FFFD)."
    )
}

/// Pages whose content stream would not walk at all.
#[must_use]
pub fn pages_unreadable(count: usize) -> String {
    format!(
        "{count} page(s) could not be read at all — their content could not be \
         decoded — and are empty in the file. This is damage or an unsupported \
         construction, not a missing text layer."
    )
}

/// Pages that named no resources of their own, so pdfcer supplied an empty set.
#[must_use]
pub fn pages_resources_defaulted(count: usize) -> String {
    format!(
        "{count} page(s) name no resources of their own — pdfcer assumed an empty set so they \
         could be read. Text on such a page that used an inherited font may be missing rather \
         than absent."
    )
}

/// The plan named no page. Reachable only from a restored or malformed plan.
#[must_use]
pub const fn no_pages() -> &'static str {
    "No pages were named, so nothing was written."
}

/// The extraction itself failed.
#[must_use]
pub fn extract_failed(detail: &str) -> String {
    format!("The document's text could not be read: {detail}")
}

/// The file could not be written.
#[must_use]
pub fn export_failed(detail: &str) -> String {
    format!("The text could not be written: {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The scan sentence names the remedy, by its ribbon label.**
    #[test]
    fn the_empty_scan_sentence_points_at_the_command_that_fixes_it() {
        for pages in [1_usize, 6] {
            let said = no_text_at_all(pages);
            assert!(
                said.contains("Recognise text"),
                "the refusal must name the remedy: {said}"
            );
            assert!(
                said.contains("no file was written"),
                "the refusal must say nothing landed: {said}"
            );
        }
    }

    /// One page and several pages are worded differently, and neither reads as
    /// the other's grammar.
    #[test]
    fn the_empty_scan_sentence_is_grammatical_for_one_page_and_for_many() {
        assert!(no_text_at_all(1).starts_with("That page carries no text"));
        assert!(no_text_at_all(6).starts_with("None of those 6 pages carries any text"));
    }

    /// The empty-page list is capped, and the cap is **disclosed** rather
    /// than silent.
    #[test]
    fn a_long_empty_page_list_says_how_many_it_did_not_show() {
        let many: Vec<usize> = (1..=20).collect();
        let said = pages_without_text(&many);
        assert!(said.contains("and 12 more"), "{said}");
        assert!(said.contains("1, 2, 3, 4, 5, 6, 7, 8"), "{said}");
        // Short lists carry no tail at all.
        assert!(!pages_without_text(&[4]).contains("more"));
        assert!(pages_without_text(&[4]).contains("Page(s) 4 carries"));
    }

    /// The departures from the clipboard's own bytes are reported, and only
    /// when they happened.
    #[test]
    fn only_a_departure_from_the_default_encoding_earns_a_sentence() {
        assert_eq!(wrote_with(false, false), None);
        assert!(wrote_with(true, false).unwrap().contains("byte-order mark"));
        assert!(wrote_with(false, true).unwrap().contains("Windows line"));
        let both = wrote_with(true, true).unwrap();
        assert!(both.contains("byte-order mark") && both.contains("Windows line"));
    }

    /// The undecodable count carries its denominator — see the doc comment.
    #[test]
    fn undecodable_characters_states_the_denominator() {
        let said = undecodable_characters(40, 400_000);
        assert!(said.contains("40 of 400000"), "{said}");
    }

    /// The page marker is surrounded by blank lines, so it cannot run on from
    /// the previous page's last line.
    #[test]
    fn the_page_marker_stands_alone() {
        let marker = page_marker(3);
        assert!(marker.starts_with("\n\n") && marker.ends_with("\n\n"));
        assert!(marker.contains("Page 3"));
    }
}
