//! The Properties panel restyling annotations another program drew: a text
//! box's border refused as foreign and then redrawn on request, a stamp's
//! opacity and words, and a caret's icon colour through the same offer.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/annot_restyle.md`.

use super::dimdrive::{Fixture, click_on, pick_colour, press, run_on, ui_rect_event};
use super::stroke_style::{into_view, type_into};
use crate::checks::driving::declared_in;
use crate::checks::{Check, CheckContext};
use crate::coords::PageGeometry;
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.edit,file.properties";
const FIXTURE: Fixture = Fixture {
    file: "restyle-annots.pdf",
    method: "Rebuild it with `python fixtures/restyle-annots.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const SELECT: &str = "annot-select"; // ui-text-exempt: a trace event name, never displayed
const OFFER: &str = "redraw-offer"; // ui-text-exempt: a trace event name, never displayed
const STYLED: &str = "set-text-annot-style-applied"; // ui-text-exempt: a trace event name, never displayed
const FADED: &str = "set-annot-opacity-applied"; // ui-text-exempt: a trace event name, never displayed
const RECOLOURED: &str = "set-marker-colour-applied"; // ui-text-exempt: a trace event name, never displayed
/// Mirror `panels::properties::markup::{textbox, anyannot, textannot}`'s regions.
const TEXT_BOX: &str = "properties.markup.textbox";
const WIDTH: &str = "properties.markup.textbox.width";
const SIZE: &str = "properties.markup.textbox.size";
const BORDER: &str = "properties.markup.textbox.border";
const FILL: &str = "properties.markup.textbox.fill";
const FILL_PICKER: &str = "properties.markup.textbox.fill.picker";
const INK: &str = "properties.markup.textbox.ink";
const OPACITY: &str = "properties.markup.any.opacity";
const MARKER: &str = "properties.markup.any.marker";
const MARKER_PICKER: &str = "properties.markup.any.marker.picker";
const REDRAW: &str = "properties.markup.any.redraw";
const WORDS: &str = "properties.markup.textannot.words";
/// Inside each annotation's `/Rect`, page space.
const ON_THE_BOX: (f64, f64) = (140.0, 320.0);
const ON_THE_STAMP: (f64, f64) = (120.0, 210.0);
const ON_THE_CARET: (f64, f64) = (290.0, 306.0);

/// A text box another program drew is restyled once the operator asks for
/// the redraw, and then restyles directly.
pub struct ATextBoxAnotherProgramDrewIsRestyledOnRequest;
/// A stamp's opacity and words change from the Properties panel.
pub struct AStampsOpacityAndWordsChange;
/// A caret another program drew is recoloured once the operator asks for
/// pdfcer's icon.
pub struct AMarkersIconIsRecolouredOnRequest;

impl Check for ATextBoxAnotherProgramDrewIsRestyledOnRequest {
    fn name(&self) -> &'static str {
        "a_text_box_another_program_drew_is_restyled_on_request"
    }

    fn defect(&self) -> &'static str {
        "a text box has no border, background, text colour, font or size rows; or a restyle \
         of one whose look another program drew is refused with no way to go on; or the redraw \
         it offers does not apply the change asked for"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "annot-restyle-box", text_box)
    }
}

impl Check for AStampsOpacityAndWordsChange {
    fn name(&self) -> &'static str {
        "a_stamps_opacity_and_words_change"
    }

    fn defect(&self) -> &'static str {
        "a stamp's opacity or its words cannot be changed from the Properties panel"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "annot-restyle-stamp", stamp)
    }
}

impl Check for AMarkersIconIsRecolouredOnRequest {
    fn name(&self) -> &'static str {
        "a_markers_icon_is_recoloured_on_request"
    }

    fn defect(&self) -> &'static str {
        "a caret, attachment, sound or screen mark has no icon colour row, or a recolour of one \
         whose icon another program drew is refused with no way to go on"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, INVOKE, "annot-restyle-marker", marker)
    }
}

