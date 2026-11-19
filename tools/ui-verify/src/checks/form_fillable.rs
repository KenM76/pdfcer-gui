//! `use_these_pages_keeps_the_fields_fillable` — Form preview ▸ Use these
//! pages, with *Keep the fields fillable* left at its default, writes the
//! form's fields as fillable fields, and one of them then fills in on the
//! canvas like any form field.
//!
//! Opens a copy of `fixtures/dynamic-xfa.pdf`, whose computed page has three
//! fields (Tenant, Rent, Deposit), presses Use these pages without touching
//! the switch, and requires `fillable=true fields_written=3` on the engine's
//! outcome. It then reads the canvas's census of fillable boxes written after
//! that edit, clicks the first text box through the canvas mapping, replaces
//! its value and presses Enter, and requires the form-fill route's own
//! `form-fill-text` line after the Enter.

use crate::checks::form_pages::{launch_form, press_use};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;
use crate::trace::Trace;

const APPLIED: &str = "form-pages-applied";
/// The canvas census of one fillable box, in canvas space.
const BOX_LINE: &str = "form-box";
const FOCUS: &str = "form-focus";
/// The form-fill route's line for a committed text fill.
const FILL_LINE: &str = "form-fill-text";
const TYPED: &str = "Ann";
/// The fixture's computed page: US letter, upright.
const PAGE: PageGeometry = PageGeometry {
    width_pt: 612.0,
    height_pt: 792.0,
};

/// See the module documentation.
pub struct UseThesePagesKeepsTheFieldsFillable;

impl Check for UseThesePagesKeepsTheFieldsFillable {
    fn name(&self) -> &'static str {
        "use_these_pages_keeps_the_fields_fillable"
    }

    fn defect(&self) -> &'static str {
        "Use these pages draws a dynamic form's values into the page by default, so the form can \
         no longer be filled in, or the fields it writes cannot be filled in on the canvas"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_form(ctx, &mut report, "form_fillable").and_then(
            |(session, pointer, ui_rect)| {
                let outcome = drive(ctx, &mut report, &session, &pointer, ui_rect);
                let parked = pointer.gone(&session);
                match outcome? {
                    Some(failure) => Ok(Some(failure)),
                    None => parked.map(|_| None),
                }
            },
        );
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// A fillable box from the census: its page, field name and canvas centre.
struct Fillable {
    page: usize,
    field: String,
    centre: (f64, f64),
}

/// The text boxes the canvas published after trace line `after`.
fn fillable_after(trace: &Trace, after: usize) -> Vec<Fillable> {
    trace
        .events(BOX_LINE)
        .filter(|l| l.lineno > after && l.get("kind") == Some("text"))
        .filter_map(|l| {
            let page = l.get("page")?.parse().ok()?;
            let field = l.get("field")?.to_owned();
            let raw = l.get("rect")?;
            let (min, size) = raw.split_once(")+(")?;
            let (x, y) = min.trim_start_matches('(').split_once(',')?;
            let (w, h) = size.trim_end_matches(')').split_once(',')?;
            let (x, y): (f64, f64) = (x.trim().parse().ok()?, y.trim().parse().ok()?);
            let (w, h): (f64, f64) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
            Some(Fillable {
                page,
                field,
                centre: (x + w / 2.0, y + h / 2.0),
            })
        })
        .collect()
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &'static str,
) -> Result<Option<String>> {
    if let Some(failure) = press_use(ctx, report, ui_rect, session, pointer)? {
        return Ok(Some(failure));
    }
    session.settle(30);
    let trace = session.trace()?;
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "★★★ no `{APPLIED}` line after Use these pages."
        )));
    };
    if applied.get("fillable") != Some("true") || applied.get("fields_written") != Some("3") {
        return Ok(Some(format!(
            "★★★ `{}` must carry fillable=true fields_written=3: the switch is on by default \
             and the fixture's page has three fields.",
            applied.raw
        )));
    }
    let boxes = fillable_after(&trace, applied.lineno);
    let Some(first) = boxes.first() else {
        return Ok(Some(format!(
            "★★★ the engine wrote 3 fillable fields and the canvas published no `{BOX_LINE} \
             kind=text` line after it: the fields cannot be reached. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "{} fillable text box(es); aiming at {:?}",
        boxes.len(),
        first.field
    ));
    fill_on_canvas(ctx, session, pointer, first)
}

/// Click `target` on the canvas, replace its value and press Enter; judge the
/// focus and the form-fill route's line.
fn fill_on_canvas(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    target: &Fillable,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, PAGE, target.page)?;
    // The census is canvas space; `doc_to_window` takes PDF space.
    let point = mapping.doc_to_window(DocPoint::new(
        target.page,
        target.centre.0,
        PAGE.height_pt - target.centre.1,
    ))?;
    pointer.click(session, point)?;
    session.settle(25);
    let trace = session.trace()?;
    if trace.last(FOCUS).is_none() {
        return Err(Error::new(format!(
            "clicking the census's text box focused nothing: no `{FOCUS}` line. The fill \
             below needs a focused field; `tab_navigation` owns canvas focus. Trace: {}.",
            session.trace_path().display()
        )));
    }
    pointer.key(session, None, "A", Some("ctrl"))?;
    pointer.type_text(session, None, TYPED)?;
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Enter", None)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(fill) = trace.last_after(FILL_LINE, mark) else {
        return Ok(Some(format!(
            "★★★ typing {TYPED} into the written field {:?} and pressing Enter traced no \
             `{FILL_LINE}` line: the field the form became cannot be filled in. Trace: {}.",
            target.field,
            session.trace_path().display()
        )));
    };
    if !fill.raw.contains("commands=1") {
        return Ok(Some(format!(
            "★★★ `{}` must carry commands=1: one fill is one undo step.",
            fill.raw
        )));
    }
    Ok(None)
}
