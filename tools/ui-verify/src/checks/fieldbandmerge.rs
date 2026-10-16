//! A Shift-marquee adds its fields to the selection and a Ctrl-marquee takes
//! them out, driven on `all-field-kinds.pdf` and read through the count a
//! following Shift-click reports.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/fieldbandmerge.md`.

use super::dimdrive::{click_on, launch_with};
use super::field_multiselect::{BAND, BANDED, COMBO, FIXTURE, SELECTED, TEXT, TOGGLED};
use super::multiselect::{after, band_with, shift_click, step};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// See the module documentation.
pub struct ModifiedMarqueesMergeFields;

impl Check for ModifiedMarqueesMergeFields {
    fn name(&self) -> &'static str {
        "shift_and_ctrl_marquees_add_and_remove_fields"
    }

    fn defect(&self) -> &'static str {
        "A Shift-marquee over form fields replaces the selection instead of adding to it, or a \
         Ctrl-marquee leaves the fields it caught selected"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit",
            "fieldbandmerge",
            &[],
        )
        .and_then(|(session, pointer)| {
            let outcome = drive(ctx, &mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A marquee with `mods` over [`BAND`], then a Shift-click on ComboOne; the
/// failure sentence unless the band merged as `merge` and the click found
/// `before` fields selected.
fn band_then_count(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    (mods, merge, before): (&str, &str, &str),
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    band_with(ctx, session, pointer, &FIXTURE, BAND, Some(mods))?;
    let banded = step!(after(
        session,
        BANDED,
        mark,
        &format!("a {mods}-marquee was dragged")
    ));
    report.note(format!("`{}`", banded.raw));
    if banded.get("caught") != Some("3") || banded.get("merge") != Some(merge) {
        return Ok(Some(format!(
            "a {mods}-marquee around CheckOne and both radio members traced `{}`: three \
             caught under `merge={merge}` was wanted.",
            banded.raw
        )));
    }
    let mark = session.trace()?.mark();
    shift_click(ctx, session, pointer, &FIXTURE, COMBO.2)?;
    let toggled = step!(after(session, TOGGLED, mark, "ComboOne was Shift-clicked"));
    report.note(format!("`{}`", toggled.raw));
    Ok((toggled.get("before") != Some(before)).then(|| {
        format!(
            "after the {mods}-marquee, Shift-clicking ComboOne traced `{}`: {before} fields \
             selected before it was wanted.",
            toggled.raw
        )
    }))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, TEXT.2)?;
    let selected = step!(after(session, SELECTED, mark, "TextOne was clicked"));
    if selected.get("field") != Some(TEXT.0) {
        return Ok(Some(format!(
            "a click on TextOne selected `{}`.",
            selected.raw
        )));
    }
    // TextOne plus the band's three: four before ComboOne joins. A band that
    // replaced would leave three.
    if let Some(why) = band_then_count(ctx, report, session, pointer, ("shift", "add", "4"))? {
        return Ok(Some(why));
    }
    // Five now; the Ctrl band takes its three out, leaving TextOne and
    // ComboOne. A band that kept its catch leaves five.
    band_then_count(ctx, report, session, pointer, ("ctrl", "subtract", "2"))
}
