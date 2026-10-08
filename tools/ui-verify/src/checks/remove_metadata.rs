//! `remove_metadata_takes_only_the_entries_ticked` — Security ▸ Remove
//! metadata lists the document's description entries and removes exactly the
//! ones ticked. Driven with the scripted pointer in a window placed off the
//! desktop, on a copy of `fixtures/four-pages.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/remove_metadata.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::security_notes::await_line;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "four-pages.pdf";
const METHOD: &str = "It is checked in.";
const UI_RECT: &str = "ui-rect";
const TAB: &str = "ribbon.tab.security"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.remove_metadata"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.security.protect.collapsed"; // ui-text-exempt: a trace region name, never displayed
const AUTHOR: &str = "remove-metadata.field.Author"; // ui-text-exempt: a trace region name, never displayed
const KEYWORDS: &str = "remove-metadata.field.Keywords"; // ui-text-exempt: a trace region name, never displayed
const COMMIT: &str = "remove-metadata.commit"; // ui-text-exempt: a trace region name, never displayed
const LISTED: &str = "remove-metadata-listed"; // ui-text-exempt: a trace event name, never displayed
const REQUESTED: &str = "remove-metadata-requested"; // ui-text-exempt: a trace event name, never displayed
/// The fixture's four entries, in `InfoField::all()` order.
const ALL_FOUR: &str = "Title,Author,Subject,Keywords";
/// What must be left: neither "nothing removed" nor "everything removed"
/// can produce it.
const LEFT: &str = "Title,Subject";

/// See the module documentation.
pub struct RemoveMetadataTakesOnlyTheEntriesTicked;

impl Check for RemoveMetadataTakesOnlyTheEntriesTicked {
    fn name(&self) -> &'static str {
        "remove_metadata_takes_only_the_entries_ticked"
    }

    fn defect(&self) -> &'static str {
        "Remove metadata does not list the document's title, author, subject and keywords, or \
         removes entries other than the ones ticked"
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

/// Open the window, tick Author and Keywords, remove; open it again and see
/// Title and Subject alone listed.
fn drive(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    open_window(session, pointer)?;
    let Some(listed) = await_line(session, LISTED)? else {
        return Ok(Some(format!(
            "Remove metadata opened no window listing the entries: no `{LISTED}` line. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("opened: `{}`", listed.raw));
    if listed.get("fields") != Some(ALL_FOUR) {
        return Ok(Some(format!(
            "the window listed `{}`, not the fixture's four entries `{ALL_FOUR}`.",
            listed.raw
        )));
    }
    click(session, pointer, AUTHOR)?;
    click(session, pointer, KEYWORDS)?;
    click(session, pointer, COMMIT)?;
    session.settle(30);
    let requested = session.trace()?.last(REQUESTED).map(|l| l.raw.clone());
    report.note(format!("requested: {requested:?}"));
    let before = session.trace()?.events(LISTED).count();
    open_window(session, pointer)?;
    let trace = session.trace()?;
    let again = trace.events(LISTED).skip(before).last();
    let left = again
        .and_then(|l| l.get("fields"))
        .unwrap_or_default()
        .to_owned();
    report.note(format!("reopened: {:?}", again.map(|l| l.raw.clone())));
    if left != LEFT {
        return Ok(Some(format!(
            "Author and Keywords were ticked and removed; the window reopened listing `{left}`, \
             not `{LEFT}`."
        )));
    }
    Ok(None)
}

fn open_window(session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    click(session, pointer, TAB)?;
    let trace = session.trace()?;
    if declared(&trace, UI_RECT, ITEM).is_none() && declared(&trace, UI_RECT, COLLAPSED).is_some() {
        click(session, pointer, COLLAPSED)?;
    }
    click(session, pointer, ITEM)?;
    session.settle(20);
    Ok(())
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
    let doc = ctx.out("remove-metadata.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("remove-metadata.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("remove-metadata.pointer.txt"))?;
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
