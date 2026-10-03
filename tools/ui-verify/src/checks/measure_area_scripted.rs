//! `an_area_measures_the_region_it_encloses` — the Area tool traces a closed
//! outline, shows the enclosed area while it is traced, commits a closed
//! perimeter ce dimension labelled with that area, and its right-click menu
//! switches the label to the perimeter and back. Driven through the scripted
//! pointer on a window placed off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/measure_area_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const MODE: &str = "ribbon.mode.review"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.measure"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.measure.area"; // ui-text-exempt: a trace region name, never displayed
/// Prefix of the Measure tab's groups, for finding a collapsed one.
const GROUPS: &str = "ribbon.group.measure."; // ui-text-exempt: a trace region name, never displayed
const VIEW_TAB: &str = "ribbon.tab.view"; // ui-text-exempt: a trace region name, never displayed
const VIEW_GROUPS: &str = "ribbon.group.view."; // ui-text-exempt: a trace region name, never displayed
const SELECT: &str = "ribbon.item.view.tool_select"; // ui-text-exempt: a trace region name, never displayed
const TO_AREA: &str = "menu.item.canvas.dimension.format.dimension_area"; // ui-text-exempt: a trace region name, never displayed
const TO_PERIMETER: &str = "menu.item.canvas.dimension.format.dimension_perimeter"; // ui-text-exempt: a trace region name, never displayed
const MENU_ROWS: &str = "menu.item.canvas.dimension."; // ui-text-exempt: a trace region name, never displayed
const VERTEX_EVENT: &str = "measure-perimeter-vertex"; // ui-text-exempt: a trace event name, never displayed
const READOUT_EVENT: &str = "measure-area-readout"; // ui-text-exempt: a trace event name, never displayed
const FINISH_EVENT: &str = "measure-finish"; // ui-text-exempt: a trace event name, never displayed
const COMMIT_EVENT: &str = "add-dimension"; // ui-text-exempt: a trace event name, never displayed
const SWITCH_EVENT: &str = "dimension-area"; // ui-text-exempt: a trace event name, never displayed
const APPLIED_EVENT: &str = "dimension-area-applied"; // ui-text-exempt: a trace event name, never displayed
const EDIT_EVENT: &str = "set-dimension-area"; // ui-text-exempt: a trace event name, never displayed
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The square's centre as fractions of the page box (y up), and its half side
/// as a fraction of the page width: the blank right half of `blank-overhang.pdf`.
const CENTRE: (f64, f64) = (0.70, 0.35);
const HALF: f64 = 0.08;
/// The corners, in half sides about the centre, counter-clockwise.
const CORNERS: [(f64, f64); 4] = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
/// Where the label is placed, in half sides: inside the square.
const PLACE_AT: (f64, f64) = (0.0, 0.3);
/// Where the outline is clicked to select it: the bottom edge's midpoint.
const EDGE: (f64, f64) = (0.0, -1.0);
/// How far the traced area may sit from the square's, as a fraction.
const AREA_TOLERANCE: f64 = 0.02;

/// A failure sentence, or the value a step read.
type Step<T> = std::result::Result<T, String>;

/// See the module documentation.
pub struct AnAreaMeasuresTheRegionItEncloses;

impl Check for AnAreaMeasuresTheRegionItEncloses {
    fn name(&self) -> &'static str {
        "an_area_measures_the_region_it_encloses"
    }

    fn defect(&self) -> &'static str {
        "the Measure tab has no Area tool beside Perimeter, the area it encloses is not shown \
         while the outline is traced, or a closed perimeter ce dimension cannot be switched \
         between its perimeter and its area"
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

/// The launched window and the pointer that drives it.
struct Rig {
    session: Session,
    pointer: ScriptedPointer,
    ui_rect: &'static str,
}

