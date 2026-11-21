//! `field_highlight` — **View ▸ Display ▸ Highlight fields tints every
//! fillable field on the canvas, undrawn ones included, and turning it off
//! takes the tint away**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_highlight.md`.

use super::dimdrive::launch_path;
use super::print_shop::{VIEW_TAB, await_line, press};
use super::undrawn_fields::{TextBox, capture_rect, text_boxes};
use crate::checks::driving;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::{Error, Result};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review"; // ui-text-exempt: a command id
const FIXTURE: &str = "need-appearances-fields.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/need-appearances-fields.PROVENANCE.py`.";
const ITEM: &str = "ribbon.item.view.highlight_fields";
const LINE: &str = "canvas-field-highlight"; // ui-text-exempt: a trace event name
/// Fewest levels the mean blue-over-red of a field's pixels must rise by.
const MIN_RISE: f64 = 8.0;
/// Most levels it may differ by once the tint is turned off again.
const MAX_DRIFT: f64 = 2.0;

/// See the module documentation.
pub struct HighlightFieldsTintsEveryField;

impl Check for HighlightFieldsTintsEveryField {
    fn name(&self) -> &'static str {
        "highlight_fields_tints_every_field"
    }

    fn defect(&self) -> &'static str {
        "View ▸ Display ▸ Highlight fields is missing, never reaches the page render, leaves a \
         field the file stores no drawing for untinted, or leaves the tint behind once it is \
         turned off"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let doc = ctx.out("field_highlight.pdf");
    let source = driving::repo_fixture(FIXTURE, METHOD)?;
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("no page size in {}.", source.display())))?;
    let (session, pointer) = launch_path(ctx, report, doc, INVOKE, "field_highlight", &[])?;
    report.artifact(session.trace_path().to_path_buf());
    let outcome = toggle(ctx, report, (&session, &pointer), page);
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Off, on, off again: the fields' blueness rises with the tint and returns.
fn toggle(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
) -> Result<Option<String>> {
    session.settle(45);
    let boxes = text_boxes(&session.trace()?);
    if boxes.len() < 3 {
        return Err(Error::new(format!(
            "{} text field box(es) traced of the fixture's three.",
            boxes.len()
        )));
    }
    let before = blueness(ctx, (session, pointer), page, &boxes, "off")?;
    report.note(format!("off: {before:.1?}"));
    press(session, pointer, VIEW_TAB)?;
    if let Some(failure) = flip(session, pointer, "1")? {
        return Ok(Some(failure));
    }
    let on = blueness(ctx, (session, pointer), page, &boxes, "on")?;
    report.note(format!("on: {on:.1?}"));
    for ((b, was), now) in boxes.iter().zip(&before).zip(&on) {
        if now - was < MIN_RISE {
            return Ok(Some(format!(
                "★★★ {} is not tinted: its blue-over-red went from {was:.1} to {now:.1}, under \
                 the {MIN_RISE} owed.",
                b.field
            )));
        }
    }
    if let Some(failure) = flip(session, pointer, "0")? {
        return Ok(Some(failure));
    }
    let after = blueness(ctx, (session, pointer), page, &boxes, "off-again")?;
    report.note(format!("off again: {after:.1?}"));
    for ((b, was), now) in boxes.iter().zip(&before).zip(&after) {
        if (now - was).abs() > MAX_DRIFT {
            return Ok(Some(format!(
                "★★★ turning the tint off left {} at {now:.1} against {was:.1} before.",
                b.field
            )));
        }
    }
    Ok(None)
}

/// Press the toggle and wait for a texture drawn in mode `mode`.
fn flip(session: &Session, pointer: &ScriptedPointer, mode: &str) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press(session, pointer, ITEM)?;
    let seen = await_line(session, LINE, mark, |l| {
        l.get("mode") == Some(mode) && l.get("drawn") == Some(mode)
    })?;
    if seen.is_none() {
        let last = session.trace()?.last(LINE).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "★★★ after pressing `{ITEM}` no texture was drawn with the tint {}: last `{LINE}` \
             line {last:?}.",
            if mode == "1" { "on" } else { "off" }
        )));
    }
    session.settle(20);
    Ok(None)
}

/// Each field's mean blue-minus-red over its box, from one screenshot.
fn blueness(
    ctx: &CheckContext,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
    boxes: &[TextBox],
    stage: &str,
) -> Result<Vec<f64>> {
    let png = ctx.out(&format!("field-highlight-{stage}.png"));
    pointer.screenshot(session, &png)?;
    let image = Image::load_png(&png)?;
    boxes
        .iter()
        .map(|b| {
            let px = capture_rect(ctx, session, page, b)?;
            let (sum, n) = image.pixels_in(px).fold((0i64, 0i64), |(s, n), p| {
                (s + i64::from(p.b) - i64::from(p.r), n + 1)
            });
            #[allow(clippy::cast_precision_loss)]
            Ok(if n == 0 { 0.0 } else { sum as f64 / n as f64 })
        })
        .collect()
}
