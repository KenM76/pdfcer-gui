//! `a_recognised_paragraph_edits_word_by_word` — a click in recognised text
//! opens its word, never the paragraph (the word keeps its in-place preview);
//! *This paragraph* then opens the recognised paragraph under it, the draft is
//! laid out in place by the engine's recognised preview, and a one-word change
//! commits through the engine's word-by-word rewrite: one
//! word replaced, the rest kept where they were, none deleted.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/recognised_paragraph.md`.

use super::dimdrive::{click_on, press, run_on};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const WIDENED: &str = "text-edit-widened"; // ui-text-exempt: a trace event name, never displayed
const ROW: &str = "menu.item.canvas.text.edit.paragraph_here"; // ui-text-exempt: a trace region name, never displayed
const APPLIED: &str = "edit-recognised-text-applied"; // ui-text-exempt: a trace event name, never displayed
const SHAPED: &str = "text-edit-shaped"; // ui-text-exempt: a trace event name, never displayed
/// On `SITE`, the first word of the layer's first line.
const ON_RECOGNISED: (f64, f64) = (85.0, 704.0);
/// Typed at the paragraph's start, so its first word changes and no other.
const TYPED: &str = "Q";

/// A recognised paragraph opens whole and commits word by word.
pub struct ARecognisedParagraphEditsWordByWord;

impl Check for ARecognisedParagraphEditsWordByWord {
    fn name(&self) -> &'static str {
        "a_recognised_paragraph_edits_word_by_word"
    }

    fn defect(&self) -> &'static str {
        "a click in recognised text opens the whole paragraph and loses the word's in-place \
         preview, This paragraph cannot open it, or an edit of the paragraph re-spaces every \
         word off the scan"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &super::ocr_select_block::FIXTURE,
            "mode.edit,view.ocr_layer,edit.text",
            "recognised-paragraph",
            drive,
        )
    }
}

/// Click, *This paragraph*, Home, type one letter, Ctrl+Enter.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let mark = session.trace()?.mark();
    click_on(
        ctx,
        session,
        pointer,
        &super::ocr_select_block::FIXTURE,
        0,
        ON_RECOGNISED,
    )?;
    session.settle(20);
    let early = session
        .trace()?
        .last_after(WIDENED, mark)
        .map(|l| l.raw.clone());
    if early.is_some() {
        return Ok(Some(format!(
            "a click in a recognised word opened {early:?}: the paragraph has no in-place \
             preview, so a click must open the word. Trace: {path}."
        )));
    }
    let fixture = &super::ocr_select_block::FIXTURE;
    let mapping =
        CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, fixture.page, 0)?;
    let point = mapping.doc_to_window(DocPoint::new(0, ON_RECOGNISED.0, ON_RECOGNISED.1))?;
    let mark = session.trace()?.mark();
    pointer.right_click(session, point)?;
    session.settle(30);
    press(ctx, session, pointer, ROW)?;
    session.settle(20);
    let widened = session
        .trace()?
        .last_after(WIDENED, mark)
        .map(|l| l.raw.clone());
    report.note(format!("this paragraph: {widened:?}"));
    if !widened
        .as_deref()
        .is_some_and(|w| w.contains("recognised=true"))
    {
        return Ok(Some(format!(
            "This paragraph on a recognised word traced {widened:?}; want {WIDENED} with \
             recognised=true. Trace: {path}."
        )));
    }
    pointer.key(session, None, "Home", None)?;
    session.settle(5);
    let typed_from = session.trace()?.mark();
    pointer.type_text(session, None, TYPED)?;
    session.settle(10);
    if let Some(failure) = laid_out_in_place(report, session, typed_from, &path)? {
        return Ok(Some(failure));
    }
    committed_word_by_word(report, session, pointer, &path)
}

/// The open recognised paragraph's last layout since `from` must be the
/// engine's in-place one; `Some` says what was traced instead.
fn laid_out_in_place(
    report: &mut CheckReport,
    session: &Session,
    from: usize,
    path: &str,
) -> Result<Option<String>> {
    let shaped = session
        .trace()?
        .last_after(SHAPED, from)
        .map(|l| l.raw.clone());
    report.note(format!("layout: {shaped:?}"));
    let in_place = shaped
        .as_deref()
        .is_some_and(|s| s.contains(" block=1 recognised=1 ") && s.contains(" shaped=1 "));
    Ok((!in_place).then(|| {
        format!(
            "the open recognised paragraph traced {shaped:?}; want {SHAPED} with block=1 \
             recognised=1 shaped=1, the engine's in-place layout. Trace: {path}."
        )
    }))
}

/// Ctrl+Enter must commit the one typed letter as one replaced word.
fn committed_word_by_word(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    path: &str,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    pointer.key(session, None, "Enter", Some("ctrl"))?;
    session.settle(40);
    let applied = session
        .trace()?
        .last_after(APPLIED, mark)
        .map(|l| l.raw.clone());
    report.note(format!("commit: {applied:?}"));
    let field = |name: &str| -> Option<usize> {
        let line = applied.as_deref()?;
        let at = line.find(&format!(" {name}="))? + name.len() + 2;
        line[at..].split_whitespace().next()?.parse().ok()
    };
    let word_by_word = field("replaced") == Some(1)
        && field("deleted") == Some(0)
        && field("kept").is_some_and(|k| k >= 1);
    Ok((!word_by_word).then(|| {
        format!(
            "typing one letter into a recognised paragraph committed {applied:?}; want \
             {APPLIED} with replaced=1 deleted=0 kept>=1. Trace: {path}."
        )
    }))
}
