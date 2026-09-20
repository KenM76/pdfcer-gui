//! `the_apply_report_lists_the_text_it_will_destroy` — the driven proof of
//! `OPERATOR_REQUESTS.md` **O217**, fourth bullet.
//!
//! # What this closes
//!
//! Until this block the apply report answered *how much*: regions, pages,
//! characters, content streams. A number cannot be checked against an
//! intention. Redaction is the one verb in this program whose mistake survives
//! the undo stack, so *"what will be removed is visible before it is
//! committed"* is a safety control and not a convenience.
//!
//! The case that makes it load-bearing: the shell's unit of text selection is
//! the visual line, and `G032` records that the engine groups a line with no
//! horizontal-gap criterion, so a bill-of-materials row welds its item number,
//! part number, description and quantity into one selectable thing. An operator
//! who marks the quantity has marked the row, R8b forbids saying so on the
//! canvas, and this list is the only place he can be told.
//!
//! # ★★★ What the oracle is, and the two things it cannot see
//!
//! The dialog publishes one trace line and one region:
//!
//! ```text
//! pdfcer-diag redact-apply-removed-text state=listed entries=1 chars=24 lines=4
//! pdfcer-diag ui-rect name=redact-apply-removed-text rect=[..]
//! ```
//!
//! `state` is the branch the derivation took, carried out of it rather than
//! re-derived, so the line reports what was drawn rather than what a second
//! reading of the report would have drawn. The region is published through
//! `diag::ui_rect_visible`, so *drawn and readable*, *drawn behind the fold*
//! and *not drawn at all* are three different observations rather than one
//! absence.
//!
//! **It carries no text, deliberately.** `PDFCER_DIAG` writes to stderr and
//! gets redirected into files; a redaction surface that copied the operator's
//! confidential strings into a second file would have undone its own job. So:
//!
//! | this check proves | this check cannot prove |
//! |---|---|
//! | the block exists in the build and was laid out | that the quoted strings read correctly |
//! | it found text rather than reporting none | that the quotation marks are the right glyphs |
//! | it listed at least as many characters as the fixture put on the page | the order of the entries |
//!
//! `chars` is what makes the middle row more than a presence assertion. The
//! fixture's page 1 carries exactly one known string, so a build that listed
//! *some* text rather than *that* text reports a count below its length and
//! fails here. That is the strongest content assertion available without
//! putting content in the log.
//!
//! # ★★ Why a whole-page mark rather than a selection
//!
//! The selection route has its own check
//! (`a_selected_object_can_be_marked_for_redaction`). This one wants a mark
//! whose contents are known *to the harness*, and the only marking gesture with
//! that property is the one that takes everything on a page the harness itself
//! generated.

use std::path::PathBuf;

use crate::checks::redaction::{
    self, EDIT_TAB, MODE, PANEL_EVENT, PREPARED_EVENT, REDACT, REFUSED_EVENT, REGION_PREFIX,
    SECRET, WHOLE_PAGE_REGION,
};
use crate::checks::{Check, CheckContext, driving};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::report::CheckReport;
use crate::trace::Trace;

/// The apply control on the marking panel.
const APPLY_REGION: &str = "redact-apply";

/// The block under test — both the trace key and the published region name.
const REMOVED_TEXT: &str = "redact-apply-removed-text";

/// What `diag::ui_rect_visible` emits instead when a region is out of view.
///
/// Its `name=` field is part of the event key rather than a field of the value,
/// so the whole line is matched on the event name and then filtered.
const CLIPPED_EVENT: &str = "ui-rect-clipped";

/// See the module documentation.
pub struct TheApplyReportListsTheTextItWillDestroy;

impl Check for TheApplyReportListsTheTextItWillDestroy {
    fn name(&self) -> &'static str {
        "the_apply_report_lists_the_text_it_will_destroy"
    }

    fn defect(&self) -> &'static str {
        "the apply report counts what redaction will remove and never says what it is, so an \
         operator whose mark took a whole welded table row instead of the one cell he aimed at \
         has no way to find out before the removal is permanent"
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

