//! **Saved Print presets** — `OPERATOR_REQUESTS.md` **O418**: the Preset row at
//! the top of the window, which puts a saved set of settings into it, saves the
//! window's settings under a name, and deletes a saved one.
//!
//! A preset is [`crate::app::prefs::PrintPrefs`], the same value the window
//! remembers between jobs, so it holds exactly what a different document would
//! still want and never the page range. Files and folders:
//! `pdfcer_gui_base::prefs::printpresets`.

use std::path::PathBuf;

use pdfcer_gui_base::prefs::printpresets::{self as store, PresetFile};
use pdfcer_gui_base::printspooler::ScaleMode;

use super::PrintDialog;
use crate::app::prefs::PrintPrefs;
use crate::text::print::saved as t;

/// The dropdown.
pub const REGION_COMBO: &str = "print.preset.combo"; // ui-text-exempt: trace region name
/// One entry of the open dropdown; its index follows.
pub const REGION_ITEM: &str = "print.preset.item."; // ui-text-exempt: trace region prefix
/// Save as….
pub const REGION_SAVE_AS: &str = "print.preset.save-as"; // ui-text-exempt: trace region name
/// The name box.
pub const REGION_NAME: &str = "print.preset.name"; // ui-text-exempt: trace region name
/// Save or Replace under the typed name.
pub const REGION_COMMIT: &str = "print.preset.commit"; // ui-text-exempt: trace region name
/// Delete, and its confirming second press.
pub const REGION_DELETE: &str = "print.preset.delete"; // ui-text-exempt: trace region name

/// The row's state for one opening of the window.
#[derive(Default)]
pub(super) struct Saved {
    /// Where new presets go; `None` when there is no settings folder.
    default: Option<PathBuf>,
    /// Every folder searched, the default first.
    folders: Vec<PathBuf>,
    list: Vec<PresetFile>,
    /// The preset last put into the window, and what it held.
    chosen: Option<(String, PrintPrefs)>,
    /// The name being typed, while Save as is open.
    naming: Option<String>,
    /// The name box takes the keyboard on the frame it first draws.
    focus_name: bool,
    /// Delete has been pressed once.
    confirm: bool,
    /// The last outcome, and whether it is a failure.
    note: Option<(String, bool)>,
}

impl Saved {
    /// The row for `folders`, the default first, listing what they hold.
    pub(super) fn new(default: Option<PathBuf>, folders: Vec<PathBuf>) -> Self {
        let mut saved = Self {
            default,
            folders,
            ..Self::default()
        };
        saved.rescan();
        saved
    }

    fn rescan(&mut self) {
        self.list = store::discover(&self.folders);
        let count = self.list.len();
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("print-preset-list count={count}"));
    }
}

impl PrintDialog {
    /// Search `folders` for presets, saving new ones in `default`.
    pub(crate) fn with_presets(mut self, default: Option<PathBuf>, folders: Vec<PathBuf>) -> Self {
        self.saved = Saved::new(default, folders);
        self
    }

