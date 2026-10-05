//! `a_widget_borders_dash_can_be_chosen` — the Properties panel offers a
//! dashed widget border's pattern, and a pick reaches the file's `/BS /D`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/widget_dash.md`.

use crate::checks::properties_pane;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "all-field-kinds.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/all-field-kinds.PROVENANCE.py`.";
/// A text field with a solid one-point border.
const FIELD: &str = "TextOne";
const STYLE_COMBO: &str = "properties.widget_edit.border";
/// Dashed is entry 1 of the style row's `STYLES`.
const DASHED_ENTRY: &str = "properties.widget_edit.border.1";
const DASH_COMBO: &str = "properties.widget_edit.dash";
/// Long dash is entry 1 of `widgetdash::DASHES`.
const LONG_DASH_ENTRY: &str = "properties.widget_edit.dash.1";
const SHOWN: &str = "widget-dash-shown";
const APPLIED: &str = "edit-widget-applied";

/// See the module documentation.
pub struct AWidgetBordersDashCanBeChosen;

impl Check for AWidgetBordersDashCanBeChosen {
    fn name(&self) -> &'static str {
        "a_widget_borders_dash_can_be_chosen"
    }

    fn defect(&self) -> &'static str {
        "a dashed field border offers no choice of pattern, or a pick never reaches the \
         widget's /BS /D, or the box is not redrawn with it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = properties_pane::launch_on_field(
            ctx,
            &mut report,
            (FIXTURE, METHOD),
            FIELD,
            "widget_dash",
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
    // A solid border offers no dash row at all (R9).
    if session
        .trace()?
        .events(SHOWN)
        .any(|l| l.get("field") == Some(FIELD))
    {
        return Ok(Some(format!(
            "★★★ the dash row drew for {FIELD}'s solid border. Trace: {}.",
            session.trace_path().display()
        )));
    }
    properties_pane::press(ctx, session, pointer, STYLE_COMBO, 15)?;
    properties_pane::press(ctx, session, pointer, DASHED_ENTRY, 30)?;
    // `/S /D` with no `/D` reads as Table 166's default dash.
    let shown = (SHOWN, FIELD);
    if let Some(failure) = properties_pane::reads(
        session,
        report,
        shown,
        "after Dashed",
        &[("dash", "dashed")],
    )? {
        return Ok(Some(failure));
    }
    properties_pane::press(ctx, session, pointer, DASH_COMBO, 15)?;
    properties_pane::press(ctx, session, pointer, LONG_DASH_ENTRY, 30)?;
    if let Some(failure) = properties_pane::reads(
        session,
        report,
        (APPLIED, FIELD),
        "after Long dash",
        &[("redrawn", "yes")],
    )? {
        return Ok(Some(failure));
    }
    // Read back off the widget dictionary's `/BS /D` on the next frame.
    properties_pane::reads(
        session,
        report,
        shown,
        "after Long dash",
        &[("dash", "long-dash")],
    )
}
