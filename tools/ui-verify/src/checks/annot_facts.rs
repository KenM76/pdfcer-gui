//! Properties reads a selected annotation's `/C` and `/Name` when neither
//! restyle verb reads the mark.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/annot_facts.md`.

use super::dimdrive::{Fixture, click_on, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,file.properties";
const FIXTURE: Fixture = Fixture {
    file: "annot-facts.pdf",
    method: "Rebuild it with `python fixtures/annot-facts.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
const SHOWN: &str = "annot-facts-shown"; // ui-text-exempt: a trace event name, never displayed
/// Inside the fixture's `/FreeText`, `/Rect [100 100 200 140]`.
const ON_MARK: (f64, f64) = (150.0, 120.0);
/// The fixture's `/C [1 0 0]`, as the trace words it.
const COLOUR: &str = "1,0,0";
/// The fixture's `/Name /Comment`.
const ICON: &str = "Comment";

/// The colour and icon a mark carries are read for any subtype.
pub struct AnAnnotationsColourAndIconAreReadForAnySubtype;

impl Check for AnAnnotationsColourAndIconAreReadForAnySubtype {
    fn name(&self) -> &'static str {
        "an_annotations_colour_and_icon_are_read_for_any_subtype"
    }

    fn defect(&self) -> &'static str {
        "Properties shows nothing of a selected annotation's colour or icon when neither \
         restyle verb reads it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "annot-facts", read)
    }
}

fn read(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, ON_MARK)?;
    session.settle(30);
    let trace = session.trace()?;
    if !trace
        .last_after(SELECTED, mark)
        .is_some_and(|l| l.get("subtype") == Some("FreeText"))
    {
        return Ok(Some(format!(
            "a click inside the mark traced no `{SELECTED}` with subtype=FreeText. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let shot = ctx.out("annot-facts-after.png");
    pointer.screenshot(session, &shot)?;
    report.artifact(shot);
    let Some(shown) = trace.last_after(SHOWN, mark) else {
        return Ok(Some(format!(
            "the mark was selected and Properties traced no `{SHOWN}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("shown: `{}`", shown.raw));
    Ok(
        (shown.get("colour") != Some(COLOUR) || shown.get("icon") != Some(ICON)).then(|| {
            format!(
                "`{}` must read the fixture's colour={COLOUR} and icon={ICON}.",
                shown.raw
            )
        }),
    )
}