    /// The Preset row and, under it, the name box or the last outcome.
    pub(super) fn preset_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(t::label());
            self.preset_combo(ui);
            self.save_as_button(ui);
            self.delete_button(ui);
        });
        if self.saved.naming.is_some() {
            self.name_row(ui);
        }
        if let Some((note, failed)) = &self.saved.note {
            let mut text = egui::RichText::new(note).small();
            text = if *failed {
                text.color(ui.visuals().error_fg_color)
            } else {
                text.weak()
            };
            ui.label(text);
        }
    }

    fn preset_combo(&mut self, ui: &mut egui::Ui) {
        let shown = match &self.saved.chosen {
            Some((name, held)) if *held != self.habits() => format!("{name}{}", t::modified()),
            Some((name, _)) => name.clone(),
            None if self.saved.list.is_empty() => t::none_saved().to_owned(),
            None => t::none_chosen().to_owned(),
        };
        let mut picked = None;
        let combo = egui::ComboBox::from_id_salt("print-preset")
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for (index, preset) in self.saved.list.iter().enumerate() {
                    let on = self
                        .saved
                        .chosen
                        .as_ref()
                        .is_some_and(|(n, _)| *n == preset.name);
                    let item = ui.selectable_label(on, &preset.name);
                    crate::diag::ui_rect(&format!("{REGION_ITEM}{index}"), item.rect);
                    if item
                        .on_hover_text(preset.path.display().to_string())
                        .clicked()
                    {
                        picked = Some(index);
                    }
                }
            });
        crate::diag::ui_rect(REGION_COMBO, combo.response.rect);
        combo.response.on_hover_text(t::combo_hover());
        if let Some(index) = picked {
            self.choose_preset(index);
        }
    }

    fn save_as_button(&mut self, ui: &mut egui::Ui) {
        let can = self.saved.default.is_some();
        let button = ui.add_enabled(can, egui::Button::new(t::save_as()));
        crate::diag::ui_rect(REGION_SAVE_AS, button.rect);
        let button = if can {
            button.on_hover_text(t::save_as_hover())
        } else {
            button.on_disabled_hover_text(t::save_nowhere())
        };
        if button.clicked() {
            let name = self
                .saved
                .chosen
                .as_ref()
                .map(|(n, _)| n.clone())
                .unwrap_or_default();
            self.saved.naming = Some(name);
            self.saved.focus_name = true;
            self.saved.confirm = false;
        }
    }

    fn delete_button(&mut self, ui: &mut egui::Ui) {
        let chosen = self.saved.chosen.as_ref().map(|(n, _)| n.clone());
        let label = match (&chosen, self.saved.confirm) {
            (Some(name), true) => t::delete_confirm(name),
            _ => t::delete().to_owned(),
        };
        let button = ui.add_enabled(chosen.is_some(), egui::Button::new(label));
        crate::diag::ui_rect(REGION_DELETE, button.rect);
        let button = if chosen.is_some() {
            button.on_hover_text(t::delete_hover())
        } else {
            button.on_disabled_hover_text(t::delete_nothing())
        };
        if button.clicked() {
            if self.saved.confirm {
                self.delete_chosen();
            } else {
                self.saved.confirm = true;
            }
        }
    }

    /// The name box with Save (or Replace) and Cancel.
    fn name_row(&mut self, ui: &mut egui::Ui) {
        let Some(mut name) = self.saved.naming.take() else {
            return;
        };
        let mut done = false;
        ui.horizontal_wrapped(|ui| {
            // escape-disposition: dialog-cancels — `dialogs::host` owns the
            // key for every field in the Print window.
            let field = ui.add(
                egui::TextEdit::singleline(&mut name)
                    .hint_text(t::name_hint())
                    .desired_width(180.0),
            );
            crate::diag::ui_rect(REGION_NAME, field.rect);
            if std::mem::take(&mut self.saved.focus_name) {
                field.request_focus();
            }
            let entered = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let taken = self.saved.default.as_ref().is_some_and(|d| {
                store::check_name(&name).is_ok_and(|n| store::path_in(d, n).is_file())
            });
            let label = if taken { t::replace() } else { t::save() };
            let commit = ui.button(label);
            crate::diag::ui_rect(REGION_COMMIT, commit.rect);
            let commit = if taken {
                commit.on_hover_text(t::replace_hover())
            } else {
                commit
            };
            if commit.clicked() || entered {
                done = self.save_preset(&name);
            }
            if ui.button(t::cancel()).clicked() {
                done = true;
            }
        });
        if !done {
            self.saved.naming = Some(name);
        }
    }

    /// Save the window's settings as `name`; `true` when the name box closes.
    fn save_preset(&mut self, name: &str) -> bool {
        let Some(folder) = self.saved.default.clone() else {
            return true;
        };
        let name = match store::check_name(name) {
            Ok(name) => name.to_owned(),
            Err(why) => {
                self.saved.note = Some((t::refused(why), true));
                return false;
            }
        };
        let habits = self.habits();
        let result = store::save(&folder, &name, &habits);
        let ok = result.is_ok();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "print-preset-saved name={name:?} ok={ok} copies={}",
                habits.copies
            )
        });
        match result {
            Ok(_) => {
                self.saved.note = Some((t::saved(&name), false));
                self.saved.chosen = Some((name, habits));
                self.saved.rescan();
                true
            }
            Err(e) => {
                self.saved.note = Some((t::failed(t::verb_save(), &e), true));
                false
            }
        }
    }

    fn choose_preset(&mut self, index: usize) {
        let Some(preset) = self.saved.list.get(index).cloned() else {
            return;
        };
        self.saved.confirm = false;
        let (held, unread) = match store::load(&preset.path) {
            Ok(read) => read,
            Err(e) => {
                self.saved.note = Some((t::failed(t::verb_read(), &e), true));
                return;
            }
        };
        let missing = self.take_preset(&held);
        let mut note = t::applied(&preset.name, unread);
        if let Some(printer) = &missing {
            note.push(' ');
            note.push_str(&t::printer_missing(printer));
        }
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed
                "print-preset-applied name={:?} copies={} scale={} printer_found={} unread={unread}",
                preset.name,
                self.copies,
                crate::app::prefs::printing::scale_key(self.scale),
                missing.is_none(),
            )
        });
        self.saved.note = Some((note, false));
        self.saved.chosen = Some((preset.name, self.habits()));
    }

    fn delete_chosen(&mut self) {
        self.saved.confirm = false;
        let Some((name, _)) = self.saved.chosen.take() else {
            return;
        };
        let path = self
            .saved
            .list
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.path.clone());
        let result = path.map_or(Ok(()), std::fs::remove_file);
        let ok = result.is_ok();
        // ui-text-exempt: diagnostic trace, never displayed
        crate::diag::trace(|| format!("print-preset-deleted name={name:?} ok={ok}"));
        self.saved.note = Some(match result {
            Ok(()) => (t::deleted(&name), false),
            Err(e) => (t::failed(t::verb_delete(), &e.to_string()), true),
        });
        self.saved.rescan();
    }

    /// Put `remembered` into the window as [`PrintDialog::open`] does; the
    /// preset's printer name when no printer here has it.
    fn take_preset(&mut self, remembered: &PrintPrefs) -> Option<String> {
        let missing = match remembered.printer.as_deref() {
            Some(name) => match self.printers.iter().position(|p| p.name == name) {
                Some(index) => {
                    self.selected = index;
                    None
                }
                None => Some(name.to_owned()),
            },
            None => None,
        };
        self.scale = match remembered.scale {
            ScaleMode::Custom(_) => ScaleMode::Custom(f64::from(remembered.custom_percent) / 100.0),
            other => other,
        };
        if self.presets.scale_locked {
            self.scale = ScaleMode::ActualSize;
        }
        self.custom_percent = remembered.custom_percent;
        self.scope = remembered.scope;
        self.max_dpi = remembered.max_dpi;
        self.edges = remembered.edges;
        self.print_as_image = remembered.print_as_image;
        self.device.orientation = remembered.orientation;
        self.device.duplex = remembered.duplex;
        self.device.pick_tray_by_page_size = remembered.pick_tray_by_page_size;
        self.device.paper = remembered.paper;
        self.subset = remembered.subset;
        self.reverse = remembered.reverse;
        self.poster = remembered.poster;
        self.impose = super::impose::State::new(remembered.impose, remembered.poster.on);
        self.lines = remembered.lines;
        self.copies = remembered.copies;
        self.uncollated = remembered.uncollated;
        self.icc = super::icc::Icc::new(remembered.colour.clone());
        self.preview_texture = None;
        missing
    }
}

