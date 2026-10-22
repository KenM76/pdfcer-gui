//! While Play 3D content is off, View… on a 3D model raises Acrobat's 3D
//! prompt instead of the viewer: Cancel shows nothing, *Trust this document
//! one time only* shows the model and lets the next View… through without
//! asking, and that one-time trust is not written to the preferences.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/model_trust.md`.

use crate::checks::driving::declared_in;
use crate::checks::model_view_window::{VIEW_REGION, launch_with_model, press_insert, ui_rect};
use crate::checks::scripteddrive::userdata;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const DECISION: &str = "model3d-decision"; // ui-text-exempt: a trace event name, never displayed
const ANSWER: &str = "model3d-answer"; // ui-text-exempt: a trace event name, never displayed
const OPENED: &str = "model-view-opened"; // ui-text-exempt: a trace event name, never displayed
const BODY: &str = "model_trust.body";
const ONCE: &str = "dialog.buttons.accept";
const REFUSE: &str = "dialog.buttons.cancel";
/// The preferences key the lasting answer is kept under.
const TRUSTED_KEY: &str = "trusted_3d_document";

/// See the module documentation.
pub struct ThreeDContentAsksWhileItIsOff;

impl Check for ThreeDContentAsksWhileItIsOff {
    fn name(&self) -> &'static str {
        "three_d_content_asks_while_it_is_off"
    }

    fn defect(&self) -> &'static str {
        "with Play 3D content off, a model opens without asking, or Cancel opens it anyway, or \
         trusting the document once does not open it or is forgotten at the next View…, or is \
         written to the preferences as if it were always"
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
    let dir = userdata(ctx)?;
    // Through `sandbox::write_prefs`, which keeps the startup offer suppressed.
    crate::sandbox::write_prefs(&dir, "play_3d = false\n")
        .map_err(|e| Error::new(format!("could not write preferences: {e}")))?;
    let (session, pointer) = launch_with_model(ctx, report, "model-trust")?;
    let failure = steps(ctx, report, &session, &pointer)?;
    pointer.gone(&session)?;
    drop(session);
    if failure.is_some() {
        return Ok(failure);
    }
    let prefs = std::fs::read_to_string(dir.join(crate::sandbox::PREFS_FILE)).unwrap_or_default();
    if prefs
        .lines()
        .any(|l| l.trim_start().starts_with(TRUSTED_KEY))
    {
        return Ok(Some(format!(
            "Trust this document one time only wrote `{TRUSTED_KEY}` to the preferences, which \
             is what Trust this document always does."
        )));
    }
    Ok(None)
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if !press_insert(ctx, session, pointer)? {
        return Ok(Some(
            "the 3D model button is on neither the Edit tab nor its overflow.".into(),
        ));
    }
    // 1 — View… asks; Cancel shows nothing.
    if let Some(failure) = view(ctx, report, session, pointer, "ask")? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, REFUSE)?;
    if let Some(failure) = answered(session, report, mark, "no")? {
        return Ok(Some(failure));
    }
    if session.trace()?.last_after(OPENED, mark).is_some() {
        return Ok(Some(format!(
            "Cancel opened the viewer anyway. Trace: {}.",
            session.trace_path().display()
        )));
    }
    // 2 — it asks again; trusting once opens the viewer.
    if let Some(failure) = view(ctx, report, session, pointer, "ask")? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, ONCE)?;
    if let Some(failure) = answered(session, report, mark, "once")? {
        return Ok(Some(failure));
    }
    if session.trace()?.last_after(OPENED, mark).is_none() {
        return Ok(Some(format!(
            "Trust this document one time only did not open the viewer (look for \
             `model-view-declined`). Trace: {}.",
            session.trace_path().display()
        )));
    }
    // 3 — the one-time trust holds while the document is open.
    view(ctx, report, session, pointer, "play")
}

/// Press View… on the model's Attachments row and require `decision=<want>`;
/// the prompt must be drawn exactly when the decision is `ask`.
fn view(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    want: &str,
) -> Result<Option<String>> {
    let ui_rect = ui_rect(ctx)?;
    let trace = session.trace()?;
    let Some((view, vp)) = declared_in(&trace, ui_rect, VIEW_REGION) else {
        return Ok(Some(format!(
            "no `{VIEW_REGION}` region beside the placed model (was it placed? look for `add-3d`)."
        )));
    };
    let mark = trace.mark();
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(view))?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(DECISION, mark) else {
        return Ok(Some(format!(
            "View… traced no `{DECISION}` line, so nothing decided whether 3D content may \
             play. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("view: `{}`", line.raw));
    if line.get("decision") != Some(want) {
        return Ok(Some(format!(
            "expected decision={want}, the trace says `{}`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    let drawn = declared_in(&trace, ui_rect, BODY).is_some();
    if (want == "ask") != drawn {
        return Ok(Some(format!(
            "decision={want}, but the 3D prompt was {} after View…. Trace: {}.",
            if drawn { "drawn" } else { "not drawn" },
            session.trace_path().display()
        )));
    }
    Ok(None)
}

/// Press a published dialog button.
fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let (rect, vp) = declared_in(&trace, ui_rect(ctx)?, region)
        .ok_or_else(|| Error::new(format!("the 3D prompt published no `{region}` button")))?;
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}

/// The prompt's answer after `mark` is `trust=<want>`.
fn answered(
    session: &Session,
    report: &mut CheckReport,
    mark: usize,
    want: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let line = trace.last_after(ANSWER, mark);
    if let Some(l) = line {
        report.note(format!("answer: `{}`", l.raw));
        if l.get("trust") == Some(want) {
            return Ok(None);
        }
    }
    Ok(Some(format!(
        "expected `{ANSWER} trust={want}`, the trace says {}. Trace: {}.",
        line.map_or("nothing", |l| l.raw.as_str()),
        session.trace_path().display()
    )))
}
