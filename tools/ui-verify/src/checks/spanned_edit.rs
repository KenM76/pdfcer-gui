//! `a_line_of_one_letter_operators_edits_as_one` — **text written one letter
//! per show operator takes an edit as one run, says so off-canvas, and undoes
//! in one step**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/spanned_edit.md`.

use crate::checks::word_styles::{Driven, launch_on};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

/// `ABC` at (72, 700), 12 pt Helvetica, one `Tj` per letter.
const FIXTURE: &str = "per-glyph-operators.pdf";
/// Inside the `A`.
const IN_A: (f64, f64) = (76.0, 704.0);
const TARGET: &str = "edit-text-target"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "edit-text"; // ui-text-exempt: a trace event name, never displayed
const UNDO: &str = "undo"; // ui-text-exempt: a trace event name, never displayed
/// The engine's own disclosure for a spanning edit opens with this word.
const SPAN_NOTE: &str = "span: the text was written across"; // ui-text-exempt: a needle for the engine's sentence

/// See the module documentation.
pub struct ALineOfOneLetterOperatorsEditsAsOne;

impl Check for ALineOfOneLetterOperatorsEditsAsOne {
    fn name(&self) -> &'static str {
        "a_line_of_one_letter_operators_edits_as_one"
    }

    fn defect(&self) -> &'static str {
        "a line written one letter per show operator refuses an edit, or takes it without \
         saying the edit ran across several operators, or needs several undos to take back"
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

/// Caret into the `A`, select the line, type `DEF` over all three operators,
/// Escape to commit; then Ctrl+Z. An append would be trimmed by the engine to
/// the last operator alone.
fn edit_and_undo(d: &Driven) -> Result<std::result::Result<usize, String>> {
    if let Err(why) = d.caret_at(IN_A)? {
        return Ok(Err(why));
    }
    d.pointer.key(&d.session, None, "Home", None)?;
    d.pointer.key(&d.session, None, "End", Some("shift"))?;
    d.pointer.type_text(&d.session, None, "DEF")?;
    d.session.settle(20);
    d.pointer.key(&d.session, None, "Escape", None)?;
    d.session.settle(30);
    let mark = d.session.trace()?.mark();
    d.pointer.key(&d.session, None, "Z", Some("ctrl"))?;
    d.session.settle(30);
    Ok(Ok(mark))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (d, _doc) = launch_on(ctx, report, "spanned-edit", FIXTURE)?;
    let mark = match edit_and_undo(&d)? {
        Ok(mark) => mark,
        Err(why) => return Ok(Some(why)),
    };
    d.pointer.gone(&d.session)?;
    let trace = d.session.trace()?;
    let spanned = trace
        .events(TARGET)
        .last()
        .and_then(|l| l.get_usize("operators_spanned"));
    let applied = trace.events(APPLIED).last().map(|l| l.raw.clone());
    let depth = trace
        .last_after(UNDO, mark)
        .and_then(|l| l.get_usize("undo_depth"));
    report.note(format!(
        "operators_spanned={spanned:?}; `{}`; undo_depth={depth:?}",
        applied.as_deref().unwrap_or("no edit-text line")
    ));
    let mut findings = Vec::new();
    if !spanned.is_some_and(|n| n > 1) {
        findings.push(format!(
            "the edit did not land across operators: `{TARGET}` operators_spanned={spanned:?}, \
             wanted more than 1."
        ));
    }
    if !applied.as_deref().is_some_and(|l| l.contains(SPAN_NOTE)) {
        findings.push(format!(
            "the commit's disclosures do not carry the engine's `{SPAN_NOTE}` sentence."
        ));
    }
    if depth != Some(1) {
        findings.push(format!(
            "Ctrl+Z after the edit reported undo_depth={depth:?}, wanted 1: the spanning edit \
             is not one undo step."
        ));
    }
    Ok((!findings.is_empty()).then(|| format!("{} Trace: {}.", findings.join(" "), d.path())))
}
