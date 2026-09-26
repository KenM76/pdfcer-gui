//! `an_invalidating_save_is_warned_about` — **the window that stands between a
//! structural edit and a signed document's next revision.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/signature_save.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The commands, rung one per frame in order.
///
/// `mode.edit` first because `pages.delete` sits behind the Edit tab's
/// capability set, and because driving from a named mode makes the run
/// reproducible rather than dependent on whatever mode the last session left
/// behind.
///
/// `pages.delete` with no page selection acts on the **current page**, which is
/// `crate::panels::pages::ops::operands`' documented fallback — so no panel has
/// to be opened and no tile has to be clicked to make the save structural.
const INVOKE: &str = "mode.edit,pages.delete,file.save_copy";
/// This repository's own signed fixture. See the header.
const FIXTURE: &str = "../../fixtures/signed-two-pages.pdf";
/// The warning window's body region.
const BODY: &str = "dialog:signature";
/// Its proceed button.
const PROCEED: &str = "signature.proceed";
/// The line the guard writes when it holds a save.
const ASKED: &str = "signature-asked";
/// The line the drain writes when the operator authorises one.
const CONFIRMED: &str = "signature-confirmed";
/// The line the page verb writes. The precondition.
const DELETED: &str = "pages-deleted";
/// The line `app::save::write_and_report` writes once the bytes are on disk.
const WRITTEN: &str = "save-copy";
/// The variable that answers the save picker.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";

/// See the module documentation.
pub struct AnInvalidatingSaveIsWarnedAbout;

impl Check for AnInvalidatingSaveIsWarnedAbout {
    fn name(&self) -> &'static str {
        "an_invalidating_save_is_warned_about"
    }

    fn defect(&self) -> &'static str {
        "saving a digitally signed document after deleting a page writes the revision and says \
         nothing — the engine computes the invalidation and the shell never asks it, so the \
         operator learns what happened to their signature from somebody else's reader"
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

