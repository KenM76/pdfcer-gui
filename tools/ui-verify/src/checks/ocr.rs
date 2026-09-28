//! `ocr_recognises_a_page_and_the_document_keeps_it` — the check for a feature
//! whose whole product is **text that was not in the document before**, and
//! whose whole risk is that it might have touched a file nobody saved.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr.md`.

//!
//! **The consequence here is specific and is reported rather than implied: the
//! Find bar's OCR offer is not driven by this check and cannot be.** Reaching
//! it needs a committed search, a search is committed by Enter in the Find
//! field, and no mouse gesture commits one — `bar::enter_intent` gives the step
//! buttons nothing to do until a search has already run. The offer's *rule* is
//! covered by unit test (`find::bar::tests`, including the falsifying case
//! where a zero-hit search on a page that has text offers nothing) and its
//! *drawing* is not covered at all. That gap is on the record here rather than
//! left for a reader of a PASS to assume away.
//!
//! # Which document it drives
//!
//! `--pdf` if given, otherwise `fixtures/synthetic-image-only.pdf`. **Pass
//! `--pdf` the day a genuine scanned document exists** — everything this check
//! asserts is true of any image-only PDF, and the synthetic one is a stand-in
//! rather than the subject.
//!
//! Note what the check does to whatever it is pointed at: **nothing**. That is
//! the whole of phase E. But it *reads* the file twice and compares, so the
//! file must be one the harness may read; it is never written to by design, and
//! a run in which it changes is a failure by definition.
//!
//! # The file picker is answered, not driven
//!
//! `PDFCER_DIAG_SAVE_PATH` supplies the save dialog's result and the dialog is
//! never opened. That is this project's established pattern for a native picker
//! (`app::files`' header, and the RAG note it quotes: *"Don't try to script the
//! dialog"*), and it is what makes phase D an assertion about **a file on disk**
//! rather than about a button having been pressed.
//!
//! # Every way this reports SKIP, and why none is a pass
//!
//! * no binary, no `--no-input`, no diagnostic channel — the harness never
//!   began;
//! * the fixture is missing, or already has extractable text (in which case it
//!   is the wrong fixture and OCR's contribution could not be isolated);
//! * a tab, a mode segment or a ribbon control was never declared, or took no
//!   click;
//! * the model weights are not beside the binary — the application says so by
//!   name, and a harness that called that a failure would be blaming the
//!   feature for a packaging step that was not run.

use std::path::PathBuf;

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the check runs in.
const READ: &str = "read";

/// The tab the command lives on. See [`READ`] for why it is not `tools`.
const TAB: &str = "file";

/// The command's id, which is also its declared region's suffix.
const COMMAND: &str = "file.ocr";

/// The fixture, relative to the workspace root.
const FIXTURE: &str = "fixtures/synthetic-image-only.pdf";

/// `ocr-started page=… models=… source=…`
const STARTED_EVENT: &str = "ocr-started";

/// `ocr-recognised page=… recognised=… written=… …`
const RECOGNISED_EVENT: &str = "ocr-recognised";

/// `ocr-refused reason=…`
const REFUSED_EVENT: &str = "ocr-refused";

/// `ocr-applied written=… skipped=… words=…` — the dialog's own record that it
/// raised the edit.
const APPLIED_EVENT: &str = "ocr-applied";

/// `ocr-layer page=… n=… epoch=… disclosures=…` — `vector_edit`'s record that
/// the **session** took it.
const EDIT_EVENT: &str = "ocr-layer";

/// How long to wait for recognition, in settle frames.
const RECOGNITION_FRAMES: u32 = 400;

/// The repository's own copy of the fixture, located from this crate rather
/// than from the working directory.
fn default_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(FIXTURE)
}

/// See the module documentation.
pub struct OcrRecognisesAPageAndTheDocumentKeepsIt;

impl Check for OcrRecognisesAPageAndTheDocumentKeepsIt {
    fn name(&self) -> &'static str {
        "ocr_recognises_a_page_and_the_document_keeps_it"
    }

    fn defect(&self) -> &'static str {
        "Recognise text is unreachable in the mode the operator asked for it in, produces no \
         layer, writes nothing, or — the one that cannot be caught anywhere else — writes the \
         recognised document back over the file that was opened"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, None) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// The same chain with the remembered recogniser set to OCRcer, asserting
