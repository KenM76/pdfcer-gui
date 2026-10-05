//! `recognised_text_is_searchable_before_saving` and
//! `stopped_recognition_is_searchable_before_saving` — with Document
//! properties showing, File ▸ Recognise text… on an image-only page, run to the
//! end or stopped after two pages, puts words into the OPEN document: Find in
//! the same session, with nothing saved, finds them where it found none before.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_live_search.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

const MODE: &str = "ribbon.mode.read"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.ocr"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const RUN: &str = "ocr-run"; // ui-text-exempt: a trace region name, never displayed
const STOP: &str = "ocr-stop"; // ui-text-exempt: a trace region name, never displayed
/// Opened at launch, so its Security notes cache exists before the run.
const INVOKE: &str = "file.document_properties"; // ui-text-exempt: a command id, never displayed
const NOTES: &str = "security-notes"; // ui-text-exempt: a trace event name, never displayed
const PROGRESS: &str = "ocr-progress"; // ui-text-exempt: a trace event name, never displayed
const RECOGNISED: &str = "ocr-recognised"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "ocr-refused"; // ui-text-exempt: a trace event name, never displayed
const LAYER: &str = "ocr-layer"; // ui-text-exempt: a trace event name, never displayed
const LAYER_REFUSED: &str = "ocr-layer-refused"; // ui-text-exempt: a trace event name, never displayed
const FIND: &str = "find"; // ui-text-exempt: a trace event name, never displayed
const FIND_REFUSED: &str = "find-refused"; // ui-text-exempt: a trace event name, never displayed
/// A letter every line of the fixtures' notes holds.
const NEEDLE: &str = "e"; // ui-text-exempt: typed into Find, never displayed
const ENGINE: &str = "ocrs"; // ui-text-exempt: a preference value, never displayed
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// Settle frames a whole run may take, polled in steps of [`SLICE`].
const RUN_FRAMES: u32 = 1_600;
const SLICE: u32 = 20;
/// Pages the stopped run must have finished before Stop is pressed.
const STOP_AFTER: usize = 2;

/// How the run ends.
#[derive(Clone, Copy)]
pub enum Ending {
    /// The one-page fixture, run to the end.
    Finished,
    /// The eight-page fixture, stopped once [`STOP_AFTER`] pages are done.
    Stopped,
}

/// See the module documentation.
pub struct RecognisedTextIsSearchableBeforeSaving {
    pub ending: Ending,
}

impl Check for RecognisedTextIsSearchableBeforeSaving {
    fn name(&self) -> &'static str {
        match self.ending {
            Ending::Finished => "recognised_text_is_searchable_before_saving",
            Ending::Stopped => "stopped_recognition_is_searchable_before_saving",
        }
    }

    fn defect(&self) -> &'static str {
        "Recognise text… finishes, or is stopped, and the open document holds no new text: the \
         layer never reaches the session, so Find, selection and search see nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report, self.ending).and_then(|(session, pointer)| {
            let outcome = steps(ctx, &mut report, &session, &pointer, self.ending);
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
    ending: Ending,
) -> Result<(Session, ScriptedPointer)> {
    let (fixture, method) = match ending {
        Ending::Finished => ("synthetic-image-only.pdf", "It is committed; restore it."),
        Ending::Stopped => (
            "synthetic-image-only-8pages.pdf",
            "Generate it: cargo test -p pdfcer-gui --lib write_synthetic_image_only_multipage -- \
             --ignored",
        ),
    };
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let userdata = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    crate::sandbox::write_prefs(
        &userdata,
        &format!("ocr_engine = {ENGINE}\nocr_model = {ENGINE}\n"),
    )
    .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;
    let name = match ending {
        Ending::Finished => "ocr-live-search-finished",
        Ending::Stopped => "ocr-live-search-stopped",
    };
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{name}.trace.txt")));
    spec.pdf = Some(repo_fixture(fixture, method)?);
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out(&format!("{name}.pointer.txt")))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(45);
    Ok((session, pointer))
}

/// Poll until `pred` holds or [`RUN_FRAMES`] are spent.
fn wait_until(session: &Session, mut pred: impl FnMut(&Trace) -> bool) -> Result<bool> {
    let mut spent = 0;
    while spent < RUN_FRAMES {
        if pred(&session.trace()?) {
            return Ok(true);
        }
        session.settle(SLICE);
        spent += SLICE;
    }
    Ok(pred(&session.trace()?))
}

