//! `compact_settings_files_load_and_save` — the compact window's Load
//! settings… runs a settings file in place of its switches, Save settings…
//! writes one the engine reads back, and Stop using returns to the switches.
//!
//! Driven off-screen on `ocr-layer.pdf` with the pickers answered by
//! `PDFCER_DIAG_COMPACT_PROFILE_LOAD` / `_SAVE`. The loaded file is
//! `extends = "lossless"` plus `linearize = true`: Fast Web View is off in
//! the window's own switches, so `linearized=true` on the rebuilt copy can
//! only come from the file. Oracles, each on a `compact-recompressed` line
//! after the press:
//!
//! - Load: `profile=true linearized=true`.
//! - Save: the written file carries `linearize = true` (it saved the loaded
//!   settings) and `allow-signed = false` (the window's own reason for
//!   allowing signed inputs is not written into a file the CLI reads).
//! - Stop using: `profile=false linearized=false`.

use super::compact_fonts::recompressed_after;
use super::dimdrive::{Fixture, press, run_on_with};
use super::stroke_style::into_view_of;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: Fixture = Fixture {
    file: "ocr-layer.pdf",
    method: "Committed with the repository; restore it from git.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact_profile::{REGION_LOAD, REGION_SAVE, REGION_STOP}`.
const LOAD: &str = "compact.profile.load";
const SAVE: &str = "compact.profile.save";
const STOP: &str = "compact.profile.stop";
const BODY: &str = "compact.body";
/// Mirror `dialogs::compact_profile::{DIAG_LOAD_PATH, DIAG_SAVE_PATH}`.
const LOAD_ENV: &str = "PDFCER_DIAG_COMPACT_PROFILE_LOAD";
const SAVE_ENV: &str = "PDFCER_DIAG_COMPACT_PROFILE_SAVE";
const LOADED: &str = "compact-profile"; // ui-text-exempt: a trace event name, never displayed
const PROFILE: &str = "extends = \"lossless\"\nlinearize = true\n";

/// See the module documentation.
pub struct CompactSettingsFilesLoadAndSave;

impl Check for CompactSettingsFilesLoadAndSave {
    fn name(&self) -> &'static str {
        "compact_settings_files_load_and_save"
    }

    fn defect(&self) -> &'static str {
        "the compact window's settings files do not work: a loaded file is not what the copy is \
         built with, Save settings writes nothing or not what the window runs, or Stop using \
         does not return to the switches"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let load = ctx.out("load-me.toml");
        let save = ctx.out("saved.toml");
        // A file left by a previous run would satisfy the Save oracle.
        let _ = std::fs::remove_file(&save);
        if let Err(e) = std::fs::write(&load, PROFILE) {
            return CheckReport::new(self.name(), self.defect())
                .fail(format!("could not write {}: {e}", load.display()));
        }
        let env = [
            (LOAD_ENV, load.to_string_lossy().into_owned()),
            (SAVE_ENV, save.to_string_lossy().into_owned()),
        ];
        run_on_with(self, ctx, &FIXTURE, INVOKE, "compact-profile", &env, body)
    }
}

/// Press `region` and return the next `compact-recompressed` line's raw text.
fn rebuilt_after_press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<Option<TraceLine>> {
    into_view_of(ctx, session, pointer, region, BODY)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, region)?;
    recompressed_after(session, mark)
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let Some(first) = recompressed_after(session, 0)? else {
        return Ok(Some(format!(
            "the compacted-copy window never finished its first recompression. Trace: {path}."
        )));
    };
    report.note(format!("switches: `{}`", first.raw));
    if first.get("linearized") != Some("false") || first.get("profile") != Some("false") {
        return Ok(Some(format!(
            "the window opened on `{}`; with no file loaded and Fast Web View off it must read \
             `profile=false` and `linearized=false`. Trace: {path}.",
            first.raw
        )));
    }
    let Some(loaded) = rebuilt_after_press(ctx, session, pointer, LOAD)? else {
        return Ok(Some(format!(
            "Load settings… was pressed and the copy was not rebuilt. `{LOADED}` lines: {:?}. \
             Trace: {path}.",
            session.trace()?.last(LOADED).map(|l| l.raw.clone())
        )));
    };
    report.note(format!("loaded: `{}`", loaded.raw));
    if loaded.get("profile") != Some("true") || loaded.get("linearized") != Some("true") {
        return Ok(Some(format!(
            "with a file saying `linearize = true` loaded, the copy reads `{}`; it must be built \
             from the file (`profile=true linearized=true`). Trace: {path}.",
            loaded.raw
        )));
    }
    if let Some(failure) = saved(ctx, report, session, pointer)? {
        return Ok(Some(failure));
    }
    let Some(stopped) = rebuilt_after_press(ctx, session, pointer, STOP)? else {
        return Ok(Some(format!(
            "Stop using was pressed and the copy was not rebuilt. Trace: {path}."
        )));
    };
    report.note(format!("stopped: `{}`", stopped.raw));
    Ok(
        (stopped.get("profile") != Some("false") || stopped.get("linearized") != Some("false"))
            .then(|| {
                format!(
                    "after Stop using the copy reads `{}`; it must be back on the switches \
                     (`profile=false linearized=false`). Trace: {path}.",
                    stopped.raw
                )
            }),
    )
}

/// Press Save settings… and read the file it wrote.
fn saved(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    into_view_of(ctx, session, pointer, SAVE, BODY)?;
    press(ctx, session, pointer, SAVE)?;
    session.settle(30);
    let target = ctx.out("saved.toml");
    let written = std::fs::read_to_string(&target).unwrap_or_default();
    report.note(format!(
        "saved {} bytes to {}",
        written.len(),
        target.display()
    ));
    let ok = written.contains("linearize = true") && written.contains("allow-signed = false");
    Ok((!ok).then(|| {
        format!(
            "Save settings… wrote {} bytes to {}; the file must carry the loaded \
             `linearize = true` and `allow-signed = false`. Trace: {}.",
            written.len(),
            target.display(),
            session.trace_path().display()
        )
    }))
}
