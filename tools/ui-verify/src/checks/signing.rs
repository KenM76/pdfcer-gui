//! `checks::signing` — **a document is signed, and the signature is read back
//! out of the file by a different subsystem in a different process**
//!
//! The driven half of `crate::sign` / `crate::dialogs::sign`, answering the
//! operator's report of 2026-09-03: *a document cannot be signed.*
//!
//! # ★★★ THE ONE THING THIS CHECK EXISTS FOR, AND WHY A TRACE LINE IS NOT IT
//!
//! `pdfcer-gui` has shipped features that traced perfectly and did nothing.
//! `EditSession::sign` emits `sign-written path=… bytes=… field=Signature1
//! self_verified=1`, and every word of that line can be true of a build that
//! wrote a PDF with no signature in it — because the line is written by the
//! same code that would have written the signature, in the same process, from
//! the same beliefs.
//!
//! ⇒ So the verdict of this check is **phase D**, which launches a **fresh
//! process** on the file phase A wrote, opens the **Signatures panel** — the
//! verification side that shipped as `Pass 10.5`, a different subsystem written
//! months earlier for a different purpose — and requires:
//!
//! ```text
//! signature-row field="Signature1" covered=… pairs=1 well_formed=1 integrity=verified …
//! ```
//!
//! `integrity=verified` is `pdfcer_core::signature_verify` re-parsing the file
//! from disk, recomputing the digest over the byte ranges, and checking the CMS
//! against the embedded certificate. A build that wrote a plausible-looking
//! file would produce `digest-mismatch` or `unverifiable`, and a build that
//! wrote no signature at all would produce **no row at all**, which this check
//! also fails on.
//!
//! # The four phases, and what each is for
//!
//! | phase | document | what it proves |
//! |---|---|---|
//! | **A** | `four-pages.pdf` | ★ THE NEGATIVE CONTROL — a document that signs. Also: the gate on the confirm control opens only after the certificate is opened, which is the dynamic range the two refusals are measured against |
//! | **B** | `encrypted-aes-128.pdf` | the **encrypted** refusal, stated instead of a form |
//! | **C** | `four-pages.pdf` with a redaction armed | the **pending-redaction** refusal, stated instead of a form |
//! | **D** | phase A's output, fresh process | ★★★ THE VERDICT — the signature is in the file |
//!
//! ★★ **Phase A is not a formality and it is not there for coverage.** A probe
//! whose baseline has no dynamic range cannot produce a verdict: without a
//! document that signs, phases B and C would pass identically on a build where
//! `file.sign` opened a window that refused *everything*, or on one where the
//! confirm control was never drawn under any circumstances. Every absence
//! phases B and C assert is a presence phase A measured, in the same build,
//! over the same region names, minutes apart.
//!
//! # ★★★ THE CERTIFICATE: read from the engine's corpus, never committed here
//!
//! `D:\Dev\pdfcer\fixtures\synthetic\signing\rsa2048-modern.pfx`, with the
//! passphrase `pdfcer` that its own `PROVENANCE.md` publishes. That file is
//! **category (a) wholly synthetic key material, minted by a committed script
//! with OpenSSL**; its subject says *"(test fixture, trust nothing)"* in its own
//! `CN`; its validity is ~100 years, so this check acquires no expiry date.
//!
//! ⚠ **No certificate is copied into this repository, and none is generated
//! here.** A committed `.pfx` is either somebody's real identity, which must
//! never enter a git history, or a throwaway that expires and starts failing a
//! suite on a date nobody chose. The engine's corpus is READ-ONLY to this
//! project, which is exactly the relationship this needs: read it, write
//! nowhere near it.
//!
//! ★ Missing corpus is a hard error naming the path, not a SKIP. A SKIP reads
//! as *"this build does not have the feature"*, and this is a fact about the
//! checkout.
//!
//! # ⚠ The passphrase is TYPED, never passed in the environment
//!
//! `crate::app::files::pick_certificate` has a `PDFCER_DIAG_CERTIFICATE_PATH`
//! seam and there is deliberately **no** `…_PASSPHRASE` beside it: this harness
//! captures the child's stderr into an evidence directory it keeps, and
//! `crate::sign`'s §5 forbids a private key's passphrase reaching any file that
//! outlives the session. So phase A clicks into the field and types it, the way
//! an operator does — which is also the only way to prove the field works.

