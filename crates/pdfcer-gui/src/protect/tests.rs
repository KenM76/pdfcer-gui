//! Design and rationale: `docs/modules/pdfcer-gui/protect/tests.md`.
#![cfg(test)]
//! # `protect::tests` — the properties this surface must hold, asserted
//! headlessly
//!
//! Split into its own file under **R2** and on `crate::viewer`'s standing rule:
//! *a rule that can only be exercised by driving a window is a rule that gets
//! asserted once, by hand, and then drifts.* Every assertion below is about the
//! operator's file — what the document says today, what a change preserves, and
//! what the engine refuses — and none of them needs a `Ui`.
//!
//! ## What each test is protecting, and what it would look like if it failed
//!
//! | test | the defect it catches |
//! |---|---|
//! | [`the_permission_model_round_trips`] | the tick-boxes and the file disagree — a drawing written as print-only that is not |
//! | [`a_document_that_forbids_printing_does_not_open_with_everything_ticked`] | the build brief's own sentence, inverted: the dialog lying before it is touched |
//! | [`a_signed_document_is_refused_before_the_form_is_drawn`] | O119 disclosure 2 — a button that fails on press |
//! | [`permissions_on_an_unprotected_document_is_refused_by_name`] | eight ticked boxes invented for a file that declares nothing |
//! | [`changing_the_password_keeps_what_the_document_allowed`] | a password change quietly un-restricting the drawing |
//! | [`removing_the_password_produces_a_file_that_opens_with_none`] | a "removed" protection that is still there |
//! | [`the_user_password_will_not_authorise_a_change`] | O119 disclosure 3 — the owner-only precondition not actually enforced |
//! | [`an_unstated_bit_is_carried_over_as_granted`] | an `/R` 2 author's silence turned into a prohibition they never wrote |
//! | [`the_suggested_name_is_never_the_source_file`] | a picker whose safe answer is to change the field |

use super::*;

use pdfcer_core::page_tree;
use pdfcer_core::writer::SaveOptions;

use crate::app::state::OpenDoc;

/// The passwords the tests use. Test data, and the fixture's own
/// `PROVENANCE.md` already publishes the same pair.
const OWNER: &[u8] = b"ownerpw"; // ui-text-exempt: test data, never displayed
const USER: &[u8] = b"userpw"; // ui-text-exempt: test data, never displayed

/// A plain, unencrypted document to work from.
fn plain() -> OpenDoc {
    crate::app::state::open_local_fixture("four-pages.pdf")
}

/// Build [`Passwords`] from three byte strings.
fn passwords(current_owner: &[u8], user: &[u8], owner: &[u8]) -> Passwords {
    Passwords {
        current_owner: Secret::new(String::from_utf8_lossy(current_owner).into_owned()),
        user: Secret::new(String::from_utf8_lossy(user).into_owned()),
        owner: Secret::new(String::from_utf8_lossy(owner).into_owned()),
    }
}

/// A scratch path in the system temp directory, unique per caller **and per
/// process**.
fn scratch(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("pdfcer-protect-{tag}-{}.pdf", std::process::id()));
    p
}

/// **Write an encrypted copy of the plain fixture to disk and open it as the
/// owner**, so the encrypted-branch tests have a real `OpenDoc` over a real
/// file — which is what [`prepare`] needs, because the mutating verbs re-open
/// the file rather than touching the open session (see the module header, §2).
fn encrypted_doc_on_disk(tag: &str, granted: &[PermissionBit]) -> OpenDoc {
    let source = plain();
    let mut settings = EncryptionSettings::new(USER.to_vec(), OWNER.to_vec());
    settings.permissions = granted.to_vec();
    let (bytes, _) = source
        .session
        .set_encryption(&settings, &SaveOptions::default())
        .expect("the plain fixture encrypts");
    let path = scratch(tag);
    std::fs::write(&path, &bytes).expect("the scratch file is writable");

    let document =
        Document::load_with_password(&path, Some(OWNER)).expect("the owner password opens it");
    let pages = page_tree::pages(&document).expect("a page tree");
    OpenDoc::new(path, EditSession::new(document), pages)
}

/// Read every bit out of a finished set of bytes.
fn grants_of(bytes: Vec<u8>, password: Option<&[u8]>) -> Vec<(PermissionBit, Option<bool>)> {
    let document = match password {
        Some(pw) => Document::from_bytes_with_password(bytes, Some(pw)),
        None => Document::from_bytes(bytes),
    }
    .expect("the written document reopens");
    let path = PathBuf::from("x.pdf"); // ui-text-exempt: a path that is never touched
    Standing::read(&EditSession::new(document), &path).grants
}

