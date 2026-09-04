//! `an_encrypted_document_can_be_opened_with_its_password` — the capability the
//! audit found missing, driven end to end.
//!
//! # ★★★ The defect
//!
//! `OPERATOR_REQUESTS.md` O108. Found 2026-09-03 by
//! `tools/security-coverage.py` — an instrument keyed on `pdfcer-core`'s own API
//! rather than on any document of ours — while answering the operator's
//! question *"can we get all of the encryption and signature features that have
//! been implemented in the engine under one new tab?"*
//!
//! **An encrypted PDF could not be opened at all.** The shell detected the case
//! perfectly and had nowhere to type a password:
//! `Document::load_with_password` and `from_bytes_with_password` were named in
//! exactly one place in the crate — a doc comment listing the loading entry
//! points — and nothing called either.
//!
//! ★★ That doc comment is why the coverage tool strips comment-only lines
//! before it searches. Its first run reported `load_with_password` as
//! **reached**, on the strength of that one sentence, which would have recorded
//! the single most important missing capability in the area as already built —
//! in the instrument written to find exactly this.
//!
//! # ★★★ Why a driven check and not the four unit tests
//!
//! Because every link in this chain is a **call site**, and this project was
//! founded on the observation that a call site's effect is observable only in a
//! running process. The unit tests prove: an empty box raises no action, a typed
//! password becomes one action, the action cannot print the password, a
//! rejection clears the field. All four pass on a build where the prompt is
//! **never shown**, because nothing in them asks whether
//! `Status::NeedsPassword` reaches `DialogsState::ask_for_password`.
//!
//! That link is the one that did not exist for as long as the detection did.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | launch on an encrypted fixture with **no** password | the document does NOT open, and `dialog:password` is declared — the prompt appeared unprompted, driven by document state |
//! | B | type a **wrong** password and press Open | the prompt is still up, and the trace says `password-rejected … reason=wrong` |
//! | C | type the right one | `password-accepted`, the prompt retires, and the canvas reports a page |
//! | D | read back the whole trace | **the password appears nowhere in it** |
//!
//! ★★★ **D is not decoration and it is the phase most worth having.** The
//! password travels through an `Action` in a queue, this crate traces liberally
//! to stderr under `PDFCER_DIAG`, and *this harness captures that stderr to a
//! file it keeps as evidence*. One `format!("{action:?}")` anywhere on the path
//! would write the operator's password into `target/ui-verify/`, in plain text,
//! in a directory whose whole purpose is to be kept and read — and it would fail
//! nothing and look like an ordinary diagnostic in review.
//!
//! `crate::secret::Secret` makes it unrepresentable and two unit tests assert
//! the type. **This asserts the whole running program**, over the real captured
//! file, which is the only place the claim is actually about.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The prompt's body region.
const BODY: &str = "dialog:password";
/// Its Open button.
const OPEN: &str = "password.open";
/// The fixture, pinned: this check's subject is *an encrypted PDF*, and
/// `--pdf` will be an ordinary one.
const FIXTURE: &str = "fixtures/encrypted-aes-128.pdf";
/// Its user password. Published in the fixture's own `PROVENANCE.md` and in
/// pdfcer's; it is synthetic test data.
const RIGHT: &str = "userpw";
/// A password that is not it.
const WRONG: &str = "notthepassword";

/// See the module documentation.
pub struct AnEncryptedDocumentCanBeOpenedWithItsPassword;

