//! The Align check's Grid and Circular phases: from the three boxes where
//! they started, **Grid › Arrange** lays them in the default 2 × 2 table as
//! one move, and **Circular › Arrange** with the first selected object as
//! the ellipse refuses (a box is not an ellipse); on a Parameterized arc
//! with *Rotate objects* it turns and places all three as one transform.
//!
//! Every expected number is worked by hand from the fixture's boxes and
//! Inkscape's rules, in `ALIGN_AND_DISTRIBUTE.md`'s terms; none was read
//! back from a run.

use crate::checks::driving::declared;
use crate::error::Result;
use crate::input::Driver;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::sys::vk;

const PANEL: &str = "align.panel";
const GRID_TAB: &str = "align.tab.grid";
const CIRCULAR_TAB: &str = "align.tab.circular";
const GRID_ARRANGE: &str = "grid.arrange";
const CIRCLE_ARRANGE: &str = "circular.arrange";
const PARAMETERIZED: &str = "circular.source.0";
const ROTATE: &str = "circular.rotate";
const MOVED_EVENT: &str = "move-each-applied"; // ui-text-exempt: a trace event name, never displayed
const TRANSFORMED_EVENT: &str = "transform-each-applied"; // ui-text-exempt: a trace event name, never displayed
const DECLINED_EVENT: &str = "align-declined"; // ui-text-exempt: a trace event name, never displayed

/// Grid, defaults (2 × 2, centred in cells, 15 pt gaps), in PDF points.
/// Canvas boxes (y down, sheet 792 tall): 0 = x 100–160, y 152–192;
/// 1 = x 250–330, y 242–292; 2 = x 180–220, y 372–442. Reading order 0, 1,
/// 2; columns 60 and 80 wide, rows 50 and 70 tall; origin (100, 152).
/// Box 0 → (100, 157); box 1 → (175, 152); box 2 → (110, 217).
const EXPECTED_GRID: &str = "0:0.0,-5.0;1:-75.0,90.0;2:-70.0,155.0";

/// Circular, Parameterized default (centre (0, 0), radius 100, 0°–180°),
/// object centres, rotated: box 0 takes 0°, target canvas (100, 0), and
/// turns a quarter clockwise on screen. In page space that is rotate(−90°)
/// carrying its centre (130, 620) to (100, 792): `[0 −1 1 0 −520 922]`.
const EXPECTED_BOX0: [f64; 6] = [0.0, -1.0, 1.0, 0.0, -520.0, 922.0];

/// Scroll the panel toward `wanted` (`notch` −1 down, +1 up) until it is
/// on screen.
fn reach(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    wanted: &str,
    notch: i32,
) -> Result<Option<crate::geom::LRect>> {
    for _ in 0..16 {
        let trace = session.trace()?;
        if let Some(rect) = declared(&trace, ui_rect, wanted) {
            return Ok(Some(rect));
        }
        let Some(panel) = declared(&trace, ui_rect, PANEL) else {
            return Ok(None);
        };
        driver.scroll_at(session.frame()?.declared_center(panel), notch)?;
        session.settle(12);
    }
    Ok(None)
}

/// Click `wanted`, scrolling to it first.
pub(super) fn press(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    wanted: &str,
    notch: i32,
) -> Result<bool> {
    let Some(rect) = reach(session, driver, ui_rect, wanted, notch)? else {
        return Ok(false);
    };
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(24);
    Ok(true)
}