#[cfg(test)]
mod tests {
    /// Every field `PrintDialog::open` reads from what it was handed is read
    /// back by [`super::PrintDialog::take_preset`] too, so a preset carries
    /// every remembered setting.
    #[test]
    fn a_preset_sets_every_field_the_window_opens_with() {
        let field = |src: &str| -> Vec<String> {
            let mut out: Vec<String> = src
                .split("remembered.")
                .skip(1)
                .filter_map(|s| s.split(|c: char| !(c.is_alphanumeric() || c == '_')).next())
                .filter(|f| *f != "clone")
                .map(str::to_owned)
                .collect();
            out.sort();
            out.dedup();
            out
        };
        let open = include_str!("mod.rs")
            .split_once("pub(super) fn open(doc: &OpenDoc, remembered:")
            .and_then(|(_, rest)| rest.split_once("presets::apply("))
            .map(|(body, _)| body)
            .expect("PrintDialog::open is where this test expects it");
        let take = include_str!("saved.rs")
            .split_once("fn take_preset(")
            .and_then(|(_, rest)| rest.split_once("#[cfg(test)]"))
            .map(|(body, _)| body)
            .expect("take_preset is where this test expects it");
        let (open, take) = (field(open), field(take));
        let missed: Vec<&String> = open.iter().filter(|f| !take.contains(f)).collect();
        assert!(missed.is_empty(), "take_preset does not read {missed:?}");
        assert!(
            open.len() >= 15,
            "only {} fields found in open: {open:?}",
            open.len()
        );
    }
}
