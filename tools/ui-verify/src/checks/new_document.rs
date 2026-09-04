//! `new_document_makes_a_page` — File ▸ New makes a document, and the page
//! actually draws.
//!
//! # What this is for
//!
//! `file.new` is the first command in this shell that produces a document out
//! of **compiled-in bytes** rather than out of a file the operator named. Every
//! link in that chain has a unit test — the template parses
//! (`app::blank::tests`), the command registers, the arm raises `Action::New`,
//! `new_document` replaces the status — and **not one of them can observe the
//! chain being performed**. That is the same gap `markup_rectangle` was written
//! for and the same one `HANDOFF.md` §2 is a list of.
//!
//! It is also the check for a failure mode this project has hit twice and which
//! a unit test structurally cannot see: **a document that is open and will not
//! draw.** A blank page is the one document where "open" and "renders" are most
//! easily confused, because a blank page and a page that failed to rasterize
//! produce the same screenshot — which is why this check reads the canvas's own
//! `drawn=` count rather than looking at pixels. It is `HANDOFF.md` §2 defect 8
//! wearing different clothes: 2,450 hairlines and a wash are the same picture,
//! and so are a blank sheet and a sheet that did not render.
//!
//! # What it asserts, in order, and why each step is separate
//!
//! | # | Assertion | The failure it separates out |
//! |---|---|---|
//! | 0 | a document with **more than one page** is open first | without it, "New produced one page" is satisfied by New doing nothing to a one-page fixture |
//! | 1 | `ribbon.item.file.new` is declared | the command is registered and on no tab — a real state, and `super::manifest::CUSTOM_BACKED` exists because of it |
//! | 2 | the click produced a **new** `ribbon-command-invoked id=file.new` | the click missed, the control is disabled, or no pointer reached the window. **SKIP, never FAIL** — see below |
//! | 3 | a `new-document` line appeared | the command was invoked and the dispatch arm is missing. Distinguished from step 2 precisely because `command-unimplemented` is the shape this project keeps finding |
//! | 4 | `open ok pages=1 path="Untitled 1.pdf"` | `new_document` ran and the template did not become a document — a corrupt asset, or a `Status::Failed` |
//! | 5 | the canvas drew it: a `canvas` line with `pages=1 drawn=1` | the document is open and the page will not rasterize. **This is the "confirm it renders" step** |
//! | 6 | a second New makes `Untitled 2.pdf` | New is idempotent — it produced a document once and the control now does nothing, which steps 3–5 would all still pass |
//!
//! # ★ The falsifying phase: what wrong implementation would pass this?
//!
//! Asked before the check was written, per `checks/mod.rs`'s rule that a check
//! must fail against a build where the wiring is absent. Four answers, and the
//! first three are why the steps are shaped the way they are:
//!
//! 1. **A `file.new` with no dispatch arm.** `ribbon-command-invoked` still
//!    appears — the *shell* publishes it, before the application sees the token
//!    — so a check that stopped at step 2 would pass against a build in which
//!    pressing New does literally nothing and traces
//!    `command-unimplemented id=file.new`. Step 3 is what fails there, and the
//!    failure message says which of the two it is.
//! 2. **An `Action::New` matched *after* the document guard** in
//!    `app::actions::apply`, so it is silently dropped whenever nothing is
//!    open. Steps 3 and 4 fail. This one is not hypothetical: the guard exists,
//!    `Action::Open` and `Action::Close` are matched before it *for this
//!    reason*, and a fourth variant added below it would compile.
//! 3. **A New that opens a document nothing can draw.** Steps 3 and 4 pass —
//!    the status really is `Open` with one page — and the operator sees an
//!    empty canvas. Step 5 is the only assertion in the workspace that fails
//!    there, because no unit test in this project rasterizes.
//! 4. **A New that works once.** Every step above passes on the first press. It
//!    is step 6 that fails, and the reason it is worth a second click is that
//!    the ordinal is *application* state (`PdfcerApp::created_documents`) while
//!    everything else is per-document — a distinction whose whole failure mode
//!    is invisible until the second press.
//!
//! ## ★ It was actually run against such a build, on 2026-08-14
//!
//! Not as a thought experiment. `checks/mod.rs` says *"every check here has
//! been run against such a build and seen to fail; that is what
//! `PROJECT_PLAN.md` §4 stage S1's acceptance criterion asks for, and it is
//! not optional"*, so answer 1 above was performed: the single line
//! `"file.new" => actions.push(Action::New)` was deleted from
//! `app::dispatch`, the release binary rebuilt, and this check run against it.
//!
//! It reported **FAIL**, at step 3, with:
//!
//! > press 1: `file.new` was invoked and traced no new `new-document
//! > name="Untitled 1.pdf"`. The application traced
//! > `command-unimplemented id=file.new`, so the token arrived at
//! > `app::dispatch` and there is no arm for it — the fix is one match arm,
//! > not a wiring hunt. Documents it did report creating this run: none.
//!
//! Two things in that sentence are the point. It failed at **step 3 and not
//! step 2** — the click was reported as having reached the control, because it
//! had — and it named the *right file*. A check that had stopped at step 2
//! would have passed against that binary; a check whose failure said "New is
//! broken" would have sent a reader looking at the manifest, the registry and
//! the ribbon before reaching the one line that was missing.
//!
//! The arm was restored, the binary rebuilt, and the check re-run: PASS.
//!
//! And what this check would **not** catch, stated so nobody reads it as
//! covering more than it does:
//!
//! * **the page being the wrong size.** A Letter template would pass every
//!   assertion here. `app::blank::tests::the_template_page_is_a4` is what pins
//!   that, and it is a unit test because a page size is a *number in a file*
//!   and this harness's job is the things a number in a file cannot be.
//! * **the page being blank.** Nothing here reads a pixel. A template with a
//!   watermark on it would pass. `app::blank::tests` and the 443-byte size
//!   assertion are the guard against that, and
//!   `crates/pdfcer-gui/src/app/assets/PROVENANCE.md` is the reason it matters.
//! * **anything about saving it.** There is no save in this build, for any
//!   document — see `app::blank` §5.
//!
//! # Why step 2's failure is a SKIP
//!
//! `driving`'s own rule, and `find_bar`'s recorded incident: a check that could
//! not deliver a click has learned nothing about the application, and naming a
//! feature as the culprit when nothing was ever clicked at it is worse than no
//! check at all. That is why the invoke count is taken **before and after** the
//! click rather than asked as "is there a line for `file.new`?" — the second
//! press would otherwise be satisfied by the first press's line.
//!
//! # It uses the mouse only
//!
//! `Ctrl+N` is bound and is deliberately **not** what this check drives. Two
//! reasons, and the second is the operative one: the chord and the control
//! reach the identical dispatch arm, so driving both would test one thing
//! twice; and chord delivery is the part of this harness that has already
//! produced a false negative once (`find_bar`'s first run reported Find broken
//! on a build where Find worked). A check about a new capability should not
//! also be a bet on the least reliable channel. Whether `Ctrl+N` is *bound* is
//! pinned by `app::keyboard`'s
//! `every_chord_the_manifest_binds_can_be_spelled` family; whether it is
//! *delivered* is `find_bar`'s subject, not this one's.

