//! `a_redraw_says_what_it_did_to_a_fields_text` — on a form whose fields ask
//! for an automatic text size, a change that redraws a field says on the
//! status line which size pdfcer chose: choosing an Alignment in Properties
//! names the field, and the Forms panel's *Redraw values* names the form.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/redraw_layout.md`.

use crate::checks::driving::{declared, declared_in};
use crate::checks::properties_pane;
use crate::checks::security_notes::await_line;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "autosize-fields.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/autosize-fields.PROVENANCE.py`.";
const FIELD: &str = "FieldOne";
const UI_RECT: &str = "ui-rect";
/// `panels::properties::fieldedit::ALIGNMENT_REGION`'s combo button and its
/// Left option.
const COMBO: &str = "properties.field_edit.alignment.combo";
const LEFT: &str = "properties.field_edit.alignment.option.0";
/// `panels::forms::REGION_REGENERATE`.
const REGENERATE: &str = "forms.regenerate";
const FORMS: &str = "mode.review,view.panel_forms";
const EDIT_FIELD: &str = "edit-field"; // ui-text-exempt: a trace event name, never displayed
const REGEN_LINE: &str = "form-regenerate-appearances"; // ui-text-exempt: a trace event name, never displayed
/// `app::status::REGION_EDIT_DISCLOSURE`.
const BAR: &str = "status-group:edit-disclosure"; // ui-text-exempt: a trace region name, never displayed
/// `text::forms::redraw_notes`' auto-size sentence for each subject.
const FIELD_SAYS: &str = "“FieldOne” was redrawn at an automatic text size";
const FORM_SAYS: &str = "A field of this form was redrawn at an automatic text size";

/// See the module documentation.
pub struct ARedrawSaysWhatItDidToAFieldsText;

impl Check for ARedrawSaysWhatItDidToAFieldsText {
    fn name(&self) -> &'static str {
        "a_redraw_says_what_it_did_to_a_fields_text"
    }

    fn defect(&self) -> &'static str {
        "a property change, a reset or a redraw picks a field's text size, colour or characters \
         for the operator and says nothing, so only a fill ever reports what pdfcer chose"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven =
            drive(ctx, &mut report, "redraw_layout_field", field_arm).and_then(|f| match f {
                Some(failure) => Ok(Some(failure)),
                None => drive(ctx, &mut report, "redraw_layout_form", form_arm),
            });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

type Arm =
    fn(&CheckContext, &mut CheckReport, &Session, &ScriptedPointer) -> Result<Option<String>>;

/// Launch for `arm`, run it, and park the pointer whatever it found.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    label: &str,
    arm: Arm,
) -> Result<Option<String>> {
    let invoke = if label.ends_with("form") {
        FORMS
    } else {
        "mode.edit,file.properties"
    };
    let (session, pointer) = properties_pane::launch_on_field_invoking(
        ctx,
        report,
        (FIXTURE, METHOD),
        (FIELD, invoke),
        label,
    )?;
    let outcome = arm(ctx, report, &session, &pointer);
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// The newest `event` line's `key` must contain `says`, and the status bar's
/// disclosure group must be drawn.
fn said(
    report: &mut CheckReport,
    session: &Session,
    (event, key): (&str, &str),
    says: &str,
) -> Result<Option<String>> {
    let Some(line) = await_line(session, event)? else {
        return Ok(Some(format!(
            "no `{event}` line: the redraw never ran. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("`{}`", line.raw));
    // The key is the line's last and its value holds spaces, so it is read
    // as the rest of the raw line.
    let value = line.raw.split_once(&format!(" {key}=")).map(|(_, v)| v);
    if !value.is_some_and(|v| v.contains(says)) {
        return Ok(Some(format!(
            "the redraw chose an automatic text size and `{key}=` does not say \"{says}\": `{}`.",
            line.raw
        )));
    }
    if declared(&session.trace()?, UI_RECT, BAR).is_none() {
        return Ok(Some(format!(
            "the sentence was recorded and no `{BAR}` region was drawn: the status line said \
             nothing."
        )));
    }
    Ok(None)
}

fn field_arm(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    properties_pane::press(ctx, session, pointer, COMBO, 20)?;
    properties_pane::press(ctx, session, pointer, LEFT, 30)?;
    said(report, session, (EDIT_FIELD, "disclosures"), FIELD_SAYS)
}

fn form_arm(
    _: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    for _ in 0..30 {
        if let Some((rect, vp)) = declared_in(&session.trace()?, UI_RECT, REGENERATE) {
            pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
            session.settle(30);
            return said(report, session, (REGEN_LINE, "redrawn"), FORM_SAYS);
        }
        session.settle(10);
    }
    Ok(Some(format!(
        "the Forms panel was opened and drew no visible `{REGENERATE}` button. Trace: {}.",
        session.trace_path().display()
    )))
}
