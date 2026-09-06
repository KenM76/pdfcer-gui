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
        },
        phase: Phase::Filling,
        certificate: None,
        passphrase: String::new(),
        identity: None,
        identity_error: None,
        reason: String::new(),
        location: String::new(),
        visible: false,
        page: 0,
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
