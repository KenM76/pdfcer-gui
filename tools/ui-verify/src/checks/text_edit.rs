//! `text_edit_pins_an_aligned_tail` — **the edit reached the bytes, and the text
//! the operator did not touch did not move**, proved by re-opening the written
//! file in a second process.
//!
//! # What this is for
//!
//! `DEFECTS.md` **D4** is the defect that began this project. Its D4b half names
//! two cases that are not merely unhelpful but **wrong on commit**, and both are
//! about text the operator never touched: a right-aligned tail pushed off the
//! edge it is flush against, and a rotated line's tail slid along user-space x
//! when its baseline runs up the page.
//!
//! `canvas::textedit::proof` already asserts both against the written bytes, in
//! process, with the old shell's own `EditOptions::default()` executed beside
//! each as the falsifier. **This check asks a different question**: does the
//! *operator's route* reach that arithmetic at all — a mode, a ribbon control, a
//! click on the page, a commit, a save, and a file somebody else can open.
//!
//! `save_copy`'s header lists the five links a unit test cannot see, and this
//! adds two more that are specific to typing:
//!
//! | # | Link | Its own test |
//! |---|---|---|
//! | 1 | the ribbon control arms the caret tool | `shell::commands` — the registration, not the arm |
//! | 2 | a click on the page resolves a **run** | nothing: it needs a rendered page and a real hit test |
//! | 3 | a draft reaches the commit | nothing: the typing loop is `egui::Event` handling |
//! | 4 | the commit plans the **right disposition** | `textedit::proof` — the arithmetic, not the route |
//! | 5 | the engine writes it | `pdfcer-core` |
//! | 6 | it is in the saved file | `save_copy` — for an annotation, not for a text edit |
//! | 7 | a **second process** reads the new text back | nothing |
//!
//! # ★★ NOTHING IS TYPED AND NO KEY IS PRESSED, and both facts are findings
//!
//! **Text cannot be injected on this machine.** `crate::sys::vk` is a closed
//! list of eight non-character keys whose own comment refuses to grow into
//! `pub const A..Z`.
//!
//! ★★ **The paragraph that used to be here was WRONG, and its wrongness is the
//! most instructive thing in this file.** It said *keyboard input does not reach
//! the target window from the session that injects it on this machine*, and
//! called that **re-confirmed on 2026-08-15**: a first cut pressed `Ctrl+E` to
//! arm the tool, the trace carried no `text-edit-tool` line, and a pointer click
//! on the same command armed it.
//!
//! `Ctrl+E` was **dead in the dispatcher**. It was one of fourteen chords the
//! manifest declared and `app::keyboard::commands` never dispatched, because it
//! matched against a hand-written table of eight spellings. The experiment was
//! sound and the conclusion was drawn one layer too low: *this chord does
//! nothing* became *this machine cannot type*.
//!
//! That reading was then written into NINE module headers as a fact about the
//! environment. While it stood, no check drove a chord; because no check drove a
//! chord, nothing contradicted it; and the dead chords stayed dead. **A
//! misdiagnosis recorded as fact protected the defect that produced it.**
//! `crate::checks::add_text` types real characters and passes.
//!
//! The seam below is kept anyway — it supplies a *known* string, which is worth
//! having — but it is a convenience now, not a workaround.
//!
//! Typing is this feature's entire input, so a check that could supply no text
//! would be reduced to asserting *"the tool armed"* — `HANDOFF.md` §2's grid
//! lesson exactly, an assertion in the right direction that measures the wrong
//! thing. So the draft's characters arrive through `PDFCER_DIAG_TYPE`, a seam in
//! the shape of `PDFCER_DIAG_OPEN_PATH` and `PDFCER_DIAG_SAVE_PATH` — both of
//! which exist because a native modal cannot be driven from here. What the seam
//! **does not** replace is any other link: the mode still has to change, the
//! tool still has to arm from a real ribbon click, the click still has to
//! resolve a run through the real hit test, the commit still has to plan the
//! disposition, the engine still has to write, and the file still has to be
//! readable by a second process. It supplies characters and nothing else, and
//! pushes them through the same `insert` a keystroke would.
//!
//! **And no key is pressed at all — not even Enter.** The commit is reached by
//! *clicking somewhere else*, which is the shell's own rule: `textedit::click`
//! commits an existing draft before starting a new one, because clicking away is
//! every editor's "that word is finished". The pointer-only path an un-typeable
//! machine forces on this check is also the path a mouse-driven operator takes.
//!
//! # The phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | launch on `fixtures/tail-alignment.pdf`, click **Edit** mode | `ribbon-mode-selected mode=edit` |
//! | B | click Edit ▸ **Edit text** | `text-edit-tool tool=TextEdit(Edit)` |
//! | C | click the right-aligned line's first word | `text-edit-caret … run=N` |
//! | D | click blank paper | `text-edit-plan … disposition=Pin reason=Flush(Right)`, `edit-text` |
//! | E | File ▸ **Save a copy** | `save-copy …`, a file at the named path |
//! | F | read the copy | the source's bytes are its prefix, verbatim (§7.5.6) |
//! | G | ★ scan the **appended** revision | the untouched line's `Tm` is there **verbatim** |
//! | H | ★ launch a **second process** on the copy | it opens and draws the edited page |
//!
//! # ★ Why phase G is the verdict and phase H cannot replace it
//!
//! Phase H proves the copy opens. It cannot see the defect at all: a
//! build that pushed the two untouched lines sideways would still have written
//! the new word, and the second process would still read it back. Only G looks
//! at the operator that was supposed to be left alone.
//!
//! And G has to scan the **appended** bytes rather than the file, which is the
//! subtlety that would otherwise make it a false pass. §7.5.6 forbids an
//! incremental update from rewriting the base revision, so the original `Tm` is
//! still in the first `source.len()` bytes of *every* build's output. A scan
//! over the whole file answers "unmoved" for a correct build and a broken one
//! alike.

