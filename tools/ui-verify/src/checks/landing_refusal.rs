//! `the_caret_says_a_line_cannot_be_edited_in_place` — **a caret landing on
//! a line the engine will not edit in place says so before a key is typed**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/landing_refusal.md`.

use crate::checks::word_styles::{Driven, launch_invoking};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;
use crate::trace::Trace;

/// `Plain line` (`Tj`, baseline 700) over `Quoted line` (`'`, baseline 680),
/// 12 pt.
const FIXTURE: &str = "quote-operator.pdf";
/// Paragraphs off, so each click opens its own line.
const INVOKE: &str = "mode.edit,edit.paragraphs,edit.text"; // ui-text-exempt: command ids, never displayed
const IN_PLAIN: (f64, f64) = (100.0, 704.0);
const IN_QUOTED: (f64, f64) = (100.0, 684.0);
const CAPABILITY: &str = "text-edit-capability"; // ui-text-exempt: a trace event name, never displayed
const NOTE: &str = "text-edit-note"; // ui-text-exempt: a trace event name, never displayed
const CLASSIFIED: &str = "edit-text-classified"; // ui-text-exempt: a trace event name, never displayed
const NOTE_REGION: &str = "status-group:draft-note"; // ui-text-exempt: a region name, never displayed

/// See the module documentation.
pub struct TheCaretSaysALineCannotBeEditedInPlace;

impl Check for TheCaretSaysALineCannotBeEditedInPlace {
    fn name(&self) -> &'static str {
        "the_caret_says_a_line_cannot_be_edited_in_place"
    }

    fn defect(&self) -> &'static str {
        "a caret lands on a line the engine will not edit in place and says nothing until the \
         operator has typed and finished, or the refusal then is the generic sentence"
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

/// The last `text-edit-capability` line after `mark`, as `cause`/`offered`.
fn capability_after(trace: &Trace, mark: usize) -> Option<(String, Option<usize>)> {
    let line = trace.last_after(CAPABILITY, mark)?;
    Some((line.get("cause")?.to_owned(), line.get_usize("offered")))
}

/// The plain line, as the control: answered editable, no note drawn.
fn plain(d: &Driven, findings: &mut Vec<String>) -> Result<()> {
    let mark = d.session.trace()?.mark();
    if let Err(why) = d.caret_at(IN_PLAIN)? {
        findings.push(why);
        return Ok(());
    }
    let trace = d.session.trace()?;
    let answer = capability_after(&trace, mark);
    if answer.as_ref().map(|(c, _)| c.as_str()) != Some("none") {
        findings.push(format!(
            "the plain line's landing traced `{CAPABILITY}` {answer:?}, wanted cause=none."
        ));
    }
    if d.declares(NOTE_REGION)? {
        findings.push(format!(
            "a note was drawn at `{NOTE_REGION}` on the plain line, before the quoted line."
        ));
    }
    d.pointer.key(&d.session, None, "Escape", None)?;
    d.session.settle(20);
    Ok(())
}

/// The quoted line: answered at landing, the note drawn, the commit's
/// refusal said by its cause.
fn quoted(d: &Driven, findings: &mut Vec<String>) -> Result<()> {
    let mark = d.session.trace()?.mark();
    if let Err(why) = d.caret_at(IN_QUOTED)? {
        findings.push(why);
        return Ok(());
    }
    let trace = d.session.trace()?;
    let answer = capability_after(&trace, mark);
    if answer != Some(("quote-operator".to_owned(), Some(1))) {
        findings.push(format!(
            "the quoted line's landing traced `{CAPABILITY}` {answer:?}, wanted \
             cause=quote-operator offered=1."
        ));
    }
    let note = trace
        .last_after(NOTE, mark)
        .and_then(|l| l.get("note").map(str::to_owned));
    if note.as_deref() != Some("cannot-edit-in-place") || !d.declares(NOTE_REGION)? {
        findings.push(format!(
            "no key was typed and the landing raised `{NOTE}` note={note:?} with \
             `{NOTE_REGION}` drawn={}; wanted cannot-edit-in-place, drawn.",
            d.declares(NOTE_REGION)?
        ));
    }
    let mark = d.session.trace()?.mark();
    d.pointer.key(&d.session, None, "End", None)?;
    d.pointer.type_text(&d.session, None, "s")?;
    d.session.settle(20);
    d.pointer.key(&d.session, None, "Escape", None)?;
    d.session.settle(30);
    let said = d
        .session
        .trace()?
        .last_after(CLASSIFIED, mark)
        .and_then(|l| l.get("said").map(str::to_owned));
    if said.as_deref() != Some("CannotEdit") {
        findings.push(format!(
            "the commit's refusal was classified said={said:?}, wanted CannotEdit."
        ));
    }
    Ok(())
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (d, _doc) = launch_invoking(ctx, report, "landing-refusal", FIXTURE, INVOKE)?;
    let mut findings = Vec::new();
    plain(&d, &mut findings)?;
    quoted(&d, &mut findings)?;
    d.pointer.gone(&d.session)?;
    let trace = d.session.trace()?;
    let lines: Vec<String> = trace.events(CAPABILITY).map(|l| l.raw.clone()).collect();
    report.note(format!("{CAPABILITY}: {}", lines.join(" | ")));
    Ok((!findings.is_empty()).then(|| format!("{} Trace: {}.", findings.join(" "), d.path())))
}
