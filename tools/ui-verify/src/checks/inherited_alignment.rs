//! `a_fields_alignment_can_go_back_to_inherited` — on a form whose `/AcroForm`
//! states centred and whose field states nothing, the Alignment chooser shows
//! *Inherited (centred)*; choosing Left writes `/Q 0` on the field, and
//! choosing Inherited removes it again, so the field is back to centred.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/inherited_alignment.md`.

use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "inherited-alignment.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/inherited-alignment.PROVENANCE.py`.";
const FIELD: &str = "FieldOne";
/// `panels::properties::fieldedit::ALIGNMENT_REGION`'s combo button.
const COMBO: &str = "properties.field_edit.alignment.combo";
const LEFT: &str = "properties.field_edit.alignment.option.0";
const INHERIT: &str = "properties.field_edit.alignment.option.inherit";
const READ: &str = "field-alignment-read"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AFieldsAlignmentCanGoBackToInherited;

impl Check for AFieldsAlignmentCanGoBackToInherited {
    fn name(&self) -> &'static str {
        "a_fields_alignment_can_go_back_to_inherited"
    }

    fn defect(&self) -> &'static str {
        "once a field's alignment is chosen it can never inherit the form's again, and an \
         inherited alignment shows as if the field stated it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = properties_pane::launch_on_field(
            ctx,
            &mut report,
            (FIXTURE, METHOD),
            FIELD,
            "inherited_alignment",
        )
        .and_then(|(session, pointer)| {
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

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let reads = |report: &mut CheckReport, after: &str, want: &[(&str, &str)]| {
        properties_pane::reads(session, report, (READ, FIELD), after, want)
    };
    // `q` is the resolved `/Q` code: 0 left, 1 centred.
    if let Some(failure) = reads(report, "on opening", &[("own", "0"), ("q", "1")])? {
        return Ok(Some(failure));
    }
    for (option, after, want) in [
        (LEFT, "after choosing Left", [("own", "1"), ("q", "0")]),
        (
            INHERIT,
            "after choosing Inherited",
            [("own", "0"), ("q", "1")],
        ),
    ] {
        properties_pane::press(ctx, session, pointer, COMBO, 20)?;
        properties_pane::press(ctx, session, pointer, option, 30)?;
        if let Some(failure) = reads(report, after, &want)? {
            return Ok(Some(failure));
        }
    }
    Ok(None)
}