use std::path::Path;

use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::save_copy::{FILE_TAB, SAVE, click_command, click_tab};

/// The mode this check has to be in.
///
/// `edit.text` is gated on `Capabilities::edit_content`, which the shipped
/// manifest gives to Edit alone — and the application opens in **Read** (its
/// remembered default), so a check that did not switch would be measuring the
/// mode gate rather than the tool. That is exactly what this check's first run
/// did: `command-declined id=edit.text reason=mode-cannot-edit-content`.
const MODE: &str = "edit";

/// The Edit tab, and the command that arms the caret tool.
const EDIT_TAB: (&str, &str) = ("ribbon.tab.edit", "edit");
/// The control under test's own ribbon region and id.
const EDIT_TEXT: (&str, &str) = ("ribbon.item.edit.text", "edit.text");

/// Where the copy goes — the seam every other write-driving check uses.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";

/// ★ The seam that supplies the draft. See this module's header.
const TYPE_ENV: &str = "PDFCER_DIAG_TYPE";

/// What is typed. Deliberately **longer** than what it replaces: the whole
/// defect is what happens to the followers when the advance delta is positive,
/// and a same-width replacement would make `Pin` and `Reflow` produce identical
/// bytes — a fixture flattering the thing it measures.
const TYPED: &str = "REVISION BBBB";

/// The word the caret is aimed at.
const REPLACED: &str = "REVISION B";

/// Blank paper, as a fraction of the page box — clear of all three text objects
/// the generator places. Clicking here **commits** the draft, which is the
/// shell's own click-away rule and this check's only route to a commit without a
/// keystroke.
const ELSEWHERE: (f64, f64) = (300.0 / 612.0, 600.0 / 792.0);

/// ★ The untouched operator. `tools/gen-textedit-fixtures.py` places the block's
/// third line at this exact `Tm`, and prints the number when it runs. Under
/// `Pin` it is re-emitted verbatim; under `Reflow` its `e` gains the advance
/// delta and this string is gone.
const UNTOUCHED_TM: &str = "412.64 668.00 Tm";

/// Where on the page to click, as a fraction of the page box.
///
/// The middle of `REVISION B`, which the generator reports as spanning
/// x = 431.31…500.00 at y = 700 on a 612 x 792 page. Written as fractions
/// because `crate::checks`' rules allow a check only `DocPoint` and `FracRect`
/// literals — a screen coordinate would be a number about this machine.
const AIM: (f64, f64) = (465.0 / 612.0, 700.0 / 792.0);

/// See the module documentation.
pub struct TextEditPinsAnAlignedTail;

impl Check for TextEditPinsAnAlignedTail {
    fn name(&self) -> &'static str {
        "text_edit_pins_an_aligned_tail"
    }

    fn defect(&self) -> &'static str {
        "Text editing does not reach the page at all, or it reaches it and drags the text the \
         operator did not touch with it — DEFECTS.md D4b's right-aligned and rotated cases, \
         which the old shell got wrong because its only call site passed EditOptions::default() \
         and never asked the page anything"
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

