//! `export_text_writes_the_documents_words` — the text reaches disk, and the
//! file holds exactly the characters the shell said it wrote.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_text.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, frame_of, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode this runs in.
///
/// ★ **Read**, deliberately, and it is an assertion rather than a convenience —
/// [`super::export_dxf`]'s reason, and here it is the stronger one. Taking the
/// words out of a drawing is the archetypal reading act, and it is the same
/// argument that moved `copy_page_text` onto the File tab in the first place:
/// *replacing Acrobat Reader* is what Read mode is for, and a Read that cannot
/// get the text out is wrong about the thing it exists to be.
const MODE: &str = "read";
/// The window's own region.
const WINDOW: &str = "dialog:export-text";
/// The Export button.
const EXPORT: &str = "export-text.export";
/// The trace the window emits when it opens.
const OPENED: &str = "export-text-open";
/// The trace the apply arm emits on a successful write.
const WROTE: &str = "export-text";
/// The trace the apply arm emits when the document carries no readable text.
///
/// ★ Read even on a successful run, because it is the branch this feature
/// exists for and a build that took it here would otherwise be reported only as
/// *"no `export-text` line"* — true, and it would not say which of four
/// different things happened.
const REFUSED: &str = "export-text-refused";
/// The environment seam that answers the save dialog.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name

/// See the module documentation.
pub struct ExportTextWritesTheDocumentsWords;

