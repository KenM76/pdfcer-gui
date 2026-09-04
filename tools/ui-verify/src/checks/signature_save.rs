//! `an_invalidating_save_is_warned_about` — **the window that stands between a
//! structural edit and a signed document's next revision.**
//!
//! # What this is for
//!
//! `pdfcer-core` publishes two verbs written specifically so a front end could
//! answer *"what will this save do to the signatures already in this
//! document?"*:
//!
//! ```text
//! session.signature_impact_of_save(mode: SaveMode) -> SignatureImpact
//! session.changes_structure() -> bool
//! ```
//!
//! Until 2026-08-28 this shell called neither. `Ctrl+S` over a signed drawing
//! whose page had been deleted wrote a revision and said nothing — not before,
//! not after, on any surface. `crate::checks::signature_save` is the assertion
//! that the fix is reachable by a hand on a keyboard rather than only by a unit
//! test.
//!
//! ## ★★★ Why no unit test can make this claim
//!
//! `crates/pdfcer-gui/src/dialogs/signature.rs` already asserts, headlessly and
//! against this same fixture, that the engine reports the invalidation and that
//! `ask_for` builds a dialog for it. Every one of those assertions passes on a
//! build where:
//!
//! * `DialogsState::ask_signature` is never called from the save arms, so the
//!   window is constructed by nothing and the save runs unannounced;
//! * the guard is called and its `bool` is dropped, so the window appears **and
//!   the file is written anyway**, which is the worst outcome available — the
//!   operator is asked a question whose answer is ignored;
//! * the window draws and its proceed button is never wired, so the save can
//!   only be cancelled and `Ctrl+S` looks broken.
//!
//! Each of those is a whole-link failure between a passing decision function
//! and a running application, which is the class `PROJECT_PLAN.md` §4 built
//! this harness for. So this check asserts the link in **both directions**, and
//! that pairing is the point:
//!
//! | # | assertion | the build it fails against |
//! |---|---|---|
//! | 1 | the page delete happened | (precondition — SKIP, not FAIL) |
//! | 2 | `signature-asked` is traced and `dialog:signature` is declared | the guard is not called; the save is silent |
//! | 3 | **no `save-copy` line exists yet** | the guard is called and its answer discarded; the write already happened behind the window |
//! | 4 | after the click: `signature-confirmed`, then `save-copy` | the proceed button is inert, and Save cannot be completed at all |
//! | 5 | the file on disk begins `%PDF-` | the write was traced and went nowhere |
//!
//! ★★ **Assertion 3 is the one worth the whole check.** It is an absence, and
//! `checks/mod.rs`'s rule 4 says an absence is only evidence once the thing
//! that would have produced it is shown to work. It is admissible here for
//! exactly that reason: the same `save-copy` line is then *demanded* in
//! assertion 4, in the same run, from the same build. A build that never writes
//! satisfies 3 and fails 4.
//!
//! ## ★★ The fixture, and why this repository had to author one
//!
//! `fixtures/signed-two-pages.pdf`, built by `tools/gen-signed-fixture.py`,
//! whose header carries the argument. The short version is that the engine's
//! own signature corpus
//! (`D:\Dev\pdfcer\fixtures\synthetic\signature\`) is three **one-page**
//! documents — they were built for `signature::byte_range_coverage`, which is
//! arithmetic over byte offsets and needs no pages — and this check has to make
//! the save **structural**, which it does by deleting a page. A one-page
//! document has no page it can spare: `pages.delete` over the only page is a
//! refusal, and the check would SKIP for a reason that has nothing to do with
//! signatures.
//!
//! It carries an **approval** signature with no `/Reference`, deliberately, so
//! `SignatureImpact::documentation_basis` answers `ImpactBasis::ConservativeReport`
//! — the arm where ISO 32000-1 is silent and pdfcer reports the cautious answer
//! under rule 4, which is the wording hardest to get right and therefore the
//! one worth driving.
//!
//! ## ★ Why it drives Save-a-**copy** and never Save-in-place
//!
//! Not a preference: `Action::Save` writes over the document's own file, and
//! the document here is a **committed fixture**. A check that drove it would
//! rewrite `fixtures/signed-two-pages.pdf` on every run — the exact hazard
//! `checks/ocr.rs` records having hit, one directory over — and the fixture's
//! whole value is that it is byte-authored and stable.
//!
//! `file.save_copy` answers its picker from `PDFCER_DIAG_SAVE_PATH`, so the
//! bytes land in the run's own output directory and the fixture is never
//! opened for writing. The guard under test is the same one on both routes —
//! `crates/pdfcer-gui/src/app/actions/apply.rs` asks it in both arms — so
//! nothing about the assertion is weakened by taking the safe route.
//!
//! ## What this does NOT cover
//!
//! * **The certification wording.** `ImpactBasis::SpecSourced` needs a
//!   `/DocMDP` transform in a signature's `/Reference`, and this repository has
//!   no such fixture. The two footings are asserted apart in
//!   `crates/pdfcer-gui/src/text/signature.rs`'s own tests; what is undriven is
//!   the certified *window*, which differs from this one only in three strings.
//! * **The `ByteRangePreserved` note.** It appears on the status bar's
//!   disclosure row after a save with no structural change, and reading a bar
//!   sentence needs a pixel oracle rather than a trace one. Its decision is
//!   unit-tested against this same fixture
//!   (`the_signed_fixture_moves_from_a_note_to_a_window_when_a_page_goes`).
//! * **Cancel.** The window's Cancel must leave no file, and asserting it needs
//!   a second launch. Stated as a gap rather than folded in, because a check
//!   that clicked two buttons in one run could not say which one the failure
//!   belonged to.

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
    // ★ The fixture is NOT `ctx.pdf`, and that is the same ruling `reflow`
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

    // ★ Removed first, and that is not tidiness: a file left by a previous run
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
