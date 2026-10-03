//! # `tableexport` — detected tables as CSV, Excel or OpenDocument
//!
//! Pure: the plan, the cell grid, and the CSV encoding. Both workbook
//! formats are the engine's (`pdfcer_core::export::{xlsx, ods}`), called from
//! the export action. CSV is one file per table; a workbook holds every table,
//! grouped onto sheets as [`SheetGrouping`] says.
//! `pdfcer_core::table_detect` finds the tables;
//! `pdfcer_gui::app::actions::export_tables` writes them.

use std::path::{Path, PathBuf};

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

/// Which tables share a workbook sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetGrouping {
    /// A sheet for each table.
    #[default]
    PerTable,
    /// A sheet for each page with a table, its tables stacked.
    PerPage,
    /// Every table stacked on one sheet.
    Single,
}

impl SheetGrouping {
    /// Every choice, in the order the window offers them.
    pub const ALL: [Self; 3] = [Self::PerTable, Self::PerPage, Self::Single];

    /// The engine's setting for this choice.
    #[must_use]
    pub const fn engine(self) -> pdfcer_core::export::xlsx::SheetLayout {
        use pdfcer_core::export::xlsx::SheetLayout;
        match self {
            Self::PerTable => SheetLayout::PerTable,
            Self::PerPage => SheetLayout::PerPage,
            Self::Single => SheetLayout::Single,
        }
    }

    /// The token in the trace and the preferences file.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: file and trace tokens, never displayed.
            Self::PerTable => "table",
            Self::PerPage => "page",
            Self::Single => "single",
        }
    }

    /// The choice `token` names, or `None` if it names none.
    #[must_use]
    pub fn from_key(token: &str) -> Option<Self> {
        let token = token.trim();
        Self::ALL.into_iter().find(|s| s.key() == token)
    }
}

/// How a workbook reads a cell's digits and separators as a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberReading {
    /// A number only where every convention reads the same value.
    #[default]
    Auto,
    /// `,` groups thousands and `.` is the decimal point.
    Us,
    /// `.` groups thousands and `,` is the decimal point.
    European,
    /// Every cell is text.
    Off,
}

impl NumberReading {
    /// Every choice, in the order the window offers them.
    pub const ALL: [Self; 4] = [Self::Auto, Self::Us, Self::European, Self::Off];

    /// The engine's setting for this choice.
    #[must_use]
    pub const fn engine(self) -> pdfcer_core::export::xlsx::NumberLocale {
        use pdfcer_core::export::xlsx::NumberLocale;
        match self {
            Self::Auto => NumberLocale::Auto,
            Self::Us => NumberLocale::Us,
            Self::European => NumberLocale::European,
            Self::Off => NumberLocale::Off,
        }
    }

    /// The token in the trace and the preferences file; the command line's
    /// `--numbers` word.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            // ui-text-exempt: file and trace tokens, never displayed.
            Self::Auto => "auto",
            Self::Us => "us",
            Self::European => "european",
            Self::Off => "off",
        }
    }

    /// The choice `token` names, or `None` if it names none.
    #[must_use]
    pub fn from_key(token: &str) -> Option<Self> {
        let token = token.trim();
        Self::ALL.into_iter().find(|n| n.key() == token)
    }
}

/// Everything the table-export window decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableExportPlan {
    /// 0-based page indices, ascending and unique.
    pub pages: Vec<usize>,
    /// What to write.
    pub format: TableFormat,
    /// Which tables share a sheet; workbooks only.
    pub sheets: SheetGrouping,
    /// How cells are read as numbers; workbooks only.
    pub numbers: NumberReading,
}

impl TableExportPlan {
    /// A plan writing `pages`, sorted and deduplicated, as `format`, with the
    /// engine's sheet and number defaults.
    #[must_use]
    pub fn new(mut pages: Vec<usize>, format: TableFormat) -> Self {
        pages.sort_unstable();
        pages.dedup();
        Self {
            pages,
            format,
            sheets: SheetGrouping::default(),
            numbers: NumberReading::default(),
        }
    }
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
    fn each_choice_has_its_own_engine_setting_and_token() {
        use pdfcer_core::export::xlsx::{NumberLocale, SheetLayout, XlsxOptions};
        assert_eq!(
            SheetGrouping::ALL.map(SheetGrouping::engine),
            [
                SheetLayout::PerTable,
                SheetLayout::PerPage,
                SheetLayout::Single
            ]
        );
        assert_eq!(
            NumberReading::ALL.map(NumberReading::engine),
            [
                NumberLocale::Auto,
                NumberLocale::Us,
                NumberLocale::European,
                NumberLocale::Off
            ]
        );
        for s in SheetGrouping::ALL {
            assert_eq!(SheetGrouping::from_key(s.key()), Some(s));
        }
        for n in NumberReading::ALL {
            assert_eq!(NumberReading::from_key(n.key()), Some(n));
        }
        let engine = XlsxOptions::default();
        assert_eq!(SheetGrouping::default().engine(), engine.sheets);
        assert_eq!(NumberReading::default().engine(), engine.numbers);
    }

    #[test]
    fn a_plans_pages_are_in_document_order_once_each() {
        let plan = TableExportPlan::new(vec![3, 1, 1], TableFormat::Xlsx);
        assert_eq!(plan.pages, vec![1, 3]);
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
}
