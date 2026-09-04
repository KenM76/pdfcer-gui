//! **The three checks about a recognition run while it is still running** —
//! progress, Stop, and Cancel.
//!
//! # ★★★ Why this file exists at all, when `checks::ocr` already drives OCR
//!
//! `checks::ocr` drives a **one-page** run and asserts its *result*: words came
//! back, the session took them, nothing was written to disk. Every one of those
//! is a statement about a run that has finished, and a one-page run has no
//! observable middle — it is started and then it is over.
//!
//! The operator's request of 2026-09-01 is entirely about the middle:
//!
//! > *"can you make it so the recognizing ocr gives feedback on what it is
//! > doing when it is running (pages done, words/characters detected, etc) so
//! > that the user can see that it is doing something and hasn't frozen on
//! > large documents? Maybe a cancel and stop button too. The cancel throws
//! > away what was done, and the stop finished the page it is on and keeps the
//! > work it has done."*
//!
//! Three separable claims, and **none of them can be observed on one page**:
//!
//! | claim | what a one-page run shows |
//! |---|---|
//! | the tally *advances* | one value, or none — a frozen label is identical |
//! | Stop keeps what was done | a Stop can only race the single page |
//! | Cancel discards what was done | there is nothing partial to discard |
//!
//! So these checks drive `fixtures/synthetic-image-only-8pages.pdf`, which
//! exists for exactly this and whose header
//! (`crates/pdfcer-gui/src/ocr/fixture.rs`) argues the eight.
//!
//! # ★★ What was already true, and why it was not enough
//!
//! The feature shipped on 2026-09-01 with unit tests over
//! `ocr::progress::Control` — including both orders of a Stop and a Cancel
//! arriving together, which is the sharp edge in the design. Those tests are
//! good and they are not the thing.
//!
//! **They call the verb. They cannot see the chain in front of it.** Between
//! `Control` and the operator there is a dialog that must *draw* the tally, a
//! frame loop that must keep *waking up* to redraw it, two buttons that must be
//! hit-testable, and a poll that must route three worker outcomes to three
//! different phases. Every one of those is off the verb, and eight green unit
//! tests say nothing about any of them. `OPERATOR_REQUESTS.md` O93 is marked
//! **not shipped** on precisely that reading of R1, and this file is what
//! closes it.
//!
//! # ★★★ The one that would have been missed, and it is not the buttons
//!
//! **A rect is not an oracle for "the user can see it is doing something".**
//!
//! `crate::diag::ui_rect("ocr-progress", …)` says a label was drawn somewhere.
//! A build whose progress label read `Page 1 of 8` for the entire run would
//! declare a perfectly substantial rect on every frame — and would be the exact
//! frozen application the request is about. So the shell now traces the
//! **numbers** as well (`ocr-progress attempted=… of=… words=… chars=…`, on
//! change), and [`OcrSaysHowFarItHasGot`] asserts that two of them differ.
//!
//! ★ Writing that check is also what found the defect underneath it. egui is
//! immediate-mode and idle: the OCR worker is on another thread and generates
//! no input events, so **nothing would have requested the next frame** and the
//! window would have held the frame it drew when the run started. It worked
//! anyway — because `egui::Spinner` calls `request_repaint()` for its own
//! animation (egui 0.35, `widgets/spinner.rs:40`). The entire visibility of
//! this feature rested on a side effect of a decorative widget. `dialogs::ocr`
//! now asks for the repaint explicitly and says why; anyone who later swaps the
//! spinner for a progress bar will not silently take live progress with it.
//!
//! # Which document these drive
//!
//! The eight-page synthetic fixture, always, by default — for the same reason
//! `checks::ocr` pins its own: a run needs pages with **no text on them**, and
//! on any document that already has text the doubling guard skips everything
//! and the honest answer to *"how far did it get"* is *"it declined to look"*.
//! A suite-wide `--pdf` is a drawing; a drawing is the wrong subject here.
//!
//! ★★ **`PDFCER_VERIFY_SCAN` overrides it, and should be used.** Point it at a
//! genuinely scanned, multi-page, text-free PDF and these checks run against
//! real material — scanner noise, skew, JPEG ringing, rotated pages, the lot.
//! The synthetic fixture is a *rendered* page and flatters the recogniser; it
//! establishes the plumbing and nothing about recognition quality, and every
//! report from these checks says so rather than leaving a green result to imply
//! otherwise.
//!
//! The operator's own material for this is the eight pages extracted from
//! `Parts Manual TH83 Telehandler.pdf` — an 883-page scanned parts manual, all
//! images, `/Rotate 270`, measured at **2.6 s and 440 recognised words per
//! page** through `pdfcer ocr`. See `OPERATOR_REQUESTS.md` O93.
//!
//! # Every way these report SKIP, and why none of them is a pass
//!
//! * no binary, no `--no-input`, no diagnostic channel — the harness never
//!   began;
//! * the fixture is missing;
//! * a tab, a mode segment or a control was never declared, or took no click;
//! * the model weights are not beside the binary — the application says so by
//!   name, and blaming the feature for a packaging step that was not run would
//!   be a false failure;
//! * **the run finished before the harness could reach the button.** Stop and
//!   Cancel are checks about interrupting something, and something that is
//!   already over cannot be interrupted. That is a property of the machine's
//!   speed on the day, not a defect, and it is reported as a SKIP that names
//!   the timing rather than as a pass that hides it.