use crate::checks::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, SHELL_TRACE_PREFIX, UNIMPLEMENTED_EVENT,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The command this check is about.
const SUBJECT_ID: &str = "file.new";

/// The region the ribbon publishes for its control.
const SUBJECT: &str = "ribbon.item.file.new";

/// `new-document name=… template-bytes=…` — `PdfcerApp::new_document`'s own line.
///
/// The event that separates "the command was invoked" from "the command ran",
/// which is the distinction step 3 exists for.
const NEW_EVENT: &str = "new-document";

/// The name the first created document of a session must have.
///
/// Spelled out rather than derived, because the derivation lives in
/// `crate::text::files::untitled` inside the application and a harness that
/// recomputed it would agree with a wrong implementation.
const FIRST_NAME: &str = "Untitled 1.pdf";

/// …and the second, which is step 6's whole subject.
const SECOND_NAME: &str = "Untitled 2.pdf";

pub struct NewDocumentMakesAPage;

impl Check for NewDocumentMakesAPage {
    fn name(&self) -> &'static str {
        "new_document_makes_a_page"
    }

    fn defect(&self) -> &'static str {
        "File ▸ New does not make a document, or makes one whose page will not draw"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;

    // A multi-page document is a PRECONDITION, not a convenience. "New produced
    // a one-page document" is satisfied by New doing nothing at all if what was
    // already open had one page — and the check would then pass against a build
    // where the control is inert.
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check proves that New REPLACED something, which needs something to \
             replace: with nothing open, `pages=1` afterwards is equally consistent with New \
             working and with New doing nothing. Pass a document with more than one page.",
        )
    })?;

    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is a click on a ribbon control. \
             Reported as SKIPPED rather than passed — a check that did not run has learned \
             nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("new_document.trace.txt"));
    spec.pdf = Some(pdf);
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
    // Long enough for the first page of a dense CAD sheet to raster. A canvas
    // read taken before the fixture has drawn would attribute the fixture's
    // slowness to the blank page.
    session.settle(60);

    // --- 0. the precondition: something with several pages is open ---------
    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and nothing below could be observed. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    let opened_pages = trace
        .events("open")
        .filter(|l| l.get("ok").is_some() || l.get("pages").is_some())
        .filter_map(|l| l.get_usize("pages"))
        .last();
    match opened_pages {
        Some(n) if n > 1 => {
            report.note(format!("the fixture opened with {n} pages"));
        }
        Some(n) => {
            return Err(Error::new(format!(
                "the fixture opened with {n} page(s). This check's whole evidence that New \
                 REPLACED the document is the page count changing to 1, and it cannot change to \
                 1 from 1. Pass a multi-page document."
            )));
        }
        None => {
            return Err(Error::new(
                "no `open` line with a page count, so nothing is known to be open and the \
                 replacement this check measures has no starting point."
                    .to_owned(),
            ));
        }
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- 1. is there a control at all? -------------------------------------
    //
    // A FAIL rather than a SKIP, and deliberately: a registered command that
    // appears on no tab is a real defect with a real name in this project's
    // vocabulary, and the reason lists what the File tab DID publish so the
    // reader can see whether the tab is the problem or the item is.
    let Some(rect) = driving::declared(&trace, ui_rect, SUBJECT) else {
        return Ok(Some(format!(
            "the application declared no `{SUBJECT}` region, so `{SUBJECT_ID}` has no control to \
             click. File is the tab pdfcer opens on, so this is not `the wrong tab is showing` — \
             it is a registered command with no reachable control, which is the state \
             `shell::manifest::CUSTOM_BACKED` exists to make deliberate. Regions it did declare \
             under `{ITEM_PREFIX}file.`: {}.",
            driving::list(&driving::declared_names(
                &trace,
                ui_rect,
                &format!("{ITEM_PREFIX}file.")
            ))
        )));
    };
    if !rect.is_substantial() {
        return Ok(Some(format!(
            "`{SUBJECT}` was declared at {rect:?}, which has no usable area — the control is \
             laid out and not on screen. Three panels in the old shell shipped with a body, a \
             rail entry and no control anyone could click, and passed every verification for \
             their whole shipped life."
        )));
    }
    report.note(format!(
        "`{SUBJECT}` occupies {:.1} x {:.1} pt",
        rect.width(),
        rect.height()
    ));

    // --- 2 and 3, twice: press New, and check what it did -------------------
    let first = press_new(&session, &driver, report, rect, 1)?;
    if let Some(failure) = first {
        return Ok(Some(failure));
    }

    // --- 6. and again, because "it worked once" is not "it works" -----------
    let second = press_new(&session, &driver, report, rect, 2)?;
    Ok(second)
}

