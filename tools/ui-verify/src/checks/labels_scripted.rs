//! `page_labels_without_the_mouse` — Pages ▸ Stamp ▸ Number pages… on
//! `fixtures/four-pages.pdf`: the window opens on all four pages with no
//! labels stored, style i, ii, iii applied relabels pages 1–4 as one edit, the
//! reopened window reads one stored range back, Remove all labels clears it,
//! and the window opened after that reads none. Driven off the desktop through
//! the scripted pointer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/labels_scripted.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_in, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OPENED: &str = "labels-opened"; // ui-text-exempt: a trace event name, never displayed
const COMMIT: &str = "labels-commit"; // ui-text-exempt: a trace event name, never displayed
/// The edit funnel's success lines.
const SET: &str = "page-labels-set"; // ui-text-exempt: a trace event name, never displayed
const CLEARED: &str = "page-labels-cleared"; // ui-text-exempt: a trace event name, never displayed
const MODE: &str = "ribbon.mode.review"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.pages"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.pages.labels"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.pages.stamp.collapsed"; // ui-text-exempt: a trace region name, never displayed
/// `STYLES[1]`, lower-case roman.
const ROMAN: &str = "labels.style.1"; // ui-text-exempt: a trace region name, never displayed
const APPLY: &str = "labels.apply"; // ui-text-exempt: a trace region name, never displayed
const CLEAR: &str = "labels.clear"; // ui-text-exempt: a trace region name, never displayed
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

pub struct PageLabelsWithoutTheMouse;

impl Check for PageLabelsWithoutTheMouse {
    fn name(&self) -> &'static str {
        "page_labels_without_the_mouse"
    }

    fn defect(&self) -> &'static str {
        "Pages ▸ Stamp ▸ Number pages… does not relabel the pages, does not read the labels back, or cannot remove them"
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("labels-scripted.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("labels-scripted.pointer.txt"))?;

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
        click(TAB)?;
        if fresh(ITEM)?.is_none() && fresh(COLLAPSED)?.is_some() {
            click(COLLAPSED)?;
        }
        click(ITEM)?;
        session.settle(10);
        Ok(())
    };

    click(MODE)?;
    open_window()?;
    click(ROMAN)?;
    click(APPLY)?;
    session.settle(20);
    open_window()?;
    // Drawn only when the window reads a stored range, so its absence is the
    // defect itself, not a harness failure.
    if fresh(CLEAR)?.is_none() {
        pointer.gone(&session)?;
        let trace = session.trace()?;
        return Ok(Some(format!(
            "after relabelling, the reopened window offers no Remove all labels, so it read no stored range back (the label cache did not refresh with the edit). Last open: `{}`.",
            trace
                .last(OPENED)
                .map_or("none".to_owned(), |l| l.raw.clone())
        )));
    }
    click(CLEAR)?;
    session.settle(20);
    open_window()?;
    pointer.gone(&session)?;

    let trace = session.trace()?;
    let opened: Vec<_> = trace.events(OPENED).collect();
    let [before, after_set, after_clear] = opened.as_slice() else {
        return Ok(Some(format!(
            "the window was opened three times and traced `{OPENED}` {} time(s). Trace: {}.",
            opened.len(),
            session.trace_path().display()
        )));
    };
    if before.get("ranges") != Some("0")
        || before.get("first") != Some("0")
        || before.get("last") != Some("3")
    {
        return Ok(Some(format!(
            "an unlabelled four-page document should open on pages 1–4 with no ranges; it traced `{}`.",
            before.raw
        )));
    }
    let Some(commit) = trace.last(COMMIT) else {
        return Ok(Some(format!(
            "Apply was clicked and no `{COMMIT}` line followed. Opened: `{}`.",
            before.raw
        )));
    };
    if commit.get("style") != Some("LowerRoman") || commit.get("last") != Some("3") {
        return Ok(Some(format!(
            "the lower-roman choice over pages 1–4 should commit as such; it traced `{}`.",
            commit.raw
        )));
    }
    let Some(set) = trace.last(SET) else {
        return Ok(Some(format!(
            "the dialog committed `{}` and the edit funnel wrote no `{SET}` line.",
            commit.raw
        )));
    };
    if after_set.get("ranges") != Some("1") {
        return Ok(Some(format!(
            "after relabelling, the reopened window should read one stored range; it traced `{}` (the label cache did not refresh with the edit).",
            after_set.raw
        )));
    }
    let Some(cleared) = trace.last(CLEARED) else {
        return Ok(Some(format!(
            "Remove all labels was clicked and the edit funnel wrote no `{CLEARED}` line."
        )));
    };
    if after_clear.get("ranges") != Some("0") {
        return Ok(Some(format!(
            "after Remove all labels, the window should read no ranges; it traced `{}`.",
            after_clear.raw
        )));
    }
    for line in [before, commit, set, after_set, cleared, after_clear] {
        report.note(format!("`{}`", line.raw));
    }
    Ok(None)
}
