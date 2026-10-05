//! `print_shop` — the render notes name the colour findings, and View ▸
//! Display ▸ Skip tiny details leaves out what is too small to see and says
//! how much.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/print_shop.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::TraceLine;

const OFFSCREEN: &str = "-4200,-4200,1200,1350";
const INVOKE: &str = "mode.edit,tools.render_diagnostics";
const FIXTURE: &str = "print-shop.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/print-shop.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const FINDINGS: &str = "render-findings"; // ui-text-exempt: a trace event name, never displayed
const TINY: &str = "canvas-tiny"; // ui-text-exempt: a trace event name, never displayed
const VIEW_TAB: &str = "ribbon.tab.view";
const SKIP_TINY: &str = "ribbon.item.view.skip_tiny_details";
const DISCLOSURE: &str = "status-group:tiny-details";
/// The fixture's 0.2 pt forms, every one under half a pixel at fit.
const SPECKS: &str = "400";

/// See the module documentation.
pub struct PrintShopFindingsAndTinyDetails;

impl Check for PrintShopFindingsAndTinyDetails {
    fn name(&self) -> &'static str {
        "print_shop"
    }

    fn defect(&self) -> &'static str {
        "the render notes stay silent about a colour the page could not resolve or a gradient \
         it could not draw, or Skip tiny details changes nothing, skips with the toggle off, or \
         skips without saying so"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = steps(&mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("print_shop.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("print_shop.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

/// The newest `name` line after `mark` satisfying `ok`, waiting up to 30 settles.
fn await_line(
    session: &Session,
    name: &str,
    mark: usize,
    ok: impl Fn(&TraceLine) -> bool,
) -> Result<Option<TraceLine>> {
    for _ in 0..30 {
        let trace = session.trace()?;
        if let Some(line) = trace
            .events(name)
            .filter(|l| l.lineno > mark && ok(l))
            .last()
        {
            return Ok(Some(line.clone()));
        }
        session.settle(10);
    }
    Ok(None)
}

/// Click the declared `region`.
fn press(session: &Session, pointer: &ScriptedPointer, region: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, region).ok_or_else(|| {
        Error::new(format!(
            "no `{region}` region. Ribbon items declared: {}.",
            list(&declared_names(&trace, UI_RECT, "ribbon.item."))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}

fn steps(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = findings(report, session)? {
        return Ok(Some(failure));
    }
    tiny_details(report, session, pointer)
}

/// The Render diagnostics window reports both colour findings by name.
fn findings(report: &mut CheckReport, session: &Session) -> Result<Option<String>> {
    let Some(line) = await_line(session, FINDINGS, 0, |_| true)? else {
        return Ok(Some(format!(
            "`{INVOKE}` was invoked and no `{FINDINGS}` line followed: the Render diagnostics \
             window never listed the page's findings."
        )));
    };
    report.note(format!("findings: `{}`", line.raw));
    let count = |k: &str| line.get(k).and_then(|v| v.parse::<usize>().ok());
    let (cs, sh, shown) = (
        count("cs_unresolved"),
        count("shadings_refused"),
        count("shown"),
    );
    if cs != Some(1) || sh != Some(1) || shown.is_none_or(|n| n < 2) {
        return Ok(Some(format!(
            "the fixture names one unresolvable colour space and one missing shading; the \
             window read cs_unresolved={cs:?} shadings_refused={sh:?} shown={shown:?}, where \
             1, 1 and at least 2 sentences were owed. `{}`.",
            line.raw
        )));
    }
    Ok(None)
}

/// Off skips nothing; on skips all 400 specks and the status bar says so.
fn tiny_details(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let off = |l: &TraceLine| l.get("mode") == Some("0") && l.get("skipped") != Some("none");
    let Some(before) = await_line(session, TINY, 0, off)? else {
        return Ok(Some(format!(
            "no `{TINY} mode=0` line with a count: the canvas never reported a render."
        )));
    };
    report.note(format!("before: `{}`", before.raw));
    if before.get("skipped") != Some("0") {
        return Ok(Some(format!(
            "with Skip tiny details OFF the render skipped something: `{}`. Off must draw \
             everything.",
            before.raw
        )));
    }
    let mark = session.trace()?.mark();
    press(session, pointer, VIEW_TAB)?;
    press(session, pointer, SKIP_TINY)?;
    let on = |l: &TraceLine| l.get("mode") == Some("1") && l.get("skipped") == Some(SPECKS);
    let Some(after) = await_line(session, TINY, mark, on)? else {
        let last = session.trace()?.last(TINY).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "`{SKIP_TINY}` was pressed and no `{TINY} skipped={SPECKS} mode=1` followed; last \
             was {last:?}. Either the toggle never reached the render request or the engine \
             skipped a different number than the fixture's {SPECKS} specks."
        )));
    };
    report.note(format!("after: `{}`", after.raw));
    let trace = session.trace()?;
    if declared_in(&trace, UI_RECT, DISCLOSURE).is_none() {
        return Ok(Some(format!(
            "the render skipped {SPECKS} specks and `{DISCLOSURE}` was never declared: the \
             canvas stopped showing what will print without saying so."
        )));
    }
    Ok(None)
}
