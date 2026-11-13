//! `view_page_boxes_outlines_trim_and_bleed` — View ▸ Display ▸ Page boxes
//! outlines the trim and bleed boxes the page writes, in the right place.
//!
//! Driven off the desktop with the scripted pointer on `page-box-sizes.pdf`,
//! whose page 1 is a 600×800 sheet with Bleed [10 10 590 790] and Trim
//! [20 20 580 780] and no Art box. The control: `page-boxes` reads on=0 (or is
//! absent) before the switch is pressed. Pressed, page 1 owes `1:bleed+trim` and no
//! art outline, and the two outlines must sit where the file puts them: the
//! trim is 560/580 of the bleed's width and inset by 10/580 of it on each side.
//! Those ratios come from the fixture's numbers, not from the program. Pressed
//! again, the line must turn to on=0.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::region_press::press_region;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "page-box-sizes.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/page-box-sizes.PROVENANCE.py`.";
const UI_RECT: &str = "ui-rect";
const EVENT: &str = "page-boxes"; // ui-text-exempt: a trace event name, never displayed
const VIEW_TAB: &str = "ribbon.tab.view";
const ITEM: &str = "ribbon.item.view.page_boxes";
/// `canvas::pageboxes::REGION_PREFIX` + page 1.
const TRIM: &str = "canvas.page-box.1.trim";
const BLEED: &str = "canvas.page-box.1.bleed";
const ART: &str = "canvas.page-box.1.art";
/// Tolerance on each ratio; 0.01 of a ~400 pt outline is about 4 screen points.
const SLACK: f32 = 0.01;

/// See the module documentation.
pub struct ViewPageBoxesOutlinesTrimAndBleed;

impl Check for ViewPageBoxesOutlinesTrimAndBleed {
    fn name(&self) -> &'static str {
        "view_page_boxes_outlines_trim_and_bleed"
    }

    fn defect(&self) -> &'static str {
        "View ▸ Display ▸ Page boxes is missing, outlines boxes the page does not write, puts \
         them in the wrong place, or does not turn off"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer);
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
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("page_boxes_view.trace.txt"));
    spec.pdf = Some(repo_fixture(FIXTURE, METHOD)?);
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("page_boxes_view.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

fn width(r: LRect) -> f32 {
    r.max.x - r.min.x
}

/// Where the trim sits inside the bleed, against the fixture's numbers.
fn misplaced(trim: LRect, bleed: LRect) -> Option<String> {
    let w = width(bleed);
    if w <= 0.0 {
        return Some(format!("the bleed outline has no width: {bleed:?}"));
    }
    let owed = [
        ("trim width / bleed width", width(trim) / w, 560.0 / 580.0),
        (
            "left inset / bleed width",
            (trim.min.x - bleed.min.x) / w,
            10.0 / 580.0,
        ),
        (
            "right inset / bleed width",
            (bleed.max.x - trim.max.x) / w,
            10.0 / 580.0,
        ),
        (
            "top inset / bleed width",
            (trim.min.y - bleed.min.y) / w,
            10.0 / 580.0,
        ),
    ];
    owed.iter()
        .find(|(_, got, want)| (got - want).abs() > SLACK)
        .map(|(what, got, want)| {
            format!(
                "{what} is {got:.4}, the file puts it at {want:.4} (trim {trim:?}, bleed {bleed:?})"
            )
        })
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(l) = session
        .trace()?
        .last(EVENT)
        .filter(|l| l.get("on") != Some("0"))
    {
        return Err(Error::new(format!(
            "before the switch was pressed `{EVENT}` read `{}`, not on=0, so what follows the \
             press would prove nothing.",
            l.raw
        )));
    }
    press_region(session, pointer, VIEW_TAB, "ribbon.")?;
    let mark = session.trace()?.mark();
    press_region(session, pointer, ITEM, "ribbon.item.")?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(line) = trace
        .last_after(EVENT, mark)
        .filter(|l| l.get("on") == Some("1"))
    else {
        return Ok(Some(format!(
            "`{ITEM}` was pressed and no `{EVENT} on=1` line followed."
        )));
    };
    report.note(line.raw.clone());
    let pages = line.get("pages").unwrap_or("");
    if !pages.split(',').any(|p| p == "1:bleed+trim") {
        return Ok(Some(format!(
            "page 1 writes a bleed and a trim box and no art box; `{EVENT}` traced pages={pages}."
        )));
    }
    if declared_in(&trace, UI_RECT, ART).is_some() {
        return Ok(Some(format!(
            "`{ART}` was drawn on a page with no art box."
        )));
    }
    let (Some((trim, _)), Some((bleed, _))) = (
        declared_in(&trace, UI_RECT, TRIM),
        declared_in(&trace, UI_RECT, BLEED),
    ) else {
        return Ok(Some(format!("`{TRIM}` or `{BLEED}` was not declared.")));
    };
    if let Some(why) = misplaced(trim, bleed) {
        return Ok(Some(why));
    }
    let png = ctx.out("page_boxes_view.on.png");
    pointer.screenshot(session, &png)?;
    report.artifact(png);
    let off = session.trace()?.mark();
    press_region(session, pointer, ITEM, "ribbon.item.")?;
    session.settle(20);
    match session.trace()?.last_after(EVENT, off) {
        Some(l) if l.get("on") == Some("0") => Ok(None),
        Some(l) => Ok(Some(format!(
            "pressed a second time, the outlines kept drawing: `{}`.",
            l.raw
        ))),
        None => Ok(Some(format!(
            "pressed a second time and no `{EVENT} on=0` followed: the outlines did not stop."
        ))),
    }
}
