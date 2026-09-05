#![cfg(test)]
//! # `dialogs::protect::tests` — the window's own rules, asserted without one
//!
//! Every rule below decides whether a control the operator can press writes
//! over their file, and every one of them is expressed as a **pure function on
//! the dialog's state** for exactly that reason — `crate::viewer`'s standing
//! split: *a rule that can only be exercised by driving a window is a rule that
//! gets asserted once, by hand, and then drifts.*
//!
//! The model's own properties — what the document says, what a change
//! preserves, what the engine refuses — live in `crate::protect::tests` and are
//! not repeated here.
//!
//! | test | the defect it catches |
//! |---|---|
//! | [`the_form_opens_seeded_from_the_document_not_from_a_default`] | the build brief's own sentence: a dialog that lies before it is touched |
//! | [`a_signed_document_draws_no_form_at_all`] | O119 disclosure 2, and R9 — a button that fails on press |
//! | [`the_confirm_control_is_shut_until_every_condition_is_met`] | a write reachable before the form is complete |
//! | [`the_greyed_confirm_names_the_outstanding_condition`] | O77's seven unexplained greyed controls, re-committed |
//! | [`changing_the_destination_retires_the_acknowledgement`] | consent standing from a decision that was withdrawn |
//! | [`changing_the_job_re_seeds_the_ticks_from_the_file`] | a permission list left wherever a previous job's editing put it |
//! | [`the_accessibility_bit_is_never_offered_as_a_choice`] | a tick-box that comes back ticked in the written file |
//! | [`the_debug_impl_does_not_carry_a_password`] | a typed password written into the trace file kept as evidence |

use super::*;

use crate::app::state::open_local_fixture;

/// A dialog over the plain fixture, opened from **Encrypt…**.
fn plain_password_dialog() -> ProtectDialog {
    ProtectDialog::open(&open_local_fixture("four-pages.pdf"), Task::Password)
}

/// **A document on disk that really does decline something**, and a
/// `Permissions…` window over it.
///
/// ★★★ This helper exists because the first draft of
/// [`the_form_opens_seeded_from_the_document_not_from_a_default`] **passed
/// under its own falsification**, and that is worth recording rather than
/// quietly fixing. The test built its dialog over the plain fixture and
/// asserted `ticks == standing.initial_ticks()`. On an unprotected document
/// every bit is granted, so replacing the seeding expression with a hard-coded
/// *eight ticks* — the exact defect the build brief names — produced a passing
/// test.
///
/// `HANDOFF.md` §2's defect 8 in its purest form: *a test that checks a
/// relation rather than a magnitude is satisfied by any absurdity in the right
/// direction.* Two values that are equal for a reason unrelated to the code
/// under test are the same trap.
///
/// ⇒ So the assertion moved to a document written with **`Print` alone**, where
/// a hard-coded default and the file's own answer are different lists and
/// cannot be confused. `granted` is what the caller wants written; the file
/// comes back also granting `AccessibilityExtract`, which the engine sets on
/// every write (see [`crate::protect::always_granted`]).
fn restricted_permissions_dialog(tag: &str, granted: &[PermissionBit]) -> ProtectDialog {
    use pdfcer_core::document::Document;
    use pdfcer_core::edit::{EditSession, EncryptionSettings};
    use pdfcer_core::writer::SaveOptions;

    // ui-text-exempt: test data, never displayed.
    const OWNER: &[u8] = b"ownerpw";
    // ui-text-exempt: test data, never displayed.
    const USER: &[u8] = b"userpw";

    let source = open_local_fixture("four-pages.pdf");
    let mut settings = EncryptionSettings::new(USER.to_vec(), OWNER.to_vec());
    settings.permissions = granted.to_vec();
    let (bytes, _) = source
        .session
        .set_encryption(&settings, &SaveOptions::default())
        .expect("the plain fixture encrypts");

    let mut path = std::env::temp_dir();
    // ★ Tagged per caller: `cargo test` runs these in parallel, and two tests
    // writing one path is a flake that reproduces about a third of the time.
    path.push(format!("pdfcer-dialog-protect-{tag}.pdf"));
    std::fs::write(&path, &bytes).expect("the scratch file is writable");

    let document =
        Document::load_with_password(&path, Some(OWNER)).expect("the owner password opens it");
    let pages = pdfcer_core::page_tree::pages(&document).expect("a page tree");
    let doc = crate::app::state::OpenDoc::new(path, EditSession::new(document), pages);
    ProtectDialog::open(&doc, Task::Permissions)
}

