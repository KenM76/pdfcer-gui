//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/sign/tests.md`.
#![cfg(test)]
//! Tests for [`super`] — the Sign window's pure decisions.
//!
//! Everything asserted here is reachable without an `egui::Context`.
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
        timestamp_server: String::new(),
        destination: Destination::NewFile,
        overwrite_acknowledged: false,
        open_certificate_requested: false,
        pick_requested: false,
        create: None,
        confirm_requested: false,
        open_signed_requested: false,
        close_requested: false,
    }
}

/// **The confirm control is dead until a certificate has been OPENED.**
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

/// **Changing the destination retires an acknowledgement already given.**
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

/// **`Debug` prints no passphrase, no certificate path, and no key.**
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

/// **Picking a different certificate retires the identity AND the error.**
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

/// **No sentence on this surface calls a signature valid, trusted, secure
/// or verified.**
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

    // `Pass 10.12`'s certification copy is on this list TOO, and the
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

    // "checked" is NOT on this list, deliberately: `identity_integrity` says
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

/// **The box is no longer described as empty, and this assertion is the
/// successor to a paragraph that could not go red.**
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

/// **AN AUTHOR-IMPOSED REFUSAL SAYS WHOSE RULE IT IS, AND PDFCER IS NOT THE
/// SUBJECT OF THE FIRST SENTENCE.**
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
#[test]
fn the_written_summary_carries_the_reuse_the_lock_and_the_notes() {
    let written = crate::text::sign::written_details(&crate::text::sign::Written {
        field: "SignHere",
        subject: "CN=Ken",
        serial: "0A1B",
        reused: true,
        lock: Some("Include: Name"),
        certification: Some("form fill-in and signing"),
        notes: &["seed value: a timestamp was recommended".to_owned()],
        appearance: &[],
        timestamp: None,
    });
    assert!(written.contains("already on the document"), "{written}");
    assert!(written.contains("SignHere"), "{written}");
    assert!(written.contains("Include: Name"), "{written}");
    assert!(written.contains("form fill-in and signing"), "{written}");
    assert!(written.contains("a timestamp was recommended"), "{written}");

    // The created-field case says none of it, rather than saying "no lock" and
    // "no notes" — an absence is not a disclosure.
    let plain = crate::text::sign::written_details(&crate::text::sign::Written {
        field: "Signature1",
        subject: "CN=Ken",
        serial: "0A1B",
        reused: false,
        lock: None,
        certification: None,
        notes: &[],
        appearance: &[],
        timestamp: None,
    });
    assert!(plain.contains("Signature field Signature1"), "{plain}");
    assert!(!plain.contains("already on the document"), "{plain}");
    assert!(!plain.contains("lock"), "{plain}");
}

/// **What the signature's box will read reaches the screen.**
#[test]
fn the_written_summary_carries_the_text_the_signature_box_shows() {
    let visible = crate::text::sign::written_details(&crate::text::sign::Written {
        field: "Signature1",
        subject: "CN=Ken",
        serial: "0A1B",
        reused: false,
        lock: None,
        certification: None,
        notes: &[],
        appearance: &[
            "Digitally signed by Ken Mantle".to_owned(),
            "Date: 2026-09-18 10:04:11 -04'00'".to_owned(),
            "Reason: I approve this drawing".to_owned(),
        ],
        timestamp: None,
    });
    assert!(
        visible.contains("Digitally signed by Ken Mantle"),
        "the engine's own line, verbatim: {visible}"
    );
    assert!(
        visible.contains("Date: 2026-09-18 10:04:11 -04'00'"),
        "including the time, which the operator did not type: {visible}"
    );
    assert!(
        visible.contains("Reason: I approve this drawing"),
        "and every line, not merely the first: {visible}"
    );

    // An invisible signature composes nothing, and an absence is not a
    // disclosure: the screen says nothing about a box rather than announcing an
    // empty one. `reused` is false in both calls, so the word cannot arrive
    // from the sentence about signing into a box that was already there.
    let invisible = crate::text::sign::written_details(&crate::text::sign::Written {
        field: "Signature1",
        subject: "CN=Ken",
        serial: "0A1B",
        reused: false,
        lock: None,
        certification: None,
        notes: &[],
        appearance: &[],
        timestamp: None,
    });
    assert!(
        !invisible.to_lowercase().contains("box"),
        "no box is mentioned when none was drawn: {invisible}"
    );
}

/// **The counter that the driven check reads measures the sentence.**
#[test]
fn the_appearance_counter_measures_the_sentence_not_the_slice() {
    let lines = [
        "Digitally signed by Ken Mantle".to_owned(),
        "Date: 2026-09-18 10:04:11 -04'00'".to_owned(),
    ];
    let full = crate::text::sign::written_details(&crate::text::sign::Written {
        field: "Signature1",
        subject: "CN=Ken",
        serial: "0A1B",
        reused: false,
        lock: None,
        certification: None,
        notes: &[],
        appearance: &lines,
        timestamp: None,
    });
    assert_eq!(
        crate::text::sign::appearance_shown(&full, &lines),
        2,
        "both lines are in it: {full}"
    );

    // The defect the counter exists to name: the composer was handed nothing,
    // so the sentence carries neither line while the engine composed two.
    let none = crate::text::sign::written_details(&crate::text::sign::Written {
        field: "Signature1",
        subject: "CN=Ken",
        serial: "0A1B",
        reused: false,
        lock: None,
        certification: None,
        notes: &[],
        appearance: &[],
        timestamp: None,
    });
    assert_eq!(
        crate::text::sign::appearance_shown(&none, &lines),
        0,
        "and it says so rather than reporting the engine's own count: {none}"
    );

    // The subject is in the sentence, and it is not an appearance line. A
    // counter matching a bare substring would score this 1.
    assert_eq!(
        crate::text::sign::appearance_shown(&none, &["CN=Ken".to_owned()]),
        0,
        "only the block's own indented lines count: {none}"
    );
}

/// **The three placement arms map to three different requests.**
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

/// **An index that outran its list falls back to drawing NOTHING.**
#[test]
fn a_field_index_with_no_field_draws_nothing() {
    let mut dialog = filling();
    dialog.place = Place::Existing;
    dialog.field = 7;
    assert_eq!(dialog.placement(), crate::sign::Placement::Invisible);
}
