//! `print_shop` — the render notes name the colour findings, an unpainted
//! pattern among them, and View ▸
//! Display ▸ Skip tiny details leaves out what is too small to see and says
//! how much. `ink_picker` — Tools ▸ Diagnostics ▸ Ink picker reads the inks at
//! a click, overprint included. `overprint_preview` — View ▸ Display ▸
//! Overprint preview starts on and turning it off reaches the render and the
//! status bar.
//! `output_preview` — View ▸ Panels ▸ Output Preview lists the page's inks and
//! hides one, and the status bar says so. `output_coverage` — resting the
//! pointer on the page shows each ink's percentage and the total there.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/print_shop.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, declared_names, list, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::TraceLine;

const OFFSCREEN: &str = "-4200,-4200,1200,1350";
const INVOKE: &str = "mode.edit,tools.render_diagnostics";
/// The Tools tab is an Edit-mode tab.
const EDIT: &str = "mode.edit";
const FIXTURE: &str = "print-shop.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/print-shop.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const FINDINGS: &str = "render-findings"; // ui-text-exempt: a trace event name, never displayed
const TINY: &str = "canvas-tiny"; // ui-text-exempt: a trace event name, never displayed
const VIEW_TAB: &str = "ribbon.tab.view";
const SKIP_TINY: &str = "ribbon.item.view.skip_tiny_details";
const DISCLOSURE: &str = "status-group:tiny-details";
const OVERPRINT: &str = "canvas-overprint"; // ui-text-exempt: a trace event name, never displayed
const OVERPRINT_ITEM: &str = "ribbon.item.view.overprint_preview";
const OVERPRINT_NOTE: &str = "status-group:overprint";
/// The fixture's 0.2 pt forms, every one under half a pixel at fit.
const SPECKS: &str = "400";
const TOOLS_TAB: &str = "ribbon.tab.tools";
const PICKER: &str = "ribbon.item.tools.ink_picker";
const ARMED: &str = "ink-picker"; // ui-text-exempt: a trace event name, never displayed
const PROBE: &str = "ink-probe"; // ui-text-exempt: a trace event name, never displayed
const SHOWN: &str = "ink-picker-shown"; // ui-text-exempt: a trace event name, never displayed
/// Page points to click and the cyan and magenta owed there: cyan alone, then
/// the overprinted magenta over it, which a press prints as both.
const AIMS: [(f64, f64, &str, &str); 2] = [
    (320.0, 110.0, "1.000", "0.000"),
    (400.0, 152.0, "1.000", "1.000"),
];

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
        let driven =
            launch(ctx, &mut report, self.name(), Some(INVOKE)).and_then(|(session, pointer)| {
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

/// See the module documentation.
pub struct InkPickerReadsOverprint;

impl Check for InkPickerReadsOverprint {
    fn name(&self) -> &'static str {
        "ink_picker"
    }

    fn defect(&self) -> &'static str {
        "Tools ▸ Diagnostics ▸ Ink picker cannot be armed from the ribbon, a click reads no ink, \
         reads the screen colour instead of the inks, loses an overprinted ink, or the tool strip \
         never shows the reading"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven =
            launch(ctx, &mut report, self.name(), Some(EDIT)).and_then(|(session, pointer)| {
                let outcome = picker(ctx, &mut report, &session, &pointer);
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

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    name: &str,
    invoke: Option<&str>,
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
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{name}.trace.txt")));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    if let Some(invoke) = invoke {
        spec.env
            .push(("PDFCER_DIAG_INVOKE".to_owned(), invoke.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{name}.pointer.txt")))?;
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
    let (cs, sh, pat, shown) = (
        count("color.spaces_unresolved"),
        count("shading.refused"),
        count("color.patterns_unpainted"),
        count("shown"),
    );
    if cs != Some(1) || sh != Some(1) || pat != Some(1) || shown.is_none_or(|n| n < 3) {
        return Ok(Some(format!(
            "the fixture names one unresolvable colour space, one missing shading and one \
             missing pattern; the window read color.spaces_unresolved={cs:?} \
             shading.refused={sh:?} color.patterns_unpainted={pat:?} shown={shown:?}, where 1, \
             1, 1 and at least 3 sentences were owed. `{}`.",
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

/// Arm the picker from the ribbon, click cyan alone and then the overprinted
/// overlap, and read the inks the engine probed and the strip showed.
fn picker(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if await_line(session, TINY, 0, |_| true)?.is_none() {
        return Ok(Some(format!(
            "no `{TINY}` line: the canvas never rendered the page."
        )));
    }
    press(session, pointer, TOOLS_TAB)?;
    press(session, pointer, PICKER)?;
    let armed = |l: &TraceLine| l.get("armed") == Some("true");
    if await_line(session, ARMED, 0, armed)?.is_none() {
        return Ok(Some(format!(
            "`{PICKER}` was pressed and no `{ARMED} armed=true` followed: the ribbon item does \
             not arm the tool."
        )));
    }
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new(format!("cannot read a page size from {}.", pdf.display())))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    for (n, &(x, y, c, m)) in AIMS.iter().enumerate() {
        let mark = session.trace()?.mark();
        pointer.click(session, mapping.doc_to_window(DocPoint::new(0, x, y))?)?;
        let Some(line) = await_line(session, PROBE, mark, |_| true)? else {
            return Ok(Some(format!(
                "a click at {x}, {y} pt with the picker armed produced no `{PROBE}` line: the \
                 click never reached a probe render, or the render never answered."
            )));
        };
        report.note(format!("probe: `{}`", line.raw));
        if line.get("source") != Some("ink") || line.get("c") != Some(c) || line.get("m") != Some(m)
        {
            return Ok(Some(format!(
                "at {x}, {y} pt the fixture prints c={c} m={m} from the ink buffer; the probe \
                 read `{}`.",
                line.raw
            )));
        }
        if n == 0 && await_line(session, SHOWN, mark, |l| l.get("kind") == Some("inks"))?.is_none()
        {
            return Ok(Some(format!(
                "the probe read inks and no `{SHOWN} kind=inks` followed: the tool strip never \
                 showed the reading."
            )));
        }
    }
    Ok(None)
}

/// See the module documentation.
pub struct OverprintPreviewSwitch;

impl Check for OverprintPreviewSwitch {
    fn name(&self) -> &'static str {
        "overprint_preview"
    }

    fn defect(&self) -> &'static str {
        "View ▸ Display ▸ Overprint preview is missing, starts off, does not reach the render, \
         or turns overprint off without the status bar saying so"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report, self.name(), None).and_then(|(session, pointer)| {
            let outcome = overprint(&mut report, &session, &pointer);
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

/// A trace field read as a count; absent or unparsable reads as `None`.
fn count(line: &TraceLine, key: &str) -> Option<usize> {
    line.get(key).and_then(|v| v.parse().ok())
}

/// Simulated by default with the fixture's overprint taking effect; pressed
/// off, the page still asks for it, none takes effect, and the status bar says so.
fn overprint(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let on =
        |l: &TraceLine| l.get("mode") == Some("1") && count(l, "requested").is_some_and(|n| n > 0);
    let Some(before) = await_line(session, OVERPRINT, 0, on)? else {
        let last = session.trace()?.last(OVERPRINT).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "no `{OVERPRINT} mode=1` line with requested>0; last was {last:?}. Overprint preview \
             must start on, and the fixture's magenta asks for overprint."
        )));
    };
    report.note(format!("before: `{}`", before.raw));
    if !count(&before, "effective").is_some_and(|n| n > 0) {
        return Ok(Some(format!(
            "simulated by default yet no overprint took effect: `{}`.",
            before.raw
        )));
    }
    let mark = session.trace()?.mark();
    press(session, pointer, VIEW_TAB)?;
    let trace = session.trace()?;
    if declared_in(&trace, UI_RECT, OVERPRINT_ITEM).is_none() {
        return Ok(Some(format!(
            "`{OVERPRINT_ITEM}` is not on the View tab. Ribbon items declared: {}.",
            list(&declared_names(&trace, UI_RECT, "ribbon.item."))
        )));
    }
    press(session, pointer, OVERPRINT_ITEM)?;
    let off = |l: &TraceLine| {
        l.get("mode") == Some("0")
            && count(l, "requested").is_some_and(|n| n > 0)
            && count(l, "effective") == Some(0)
    };
    let Some(after) = await_line(session, OVERPRINT, mark, off)? else {
        let last = session.trace()?.last(OVERPRINT).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "`{OVERPRINT_ITEM}` was pressed and no `{OVERPRINT} mode=0 effective=0` with \
             requested>0 followed; last was {last:?}. The switch never reached the render."
        )));
    };
    report.note(format!("after: `{}`", after.raw));
    if declared_in(&session.trace()?, UI_RECT, OVERPRINT_NOTE).is_none() {
        return Ok(Some(format!(
            "overprint preview is off on a page that asks for it and `{OVERPRINT_NOTE}` was \
             never declared: the canvas stopped showing what will print without saying so."
        )));
    }
    Ok(None)
}

const OUTPUT_ITEM: &str = "ribbon.item.view.output_preview";
const OUTPUT_SWITCH: &str = "output-preview.on";
const OUTPUT_TRACE: &str = "output-preview"; // ui-text-exempt: a trace event name, never displayed
const OUTPUT_NOTE: &str = "status-group:output-preview";
const MAGENTA: &str = "output-preview.ink:Magenta";

/// See the module documentation.
pub struct OutputPreviewHidesAnInk;

impl Check for OutputPreviewHidesAnInk {
    fn name(&self) -> &'static str {
        "output_preview"
    }

    fn defect(&self) -> &'static str {
        "View ▸ Panels ▸ Output Preview is missing, lists no inks once switched on, cannot hide \
         one, or hides one without the status bar saying the page is not what will print"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report, self.name(), None).and_then(|(session, pointer)| {
            let outcome = output_preview(&mut report, &session, &pointer);
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

/// Open the panel, switch the preview on, read the four process plates, hide
/// Magenta, and require the hidden list and the status line to follow.
fn output_preview(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    press(session, pointer, VIEW_TAB)?;
    if declared_in(&session.trace()?, UI_RECT, OUTPUT_ITEM).is_none() {
        return Ok(Some(format!(
            "`{OUTPUT_ITEM}` is not on the View tab. Ribbon items declared: {}.",
            list(&declared_names(&session.trace()?, UI_RECT, "ribbon.item."))
        )));
    }
    press(session, pointer, OUTPUT_ITEM)?;
    let mark = session.trace()?.mark();
    press(session, pointer, OUTPUT_SWITCH)?;
    let listed = |l: &TraceLine| {
        l.get("on") == Some("1")
            && l.get("inks").is_some_and(|i| {
                ["Cyan", "Magenta", "Yellow", "Black"]
                    .iter()
                    .all(|k| i.contains(k))
            })
    };
    let Some(on) = await_line(session, OUTPUT_TRACE, mark, listed)? else {
        let last = session.trace()?.last(OUTPUT_TRACE).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "switched on and no `{OUTPUT_TRACE} on=1` line listed the four process inks; last \
             was {last:?}."
        )));
    };
    report.note(format!("on: `{}`", on.raw));
    let mark = session.trace()?.mark();
    press(session, pointer, MAGENTA)?;
    let hidden = |l: &TraceLine| l.get("on") == Some("1") && l.get("hidden") == Some("Magenta");
    let Some(after) = await_line(session, OUTPUT_TRACE, mark, hidden)? else {
        let last = session.trace()?.last(OUTPUT_TRACE).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "`{MAGENTA}` was pressed and no `hidden=Magenta` line followed; last was {last:?}."
        )));
    };
    report.note(format!("after: `{}`", after.raw));
    if declared_in(&session.trace()?, UI_RECT, OUTPUT_NOTE).is_none() {
        return Ok(Some(format!(
            "an ink is hidden and `{OUTPUT_NOTE}` was never declared: the canvas stopped showing \
             what will print without saying so."
        )));
    }
    Ok(None)
}