/// ★★★ **The form opens showing what the FILE says, not what is convenient.**
///
/// The build brief's strongest requirement on this surface, asserted at the
/// seam where it could be lost: *"a permissions dialog that opens with
/// everything ticked, on a document that forbids printing, has told him a
/// falsehood before he touches anything."*
///
/// ★★★ **It is asserted on a document that FORBIDS something**, and the first
/// draft was not — see [`restricted_permissions_dialog`] for the falsification
/// this test failed and what changed because of it. On the plain fixture the
/// answer genuinely IS eight ticks, so a hard-coded eight ticks and the file's
/// own answer are the same list and the assertion cannot tell them apart.
#[test]
fn the_form_opens_seeded_from_the_document_not_from_a_default() {
    let dialog = restricted_permissions_dialog("seeded", &[PermissionBit::Print]);

    // ★ THE ASSERTION THAT BITES: the document declines six of the eight, so a
    // form seeded from a constant would have eight `true`s here.
    let ticked: Vec<PermissionBit> = dialog
        .ticks
        .iter()
        .filter(|(_, on)| *on)
        .map(|(bit, _)| *bit)
        .collect();
    assert_eq!(
        ticked,
        vec![PermissionBit::Print, PermissionBit::AccessibilityExtract],
        "the boxes are the FILE's answer — printing, and the accessibility bit \
         the engine grants on every write — not a convenient default"
    );
    assert!(
        !dialog
            .ticks
            .iter()
            .any(|(bit, on)| *bit == PermissionBit::Copy && *on),
        "this document forbids copying and the window must not open saying otherwise"
    );
    assert_eq!(
        dialog.ticks,
        dialog.standing.initial_ticks(),
        "and the seed is that function rather than a second derivation of it"
    );
    assert_eq!(
        dialog.standing.grants.len(),
        8,
        "all eight bits are reported, never a partial list"
    );
    assert_eq!(dialog.job, Job::SetPermissions);
    assert!(matches!(dialog.phase, Phase::Filling));

    let _ = std::fs::remove_file(&dialog.source);

    // The plain document's own opening state, which is about defaults rather
    // than about the seed: nothing pre-filled, and the safe destination.
    let plain = plain_password_dialog();
    assert_eq!(plain.job, Job::SetPassword);
    assert!(plain.current_owner.is_empty());
    assert!(plain.owner.is_empty());
    assert_eq!(plain.destination, Destination::NewFile);
    assert!(plain.encrypt_metadata, "the engine's own default");
}

/// ★★★ **O119 disclosure 2, and R9: a signed document draws no form.**
///
/// Not a greyed form and not a button that refuses on press — the phase is
/// `Refused` before anything is drawn, so [`ProtectDialog::body`] takes the
/// branch that states the refusal and offers nothing.
#[test]
fn a_signed_document_draws_no_form_at_all() {
    let doc = open_local_fixture("signed-two-pages.pdf");
    for task in [Task::Password, Task::Permissions] {
        let dialog = ProtectDialog::open(&doc, task);
        let Phase::Refused(Refusal::Signed { signatures }) = dialog.phase else {
            panic!(
                "a signed document must refuse before the form: {:?}",
                dialog.phase
            );
        };
        assert!(signatures > 0);
        assert!(
            !dialog.ready_to_confirm(),
            "there is nothing to confirm on a refused document"
        );
        // The sentence names the count and explains the mechanism, which is
        // what tells the operator the real remedy: protect first, sign second.
        let line = t::signed_refusal(signatures);
        assert!(line.contains("rewrites every byte"), "{line}");
        assert!(line.contains("sign it afterwards"), "{line}");
    }
}

