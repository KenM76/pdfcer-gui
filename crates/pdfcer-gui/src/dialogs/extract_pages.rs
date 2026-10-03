//! # `dialogs::extract_pages` — Pages ▸ Extract…
//!
//! Which pages, whether they keep their page labels, and whether they are
//! deleted from this document once the new file is written — Acrobat's
//! Extract Pages bar, whose *Delete pages after extracting* box this follows
//! (R11). The pages the command was given are the starting range.

use egui::Ui;
use pdfcer_core::pageops::ExtractedPageLabels;

use crate::app::actions::Action;
use crate::app::actions::imageexport::{PageScope, resolve_pages};
use crate::app::actions::pages::PageAction;
use crate::app::state::OpenDoc;
use crate::text::export_text as tt;
use crate::text::extract_pages as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:extract-pages"; // ui-text-exempt: trace region name, never displayed
/// The region the Extract button publishes.
pub const REGION_EXTRACT: &str = "extract-pages.extract"; // ui-text-exempt: trace region name, never displayed
/// The region the typed page range publishes.
pub const REGION_RANGE: &str = "extract-pages.pages.range"; // ui-text-exempt: trace region name, never displayed
/// The region the keep-labels box publishes.
pub const REGION_LABELS: &str = "extract-pages.labels"; // ui-text-exempt: trace region name, never displayed
/// The region the delete-afterwards box publishes.
pub const REGION_DELETE_AFTER: &str = "extract-pages.delete_after"; // ui-text-exempt: trace region name, never displayed

/// The region ONE page-scope radio publishes.
#[must_use]
pub const fn region_for_scope(scope: PageScope) -> &'static str {
    match scope {
        // ui-text-exempt: trace region names, never displayed.
        PageScope::AllPages => "extract-pages.pages.all",
        PageScope::CurrentPage => "extract-pages.pages.current",
        PageScope::Typed => "extract-pages.pages.typed",
    }
}

/// The Extract-pages window's live state.
pub struct ExtractPagesDialog {
    /// The page on screen at open, frozen so paging behind the window does
    /// not change what *This page* means.
    page_index: usize,
    /// Page count, frozen with `page_index`.
    page_count: usize,
    scope: PageScope,
    range_text: String,
    /// `None` when the document has no page labels, so there is nothing to
    /// keep and the box is not drawn.
    keep_labels: Option<bool>,
    delete_after: bool,
    extract_requested: bool,
    close_requested: bool,
}

impl ExtractPagesDialog {
    /// Open for `pages` of the document on screen. A single page that is the
    /// page on screen opens as *This page*; anything else as a typed range.
    #[must_use]
    pub fn open(doc: &OpenDoc, pages: &[usize]) -> Self {
        let page_index = doc.view.page_index;
        let (scope, range_text) = if pages == [page_index] {
            (PageScope::CurrentPage, String::new())
        } else {
            (
                PageScope::Typed,
                pdfcer_gui_base::pageselection::format_page_range(pages),
            )
        };
        let dialog = Self {
            page_index,
            page_count: doc.pages.len(),
            scope,
            range_text,
            keep_labels: doc.page_labels().is_some().then_some(true),
            delete_after: false,
            extract_requested: false,
            close_requested: false,
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "extract-pages-open page={} pages={} given={} labelled={}",
                dialog.page_index,
                dialog.page_count,
                pages.len(),
                u8::from(dialog.keep_labels.is_some()),
            )
        });
        dialog
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "extract-pages", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(440.0, 400.0),
            egui::vec2(340.0, 280.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.extract_requested)
            && let Some(action) = self.action()
        {
            actions.push(Action::Page(action));
            return false;
        }
        open && !std::mem::take(&mut self.close_requested)
    }

    /// The page action Extract raises, or `None` when the range names no page.
    fn action(&self) -> Option<PageAction> {
        let pages = resolve_pages(
            self.scope,
            &self.range_text,
            self.page_count,
            self.page_index,
        )?;
        let labels = if self.keep_labels == Some(false) {
            ExtractedPageLabels::Drop
        } else {
            ExtractedPageLabels::Keep
        };
        Some(PageAction::ExtractPages {
            pages,
            labels,
            delete_after: self.delete_after,
        })
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::intro());
        ui.add_space(8.0);
        self.pages_section(ui);
        ui.add_space(8.0);

        if let Some(keep) = self.keep_labels.as_mut() {
            let response = ui
                .checkbox(keep, t::keep_labels_label())
                .on_hover_text(t::keep_labels_tooltip());
            crate::diag::ui_rect(REGION_LABELS, response.rect);
        }
        let response = ui
            .checkbox(&mut self.delete_after, t::delete_after_label())
            .on_hover_text(t::delete_after_tooltip());
        crate::diag::ui_rect(REGION_DELETE_AFTER, response.rect);
        ui.add_space(8.0);

        ui.separator();
        let ready = self.action().is_some();
        ui.horizontal(|ui| {
            let response = ui.add_enabled(ready, egui::Button::new(t::extract_button()));
            crate::diag::ui_rect(REGION_EXTRACT, response.rect);
            if response.clicked() {
                self.extract_requested = true;
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
        if self.scope == PageScope::Typed && self.action().is_none() {
            ui.label(tt::pages_range_invalid(self.page_count));
        }
    }
}
