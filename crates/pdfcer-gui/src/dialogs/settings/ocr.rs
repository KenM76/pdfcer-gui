//! # `dialogs::settings::ocr` — the Settings window's OCR models page
//!
//! The extra folders Recognise text searches for models, after the bundled
//! `models` folder, and the models they hold. The list is
//! [`crate::app::prefs::OcrModelPrefs`]; the dialog reads it when it opens.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/ocr.md`.

use std::path::PathBuf;

use egui::Ui;

use crate::app::prefs::{OcrModelPrefs, Prefs};
use crate::ocr::catalog;
use crate::text::ocrmodels as tm;
use crate::text::settings::ocrmodels as t;

use super::widgets;

/// The page's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "settings.ocr";
/// The Add button.
// ui-text-exempt: trace region name, never displayed
pub const ADD_REGION: &str = "settings.ocr.add";
/// The allow-programs checkbox.
// ui-text-exempt: trace region name, never displayed
pub const PROGRAMS_REGION: &str = "settings.ocr.programs";

/// Draw the folder list, its Add button and the models it yields.
pub fn folders(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(ui, t::title(), t::silence(), t::radius());

    let models = &mut prefs.ocr_models;
    let mut remove: Option<usize> = None;
    for (index, folder) in models.folders().iter().enumerate() {
        ui.horizontal(|ui| {
            if ui
                .small_button(crate::text::settings::font_folder_remove())
                .on_hover_text(crate::text::settings::font_folder_remove_hover())
                .clicked()
            {
                remove = Some(index);
            }
            let text = folder.display().to_string();
            ui.add(egui::Label::new(&text).truncate())
                .on_hover_text(&text);
        });
    }
    if let Some(index) = remove {
        models.remove_folder(index);
    }
    if models.folders().is_empty() {
        ui.small(t::folders_none());
    }

    ui.add_space(4.0);
    let full = models.folders().len() >= OcrModelPrefs::MAX_FOLDERS;
    let add = ui.add_enabled(
        !full,
        egui::Button::new(crate::text::settings::font_folder_add()),
    );
    crate::diag::ui_rect_visible(ADD_REGION, add.rect, ui.clip_rect());
    let add = if full {
        add.on_disabled_hover_text(crate::text::settings::font_folders_full(
            OcrModelPrefs::MAX_FOLDERS,
        ))
    } else {
        add.on_hover_text(t::add_hover())
    };
    if add.clicked()
        && let crate::app::files::Picked::Path(path) = crate::app::files::pick_ocr_folder()
        && models.add_folder(&path)
    {
        let count = models.folders().len();
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("ocr-folder-added folders={count}"));
    }

    ui.add_space(6.0);
    programs(ui, models);
    if !models.folders().is_empty() {
        found(ui, models.folders(), models.refuse_programs);
    }
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
}

/// The checkbox allowing model folders that run a separate program.
fn programs(ui: &mut Ui, models: &mut OcrModelPrefs) {
    let mut allow = !models.refuse_programs;
    let box_ = ui
        .checkbox(&mut allow, t::allow_programs())
        .on_hover_text(t::allow_programs_hover());
    crate::diag::ui_rect_visible(PROGRAMS_REGION, box_.rect, ui.clip_rect());
    if box_.changed() {
        models.refuse_programs = !allow;
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("ocr-programs-allowed allow={allow}"));
    }
}

/// The models the extra folders hold, discovered once per folder list and
/// program policy.
fn found(ui: &mut Ui, folders: &[PathBuf], refuse_programs: bool) {
    let id = egui::Id::new("settings.ocr.found");
    let cached: Option<(Vec<PathBuf>, bool, Vec<String>)> = ui.data(|d| d.get_temp(id));
    let lines = match cached {
        Some((key, refused, lines)) if key == folders && refused == refuse_programs => lines,
        _ => {
            let policy = catalog::policy(!refuse_programs);
            let found = catalog::discover(folders.to_vec(), None, policy);
            let lines: Vec<String> = found
                .choices
                .iter()
                .map(|c| {
                    if c.runnable() {
                        tm::label(c)
                    } else {
                        tm::unrunnable_label(c)
                    }
                })
                .chain(found.notes)
                .collect();
            ui.data_mut(|d| d.insert_temp(id, (folders.to_vec(), refuse_programs, lines.clone())));
            lines
        }
    };
    ui.add_space(6.0);
    if lines.is_empty() {
        ui.small(t::found_none());
        return;
    }
    ui.small(t::found());
    for line in &lines {
        ui.add(egui::Label::new(egui::RichText::new(line).weak().small()).truncate())
            .on_hover_text(line);
    }
}