impl Rig {
    fn click(&self, region: &str) -> Result<()> {
        let trace = self.session.trace()?;
        let rect = declared(&trace, self.ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, self.ui_rect, prefix))
            ))
        })?;
        self.pointer
            .click(&self.session, WindowPoint::centre_of(rect))?;
        self.session.settle(15);
        Ok(())
    }

    /// Click `item`, opening each collapsed group under `groups` until it shows.
    fn click_item(&self, item: &str, groups: &str) -> Result<()> {
        if declared(&self.session.trace()?, self.ui_rect, item).is_none() {
            let collapsed: Vec<String> =
                declared_names(&self.session.trace()?, self.ui_rect, groups)
                    .into_iter()
                    .filter(|n| n.ends_with(".collapsed"))
                    .collect();
            for group in collapsed {
                self.click(&group)?;
                if declared(&self.session.trace()?, self.ui_rect, item).is_some() {
                    break;
                }
            }
        }
        self.click(item)
    }
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Rig, PageGeometry)> {
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
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join("blank-overhang.pdf");
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new(format!("cannot read a page size from {}.", pdf.display())))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("measure-area-scripted.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("measure-area-scripted.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    Ok((
        Rig {
            session,
            pointer,
            ui_rect,
        },
        page,
    ))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (rig, page) = launch(ctx, report)?;
    rig.click(MODE)?;
    rig.click(TAB)?;
    rig.click_item(ITEM, GROUPS)?;

    let mapping = CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?;
    let h = HALF * page.width_pt;
    let centre = (CENTRE.0 * page.width_pt, CENTRE.1 * page.height_pt);
    let at = |(u, v): (f64, f64)| -> Result<WindowPoint> {
        mapping.doc_to_window(DocPoint::new(0, centre.0 + u * h, centre.1 + v * h))
    };

    let area_text = match trace_square(ctx, &rig, report, &at, 4.0 * h * h)? {
        Ok(text) => text,
        Err(why) => return Ok(Some(why)),
    };

    // Select it, then switch to the perimeter and back.
    rig.click(VIEW_TAB)?;
    rig.click_item(SELECT, VIEW_GROUPS)?;
    let edge = at(EDGE)?;
    rig.pointer.click(&rig.session, edge)?;
    rig.session.settle(18);
    let perimeter_text = match switch(&rig, edge, TO_PERIMETER, TO_AREA, false)? {
        Ok(text) => text,
        Err(why) => return Ok(Some(why)),
    };
    if perimeter_text == area_text {
        return Ok(Some(format!(
            "★★★ Show perimeter left the label reading `{perimeter_text}`, the area."
        )));
    }
    report.note(format!(
        "★★★ Show perimeter relabelled it `{perimeter_text}`"
    ));
    let back = match switch(&rig, edge, TO_AREA, TO_PERIMETER, true)? {
        Ok(text) => text,
        Err(why) => return Ok(Some(format!("after Show perimeter: {why}"))),
    };
    if back != area_text {
        return Ok(Some(format!(
            "★★★ Show area labelled it `{back}`, and the area shown while tracing was \
             `{area_text}`."
        )));
    }
    report.note(format!("★★★ Show area restored `{back}`"));
    Ok(None)
}

