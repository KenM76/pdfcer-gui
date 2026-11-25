//! `each_tab_keeps_its_own_position` — scrolling one document leaves every
//! other open document where it was. See
//! `docs/modules/ui-verify/checks/tab_keeps_position.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, declared};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::trace::Trace;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Two multi-page documents, so each has somewhere to scroll to.
const FIRST: &str = "four-pages.pdf";
const SECOND: &str = "drawing-sheets.pdf";
const OPEN_PATH_ENV: &str = "PDFCER_DIAG_OPEN_PATH"; // ui-text-exempt: an environment variable name
/// `canvas-pos at=<x>,<y> …`: the page point at the viewport's centre.
const POSITION: &str = "canvas-pos"; // ui-text-exempt: a trace event name
const VIEWPORT: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
const ACTIVATE: &str = "document-activate"; // ui-text-exempt: a trace event name
const FIRST_TAB: &str = "doc-tab.0"; // ui-text-exempt: a trace region name
/// Wheel lines on the first document, and the far larger scroll on the
/// second: a shared offset would carry the second's into the first.
const FIRST_LINES: f32 = -6.0;
const SECOND_LINES: f32 = -30.0;
/// Page points the first document may move by across the switch.
const TOLERANCE: f64 = 1.0;

pub struct EachTabKeepsItsOwnPosition;

impl Check for EachTabKeepsItsOwnPosition {
    fn name(&self) -> &'static str {
        "each_tab_keeps_its_own_position"
    }

    fn defect(&self) -> &'static str {
        "scrolling one open document moves the others, so switching back to a tab shows a \
         different place from the one the operator left"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The last `canvas-pos` point.
fn position(trace: &Trace) -> Result<(f64, f64)> {
    let line = trace
        .last(POSITION)
        .ok_or_else(|| Error::new(format!("the canvas traced no `{POSITION}` line.")))?;
    match line.get_f64_list("at").as_deref() {
        Some([x, y]) => Ok((*x, *y)),
        _ => Err(Error::new(format!("`{}` carries no `at=x,y`.", line.raw))),
    }
}

/// Wheel `lines` at the canvas viewport's centre and return where it landed.
fn scroll(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    lines: f32,
) -> Result<(f64, f64)> {
    let viewport = declared(&session.trace()?, ui_rect, VIEWPORT)
        .ok_or_else(|| Error::new(format!("no `{VIEWPORT}` region to scroll.")))?;
    for _ in 0..(lines.abs() / 3.0).ceil() as usize {
        pointer.wheel_in(
            session,
            None,
            WindowPoint::centre_of(viewport),
            3.0 * lines.signum(),
        )?;
        session.settle(6);
    }
    session.settle(20);
    position(&session.trace()?)
}

/// Launch off-screen on the first document, with Open answering the second.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
) -> Result<(Session, ScriptedPointer, &'static str)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let first = driving::repo_fixture(FIRST, "It is a committed fixture.")?;
    let second = driving::repo_fixture(
        SECOND,
        "Run `python fixtures/drawing-sheets.PROVENANCE.py`.",
    )?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("tab_keeps_position.trace.txt"));
    spec.pdf = Some(first);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push((OPEN_PATH_ENV.to_owned(), second.display().to_string()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("tab_keeps_position.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    Ok((session, pointer, ui_rect))
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer, ui_rect) = launch(ctx, report)?;
    let opened = position(&session.trace()?)?;
    let left = scroll(&session, &pointer, ui_rect, FIRST_LINES)?;
    report.note(format!(
        "first document scrolled from {opened:?} to {left:?}"
    ));
    if (left.1 - opened.1).abs() <= TOLERANCE {
        return Err(Error::new(
            "the wheel did not move the first document, so there is no position to keep.",
        ));
    }

    pointer.key(&session, None, "O", Some("ctrl"))?;
    session.settle(60);
    let arrived = position(&session.trace()?)?;
    let far = scroll(&session, &pointer, ui_rect, SECOND_LINES)?;
    report.note(format!(
        "second document opened at {arrived:?} and was scrolled to {far:?}"
    ));

    pointer.click_region(&session, ui_rect, FIRST_TAB)?;
    session.settle(40);
    let trace = session.trace()?;
    match trace.last(ACTIVATE) {
        Some(line) if line.get("slot") == Some("0") => {}
        _ => {
            return Err(Error::new(format!(
                "clicking `{FIRST_TAB}` traced no `{ACTIVATE} slot=0`, so the first document \
                 was never brought back."
            )));
        }
    }
    let back = position(&trace)?;
    report.note(format!("back on the first document at {back:?}"));
    let moved = ((back.0 - left.0).powi(2) + (back.1 - left.1).powi(2)).sqrt();
    if moved > TOLERANCE {
        return Ok(Some(format!(
            "★ the first document was left at {left:?} and came back at {back:?}, {moved:.1} pt \
             away, after only the second document was scrolled."
        )));
    }
    Ok(None)
}