/// The state of one bit in a grants vector.
fn bit(grants: &[(PermissionBit, Option<bool>)], want: PermissionBit) -> Option<bool> {
    grants
        .iter()
        .find(|(b, _)| *b == want)
        .map(|(_, g)| *g)
        .expect("every bit is enumerated")
}

// ---------------------------------------------------------------------------

/// **What the operator ticks is what the file says.**
#[test]
fn the_permission_model_round_trips() {
    let doc = plain();
    let granted = [PermissionBit::Print, PermissionBit::AccessibilityExtract];
    let prepared = prepare(
        &doc,
        Job::SetPassword,
        &passwords(b"", USER, OWNER),
        &granted,
        true,
    )
    .expect("a plain document takes a password");
    assert_eq!(prepared.job(), Job::SetPassword);
    assert!(prepared.byte_len() > 0);

    let grants = grants_of(prepared.bytes.clone(), Some(USER));
    assert_eq!(bit(&grants, PermissionBit::Print), Some(true));
    assert_eq!(
        bit(&grants, PermissionBit::AccessibilityExtract),
        Some(true)
    );
    for denied in [
        PermissionBit::Copy,
        PermissionBit::ModifyContents,
        PermissionBit::Annotate,
        PermissionBit::FillForms,
        PermissionBit::Assemble,
        PermissionBit::PrintHighQuality,
    ] {
        assert_eq!(
            bit(&grants, denied),
            Some(false),
            "{denied:?} was not granted and must come back refused, not permitted"
        );
    }
}

/// **The build brief's own sentence, asserted.**
#[test]
fn a_document_that_forbids_printing_does_not_open_with_everything_ticked() {
    // A plaintext document declines nothing — all eight ticked is the read-back.
    let open = plain();
    let standing = Standing::read(&open.session, &open.path);
    assert!(!standing.encrypted);
    assert!(
        standing.initial_ticks().iter().all(|(_, on)| *on),
        "an unprotected document declines nothing, so every box is ticked"
    );
    assert!(standing.refusal(Task::Password).is_none());

    // The same read over a document whose author forbade printing.
    let doc = encrypted_doc_on_disk("no-print", &[PermissionBit::Copy]);
    let standing = Standing::read(&doc.session, &doc.path);
    assert!(standing.encrypted);
    assert_eq!(standing.auth, Some(AuthKind::Owner));
    assert_eq!(standing.cipher, Some(Cipher::Aes256));
    assert_eq!(standing.revision, 6);
    assert!(
        !standing.has_unstated_bits(),
        "every bit is meaningful at /R 6, so none is 'not stated'"
    );

    let ticks = standing.initial_ticks();
    let print_on = ticks
        .iter()
        .find(|(b, _)| *b == PermissionBit::Print)
        .map(|(_, on)| *on)
        .expect("Print is enumerated");
    assert!(
        !print_on,
        "the document forbids printing, so the Print box must open unticked"
    );
    let copy_on = ticks
        .iter()
        .find(|(b, _)| *b == PermissionBit::Copy)
        .map(|(_, on)| *on)
        .expect("Copy is enumerated");
    assert!(
        copy_on,
        "the document permits copying, so that box is ticked"
    );

    let _ = std::fs::remove_file(&doc.path);
}

/// **O119's second disclosure: a signed document is refused, and it is
/// refused before anything is offered.**
#[test]
fn a_signed_document_is_refused_before_the_form_is_drawn() {
    let doc = crate::app::state::open_local_fixture("signed-two-pages.pdf");
    let standing = Standing::read(&doc.session, &doc.path);
    assert!(
        standing.signatures > 0,
        "the fixture is the signed one, or this test asserts nothing"
    );
    for task in [Task::Password, Task::Permissions] {
        assert_eq!(
            standing.refusal(task),
            Some(Refusal::Signed {
                signatures: standing.signatures
            }),
            "{task:?} must be refused on a signed document"
        );
    }
}

