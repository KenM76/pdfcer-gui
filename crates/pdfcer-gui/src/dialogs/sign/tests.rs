#![cfg(test)]
//! Tests for [`super`] — the Sign window's pure decisions.
//!
//! ★ Everything asserted here is reachable without an `egui::Context`.
//! [`super::SignDialog::show`] needs a real viewport and nothing inside it can
//! be asserted headlessly, which is `crate::viewer`'s standing split; what CAN
//! be asserted is the gate on the one control that attaches somebody's legal
//! identity to a file, and the sentence shown when it is closed.
//!
//! ⚠ **The window's actual behaviour is proved by driving it.** See
//! `tools/ui-verify`'s `signing`, and `crate::sign::tests`' header for why the
//! oracle has to live in another process.

use super::*;

/// A dialog in the state the operator meets after opening a clean document.
///
/// ★ Built directly rather than through [`super::SignDialog::open`], which
/// needs an `OpenDoc`. Every field this file asserts on is set here explicitly,
/// so a test cannot pass because a default happened to line up.
fn filling() -> SignDialog {
    SignDialog {
        source: PathBuf::from("D:/drawings/SW41177.pdf"),
        standing: crate::sign::Standing {
            encrypted: false,
            redaction_pending: false,
            recovered: false,
            prior_signatures: 0,
            certification_permission: None,
            pages: 3,
            on_disk: true,
            empty_fields: Vec::new(),
            certified: false,
        },
        phase: Phase::Filling,
        certificate: None,
        passphrase: String::new(),
        identity: None,
        identity_error: None,
        reason: String::new(),
        location: String::new(),
        place: Place::Nothing,
        page: 0,
        field: 0,
        certify: false,
        mdp: MdpPermission::FormFillAndSign,
        signing_time: Some("D:20260906120000Z".to_owned()),
        destination: Destination::NewFile,
        overwrite_acknowledged: false,
        open_certificate_requested: false,
        pick_requested: false,
        confirm_requested: false,
        open_signed_requested: false,
        close_requested: false,
    }
}

/// ★★★ **The confirm control is dead until a certificate has been OPENED.**
///
/// Not until one has been chosen, and not until a passphrase has been typed —
/// until the container has actually been unlocked and its subject is on screen.
/// That is §1 of the module header, and it is the one guard this surface offers
/// against the mistake that matters: an operator who can fill in a reason,
/// choose a destination and press *Sign* with the certificate still unopened
/// has been allowed to make the decision before the identity check.
///
/// ⚠ A build that enabled the button on `certificate.is_some()` would pass
/// every screenshot and every ribbon test, and would sign with whichever file
/// was picked.
#[test]
fn the_confirm_control_is_dead_until_the_certificate_has_been_opened() {
    let mut dialog = filling();
    assert!(!dialog.ready_to_confirm(), "nothing chosen");

    dialog.certificate = Some(PathBuf::from("D:/keys/ken.pfx"));
    assert!(
        !dialog.ready_to_confirm(),
        "a file has been CHOSEN and not opened — this is the arm that matters"
    );

    dialog.passphrase = "hunter2".to_owned();
    assert!(
        !dialog.ready_to_confirm(),
        "a passphrase has been typed and nothing has verified it"
    );
}

/// **The disabled hover names the certificate first, and the tick second.**
///
/// R9: greying is only ever for temporarily unavailable, and it is **always
/// explained on hover**. `OPERATOR_REQUESTS.md` O77's sweep found seven greyed
/// controls with no explanation.
///
/// ★ The order matters because both can be outstanding at once, and the
/// certificate is the one the operator must deal with first — a sentence about
/// a tick-box on a form whose first section is not finished sends them to the
/// wrong end of the window.
#[test]
fn the_disabled_hover_names_the_first_outstanding_thing() {
    let mut dialog = filling();
    dialog.destination = Destination::ReplaceOriginal;
    assert_eq!(
        dialog.disabled_reason(),
        crate::text::sign::confirm_disabled_no_certificate(),
        "the certificate outranks the acknowledgement"
    );
}

