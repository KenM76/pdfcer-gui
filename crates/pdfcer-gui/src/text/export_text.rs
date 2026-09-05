//! # `text::export_text` — every word the Export-text window shows, and every
//! sentence a text export owes afterwards
//!
//! The operator, 2026-09-04, verbatim:
//!
//! > *"also the engine can export PDFs as text. we should have export/import
//! > for that."*
//!
//! Half of that sentence is buildable and half of it is not, and this catalog
//! is the buildable half's copy. See
//! [`crate::app::actions::exporttext`]'s header for the whole finding on the
//! import side; the short version is that **`pdfcer-core` has no verb that
//! turns a text file into PDF page content**, and a request has been filed
//! rather than a round trip faked.
//!
//! ## ★★★ The one sentence this whole window is arranged around
//!
//! > **A scanned drawing has no text layer, so exporting it writes an empty
//! > file — and an empty file looks exactly like a successful export.**
//!
//! That is this feature's version of the DXF export's *"a 1:2 detail arrives at
//! half size and looks plausible"*: the failure is silent, the artifact opens
//! cleanly, and the person who finds out is whoever needed the words. So the
//! export **refuses** rather than writing nothing — [`no_text_at_all`] — and it
//! names the remedy, which is `File ▸ Recognise text`.
//!
//! ## ★★ What "the text of this document" means, and why it is not decided here
//!
//! `file.copy_document_text` has been putting a string on the clipboard since
//! 2026-08-20, and that string is
//! `text_extract::extract_document_view(…).plain_text()`. This export writes
//! **the same string**, byte for byte, at its default settings.
//!
//! That is not laziness, it is the whole point. Two answers to *"what is the
//! text of this document"* inside one program is worse than either answer on
//! its own, because both of them look like text and nothing on screen would
//! ever say which one you have. `app::dispatch::textcopy`'s header already
//! makes this argument for the two clipboard verbs sharing one extraction; this
//! is the same argument with a file on the end of it.
//!
//! ⇒ Every control in the window that departs from that string
//! ([`separator_marker`], [`line_endings_windows`], [`bom`]) is **opt-in**,
//! and every one of them is named in the receipt afterwards. The default is the
//! clipboard's own bytes.
//!
//! ## ★ Why the losses are said TWICE — in the window and in the receipt
//!
//! They are different losses.
//!
//! * **In the window**: the *standing* truths, which are true of every text
//!   export of every document and are therefore knowable before the press —
//!   layout is gone, a table becomes lines, columns may interleave, nothing
//!   about position or font or colour travels.
//! * **In the receipt**: the *counted* ones, which are facts about **this**
//!   document and cannot be known until the extraction has run — pages that
//!   came out empty, fonts carrying text that was never recoverable as Unicode
//!   at all, pages whose content stream would not walk.
//!
//! Rule 4 as narrowed by decision 059 puts the second set **off-canvas and
//! after the fact**: the status bar, never a mark drawn on the page.

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Export text"
}

/// The paragraph under the title.
///
/// Leads with what a text file **is not**, on [`crate::text::export_dxf`]'s
/// rule: the operator opening one expects the page, and what arrives is the
/// words in content order with every trace of the page removed. The sentence
/// that saves a support question is the one about what was left behind.
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
///
/// The same syntax the Print window, the Insert-pages window, the OCR window
/// and the image export all accept, because they all call
/// `dialogs::print::tabs::parse_page_range`. An operator who learned it in one
/// window is entitled to it in the next.
#[must_use]
pub const fn pages_range_hint() -> &'static str {
    "For example 3, or 1-4, or 5,1-2. Page numbers are the ones printed on the \
     paper."
}

/// The typed range names no page.
///
/// Says the document's own count, because the commonest cause is a range that
/// runs past the end and the operator cannot check that against a number they
/// have not been given.
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

