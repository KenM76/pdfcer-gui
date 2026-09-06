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
//! # The eight phases, and what each is for
//!
//! | phase | document | what it proves |
//! |---|---|---|
//! | **A** | `four-pages.pdf` | ★ THE NEGATIVE CONTROL — a document that signs. Also: the gate on the confirm control opens only after the certificate is opened, which is the dynamic range the two refusals are measured against |
//! | **B** | `encrypted-aes-128.pdf` | the **encrypted** refusal, stated instead of a form |
//! | **C** | `four-pages.pdf` with a redaction armed | the **pending-redaction** refusal, stated instead of a form |
//! | **D** | phase A's output, fresh process | ★★★ THE VERDICT — the signature is in the file |
//! | **E** | `sig-field-empty.pdf` | `Pass 10.13` — a box the SENDER placed is listed, chosen, and the placement controls RETIRE |
//! | **F** | phase E's output, fresh process | ★★★ THE SECOND VERDICT — the signature went INTO that box |
//! | **G** | `four-pages.pdf`, certifying | `Pass 10.12` — the operator can sign as the document's AUTHOR |
//! | **H** | phase G's output, fresh process | ★★★ THE THIRD VERDICT — the `/DocMDP` is in the file |
//!
//! # ★★★ PHASE H'S ORACLE IS THE DOCUMENT CENSUS, WHICH IS NOT THE SIGNING CODE
//!
//! There is no signature-panel row that reports a certification, so phase F's
//! trick — read the name back through the verification side — has no equivalent
//! here. What does exist is `EditSession::signature_census`, which parses
//! `/Reference … /TransformMethod /DocMDP` and the catalog's `/Perms` out of the
//! bytes on disk. It shipped months before the signing verb, for a different
//! purpose (deciding whether a save would break somebody else's signature), and
//! this shell reads it **when the Sign window opens**.
//!
//! ⇒ So phase H launches a fresh process on the certified file and presses
//! `Sign…` again. `sign-opened certification=2` is the census finding a
//! `/DocMDP` transform where the document had none — asked of a subsystem that
//! knows nothing about how the file was produced.
//!
//! # ★★★ WHY PHASE F IS A SEPARATE VERDICT AND NOT A REPEAT OF PHASE D
//!
//! Phase D asks *"is there a signature in the file?"* Phase F asks a question
//! phase D cannot distinguish: **which box did it go into?**
//!
//! Signing into a pre-placed field and signing beside one produce outcomes that
//! are identical in every respect this check could otherwise measure — a file
//! exists, the bytes grew, `self_verified=1`, the Signatures panel shows one
//! row, `integrity=verified`. A build that quietly ignored `field_name` and
//! created its own field would pass every assertion in phase D.
//!
//! ⇒ The discriminator is **the field's name**. A signature written into the
//! author's box carries the author's own `/T` — `SignHere` on this fixture — and
//! one written into a field pdfcer invented carries `Signature1`, Acrobat's
//! convention. That name is read back **in a fresh process, by the verification
//! side**, from `signature-row field=…`, so it is not the signing code's account
//! of its own behaviour.
//!
//! ★★ And the same phase measures the thing that has no in-process oracle at
//! all: `sign-written field_reused=1`. That line is written by the same beliefs
//! as the signing, so it is reported as a **note**, never as the verdict — the
//! verdict is the name, read by somebody else.
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

use super::driving::{self, declared_names, list};
use super::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::Session;
use crate::report::CheckReport;

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
/// The scrolling body's own viewport - what a control must be inside before it
/// can be clicked. See [`click_scrolled`].
const REGION_BODY: &str = "sign-body";
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
/// The radio that chooses *sign into a box already on the document*, declared
/// only while the document has one to offer.
const REGION_EXISTING: &str = "sign-existing-field";
/// The first pre-placed field's row.
const REGION_FIELD_0: &str = "sign-field-0";
/// The radio that chooses *draw a signature box on the page*.
const REGION_PLACE_BOX: &str = "sign-place-box";
/// The radio that makes this a certifying signature, declared only while the
/// document permits one.
const REGION_CERTIFY: &str = "sign-certify";
/// The page chooser, drawn only for a box this shell places on a document of
/// more than one page.
const REGION_PAGE: &str = "sign-page";
/// The line saying where a box THIS SHELL places will go — declared for
/// `Place::Box` and nothing else. Phase E's retirement probe; see that phase.
const REGION_BOX_WHERE: &str = "sign-box-where";

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

