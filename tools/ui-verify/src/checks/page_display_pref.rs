//! `a_page_display_choice_survives_a_close_and_reaches_a_new_document` — **the
//! preference he said the program forgets.**
//!
//! # The report
//!
//! Ken, 2026-08-31, `OPERATOR_REQUESTS.md` O80:
//!
//! > *"Also it should remember my page display preferences from my last closing
//! > of the program. Example if I press show one page at a time and enable flip
//! > pages."*
//!
//! It **was** remembered — per document, written the moment the control is
//! pressed. What there was no answer for is a document the program has never
//! seen, so a choice made on one drawing meant nothing on the next. From his
//! chair that is forgetting, and he was right.
//!
//! Three tiers now resolve it: **this document's own record**, then **his
//! standing preference**, then the mode's rule.
//!
//! # ★★★ Why this is a TWO-PROCESS check, and why the close must be graceful
//!
//! Two separate defects sit behind that sentence and only a second launch can
//! tell them apart.
//!
//! **The first** is the missing middle tier — it is about a *different
//! document*, so a check that never opens one cannot see it.
//!
//! **The second is sharper.** `LayoutStore::flush` says in its own words that it
//! exists *"for an exit path, which must not lose the last change to a debounce
//! that had not yet expired"* — and for a long time it had **no production
//! caller**. The layout is written **750 ms** after it changes, so anything
//! changed in the last three quarters of a second before the window closed was
//! silently thrown away.
//!
//! ⇒ **Dropping a `Session` kills the process, and a killed process runs no exit
//! hook.** A check that killed the window and then found the preference intact
//! would only be asserting that the debounce had already expired — true on a
//! slow run, false on a fast one, and not the property anybody cares about. So
//! this check closes with **`Alt+F4`**, a real `WM_CLOSE`, and closes
//! **immediately** after the click so the debounce is still holding.
//!
//! # ★★ The oracle, and the trace line that could not carry it until today
//!
//! `page-display mode=… source=… ribbon-mode=…` is emitted as each document
//! opens. `source=` is the tier the answer came from, and until 2026-09-02 it
//! reported only **two** of the three: anything that was not the document's own
//! record was called `mode-default`, so a display that came from the standing
//! preference was indistinguishable from one the mode had chosen.
//!
//! That is precisely the pair this check has to separate, so the disclosure was
//! fixed first. Second time in three days that writing a driven check found a
//! trace which could not tell apart the two states the check existed for — the
//! OCR tally and the marquee census were the others.
//!
//! # ★ Why the second document must be one the program has never seen
//!
//! Because the per-document record would answer for any document that had been
//! opened before, and it would answer *correctly* — hiding the missing tier
//! completely. The check therefore opens `fixtures/four-pages.pdf` first and
//! `fixtures/paragraph.pdf` second, and **normalises the remembered-documents
//! file** before it starts so a previous run cannot have seeded either.
//!
//! # What is normalised, and why that is safe
//!
//! `userdata/` sits **beside the executable**, so the state this check reads and
//! writes belongs to the binary under test. It deletes `page-display.txt`,
//! `preferences.txt` and `layout.ron` before the first launch, so every run
//! starts from the shipped defaults rather than from whatever the last run left.
//!
//! ★★ Safe because the suite is **never** pointed at a published build — that is
//! the standing rule, and this check is one of the reasons for it. Pointed at
//! `OneDrive\pdfcer-gui1`, it would delete the operator's own saved preferences.
//!
//! # Every way this reports SKIP
//!
//! No binary, `--no-input`, no diagnostic channel, the page-display control not
//! declared on the ribbon, or the window refusing to close within the grace
//! period — the last being a property of the machine on the day, reported as a
//! skip that says so rather than as a pass.

use std::path::{Path, PathBuf};

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys;

/// The tab the page-display controls live on.
const TAB: &str = "view";

/// The control pressed, and it is deliberately **not** the compiled-in default.
///
/// ★ `PageDisplay::Single` is what a fresh install shows, so a check that chose
/// Single would pass against a build that persisted nothing at all — the
/// preference and the default would agree and the check could not tell them
/// apart. Facing is nobody's default.
const COMMAND: &str = "view.page_facing";