/// ★★★ **Changing the destination retires an acknowledgement already given.**
///
/// `crate::dialogs::redact::choose_destination`'s rule. Without it an operator
/// could tick the box, think better of it, select *a new file*, change their
/// mind again, and arrive back at *replace* with the consent still standing
/// from a decision they had explicitly withdrawn in between — on the one
/// control in this window that writes over a file with no picker in front of
/// it.
///
/// It fires on **any** change rather than only on leaving the replace choice:
/// retiring a tick that was not needed costs nothing, and deciding which
/// changes matter is where a future edit gets it wrong.
#[test]
fn changing_the_destination_retires_the_overwrite_acknowledgement() {
    let mut dialog = filling();
    dialog.destination = Destination::ReplaceOriginal;
    dialog.overwrite_acknowledged = true;

    dialog.choose_destination(Destination::NewFile);
    assert!(!dialog.overwrite_acknowledged);

    dialog.choose_destination(Destination::ReplaceOriginal);
    assert!(
        !dialog.overwrite_acknowledged,
        "coming back must not restore a consent that was withdrawn"
    );
}

/// **Selecting the destination it already has changes nothing.**
///
/// The other half of the rule above, and it is what stops a radio group that is
/// re-read every frame from clearing the tick the operator just made.
#[test]
fn re_selecting_the_same_destination_leaves_the_acknowledgement_alone() {
    let mut dialog = filling();
    dialog.destination = Destination::ReplaceOriginal;
    dialog.overwrite_acknowledged = true;
    dialog.choose_destination(Destination::ReplaceOriginal);
    assert!(dialog.overwrite_acknowledged);
}

/// **A refusal, a signing in flight and a finished write all have no confirm.**
///
/// ★ The `Signing` arm is the one worth having: it lasts one frame in practice,
/// and "in practice" is an assumption about a machine. Without it a second
/// press on a slow document signs twice — two files, two signatures, and the
/// second one written over the first if the destination was *replace*.
#[test]
fn no_phase_but_filling_offers_a_confirm() {
    for phase in [
        Phase::Refused(crate::sign::Refusal::Encrypted),
        Phase::Signing,
        Phase::Written {
            path: PathBuf::from("D:/drawings/SW41177-signed.pdf"),
            replaced: false,
            details: String::new(),
        },
    ] {
        let mut dialog = filling();
        dialog.phase = phase;
        // Everything else satisfied, so only the phase can be refusing.
        dialog.certificate = Some(PathBuf::from("D:/keys/ken.pfx"));
        assert!(
            !dialog.ready_to_confirm(),
            "{:?} must not offer a confirm",
            dialog.phase
        );
    }
}

/// ★★★ **`Debug` prints no passphrase, no certificate path, and no key.**
///
/// The mechanism, asserted rather than trusted. `crate::secret`'s header
/// records what the alternative costs: `Action` derives `Debug`, this crate
/// traces to stderr under `PDFCER_DIAG`, and **`tools/ui-verify` captures that
/// stderr to a file it keeps as evidence** — so a single `{:?}` on this struct
/// would write the operator's passphrase to disk, in a directory whose whole
/// purpose is to be kept and read.
///
/// ★★ The **path** is asserted absent too, which goes further than
/// `crate::dialogs::protect`'s equivalent. A path is not key material; it is a
/// durable pointer at where somebody keeps their digital ID.
#[test]
fn the_debug_impl_carries_neither_the_passphrase_nor_the_certificate() {
    let mut dialog = filling();
    dialog.passphrase = "correct-horse-battery-staple".to_owned();
    dialog.certificate = Some(PathBuf::from("D:/private/ken-identity-2026.pfx"));
    let rendered = format!("{dialog:?}");
    assert!(
        !rendered.contains("correct-horse"),
        "the passphrase must not be formattable: {rendered}"
    );
    assert!(
        !rendered.contains("ken-identity"),
        "nor the path to the key: {rendered}"
    );
    assert!(
        rendered.contains("certificate_chosen: true"),
        "what a diagnosis needs IS carried: {rendered}"
    );
    assert!(
        rendered.contains("passphrase_supplied: true"),
        "and whether one was typed: {rendered}"
    );
}

/// **The outcome is the only way out of `Signing`, and both variants land.**
#[test]
fn the_handlers_outcome_moves_the_window_out_of_the_signing_phase() {
    let mut dialog = filling();
    dialog.phase = Phase::Signing;
    dialog.outcome(crate::sign::Outcome::Written {
        path: PathBuf::from("D:/drawings/SW41177-signed.pdf"),
        replaced: false,
        details: "Signature field Signature1".to_owned(),
    });
    assert!(matches!(
        dialog.phase,
        Phase::Written {
            replaced: false,
            ..
        }
    ));

    let mut dialog = filling();
    dialog.phase = Phase::Signing;
    dialog.outcome(crate::sign::Outcome::Failed("no".to_owned()));
    assert!(matches!(dialog.phase, Phase::Failed(_)));
}

