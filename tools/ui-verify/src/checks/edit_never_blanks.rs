//! `an_edit_never_blanks_or_blocks` — one edit on a multi-page drawing
//! re-renders only the page it touched, never leaves a page or a thumbnail
//! blank while the new picture is drawn, and leaves the next input accepted
//! promptly.
//!
//! Two cases, one launch each: a page-scoped edit (Edit mode, drag an object)
//! and a markup added in Review mode. Each case reads the trace after the
//! edit for which thumbnails and strip pages were re-rendered, for any frame
//! that drew a page or a tile with no picture, and for frames that held the UI
//! thread; and times, from outside the process, how long pointer steps sent
//! after the edit took to be accepted.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/edit_never_blanks.md`.

use std::time::{Duration, Instant};

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "heavy-pages.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/heavy-pages.PROVENANCE.py`.";
/// The fixture's movable rectangle, `40 700 120 60 re f`, by its centre.
const RECT_CENTRE: (f64, f64) = (100.0, 730.0);
/// Where a Review-mode rectangle markup is drawn, corner to corner.
const MARKUP: ((f64, f64), (f64, f64)) = ((250.0, 710.0), (400.0, 770.0));

const THUMB: &str = "pages-thumbnail";
const TILES: &str = "pages-tiles";
const BLANK: &str = "canvas-pages-blank";
const SPAWN: &str = "render-spawn";
const LONG: &str = "frame-long";
const HIT: &str = "hit-test";
const BUILT: &str = "page-objects-built";
const PRESS: &str = "canvas-press";
/// The most of one frame the interface may spend on its own work, with the
/// engine's traced time (the edit's verb, the page model it rebuilds, a hit
/// test) taken out. More is the UI thread held by the GUI.
const LONG_FRAME_MS: u64 = 120;
/// The longest a pointer step sent after the edit may wait to be accepted,
/// beyond the time the edit's own engine work took.
const NEXT_INPUT_MS: u128 = 300;
/// Engine hit tests one press may cost: one at the pick tolerance and one
/// dead-on. A drag that repeats them on every frame exceeds this.
const HITS_PER_PRESS: usize = 2;
/// How many probe steps follow the edit, and how far apart.
const PROBES: usize = 12;
const PROBE_GAP: Duration = Duration::from_millis(300);
/// How long the thumbnails may take to fill, at launch and after the edit.
const FILL_TIMEOUT: Duration = Duration::from_secs(90);

/// One driven case.
#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    invoke: &'static str,
    /// The `{label}` the funnel traces when this case's edit commits.
    applied: &'static str,
}

const CASES: [Case; 2] = [
    Case {
        name: "move",
        invoke: "mode.edit,view.page_continuous,view.zoom_fit_page",
        applied: "move-objects",
    },
    Case {
        name: "markup",
        invoke: "mode.review,view.page_continuous,view.zoom_fit_page,\
                 markup.rectangle",
        applied: "add-markup",
    },
];

/// See the module documentation.
pub struct AnEditNeverBlanksOrBlocks;

