//! # `text_box_live` — a markup text box is typed on the page
//!
//! Dragging Markup ▸ Text box opens the box where it was dragged; the words
//! appear in it as typed, in the engine's own appearance, and Ctrl+Enter
//! writes the annotation. Driven with the scripted pointer, off the desktop.

use super::annot_multiselect::FIXTURE;
use super::dimdrive::run_on;
use super::invisible_text_scripted::streams;
use super::multiselect::{after, step};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const BEGUN: &str = "text-box-draft-begun"; // ui-text-exempt: a trace event name
const PREVIEW: &str = "text-box-preview"; // ui-text-exempt: a trace event name
const DIALOG: &str = "text-annot-open"; // ui-text-exempt: a trace event name
const AUTHORED: &str = "add-text-annot"; // ui-text-exempt: a trace event name
const WORD: &str = "QKMYW";
/// The dragged box's corners, page points, on blank paper.
const FROM: (f64, f64) = (230.0, 140.0);
const TO: (f64, f64) = (380.0, 80.0);
/// A second box, kept by dragging a third.
const SECOND: ((f64, f64), (f64, f64)) = ((240.0, 285.0), (380.0, 235.0));
/// A third, kept by Escape.
const THIRD: ((f64, f64), (f64, f64)) = ((110.0, 50.0), (210.0, 15.0));
const ESCAPED: &str = "XQESCD";
const KEPT: &str = "XQKEPT";

/// See the module header.
pub struct ATextBoxIsTypedOnThePage;

impl Check for ATextBoxIsTypedOnThePage {
    fn name(&self) -> &'static str {
        "a_text_box_is_typed_on_the_page"
    }
    fn defect(&self) -> &'static str {
        "dragging a text box asks for its words in a dialog instead of on the page, the typed \
         words are not drawn in the box until it is committed, or committing writes nothing"
    }
    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "mode.review,markup.text_box",
            "text-box-live",
            drive,
        )
    }
}

/// The value of `key` on `line` as a number.
fn number(line: &crate::trace::TraceLine, key: &str) -> Option<usize> {
    line.get(key)?.parse().ok()
}

/// `/Subtype /FreeText` annotations named in `bytes` and its streams.
fn free_texts(bytes: &[u8]) -> usize {
    let count = |b: &[u8]| {
        String::from_utf8_lossy(b)
            .replace("/Subtype /", "/Subtype/")
            .matches("/Subtype/FreeText")
            .count()
    };
    count(bytes) + streams(bytes).iter().map(|s| count(s)).sum::<usize>()
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let doc = ctx.out("text-box-live.pdf");
    let read = |what: &str| {
        std::fs::read(&doc).map_err(|e| Error::new(format!("reading the {what} file: {e}")))
    };
    let before = free_texts(&read("opened")?);
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let at = |p: (f64, f64)| mapping.doc_to_window(DocPoint::new(0, p.0, p.1));
    if let Some(failure) = type_two_lines(session, pointer, report, &at)? {
        return Ok(Some(failure));
    }
    let mark = session.trace()?.mark();
    pointer.screenshot(session, &ctx.out("text-box-live.png"))?;
    pointer.key(session, None, "Enter", Some("ctrl"))?;
    session.settle(30);
    let authored = step!(after(
        session,
        AUTHORED,
        mark,
        "Ctrl+Enter was pressed in the box"
    ));
    report.note(format!("commit: `{}`", authored.raw));
    if let Some(failure) = kept_by_a_drag_and_by_escape(session, pointer, &at)? {
        return Ok(Some(failure));
    }
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    let bytes = read("saved")?;
    let now = free_texts(&bytes);
    let holds = |word: &str| {
        streams(&bytes)
            .iter()
            .any(|s| String::from_utf8_lossy(s).contains(word))
            || String::from_utf8_lossy(&bytes).contains(word)
    };
    let (worded, kept, escaped) = (holds(WORD), holds(KEPT), holds(ESCAPED));
    report.note(format!(
        "free text annotations: {before} before, {now} saved"
    ));
    Ok(
        (now != before + 3 || !worded || !kept || !escaped).then(|| {
            format!(
                "the saved file holds {now} free text annotation(s) where {} were wanted; \
             `{WORD}` present: {worded}, `{KEPT}` present: {kept}, `{ESCAPED}` present: \
             {escaped}.",
                before + 3
            )
        }),
    )
}

/// Drag the box, type a word, break the line and type again; nothing is
/// written meanwhile.
fn type_two_lines(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut CheckReport,
    at: &dyn Fn((f64, f64)) -> Result<crate::coords::WindowPoint>,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pointer.drag(session, at(FROM)?, at(TO)?, 8)?;
    session.settle(20);
    let begun = step!(after(
        session,
        BEGUN,
        mark,
        "a box was dragged with Text box armed"
    ));
    report.note(format!("drag: `{}`", begun.raw));
    if session.trace()?.last_after(DIALOG, mark).is_some() {
        return Ok(Some(
            "the drag opened the text dialog as well as the box on the page.".to_owned(),
        ));
    }
    pointer.type_text(session, None, WORD)?;
    session.settle(20);
    let one = step!(after(
        session,
        PREVIEW,
        mark,
        "a word was typed into the box"
    ));
    report.note(format!("typed: `{}`", one.raw));
    let inked = one.get("px").is_some_and(|px| px != "0x0");
    if one.get("baked") != Some("1")
        || number(&one, "chars") != Some(WORD.len())
        || number(&one, "lines") != Some(1)
        || !inked
    {
        return Ok(Some(format!(
            "after typing `{WORD}` the box traced `{}`: one baked line of {} characters, drawn \
             to pixels, was wanted.",
            one.raw,
            WORD.len()
        )));
    }
    pointer.key(session, None, "Enter", None)?;
    pointer.type_text(session, None, "ZJ")?;
    session.settle(20);
    let two = step!(after(
        session,
        PREVIEW,
        mark,
        "Enter and two more letters were typed"
    ));
    report.note(format!("second line: `{}`", two.raw));
    if number(&two, "lines") != Some(2) {
        return Ok(Some(format!(
            "after Enter and two more letters the box traced `{}`: two lines were wanted, so \
             Enter breaks the line and does not commit.",
            two.raw
        )));
    }
    if session.trace()?.last_after(AUTHORED, mark).is_some() {
        return Ok(Some(
            "an annotation was written while the box was still being typed in.".to_owned(),
        ));
    }
    Ok(None)
}

/// A drag elsewhere keeps the box being typed in, and so does Escape: every
/// way out of a draft writes it.
fn kept_by_a_drag_and_by_escape(
    session: &Session,
    pointer: &ScriptedPointer,
    at: &dyn Fn((f64, f64)) -> Result<crate::coords::WindowPoint>,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pointer.drag(session, at(SECOND.0)?, at(SECOND.1)?, 8)?;
    session.settle(20);
    step!(after(session, BEGUN, mark, "a second box was dragged"));
    pointer.type_text(session, None, KEPT)?;
    session.settle(10);
    let mark = session.trace()?.mark();
    pointer.drag(session, at(THIRD.0)?, at(THIRD.1)?, 8)?;
    session.settle(30);
    step!(after(
        session,
        AUTHORED,
        mark,
        "a third box was dragged while the second held words"
    ));
    step!(after(session, BEGUN, mark, "a third box was dragged"));
    pointer.type_text(session, None, ESCAPED)?;
    session.settle(10);
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Escape", None)?;
    session.settle(30);
    step!(after(
        session,
        AUTHORED,
        mark,
        "Escape was pressed in a box holding words"
    ));
    Ok(None)
}
