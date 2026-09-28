//! # `app::actions::export_tables` — File ▸ Export ▸ Tables…
//!
//! Runs `pdfcer_core::table_detect::detect_tables`, keeps the tables on the
//! plan's pages, and writes one CSV per table or one workbook with a sheet
//! per table. Every inference — an aligned table, a guessed header, a merged
//! cell, a cell written as a number — is counted in the receipt, off-canvas
//! (R8b).
//!
//! Detection covers the whole document and the tables are filtered to the
//! plan afterwards: the engine has no page-subset form (request `G061`). The
//! page-level counters (`pages_over_limit`, `pages_unreadable`) are therefore
//! reported only when the plan is every page, since they cannot be attributed
//! to a subset.

use std::path::{Path, PathBuf};

use pdfcer_core::export::xlsx::{self, XlsxOptions, XlsxReport};
use pdfcer_core::table_detect::{self, BoundarySource, Table, TableCell, TableOptions};

use super::tableexport::{self, CellSpec, Grid, Sheet, TableExportPlan, TableFormat};
use crate::app::state::OpenDoc;
use crate::text::export_tables as t;

/// One table ready to write: its name parts and its grid.
struct Named {
    /// 1-based page number.
    page: usize,
    /// 1-based position among the tables on its page.
    nth: usize,
    grid: Grid,
}

/// The counts the receipt reports, over the exported tables only.
#[derive(Default)]
struct Counts {
    aligned: usize,
    headers: usize,
    merged: usize,
}

pub(super) fn export(doc: &mut OpenDoc, plan: &TableExportPlan) {
    use crate::app::settings::SettingsExt;

    if plan.pages.is_empty() {
        super::record_note(
            doc.edit_epoch,
            crate::text::export_text::no_pages().to_owned(),
        );
        return;
    }
    let options = doc.settings.extract_options();
    let found = match table_detect::detect_tables(
        &doc.session.view(),
        &options,
        &TableOptions::default(),
    ) {
        Ok(found) => found,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-tables-failed reason=detect detail={error}")
            });
            super::record_note(doc.edit_epoch, t::detect_failed(&error.to_string()));
            return;
        }
    };

    let mut counts = Counts::default();
    let tables = named_tables(&found.tables, &plan.pages, &mut counts);
    let every_page = plan.pages.len() == doc.pages.len();
    let dense = if every_page {
        found.diagnostics.pages_over_limit
    } else {
        0
    };
    let unreadable = if every_page {
        found.diagnostics.pages_unreadable()
    } else {
        0
    };

    if tables.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-tables-refused reason=no-tables pages={} found_elsewhere={}",
                plan.pages.len(),
                found.tables.len()
            )
        });
        let mut notes = vec![t::no_tables(plan.pages.len())];
        notes.extend(page_notes(dense, unreadable));
        super::record_notes(doc.edit_epoch, notes);
        return;
    }

    let suggested = tableexport::suggested_path(&doc.path, plan.format);
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested, t::save_dialog_title())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-tables-cancelled".to_owned()
        });
        return;
    };

    // Cells written as numbers, and the format's own disclosures.
    let mut numbers = 0;
    let mut format_notes = Vec::new();
    let written = match plan.format {
        TableFormat::Csv => write_csv(&target, &tables),
        TableFormat::Xlsx => write_xlsx(&target, &found.tables, &plan.pages).map(|report| {
            numbers = report.numbers;
            format_notes = xlsx_notes(&report);
            target.clone()
        }),
        TableFormat::Ods => write_ods(&target, &tables).inspect(|_| {
            numbers = tables
                .iter()
                .map(|n| tableexport::numeric_cells(&n.grid))
                .sum();
            let dropped: usize = tables
                .iter()
                .map(|n| tableexport::control_characters(&n.grid))
                .sum();
            if dropped > 0 {
                format_notes.push(t::characters_dropped(dropped));
            }
        }),
    };
    match written {
        Ok(first) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-tables format={} tables={} aligned={} headers={} merged={} \
                     numbers={numbers} dense={dense} unreadable={unreadable} first={}",
                    plan.format.extension(),
                    tables.len(),
                    counts.aligned,
                    counts.headers,
                    counts.merged,
                    first.display(),
                )
            });
            let shown = first.display().to_string();
            let mut notes = vec![if plan.format == TableFormat::Csv {
                t::wrote_csv(&shown, tables.len())
            } else {
                t::wrote_workbook(&shown, tables.len())
            }];
            if numbers > 0 {
                notes.push(t::numbers_written(numbers));
            }
            notes.append(&mut format_notes);
            if counts.aligned > 0 {
                notes.push(t::aligned_tables(counts.aligned));
            }
            if counts.headers > 0 {
                notes.push(t::headers_guessed(counts.headers));
            }
            if counts.merged > 0 {
                notes.push(t::merged_cells(counts.merged));
            }
            notes.extend(page_notes(dense, unreadable));
            super::record_notes(doc.edit_epoch, notes);
        }
        Err(detail) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-tables-failed reason=write detail={detail}")
            });
            super::record_note(doc.edit_epoch, t::export_failed(&detail));
        }
    }
}

