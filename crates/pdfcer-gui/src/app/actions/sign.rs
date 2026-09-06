//! # `app::actions::sign` — the one arm that signs a document
//!
//! [`Action::SignDocument`]'s body, split out of [`super::apply`] under **R2**
//! on the same seam [`super::saving`], [`super::redact`] and
//! [`super::destination`] already occupy: one subject, one file, with its
//! reasoning beside its mechanism rather than three screens away in a match.
//!
//! The window is [`crate::dialogs::sign`] and the model is [`crate::sign`];
//! read the second of those first, because everything about *what a signature
//! is and which refusals exist* is argued there. This file is about the four
//! things that have to happen, in order, on the far side of the action queue.
//!
//! ---
//!
//! # 1. ★★★ WHY THIS IS MATCHED BEFORE THE DOCUMENT GUARD — a BORROW reason
//!
//! [`super::apply`] takes `let Status::Open(doc) = &mut self.status` and keeps
//! that borrow for the rest of the function. This arm needs **two** of
//! `PdfcerApp`'s fields at once: the open document, to sign it, and
//! [`crate::dialogs::DialogsState`], to hand the outcome back to the window
//! that asked. Splitting the borrow has to happen while `self` is still whole.
//!
//! ★ That is exactly `Action::Find`'s argument, which is matched in the same
//! early block and says so at its arm. Two arms, one reason, and the reason is
//! about Rust rather than about signing.
//!
//! # 2. ★★★ THE FOUR STEPS, AND THE ORDER IS LOAD-BEARING
//!
//! `EditSession::sign` takes `&mut self`, so this is a funnel path and it owes
//! the funnel's protocol — [`super::apply::vector_edit`]'s four steps, which
//! this arm performs **by hand** rather than through that function. The reason
//! it cannot use it is stated below; the reason it must still do the same
//! things is that each step is a separate way to end up with an edit that is
//! silently declined.
//!
//! 1. **Stop the render worker.** `OpenDoc::session` is an `Arc` precisely so a
//!    worker can hold a clone while it rasterizes, and
//!    `RenderWorker::cancel_and_wait` is *"the choke point that makes
//!    `Arc<EditSession>` sound"*: `Arc::get_mut` fails while any other strong
//!    reference exists, so a signing attempted mid-render would simply be
//!    **refused**. Cancelling first is what turns *"sometimes refused,
//!    depending on how fast the page rasterized"* into *"always applied"*.
//! 2. **Reach the session through `Arc::get_mut`.** A `None` here is not a
//!    panic: it means something else still holds the session, which is a bug in
//!    the caller's ordering rather than in the operator's document. It is
//!    reported as a failure the window prints, because declining is
//!    recoverable and signing a document twice is not.
//! 3. **Sign, and write.** [`crate::sign::prepare`] then
//!    [`crate::sign::Prepared::write_to`], which is atomic.
//! 4. **Hand the outcome back.** The window is showing
//!    `Phase::Signing` until it hears, and that is its only way out.
//!
//! ## ★★ Why NOT `vector_edit`, when every other `&mut` verb uses it
//!
//! Because two of that funnel's four steps would be **wrong here**, and both
//! wrongs are silent:
//!
//! | `vector_edit` step | why not |
//! |---|---|
//! | bump `edit_epoch` | the epoch is what makes the canvas re-resolve its selection and rebuild its raster. Signing changes **nothing the canvas draws** — a visible signature's widget goes into the bytes that were written to disk, not into the session, which keeps only the zero-filled placeholder. Bumping it would re-rasterize a CAD sheet to draw an identical picture. |
//! | drop the cached texture | same fact, same cost. |
//!
//! ★★★ And the deeper reason, which is the one worth carrying: **the session
//! is left holding a placeholder, not a signature.** The engine says so —
//! *"the session still holds the staged placeholder objects (zeros in
//! `/Contents`) … a caller that wants to keep editing must re-open the returned
//! bytes."* So there is no state here for the canvas to catch up with. The
//! document on screen is, and remains, the version the operator started from,
//! and [`crate::text::sign::open_document_unchanged`] says so on the window
//! rather than leaving them to find out at the next `Ctrl+S`.
//!
//! ⚠ **Nothing is undone either.** The staged `CommandKind::AddSignatureField`
//! stays on the undo stack; the engine calls undoing it *"harmless and
//! pointless"*. Rewinding it here would look like tidying up and would put an
//! entry on the operator's undo stack for an act that produced a file.
//!
//! # 3. ★★★ THE IDENTITY IS LOADED AGAIN, HERE, AND THAT IS THE DESIGN
//!
//! The dialog has already opened this `.pfx` — that is how the operator saw
//! whose certificate it is — and it does **not** hand the loaded key over.
//! [`Action`] derives `Debug`, `Clone` and `PartialEq`, and every one of those
//! is wrong for a private key: `Debug` writes it into a trace `tools/ui-verify`
//! keeps on disk, `Clone` makes copies nobody counts, and `PartialEq` is a
//! **non-constant-time comparison over secret bytes**.
//!
//! So the action carries the path and a [`crate::secret::Secret`], and this
//! file opens the container a second time. The second read is not redundant:
//! it is the read that actually signs, so a file that changed under the
//! operator between the two is caught rather than assumed away.
//!
//! # 4. What is traced, and what is deliberately not
//!
//! `crate::sign`'s §5 binds here: **no line in this file carries the
//! passphrase, its length, or the certificate's path.** A trace is captured to
//! a file `tools/ui-verify` keeps as evidence, and a length is a search-space
//! reduction while a path is a durable pointer at where somebody keeps their
//! digital ID. What is traced is what a diagnosis needs — that a signing was
//! asked for, which step it reached, and what the engine said.

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::secret::Secret;
use crate::sign::{Authored, Identity, IdentityFailure, Outcome, PrepareFailure};
use crate::text::sign as t;
use std::path::{Path, PathBuf};

