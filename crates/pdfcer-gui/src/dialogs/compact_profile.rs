//! Save a compacted copy's settings files: the engine's optimize profile
//! (`pdfcer_core::optimize::profile`), the file `pdfcer optimize --profile`
//! reads and `--write-profile` writes.
//!
//! Load settings… reads one; while it is in use the window runs it in place
//! of the starting point, switches and size target. Save settings… writes
//! what the window would run. A profile cannot say "cut fonts to the glyphs
//! drawn" (`OptimizeSettings::resubset`), so the fonts switch stays the
//! window's own and is laid over a loaded file; the size target is not an
//! optimize setting at all.
//!
//! `allow-signed`: the window always runs with it on, because the compacted
//! copy it optimizes already keeps no signatures. That reason is the
//! window's, so a saved file carries `allow-signed = false`, and the CLI
//! reading it still refuses a signed input.

use std::path::{Path, PathBuf};

use egui::Ui;
use pdfcer_core::optimize::{OptimizeSettings, profile};

use crate::app::files::{Picked, from_env};
use crate::dialogs::sign::file_name_of;
use crate::text::compact_profile as t;

/// The file Load settings… reads, for `ui-verify`; empty is a cancel.
pub const DIAG_LOAD_PATH: &str = "PDFCER_DIAG_COMPACT_PROFILE_LOAD"; // ui-text-exempt: an environment variable name, never displayed
/// The file Save settings… writes, for `ui-verify`; empty is a cancel.
pub const DIAG_SAVE_PATH: &str = "PDFCER_DIAG_COMPACT_PROFILE_SAVE"; // ui-text-exempt: an environment variable name, never displayed
/// The Load settings… button.
// ui-text-exempt: trace region name, never displayed
pub const REGION_LOAD: &str = "compact.profile.load";
/// The Save settings… button.
// ui-text-exempt: trace region name, never displayed
pub const REGION_SAVE: &str = "compact.profile.save";
/// The Stop using button.
// ui-text-exempt: trace region name, never displayed
pub const REGION_STOP: &str = "compact.profile.stop";

/// A loaded settings file.
pub(crate) struct Loaded {
    name: String,
    settings: OptimizeSettings,
}

/// The row's state: the file in use, and the last thing it said.
#[derive(Default)]
pub(crate) struct State {
    loaded: Option<Loaded>,
    said: Option<String>,
}

impl State {
    /// The loaded file's settings, when one is in use.
    pub(crate) fn settings(&self) -> Option<&OptimizeSettings> {
        self.loaded.as_ref().map(|l| &l.settings)
    }

    /// Draw the row. `current` is what the window would run without a
    /// file. Returns whether what the window runs changed.
    pub(crate) fn row(
        &mut self,
        ui: &mut Ui,
        current: impl FnOnce() -> Result<OptimizeSettings, String>,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            let load = ui.button(t::load_button()).on_hover_text(t::load_tooltip());
            crate::diag::ui_rect_visible(REGION_LOAD, load.rect, ui.clip_rect());
            let save = ui.button(t::save_button()).on_hover_text(t::save_tooltip());
            crate::diag::ui_rect_visible(REGION_SAVE, save.rect, ui.clip_rect());
            if load.clicked() {
                changed = self.load();
            }
            if save.clicked() {
                let settings = match self.settings() {
                    Some(s) => Ok(s.clone()),
                    None => current(),
                };
                match settings.and_then(write_file) {
                    Ok(Some(name)) => self.said = Some(t::saved(&name)),
                    Ok(None) => {}
                    Err(why) => self.said = Some(t::failed(&why)),
                }
            }
        });
        if let Some(name) = self.loaded.as_ref().map(|l| l.name.clone()) {
            ui.horizontal_wrapped(|ui| {
                ui.label(t::in_use(&name));
                let stop = ui.button(t::stop_button());
                crate::diag::ui_rect_visible(REGION_STOP, stop.rect, ui.clip_rect());
                if stop.clicked() {
                    // ui-text-exempt: diagnostic trace
                    crate::diag::trace(|| "compact-profile stopped".to_owned());
                    self.loaded = None;
                    self.said = None;
                    changed = true;
                }
            });
        }
        if let Some(said) = &self.said {
            ui.small(said);
        }
        changed
    }

    /// Ask for a file and use it; whether one was taken.
    fn load(&mut self) -> bool {
        let Picked::Path(path) = pick(true) else {
            return false;
        };
        match read(&path) {
            Ok(settings) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace
                        "compact-profile loaded name={:?} rules={} objects={} linearize={}",
                        file_name_of(&path),
                        settings.rules.len(),
                        settings.objects.len(),
                        settings.structure.linearize
                    )
                });
                self.loaded = Some(Loaded {
                    name: file_name_of(&path),
                    settings,
                });
                self.said = None;
                true
            }
            Err(why) => {
                // ui-text-exempt: diagnostic trace
                crate::diag::trace(|| format!("compact-profile refused={why:?}"));
                self.said = Some(t::failed(&why));
                false
            }
        }
    }
}