/// **The confirm control is shut until every condition is met.**
///
/// Walked one condition at a time, so a build that dropped any single gate
/// fails here rather than passing on the strength of the others.
#[test]
fn the_confirm_control_is_shut_until_every_condition_is_met() {
    let mut d = plain_password_dialog();
    assert!(!d.ready_to_confirm(), "a blank form confirms nothing");

    // The owner password alone is not enough: its confirmation is still blank,
    // so the two copies differ.
    d.owner = "ownerpw".to_owned();
    assert!(
        !d.ready_to_confirm(),
        "a password typed once is not confirmed"
    );

    d.owner_again = "ownerpw".to_owned();
    assert!(
        d.ready_to_confirm(),
        "an owner password and its match is a complete form"
    );

    // ★ The two passwords must differ — the owner password ignores `/P`
    // entirely, so if it also opens the document the permission list below is
    // decoration.
    d.user = "ownerpw".to_owned();
    d.user_again = "ownerpw".to_owned();
    assert!(
        !d.ready_to_confirm(),
        "the two passwords must not be the same"
    );

    d.user = "userpw".to_owned();
    d.user_again = "userpw".to_owned();
    assert!(d.ready_to_confirm());

    // ★ And choosing to replace closes it again until the extra
    // acknowledgement is given.
    d.choose_destination(Destination::ReplaceOriginal);
    assert!(
        !d.ready_to_confirm(),
        "replacing the operator's file needs the extra acknowledgement"
    );
    d.overwrite_acknowledged = true;
    assert!(d.ready_to_confirm());
}

/// ★★ **The greyed confirm names WHICH condition is outstanding.**
///
/// `OPERATOR_REQUESTS.md` O77's sweep found seven greyed controls with no hover
/// explanation. Several different conditions gate this one button and they
/// appear at different times, so *"fill in the form"* would be vague exactly
/// when it matters.
#[test]
fn the_greyed_confirm_names_the_outstanding_condition() {
    let mut d = plain_password_dialog();
    let g = d.gates();
    assert!(g.owner_missing);
    assert!(
        !g.current_owner_missing,
        "a plain document authorises nothing"
    );
    let line = t::confirm_disabled(
        g.current_owner_missing,
        g.owner_missing,
        g.mismatch,
        g.same,
        g.overwrite_unacknowledged,
    );
    assert!(line.contains("owner password cannot be blank"), "{line}");
    assert!(
        !line.contains("current owner"),
        "a condition that was never on screen is not owed: {line}"
    );

    // The replace acknowledgement, when and only when it applies.
    d.owner = "ownerpw".to_owned();
    d.owner_again = "ownerpw".to_owned();
    d.choose_destination(Destination::ReplaceOriginal);
    let g = d.gates();
    assert!(g.overwrite_unacknowledged);
    let line = t::confirm_disabled(false, false, false, false, g.overwrite_unacknowledged);
    assert_eq!(line, t::overwrite_outstanding());
}

/// ★★★ **Changing the destination retires the acknowledgement.**
///
/// `crate::dialogs::redact::choose_destination`'s rule, and it stops the one
/// sequence that would otherwise leave a live button over a withdrawn consent:
/// tick, think better of it, choose *a new file*, change your mind, and arrive
/// back at *replace* with the button already enabled.
#[test]
fn changing_the_destination_retires_the_acknowledgement() {
    let mut d = plain_password_dialog();
    d.choose_destination(Destination::ReplaceOriginal);
    d.overwrite_acknowledged = true;
    d.choose_destination(Destination::NewFile);
    assert!(!d.overwrite_acknowledged, "the consent was withdrawn");
    d.choose_destination(Destination::ReplaceOriginal);
    assert!(
        !d.overwrite_acknowledged,
        "returning to replace does not restore a consent that was withdrawn"
    );
    // ★ Re-selecting the SAME destination is not a change and must not retire
    // a tick the operator has just given.
    d.overwrite_acknowledged = true;
    d.choose_destination(Destination::ReplaceOriginal);
    assert!(d.overwrite_acknowledged);
}

/// **Changing the job re-seeds the permission ticks from the file.**
///
/// Without it, editing the list under one job and then selecting another leaves
/// the boxes wherever the previous editing put them — which on
/// [`Job::ChangePassword`] would be a permission set the operator chose for a
/// job that does not write one.
#[test]
fn changing_the_job_re_seeds_the_ticks_from_the_file() {
    let mut d = plain_password_dialog();
    for (_, on) in &mut d.ticks {
        *on = false;
    }
    d.choose_job(Job::SetPermissions);
    assert_eq!(
        d.ticks,
        d.standing.initial_ticks(),
        "the seed is always the document"
    );
}