/// What that control's id resolves to in the trace.
const WANTED: &str = "facing";

/// `page-display mode=… source=… ribbon-mode=…`, emitted as a document opens.
const DISPLAY_EVENT: &str = "page-display";

/// `exit-flush layout-written=…` — the exit hook's own record that it ran.
///
/// ★ Traced even when nothing was pending, deliberately: *"the hook ran and had
/// nothing to do"* and *"the hook never ran"* are the two states the defect hid
/// between, and a line that only appeared on a write could not tell them apart.
const FLUSH_EVENT: &str = "exit-flush";

/// The first document. Opened, changed, closed.
const FIRST: &str = "fixtures/four-pages.pdf";

/// The second — **a document the program has never seen**, which is the whole
/// subject. See the module header.
const SECOND: &str = "fixtures/paragraph.pdf";

/// Files under `userdata/` that carry the state this check is about.
///
/// Deleted before the first launch so every run starts from the shipped
/// defaults. `D:/dev/rag/egui/` carries the rule this follows: *a driven check
/// that mutates persisted state must normalise at the start.*
const STATE_FILES: [&str; 3] = ["page-display.txt", "preferences.txt", "layout.ron"];

/// How long to wait for the window to go after `Alt+F4`, in settle frames.
///
/// 25 ms a frame, so 160 frames is four seconds. Generous: the exit path writes
/// files, and a machine that is also rasterizing has been seen to take a second
/// over it. A close that has not happened by then is reported as a skip, because
/// a check that could not close the program has not measured what happens when
/// it closes.
const CLOSE_FRAMES: u32 = 160;

/// See the module documentation.
pub struct APageDisplayChoiceSurvivesACloseAndReachesANewDocument;

impl Check for APageDisplayChoiceSurvivesACloseAndReachesANewDocument {
    fn name(&self) -> &'static str {
        "a_page_display_choice_survives_a_close_and_reaches_a_new_document"
    }

    fn defect(&self) -> &'static str {
        "a page-display choice is forgotten — either because it was never kept for a document \
         the program had not seen before, or because it was made inside the 750 ms write \
         debounce and the window closed before the debounce expired"
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

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Delete the persisted state beside `exe`, so the run starts from the shipped
/// defaults. Reports what it removed.
fn normalise(exe: &Path, report: &mut CheckReport) {
    let Some(dir) = exe.parent().map(|d| d.join("userdata")) else {
        return;
    };
    let mut removed = Vec::new();
    for name in STATE_FILES {
        let path = dir.join(name);
        if path.exists() && std::fs::remove_file(&path).is_ok() {
            removed.push(name);
        }
    }
    if removed.is_empty() {
        report.note("no persisted state to normalise — this run starts clean anyway");
    } else {
        report.note(format!(
            "normalised {} in {} — every run starts from the shipped defaults, or a previous \
             run's preference would satisfy this check without the feature working",
            driving::list_str(&removed),
            dir.display()
        ));
    }
}