use std::path::PathBuf;

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::ocr::{click_command, click_region, click_tab};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode the command is driven in. See `checks::ocr`'s [`READ`] for the
/// argument — OCR must be reachable in Read, and once was not.
///
/// [`READ`]: crate::checks::ocr
const READ: &str = "read";

/// The tab the command lives on.
const TAB: &str = "file";

/// The command's id, which is also its declared region's suffix.
const COMMAND: &str = "file.ocr";

/// The multi-page fixture, relative to the workspace root.
///
/// Generated by `crates/pdfcer-gui/src/ocr/fixture.rs`:
/// `cargo test -p pdfcer-gui --lib write_synthetic_image_only_multipage -- --ignored`
const FIXTURE: &str = "fixtures/synthetic-image-only-8pages.pdf";

/// How many pages [`FIXTURE`] has.
///
/// Used **only** to sanity-check the `of=` the application reports when the
/// default fixture is driven. Everything else derives the denominator from the
/// trace, so `PDFCER_VERIFY_SCAN` can point at a document of any length.
const FIXTURE_PAGES: usize = 8;

/// Environment override naming a real scanned document to drive instead.
///
/// An environment variable rather than a flag, deliberately: it is a property
/// of the *machine* — whether this box happens to have a scan on it — and not
/// of the run. A flag would have to be passed by every caller of the suite,
/// including the ones that have no such file, and would then be forgotten.
const SCAN_ENV: &str = "PDFCER_VERIFY_SCAN";

/// `ocr-progress attempted=… of=… words=… chars=…` — the live tally's content.
const PROGRESS_EVENT: &str = "ocr-progress";

/// `ocr-recognised pages=… skipped=… recognised=… dpi=…`
const RECOGNISED_EVENT: &str = "ocr-recognised";

/// `ocr-refused reason=…`
const REFUSED_EVENT: &str = "ocr-refused";

/// `ocr-cancelled attempted=…` — the operator threw the run away.
const CANCELLED_EVENT: &str = "ocr-cancelled";

/// `ocr-applied written=… skipped=… words=…` — the dialog raised the edit.
const APPLIED_EVENT: &str = "ocr-applied";

/// `ocr-layer page=… n=… …` — `vector_edit`'s record that the session took it.
const EDIT_EVENT: &str = "ocr-layer";

/// The progress region, so the check can assert the operator can SEE it.
const PROGRESS_REGION: &str = "ocr-progress";

/// The control that finishes the page in hand and keeps everything.
const STOP_REGION: &str = "ocr-stop";

/// The control that abandons the run.
const CANCEL_REGION: &str = "ocr-cancel";

/// How long a whole eight-page run may take, in settle frames.
///
/// A frame here is 25 ms (`Session::settle`), so 1,600 frames is **40
/// seconds**. Measured inputs: the synthetic page recognises in about a second
/// in a release build, and the operator's scanned parts manual measured 2.6 s a
/// page — so eight pages is 8–21 s and this is roughly twice the worst of
/// those.
///
/// ★ Generous on purpose, and the reasoning is `checks::ocr`'s: a budget that
/// was too short would report *"recognition never finished"* about a build that
/// was still working, which is the worst available failure message. This
/// harness also drives whichever binary it was pointed at, and a debug build is
/// twenty times slower.
const RUN_FRAMES: u32 = 1_600;

/// How long to wait for the FIRST progress line before giving up.
///
/// One page's worth plus a wide margin. If no page has finished in twelve
/// seconds the run is either refused, stuck, or being driven in a debug build,
/// and all three want a different message from "the tally did not advance".
const FIRST_PAGE_FRAMES: u32 = 480;

/// The polling granularity. Eight frames is 200 ms.
///
/// Small enough that a Stop lands with pages still to go on a one-second-a-page
/// run; large enough that the loop is not re-reading a growing trace file forty
/// times a second.
const SLICE: u32 = 8;

/// The workspace root, from this crate's manifest directory.
///
/// `tools/ui-verify/` → up two. Stable whatever the harness was invoked from
/// and whatever `--source-root` says — see `checks::ocr`'s `default_fixture`
/// for the two wrong ways this was done first, one of which overwrote the
/// repository's own fixture.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The document to drive: `PDFCER_VERIFY_SCAN` if it names a real file, else the
/// committed multi-page fixture.
///
/// Returns the path and whether it is the real-material one, because every
/// report says which it drove. A green result over the synthetic fixture and a
/// green result over a scanned manual are different amounts of evidence and
/// must not read the same.
fn document() -> (PathBuf, bool) {
    if let Some(from_env) = std::env::var_os(SCAN_ENV) {
        let path = PathBuf::from(from_env);
        if path.is_file() {
            return (path, true);
        }
    }
    (workspace_root().join(FIXTURE), false)
}

