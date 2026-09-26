//! `the_off_page_census_finds_the_object_and_marks_it` — **the discovery
//! third of the operator's off-page question, driven end to end.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/off_page_census.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Enter Edit mode, then open the census — through the harness seam rather than
/// by hunting the ribbon, because the ribbon route is `reach`'s subject and not
/// this one's.
const INVOKE: &str = "mode.edit,edit.offpage";

/// The window's body region.
const BODY: &str = "offpage.body"; // ui-text-exempt: a trace region name, never displayed

/// The Mark button's region.
const MARK: &str = "offpage.mark"; // ui-text-exempt: a trace region name, never displayed

/// Written once, when the window opens and knows how many sheets it must walk.
const OPENED: &str = "offpage-opened"; // ui-text-exempt: a trace event name, never displayed

/// Written once, on the frame the walk completes.
const SCANNED: &str = "offpage-scanned"; // ui-text-exempt: a trace event name, never displayed

/// Written when the Mark button is pressed and the action is raised.
const REQUESTED: &str = "offpage-mark-requested"; // ui-text-exempt: a trace event name

/// Written by the edit funnel when the marks reach the document. See the module
/// header for why this is matched on `epoch=` rather than on the name.
const APPLIED: &str = "redact-mark-offpage"; // ui-text-exempt: a trace event name

/// The fixture, relative to the workspace root. Shared with every sibling.
const FIXTURE: &str = "fixtures/off-page-object.pdf";

/// See the module documentation.
pub struct TheOffPageCensusFindsTheObjectAndMarksIt;

