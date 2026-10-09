//! `remove_metadata_writes_a_copy_without_the_items_ticked` — Security ▸
//! Remove metadata lists what the file carries besides its pages and writes a
//! copy without exactly the items ticked, leaving the open document as it
//! was. Driven with the scripted pointer in a window placed off the desktop,
//! on a copy of `fixtures/four-pages.pdf`; the written copy's bytes are the
//! oracle.
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
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";
const TAB: &str = "ribbon.tab.security"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.remove_metadata"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.security.protect.collapsed"; // ui-text-exempt: a trace region name, never displayed
const AUTHOR: &str = "remove-metadata.item.info/Author"; // ui-text-exempt: a trace region name, never displayed
const COMMENT: &str = "remove-metadata.item.comment/10-0"; // ui-text-exempt: a trace region name, never displayed
const COMMIT: &str = "remove-metadata.commit"; // ui-text-exempt: a trace region name, never displayed
const LISTED: &str = "remove-metadata-listed"; // ui-text-exempt: a trace event name, never displayed
const WROTE: &str = "remove-metadata-wrote"; // ui-text-exempt: a trace event name, never displayed
/// Ids the fixture must list: the two ticked and a neighbour of each.
const MUST_LIST: [&str; 4] = ["info/Title", "info/Author", "comment/10-0", "comment/11-0"];
/// Bytes the copy must not hold: the ticked entry and the ticked comment.
const GONE: [&str; 2] = [
    "/Author (OpenAEC Foundation)",
    "/Contents (Construction drawing)",
];
/// Bytes the copy must still hold: their unticked neighbours. Removing
/// nothing fails [`GONE`]; removing everything fails this.
const KEPT: [&str; 2] = [
    "/Title (OpenAEC Foundation drawing frame grootformaat A1 liggend)",
    "/Contents (OA-2026-001-A100)",
];

/// See the module documentation.
pub struct RemoveMetadataWritesACopyWithoutTheItemsTicked;

impl Check for RemoveMetadataWritesACopyWithoutTheItemsTicked {
    fn name(&self) -> &'static str {
        "remove_metadata_writes_a_copy_without_the_items_ticked"
    }

    fn defect(&self) -> &'static str {
        "Remove metadata does not list what the file carries, writes a copy that keeps a ticked \
         item or loses an unticked one, or changes the open document"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let copy = ctx.out("remove-metadata-copy.pdf");
        let _ = std::fs::remove_file(&copy);
        let outcome = launch(ctx, &mut report, &copy).and_then(|(session, pointer)| {
            let outcome = drive(&mut report, &session, &pointer, &copy);
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

/// Open the window, tick Author and one comment, write the copy, read it;
/// then open the window again and see the open document still lists both.
fn drive(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    copy: &std::path::Path,
) -> Result<Option<String>> {
    open_window(session, pointer)?;
    let Some(listed) = await_line(session, LISTED)? else {
        return Ok(Some(format!(
            "Remove metadata opened no window listing the items: no `{LISTED}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("opened: `{}`", listed.raw));
    if let Some(missing) = missing_ids(listed.get("ids")) {
        return Ok(Some(format!(
            "the window did not list `{missing}`: `{}`.",
            listed.raw
        )));
    }
    click(session, pointer, AUTHOR)?;
    click(session, pointer, COMMENT)?;
    click(session, pointer, COMMIT)?;
    let Some(wrote) = await_line(session, WROTE)? else {
        return Ok(Some(format!(
            "Remove and save a copy wrote nothing: no `{WROTE}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("wrote: `{}`", wrote.raw));
    let bytes = std::fs::read(copy)
        .map_err(|e| Error::new(format!("reading the copy {}: {e}", copy.display())))?;
    if let Some(wrong) = judge_copy(&bytes) {
        return Ok(Some(wrong));
    }
    let before = session.trace()?.events(LISTED).count();
    open_window(session, pointer)?;
    session.settle(20);
    let trace = session.trace()?;
    let again = trace.events(LISTED).skip(before).last();
    report.note(format!("reopened: {:?}", again.map(|l| l.raw.clone())));
    if let Some(missing) = missing_ids(again.and_then(|l| l.get("ids"))) {
        return Ok(Some(format!(
            "after the copy was written the open document no longer lists `{missing}`: the \
             removal changed it."
        )));
    }
    Ok(None)
}

/// The first of [`MUST_LIST`] not in `ids`.
fn missing_ids(ids: Option<&str>) -> Option<&'static str> {
    let listed: Vec<&str> = ids.unwrap_or_default().split(',').collect();
    MUST_LIST.into_iter().find(|id| !listed.contains(id))
}

/// The copy against [`GONE`], [`KEPT`] and one version.
fn judge_copy(bytes: &[u8]) -> Option<String> {
    let holds = |needle: &str| bytes.windows(needle.len()).any(|w| w == needle.as_bytes());
    if let Some(kept) = GONE.into_iter().find(|s| holds(s)) {
        return Some(format!("the copy still holds `{kept}`, which was ticked."));
    }
    if let Some(lost) = KEPT.into_iter().find(|s| !holds(s)) {
        return Some(format!("the copy lost `{lost}`, which was not ticked."));
    }
    let versions = bytes.windows(5).filter(|w| w == b"%%EOF").count();
    (versions != 1).then(|| {
        format!("the copy holds {versions} versions; one is needed for the removal to be real.")
    })
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

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    copy: &std::path::Path,
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
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), copy.display().to_string()));
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