/// ★★★ **The accessibility bit is never offered as a choice.**
///
/// `pdfcer-core` sets bit 10 on every file it writes (rule W19), so a tick-box
/// the operator could clear would come back ticked in the result. The row is a
/// statement instead, and [`ProtectDialog::granted`] forces the bit in
/// regardless of what the tick says — so the list passed to the engine is what
/// the written file will actually say.
///
/// ★ The second half is the one that would rot silently: a future edit that
/// made the checkbox editable would still pass the first assertion.
#[test]
fn the_accessibility_bit_is_never_offered_as_a_choice() {
    let mut d = plain_password_dialog();
    assert!(always_granted(PermissionBit::AccessibilityExtract));
    for (_, on) in &mut d.ticks {
        *on = false;
    }
    let granted = d.granted();
    assert_eq!(
        granted,
        vec![PermissionBit::AccessibilityExtract],
        "every box cleared, and the one the engine grants anyway is still in the list"
    );
    assert!(
        d.ticks
            .iter()
            .any(|(bit, _)| *bit == PermissionBit::AccessibilityExtract),
        "the bit is still REPORTED — it is drawn as a statement, not omitted"
    );
}

/// ★★★ **A `{:?}` on this dialog does not print a password.**
///
/// `crate::secret`'s header names the exact cost: *"a `{:?}` on an action
/// carrying a password writes it into the trace file `tools/ui-verify` keeps as
/// evidence."* Five fields here hold one, as `String` rather than `Secret`,
/// because `egui::TextEdit` binds to a `String` — so the type cannot do the
/// protecting and [`ProtectDialog`]'s hand-written `Debug` must.
///
/// A derived `Debug` would pass every other test in this file.
#[test]
fn the_debug_impl_does_not_carry_a_password() {
    let mut d = plain_password_dialog();
    d.current_owner = "CURRENTOWNERSECRET".to_owned();
    d.user = "USERSECRETVALUE".to_owned();
    d.user_again = "USERSECRETVALUE".to_owned();
    d.owner = "OWNERSECRETVALUE".to_owned();
    d.owner_again = "OWNERSECRETVALUE".to_owned();
    let printed = format!("{d:?}");
    for secret in ["CURRENTOWNERSECRET", "USERSECRETVALUE", "OWNERSECRETVALUE"] {
        assert!(
            !printed.contains(secret),
            "a password reached a Debug string: {printed}"
        );
    }
    // …and what IS printed is the length, which is what diagnosing "my password
    // is not accepted" actually needs.
    assert!(printed.contains("user_len: 15"), "{printed}");
    assert!(printed.contains("current_owner_len: 18"), "{printed}");
}

/// **Every failure the model can report has its own sentence.**
///
/// The exhaustive `match` in [`failure_line`] is what makes a new
/// [`PrepareFailure`] variant a compile error rather than a silent
/// fall-through; this asserts the sentences are actually different, which the
/// compiler cannot.
#[test]
fn each_failure_says_something_different() {
    use pdfcer_core::crypto::AuthKind;
    let lines = [
        failure_line(&PrepareFailure::Refused(Refusal::NotEncrypted)),
        failure_line(&PrepareFailure::Refused(Refusal::NoFile)),
        failure_line(&PrepareFailure::Reopen("bad password".to_owned())),
        failure_line(&PrepareFailure::NotOwner {
            opened_as: AuthKind::User,
        }),
        failure_line(&PrepareFailure::Engine(
            crate::protect::EngineRefusal::AlreadyEncrypted,
        )),
    ];
    for (i, a) in lines.iter().enumerate() {
        for b in lines.iter().skip(i + 1) {
            assert_ne!(a, b, "two failures share one sentence");
        }
    }
    // ★ The one that turns a dead end into a next step: it names which password
    // DID work, so an operator told "that is the user password" goes and finds
    // the other one instead of re-typing the one they have.
    assert!(lines[3].contains("user password"), "{}", lines[3]);
}
