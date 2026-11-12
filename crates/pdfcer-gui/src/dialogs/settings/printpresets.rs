//! # `dialogs::settings::printpresets` — where the Print window finds presets
//!
//! `OPERATOR_REQUESTS.md` **O418**: the default `print-presets` folder first,
//! fixed, then the extra folders, which are added and removed here. The list
//! is [`crate::app::prefs::printpresets::PrintPresetFolders`]; the Print window
//! reads it when it opens.

use egui::Ui;
use pdfcer_gui_base::prefs::printpresets::{self as store, PrintPresetFolders};

use crate::app::prefs::Prefs;
use crate::text::settings::printpresets as t;

use super::widgets;

/// The page's rect, for `ui-verify`.
pub const REGION: &str = "settings.printpresets"; // ui-text-exempt: trace region name
/// The Add button.
pub const ADD_REGION: &str = "settings.printpresets.add"; // ui-text-exempt: trace region name
/// The Remove button of an extra folder; its index follows.
pub const REMOVE_REGION: &str = "settings.printpresets.remove."; // ui-text-exempt: trace region prefix
/// Answers the Add picker when set, for a driven check.
pub const DIAG_FOLDER_PATH: &str = "PDFCER_DIAG_PRINT_PRESET_FOLDER"; // ui-text-exempt: an environment variable name

/// Draw the default folder, the extra folders, Add, and how many presets they hold.
pub fn folders(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(ui, t::title(), t::silence(), t::radius());
    default_row(ui);
    let list = &mut prefs.print_presets;
    let mut remove: Option<usize> = None;
    for (index, folder) in list.folders().iter().enumerate() {
        ui.horizontal(|ui| {
            let button = ui
                .small_button(crate::text::settings::font_folder_remove())
                .on_hover_text(crate::text::settings::font_folder_remove_hover());
            crate::diag::ui_rect_visible(
                &format!("{REMOVE_REGION}{index}"),
                button.rect,
                ui.clip_rect(),
            );
            if button.clicked() {
                remove = Some(index);
            }
            let text = folder.display().to_string();
            ui.add(egui::Label::new(&text).truncate())
                .on_hover_text(&text);
        });
    }
    if let Some(index) = remove {
        list.remove_folder(index);
    }
    ui.add_space(4.0);
    add_button(ui, list);
    ui.add_space(6.0);
    let count = store::discover(&list.search_list()).len();
    ui.small(t::found(count));
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
}

/// The default folder, which is not removable, and its Open button.
fn default_row(ui: &mut Ui) {
    let Some(folder) = store::default_folder() else {
        ui.small(t::no_default());
        return;
    };
    ui.horizontal(|ui| {
        if cfg!(windows)
            && ui
                .small_button(t::open_folder())
                .on_hover_text(t::open_hover())
                .clicked()
        {
            reveal(&folder);
        }
        let text = folder.display().to_string();
        ui.add(egui::Label::new(&text).truncate())
            .on_hover_text(t::default_hover());
    });
    ui.label(egui::RichText::new(t::default_note()).small().weak());
}

fn add_button(ui: &mut Ui, list: &mut PrintPresetFolders) {
    let full = list.folders().len() >= PrintPresetFolders::MAX_FOLDERS;
    let add = ui.add_enabled(
        !full,
        egui::Button::new(crate::text::settings::font_folder_add()),
    );
    crate::diag::ui_rect_visible(ADD_REGION, add.rect, ui.clip_rect());
    let add = if full {
        add.on_disabled_hover_text(crate::text::settings::font_folders_full(
            PrintPresetFolders::MAX_FOLDERS,
        ))
    } else {
        add.on_hover_text(t::add_hover())
    };
    if add.clicked()
        && let crate::app::files::Picked::Path(path) = pick()
        && list.add_folder(&path)
    {
        let count = list.folders().len();
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| format!("print-preset-folder-added folders={count}"));
    }
}

fn pick() -> crate::app::files::Picked {
    crate::app::files::pick_folder_traced(
        DIAG_FOLDER_PATH,
        t::dialog_title(),
        "print-preset-folder-picked", // ui-text-exempt: a trace event name
    )
}

/// Show `folder` in File Explorer, creating it first so a new install has one.
fn reveal(folder: &std::path::Path) {
    let made = std::fs::create_dir_all(folder).is_ok();
    // ui-text-exempt: a program name, never displayed.
    let shown = std::process::Command::new("explorer")
        .arg(folder)
        .spawn()
        .is_ok();
    // ui-text-exempt: diagnostic trace, never displayed.
    crate::diag::trace(|| format!("print-preset-folder-opened made={made} shown={shown}"));
}