#[expect(
    clippy::too_many_lines,
    reason = "the five assertions are one narrative and each carries the failure text a reader \
              of a red run needs; splitting them would put the evidence in one function and the \
              sentence describing it in another"
)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks the warning window's proceed \
             button, which is the half that proves the guard RELEASES the save as well as \
             holding it.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // The fixture is NOT `ctx.pdf`, and that is the same ruling `reflow`
    // makes: the oracle here is bound to a document with one approval
    // signature and a page to spare, so a `--pdf` an operator passed would be
    // measured against an expectation that is not about it. A signed drawing
    // is also not something this check may improvise.
    let pdf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the signed fixture is missing at {}. Regenerate it: \
             python tools/gen-signed-fixture.py — the engine's own signature fixtures are all \
             one page, and this check has to delete one.",
            pdf.display()
        )));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // Removed first, and that is not tidiness: a file left by a previous run
    // would satisfy assertion 5 on a build that wrote nothing — the single most
    // likely way for a file-oracle check to go quietly green.
    let target = ctx.out("signed-copy.pdf");
    let _ = std::fs::remove_file(&target);
    if target.exists() {
        return Err(Error::new(format!(
            "{} could not be removed before the run, so a file found afterwards would prove \
             nothing. SKIPPED rather than failed.",
            target.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("signature-save.trace.txt"));
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
        "launched {} on fixtures/signed-two-pages.pdf as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(90);

    // --- 1. the precondition, and it SKIPs rather than failing ---------------
    //
    // `checks/mod.rs` rule 3: "the click selected something" must be ASSERTED
    // before "Delete removed it" can be a failure rather than a mystery. Here
    // the precondition is that the save is **structural** — without the page
    // delete the engine answers `ByteRangePreserved`, the surface is correctly
    // a status-bar note rather than a window, and a check that failed on that
    // would be reporting the right behaviour as a defect.
    let trace = session.trace()?;
    let Some(deleted) = trace.events(DELETED).last() else {
        return Err(Error::new(format!(
            "the page delete did not happen: no `{DELETED}` line, so this save would not be \
             structural and the engine would correctly answer `ByteRangePreserved`. That is a \
             fact about `pages.delete` rather than about the signature guard. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the save is structural: `{}`", deleted.raw));

    // --- 2. the guard fired, and the window drew -----------------------------
    let asked = trace.events(ASKED).last();
    let body = declared(&trace, ui_rect, BODY);
    if asked.is_none() || body.is_none() {
        return Ok(Some(format!(
            "★★★ A SIGNED DOCUMENT WAS SAVED WITH NO WARNING: `{ASKED}` {} and the `{BODY}` \
             region {}.\n\
             The engine computed the invalidation — `pdfcer-core`'s \
             `signature_impact_of_save` is a pure function of the session and the unit tests \
             assert it answers `Invalidated` for this fixture — so the missing link is the \
             shell's. Check that `Action::SaveCopy`'s arm in \
             `crates/pdfcer-gui/src/app/actions/apply.rs` calls \
             `DialogsState::ask_signature` and RETURNS on `true`. Regions beginning \
             `signature`: {}. Trace: {}.",
            if asked.is_some() {
                "was traced"
            } else {
                "was not traced"
            },
            if body.is_some() {
                "was declared"
            } else {
                "was not declared"
            },
            list(&declared_names(&trace, ui_rect, "signature")),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ the save was held and the window drew: `{}`",
        asked.map_or("", |e| e.raw.as_str())
    ));

    // --- 3. …and NOTHING was written while the question was on screen --------
    //
    // The assertion this check exists for. See the header: it is an absence,
    // and it is admissible because assertion 4 demands the same line from the
    // same build a few seconds later.
    if let Some(early) = trace.events(WRITTEN).last() {
        return Ok(Some(format!(
            "★★★ THE WINDOW APPEARED AND THE FILE WAS WRITTEN ANYWAY: `{}` was traced while \
             the question was still on screen.\n\
             This is worse than no warning at all — the operator is being asked to authorise \
             something that has already happened. The guard's `bool` is being discarded: \
             `ask_signature` returns *did I interrupt you*, and its arm must `return` on \
             `true`. Trace: {}.",
            early.raw,
            session.trace_path().display()
        )));
    }
    report.note("★★ no file was written while the question was on screen");

    // --- 4. the operator authorises it, and the save runs --------------------
    let driver = Driver::new(session.window());
    let Some(button) = stable_rect(&session, ui_rect, PROCEED, 8)? else {
        return Ok(Some(format!(
            "the warning window drew and declared no `{PROCEED}` region. The proceed button is \
             never greyed — an operator who has read the sentences is entitled to save — so an \
             absence means it was not laid out. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, PROCEED)?;
    driver.click_at(frame.declared_center(button))?;
    session.settle(60);

    let trace = session.trace()?;
    let Some(confirmed) = trace.events(CONFIRMED).last() else {
        return Ok(Some(format!(
            "★★ THE PROCEED BUTTON IS INERT: it was clicked at {button:?} in the window's own \
             frame and no `{CONFIRMED}` line appeared.\n\
             The answer is parked by the window and drained by \
             `PdfcerApp::resume_after_signature`, which runs once a frame after the dialogs \
             draw. A missing drain leaves a window the operator can only cancel, which makes \
             Save unusable on every signed document. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the operator authorised it: `{}`", confirmed.raw));

    let Some(written) = trace.events(WRITTEN).last() else {
        return Ok(Some(format!(
            "★★★ THE SAVE WAS AUTHORISED AND NEVER RAN: `{}` and no `{WRITTEN}` line.\n\
             Three candidates: `resume_after_signature` traced the answer and did not perform \
             the write; the picker was not answered (`{SAVE_PATH_ENV}` supplies it, and an \
             empty value means *cancelled*); or the write failed, which traces \
             `save-copy-failed` with the engine's own reason. Trace: {}.",
            confirmed.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the write ran: `{}`", written.raw));

    // --- 5. the oracle: a real file ------------------------------------------
    let Ok(bytes) = std::fs::read(&target) else {
        return Ok(Some(format!(
            "★★★ THE SAVE TRACED SUCCESS AND WROTE NO FILE: `{}` and {} does not exist.\n\
             This is the case a trace-only check cannot see. Trace: {}.",
            written.raw,
            target.display(),
            session.trace_path().display()
        )));
    };
    if !bytes.starts_with(b"%PDF-") {
        return Ok(Some(format!(
            "★★★ THE AUTHORISED COPY IS NOT A PDF: {} bytes were written to {} and they do not \
             begin `%PDF-`. Trace: {}.",
            bytes.len(),
            target.display(),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the warning held the save, the operator released it, and {} bytes of PDF reached \
         {} — the guard blocks and releases, which is the whole claim",
        bytes.len(),
        target.display()
    ));
    Ok(None)
}