// --- phases E and F's document ---------------------------------------------

/// ★★★ **A page carrying a PRE-PLACED, EMPTY signature field** — the *"sign
/// here"* box a form author puts on a drawing before mailing it out.
///
/// `/FT /Sig /T (SignHere) /Rect [72 600 300 660] /P <page>`, a merged widget,
/// **no `/V`**, no `/Lock`, no `/SV` — and a text field `Name` beside it, so a
/// build that offered every field rather than only the signature fields would
/// be caught by the count rather than by inspection.
///
/// Read from the engine's own corpus (`tools/gen-sig-field-fixtures.py`, which
/// is committed there and is deterministic — no clock, no randomness) for
/// [`CERT`]'s reason, applied to a document instead of a key: it is the corpus
/// the engine's own `Pass 10.13` tests run against, so the shape this check
/// drives and the shape the engine was built for cannot drift apart. **Nothing
/// is written anywhere near it.**
const FIELD_DOC: &str = "signing/sig-field-empty.pdf";

/// ★★ The field's name, and it is **the oracle of phase F.**
///
/// The one fact that distinguishes *"pdfcer signed the box the sender placed"*
/// from *"pdfcer made a new box beside it"*. Both produce a signed file, both
/// self-verify, both trace a plausible `sign-written`; the second names its
/// field `Signature1`, Acrobat's convention for a field pdfcer invented. Read
/// back in a **fresh process** by the Signatures panel, so the claim is not
/// checked by the code that made it.
const FIELD_NAME: &str = "SignHere";

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

mod reaching;

