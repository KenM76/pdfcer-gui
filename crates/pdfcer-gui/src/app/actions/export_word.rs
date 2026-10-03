//! # `app::actions::export_word` — File ▸ Export ▸ Word document…
//!
//! The plan's pages through `pdfcer_core::export::docx::write_docx`, with the
//! window's page-break, table and structure choices. A tagged PDF whose tree
//! qualifies gives its own headings, paragraphs, lists and tables
//! (`super::tagged`); otherwise blocks are inferred and tables come from
//! `detect_tables_in_pages`. Every inference (a block's style guessed, running
//! text moved to the header or footer, a table found by alignment) is counted
//! in the receipt, off-canvas (R8b).
//!
//! The refusal runs before the picker: pages with no text would write an
//! empty `.docx`, indistinguishable on disk from a successful export.

use std::path::PathBuf;

use pdfcer_core::export::docx::{self, DocxOptions, DocxReport};
use pdfcer_core::table_detect::{self, BoundarySource, Table, TableOptions};
use pdfcer_core::tagged_layout::FallbackReason;

use super::tagged::{StructureSource, Structured};
use super::wordexport::WordExportPlan;
use crate::app::state::OpenDoc;
use crate::text::export_tables as tt;
use crate::text::export_word as t;

/// The tables to write, and the receipt's counts about how they were found.
#[derive(Default)]
struct Found {
    tables: Vec<Table>,
    aligned: usize,
    dense: usize,
    unreadable: usize,
    /// A failed search: the text is still written, and this says so.
    failed: Option<String>,
}

pub(super) fn export(doc: &mut OpenDoc, plan: &WordExportPlan) {
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
    let structured = match super::tagged::lay_out(
        &view,
        &doc.pages,
        &options,
        Some(&plan.pages),
        plan.structure,
    ) {
        Ok(structured) => structured,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-word-failed reason=layout detail={error}")
            });
            super::record_note(doc.edit_epoch, t::layout_failed(&error.to_string()));
            return;
        }
    };

    let layout = &structured.layout;
    if layout.pages.iter().all(|page| page.blocks.is_empty()) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-word-refused reason=no-text pages={}",
                plan.pages.len()
            )
        });
        let mut notes = vec![t::no_text(plan.pages.len())];
        notes.extend(super::export::honesty_notes(&layout.text.diagnostics));
        super::record_notes(doc.edit_epoch, notes);
        return;
    }

    let found = find_tables(&view, &structured, plan, &options);
    let suggested = suggested_path(&doc.path);
    let crate::app::files::Picked::Path(target) =
        crate::app::files::pick_save_path(&suggested, t::save_dialog_title())
    else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            "export-word-cancelled".to_owned()
        });
        return;
    };
    write(doc, &structured, &found, plan, &target);
}

/// Tables are an improvement, not a precondition: none are searched for when
/// the plan writes tables as text, and a failed search still writes the text.
fn find_tables(
    view: &pdfcer_core::view::DocumentView<'_>,
    structured: &Structured,
    plan: &WordExportPlan,
    options: &pdfcer_core::text_extract::ExtractOptions,
) -> Found {
    if !plan.tables {
        return Found::default();
    }
    if structured.followed() {
        return Found {
            tables: structured.tables.clone(),
            ..Found::default()
        };
    }
    match table_detect::detect_tables_in_pages(view, &plan.pages, options, &TableOptions::default())
    {
        Ok(found) => Found {
            aligned: found
                .tables
                .iter()
                .filter(|table| table.source == BoundarySource::Aligned)
                .count(),
            dense: found.diagnostics.pages_over_limit,
            unreadable: found.diagnostics.pages_unreadable(),
            tables: found.tables,
            failed: None,
        },
        Err(error) => Found {
            failed: Some(error.to_string()),
            ..Found::default()
        },
    }
}

