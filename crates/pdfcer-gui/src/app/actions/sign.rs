//! # `app::actions::sign` — the one arm that signs a document
//!
//! [`Action::SignDocument`]'s body, kept beside its reasoning rather than
//! inside [`super::apply`]'s match.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/sign.md`.

use pdfcer_core::sign::apply::MdpPermission;

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::secret::Secret;
use crate::sign::{Authored, Identity, IdentityFailure, Outcome, PrepareFailure};
use crate::text::sign as t;
use std::path::{Path, PathBuf};

/// Whether `action` is the one this module handles.
///
/// The predicate half of the guard/handler pair
/// [`crate::app::dispatch::security::claims`] uses: a guard and a handler that
/// disagree turn a raised action into one that silently does nothing, which is
/// indistinguishable from the outside from an action nobody wired.
///
/// [`super::apply`] currently re-matches the variant itself rather than calling
/// this, so the two can drift; a caller that routes on the predicate should use
/// this one rather than spelling the pattern a second time.
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
        // `written=` rather than the outcome Debug-formatted, because a
        // check parses this line and `{:?}` on a domain type is a spelling
        // nobody chose: it changes whenever a variant's fields change, and
        // the check then reports a failure that did not happen. The failure's
        // own sentence is on screen, where it belongs; here it is one bit.
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
/// Separate from [`apply`] so that every exit is a `return` of a value the
/// compiler counts, rather than a `return` after a call somebody has to
/// remember to make. Every early exit here leaves the window stuck in
/// `Phase::Signing` until its outcome is handed back, and the compiler is what
/// guarantees one exists.
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
        Err(PrepareFailure::Engine(error)) => return Outcome::Failed(worded(&error)),
    };

    // --- 4. write, and say exactly what was written -----------------------
    let report = prepared.report();
    let details = t::written_details(&t::Written {
        field: &report.field_name,
        subject: &report.signer_subject,
        serial: &report.signer_serial_hex,
        reused: report.field_reused,
        lock: report.field_lock.as_deref(),
        certification: report.certification.map(MdpPermission::meaning),
        notes: &report.notes,
        // The one part of the disclosure whose subject is not the file but the
        // PAGE: what the box will read when somebody opens the signed
        // document. Empty for an invisible signature.
        appearance: &report.appearance_lines,
    });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        //
        // `shown` is counted against the SENTENCE, not against the report:
        // reading the slice's length twice would be satisfied by a call site
        // that handed the composer nothing. See `text::sign::appearance_shown`,
        // and `ui-verify`'s `signing`, which is that link's only oracle.
        let shown = t::appearance_shown(&details, &report.appearance_lines);
        format!(
            "sign-disclosed appearance_lines={} appearance_shown={shown}",
            report.appearance_lines.len(),
        )
    });
    match prepared.write_to(target) {
        Ok(_) => Outcome::Written {
            path: PathBuf::from(target),
            replaced: replace,
            details,
        },
        Err(failure) => Outcome::Failed(t::write_failed(&failure.to_string())),
    }
}

/// **Which operator-facing sentence an engine refusal gets.**
///
/// Pure, so every arm is asserted headlessly rather than by driving a window —
/// which matters more here than usual, because some of these arms are only
/// reachable on documents this repository does not commit.
///
/// # 5. The one decision in this function: whose rule refused
///
/// `SignApplyError` has a distinct, already-written sentence per variant, and
/// [`crate::text::sign::engine_refused`] frames them all as *"pdfcer did not
/// sign the document: …"*. That framing is right for most of them and **wrong
/// for the seed-value pair**, and the wrongness is expensive rather than
/// cosmetic.
///
/// The engine enforces a signature field's `/SV` dictionary (Table 234) **in
/// full** and is deliberately **stricter than Acrobat**: a required constraint
/// unmet is refused by name, and a constraint pdfcer cannot evaluate is refused
/// **rather than skipped**. So an operator will meet refusals here on documents
/// Acrobat signs — and *"pdfcer did not sign the document"* beside one of those
/// tells him, in plain English, that pdfcer is broken. He would be right to
/// conclude that from the sentence and wrong about the program, and a working
/// feature would be reported as a defect.
///
/// So [`crate::text::sign::author_imposed`] puts **the person who prepared the
/// document** in the subject position, quotes the engine's message verbatim
/// (because it names the constraint AND the satisfying values, which are the
/// actionable half), states the strictness as a deliberate choice, and gives two
/// remedies that do not require pdfcer to change.
///
/// A few other variants get their own wording for smaller reasons, each noted
/// at its arm. Everything else keeps the general form: the engine's sentence is
/// already an operator-facing one and re-wording it here would be a second
/// spelling of a fact with one author.
fn worded(error: &pdfcer_core::sign::apply::SignApplyError) -> String {
    use pdfcer_core::sign::apply::SignApplyError as E;
    let detail = error.to_string();
    match error {
        // The author's rule, not pdfcer's. See above.
        E::SeedValueViolated { .. } | E::SeedValueUnevaluable { .. } => t::author_imposed(&detail),
        // The chosen box turned out not to be usable. Reachable despite the
        // window filtering its list, because the list is read once when the
        // window opens and the document can change under it — and the remedy
        // ("choose another box, or place your own") is a thing the operator can
        // do on the form he is still looking at, which the engine's own message
        // has no way to know.
        //
        // ⚠ `RectRefusedForExistingField` is here for completeness and is
        // UNREACHABLE from this shell: `crate::sign::Placement`'s three arms are
        // exclusive, so a rectangle and a field name cannot both be sent. It is
        // matched rather than left to the catch-all so that the day somebody
        // splits that enum, the arm is already correct.
        E::FieldNotSignature { .. }
        | E::FieldAlreadySigned { .. }
        | E::FieldHasKids { .. }
        | E::FieldNameTaken { .. }
        | E::RectRefusedForExistingField { .. } => t::field_refused(&detail),
        // The composed appearance does not fit. The engine's advice is
        // "enlarge --visible", and there is no such control here — the box's
        // size is fixed by `crate::sign::default_rect` — so the remedy offered
        // is the one that exists.
        E::AppearanceOverflow { .. } => t::appearance_overflow(&detail),
        // The refusal whose own advice this shell cannot follow: it ends
        // "sign again with a larger reserve" and there is no control that sets
        // one. `crate::sign::prepare`'s note argues why asking would be handing
        // the operator arithmetic.
        E::ReservationTooSmall { .. } => t::reservation_too_small(&detail),
        // No arm for `Edit(FieldAuthoring(DottedPartialName))`, deliberately,
        // and this comment is the record of why — the engine's own doc names
        // `sign` among that variant's raisers, so its absence here looks like
        // an omission.
        //
        // The engine raises it on the CREATE path only, and this shell never
        // takes that path. `crate::sign::Placement::ExistingField { name }` is
        // the only thing that sets `SignRequest::field_name`, and its `name`
        // comes from a list of fields the open document already has — so the
        // engine's `existing.contains(n)` branch is taken and it reuses the
        // field instead of authoring one. Where a period in that name is
        // **correct**: `Approvals.Engineer` is a legitimate nested placeholder
        // to sign into, which is the shape a title block on a drawing leaves,
        // and the engine allows it for exactly that reason.
        //
        // An arm here would word a refusal no operator can provoke. If this
        // shell ever grows a "name a NEW signature box" control, the arm and
        // its sentence are wanted then — and the sentence must say *the name
        // of a new signature box*, because a flat *"a signature box's name
        // cannot contain a dot"* would be false and would stop an operator
        // trying a nested box that signs perfectly well.
        _ => t::engine_refused(&detail),
    }
}