/// Parse `path`, an `extends` naming a file read beside it, as the CLI does.
fn read(path: &Path) -> Result<OptimizeSettings, String> {
    let read = |p: &Path| {
        std::fs::read_to_string(p)
            .map_err(|e| t::file_problem(&p.display().to_string(), &e.to_string()))
    };
    let text = read(path)?;
    let base = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut resolve = |name: &str| read(&base.join(name));
    profile::parse(&file_name_of(path), &text, &mut resolve).map_err(|e| e.to_string())
}

/// Ask where, and write `settings` there; the file's name, or `None` when
/// the picker was cancelled.
fn write_file(mut settings: OptimizeSettings) -> Result<Option<String>, String> {
    let Picked::Path(path) = pick(false) else {
        return Ok(None);
    };
    settings.allow_signed = false;
    let text = profile::to_profile(&settings);
    std::fs::write(&path, &text).map_err(|e| t::file_problem(&path.display().to_string(), &e.to_string()))?;
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace
            "compact-profile saved name={:?} bytes={}",
            file_name_of(&path),
            text.len()
        )
    });
    Ok(Some(file_name_of(&path)))
}

/// The file picker, or the diagnostic seam.
fn pick(load: bool) -> Picked {
    let seam = if load { DIAG_LOAD_PATH } else { DIAG_SAVE_PATH };
    if let Some(answer) = from_env(std::env::var_os(seam)) {
        return answer;
    }
    let dialog = crate::filedialog::FileDialog::new()
        .add_filter(t::filter(), &["toml"])
        .add_filter(crate::text::files::filter_all(), &["*"]);
    let picked: Option<PathBuf> = if load {
        dialog.set_title(t::load_title()).pick_file()
    } else {
        dialog
            .set_title(t::save_title())
            .set_file_name(t::suggested_name())
            .save_file()
    };
    picked.map_or(Picked::Cancelled, Picked::Path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::optimize::Preset;

    #[test]
    fn a_saved_file_reads_back_and_refuses_signed_inputs() {
        let dir = std::env::temp_dir().join("pdfcer-gui-compact-profile");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mine.toml");
        let mut settings = Preset::Screen.settings();
        settings.allow_signed = true;
        settings.structure.linearize = true;
        let mut written = settings.clone();
        written.allow_signed = false;
        std::fs::write(&path, profile::to_profile(&written)).unwrap();
        let back = read(&path).unwrap();
        assert!(!back.allow_signed);
        assert!(back.structure.linearize);
        assert_eq!(back.images, settings.images);
    }

    #[test]
    fn extends_names_a_file_beside_it() {
        let dir = std::env::temp_dir().join("pdfcer-gui-compact-profile-extends");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("base.toml"),
            "extends = \"lossless\"\nlinearize = true\n",
        )
        .unwrap();
        std::fs::write(dir.join("child.toml"), "extends = \"base.toml\"\n").unwrap();
        assert!(read(&dir.join("child.toml")).unwrap().structure.linearize);
    }
}
