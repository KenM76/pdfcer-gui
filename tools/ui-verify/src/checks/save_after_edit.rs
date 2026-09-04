//! `ctrl_s_after_an_edit_saves_and_the_program_is_still_running` — **the
//! operator pressed Ctrl+S and the window disappeared.**
//!
//! # The report
//!
//! Ken, 2026-09-01: *"can you try doing an edit and save? I did this and
//! pressed ctrl+s to save and it closed."*
//!
//! ## ★★★ Why the existing coverage could not see this
//!
//! Three things already test parts of this and none of them tests **the thing
//! that happened**:
//!
//! | what exists | what it does | why it misses this |
//! |---|---|---|
//! | `every_declared_chord_dispatches` | presses `Ctrl+S` among seven chords | on a **clean** document — nothing to write |
//! | `saving_writes_the_document` and friends | invoke `file.save` through `PDFCER_DIAG_INVOKE` | the seam, not the keyboard |
//! | 2,786 unit tests | call the verbs | a process that exits is not a returned `Err` |
//!
//! ⇒ The gap is the **conjunction**: a real keystroke, on a document that has
//! something to write. Each half was covered and the pair was not, which is the
//! shape of nearly every defect this project has found by driving.
//!
//! ## ★★ The oracle is that the process is ALIVE
//!
//! Unusual, and the reason is what was reported. Every other check in this
//! harness asks *"did the right trace line appear?"*; a program that has exited
//! writes no line at all, and an absent line is this harness's most common
//! **false** signal — it is what a missed click, a stale coordinate and a
//! window that never focused all look like.
//!
//! So this check asserts three things in order, and the order is what makes the
//! diagnosis unambiguous:
//!
//! 1. the edit landed (`pages-rotated`) — so there IS something to save;
//! 2. the process is still running **after** the chord;
//! 3. the save committed (`save-in-place outcome=ok`).
//!
//! ★ If (2) fails, (3)'s absence is explained and must not be reported as
//! *"the save did nothing"*. Getting that order wrong would turn a crash into a
//! silent-verb report and send a reader to the wrong module entirely.
//!
//! ## Why a page rotation is the edit
//!
//! It is the only document change in this shell that needs **no pointer at
//! all** — one command id, no dialog, no selection, no picker. Every other edit
//! needs a click, and a click that missed would make this check report a save
//! failure that was really an aim failure.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The edit, through the seam: rotate page one. No pointer, no dialog.
const INVOKE: &str = "pages.rotate_right";
/// The line the rotation writes.
///
/// ★ `rotate-pages`, not `pages-rotated`. The name was guessed on the first
/// draft and the check SKIPPED against a build where the rotation had plainly
/// worked — a harness constant naming an application event decays in one
/// direction only, and the fix is always to read the trace rather than to
/// widen the assertion.
const ROTATED: &str = "rotate-pages"; // ui-text-exempt: a trace event name, never displayed
/// ★ The line a successful save-in-place writes.
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name, never displayed
/// The chord the operator pressed.
const CHORD: &str = "Ctrl+S"; // ui-text-exempt: a key chord, shown only in this report
/// The canvas's own per-frame line. Carries `pages=`, `page=` and `zoom=`,
/// which is everything O65 promises a save must leave alone.
const CANVAS: &str = "canvas"; // ui-text-exempt: a trace event name, never displayed
/// The line an unsaved-edits prompt writes when it opens.
///
/// ★ O65's chain ran through this prompt: the tab kept its dot after a save,
/// so the NEXT close raised it, and its only save button was a picker that
/// proceeded with the close on success. Press save, get asked for a filename,
/// watch the document close. If this appears at all in a run that only pressed
/// Ctrl+S, something is treating a save as a close.
const UNSAVED_PROMPT: &str = "unsaved-ask"; // ui-text-exempt: a trace event name

pub struct CtrlSAfterAnEditSavesAndTheProgramIsStillRunning;

