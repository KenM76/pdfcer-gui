//! `form_preview_fields_open_as_their_widgets` — a check box in Form preview
//! turns on with one click, and a choice list opens as its choices, a
//! drop-down below the field and a list always shown over it, writing the
//! choice picked; a list that takes several keeps every Ctrl+click pick
//! through the fill, the redraw and Use these pages.
//!
//! Opens a copy of `fixtures/xfa-widgets.pdf`: Agree is a check box whose
//! data holds its off value, City a drop-down holding Ottawa, Colour a list
//! always shown holding Green, Towns a `multiSelect` list holding nothing.
//! The check clicks Agree, then opens City and picks its first item, then
//! opens Colour and picks its first item. Each step requires the window's
//! own trace of the widget it opened, the engine's outcome after it, and a
//! redraw whose pixels differ. Then it opens Towns, Ctrl+clicks New York
//! and Boston, presses Enter, requires both written and redrawn, reopens
//! Towns and requires both ticked from the form's data, and presses Use
//! these pages and requires the written Towns list box to select two.

use crate::checks::driving::declared_in;
use crate::checks::form_pages::{launch_fixture, press_use};
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
const TOGGLE: &str = "form-preview-toggle";
const PAGES: &str = "form-pages-applied";
const BOX_LINE: &str = "form-box";
const AGREE: &str = "form-preview.field.0.0";
const CITY: &str = "form-preview.field.0.4";
const COLOUR: &str = "form-preview.field.0.5";
const TOWNS: &str = "form-preview.field.0.7";
const FIRST_ITEM: &str = "form-preview.item.0";

/// See the module documentation.
pub struct FormPreviewFieldsOpenAsTheirWidgets;

impl Check for FormPreviewFieldsOpenAsTheirWidgets {
    fn name(&self) -> &'static str {
        "form_preview_fields_open_as_their_widgets"
    }

    fn defect(&self) -> &'static str {
        "a dynamic form's check box or choice list opens as a text box in Form preview, so its \
         value must be typed, or a click on it writes nothing, or a list that takes several \
         choices keeps only one"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch_fixture(ctx, &mut report, "form_widgets", FIXTURE, METHOD).and_then(
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

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
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
        None,
        PICK,
        "kind=check state=on",
    )?;
    if let Some(failure) = toggled.or(written(session, "Agree")?) {
        return Ok(Some(failure));
    }
    for (region, label, kind) in [(CITY, "City", "drop-down"), (COLOUR, "Colour", "list")] {
        let expect = format!("kind={kind} items=3");
        if let Some(failure) = step(session, pointer, ui_rect, region, None, LIST, &expect)? {
            return Ok(Some(failure));
        }
        let picked = step(session, pointer, ui_rect, FIRST_ITEM, None, PICK, "item=0")?;
        if let Some(failure) = picked.or(written(session, label)?) {
            return Ok(Some(failure));
        }
    }
    if let Some(failure) = several(session, pointer, ui_rect)? {
        return Ok(Some(failure));
    }
    materialized(ctx, report, session, pointer, ui_rect)
}

/// Towns: Ctrl+click New York and Boston, Enter, then reopen and require
/// both ticked from the redrawn form's data.
fn several(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<Option<String>> {
    let open = "kind=multi items=3 ticked=none";
    if let Some(failure) = step(session, pointer, ui_rect, TOWNS, None, LIST, open)? {
        return Ok(Some(failure));
    }
    for k in [1, 2] {
        let item = format!("form-preview.item.{k}");
        let expect = format!("item={k} state=on");
        let toggled = step(
            session,
            pointer,
            ui_rect,
            &item,
            Some("ctrl"),
            TOGGLE,
            &expect,
        )?;
        if let Some(failure) = toggled {
            return Ok(Some(failure));
        }
    }
    // Keys go to the window's own viewport, the one its regions name.
    let trace = session.trace()?;
    let vp = declared_in(&trace, ui_rect, TOWNS).and_then(|(_, vp)| vp);
    let mark = trace.mark();
    pointer.key(session, vp.as_deref(), "Enter", None)?;
    session.settle(60);
    let trace = session.trace()?;
    match trace.last_after(PICK, mark) {
        Some(line) if line.raw.contains("kind=multi item=1,2") => {}
        Some(line) => {
            return Ok(Some(format!(
                "★★★ Enter after Ctrl+clicking New York and Boston traced `{}`, which must \
                 carry kind=multi item=1,2: a Ctrl+click replaced the pick before it.",
                line.raw
            )));
        }
        None => {
            return Ok(Some(format!(
                "★★★ Enter on the open Towns list traced no `{PICK}` line: its picks were not \
                 written. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }
    if let Some(failure) = written(session, "Towns")? {
        return Ok(Some(failure));
    }
    let reread = "kind=multi items=3 ticked=1,2";
    if let Some(failure) = step(session, pointer, ui_rect, TOWNS, None, LIST, reread)? {
        return Ok(Some(format!(
            "{failure} The form's data, read back after the fill, does not hold both picks."
        )));
    }
    pointer.key(session, vp.as_deref(), "Enter", None)?;
    session.settle(30);
    Ok(None)
}

/// Use these pages, then require the census's Towns list box to select two.
fn materialized(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
    if let Some(failure) = press_use(ctx, report, ui_rect, session, pointer)? {
        return Ok(Some(failure));
    }
    session.settle(30);
    let trace = session.trace()?;
    let Some(applied) = trace.last(PAGES) else {
        return Ok(Some(format!(
            "★★★ no `{PAGES}` line after Use these pages."
        )));
    };
    let towns = trace
        .events(BOX_LINE)
        .filter(|l| l.lineno > applied.lineno)
        .find(|l| {
            l.get("kind") == Some("choice")
                && l.get("field")
                    .is_some_and(|f| f.to_ascii_lowercase().contains("towns"))
        });
    let Some(towns) = towns else {
        return Ok(Some(format!(
            "★★★ `{}` was followed by no `{BOX_LINE} kind=choice` line naming Towns: the \
             written page has no fillable Towns list. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    };
    report.note(towns.raw.clone());
    if towns.get("selected") != Some("2") {
        return Ok(Some(format!(
            "★★★ `{}` must carry selected=2: Use these pages wrote the Towns list without both \
             of its picks.",
            towns.raw
        )));
    }
    Ok(None)
}

/// Click `region`, with `mods` held when given, and require an `event` line
/// carrying `expect` after it.
fn step(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    region: &str,
    mods: Option<&str>,
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
    let at = WindowPoint::centre_of(rect);
    match mods {
        Some(m) => pointer.click_in_with(session, vp.as_deref(), at, m)?,
        None => pointer.click_in(session, vp.as_deref(), at)?,
    };
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
