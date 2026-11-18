//! `typing_into_a_form_preview_field_fills_the_form` — a click on a field in
//! Form preview opens it for typing, and Enter writes the value into the
//! form's data, after which the preview draws it.
//!
//! Rent holds 1250 in the fixture's data. The check selects it, types 975,
//! presses Enter, and requires the engine's outcome, the funnel's disclosure,
//! and a redraw after the edit whose pixel digest differs from the first.

use crate::checks::driving::{declared, declared_in};
use crate::checks::form_pages::launch_form;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::launch::Session;
use crate::report::CheckReport;

const DRAWN: &str = "form-preview";
/// The second field on the first page: Rent.
const RENT: &str = "form-preview.field.0.1";
const APPLIED: &str = "form-fill-applied";
/// The funnel's line for the edit, carrying its disclosure.
const FUNNEL: &str = "form-fill";
const TYPED: &str = "975";

/// See the module documentation.
pub struct TypingIntoAFormPreviewFieldFillsTheForm;

impl Check for TypingIntoAFormPreviewFieldFillsTheForm {
    fn name(&self) -> &'static str {
        "typing_into_a_form_preview_field_fills_the_form"
    }

    fn defect(&self) -> &'static str {
        "a dynamic form's fields cannot be filled in: a click on one in Form preview opens \
         nothing, or Enter writes nothing, writes it silently, or the preview keeps the old value"
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
    let (session, pointer, ui_rect) = launch_form(ctx, report, "form_fill")?;
    let trace = session.trace()?;
    if trace.last(DRAWN).is_none() {
        return Err(Error::new(format!(
            "the preview that opens with a dynamic form traced no `{DRAWN}` line; \
             `a_dynamic_form_opens_on_its_own_pages` owns that defect. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some((rect, vp)) = declared_in(&trace, ui_rect, RENT) else {
        return Ok(Some(format!(
            "★★★ the preview drew the form and declared no `{RENT}` region: its fields \
             cannot be clicked. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let digest_before = trace
        .last(DRAWN)
        .and_then(|l| l.get("digest").map(str::to_owned));
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(10);
    pointer.key(&session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(&session, vp.as_deref(), TYPED)?;
    pointer.key(&session, vp.as_deref(), "Enter", None)?;
    session.settle(60);
    let shot = ctx.out("form_fill.after.png");
    pointer.screenshot(&session, &shot)?;
    report.artifact(shot);
    judge(&session, ui_rect, digest_before.as_deref())
}

/// The engine's outcome, the funnel's disclosure, and a redraw after the
/// edit whose pixels differ from the first drawing's.
fn judge(session: &Session, ui_rect: &str, before: Option<&str>) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "★★★ typing {TYPED} into Rent and pressing Enter traced no `{APPLIED}` line: \
             nothing was written. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if applied.get("chars") != Some("3") {
        return Ok(Some(format!(
            "★★★ `{}` must carry chars=3: Ctrl+A then {TYPED} replaces the field's 1250.",
            applied.raw
        )));
    }
    let Some(funnel) = trace
        .events(FUNNEL)
        .find(|l| l.raw.contains("disclosures=") && l.raw.contains("epoch="))
    else {
        return Ok(Some(format!(
            "★★★ no `{FUNNEL}` edit line followed `{APPLIED}`: the value did not pass the \
             funnel, so it is not on the undo stack."
        )));
    };
    if funnel.raw.contains("disclosures=none") {
        return Ok(Some(format!(
            "★★★ `{}` disclosed nothing: the status row must say what was filled and how to \
             undo it.",
            funnel.raw
        )));
    }
    let Some(redrawn) = trace.last_after(DRAWN, applied.lineno) else {
        return Ok(Some(format!(
            "★★★ the preview drew no page after `{APPLIED}`: it still shows the old value."
        )));
    };
    if before.is_none() || redrawn.get("digest") == before {
        return Ok(Some(format!(
            "★★★ the preview redrew with the same pixels as before the fill (digest {before:?}): \
             it does not show the typed value."
        )));
    }
    if declared(&trace, ui_rect, RENT).is_none() {
        return Ok(Some(format!(
            "★★★ `{RENT}` was not declared after the redraw: the filled field cannot be \
             clicked again."
        )));
    }
    Ok(None)
}