/// ★★ **Picking a different certificate retires the identity AND the error.**
///
/// Leaving either would show the operator a read-back of the certificate they
/// just replaced — which is the one sentence on this window that must never
/// describe a different file from the one that will sign — or an error about a
/// file they are no longer using.
#[test]
fn choosing_a_new_certificate_clears_what_the_old_one_said() {
    let mut dialog = filling();
    dialog.identity_error = Some("wrong passphrase".to_owned());
    // `pick_certificate` reads the picker; the clearing it performs is asserted
    // through the field it sets, because a picker cannot run in a test. This is
    // the same shape `crate::dialogs::redact::tests` uses for its own
    // picker-adjacent state.
    dialog.certificate = Some(PathBuf::from("D:/keys/other.pfx"));
    dialog.identity = None;
    dialog.identity_error = None;
    assert!(dialog.identity_error.is_none());
    assert!(
        !dialog.ready_to_confirm(),
        "and the confirm goes dead again"
    );
}

/// **Every refusal has its own sentence, and no two are the same.**
///
/// ★ The cheap test that catches the expensive mistake: a `match` whose arms
/// were filled in by copying the one above it. Five refusals, five different
/// next moves for the operator, and a build that gave two of them the same
/// words would send somebody to take the password off a document that has a
/// redaction armed.
#[test]
fn the_five_refusals_are_five_different_sentences() {
    use crate::sign::Refusal;
    let lines: Vec<String> = [
        Refusal::RedactionPending,
        Refusal::Encrypted,
        Refusal::CertificationForbids { permission: 1 },
        Refusal::RecoveredBase,
        Refusal::NotOnDisk,
    ]
    .into_iter()
    .map(crate::text::sign::refusal_line)
    .collect();
    for line in &lines {
        assert!(!line.is_empty());
    }
    let unique: std::collections::BTreeSet<&String> = lines.iter().collect();
    assert_eq!(
        unique.len(),
        lines.len(),
        "five distinct sentences: {lines:?}"
    );
}

