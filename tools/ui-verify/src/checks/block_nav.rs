//! `arrow_keys_walk_between_blocks` — **the cursor moves to the next block of
//! text**, driven on a real drawing.
//!
//! # What this is for
//!
//! The operator, 2026-08-21:
//!
//! > *"there was an acrobat feature in the original pdfcer-gui that attempted to
//! > reassemble individual lines into paragraphs and the cursor would move to
//! > the next block of text using the navigation keys."*
//!
//! It is **salvage**: the shell this project replaces did it, and this one had
//! never bound Up or Down at all — its caret is a character index into one run,
//! and a single run has no line above it.
//!
//! # ★★ Why the assertion is a CHANGE OF RUN and not a caret movement
//!
//! Because those are different facts and only one of them is the feature. A
//! build that moved the caret within the run it was already in would look
//! identical from the outside — the caret moves, the draft changes, a trace line
//! appears — and would have done nothing the operator asked for.
//!
//! So `text-caret-step` carries **both** run indices and this check asserts they
//! differ. That is `DEFECTS.md` D14's rule applied to a navigation key: *a trace
//! line must carry the number a wrong build would get wrong.*
//!
//! # ★ Why a real drawing, and why this check would pass vacuously on a fixture
//!
//! The whole point is crossing from one recognised block to another, which needs
//! a page with **more than one line of text in more than one place**. This
//! project's own fixtures are one-run pages and blank sheets by construction —
//! and `FEATURES.md` records what that cost the last time it was forgotten:
//!
//! > *"a check that drives a document this project authored tests the shape this
//! > project imagined, and the operator's documents are the only ones with the
//! > shape that broke."*
//!
//! A page with only one line of text is a fact about the fixture, so this check
//! **skips** on it rather than passing.
//!
//! # The chain
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | a click on real text opens a draft anchored to a run | `text_edit_on_a_real_drawing` |
//! | 2 | Up/Down reach `typing`'s arrow arm at all | nothing — they were unbound until today |
//! | 3 | the character index survives the hop to a byte offset and back | `blocks` — on `"café"`, and not on a page |
//! | 4 | **the caret lands in a DIFFERENT run** | nothing |
//! | 5 | the draft it left is committed rather than discarded | nothing |
//!
//! Link 5 is the quiet one: a caret that leaves a run with unsaved keystrokes in
//! it would silently drop them, which is this project's defining defect class.
//! It is not asserted here — the check does not type before navigating,
//! deliberately, because doing so would put an edit on the operator's document
//! to prove a navigation. Named rather than claimed.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::save_copy::{click_command, click_tab};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas may edit content.
const MODE: &str = "edit";
/// The Edit tab, where the text control lives.
const EDIT_TAB: (&str, &str) = ("ribbon.tab.edit", "edit");
/// The control that arms the caret on existing text.
const TOOL: (&str, &str) = ("ribbon.item.edit.text", "edit.text");
/// `text-edit-caret kind=… page=… run=… len=…`.
const CARET_EVENT: &str = "text-edit-caret";
/// `text-caret-step dir=… from_run=… to_run=… to_caret=…`.
const STEP_EVENT: &str = "text-caret-step";
/// `text-caret-nowhere dir=… run=… caret=…` — the arm ran and the model had
/// nothing that way. A SKIP rather than a failure; see its use below.
const NOWHERE_EVENT: &str = "text-caret-nowhere";
/// `text-caret-line end=… from_run=… to_run=… to_caret=…` — Home or End
/// reached a slot in ANOTHER run, because the page's line spans several.
const LINE_EVENT: &str = "text-caret-line";

/// See the module documentation.
pub struct ArrowKeysWalkBetweenBlocks;

