//! # `dialogs::export_tables` — File ▸ Export ▸ Tables…
//!
//! Which pages, what to write, and for a workbook how its tables share
//! sheets and how its cells are read as numbers. The detection runs on
//! Export, so the window cannot say how many tables there are; the receipt
//! does. Every choice but a typed range is remembered on Export.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::imageexport::{PageScope, resolve_pages};
use crate::app::actions::tableexport::{
    NumberReading, SheetGrouping, TableExportPlan, TableFormat,
};
use crate::app::state::{OpenDoc, Status};
use crate::text::export_tables as t;
use crate::text::export_text as tt;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:export-tables"; // ui-text-exempt: trace region name, never displayed
/// Height kept below the scrolling body for the separator and button row.
const FOOTER_PTS: f32 = 44.0;
/// The body's least height: a negative `max_height` draws nothing at all.
const BODY_FLOOR_PTS: f32 = 80.0;

/// The region the Export button publishes.
pub const REGION_EXPORT: &str = "export-tables.export"; // ui-text-exempt: trace region name, never displayed
/// The region the typed-range field publishes.
pub const REGION_RANGE: &str = "export-tables.pages.range"; // ui-text-exempt: trace region name, never displayed

/// The region ONE page-scope radio publishes.
#[must_use]
pub const fn region_for_scope(scope: PageScope) -> &'static str {
    match scope {
        // ui-text-exempt: trace region names, never displayed.
        PageScope::AllPages => "export-tables.pages.all",
        PageScope::CurrentPage => "export-tables.pages.current",
        PageScope::Typed => "export-tables.pages.typed",
    }
}

/// The region ONE format radio publishes.
#[must_use]
pub const fn region_for_format(format: TableFormat) -> &'static str {
    match format {
        // ui-text-exempt: trace region names, never displayed.
        TableFormat::Csv => "export-tables.format.csv",
        TableFormat::Xlsx => "export-tables.format.xlsx",
        TableFormat::Ods => "export-tables.format.ods",
    }
}

/// The region ONE sheet-grouping radio publishes.
#[must_use]
pub const fn region_for_sheets(sheets: SheetGrouping) -> &'static str {
    match sheets {
        // ui-text-exempt: trace region names, never displayed.
        SheetGrouping::PerTable => "export-tables.sheets.table",
        SheetGrouping::PerPage => "export-tables.sheets.page",
        SheetGrouping::Single => "export-tables.sheets.single",
    }
}

/// The region ONE number-reading radio publishes.
#[must_use]
pub const fn region_for_numbers(numbers: NumberReading) -> &'static str {
    match numbers {
        // ui-text-exempt: trace region names, never displayed.
        NumberReading::Auto => "export-tables.numbers.auto",
        NumberReading::Us => "export-tables.numbers.us",
        NumberReading::European => "export-tables.numbers.european",
        NumberReading::Off => "export-tables.numbers.off",
    }
}

/// The Export-tables window's live state.
pub struct ExportTablesDialog {
    /// The page on screen at open, frozen so paging behind the window does
    /// not change what *This page* means.
    page_index: usize,
    /// Page count, frozen with `page_index`.
    page_count: usize,
    scope: PageScope,
    format: TableFormat,
    sheets: SheetGrouping,
    numbers: NumberReading,
    /// The typed range; not remembered, since it names this document's pages.
    range_text: String,
    export_requested: bool,
    close_requested: bool,
}