const COVERAGE: &str = "output-preview-coverage"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct OutputPreviewReadsTheInksUnderThePointer;

impl Check for OutputPreviewReadsTheInksUnderThePointer {
    fn name(&self) -> &'static str {
        "output_coverage"
    }

    fn defect(&self) -> &'static str {
        "with View ▸ Panels ▸ Output Preview on, resting the pointer on the page does not show \
         each ink's percentage and the total area coverage there, or shows the wrong ones"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report, self.name(), None).and_then(|(session, pointer)| {
            let outcome = coverage(ctx, &mut report, &session, &pointer);
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

/// Switch the preview on, rest the pointer on cyan alone and then on the
/// overprinted magenta, and require the panel's coverage line to read the
/// fixture's tints at each.
fn coverage(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    press(session, pointer, VIEW_TAB)?;
    press(session, pointer, OUTPUT_ITEM)?;
    let mark = session.trace()?.mark();
    press(session, pointer, OUTPUT_SWITCH)?;
    if await_line(session, OUTPUT_TRACE, mark, |l| l.get("on") == Some("1"))?.is_none() {
        return Ok(Some(format!(
            "`{OUTPUT_SWITCH}` was pressed and no `{OUTPUT_TRACE} on=1` followed."
        )));
    }
    // The dock is open now, so the canvas has its final width; map after it.
    let pdf = repo_fixture(FIXTURE, METHOD)?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new(format!("cannot read a page size from {}.", pdf.display())))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    for &(x, y, c, m) in &AIMS {
        let mark = session.trace()?.mark();
        pointer.hover(session, mapping.doc_to_window(DocPoint::new(0, x, y))?)?;
        let read = |l: &TraceLine| l.get("inks").is_some();
        let Some(line) = await_line(session, COVERAGE, mark, read)? else {
            let last = session.trace()?.last(COVERAGE).map(|l| l.raw.clone());
            return Ok(Some(format!(
                "the pointer rested at {x}, {y} pt and no `{COVERAGE}` line carried inks; last \
                 was {last:?}."
            )));
        };
        report.note(format!("at {x}, {y}: `{}`", line.raw));
        let inks = line.get("inks").unwrap_or_default();
        let owed = [format!("Cyan:{c}"), format!("Magenta:{m}")];
        let total = line.get("total").and_then(|t| t.parse::<f32>().ok());
        let at_least: f32 = [c, m].iter().filter_map(|v| v.parse::<f32>().ok()).sum();
        if !owed.iter().all(|o| inks.split('|').any(|i| i == o))
            || total.is_none_or(|t| t + 0.001 < at_least)
        {
            return Ok(Some(format!(
                "at {x}, {y} pt the fixture prints {} and {} (total at least {at_least}); the \
                 panel read `{}`.",
                owed[0], owed[1], line.raw
            )));
        }
    }
    Ok(None)
}
