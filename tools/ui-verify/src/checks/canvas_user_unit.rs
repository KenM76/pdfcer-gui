//! `actual_size_shows_a_user_unit_page_physically` — at Actual size a page with
//! `/UserUnit` (ISO 32000-1 §14.11.2) is drawn at its PHYSICAL size and the
//! zoom reads 100 %.
//!
//! Opens `fixtures/user-unit.pdf` — a 306 x 396 box with `/UserUnit 2`,
//! physically Letter — off the desktop with `view.page_single,view.zoom_actual`,
//! and reads the `page` region and the `status` line. One window point is one
//! physical point at 100 %, so the sheet is 612 x 792; a 306 x 396 sheet is the
//! box read as points. Ignores `--pdf`; the fixture is the method.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, repo_fixture};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,1000";
const FIXTURE: &str = "user-unit.pdf";
const METHOD: &str = "A page whose /UserUnit makes its box half its physical size; any other file \
                      cannot tell the two measures apart. Rebuild it from its PROVENANCE file.";
const INVOKE: &str = "view.page_single,view.zoom_actual";
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name
const STATUS: &str = "status"; // ui-text-exempt: a trace event name

const PHYSICAL: (f32, f32) = (612.0, 792.0);
const BOX: (f32, f32) = (306.0, 396.0);
/// The region is traced to a tenth of a point; this absorbs pixel snapping.
const TOLERANCE_PT: f32 = 2.0;

pub struct ActualSizeShowsAUserUnitPagePhysically;

impl Check for ActualSizeShowsAUserUnitPagePhysically {
    fn name(&self) -> &'static str {
        "actual_size_shows_a_user_unit_page_physically"
    }

    fn defect(&self) -> &'static str {
        "a page with /UserUnit is drawn at its box size in points, so Actual size shows it at a fraction of its real size"
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<Session> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out(FIXTURE);
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("canvas_user_unit.trace.txt"));
    spec.pdf = Some(doc);
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
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    Ok(session)
}

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() <= TOLERANCE_PT && (a.1 - b.1).abs() <= TOLERANCE_PT
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no region event."))?;
    let session = launch(ctx, report)?;
    let trace = session.trace()?;
    let zoom = trace
        .last(STATUS)
        .and_then(|l| l.get("zoom").map(str::to_owned))
        .ok_or_else(|| Error::new(format!("no `{STATUS}` line carrying `zoom=`.")))?;
    let page = declared(&trace, ui_rect, PAGE_REGION)
        .ok_or_else(|| Error::new(format!("no `{PAGE_REGION}` region, so no sheet was drawn.")))?;
    let size = (page.width(), page.height());
    report.note(format!("page={:.1}x{:.1} zoom={zoom}", size.0, size.1));
    let percent: f64 = zoom
        .parse()
        .map_err(|_| Error::new(format!("`zoom={zoom}` is not a number.")))?;
    if near(size, BOX) {
        return Ok(Some(format!(
            "at Actual size the sheet is {:.1}x{:.1}: its box read as points, ignoring \
             `/UserUnit 2`, so it shows at half its size on each side.",
            size.0, size.1
        )));
    }
    if !near(size, PHYSICAL) {
        return Ok(Some(format!(
            "at Actual size the sheet is {:.1}x{:.1}, neither its physical {}x{} nor its box {}x{}.",
            size.0, size.1, PHYSICAL.0, PHYSICAL.1, BOX.0, BOX.1
        )));
    }
    if (percent - 100.0).abs() > 0.5 {
        return Ok(Some(format!(
            "the sheet is at its physical size and the zoom reads {zoom} %, not 100 %."
        )));
    }
    Ok(None)
}
