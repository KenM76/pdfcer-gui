//! `a_part_inside_a_placed_drawing_can_be_deleted` — one subpath, one anchor
//! and one text line inside a form XObject are each selected on the canvas and
//! deleted with the Delete key, and the engine's in-form verb applies.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/form_part_delete.md`.

use crate::checks::driving;
use crate::checks::form_node_move::{enter_leaf_at, run_body};
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "../../fixtures/form-parts.pdf";
const SELECTION: &str = "canvas-selection"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "canvas-delete-declined"; // ui-text-exempt: a trace event name, never displayed
/// The second drawn anchor mark: a middle anchor of the four-anchor polyline.
/// Mirrors `canvas::overlay`'s region names.
const MIDDLE_ANCHOR: &str = "canvas.anchor.1";
/// The selected anchor. Mirrors `canvas::overlay::SELECTED_ANCHOR_REGION`.
const SELECTED_ANCHOR: &str = "canvas.selected-anchor";

/// Page-space aim points on `form-parts.pdf` (the form sits at (40, 40)).
/// The upper bar of leaf 0.
const ON_THE_UPPER_BAR: (f64, f64) = (100.0, 220.0);
/// Mid-segment of the polyline's first leg, leaf 1.
const ON_THE_POLYLINE: (f64, f64) = (240.0, 120.0);
/// Inside "Second line", leaf 2's last line.
const ON_THE_SECOND_LINE: (f64, f64) = (80.0, 80.0);

/// Which part of the form a run deletes.
#[derive(Clone, Copy)]
enum Part {
    Subpath,
    Node,
    TextLine,
}

impl Part {
    const fn verb(self) -> &'static str {
        // ui-text-exempt: trace event names, never displayed
        match self {
            Self::Subpath => "delete-subpath-in-form",
            Self::Node => "delete-node-in-form",
            Self::TextLine => "delete-text-line-in-form",
        }
    }

    const fn aim(self) -> (f64, f64) {
        match self {
            Self::Subpath => ON_THE_UPPER_BAR,
            Self::Node => ON_THE_POLYLINE,
            Self::TextLine => ON_THE_SECOND_LINE,
        }
    }
}

macro_rules! part_check {
    ($ty:ident, $name:literal, $part:expr, $body:ident) => {
        pub struct $ty;

        impl Check for $ty {
            fn name(&self) -> &'static str {
                $name
            }

            fn defect(&self) -> &'static str {
                "a part inside a placed drawing (a markup made part of the page) is selected and \
                 Delete refuses it, or reports it deleted and the engine changed nothing"
            }

            fn run(&self, ctx: &CheckContext) -> CheckReport {
                let mut report = CheckReport::new(self.name(), self.defect());
                match run_body(ctx, &mut report, FIXTURE, $name, $body) {
                    Ok(Some(failure)) => report.fail(failure),
                    Ok(None) => report.pass(),
                    Err(why) => report.from_error(&why),
                }
            }
        }

        fn $body(
            ctx: &CheckContext,
            report: &mut CheckReport,
            session: &Session,
            pointer: &ScriptedPointer,
            page: PageGeometry,
        ) -> Result<Option<String>> {
            select_and_delete(ctx, report, session, pointer, page, $part)
        }
    };
}

part_check!(
    ASubpathInsideAPlacedDrawingCanBeDeleted,
    "a_subpath_inside_a_placed_drawing_can_be_deleted",
    Part::Subpath,
    delete_subpath
);
part_check!(
    AnAnchorInsideAPlacedDrawingCanBeDeleted,
    "an_anchor_inside_a_placed_drawing_can_be_deleted",
    Part::Node,
    delete_node
);
part_check!(
    ATextLineInsideAPlacedDrawingCanBeDeleted,
    "a_text_line_inside_a_placed_drawing_can_be_deleted",
    Part::TextLine,
    delete_text_line
);

/// Descend to the Part rung (and, for `Part::Node`, the Node rung), press
/// Delete, and read the engine's answer.
fn select_and_delete(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    page: PageGeometry,
    part: Part,
) -> Result<Option<String>> {
    let (ui_rect, at) = enter_leaf_at(ctx, session, pointer, page, part.aim())?;
    // A path's parts are a double-click deeper; a text block's lines are its
    // chunks, which a single click on the selected block picks (a double-click
    // there opens the text for editing instead).
    if matches!(part, Part::TextLine) {
        pointer.click(session, at)?;
    } else {
        pointer.double_click(session, at)?;
    }
    session.settle(30);
    if matches!(part, Part::Node) {
        let trace = session.trace()?;
        let anchor = driving::declared(&trace, ui_rect, MIDDLE_ANCHOR).ok_or_else(|| {
            Error::new(format!(
                "no `{MIDDLE_ANCHOR}` after descending into the polyline. Trace: {}.",
                session.trace_path().display()
            ))
        })?;
        pointer.double_click(session, WindowPoint::centre_of(anchor))?;
        session.settle(20);
        if driving::declared(&session.trace()?, ui_rect, SELECTED_ANCHOR).is_none() {
            return Err(Error::new(format!(
                "the double-click on a middle anchor selected none. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }
    let trace = session.trace()?;
    report.note(format!(
        "selected: `{}`",
        trace.last(SELECTION).map_or("none", |l| l.raw.as_str())
    ));
    let mark = trace.mark();
    pointer.key(session, None, "Delete", None)?;
    session.settle(40);

    let trace = session.trace()?;
    if let Some(refused) = trace.last_after(DECLINED, mark) {
        return Ok(Some(format!(
            "DELETE WAS REFUSED: `{}`. The engine has `{}`'s verb; \
             `canvas::deleting` must address the part by its leaf. Trace: {}.",
            refused.raw,
            part.verb(),
            session.trace_path().display()
        )));
    }
    let verb = part.verb();
    let Some(applied) = trace.last_after(verb, mark) else {
        let engine = trace.last_after(&format!("{verb}-refused"), mark);
        return Ok(Some(format!(
            "Delete was pressed and no `{verb}` line followed (engine refusal: {}). Trace: {}.",
            engine.map_or("none", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    if applied.get_usize("n").is_none_or(|n| n == 0) {
        return Ok(Some(format!(
            "`{}` carries no operand count. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("the engine applied it: `{}`", applied.raw));
    Ok(None)
}
