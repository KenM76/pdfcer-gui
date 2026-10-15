//! `an_align_too_large_for_one_undo_is_refused` — a marquee over 300 markups
//! and Align left, which would move 285 of them: more commands than one undo
//! step folds, so nothing moves and the status line says why.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/annot_too_many.md`.

use super::dimdrive::{Fixture, launch_with};
use super::driving;
use super::multiselect::{after, band, step};
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "annots-many.pdf",
    method: "Rebuild it with `python fixtures/annots-many.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 600.0,
        height_pt: 500.0,
    },
};
/// A window band holding all 300 markups.
const ALL: ((f64, f64), (f64, f64)) = ((5.0, 495.0), (595.0, 5.0));

const BANDED: &str = "annot-band"; // ui-text-exempt: a trace event name
const REFUSED: &str = "move-mixed-refused"; // ui-text-exempt: a trace event name
const APPLIED: &str = "move-mixed-applied"; // ui-text-exempt: a trace event name
const DECLINED: &str = "undo-depth-declined"; // ui-text-exempt: a trace event name
const DECLINE: &str = "status-group:decline"; // ui-text-exempt: a trace region name

/// See the module documentation.
pub struct AnAlignTooLargeForOneUndoIsRefused;

impl Check for AnAlignTooLargeForOneUndoIsRefused {
    fn name(&self) -> &'static str {
        "an_align_too_large_for_one_undo_is_refused"
    }

    fn defect(&self) -> &'static str {
        "Align over more annotations than one undo step holds moves them anyway, so one Undo \
         takes back only part of it; or it is refused without the status line saying why"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let outcome = launch_with(
            ctx,
            &mut report,
            &FIXTURE,
            "mode.edit",
            "annot-too-many",
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

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    if driving::declared(&session.trace()?, ui_rect, DECLINE).is_some() {
        return Ok(Some(format!(
            "`{DECLINE}` was on screen before anything was done, so the refusal's own \
             sentence could not be told from it."
        )));
    }
    let mark = session.trace()?.mark();
    band(ctx, session, pointer, &FIXTURE, ALL)?;
    let banded = step!(after(
        session,
        BANDED,
        mark,
        "a marquee was dragged around all 300 markups"
    ));
    report.note(format!("`{}`", banded.raw));
    if banded.get("selected") != Some("300") {
        return Ok(Some(format!(
            "a window marquee around 300 markups traced `{}`: all 300 selected was wanted.",
            banded.raw
        )));
    }

    let mark = session.trace()?.mark();
    pointer.key(session, None, "4", Some("ctrl+alt"))?;
    session.settle(40);
    let applied = step!(after(
        session,
        APPLIED,
        mark,
        "Ctrl+Alt+4 (Align left) was pressed"
    ));
    report.note(format!("`{}`", applied.raw));
    if applied.get("annots") != Some("0/285") {
        return Ok(Some(format!(
            "Align left over 300 markups, 285 right of the first column, traced `{}`: none \
             moved of 285 asked was wanted, since 285 moves are more than one undo step holds.",
            applied.raw
        )));
    }
    let refused = step!(after(session, REFUSED, mark, "Align left moved nothing"));
    report.note(format!("`{}`", refused.raw));
    let declined = step!(after(session, DECLINED, mark, "Align left was refused"));
    report.note(format!("`{}`", declined.raw));
    if declined.get("changes") != Some("285") || declined.get("limit") != Some("256") {
        return Ok(Some(format!(
            "the refusal recorded `{}`: 285 changes against a limit of 256 was wanted, so the \
             status line names the generic refusal or the wrong count.",
            declined.raw
        )));
    }
    session.settle(20);
    if driving::declared(&session.trace()?, ui_rect, DECLINE).is_none() {
        return Ok(Some(format!(
            "Align left was refused for being too large for one undo step and `{DECLINE}` was \
             not drawn: the operator is not told why nothing moved."
        )));
    }
    Ok(None)
}
