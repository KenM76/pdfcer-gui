//! `form_preview_fields_open_as_their_widgets` — a check box in Form preview
//! turns on with one click, and a choice list opens as its choices, a
//! drop-down below the field and a list always shown over it, writing the
//! choice picked.
//!
//! Opens a copy of `fixtures/xfa-widgets.pdf`: Agree is a check box whose
//! data holds its off value, City a drop-down holding Ottawa, Colour a list
//! always shown holding Green. The check clicks Agree, then opens City and
//! picks its first item, then opens Colour and picks its first item. Each
//! step requires the window's own trace of the widget it opened, the
//! engine's outcome after it, and a redraw whose pixels differ.

use crate::checks::driving::declared_in;
use crate::checks::form_pages::launch_fixture;
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: &str = "xfa-widgets.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/xfa-widgets.PROVENANCE.py`.";
const DRAWN: &str = "form-preview";
const APPLIED: &str = "form-fill-applied";
const PICK: &str = "form-preview-pick";
const LIST: &str = "form-preview-list";
const AGREE: &str = "form-preview.field.0.0";
const CITY: &str = "form-preview.field.0.4";
const COLOUR: &str = "form-preview.field.0.5";
const FIRST_ITEM: &str = "form-preview.item.0";

/// See the module documentation.
pub struct FormPreviewFieldsOpenAsTheirWidgets;

impl Check for FormPreviewFieldsOpenAsTheirWidgets {
    fn name(&self) -> &'static str {
        "form_preview_fields_open_as_their_widgets"
    }

    fn defect(&self) -> &'static str {
        "a dynamic form's check box or choice list opens as a text box in Form preview, so its \
         value must be typed, or a click on it writes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_fixture(ctx, &mut report, "form_widgets", FIXTURE, METHOD).and_then(
            |(session, pointer, ui_rect)| {
                let outcome = drive(&session, &pointer, ui_rect);
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

fn drive(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<Option<String>> {
    if session.trace()?.last(DRAWN).is_none() {
        return Err(Error::new(format!(
            "the preview that opens with a dynamic form traced no `{DRAWN}` line; \
             `a_dynamic_form_opens_on_its_own_pages` owns that defect. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let toggled = step(
        session,
        pointer,
        ui_rect,
        AGREE,
        PICK,
        "kind=check state=on",
    )?;
    if let Some(failure) = toggled.or(written(session, "Agree")?) {
        return Ok(Some(failure));
    }
    for (region, label, kind) in [(CITY, "City", "drop-down"), (COLOUR, "Colour", "list")] {
        let opened = step(
            session,
            pointer,
            ui_rect,
            region,
            LIST,
            &format!("kind={kind} items=3"),
        )?;
        if let Some(failure) = opened {
            return Ok(Some(failure));
        }
        let picked = step(session, pointer, ui_rect, FIRST_ITEM, PICK, "item=0")?;
        if let Some(failure) = picked.or(written(session, label)?) {
            return Ok(Some(failure));
        }
    }
    Ok(None)
}

/// Click `region` and require an `event` line carrying `expect` after it.
fn step(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
    event: &str,
    expect: &str,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some((rect, vp)) = declared_in(&trace, ui_rect, region) else {
        return Ok(Some(format!(
            "★★★ the preview declared no `{region}` region. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let mark = trace.mark();
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(60);
    let trace = session.trace()?;
    match trace.last_after(event, mark) {
        Some(line) if line.raw.contains(expect) => Ok(None),
        Some(line) => Ok(Some(format!(
            "★★★ clicking `{region}` traced `{}`, which must carry {expect}.",
            line.raw
        ))),
        None => Ok(Some(format!(
            "★★★ clicking `{region}` traced no `{event}` line: the field did not open as its \
             widget. Trace: {}.",
            session.trace_path().display()
        ))),
    }
}

/// The step just driven wrote `label`'s field and the preview redrew it.
fn written(session: &Session, label: &str) -> Result<Option<String>> {
    let trace = session.trace()?;
    let Some(pick) = trace.last(PICK) else {
        return Ok(None);
    };
    let Some(applied) = trace.last_after(APPLIED, pick.lineno) else {
        return Ok(Some(format!(
            "★★★ {label} opened and was picked, and no `{APPLIED}` line followed: the value \
             was not written."
        )));
    };
    let before = trace
        .events(DRAWN)
        .filter(|l| l.lineno < applied.lineno)
        .last()
        .and_then(|l| l.get("digest"));
    let after = trace
        .last_after(DRAWN, applied.lineno)
        .and_then(|l| l.get("digest"));
    if after.is_none() || after == before {
        return Ok(Some(format!(
            "★★★ the preview did not redraw {label} with new pixels after `{}`.",
            applied.raw
        )));
    }
    Ok(None)
}
