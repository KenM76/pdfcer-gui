//! # `app::actions::export_tables` — File ▸ Export ▸ Tables…
//!
//! Takes the tables a tagged PDF states when its tree is followed
//! (`super::tagged`), else runs `pdfcer_core::table_detect::detect_tables_in_pages`
//! on the plan's pages, and writes one CSV per table or one workbook with a sheet per table,
//! through the engine's Excel and OpenDocument writers. Every inference — an aligned table, a guessed header, a merged
//! cell, a cell written as a number — is counted in the receipt, off-canvas
//! (R8b).

use std::path::{Path, PathBuf};

use pdfcer_core::export::ods::{self, OdsOptions};
use pdfcer_core::export::xlsx::{self, XlsxOptions};
use pdfcer_core::table_detect::{
    self, BoundarySource, HeaderEvidence, Table, TableCell, TableOptions,
};

use super::tableexport::{self, CellSpec, Grid, TableExportPlan, TableFormat};
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
    let view = doc.session.view();
    // An untagged file skips the tree read, which extracts every page.
    let structured = pdfcer_core::structure_tree::has_structure_tree(&view)
        .then(|| super::tagged::lay_out(&view, &doc.pages, &options, Some(&plan.pages)).ok())
        .flatten();
    let found = match structured.as_ref().filter(|s| s.followed()) {
        Some(tree) => Ok((tree.tables.clone(), 0, 0)),
        None => table_detect::detect_tables_in_pages(
            &view,
            &plan.pages,
            &options,
            &TableOptions::default(),
        )
        .map(|found| {
            let dense = found.diagnostics.pages_over_limit;
            let unreadable = found.diagnostics.pages_unreadable();
            (found.tables, dense, unreadable)
        }),
    };
    let (found_tables, dense, unreadable) = match found {
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
    let tables = named_tables(&found_tables, &mut counts);
    let tree_notes = structured
        .as_ref()
        .map(|s| super::tagged::notes(&s.report))
        .unwrap_or_default();
    let tree_trace = structured.as_ref().map_or_else(
        || "structure=untagged".to_owned(),
        |s| super::tagged::trace_fields(&s.report),
    ); // ui-text-exempt: a trace field, never displayed

    if tables.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-tables-refused reason=no-tables pages={}",
                plan.pages.len()
            )
        });
        let mut notes = vec![t::no_tables(plan.pages.len())];
        notes.extend(tree_notes);
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
        TableFormat::Xlsx => write_workbook(&target, || {
            xlsx::write_xlsx(&found_tables, &XlsxOptions::default())
        })
        .map(|report| {
            numbers = report.numbers;
            format_notes = workbook_notes(
                report.ambiguous_numbers,
                report.characters_dropped,
                report.cells_truncated,
                report.cells_beyond_limits,
            );
            target.clone()
        }),
        TableFormat::Ods => write_workbook(&target, || {
            ods::write_ods(&found_tables, &OdsOptions::default())
        })
        .map(|report| {
            numbers = report.numbers;
            format_notes = workbook_notes(
                report.ambiguous_numbers,
                report.characters_dropped,
                0,
                report.cells_beyond_limits,
            );
            target.clone()
        }),
    };
    match written {
        Ok(first) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-tables format={} tables={} aligned={} headers={} merged={} \
                     numbers={numbers} dense={dense} unreadable={unreadable} {tree_trace} first={}",
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
            notes.extend(tree_notes);
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

/// The detected tables, in the engine's order (page, then top to bottom),
/// each numbered within its page, with the inference counts gathered.
fn named_tables(all: &[Table], counts: &mut Counts) -> Vec<Named> {
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
        let page = page_index.saturating_add(1);
        let nth = out.iter().filter(|n| n.page == page).count() + 1;
        if *source == BoundarySource::Aligned {
            counts.aligned += 1;
        }
        // A tagged header is stated by the file, not guessed.
        if header_evidence.is_some_and(|e| e != HeaderEvidence::Tagged) && *header_rows > 0 {
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

/// Writes the package `write` produces to `target`, returning what the
/// writer reported.
fn write_workbook<O: Workbook>(
    target: &Path,
    write: impl FnOnce() -> Result<O, pdfcer_core::export::PackageError>,
) -> Result<O::Report, String> {
    let (bytes, report) = write().map_err(|e| e.to_string())?.into_parts();
    std::fs::write(target, bytes).map_err(|e| e.to_string())?;
    Ok(report)
}

/// A written workbook, as its bytes and its writer's report.
trait Workbook {
    type Report;
    fn into_parts(self) -> (Vec<u8>, Self::Report);
}

impl Workbook for xlsx::XlsxOutput {
    type Report = xlsx::XlsxReport;
    fn into_parts(self) -> (Vec<u8>, Self::Report) {
        (self.bytes, self.report)
    }
}

impl Workbook for ods::OdsOutput {
    type Report = ods::OdsReport;
    fn into_parts(self) -> (Vec<u8>, Self::Report) {
        (self.bytes, self.report)
    }
}

/// What a workbook writer changed or left out, as receipt lines. The
/// OpenDocument writer truncates nothing (ODF has no cell length limit), so
/// it passes zero for `truncated`.
fn workbook_notes(
    ambiguous: usize,
    dropped: usize,
    truncated: usize,
    beyond_limits: usize,
) -> Vec<String> {
    let mut notes = Vec::new();
    if ambiguous > 0 {
        notes.push(t::ambiguous_numbers(ambiguous));
    }
    if dropped > 0 {
        notes.push(t::characters_dropped(dropped));
    }
    if truncated > 0 {
        notes.push(t::cells_truncated(truncated));
    }
    if beyond_limits > 0 {
        notes.push(t::cells_beyond_limits(beyond_limits));
    }
    notes
}
