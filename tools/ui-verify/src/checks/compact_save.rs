//! `a_compacted_copy_is_actually_smaller` — **the save that reclaims the space
//! a deletion freed.**
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` **O48**, answered *"yes to all three"* on 2026-08-28.
//! It was raised by this project rather than by him, from a limit found while
//! wiring Remove-embedded-fonts:
//!
//! > **Removing fonts does not make the file smaller.** pdfcer saves by adding
//! > your changes to the end of the file and leaving the earlier version
//! > intact, so the outlines stop being used and are still there.
//!
//! §7.5.6's update section is *appended*, so every space-reclaiming operation
//! pdfcer has produced a file that was very slightly **larger**. Only a full
//! rewrite drops the bytes, and this is the command that asks for one.
//!
//! ## ★★★ The oracle is the FILE ON DISK, and nothing else would do
//!
//! Every link in this chain can be satisfied by a build that saves nothing.
//! The window opens on a serialisation, quotes a number, and hands bytes to an
//! action; a trace line saying `compact-written after=1048576` proves that a
//! `Vec<u8>` of that length existed. It does not prove a file was written, that
//! the file has those bytes in it, or that the operator can open it.
//!
//! So this reads the file back and asserts three things about it, in order of
//! how badly each one fails:
//!
//! | # | assertion | what its failure means |
//! |---|---|---|
//! | 1 | the file exists and is non-empty | the bytes went nowhere |
//! | 2 | it begins `%PDF-` | something was written and it is not a PDF |
//! | 3 | it is **smaller** than the original | the rewrite reclaimed nothing, which is the entire feature |
//!
//! ★★ Assertion 3 is the one this check exists for and the one no unit test can
//! make: it is a claim about two real files on a real disk, produced by two
//! different code paths in `pdfcer-core` — the incremental writer that made the
//! fixture and the full writer that made the copy.
//!
//! ## ★★★ The fixture IS the operator-visible problem, built by the CLI
//!
//! A tidy file compacts to roughly its own size, and the window says so in words
//! — *"this file has nothing unused in it"* — which is a correct answer and
//! would fail assertion 3. So the fixture is one with real waste in it, and it
//! was made by doing to a drawing exactly what O48 is about:
//!
//! ```text
//! a1-titleblock.pdf                                    39,509 bytes
//!   embed-font --all-missing --apply       ->       1,708,740     (fonts appended)
//!   unembed-font --all-removable --apply   ->       1,709,629     (fonts removed)
//! ```
//!
//! ★★ **Look at the last line.** Removing 1.6 MB of font programs made the file
//! **889 bytes bigger**, because §7.5.6 appends the removal as a new revision
//! and leaves the programs in the old one. That is the sentence O48 was written
//! about, reproduced in a file, and it is what this check measures the fix
//! against: a compacted copy of `reclaimable.pdf` should be tens of kilobytes,
//! not 1.7 MB.
//!
//! ★ Built by `pdfcer` rather than by this check, and committed. A check that
//! manufactured its own multi-megabyte fixture on every run would spend most of
//! its wall clock building the thing it measures, and the two CLI calls are a
//! provenance line rather than a program.
//!
//! ## What this does NOT cover
//!
//! **That the compacted copy renders identically.** It is a different byte
//! sequence for the same document and the engine's writer tests own that claim.
//! What is asserted here is that the shell reaches the full writer, keeps the
//! bytes it measured, and puts them where the operator asked.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command under test.
///
/// ★ `mode.edit` first, for the usual reason: driving from a named mode makes
/// the run reproducible rather than dependent on whatever mode the last session
/// left behind. Compaction is permitted in every mode — it is a save — so this
/// is reproducibility rather than a gate.
const INVOKE: &str = "mode.edit,file.save_compacted";
/// The compacted window's body and button.
const BODY: &str = "compact.body";
const BUTTON: &str = "compact.commit";
/// The line the window writes when the operator accepts.
const REQUESTED: &str = "compact-requested";
/// The line `app::save::compacted` writes when the file is on disk.
const WRITTEN: &str = "compact-written";
/// The refusal, when the engine will not rewrite this file.
const REFUSED: &str = "compact-refused";
/// The variable that answers the save picker.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";

/// See the module documentation.
pub struct ACompactedCopyIsActuallySmaller;