impl Check for AnEncryptedDocumentCanBeOpenedWithItsPassword {
    fn name(&self) -> &'static str {
        "an_encrypted_document_can_be_opened_with_its_password"
    }

    fn defect(&self) -> &'static str {
        "an encrypted PDF cannot be opened at all — the shell detects that a password is needed, \
         says so in the document tab, and offers nowhere to type one, so the file is simply \
         unreachable (OPERATOR_REQUESTS.md O108)"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check types a password and presses a button. \
             Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let pdf = ctx.source_root.clone().unwrap_or_default().join(FIXTURE);
    let pdf = if pdf.exists() {
        pdf
    } else {
        std::path::PathBuf::from(FIXTURE)
    };
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is not on disk. It is a synthetic encrypted PDF; see \
             `fixtures/encrypted-aes-128.PROVENANCE.md`."
        )));
    }
    if ctx.pdf.is_some() {
        report.note(format!(
            "--pdf was supplied and is IGNORED: this check pins {FIXTURE}, because its subject \
             is an ENCRYPTED document and an ordinary one cannot exhibit the case"
        ));
    }

    // --- A: it opens to a prompt, not to a page ----------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("password_prompt.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {} on an AES-128 encrypted document",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "opening an encrypted document declared no `{BODY}` region, so no password prompt \
             appeared and the file cannot be opened by any route. That is the defect this check \
             exists for: the shell DOES detect the case — look for a `needs-password` status in \
             the trace — and offers nowhere to answer it. Dialog regions declared this run: {}.",
            list(&declared_names(&trace, ui_rect, "dialog:"))
        )));
    }
    report.note("the prompt appeared on its own, driven by the document's state");

    // --- B: a wrong password is refused, and SAYS it was refused -----------
    //
    // ★ Typed through the OS, not seeded: the field is a real `TextEdit` behind
    // a real viewport, and a check that wrote the string into memory would be
    // asserting about a program nobody can operate.
    driver.type_ascii(WRONG)?;
    session.settle(8);
    let open = declared(&session.trace()?, ui_rect, OPEN)
        .ok_or_else(|| Error::new(format!("the prompt declared no `{OPEN}` button to press.")))?;
    let frame = crate::checks::driving::frame_of(&session, &session.trace()?, ui_rect, OPEN)?;
    driver.click_at(frame.declared_center(open))?;
    session.settle(20);

    let trace = session.trace()?;
    if !trace
        .events("password-rejected")
        .any(|l| l.get("reason") == Some("wrong"))
    {
        return Ok(Some(
            "a wrong password produced no `password-rejected reason=wrong` line. Either it was \
             accepted — which would mean the document opened without the right password — or the \
             press never reached the button."
                .to_owned(),
        ));
    }
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(
            "the prompt CLOSED after a wrong password. It must stay open and say so: closing \
             leaves the operator with a document that did not open and no way back to the box \
             except re-opening the file."
                .to_owned(),
        ));
    }
    report.note("a wrong password was refused and the prompt stayed up");

    // --- C: the right one opens it -----------------------------------------
    driver.type_ascii(RIGHT)?;
    session.settle(8);
    let trace = session.trace()?;
    let open = declared(&trace, ui_rect, OPEN)
        .ok_or_else(|| Error::new(format!("the prompt no longer declares `{OPEN}`.")))?;
    let frame = crate::checks::driving::frame_of(&session, &trace, ui_rect, OPEN)?;
    driver.click_at(frame.declared_center(open))?;
    session.settle(30);

    let trace = session.trace()?;
    let mut failures: Vec<String> = Vec::new();
    if trace.events("password-accepted").next().is_none() {
        failures.push(
            "the correct password produced no `password-accepted` line, so the document did not \
             open. The fixture's user password is published in its PROVENANCE file; if it has \
             changed, this check is aimed at the wrong string and says so here rather than \
             reporting the feature broken."
                .to_owned(),
        );
    }
    // ★★★ "THE PROMPT CLOSED" IS NOT ASSERTED, AND THE REASON IS A FINDING
    // ABOUT THIS HARNESS RATHER THAN A GAP IN THE CHECK.
    //
    // The obvious assertion is `declared(&trace, ui_rect, BODY).is_none()` —
    // the region retired, so the window is gone. It does not work, and it does
    // not work for EVERY dialog in this application. Measured 2026-09-03:
    //
    //   grep -c 'ui-rect-gone name=dialog:' target/ui-verify/*.trace.txt
    //   -> 0 in every trace in the directory, across the whole suite.
    //
    // A dialog's regions are declared inside a child viewport's closure. When
    // the dialog is dropped that closure stops running, and the parent's
    // end-of-frame census -- which is what emits `ui-rect-gone` -- has never
    // once retired one. So the `ui-rect` channel can say a dialog APPEARED and
    // cannot say it went away, and `declared` answers with the fossil.
    //
    // => This check asserts closure through what IS observable: the
    // application's own `password-accepted` line, emitted at the one place that
    // sets the dialog to `None`, plus the document being on screen behind it.
    // Written up in `D:/dev/rag/egui/` so the next check that wants "did the
    // dialog close?" finds the answer rather than the fossil.
    //
    // * The first version of this check DID assert it, failed, and reported
    // "the prompt is still on screen after the correct password" about a build
    // whose prompt had closed. A confident, specific, wrong defect report --
    // this harness's own stated worst outcome, and another instance of a change
    // log read as a snapshot.
    // ★ …and the DOCUMENT is on screen, which is the operator's actual claim.
    // `password-accepted` alone would pass on a build that closed the prompt and
    // opened nothing.
    let canvas_page = trace
        .last(ctx.profile.vocab.canvas_event)
        .and_then(|l| l.get_usize("page"));
    if canvas_page.is_none() {
        failures.push(format!(
            "the password was accepted and the canvas reported no page, so nothing was drawn. \
             Trace: {}.",
            session.trace_path().display()
        ));
    } else {
        report.note("the document opened and the canvas is showing a page");
    }

    // --- D: the password is nowhere in the evidence ------------------------
    //
    // ★★★ See the module header. This is the phase most worth having, and it
    // reads the file this harness itself wrote.
    let raw = std::fs::read_to_string(session.trace_path()).unwrap_or_default();
    for secret in [RIGHT, WRONG] {
        if raw.contains(secret) {
            failures.push(format!(
                "THE PASSWORD IS IN THE TRACE FILE. `{}` contains the string `{secret}`, which \
                 means something on the path formatted it — an `{{:?}}` on the action queue is \
                 the likely culprit, and `crate::secret::Secret` exists to make that \
                 unrepresentable. This harness KEEPS that file as evidence.",
                session.trace_path().display()
            ));
        }
    }
    if failures.is_empty() {
        report.note("neither password appears anywhere in the captured trace");
    }

    let shot = ctx.out("password-prompt.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
        }
        Err(e) => {
            report.note(format!("the window could not be captured ({e})"));
        }
    }

    if failures.is_empty() {
        Ok(None)
    } else {
        Ok(Some(failures.join("  |  ")))
    }
}
