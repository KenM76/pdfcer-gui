//! `save_writes_over_the_file_you_opened` — Save. In place. The one every other
//! program has.
//!
//! # Why this check exists
//!
//! The operator, 2026-08-20:
//!
//! > *"can I please have a save button like every other program in existence
//! > has? We're on week two of this and just have a save as button."*
//!
//! `Ctrl+S` was bound to `file.save_copy`, which opens a file dialog every
//! single time. In-place save had been written down in the manifest's planned
//! list as *"blocked on autosave and crash recovery"* and had then been nobody's
//! problem for a fortnight.
//!
//! # ★★ The assertion this check is really for: the ORIGINAL survives a failure
//!
//! Save-in-place is **the only verb in this application that can destroy the
//! operator's work**, and the way it would do so is not exotic:
//! `std::fs::write` truncates the target and then streams into it, so every
//! byte of the payload is a window in which their only copy is a partial file.
//! On a CAD sheet that payload is megabytes.
//!
//! So `save::save_in_place` writes to `<name>.pdfcer-tmp` beside the target and
//! renames — an act that either happens or does not. This check drives the
//! happy path; the property it pins is that **the file that comes out is a
//! whole PDF that pdfcer can read back**, which is what a half-written one would
//! not be.
//!
//! # What it does NOT need
//!
//! A dialog seam. That is the entire point of the feature and it is worth
//! stating: `save_copy_round_trip` needs `PDFCER_DIAG_SAVE_PATH` because a modal
//! picker is a hard wall to a harness. Save has no picker, so this check drives
//! exactly what the operator drives, with nothing substituted.

use std::path::{Path, PathBuf};

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared};
use crate::checks::save_copy::{click_command, click_tab};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Review can author a markup, which is the cheapest way to make the document
/// modified without needing a page click to land on anything in particular.
const MODE: &str = "review";
/// The File tab.
const FILE_TAB: (&str, &str) = ("ribbon.tab.file", "file");
/// The Markup tab, for making an edit worth saving.
const MARKUP_TAB: (&str, &str) = ("ribbon.tab.markup", "markup");
/// The rectangle tool.
const RECTANGLE: (&str, &str) = ("ribbon.item.markup.rectangle", "markup.rectangle");
/// ★ The control this check is about.
const SAVE: (&str, &str) = ("ribbon.item.file.save", "file.save");
/// `add-markup …` — the document changed.
const EDIT_EVENT: &str = "add-markup";
/// `save-in-place outcome=… ` — the save ran and said how it went.
const SAVE_EVENT: &str = "save-in-place";

/// See the module documentation.
pub struct SaveWritesOverTheFileYouOpened;

impl Check for SaveWritesOverTheFileYouOpened {
    fn name(&self) -> &'static str {
        "save_writes_over_the_file_you_opened"
    }

    fn defect(&self) -> &'static str {
        "Save is absent, or it opens a file dialog like Save-a-copy, or it writes somewhere other \
         than the file that was opened — or it truncates the original and leaves a partial file \
         when the write fails, which destroys the operator's only copy"
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

