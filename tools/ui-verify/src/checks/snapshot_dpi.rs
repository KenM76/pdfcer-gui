//! `the_snapshot_resolution_persists`: Settings ▸ Images takes a snapshot
//! resolution of 150 dpi, Save writes it to the preferences file, and a fresh
//! launch shows 150 again (`OPERATOR_REQUESTS.md` O272). Driven off the
//! desktop through the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snapshot_dpi.md`.

use std::path::Path;

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const OPEN_SETTINGS: &str = "file.settings";
const HEADING: &str = "settings.heading.images";
const FIELD: &str = "settings.snapshot.dpi";
const SAVE: &str = "dialog:settings.save";
/// Typed into the box; any value but the default proves the round trip.
const TYPED: &str = "150"; // ui-text-exempt: typed input, not prose
const SHOWN: &str = "snapshot-dpi-setting"; // ui-text-exempt: a trace event name, never displayed
const PREFS_SAVED: &str = "prefs-saved"; // ui-text-exempt: a trace event name, never displayed
const PREFS_FILE: &str = "preferences.txt";
/// The line Save must write.
const WRITTEN: &str = "snapshot_dpi = 150"; // ui-text-exempt: a preferences line

/// See the module documentation.
pub struct TheSnapshotResolutionPersists;

impl Check for TheSnapshotResolutionPersists {
    fn name(&self) -> &'static str {
        "the_snapshot_resolution_persists"
    }

    fn defect(&self) -> &'static str {
        "the snapshot resolution typed in Settings ▸ Images is not saved, or is back to \
         300 dpi after a restart"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    exe: &Path,
    stem: &str,
) -> Result<(Session, ScriptedPointer)> {
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), OPEN_SETTINGS.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(40);
    Ok((session, pointer))
}

/// Click a declared region in whichever viewport declared it; `None` when it
/// is not declared, else the viewport.
fn click(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<Option<Option<String>>> {
    let Some((rect, viewport)) = declared_in(&session.trace()?, ui_rect, name) else {
        return Ok(None);
    };
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(20);
    Ok(Some(viewport))
}

/// The dpi the page on show last traced.
fn shown(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .last(SHOWN)
        .and_then(|l| l.get("dpi").map(str::to_owned)))
}

/// Open the Images page; a failure sentence when it is missing.
fn open_page(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
    if click(session, pointer, ui_rect, HEADING)?.is_none() {
        return Ok(Some(format!(
            "the Settings window lists no Images page: no `{HEADING}`. Pages: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "settings.heading."
            ))
        )));
    }
    Ok(None)
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(&userdata, "")
        .map_err(|e| Error::new(format!("could not clear the preferences: {e}")))?;

    let (session, pointer) = launch(ctx, report, &exe, "snapshot-dpi")?;
    if let Some(failure) = open_page(&session, &pointer, ui_rect)? {
        return Ok(Some(failure));
    }
    let Some(vp) = click(&session, &pointer, ui_rect, FIELD)? else {
        return Ok(Some(format!(
            "the Images page declares no `{FIELD}` box. Regions: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "settings."))
        )));
    };
    pointer.key(&session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(&session, vp.as_deref(), TYPED)?;
    session.settle(20);
    if shown(&session)?.as_deref() != Some(TYPED) {
        return Ok(Some(format!(
            "{TYPED:?} was typed into the box and the page last traced `{SHOWN} dpi={}`.",
            shown(&session)?.unwrap_or_default()
        )));
    }
    let saves = session.trace()?.events(PREFS_SAVED).count();
    if click(&session, &pointer, ui_rect, SAVE)?.is_none() {
        return Ok(Some(format!("the Settings window declares no `{SAVE}`.")));
    }
    session.settle(20);
    if session.trace()?.events(PREFS_SAVED).count() == saves {
        return Ok(Some(format!(
            "Save was pressed and no `{PREFS_SAVED}` line followed."
        )));
    }
    pointer.gone(&session)?;
    drop(session);
    let text = std::fs::read_to_string(userdata.join(PREFS_FILE))
        .map_err(|e| Error::new(format!("cannot read the preferences file: {e}")))?;
    if !text.lines().any(|l| l.trim() == WRITTEN) {
        return Ok(Some(format!(
            "Save wrote no `{WRITTEN}` line to {}.",
            userdata.join(PREFS_FILE).display()
        )));
    }
    report.note("Save wrote the typed resolution to the preferences file");

    let (session, pointer) = launch(ctx, report, &exe, "snapshot-dpi-relaunch")?;
    if let Some(failure) = open_page(&session, &pointer, ui_rect)? {
        return Ok(Some(failure));
    }
    let after = shown(&session)?;
    pointer.gone(&session)?;
    if after.as_deref() != Some(TYPED) {
        return Ok(Some(format!(
            "after a restart the Images page shows `{SHOWN} dpi={}`, not {TYPED}. The saved \
             line was not read back.",
            after.unwrap_or_default()
        )));
    }
    report.note("a fresh launch shows the saved resolution");
    Ok(None)
}