use std::path::{Path, PathBuf};

use super::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names,
    declared_or_in_overflow, list, shell_trace,
};
use super::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The mode phases A, B and D drive from.
///
/// **Read**, and it is itself an assertion. `catalog::file`'s registration says
/// in words why signing is reachable from a reading stance: *signing a drawing
/// before sending it out changes nothing on any page, so it is not authoring,
/// and an operator reading a document in Read mode is exactly the operator
/// about to email it to somebody.* Driving from Read is how that claim gets
/// checked rather than merely written.
const MODE: &str = "read";

/// The mode phase C drives from. Redaction **is** authoring.
const EDIT_MODE: &str = "edit";

/// The File tab.
const FILE_TAB: (&str, &str) = ("ribbon.tab.file", "file");
/// The Edit tab, for phase C's redaction.
const EDIT_TAB: (&str, &str) = ("ribbon.tab.edit", "edit");

/// The command under test.
const SIGN: &str = "file.sign";

// --- the Sign window's regions ---------------------------------------------

/// The whole window.
const REGION_DIALOG: &str = "sign-dialog";
/// A refusal, declared only while one is on screen.
const REGION_REFUSAL: &str = "sign-refusal";
/// The file-picker button.
const REGION_CHOOSE: &str = "sign-choose-certificate";
/// The passphrase field.
const REGION_PASSPHRASE: &str = "sign-passphrase";
/// The button that opens the chosen certificate.
const REGION_OPEN_CERT: &str = "sign-open-certificate";
/// The identity read-back, declared only once a certificate is open.
const REGION_IDENTITY: &str = "sign-identity";
/// The control that commits, declared only while it is live.
const REGION_CONFIRM: &str = "sign-confirm";
/// The control that opens what was just written.
const REGION_OPEN_SIGNED: &str = "sign-open-signed";

// --- trace events ----------------------------------------------------------

/// The window's own reading of the document, emitted before anything is drawn.
const OPENED_EVENT: &str = "sign-opened";
/// What the engine said it wrote.
const WRITTEN_EVENT: &str = "sign-written";
/// What came out of the container. Carries no subject, no serial and no path.
const IDENTITY_EVENT: &str = "sign-identity";
/// The Signatures panel's per-row line — phase D's oracle.
const ROW_EVENT: &str = "signature-row";

/// The certificate, in the engine's own corpus.
const CERT: &str = "signing/rsa2048-modern.pfx";
/// Its passphrase, published in that corpus's `PROVENANCE.md`.
const PASSPHRASE: &str = "pdfcer";

// --- phase C's redaction controls ------------------------------------------

/// Opens the Redact panel.
const REDACT_CMD: &str = "edit.redact";
/// Opens the apply window.
const REDACT_APPLY_CMD: &str = "edit.redact_apply";
/// The panel's *Mark whole page* control.
const REGION_WHOLE_PAGE: &str = "redact-whole-page";
/// The apply window's *into the open document* destination.
const REGION_INTO_DOCUMENT: &str = "redact-apply-destination-into-document";
/// The apply window's acknowledgement.
const REGION_REDACT_ACK: &str = "redact-apply-ack";
/// The apply window's confirm.
const REGION_REDACT_CONFIRM: &str = "redact-apply-confirm";

// --- phase B's document ----------------------------------------------------

