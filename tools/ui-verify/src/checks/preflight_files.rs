//! `the_preflight_panel_saves_its_report_and_loads_a_profile` — the
//! Preflight panel refuses a broken profile file and says why, runs a loaded
//! one, saves the report as the engine's text, and saves the profile back.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/preflight_files.md`.

use std::path::Path;

use super::dimdrive::{Fixture, launch_with, press};
use super::print_shop::await_line;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const FIXTURE: Fixture = Fixture {
    file: "prepress-mix.pdf",
    method: "Rebuild it with `python fixtures/prepress-mix.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const LOAD_ENV: &str = "PDFCER_DIAG_PREFLIGHT_PROFILE_LOAD"; // ui-text-exempt: an environment variable name
const SAVE_ENV: &str = "PDFCER_DIAG_PREFLIGHT_PROFILE_SAVE"; // ui-text-exempt: an environment variable name
const REPORT_ENV: &str = "PDFCER_DIAG_PREFLIGHT_REPORT_SAVE"; // ui-text-exempt: an environment variable name
const LOADED: &str = "preflight-profile-loaded"; // ui-text-exempt: a trace event name
const FINISHED: &str = "preflight-finished"; // ui-text-exempt: a trace event name
const REPORT_SAVED: &str = "preflight-report-saved"; // ui-text-exempt: a trace event name
const PROFILE_SAVED: &str = "preflight-profile-saved"; // ui-text-exempt: a trace event name

/// A check the engine does not know: `from_json` refuses the file.
const BROKEN: &str =
    r#"{"name": "Broken", "rules": [{"id": "x", "severity": "error", "check": "no-such-check"}]}"#; // ui-text-exempt: a fixture file's text
/// One rule the fixture breaks (it has no trim box).
const GOOD: &str = r#"{"name": "Trim only", "description": "Every page needs a trim box.", "rules": [{"id": "needs-trim", "severity": "error", "check": "no-trim-box"}]}"#; // ui-text-exempt: a fixture file's text

/// `the_preflight_panel_saves_its_report_and_loads_a_profile`.
pub struct ThePreflightPanelSavesItsReportAndLoadsAProfile;

impl Check for ThePreflightPanelSavesItsReportAndLoadsAProfile {
    fn name(&self) -> &'static str {
        "the_preflight_panel_saves_its_report_and_loads_a_profile"
    }

    fn defect(&self) -> &'static str {
        "The Preflight panel takes a profile file the engine refuses, runs something other \
         than the profile it loaded, saves no report or one that is not the engine's text, \
         or saves a profile that has lost its rule"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let profile_in = ctx.out("preflight-files.profile-in.json");
        let profile_out = ctx.out("preflight-files.profile-out.json");
        let report_out = ctx.out("preflight-files.report.txt");
        for p in [&profile_out, &report_out] {
            let _ = std::fs::remove_file(p);
        }
        let env = [
            (LOAD_ENV, profile_in.display().to_string()),
            (SAVE_ENV, profile_out.display().to_string()),
            (REPORT_ENV, report_out.display().to_string()),
        ];
        let files = Files {
            profile_in: &profile_in,
            profile_out: &profile_out,
            report_out: &report_out,
        };
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit,view.preflight",
            "preflight-files",
            &env,
        )
        .and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer, &files);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

struct Files<'a> {
    profile_in: &'a Path,
    profile_out: &'a Path,
    report_out: &'a Path,
}

fn put(path: &Path, text: &str) -> Result<()> {
    std::fs::write(path, text).map_err(|e| Error::new(format!("writing {}: {e}", path.display())))
}

fn read(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| Error::new(format!("reading {}: {e}", path.display())))
}

/// Press `region` and wait for the next `event` line.
fn pressed(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
    event: &str,
) -> Result<std::result::Result<TraceLine, String>> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, region)?;
    Ok(await_line(session, event, mark, |_| true)?.ok_or_else(|| {
        format!(
            "`{region}` was pressed and no `{event}` followed. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

macro_rules! wait {
    ($report:expr, $e:expr) => {
        match $e? {
            Ok(line) => {
                $report.note(format!("`{}`", line.raw));
                line
            }
            Err(why) => return Ok(Some(why)),
        }
    };
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    files: &Files,
) -> Result<Option<String>> {
    put(files.profile_in, BROKEN)?;
    let refused = wait!(
        report,
        pressed(ctx, session, pointer, "preflight.load_profile", LOADED)
    );
    if refused.get("ok") != Some("0") || refused.get("why").is_none() {
        return Ok(Some(format!(
            "`{}` for a profile naming an unknown check; expected ok=0 with a why=.",
            refused.raw
        )));
    }
    put(files.profile_in, GOOD)?;
    let loaded = wait!(
        report,
        pressed(ctx, session, pointer, "preflight.load_profile", LOADED)
    );
    if loaded.get("ok") != Some("1") || loaded.get_usize("rules") != Some(1) {
        return Ok(Some(format!(
            "`{}` for a one-rule profile; expected ok=1 rules=1.",
            loaded.raw
        )));
    }
    let ran = wait!(
        report,
        pressed(ctx, session, pointer, "preflight.run", FINISHED)
    );
    if ran.get("profile") != Some("custom") || ran.get_usize("findings").unwrap_or(0) == 0 {
        return Ok(Some(format!(
            "`{}`; expected profile=custom with findings>0 (the fixture has no trim box).",
            ran.raw
        )));
    }
    if let Some(failure) = saved_report(ctx, report, session, pointer, files)? {
        return Ok(Some(failure));
    }
    let saved = wait!(
        report,
        pressed(
            ctx,
            session,
            pointer,
            "preflight.save_profile",
            PROFILE_SAVED
        )
    );
    let text = read(files.profile_out)?;
    if saved.get("ok") != Some("1")
        || !text.contains("\"needs-trim\"")
        || !text.contains("no-trim-box")
    {
        return Ok(Some(format!(
            "`{}`, and the saved profile reads {text:?}; expected rule needs-trim with check \
             no-trim-box.",
            saved.raw
        )));
    }
    Ok(None)
}

/// Save report…: the file holds the engine's text for the loaded profile.
fn saved_report(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    files: &Files,
) -> Result<Option<String>> {
    let saved = wait!(
        report,
        pressed(ctx, session, pointer, "preflight.save_report", REPORT_SAVED)
    );
    let text = read(files.report_out)?;
    report.artifact(files.report_out.to_path_buf());
    let whole = saved.get("ok") == Some("1") && saved.get_usize("bytes") == Some(text.len());
    let engine = text.starts_with("Preflight report: ") && text.contains("Rule needs-trim (error)");
    if !(whole && engine) {
        return Ok(Some(format!(
            "`{}`, and the file ({} bytes) begins {:?}; expected the trace's byte count, \
             `Preflight report: ` and `Rule needs-trim (error)`.",
            saved.raw,
            text.len(),
            text.chars().take(160).collect::<String>()
        )));
    }
    Ok(None)
}