/// The last `redact-apply-removed-text` line, if the dialog drew the block.
fn drawn(trace: &Trace) -> Option<&crate::trace::TraceLine> {
    trace.last(REMOVED_TEXT)
}

/// What fraction of the block was on screen, when it was reported as clipped.
fn clipped_fraction(trace: &Trace) -> Option<f32> {
    trace
        .events(CLIPPED_EVENT)
        .filter(|line| line.get("name") == Some(REMOVED_TEXT))
        .last()
        .and_then(|line| line.get_f32("shown"))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check has to mark a page and open the apply \
             report, which is four clicks. Reported as SKIPPED rather than passed: a check that \
             did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;

    // The fixture is generated rather than taken from `--pdf`, and that is the
    // whole reason this check can assert anything about CONTENT: `chars` is
    // only meaningful against a page whose text the harness wrote itself.
    let fixture: PathBuf = ctx.out("redact-preview-fixture.pdf");
    let source = redaction::fixture_bytes();
    std::fs::write(&fixture, &source)
        .map_err(|e| Error::new(format!("cannot write {}: {e}", fixture.display())))?;

    // ★ The falsifying phase. If the string this check measures against is not
    // in the file it just wrote, then `chars >= SECRET.len()` is an assertion
    // about nothing and would pass on a build that listed the wrong text.
    if !redaction::contains(&source, SECRET.as_bytes()) {
        return Ok(Some(format!(
            "★ THE INSTRUMENT HAS NOTHING TO MEASURE AGAINST. `{SECRET}` was written into the \
             fixture at {} and a byte scan of the same file does not find it, so this check's \
             character-count assertion could not fail. Harness defect, reported as a FAILURE so \
             it cannot be mistaken for a pass.",
            fixture.display()
        )));
    }
    let expected = SECRET.chars().count();
    report.note(format!(
        "fixture {} — page 1 carries `{SECRET}`, {expected} characters, and nothing else",
        fixture.display()
    ));

    let target = ctx.out("redact-preview-unused.pdf");
    let _ = std::fs::remove_file(&target);

    let session = redaction::launch(ctx, report, &fixture, &target, "redact-preview.trace.txt")?;
    let driver = Driver::new(session.window());

    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(16);

    // --- mark page 1 ------------------------------------------------------
    redaction::click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
    redaction::click_command(&session, &driver, ui_rect, REDACT, 20)?;
    redaction::region(&session.trace()?, ui_rect, WHOLE_PAGE_REGION, REGION_PREFIX).map_err(
        |e| {
            Error::new(format!(
                "{e}\nthe marking panel published no controls, so this check never reached the \
             report it is about."
            ))
        },
    )?;
    redaction::click_region(&session, &driver, ui_rect, WHOLE_PAGE_REGION, 18)?;

    let trace = session.trace()?;
    let Some((marks, _)) = redaction::census(&trace) else {
        return Err(Error::new(format!(
            "the panel traced no `{PANEL_EVENT}` line after the whole-page click, so this check \
             cannot tell a mark that was made from a click that missed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if marks == 0 {
        return Err(Error::new(
            "the whole-page click produced no mark, so there is no report to open. A panel \
             control has no input-channel confirmation, so this is a SKIP rather than a failure \
             of the block under test: the click may never have been delivered."
                .to_owned(),
        ));
    }

    // --- open the apply report --------------------------------------------
    redaction::click_region(&session, &driver, ui_rect, APPLY_REGION, 24)?;
    let trace = session.trace()?;
    if trace.last(PREPARED_EVENT).is_none() {
        if let Some(refused) = trace.last(REFUSED_EVENT) {
            return Ok(Some(format!(
                "★ THE APPLY WAS REFUSED on a fixture that should redact cleanly: `{}`. No report \
                 was drawn, so the block under test had nothing to draw into.",
                refused.raw
            )));
        }
        return Err(Error::new(format!(
            "the apply control was clicked and traced neither `{PREPARED_EVENT}` nor \
             `{REFUSED_EVENT}`, so the dialog never opened. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- ★ the assertion ---------------------------------------------------
    let Some(line) = drawn(&trace) else {
        return Ok(Some(format!(
            "★ THE REPORT DOES NOT SAY WHAT IT WILL REMOVE. The apply dialog is open and no \
             `{REMOVED_TEXT}` line was traced, so `dialogs::redact::disclosures::removed_text` \
             was never called — the block is either absent from this build or its call site in \
             `RedactDialog::report` has been deleted. Every count in the report is still drawn, \
             which is exactly why this is invisible to a reading of the dialog: it looks \
             complete. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let raw = line.raw.clone();
    report.note(format!("the block reported `{raw}`"));

    match line.get("state") {
        Some("listed") => {}
        Some("no-text") => {
            return Ok(Some(format!(
                "★ THE REPORT SAYS THE MARK HOLDS NO TEXT and the marked page carries `{SECRET}` \
                 in an uncompressed Helvetica content stream. That sentence is the one this \
                 surface must never print wrongly: it tells an operator his mark took only line \
                 work, and he confirms a removal believing nothing readable is inside it. `{raw}`"
            )));
        }
        Some("unreported") => {
            return Ok(Some(format!(
                "★ THE ENGINE COUNTED CODES AND REPORTED NO TEXT FOR THEM on a fixture whose page \
                 is one Helvetica string. `RedactionReport::redacted_text` came back empty with a \
                 non-zero glyph count, which is an engine finding rather than a shell one — the \
                 disclosure drawn here is correct for the state, and the state is wrong. `{raw}`"
            )));
        }
        other => {
            return Ok(Some(format!(
                "the block traced state={other:?}, which is not one of the three branches \
                 `removed_text_lines` can take. Either a fourth branch was added without a word \
                 for it, or the trace line's shape has changed under this check. `{raw}`"
            )));
        }
    }

    let Some(chars) = line.get_usize("chars") else {
        return Ok(Some(format!(
            "the block traced no `chars` field, so this check can tell that text was listed but \
             not whether it was THE text. That field is the only content assertion available to a \
             trace that deliberately carries no content. `{raw}`"
        )));
    };
    if chars < expected {
        return Ok(Some(format!(
            "★ THE REPORT LISTS LESS TEXT THAN THE PAGE CARRIES. The mark covers the whole of a \
             page whose only content is `{SECRET}` — {expected} characters — and the block lists \
             {chars}. Something between the engine's decode and this list is dropping part of \
             what is about to be destroyed, and the operator would read the surviving fragment as \
             the whole of it. `{raw}`"
        )));
    }

    // --- ★★ and it has to be somewhere he can read ------------------------
    //
    // A region published means it was laid out; a region reported clipped means
    // it was laid out below the fold of a scrolling report, which is not a
    // defect on its own — the acknowledgement control sits below this block, so
    // an operator who confirms has necessarily scrolled past it. It is recorded
    // rather than asserted for exactly that reason, and it is recorded because
    // a block that is ALWAYS entirely out of view is the panel-shipped-
    // unreachable failure in a new place, and nothing else would report it.
    if driving::declared(&trace, ui_rect, REMOVED_TEXT).is_some() {
        report.note(
            "the block is published as visible, so it is readable without scrolling the report"
                .to_owned(),
        );
    } else if let Some(shown) = clipped_fraction(&trace) {
        report.note(format!(
            "⚠ the block was laid out with {:.0}% of it inside the report's clip rectangle, so \
             the operator reaches it by scrolling. Recorded rather than failed: the \
             acknowledgement control is below this block, so confirming requires scrolling past \
             it either way.",
            shown * 100.0
        ));
    } else {
        return Ok(Some(format!(
            "★ THE BLOCK REPORTED ITS CONTENT AND NEVER PUBLISHED A RECTANGLE. `{REMOVED_TEXT}` \
             was traced with {chars} character(s) listed and neither `{ui_rect} \
             name={REMOVED_TEXT}` nor `{CLIPPED_EVENT} name={REMOVED_TEXT}` followed it, so the \
             derivation ran and the layout did not. Trace: {}.",
            session.trace_path().display()
        )));
    }

    Ok(None)
}
