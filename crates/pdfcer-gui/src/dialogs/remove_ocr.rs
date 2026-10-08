//! # `dialogs::remove_ocr` — File ▸ Remove OCR text, with Recognise text's
//! page choices and a choice of which recogniser's text
//!
//! Contract: opened only on a document holding at least one OCR layer pdfcer
//! wrote — the layers are listed once, at open, as `(page, engine)` pairs.
//! The operator picks the pages with the same [`PageScope`] Recognise text
//! uses, and, when the document holds text from more than one recogniser,
//! which recognisers' text comes off. The window says how many layers on how
//! many pages that answer names; Remove is greyed while it names none.
//! Pressing Remove pushes [`Action::RemoveOcrLayers`] carrying the pages and
//! engines; the removal itself is `app::actions::ocrlayers`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/remove_ocr.md`.

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::ocr as t;

use super::page_scope::PageScope;

/// The window body.
pub const REGION_BODY: &str = "remove-ocr.body"; // ui-text-exempt: trace region name, never displayed
/// The page-scope group.
pub const REGION_SCOPE: &str = "remove-ocr.scope"; // ui-text-exempt: trace region name, never displayed
/// The button that removes.
pub const REGION_COMMIT: &str = "remove-ocr.commit"; // ui-text-exempt: trace region name, never displayed
/// One recogniser's checkbox; the engine token follows the dot.
pub const REGION_ENGINE: &str = "remove-ocr.engine"; // ui-text-exempt: trace region name, never displayed

/// One recogniser whose text the document holds.
#[derive(Debug)]
struct Engine {
    /// The `/Engine` the marker recorded; `None` for a layer that recorded none.
    token: Option<String>,
    /// Whether its text is to come off.
    chosen: bool,
}

/// The window.
#[derive(Debug)]
pub struct RemoveOcrDialog {
    /// Every pdfcer OCR layer at open: `(page, engine)`.
    layers: Vec<(usize, Option<String>)>,
    engines: Vec<Engine>,
    scope: PageScope,
    page_count: usize,
    /// The summary last traced, so the trace fires on a change.
    traced: (usize, usize),
    commit_requested: bool,
    close_requested: bool,
}

impl RemoveOcrDialog {
    /// `None` when the document holds no pdfcer OCR layer, or its page tree
    /// cannot be walked: the caller sends the unfiltered removal, whose
    /// refusal sentence says why.
    fn open(doc: &OpenDoc, picked: Vec<usize>) -> Option<Self> {
        let found = doc.session.find_ocr_layers().ok()?;
        if found.is_empty() {
            return None;
        }
        let layers: Vec<(usize, Option<String>)> = found
            .into_iter()
            .map(|l| (l.page_index, l.engine))
            .collect();
        let mut engines: Vec<Engine> = Vec::new();
        for (_, token) in &layers {
            if !engines.iter().any(|e| &e.token == token) {
                engines.push(Engine {
                    token: token.clone(),
                    chosen: true,
                });
            }
        }
        Some(Self {
            layers,
            engines,
            scope: PageScope::new(doc.view.page_index, picked),
            page_count: doc.pages.len(),
            traced: (usize::MAX, usize::MAX),
            commit_requested: false,
            close_requested: false,
        })
    }

    /// The pages and engines the current answer names.
    fn filter(&self) -> (Vec<usize>, Vec<Option<String>>) {
        let pages = self.scope.pages(self.page_count).unwrap_or_default();
        let engines = self
            .engines
            .iter()
            .filter(|e| e.chosen)
            .map(|e| e.token.clone())
            .collect();
        (pages, engines)
    }

    /// How many layers on how many pages the answer names.
    fn named(&self) -> (usize, usize) {
        let (pages, engines) = self.filter();
        let hit: Vec<usize> = self
            .layers
            .iter()
            .filter(|(p, e)| pages.contains(p) && engines.contains(e))
            .map(|(p, _)| *p)
            .collect();
        let mut distinct = hit.clone();
        distinct.sort_unstable();
        distinct.dedup();
        (hit.len(), distinct.len())
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "remove-ocr", // ui-text-exempt: a viewport key, never displayed.
            t::remove_title(),
            egui::vec2(460.0, 420.0),
            egui::vec2(360.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if std::mem::take(&mut self.commit_requested) {
            let (pages, engines) = self.filter();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "remove-ocr-requested pages={} engines={}",
                    pages.len(),
                    engines.len()
                )
            });
            actions.push(Action::RemoveOcrLayers {
                pages: Some(pages),
                engines: Some(engines),
            });
            return false;
        }
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        ui.label(t::remove_intro());
        ui.add_space(8.0);
        self.scope
            .show(ui, self.page_count, REGION_SCOPE, "remove-ocr-scope");
        if self.engines.len() > 1 {
            ui.add_space(8.0);
            ui.label(t::remove_engines_heading());
            for engine in &mut self.engines {
                let token = engine.token.as_deref().unwrap_or_default();
                let label = t::token_label(token).map_or_else(
                    || {
                        if token.is_empty() {
                            t::remove_engine_unrecorded().to_owned()
                        } else {
                            token.to_owned()
                        }
                    },
                    str::to_owned,
                );
                let r = ui.checkbox(&mut engine.chosen, label);
                crate::diag::ui_rect(&format!("{REGION_ENGINE}.{token}"), r.rect);
            }
        }
        let (layers, pages) = self.named();
        if (layers, pages) != self.traced {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!("remove-ocr-named layers={layers} pages={pages}")
            });
            self.traced = (layers, pages);
        }
        ui.add_space(8.0);
        ui.label(t::remove_summary(layers, pages));
        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            // Greyed, not hidden, while the answer names nothing: the operator
            // is mid-way through typing a range or ticking boxes.
            let remove = ui
                .add_enabled(layers > 0, egui::Button::new(t::remove_button()))
                .on_disabled_hover_text(t::remove_names_nothing());
            crate::diag::ui_rect_visible(REGION_COMMIT, remove.rect, ui.clip_rect());
            if remove.clicked() {
                self.commit_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }
}

/// Build it for the current document. `None` when there is no document or no
/// pdfcer OCR layer in it.
pub fn open_for(status: &Status, picked: Vec<usize>) -> Option<RemoveOcrDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    RemoveOcrDialog::open(doc, picked)
}
