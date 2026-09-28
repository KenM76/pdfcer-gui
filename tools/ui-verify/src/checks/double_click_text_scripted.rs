//! `double_click_text_without_the_mouse` — the Smart-Selector's last rung,
//! driven entirely through the scripted-pointer seam on a window placed off
//! the desktop. It proves the seam carries every class of pointer gesture a
//! canvas check needs: a ribbon click, a canvas click, click counting.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/double_click_text_scripted.md`.

use crate::checks::driving::{
    MODE_EVENT, SHELL_DIAG_ENV, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Content editing needs it.
const MODE: &str = "edit";
/// The line the caret writes when it opens.
const CARET: &str = "text-edit-caret"; // ui-text-exempt: a trace event name, never displayed
/// The line a canvas click writes.
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
/// Off the desktop, so no OS input can reach it and none of his is taken.
const OFFSCREEN: &str = "-4200,-4200,1400,900";

pub struct DoubleClickTextWithoutTheMouse;

impl Check for DoubleClickTextWithoutTheMouse {
    fn name(&self) -> &'static str {
        "double_click_text_without_the_mouse"
    }

    fn defect(&self) -> &'static str {
        "the scripted-pointer seam does not carry a ribbon click, a canvas click and a \
         double-click through egui's own hit-testing, so no check can run while the \
         operator is using the machine"
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
    let (pdf, target) = crate::fixture::text_point_target();
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the text fixture is not at {}; it is committed, so this is a broken checkout.",
            pdf.display()
        )));
    }
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("double-click-scripted.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("double-click-scripted.pointer.txt"))?;

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);

    // --- A: the ribbon ------------------------------------------------------
    let region = format!("ribbon.mode.{MODE}");
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, &region).ok_or_else(|| {
        Error::new(format!(
            "no `{region}` region. Declared under `ribbon.mode.`: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.mode."))
        ))
    })?;
    let mode_lines = |s: &Session| -> Result<usize> {
        Ok(shell_trace(s)?
            .events(MODE_EVENT)
            .filter(|l| l.get("mode") == Some(MODE))
            .count())
    };
    let before = mode_lines(&session)?;
    pointer.click(&session, WindowPoint::centre_of(rect))?;
    session.settle(12);
    if mode_lines(&session)? <= before {
        return Ok(Some(format!(
            "★ a scripted click on `{region}` at its centre was delivered and selected no mode. \
             egui saw the press and nothing under it answered: either the press landed before \
             hover did, or the point is not in the space `ui-rect` lines are written in. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ a scripted click reached the ribbon");

    // --- B: the canvas ------------------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let at = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    pointer.click(&session, at)?;
    session.settle(25);
    let selected = session
        .trace()?
        .last(SELECTION)
        .and_then(|l| l.get("first").map(str::to_owned))
        .unwrap_or_else(|| "none".to_owned());
    if selected == "none" {
        return Ok(Some(format!(
            "★★ a scripted click on the pinned text point selected nothing. The OS-driven \
             twin `double_clicking_a_text_box_edits_the_text` selects text at this point, so \
             the seam, not the fixture, is the suspect. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ a scripted canvas click selected `{selected}`"));

    // --- C: click counting --------------------------------------------------
    let ack = pointer.double_click(&session, at)?;
    session.settle(35);
    pointer.gone(&session)?;
    if session.trace()?.last(CARET).is_none() {
        return Ok(Some(format!(
            "★★★ a scripted double-click produced no `{CARET}` line (`{}`). A double-click \
             split into two singles by a slow frame looks like this; so does one egui never \
             counted. Trace: {}.",
            ack.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★★ a scripted double-click opened the caret — no OS input was used");
    Ok(None)
}
