//! Calibrating by two points on a `/UserUnit` page: the length the operator
//! states is the length the page's ce dimensions then read.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scale_user_unit.md`.

use super::dimdrive::{Fixture, click_on, press, press_on_tab, run_on, ui_rect_event};
use super::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// `fixtures/user-unit.pdf`: a 306 × 396 box at `/UserUnit 2`, framed by
/// `18 18 270 360 re`.
const USER_UNIT: Fixture = Fixture {
    file: "user-unit.pdf",
    method: "A page whose /UserUnit makes its box half its physical size; it is checked in \
             beside its PROVENANCE file.",
    page: PageGeometry {
        width_pt: 306.0,
        height_pt: 396.0,
    },
};
/// The frame's bottom edge, 270 box units: 540 pt at `/UserUnit 2`.
const FROM: (f64, f64) = (18.0, 18.0);
const TO: (f64, f64) = (288.0, 18.0);
const LINE_AT: (f64, f64) = (153.0, 90.0);
const SET_SCALE: &str = "ribbon.item.measure.set_scale";
const MEASURE_TAB: &str = "ribbon.tab.measure";
/// Mirrors `dialogs::scale`'s regions.
const CALIBRATE: &str = "scale.calibrate";
const REAL_LENGTH: &str = "scale.real_length";
const ACCEPT: &str = "dialog:set-scale.accept";
const TYPED: &str = "5 m";
const WANT_M: f64 = 5.0;
const TOLERANCE_M: f64 = 0.05;
const ADDED: &str = "dimension-added"; // ui-text-exempt: a trace event name, never displayed
const SHOWN: &str = "dimension-member-shown"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct ACalibrationOnAUserUnitPageReadsTheStatedLength;

impl Check for ACalibrationOnAUserUnitPageReadsTheStatedLength {
    fn name(&self) -> &'static str {
        "a_calibration_on_a_user_unit_page_reads_the_stated_length"
    }

    fn defect(&self) -> &'static str {
        "calibrating by two points on a /UserUnit page takes the line's box units as points, so \
         a line stated to be 5 m then reads 10 m on a /UserUnit 2 page"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &USER_UNIT,
            "mode.review,measure.linear",
            "scale-user-unit",
            drive,
        )
    }
}

fn pick(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: &[(f64, f64)],
) -> Result<()> {
    for &point in at {
        click_on(ctx, session, pointer, &USER_UNIT, 0, point)?;
    }
    session.settle(20);
    Ok(())
}

/// The metres a shown measurement reads, `"5.00 m"` → 5, with either marker.
fn metres(text: &str) -> Option<f64> {
    text.trim()
        .strip_suffix(" m")?
        .trim()
        .replace(',', ".")
        .parse()
        .ok()
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pick(ctx, session, pointer, &[FROM, TO, LINE_AT])?;
    let Some(added) = session
        .trace()?
        .last_after(ADDED, mark)
        .map(|l| l.raw.clone())
    else {
        return Ok(Some(format!(
            "three picks along the frame's bottom edge committed no ce dimension (no `{ADDED}`). \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("`{added}`"));
    press_on_tab(ctx, session, pointer, SET_SCALE, MEASURE_TAB)?;
    press(ctx, session, pointer, CALIBRATE)?;
    pick(ctx, session, pointer, &[FROM, TO])?;
    press(ctx, session, pointer, REAL_LENGTH)?;
    let vp =
        declared_in(&session.trace()?, ui_rect_event(ctx)?, REAL_LENGTH).and_then(|(_, vp)| vp);
    pointer.key(session, vp.as_deref(), "A", Some("ctrl"))?;
    pointer.type_text(session, vp.as_deref(), TYPED)?;
    session.settle(15);
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, ACCEPT)?;
    session.settle(30);
    let trace = session.trace()?;
    let text = trace
        .events(SHOWN)
        .filter(|l| l.lineno > mark)
        .last()
        .and_then(|l| l.get("text"))
        .map(str::to_owned)
        .ok_or_else(|| {
            Error::new(format!(
                "Accept on `{TYPED}` traced no `{SHOWN}` line, so the scale never reached the \
                 group. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
    report.note(format!(
        "after calibrating the edge as `{TYPED}` the ce dimension reads `{text}`"
    ));
    let Some(read) = metres(&text) else {
        return Ok(Some(format!(
            "the edge was stated as `{TYPED}` and its ce dimension reads `{text}`, not metres."
        )));
    };
    Ok(((read - WANT_M).abs() > TOLERANCE_M).then(|| {
        format!(
            "the edge was stated as `{TYPED}` and its ce dimension reads `{text}`: the reference \
             line's box units were taken as points, ignoring /UserUnit. Trace: {}.",
            session.trace_path().display()
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metres_reads_either_marker_and_nothing_else() {
        assert_eq!(metres("5.00 m"), Some(5.0));
        assert_eq!(metres("10,00 m"), Some(10.0));
        assert_eq!(metres("5.00 mm"), None);
        assert_eq!(metres("540.00 pt"), None);
    }
}