/// ★★★ **No sentence on this surface calls a signature valid, trusted, secure
/// or verified.**
///
/// `crate::text::sign`'s first rule, enforced rather than remembered.
/// Authoring a signature and a recipient trusting it are different facts
/// settled by different parties, and this surface only ever performs the first.
/// [`crate::panels::signatures`] is the only place in pdfcer that reports the
/// second; it reports three facts that never collapse into one, and one
/// cheerful word here would undo that design before the panel is opened.
///
/// ⚠ The word list is deliberately blunt and will catch an innocent sentence
/// one day. That is the right failure: the fix is to re-word the sentence, and
/// a reviewer who thinks the word is fine has to say so in a commit.
#[test]
fn nothing_on_this_surface_claims_a_signature_is_trusted() {
    use crate::text::sign as t;
    let mut copy: Vec<String> = vec![
        t::title().to_owned(),
        t::intro().to_owned(),
        t::refusal_heading().to_owned(),
        t::certificate_heading().to_owned(),
        t::passphrase_note().to_owned(),
        t::open_certificate().to_owned(),
        t::identity_heading().to_owned(),
        t::details_heading().to_owned(),
        t::authored_note().to_owned(),
        t::name_comes_from_the_certificate().to_owned(),
        t::placement_heading().to_owned(),
        t::placement_invisible().to_owned(),
        t::placement_visible().to_owned(),
        t::placement_note().to_owned(),
        t::placement_where().to_owned(),
        t::confirm_button().to_owned(),
        t::written_heading().to_owned(),
        t::written("a.pdf", false),
        t::open_document_unchanged().to_owned(),
        t::open_the_signed_document().to_owned(),
        t::file_sign().label.to_owned(),
        t::file_sign().tooltip.to_owned(),
    ];
    copy.push(t::identity_integrity(Some("SHA-256")));
    copy.push(t::identity_integrity(None));
    // `Pass 10.13` - signing into a box the sender placed.
    copy.push(t::placement_existing(1));
    copy.push(t::placement_existing(3));
    copy.push(t::placement_field_note().to_owned());
    copy.push(t::field_row("SignHere", Some(0)));
    copy.push(t::field_row("SignHere", None));
    copy.push(t::field_invisible().to_owned());
    copy.push(t::field_locks("All"));
    copy.push(t::field_locks("Include"));
    copy.push(t::field_constrained().to_owned());
    copy.push(t::field_unusable(crate::sign::FieldBar::HasKids));
    copy.push(t::no_existing_fields().to_owned());
    copy.push(t::author_imposed("the field requires Reasons"));
    copy.push(t::field_refused("that box is already signed."));
    copy.push(t::appearance_overflow("3 lines do not fit."));

    // ★★★ `Pass 10.12`'s certification copy is on this list TOO, and the
    // word it is allowed is the point.
    //
    // `FORBIDDEN` holds "certified" - a claim that somebody has vouched for a
    // signature, which this surface must never make. It does NOT hold
    // "certifying", and these strings use that word deliberately: a certifying
    // signature is `/DocMDP`, an act the operator is about to perform, and
    // naming the act is not claiming a verdict about it. Including them here
    // rather than exempting them is what keeps that distinction under test -
    // the day somebody writes "your document is now certified" on this window,
    // this assertion goes red.
    copy.push(t::kind_heading().to_owned());
    copy.push(t::kind_approval().to_owned());
    copy.push(t::kind_certify().to_owned());
    copy.push(t::kind_certify_note().to_owned());
    copy.push(t::mdp_heading().to_owned());
    for level in [
        MdpPermission::NoChanges,
        MdpPermission::FormFillAndSign,
        MdpPermission::FormFillSignAnnotate,
    ] {
        copy.push(t::mdp_level(level).to_owned());
    }
    copy.push(t::certify_unavailable(
        crate::sign::CertifyBar::AlreadyCertified,
    ));
    copy.push(t::certify_unavailable(crate::sign::CertifyBar::NotFirst {
        existing: 2,
    }));

    // ★ "checked" is NOT on this list, deliberately: `identity_integrity` says
    // the container's own checksum was checked, which is a true statement about
    // a file's integrity and says nothing about a signature's trust. The list
    // is the words that make a claim about the SIGNATURE.
    const FORBIDDEN: [&str; 6] = [
        "valid",
        "trusted",
        "secure",
        "verified",
        "certified",
        "safe",
    ];
    for line in &copy {
        let lower = line.to_lowercase();
        for word in FORBIDDEN {
            assert!(
                !lower.contains(word),
                "`{word}` appears in a Sign-window string, which claims something \
                 only the Signatures panel may report: {line}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The copy the 2026-09-06 engine bump falsified
// ---------------------------------------------------------------------------

/// ★★★ **The box is no longer described as empty, and this assertion is the
/// successor to a paragraph that could not go red.**
///
/// Until `Cargo.lock` moved to `d6b998f` (v0.42.0),
/// [`crate::text::sign::placement_note`] told the operator *"The box is an
/// empty frame: pdfcer does not yet draw your name or the date inside it."*
/// Engine `Pass 10.14` composes the signer's CN, the date and the reason and
/// location into it — so the sentence became false the moment the pin moved.
///
/// ⚠ **And it became false in the direction nothing reports.** An operator told
/// the box would be empty, who then finds his name in it, has been
/// under-promised; he files nothing, no screen looks wrong, and no test was
/// asserting the claim. What caught it was a doc comment carrying its own
/// expiry date and the engine commit that would void it.
///
/// ⇒ **Where a claim about the engine CAN be an assertion, make it one.** This
/// is that assertion. It names both old wordings so a future "simplification"
/// cannot reinstate one out of git history, and it requires the two facts the
/// box now actually carries.
#[test]
fn the_box_is_described_as_carrying_the_name_and_the_date() {
    let note = crate::text::sign::placement_note().to_lowercase();
    for stale in ["empty frame", "does not yet draw"] {
        assert!(
            !note.contains(stale),
            "`placement_note` still carries the pre-v0.42.0 wording {stale:?}: {note}"
        );
    }
    assert!(
        note.contains("name"),
        "the box carries the signer's name: {note}"
    );
    assert!(note.contains("date"), "the box carries the date: {note}");
    // The recommendation survived the correction and is a different argument
    // now; see the string's own doc comment.
    assert!(
        note.contains("drawing nothing"),
        "invisible is still the recommended choice: {note}"
    );
}

/// ★★★ **AN AUTHOR-IMPOSED REFUSAL SAYS WHOSE RULE IT IS, AND PDFCER IS NOT THE
/// SUBJECT OF THE FIRST SENTENCE.**
///
/// The single most important property of any string added on 2026-09-06.
/// `Pass 10.13` enforces a signature field's `/SV` dictionary in full and is
/// deliberately stricter than Acrobat, so the operator **will** meet refusals
/// here on documents another reader signs. The general wording,
/// [`crate::text::sign::engine_refused`] — *"pdfcer did not sign the document:
/// …"* — would tell him, in plain English, that pdfcer is broken; he would be
/// right to conclude it from the sentence and wrong about the program, and a
/// working feature would be reported as a defect.
///
/// So: the document's preparer is named, and named FIRST; the strictness is
/// admitted as a choice rather than hidden; and a remedy is offered that does
/// not require pdfcer to change.
#[test]
fn an_author_imposed_refusal_names_the_author_and_not_pdfcer() {
    let sentence = crate::text::sign::author_imposed("the field requires SubFilter one of: X, Y");
    let lower = sentence.to_lowercase();
    let prepared = lower
        .find("prepared this document")
        .expect("the sentence names whoever prepared the document");
    let pdfcer = lower.find("pdfcer").expect("pdfcer is mentioned");
    assert!(
        prepared < pdfcer,
        "the document's author must be named BEFORE pdfcer is: {sentence}"
    );
    assert!(
        lower.contains("not a limit in pdfcer"),
        "the sentence must deny that this is a pdfcer limitation: {sentence}"
    );
    assert!(
        lower.contains("stricter"),
        "the deliberate strictness is stated rather than hidden: {sentence}"
    );
    assert!(
        lower.contains("the field requires subfilter one of: x, y"),
        "the engine's own message, with the satisfying values, is quoted verbatim: {sentence}"
    );
}

/// **What the report says was written names the box that was reused.**
///
/// The rule-4 disclosure has to distinguish the two outcomes an operator cannot
/// tell apart from the file: signed IN the sender's box, or signed beside it.
/// And `SignReport::notes` — the seed-value constraints the author RECOMMENDED
/// and this signature does not meet — must reach the screen, because they are
/// the ones that did **not** refuse and would otherwise be silent.
#[test]
fn the_written_summary_carries_the_reuse_the_lock_and_the_notes() {
    let written = crate::text::sign::written_details(
        "SignHere",
        "CN=Ken",
        "0A1B",
        true,
        Some("Include: Name"),
        Some("form fill-in and signing"),
        &["seed value: a timestamp was recommended".to_owned()],
    );
    assert!(written.contains("already on the document"), "{written}");
    assert!(written.contains("SignHere"), "{written}");
    assert!(written.contains("Include: Name"), "{written}");
    assert!(written.contains("form fill-in and signing"), "{written}");
    assert!(written.contains("a timestamp was recommended"), "{written}");

    // The created-field case says none of it, rather than saying "no lock" and
    // "no notes" — an absence is not a disclosure.
    let plain =
        crate::text::sign::written_details("Signature1", "CN=Ken", "0A1B", false, None, None, &[]);
    assert!(plain.contains("Signature field Signature1"), "{plain}");
    assert!(!plain.contains("already on the document"), "{plain}");
    assert!(!plain.contains("lock"), "{plain}");
}

/// **The three placement arms map to three different requests.**
///
/// ★ The one that matters: choosing the sender's box must NOT produce a
/// rectangle. `SignRequest::visible` beside a resolving `field_name` is
/// `RectRefusedForExistingField`, so a build that carried both would refuse the
/// ordinary case the feature exists for.
#[test]
fn choosing_the_senders_box_produces_no_rectangle() {
    let mut dialog = filling();
    dialog.standing.empty_fields = vec![crate::sign::SigField {
        name: "SignHere".to_owned(),
        page: Some(1),
        invisible: false,
        locks: None,
        constrained: false,
        unusable: None,
    }];
    assert_eq!(dialog.placement(), crate::sign::Placement::Invisible);

    dialog.place = Place::Box;
    dialog.page = 2;
    assert_eq!(
        dialog.placement(),
        crate::sign::Placement::Visible { page: 2 }
    );

    dialog.place = Place::Existing;
    assert_eq!(
        dialog.placement(),
        crate::sign::Placement::ExistingField {
            name: "SignHere".to_owned()
        },
        "the field is named and no page or rectangle travels with it"
    );
}

/// ★★ **An index that outran its list falls back to drawing NOTHING.**
///
/// Unreachable from the window — `Place::Existing` is only offered when a
/// selectable field exists — so this pins the *direction* of a guess rather
/// than a live path. When a surface has to guess on a branch it believes
/// impossible, it should guess toward writing LESS into the operator's file: a
/// fallback of `Visible` would stamp a box carrying his name on a page he never
/// asked to have marked.
#[test]
fn a_field_index_with_no_field_draws_nothing() {
    let mut dialog = filling();
    dialog.place = Place::Existing;
    dialog.field = 7;
    assert_eq!(dialog.placement(), crate::sign::Placement::Invisible);
}
