//! `save_as_rebinds_the_document_so_the_next_save_goes_to_the_new_file` —
//! **the half of Save As that is not the write.**
//!
//! # The report
//!
//! Ken, 2026-09-02, `OPERATOR_REQUESTS.md` O95:
//!
//! > *"we need a Save As option so that we are then making edits in the save as
//! > file instead of the original just like other programs have it."*
//!
//! **The second half is the request.** `file.save_copy` already wrote the bytes
//! anywhere he pointed it; what it could not do was *move the document*, so the
//! next `Ctrl+S` went back to the file he was trying to leave.
//!
//! # ★★★ Why the oracle is the ORIGINAL file's digest
//!
//! Every cheaper oracle passes against the defect this exists to catch.
//!
//! | oracle | what a still-bound build does |
//! |---|---|
//! | the new file exists | **passes** — Save-a-copy has always written it |
//! | `save-as` was traced | **passes** — the line is emitted by the write |
//! | the title changed | passes only if the title is the thing that broke |
//!
//! ⇒ The question is *"where does the NEXT save go"*, and the only way to ask it
//! is to **do another save and see which file moved.** So this check saves
//! twice: once with Save As, once with `Ctrl+S`, and hashes the original both
//! before and after.
//!
//! ★★ A build that wrote the copy and stayed bound to the original passes the
//! first three rows above and **fails phase D**, because the original changes
//! under it. That is a genuinely falsifying assertion rather than a confirming
//! one, and it is the same shape `checks::ocr` phase E uses for the same reason.
//!
//! # The second edit, and why there has to be one
//!
//! Between the two saves the check makes **another** edit. Without it the second
//! save has nothing to write, `save::has_unsaved_edits` is false, and a correct
//! build would legitimately do nothing at all — which is indistinguishable from
//! a broken one that did nothing because it was pointed at the wrong file.
//!
//! # Nothing of the operator's is touched
//!
//! Both files are in the run's own output directory: the fixture is **copied**
//! there first, and the Save As destination is a sibling. The check writes only
//! inside `--out`.
//!
//! ★ The native picker is never opened. `PDFCER_DIAG_SAVE_PATH` supplies its
//! answer, which is this project's established seam for a system dialog and is
//! what makes phase B an assertion about **a file on disk** rather than about a
//! button having been pressed.
//!
//! # Every way this reports SKIP
//!
//! No binary, `--no-input`, no diagnostic channel, the fixture missing, or a
//! ribbon control that was never declared or took no click.

use std::path::{Path, PathBuf};

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Content selection needs Edit, and so does the rotation this check edits with.
const MODE: &str = "edit";

/// `save-as from=… to=…` — the shell's record that the document MOVED.
///
/// ★ The old path is on the line as well as the new one, and that is the point
/// of tracing it at all: *"the document moved"* and *"a copy was written"*
/// produce the same `save-copy` line, and only the pair says which file the next
/// `Ctrl+S` will reach.
const SAVE_AS: &str = "save-as";

/// The fixture, copied to scratch before anything is driven.
const FIXTURE: &str = "fixtures/four-pages.pdf";

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// A cheap content digest — length plus FNV-1a over the bytes.
///
/// `checks::ocr`'s, verbatim and for its stated reason: the question is *"did
/// this file change"*, the adversary is a bug rather than a forger, and the
/// **length is part of the digest** so a truncation cannot hide behind a
/// collision.
fn digest(bytes: &[u8]) -> (usize, u64) {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (bytes.len(), hash)
}

/// Read and digest, or say which file could not be read.
fn digest_of(path: &Path) -> Result<(usize, u64)> {
    let bytes = std::fs::read(path)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", path.display())))?;
    Ok(digest(&bytes))
}

/// See the module documentation.
pub struct SaveAsRebindsTheDocument;

impl Check for SaveAsRebindsTheDocument {
    fn name(&self) -> &'static str {
        "save_as_rebinds_the_document_so_the_next_save_goes_to_the_new_file"
    }

    fn defect(&self) -> &'static str {
        "Save As writes the new file and leaves the document bound to the old one, so the next \
         Ctrl+S overwrites the file the operator was trying to leave — which is Save-a-copy \
         wearing Save-As's label"
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