/// ★ The marker's hint, and it discloses that this is pdfcer's own text.
///
/// The operator is asking for something readable and getting something the
/// document does not contain. That is worth one clause, because a later reader
/// of the file has no way to tell the marker from a line that was on the page —
/// and on a drawing whose title block genuinely says `Page 2 of 6`, they would
/// be right not to be able to.
#[must_use]
pub const fn separator_marker_hint() -> &'static str {
    "Easier to read, but these lines are pdfcer's own words — they are not in \
     the document, and nothing in the file will say so later."
}

/// The marker line itself, written into the exported file.
///
/// ★ Catalogued rather than formatted at the call site even though it lands in
/// a file rather than on screen, because it is **prose an operator reads** and
/// the whole point of the catalog is that such prose lives in one place. The
/// blank line before it is part of the string: without it the marker runs on
/// from whatever the previous page's last line was.
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

/// ★ The encoding, stated rather than assumed.
///
/// A CAD drawing carries degree signs, diameter marks, plus-or-minus and
/// occasionally a Greek letter, and every one of those is multi-byte in UTF-8
/// and mangled by anything that guesses a code page. Saying so costs one line
/// and answers the question an operator asks after the mangling, not before.
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

/// ★ Layout. The loss an operator is most likely to be surprised by.
///
/// A table is the worked example on purpose: it is the shape whose loss is
/// **invisible in the output**. A table exported as text is a perfectly
/// plausible list of lines, and nothing about it says that the columns used to
/// line up — which is exactly the shape of failure this project's rule 4
/// exists to refuse.
#[must_use]
pub const fn loses_layout() -> &'static str {
    "A table becomes a run of lines, and side-by-side columns can come out \
     interleaved a line at a time. The words are all there; the arrangement is \
     not, and the file gives no sign that it used to have one."
}

/// ★ Line and word breaks are pdfcer's, not the document's.
///
/// `text_extract`'s negative result S5: line breaks are **always** derived,
/// even in Tagged PDF, because a PDF content stream records where glyphs were
/// painted and nowhere records that two of them are in the same word. Saying so
/// matters because the operator is about to diff, grep or re-import this file,
/// and every one of those acts treats a line break as a fact about the source.
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

