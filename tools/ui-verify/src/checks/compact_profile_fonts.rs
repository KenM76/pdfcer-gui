//! `a_compact_settings_file_cuts_fonts` — **a settings file saying
//! `resubset-fonts = true` cuts the fonts with the window's own switch off,
//! and Save settings… writes both font keys back.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/compact_profile_fonts.md`.

use super::compact_fonts::recompressed_after;
use super::dimdrive::{Fixture, press, run_on_with};
use super::stroke_style::into_view_of;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// One face embedded twice; the window's font switch starts off.
const FIXTURE: Fixture = Fixture {
    file: "resubset-pair.pdf",
    method: "Copy it again from the engine's `fixtures/synthetic/resubset/pair.pdf`.",
    page: PageGeometry {
        width_pt: 140.0,
        height_pt: 80.0,
    },
};
const INVOKE: &str = "mode.edit,file.save_compacted";
/// Mirror `dialogs::compact_profile::{REGION_LOAD, REGION_SAVE}` and
/// `dialogs::compact::REGION_BODY`.
const LOAD: &str = "compact.profile.load";
const SAVE: &str = "compact.profile.save";
const BODY: &str = "compact.body";
/// Mirror `dialogs::compact_profile::{DIAG_LOAD_PATH, DIAG_SAVE_PATH}`.
const LOAD_ENV: &str = "PDFCER_DIAG_COMPACT_PROFILE_LOAD";
const SAVE_ENV: &str = "PDFCER_DIAG_COMPACT_PROFILE_SAVE";
const LOADED: &str = "compact-profile"; // ui-text-exempt: a trace event name, never displayed
const SAVED_NAME: &str = "fonts-saved.toml";
/// `merge-font-subsets = false` is the engine's non-default, so finding it in
/// the saved file shows the loaded request was written, not a fresh one.
const PROFILE: &str = "extends = \"lossless\"\nresubset-fonts = true\nmerge-font-subsets = false\n";

/// See the module documentation.
pub struct ACompactSettingsFileCutsFonts;

impl Check for ACompactSettingsFileCutsFonts {
    fn name(&self) -> &'static str {
        "a_compact_settings_file_cuts_fonts"
    }

    fn defect(&self) -> &'static str {
        "a compact settings file asking for fonts cut does not cut them, or Save settings… does \
         not write the font keys it was loaded with"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let load = ctx.out("fonts-profile.toml");
        let save = ctx.out(SAVED_NAME);
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
        run_on_with(
            self,
            ctx,
            &FIXTURE,
            INVOKE,
            "compact-profile-fonts",
            &env,
            body,
        )
    }
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
    if first.get_usize("fonts_cut") != Some(0) {
        return Ok(Some(format!(
            "with the window's font switch off and no file loaded the copy reads `{}`; it must \
             cut no font. Trace: {path}.",
            first.raw
        )));
    }
    into_view_of(ctx, session, pointer, LOAD, BODY)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, LOAD)?;
    let Some(loaded) = recompressed_after(session, mark)? else {
        return Ok(Some(format!(
            "Load settings… was pressed and the copy was not rebuilt. `{LOADED}`: {:?}. \
             Trace: {path}.",
            session.trace()?.last(LOADED).map(|l| l.raw.clone())
        )));
    };
    report.note(format!("loaded: `{}`", loaded.raw));
    let read = session.trace()?.last(LOADED).cloned();
    if read.as_ref().and_then(|l| l.get("resubset")) != Some("true")
        || read.as_ref().and_then(|l| l.get("merge")) != Some("false")
    {
        return Ok(Some(format!(
            "the file was read as {:?}; it says `resubset-fonts = true` and \
             `merge-font-subsets = false`. Trace: {path}.",
            read.map(|l| l.raw)
        )));
    }
    if loaded.get("profile") != Some("true") || loaded.get_usize("fonts_cut").unwrap_or(0) == 0 {
        return Ok(Some(format!(
            "with a file saying `resubset-fonts = true` loaded the copy reads `{}`; owed \
             `profile=true` and `fonts_cut>0`. Trace: {path}.",
            loaded.raw
        )));
    }
    saved(ctx, report, session, pointer)
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
    let target = ctx.out(SAVED_NAME);
    let written = std::fs::read_to_string(&target).unwrap_or_default();
    report.note(format!(
        "saved {} bytes to {}",
        written.len(),
        target.display()
    ));
    let ok =
        written.contains("resubset-fonts = true") && written.contains("merge-font-subsets = false");
    Ok((!ok).then(|| {
        format!(
            "Save settings… wrote {} bytes to {}; the file must carry the loaded \
             `resubset-fonts = true` and `merge-font-subsets = false`. Trace: {}.",
            written.len(),
            target.display(),
            session.trace_path().display()
        )
    }))
}