/// Writes the package to `target` and records the receipt or the failure.
fn write(
    doc: &OpenDoc,
    structured: &Structured,
    found: &Found,
    plan: &WordExportPlan,
    target: &std::path::Path,
) {
    let options = DocxOptions::default()
        .with_page_breaks(plan.page_breaks)
        .with_tables(plan.tables);
    let written = docx::write_docx(
        &structured.layout,
        &structured.geometry,
        &found.tables,
        &options,
    )
    .map_err(|error| error.to_string())
    .and_then(|out| {
        std::fs::write(target, &out.bytes)
            .map(|()| out.report)
            .map_err(|error| error.to_string())
    });
    match written {
        Ok(report) => {
            trace_report(&report, found.aligned, structured, &options, target);
            let mut notes = receipt(&report, target);
            notes.extend(super::tagged::notes(&structured.report));
            if plan.structure == StructureSource::Tags
                && structured.report.fallback == Some(FallbackReason::NoStructureTree)
            {
                notes.push(t::no_tags_to_follow().to_owned());
            }
            notes.extend(table_notes(found));
            notes.extend(super::export::honesty_notes(
                &structured.layout.text.diagnostics,
            ));
            super::record_notes(doc.edit_epoch, notes);
        }
        Err(detail) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-word-failed reason=write detail={detail}")
            });
            super::record_note(doc.edit_epoch, t::export_failed(&detail));
        }
    }
}

/// What the table search inferred or could not do.
fn table_notes(found: &Found) -> Vec<String> {
    let mut notes = Vec::new();
    if found.aligned > 0 {
        notes.push(tt::aligned_tables(found.aligned));
    }
    if found.dense > 0 {
        notes.push(tt::pages_too_dense(found.dense));
    }
    if found.unreadable > 0 {
        notes.push(tt::pages_unreadable(found.unreadable));
    }
    if let Some(detail) = &found.failed {
        notes.push(t::tables_not_searched(detail));
    }
    notes
}

/// `<stem>.docx` beside the document. Not `set_extension`: a stem holding a
/// dot would lose everything after it.
fn suggested_path(document: &std::path::Path) -> PathBuf {
    let stem = document
        .file_stem()
        .map_or_else(|| "export".to_owned(), |s| s.to_string_lossy().into_owned()); // ui-text-exempt: a fallback file name
    document.with_file_name(format!("{stem}.docx")) // ui-text-exempt: a file name
}

/// The receipt's sentences from the engine's report, the counts first.
fn receipt(report: &DocxReport, target: &std::path::Path) -> Vec<String> {
    let mut notes = vec![t::wrote(
        &target.display().to_string(),
        report.pages,
        report.headings,
        report.paragraphs.saturating_add(report.list_items),
        report.tables,
    )];
    if report.inferred_blocks > 0 {
        notes.push(t::styles_inferred(report.inferred_blocks));
    }
    if report.header || report.footer {
        notes.push(t::running_moved(
            report.header,
            report.footer,
            report.page_number_field,
        ));
    }
    if report.running_variants_dropped > 0 {
        notes.push(t::running_variants_dropped(report.running_variants_dropped));
    }
    if report.tables_too_wide > 0 {
        notes.push(t::tables_too_wide(report.tables_too_wide));
    }
    if report.characters_dropped > 0 {
        notes.push(t::characters_dropped(report.characters_dropped));
    }
    notes
}

/// The `export-word` line: the engine's report, then the options handed to
/// `write_docx` (`page_breaks`, `table_option`) as the writer received them.
fn trace_report(
    report: &DocxReport,
    aligned: usize,
    structured: &Structured,
    options: &DocxOptions,
    target: &std::path::Path,
) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "export-word pages={} headings={} paragraphs={} list_items={} captions={} \
             inferred={} tables={} cells={} merged={} aligned={aligned} too_wide={} \
             header={} footer={} page_field={} running={} variants_dropped={} \
             dropped_chars={} page_breaks={} table_option={} {} path={}",
            report.pages,
            report.headings,
            report.paragraphs,
            report.list_items,
            report.captions,
            report.inferred_blocks,
            report.tables,
            report.table_cells,
            report.merged_cells,
            report.tables_too_wide,
            u8::from(report.header),
            u8::from(report.footer),
            u8::from(report.page_number_field),
            report.running_blocks,
            report.running_variants_dropped,
            report.characters_dropped,
            u8::from(options.page_breaks),
            u8::from(options.tables),
            super::tagged::trace_fields(&structured.report),
            target.display(),
        )
    });
}