/// The page-level disclosures, when there is anything to say.
fn page_notes(dense: usize, unreadable: usize) -> Vec<String> {
    let mut notes = Vec::new();
    if dense > 0 {
        notes.push(t::pages_too_dense(dense));
    }
    if unreadable > 0 {
        notes.push(t::pages_unreadable(unreadable));
    }
    notes
}

/// The tables on `pages`, in the engine's order (page, then top to bottom),
/// each numbered within its page, with the inference counts gathered.
fn named_tables(all: &[Table], pages: &[usize], counts: &mut Counts) -> Vec<Named> {
    let mut out: Vec<Named> = Vec::new();
    for table in all {
        let Table {
            page_index,
            source,
            rows,
            columns,
            cells,
            header_rows,
            header_evidence,
            ..
        } = table;
        if pages.binary_search(page_index).is_err() {
            continue;
        }
        let page = page_index.saturating_add(1);
        let nth = out.iter().filter(|n| n.page == page).count() + 1;
        if *source == BoundarySource::Aligned {
            counts.aligned += 1;
        }
        if header_evidence.is_some() && *header_rows > 0 {
            counts.headers += 1;
        }
        let specs: Vec<CellSpec<'_>> = cells
            .iter()
            .map(
                |TableCell {
                     row,
                     col,
                     row_span,
                     col_span,
                     text,
                     ..
                 }| CellSpec {
                    row: *row,
                    col: *col,
                    row_span: *row_span,
                    col_span: *col_span,
                    text,
                },
            )
            .collect();
        let grid = tableexport::grid(rows.len(), columns.len(), *header_rows, &specs);
        counts.merged += grid.merges.len();
        out.push(Named { page, nth, grid });
    }
    out
}

/// One CSV per table. With one table the picked path is the file; with more,
/// each is a sibling named for its page and position. Returns the first path.
fn write_csv(target: &Path, tables: &[Named]) -> Result<PathBuf, String> {
    let mut first = None;
    for table in tables {
        let path = if tables.len() == 1 {
            target.to_path_buf()
        } else {
            tableexport::csv_sibling(target, table.page, table.nth)
        };
        std::fs::write(&path, tableexport::csv_bytes(&table.grid)).map_err(|e| e.to_string())?;
        first.get_or_insert(path);
    }
    first.ok_or_else(String::new)
}

/// The plan's tables as the engine's Excel workbook at `target`.
fn write_xlsx(target: &Path, all: &[Table], pages: &[usize]) -> Result<XlsxReport, String> {
    let tables: Vec<Table> = all
        .iter()
        .filter(|t| pages.binary_search(&t.page_index).is_ok())
        .cloned()
        .collect();
    let output = xlsx::write_xlsx(&tables, &XlsxOptions::default()).map_err(|e| e.to_string())?;
    std::fs::write(target, &output.bytes).map_err(|e| e.to_string())?;
    Ok(output.report)
}

/// What the Excel writer changed or left out, as receipt lines.
fn xlsx_notes(report: &XlsxReport) -> Vec<String> {
    let mut notes = Vec::new();
    if report.ambiguous_numbers > 0 {
        notes.push(t::ambiguous_numbers(report.ambiguous_numbers));
    }
    if report.characters_dropped > 0 {
        notes.push(t::characters_dropped(report.characters_dropped));
    }
    if report.cells_truncated > 0 {
        notes.push(t::cells_truncated(report.cells_truncated));
    }
    if report.cells_beyond_limits > 0 {
        notes.push(t::cells_beyond_limits(report.cells_beyond_limits));
    }
    notes
}

/// Every table as a sheet of one OpenDocument spreadsheet at `target`,
/// which is returned.
fn write_ods(target: &Path, tables: &[Named]) -> Result<PathBuf, String> {
    let sheets: Vec<Sheet<'_>> = tables
        .iter()
        .enumerate()
        .map(|(i, n)| Sheet {
            name: t::sheet_name(i + 1),
            grid: &n.grid,
        })
        .collect();
    std::fs::write(target, tableexport::ods::bytes(&sheets)?).map_err(|e| e.to_string())?;
    Ok(target.to_path_buf())
}
