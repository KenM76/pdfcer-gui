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
    "Finds the tables on the chosen pages and writes their rows and columns out. \
     Tables with drawn lines are found from the lines; tables without them are found \
     from how the words line up."
}

/// The format group's heading.
#[must_use]
pub const fn format_heading() -> &'static str {
    "Save as"
}

/// A format's radio label.
#[must_use]
pub const fn format_name(format: crate::tableexport::TableFormat) -> &'static str {
    use crate::tableexport::TableFormat;
    match format {
        TableFormat::Csv => "CSV (.csv)",
        TableFormat::Xlsx => "Excel workbook (.xlsx)",
        TableFormat::Ods => "LibreOffice spreadsheet (.ods)",
    }
}

/// What the chosen format produces.
#[must_use]
pub const fn format_hint(format: crate::tableexport::TableFormat) -> &'static str {
    use crate::tableexport::TableFormat;
    match format {
        TableFormat::Csv => {
            "One file per table, UTF-8 with a byte-order mark so Excel reads symbols \
             correctly. A merged cell's text is in its top-left cell."
        }
        TableFormat::Xlsx | TableFormat::Ods => {
            "One file holding every table. Merged cells stay merged and header rows \
             are bold."
        }
    }
}

/// The heading over the sheet grouping.
#[must_use]
pub const fn sheets_heading() -> &'static str {
    "Sheets"
}

/// One sheet grouping.
#[must_use]
pub const fn sheets_name(sheets: crate::tableexport::SheetGrouping) -> &'static str {
    use crate::tableexport::SheetGrouping;
    match sheets {
        SheetGrouping::PerTable => "A sheet for each table",
        SheetGrouping::PerPage => "A sheet for each page, its tables one under another",
        SheetGrouping::Single => "Every table on one sheet, one under another",
    }
}

/// The heading over the number reading.
#[must_use]
pub const fn numbers_heading() -> &'static str {
    "Write numbers as numbers"
}

/// One number reading.
#[must_use]
pub const fn numbers_name(numbers: crate::tableexport::NumberReading) -> &'static str {
    use crate::tableexport::NumberReading;
    match numbers {
        NumberReading::Auto => "Only where every country reads them the same",
        NumberReading::Us => "Reading 1,234.5 — comma for thousands, point for decimals",
        NumberReading::European => "Reading 1.234,5 — point for thousands, comma for decimals",
        NumberReading::Off => "Never — keep every cell as text",
    }
}

/// The line under the number reading, for the one chosen.
#[must_use]
pub const fn numbers_hint(numbers: crate::tableexport::NumberReading) -> &'static str {
    use crate::tableexport::NumberReading;
    match numbers {
        NumberReading::Auto => {
            "1.234 stays text, because it is a thousand and more in Germany and just \
             over one in Canada. Choose a convention to convert such cells."
        }
        NumberReading::Us | NumberReading::European => {
            "Cells are read with this convention. A cell it cannot read stays text, \
             and so does anything with a leading zero, like 007."
        }
        NumberReading::Off => "Every cell is written as text, exactly as it reads on the page.",
    }
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

/// The receipt for a workbook: `tables` tables on `sheets` sheets of one file.
#[must_use]
pub fn wrote_workbook(path: &str, tables: usize, sheets: usize) -> String {
    if tables == 1 {
        format!("1 table written to {path}.")
    } else if sheets == tables {
        format!("{tables} tables written to {path}, one sheet each.")
    } else if sheets == 1 {
        format!("{tables} tables written to {path}, all on one sheet.")
    } else {
        format!("{tables} tables written to {path}, on {sheets} sheets.")
    }
}

/// Cells a workbook format wrote as numbers rather than text.
#[must_use]
pub fn numbers_written(count: usize) -> String {
    let cell_word = if count == 1 { "cell was" } else { "cells were" };
    format!(
        "{count} {cell_word} written as numbers; anything with units or leading zeros \
         was kept as text."
    )
}

/// Cells whose value depends on the reader's number convention, kept as text.
#[must_use]
pub fn ambiguous_numbers(count: usize) -> String {
    let cell_word = if count == 1 {
        "cell reads"
    } else {
        "cells read"
    };
    format!(
        "{count} {cell_word} as a different number in different countries — 1.234 is \
         one thousand two hundred and thirty-four in Germany and just over one in \
         Canada — so they were kept as text. To convert them, export again and choose \
         how numbers are read."
    )
}

/// Control characters a spreadsheet cannot hold, left out.
#[must_use]
pub fn characters_dropped(count: usize) -> String {
    let word = if count == 1 {
        "character"
    } else {
        "characters"
    };
    format!("{count} invisible control {word} a spreadsheet cannot hold were left out.")
}

/// Cells cut to Excel's per-cell limit.
#[must_use]
pub fn cells_truncated(count: usize) -> String {
    let cell_word = if count == 1 { "cell was" } else { "cells were" };
    format!("{count} {cell_word} cut to Excel's limit of 32,767 characters.")
}

/// Cells past Excel's last row or column, left out.
#[must_use]
pub fn cells_beyond_limits(count: usize) -> String {
    let cell_word = if count == 1 {
        "cell falls"
    } else {
        "cells fall"
    };
    format!("{count} {cell_word} past Excel's last row or column and were left out.")
}

/// A spreadsheet too large for its zip container.
#[must_use]
pub const fn archive_too_large() -> &'static str {
    "the spreadsheet is too large to save as one file"
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