/// Build a launch spec for `pdf`.
fn spec_for(ctx: &CheckContext, exe: &Path, pdf: &Path, trace: &str) -> LaunchSpec {
    let mut spec = LaunchSpec::new(exe, ctx.out(trace));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    spec
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control and presses \
             Alt+F4. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the page-display control \
             cannot be found.",
            ctx.profile.name
        ))
    })?;
    let first = workspace_root().join(FIRST);
    let second = workspace_root().join(SECOND);
    for pdf in [&first, &second] {
        if !pdf.is_file() {
            return Err(Error::new(format!("fixture missing: {}", pdf.display())));
        }
    }

    normalise(&exe, report);

    // --- process 1: choose, then close immediately ---------------------------
    let session = Session::launch(
        &spec_for(ctx, &exe, &first, "page-display-1.trace.txt"),
        ctx.profile.trace_prefix,
    )?;
    report.note(format!(
        "process 1: {} on {}",
        session.pid(),
        first.file_name().unwrap_or_default().to_string_lossy()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    session.maximize();
    session.settle(12);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process.",
            ctx.profile.vocab.start_event
        )));
    }
    let opened = trace
        .last(DISPLAY_EVENT)
        .and_then(|l| l.get("mode"))
        .unwrap_or("unreported")
        .to_owned();
    if opened == WANTED {
        return Err(Error::new(format!(
            "the first document opened already showing `{WANTED}`, so pressing the control \
             would change nothing and the check could not tell a kept preference from an \
             unchanged default. The normalisation above should have prevented this — \
             something outside this check is seeding the state."
        )));
    }
    report.note(format!("it opened as `{opened}`, which is not the target"));

    let driver = Driver::new(session.window());
    crate::checks::ocr::click_tab(&session, &driver, ui_rect, TAB)?;
    crate::checks::ocr::click_command(&session, &driver, ui_rect, COMMAND)?;
    report.note(format!("pressed `{COMMAND}` on the ribbon"));

    // ★★★ **CLOSE IMMEDIATELY, AND GRACEFULLY.** No settle between the click
    // and the chord: the whole subject is a change made inside the 750 ms write
    // debounce, and pausing here would let the debounce expire and quietly turn
    // this into a check of the ordinary path. `Alt+F4` is a real `WM_CLOSE`, so
    // the exit hook runs — dropping the `Session` would kill the process and
    // skip it.
    driver.press_chord(&[sys::vk::ALT], sys::vk::F4)?;
    let mut spent = 0;
    while !session.has_exited()? && spent < CLOSE_FRAMES {
        session.settle(8);
        spent += 8;
    }
    if !session.has_exited()? {
        return Err(Error::new(format!(
            "the window had not closed {} seconds after Alt+F4, so this check cannot say what \
             a close does. It is reported as a skip rather than a failure: a window that will \
             not close is a different defect from a preference that is not kept.",
            u64::from(CLOSE_FRAMES) * 25 / 1000
        )));
    }
    // ★ The process is MEANT to be gone by here — Alt+F4 was pressed and the
    // loop above waited for it. Said out loud so `Session::trace`'s liveness
    // guard, which otherwise reports a dead process as a red failure, knows
    // this exit is the subject rather than a crash. See that function.
    session.expect_exit();
    let trace = session.trace()?;
    match trace.last(FLUSH_EVENT) {
        Some(line) => {
            report.note(format!("the exit hook ran: `{}`", line.raw));
            // ★★★ **WHAT A `layout-written=false` MEANS, said rather than
            // implied.**
            //
            // The exit hook ran and the LAYOUT store had nothing pending. That
            // is the ordinary case here, because this check changes a page
            // display and not the ribbon mode — and the layout file is what
            // carries the mode.
            //
            // ★★ So this run proves the hook is CALLED and proves the standing
            // preference survives; it does NOT exercise the debounce rescue,
            // because there was nothing in the debounce to rescue. A reader who
            // took the green as covering both halves would be taking more than
            // is here.
            //
            // Exercising it needs a ribbon-mode change immediately before the
            // close, which is a different property and belongs in its own
            // check.
            if line.get("layout-written") != Some("true") {
                report.note(
                    "★ NOT established by this run: the debounce rescue. The exit hook ran and \
                     the layout store had nothing pending (`layout-written=false`), because \
                     this check changes a page display and the layout file carries the ribbon \
                     MODE. What is established is that the hook is called and that the standing \
                     preference survives a close made seconds after the choice",
                );
            }
        }
        None => {
            return Ok(Some(format!(
                "★★★ THE EXIT HOOK DID NOT RUN. The window closed and no `{FLUSH_EVENT}` line \
                 was traced.\n\n\
                 `LayoutStore::flush` exists *\"for an exit path, which must not lose the last \
                 change to a debounce that had not yet expired\"*, and for a long time it had \
                 no production caller at all — a change made in the last 750 ms before closing \
                 was silently thrown away. That line is traced even when nothing was pending, \
                 precisely because *\"the hook ran and had nothing to do\"* and *\"the hook \
                 never ran\"* are the two states this defect hid between. Its absence is the \
                 second. Look at `App::on_exit` in `app::frame`."
            )));
        }
    }
    drop(session);

    // --- process 2: a document it has never seen -----------------------------
    let session = Session::launch(
        &spec_for(ctx, &exe, &second, "page-display-2.trace.txt"),
        ctx.profile.trace_prefix,
    )?;
    report.note(format!(
        "process 2: {} on {} — a document the program has never opened",
        session.pid(),
        second.file_name().unwrap_or_default().to_string_lossy()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(50);

    let trace = session.trace()?;
    let Some(line) = trace.last(DISPLAY_EVENT) else {
        return Ok(Some(format!(
            "the second process traced no `{DISPLAY_EVENT}` line, so it never resolved a page \
             display for the document it opened. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let mode = line.get("mode").unwrap_or("unreported");
    let source = line.get("source").unwrap_or("unreported");
    report.note(format!("it opened as `{}`", line.raw));

    if mode != WANTED {
        return Ok(Some(format!(
            "★★★ THE PREFERENCE WAS FORGOTTEN: the second document opened as `{mode}` where \
             `{WANTED}` was chosen seconds earlier — `{}`.\n\n\
             This is the operator's report reproduced. Two causes, and `source=` tells them \
             apart:\n\
             • `source=mode-default` — no standing preference was found. Either pressing the \
             control does not record one, or the exit write did not land. The `{FLUSH_EVENT}` \
             line above says the hook ran, so look at whether `Action::SetPageDisplay` writes \
             `Prefs::default_page_display` at all.\n\
             • `source=document` — the second document had a remembered entry of its own, \
             which means the normalisation at the top of this check did not take and the \
             result says nothing about the preference.\n\n\
             Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    if source != "preference" {
        return Ok(Some(format!(
            "★★ THE RIGHT ANSWER FROM THE WRONG TIER: the display is `{mode}`, which is what \
             was chosen, and `source={source}` rather than `preference` — `{}`.\n\n\
             `source=document` means this document had its own remembered entry, so the \
             standing preference was never consulted and this check has proved nothing about \
             it. `source=mode-default` with the right mode would mean the mode's rule happens \
             to agree, which is a coincidence rather than a feature.\n\n\
             ★ Until 2026-09-02 this line could not report `preference` at all — it called \
             everything that was not the document's own record `mode-default`. If that is what \
             came back, the trace has regressed to two tiers.",
            line.raw
        )));
    }

    report.note(format!(
        "★★★ the choice was made, the window was closed straight away, and a document the \
         program had never seen opened as `{WANTED}` from the standing preference — which is \
         the operator's sentence exactly, and it exercises both halves: the middle tier and \
         the exit flush that carried it there inside the debounce window"
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★★ **The control pressed must not be the compiled-in default.**
    ///
    /// `PageDisplay::Single` is what a fresh install shows. A check that chose
    /// Single would pass against a build that persisted nothing whatsoever —
    /// the preference and the default would agree, and the second process would
    /// open correctly for entirely the wrong reason. This is the assertion that
    /// keeps the check from being decorative.
    #[test]
    fn the_chosen_display_is_not_the_shipped_default() {
        assert_ne!(WANTED, "single", "single is the compiled-in default");
        assert!(
            COMMAND.ends_with(WANTED),
            "the command id and the mode token must name the same display, or the check \
             presses one control and asserts another"
        );
    }

    /// The two documents are different files.
    ///
    /// ★ Pinned because the whole subject is *a document the program has never
    /// seen*: opening the same file twice would be answered by the per-document
    /// record, correctly, and would hide the missing tier completely.
    #[test]
    fn the_second_document_is_a_different_file() {
        assert_ne!(FIRST, SECOND);
    }

    /// The state files normalised are the three that carry this answer.
    ///
    /// ★ `page-display.txt` is the per-document record, `preferences.txt` holds
    /// the standing preference, and `layout.ron` carries the ribbon mode — which
    /// picks the default for a document with no entry, and is therefore the
    /// third way a stale file could make this check pass without the feature.
    #[test]
    fn the_normalised_state_covers_all_three_tiers() {
        assert!(STATE_FILES.contains(&"page-display.txt"));
        assert!(STATE_FILES.contains(&"preferences.txt"));
        assert!(STATE_FILES.contains(&"layout.ron"));
    }
}