impl Check for ArrowKeysWalkBetweenBlocks {
    fn name(&self) -> &'static str {
        "arrow_keys_walk_between_blocks"
    }

    fn defect(&self) -> &'static str {
        "the arrow keys do nothing once a caret is in a piece of text — there is no way to move \
         to the line above or below without clicking, and no way at all to step from one block \
         of text to the next. The shell this project replaces did both"
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

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs a document with text in more than one place — see the \
             module header for why a one-run fixture would pass it vacuously.",
        )
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point that sits ON TEXT — \
             the caret has to land in a run before there is anywhere to navigate from.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, arms a tool, \
             clicks text and presses the arrow keys. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("block-nav.trace.txt"));
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
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit mode, Edit tab, the caret tool -----------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
    click_command(&session, &driver, ui_rect, TOOL, 16)?;

    // --- 2: put a caret in real text ----------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let at = session
        .frame()?
        .to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    driver.click_at(at)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(caret) = trace
        .events(CARET_EVENT)
        .filter(|l| l.get("run").is_some())
        .last()
    else {
        return Err(Error::new(format!(
            "the click at (page {}, {:.1}, {:.1}) placed no caret in a run, so there is nowhere \
             to navigate FROM. A fact about the fixture and the point rather than about the \
             build — aim at a piece of text. SKIPPED for that reason.",
            target.page + 1,
            target.x,
            target.y
        )));
    };
    let from = caret.get("run").unwrap_or("?").to_owned();
    report.note(format!("★ the caret landed in a run: `{}`", caret.raw));

    // --- 3: ★★ press Down, then Up ------------------------------------------
    //
    // Both directions, because they are separate calls into the model and a
    // build that wired one and not the other is a plausible half-job. Down
    // first: `--doc-point` is more often above the middle of a sheet than below
    // it, so there is more often something under the caret than over it.
    let before = session.trace()?.events(STEP_EVENT).count();
    let before_nowhere = session.trace()?.events(NOWHERE_EVENT).count();
    driver.press(vk::ARROW_DOWN)?;
    session.settle(24);
    driver.press(vk::ARROW_UP)?;
    session.settle(24);

    let trace = session.trace()?;
    let steps: Vec<_> = trace.events(STEP_EVENT).skip(before).collect();
    let nowhere = trace.events(NOWHERE_EVENT).count() - before_nowhere;
    if steps.is_empty() && nowhere > 0 {
        // ★★ THE KEYS ARRIVED AND THE PAGE HAD NOWHERE TO GO, which is a fact
        // about the document and the point rather than about the build — and
        // the two were indistinguishable on this check's first live run, which
        // is why `text-caret-nowhere` exists at all.
        //
        // `caret_up`/`caret_down` never cross a COLUMN BAND, so a lone label in
        // the middle of a drawing has no line above or below it by
        // construction. Aim `--doc-point` at a stack of text and the same build
        // passes.
        return Err(Error::new(format!(
            "the arrow keys REACHED the caret ({nowhere} press(es) answered) and the model had {}",
            format_args!(
                "no line above or below run {from}. `caret_up`/`caret_down` never cross a column \
                     band, so a piece of text with nothing stacked over or under it has nowhere to \
                     go. Aim --doc-point at a stack of lines: a title block's rows, a note list. \
                     SKIPPED for that reason. Trace: {}.",
                session.trace_path().display()
            )
        )));
    }
    if steps.is_empty() {
        // ★ Ask what else happened before accusing. A caret on the ONLY line of
        // text on a page has nowhere to go in either direction, and that is a
        // fact about the document.
        return Ok(Some(format!(
            "★ THE ARROW KEYS NEVER REACHED THE CARET: neither a `{STEP_EVENT}` nor a {}",
            format_args!(
                "`{NOWHERE_EVENT}` line after Down and Up, from run {from}. The second half of \
                     that is what makes this an accusation rather than a guess: \
                     `canvas::textedit::blocks::step` traces BOTH outcomes, so a build whose model \
                     merely found nothing would still have said so. Silence means the key events \
                     did not arrive at `typing`'s arrow arm at all — look for an earlier arm that \
                     swallowed them, a draft that is not run-anchored, or a frame where the canvas \
                     did not own the keyboard. Trace: {}.",
                session.trace_path().display()
            )
        )));
    }

    // --- 4: ★★★ and it landed in a DIFFERENT run ----------------------------
    //
    // The assertion the whole check exists for. Moving the caret *within* the
    // run it was already in is a caret movement; moving it to another run is
    // navigation between blocks of text, and only the second is what was asked
    // for.
    let crossed = steps.iter().find(|l| l.get("from_run") != l.get("to_run"));
    let Some(crossed) = crossed else {
        return Ok(Some(format!(
            "★ THE CARET MOVED AND STAYED IN THE SAME RUN. {} step(s), every one with \
             `from_run` equal to `to_run` — the last was `{}`.\n\
             That is a caret movement, not navigation between blocks. `blocks::neighbour` asks \
             `EditableTextModel::caret_up`/`caret_down`, which walk the model's LINES: a run \
             that is its own line has a line above it belonging to a different run, so an \
             answer inside the same run means the model was asked about the wrong position — \
             most likely a character index handed over where a byte offset was wanted. Trace: \
             {}.",
            steps.len(),
            steps.last().map_or("", |l| l.raw.as_str()),
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★★★ the caret crossed into another block of text: `{}`",
        crossed.raw
    ));
    report.note(format!(
        "{} navigation step(s) in total across Down and Up",
        steps.len()
    ));

    // --- 5: End, which reaches the end of the LINE and not of the run --------
    //
    // ★ Recorded rather than asserted, and the difference matters. A page line
    // drawn as ONE show operator has its end inside the run the caret is
    // already in, and the shell handles that with a single assignment and no
    // trace — indistinguishable from a build where End does nothing. So the
    // presence of `text-caret-line` is evidence and its absence is not an
    // accusation. Naming that here beats a check that fails on some documents
    // and passes on others for reasons nobody can see.
    driver.press(vk::END)?;
    session.settle(20);
    let trace = session.trace()?;
    match trace.events(LINE_EVENT).last() {
        Some(l) => report.note(format!(
            "★ End crossed into the rest of the line, which is several pieces of text wide: `{}`",
            l.raw
        )),
        None => report.note(
            "End raised no `text-caret-line`: this line is one piece of text, so its end is"
                .to_owned()
                + " inside the run the caret was already in. Not evidence either way.",
        ),
    };
    Ok(None)
}
