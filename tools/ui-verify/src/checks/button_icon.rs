//! `checks::button_icon` — **a push button takes a picture from the
//! Properties panel, its caption can be moved beside it, and it can be
//! removed**
//!
//! Drives `all-field-kinds.pdf`'s push button `PushOne` off the desktop
//! through the scripted pointer, with the picker answered by the
//! `PDFCER_DIAG_IMAGE_PATH` seam.
//!
//! Oracles: with a PNG, *Choose picture…* traces `edit-widget-applied
//! field=PushOne` and the panel re-reads `button-icon-shown icon=present`;
//! the caption combo's *Picture only* entry re-reads `position=1`; *Remove
//! picture* re-reads `icon=absent`; each edit traces `redrawn=yes`, because the
//! fixture's artwork is another program's and kept it would hide the change.
//! With an SVG, *Choose picture…* traces
//! `button-icon-declined reason=svg` and applies no edit.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused; tall so the rows need little scrolling.
const OFFSCREEN: &str = "-4200,-4200,1200,1350";
const INVOKE: &str = "mode.edit,file.properties";
const FIXTURE: &str = "all-field-kinds.pdf";
const METHOD: &str = "One widget of every field kind pdfcer reads — see \
                      `fixtures/all-field-kinds.PROVENANCE.py`.";
const DRAWING: &str = "vector-art.svg";
const DRAWING_METHOD: &str = "See fixtures/vector-art.PROVENANCE.md.";
const FIELD: &str = "PushOne";
const CHOOSE: &str = "properties.widget_edit.button_icon.choose";
const REMOVE: &str = "properties.widget_edit.button_icon.remove";
const POSITION: &str = "properties.widget_edit.button_icon.position";
const ICON_ONLY: &str = "properties.widget_edit.button_icon.position.1";
const SHOWN: &str = "button-icon-shown";
const APPLIED: &str = "edit-widget-applied";
const DECLINED: &str = "button-icon-declined";

/// See the module documentation.
pub struct APushButtonTakesAPicture;

impl Check for APushButtonTakesAPicture {
    fn name(&self) -> &'static str {
        "a_push_button_takes_a_picture"
    }

    fn defect(&self) -> &'static str {
        "a push button cannot be given a picture, have its caption placed beside one, or lose \
         it, from anywhere in the shell"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = match picture(ctx, &mut report) {
            Ok(None) => drawing(ctx, &mut report),
            other => other,
        };
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Launch on a copy of the fixture with the picker answering `image`.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
    image: &std::path::Path,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    // Driven on a copy, so a stray save never touches the repository's fixture.
    let doc = ctx.out(&format!("{stem}.pdf"));
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{stem}.trace.txt")));
    spec.pdf = Some(doc);
    let image = image.display().to_string();
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
        ("PDFCER_DIAG_SELECT_FIELD", FIELD),
        ("PDFCER_DIAG_IMAGE_PATH", image.as_str()),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{stem}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Scroll the Properties panel until `region` is declared, then click it.
fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<()> {
    properties_pane::press(ctx, session, pointer, region, 30).map(|_| ())
}

/// The panel's latest `button-icon-shown` line for the button, as
/// `(icon, position)`.
fn shown(session: &Session) -> Result<Option<(String, String)>> {
    let trace = session.trace()?;
    Ok(trace
        .events(SHOWN)
        .filter(|l| l.get("field") == Some(FIELD))
        .last()
        .map(|l| {
            (
                l.get("icon").unwrap_or("-").to_owned(),
                l.get("position").unwrap_or("-").to_owned(),
            )
        }))
}

/// `None` when the latest edit redrew the button, which the fixture's
/// foreign artwork needs for the change to show at all.
fn undrawn(session: &Session, step: &str) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.events(APPLIED).last();
    Ok(line
        .is_none_or(|l| l.get("redrawn") != Some("yes"))
        .then(|| {
            format!(
                "★★★ {step} left the button's old artwork in place, so the change does not \
                 show: `{}`.",
                line.map_or("-", |l| l.raw.as_str())
            )
        }))
}

fn count(session: &Session, event: &str) -> Result<usize> {
    Ok(session.trace()?.events(event).count())
}

/// Park the pointer after the verdict, so a park failure never hides a FAIL.
fn finish(
    session: &Session,
    pointer: &ScriptedPointer,
    outcome: Result<Option<String>>,
) -> Result<Option<String>> {
    let parked = pointer.gone(session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// A PNG: choose, move the caption, remove.
fn picture(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let png = crate::png::encode_rgb(16, 16, &[200u8; 16 * 16 * 3])
        .ok_or_else(|| Error::new("the harness's own PNG encoder refused its fixture"))?;
    let path = ctx.out("button-icon.png");
    std::fs::write(&path, png)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", path.display())))?;
    let (session, pointer) = launch(ctx, report, "button-icon.picture", &path)?;
    let outcome = steps(ctx, report, &session, &pointer);
    finish(&session, &pointer, outcome)
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if shown(session)?.is_none_or(|(icon, _)| icon != "absent") {
        return Ok(Some(format!(
            "before any edit the panel traced no `{SHOWN} field={FIELD} icon=absent` line."
        )));
    }
    let applied = count(session, APPLIED)?;
    press(ctx, session, pointer, CHOOSE)?;
    let after = shown(session)?;
    report.note(format!("after Choose picture: {after:?}"));
    if count(session, APPLIED)? == applied
        || after.as_ref().is_none_or(|(icon, _)| icon != "present")
    {
        return Ok(Some(format!(
            "★★★ Choose picture with a PNG did not give the button an icon: panel shows {after:?}."
        )));
    }
    if let Some(failure) = undrawn(session, "Choose picture")? {
        return Ok(Some(failure));
    }
    press(ctx, session, pointer, POSITION)?;
    press(ctx, session, pointer, ICON_ONLY)?;
    let after = shown(session)?;
    report.note(format!("after Picture only: {after:?}"));
    if after.as_ref().is_none_or(|(_, position)| position != "1") {
        return Ok(Some(format!(
            "★★★ the caption combo's Picture only did not set /TP 1: panel shows {after:?}."
        )));
    }
    if let Some(failure) = undrawn(session, "Picture only")? {
        return Ok(Some(failure));
    }
    press(ctx, session, pointer, REMOVE)?;
    let after = shown(session)?;
    report.note(format!("after Remove picture: {after:?}"));
    if after.as_ref().is_none_or(|(icon, _)| icon != "absent") {
        return Ok(Some(format!(
            "★★★ Remove picture left the icon in place: panel shows {after:?}."
        )));
    }
    undrawn(session, "Remove picture")
}

/// An SVG is refused with a reason and changes nothing.
fn drawing(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let svg = repo_fixture(DRAWING, DRAWING_METHOD)?;
    let (session, pointer) = launch(ctx, report, "button-icon.drawing", &svg)?;
    let outcome = refused(ctx, report, &session, &pointer);
    finish(&session, &pointer, outcome)
}

fn refused(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let applied = count(session, APPLIED)?;
    press(ctx, session, pointer, CHOOSE)?;
    let trace = session.trace()?;
    let declined = trace.events(DECLINED).last();
    report.note(format!("SVG: {:?}", declined.map(|l| l.raw.clone())));
    if trace.events(APPLIED).count() != applied {
        return Ok(Some(
            "★★★ an SVG chosen as a button's picture applied an edit.".to_owned(),
        ));
    }
    Ok(declined
        .is_none_or(|l| l.get("reason") != Some("svg"))
        .then(|| {
            format!("★★★ an SVG chosen as a button's picture traced no `{DECLINED} reason=svg`.")
        }))
}
