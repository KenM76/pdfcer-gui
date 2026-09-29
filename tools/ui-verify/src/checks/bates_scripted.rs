//! `bates_numbering_without_the_mouse` — Pages ▸ Stamp ▸ Bates numbering… on
//! `fixtures/four-pages.pdf`, with nothing picked in the rail, stamps all four
//! pages `000001` to `000004` as one edit, and the window reopened afterwards
//! continues the run at 5. The window is placed off the desktop and driven
//! through the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/bates_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The engine call's outcome line.
const APPLIED: &str = "bates-applied"; // ui-text-exempt: a trace event name, never displayed
/// The edit funnel's line for the same act.
const STAMPED: &str = "bates-stamped"; // ui-text-exempt: a trace event name, never displayed
const OPENED: &str = "bates-opened"; // ui-text-exempt: a trace event name, never displayed
/// Pages is not a Read-mode tab.
const MODE: &str = "ribbon.mode.review"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.pages"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.pages.bates"; // ui-text-exempt: a trace region name, never displayed
/// The Stamp group when the band is too narrow to show it open.
const COLLAPSED: &str = "ribbon.group.pages.stamp.collapsed"; // ui-text-exempt: a trace region name, never displayed
const STAMP: &str = "bates.stamp"; // ui-text-exempt: a trace region name, never displayed
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

pub struct BatesNumberingWithoutTheMouse;

impl Check for BatesNumberingWithoutTheMouse {
    fn name(&self) -> &'static str {
        "bates_numbering_without_the_mouse"
    }

    fn defect(&self) -> &'static str {
        "Pages ▸ Stamp ▸ Bates numbering… does not number every page of the document, or the next window does not continue the run"
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
        .join("four-pages.pdf");
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the four-page fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("bates-scripted.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("bates-scripted.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    let fresh = |region: &str| -> Result<Option<crate::geom::LRect>> {
        Ok(declared(&session.trace()?, ui_rect, region))
    };
    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(&session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };
    let open_window = || -> Result<()> {
        if fresh(ITEM)?.is_none() && fresh(COLLAPSED)?.is_some() {
            click(COLLAPSED)?;
        }
        click(ITEM)
    };

    click(MODE)?;
    click(TAB)?;
    open_window()?;
    click(STAMP)?;
    session.settle(20);
    // Reopen: the run must continue where the stamp left it.
    click(TAB)?;
    open_window()?;
    session.settle(10);
    pointer.gone(&session)?;

    let trace = session.trace()?;
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "Stamp was clicked and no `{APPLIED}` line followed. Funnel: {}. Opened: {}. Trace: {}.",
            trace
                .last("bates-stamped-refused")
                .or_else(|| trace.last(STAMPED))
                .map_or("none".to_owned(), |l| l.raw.clone()),
            trace
                .last(OPENED)
                .map_or("none".to_owned(), |l| l.raw.clone()),
            session.trace_path().display()
        )));
    };
    // Nothing picked means the whole document, not the page on screen.
    if applied.get("n") != Some("4")
        || applied.get("first_label") != Some("000001")
        || applied.get("last_label") != Some("000004")
        || applied.get("next") != Some("5")
    {
        return Ok(Some(format!(
            "four pages with nothing picked should be numbered 000001 to 000004 with 5 next; the stamp traced `{}`.",
            applied.raw
        )));
    }
    let Some(funnel) = trace.last(STAMPED) else {
        return Ok(Some(format!(
            "the engine traced `{}` and the edit funnel wrote no `{STAMPED}` line.",
            applied.raw
        )));
    };
    let reopened: Vec<_> = trace.events(OPENED).collect();
    let Some(second) = reopened.get(1) else {
        return Ok(Some(format!(
            "the window was opened twice and traced `{OPENED}` {} time(s).",
            reopened.len()
        )));
    };
    if second.get("start") != Some("5") {
        return Ok(Some(format!(
            "the window reopened after stamping 1 to 4 should start at 5; it traced `{}`.",
            second.raw
        )));
    }
    report.note(format!("`{}`", applied.raw));
    report.note(format!("`{}`", funnel.raw));
    report.note(format!("`{}`", second.raw));
    Ok(None)
}