/// Whether `action` is the one this module handles.
///
/// ★ A predicate paired with [`apply`] over one variant, on
/// [`crate::app::dispatch::security::claims`]'s arrangement: a guard and a
/// handler that disagree turn a raised action into one that silently does
/// nothing, which is indistinguishable from the outside from an action nobody
/// wired.
#[must_use]
pub fn claims(action: &Action) -> bool {
    matches!(action, Action::SignDocument { .. })
}

/// **Sign the open document and write it where the operator chose.**
///
/// See §2 for the four steps and for why this arm does not go through
/// `vector_edit`. Every exit hands an [`Outcome`] to the window; there is no
/// path out of [`crate::dialogs::sign`]'s `Signing` phase but this one, so a
/// `return` that said nothing would leave the operator looking at a window that
/// never answers.
pub fn apply(app: &mut PdfcerApp, action: &Action) {
    let Action::SignDocument {
        certificate,
        passphrase,
        authored,
        target,
        replace,
    } = action
    else {
        return;
    };
    let outcome = run(app, certificate, passphrase, authored, target, *replace);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // ★ `written=` rather than the outcome Debug-formatted, because a
        // check parses this line and `{:?}` on a domain type is a spelling
        // nobody chose — which produced two false failure reports on
        // 2026-09-05. The failure's own sentence is on screen, where it
        // belongs; here it is one bit.
        format!(
            "sign-applied written={} replaced={}",
            u8::from(matches!(outcome, Outcome::Written { .. })),
            u8::from(*replace),
        )
    });
    app.dialogs.sign_outcome(outcome);
}

/// The body, returning the outcome rather than reporting it.
///
/// ★ Split from [`apply`] so that every exit is a `return` of a value the
/// compiler counts, rather than a `return` after a call somebody has to
/// remember to make. There are six ways out of this function and the window is
/// stuck until one of them is taken.
fn run(
    app: &mut PdfcerApp,
    certificate: &Path,
    passphrase: &Secret,
    authored: &Authored,
    target: &Path,
    replace: bool,
) -> Outcome {
    // --- 3. the identity, opened again -----------------------------------
    //
    // Before the worker is cancelled, deliberately: a wrong passphrase or a
    // certificate that has moved should not cost the operator a re-raster of
    // the page they are looking at.
    let identity = match Identity::open(certificate, passphrase) {
        Ok(identity) => identity,
        Err(IdentityFailure::Unreadable(detail)) => {
            return Outcome::Failed(t::identity_unreadable(&detail));
        }
        Err(IdentityFailure::Import(error)) => {
            return Outcome::Failed(t::identity_refused(&error.to_string()));
        }
    };

    let Status::Open(doc) = &mut app.status else {
        // Unreachable from the window, which cannot exist without a document.
        // Answered rather than asserted, on this project's standing preference
        // against panicking on a branch a guard has already excluded.
        return Outcome::Failed(t::refusal_not_on_disk().to_owned());
    };
    // --- 1. stop the render worker ---------------------------------------
    //
    // Before `Arc::get_mut`, always. See §2. It lives on the document rather
    // than on the app — one worker per open document — which is why it is
    // reached after the guard rather than before it.
    doc.render_worker.cancel_and_wait();

    let options = crate::app::settings::SettingsExt::save_options(&doc.settings);
    let pages = doc.pages.clone();

    // --- 2. reach the session --------------------------------------------
    let Some(session) = std::sync::Arc::get_mut(&mut doc.session) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            "sign-declined reason=session-held".to_owned()
        });
        return Outcome::Failed(t::engine_refused(
            // ui-text-exempt: this IS the operator sentence, and it is built
            // here rather than in `text::sign` for one reason: it describes a
            // state that cannot be reached except by a bug in this file's own
            // ordering, so a catalogue entry for it would be a permanent
            // invitation to make the state reachable. See `crate::text`'s rule
            // on strings that describe a programming error.
            "another part of pdfcer is still using this document. Try again.",
        ));
    };

    // --- 3. sign ----------------------------------------------------------
    let prepared = match crate::sign::prepare(session, &pages, &identity, authored, &options) {
        Ok(prepared) => prepared,
        Err(PrepareFailure::Refused(refusal)) => {
            return Outcome::Failed(t::refusal_line(refusal));
        }
        Err(PrepareFailure::Engine(error)) => {
            // ★ The reservation gets its own sentence, and it is the one engine
            // refusal whose own advice this shell cannot follow: its message
            // ends "sign again with a larger reserve" and there is no control
            // here that sets one. See `crate::sign::prepare`'s note.
            let detail = error.to_string();
            return Outcome::Failed(
                if matches!(
                    error,
                    pdfcer_core::sign::apply::SignApplyError::ReservationTooSmall { .. }
                ) {
                    t::reservation_too_small(&detail)
                } else {
                    t::engine_refused(&detail)
                },
            );
        }
    };

    // --- 4. write, and say exactly what was written -----------------------
    let report = prepared.report();
    let details = t::written_details(
        &report.field_name,
        &report.signer_subject,
        &report.signer_serial_hex,
    );
    match prepared.write_to(target) {
        Ok(_) => Outcome::Written {
            path: PathBuf::from(target),
            replaced: replace,
            details,
        },
        Err(failure) => Outcome::Failed(t::write_failed(&failure.to_string())),
    }
}
