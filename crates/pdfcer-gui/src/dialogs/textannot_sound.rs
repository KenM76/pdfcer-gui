//! # `dialogs::textannot::sound` — the attach-sound half of the note dialog
//!
//! The recording is picked before the window opens (in `BeginTextAnnot`'s
//! apply arm); this names it, offers the icon and the two import choices the
//! standard leaves open, and builds the action Add raises. The WAV is read
//! and converted at apply time, where a refusal is reported.

use std::path::PathBuf;

use egui::Ui;
use pdfcer_core::annot_author::SoundIcon;
use pdfcer_core::page_tree::Rect;
use pdfcer_core::sound::{SoundRatePolicy, WavImportOptions};

use crate::app::actions::Action;
use crate::canvas::textannot::painted_text;
use crate::text::soundannot as t;
use pdfcer_gui_base::wordmarkup::{DEFAULT_SOUND_ICON, SOUND_ICONS};

/// The region the icon chooser publishes; each icon publishes
/// `<this>.<PDF icon name>`, e.g. `text-annot.sound-icon.Mic`.
pub const REGION_SOUND_ICON: &str = "text-annot.sound-icon"; // ui-text-exempt: trace region name, never displayed
/// The resample checkbox's region.
pub const REGION_SOUND_RESAMPLE: &str = "text-annot.sound-resample"; // ui-text-exempt: trace region name, never displayed
/// The downmix checkbox's region.
pub const REGION_SOUND_DOWNMIX: &str = "text-annot.sound-downmix"; // ui-text-exempt: trace region name, never displayed

/// The recording chosen for a sound comment, and how it will be imported.
pub(super) struct Chosen {
    /// The file, as the picker named it.
    file: PathBuf,
    /// Its base name, as shown.
    name: String,
    /// Its size when the window opened, or `None` if it could not be read.
    bytes: Option<u64>,
    /// The icon.
    icon: SoundIcon,
    /// Resample to 11,025 or 22,050 Hz (`SoundRatePolicy::SpecRate`).
    resample: bool,
    /// Average more than two channels to mono.
    downmix: bool,
}

impl Chosen {
    /// Name `file` for the dialog. The import choices start at the engine's
    /// defaults: keep the recorded rate, refuse more than two channels.
    pub(super) fn new(file: PathBuf) -> Self {
        let name = file
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let bytes = std::fs::metadata(&file).ok().map(|m| m.len());
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            let size = bytes.map_or_else(|| "unreadable".to_owned(), |b| b.to_string());
            format!("sound-annot-open name={name:?} bytes={size}")
        });
        let defaults = WavImportOptions::default();
        Self {
            file,
            name,
            bytes,
            icon: DEFAULT_SOUND_ICON,
            resample: defaults.rate_policy == SoundRatePolicy::SpecRate,
            downmix: defaults.downmix,
        }
    }

    /// The file line, the icon chooser and the two import choices.
    pub(super) fn show(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.label(t::file_line(&self.name, self.bytes.unwrap_or(0)));
        ui.add_space(8.0);
        ui.label(t::icon_heading());
        let top = ui.cursor().min;
        for icon in SOUND_ICONS {
            let r = ui.radio_value(&mut self.icon, icon.clone(), t::icon_label(icon));
            crate::diag::ui_rect(
                &format!(
                    "{REGION_SOUND_ICON}.{}",
                    String::from_utf8_lossy(icon.name())
                ),
                r.rect,
            );
        }
        crate::diag::ui_rect(
            REGION_SOUND_ICON,
            egui::Rect::from_min_max(top, ui.cursor().min),
        );
        ui.add_space(8.0);
        let r = ui
            .checkbox(&mut self.resample, t::resample_label())
            .on_hover_text(t::resample_hover());
        crate::diag::ui_rect(REGION_SOUND_RESAMPLE, r.rect);
        let r = ui
            .checkbox(&mut self.downmix, t::downmix_label())
            .on_hover_text(t::downmix_hover());
        crate::diag::ui_rect(REGION_SOUND_DOWNMIX, r.rect);
    }

    /// The action Add raises. An empty description is `None`.
    pub(super) fn action(&self, page: usize, rect: Rect, typed: &str) -> Action {
        let text = painted_text(typed);
        Action::NewComment(pdfcer_gui_base::newcomment::NewComment::Sound {
            page,
            rect,
            file: self.file.clone(),
            icon: self.icon.clone(),
            description: (!text.is_empty()).then(|| text.to_owned()),
            import: WavImportOptions {
                rate_policy: if self.resample {
                    SoundRatePolicy::SpecRate
                } else {
                    SoundRatePolicy::KeepNative
                },
                downmix: self.downmix,
            },
        })
    }
}
