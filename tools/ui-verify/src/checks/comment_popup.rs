//! `a_comment_can_be_read_on_the_page_in_read_mode` — **the operator's report,
//! turned into a gate.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/comment_popup.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Read mode, and nothing else. See the module header.
const INVOKE: &str = "mode.read";
/// The per-frame line the pop-up surface writes when it has anything to say.
const CENSUS: &str = "note-popup";
/// The line `canvas::clicking` writes when a click toggles a pop-up.
const TOGGLE: &str = "note-popup-toggle";
/// The region an open pop-up publishes — **only when it is visible**. See the
/// module header on why that distinction is the whole check.
const POPUP_REGION: &str = "notepopup.window";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";
/// The fixture, pinned. `--pdf` is ignored and the report says so.
const FIXTURE: &str = "fixtures/comment-note.pdf";
/// The **closed** note's centre, in PDF user space. Its `/Rect` is
/// `[100 300 120 320]`; see the generator's docstring.
const CLOSED_NOTE: (f64, f64) = (110.0, 310.0);

/// The repository's own copy of the fixture, located from **this crate**
/// rather than from the working directory or from `--source-root`.
fn fixture() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE)
}

/// See the module documentation.
pub struct ACommentCanBeReadOnThePageInReadMode;

impl Check for ACommentCanBeReadOnThePageInReadMode {
    fn name(&self) -> &'static str {
        "a_comment_can_be_read_on_the_page_in_read_mode"
    }

    fn defect(&self) -> &'static str {
        "a sticky note can be placed and never read: nothing on the canvas shows an annotation's \
         /Contents, and the only surface that does is a panel on a tab Read mode is not shown — \
         so pdfcer's reading mode cannot read the comments, which is the posture of a PDF reader \
         exactly backwards"
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

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a note on the page twice. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;

    // The fixture is pinned and any `--pdf` is discarded — and SAID so,
    // because a sweep that silently ignored a flag is indistinguishable from
    // one that honoured it.
    let pdf = fixture();
    if !pdf.exists() {
        return Err(Error::new(format!(
            "{FIXTURE} was not found at {}. Regenerate it with \
             `python tools/gen-comment-note-fixture.py`.",
            pdf.display()
        )));
    }
    if let Some(supplied) = &ctx.pdf
        && supplied != &pdf
    {
        report.note(format!(
            "--pdf {} was IGNORED. This check pins {FIXTURE}, because a document with no \
             comments cannot exercise it and would SKIP for ever while looking healthy.",
            supplied.display()
        ));
    }

    // The fixture is US Letter and its annotation rectangles are absolute, so
    // the geometry is read from the file rather than taken from `--page-size`:
    // a size supplied for a different document would aim these clicks at
    // nothing.
    let page: PageGeometry = crate::fixture::page_geometry(&pdf).ok_or_else(|| {
        Error::new(format!(
            "could not read a page size from {}. The fixture may be corrupt; regenerate it.",
            pdf.display()
        ))
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("comment-popup.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
        "launched {} as pid {} on {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- A: the document opened, and the pop-up its file asked for is OPEN ---
    let trace = session.trace()?;
    if declared(&trace, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nothing to click. Regions beginning `page`: {}.",
            list(&declared_names(&trace, ui_rect, "page"))
        )));
    }

    let Some(first) = trace.last(CENSUS) else {
        return Ok(Some(format!(
            "no `{CENSUS}` line after opening a document carrying three comments, one of them \
             authored `/Open true`. The canvas surfaced nothing at all about the notes on the \
             page — which is the operator's report of 2026-09-05 verbatim: a sticky note can be \
             placed and not read. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("on load: {}", first.raw));

    // Through `TraceLine::get_usize` rather than a hand-rolled split: the
    // parser already handles quoting and the `Some(3)` wrapper, and a second
    // field reader in this file would be a second thing to keep in step with
    // the trace vocabulary.
    let raw = first.raw.clone();
    let Some(open_now) = first.get_usize("open") else {
        return Err(Error::new(format!(
            "the `{CENSUS}` line carries no `open=` field, so this check cannot tell an open \
             pop-up from a closed one. The trace vocabulary changed; fix the check. Line: \
             {raw}"
        )));
    };
    let Some(from_file) = first.get_usize("from_file") else {
        return Err(Error::new(format!(
            "the `{CENSUS}` line carries no `from_file=` field. That field is the ONLY oracle \
             for whether a pop-up is open because the document said `/Open true` rather than \
             because something was clicked, and without it phase A proves nothing. Line: \
             {raw}"
        )));
    };

    if open_now == 0 {
        return Ok(Some(format!(
            "the fixture's note is authored `/Open true` (§12.5.6.4 Table 172, and its `/Popup` \
             repeats it per Table 183) and NO pop-up opened: `{raw}`. The open state is in the \
             file and pdfcer defaulted it instead of reading it — so a document another product \
             authored with its notes open arrives here silently shut."
        )));
    }
    if from_file == 0 {
        return Ok(Some(format!(
            "a pop-up is open but `from_file=0`, so it is open because something was clicked \
             rather than because the file said so. Nothing has been clicked yet. `{raw}`"
        )));
    }
    // The negative half, and it is what stops this check passing on a
    // build that simply opens everything. The fixture carries TWO notes with
    // words: one `/Open true` and one `/Open false`.
    if open_now != 1 {
        return Ok(Some(format!(
            "{open_now} pop-ups are open on load and the fixture authored exactly ONE `/Open \
             true` (the other note is `/Open false` and also carries words). A build that opens \
             every pop-up it finds covers the drawing with windows the operator did not ask \
             for. `{raw}`"
        )));
    }

    // …and it is VISIBLE, not merely laid out. `ui_rect_visible` publishes
    // nothing unless the rectangle is visible enough inside the canvas clip it
    // was handed, so the region's presence is the assertion. See the module
    // header for why layout alone is not enough.
    if declared(&trace, ui_rect, POPUP_REGION).is_none() {
        return Ok(Some(format!(
            "a pop-up is open by the census (`{raw}`) and `{POPUP_REGION}` was never \
             published, so the window laid out somewhere the operator cannot see it — off the \
             canvas edge, behind the docked panels, or clipped away. This is the failure three \
             panels in this project shipped with every gate green. Regions beginning \
             `notepopup`: {}.",
            list(&declared_names(&trace, ui_rect, "notepopup"))
        )));
    }

    // --- C: a click on the CLOSED note opens it ------------------------------
    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, CLOSED_NOTE.0, CLOSED_NOTE.1),
    )?;
    driver.click_at(at)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(TOGGLE).count() == 0 {
        return Ok(Some(format!(
            "a click at the closed note's own rectangle ({:?} in PDF user space) produced no \
             `{TOGGLE}` line, so the canvas did not recognise a click on a comment. In Read \
             mode this is the ONLY route to a note's words — the Comments panel lives on the \
             Markup tab, which Read is not shown — so the operator has no way to read it. \
             Trace: {}.",
            CLOSED_NOTE,
            session.trace_path().display()
        )));
    }
    let after = trace
        .last(CENSUS)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    let opened = trace
        .last(CENSUS)
        .and_then(|l| l.get_usize("open"))
        .unwrap_or(0);
    if opened != 2 {
        return Ok(Some(format!(
            "after clicking the closed note, {opened} pop-ups are open and 2 were expected — the \
             one the file authored open, plus the one just clicked. `{after}`"
        )));
    }
    report.note(format!("after the first click: {after}"));

    // --- D: clicking it again CLOSES it --------------------------------------
    //
    // The class convention, and the gesture an operator who opened one by
    // accident tries first. A build that only ever opened would leave every
    // note the reviewer glanced at covering the drawing.
    driver.click_at(at)?;
    session.settle(30);
    let trace = session.trace()?;
    let after = trace
        .last(CENSUS)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    let closed_again = trace
        .last(CENSUS)
        .and_then(|l| l.get_usize("open"))
        .unwrap_or(0);
    if closed_again != 1 {
        return Ok(Some(format!(
            "clicking the note a second time left {closed_again} pop-ups open and 1 was expected \
             — a click toggles. A window that cannot be dismissed by the gesture that opened it \
             is one an operator has to hunt for a close button to be rid of. `{after}`"
        )));
    }
    report.note(format!("after the second click: {after}"));

    Ok(None)
}