/// ★★★ **Nothing on any requested page carries readable text, so nothing was
/// written.**
///
/// The most important string in this catalog, and the reason the export refuses
/// before the save picker opens rather than after it.
///
/// A scanned drawing is a **picture of** text. There is no text layer, so the
/// extraction is correct, complete, and empty — and a zero-byte `.txt` on disk
/// is indistinguishable from a successful export of a page that happened to be
/// blank. The operator would find out when they opened it, or worse, when
/// whoever they sent it to did.
///
/// So it says three things in order: that nothing was written, **why** (the
/// page is a picture, which is a fact about their file rather than a pdfcer
/// failure), and the command that fixes it — named exactly as it appears on the
/// ribbon, because a remedy the operator cannot find is not a remedy.
#[must_use]
pub fn no_text_at_all(pages: usize) -> String {
    let subject = if pages == 1 {
        "That page carries no text pdfcer can read".to_owned()
    } else {
        format!("None of those {pages} pages carries any text pdfcer can read")
    };
    // ★ "Recognise text, on the File tab" rather than the ribbon-path
    // spelling with a `▸` in it. `crate::text::dropped` records the same
    // refusal and the reason binds hardest here: `icons::glyphs` proves the
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
///
/// The lead-in, so it is the sentence an operator reads if they read only one —
/// `super::super::app::actions::record_notes`' own rule. Characters rather than
/// bytes, because the operator asked for words and a byte count of UTF-8 is a
/// number about the encoding.
#[must_use]
pub fn wrote(path: &str, pages: usize, characters: usize) -> String {
    let page_word = if pages == 1 { "page" } else { "pages" };
    format!("{pages} {page_word} written to {path} — {characters} characters.")
}

/// The encoding actually used, when it was not the plain default.
///
/// Reported only when a departure was chosen, on the image export's rule that a
/// bar which narrates non-events stops being read. UTF-8 without a mark is what
/// the window promised and what the clipboard already carries.
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
///
/// ★ Repeated here even though the window said it, because the window is gone
/// and the file is not. This is the one added-text disclosure that survives the
/// act — an operator who sends the file on has sent lines pdfcer wrote.
#[must_use]
pub fn marker_lines_added(count: usize) -> String {
    format!(
        "{count} page-marker line(s) were added by pdfcer. They are not in the \
         document and nothing in the file distinguishes them from text that is."
    )
}

/// ★ Some of the pages asked for produced nothing, and they are named.
///
/// Named rather than counted, because *which* page came out empty is the whole
/// of what the operator does next with this sentence: an empty page 4 in a
/// six-page set is a scanned insert, and they can go and look at it.
///
/// Capped, because a fifty-page scan set would otherwise put fifty numbers in a
/// status bar. The cap is stated rather than silent — a trailing "and N more"
/// is a count the operator can act on; a truncated list they were not told was
/// truncated is a wrong answer.
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

/// ★★ Text that exists, renders perfectly, and was never recoverable as
/// Unicode.
///
/// `text_extract`'s two dead ends, and the engine is emphatic that neither is a
/// pdfcer shortfall: a **Type 3** font names its glyphs with arbitrary
/// `/CharProcs` keys and an **Identity-H** font with no `/ToUnicode` publishes
/// no mapping at all, so ISO 32000-1 §9.10.2's own answer is that no Unicode
/// exists to be recovered. Acrobat is gated on the identical entry.
///
/// ⇒ Which is exactly why it must be said. The page looks right, the export
/// looks like it worked, and the words are missing — and Acrobat's answer to
/// this case is to give up silently, which rule 4 forbids.
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
///
/// The engine's headline honesty metric, and it is reported as a **fraction**
/// rather than a bare count: 40 failures out of 200 characters is a broken
/// export and 40 out of 400,000 is a stray glyph, and the two need different
/// reactions from the operator.
///
/// ★ It **describes** the replacement character rather than printing one.
/// `icons::glyphs` proves the font stack cannot draw U+FFFD, so a literal one
/// here would render as a substitution box — and a sentence explaining that
/// unreadable characters became a box, in which the box is itself unreadable,
/// is a joke at the operator's expense. The name is also what they can search
/// their text editor for.
#[must_use]
pub fn undecodable_characters(failures: u64, total: u64) -> String {
    format!(
        "{failures} of {total} characters could not be decoded and stand in the \
         file as the Unicode replacement character (U+FFFD)."
    )
}

/// Pages whose content stream would not walk at all.
///
/// A different fact from [`pages_without_text`] and kept apart from it: an
/// empty page is a page pdfcer read successfully and found nothing on; this is a
/// page pdfcer could not read. Rolling the two together would let a damaged
/// file present as a scan.
#[must_use]
pub fn pages_unreadable(count: usize) -> String {
    format!(
        "{count} page(s) could not be read at all — their content could not be \
         decoded — and are empty in the file. This is damage or an unsupported \
         construction, not a missing text layer."
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

    /// ★★★ **The scan sentence names the remedy, by its ribbon label.**
    ///
    /// This is the assertion the whole feature's honesty rests on. A refusal
    /// that says *"nothing to export"* and stops is a dead end; one that names
    /// `Recognise text` is a next step. The label is asserted **literally**, so
    /// that renaming the command on the ribbon without renaming it here fails
    /// here rather than in front of an operator hunting for a control that no
    /// longer has that name.
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

    /// ★ The empty-page list is capped, and the cap is **disclosed** rather
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

    /// ★ The page marker is surrounded by blank lines, so it cannot run on from
    /// the previous page's last line.
    #[test]
    fn the_page_marker_stands_alone() {
        let marker = page_marker(3);
        assert!(marker.starts_with("\n\n") && marker.ends_with("\n\n"));
        assert!(marker.contains("Page 3"));
    }
}
