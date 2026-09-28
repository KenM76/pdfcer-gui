//! # `tableexport` — detected tables as CSV, Excel or OpenDocument
//!
//! Pure: the plan, the cell grid, and the CSV and OpenDocument encodings.
//! Excel is the engine's (`pdfcer_core::export::xlsx`), called from the
//! export action. CSV is one file per table; a workbook holds every table,
//! one sheet each.
//! `pdfcer_core::table_detect` finds the tables;
//! `pdfcer_gui::app::actions::export_tables` writes them.

use std::path::{Path, PathBuf};

pub mod ods;

/// What the tables are written as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableFormat {
    /// One UTF-8 CSV file per table.
    #[default]
    Csv,
    /// One Excel workbook, a sheet per table.
    Xlsx,
    /// One OpenDocument spreadsheet (LibreOffice Calc), a sheet per table.
    Ods,
}

impl TableFormat {
    /// Every format, in the order the window offers them.
    pub const ALL: [Self; 3] = [Self::Csv, Self::Xlsx, Self::Ods];

    /// The file extension, without the dot.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            // ui-text-exempt: file extensions.
            Self::Csv => "csv",
            Self::Xlsx => "xlsx",
            Self::Ods => "ods",
        }
    }
}

/// Everything the table-export window decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableExportPlan {
    /// 0-based page indices, ascending and unique.
    pub pages: Vec<usize>,
    /// What to write.
    pub format: TableFormat,
}

/// One worksheet of a workbook export.
pub struct Sheet<'a> {
    /// The tab's name; at most 31 characters, none of `[]:*?/\`.
    pub name: String,
    pub grid: &'a Grid,
}

/// The value of a cell the OpenDocument export writes as a number, or `None`
/// to keep it text. Deliberately narrow: an optional minus, digits with no
/// leading zero, an optional decimal part, at most 15 significant digits.
/// `007`, `1,200`, `1/2`, `+5` and a 20-digit part number stay text, because
/// turning them into numbers would change what they say.
#[must_use]
pub fn number(text: &str) -> Option<f64> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = match digits.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (digits, None),
    };
    let all_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !all_digits(whole) || (whole.len() > 1 && whole.starts_with('0')) {
        return None;
    }
    if fraction.is_some_and(|f| !all_digits(f)) {
        return None;
    }
    let significant = whole.trim_start_matches('0').len() + fraction.map_or(0, str::len);
    if significant > 15 {
        return None;
    }
    text.parse().ok()
}

/// How many cells of `grid` [`number`] turns into numbers.
#[must_use]
pub fn numeric_cells(grid: &Grid) -> usize {
    grid.cells
        .iter()
        .flatten()
        .filter(|t| number(t).is_some())
        .count()
}

/// How many characters of `grid` XML cannot carry, which the OpenDocument
/// export leaves out: the C0 controls other than tab, newline and return.
#[must_use]
pub fn control_characters(grid: &Grid) -> usize {
    grid.cells
        .iter()
        .flatten()
        .flat_map(|t| t.chars())
        .filter(|&c| (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r'))
        .count()
}

/// A merged block, by its top-left cell and its extent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Merge {
    /// Top row, 0-based.
    pub row: usize,
    /// Left column, 0-based.
    pub col: usize,
    /// Rows covered, at least 1.
    pub rows: usize,
    /// Columns covered, at least 1.
    pub cols: usize,
}

/// One table as a full rectangle of strings, with its merged blocks.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Grid {
    /// `cells[row][col]`; a position a merge covers, or no cell names, is empty.
    pub cells: Vec<Vec<String>>,
    /// Every cell spanning more than one position.
    pub merges: Vec<Merge>,
    /// Rows at the top that read as a header.
    pub header_rows: usize,
}

/// One detected cell, reduced to what a grid needs.
#[derive(Debug, Clone, Copy)]
pub struct CellSpec<'a> {
    /// Top row, 0-based.
    pub row: usize,
    /// Left column, 0-based.
    pub col: usize,
    /// Rows covered.
    pub row_span: usize,
    /// Columns covered.
    pub col_span: usize,
    /// The words in it; `\n` between lines.
    pub text: &'a str,
}