#[cfg(test)]
mod tests {
    use super::worded;
    use pdfcer_core::sign::apply::SignApplyError as E;

    /// **A seed-value refusal is worded as the author's rule, and every
    /// other refusal is not.**
    ///
    /// The whole of §5, asserted rather than argued. The engine enforces
    /// `/SV` in full and is deliberately stricter than Acrobat, so the operator
    /// will meet these on documents another reader signs — and the general
    /// wording, *"pdfcer did not sign the document: …"*, would tell him in
    /// plain English that pdfcer is broken.
    ///
    /// The negative half matters as much: a refusal that is genuinely
    /// pdfcer's (the fixed reservation) must NOT be dressed up as somebody
    /// else's rule. Blaming the document's author for a pdfcer limit is the
    /// same defect pointed the other way.
    #[test]
    fn only_a_seed_value_refusal_blames_the_documents_author() {
        let authors_rule = [
            E::SeedValueViolated {
                name: "SignHere".to_owned(),
                constraint: "Reasons one of: Approved".to_owned(),
            },
            E::SeedValueUnevaluable {
                name: "SignHere".to_owned(),
                what: "/Cert".to_owned(),
            },
        ];
        for error in authors_rule {
            let sentence = worded(&error);
            assert!(
                sentence.contains("prepared this document"),
                "a seed-value refusal must name the document's author: {sentence}"
            );
            assert!(sentence.contains("not a limit in pdfcer"), "{sentence}");
        }

        // pdfcer's own limits keep pdfcer as the subject.
        let ours = worded(&E::ReservationTooSmall {
            needed: 20_000,
            reserved: 12_288,
        });
        assert!(!ours.contains("prepared this document"), "{ours}");
        assert!(ours.contains("pdfcer reserves a fixed amount"), "{ours}");
    }

    /// **A field that cannot be used sends the operator back to the form.**
    ///
    /// Reachable even though the window filters its list, because the list is
    /// read once when the window opens and the document can change under it.
    /// The engine's message says what is wrong; this adds the remedy that
    /// exists on the screen the operator is still looking at.
    #[test]
    fn an_unusable_field_offers_the_other_two_routes() {
        let sentence = worded(&E::FieldAlreadySigned {
            name: "SignHere".to_owned(),
        });
        assert!(sentence.contains("Choose another box"), "{sentence}");
        assert!(sentence.contains("place your own"), "{sentence}");
    }

    /// **The appearance-overflow refusal does not repeat advice this shell
    /// cannot take.**
    ///
    /// The engine's message ends *"enlarge --visible, or drop
    /// --reason/--location"*, and there is no control here that enlarges the
    /// box — `crate::sign::default_rect` fixes it. So the engine's sentence is
    /// shown and the remedy offered is one the operator can actually perform.
    #[test]
    fn the_overflow_refusal_offers_a_remedy_this_window_has() {
        let sentence = worded(&E::AppearanceOverflow {
            lines: 4,
            width: 180.0,
            height: 60.0,
            min_size: 4.0,
        });
        assert!(
            sentence.contains("Shorten or clear the reason"),
            "{sentence}"
        );
        assert!(
            sentence.contains("do not draw anything on the page"),
            "the alternative that always works is named: {sentence}"
        );
    }
}