fn text_box(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = select(ctx, session, pointer, ON_THE_BOX, "/FreeText")? {
        return Ok(Some(failure));
    }
    let ui_rect = ui_rect_event(ctx)?;
    into_view(ctx, session, pointer, INK)?;
    let trace = session.trace()?;
    let missing: Vec<&str> = [TEXT_BOX, WIDTH, SIZE, BORDER, FILL, INK]
        .into_iter()
        .filter(|r| declared_in(&trace, ui_rect, r).is_none())
        .collect();
    if !missing.is_empty() {
        return Ok(Some(format!(
            "the selected text box shows no {missing:?}: its rows are not drawn. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // The fixture's look is hand-drawn, so the first restyle is refused.
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, WIDTH, "3")?;
    let trace = session.trace()?;
    if let Some(line) = trace.last_after(STYLED, mark) {
        return Ok(Some(format!(
            "a border width on a text box another program drew was applied without asking: \
             `{}`.",
            line.raw
        )));
    }
    let Some(offer) = trace.last_after(OFFER, mark) else {
        return Ok(Some(format!(
            "a border width on a text box another program drew left no `{OFFER}`: the refusal \
             offers no way on. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if offer.get("why") != Some("text-box-foreign") {
        return Ok(Some(format!(
            "`{}` must name why=text-box-foreign.",
            offer.raw
        )));
    }
    report.note(format!("refused: `{}`", offer.raw));

    let mark = session.trace()?.mark();
    if let Some(failure) = press_redraw(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    session.settle(30);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(STYLED, mark) else {
        return Ok(Some(format!(
            "Redraw as plain text was pressed and no `{STYLED}` followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("redrawn: `{}`", line.raw));
    if !(line.get("was_foreign") == Some("1")
        && line.get("redraw") == Some("true")
        && line.get("frame") == Some("true"))
    {
        return Ok(Some(format!(
            "`{}` must replace a foreign look (was_foreign=1, redraw=true) and write the border \
             (frame=true).",
            line.raw
        )));
    }

    // pdfcer's own look now: the next change applies with no offer.
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, SIZE, "16")?;
    let trace = session.trace()?;
    let Some(line) = trace.last_after(STYLED, mark) else {
        return Ok(Some(format!(
            "after the redraw a text size was refused or never sent (no `{STYLED}`). Trace: {}.",
            session.trace_path().display()
        )));
    };
    if line.get("was_foreign") != Some("0") || line.get("size") != Some("true") {
        return Ok(Some(format!(
            "`{}` must apply the size (size=true) to pdfcer's own look (was_foreign=0).",
            line.raw
        )));
    }

    let mark = session.trace()?.mark();
    if let Some(failure) = pick_colour(ctx, session, pointer, FILL, FILL_PICKER)? {
        return Ok(Some(failure));
    }
    match session.trace()?.last_after(STYLED, mark) {
        Some(l) if l.get("frame") == Some("true") => {
            report.note(format!("background: `{}`", l.raw));
            Ok(None)
        }
        other => Ok(Some(format!(
            "a background was picked and the restyle reads {:?}; it must write the frame \
             (frame=true).",
            other.map(|l| l.raw.clone())
        ))),
    }
}

fn stamp(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = select(ctx, session, pointer, ON_THE_STAMP, "/Stamp")? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, OPACITY, "40")?;
    let trace = session.trace()?;
    let Some(line) = trace.last_after(FADED, mark) else {
        return Ok(Some(format!(
            "40 % typed into a stamp's opacity sent no `{FADED}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("opacity: `{}`", line.raw));
    if line.get("current") != Some("0.4") {
        return Ok(Some(format!("`{}` must read current=0.4.", line.raw)));
    }

    let mark = session.trace()?.mark();
    type_into(ctx, session, pointer, WORDS, "CHECKED")?;
    let trace = session.trace()?;
    let Some(line) = trace.last_after(STYLED, mark) else {
        return Ok(Some(format!(
            "new words typed into a stamp sent no `{STYLED}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("words: `{}`", line.raw));
    Ok((line.get("label") != Some("true"))
        .then(|| format!("`{}` must write the words (label=true).", line.raw)))
}

fn marker(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    if let Some(failure) = select(ctx, session, pointer, ON_THE_CARET, "/Caret")? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    if let Some(failure) = pick_colour(ctx, session, pointer, MARKER, MARKER_PICKER)? {
        return Ok(Some(failure));
    }
    let trace = session.trace()?;
    if let Some(line) = trace.last_after(RECOLOURED, mark) {
        return Ok(Some(format!(
            "a caret another program drew was recoloured without asking: `{}`.",
            line.raw
        )));
    }
    match trace.last_after(OFFER, mark) {
        Some(o) if o.get("why") == Some("marker-foreign") => {
            report.note(format!("refused: `{}`", o.raw));
        }
        other => {
            return Ok(Some(format!(
                "a colour picked for a caret another program drew left {:?}; it must leave a \
                 `{OFFER}` with why=marker-foreign.",
                other.map(|l| l.raw.clone())
            )));
        }
    }
    let mark = session.trace()?.mark();
    if let Some(failure) = press_redraw(ctx, session, pointer)? {
        return Ok(Some(failure));
    }
    session.settle(30);
    let trace = session.trace()?;
    let Some(line) = trace.last_after(RECOLOURED, mark) else {
        return Ok(Some(format!(
            "Redraw with pdfcer's icon was pressed and no `{RECOLOURED}` followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("redrawn: `{}`", line.raw));
    Ok(
        (!(line.get("was_foreign") == Some("1") && line.get("redraw") == Some("true"))).then(
            || {
                format!(
                    "`{}` must replace a foreign icon (was_foreign=1, redraw=true).",
                    line.raw
                )
            },
        ),
    )
}

/// Click `at` and require an `annot-select` after it.
fn select(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: (f64, f64),
    what: &str,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    click_on(ctx, session, pointer, &FIXTURE, 0, at)?;
    session.settle(20);
    Ok(session
        .trace()?
        .last_after(SELECT, mark)
        .is_none()
        .then(|| {
            format!(
                "a click inside the {what} traced no `{SELECT}`. Trace: {}.",
                session.trace_path().display()
            )
        }))
}

/// Press the refusal's redraw button, failing when the panel does not show it:
/// a `redraw-offer` trace with no button is an offer the operator cannot take.
fn press_redraw(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    into_view(ctx, session, pointer, REDRAW)?;
    if declared_in(&session.trace()?, ui_rect_event(ctx)?, REDRAW).is_none() {
        return Ok(Some(format!(
            "the refusal was traced and the Properties panel shows no `{REDRAW}` button to take it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    press(ctx, session, pointer, REDRAW)?;
    Ok(None)
}