impl ExportTablesDialog {
    /// Open for the document on screen, seeded from the last table export.
    #[must_use]
    pub fn open(doc: &OpenDoc, remembered: &crate::app::prefs::ExportTablePrefs) -> Self {
        let dialog = Self {
            page_index: doc.view.page_index,
            page_count: doc.pages.len(),
            scope: remembered.scope,
            format: remembered.format,
            sheets: remembered.sheets,
            numbers: remembered.numbers,
            range_text: String::new(),
            export_requested: false,
            close_requested: false,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-tables-open page={} pages={} scope={} format={} sheets={} numbers={}",
                dialog.page_index,
                dialog.page_count,
                crate::app::prefs::exporting::page_scope_key_or(
                    dialog.scope,
                    crate::app::prefs::ExportTablePrefs::default().scope,
                ),
                dialog.format.extension(),
                dialog.sheets.key(),
                dialog.numbers.key(),
            )
        });
        dialog
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        actions: &mut Vec<Action>,
        prefs: &mut crate::app::prefs::Prefs,
    ) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "export-tables", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(460.0, 720.0),
            egui::vec2(340.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            // The body scrolls above the button row, so Export stays on screen
            // when the workbook choices make the body taller than the window.
            let pages = egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height((ui.available_height() - FOOTER_PTS).max(BODY_FLOOR_PTS))
                .show(ui, |ui| self.body(ui))
                .inner;
            self.footer(ui, pages.is_some());
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.export_requested)
            && let Some(pages) = self.pages()
        {
            self.request(pages, actions, prefs);
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// Remembers the choices and raises the export of `pages`.
    fn request(
        &self,
        pages: Vec<usize>,
        actions: &mut Vec<Action>,
        prefs: &mut crate::app::prefs::Prefs,
    ) {
        crate::dialogs::export_remembered::remember_tables(
            crate::app::prefs::ExportTablePrefs {
                scope: self.scope,
                format: self.format,
                sheets: self.sheets,
                numbers: self.numbers,
            },
            prefs,
        );
        let mut plan = TableExportPlan::new(pages, self.format);
        plan.sheets = self.sheets;
        plan.numbers = self.numbers;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-tables-requested pages={} format={} sheets={} numbers={}",
                plan.pages.len(),
                plan.format.extension(),
                plan.sheets.key(),
                plan.numbers.key(),
            )
        });
        actions.push(Action::Write(
            crate::app::actions::write::WriteAction::Tables { plan },
        ));
    }

    /// The pages offered, or `None` when the typed range names none.
    fn pages(&self) -> Option<Vec<usize>> {
        resolve_pages(
            self.scope,
            &self.range_text,
            self.page_count,
            self.page_index,
        )
    }

    /// Everything above the button row; returns the pages offered.
    fn body(&mut self, ui: &mut Ui) -> Option<Vec<usize>> {
        ui.label(t::intro());
        ui.add_space(8.0);
        let pages = self.pages_section(ui);
        ui.add_space(8.0);

        ui.label(t::format_heading());
        for format in TableFormat::ALL {
            let response = ui.radio_value(&mut self.format, format, t::format_name(format));
            crate::diag::ui_rect(region_for_format(format), response.rect);
        }
        ui.weak(t::format_hint(self.format));
        ui.add_space(8.0);
        // A CSV is one table per file of plain text: neither choice applies.
        if self.format != TableFormat::Csv {
            self.workbook_section(ui);
        }
        pages
    }

    /// Export and Cancel, below the scrolling body.
    fn footer(&mut self, ui: &mut Ui, has_pages: bool) {
        ui.separator();
        ui.horizontal(|ui| {
            let response = ui.add_enabled(has_pages, egui::Button::new(t::export_button()));
            crate::diag::ui_rect(REGION_EXPORT, response.rect);
            if response.clicked() {
                self.export_requested = true;
            }
            if ui.button(tt::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// The page-scope radios and the range field; returns the pages offered.
    fn pages_section(&mut self, ui: &mut Ui) -> Option<Vec<usize>> {
        ui.label(tt::pages_heading());
        let response = ui.radio_value(
            &mut self.scope,
            PageScope::AllPages,
            tt::pages_all(self.page_count),
        );
        crate::diag::ui_rect(region_for_scope(PageScope::AllPages), response.rect);
        let response = ui.radio_value(
            &mut self.scope,
            PageScope::CurrentPage,
            tt::pages_current(self.page_index.saturating_add(1)),
        );
        crate::diag::ui_rect(region_for_scope(PageScope::CurrentPage), response.rect);
        ui.horizontal(|ui| {
            let response = ui.radio_value(&mut self.scope, PageScope::Typed, tt::pages_range());
            crate::diag::ui_rect(region_for_scope(PageScope::Typed), response.rect);
            // Typing selects the radio, so a typed range is never ignored.
            let field = ui.text_edit_singleline(&mut self.range_text);
            crate::diag::ui_rect(REGION_RANGE, field.rect);
            if field.changed() {
                self.scope = PageScope::Typed;
            }
        });
        ui.weak(tt::pages_range_hint());
        let pages = self.pages();
        if pages.is_none() && self.scope == PageScope::Typed {
            ui.label(tt::pages_range_invalid(self.page_count));
        }
        pages
    }

    /// The sheet grouping and number reading, which only a workbook has.
    fn workbook_section(&mut self, ui: &mut Ui) {
        ui.label(t::sheets_heading());
        for sheets in SheetGrouping::ALL {
            let response = ui.radio_value(&mut self.sheets, sheets, t::sheets_name(sheets));
            crate::diag::ui_rect(region_for_sheets(sheets), response.rect);
        }
        ui.add_space(8.0);
        ui.label(t::numbers_heading());
        for numbers in NumberReading::ALL {
            let response = ui.radio_value(&mut self.numbers, numbers, t::numbers_name(numbers));
            crate::diag::ui_rect(region_for_numbers(numbers), response.rect);
        }
        ui.weak(t::numbers_hint(self.numbers));
        ui.add_space(8.0);
    }
}

/// Open the window for `status`, or decline.
#[must_use]
pub fn open_for(
    status: &Status,
    remembered: &crate::app::prefs::ExportTablePrefs,
) -> Option<ExportTablesDialog> {
    match status {
        Status::Open(doc) if !doc.pages.is_empty() => {
            Some(ExportTablesDialog::open(doc, remembered))
        }
        _ => None,
    }
}
