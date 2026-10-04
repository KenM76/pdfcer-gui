//! `a_text_fields_extras_reach_the_file` — the Properties panel's Scroll long
//! text, Check spelling, Sends a file and Sent with the form checkboxes and its Export name box each
//! change the selected text field in the document, read back from it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_extras.md`.

use crate::checks::driving::declared;
use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "three-text-fields.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/three-text-fields.PROVENANCE.py`.";
const FIELD: &str = "FieldOne";
const SCROLL: &str = "properties.field_edit.scroll";
const SPELL: &str = "properties.field_edit.spell_check";
const FILE_SELECT: &str = "properties.field_edit.file_select";
const FILE_NOTE: &str = "properties.field_edit.file_select.note";
const EXPORT: &str = "properties.field_edit.export_name";
const SENT: &str = "properties.field_edit.sent";
const READ: &str = "field-extras-read";
const TYPED: &str = "qty_total";

/// See the module documentation.
pub struct ATextFieldsExtrasReachTheFile;

impl Check for ATextFieldsExtrasReachTheFile {
    fn name(&self) -> &'static str {
        "a_text_fields_extras_reach_the_file"
    }

    fn defect(&self) -> &'static str {
        "a text field's Properties panel offers no scroll, spell-check, file-select or export-name \
         control, or a change to one never reaches edit_field and is not read back from the file"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = steps(ctx, &mut report, &session, &pointer);
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

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    properties_pane::launch_on_field(ctx, report, (FIXTURE, METHOD), FIELD, "field_extras")
}

/// The newest `field-extras-read` line for the field must carry `want`.
fn reads(
    session: &Session,
    report: &mut CheckReport,
    after: &str,
    want: &[(&str, &str)],
) -> Result<Option<String>> {
    properties_pane::reads(session, report, (READ, FIELD), after, want)
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let start = [
        ("scroll", "1"),
        ("spell", "1"),
        ("file", "0"),
        ("sent", "1"),
        ("export", "-"),
    ];
    if let Some(failure) = reads(session, report, "before any press", &start)? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, SCROLL, 30)?;
    if let Some(failure) = reads(
        session,
        report,
        "after Scroll long text",
        &[("scroll", "0")],
    )? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, SPELL, 30)?;
    if let Some(failure) = reads(session, report, "after Check spelling", &[("spell", "0")])? {
        return Ok(Some(failure));
    }
    let vp = properties_pane::press(ctx, session, pointer, EXPORT, 20)?;
    pointer.type_text(session, vp.as_deref(), TYPED)?;
    pointer.key(session, vp.as_deref(), "Enter", None)?;
    session.settle(30);
    if let Some(failure) = reads(
        session,
        report,
        "after typing an export name",
        &[("export", TYPED)],
    )? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, FILE_SELECT, 30)?;
    let after = [
        ("file", "1"),
        ("scroll", "0"),
        ("spell", "0"),
        ("export", TYPED),
    ];
    if let Some(failure) = reads(session, report, "after Sends a file", &after)? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, SENT, 30)?;
    if let Some(failure) = reads(
        session,
        report,
        "after Sent with the form",
        &[("sent", "0"), ("file", "1"), ("export", TYPED)],
    )? {
        return Ok(Some(failure));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    Ok(declared(&session.trace()?, ui_rect, FILE_NOTE)
        .is_none()
        .then(|| {
            format!(
                "★★★ the field now sends a file, but no `{FILE_NOTE}` sentence says what that \
             changes. Trace: {}.",
                session.trace_path().display()
            )
        }))
}
