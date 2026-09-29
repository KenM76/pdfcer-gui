//! # `app::actions::export_word` — File ▸ Export ▸ Word document…
//!
//! The whole document through `pdfcer_core::export::docx::write_docx` with
//! the engine's defaults: reading-order blocks from `analyze_layout`, tables
//! from `detect_tables`, page breaks kept. No window — there is nothing to
//! choose that the receipt cannot disclose afterwards. Every inference (a
//! block's style guessed, running text moved to the header or footer, a table
//! found by alignment) is counted in the receipt, off-canvas (R8b).
//!
//! The refusal runs before the picker: a document with no text would write
//! an empty `.docx`, indistinguishable on disk from a successful export.

use std::path::PathBuf;

use pdfcer_core::block_layout::{self, LayoutOptions, PageGeometry};
use pdfcer_core::export::docx::{self, DocxOptions, DocxReport};
use pdfcer_core::table_detect::{self, BoundarySource, TableOptions};

use crate::app::state::OpenDoc;
use crate::text::export_tables as tt;
use crate::text::export_word as t;

pub(super) fn export(doc: &mut OpenDoc) {
    use crate::app::settings::SettingsExt;

    let options = doc.settings.extract_options();
    let view = doc.session.view();
    let layout = match block_layout::analyze_layout(&view, &options, &LayoutOptions::default()) {
        Ok(layout) => layout,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-word-failed reason=layout detail={error}")
            });
            super::record_note(doc.edit_epoch, t::layout_failed(&error.to_string()));
            return;
        }
    };

    if layout.pages.iter().all(|page| page.blocks.is_empty()) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-word-refused reason=no-text pages={}",
                doc.pages.len()
            )
        });
        let mut notes = vec![t::no_text(doc.pages.len())];
        notes.extend(super::export::honesty_notes(&layout.text.diagnostics));
        super::record_notes(doc.edit_epoch, notes);
        return;
    }

    // Geometry per LAYOUT page, by its own index: `write_docx` pairs
    // `geometry[i]` with `layout.pages[i]`.
    let geometry: Vec<PageGeometry> = layout
        .pages
        .iter()
        .map(|page| {
            doc.pages.get(page.page_index).map_or(
                PageGeometry::new(
                    pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, 612.0, 792.0),
                    0,
                ),
                |page| PageGeometry::new(page.crop_box, page.rotate),
            )
        })
        .collect();

    // Tables are an improvement, not a precondition: a failed search still
    // writes the text, and says the tables arrived as paragraphs.
    let mut notes_after = Vec::new();
    let (tables, aligned, dense, unreadable) =
        match table_detect::detect_tables(&view, &options, &TableOptions::default()) {
            Ok(found) => {
                let aligned = found
                    .tables
                    .iter()
                    .filter(|table| table.source == BoundarySource::Aligned)
                    .count();
                let dense = found.diagnostics.pages_over_limit;
                let unreadable = found.diagnostics.pages_unreadable();
                (found.tables, aligned, dense, unreadable)
            }
            Err(error) => {
                notes_after.push(t::tables_not_searched(&error.to_string()));
                (Vec::new(), 0, 0, 0)
            }
        };

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

    let written = docx::write_docx(&layout, &geometry, &tables, &DocxOptions::default())
        .map_err(|error| error.to_string())
        .and_then(|out| {
            std::fs::write(&target, &out.bytes)
                .map(|()| out.report)
                .map_err(|error| error.to_string())
        });
    match written {
        Ok(report) => {
            trace_report(&report, aligned, &target);
            let mut notes = receipt(&report, &target);
            if aligned > 0 {
                notes.push(tt::aligned_tables(aligned));
            }
            if dense > 0 {
                notes.push(tt::pages_too_dense(dense));
            }
            if unreadable > 0 {
                notes.push(tt::pages_unreadable(unreadable));
            }
            notes.append(&mut notes_after);
            notes.extend(super::export::honesty_notes(&layout.text.diagnostics));
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

fn trace_report(report: &DocxReport, aligned: usize, target: &std::path::Path) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!(
            "export-word pages={} headings={} paragraphs={} list_items={} captions={} \
             inferred={} tables={} cells={} merged={} aligned={aligned} too_wide={} \
             header={} footer={} page_field={} running={} variants_dropped={} \
             dropped_chars={} path={}",
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
            target.display(),
        )
    });
}