/// Launch the binary on `pdf`, with the three seams set.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &Path,
    target: Option<&Path>,
    trace_name: &str,
) -> Result<Session> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let mut spec = LaunchSpec::new(exe, ctx.out(trace_name));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    if let Some(t) = target {
        spec.env
            .push((SAVE_PATH_ENV.to_owned(), t.display().to_string()));
        // The seam is set only on the authoring process. The second process
        // must read the file back with nothing supplying it text, or phase G
        // would be reading a draft rather than a document.
        spec.env.push((TYPE_ENV.to_owned(), TYPED.to_owned()));
    }
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched on {} as pid {}",
        pdf.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the process launched and never traced `{}`. Nothing below can be attributed to the \
             feature. Trace: {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    Ok(session)
}

/// Whether `hay` holds `needle`.
fn holds(hay: &[u8], needle: &str) -> bool {
    hay.windows(needle.len()).any(|w| w == needle.as_bytes())
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is six clicks across two processes. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    // ★ NOT `ctx.pdf`. This check's verdict is a byte scan for a `Tm` this
    // repository's own generator placed, so it is meaningless against any other
    // document — and a check that silently measured whatever `--pdf` named would
    // be measuring a different claim under the same name. `redaction` makes the
    // same decision for the same reason and generates its own fixture.
    let pdf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tail-alignment.pdf");
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture is missing at {}. Regenerate it: python \
             tools/gen-textedit-fixtures.py",
            pdf.display()
        )));
    }
    let page = PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    };
    let source = std::fs::read(&pdf)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", pdf.display())))?;
    report.note(format!(
        "fixture {} — {} bytes, page 1 is 612 x 792 pt",
        pdf.display(),
        source.len()
    ));

    let target = ctx.out("text-edit-round-trip.pdf");
    let _ = std::fs::remove_file(&target);

    // =======================================================================
    // The authoring process. Scoped, so it is gone before the second launches.
    // =======================================================================
    {
        let session = launch(ctx, report, &pdf, Some(&target), "text_edit.trace.txt")?;
        let driver = Driver::new(session.window());

        // --- PHASE A: Edit mode --------------------------------------------
        let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
            Error::new(format!(
                "the `{}` profile declares no ui-rect trace event, so the application cannot \
                 state where its controls are and this check has nothing to aim at.",
                ctx.profile.name
            ))
        })?;
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);

        // --- PHASE B: arm the caret tool from the ribbon --------------------
        click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
        click_command(&session, &driver, ui_rect, EDIT_TEXT, 16)?;
        let trace = session.trace()?;
        if !trace
            .events("text-edit-tool")
            .any(|l| l.get("tool") == Some("TextEdit(Edit)"))
        {
            return Ok(Some(format!(
                "EDIT TEXT WAS INVOKED AND THE CANVAS TOOL DID NOT ARM. The shell traced the \
                 ribbon invoke and there is no `text-edit-tool tool=TextEdit(Edit)`, so the \
                 dispatch arm declined (a `command-declined id=edit.text \
                 reason=mode-cannot-edit-content` line would say so, and this check is in \
                 `{MODE}`) or `canvas::tool::arm_text_edit` was not reached. Trace: {}.",
                session.trace_path().display()
            )));
        }
        report.note("the ribbon control armed the caret tool".to_owned());

        // --- PHASE C: click the right-aligned line's first word -------------
        let at = crate::checks::text_selection::aim(
            ctx,
            &session,
            page,
            DocPoint::new(0, AIM.0 * page.width_pt, AIM.1 * page.height_pt),
        )?;
        driver.click_at(at)?;
        session.settle(20);
        let trace = session.trace()?;
        let Some(caret) = trace.last("text-edit-caret") else {
            return Ok(Some(format!(
                "THE CLICK PLACED NO CARET. The tool armed and a click on `{REPLACED}` produced \
                 no `text-edit-caret` line. Either the press was not routed to \
                 `canvas::textedit::click` (check `gesture::press_kind`'s caret rung and \
                 `canvas::interact`'s click routing) or the hit test found no run — a \
                 `text-edit-declined` line would say which. Trace: {}.",
                session.trace_path().display()
            )));
        };
        report.note(format!("the click placed a caret: `{}`", caret.raw));
        if trace.last("text-edit-seeded").is_none() {
            return Err(Error::new(format!(
                "`{TYPE_ENV}` was set and no `text-edit-seeded` line followed the caret, so the \
                 draft is empty and the commit below would correctly be a no-op. SKIPPED \
                 rather than failed: this is the harness's own seam, not the application's \
                 feature. Trace: {}.",
                session.trace_path().display()
            )));
        }

        // --- PHASE D: click elsewhere, which commits ------------------------
        let away = crate::checks::text_selection::aim(
            ctx,
            &session,
            page,
            DocPoint::new(0, ELSEWHERE.0 * page.width_pt, ELSEWHERE.1 * page.height_pt),
        )?;
        driver.click_at(away)?;
        session.settle(24);
        let trace = session.trace()?;
        let Some(plan) = trace.last("text-edit-plan") else {
            return Ok(Some(format!(
                "CLICKING AWAY COMMITTED NOTHING. A caret was placed, `{TYPE_ENV}` seeded it \
                 with {TYPED:?}, and a click on blank paper produced no `text-edit-plan` — \
                 so `canvas::textedit::click`'s commit-the-existing-draft rule did not fire. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        };
        report.note(format!("the commit planned: `{}`", plan.raw));

        // ★★ The disposition, named. This is the assertion a build with the fix
        // reverted fails: it would plan `Reflow` here, and everything else in
        // this check would still pass except phase G.
        if plan.get("disposition") != Some("Pin") {
            return Ok(Some(format!(
                "★ THE WRONG FOLLOWER DISPOSITION. This block is right-aligned, so the tail \
                 must be pinned — `DEFECTS.md` D4b case 1, and `FollowerDisposition::Pin`'s \
                 own doc comment says it exists \"for a justified / right-aligned tail that \
                 must not move\". The shell planned `{}` (reason `{}`). That is what the \
                 old shell did at its only call site, and it is the defect.",
                plan.get("disposition").unwrap_or("nothing"),
                plan.get("reason").unwrap_or("none")
            )));
        }
        if trace.events("edit-text").count() == 0 {
            return Ok(Some(format!(
                "THE ENGINE NEVER RAN. A plan was made and no `edit-text` line followed, so \
                 `vector_edit` declined — an `edit-text-refused … reason=session-borrowed` \
                 line would say so. Trace: {}.",
                session.trace_path().display()
            )));
        }

        // --- PHASE E: save a copy -------------------------------------------
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        click_command(&session, &driver, ui_rect, SAVE, 40)?;
        if session.trace()?.events("save-copy").count() == 0 {
            return Ok(Some(format!(
                "SAVE A COPY WROTE NOTHING. The command was invoked and no `save-copy` line \
                 followed. That is `save_copy`'s subject rather than this one's — read its \
                 verdict first. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }

    // --- PHASE F + G: the file ---------------------------------------------
    let copy = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "the shell traced a save and there is no file at {}: {e}",
            target.display()
        ))
    })?;
    if copy.len() <= source.len() || copy[..source.len()] != source[..] {
        return Ok(Some(format!(
            "THE SAVE WAS NOT AN INCREMENTAL UPDATE. §7.5.6 forbids rewriting the base revision, \
             and `file.save_copy`'s own tooltip promises the previous revision \"stays intact \
             inside the file\" — the copy is {} bytes against a source of {}, and the first {} \
             do not match.",
            copy.len(),
            source.len(),
            source.len()
        )));
    }
    let appended = &copy[source.len()..];
    report.note(format!(
        "the copy is {} bytes, of which {} were appended",
        copy.len(),
        appended.len()
    ));

    if !holds(appended, TYPED) {
        return Ok(Some(format!(
            "THE TYPED TEXT IS NOT IN THE APPENDED REVISION. {TYPED:?} does not appear in the \
             {} bytes the save appended, so whatever was written, it was not this edit.",
            appended.len()
        )));
    }
    // ★★★ THE VERDICT.
    if !holds(appended, UNTOUCHED_TM) {
        return Ok(Some(format!(
            "★ THE UNTOUCHED LINE MOVED — `DEFECTS.md` D4b case 1, in the bytes. The fixture's \
             third right-aligned line sits at `{UNTOUCHED_TM}` and the operator did not touch it; \
             the appended revision does not contain that operator, so it was re-emitted with a \
             different translation. A right-aligned tail that moves is the defect this whole \
             piece of work is about."
        )));
    }
    report.note(format!(
        "★ the untouched line's operator `{UNTOUCHED_TM}` is in the appended revision verbatim"
    ));

    // --- PHASE H: a second process reads it back ---------------------------
    let second = launch(ctx, report, &target, None, "text_edit.reopen.trace.txt")?;
    let trace = second.trace()?;
    if trace.events(ctx.profile.vocab.canvas_event).count() == 0 {
        return Ok(Some(format!(
            "THE COPY DID NOT OPEN IN A SECOND PROCESS. It exists and carries the edit, and a \
             fresh pdfcer could not draw it — which is a worse outcome than a failed save. Trace: \
             {}.",
            second.trace_path().display()
        )));
    }
    report.note(format!(
        "a second process (pid {}) opened the copy and drew it",
        second.pid()
    ));
    Ok(None)
}
