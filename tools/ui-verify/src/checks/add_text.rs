//! `add_text` — **`edit.add_text` places a caret and typed characters reach it.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/add_text.md`.

use crate::checks::ctxdrive;
use crate::checks::driving::{declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::DocPoint;
use crate::error::{Error, Result};
use crate::input::Click;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// How far from the caret the committing click lands, in PDF points.
///
/// Far enough that it cannot be read as a second click on the same word, and
/// still on the sheet for any `--doc-point` that was itself on it.
const AWAY_PT: f64 = 140.0;

/// See the module documentation.
pub struct AddTextTakesRealKeystrokes;

impl Check for AddTextTakesRealKeystrokes {
    fn name(&self) -> &'static str {
        "add_text_takes_real_keystrokes"
    }

    fn defect(&self) -> &'static str {
        "the Add text tool arms and places a caret, and characters typed into it \
         never reach the draft — so the operator gets a caret that ignores them and nothing is \
         ever written to the page"
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
    let target = ctx.target.ok_or_else(|| {
        Error::new("no --doc-point. This check needs somewhere on the page to place the caret.")
    })?;
    let (session, pointer) = ctxdrive::launch(ctx, report, "add_text", "", &[])?;
    let outcome = typed_and_committed(ctx, report, &session, &pointer, target);
    pointer.gone(&session)?;
    outcome
}

/// Arm Add text, place the caret at `target`, type, click away.
fn typed_and_committed(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    target: DocPoint,
) -> Result<Option<String>> {
    if let Some(failure) = armed(ctx, report, session, pointer)? {
        return Ok(Some(failure));
    }
    // --- B: place the caret -------------------------------------------------
    ctxdrive::click_doc(ctx, session, pointer, target)?;
    let trace = session.trace()?;
    if !trace
        .events("text-edit-caret")
        .any(|l| l.get("kind") == Some("Add"))
    {
        let declined: Vec<&str> = trace
            .events("text-edit-declined")
            .filter_map(|l| l.get("reason"))
            .collect();
        return Ok(Some(format!(
            "the click on the page traced no `text-edit-caret kind=Add`, so no draft was started and there is nothing for a keystroke to reach. Refusals traced: {}.",
            list_str(&declined)
        )));
    }
    report.note("the click on blank paper started an Add draft");
    if let Some(failure) = typed(report, session, pointer)? {
        return Ok(Some(failure));
    }
    // The in-place editor mid-draft: the trace can say a keystroke reached the
    // draft, not that the operator can see it. A picture for a human to read.
    ctxdrive::picture(ctx, report, session, pointer, "add_text_draft.png");

    // --- D: click away, which is how a draft commits ------------------------
    ctxdrive::click_doc(
        ctx,
        session,
        pointer,
        DocPoint {
            page: target.page,
            x: target.x + AWAY_PT,
            y: target.y - AWAY_PT,
        },
    )?;
    session.settle(10);
    let trace = session.trace()?;
    if trace.events("add-text").next().is_none() {
        return Ok(Some(
            "the draft had characters in it and clicking away traced no `add-text`, so the commit never reached the engine. `textedit::click` is supposed to commit an existing draft before starting a new one — clicking away is every editor's \"that word is finished\", and it is the ONLY commit route a mouse-driven operator has, because Enter is the other one."
                .to_owned(),
        ));
    }
    report.note("clicking away committed the draft: `add-text` reached the engine");
    Ok(None)
}

/// Edit mode, Edit tab, Add text; `Some(failure)` when nothing armed.
fn armed(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ctxdrive::ui_rect(ctx)?;
    // --- A: Edit mode, Edit tab, arm Add text ------------------------------
    crate::checks::driving::click_mode_segment(session, pointer, ui_rect, "edit")?;
    ctxdrive::open_tab(ctx, session, pointer, "edit")?;
    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, "ribbon.item.edit.add_text").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.item.edit.add_text` region on the Edit tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.edit."))
        ))
    })?;
    pointer.click_rect(session, item)?;
    session.settle(14);
    let trace = session.trace()?;
    if !trace
        .events("text-edit-tool")
        .any(|l| l.get("tool").is_some_and(|t| t.contains("Add")))
    {
        let tools: Vec<&str> = trace
            .events("text-edit-tool")
            .filter_map(|l| l.get("tool"))
            .collect();
        return Ok(Some(format!(
            "clicking Edit > Add text traced no `text-edit-tool tool=TextEdit(Add)` line, so the control armed nothing. Tools traced this run: {}.",
            list_str(&tools)
        )));
    }
    report.note("Edit > Add text armed the caret in Add mode");
    Ok(None)
}

/// Type two characters into the draft; `Some(failure)` when it did not grow.
///
/// What is typed does not matter: the assertion is that the draft grew.
/// `text-edit-typing` is a change-only trace of `draft / owns_keyboard /
/// text_events / len`, and each field kills a different hypothesis, which is
/// why the failure quotes the whole line.
fn typed(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    for text in ["f", "2"] {
        pointer.type_text(session, None, text)?;
        session.settle(10);
    }
    let trace = session.trace()?;
    let typing: Vec<&str> = trace
        .events("text-edit-typing")
        .map(|l| l.raw.as_str())
        .collect();
    let grew = trace
        .events("text-edit-typing")
        .filter_map(|l| l.get("len"))
        .filter_map(|v| v.parse::<usize>().ok())
        .any(|n| n > 0);
    if !grew {
        return Ok(Some(format!(
            "TWO TYPED CHARACTERS REACHED NOTHING. The tool armed, the click placed a caret, and no `text-edit-typing` line ever reported a draft longer than zero. Read the fields of the lines below: `owns_keyboard=false` means a text field held focus and `typing` refused the events; `text_events=0` means egui delivered no `Event::Text` at all; a rising `text_events` with a flat `len` means the events arrived and `insert` did not land them. Lines traced: {}.",
            list_str(&typing)
        )));
    }
    report.note("typed characters reached the draft");
    Ok(None)
}

/// Render a list of borrowed strings for a failure message.
fn list_str(items: &[&str]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}
