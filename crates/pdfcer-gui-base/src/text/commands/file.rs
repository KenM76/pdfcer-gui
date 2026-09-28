//! # `text::commands::file` — the File tab's command copy
//!
//!
//! Only the entries that need to be here have moved. The rest of the File
//! tab's copy stays in [`super`] until the next thing pushes it over, because
//! moving text that nothing is asking about would be churn dressed as tidying —
//! and every move is a chance for a string to lose its `ui-text-exempt` context
//! or its place in a gate's scan.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/commands/file.md`.

use super::CommandText;

/// **Export image** — `OPERATOR_REQUESTS.md` **O120**, the operator's own
/// ask of 2026-09-03: *"can you add the ability to export page(es) to png, jpg,
/// svg."*
pub const fn file_export_image() -> CommandText {
    CommandText::new(
        "Export image…",
        "Save pages as PNG, JPEG or SVG pictures, at a resolution you choose \
         and with the page's transparency kept where the format allows it.",
    )
}

/// **Export text** — the operator's ask of 2026-09-04: *"also the engine
/// can export PDFs as text. we should have export/import for that."*
pub const fn file_export_text() -> CommandText {
    CommandText::new(
        "Export text…",
        "Write the words on the page — or on every page — to a plain text file, \
         encoded as UTF-8. Only the words travel: layout, fonts and position do \
         not, so a table arrives as lines. A scanned page carries no words to \
         export; use Recognise text on it first.",
    )
}

/// **Export tables** — the tables on the page as spreadsheet cells.
pub const fn file_export_tables() -> CommandText {
    CommandText::new(
        "Export tables…",
        "Find the tables on the chosen pages and write each one as a CSV file of \
         rows and columns. Tables with drawn lines and tables laid out by \
         alignment are both found; the receipt says which were guessed.",
    )
}

/// **Import text** — the return journey, `pdfcer-core` `Pass 252.0`.
pub const fn file_import_text() -> CommandText {
    CommandText::new(
        "Import text as pages…",
        "Turn a plain text file into new pages and add them to this document. \
         The words are set in one standard font at one size — this makes pages \
         you can read and search, not a copy of the original layout. You choose \
         the sheet size, the margins and where the new pages go.",
    )
}

/// **Save As** — `OPERATOR_REQUESTS.md` O95.
pub const fn file_save_as() -> CommandText {
    CommandText::new(
        "Save as…",
        "Write the document to a file you choose, and carry on editing THAT \
         file — the next Save goes to it, not to the original. Use Save a copy \
         instead when you want a snapshot to send somewhere and want to keep \
         editing the file you already have.",
    )
}