impl Check for TheOffPageCensusFindsTheObjectAndMarksIt {
    fn name(&self) -> &'static str {
        "the_off_page_census_finds_the_object_and_marks_it"
    }

    fn defect(&self) -> &'static str {
        "an operator cannot find out which sheets carry content beside the page — pdfcer can draw \
         it, band it and zoom it, but only for somebody who already knows it is there, and on a \
         drawing set nobody does"
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

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check's subject is a press on the Mark \
             button. The window opening proves only that the command dispatches; a document with \
             nothing off the sheet draws the same window with the button greyed, which is \
             correct. Reported as SKIPPED rather than passed.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // The fixture is NOT `--pdf`. This check's oracle is a count of objects
    // beside the sheet, and a count is only an oracle against a file whose
    // answer is known — an arbitrary `--pdf` would make a clean census
    // indistinguishable from a blind one, which is this harness's
    // best-documented way of reporting a defect that is not there.
    let pdf = workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the off-page fixture is not at {}. It is 485 bytes of hand-written PDF syntax — one \
             200 x 200 page, one square on it and one entirely beside it — shared with \
             `off_page_marquee`, `off_page_press`, `off_page_visible` and `off_page_zoom`.",
            pdf.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("off-page-census.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} on {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    session.settle(60);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process. \
             Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }

    let Some(opened) = trace.events(OPENED).last().cloned() else {
        return Ok(Some(format!(
            "★ CHECK FOR CONTENT OFF THE SHEET WAS INVOKED AND NO WINDOW APPEARED: no `{OPENED}` \
             line at all.\n\
             The command has no dispatch arm, or the dispatcher declined because it read the \
             document as closed. Regions beginning `offpage`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "offpage")),
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the window opened: `{}`", opened.raw));

    // --- the walk ------------------------------------------------------------
    //
    // A settle loop rather than one long settle, because the subject is a
    // window that does work AFTER it opens, one page per frame, and the number
    // of frames is a property of the document. A fixed wait long enough for a
    // thirty-six sheet set would be dead time on every run; one short enough
    // for this fixture would report a slow scan as a stalled one.
    let mut scanned = None;
    for _ in 0..30 {
        let trace = session.trace()?;
        if let Some(line) = trace.events(SCANNED).last() {
            scanned = Some(line.clone());
            break;
        }
        session.settle(10);
    }
    let Some(scanned) = scanned else {
        return Ok(Some(format!(
            "★ THE CENSUS OPENED AND NEVER FINISHED: `{}` and no `{SCANNED}` line after 30 \
             settles.\n\
             The window walks one page per frame and emits this line once, on the frame the walk \
             completes. Its absence means the walk stalled — the likeliest cause is the repaint \
             request not being made, which leaves an immediate-mode window having scanned exactly \
             one page and waiting for the operator to jiggle the mouse over it. Trace: {}.",
            opened.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the walk finished: `{}`", scanned.raw));

    let objects: usize = scanned
        .get("objects")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let dirty: usize = scanned
        .get("dirty")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if objects == 0 {
        return Ok(Some(format!(
            "★ THE CENSUS FOUND NOTHING ON A FIXTURE BUILT TO CARRY SOMETHING: `{}`.\n\
             `{FIXTURE}` has one 200 x 200 page with a square drawn entirely outside it, and four \
             sibling checks in this suite assert that square is painted, bandable and zoomable. A \
             census reporting objects=0 against it is not a fact about the input. Read the scan's \
             tolerance and the page box it measured against. Trace: {}.",
            scanned.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ {objects} object(s) beside the sheet on {dirty} page(s)"
    ));

    if declared(&session.trace()?, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "the census finished and drew no `{BODY}` region, so the window did its work and put \
             nothing on screen. Regions beginning `offpage`: {}. Trace: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "offpage")),
            session.trace_path().display()
        )));
    }

    // --- the press -----------------------------------------------------------
    //
    // `stable_rect`, because the list lays out over several frames as the
    // scroll area measures its rows and the button sits below them. A
    // coordinate read while the rows are still arriving is a number rather than
    // an error, which is worse.
    let Some(button) = stable_rect(&session, ui_rect, MARK, 8)? else {
        return Ok(Some(format!(
            "the census window drew its body and declared no `{MARK}` region, so the one control \
             that changes anything was never laid out. Regions beginning `offpage`: {}. Trace: \
             {}.",
            list(&declared_names(&session.trace()?, ui_rect, "offpage")),
            session.trace_path().display()
        )));
    };

    // `frame_of`, never `session.frame()`. This window is a child viewport
    // and its coordinates are its own; asking the main window yields a point on
    // the ribbon, and the click lands somewhere plausible and wrong.
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, MARK)?;
    driver.click_at(frame.declared_center(button))?;
    session.settle(50);

    let trace = session.trace()?;
    let Some(requested) = trace.events(REQUESTED).last().cloned() else {
        return Ok(Some(format!(
            "the Mark button was clicked and the window raised nothing: no `{REQUESTED}` line.\n\
             Either the click missed — the button is at {button:?} in the window's own frame — or \
             it landed on a GREYED button. The window greys it while the walk is running and when \
             the walk found nothing, and this run's walk finished having found {objects}. Trace: \
             {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the press raised: `{}`", requested.raw));

    // --- the document --------------------------------------------------------
    let applied = trace
        .events(APPLIED)
        .filter(|line| line.get("epoch").is_some())
        .last()
        .cloned();
    let Some(applied) = applied else {
        let seen = trace
            .events(APPLIED)
            .last()
            .map_or_else(|| "none".to_owned(), |line| line.raw.clone());
        return Ok(Some(format!(
            "★ THE MARKS WERE REQUESTED AND NOTHING REACHED THE DOCUMENT: `{}`, and no \
             `{APPLIED}` line carrying an `epoch`.\n\
             The action was raised and its apply arm did not run, or every page refused. A \
             refusal traces the same first token with a `detail=` instead of an `epoch=`, and the \
             last line carrying this token was: `{seen}`. Trace: {}.",
            requested.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the marks reached the document: `{}`",
        applied.raw
    ));

    // Reported, never asserted. How many bands a sheet needs is a property of
    // where the content sits — a mark off one edge alone needs one band, one
    // off a corner needs two — and asserting a number here would pin this check
    // to the fixture's geometry rather than to the feature.
    Ok(None)
}