/// **Permissions… on an unprotected document says so, rather than inventing a
/// declaration.**
#[test]
fn permissions_on_an_unprotected_document_is_refused_by_name() {
    let doc = plain();
    let standing = Standing::read(&doc.session, &doc.path);
    assert_eq!(
        standing.refusal(Task::Permissions),
        Some(Refusal::NotEncrypted)
    );
    // …and the other control is offered on the same document, which is what
    // makes the refusal a statement about permissions rather than about the file.
    assert_eq!(standing.refusal(Task::Password), None);
    assert_eq!(standing.jobs(Task::Password), vec![Job::SetPassword]);
    assert!(standing.jobs(Task::Permissions).is_empty());
}

/// **Changing the password does not quietly unlock the drawing.**
#[test]
fn changing_the_password_keeps_what_the_document_allowed() {
    let doc = encrypted_doc_on_disk("change-pw", &[PermissionBit::Print]);
    let standing = Standing::read(&doc.session, &doc.path);
    let carried = standing.preserved_grants();
    assert_eq!(
        carried,
        vec![PermissionBit::Print, PermissionBit::AccessibilityExtract],
        "the document permits printing, and accessibility extraction because \
         the engine's W19 rule sets bit 10 on every write regardless"
    );

    let prepared = prepare(
        &doc,
        Job::ChangePassword,
        &passwords(OWNER, b"newuser", b"newowner"),
        &carried,
        true,
    )
    .expect("the owner may re-key");
    assert_eq!(prepared.job(), Job::ChangePassword);

    let bytes = prepared.bytes.clone();
    assert!(
        Document::from_bytes_with_password(bytes.clone(), Some(b"newuser")).is_ok(),
        "the new user password opens the re-keyed document"
    );
    assert!(
        Document::from_bytes_with_password(bytes.clone(), Some(USER)).is_err(),
        "the old user password does not"
    );

    let grants = grants_of(bytes, Some(b"newuser"));
    assert_eq!(bit(&grants, PermissionBit::Print), Some(true));
    assert_eq!(
        bit(&grants, PermissionBit::Copy),
        Some(false),
        "changing a password must not grant what the document refused"
    );

    let _ = std::fs::remove_file(&doc.path);
}

/// **Removing the protection produces a file that opens with no password at
/// all.**
#[test]
fn removing_the_password_produces_a_file_that_opens_with_none() {
    let doc = encrypted_doc_on_disk("remove-pw", &[PermissionBit::Print]);
    let prepared = prepare(
        &doc,
        Job::RemovePassword,
        &passwords(OWNER, b"", b""),
        &[],
        true,
    )
    .expect("the owner may remove the protection");
    assert_eq!(prepared.job(), Job::RemovePassword);

    let reopened =
        Document::from_bytes(prepared.bytes.clone()).expect("plaintext opens with no password");
    assert!(
        reopened.encryption().is_none(),
        "the /Encrypt dictionary is gone, not merely un-prompted"
    );

    // And the OPEN document is untouched — the whole of §2. The session the
    // operator is still looking at must still report itself as encrypted, or
    // its next incremental save would append plaintext to an encrypted base.
    let after = Standing::read(&doc.session, &doc.path);
    assert!(
        after.encrypted,
        "the open session must be untouched by a protection change"
    );

    let _ = std::fs::remove_file(&doc.path);
}

/// **O119's third disclosure, enforced rather than merely printed: the
/// USER password will not authorise a change.**
#[test]
fn the_user_password_will_not_authorise_a_change() {
    let doc = encrypted_doc_on_disk("not-owner", &[PermissionBit::Print]);
    let failure = prepare(
        &doc,
        Job::RemovePassword,
        &passwords(USER, b"", b""),
        &[],
        true,
    )
    .expect_err("the user password is not enough");
    match failure {
        PrepareFailure::NotOwner { opened_as } => assert_eq!(opened_as, AuthKind::User),
        other => panic!("expected NotOwner, got {other:?}"),
    }

    // A password that opens nothing at all is a different failure, and it says
    // so — `crate::dialogs::password`'s rule about not sending an operator to
    // re-check a password that was correct.
    let failure = prepare(
        &doc,
        Job::RemovePassword,
        &passwords(b"neither", b"", b""),
        &[],
        true,
    )
    .expect_err("a wrong password opens nothing");
    assert!(matches!(failure, PrepareFailure::Reopen(_)));

    let _ = std::fs::remove_file(&doc.path);
}

