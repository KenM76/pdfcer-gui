//! `a_small_first_page_is_the_current_page` — in continuous mode a small page
//! seen whole is the page commands act on, though a larger page below it
//! covers more of the view. See
//! `docs/modules/ui-verify/checks/small_first_page.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

const OFFSCREEN: &str = "-4200,-4200,1400,900";

/// A 200 x 280 pt page above a Letter page.
const FIXTURE: &str = "small-page-first.pdf";

/// `canvas … page=<0-based> … display=… visible=…`, the per-frame canvas line.
const CANVAS_EVENT: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed

pub struct ASmallFirstPageIsTheCurrentPage;

impl Check for ASmallFirstPageIsTheCurrentPage {
    fn name(&self) -> &'static str {
        "a_small_first_page_is_the_current_page"
    }

    fn defect(&self) -> &'static str {
        "a small first page shown whole is not the current page because a larger page below it \
         covers more of the view, so page commands act on the wrong page"
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

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let pdf = driving::repo_fixture(
        FIXTURE,
        "Run `python fixtures/small-page-first.PROVENANCE.py`.",
    )?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("small_first_page.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("small_first_page.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    session.settle(30);
    let shot = ctx.out("small_first_page.png");
    pointer.screenshot(&session, &shot)?;
    report.artifact(shot);

    let trace = session.trace()?;
    let line = trace
        .last(CANVAS_EVENT)
        .ok_or_else(|| Error::new(format!("the canvas traced no `{CANVAS_EVENT}` line.")))?;
    let display = line.get("display").unwrap_or("?");
    let visible = line.get_usize("visible").unwrap_or(0);
    let page = line
        .get_usize("page")
        .ok_or_else(|| Error::new("the canvas line carries no `page=`."))?;
    report.note(format!(
        "display {display}, {visible} pages visible, current page index {page}"
    ));
    if display != "continuous" {
        return Err(Error::new(format!(
            "the document opened in `{display}`, not continuous, so the current page is set by \
             navigation and this check measures nothing."
        )));
    }
    if visible < 2 {
        return Err(Error::new(
            "only one page is visible, so there is no larger page to outvote the small one.",
        ));
    }
    if page != 0 {
        return Ok(Some(format!(
            "★ the small first page is shown whole and the current page is index {page}: the \
             larger page below it won on raw area."
        )));
    }
    Ok(None)
}