impl Check for ACompactedCopyIsActuallySmaller {
    fn name(&self) -> &'static str {
        "a_compacted_copy_is_actually_smaller"
    }

    fn defect(&self) -> &'static str {
        "removing pages, images or embedded fonts never makes the file smaller — pdfcer appends \
         every save, so the freed bytes stay in the previous revision and the file grows, and \
         there is no command that asks for the full rewrite that would drop them"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks two window buttons: one to embed \
             fonts (which creates the waste it then measures) and one to compact.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs a document naming fonts it does not carry, so that \
             embedding them creates something for the compaction to reclaim.",
        )
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // ★ Deleted first, and that is not tidiness: a file left by a previous run
    // would satisfy every assertion below on a build that wrote nothing at all
    // — the single most likely way for a file-oracle check to go quietly green.
    let target = ctx.out("compacted-copy.pdf");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "{} could not be removed before the run, so a file found afterwards would prove \
             nothing. SKIPPED rather than failed.",
            target.display()
        )));
    }
    let original = std::fs::metadata(&pdf).map(|m| m.len()).unwrap_or(0);
    if original == 0 {
        return Err(Error::new(format!(
            "{} reports zero bytes, so there is nothing to compare a copy against.",
            pdf.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("compact-save.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env.push((
        SAVE_PATH_ENV.to_owned(),
        target.to_string_lossy().into_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}; the --pdf is {original} bytes on disk",
        exe.display(),
        session.pid()
    ));
    session.settle(90);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    if let Some(refused) = trace.events(REFUSED).last() {
        return Err(Error::new(format!(
            "the engine refused the rewrite: `{}`. SKIPPED rather than failed: pdfcer refuses a \
             full rewrite of a hybrid-reference file by name, and of one whose object numbering \
             is too sparse for §7.5.4's single-section table. Both are facts about the --pdf. \
             Trace: {}.",
            refused.raw,
            session.trace_path().display()
        )));
    }
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "★ SAVE A COMPACTED COPY WAS INVOKED AND NO WINDOW APPEARED: no `{BODY}` region and \
             no `{REFUSED}` line. The command has no dispatch arm, or the serialisation panicked \
             before the window was built. Regions beginning `compact`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "compact")),
            session.trace_path().display()
        )));
    }
    let Some(button) = stable_rect(&session, ui_rect, BUTTON, 8)? else {
        return Ok(Some(format!(
            "the compacted-copy window drew and declared no `{BUTTON}` region. It is never \
             greyed — every state this window can be in is one an operator may proceed from — so \
             an absence means it was not laid out. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, BUTTON)?;
    driver.click_at(frame.declared_center(button))?;
    session.settle(60);

    let trace = session.trace()?;
    let Some(requested) = trace.events(REQUESTED).last() else {
        return Ok(Some(format!(
            "the button was clicked and the window raised nothing: no `{REQUESTED}` line. The \
             button is at {button:?} in the window's own frame. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the window raised the save: `{}`", requested.raw));

    let Some(written) = trace.events(WRITTEN).last() else {
        return Ok(Some(format!(
            "★★ THE SAVE WAS REQUESTED AND NO FILE WAS WRITTEN: `{}` and no `{WRITTEN}` line.\n\
             Three candidates: the apply arm never ran; the picker was not answered \
             (`{SAVE_PATH_ENV}` supplies it, and an empty value means *cancelled*, which traces \
             `compact-cancelled`); or the write failed, which traces `compact-failed` with the \
             operating system's own reason. Trace: {}.",
            requested.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the file was written: `{}`", written.raw));

    // --- the oracle: a real file, smaller than the original ------------------
    let Ok(bytes) = std::fs::read(&target) else {
        return Ok(Some(format!(
            "★★★ THE SAVE TRACED SUCCESS AND WROTE NO FILE: `{}` and {} does not exist.\n\
             This is the case a trace-only check cannot see: the bytes were measured, quoted to \
             the operator in a window, handed to an action, and went nowhere. Trace: {}.",
            written.raw,
            target.display(),
            session.trace_path().display()
        )));
    };
    if !bytes.starts_with(b"%PDF-") {
        return Ok(Some(format!(
            "★★★ THE COMPACTED COPY IS NOT A PDF: {} bytes were written to {} and they do not \
             begin `%PDF-`.\n\
             The shell carries the serialised bytes from the window to the writer without \
             touching them, so a wrong header means the wrong buffer travelled — the assertion \
             no length check could make. Trace: {}.",
            bytes.len(),
            target.display(),
            session.trace_path().display()
        )));
    }
    let after = bytes.len() as u64;
    report.note(format!(
        "★★ the copy is a real PDF of {after} bytes, against the original's {original}"
    ));
    if after >= original {
        return Ok(Some(format!(
            "★★★ THE COMPACTED COPY IS NOT SMALLER: {after} bytes against the original's \
             {original}.\n\
             **That is the whole feature.** The fixture carries 1.6 MB of font programs \
             that nothing references, left behind by an incremental save. A \
             copy that is no smaller means the full writer was not reached — check that \
             `dialogs::compact::open` calls `to_full_bytes` and not `to_incremental_bytes`, \
             which is the substitution that produces a correct, valid, useless file. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the compacted copy is {} bytes smaller than the original — the space the embed \
         appended was reclaimed, which is what O48 asked for",
        original - after
    ));
    Ok(None)
}