/// ★★★ **An encrypted document that opens with NO password.**
///
/// `/V` 4, `/R` 4, `/AESV2`, empty user password — the §7.6.3.1 case a reader
/// must try silently. Read from the engine's corpus rather than from this
/// repository's `fixtures/encrypted-aes-128.pdf`, and the swap is a **harness
/// finding** rather than a preference, recorded here because it will bite the
/// next check that needs a protected fixture:
///
/// This check first drove `fixtures/encrypted-aes-128.pdf`, whose user password
/// is `userpw`. The password dialog appeared, the password was typed, the
/// document opened — `password-accepted` is in the trace — and the very next
/// click failed with *"GetClientRect failed for the target window"*.
/// `Session::launch` resolves its target with `find_window_for_pid` and accepts
/// the first window over `MIN_CLIENT_PX`; with a modal password dialog up at
/// start-up, **the dialog is the window it finds**. Once the dialog closed,
/// every subsequent click was aimed at a handle that no longer existed.
///
/// ⇒ A real limitation of the launcher for any password-protected fixture, and
/// **not this check's to fix.** What phase B is about is whether an ENCRYPTED
/// document is refused, and `crate::sign::Refusal::Encrypted` keys on
/// `/Encrypt` being present — equally true of a file that needed no password to
/// open. ★ A fixture that reaches the state under test **without a modal in the
/// way** is strictly better evidence: one fewer thing between the launch and
/// the measurement, and one fewer way for the check to fail about itself.
const ENCRYPTED: &str = "encryption/enc-emptyuser.pdf";

/// See the module documentation.
pub struct ADocumentCanBeSignedAndTheSignatureIsInTheFile;

impl Check for ADocumentCanBeSignedAndTheSignatureIsInTheFile {
    fn name(&self) -> &'static str {
        "a_document_can_be_signed_and_the_signature_is_in_the_file"
    }

    fn defect(&self) -> &'static str {
        "Signing produces no file, or produces one whose signature a fresh process cannot verify \
         — a feature that traces perfectly and does nothing; or the window offers a form over a \
         document the engine refuses outright (encrypted, or carrying an armed redaction), so the \
         operator meets the refusal by pressing rather than by reading; or the control that \
         attaches an identity to a legal document is live before any certificate has been opened"
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

// ---------------------------------------------------------------------------
// Driving
// ---------------------------------------------------------------------------

/// Launch one process, optionally with the certificate and save-path seams set.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &Path,
    trace_name: &str,
    env: &[(&str, PathBuf)],
) -> Result<Session> {
    let mut spec = LaunchSpec::new(
        ctx.resolve_exe().ok_or_else(|| {
            Error::new(format!(
                "no binary to drive. Pass --exe, or build the profile's default at {}.",
                ctx.profile.default_exe
            ))
        })?,
        ctx.out(trace_name),
    );
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    for (key, value) in env {
        spec.env
            .push(((*key).to_owned(), value.display().to_string()));
    }
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} on {} as pid {}",
        spec.exe.display(),
        pdf.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the process \
             and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    Ok(session)
}