/// Poll the trace until `pred` holds, or `budget` frames are spent.
///
/// Returns whether it held. **Checks before it sleeps**, so a condition that is
/// already true costs nothing — which matters for the Stop and Cancel checks,
/// where the whole question is whether the harness got there in time.
fn wait_until(
    session: &Session,
    budget: u32,
    mut pred: impl FnMut(&Trace) -> bool,
) -> Result<bool> {
    let mut spent = 0;
    loop {
        if pred(&session.trace()?) {
            return Ok(true);
        }
        if spent >= budget {
            return Ok(false);
        }
        session.settle(SLICE);
        spent += SLICE;
    }
}

/// Every `attempted=` value the trace has reported, in order.
fn attempts(trace: &Trace) -> Vec<usize> {
    trace
        .events(PROGRESS_EVENT)
        .filter_map(|l| l.get_usize("attempted"))
        .collect()
}

/// The run's shared preamble: launch, reach `file.ocr` in Read, press
/// Recognise.
///
/// Everything up to the moment work starts is identical in all three checks,
/// and duplicating it three times would be three places for the ribbon path to
/// rot. `Err` is a SKIP in every caller.
///
/// Returns the session, the driver, and the resolved document with its
/// real-material flag.
fn start_a_run(
    ctx: &CheckContext,
    report: &mut CheckReport,
    trace_name: &str,
) -> Result<(Session, Driver, bool)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control and dialog \
             buttons. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;

    let (doc, real) = document();
    if !doc.is_file() {
        return Err(Error::new(format!(
            "the multi-page image-only fixture is not at {}. Generate it:\n  cargo test -p \
             pdfcer-gui --lib write_synthetic_image_only_multipage -- --ignored\nOr set \
             {SCAN_ENV} to a scanned, text-free, multi-page PDF.",
            doc.display()
        )));
    }
    report.note(format!(
        "driving {} — {}",
        doc.display(),
        if real {
            "REAL SCANNED MATERIAL, named by $PDFCER_VERIFY_SCAN. Recognition quality is \
             exercised here, not only the plumbing"
        } else {
            "the committed synthetic fixture. ★ It is a RENDERED page, not a scan: no scanner \
             noise, no skew, no JPEG ringing. This establishes the plumbing of a multi-page run \
             and establishes NOTHING about recognition quality. Set $PDFCER_VERIFY_SCAN to a \
             real scan to exercise that"
        }
    ));

    let mut spec = LaunchSpec::new(&exe, ctx.out(trace_name));
    spec.pdf = Some(doc);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // ★★★ **MAXIMIZE, and this line is a repair.**
    //
    // Found on 2026-09-01 by writing `checks::ocr_progress` and watching it
    // SKIP for a reason that turned out to apply to THIS check too: at the
    // window's default width the `file` tab's **Recognise group collapses**,
    // and a collapsed group declares `ribbon.group.file.recognise.collapsed`
    // instead of `ribbon.item.file.ocr`. The harness then reports *"the
    // application declared no `ribbon.item.file.ocr` region"* — which reads as
    // the command having been removed, and is in fact the ribbon doing exactly
    // what a ribbon is for.
    //
    // ★★ This check had therefore been reporting **SKIP** rather than PASS, and
    // a SKIP is not a failure, so nothing was red and nothing prompted a look.
    // `Session::maximize`'s own doc comment describes this precise symptom —
    // *"would have been handed ten controls ending at `file.print`, and would
    // have reported a shipped feature as missing"* — so the lesson was already
    // written down and this call site simply never got it.
    //
    // Opt-in per check by design, because a maximised window is a different
    // layout and several checks measure the layout. Nothing here does: this
    // check's subject is a command, a dialog and a file on disk.
    session.maximize();
    session.settle(12);

    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process and \
             this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    let driver = Driver::new(session.window());

    driving::click_mode_segment(&session, &driver, ui_rect, READ)?;
    click_tab(&session, &driver, ui_rect, TAB)?;
    click_command(&session, &driver, ui_rect, COMMAND)?;
    session.settle(16);

    let trace = session.trace()?;
    if driving::declared(&trace, ui_rect, "ocr-dialog").is_none() {
        return Err(Error::new(format!(
            "`{COMMAND}` opened no dialog, so there is no run to observe. That is \
             `checks::ocr`'s subject and it reports it as a FAILURE; here it is a SKIP, because \
             a check about a run in progress that never got a run has measured nothing. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    // ★ The scope is left at its default, which is **All pages**. That is both
    // what the operator asked for on 2026-08-26 and what these checks need: a
    // run over one page has no middle to observe. It is asserted rather than
    // assumed — `ocr-scope pages=` is the dialog's own report of what it
    // resolved, and a default that silently became "this page" would turn all
    // three of these checks into tests of nothing.
    let pages = trace
        .last("ocr-scope")
        .and_then(|l| l.get_usize("pages"))
        .unwrap_or(0);
    if pages < 2 {
        return Err(Error::new(format!(
            "the dialog resolved a scope of {pages} page(s). These checks are about a run with \
             a MIDDLE; a one-page run is started and then over, and Stop, Cancel and an \
             advancing tally are all unobservable across it. Either the fixture is not the \
             multi-page one or the default scope is no longer All pages."
        )));
    }
    report.note(format!("the dialog resolved a scope of {pages} page(s)"));
    if !real && pages != FIXTURE_PAGES {
        return Err(Error::new(format!(
            "the committed fixture resolved {pages} pages and it is built with \
             {FIXTURE_PAGES}. The file on disk is not the one the generator produces — \
             regenerate it, or a denominator these checks quote is a lie."
        )));
    }

    if driving::declared(&trace, ui_rect, "ocr-run").is_none() {
        return Err(Error::new(format!(
            "the dialog drew no `ocr-run` control, so it is refusing rather than offering to \
             recognise. The overwhelmingly likely cause is that the `models/ocrs` folder is not \
             beside {} — these checks need a PACKAGED build, or the weights copied next to the \
             executable.",
            exe.display()
        )));
    }
    click_region(&session, &driver, ui_rect, "ocr-run")?;
    Ok((session, driver, real))
}

/// A refusal, if the trace carries one. Every check treats it the same way:
/// a named refusal after a drawn Run button is a FAILURE, not a skip.
fn refusal(session: &Session) -> Result<Option<String>> {
    Ok(session.trace()?.last(REFUSED_EVENT).map(|l| l.raw.clone()))
}

// ---------------------------------------------------------------------------
// 1 — the tally advances
// ---------------------------------------------------------------------------

/// ★ **The operator can watch the run move.**
///
/// Runs to completion and asserts that at least two *different* `attempted`
/// values were reported, that they rose, that the last one equals the number of
/// pages in scope, and that words and characters were counted along the way.
///
/// See the module header for why a declared rect is not enough on its own.
pub struct OcrSaysHowFarItHasGot;

impl Check for OcrSaysHowFarItHasGot {
    fn name(&self) -> &'static str {
        "ocr_says_how_far_it_has_got_while_it_runs"
    }

    fn defect(&self) -> &'static str {
        "A multi-page recognition gives the operator no sign it is progressing — no tally, or a \
         tally that is drawn once and then never redrawn, which is pixel-identical to the \
         frozen application the feature exists to rule out"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_progress(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive_progress(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    let (session, _driver, real) = start_a_run(ctx, report, "ocr-progress.trace.txt")?;

    // Wait for the first page to finish. Until one has, the label is
    // deliberately not drawn: `Page 0 of 8 — 0 words` beside a spinner says
    // less than the spinner alone while looking like a stall.
    let saw_first = wait_until(&session, FIRST_PAGE_FRAMES, |t| {
        !attempts(t).is_empty() || t.last(REFUSED_EVENT).is_some()
    })?;
    if let Some(why) = refusal(&session)? {
        return Ok(Some(format!(
            "RECOGNITION REFUSED: `{why}`. The Run button was drawn — so preflight passed and \
             this build has both an engine and models — and the job then came back with a named \
             refusal, so there was never a run to report progress about."
        )));
    }
    if !saw_first {
        return Ok(Some(format!(
            "★ NO PROGRESS WAS EVER REPORTED. The run started and {FIRST_PAGE_FRAMES} frames \
             (about {} seconds) passed with no `{PROGRESS_EVENT}` line at all — not one page \
             finished, or a page finished and the dialog did not draw the tally.\n\n\
             This is the operator's own complaint, reproduced: *\"the user can see that it is \
             doing something and hasn't frozen\"*. Look at `dialogs::ocr`'s `Phase::Working` \
             arm and at `Job::tally`. Trace: {}.",
            u64::from(FIRST_PAGE_FRAMES) * 25 / 1000,
            session.trace_path().display()
        )));
    }

    // ★ The label must be ON SCREEN, not merely traced. A trace line proves the
    // shell computed a tally; the declared rect proves it drew one. The
    // request is about what the operator can SEE, so both are asserted — this
    // codebase has shipped a panel that existed and was unreachable, with every
    // gate green, and `D:/dev/rag/egui/` carries the finding.
    let trace = session.trace()?;
    let Some(rect) = driving::declared(&trace, ui_rect, PROGRESS_REGION) else {
        return Ok(Some(format!(
            "★ THE TALLY WAS COMPUTED AND NEVER DRAWN. `{PROGRESS_EVENT}` lines are in the \
             trace — so the shell knows how far it has got — and no `{PROGRESS_REGION}` region \
             was ever declared, which means no label reached the screen. Regions the dialog did \
             declare: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, "ocr-"))
        )));
    };
    if !rect.is_substantial() {
        return Ok(Some(format!(
            "the progress label was declared at {rect:?}, which has no usable area — it is on \
             screen in the sense that a zero-height label is on screen."
        )));
    }
    report.note(format!("the progress label is drawn at {rect:?}"));

    // Now let the run finish, and read the whole series.
    let finished = wait_until(&session, RUN_FRAMES, |t| {
        t.last(RECOGNISED_EVENT).is_some() || t.last(REFUSED_EVENT).is_some()
    })?;
    let trace = session.trace()?;
    if let Some(why) = refusal(&session)? {
        return Ok(Some(format!("RECOGNITION REFUSED MID-RUN: `{why}`.")));
    }
    if !finished {
        return Ok(Some(format!(
            "RECOGNITION NEVER FINISHED within {RUN_FRAMES} frames (about {} seconds). The last \
             tally reported was {:?}. A job that neither answers nor refuses leaves the dialog \
             saying `Recognising…` forever. Trace: {}.",
            u64::from(RUN_FRAMES) * 25 / 1000,
            attempts(&trace).last(),
            session.trace_path().display()
        )));
    }

    // ★★★ THE ASSERTION THIS CHECK IS FOR.
    //
    // Two distinct values, rising. One value is a label that was drawn and
    // never changed, which is the frozen-looking application. `dedup` on an
    // already-ordered series rather than a set, so the count is of *changes*.
    let series = attempts(&trace);
    let mut distinct = series.clone();
    distinct.dedup();
    if distinct.len() < 2 {
        return Ok(Some(format!(
            "★★★ THE TALLY NEVER MOVED. The whole run reported the single value {distinct:?}. \
             A progress line that is drawn once and never updated is not progress — it is a \
             still frame, and it is exactly what an operator reads as a frozen program.\n\n\
             The likeliest cause is that nothing requested the next frame. egui is \
             immediate-mode and idle; the OCR worker is on another thread and generates no \
             input events, so unless `dialogs::ocr`'s `Phase::Working` arm calls \
             `ctx.request_repaint()` the window holds whatever frame it drew when the run \
             started. It also worked, for a while, purely because `egui::Spinner` requests a \
             repaint for its own animation — so check whether the spinner is still there. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }
    if distinct.windows(2).any(|w| w[1] <= w[0]) {
        return Ok(Some(format!(
            "THE TALLY WENT BACKWARDS OR REPEATED A VALUE AFTER CHANGING: {distinct:?}. \
             `attempted` is a running count of pages finished and must be monotonically \
             increasing."
        )));
    }
    report.note(format!(
        "★ the tally advanced through {} distinct values: {distinct:?}",
        distinct.len()
    ));

    // ★★★ **The tally must reach the last page but ONE, and not the last one.**
    //
    // The strict `last == scope` was written first and it FAILED,
    // deterministically, at 7 of 8 — which is the correct behaviour, so the
    // assertion was the thing that was wrong. The reason is worth writing down
    // because it is not obvious and it will otherwise be rediscovered:
    //
    // `Job::poll` **drains** the channel — deliberately, so that several pages
    // finishing during one slow frame do not make the label lag the work by
    // exactly as long as the work takes. On the frame the run ends, that single
    // drain reads `Page(8)` *and* `Finished`, and returns the outcome.
    // `dialogs::ocr` leaves `Phase::Working` immediately, so the body never
    // draws a `Working` frame carrying `attempted = 8`. What the operator sees
    // is `Page 7 of 8` and then the outcome, which states the true totals
    // including page 8.
    //
    // ★ That is right, and it is what a progress display is supposed to do: be
    // superseded by its result rather than flash 100% on the way past. So this
    // is a **band**, not a widened tolerance — and it is still strong, because
    // what it is aimed at is a tally that goes quiet partway (2 of 8, 3 of 8),
    // which is symptom-identical to a freeze and is the whole subject here.
    let scope = trace
        .last("ocr-scope")
        .and_then(|l| l.get_usize("pages"))
        .unwrap_or(0);
    let last = distinct.last().copied().unwrap_or(0);
    let floor = scope.saturating_sub(1);
    if last < floor {
        return Ok(Some(format!(
            "THE TALLY STOPPED SHORT. The scope was {scope} page(s) and the last progress \r
             line reported {last} attempted, where at least {floor} was expected. \r
             Either pages were dropped from the run without being counted, or the \r
             tally stopped being published partway — a progress line that goes quiet \r
             before the end is the same failure as one that never starts, arriving \r
             later.

\r
             ★ {floor} rather than {scope} is not slack. The frame on which the last \r
             page finishes is the frame the run ends on, and `dialogs::ocr` leaves \r
             `Phase::Working` before drawing it — see this check's source for the \r
             drain that makes that deterministic rather than a race."
        )));
    }
    if last > scope {
        return Ok(Some(format!(
            "THE TALLY OVERRAN ITS OWN DENOMINATOR: {last} attempted of a scope of {scope}. \r
             A page was counted twice, which means `Job::poll` is accumulating \r
             `attempted` rather than assigning it."
        )));
    }
    report.note(format!(
        "and it got as far as {last} of {scope} — the last page's frame is the frame the \r
         run ends on, so the outcome supersedes the tally rather than the tally \r
         briefly reading {scope} of {scope}"
    ));

    // And it must have said what it FOUND, not only how far it had got — the
    // operator asked for "words/characters detected" by name.
    let last_line = trace
        .events(PROGRESS_EVENT)
        .last()
        .ok_or_else(|| Error::new("the progress series vanished between two reads"))?;
    let words = last_line.get_usize("words").unwrap_or(0);
    let chars = last_line.get_usize("chars").unwrap_or(0);
    if words == 0 || chars == 0 {
        return Ok(Some(format!(
            "THE TALLY COUNTED PAGES AND NOT CONTENT: `{}`. The request names \
             *\"words/characters detected\"* explicitly, and a page count alone does not \
             distinguish a recogniser that is reading from one that is walking pages and \
             finding nothing on any of them.",
            last_line.raw
        )));
    }
    report.note(format!(
        "and it reported what it found as it went: `{}`",
        last_line.raw
    ));
    if !real {
        report.note(
            "NOT established: recognition quality. The fixture is a rendered page. What is \
             established is that a multi-page run tells the operator where it has got to, in \
             numbers that move, on a window that keeps redrawing itself",
        );
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// 2 — Stop keeps the work
// ---------------------------------------------------------------------------

/// ★★ **Stop finishes the page in hand and keeps every page before it.**
///
/// The operator's wording: *"the stop finished the page it is on and keeps the
/// work it has done."* So the assertion is a conjunction, and both halves
/// matter: some pages were kept (`ocr-recognised pages=` is not zero, and the
/// session took a layer), **and** it really was an early end (fewer pages than
/// the scope).
pub struct StoppingOcrKeepsWhatItHasDone;

impl Check for StoppingOcrKeepsWhatItHasDone {
    fn name(&self) -> &'static str {
        "stopping_ocr_keeps_the_pages_it_had_already_done"
    }

    fn defect(&self) -> &'static str {
        "Stop throws away the recognition it was asked to keep, or does not stop at all — the \
         operator waits out a run they had ended, or loses the pages they had explicitly asked \
         for"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_ending(ctx, &mut report, Ending::Stop) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

// ---------------------------------------------------------------------------
// 3 — Cancel throws it away
// ---------------------------------------------------------------------------

/// ★★ **Cancel discards the run and touches the document not at all.**
///
/// The falsifying half of the pair. Every assertion in [`StoppingOcrKeepsWhatItHasDone`]
/// would be satisfied by a build in which Stop and Cancel were the same button;
/// this one fails against that build, because it asserts the **absence** of the
/// two lines the other one requires — no `ocr-applied`, no `ocr-layer`.
///
/// ★ That absence is asserted over the *whole* trace rather than over the tail,
/// deliberately. A Cancel that raised the edit and then tried to take it back
/// would leave both lines behind and an undo entry in the operator's stack,
/// which is not "nothing was kept".
pub struct CancellingOcrThrowsAwayWhatItHadDone;

impl Check for CancellingOcrThrowsAwayWhatItHadDone {
    fn name(&self) -> &'static str {
        "cancelling_ocr_throws_away_what_it_had_done"
    }

    fn defect(&self) -> &'static str {
        "Cancel keeps the partial recognition — the operator asked for none of it and gets a \
         half-recognised document, with a layer over some pages and not others and nothing to \
         say which"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_ending(ctx, &mut report, Ending::Cancel) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Which of the two early endings is being driven.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ending {
    /// Finish the page in hand, keep everything.
    Stop,
    /// Abandon the run, keep nothing.
    Cancel,
}

impl Ending {
    /// The region to click.
    fn region(self) -> &'static str {
        match self {
            Self::Stop => STOP_REGION,
            Self::Cancel => CANCEL_REGION,
        }
    }

    /// The operator's own word for it, for the report.
    fn word(self) -> &'static str {
        match self {
            Self::Stop => "Stop",
            Self::Cancel => "Cancel",
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive_ending(
    ctx: &CheckContext,
    report: &mut CheckReport,
    ending: Ending,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("no ui-rect event in this profile"))?;
    let trace_name = match ending {
        Ending::Stop => "ocr-stop.trace.txt",
        Ending::Cancel => "ocr-cancel.trace.txt",
    };
    let (session, driver, real) = start_a_run(ctx, report, trace_name)?;

    // ★★★ **Wait for a page to finish before pressing anything, and that is
    // not politeness — it is what makes the result mean something.**
    //
    // Pressing Stop before any page has completed makes "Stop keeps the work"
    // vacuous: there is no work. Pressing Cancel then makes "Cancel discards
    // the work" equally vacuous. Both would pass against a build in which the
    // two buttons were the same button, which is the exact confusion this pair
    // exists to rule out. So the harness waits for `attempted >= 1` and only
    // then reaches for the control.
    let ready = wait_until(&session, FIRST_PAGE_FRAMES, |t| {
        attempts(t).iter().any(|n| *n >= 1)
            || t.last(REFUSED_EVENT).is_some()
            || t.last(RECOGNISED_EVENT).is_some()
    })?;
    if let Some(why) = refusal(&session)? {
        return Ok(Some(format!(
            "RECOGNITION REFUSED: `{why}` — there was never a run to {}.",
            ending.word()
        )));
    }
    if !ready {
        return Ok(Some(format!(
            "NO PAGE FINISHED in {FIRST_PAGE_FRAMES} frames, so `{}` had nothing to act on. \
             That is `{}`'s subject; here it means this check could not be set up. Trace: {}.",
            ending.word(),
            OcrSaysHowFarItHasGot.name(),
            session.trace_path().display()
        )));
    }

    // ★ The run may have finished already on a fast machine and a short
    // document. That is a SKIP, and it is a real one: you cannot interrupt
    // something that is over, and calling it a pass would be this harness's own
    // worst outcome — a green result that measured nothing.
    let trace = session.trace()?;
    if trace.last(RECOGNISED_EVENT).is_some() {
        return Err(Error::new(format!(
            "the whole run finished before the harness could press {}. Nothing was \
             interrupted, so nothing about interrupting was measured. Drive a longer document \
             — set {SCAN_ENV} to a scanned PDF with more pages — or accept that this machine \
             recognises {} pages faster than this harness can react.",
            ending.word(),
            trace
                .last("ocr-scope")
                .and_then(|l| l.get_usize("pages"))
                .unwrap_or(0),
        )));
    }
    let before = attempts(&trace).last().copied().unwrap_or(0);
    let scope = trace
        .last("ocr-scope")
        .and_then(|l| l.get_usize("pages"))
        .unwrap_or(0);
    report.note(format!(
        "pressing {} with {before} of {scope} page(s) attempted",
        ending.word()
    ));

    if driving::declared(&trace, ui_rect, ending.region()).is_none() {
        return Ok(Some(format!(
            "★ THERE IS NO {} BUTTON WHILE THE RUN IS GOING. No `{}` region was declared on any \
             frame of a run that has been going for at least one page. The operator asked for \
             both controls by name and a run with no way out is the thing they were asking to \
             be rid of. Regions the dialog did declare: {}.",
            ending.word().to_uppercase(),
            ending.region(),
            driving::list(&driving::declared_names(&trace, ui_rect, "ocr-"))
        )));
    }
    click_region(&session, &driver, ui_rect, ending.region())?;

    // The stop flag is read between pages, so the wait is bounded by one page
    // plus the poll — generous margin, same reasoning as everywhere else here.
    let ended = wait_until(&session, FIRST_PAGE_FRAMES + SLICE * 8, |t| {
        t.last(RECOGNISED_EVENT).is_some()
            || t.last(CANCELLED_EVENT).is_some()
            || t.last(REFUSED_EVENT).is_some()
    })?;
    let trace = session.trace()?;
    if !ended {
        return Ok(Some(format!(
            "★ {} DID NOTHING. The button was clicked and the run neither ended nor \
             acknowledged it within {} seconds — more than one page's worth. The flag is read \
             between pages (`ocr::progress`), so a press that is never observed means the \
             worker is not reading it or the button is not setting it. Last tally: {:?}. \
             Trace: {}.",
            ending.word().to_uppercase(),
            u64::from(FIRST_PAGE_FRAMES + SLICE * 8) * 25 / 1000,
            attempts(&trace).last(),
            session.trace_path().display()
        )));
    }

    match ending {
        // ---------------------------------------------------------------
        Ending::Stop => {
            let Some(recognised) = trace.last(RECOGNISED_EVENT) else {
                return Ok(Some(format!(
                    "★★★ STOP THREW THE WORK AWAY. The run ended after the press and the \
                     trace carries {} rather than `{RECOGNISED_EVENT}`.\n\n\
                     These are not two names for one act. An operator who presses Stop on \
                     page 40 of 200 has ASKED FOR those forty pages; one who presses Cancel \
                     has asked for none of them. Collapsing Stop into Cancel discards work \
                     that was explicitly kept. Trace: {}.",
                    if trace.last(CANCELLED_EVENT).is_some() {
                        "`ocr-cancelled` — the CANCEL path"
                    } else {
                        "neither ending"
                    },
                    session.trace_path().display()
                )));
            };
            let kept = recognised.get_usize("pages").unwrap_or(0);
            if kept == 0 {
                return Ok(Some(format!(
                    "STOP KEPT NOTHING: `{}`. At least {before} page(s) had been attempted \
                     when the button was pressed, and none of them survived.",
                    recognised.raw
                )));
            }
            if kept >= scope {
                return Ok(Some(format!(
                    "★ STOP DID NOT STOP. `{}` reports {kept} page(s) written out of a scope \
                     of {scope}, so the run went to completion after the press. The result is \
                     correct and the control is inert — which the operator discovers by \
                     waiting out the twenty minutes they had just cancelled.",
                    recognised.raw
                )));
            }
            report.note(format!(
                "★ the run ended early and kept what it had: `{}` — {kept} of {scope} pages",
                recognised.raw
            ));

            // And the kept pages reached the SESSION. `ocr-applied` is the
            // dialog saying it asked; `ocr-layer` is `vector_edit` saying it
            // happened. Only the pair distinguishes "kept" from "said kept".
            if trace.last(APPLIED_EVENT).is_none() {
                return Ok(Some(format!(
                    "THE STOPPED RUN'S PAGES WERE NEVER APPLIED. `{}` reports {kept} page(s) \
                     and no `{APPLIED_EVENT}` line followed, so the dialog never raised the \
                     edit. Trace: {}.",
                    recognised.raw,
                    session.trace_path().display()
                )));
            }
            let Some(edit) = trace.last(EDIT_EVENT) else {
                return Ok(Some(format!(
                    "★ THE STOPPED RUN'S EDIT NEVER REACHED THE SESSION. `{APPLIED_EVENT}` \
                     was traced and no `{EDIT_EVENT}` followed. The operator's version of this \
                     state is a dialog reporting {kept} recognised pages over a document with \
                     no text in it. Trace: {}.",
                    session.trace_path().display()
                )));
            };
            report.note(format!("and the session took them: `{}`", edit.raw));
        }
        // ---------------------------------------------------------------
        Ending::Cancel => {
            let Some(cancelled) = trace.last(CANCELLED_EVENT) else {
                return Ok(Some(format!(
                    "★★★ CANCEL KEPT THE WORK. The run ended after the press and the trace \
                     carries {} rather than `{CANCELLED_EVENT}`.\n\n\
                     Cancel means the document is untouched. A build that applies the pages \
                     done so far leaves a half-recognised document — a layer over some pages \
                     and not others, invisible by design (Table 106 mode 3), with nothing on \
                     screen to say which. Trace: {}.",
                    if trace.last(RECOGNISED_EVENT).is_some() {
                        "`ocr-recognised` — the STOP path"
                    } else {
                        "neither ending"
                    },
                    session.trace_path().display()
                )));
            };
            report.note(format!("the run was abandoned: `{}`", cancelled.raw));

            // ★★★ The falsifying assertion, and it is an ABSENCE over the whole
            // trace. See the type's doc comment.
            let applied = trace.events(APPLIED_EVENT).count();
            let edits = trace.events(EDIT_EVENT).count();
            if applied > 0 || edits > 0 {
                return Ok(Some(format!(
                    "★★★ CANCEL APPLIED A LAYER ANYWAY. The trace carries {applied} \
                     `{APPLIED_EVENT}` line(s) and {edits} `{EDIT_EVENT}` line(s) across a run \
                     the operator abandoned. `Cancel` must touch the document not at all — not \
                     apply-then-undo, which leaves an entry in their undo stack and a document \
                     that was momentarily something they did not ask for. Trace: {}.",
                    session.trace_path().display()
                )));
            }
            report.note(
                "★ and nothing was applied — no `ocr-applied` and no `ocr-layer` anywhere in \
                 the run. The document is exactly as it was opened",
            );
        }
    }

    if !real {
        report.note(
            "driven on the synthetic fixture. The ENDING is what is established here and it is \
             document-independent; recognition quality is not, and is not claimed",
        );
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture these drive is the multi-page image-only one.
    ///
    /// Pinned because both properties are load-bearing and each fails silently
    /// on its own: a single-page fixture makes all three checks vacuous, and a
    /// fixture with text makes the doubling guard skip every page so the run
    /// reports nothing and the tally never moves — which reads as the very
    /// defect being looked for.
    #[test]
    fn the_fixture_is_multi_page_and_image_only() {
        assert!(
            FIXTURE.contains("image-only"),
            "{FIXTURE} must have no text"
        );
        assert!(
            FIXTURE.contains("8pages"),
            "{FIXTURE} must be the multi-page one — a one-page run has no middle to observe"
        );
        const { assert!(FIXTURE_PAGES > 1) };
    }

    /// Every trace event read here is spelled once and is an OCR event.
    #[test]
    fn the_event_names_are_distinct_and_all_ocr() {
        let all = [
            PROGRESS_EVENT,
            RECOGNISED_EVENT,
            REFUSED_EVENT,
            CANCELLED_EVENT,
            APPLIED_EVENT,
            EDIT_EVENT,
        ];
        for (i, a) in all.iter().enumerate() {
            assert!(a.starts_with("ocr-"), "{a} is not an OCR event");
            for b in &all[i + 1..] {
                assert_ne!(a, b, "two constants name the same event");
            }
        }
    }

    /// ★ **Stop and Cancel click different controls.**
    ///
    /// The one assertion in this file that could not be got wrong by accident
    /// and would invalidate everything if it were: the pair of checks is a pair
    /// precisely because the two buttons must not be the same button, and a
    /// harness that clicked one region for both would report that they behave
    /// identically — correctly, and about itself.
    #[test]
    fn the_two_endings_reach_two_different_controls() {
        assert_ne!(Ending::Stop.region(), Ending::Cancel.region());
        assert_ne!(Ending::Stop.word(), Ending::Cancel.word());
        assert_eq!(Ending::Stop.region(), STOP_REGION);
        assert_eq!(Ending::Cancel.region(), CANCEL_REGION);
    }

    /// The budgets are ordered: a whole run may take longer than one page.
    ///
    /// An inversion here would make the run budget expire before the first-page
    /// budget and report "recognition never finished" on every run.
    #[test]
    fn the_waiting_budgets_are_ordered() {
        const {
            assert!(
                RUN_FRAMES > FIRST_PAGE_FRAMES,
                "a whole run must be allowed more time than its first page"
            );
        };
        const { assert!(SLICE > 0 && SLICE < FIRST_PAGE_FRAMES) };
    }

    /// With no override, the document is the committed fixture.
    ///
    /// ★ Deliberately does not test the override branch: setting a process-wide
    /// environment variable from a test races every other test in the binary,
    /// and this crate runs its tests in threads. The branch is three lines and
    /// its risk is a typo in the variable name, which this pins instead.
    #[test]
    fn the_default_document_is_the_committed_fixture() {
        assert_eq!(SCAN_ENV, "PDFCER_VERIFY_SCAN");
        if std::env::var_os(SCAN_ENV).is_none() {
            let (path, real) = document();
            assert!(!real);
            assert!(path.ends_with(FIXTURE.rsplit('/').next().unwrap()));
        }
    }
}
