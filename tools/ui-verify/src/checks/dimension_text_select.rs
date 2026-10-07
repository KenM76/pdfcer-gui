//! `clicking_a_dimensions_text_selects_it` — a click on a ce dimension's value
//! text, away from its lines, selects the dimension.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_text_select.md`.

use super::dimdrive::{PAGE, click_page, run, ui_rect_event};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The selected linear ce dimension's text, window bounds.
const LABEL: &str = "canvas.dimension-label"; // ui-text-exempt: trace region name
const SELECTED: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
/// The fixture's dimension runs (100,150)→(300,150) with a 30 pt standoff,
/// whose side is the engine's; both candidates for its line, left of centre.
const LINE_CANDIDATES: [(f64, f64); 2] = [(130.0, 180.0), (130.0, 120.0)];
/// Blank paper, to clear the selection.
const BLANK: (f64, f64) = (30.0, 280.0);
/// How far inside the text's far edge the click lands, logical points.
const INSET: f32 = 2.0;

/// A click on a ce dimension's text selects it.
pub struct ClickingADimensionsTextSelectsIt;

impl Check for ClickingADimensionsTextSelectsIt {
    fn name(&self) -> &'static str {
        "clicking_a_dimensions_text_selects_it"
    }

    fn defect(&self) -> &'static str {
        "a click on a ce dimension's value text selects nothing: only its lines claim a click"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, "mode.review", "dim-text-select", drive)
    }
}

/// The last `annot-select` after `mark` names a ce dimension.
fn selected_after(session: &Session, mark: usize) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .last_after(SELECTED, mark)
        .filter(|l| l.get("kind") == Some("CeDimension"))
        .map(|l| l.raw.clone()))
}

/// Select the dimension by its line; the window y of the line click.
fn select_by_line(ctx: &CheckContext, session: &Session, pointer: &ScriptedPointer) -> Result<f32> {
    for at in LINE_CANDIDATES {
        let mark = session.trace()?.mark();
        click_page(ctx, session, pointer, at)?;
        if selected_after(session, mark)?.is_some() {
            let mapping =
                CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, PAGE, 0)?;
            return Ok(mapping.doc_to_window(DocPoint::new(0, at.0, at.1))?.y());
        }
    }
    Err(Error::new(format!(
        "a click on either side of the fixture's ce dimension at 30 pt selected nothing, so the \
         Select tool is not armed or the fixture changed. Trace: {}.",
        session.trace_path().display()
    )))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let line_y = select_by_line(ctx, session, pointer)?;
    let Some((label, _)) = declared_in(&session.trace()?, ui_rect_event(ctx)?, LABEL) else {
        return Err(Error::new(format!(
            "the selected ce dimension published no `{LABEL}` region. Trace: {}.",
            session.trace_path().display()
        )));
    };
    click_page(ctx, session, pointer, BLANK)?;
    // The text's edge farthest from the line, so a hit cannot be the line's.
    let centre = WindowPoint::centre_of(label);
    let y = if centre.y() < line_y {
        label.min.y + INSET
    } else {
        label.max.y - INSET
    };
    let at = WindowPoint::centre_of(LRect {
        min: Pt::new(centre.x(), y),
        max: Pt::new(centre.x(), y),
    });
    report.note(format!(
        "text {label:?}, line at y={line_y:.1}, clicking ({:.1}, {y:.1})",
        centre.x()
    ));
    let mark = session.trace()?.mark();
    pointer.click(session, at)?;
    session.settle(20);
    Ok(match selected_after(session, mark)? {
        Some(line) => {
            report.note(format!("`{line}`"));
            None
        }
        None => Some(format!(
            "a click on the ce dimension's text, {:.1} pt from its line, selected nothing \
             (no `{SELECTED} kind=CeDimension`). Trace: {}.",
            (y - line_y).abs(),
            session.trace_path().display()
        )),
    })
}
