//! # `dialogs::export_tables` — File ▸ Export ▸ Tables…
//!
//! Which pages. The detection runs on Export, so the window cannot say how
//! many tables there are; the receipt does. The scope is remembered on
//! Export, as the other export windows remember theirs.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::imageexport::{PageScope, resolve_pages};
use crate::app::actions::tableexport::TableExportPlan;
use crate::app::state::{OpenDoc, Status};
use crate::text::export_tables as t;
use crate::text::export_text as tt;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:export-tables"; // ui-text-exempt: trace region name, never displayed
/// The region the Export button publishes.
pub const REGION_EXPORT: &str = "export-tables.export"; // ui-text-exempt: trace region name, never displayed

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

/// The Export-tables window's live state.
pub struct ExportTablesDialog {
    /// The page on screen at open, frozen so paging behind the window does
    /// not change what *This page* means.
    page_index: usize,
    /// Page count, frozen with `page_index`.
    page_count: usize,
    scope: PageScope,
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
            range_text: String::new(),
            export_requested: false,
            close_requested: false,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-tables-open page={} pages={} scope={}",
                dialog.page_index,
                dialog.page_count,
                crate::app::prefs::exporting::page_scope_key_or(
                    dialog.scope,
                    crate::app::prefs::ExportTablePrefs::default().scope,
                ),
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
            egui::vec2(440.0, 340.0),
            egui::vec2(340.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.export_requested)
            && let Some(pages) = self.pages()
        {
            crate::dialogs::export_remembered::remember_tables(
                crate::app::prefs::ExportTablePrefs { scope: self.scope },
                prefs,
            );
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("export-tables-requested pages={}", pages.len(),)
            });
            actions.push(Action::Write(
                crate::app::actions::write::WriteAction::Tables {
                    plan: TableExportPlan { pages },
                },
            ));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
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

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);

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
            if ui.text_edit_singleline(&mut self.range_text).changed() {
                self.scope = PageScope::Typed;
            }
        });
        ui.weak(tt::pages_range_hint());
        let pages = self.pages();
        if pages.is_none() && self.scope == PageScope::Typed {
            ui.label(tt::pages_range_invalid(self.page_count));
        }
        ui.add_space(8.0);

        ui.weak(t::format_hint());
        ui.add_space(8.0);

        ui.separator();
        ui.horizontal(|ui| {
            let response = ui.add_enabled(pages.is_some(), egui::Button::new(t::export_button()));
            crate::diag::ui_rect(REGION_EXPORT, response.rect);
            if response.clicked() {
                self.export_requested = true;
            }
            if ui.button(tt::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
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