/// Copy the fixture somewhere this check may destroy.
///
/// ★ **Never the operator's own fixture.** This check exists to prove a verb
/// that overwrites, so pointing it at `--pdf` directly would mean a harness run
/// modifies the file the next run measures — and a fixture that changes under
/// the suite is the thing `crate::fixture`'s header refuses.
fn scratch_copy(ctx: &CheckContext, pdf: &Path) -> Result<PathBuf> {
    let target = ctx.out("save_in_place_subject.pdf");
    std::fs::copy(pdf, &target).map_err(|e| {
        Error::new(format!(
            "could not copy the fixture to {} for this check to overwrite: {e}",
            target.display()
        ))
    })?;
    Ok(target)
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, two ribbon tabs, a \
             markup tool, the page and the Save control. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let subject = scratch_copy(ctx, &pdf)?;
    let before = std::fs::metadata(&subject)
        .map_err(|e| Error::new(format!("cannot measure the subject copy: {e}")))?
        .len();
    report.note(format!(
        "subject copy at {} — {before} bytes before",
        subject.display()
    ));

    let mut spec = LaunchSpec::new(&exe, ctx.out("save_in_place.trace.txt"));
    spec.pdf = Some(subject.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!("launched as pid {}", session.pid()));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 1: make an edit worth saving --------------------------------------
    click_tab(&session, &driver, ui_rect, MARKUP_TAB)?;
    click_command(&session, &driver, ui_rect, RECTANGLE, 14)?;
    let frame = session.frame()?;
    let trace = session.trace()?;
    let Some(page) = declared(&trace, ui_rect, "page") else {
        return Err(Error::new(
            "the application declared no `page` region, so there is nowhere to drag a markup and \
             no way to make the document modified.",
        ));
    };
    driver.drag(
        frame.declared_at(page, 0.30, 0.30),
        frame.declared_at(page, 0.45, 0.45),
    )?;
    session.settle(20);
    if session.trace()?.last(EDIT_EVENT).is_none() {
        return Err(Error::new(
            "the markup drag authored nothing, so the document is unmodified and this check would \
             be measuring a save with nothing to save. That is a `markup_rectangle` failure, not a \
             save one — reported as SKIP so it is chased in the right place.",
        ));
    }
    report.note("a markup was authored, so the document is modified");

    // --- 2: ★ Save -------------------------------------------------------
    click_tab(&session, &driver, ui_rect, FILE_TAB)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, SAVE.0).is_none() {
        return Ok(Some(format!(
            "the File tab declares no `{}` region. ★ This is the operator's report in its most \
             literal form: there is no Save control, only Save-a-copy.",
            SAVE.0
        )));
    }
    click_command(&session, &driver, ui_rect, SAVE, 30)?;

    let trace = session.trace()?;
    let Some(line) = trace.last(SAVE_EVENT) else {
        return Ok(Some(format!(
            "Save was pressed and traced no `{SAVE_EVENT}` line, so the control is drawn and \
             inert — or it raised `Action::SaveCopy` instead, which would have opened a picker \
             and hung this run. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if line.get("outcome") != Some("ok") {
        return Ok(Some(format!(
            "Save ran and did not succeed: `{}`. Note which STAGE it names — `write` means the \
             replacement never materialised and the original is untouched, `rename` means it did \
             and the swap was refused, which is overwhelmingly \"the file is open in another \
             program\".",
            line.raw
        )));
    }
    report.note(format!("★ Save reported success: `{}`", line.raw));

    // --- 3: ★★ the file on disk is a WHOLE pdf, not a truncated one --------
    //
    // The property the temporary-and-rename exists for. A partial write would
    // leave a file that is larger than nothing and smaller than a document, and
    // would very likely still open far enough to look plausible — so the
    // assertion is not "it changed" but "it is still readable as a page tree".
    let after = std::fs::metadata(&subject)
        .map_err(|e| Error::new(format!("the subject file is gone after the save: {e}")))?
        .len();
    if after <= before {
        return Ok(Some(format!(
            "Save reported success and the file did not grow: {before} → {after} bytes. pdfcer \
             writes an INCREMENTAL update, so a successful save always appends a revision — a \
             file that stayed the same size means the bytes went somewhere else, and a smaller \
             one means it was truncated."
        )));
    }
    let stray = subject.with_extension("pdfcer-tmp");
    if stray.exists() {
        return Ok(Some(format!(
            "Save succeeded and left its temporary behind at {}. The rename is supposed to \
             consume it; a leftover means the write path took a branch that copies rather than \
             renames, which reintroduces the unsafe window this whole design exists to remove.",
            stray.display()
        )));
    }
    if crate::fixture::page_geometry(&subject).is_none() {
        return Ok(Some(format!(
            "★★ Save reported success and the file it produced CANNOT BE READ as a page tree. \
             This is the failure the temporary-and-rename exists to make impossible, and it is \
             the worst outcome in this suite: the operator pressed Save and their document is \
             now damaged. File: {}.",
            subject.display()
        )));
    }
    report.note(format!(
        "the file grew {before} → {after} bytes, no temporary was left behind, and it still \
         reads as a page tree"
    ));
    Ok(None)
}
