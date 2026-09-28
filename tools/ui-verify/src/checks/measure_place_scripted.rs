//! `measure_place_without_the_mouse` — a finished radius measurement follows
//! the pointer and is committed by the click that places it, with its value
//! text at that click. Driven through the scripted pointer on a window placed
//! off the desktop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/measure_place_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const MODE: &str = "ribbon.mode.review"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.measure"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.measure.radius_diameter"; // ui-text-exempt: a trace region name, never displayed
/// Prefix of the Measure tab's groups, for finding a collapsed one.
const GROUPS: &str = "ribbon.group.measure."; // ui-text-exempt: a trace region name, never displayed
const FINISH_EVENT: &str = "measure-finish"; // ui-text-exempt: a trace event name, never displayed
const PLACE_EVENT: &str = "measure-place"; // ui-text-exempt: a trace event name, never displayed
const COMMIT_EVENT: &str = "add-dimension"; // ui-text-exempt: a trace event name, never displayed
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The circle, as fractions of the page box (y up) and of its width: the
/// blank right half of `blank-overhang.pdf`.
const CENTRE: (f64, f64) = (0.70, 0.35);
const RADIUS: f64 = 0.10;
/// Rim picks in degrees; the last is the double-click that ends the fit.
const PICKS: [f64; 3] = [90.0, 210.0, 330.0];
const LAST: f64 = 150.0;
/// Where the text is placed, in radii about the centre: outside the rim.
const PLACE_AT: (f64, f64) = (1.6, 0.9);
/// How far the traced value text may sit from the aimed one, in points.
const TOLERANCE_PT: f64 = 1.5;

pub struct MeasurePlaceWithoutTheMouse;

impl Check for MeasurePlaceWithoutTheMouse {
    fn name(&self) -> &'static str {
        "measure_place_without_the_mouse"
    }

    fn defect(&self) -> &'static str {
        "a finished measurement is committed at once instead of following the pointer until a \
         click places it, or the click does not put the value text where it landed"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("measure-place-scripted.trace.txt"));
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
    let pointer =
        ScriptedPointer::attach(&mut spec, ctx.out("measure-place-scripted.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click(&session, WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };

    click(MODE)?;
    click(TAB)?;
    // A narrow band collapses groups into buttons; open each until the item shows.
    if declared(&session.trace()?, ui_rect, ITEM).is_none() {
        let collapsed: Vec<String> = declared_names(&session.trace()?, ui_rect, GROUPS)
            .into_iter()
            .filter(|n| n.ends_with(".collapsed"))
            .collect();
        for group in collapsed {
            click(&group)?;
            if declared(&session.trace()?, ui_rect, ITEM).is_some() {
                break;
            }
        }
    }
    click(ITEM)?;

    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let r = RADIUS * page.width_pt;
    let centre = (CENTRE.0 * page.width_pt, CENTRE.1 * page.height_pt);
    let about = |(u, v): (f64, f64)| (centre.0 + u * r, centre.1 + v * r);
    let rim = |deg: f64| {
        let (s, c) = deg.to_radians().sin_cos();
        about((c, s))
    };
    let window = |(x, y): (f64, f64)| mapping.doc_to_window(DocPoint::new(0, x, y));

    for deg in PICKS {
        pointer.click(&session, window(rim(deg))?)?;
        session.settle(10);
    }
    let commits = session.trace()?.events(COMMIT_EVENT).count();
    pointer.double_click(&session, window(rim(LAST))?)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(finish) = trace.last(FINISH_EVENT) else {
        return Ok(Some(format!(
            "four rim picks and a double-click traced no `{FINISH_EVENT}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if trace.events(COMMIT_EVENT).count() > commits {
        return Ok(Some(format!(
            "★ the fit was committed by `{}` itself. It must follow the pointer until a \
             placing click drops it.",
            finish.raw
        )));
    }
    report.note(format!("★ `{}` committed nothing yet", finish.raw));

    // Hover, photograph the preview, then place.
    let target = about(PLACE_AT);
    let aim = window(target)?;
    pointer.hover(&session, aim)?;
    session.settle(12);
    let shot = ctx.out("measure-place-scripted-preview.png");
    pointer.screenshot(&session, &shot)?;
    report.artifact(shot);
    pointer.click(&session, aim)?;
    session.settle(25);
    pointer.gone(&session)?;
    session.settle(10);
    // The committed dimension, to set beside the preview: they must match.
    let placed = ctx.out("measure-place-scripted-placed.png");
    pointer.screenshot(&session, &placed)?;
    report.artifact(placed);

    let trace = session.trace()?;
    let Some(place) = trace.last(PLACE_EVENT) else {
        return Ok(Some(format!(
            "★★ the placing click traced no `{PLACE_EVENT}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let got = |k: &str| place.get(k).and_then(|v| v.parse::<f64>().ok());
    let (Some(x), Some(y)) = (got("text_x"), got("text_y")) else {
        return Ok(Some(format!("`{}` carries no text_x/text_y.", place.raw)));
    };
    if (x - target.0).hypot(y - target.1) > TOLERANCE_PT {
        return Ok(Some(format!(
            "★★ the text was placed at ({x:.1}, {y:.1}) and the click was aimed at \
             ({:.1}, {:.1}) — the value text is not where the click landed.",
            target.0, target.1
        )));
    }
    if trace.events(COMMIT_EVENT).count() <= commits {
        return Ok(Some(format!(
            "★★★ `{}` was traced and the engine accepted no dimension. Trace: {}.",
            place.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ placed and committed at the click: `{}`",
        place.raw
    ));
    Ok(None)
}