/// One edit — a page rotation.
///
/// ★ The cheapest edit that needs no canvas aim, no armed tool and no typing:
/// two ribbon clicks. This check's subject is *where a save goes*, so the edit
/// should be the least interesting thing in it.
fn make_an_edit(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    crate::checks::ocr::click_tab(session, driver, ui_rect, "pages")?;
    crate::checks::ocr::click_command(session, driver, ui_rect, "pages.rotate_right")?;
    session.settle(20);
    Ok(())
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
            "input is disabled (--no-input). This check clicks ribbon controls. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new("the profile declares no ui-rect event, so no ribbon control can be found")
    })?;

    // --- scratch ------------------------------------------------------------
    //
    // ★ Both files live under `--out`. The fixture is copied rather than driven
    // in place, because this check deliberately WRITES to it — twice — and the
    // repository's own fixtures must come out of a run byte-identical.
    let source = workspace_root().join(FIXTURE);
    if !source.is_file() {
        return Err(Error::new(format!("fixture missing: {}", source.display())));
    }
    let original = ctx.out("save-as-original.pdf");
    let destination = ctx.out("save-as-destination.pdf");
    if let Some(dir) = original.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| Error::new(format!("cannot make {}: {e}", dir.display())))?;
    }
    // A destination left by a previous run would make phase B's "the file
    // exists" vacuous.
    let _ = std::fs::remove_file(&destination);
    std::fs::copy(&source, &original)
        .map_err(|e| Error::new(format!("cannot copy the fixture: {e}")))?;
    report.note(format!(
        "driving a scratch copy at {} — the repository's own fixture is never written to",
        original.display()
    ));

    // --- launch -------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("save-as.trace.txt"));
    spec.pdf = Some(original.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // ★ The picker's answer, so no system dialog opens. `app::files`' header and
    // the RAG note it quotes: *"Don't try to script the dialog."*
    spec.env.push((
        "PDFCER_DIAG_SAVE_PATH".to_owned(),
        destination.display().to_string(),
    ));
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
    session.maximize();
    session.settle(12);
    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(
            "the trace has no start line, so the diagnostic switch did not reach the process.",
        ));
    }
    let driver = Driver::new(session.window());
    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- phase A: edit, then Save As ----------------------------------------
    make_an_edit(&session, &driver, ui_rect)?;
    let before_save_as = digest_of(&original)?;
    report.note(format!(
        "one edit made; the original is {} bytes, digest {:016x}",
        before_save_as.0, before_save_as.1
    ));

    crate::checks::ocr::click_tab(&session, &driver, ui_rect, "file")?;
    crate::checks::ocr::click_command(&session, &driver, ui_rect, "file.save_as")?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(line) = trace.last(SAVE_AS) else {
        return Ok(Some(format!(
            "SAVE AS WROTE NOTHING: no `{SAVE_AS}` line followed the command. Either the \
             dispatch arm is missing (look for `{}` in the trace) or the picker's answer did \
             not reach it — this check supplies it through `PDFCER_DIAG_SAVE_PATH` and never \
             opens a dialog. Trace: {}.",
            driving::UNIMPLEMENTED_EVENT,
            session.trace_path().display()
        )));
    };
    report.note(format!("the document moved: `{}`", line.raw));

    // --- phase B: the new file is on disk -----------------------------------
    if !destination.is_file() {
        return Ok(Some(format!(
            "★ THE MOVE WAS TRACED AND NO FILE APPEARED at {}. `save-as` is emitted only after \
             `write_and_report` succeeded, so a missing file here means the write reported \
             success and produced nothing.",
            destination.display()
        )));
    }
    let new_after_save_as = digest_of(&destination)?;
    report.note(format!(
        "the new file is there — {} bytes, digest {:016x}",
        new_after_save_as.0, new_after_save_as.1
    ));

    // --- phase C: a SECOND edit, then Ctrl+S --------------------------------
    //
    // ★★ The second edit is not decoration. Without it the second save has
    // nothing to write, and a correct build doing nothing is indistinguishable
    // from a broken one doing nothing because it is aimed at the wrong file.
    make_an_edit(&session, &driver, ui_rect)?;
    report.note("a second edit, so the save below has something to write");
    driver.press_chord(&[crate::sys::vk::CONTROL], crate::sys::vk::S)?;
    session.settle(40);

    // --- ★★★ phase D: the falsifying one ------------------------------------
    let original_now = digest_of(&original)?;
    if original_now != before_save_as {
        return Ok(Some(format!(
            "★★★ THE NEXT SAVE WENT TO THE OLD FILE. {} was {} bytes (digest {:016x}) before \
             Save As and is {} bytes (digest {:016x}) after a later Ctrl+S.\n\n\
             This is the operator's request unmet, and it is the failure with real \
             consequences: he asked for Save As *\"so that we are then making edits in the \
             save as file instead of the original\"*, and this build wrote the copy and stayed \
             bound to the file he was trying to leave. Every cheaper oracle — the new file \
             exists, the line was traced — passes against it.\n\n\
             Look at `PdfcerApp::save_as_somewhere`: `doc.path` is the binding, and the title, \
             the tab and the next save all follow it.",
            original.display(),
            before_save_as.0,
            before_save_as.1,
            original_now.0,
            original_now.1
        )));
    }
    report.note(format!(
        "★★★ the original is byte-identical after a later save — {} bytes, digest {:016x}. The \
         document really moved",
        original_now.0, original_now.1
    ));

    // …and the new file did take the second edit, or "nothing moved" would also
    // satisfy the assertion above.
    let new_now = digest_of(&destination)?;
    if new_now == new_after_save_as {
        return Ok(Some(format!(
            "★★ NEITHER FILE CHANGED. The original is untouched, which is right, and {} is \
             also unchanged after a second edit and a Ctrl+S — so the save went nowhere at \
             all.\n\n\
             That satisfies the assertion above for the wrong reason, which is why this arm \
             exists. Look at whether the second edit registered (`save::has_unsaved_edits` \
             false would make Ctrl+S a legitimate no-op) before looking at the save itself.",
            destination.display()
        )));
    }
    report.note(format!(
        "★★ and the NEW file took the second edit — {} → {} bytes. So the save went somewhere, \
         and it went there",
        new_after_save_as.0, new_now.0
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The digest notices a changed byte and a truncation.
    ///
    /// ★ Phase D's whole verdict rests on this, so a digest that answered
    /// "unchanged" for a modified file would turn the check's most important
    /// assertion into a formality that always passes.
    #[test]
    fn the_digest_notices_a_single_changed_byte_and_a_truncation() {
        let a = b"%PDF-1.4 hello world";
        let mut b = a.to_vec();
        b[10] ^= 0x01;
        assert_ne!(digest(a), digest(&b), "one flipped bit must change it");
        assert_ne!(
            digest(a),
            digest(&a[..a.len() - 1]),
            "a truncation must change it, which is what the length is in the tuple for"
        );
    }

    /// ★★★ **The two files are different paths.**
    ///
    /// Trivial and load-bearing: if the destination resolved to the original,
    /// phase D would compare a file with itself and pass against every possible
    /// build, including one with no Save As at all.
    #[test]
    fn the_destination_is_not_the_original() {
        assert_ne!("save-as-original.pdf", "save-as-destination.pdf");
    }

    /// The check writes only inside the run's output directory.
    ///
    /// ★ Both names are relative and are joined to `--out` by `CheckContext`.
    /// This pins the intent: a future edit that reached for the repository's own
    /// fixture directly would be writing to a tracked file, twice, on every run.
    #[test]
    fn nothing_outside_the_output_directory_is_written() {
        for name in ["save-as-original.pdf", "save-as-destination.pdf"] {
            assert!(!name.contains('/') && !name.contains('\\'), "{name}");
        }
        assert!(FIXTURE.starts_with("fixtures/"), "the source is read-only");
    }
}