/// that OCRcer is the engine that ran: the first check takes the default,
/// which is ocrs, so OCRcer's model and adapter are otherwise never driven.
pub struct OcrcerRecognisesAPageAndTheDocumentKeepsIt;

/// The `ocr_engine` preference key and the value `ocr-started engine=` carries.
const OCRCER: &str = "ocrcer";

impl Check for OcrcerRecognisesAPageAndTheDocumentKeepsIt {
    fn name(&self) -> &'static str {
        "ocrcer_recognises_a_page_and_the_document_keeps_it"
    }

    fn defect(&self) -> &'static str {
        "choosing OCRcer as the recogniser runs ocrs instead, or OCRcer cannot find its model in a packaged build, or it reads nothing from a page of plain text"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, Some(OCRCER)) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Seed `ocr_engine` in the preferences beside the (sandboxed) binary.
fn choose_engine(exe: &std::path::Path, engine: &str) -> Result<()> {
    let dir = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("userdata");
    std::fs::create_dir_all(&dir)
        .map_err(|e| Error::new(format!("could not create {}: {e}", dir.display())))?;
    crate::sandbox::write_prefs(&dir, &format!("ocr_engine = {engine}\n"))
        .map_err(|e| Error::new(format!("could not write preferences: {e}")))
}

/// The same chain with the recogniser set to PaddleOCR. The check first stages
/// the PP-OCR models the portable package ships beside the sandboxed binary,
/// where a packaged build finds them.
pub struct PaddleRecognisesAPageAndTheDocumentKeepsIt;

/// The `ocr_engine` preference key for PaddleOCR.
const PADDLE: &str = "paddle";

/// The environment variable naming a directory holding `det.onnx` and `rec.onnx`.
const PADDLE_MODELS_VAR: &str = "UI_VERIFY_PADDLE_MODELS";

impl Check for PaddleRecognisesAPageAndTheDocumentKeepsIt {
    fn name(&self) -> &'static str {
        "paddle_recognises_a_page_and_the_document_keeps_it"
    }

    fn defect(&self) -> &'static str {
        "choosing PaddleOCR runs another recogniser instead, or it cannot find models placed in the paddle models folder beside the binary, or it reads nothing from a page of plain text"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let staged = stage_paddle_models(ctx).map(|from| {
            report.note(format!("staged PP-OCR models from {}", from.display()));
        });
        let outcome = staged.and_then(|()| drive(ctx, &mut report, Some(PADDLE)));
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Copy `det.onnx` and `rec.onnx` into `models/paddle` beside the binary this
/// check will drive, with `dict.txt` when the source has one. The source is
/// `UI_VERIFY_PADDLE_MODELS`, else `crates/pdfcer-gui/assets/models/paddle`,
/// the directory the packager copies; a missing model is a SKIP, not a pass.
fn stage_paddle_models(ctx: &CheckContext) -> Result<std::path::PathBuf> {
    let from = std::env::var_os(PADDLE_MODELS_VAR).map_or_else(
        || {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../crates/pdfcer-gui/assets/models/paddle")
        },
        std::path::PathBuf::from,
    );
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive"))?;
    let to = exe
        .parent()
        .ok_or_else(|| Error::new("the binary has no parent directory"))?
        .join("models")
        .join(PADDLE);
    std::fs::create_dir_all(&to)
        .map_err(|e| Error::new(format!("could not create {}: {e}", to.display())))?;
    for file in ["det.onnx", "rec.onnx"] {
        let source = from.join(file);
        if !source.is_file() {
            return Err(Error::new(format!(
                "no PP-OCR model at {}; set {PADDLE_MODELS_VAR} to a directory holding det.onnx and rec.onnx.",
                source.display()
            )));
        }
        std::fs::copy(&source, to.join(file))
            .map_err(|e| Error::new(format!("could not copy {}: {e}", source.display())))?;
    }
    let dict = from.join("dict.txt");
    if dict.is_file() {
        std::fs::copy(&dict, to.join("dict.txt"))
            .map_err(|e| Error::new(format!("could not copy {}: {e}", dict.display())))?;
    }
    Ok(from)
}

/// A cheap content digest — length plus FNV-1a over the bytes.
fn digest(bytes: &[u8]) -> (usize, u64) {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (bytes.len(), hash)
}

