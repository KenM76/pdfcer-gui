//! `closing_the_program_asks_before_losing_unsaved_work` — **the prompt that
//! did not exist.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/quit_unsaved.md`.

use std::path::PathBuf;

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys;

/// `quit-held dirty=…` — the close was requested and held back.
const HELD: &str = "quit-held";

/// `quit-cancelled` — the operator abandoned the quit.
const CANCELLED: &str = "quit-cancelled";

/// `unsaved-asked cycle=true dirty=…` — the question reached the screen.
const ASKED: &str = "unsaved-asked";

/// `exit-flush layout-written=…` — proof the process really did exit in phase A.
const EXIT: &str = "exit-flush";

/// The Cancel control on the unsaved window.
const CANCEL_REGION: &str = "unsaved.cancel";

/// The **Save all** control, which must be ABSENT with one dirty document.
const SAVE_ALL_REGION: &str = "unsaved.save_all";

/// A document with something editable on it.
const FIXTURE: &str = "fixtures/four-pages.pdf";

/// How long to wait for the process to go, in settle frames (25 ms each).
const CLOSE_FRAMES: u32 = 160;

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// See the module documentation.
pub struct ClosingTheProgramAsksBeforeLosingUnsavedWork;

impl Check for ClosingTheProgramAsksBeforeLosingUnsavedWork {
    fn name(&self) -> &'static str {
        "closing_the_program_asks_before_losing_unsaved_work"
    }

    fn defect(&self) -> &'static str {
        "the window's ✕ ends the process with unsaved work still unsaved and no question asked \
         — or the opposite, a build that asks on every close including the ones with nothing \
         outstanding"
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

