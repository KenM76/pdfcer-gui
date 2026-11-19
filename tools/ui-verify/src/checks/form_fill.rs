//! Typing into a Form preview field writes the form's data.
//!
//! - `typing_into_a_form_preview_field_fills_the_form`: Rent holds 1250 in
//!   the fixture's data. The check selects it, types 975 and presses Enter.
//! - `a_field_with_no_data_value_yet_fills_in`: Deposit's data reference
//!   `$.Deposit` names a value the data does not hold. Typing 500 must create
//!   it (`created=1`), not refuse.
//!
//! Both require the engine's outcome, the funnel's disclosure, and a redraw
//! after the edit whose pixel digest differs from the first.

use crate::checks::driving::{declared, declared_in};
use crate::checks::form_pages::launch_form;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::launch::Session;
use crate::report::CheckReport;

const DRAWN: &str = "form-preview";
const APPLIED: &str = "form-fill-applied";
/// The funnel's line for the edit, carrying its disclosure.
const FUNNEL: &str = "form-fill";

/// One field typed into: its region, what is typed, and what the outcome
/// must say.
struct Typing {
    stem: &'static str,
    label: &'static str,
    region: &'static str,
    typed: &'static str,
    /// The data values the fill must create.
    created: &'static str,
}

/// The second field on the first page: Rent, bound to `$.Rent`.
const RENT: Typing = Typing {
    stem: "form_fill",
    label: "Rent",
    region: "form-preview.field.0.1",
    typed: "975",
    created: "0",
};

/// The third field on the first page: Deposit, whose `$.Deposit` is absent.
const DEPOSIT: Typing = Typing {
    stem: "form_fill_created",
    label: "Deposit",
    region: "form-preview.field.0.2",
    typed: "500",
    created: "1",
};

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
        run_typing(self, ctx, &RENT)
    }
}

/// See the module documentation.
pub struct AFieldWithNoDataValueYetFillsIn;

impl Check for AFieldWithNoDataValueYetFillsIn {
    fn name(&self) -> &'static str {
        "a_field_with_no_data_value_yet_fills_in"
    }

    fn defect(&self) -> &'static str {
        "a dynamic form's field whose data reference names a value the data does not hold yet \
         refuses to be filled in, or fills without creating the value"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_typing(self, ctx, &DEPOSIT)
    }
}

fn run_typing(check: &dyn Check, ctx: &CheckContext, field: &Typing) -> CheckReport {
    let mut report = CheckReport::new(check.name(), check.defect());
    match drive(ctx, &mut report, field) {
        Ok(Some(failure)) => report.fail(failure),
        Ok(None) => report.pass(),
        Err(why) => report.from_error(&why),
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport, field: &Typing) -> Result<Option<String>> {
    let (session, pointer, ui_rect) = launch_form(ctx, report, field.stem)?;
    let trace = session.trace()?;
    if trace.last(DRAWN).is_none() {
        return Err(Error::new(format!(
            "the preview that opens with a dynamic form traced no `{DRAWN}` line; \
             `a_dynamic_form_opens_on_its_own_pages` owns that defect. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some((rect, vp)) = declared_in(&trace, ui_rect, field.region) else {
        return Ok(Some(format!(
            "★★★ the preview drew the form and declared no `{}` region: its fields \
             cannot be clicked. Trace: {}.",
            field.region,
            session.trace_path().display()
        )));
    };
    let digest_before = trace
        .last(DRAWN)
        .and_then(|l| l.get("digest").map(str::to_owned));
    pointer.click_in(&session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(10);
    pointer.key(&session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(&session, vp.as_deref(), field.typed)?;
    pointer.key(&session, vp.as_deref(), "Enter", None)?;
    session.settle(60);
    let shot = ctx.out(&format!("{}.after.png", field.stem));
    pointer.screenshot(&session, &shot)?;
    report.artifact(shot);
    judge(&session, ui_rect, field, digest_before.as_deref())
}

/// The engine's outcome, the funnel's disclosure, and a redraw after the
/// edit whose pixels differ from the first drawing's.
fn judge(
    session: &Session,
    ui_rect: &str,
    field: &Typing,
    before: Option<&str>,
) -> Result<Option<String>> {
    let (label, typed) = (field.label, field.typed);
    let trace = session.trace()?;
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "★★★ typing {typed} into {label} and pressing Enter traced no `{APPLIED}` line: \
             nothing was written. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let chars = typed.chars().count().to_string();
    if applied.get("chars") != Some(chars.as_str()) || applied.get("created") != Some(field.created)
    {
        return Ok(Some(format!(
            "★★★ `{}` must carry chars={chars} created={}: Ctrl+A then {typed} replaces \
             {label}'s value, creating {} data value(s).",
            applied.raw, field.created, field.created
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
    if declared(&trace, ui_rect, field.region).is_none() {
        return Ok(Some(format!(
            "★★★ `{}` was not declared after the redraw: the filled field cannot be \
             clicked again.",
            field.region
        )));
    }
    Ok(None)
}
