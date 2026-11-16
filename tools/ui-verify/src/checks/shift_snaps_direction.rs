//! `shift_snaps_a_drawn_direction` — with Markup ▸ Polygon armed, a vertex
//! clicked with Shift held lands on a multiple of 15° from the previous one,
//! though the pointer was aimed 7° and 52° off it, and the status row's lock is
//! announced (`constrain lock=Direction`).
//!
//! Oracle: the app trace's `markup-vertex n=… x=… y=…` lines (PDF user space)
//! and `constrain lock=…`. An unsnapped click lands at the aimed 7° and 52°,
//! neither of which is within tolerance of a step.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/shift_snaps_direction.md`.

use super::dimdrive::{PAGE, run};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

// ui-text-exempt: command ids and trace event names, never displayed.
const INVOKE: &str = "mode.review,markup.polygon";
const VERTEX: &str = "markup-vertex";
const LOCK: &str = "constrain";

/// The first corner, in page points, clear of the fixture's ce dimension.
const START: (f64, f64) = (80.0, 60.0);
/// How far each aimed segment reaches, in page points.
const REACH: f64 = 110.0;
/// The bearings aimed at, in degrees: each is off a 15° step by 7°.
const AIMED: [f64; 2] = [7.0, 52.0];
/// How close a landed bearing must be to a step, in degrees: the trace's two
/// decimals over a 100 pt segment resolve about 0.01°.
const TOLERANCE_DEG: f64 = 0.2;

/// See the module documentation.
pub struct ShiftSnapsADrawnDirection;

impl Check for ShiftSnapsADrawnDirection {
    fn name(&self) -> &'static str {
        "shift_snaps_a_drawn_direction"
    }

    fn defect(&self) -> &'static str {
        "holding Shift while placing the next corner of a polygon or polyline leaves it where \
         the pointer is, instead of on the nearest 15° direction from the previous corner as \
         rotating does"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, INVOKE, "shift-snaps-direction", body)
    }
}

/// Click `at` (page points) on page 1, holding `mods` when given.
fn click(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
    mods: Option<&str>,
) -> Result<()> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PAGE, 0)?;
    let point = mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?;
    pointer.hover(session, point)?;
    session.settle(10);
    match mods {
        Some(m) => pointer.click_with(session, point, m)?,
        None => pointer.click(session, point)?,
    };
    session.settle(20);
    Ok(())
}

/// The `(x, y)` of every `markup-vertex` line after `mark`, in order.
fn vertices(session: &Session, mark: usize) -> Result<Vec<(f64, f64)>> {
    Ok(session
        .trace()?
        .events(VERTEX)
        .filter(|l| l.lineno > mark)
        .filter_map(|l| Some((l.get("x")?.parse().ok()?, l.get("y")?.parse().ok()?)))
        .collect())
}

/// The bearing of `b` from `a` in degrees, and its distance from a 15° step.
fn off_step(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let deg = (b.1 - a.1).atan2(b.0 - a.0).to_degrees();
    let off = (deg - (deg / 15.0).round() * 15.0).abs();
    (deg, off)
}

fn body(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let mark = session.trace()?.mark();
    click(ctx, session, pointer, START, None)?;
    let mut from = START;
    for (i, aimed) in AIMED.into_iter().enumerate() {
        let r = aimed.to_radians();
        let at = (from.0 + REACH * r.cos(), from.1 + REACH * r.sin());
        click(ctx, session, pointer, at, Some("shift"))?;
        let placed = vertices(session, mark)?;
        // The count, not just the last line: a Shift-click that placed nothing
        // leaves the previous corner last, whose bearing from itself reads 0°.
        let Some(&landed) = placed.get(i + 1).filter(|_| placed.len() == i + 2) else {
            return Err(Error::new(format!(
                "{} `{VERTEX}` lines after {} clicks: the Polygon tool did not take each click \
                 once, so the direction cannot be measured. Trace: {path}.",
                placed.len(),
                i + 2
            )));
        };
        let (deg, off) = off_step(from, landed);
        report.note(format!(
            "aimed {aimed}°, landed {deg:.2}° ({off:.2}° off a step)"
        ));
        if off > TOLERANCE_DEG {
            return Ok(Some(format!(
                "a Shift-click aimed {aimed}° from the previous corner landed at {deg:.2}°, \
                 {off:.2}° off the nearest 15° step: the direction lock did not reach the vertex \
                 (`markup::vertex::aim` from `canvas::clicking`). Trace: {path}."
            )));
        }
        from = landed;
    }
    let locked = session
        .trace()?
        .events(LOCK)
        .any(|l| l.lineno > mark && l.get("lock") == Some("Direction"));
    Ok((!locked).then(|| {
        format!(
            "the corners snapped and no `{LOCK} lock=Direction` line was traced: the lock is not \
             announced on the status row. Trace: {path}."
        )
    }))
}