/// Find [`NEEDLE`] in the open document; the new `find` line's hit count, or
/// the failure that stands for it.
fn find(
    session: &Session,
    pointer: &ScriptedPointer,
    when: &str,
) -> Result<std::result::Result<usize, String>> {
    let before = session.trace()?.events(FIND).count();
    let refused = session.trace()?.events(FIND_REFUSED).count();
    pointer.key(session, None, "F", Some("ctrl"))?;
    session.settle(10);
    pointer.key(session, None, "A", Some("ctrl"))?;
    pointer.type_text(session, None, NEEDLE)?;
    pointer.key(session, None, "Enter", None)?;
    session.settle(20);
    let trace = session.trace()?;
    if let Some(line) = trace.events(FIND_REFUSED).nth(refused) {
        return Ok(Err(format!("Find {when} was refused: `{}`.", line.raw)));
    }
    let hits = trace
        .events(FIND)
        .nth(before)
        .and_then(|l| l.get_usize("hits"));
    pointer.key(session, None, "Escape", None)?;
    session.settle(5);
    Ok(hits.ok_or_else(|| format!("Find {when} traced no `{FIND}` line.")))
}

/// Read mode ▸ File ▸ Recognise text…, with the seeded recogniser, then Run.
fn start(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };
    click(MODE)?;
    click(TAB)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, ITEM).is_none() && declared(&trace, ui_rect, COLLAPSED).is_some() {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    session.settle(15);
    let trace = session.trace()?;
    if declared(&trace, ui_rect, RUN).is_none() {
        return Err(Error::new(format!(
            "the dialog drew no `{RUN}` control: no `{ENGINE}` models are beside the binary. \
             Point --exe at a packaged build."
        )));
    }
    click(RUN)?;
    if matches!(
        session.trace()?.last("ocr-started").and_then(|l| l.get("engine")),
        Some(e) if e != ENGINE
    ) {
        return Err(Error::new(format!(
            "`ocr_engine = {ENGINE}` was seeded and another engine ran."
        )));
    }
    Ok(())
}

/// Press Stop once [`STOP_AFTER`] pages are done.
fn stop(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<()> {
    let done = |t: &Trace| {
        t.events(PROGRESS)
            .filter_map(|l| l.get_usize("attempted"))
            .any(|n| n >= STOP_AFTER)
    };
    wait_until(session, |t| done(t) || t.last(REFUSED).is_some())?;
    let trace = session.trace()?;
    if trace.last(RECOGNISED).is_some() {
        return Err(Error::new(
            "the whole run finished before Stop could be pressed, so nothing was stopped.",
        ));
    }
    if !done(&trace) {
        return Err(Error::new(format!(
            "{STOP_AFTER} pages never finished, so Stop was not pressed."
        )));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    let (rect, viewport) = declared_in(&trace, ui_rect, STOP)
        .ok_or_else(|| Error::new(format!("no `{STOP}` region while the run was going.")))?;
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    Ok(())
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ending: Ending,
) -> Result<Option<String>> {
    if !wait_until(session, |t| t.last(NOTES).is_some())? {
        return Ok(Some(format!(
            "`{INVOKE}` was invoked and no `{NOTES}` line followed: Document properties never \
             drew, so the condition this check holds was never set up."
        )));
    }
    match find(session, pointer, "before recognition")? {
        Ok(0) => {}
        Ok(n) => {
            return Ok(Some(format!(
                "the image-only fixture already has {n} hit(s) for `{NEEDLE}`."
            )));
        }
        Err(failure) => return Ok(Some(failure)),
    }
    start(ctx, session, pointer)?;
    if matches!(ending, Ending::Stopped) {
        stop(ctx, session, pointer)?;
    }
    wait_until(session, |t| {
        t.last(RECOGNISED).is_some() || t.last(REFUSED).is_some()
    })?;
    session.settle(SLICE);
    let trace = session.trace()?;
    let Some(recognised) = trace.last(RECOGNISED) else {
        return Ok(Some(format!(
            "no `{RECOGNISED}` line; refusal {:?}.",
            trace.last(REFUSED).map(|l| l.raw.clone())
        )));
    };
    report.note(format!("run: `{}`", recognised.raw));
    let Some(layer) = trace.last(LAYER) else {
        return Ok(Some(format!(
            "★★★ `{}` and the layer never reached the open document: {:?}.",
            recognised.raw,
            trace.last(LAYER_REFUSED).map(|l| l.raw.clone())
        )));
    };
    report.note(format!("layer: `{}`", layer.raw));
    pointer.key(session, None, "Escape", None)?;
    session.settle(10);
    match find(session, pointer, "after recognition")? {
        Ok(0) => Ok(Some(format!(
            "★★★ the layer was applied (`{}`) and Find in the open document finds no `{NEEDLE}`.",
            layer.raw
        ))),
        Ok(n) => {
            report.note(format!("Find after recognition: {n} hit(s), nothing saved"));
            Ok(None)
        }
        Err(failure) => Ok(Some(format!("★★★ {failure}"))),
    }
}
