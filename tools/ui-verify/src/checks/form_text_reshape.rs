//! `a_text_line_inside_a_placed_drawing_can_be_merged`,
//! `a_text_inside_a_placed_drawing_can_be_split_into_lines` and
//! `a_text_line_inside_a_placed_drawing_takes_a_typed_width` — the three text
//! reshapes, driven on a text object inside a form XObject; each must reach
//! the engine's in-form verb.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/form_text_reshape.md`.

use crate::checks::driving;
use crate::checks::form_node_move::{enter_leaf_at, run_body_with};
use crate::checks::stroke_style::type_into;
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "../../fixtures/form-text-runs.pdf";
// ui-text-exempt: trace event names and region ids, never displayed
const SELECTION: &str = "canvas-selection";
const DECLINE: &str = "canvas-decline-recorded";
const MERGED: &str = "merge-text-runs-applied";
const SPLIT: &str = "split-text-lines-applied";
const SPLIT_DECLINED: &str = "split-text-lines-declined";
const WIDTH: &str = "text-run-width";
const MERGE_ROW: &str = "menu.item.canvas.object.format.merge_text_runs";
const SPLIT_ROW: &str = "menu.item.canvas.object.format.split_text_lines";
const WIDTH_FIELD: &str = "properties.text.run-width";
const PROPERTIES: &str = "file.properties";

/// Page-space aim points on `form-text-runs.pdf` (the form sits at (40, 40)).
/// Inside "Two", the first of line 0's two show operators.
const ON_THE_FIRST_LINE: (f64, f64) = (75.0, 104.0);
/// Inside "Second line", line 1's one show operator.
const ON_THE_SECOND_LINE: (f64, f64) = (80.0, 80.0);

macro_rules! reshape_check {
    ($ty:ident, $name:literal, $defect:literal, $invoke:expr, $body:ident) => {
        pub struct $ty;

        impl Check for $ty {
            fn name(&self) -> &'static str {
                $name
            }

            fn defect(&self) -> &'static str {
                $defect
            }

            fn run(&self, ctx: &CheckContext) -> CheckReport {
                let mut report = CheckReport::new(self.name(), self.defect());
                match run_body_with(ctx, &mut report, FIXTURE, $name, $invoke, $body) {
                    Ok(Some(failure)) => report.fail(failure),
                    Ok(None) => report.pass(),
                    Err(why) => report.from_error(&why),
                }
            }
        }
    };
}

reshape_check!(
    ATextLineInsideAPlacedDrawingCanBeMerged,
    "a_text_line_inside_a_placed_drawing_can_be_merged",
    "Merge text runs on a line inside a placed drawing is absent, refused, or edits a page object",
    None,
    merge
);
reshape_check!(
    ATextInsideAPlacedDrawingCanBeSplitIntoLines,
    "a_text_inside_a_placed_drawing_can_be_split_into_lines",
    "Split into lines on a text inside a placed drawing is absent, refused, or edits a page object",
    None,
    split
);
reshape_check!(
    ATextLineInsideAPlacedDrawingTakesATypedWidth,
    "a_text_line_inside_a_placed_drawing_takes_a_typed_width",
    "a width typed for a line inside a placed drawing is greyed, refused, or edits a page object",
    Some(PROPERTIES),
    width
);

/// Enter the text leaf, click line 0 (its two runs), pick Merge text runs
/// from the canvas menu; the engine must merge two runs inside the form.
fn merge(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    let (ui_rect, at) = enter_leaf_at(ctx, session, pointer, page, ON_THE_FIRST_LINE)?;
    pointer.click(session, at)?;
    session.settle(30);
    let mark = selected(report, session)?;
    pick_from_menu(session, pointer, ui_rect, at, MERGE_ROW)?;
    let line = match in_form_line(session, mark, MERGED, "Merge text runs was picked")? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    if line.get_usize("merged") != Some(2) {
        return Ok(Some(format!(
            "the merge did not join line 0's two show operators: `{}`.",
            line.raw
        )));
    }
    report.note(format!("the engine merged them: `{}`", line.raw));
    Ok(None)
}

/// Enter the text leaf (whole text selected) and pick Split into lines; the
/// engine must cut the two-line text into two pieces inside the form.
fn split(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    let (ui_rect, at) = enter_leaf_at(ctx, session, pointer, page, ON_THE_SECOND_LINE)?;
    let mark = selected(report, session)?;
    pick_from_menu(session, pointer, ui_rect, at, SPLIT_ROW)?;
    let line = match in_form_line(session, mark, SPLIT, "Split into lines was picked")? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    if line.get_usize("pieces") != Some(2) {
        return Ok(Some(format!(
            "the split did not make two pieces of a two-line text: `{}`.",
            line.raw
        )));
    }
    report.note(format!("the engine split it: `{}`", line.raw));
    Ok(None)
}

/// Enter the text leaf, click line 1 (one run), type 150 pt into the
/// Properties width field; the engine must fit the run inside the form.
fn width(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
) -> Result<Option<String>> {
    let (_, at) = enter_leaf_at(ctx, session, pointer, page, ON_THE_SECOND_LINE)?;
    pointer.click(session, at)?;
    session.settle(30);
    let mark = selected(report, session)?;
    type_into(ctx, session, pointer, WIDTH_FIELD, "150")?;
    let line = match in_form_line(session, mark, WIDTH, "150 pt was typed for the line")? {
        Ok(line) => line,
        Err(failure) => return Ok(Some(failure)),
    };
    if line.get("detail") != Some("applied") {
        return Ok(Some(format!("the width was not applied: `{}`.", line.raw)));
    }
    report.note(format!("the engine fitted it: `{}`", line.raw));
    Ok(None)
}

/// Right-click `at` and click the menu row `row`; an absent row is an error
/// naming the rows that were offered.
fn pick_from_menu(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    at: WindowPoint,
    row: &str,
) -> Result<()> {
    pointer.right_click(session, at)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(rect) = driving::declared(&trace, ui_rect, row) else {
        return Err(Error::new(format!(
            "the canvas menu offered no `{row}`. Rows present: {}. Trace: {}.",
            driving::declared_names(&trace, ui_rect, "menu.item.").join(", "),
            session.trace_path().display()
        )));
    };
    pointer.click(session, WindowPoint::centre_of(rect))?;
    session.settle(40);
    Ok(())
}

/// The last `event` line after `mark`, which must name an in-form edit; `Err`
/// is the failure sentence quoting what the trace held instead.
fn in_form_line(
    session: &Session,
    mark: usize,
    event: &str,
    asked: &str,
) -> Result<std::result::Result<crate::trace::TraceLine, String>> {
    let trace = session.trace()?;
    let Some(line) = trace.last_after(event, mark) else {
        let decline = trace
            .last_after(SPLIT_DECLINED, mark)
            .or_else(|| trace.last_after(DECLINE, mark))
            .map_or("none", |l| l.raw.as_str());
        return Ok(Err(format!(
            "{asked} and no `{event}` line followed. Decline: {decline}. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if line.get("in_form") != Some("true") {
        return Ok(Err(format!(
            "{asked} and the edit did not go to the form leaf: `{}`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    Ok(Ok(line.clone()))
}

/// Note the selection and return the trace mark the edit is read after.
fn selected(report: &mut CheckReport, session: &Session) -> Result<usize> {
    let trace = session.trace()?;
    report.note(format!(
        "selected: `{}`",
        trace.last(SELECTION).map_or("none", |l| l.raw.as_str())
    ));
    Ok(trace.mark())
}
