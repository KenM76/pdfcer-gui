//! # `panels::preflight::files` — the report and profiles, to and from files
//!
//! Save report… writes `report_text`, the engine's only report format, as
//! plain text. Load profile… reads a profile with `Profile::from_json` and
//! puts it in the picker under [`CUSTOM`]; a file the engine refuses is
//! named with `ProfileError`'s own sentence and changes nothing. Save
//! profile… writes the picked profile with `Profile::to_json`. Each outcome
//! is one line under the picker, never on the canvas.
//!
//! Traces `preflight-report-saved ok= bytes=`, `preflight-profile-saved ok=
//! bytes=` and `preflight-profile-loaded ok= rules=`; a failure adds
//! `why=` as the last field.

use std::path::{Path, PathBuf};

use pdfcer_core::preflight::{Profile, report_text};

use super::{PreflightUi, Results};
use crate::app::files::{Picked, from_env};
use crate::dialogs::filename::file_name_of;
use crate::text::panels::preflight as t;

/// The picker key of a loaded profile.
pub(super) const CUSTOM: &str = "custom"; // ui-text-exempt: a picker key, never displayed

/// The file Save report… writes, for `ui-verify`; empty is a cancel.
pub const DIAG_REPORT_SAVE: &str = "PDFCER_DIAG_PREFLIGHT_REPORT_SAVE"; // ui-text-exempt: an environment variable name, never displayed
/// The file Load profile… reads, for `ui-verify`; empty is a cancel.
pub const DIAG_PROFILE_LOAD: &str = "PDFCER_DIAG_PREFLIGHT_PROFILE_LOAD"; // ui-text-exempt: an environment variable name, never displayed
/// The file Save profile… writes, for `ui-verify`; empty is a cancel.
pub const DIAG_PROFILE_SAVE: &str = "PDFCER_DIAG_PREFLIGHT_PROFILE_SAVE"; // ui-text-exempt: an environment variable name, never displayed

const REGION_SAVE_REPORT: &str = "preflight.save_report"; // ui-text-exempt: diagnostic region name
const REGION_LOAD_PROFILE: &str = "preflight.load_profile"; // ui-text-exempt: diagnostic region name
const REGION_SAVE_PROFILE: &str = "preflight.save_profile"; // ui-text-exempt: diagnostic region name

/// Which file a picker asks for.
#[derive(Clone, Copy)]
enum Ask {
    Report,
    LoadProfile,
    SaveProfile,
}

/// The file picker, or the diagnostic seam.
fn pick(ask: Ask) -> Picked {
    let seam = match ask {
        Ask::Report => DIAG_REPORT_SAVE,
        Ask::LoadProfile => DIAG_PROFILE_LOAD,
        Ask::SaveProfile => DIAG_PROFILE_SAVE,
    };
    if let Some(answer) = from_env(std::env::var_os(seam)) {
        return answer;
    }
    let all = crate::text::files::filter_all();
    let picked: Option<PathBuf> = match ask {
        Ask::Report => crate::filedialog::FileDialog::new()
            .add_filter(t::report_filter(), &["txt"])
            .add_filter(all, &["*"])
            .set_title(t::save_report_title())
            .set_file_name(t::report_file_name())
            .save_file(),
        Ask::LoadProfile => crate::filedialog::FileDialog::new()
            .add_filter(t::profile_filter(), &["json"])
            .add_filter(all, &["*"])
            .set_title(t::load_profile_title())
            .pick_file(),
        Ask::SaveProfile => crate::filedialog::FileDialog::new()
            .add_filter(t::profile_filter(), &["json"])
            .add_filter(all, &["*"])
            .set_title(t::save_profile_title())
            .set_file_name(t::profile_file_name())
            .save_file(),
    };
    picked.map_or(Picked::Cancelled, Picked::Path)
}

/// Write `text` to a picked file; the line saying how it went, `None` on a
/// cancel. `event` names the trace line.
fn write(ask: Ask, event: &str, text: &str) -> Option<String> {
    let Picked::Path(path) = pick(ask) else {
        return None;
    };
    let outcome = std::fs::write(&path, text);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        let why = outcome
            .as_ref()
            .err()
            .map_or_else(String::new, |e| format!(" why={e}"));
        format!(
            "{event} ok={} bytes={}{why}",
            u8::from(outcome.is_ok()),
            text.len()
        )
    });
    Some(match outcome {
        Ok(()) => t::saved_to(&file_name_of(&path)),
        Err(e) => t::save_failed(&e.to_string()),
    })
}

/// Save report…, beside Copy report.
pub(super) fn save_report(ui: &mut egui::Ui, results: &Results, note: &mut Option<String>) {
    let save = ui
        .button(t::save_report())
        .on_hover_text(t::save_report_hover());
    crate::diag::ui_rect(REGION_SAVE_REPORT, save.rect);
    if save.clicked() {
        let text = report_text(&results.report, &results.profile);
        if let Some(said) = write(Ask::Report, "preflight-report-saved", &text) {
            *note = Some(said);
        }
    }
}

/// Load profile… and Save profile…, beside the picker.
pub(super) fn profile_buttons(ui: &mut egui::Ui, pf: &mut PreflightUi) {
    let load = ui
        .button(t::load_profile())
        .on_hover_text(t::load_profile_hover());
    crate::diag::ui_rect(REGION_LOAD_PROFILE, load.rect);
    if load.clicked()
        && let Picked::Path(path) = pick(Ask::LoadProfile)
    {
        pf.file_note = Some(load_from(&path, pf));
    }
    let save = ui
        .button(t::save_profile())
        .on_hover_text(t::save_profile_hover());
    crate::diag::ui_rect(REGION_SAVE_PROFILE, save.rect);
    if save.clicked()
        && let Some(profile) = pf.picked_profile()
        && let Some(said) = write(
            Ask::SaveProfile,
            "preflight-profile-saved",
            &profile.to_json(),
        )
    {
        pf.file_note = Some(said);
    }
}

/// Read `path` as a profile; on success it becomes the picked one.
fn load_from(path: &Path, pf: &mut PreflightUi) -> String {
    let name = file_name_of(path);
    let read = std::fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| Profile::from_json(&bytes).map_err(|e| e.to_string()));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        match &read {
            Ok(p) => format!("preflight-profile-loaded ok=1 rules={}", p.rules.len()),
            Err(why) => format!("preflight-profile-loaded ok=0 rules=0 why={why}"),
        }
    });
    match read {
        Ok(profile) => {
            let said = t::loaded(&profile.name, profile.rules.len());
            pf.custom = Some(profile);
            pf.profile = CUSTOM;
            said
        }
        Err(why) => t::load_failed(&name, &why),
    }
}
