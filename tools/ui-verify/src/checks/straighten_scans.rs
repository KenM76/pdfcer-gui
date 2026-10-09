//! `straighten_scans_turns_tilted_pages_and_leaves_the_rest` — File ▸
//! Straighten scans measures every page, straightens the tilted ones, leaves
//! a level page and, by default, a page carrying text alone, and folds the
//! corrections into one undo entry. Driven with the scripted pointer in a
//! window placed off the desktop, on a copy of `fixtures/skewed-scans.pdf`,
//! whose tilts are known from how it was drawn.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::TraceLine;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "skewed-scans.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/skewed-scans.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.deskew"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const COMMIT: &str = "deskew.commit"; // ui-text-exempt: a trace region name, never displayed
const PAGE: &str = "deskew-page"; // ui-text-exempt: a trace event name, never displayed
const FINISHED: &str = "deskew-finished"; // ui-text-exempt: a trace event name, never displayed

/// Each page's tilt as drawn, degrees counter-clockwise, and what the run
/// must make of it.
const EXPECTED: [(&str, Option<f64>); 4] = [
    ("straightened", Some(2.0)),
    ("straightened", Some(-1.5)),
    ("level", None),
    ("has-text", None),
];

/// How far a measured angle may sit from the drawn one: the detector's
/// fine step is 0.05 degrees.
const TOLERANCE: f64 = 0.1;

/// See the module documentation.
pub struct StraightenScansTurnsTiltedPages;

impl Check for StraightenScansTurnsTiltedPages {
    fn name(&self) -> &'static str {
        "straighten_scans_turns_tilted_pages_and_leaves_the_rest"
    }

    fn defect(&self) -> &'static str {
        "Straighten scans misses a tilted page, turns a level one or one carrying text, \
         measures the wrong angle, or leaves the run as one undo step per page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = drive(&mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            let outcome = outcome?;
            parked?;
            Ok(outcome)
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Open the window, press Straighten with its defaults, read the run.
fn drive(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    click(session, pointer, TAB)?;
    let trace = session.trace()?;
    if declared(&trace, UI_RECT, ITEM).is_none() && declared(&trace, UI_RECT, COLLAPSED).is_some() {
        click(session, pointer, COLLAPSED)?;
    }
    click(session, pointer, ITEM)?;
    session.settle(20);
    click(session, pointer, COMMIT)?;
    let Some(finished) = await_finished(session)? else {
        return Ok(Some(format!(
            "Straighten was pressed and no `{FINISHED}` line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("finished: `{}`", finished.raw));
    let trace = session.trace()?;
    let pages: Vec<TraceLine> = trace.events(PAGE).cloned().collect();
    for line in &pages {
        report.note(format!("page: `{}`", line.raw));
    }
    if let Some(wrong) = judge_pages(&pages) {
        return Ok(Some(wrong));
    }
    if finished.get("corrected") != Some("2") || finished.get("folded") != Some("one") {
        return Ok(Some(format!(
            "the run ended `{}`: two pages were straightened and they should be one undo \
             entry (`corrected=2 folded=one`).",
            finished.raw
        )));
    }
    Ok(None)
}

/// Each page's line against [`EXPECTED`]; the first disagreement, worded.
fn judge_pages(pages: &[TraceLine]) -> Option<String> {
    if pages.len() != EXPECTED.len() {
        return Some(format!(
            "the run logged {} page(s), not the fixture's {}.",
            pages.len(),
            EXPECTED.len()
        ));
    }
    for (index, (line, (outcome, tilt))) in pages.iter().zip(EXPECTED).enumerate() {
        let page = index.to_string();
        if line.get("page") != Some(page.as_str()) || line.get("outcome") != Some(outcome) {
            return Some(format!(
                "page {} should be `{outcome}` and the run logged `{}`.",
                index + 1,
                line.raw
            ));
        }
        if let Some(drawn) = tilt {
            let measured = line.get("degrees").and_then(|d| d.parse::<f64>().ok());
            if !measured.is_some_and(|m| (m - drawn).abs() <= TOLERANCE) {
                return Some(format!(
                    "page {} was drawn tilted {drawn}° and the run measured {measured:?}: `{}`.",
                    index + 1,
                    line.raw
                ));
            }
        }
    }
    None
}

/// Wait for the run's end: four pages of most of a second each.
fn await_finished(session: &Session) -> Result<Option<TraceLine>> {
    for _ in 0..120 {
        if let Some(line) = session.trace()?.events(FINISHED).last() {
            return Ok(Some(line.clone()));
        }
        session.settle(10);
    }
    Ok(None)
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
    let source = repo_fixture(FIXTURE, METHOD)?;
    let doc = ctx.out("straighten-scans.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("straighten-scans.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("straighten-scans.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(45);
    Ok((session, pointer))
}

/// Click `name` in whichever viewport declared it.
fn click(session: &Session, pointer: &ScriptedPointer, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let (rect, viewport) = declared_in(&trace, UI_RECT, name).ok_or_else(|| {
        let prefix = name.rsplit_once('.').map_or(name, |(head, _)| head);
        Error::new(format!(
            "no `{name}` region. Declared under `{prefix}`: {}.",
            list(&declared_names(&trace, UI_RECT, prefix))
        ))
    })?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(15);
    Ok(())
}
