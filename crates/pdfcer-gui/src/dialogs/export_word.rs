//! # `dialogs::export_word` — File ▸ Export ▸ Word document…
//!
//! Which pages, whether each PDF page starts a new Word page, whether tables
//! stay tables, and where headings and tables come from. The engine's
//! defaults are the window's starting state; the receipt reports what the
//! conversion inferred.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::imageexport::{PageScope, resolve_pages};
use crate::app::actions::wordexport::{StructureSource, WordExportPlan};
use crate::app::state::{OpenDoc, Status};
use crate::text::export_text as tt;
use crate::text::export_word as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:export-word"; // ui-text-exempt: trace region name, never displayed
/// The region the Export button publishes.
pub const REGION_EXPORT: &str = "export-word.export"; // ui-text-exempt: trace region name, never displayed
/// The region the typed page range publishes.
pub const REGION_RANGE: &str = "export-word.pages.range"; // ui-text-exempt: trace region name, never displayed
/// The region the page-break box publishes.
pub const REGION_PAGE_BREAKS: &str = "export-word.page_breaks"; // ui-text-exempt: trace region name, never displayed
/// The region the tables box publishes.
pub const REGION_TABLES: &str = "export-word.tables"; // ui-text-exempt: trace region name, never displayed

/// The region ONE page-scope radio publishes.
#[must_use]
pub const fn region_for_scope(scope: PageScope) -> &'static str {
    match scope {
        // ui-text-exempt: trace region names, never displayed.
        PageScope::AllPages => "export-word.pages.all",
        PageScope::CurrentPage => "export-word.pages.current",
        PageScope::Typed => "export-word.pages.typed",
    }
}

/// The region ONE structure radio publishes.
#[must_use]
pub const fn region_for_structure(structure: StructureSource) -> &'static str {
    match structure {
        // ui-text-exempt: trace region names, never displayed.
        StructureSource::Auto => "export-word.structure.auto",
        StructureSource::Tags => "export-word.structure.tags",
        StructureSource::Layout => "export-word.structure.layout",
    }
}

/// The Export-to-Word window's live state.
pub struct ExportWordDialog {
    /// The page on screen at open, frozen so paging behind the window does
    /// not change what *This page* means.
    page_index: usize,
    /// Page count, frozen with `page_index`.
    page_count: usize,
    scope: PageScope,
    range_text: String,
    page_breaks: bool,
    tables: bool,
    structure: StructureSource,
    export_requested: bool,
    close_requested: bool,
}

impl ExportWordDialog {
    /// Open for the document on screen, at the engine's defaults.
    #[must_use]
    pub fn open(doc: &OpenDoc) -> Self {
        let defaults = WordExportPlan::new(Vec::new());
        let dialog = Self {
            page_index: doc.view.page_index,
            page_count: doc.pages.len(),
            scope: PageScope::AllPages,
            range_text: String::new(),
            page_breaks: defaults.page_breaks,
            tables: defaults.tables,
            structure: defaults.structure,
            export_requested: false,
            close_requested: false,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "export-word-open page={} pages={}",
                dialog.page_index, dialog.page_count
            )
        });
        dialog
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "export-word", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(460.0, 500.0),
            egui::vec2(360.0, 320.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.export_requested)
            && let Some(plan) = self.plan()
        {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "export-word-requested pages={} page_breaks={} table_option={} structure={}",
                    plan.pages.len(),
                    u8::from(plan.page_breaks),
                    u8::from(plan.tables),
                    plan.structure.key(),
                )
            });
            actions.push(Action::Write(
                crate::app::actions::write::WriteAction::Word { plan },
            ));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The plan, or `None` when the typed range names no page.
    fn plan(&self) -> Option<WordExportPlan> {
        let pages = resolve_pages(
            self.scope,
            &self.range_text,
            self.page_count,
            self.page_index,
        )?;
        Some(WordExportPlan {
            page_breaks: self.page_breaks,
            tables: self.tables,
            structure: self.structure,
            ..WordExportPlan::new(pages)
        })
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);
        self.pages_section(ui);
        ui.add_space(8.0);

        let response = ui.checkbox(&mut self.page_breaks, t::page_breaks_label());
        crate::diag::ui_rect(REGION_PAGE_BREAKS, response.rect);
        let response = ui
            .checkbox(&mut self.tables, t::tables_label())
            .on_hover_text(t::tables_tooltip());
        crate::diag::ui_rect(REGION_TABLES, response.rect);
        ui.add_space(8.0);

        ui.label(t::structure_heading());
        for structure in StructureSource::ALL {
            let response =
                ui.radio_value(&mut self.structure, structure, t::structure_name(structure));
            crate::diag::ui_rect(region_for_structure(structure), response.rect);
        }
        ui.weak(t::structure_hint(self.structure));
        ui.add_space(8.0);

        ui.separator();
        let ready = self.plan().is_some();
        ui.horizontal(|ui| {
            let response = ui.add_enabled(ready, egui::Button::new(t::export_button()));
            crate::diag::ui_rect(REGION_EXPORT, response.rect);
            if response.clicked() {
                self.export_requested = true;
            }
            if ui.button(tt::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    fn pages_section(&mut self, ui: &mut Ui) {
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
        if self.scope == PageScope::Typed && self.plan().is_none() {
            ui.label(tt::pages_range_invalid(self.page_count));
        }
    }
}

/// Open the window for `status`, or decline.
#[must_use]
pub fn open_for(status: &Status) -> Option<ExportWordDialog> {
    match status {
        Status::Open(doc) if !doc.pages.is_empty() => Some(ExportWordDialog::open(doc)),
        _ => None,
    }
}