/// **A bit the document's revision has no opinion about is carried over as
/// GRANTED, not as refused.**
#[test]
fn an_unstated_bit_is_carried_over_as_granted() {
    let standing = Standing {
        encrypted: true,
        cipher: Some(Cipher::Rc4),
        auth: Some(AuthKind::Owner),
        revision: 2,
        grants: vec![
            (PermissionBit::Print, Some(true)),
            (PermissionBit::Copy, Some(false)),
            (PermissionBit::FillForms, None),
            (PermissionBit::Assemble, None),
        ],
        signatures: 0,
        on_disk: true,
    };
    assert!(standing.has_unstated_bits());
    assert_eq!(
        standing.preserved_grants(),
        vec![
            PermissionBit::Print,
            PermissionBit::FillForms,
            PermissionBit::Assemble
        ],
        "silence is carried over as permission, refusal as refusal"
    );
    let ticks = standing.initial_ticks();
    assert_eq!(
        ticks,
        vec![
            (PermissionBit::Print, true),
            (PermissionBit::Copy, false),
            (PermissionBit::FillForms, true),
            (PermissionBit::Assemble, true),
        ]
    );
}

/// **The suggested name is never the file that was opened, and it names the
/// right outcome.**
#[test]
fn the_suggested_name_is_never_the_source_file() {
    let source = PathBuf::from("D:\\jobs\\4471\\Sheet 1.pdf");
    for job in [
        Job::SetPassword,
        Job::ChangePassword,
        Job::SetPermissions,
        Job::RemovePassword,
    ] {
        let suggested = suggested_path(&source, job);
        assert_ne!(
            suggested, source,
            "{job:?} must not suggest the source file"
        );
        assert_eq!(
            suggested.parent(),
            source.parent(),
            "the copy lands beside the original, where the operator will look"
        );
        assert!(
            suggested
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("pdf")),
            "the suggestion is always a usable PDF name"
        );
    }
    assert_eq!(
        suggested_path(&source, Job::RemovePassword)
            .file_name()
            .expect("a name")
            .to_string_lossy(),
        "Sheet 1-unprotected.pdf"
    );
    assert_eq!(
        suggested_path(&source, Job::SetPassword)
            .file_name()
            .expect("a name")
            .to_string_lossy(),
        "Sheet 1-protected.pdf"
    );
}

/// **The document's own facts decide which jobs exist**, so the radio group,
/// the confirm control's label and the engine call cannot disagree about what
/// is being done.
#[test]
fn the_offered_jobs_follow_the_documents_own_state() {
    let open = plain();
    let plain_standing = Standing::read(&open.session, &open.path);
    assert_eq!(
        plain_standing.jobs(Task::Password),
        vec![Job::SetPassword],
        "a document with no password can only be given one"
    );

    let doc = encrypted_doc_on_disk("jobs", &[PermissionBit::Print]);
    let standing = Standing::read(&doc.session, &doc.path);
    assert_eq!(
        standing.jobs(Task::Password),
        vec![Job::ChangePassword, Job::RemovePassword],
        "a protected document can be re-keyed or unprotected, never re-protected"
    );
    assert_eq!(standing.jobs(Task::Permissions), vec![Job::SetPermissions]);

    // The per-job properties the window reads, asserted rather than trusted:
    // only the first job needs no current owner password, and only removal
    // asks for no new ones.
    assert!(!Job::SetPassword.needs_current_owner());
    assert!(Job::ChangePassword.needs_current_owner());
    assert!(Job::RemovePassword.needs_current_owner());
    assert!(Job::SetPermissions.needs_current_owner());
    assert!(!Job::RemovePassword.sets_new_passwords());
    assert!(Job::SetPassword.edits_permissions());
    assert!(Job::SetPermissions.edits_permissions());
    assert!(!Job::ChangePassword.edits_permissions());

    let _ = std::fs::remove_file(&doc.path);
}

/// **The write is atomic and it lands where it was told.**
#[test]
fn the_write_lands_and_leaves_no_temporary_behind() {
    let doc = plain();
    let prepared = prepare(
        &doc,
        Job::SetPassword,
        &passwords(b"", USER, OWNER),
        &PermissionBit::all(),
        true,
    )
    .expect("a plain document takes a password");
    let target = scratch("written");
    let written = prepared
        .write_to(&target)
        .expect("the scratch path is writable");
    assert_eq!(written, prepared.byte_len());
    assert!(target.is_file());
    assert!(
        !target.with_extension("pdfcer-tmp").exists(),
        "the temporary is renamed, never left behind"
    );
    assert!(
        Document::from_bytes_with_password(std::fs::read(&target).expect("readable"), Some(OWNER))
            .is_ok(),
        "what landed on disk is the protected document"
    );
    let _ = std::fs::remove_file(&target);
}