impl Check for AnEditNeverBlanksOrBlocks {
    fn name(&self) -> &'static str {
        "an_edit_never_blanks_or_blocks"
    }

    fn defect(&self) -> &'static str {
        "an edit blanks the page or the page previews while they redraw, re-renders pages it \
         did not touch, or holds the interface so the next click waits for the redraw"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut failures = Vec::new();
        for case in CASES {
            match drive(ctx, &mut report, case) {
                Ok(mut found) => failures.append(&mut found),
                Err(why) => return report.from_error(&why),
            }
        }
        if failures.is_empty() {
            report.pass()
        } else {
            report.fail(failures.join(" "))
        }
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    case: Case,
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
    let doc = ctx.out(&format!("never-blanks-{}.pdf", case.name));
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(
        &exe,
        ctx.out(&format!("never_blanks_{}.trace.txt", case.name)),
    );
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", case.invoke),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(
        &mut spec,
        ctx.out(&format!("never_blanks_{}.pointer.txt", case.name)),
    )?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("{}: launched as pid {}", case.name, session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

/// Launch, wait for the rail to fill, make the edit, probe, wait for the
/// redraw, and judge. Returns the failures found; an `Err` is a harness fault.
fn drive(ctx: &CheckContext, report: &mut CheckReport, case: Case) -> Result<Vec<String>> {
    let (session, pointer) = launch(ctx, report, case)?;
    let outcome = edit_and_judge(ctx, report, case, &session, &pointer);
    let parked = pointer.gone(&session);
    let found = outcome?;
    parked?;
    Ok(found)
}

fn edit_and_judge(
    ctx: &CheckContext,
    report: &mut CheckReport,
    case: Case,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Vec<String>> {
    let filled = wait_until(session, FILL_TIMEOUT, |t| tiles_quiet(t, 0))?;
    report.note(format!(
        "{}: the rail filled in {} ms; first renders {}",
        case.name,
        filled.as_millis(),
        thumbnail_costs(&session.trace()?, 0)
    ));
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let (from, to) = gesture(&mapping, case)?;

    let mark = session.trace()?.mark();
    let started = Instant::now();
    pointer.drag(session, from, to, 8)?;
    let edit_ms = started.elapsed().as_millis();
    let latencies = probe(session, pointer, &mapping)?;
    let mut failures = Vec::new();
    if session.trace()?.last_after(case.applied, mark).is_none() {
        failures.push(format!(
            "{}: the edit never committed — no `{}` line after the drag.",
            case.name, case.applied
        ));
        return Ok(failures);
    }
    let settled = wait_until(session, FILL_TIMEOUT, |t| {
        t.last_after(THUMB, mark).is_some() && tiles_quiet(t, mark)
    })?;
    let trace = session.trace()?;
    report.note(format!(
        "{}: drag accepted in {edit_ms} ms; the edit spent {} ms in the engine; probe \
         latencies {latencies:?} ms; the rail settled {} ms after the probes; redraws {}",
        case.name,
        edit_engine_ms(&trace, mark, case),
        settled.as_millis(),
        thumbnail_costs(&trace, mark)
    ));
    judge(&trace, mark, case, &latencies, &mut failures);
    Ok(failures)
}

/// The drag that makes this case's edit, in window coordinates.
fn gesture(mapping: &CanvasMapping, case: Case) -> Result<(WindowPoint, WindowPoint)> {
    let at = |(x, y): (f64, f64)| mapping.doc_to_window(DocPoint::new(0, x, y));
    if case.name == "move" {
        let (x, y) = RECT_CENTRE;
        Ok((at((x, y))?, at((x + 60.0, y))?))
    } else {
        Ok((at(MARKUP.0)?, at(MARKUP.1)?))
    }
}

/// Hover steps sent at a fixed gap after the edit, each timed send-to-accept.
fn probe(
    session: &Session,
    pointer: &ScriptedPointer,
    mapping: &CanvasMapping,
) -> Result<Vec<u128>> {
    let a = mapping.doc_to_window(DocPoint::new(0, 500.0, 300.0))?;
    let b = mapping.doc_to_window(DocPoint::new(0, 520.0, 320.0))?;
    let mut out = Vec::with_capacity(PROBES);
    for i in 0..PROBES {
        let started = Instant::now();
        pointer.hover(session, if i % 2 == 0 { a } else { b })?;
        out.push(started.elapsed().as_millis());
        std::thread::sleep(PROBE_GAP);
    }
    Ok(out)
}

/// Every verdict this case owes, read from the trace after `mark`.
fn judge(trace: &Trace, mark: usize, case: Case, latencies: &[u128], failures: &mut Vec<String>) {
    let after = |name: &'static str| {
        trace
            .lines
            .iter()
            .filter(move |l| l.lineno > mark && l.event == name)
    };
    let redrawn: std::collections::BTreeSet<usize> =
        after(THUMB).filter_map(|l| l.get_usize("page")).collect();
    if redrawn.iter().any(|p| *p != 1) {
        failures.push(format!(
            "{}: an edit on page 1 re-rendered the thumbnails of pages {redrawn:?}.",
            case.name
        ));
    }
    let strip: std::collections::BTreeSet<usize> =
        after(SPAWN).filter_map(|l| l.get_usize("page")).collect();
    if strip.iter().any(|p| *p != 0) {
        failures.push(format!(
            "{}: an edit on page 1 re-rendered strip pages (0-based) {strip:?}.",
            case.name
        ));
    }
    if let Some(line) = after(TILES).find(|l| l.get_usize("blank").is_some_and(|b| b > 0)) {
        failures.push(format!(
            "{}: a thumbnail went blank: `{}`.",
            case.name, line.raw
        ));
    }
    if let Some(line) = after(BLANK).find(|l| l.get("pages") != Some("-")) {
        failures.push(format!(
            "{}: a page on the canvas went blank: `{}`.",
            case.name, line.raw
        ));
    }
    if let Some((line, gui)) = gui_held(trace, mark) {
        failures.push(format!(
            "{}: a frame after the edit held the interface for {gui} ms of its own work:              `{}`.",
            case.name, line.raw
        ));
    }
    let presses = after(PRESS).count();
    let hits = after(HIT).count();
    if hits > presses * HITS_PER_PRESS {
        failures.push(format!(
            "{}: {hits} engine hit tests for {presses} presses; a drag is repeating the press's hit test on every frame.",
            case.name
        ));
    }
    let engine = edit_engine_ms(trace, mark, case);
    if let Some(worst) = latencies
        .iter()
        .max()
        .filter(|ms| **ms > NEXT_INPUT_MS + engine)
    {
        failures.push(format!(
            "{}: an input sent after the edit waited {worst} ms to be accepted (limit {NEXT_INPUT_MS} ms beyond the edit's {engine} ms in the engine).",
            case.name
        ));
    }
}

/// The first `frame-long` line after `mark` whose own work, its traced engine
/// time taken out, exceeds [`LONG_FRAME_MS`], with that remainder.
fn gui_held(trace: &Trace, mark: usize) -> Option<(&crate::trace::TraceLine, u64)> {
    let mut since = mark;
    for line in trace.lines.iter().filter(|l| l.lineno > mark) {
        if line.event != LONG {
            continue;
        }
        let engine: u64 = trace
            .lines
            .iter()
            .filter(|l| l.lineno > since && l.lineno < line.lineno && is_engine(l))
            .filter_map(|l| l.get_usize("ms"))
            .map(|ms| ms as u64)
            .sum();
        since = line.lineno;
        let gui = (line.get_usize("ms")? as u64).saturating_sub(engine);
        if gui > LONG_FRAME_MS {
            return Some((line, gui));
        }
    }
    None
}

/// Whether `line` reports time spent inside the engine.
fn is_engine(line: &crate::trace::TraceLine) -> bool {
    [HIT, BUILT, "move-objects", "add-markup"].contains(&line.event.as_str())
}

/// The edit's own engine time: its verb, and the page model rebuilt after it.
fn edit_engine_ms(trace: &Trace, mark: usize, case: Case) -> u128 {
    let Some(applied) = trace.last_after(case.applied, mark) else {
        return 0;
    };
    let rebuilt = trace
        .lines
        .iter()
        .find(|l| l.lineno > applied.lineno && l.event == BUILT)
        .and_then(|l| l.get_usize("ms"))
        .unwrap_or(0);
    (applied.get_usize("ms").unwrap_or(0) + rebuilt) as u128
}

/// Whether the newest `pages-tiles` line after `mark` says nothing is pending.
fn tiles_quiet(trace: &Trace, mark: usize) -> bool {
    trace
        .last_after(TILES, mark)
        .is_some_and(|l| l.get_usize("pending") == Some(0) && l.get_usize("blank") == Some(0))
}

/// `page:ms` for every thumbnail render after `mark`.
fn thumbnail_costs(trace: &Trace, mark: usize) -> String {
    let costs: Vec<String> = trace
        .lines
        .iter()
        .filter(|l| l.lineno > mark && l.event == THUMB)
        .map(|l| {
            format!(
                "{}:{}",
                l.get("page").unwrap_or("?"),
                l.get("ms").unwrap_or("?")
            )
        })
        .collect();
    if costs.is_empty() {
        "none".to_owned()
    } else {
        costs.join(" ")
    }
}

/// Poll the trace until `done` holds; the elapsed time, or an error at `limit`.
fn wait_until(
    session: &Session,
    limit: Duration,
    done: impl Fn(&Trace) -> bool,
) -> Result<Duration> {
    let started = Instant::now();
    loop {
        if done(&session.trace()?) {
            return Ok(started.elapsed());
        }
        if started.elapsed() > limit {
            return Err(Error::new(format!(
                "the page previews did not settle within {} s. Trace: {}.",
                limit.as_secs(),
                session.trace_path().display()
            )));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
