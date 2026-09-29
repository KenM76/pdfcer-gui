//! The words for **File ▸ Export ▸ Word document…** — the picker, the
//! refusal, and the receipt that counts what the conversion inferred.

/// The picker's title.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Export to Word"
}

/// The refusal when no page carries text, raised before the picker.
#[must_use]
pub fn no_text(pages: usize) -> String {
    let page_word = if pages == 1 {
        "page carries"
    } else {
        "pages carry"
    };
    format!(
        "None of the {pages} {page_word} text, so no Word document was written. A \
         scanned page is a picture of words; use Recognise text on it first."
    )
}

/// The receipt's first line: where it went and what it holds.
#[must_use]
pub fn wrote(
    path: &str,
    pages: usize,
    headings: usize,
    paragraphs: usize,
    tables: usize,
) -> String {
    let page_word = if pages == 1 { "page" } else { "pages" };
    let heading_word = if headings == 1 { "heading" } else { "headings" };
    let paragraph_word = if paragraphs == 1 {
        "paragraph"
    } else {
        "paragraphs"
    };
    let table_word = if tables == 1 { "table" } else { "tables" };
    format!(
        "Word document written to {path}: {pages} {page_word}, {headings} \
         {heading_word}, {paragraphs} {paragraph_word}, {tables} {table_word}."
    )
}

/// Paragraph styles judged from size and position because the PDF does not
/// say what its blocks are.
#[must_use]
pub fn styles_inferred(count: usize) -> String {
    let word = if count == 1 {
        "block was"
    } else {
        "blocks were"
    };
    format!(
        "{count} {word} judged a heading, list item, caption or paragraph from its \
         size and position — the PDF does not say which. Check the styles in Word."
    )
}

/// The repeating header and footer became Word's own.
#[must_use]
pub fn running_moved(header: bool, footer: bool, page_number_field: bool) -> String {
    let place = match (header, footer) {
        (true, true) => "Word's own header and footer",
        (true, false) => "Word's own header",
        _ => "Word's own footer",
    };
    if page_number_field {
        format!(
            "The text repeated on every page is in {place}, not the body; the page \
             number is a field Word keeps up to date."
        )
    } else {
        format!("The text repeated on every page is in {place}, not the body.")
    }
}

/// Repeating lines that differed from the kept one, left out.
#[must_use]
pub fn running_variants_dropped(count: usize) -> String {
    let (word, verb) = if count == 1 {
        ("line", "was")
    } else {
        ("lines", "were")
    };
    format!(
        "{count} repeating header or footer {word} differed from the one kept and \
         {verb} left out."
    )
}

/// Tables past Word's column limit, written as paragraphs.
#[must_use]
pub fn tables_too_wide(count: usize) -> String {
    let word = if count == 1 {
        "table has"
    } else {
        "tables have"
    };
    format!("{count} {word} more than Word's 63 columns and arrived as paragraphs.")
}

/// Control characters Word cannot hold, left out.
#[must_use]
pub fn characters_dropped(count: usize) -> String {
    let word = if count == 1 {
        "character"
    } else {
        "characters"
    };
    format!("{count} invisible control {word} a Word document cannot hold were left out.")
}

/// Table detection failed; the export went ahead without tables.
#[must_use]
pub fn tables_not_searched(detail: &str) -> String {
    format!("Tables could not be looked for ({detail}), so any table arrived as paragraphs.")
}

/// Reading the text failed outright.
#[must_use]
pub fn layout_failed(detail: &str) -> String {
    format!("Could not read the document's text: {detail}")
}

/// Writing failed.
#[must_use]
pub fn export_failed(detail: &str) -> String {
    format!("Could not write the Word document: {detail}")
}