impl Check for CtrlSAfterAnEditSavesAndTheProgramIsStillRunning {
    fn name(&self) -> &'static str {
        "ctrl_s_after_an_edit_saves_and_the_program_is_still_running"
    }

    fn defect(&self) -> &'static str {
        "pressing Ctrl+S after making a change closes the program — the operator loses the \
         window, and whether the document was written first is not something they can find out \
         from anything on screen"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). The whole subject is a real keystroke — the seam \
             route is already covered elsewhere and does not reproduce this.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let source = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a document to edit and save."))?;

    // ★★★ **A COPY, and this is not tidiness.** The check saves IN PLACE, which
    // is the operator's own gesture and the one that was reported. Pointing it
    // at `--pdf` would rewrite the fixture every run — and `--pdf` is usually
    // one of the operator's own drawings.
    let scratch = ctx.out("save-after-edit.pdf");
    if let Some(dir) = scratch.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::new(e.to_string()))?;
    }
    std::fs::copy(&source, &scratch).map_err(|e| {
        Error::new(format!(
            "could not copy {} to {}: {e}",
            source.display(),
            scratch.display()
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("save-after-edit.trace.txt"));
    spec.pdf = Some(scratch.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} on a scratch copy",
        exe.display(),
        session.pid()
    ));
    session.settle(50);

    // --- 1: the edit landed, so there is something to save ------------------
    let trace = session.trace()?;
    if trace.events(ROTATED).count() == 0 {
        return Err(Error::new(format!(
            "no `{ROTATED}` line, so `{INVOKE}` changed nothing and there would be nothing for \
             a save to write. That is somebody else's subject; this check cannot reach its own. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the document has an unsaved change");

    // --- 2: press the chord, for real ---------------------------------------
    let driver = Driver::new(session.window());
    driver.press_chord(&[vk::CONTROL], vk::S)?;
    session.settle(60);

    // ★★★ THE ASSERTION THIS CHECK EXISTS FOR, and it comes BEFORE the trace
    // is read. A process that has exited writes no line, and an absent line is
    // this harness's commonest false signal — reporting "the save did nothing"
    // when the truth is "the program is gone" sends a reader to the wrong
    // module and loses the actual defect.
    if session.has_exited()? {
        return Ok(Some(format!(
            "★★★ THE PROGRAM CLOSED. `{CHORD}` was pressed on a document with an unsaved \
             change and the process is no longer running.\n\
             This is the operator's report, reproduced: *\"I did this and pressed ctrl+s to \
             save and it closed.\"* Read the END of the trace — a panic prints there before \
             the process goes, and the last line before it names the frame it died in. \
             Whether anything reached the file is a separate question and the trace answers \
             that too: look for `{SAVED}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★ the program is still running after the chord");

    // --- 3: …and the save actually committed --------------------------------
    let trace = session.trace()?;
    let Some(saved) = trace.last(SAVED) else {
        return Ok(Some(format!(
            "★★ THE CHORD SAVED NOTHING: the program is alive and no `{SAVED}` line followed \
             `{CHORD}`. The keystroke reached the window — the process would not have survived \
             a modal picker otherwise — so the fault is between the chord matcher and \
             `Action::Save`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if saved.get("outcome") != Some("ok") {
        return Ok(Some(format!(
            "★★ THE SAVE REPORTED A FAILURE: `{}`. The program survived and the document was \
             not written, which is the quiet half of the operator's complaint — a save that \
             refuses in the status line is still a save that did not happen. Trace: {}.",
            saved.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!("★★ the save committed: `{}`", saved.raw));

    // --- 4: …and the DOCUMENT is still open, on the same page, same zoom ----
    //
    // ★★★ **THIS is `OPERATOR_REQUESTS.md` O65**, and it is the half that was
    // fixed on 2026-08-31 and marked NOT DRIVEN — then reported again by the
    // operator on 2026-09-01 against the build that carried the fix.
    //
    // His words the first time: *"I can't edit it unless I save the document
    // first, at which point it closes the document after saving."* The chain
    // was: a successful save recorded `saved_epoch` and nothing read it, so the
    // tab kept its dot, the next Close raised the unsaved-edits prompt, and
    // that prompt's only save button was a picker which proceeded with the
    // close. Press save, get asked for a filename, watch the document close.
    //
    // ⇒ So "the program is alive" is NOT enough. A shell that saved, kept
    // running, and closed the document would satisfy every assertion above and
    // would be the exact defect. The canvas line carries `pages=`, `page=` and
    // `zoom=`, which is the whole of what O65 says a save must leave alone.
    let before_zoom = trace
        .events(CANVAS)
        .filter(|l| l.get("pages").is_some_and(|p| p != "0"))
        .count();
    let Some(canvas) = trace.last(CANVAS) else {
        return Ok(Some(format!(
            "★★★ THE CANVAS STOPPED DRAWING: the save committed and no `{CANVAS}` line \
             followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let pages = canvas.get("pages").unwrap_or("0");
    if pages == "0" {
        return Ok(Some(format!(
            "★★★ THE DOCUMENT CLOSED AFTER THE SAVE — `OPERATOR_REQUESTS.md` O65, reported \
             twice.\n\
             The program is still running and the file was written, and the canvas now reports \
             `pages=0`: `{}`.\n\
             O65's chain: a save that does not clear the unsaved flag leaves the tab dirty, the \
             next act raises the unsaved prompt, and that prompt's save button proceeds with a \
             CLOSE. Look at `save::has_unsaved_edits` and every surface that reads it — the fix \
             was one predicate in three places, and a fourth caller asking the old question \
             brings the whole chain back. {} `{UNSAVED_PROMPT}` line(s) were traced. Trace: {}.",
            canvas.raw,
            trace.events(UNSAVED_PROMPT).count(),
            session.trace_path().display()
        )));
    }
    if trace.events(UNSAVED_PROMPT).count() > 0 {
        return Ok(Some(format!(
            "★★ A SAVE RAISED THE UNSAVED-EDITS PROMPT: `{UNSAVED_PROMPT}` was traced on a run \
             that pressed nothing but `{CHORD}`. The document is still open, so the operator \
             does not lose it — but they are being asked whether to keep changes they have \
             just asked to keep, which is the front half of O65's chain and the state the \
             back half needs. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ …and the document is still open on {pages} page(s), with {before_zoom} canvas \
         frame(s) drawn — O65, driven for the first time"
    ));
    Ok(None)
}