use reaching::{
    click, click_scrolled, click_tab, drawn, engine_fixture, field_name_of, last_refusal, launch,
    press, raise_signatures, repo_fixture,
};

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
        // defect once and was not one. `raise_signatures` carries the rest of
        // the reasoning, including why the fallback is not optional.
        raise_signatures(&session, &driver, ui_rect)?;

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
            // Through `field_name_of` for the reason that function records:
            // the panel Debug-formats an `Option<String>`, so the raw value
            // reads `Some("Signature1")` and this note had been printing the
            // wrapper and the quotes as though they were the name.
            let field = field_name_of(rows.last().and_then(|l| l.get("field")).unwrap_or_default());
            // ⚠ "row(s)" is FRAMES, not signatures: the panel publishes one
            // line per signature per frame it draws. Said in the note rather
            // than corrected here, because phase D's verdict does not rest on
            // the count; phase F's does, and it counts distinct names instead.
            report.note(format!(
                "★ phase D: {} row line(s) (one per signature per frame); last {ROW_EVENT} field={field} integrity={integrity}",
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

    // =======================================================================
    // PHASE E — ★★★ `Pass 10.13`: THE BOX THE SENDER PLACED.
    // =======================================================================
    let field_doc = engine_fixture(
        FIELD_DOC,
        "the document phase E needs (a pre-placed, EMPTY /FT /Sig field). Regenerate the engine's corpus with `python tools/gen-sig-field-fixtures.py` in D:\\Dev\\pdfcer",
    )?;
    let field_out = ctx.out("signed-into-field-by-ui-verify.pdf");
    let _ = std::fs::remove_file(&field_out);
    {
        let session = launch(
            ctx,
            report,
            &field_doc,
            "sign-into-field.trace.txt",
            &[
                ("PDFCER_DIAG_CERTIFICATE_PATH", certificate.clone()),
                ("PDFCER_DIAG_SAVE_PATH", field_out.clone()),
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
                "PHASE E: `{SIGN}` fired on {} and declared no `{REGION_DIALOG}` region.",
                field_doc.display()
            )));
        }
        // ★★ The COUNT, before anything is clicked. Two numbers rather than
        // one: `empty_fields` is what the document holds and `signable_fields`
        // is what this build will offer, and a build that listed the text field
        // `Name` beside the signature field would read 2 here.
        match session.trace()?.events(OPENED_EVENT).last() {
            Some(line) => {
                let held = line.get("empty_fields").unwrap_or_default().to_owned();
                let usable = line.get("signable_fields").unwrap_or_default().to_owned();
                report.note(format!(
                    "phase E: {OPENED_EVENT} empty_fields={held} signable_fields={usable}"
                ));
                if held != "1" || usable != "1" {
                    findings.push(format!(
                        "PHASE E: the window read `empty_fields={held} signable_fields={usable}` \
                         on a document carrying exactly ONE empty signature field (and one text \
                         field beside it, which must not be counted). `0` means the pre-placed \
                         box the whole feature exists for was never found; anything above 1 means \
                         fields that are not signature fields — or fields that are already \
                         signed — are being offered as places to put a signature."
                    ));
                }
            }
            None => findings.push(format!(
                "PHASE E: no `{OPENED_EVENT}` line at all after `{SIGN}` fired."
            )),
        }

        // --- open the identity, so the rest of the form exists --------------
        click(&session, &driver, ui_rect, REGION_CHOOSE)?;
        click(&session, &driver, ui_rect, REGION_PASSPHRASE)?;
        driver.type_ascii(PASSPHRASE)?;
        session.settle(10);
        click(&session, &driver, ui_rect, REGION_OPEN_CERT)?;
        session.settle(16);

        if !drawn(&session.trace()?, ui_rect, REGION_EXISTING) {
            findings.push(format!(
                "PHASE E: no `{REGION_EXISTING}` region on a document that HAS an empty signature \
                 field. The operator is offered no way to sign in the box the sender placed, \
                 which is the ordinary case for a drawing sent out for approval. Regions declared \
                 under `sign-`: {}.",
                list(&declared_names(&session.trace()?, ui_rect, "sign-"))
            ));
        }

        // ★★★ THE DYNAMIC RANGE FOR THE RETIREMENT ASSERTION, taken FIRST.
        // `--visible`/`--page` are refused by the engine alongside a field
        // name, so those controls must retire when a box is picked — and
        // "retired" is only a claim if this build draws them at all. So: select
        // *draw a box*, measure `sign-box-where`; then select the field and
        // measure it again. Without the first measurement, a build that never
        // drew the control would pass the second one perfectly.
        click_scrolled(&session, &driver, ui_rect, REGION_PLACE_BOX, report)?;
        let with_box = drawn(&session.trace()?, ui_rect, REGION_BOX_WHERE);
        if !with_box {
            findings.push(format!(
                "PHASE E: `{REGION_BOX_WHERE}` was not declared with *draw a box on the page* \
                 selected, so this build never draws the control whose retirement the next \
                 assertion is about. Everything below would pass vacuously."
            ));
        }

        // --- choose the sender's box ---------------------------------------
        click_scrolled(&session, &driver, ui_rect, REGION_EXISTING, report)?;
        session.settle(14);
        let trace = session.trace()?;
        if !drawn(&trace, ui_rect, REGION_FIELD_0) {
            findings.push(format!(
                "PHASE E: `{REGION_EXISTING}` was chosen and no `{REGION_FIELD_0}` row appeared, \
                 so the document's own signature box is not listed and cannot be picked. Regions \
                 declared under `sign-`: {}.",
                list(&declared_names(&trace, ui_rect, "sign-"))
            ));
        }
        click_scrolled(&session, &driver, ui_rect, REGION_FIELD_0, report)?;
        session.settle(14);

        // ★★★ THE RETIREMENT. Paired with the measurement above: together they
        // say the choice CONTROLS the control, where either alone says only
        // that it was drawn or was not.
        let trace = session.trace()?;
        if with_box && drawn(&trace, ui_rect, REGION_BOX_WHERE) {
            findings.push(format!(
                "PHASE E: `{REGION_BOX_WHERE}` is STILL declared with the sender's box chosen. \
                 The page and position controls have not retired, so the window is offering the \
                 operator a placement the engine refuses by name alongside a field name \
                 (`RectRefusedForExistingField`) — a control whose only possible outcome is a \
                 refusal."
            ));
        }
        if drawn(&trace, ui_rect, REGION_PAGE) {
            findings.push(format!(
                "PHASE E: `{REGION_PAGE}` is declared with the sender's box chosen. The field's \
                 own /Rect and page place the signature; a page chooser beside it is a control \
                 that cannot do anything."
            ));
        }

        // --- sign ------------------------------------------------------------
        if !drawn(&trace, ui_rect, REGION_CONFIRM) {
            return Err(Error::new(format!(
                "PHASE E: `{REGION_CONFIRM}` is not declared with an identity open and a box \
                 chosen, so nothing can be signed and phase F has no file to read."
            )));
        }
        click(&session, &driver, ui_rect, REGION_CONFIRM)?;
        session.settle(60);
        match session.trace()?.events(WRITTEN_EVENT).last() {
            Some(line) => {
                let reused = line.get("field_reused").unwrap_or_default().to_owned();
                let field = line.get("field").unwrap_or_default().to_owned();
                report.note(format!(
                    "phase E: {WRITTEN_EVENT} field={field} field_reused={reused}"
                ));
                // ⚠ A NOTE that is also asserted, and the asymmetry is
                // deliberate: this line is written by the same beliefs as the
                // signing, so on its own it proves nothing — the verdict is
                // phase F. It is still checked, because a `0` here beside a
                // correct name in phase F would mean the two disagree, which is
                // worth knowing.
                if reused != "1" {
                    findings.push(format!(
                        "PHASE E: `{WRITTEN_EVENT} field_reused={reused}` after choosing the \
                         document's own signature box. The engine reports it created a field \
                         rather than signing into the one that was there."
                    ));
                }
            }
            None => findings.push(format!(
                "PHASE E: no `{WRITTEN_EVENT}` line after pressing the confirm control."
            )),
        }
    }

    if !field_out.is_file() {
        findings.push(format!(
            "PHASE E: {} does not exist, so the second verdict cannot be taken.",
            field_out.display()
        ));
        return Ok(Some(findings.join("\n\n")));
    }
    report.artifact(field_out.clone());

    // =======================================================================
    // PHASE F — ★★★ THE SECOND VERDICT. A fresh process, reading the NAME.
    // =======================================================================
    {
        let session = launch(ctx, report, &field_out, "sign-field-verify.trace.txt", &[])?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);
        raise_signatures(&session, &driver, ui_rect)?;

        let trace = session.trace()?;
        let rows: Vec<_> = trace.events(ROW_EVENT).collect();
        if rows.is_empty() {
            findings.push(format!(
                "★★★ PHASE F — THE SECOND VERDICT: a fresh process opened {} and the Signatures \
                 panel reported NO `{ROW_EVENT}` line. The file was written and carries no \
                 signature `pdfcer_core::signature` can find.",
                field_out.display()
            ));
        } else {
            let last = rows.last();
            let integrity = last
                .and_then(|l| l.get("integrity"))
                .unwrap_or_default()
                .to_owned();
            // ⚠ Through `field_name_of`, because the panel Debug-formats an
            // `Option<String>` and the raw value is `Some("SignHere")`. See
            // that function.
            let field = field_name_of(last.and_then(|l| l.get("field")).unwrap_or_default());
            // ⚠ "row line(s)", not signatures — see phase D's note and the
            // distinct-name count below.
            report.note(format!(
                "★ phase F: {} row line(s); last {ROW_EVENT} field={field:?} integrity={integrity}",
                rows.len()
            ));
            if integrity != "verified" {
                findings.push(format!(
                    "★★★ PHASE F — THE SECOND VERDICT: the signature in {} reads \
                     `integrity={integrity}` in a fresh process.",
                    field_out.display()
                ));
            }
            // ★★★ THE DISCRIMINATOR.
            if field != FIELD_NAME {
                findings.push(format!(
                    "★★★ PHASE F — THE SECOND VERDICT: a fresh process read the signature back \
                     under the field name {field:?}, and the box the document's author placed is \
                     called {FIELD_NAME:?}. The signature did not go into the sender's box — it \
                     went into a field pdfcer created beside it, which is precisely the outcome \
                     `Pass 10.13` exists to replace. ⚠ A name of \"Signature1\" is pdfcer's own \
                     convention for a field it invented, so that value means `field_name` reached \
                     the engine as `None` or was ignored."
                ));
            }
            // ★★★ DISTINCT NAMES, NOT ROWS — and the first run of this phase is
            // why. `signature-row` is published by the Signatures panel **when
            // it draws**, which is once per frame per signature, so phase D's
            // own note has been reading "37 row(s)" for one signature since the
            // day it was written. Counting lines counts FRAMES.
            //
            // ⇒ Ask what the check SAMPLED. This project has recorded the same
            // finding about `ui-rect` standing in for "the application drew a
            // frame"; it is the identical mistake with the identical shape, and
            // an assertion of `rows.len() == 1` here would have failed on a
            // perfectly correct build.
            let names: std::collections::BTreeSet<String> = rows
                .iter()
                .filter_map(|l| l.get("field"))
                .map(field_name_of)
                .collect();
            if names.len() != 1 {
                findings.push(format!(
                    "PHASE F: the signed document carries {} distinct signature field(s) — {}. \
                     Signing into a pre-placed field must append nothing to /Annots or /Fields, \
                     so anything but the author's own single box means a second field was created \
                     as well as theirs being filled.",
                    names.len(),
                    list(&names.iter().cloned().collect::<Vec<_>>())
                ));
            }
        }
    }

    // =======================================================================
    // PHASE G — ★★★ `Pass 10.12`: CERTIFYING, as the document's author.
    // =======================================================================
    let certified_out = ctx.out("certified-by-ui-verify.pdf");
    let _ = std::fs::remove_file(&certified_out);
    {
        let session = launch(
            ctx,
            report,
            &plain,
            "sign-certify.trace.txt",
            &[
                ("PDFCER_DIAG_CERTIFICATE_PATH", certificate.clone()),
                ("PDFCER_DIAG_SAVE_PATH", certified_out.clone()),
            ],
        )?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        press(&session, &driver, ui_rect, SIGN)?;

        // ★ THE DYNAMIC RANGE for phase H's absence assertion, taken on a
        // document that has never been signed. Without it, phase H's
        // `may_certify=0` would pass identically on a build that never offers
        // the option at all.
        match last_may_certify(&session)? {
            Some(token) if token == "1" => {
                report.note("phase G: sign-opened may_certify=1 on an unsigned document");
            }
            other => findings.push(format!(
                "PHASE G: `{OPENED_EVENT} may_certify=` was {other:?} on a clean, unsigned \
                 document. Certifying is refused before it is offered, so phase H would measure \
                 an option this build never draws."
            )),
        }

        click(&session, &driver, ui_rect, REGION_CHOOSE)?;
        click(&session, &driver, ui_rect, REGION_PASSPHRASE)?;
        driver.type_ascii(PASSPHRASE)?;
        session.settle(10);
        click(&session, &driver, ui_rect, REGION_OPEN_CERT)?;
        session.settle(16);

        if !drawn(&session.trace()?, ui_rect, REGION_CERTIFY) {
            findings.push(format!(
                "PHASE G: no `{REGION_CERTIFY}` region on a clean document. An operator cannot \
                 sign as the document's author, which is the whole of `Pass 10.12`. Regions \
                 declared under `sign-`: {}.",
                list(&declared_names(&session.trace()?, ui_rect, "sign-"))
            ));
        }
        click_scrolled(&session, &driver, ui_rect, REGION_CERTIFY, report)?;
        // ★ Plain `click`, NOT `click_scrolled` — and the first run of this
        // phase is why. The confirm control lives in the window's FOOTER,
        // below the scroll area, so it is never inside `sign-body` and
        // `click_scrolled` waited eight notches for a containment that
        // cannot happen. ⇒ The two helpers are not interchangeable: one is
        // for the form, the other for the furniture around it.
        click(&session, &driver, ui_rect, REGION_CONFIRM)?;
        session.settle(60);
        match session.trace()?.events(WRITTEN_EVENT).last() {
            Some(line) => {
                let certified = line.get("certified").unwrap_or_default().to_owned();
                report.note(format!("phase G: {WRITTEN_EVENT} certified={certified}"));
                // A NOTE that is also asserted; the verdict is phase H. `2` is
                // Table 254's own default and this window's, so anything else
                // means the level travelled wrong rather than not at all.
                if certified != "2" {
                    findings.push(format!(
                        "PHASE G: `{WRITTEN_EVENT} certified={certified}` after choosing to sign \
                         as the document's author. `none` means the choice never reached the \
                         request; any other number means a /DocMDP level the operator did not \
                         pick, which decides what anybody may do to his document afterwards."
                    ));
                }
            }
            None => findings.push(format!(
                "PHASE G: no `{WRITTEN_EVENT}` line after pressing the confirm control."
            )),
        }
    }

    if !certified_out.is_file() {
        findings.push(format!(
            "PHASE G: {} does not exist, so the third verdict cannot be taken.",
            certified_out.display()
        ));
        return Ok(Some(findings.join("\n\n")));
    }
    report.artifact(certified_out.clone());

    // =======================================================================
    // PHASE H — ★★★ THE THIRD VERDICT. A fresh process, reading /Perms.
    // =======================================================================
    //
    // ★★★ THE ORACLE IS THE DOCUMENT CENSUS, WHICH IS NOT THE SIGNING CODE.
    //
    // `EditSession::signature_census` parses `/Reference … /TransformMethod
    // /DocMDP` and the catalog's `/Perms` out of the bytes on disk. It shipped
    // months before the signing verb, for a different purpose — deciding
    // whether a SAVE would break somebody else's signature — and this shell
    // reads it when the Sign window opens, which is how a second launch on the
    // written file can report what the first one wrote **without asking the
    // code that wrote it**.
    //
    // ⇒ So the verdict is two numbers from `sign-opened`: `certification=2`,
    // which is the census finding a /DocMDP transform where there was none, and
    // `may_certify=0`, which is this shell refusing a SECOND certification —
    // §12.8.2.2.1's "a document can contain only one". A build that wrote the
    // `certify` flag into a trace and no /DocMDP into the file produces
    // `certification=none` here.
    {
        let session = launch(
            ctx,
            report,
            &certified_out,
            "sign-certify-verify.trace.txt",
            &[],
        )?;
        let driver = Driver::new(session.window());
        driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
        session.settle(16);
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        press(&session, &driver, ui_rect, SIGN)?;
        session.settle(20);

        match session.trace()?.events(OPENED_EVENT).last() {
            Some(line) => {
                let certification = line.get("certification").unwrap_or_default().to_owned();
                let may = line.get("may_certify").unwrap_or_default().to_owned();
                report.note(format!(
                    "★ phase H: {OPENED_EVENT} certification={certification} may_certify={may}"
                ));
                if certification != "2" {
                    findings.push(format!(
                        "★★★ PHASE H — THE THIRD VERDICT: a fresh process read \
                         `certification={certification}` out of {}. The document census — a \
                         different subsystem, parsing /Reference and the catalog /Perms from the \
                         bytes on disk — finds no /DocMDP transform at permission 2. The \
                         certification was reported and not written.",
                        certified_out.display()
                    ));
                }
                // ⚠ **WEAKER THAN IT LOOKS, and the falsification run proved
                // it.** With the certification planted out, this assertion still
                // held — because the written file carries a SIGNATURE either
                // way, so `CertifyBar::NotFirst` closes the option even when no
                // /DocMDP was written. It cannot distinguish "already certified"
                // from "already signed", and it is kept as a guard against the
                // option being offered on a document that would refuse it, NOT
                // as evidence that certifying worked. **The verdict is
                // `certification=2` above**, which the plant did move.
                if may != "0" {
                    findings.push(format!(
                        "PHASE H: `may_certify={may}` on a document that is already certified. \
                         §12.8.2.2.1 permits ONE certification per document, so this build would \
                         offer the operator a second one and let the engine refuse it after he had \
                         filled in the form and chosen a destination."
                    ));
                }
            }
            None => findings.push(format!(
                "PHASE H: no `{OPENED_EVENT}` line after `{SIGN}` on the certified document."
            )),
        }
    }

    if findings.is_empty() {
        Ok(None)
    } else {
        Ok(Some(findings.join("\n\n")))
    }
}

/// The last `sign-opened` line's `may_certify=` bit.
fn last_may_certify(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(OPENED_EVENT)
        .last()
        .and_then(|l| l.get("may_certify").map(str::to_owned)))
}