/// Run the phases. The boxes must be at their fixture positions, selected
/// 0, 1, 2.
pub(super) fn drive(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let trace_path = session.trace_path().display().to_string();

    // --- Grid ---------------------------------------------------------------
    if !press(session, driver, ui_rect, GRID_TAB, 1)? {
        return Ok(Some(format!(
            "★ THE PANEL PUBLISHED NO `{GRID_TAB}`: the Grid tab is missing. Trace: {trace_path}."
        )));
    }
    let mark = session.trace()?.mark();
    if !press(session, driver, ui_rect, GRID_ARRANGE, -1)? {
        return Ok(Some(format!(
            "★ THE GRID TAB PUBLISHED NO `{GRID_ARRANGE}`. Trace: {trace_path}."
        )));
    }
    let trace = session.trace()?;
    let Some(applied) = trace.last_after(MOVED_EVENT, mark) else {
        return Ok(Some(format!(
            "★ Grid › Arrange was clicked and no `{MOVED_EVENT}` followed. Trace: {trace_path}."
        )));
    };
    report.note(format!("★ `{}`", applied.raw));
    let asked = applied.get("moves").unwrap_or("");
    if applied.get("gesture") != Some("grid") || asked != EXPECTED_GRID {
        return Ok(Some(format!(
            "★★ GRID MOVED `moves={asked}`, expected `{EXPECTED_GRID}` — the default 2 × 2 \
             table, 15 pt gaps, each box centred in its cell. `{}`",
            applied.raw
        )));
    }
    report.note("★★ Grid laid the three boxes in a 2 × 2 table");
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(24);

    // --- Circular: a box is not an ellipse ----------------------------------
    if !press(session, driver, ui_rect, CIRCULAR_TAB, 1)? {
        return Ok(Some(format!(
            "★ THE PANEL PUBLISHED NO `{CIRCULAR_TAB}`. Trace: {trace_path}."
        )));
    }
    let mark = session.trace()?.mark();
    if !press(session, driver, ui_rect, CIRCLE_ARRANGE, -1)? {
        return Ok(Some(format!(
            "★ THE CIRCULAR TAB PUBLISHED NO `{CIRCLE_ARRANGE}`. Trace: {trace_path}."
        )));
    }
    let trace = session.trace()?;
    if trace.last_after(MOVED_EVENT, mark).is_some()
        || trace.last_after(TRANSFORMED_EVENT, mark).is_some()
    {
        return Ok(Some(
            "★★ CIRCULAR ARRANGED ON THE FIRST SELECTED OBJECT, A RECTANGLE: it must refuse, \
             since a box is not an ellipse."
                .to_owned(),
        ));
    }
    let refused = trace
        .last_after(DECLINED_EVENT, mark)
        .filter(|d| d.get("reason") == Some("not-an-ellipse"));
    let Some(refused) = refused else {
        return Ok(Some(format!(
            "★ Circular on the first selected rectangle wrote no \
             `{DECLINED_EVENT} … reason=not-an-ellipse`. Trace: {trace_path}."
        )));
    };
    report.note(format!("★ `{}`", refused.raw));

    // --- Circular: Parameterized, rotated -----------------------------------
    for (region, notch) in [(PARAMETERIZED, 1), (ROTATE, -1)] {
        if !press(session, driver, ui_rect, region, notch)? {
            return Ok(Some(format!(
                "★ THE CIRCULAR TAB PUBLISHED NO `{region}`. Trace: {trace_path}."
            )));
        }
    }
    let mark = session.trace()?.mark();
    if !press(session, driver, ui_rect, CIRCLE_ARRANGE, -1)? {
        return Ok(Some(format!(
            "`{CIRCLE_ARRANGE}` vanished. Trace: {trace_path}."
        )));
    }
    let trace = session.trace()?;
    let Some(applied) = trace.last_after(TRANSFORMED_EVENT, mark) else {
        return Ok(Some(format!(
            "★ Circular with Rotate objects wrote no `{TRANSFORMED_EVENT}`: it moved without \
             turning, or planned nothing. Trace: {trace_path}."
        )));
    };
    report.note(format!("★ `{}`", applied.raw));
    let counts = (applied.get_usize("transformed"), applied.get_usize("of"));
    if counts != (Some(3), Some(3)) {
        return Ok(Some(format!(
            "★★ CIRCULAR TRANSFORMED {counts:?}, not 3 of 3. `{}`",
            applied.raw
        )));
    }
    let box0: Option<Vec<f64>> = applied.get("m").and_then(|m| {
        m.split(';')
            .find_map(|item| item.strip_prefix("0:"))
            .map(|nums| nums.split(',').filter_map(|v| v.parse().ok()).collect())
    });
    let close = box0.as_ref().is_some_and(|got| {
        got.len() == 6
            && got
                .iter()
                .zip(EXPECTED_BOX0)
                .all(|(g, e)| (g - e).abs() < 0.05)
    });
    if !close {
        return Ok(Some(format!(
            "★★ BOX 0's MATRIX IS {box0:?}, expected {EXPECTED_BOX0:?} — a quarter turn \
             clockwise on screen carrying its centre to the arc's 0° point. A sign error in the \
             canvas-to-page conjugation turns it the wrong way. `{}`",
            applied.raw
        )));
    }
    report.note("★★ Circular turned and placed all three as one transform");
    Ok(None)
}
