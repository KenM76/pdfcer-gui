//! # `dialogs::ocr_model` — Recognise text's model list
//!
//! Every model `ocr::catalog` finds, in a drop-down. One this build cannot run
//! is listed greyed with its reason; a remembered model that is gone or cannot
//! run is named, and nothing is chosen in its place.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/ocr/catalog.md`.

use crate::app::prefs::Prefs;
use crate::ocr::catalog::{self, Catalog, Choice, Start};
use crate::text::ocrmodels as t;

/// The drop-down's rect; item `N` is `ocr-model.item.N`.
const REGION: &str = "ocr-model"; // ui-text-exempt: trace region name, never displayed
const ITEM_PREFIX: &str = "ocr-model.item."; // ui-text-exempt: trace region name, never displayed

/// The models on offer and the one chosen.
#[derive(Debug)]
pub(super) struct ModelPicker {
    catalog: Catalog,
    chosen: Option<usize>,
    /// The remembered model's sentence, while nothing has been chosen over it.
    notice: Option<String>,
}

impl ModelPicker {
    /// Discover the models under the bundled folder and `prefs`' extra
    /// folders, and pick the starting choice.
    pub(super) fn open(prefs: &Prefs) -> Self {
        let exe = crate::ocr::exe_dir();
        let roots = catalog::roots(exe.as_deref(), prefs.ocr_models.folders());
        let bundled = exe.map(|d| d.join(catalog::BUNDLED_DIR));
        let catalog = catalog::discover(roots, bundled.as_deref());
        let remembered = prefs.ocr_models.model.as_deref();
        let start = catalog::start(&catalog, remembered, prefs.ocr_engine);
        for c in &catalog.choices {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "ocr-model name={} engine={} runnable={} why={} folder={}",
                    c.name,
                    c.engine_token,
                    if c.runnable() { "yes" } else { "no" },
                    c.unrunnable
                        .as_ref()
                        .map_or("none", catalog::Unrunnable::token),
                    c.folder.display()
                )
            });
        }
        let (chosen, notice) = match start {
            Start::Chosen(i) => (Some(i), None),
            Start::Remembered { name, why: None } => (None, Some(t::remembered_missing(&name))),
            Start::Remembered { name, why: Some(w) } => {
                (None, Some(t::remembered_unrunnable(&name, &w)))
            }
            Start::Nothing => (None, None),
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "ocr-model-start chosen={} remembered={} roots={}",
                chosen.map_or("none", |i| catalog.choices[i].name.as_str()),
                remembered.unwrap_or("none"),
                catalog.roots.len()
            )
        });
        Self {
            catalog,
            chosen,
            notice,
        }
    }

    /// The chosen model; always one this build can run.
    pub(super) fn chosen(&self) -> Option<&Choice> {
        self.chosen.map(|i| &self.catalog.choices[i])
    }

    /// Whether any listed model can run.
    pub(super) fn any_runnable(&self) -> bool {
        self.catalog.choices.iter().any(Choice::runnable)
    }

    /// Every folder searched, for the no-models sentence.
    pub(super) fn roots(&self) -> &[std::path::PathBuf] {
        &self.catalog.roots
    }

    /// Draw the drop-down, the remembered-model sentence and discovery's notes.
    pub(super) fn show(&mut self, ui: &mut egui::Ui) {
        let before = self.chosen;
        let selected = self
            .chosen()
            .map_or_else(|| t::none_chosen().to_owned(), t::label);
        ui.horizontal(|ui| {
            ui.label(t::heading());
            let combo = egui::ComboBox::from_id_salt(REGION)
                .selected_text(selected)
                .width(320.0)
                .show_ui(ui, |ui| self.items(ui));
            crate::diag::ui_rect(REGION, combo.response.rect);
            if let Some(c) = self.chosen() {
                combo.response.on_hover_text(t::hover(c));
            }
        });
        if self.chosen != before {
            self.notice = None;
            let name = self.chosen().map_or("none", |c| c.name.as_str());
            // ui-text-exempt: diagnostic trace, never displayed.
            crate::diag::trace(|| format!("ocr-model-chosen name={name}"));
        }
        if let Some(notice) = &self.notice {
            ui.label(notice);
        }
        for note in &self.catalog.notes {
            ui.small(note);
        }
        ui.add_space(8.0);
    }

    fn items(&mut self, ui: &mut egui::Ui) {
        for (index, c) in self.catalog.choices.iter().enumerate() {
            let entry = if c.runnable() {
                ui.selectable_value(&mut self.chosen, Some(index), t::label(c))
                    .on_hover_text(t::hover(c))
            } else {
                ui.add_enabled(
                    false,
                    egui::Button::selectable(false, t::unrunnable_label(c)),
                )
                .on_disabled_hover_text(t::hover(c))
            };
            crate::diag::ui_rect(&format!("{ITEM_PREFIX}{index}"), entry.rect);
        }
    }
}