/// Launch on the fixture. `Err` is a SKIP.
fn launch(ctx: &CheckContext, trace: &str) -> Result<(Session, PathBuf)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!("fixture missing: {}", pdf.display())));
    }
    let mut spec = LaunchSpec::new(&exe, ctx.out(trace));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pdf))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check edits a document and presses Alt+F4. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;

    // --- phase A: a CLEAN document closes with no question ------------------
    //
    // The falsifying half, and it runs first. A build that popped the dialog
    // on every close would satisfy every assertion in phase B, so this one has
    // to establish that the question is CONDITIONAL before the next one
    // establishes that it appears.
    {
        let (session, _) = launch(ctx, "quit-clean.trace.txt")?;
        report.note(format!(
            "phase A: pid {} on a clean document",
            session.pid()
        ));
        report.artifact(session.trace_path().to_path_buf());
        session.settle(40);
        if !session.trace()?.started(ctx.profile.vocab.start_event) {
            return Err(Error::new(
                "the trace has no start line, so the diagnostic switch did not reach the \
                 process.",
            ));
        }
        let driver = Driver::new(session.window());
        driver.press_chord(&[sys::vk::ALT], sys::vk::F4)?;
        let mut spent = 0;
        while !session.has_exited()? && spent < CLOSE_FRAMES {
            session.settle(8);
            spent += 8;
        }
        if !session.has_exited()? {
            return Ok(Some(format!(
                "★★★ A CLEAN DOCUMENT'S CLOSE WAS HELD. The program was still running {} \
                 seconds after Alt+F4 with nothing edited.\n\n\
                 The question must be conditional: an operator who has saved everything and \
                 presses ✕ should get an immediate exit, not a dialog with nothing in it. \
                 Look at `quitting::first_dirty` answering `Some` for a document nobody has \
                 touched — which would mean `save::has_unsaved_edits` is true on open.",
                u64::from(CLOSE_FRAMES) * 25 / 1000
            )));
        }
        // The process is MEANT to be gone by here — Alt+F4 was pressed and the
        // loop above waited for it. Said out loud so `Session::trace`'s liveness
        // guard, which otherwise reports a dead process as a red failure, knows
        // this exit is the subject rather than a crash. See that function.
        session.expect_exit();
        let trace = session.trace()?;
        if let Some(held) = trace.last(HELD) {
            return Ok(Some(format!(
                "★★ A CLEAN CLOSE WAS HELD BACK: `{}`. It exited anyway, so nothing is lost — \
                 but the close was interrupted on a document with nothing outstanding, and the \
                 next thing an operator meets is a modal asking about work they have not done.",
                held.raw
            )));
        }
        report.note(format!(
            "★ it exited with no `{HELD}` line — the question is conditional, which is what \
             makes phase B's assertion mean something{}",
            trace
                .last(EXIT)
                .map_or_else(String::new, |l| format!(" (`{}`)", l.raw))
        ));
    }

    // --- phase B: a DIRTY document holds the close and asks -----------------
    let (session, _) = launch(ctx, "quit-dirty.trace.txt")?;
    report.note(format!(
        "phase B: pid {} — now to make an edit",
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    session.maximize();
    session.settle(12);
    let driver = Driver::new(session.window());

    // **THE STATE NOTHING ELSE CONSTRUCTS.** Everything below is ordinary;
    // this line is the reason the defect survived a suite of 140 checks. A
    // rotation is the cheapest edit that needs no canvas aim, no tool and no
    // typing — it is one ribbon command and it dirties the session.
    driving::click_mode_segment(&session, &driver, ui_rect, "edit")?;
    crate::checks::ocr::click_tab(&session, &driver, ui_rect, "pages")?;
    crate::checks::ocr::click_command(&session, &driver, ui_rect, "pages.rotate_right")?;
    session.settle(20);
    report.note("made one edit — a page rotation, the cheapest edit that needs no canvas aim");

    driver.press_chord(&[sys::vk::ALT], sys::vk::F4)?;
    session.settle(30);

    // Still running is the FIRST thing asserted, because a process that has
    // gone cannot be asked anything and every assertion below would then be
    // about a trace that stops mid-sentence.
    if session.has_exited()? {
        return Ok(Some(format!(
            "★★★ THE PROGRAM CLOSED WITH UNSAVED WORK AND ASKED NOTHING.\n\n\
             This is the operator's report exactly, and it is the state the feature was built \
             for: one edit, one Alt+F4, and the process is gone. Before 2026-09-02 this is \
             what always happened — `eframe`'s close request was never read. Look at \
             `PdfcerApp::step_quit_cycle` being called from the frame at all, and at whether \
             `ViewportCommand::CancelClose` reaches egui on the same frame the request is \
             read. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let trace = session.trace()?;
    let Some(held) = trace.last(HELD) else {
        return Ok(Some(format!(
            "the program is still running and traced no `{HELD}` line, so the close was never \
             requested or never held. If `{ASKED}` is present the cycle ran without recording \
             that it held the door, which is a trace defect rather than a behaviour one."
        )));
    };
    report.note(format!("★★ the close was held: `{}`", held.raw));

    let Some(asked) = trace.last(ASKED) else {
        return Ok(Some(format!(
            "★★ THE CLOSE WAS HELD AND NOTHING WAS ASKED: `{}` with no `{ASKED}` line.\n\n\
             That is the worst of the three states — the program refuses to close and does not \
             say why, so the ✕ appears broken. Look at `ask_unsaved_for_quit` and at whether \
             `unsaved::ask_for` is refusing (it consults `save::has_unsaved_edits`, which the \
             cycle has just used to decide there IS something dirty — if those two disagree, \
             that is the defect).",
            held.raw
        )));
    };
    report.note(format!("and the question was asked: `{}`", asked.raw));

    // **Save all must be ABSENT here**, because exactly one document is
    // dirty. Its presence would mean the count is not reaching the dialog, and
    // the operator would meet a button that does the same as the one beside it
    // on a modal standing between them and leaving.
    let live = session.trace()?;
    if driving::declared(&live, ui_rect, SAVE_ALL_REGION).is_some() {
        return Ok(Some(format!(
            "★ SAVE ALL WAS DRAWN FOR ONE DIRTY DOCUMENT. `{SAVE_ALL_REGION}` was declared \
             with a single document open, where it must be absent: with one document it is the \
             same act as Save. Look at `UnsavedDialog::dirty_documents` — `new` passes 1 and \
             only `for_cycle` should pass more."
        )));
    }
    report.note(
        "★ and `Save all` is absent, which is correct for one dirty document — with one it \
         would be the same act as Save",
    );

    // --- Cancel, and the program stays ---------------------------------------
    //
    // The ending is Cancel on purpose. Save would write a file and Discard
    // would destroy work, and a check running unattended on the operator's own
    // machine should do neither. Cancel proves the whole chain and leaves
    // nothing behind.
    crate::checks::ocr::click_region(&session, &driver, ui_rect, CANCEL_REGION)?;
    session.settle(24);

    if session.has_exited()? {
        return Ok(Some(
            "★★★ CANCEL CLOSED THE PROGRAM. The operator pressed Cancel on a question about \
             losing their work and lost it. `Cancel` must abandon the quit — see \
             `quitting::Quitting::stand_down` and `DialogsState::unsaved_cancelled`, which is \
             the drain that exists because a Cancel parks no outcome and would otherwise be \
             indistinguishable from 'they have not answered yet'."
                .to_owned(),
        ));
    }
    let trace = session.trace()?;
    if trace.last(CANCELLED).is_none() {
        return Ok(Some(format!(
            "the program is still running, which is right, and no `{CANCELLED}` line was \
             traced. The cycle may still believe it is quitting, in which case the next frame \
             re-asks — check whether the dialog is up again."
        )));
    }
    report.note(
        "★★★ Cancel abandoned the quit and the program is still running — so an operator who \
         changes their mind keeps both their work and their session",
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two phases assert opposite things, and both are needed.**
    #[test]
    fn the_check_asserts_both_directions() {
        // Held is read in both phases and means opposite things in each. This
        // is a statement about the file rather than a computation, and it is
        // here so that a later edit which deletes phase A has to delete this
        // sentence too.
        assert_eq!(HELD, "quit-held");
        assert_ne!(HELD, ASKED, "the two phases read different events");
    }

    /// Every trace event and region is spelled once and distinctly.
    #[test]
    fn the_names_are_distinct() {
        let all = [HELD, CANCELLED, ASKED, EXIT, CANCEL_REGION, SAVE_ALL_REGION];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b, "two constants name the same thing");
            }
        }
    }

    /// The check ends on Cancel, which writes nothing and destroys nothing.
    #[test]
    fn the_ending_is_the_harmless_one() {
        assert!(
            CANCEL_REGION.contains("cancel"),
            "the ending must be Cancel — see this test's doc comment"
        );
    }
}