impl Check for ExportTextWritesTheDocumentsWords {
    fn name(&self) -> &'static str {
        "export_text_writes_the_documents_words"
    }

    fn defect(&self) -> &'static str {
        "File > Export text is drawn and does nothing — or it opens a window whose Export \
         button writes no file, or writes one holding different characters from the ones the \
         shell reported, so an operator searches or diffs a text file that is not what is in \
         the drawing"
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
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, \
             a ribbon control and a button. Reported as SKIPPED rather than passed: a check \
             that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // ★ Removed before the run, not merely named. A file left by an earlier run
    // would let a build that writes NOTHING pass every assertion below — the
    // `a_driven_check_that_does_not_establish_its_preconditions_measures_the_previous_run`
    // finding, and the line that prevents it.
    let target = ctx.out("export_text.txt");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "cannot clear {} before the run, so a file written by an earlier run could be \
             mistaken for this one's.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("export_text.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
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
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 1: the File tab ---------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.file").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.file` region in {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);

    // --- 2: open the window ------------------------------------------------
    let Some(item) =
        declared_or_in_overflow(&session, &driver, ui_rect, "ribbon.item.file.export_text")?
    else {
        return Ok(Some(format!(
            "the File tab declares no `ribbon.item.file.export_text`, on the band or in the \
             overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.file."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, WINDOW).is_none() {
        let unimplemented = trace
            .events("command-unimplemented")
            .any(|l| l.get("id") == Some("file.export_text"));
        return Ok(Some(if unimplemented {
            "`file.export_text` was clicked and traced `command-unimplemented` — it is drawn, \
             on the ribbon, and has no dispatch arm behind it."
                .to_owned()
        } else {
            format!(
                "`file.export_text` was clicked and no `{WINDOW}` region appeared. The arm ran \
                 and built no window — or declined it. In {MODE} it must not decline: taking \
                 the words out of a drawing is what a reading stance is for."
            )
        }));
    }
    let Some(opened) = trace.last(OPENED) else {
        return Ok(Some(format!(
            "the window drew and traced no `{OPENED}` line, so it never read the document's \
             page list. Every control in it is a statement about those pages."
        )));
    };
    report.note(format!("opened: `{}`", opened.raw));

    // --- 3: export ---------------------------------------------------------
    let Some(button) = declared(&trace, ui_rect, EXPORT) else {
        return Ok(Some(format!(
            "the window declares no `{EXPORT}` region, so there is nothing to press."
        )));
    };
    driver.click_at(frame_of(&session, &trace, ui_rect, EXPORT)?.declared_center(button))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(wrote) = trace.last(WROTE) else {
        // ★★ The four outcomes are told apart, because they send an operator —
        // and whoever reads this verdict — to four different places. A refusal
        // in particular is not a bug: it is what a scanned fixture SHOULD
        // produce, and reporting it as "the export is broken" would send
        // somebody to fix working code.
        let refused = trace.last(REFUSED);
        let declined = trace.last("export-text-declined");
        let cancelled = trace.last("export-text-cancelled");
        let failed = trace.last("export-text-failed");
        return Ok(Some(format!(
            "Export was pressed and no `{WROTE}` line followed. refused={} declined={} \
             cancelled={} failed={} — a REFUSAL means the fixture carries no extractable \
             text, which is correct behaviour on a scan and means this check needs a \
             different --pdf; a FAILURE names the engine's own error.",
            refused.map_or("none".to_owned(), |l| l.raw.clone()),
            declined.map_or("none".to_owned(), |l| l.raw.clone()),
            cancelled.map_or("none".to_owned(), |l| l.raw.clone()),
            failed.map_or("none".to_owned(), |l| l.raw.clone()),
        )));
    };
    report.note(format!("wrote: `{}`", wrote.raw));

    // --- 4: the file is on disk, and it is UTF-8 ---------------------------
    if !target.exists() {
        return Ok(Some(format!(
            "the shell traced a successful export and {} does not exist. The disclosure and \
             the disk disagree, which is the worst of the failures available here: the \
             operator has been told a file was written.",
            target.display()
        )));
    }
    let bytes = std::fs::read(&target).map_err(|e| {
        Error::new(format!(
            "the export wrote {} and it cannot be read back: {e}",
            target.display()
        ))
    })?;
    // ★ `from_utf8` rather than `from_utf8_lossy`. The window promises UTF-8 by
    // name, because a CAD drawing carries degree signs and diameter marks; a
    // lossy read would turn a broken encoding into replacement characters and
    // then count them as text, which is the exact failure the promise is about.
    let Ok(written) = String::from_utf8(bytes) else {
        return Ok(Some(format!(
            "{} is not valid UTF-8, and the window states that it is. Any tool the operator \
             opens it with will mangle every character beyond ASCII — which on a drawing is \
             every dimension symbol.",
            target.display()
        )));
    };

    // --- 5: ★ the preconditions the identity below depends on --------------
    //
    // Read from the trace rather than assumed. A later build that changed the
    // window's defaults must produce "the defaults moved", not "the export is
    // broken": those are opposite findings.
    let separator = wrote.get("separator").unwrap_or("");
    let bom = wrote.get("bom").unwrap_or("");
    let endings = wrote.get("crlf").unwrap_or("");
    if separator != "FormFeed" || bom != "0" || endings != "AsExtracted" {
        return Ok(Some(format!(
            "★ the window's defaults have moved — this run wrote separator={separator} bom={bom} \
             crlf={endings}, and the character identity below is only valid for FormFeed / 0 / \
             AsExtracted. This is not a broken export; it is a check that can no longer \
             measure one. Re-derive the identity for the new defaults, or drive the controls \
             back to them before pressing Export."
        )));
    }

    // --- 6: ★★ the file holds exactly the characters that were reported ----
    let (Some(reported), Some(pages)) = (
        wrote.get("chars").and_then(|v| v.parse::<usize>().ok()),
        wrote.get("pages").and_then(|v| v.parse::<usize>().ok()),
    ) else {
        return Err(Error::new(format!(
            "the `{WROTE}` line carries no readable `chars=` and `pages=` counts, so the file \
             cannot be reconciled against it: `{}`",
            wrote.raw
        )));
    };
    let in_file = written.chars().count();
    let expected = reported + pages.saturating_sub(1);
    if in_file != expected {
        return Ok(Some(format!(
            "★ the shell reported {reported} characters over {pages} page(s) — {expected} with \
             the {} separator(s) between them — and the file holds {in_file}. The disclosure \
             describes text the bytes do not, which an operator has no way to notice: both \
             numbers look reasonable and only one of them is in the file they search.",
            pages.saturating_sub(1)
        )));
    }
    report.note(format!(
        "{in_file} characters on disk = {reported} reported + {} separator(s)",
        pages.saturating_sub(1)
    ));

    // --- 7: ★ the separators are BETWEEN, never bracketing ------------------
    //
    // The identity in step 6 counts characters and cannot see where they sit,
    // so a build that wrote a leading form feed and dropped one between two
    // pages would satisfy it exactly. This is the placement half, and it is the
    // property `plain_text()`'s own `if i > 0` guarantees on the clipboard side
    // — so a divergence here is the two answers to "the text of this document"
    // coming apart, which is the whole thing this feature was arranged to
    // prevent.
    if written.starts_with('\u{000C}') || written.ends_with('\u{000C}') {
        return Ok(Some(
            "★ the file opens or closes with a page separator. `plain_text()` puts one only \
             BETWEEN pages, so this export no longer agrees byte-for-byte with what Copy \
             document text puts on the clipboard — two answers to \"the text of this \
             document\", both of which look like text."
                .to_owned(),
        ));
    }
    if written.matches('\u{000C}').count() != pages.saturating_sub(1) {
        return Ok(Some(format!(
            "★ {} page(s) were exported and the file holds {} separator(s) where it should \
             hold {}. Splitting this file on U+000C — which is the documented way to recover \
             page boundaries — gives the wrong pages.",
            pages,
            written.matches('\u{000C}').count(),
            pages.saturating_sub(1)
        )));
    }

    // --- 8: the fixture actually had text, so this run proved something -----
    //
    // ★ Last rather than first, because everything above is a real assertion
    // whatever the fixture holds. But a document of scans exports zero
    // characters through a completely correct code path, and a check that
    // reported PASS on that has measured nothing — the same shape as the DXF
    // check's vacuous-cross-check note, stated rather than passed over.
    if reported == 0 {
        return Err(Error::new(format!(
            "the fixture exported {reported} characters, so every assertion above is vacuous \
             and this run proves nothing about the export. Pass a --pdf with a text layer. \
             (If this document IS a scan, the interesting check is that it reaches \
             `{REFUSED}` before the save picker — which it did not, or there would be no \
             `{WROTE}` line.)"
        )));
    }
    Ok(None)
}