/// Lays `cells` into a `rows` × `cols` rectangle. A cell outside it is
/// dropped rather than growing the grid: the band counts are the table's.
#[must_use]
pub fn grid(rows: usize, cols: usize, header_rows: usize, cells: &[CellSpec<'_>]) -> Grid {
    let mut out = vec![vec![String::new(); cols]; rows];
    let mut merges = Vec::new();
    for c in cells {
        let Some(slot) = out.get_mut(c.row).and_then(|r| r.get_mut(c.col)) else {
            continue;
        };
        c.text.clone_into(slot);
        let rows_covered = c.row_span.max(1).min(rows - c.row);
        let cols_covered = c.col_span.max(1).min(cols - c.col);
        if rows_covered > 1 || cols_covered > 1 {
            merges.push(Merge {
                row: c.row,
                col: c.col,
                rows: rows_covered,
                cols: cols_covered,
            });
        }
    }
    Grid {
        cells: out,
        merges,
        header_rows: header_rows.min(rows),
    }
}

/// RFC 4180 text for `grid`: CRLF rows, a field quoted when it holds a comma,
/// a quote, a line break or edge spaces. A multi-line cell stays one field.
#[must_use]
pub fn csv(grid: &Grid) -> String {
    let mut out = String::new();
    for row in &grid.cells {
        for (i, field) in row.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let quote = field.contains([',', '"', '\n', '\r'])
                || field.starts_with(' ')
                || field.ends_with(' ');
            if quote {
                out.push('"');
                out.push_str(&field.replace('"', "\"\"")); // ui-text-exempt: CSV quote escaping
                out.push('"');
            } else {
                out.push_str(field);
            }
        }
        out.push_str("\r\n");
    }
    out
}

/// The CSV file's bytes: a UTF-8 byte-order mark, which is what makes Excel
/// read the file as UTF-8 rather than the machine's code page, then the text.
#[must_use]
pub fn csv_bytes(grid: &Grid) -> Vec<u8> {
    let text = csv(grid);
    let mut bytes = Vec::with_capacity(text.len() + 3);
    bytes.extend_from_slice(b"\xEF\xBB\xBF");
    bytes.extend_from_slice(text.as_bytes());
    bytes
}

/// The picker's suggestion: the document's name with `format`'s extension.
#[must_use]
pub fn suggested_path(document: &Path, format: TableFormat) -> PathBuf {
    let stem = document
        .file_stem()
        .map_or_else(|| "tables".to_owned(), |s| s.to_string_lossy().into_owned()); // ui-text-exempt: a fallback file name
    document.with_file_name(format!("{stem}.{}", format.extension())) // ui-text-exempt: a file name
}

/// The path of one table's CSV when the export writes more than one:
/// `plan.csv` → `plan-p3-t2.csv`, the second table on page 3 (both 1-based).
/// With a single table the picked path is used as it is.
#[must_use]
pub fn csv_sibling(picked: &Path, page: usize, nth_on_page: usize) -> PathBuf {
    let stem = picked
        .file_stem()
        .map_or_else(|| "tables".to_owned(), |s| s.to_string_lossy().into_owned()); // ui-text-exempt: a fallback file name
    picked.with_file_name(format!("{stem}-p{page}-t{nth_on_page}.csv")) // ui-text-exempt: a file name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_merged_cell_fills_its_corner_and_leaves_the_rest_empty() {
        let cells = [
            CellSpec {
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 2,
                text: "Title",
            },
            CellSpec {
                row: 1,
                col: 0,
                row_span: 1,
                col_span: 1,
                text: "a",
            },
            CellSpec {
                row: 1,
                col: 1,
                row_span: 1,
                col_span: 1,
                text: "b",
            },
            CellSpec {
                row: 9,
                col: 0,
                row_span: 1,
                col_span: 1,
                text: "outside",
            },
        ];
        let g = grid(2, 2, 1, &cells);
        assert_eq!(
            g.cells,
            vec![
                vec!["Title".to_owned(), String::new()],
                vec!["a".into(), "b".into()]
            ]
        );
        assert_eq!(
            g.merges,
            vec![Merge {
                row: 0,
                col: 0,
                rows: 1,
                cols: 2
            }]
        );
        assert_eq!(g.header_rows, 1);
    }

    #[test]
    fn csv_quotes_only_what_would_break_a_field() {
        let g = Grid {
            cells: vec![vec![
                "a".into(),
                "b,c".into(),
                "say \"hi\"".into(),
                "two\nlines".into(),
                " pad".into(),
            ]],
            ..Grid::default()
        };
        assert_eq!(
            csv(&g),
            "a,\"b,c\",\"say \"\"hi\"\"\",\"two\nlines\",\" pad\"\r\n"
        );
        assert!(csv_bytes(&g).starts_with(b"\xEF\xBB\xBF"));
    }

    #[test]
    fn paths_carry_page_and_order() {
        let p = csv_sibling(Path::new(r"C:\jobs\plan.rev2.csv"), 3, 2);
        assert_eq!(p.file_name().unwrap(), "plan.rev2-p3-t2.csv");
        let s = suggested_path(Path::new(r"C:\jobs\plan.rev2.pdf"), TableFormat::Ods);
        assert_eq!(s.file_name().unwrap(), "plan.rev2.ods");
    }

    #[test]
    fn only_plain_decimals_become_numbers() {
        for yes in ["0", "12", "-3.25", "0.5", "123456789012345"] {
            assert!(number(yes).is_some(), "{yes}");
        }
        for no in [
            "",
            "007",
            "1,200",
            "1/2",
            "+5",
            "1e3",
            ".5",
            "5.",
            "-",
            "1234567890123456",
            "12 mm",
        ] {
            assert!(number(no).is_none(), "{no}");
        }
    }
}