/// Click a ribbon band control by command id, and confirm the shell saw it.
pub(super) fn click_command(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    id: &str,
) -> Result<()> {
    let region = format!("{}{id}", driving::ITEM_PREFIX);
    let trace = session.trace()?;
    let rect = driving::declared(&trace, ui_rect, &region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region, so `{id}` has no control to click. \
             Band controls it did declare: {}.",
            driving::list(&driving::declared_names(
                &trace,
                ui_rect,
                driving::ITEM_PREFIX
            ))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{region}` was declared at {rect:?}, which has no usable area to click."
        )));
    }
    let before = driving::shell_trace(session)?
        .events(driving::INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(16);
    let after = driving::shell_trace(session)?
        .events(driving::INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count();
    if after <= before {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{} id={id}` line, so no click reached the \
             ribbon and nothing after it would mean anything. Trace: {}.",
            driving::INVOKE_EVENT,
            session.trace_path().display()
        )));
    }
    Ok(())
}

/// Click a tab by id.
pub(super) fn click_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    tab: &str,
) -> Result<()> {
    let region = format!("ribbon.tab.{tab}");
    let trace = session.trace()?;
    let rect = driving::declared(&trace, ui_rect, &region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region. Tabs it did declare: {}. \
             ★ If `{TAB}` is missing while `{READ}` is selected, that is the finding rather \
             than a harness problem: the command would be unreachable in the mode the operator \
             asked for it in.",
            driving::list(&driving::declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    let before = driving::shell_trace(session)?
        .events(driving::TAB_EVENT)
        .filter(|l| l.get("tab") == Some(tab))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(12);
    let after = driving::shell_trace(session)?
        .events(driving::TAB_EVENT)
        .filter(|l| l.get("tab") == Some(tab))
        .count();
    if after <= before {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{} tab={tab}` line.",
            driving::TAB_EVENT
        )));
    }
    Ok(())
}

/// Click a region the *application* declared (a dialog control), by name.
pub(super) fn click_region(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    name: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let rect = driving::declared(&trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{name}` region. Regions it did declare beginning \
             `ocr-`: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, "ocr-"))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{name}` was declared at {rect:?}, which has no usable area to click."
        )));
    }
    // **`frame_of`, never `session.frame()`.**
    //
    //
    driver.click_at(driving::frame_of(session, &trace, ui_rect, name)?.declared_center(rect))?;
    session.settle(12);
    Ok(())
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
/// `engine`: `None` drives whatever recogniser the build defaults to; `Some`
/// seeds that preference and fails unless it is the one that ran.
fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    engine: Option<&str>,
) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control and two dialog \
             buttons. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;

    // `--pdf` first, the repository default second — and the ORDER is a
    // repair rather than a preference.
    //
    // The first version of this check resolved the fixture from
    // `ctx.source_root`, which is the *staleness comparison's* root and is
    // `None` under `--no-staleness-check`. Falsifying the check against a
    // deliberately broken build was supposed to run against a COPY of the
    // fixture in a scratch directory; the path collapsed to `.` instead, the
    // planted build overwrote the repository's real fixture, and the check
    // reported the overwrite correctly while the harness was the thing that
    // aimed it at the wrong file.
    //
    // Both halves of that are worth keeping. The check did its job — it is the
    // reason the damage was noticed within one run rather than at the next
    // commit — and a harness that decides which file to destroy from a flag
    // about staleness had no business doing so. `--pdf` is now the explicit
    // control, which also makes the right thing possible the day a genuine
    // scanned document exists: point this check at it.
    // The default is resolved from THIS CRATE'S manifest directory, not from
    // the working directory and not from `--source-root`. Both of those were
    // tried and both were wrong: `--source-root` defaults to `crates`, so the
    // fixture resolved to `crates/fixtures/...` and the check SKIPped; and a
    // bare `.` depends on where the harness was invoked from, which is how the
    // planted-build run came to aim at the repository's own copy.
    // **This check pins its own fixture and IGNORES `--pdf`.**
    //
    //
    // A check whose subject is *"did the recogniser read this page"* cannot
    // take an arbitrary document, because on a document that already has text
    // the honest answer is *"it declined to look"* and that is neither a pass
    // nor a defect. The fixture is the only kind of document on which the
    // result is unambiguous: any text in the output came from the recogniser.
    //
    // A suite-wide `--pdf` is a convenience for the checks that need *some*
    // drawing. This one needs a specific absence.
    let fixture = default_fixture();
    if !fixture.is_file() {
        return Err(Error::new(format!(
            "the image-only fixture is not at {}. Generate it:\n cargo test -p pdfcer-gui \
             --lib write_synthetic_image_only -- --ignored",
            fixture.display()
        )));
    }
    let before_bytes = std::fs::read(&fixture)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", fixture.display())))?;
    let before = digest(&before_bytes);
    report.note(format!(
        "fixture {} — {} bytes, digest {:016x}",
        fixture.display(),
        before.0,
        before.1
    ));

    if let Some(e) = engine {
        choose_engine(&exe, e)?;
        report.note(format!("seeded `ocr_engine = {e}`"));
    }

    // --- launch ------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("ocr.trace.txt"));
    spec.pdf = Some(fixture.clone());
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
    session.settle(40);
    // **MAXIMIZE, and this line is a repair.**
    //
    // Found on 2026-09-01 by writing `checks::ocr_progress` and watching it
    // SKIP for a reason that turned out to apply to THIS check too: at the
    // window's default width the `file` tab's **Recognise group collapses**,
    // and a collapsed group declares `ribbon.group.file.recognise.collapsed`
    // instead of `ribbon.item.file.ocr`. The harness then reports *"the
    // application declared no `ribbon.item.file.ocr` region"* — which reads as
    // the command having been removed, and is in fact the ribbon doing exactly
    // what a ribbon is for.
    //
    // This check had therefore been reporting **SKIP** rather than PASS, and
    // a SKIP is not a failure, so nothing was red and nothing prompted a look.
    // `Session::maximize`'s own doc comment describes this precise symptom —
    // *"would have been handed ten controls ending at `file.print`, and would
    // have reported a shipped feature as missing"* — so the lesson was already
    // written down and this call site simply never got it.
    //
    // Opt-in per check by design, because a maximised window is a different
    // layout and several checks measure the layout. Nothing here does: this
    // check's subject is a command, a dialog and a file on disk.
    session.maximize();
    session.settle(12);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process and \
             this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    let driver = Driver::new(session.window());

    // --- phase A: reach the command IN READ --------------------------------
    driving::click_mode_segment(&session, &driver, ui_rect, READ)?;
    click_tab(&session, &driver, ui_rect, TAB)?;
    report.note(format!(
        "the `{TAB}` tab is present and took a click while the mode selector is on `{READ}` — so \
         the command is reachable in the mode the operator asked for it in, which the \
         specification's Tools placement would not have been"
    ));
    click_command(&session, &driver, ui_rect, COMMAND)?;

    // --- phase B: the dialog is up ------------------------------------------
    session.settle(16);
    let trace = session.trace()?;
    if driving::declared(&trace, ui_rect, "ocr-dialog").is_none() {
        return Ok(Some(format!(
            "THE COMMAND OPENED NO DIALOG. `{COMMAND}` was invoked — the shell traced it — and \
             the application declared no `ocr-dialog` region on any frame afterwards. Either the \
             dispatch arm is missing (look for `{}` in the trace) or `DialogsState::show` is not \
             drawing it. Trace: {}.",
            driving::UNIMPLEMENTED_EVENT,
            session.trace_path().display()
        )));
    }
    report.note("the dialog is up and declared its own rect");

    // --- phase C: recognise -------------------------------------------------
    if driving::declared(&trace, ui_rect, "ocr-run").is_none() {
        // The dialog is showing a refusal rather than a run button. That is a
        // legitimate state and this check must not call it a failure: the
        // commonest cause by far is that the model weights are not beside this
        // binary, which is a packaging step rather than a defect in the
        // feature.
        return Err(Error::new(format!(
            "the dialog drew no `ocr-run` control, so it is refusing rather than offering to \
             recognise. The overwhelmingly likely cause is that the `models/ocrs` folder is not \
             beside {} — this check needs a PACKAGED build, or the weights copied next to the \
             executable. Point --exe at a folder produced by `tools/package-portable.py`.",
            exe.display()
        )));
    }
    click_region(&session, &driver, ui_rect, "ocr-run")?;
    session.settle(RECOGNITION_FRAMES);

    let trace = session.trace()?;
    if let Some(refusal) = trace.last(REFUSED_EVENT) {
        return Ok(Some(format!(
            "RECOGNITION REFUSED: `{}`. The run control was drawn — so the preflight checks \
             passed and this build has both an engine and models — and the job then came back \
             with a named refusal. Trace: {}.",
            refusal.raw,
            session.trace_path().display()
        )));
    }
    let Some(recognised) = trace.last(RECOGNISED_EVENT) else {
        return Ok(Some(format!(
            "RECOGNITION NEVER FINISHED. `{STARTED_EVENT}` was {}, and no `{RECOGNISED_EVENT}` \
             or `{REFUSED_EVENT}` line followed within {RECOGNITION_FRAMES} frames. A job that \
             neither answers nor refuses leaves the dialog saying `Recognising…` forever, which \
             is the one outcome `ocr::Job::poll`'s disconnected arm exists to prevent. Trace: {}.",
            if trace.last(STARTED_EVENT).is_some() {
                "traced"
            } else {
                "NOT traced either, so the click never reached the button"
            },
            session.trace_path().display()
        )));
    };
    report.note(format!("recognition finished: `{}`", recognised.raw));
    if let Some(wanted) = engine {
        let ran = trace
            .last(STARTED_EVENT)
            .and_then(|l| l.get("engine").map(str::to_owned));
        report.note(format!("the recogniser that ran: {ran:?}"));
        if ran.as_deref() != Some(wanted) {
            return Ok(Some(format!(
                "`ocr_engine = {wanted}` was seeded and `{STARTED_EVENT}` names {ran:?}: the remembered recogniser is not the one that runs. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }

    let words = recognised.get_usize("recognised").unwrap_or(0);
    if words == 0 {
        return Ok(Some(format!(
            "RECOGNITION PRODUCED NO WORDS. The job completed and reported `{}`. On a page \
             whose every mark is text this means the detector or the recogniser found nothing \
             — check `ocr::fitted_dpi` against `ocr::TARGET_PIXELS`, which is the constant \
             that most affects this and which measured a 13× accuracy swing across five \
             resolutions.",
            recognised.raw
        )));
    }
    if recognised.get_usize("pages").unwrap_or(0) == 0 {
        return Ok(Some(format!(
            "RECOGNITION PLACED NOTHING. `{}` reports words found and no page kept any of \
             them, which means every word was refused by `words_to_page_space_on` — the \
             page-space mapping, not the recogniser. That is the failure mode that is \
             invisible on screen, because an OCR layer is Table 106 mode 3 and a page whose \
             every word is misplaced looks exactly like a page whose every word is right.",
            recognised.raw
        )));
    }

    // --- phase D: the layer reached the OPEN DOCUMENT -----------------------
    //
    // **This phase used to click a Save control.** Until 2026-08-27 the
    // only way out of this dialog was Save-a-copy, because
    // `ocr::layer::add_ocr_layer` took an immutable `&Document` and returned a
    // whole PDF — there was nothing to put the layer *into*. The operator's
    // objection was exactly that: *"Why do I have to save a copy instead of
    // just go back into my pdf and save over it?"*
    //
    // `EditSession::add_ocr_layer` (engine Pass 135.0) made recognition an
    // edit. So the thing to assert is no longer *a file appeared where I asked*
    // but *the open document changed*, which is a different link and a stronger
    // one: the words are in the session the operator is looking at.
    let trace = session.trace()?;
    let Some(applied) = trace.last(APPLIED_EVENT) else {
        return Ok(Some(format!(
            "RECOGNITION PRODUCED NOTHING THE DOCUMENT KEPT. The worker reported `{}` and no \
             `{APPLIED_EVENT}` line followed, so the dialog either never raised \
             `Action::ApplyOcr` or the action was dropped before it reached `vector_edit`. \
             Trace: {}.",
            recognised.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("the layer was applied: `{}`", applied.raw));

    let Some(edit) = trace.last(EDIT_EVENT) else {
        return Ok(Some(format!(
            "★ THE EDIT NEVER REACHED THE SESSION. `{APPLIED_EVENT}` was traced — the dialog \
             believes it applied a layer — and no `{EDIT_EVENT}` line followed it. \
             `vector_edit` emits that line on every successful edit and a `…-refused` line on \
             every declined one, so the absence of both means the call did not happen. The \
             operator's version of this state is a dialog that says the text is in their \
             document and a document with no text in it. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the session took the edit: `{}`", edit.raw));

    // --- phase E: the falsifying one --------------------------------------
    //
    //
    // This is still a genuinely falsifying assertion rather than a confirming
    // one. It fails against the plausible wrong implementation and there is no
    // way to satisfy it accidentally.
    let after_bytes = std::fs::read(&fixture)
        .map_err(|e| Error::new(format!("cannot re-read {}: {e}", fixture.display())))?;
    let after = digest(&after_bytes);
    if after != before {
        return Ok(Some(format!(
            "★ THE DOCUMENT ON DISK WAS MODIFIED BY A RECOGNITION NOBODY SAVED. {} was {} \
             bytes (digest {:016x}) before the run and is {} bytes (digest {:016x}) after it.\n\n\
             Recognition is an EDIT as of 2026-08-27 — it belongs in the session, and it \
             reaches the file only when the operator saves. Nothing in this run saved. Every \
             other assertion here passed and the operator's file changed under them anyway. \
             Look at `Action::ApplyOcr`'s arm and at anything on the frame path that calls \
             `save_in_place`.",
            fixture.display(),
            before.0,
            before.1,
            after.0,
            after.1
        )));
    }
    report.note(format!(
        "★ the file on disk is byte-identical after the run — {} bytes, digest {:016x}. The \
         recognition is in the session and nowhere else, which is where an unsaved edit \
         belongs",
        after.0, after.1
    ));

    // --- what this does and does not establish ------------------------------
    report.note(
        "NOT established by this check: recognition quality on real scanned material. The \
         fixture is a rendered page with no scanner noise, skew, JPEG ringing or uneven \
         lighting, so it flatters the recogniser — see `crates/pdfcer-gui-base/src/ocr/fixture.rs`'s \
         header. What is established is the chain: reachable in Read, recognises, discloses, \
         writes where it was told, and leaves the original alone",
    );
    report.note(
        "NOT covered here: the Find bar's OCR offer. Reaching it needs a committed search, a \
         search is committed by Enter, and synthetic keystrokes do not reach the target window \
         from this session (see find_bar). Its rule is covered by unit test including the \
         falsifying case; its drawing is covered by nothing, and that gap is stated rather than \
         implied by a green result",
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The digest changes when one byte does, and the length is part of it.**
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
        assert_eq!(digest(a), digest(a), "and it must be stable");
    }

    /// The mode this check drives is the one the operator's instruction is
    /// about, and the tab is one that mode is actually shown.
    #[test]
    fn the_check_drives_read_and_a_tab_read_actually_has() {
        assert_eq!(READ, "read");
        assert_eq!(TAB, "file");
        assert!(
            COMMAND.starts_with(TAB),
            "a command id names its owning tab, so `{COMMAND}` on `{TAB}` must share the prefix \
             — and if it does not, the manifest and this check disagree about where it is"
        );
    }

    /// The fixture path is the generated image-only one, not the drawing.
    #[test]
    fn the_fixture_is_the_image_only_one() {
        assert!(FIXTURE.contains("image-only"));
        assert!(
            !FIXTURE.contains("titleblock"),
            "a1-titleblock.pdf has extractable text, so OCR's contribution to it could not be \
             isolated from what was already there"
        );
    }

    /// Every trace event this check reads is spelled once.
    #[test]
    fn the_event_names_are_distinct() {
        let all = [
            STARTED_EVENT,
            RECOGNISED_EVENT,
            REFUSED_EVENT,
            APPLIED_EVENT,
            EDIT_EVENT,
        ];
        for (i, a) in all.iter().enumerate() {
            assert!(a.starts_with("ocr-"), "{a} is not an OCR event");
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    /// The check never writes into `fixtures/`.
    #[test]
    fn the_output_path_is_not_beside_the_fixture() {
        assert_eq!(
            std::path::Path::new(FIXTURE)
                .parent()
                .and_then(std::path::Path::to_str),
            Some("fixtures")
        );
    }
}