/// Four corners, the running area, the readout over the first corner, the
/// closing click and the placing click. Returns the readout's text.
fn trace_square(
    ctx: &CheckContext,
    rig: &Rig,
    report: &mut CheckReport,
    at: &dyn Fn((f64, f64)) -> Result<WindowPoint>,
    expected_pt2: f64,
) -> Result<Step<String>> {
    for corner in CORNERS {
        rig.pointer.click(&rig.session, at(corner)?)?;
        rig.session.settle(10);
    }
    let trace = rig.session.trace()?;
    let vertex = trace.last(VERTEX_EVENT);
    let traced = vertex
        .and_then(|l| l.get("area_pt2"))
        .and_then(|v| v.parse::<f64>().ok());
    match traced {
        Some(a) if (a - expected_pt2).abs() <= AREA_TOLERANCE * expected_pt2 => {
            report.note(format!(
                "★ four corners enclose {a:.1} pt², the square's {expected_pt2:.1}"
            ));
        }
        _ => {
            return Ok(Err(format!(
                "★ four corners of a {expected_pt2:.1} pt² square traced `{}`.",
                vertex.map_or("no vertex line", |l| l.raw.as_str())
            )));
        }
    }
    // Back over the first corner: the readout is the closed outline's area.
    rig.pointer.hover(&rig.session, at(CORNERS[0])?)?;
    rig.session.settle(12);
    let trace = rig.session.trace()?;
    let Some(readout) = trace
        .last(READOUT_EVENT)
        .and_then(|l| l.get("text").map(str::to_owned))
        .filter(|t| !t.is_empty())
    else {
        return Ok(Err(format!(
            "★★ the pointer is over a traced outline and no `{READOUT_EVENT}` line shows the \
             area it encloses. Trace: {}.",
            rig.session.trace_path().display()
        )));
    };
    let shot = ctx.out("measure-area-scripted-readout.png");
    rig.pointer.screenshot(&rig.session, &shot)?;
    report.artifact(shot);
    let commits = trace.events(COMMIT_EVENT).count();
    rig.pointer.click(&rig.session, at(CORNERS[0])?)?;
    rig.session.settle(20);
    let trace = rig.session.trace()?;
    match trace.last(FINISH_EVENT) {
        Some(l) if l.get("via") == Some("close-ring") && l.get("kind") == Some("area") => {}
        other => {
            return Ok(Err(format!(
                "★★ clicking the first corner again traced `{}`, not a closed area.",
                other.map_or("no finish line", |l| l.raw.as_str())
            )));
        }
    }
    rig.pointer.click(&rig.session, at(PLACE_AT)?)?;
    rig.session.settle(25);
    if rig.session.trace()?.events(COMMIT_EVENT).count() <= commits {
        return Ok(Err(format!(
            "★★ the placing click committed no dimension. Trace: {}.",
            rig.session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ readout `{readout}` and a committed area dimension"
    ));
    Ok(Ok(readout))
}

/// Right-click `at`, require `offered` and not `withheld`, click it, and return
/// the label the engine reports after the switch.
fn switch(
    rig: &Rig,
    at: WindowPoint,
    offered: &str,
    withheld: &str,
    want_area: bool,
) -> Result<Step<String>> {
    rig.pointer.right_click(&rig.session, at)?;
    rig.session.settle(20);
    let trace = rig.session.trace()?;
    let rows = || list(&declared_names(&trace, rig.ui_rect, MENU_ROWS));
    if declared(&trace, rig.ui_rect, withheld).is_some() {
        return Ok(Err(format!(
            "the menu offers `{withheld}`, the measure the label already shows. Rows: {}.",
            rows()
        )));
    }
    let Some(row) = declared(&trace, rig.ui_rect, offered) else {
        return Ok(Err(format!(
            "the right-click menu offers no `{offered}`. Rows: {}.",
            rows()
        )));
    };
    let mark = trace.mark();
    rig.pointer
        .click(&rig.session, WindowPoint::centre_of(row))?;
    rig.session.settle(30);
    let trace = rig.session.trace()?;
    let want = if want_area { "1" } else { "0" };
    let raised = trace.last_after(SWITCH_EVENT, mark);
    if raised.and_then(|l| l.get("area")) != Some(want) {
        return Ok(Err(format!(
            "`{offered}` was clicked and raised `{}`, not area={want}.",
            raised.map_or("nothing", |l| l.raw.as_str())
        )));
    }
    if trace.last_after(EDIT_EVENT, mark).is_none() {
        return Ok(Err(format!(
            "`{offered}` raised the switch and the engine committed no `{EDIT_EVENT}`."
        )));
    }
    match trace.last_after(APPLIED_EVENT, mark) {
        Some(l) if l.get("area") == Some(want) => Ok(Ok(l.get("text").unwrap_or("").to_owned())),
        other => Ok(Err(format!(
            "`{offered}` committed and the label reports `{}`.",
            other.map_or("nothing", |l| l.raw.as_str())
        ))),
    }
}