/// One press of New, and everything that must follow it.
///
/// `ordinal` is which press this is, and it is what the expected document name
/// is derived from — so the second press asserting `Untitled 2.pdf` is the same
/// code path as the first asserting `Untitled 1.pdf`, rather than a
/// copy-and-pasted variant that could be weakened independently.
///
/// Returns `Ok(None)` for a clean press, `Ok(Some(_))` for a failure sentence,
/// and `Err` only for the states that are the *harness's* business — no click
/// delivered, no trace readable.
fn press_new(
    session: &Session,
    driver: &Driver,
    report: &mut CheckReport,
    rect: crate::geom::LRect,
    ordinal: usize,
) -> Result<Option<String>> {
    let expected = if ordinal == 1 {
        FIRST_NAME
    } else {
        SECOND_NAME
    };

    let invokes_before = driving::shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(SUBJECT_ID))
        .count();
    let news_before = session
        .trace()?
        .events(NEW_EVENT)
        .filter(|l| l.get("name") == Some(expected))
        .count();

    driver.click_at(session.frame()?.declared_center(rect))?;
    // Enough for the click, the action queue, the open and a raster of one
    // blank A4 page — which is the cheapest render this application ever does.
    session.settle(30);

    // --- 2. did the click reach the control? -------------------------------
    let invokes_after = driving::shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(SUBJECT_ID))
        .count();
    if invokes_after <= invokes_before {
        let shell = driving::shell_trace(session)?;
        return Err(Error::new(format!(
            "press {ordinal}: the click on `{SUBJECT}` produced no new `{INVOKE_EVENT} \
             id={SUBJECT_ID}` line, so no click reached the ribbon and nothing after it would \
             mean anything. Two readings, and this check declines to choose between them: the \
             pointer injection is not reaching this window, or the shell diagnostic switch \
             {}={} did not reach the process — the shell trace carries {} line(s) under \
             `{SHELL_TRACE_PREFIX}`. Trace: {}.",
            SHELL_DIAG_ENV.0,
            SHELL_DIAG_ENV.1,
            shell.lines.len(),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "press {ordinal}: the shell traced `{INVOKE_EVENT} id={SUBJECT_ID}`, so the click \
         reached the control"
    ));

    let trace = session.trace()?;

    // --- 3. did the command RUN? -------------------------------------------
    //
    // The step that separates an invoked command from an implemented one. The
    // `command-unimplemented` line is read only to say which of the two this
    // is; its presence and its absence send a reader to two different files.
    let news_after = trace
        .events(NEW_EVENT)
        .filter(|l| l.get("name") == Some(expected))
        .count();
    if news_after <= news_before {
        let unimplemented = trace
            .events(UNIMPLEMENTED_EVENT)
            .any(|l| l.get("id") == Some(SUBJECT_ID));
        let names: Vec<&str> = trace
            .events(NEW_EVENT)
            .filter_map(|l| l.get("name"))
            .collect();
        return Ok(Some(format!(
            "press {ordinal}: `{SUBJECT_ID}` was invoked and traced no new `{NEW_EVENT} \
             name={expected}`. {} Documents it did report creating this run: {}.",
            if unimplemented {
                format!(
                    "The application traced `{UNIMPLEMENTED_EVENT} id={SUBJECT_ID}`, so the \
                     token arrived at `app::dispatch` and there is no arm for it — the fix is \
                     one match arm, not a wiring hunt."
                )
            } else {
                "The application traced no `command-unimplemented` for it either, so the token \
                 reached an arm that did not do what the arm is for — look at \
                 `Action::New`'s position in `app::actions::apply`, which must be matched \
                 BEFORE the open-document guard."
                    .to_owned()
            },
            driving::list_str(&names)
        )));
    }
    report.note(format!(
        "press {ordinal}: the application traced `{NEW_EVENT} name={expected}`"
    ));

    // --- 4. did a document actually open, with one page? -------------------
    let opened = trace
        .events("open")
        .filter(|l| l.get("path") == Some(expected))
        .filter_map(|l| l.get_usize("pages"))
        .last();
    match opened {
        Some(1) => {
            report.note(format!(
                "press {ordinal}: `open ok pages=1 path={expected}` — the template became a \
                 document"
            ));
        }
        Some(n) => {
            return Ok(Some(format!(
                "press {ordinal}: New opened {expected} with {n} pages. The bundled template is \
                 a ONE-page document (`app::blank`), so this is either a different asset or a \
                 page tree read wrongly."
            )));
        }
        None => {
            return Ok(Some(format!(
                "press {ordinal}: the application said it was creating {expected} and no `open` \
                 line for it followed, so the compiled-in template did not become a document. \
                 `app::blank::document` returns the engine's own message in that case and \
                 `new_document` turns it into a Failed status — look for the shell's \
                 could-not-be-opened sentence on the canvas."
            )));
        }
    }

    // --- 5. ★ did the page DRAW? -------------------------------------------
    //
    // The step this check exists for. Everything above is satisfied by a
    // document that is open and blank on screen because it never rasterized,
    // and a screenshot cannot tell that from a blank page — which is the whole
    // reason the canvas publishes `drawn=` beside `pages=` rather than leaving
    // a reader to infer it.
    let Some(canvas) = trace.events("canvas").last() else {
        return Ok(Some(format!(
            "press {ordinal}: the document opened and the canvas traced no layout line at all, \
             so there is no evidence the page was ever laid out. `canvas-unavailable \
             reason=no-pages` would have been traced for a document with no pages; neither \
             appeared."
        )));
    };
    let pages = canvas.get_usize("pages").unwrap_or(0);
    let drawn = canvas.get_usize("drawn").unwrap_or(0);
    if pages != 1 {
        return Ok(Some(format!(
            "press {ordinal}: the canvas is laying out {pages} page(s) after New, not 1 — the \
             created document did not replace what was open."
        )));
    }
    if drawn == 0 {
        return Ok(Some(format!(
            "press {ordinal}: the canvas is laying out 1 page and reports `drawn=0`, so the \
             blank page is OPEN AND WILL NOT DRAW. A screenshot cannot tell this state from a \
             page that drew correctly and is blank, which is why the count is read instead."
        )));
    }
    report.note(format!(
        "press {ordinal}: the canvas drew the page (`pages=1 drawn={drawn}`)"
    ));

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::Trace;

    /// ★ **The expected names are the ones the application actually writes.**
    ///
    /// These constants used to carry the surrounding quotes, because `PathBuf`
    /// is traced through `{:?}` and the trace reads `name="Untitled 1.pdf"`.
    /// **`TraceLine::get` now strips a value's surrounding quotes**, so they are
    /// written bare and this test is what keeps the two in step: it parses a
    /// real quoted trace line and asserts the bare constant matches it.
    ///
    /// The change was forced by a chord spelled `[`. An unquoted `chord=[`
    /// opened a bracket the field splitter never saw closed, and swallowed every
    /// field after it on the line — so the application now quotes values that
    /// may contain structural characters, and `get` unwraps them, so that no
    /// caller has to know which values those are. Getting this backwards makes
    /// the check report New as broken on a build where it works, which is the
    /// exact false negative it was written to avoid.
    #[test]
    fn the_expected_names_survive_the_trace_quoting_them() {
        let trace = Trace::parse(
            "pdfcer-diag start argv1=None\n\
             pdfcer-diag new-document name=\"Untitled 1.pdf\" template-bytes=443\n\
             pdfcer-diag open ok pages=1 path=\"Untitled 1.pdf\"",
            "pdfcer-diag",
        );
        assert_eq!(
            trace
                .events(NEW_EVENT)
                .filter(|l| l.get("name") == Some(FIRST_NAME))
                .count(),
            1,
            "the constant must match the quoted form `{{:?}}` produces for a PathBuf"
        );
        assert_ne!(FIRST_NAME, SECOND_NAME);
    }

    /// A `canvas` line with a raster is told apart from one without.
    ///
    /// The distinction step 5 rests on, pinned against literal trace text so
    /// that a change to the canvas line's shape fails here rather than turning
    /// step 5 into an assertion that always reads `drawn=0`… which, being a
    /// FAIL, would at least be loud. The opposite — a parse that always yields
    /// a non-zero `drawn` — is the quiet one, so both are asserted.
    #[test]
    fn the_canvas_line_carries_whether_anything_was_rastered() {
        let with = Trace::parse(
            "pdfcer-diag canvas rect=[[0.0 0.0] - [10.0 10.0]] zoom=1.0000 page=0 pages=1 \
             off=[0.0 0.0] sel=0 display=single visible=1 drawn=1",
            "pdfcer-diag",
        );
        let line = with.events("canvas").last().expect("one canvas line");
        assert_eq!(line.get_usize("pages"), Some(1));
        assert_eq!(line.get_usize("drawn"), Some(1));

        let without = Trace::parse(
            "pdfcer-diag canvas rect=[[0.0 0.0] - [10.0 10.0]] zoom=1.0000 page=0 pages=1 \
             off=[0.0 0.0] sel=0 display=single visible=1 drawn=0",
            "pdfcer-diag",
        );
        let line = without.events("canvas").last().expect("one canvas line");
        assert_eq!(line.get_usize("drawn"), Some(0));
    }
}
