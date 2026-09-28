//! The words for **File ▸ Export ▸ Tables…** — the window, and the receipt
//! its export raises. The page-scope strings are `export_text`'s, shared.

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Export tables"
}

/// What the window does, and what it finds.
#[must_use]
pub const fn intro() -> &'static str {
    "Finds the tables on the chosen pages and writes each one as a CSV file of rows \
     and columns. Tables with drawn lines are found from the lines; tables without \
     them are found from how the words line up."
}

/// What the files hold.
#[must_use]
pub const fn format_hint() -> &'static str {
    "One file per table, UTF-8 with a byte-order mark so Excel reads symbols correctly. \
     A merged cell's text is in its top-left cell."
}

/// The Export button.
#[must_use]
pub const fn export_button() -> &'static str {
    "Export…"
}

/// The picker's title.
#[must_use]
pub const fn save_dialog_title() -> &'static str {
    "Export tables"
}

/// The refusal when the pages hold no table.
#[must_use]
pub fn no_tables(pages: usize) -> String {
    let page_word = if pages == 1 { "page" } else { "pages" };
    format!(
        "No tables were found on the {pages} {page_word}, so nothing was written. \
         A table drawn as a picture, or a scanned one, has no lines or words to find."
    )
}

/// The receipt. `first` is the first file's path.
#[must_use]
pub fn wrote_csv(first: &str, files: usize) -> String {
    if files == 1 {
        format!("1 table written to {first}.")
    } else {
        format!("{files} tables written as separate files, starting with {first}.")
    }
}

/// Tables inferred from alignment alone, which deserve a look.
#[must_use]
pub fn aligned_tables(count: usize) -> String {
    let table_word = if count == 1 {
        "table was"
    } else {
        "tables were"
    };
    format!(
        "{count} {table_word} found from how the words line up, not from drawn lines — \
         check their columns."
    )
}

/// Header rows guessed from bold type, a fill or a heavier rule.
#[must_use]
pub fn headers_guessed(count: usize) -> String {
    let table_word = if count == 1 {
        "table's header was"
    } else {
        "tables' headers were"
    };
    format!("{count} {table_word} guessed from bold type, a fill or a heavier rule.")
}

/// Merged cells, whose text lands in their top-left field.
#[must_use]
pub fn merged_cells(count: usize) -> String {
    let cell_word = if count == 1 {
        "cell spans"
    } else {
        "cells span"
    };
    format!(
        "{count} merged {cell_word} more than one row or column; the text is in the \
         top-left field and the rest are left empty."
    )
}

/// Pages skipped because their drawing is too dense to search.
#[must_use]
pub fn pages_too_dense(count: usize) -> String {
    let page_word = if count == 1 { "page has" } else { "pages have" };
    format!("{count} {page_word} too many lines to search for tables and were skipped.")
}

/// Pages whose drawing could not be read.
#[must_use]
pub fn pages_unreadable(count: usize) -> String {
    let page_word = if count == 1 { "page's" } else { "pages'" };
    format!("{count} {page_word} drawing could not be read, so no tables were looked for there.")
}

/// Detection failed outright.
#[must_use]
pub fn detect_failed(detail: &str) -> String {
    format!("Could not look for tables: {detail}")
}

/// Writing failed.
#[must_use]
pub fn export_failed(detail: &str) -> String {
    format!("Could not write the tables: {detail}")
}