/// Click a ribbon tab and confirm the shell reported it.
fn click_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    let before = shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(14);
    if shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count()
        <= before
    {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{TAB_EVENT} tab={id}` line."
        )));
    }
    Ok(())
}

/// **Click a ribbon tab without requiring that it CHANGED.**
///
/// [`click_tab`] asserts a new `ribbon-tab-activated` line, which is the right
/// test when a check is switching away from a tab it knows is active. It is the
/// wrong test for *"make sure this tab is on top"*: a tab that is already active
/// emits nothing when clicked, and the strict form then reports a perfectly good
/// click as a failure.
///
/// ★ A missing tab is still an error. This tolerates *no change*, never *no tab*.
fn click_tab_tolerant(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    region: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(14);
    Ok(())
}

/// **Find a ribbon control and press it**, through the overflow if it is there.
///
/// ★★★ Through [`declared_or_in_overflow`] rather than a bare rect lookup, and
/// for `checks::protect`'s reason word for word: at the harness's window width
/// the File band runs out of room, and `file.sign` is the **third** control in a
/// Security group that was already the last group added to a full band. A plain
/// `declared` would report *"the application declared no
/// `ribbon.item.file.sign` region"* — which would be true, and would be
/// reported as a missing feature when what is missing is a scroll.
fn press(session: &Session, driver: &Driver, ui_rect: &str, id: &str) -> Result<()> {
    let name = format!("{ITEM_PREFIX}{id}");
    let found = declared_or_in_overflow(session, driver, ui_rect, &name)?;
    let items = list(&declared_names(&session.trace()?, ui_rect, ITEM_PREFIX));
    let rect = found.ok_or_else(|| {
        Error::new(format!(
            "`{id}` is on no band, in no collapsed group's popup and behind no overflow button — \
             so an operator cannot reach it. ⚠ If this build was compiled WITHOUT the `signing` \
             feature that is the correct behaviour and this check should not have been run \
             against it. Ribbon items declared: {items}."
        ))
    })?;
    let before = invokes(session, id)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(24);
    if invokes(session, id)? <= before {
        return Err(Error::new(format!(
            "the click on `{id}` produced no new `{INVOKE_EVENT} id={id}` line, so the control was \
             found and did not fire. Every assertion below would then be measuring a window that \
             never opened."
        )));
    }
    Ok(())
}

/// How many times the shell has reported `id` invoked.
fn invokes(session: &Session, id: &str) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count())
}

/// Whether the application declared `name` at a usable rectangle.
///
/// ★ A degenerate rect counts as **absent**, not present. A region declared at
/// zero area is not something an operator can see.
fn drawn(trace: &Trace, ui_rect: &str, name: &str) -> bool {
    declared(trace, ui_rect, name).is_some_and(|r| r.is_substantial())
}

/// Click a region's centre, refusing when it was never drawn.
///
/// ★★★ Through [`driving::frame_of`] rather than `session.frame()`, and the
/// first driven run of this check is why. **A dialog is an OS window**
/// (`ui-conventions/dialogs.md` G1), so every region inside the Sign window is
/// declared in a CHILD viewport with its own origin; `session.frame()` is the
/// application window's, and clicking `declared_center` against it aims the
/// real pointer hundreds of points away — at whatever happens to be there.
///
/// The symptom was silence: the trace showed `certificate-picked chosen=1` and
/// then nothing at all, because the press on *Open certificate* landed outside
/// the button. ⇒ **Ask what the check AIMED AT**, which is the same finding
/// this project has recorded about the rotation buttons and about
/// `panning_past_the_overscan`, arriving a third way.
///
/// ★ `frame_of` is safe on a main-window region too — an untagged one answers
/// with `session.frame()`, unchanged — so there is no reason for a call site to
/// use the other form.
fn click(session: &Session, driver: &Driver, ui_rect: &str, name: &str) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "no `{name}` region to click. Regions declared under `sign-`: {}.",
            list(&declared_names(&trace, ui_rect, "sign-"))
        ))
    })?;
    let frame = driving::frame_of(session, &trace, ui_rect, name)?;
    driver.click_at(frame.declared_center(rect))?;
    session.settle(18);
    Ok(())
}

/// The last `sign-opened` line's `refusal=` token.
///
/// ★ A token the application spells as a `const fn`, never a `{:?}` of a domain
/// type — `dialogs::sign::refusal_token` says why at its definition, and the
/// reason is that Debug-formatting a value a check parses produced two false
/// failure reports on 2026-09-05.
fn last_refusal(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(OPENED_EVENT)
        .last()
        .and_then(|l| l.get("refusal").map(str::to_owned)))
}

/// Resolve a fixture from this repository.
fn repo_fixture(name: &str) -> Result<PathBuf> {
    // Resolved from this crate's manifest directory at COMPILE time, not from
    // `--source-root`, for the reason `checks::protect::repo_fixture` records:
    // `--source-root` is the staleness comparison's root and defaults to
    // `crates`, so joining `fixtures` onto it produced a path that does not
    // exist and a check that SKIPPED for ever while looking healthy.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join(name);
    if !path.is_file() {
        return Err(Error::new(format!(
            "the fixture {} is missing.",
            path.display()
        )));
    }
    Ok(path)
}

/// Resolve something from the engine repository's synthetic corpus.
///
/// ★ The path is derived, not configured. `D:\Dev\pdfcer` is READ-ONLY to this
/// project and its corpus is the only place these shapes exist, so the check
/// reads from it and writes nowhere near it — `checks::adopt_widget`'s
/// precedent, unchanged.
///
/// A missing corpus is a hard error naming the path, not a SKIP: a SKIP reads
/// as *"this build does not have the feature"*, and this is a fact about the
/// checkout rather than about the program.
fn engine_fixture(rel: &str, what: &str) -> Result<PathBuf> {
    let path = Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(rel);
    if !path.is_file() {
        return Err(Error::new(format!(
            "{what} is missing at {}. It lives in `pdfcer-core`'s own synthetic corpus, which this check READS — see this module's header for why nothing like it is committed into this repository.",
            path.display()
        )));
    }
    Ok(path)
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is roughly twenty clicks across four \
             processes. Reported as SKIPPED rather than passed: a check that did not run has \
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

    let plain = repo_fixture("four-pages.pdf")?;
    let encrypted = engine_fixture(
        ENCRYPTED,
        "the encrypted document phase B needs (AES-128, empty user password)",
    )?;
    let certificate = engine_fixture(
        CERT,
        "the signing certificate (category (a) synthetic key material, minted by `tools/gen-signing-fixtures.py`, subject \"(test fixture, trust nothing)\")",
    )?;
    let signed_out = ctx.out("signed-by-ui-verify.pdf");
    // A previous run's output must not be mistaken for this one's.
    let _ = std::fs::remove_file(&signed_out);

    let mut findings: Vec<String> = Vec::new();

    // =======================================================================
    // PHASE A — the negative control: a document that SIGNS.
    // =======================================================================
    {
        let session = launch(
            ctx,
            report,
            &plain,
            "sign-plain.trace.txt",
            &[
                ("PDFCER_DIAG_CERTIFICATE_PATH", certificate.clone()),
                ("PDFCER_DIAG_SAVE_PATH", signed_out.clone()),
            ],
        )?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        press(&session, &driver, ui_rect, SIGN)?;

        let trace = session.trace()?;
        if !drawn(&trace, ui_rect, REGION_DIALOG) {
            return Err(Error::new(format!(
                "PHASE A: `{SIGN}` fired and declared no `{REGION_DIALOG}` region, so no window \
                 opened and every assertion below would be about nothing. Regions declared under \
                 `sign-`: {}.",
                list(&declared_names(&trace, ui_rect, "sign-"))
            )));
        }
        if drawn(&trace, ui_rect, REGION_REFUSAL) {
            findings.push(format!(
                "PHASE A: `{REGION_REFUSAL}` was declared on `four-pages.pdf` — a plain, unsigned, \
                 saved document with nothing armed on it. The window refused a document it must \
                 offer a form for, which would make phases B and C pass vacuously."
            ));
        }
        match last_refusal(&session)? {
            Some(token) if token == "none" => {
                report.note(format!("phase A: {OPENED_EVENT} refusal={token}"));
            }
            other => findings.push(format!(
                "PHASE A: `{OPENED_EVENT} refusal=` was {other:?}; a clean document must read \
                 `none`."
            )),
        }

        // ★★★ THE GATE, before the certificate is opened. This is the
        // measurement that gives phases B and C their dynamic range: the
        // confirm control is declared only while it is live, so its absence
        // here is evidence the gate is shut rather than evidence a click
        // missed — and its PRESENCE four clicks later is what proves this
        // build declares it at all.
        if drawn(&trace, ui_rect, REGION_CONFIRM) {
            findings.push(format!(
                "PHASE A: `{REGION_CONFIRM}` was declared before any certificate had been opened. \
                 The one control in pdfcer that attaches somebody's legal identity to a document \
                 is live before they have seen whose identity it is."
            ));
        }

        // --- choose, type, open --------------------------------------------
        click(&session, &driver, ui_rect, REGION_CHOOSE)?;
        click(&session, &driver, ui_rect, REGION_PASSPHRASE)?;
        driver.type_ascii(PASSPHRASE)?;
        session.settle(10);
        click(&session, &driver, ui_rect, REGION_OPEN_CERT)?;

        let trace = session.trace()?;
        match trace.events(IDENTITY_EVENT).last() {
            Some(line) => {
                let key = line.get("key").unwrap_or_default().to_owned();
                let chain = line.get("chain").unwrap_or_default().to_owned();
                report.note(format!("phase A: {IDENTITY_EVENT} key={key} chain={chain}"));
                if !key.contains("RSA") {
                    findings.push(format!(
                        "PHASE A: the container reported key={key:?}; \
                         `{CERT}` holds an RSA-2048 key, so either the wrong file was opened or \
                         the importer read it wrong."
                    ));
                }
            }
            None => findings.push(format!(
                "PHASE A: no `{IDENTITY_EVENT}` line after clicking `{REGION_OPEN_CERT}` with the \
                 certificate chosen and the passphrase typed — the container was never opened, so \
                 either the picker seam did not answer, the field did not take the keystrokes, or \
                 the importer refused. Regions declared under `sign-`: {}.",
                list(&declared_names(&trace, ui_rect, "sign-"))
            )),
        }
        if !drawn(&trace, ui_rect, REGION_IDENTITY) {
            findings.push(format!(
                "PHASE A: no `{REGION_IDENTITY}` region after the certificate was opened. The \
                 operator is not shown whose certificate they are about to sign with, which is \
                 the one guard this window offers against the mistake that matters."
            ));
        }
        // ★★★ THE GATE OPENS. Paired with the assertion above it: together
        // they say the feature CONTROLS the button, where either alone says
        // only that it was drawn or was not.
        if !drawn(&trace, ui_rect, REGION_CONFIRM) {
            findings.push(format!(
                "PHASE A: `{REGION_CONFIRM}` is still not declared with an identity open and a \
                 destination chosen. The window cannot be completed, so nothing can ever be \
                 signed."
            ));
        }

        // --- sign -----------------------------------------------------------
        click(&session, &driver, ui_rect, REGION_CONFIRM)?;
        session.settle(60);
        let trace = session.trace()?;
        match trace.events(WRITTEN_EVENT).last() {
            Some(line) => {
                let field = line.get("field").unwrap_or_default().to_owned();
                let bytes = line.get("bytes").unwrap_or_default().to_owned();
                let verified = line.get("self_verified").unwrap_or_default().to_owned();
                report.note(format!(
                    "phase A: {WRITTEN_EVENT} field={field} bytes={bytes} self_verified={verified}"
                ));
                if verified != "1" {
                    findings.push(format!(
                        "PHASE A: `{WRITTEN_EVENT} self_verified={verified}`. The engine returns \
                         bytes only when it has re-read them and found the signature intact, so \
                         anything but 1 means this line was written by something other than the \
                         engine's own report."
                    ));
                }
            }
            None => findings.push(format!(
                "PHASE A: no `{WRITTEN_EVENT}` line after pressing the confirm control. Nothing \
                 was written, so phase D has no file to read."
            )),
        }
        if !drawn(&session.trace()?, ui_rect, REGION_OPEN_SIGNED) {
            findings.push(format!(
                "PHASE A: no `{REGION_OPEN_SIGNED}` region after a successful write. The engine's \
                 own instruction is that the session still holds the placeholder and a GUI must \
                 reload; without this control the operator is left with a document whose only \
                 honest description is \"this is not the file you signed\" and no way to reach \
                 the one that is."
            ));
        }
    }

    // ★ The file exists, before phase D tries to open it — so \"the signed
    // document has no signature\" and \"there is no signed document\" are two
    // different failure messages rather than one confusing one.
    if !signed_out.is_file() {
        findings.push(format!(
            "PHASE A: {} does not exist. Nothing reached the disk, so the verdict cannot be \
             taken.",
            signed_out.display()
        ));
        return Ok(Some(findings.join("\n\n")));
    }
    report.artifact(signed_out.clone());

    // =======================================================================
    // PHASE B — the ENCRYPTED refusal.
    // =======================================================================
    {
        let session = launch(
            ctx,
            report,
            &encrypted,
            "sign-encrypted.trace.txt",
            &[("PDFCER_DIAG_CERTIFICATE_PATH", certificate.clone())],
        )?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        press(&session, &driver, ui_rect, SIGN)?;
        let trace = session.trace()?;

        // The instrument still works: the window opened.
        if !drawn(&trace, ui_rect, REGION_DIALOG) {
            findings.push(format!(
                "PHASE B: `{SIGN}` on an encrypted document declared no `{REGION_DIALOG}` region. \
                 R9: the control is absent or EXPLAINED, and an explanation needs a window — a \
                 control that silently does nothing is the worse failure."
            ));
        }
        // ★★★ THE REFUSAL, by name.
        if !drawn(&trace, ui_rect, REGION_REFUSAL) {
            findings.push(format!(
                "PHASE B: no `{REGION_REFUSAL}` region on an encrypted document. Regions declared \
                 under `sign-`: {}.",
                list(&declared_names(&trace, ui_rect, "sign-"))
            ));
        }
        match last_refusal(&session)? {
            Some(token) if token == "encrypted" => {
                report.note(format!("phase B: {OPENED_EVENT} refusal={token}"));
            }
            other => findings.push(format!(
                "PHASE B: `{OPENED_EVENT} refusal=` was {other:?} on an encrypted document; it \
                 must be `encrypted`. A refusal for the wrong reason sends the operator to fix \
                 the wrong thing."
            )),
        }
        // ★★★ AND NO FORM. Every one of these is a presence phase A measured.
        for (name, what) in [
            (REGION_CHOOSE, "the certificate picker"),
            (REGION_PASSPHRASE, "the passphrase field"),
            (REGION_CONFIRM, "the control that writes a file"),
        ] {
            if drawn(&trace, ui_rect, name) {
                findings.push(format!(
                    "PHASE B: `{name}` was declared over an ENCRYPTED document — {what} is on \
                     screen for a document the engine refuses outright. R9: the control is absent \
                     or explained, never a form whose only possible outcome is a failure. Phase A \
                     proves this build declares `{name}` when it should, so this is the refusal \
                     branch drawing a body it must not."
                ));
            }
        }
    }

    // =======================================================================
    // PHASE C — the PENDING-REDACTION refusal.
    // =======================================================================
    {
        // ★★★ NOT `four-pages.pdf`, and the reason is a measurement rather
        // than a preference: a whole-page redaction on it is REFUSED by the
        // engine with `VerificationFailed { survivors: ["SCALE", "REVISION"] }`,
        // so the apply window opens on a refusal, offers no destination, and
        // nothing gets armed. A check that used it would have reported *"the
        // pending-redaction refusal does not work"* about a build whose refusal
        // is fine and whose fixture could not reach the state.
        //
        // ⇒ `checks::redaction`'s own fixture, borrowed rather than re-authored.
        // It is two pages of uncompressed Helvetica whose whole-page removal
        // verifies, and it is the document that check's verdict already rests
        // on — so if it ever stops having that property, two checks say so
        // instead of one silently drifting.
        let staged_source = ctx.out("sign-redaction-source.pdf");
        std::fs::write(&staged_source, super::redaction::fixture_bytes()).map_err(|e| {
            Error::new(format!(
                "could not write the phase C fixture to {}: {e}",
                staged_source.display()
            ))
        })?;
        let session = launch(
            ctx,
            report,
            &staged_source,
            "sign-redaction.trace.txt",
            &[("PDFCER_DIAG_CERTIFICATE_PATH", certificate.clone())],
        )?;
        let driver = Driver::new(session.window());
        // Redaction is authoring, so it is reached from Edit.
        driving::click_mode_segment(&session, &driver, ui_rect, EDIT_MODE)?;
        session.settle(16);
        click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
        press(&session, &driver, ui_rect, REDACT_CMD)?;
        session.settle(20);
        click(&session, &driver, ui_rect, REGION_WHOLE_PAGE)?;
        press(&session, &driver, ui_rect, REDACT_APPLY_CMD)?;
        session.settle(20);
        click(&session, &driver, ui_rect, REGION_INTO_DOCUMENT)?;
        click(&session, &driver, ui_rect, REGION_REDACT_ACK)?;
        click(&session, &driver, ui_rect, REGION_REDACT_CONFIRM)?;
        session.settle(40);

        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        press(&session, &driver, ui_rect, SIGN)?;
        let trace = session.trace()?;

        if !drawn(&trace, ui_rect, REGION_DIALOG) {
            findings.push(format!(
                "PHASE C: `{SIGN}` with a redaction armed declared no `{REGION_DIALOG}` region."
            ));
        }
        if !drawn(&trace, ui_rect, REGION_REFUSAL) {
            findings.push(format!(
                "PHASE C: no `{REGION_REFUSAL}` region with a redaction armed. Regions declared \
                 under `sign-`: {}.",
                list(&declared_names(&trace, ui_rect, "sign-"))
            ));
        }
        match last_refusal(&session)? {
            Some(token) if token == "redaction-pending" => {
                report.note(format!("phase C: {OPENED_EVENT} refusal={token}"));
            }
            other => findings.push(format!(
                "PHASE C: `{OPENED_EVENT} refusal=` was {other:?} with a deferred redaction \
                 armed; it must be `redaction-pending`. ⚠ If it is `none`, the arming did not \
                 happen and this phase measured a clean document — check the `redact-staged` \
                 line in the same trace before concluding the refusal is broken."
            )),
        }
        if drawn(&trace, ui_rect, REGION_CONFIRM) {
            findings.push(format!(
                "PHASE C: `{REGION_CONFIRM}` was declared with a redaction armed. Signing now \
                 would sign the version that still contains what the operator marked for removal."
            ));
        }
    }

    // =======================================================================
    // PHASE D — ★★★ THE VERDICT. A fresh process, reading the FILE.
    // =======================================================================
    {
        let session = launch(ctx, report, &signed_out, "sign-verify.trace.txt", &[])?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);

        // ★ The panel must be brought to the FRONT before it is read: a docked
        // pane that is not in front publishes nothing, which is
        // indistinguishable from a panel with nothing to say. That was called a
        // defect once and was not one.
        // ★★★ THE FALLBACK IS NOT OPTIONAL, and the first full re-run of this
        // check is why. `raise_dock_tab` succeeded on the earlier run purely
        // because a PREVIOUS launch had left the Signatures panel selected and
        // the shell had saved that layout — so the verdict was resting on
        // inherited state, which is a trap this project has already recorded
        // once ("one relaunched the binary and inherited the dock layout its
        // own previous launch had saved"). On a machine whose saved layout has
        // that panel behind another tab, phase D would have had no oracle and
        // would have SKIPPED — reporting nothing, in green.
        //
        // ⇒ Raise it; if it is not mounted, mount it from the ribbon and raise
        // it again. Three attempts at one fact, because the fact is the whole
        // verdict of this check.
        if !super::reaching::raise_dock_tab(&session, &driver, ui_rect, "view.panel_signatures")? {
            // ★★ TOLERANT, unlike `click_tab`: the View tab may already be the
            // active one, in which case a correct click emits **no new**
            // `ribbon-tab-activated` line and the strict form reports a click
            // that landed as one that did not. Ask what the check SAMPLED.
            click_tab_tolerant(&session, &driver, ui_rect, "ribbon.tab.view")?;
            press(&session, &driver, ui_rect, "view.panel_signatures")?;
            session.settle(24);
            let _ = super::reaching::raise_dock_tab(
                &session,
                &driver,
                ui_rect,
                "view.panel_signatures",
            )?;
        }
        session.settle(30);

        let trace = session.trace()?;
        let rows: Vec<_> = trace.events(ROW_EVENT).collect();
        if rows.is_empty() {
            findings.push(format!(
                "★★★ PHASE D — THE VERDICT: a fresh process opened {} and the Signatures panel \
                 reported NO `{ROW_EVENT}` line at all. The file was written, it is on disk, and \
                 it contains no signature that `pdfcer_core::signature` can find. That is the \
                 defect this whole check exists for: phase A's trace said `self_verified=1` and \
                 the file does not carry a signature. Regions declared: {}.",
                signed_out.display(),
                list(&declared_names(&trace, ui_rect, "panel:"))
            ));
        } else {
            let integrity = rows
                .last()
                .and_then(|l| l.get("integrity"))
                .unwrap_or_default()
                .to_owned();
            let field = rows
                .last()
                .and_then(|l| l.get("field"))
                .unwrap_or_default()
                .to_owned();
            report.note(format!(
                "★ phase D: {} row(s); last {ROW_EVENT} field={field} integrity={integrity}",
                rows.len()
            ));
            if integrity != "verified" {
                findings.push(format!(
                    "★★★ PHASE D — THE VERDICT: the signature in {} reads `integrity={integrity}` \
                     in a fresh process. `pdfcer_core::signature_verify` re-parsed the file from \
                     disk, recomputed the digest over the byte ranges and checked the CMS, and \
                     the answer is not `verified`. The file was signed and the signature does not \
                     hold.",
                    signed_out.display()
                ));
            }
        }
    }

    if findings.is_empty() {
        Ok(None)
    } else {
        Ok(Some(findings.join("\n\n")))
    }
}
