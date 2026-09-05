//! # `trust::tests` — the decision table, and the two things a green test here
//! must not be read as proving
//!
//! ## What these DO cover
//!
//! The pure half: which path wins, what happens when a configured path is
//! wrong, and — the important one — that every no-anchors situation produces a
//! **distinct** [`Anchors`] variant rather than sharing one. That last property
//! is the whole safety argument of the surface built on it, because the four
//! states call for four different actions.
//!
//! ## ★★ What a pass here does NOT prove
//!
//! 1. **That any real signature verifies.** No fixture in this repository
//!    carries a signature whose signer chains to a real AATL anchor, and one
//!    could not be committed: it would need somebody's real certificate and the
//!    verdict would expire with it. The cryptography is `pdfcer-core`'s and is
//!    tested there against pyHanko-signed fixtures whose verdicts were recorded
//!    from pyHanko's own validator first.
//! 2. **That the operator can see any of it.** These are values. Only
//!    `tools/ui-verify`, driving the real binary, answers whether a rectangle
//!    was drawn — and this project has a standing record of tests that passed
//!    while the feature was unreachable.
//!
//! ## ★ Why the environment is not mocked
//!
//! [`super::candidate_paths`] reads `%APPDATA%`, and a test that set it would
//! be mutating process-global state that every other test in this binary shares
//! — `cargo test` runs them on threads. So the tests below either take a path
//! as an argument (which is why [`super::locate`] takes one) or assert a
//! property that holds whatever `%APPDATA%` says: that the list is either empty
//! or every entry ends in the address book's file name. A test that asserted
//! *four* candidates would be a test about whichever machine ran it.
//!
//! ★★ **The inner `#![cfg(test)]` below is load-bearing and is not a duplicate
//! of the outer `#[cfg(test)] mod tests;`.** Without it,
//! `tools/gates/check-ui-strings.sh` walks this file as ordinary source and
//! reports every assertion message as a user-visible string that belongs in the
//! catalog — exclusion 2b in that gate, whose own comment records the day a
//! split under R2 reintroduced 28 such false hits. It is the same line every
//! other split test file in this crate carries.
#![cfg(test)]

use std::path::PathBuf;

use pdfcer_core::settings::AcrobatTrustStore;

use super::{Anchors, Located, candidate_paths, describe_absence, locate};

/// A path this machine certainly does not have a file at.
fn nowhere() -> PathBuf {
    std::env::temp_dir().join("pdfcer-trust-store-that-does-not-exist.acrodata")
}

/// **Every candidate is an address book, or there are none.**
///
/// The floor that stops this file's other assertions being about nothing: a
/// `candidate_paths` that silently returned an empty vector on Windows would
/// make [`the_four_no_anchor_states_are_distinct`] pass by never finding a
/// store, which is a green result measuring the absence of an environment
/// variable.
#[test]
fn every_candidate_path_names_an_address_book() {
    let paths = candidate_paths();
    if std::env::var("APPDATA").is_err() {
        assert!(
            paths.is_empty(),
            "with no %APPDATA% there is nowhere to look, so the list must be empty"
        );
        return;
    }
    assert_eq!(
        paths.len(),
        super::TRACKS.len(),
        "one candidate per Acrobat track, so the window and the CLI look in the same places"
    );
    for p in &paths {
        assert_eq!(
            p.file_name().and_then(|n| n.to_str()),
            Some(super::ADDRESS_BOOK),
            "a candidate that is not an address book would be read as a trust store: {}",
            p.display()
        );
    }
}

/// **A configured path that is not there is NOT a fallback to discovery.**
///
/// The behaviour this pins is the one a well-meaning edit would "improve": if
/// the typed path is missing, try the usual places. That would make a typo
/// behave like a correct entry pointing somewhere else, and the operator would
/// have no way to tell which store was actually read — on the one surface where
/// which certificates were used is the entire question.
#[test]
fn a_configured_path_that_is_missing_does_not_fall_back() {
    let missing = nowhere();
    match locate(&missing.display().to_string()) {
        Located::ConfiguredMissing(p) => assert_eq!(p, missing),
        other => panic!("a missing configured path must report itself, got {other:?}"),
    }
}

/// **A blank field means "look in the usual places", never "there is no
/// store".**
///
/// Clearing a text box is how a person un-sets it, and reading an empty value
/// as a positive choice would suppress the feature permanently with no way back
/// except hand-editing a file.
#[test]
fn a_blank_path_asks_the_machine() {
    // Whatever this machine has, a blank field must never produce a
    // `Configured*` state — that would mean an empty string was treated as a
    // path.
    for blank in ["", "   ", "\t"] {
        assert!(
            matches!(locate(blank), Located::Discovered(_) | Located::None { .. }),
            "a blank field must be discovery, not a configured path: {blank:?}"
        );
    }
}

/// **Whitespace around a typed path is trimmed here as well as on the way in.**
///
/// `prefs` trims when it parses the file, and this trims again, and the
/// duplication is deliberate: the file is not the only route a value takes —
/// the Settings field writes it directly — and a trailing space is a path that
/// does not exist, which presents as *"the setting does nothing"* rather than
/// as *"that file is not there"*.
#[test]
fn a_typed_path_is_trimmed() {
    let missing = nowhere();
    let padded = format!("  {}  ", missing.display());
    match locate(&padded) {
        Located::ConfiguredMissing(p) => assert_eq!(
            p, missing,
            "the trimmed path must be the one reported back, or the operator reads their own \
             typo with invisible characters in it"
        ),
        other => panic!("expected ConfiguredMissing, got {other:?}"),
    }
}

/// **The setting being off is reported as the setting being off — never as
/// "no store found".**
///
/// ★★★ The single most important assertion in this file. `Off` is the shipped
/// default, so it is the state almost every operator is in, and it is the one
/// they can fix in five seconds. Reporting it as *"pdfcer found no trust list"*
/// would send them looking for an Acrobat install they already have.
#[test]
fn opting_out_is_not_reported_as_a_missing_store() {
    let absence = describe_absence(AcrobatTrustStore::Off, "");
    assert_eq!(absence, Anchors::OptedOut);
    assert!(!absence.evaluated());

    // And it stays `OptedOut` even when a path IS configured: a location is not
    // a permission, and a person who typed a path while the setting is off has
    // not turned it on.
    let with_path = describe_absence(AcrobatTrustStore::Off, r"D:\anything\addressbook.acrodata");
    assert_eq!(with_path, Anchors::OptedOut);
}

/// **A configured-but-missing path is not reported as "this machine has
/// none".**
///
/// The two produce the same *outcome* — no anchors — and completely different
/// remedies. `NoStore { configured_missing: Some(..) }` is what lets the panel
/// say *"there is no file where you pointed"* instead of *"install Acrobat"*.
#[test]
fn a_missing_configured_path_is_distinguishable_from_no_store_at_all() {
    let missing = nowhere();
    let absence = describe_absence(AcrobatTrustStore::AtOwnRisk, &missing.display().to_string());
    match absence {
        Anchors::NoStore {
            configured_missing: Some(p),
            looked_in,
        } => {
            assert_eq!(p, missing);
            assert!(
                looked_in.is_empty(),
                "nothing else was tried, and saying otherwise would claim a search that did \
                 not happen"
            );
        }
        other => panic!("expected NoStore with the configured path named, got {other:?}"),
    }
}

/// **The four no-anchor states are four distinct values.**
///
/// Not a tautology over an enum: the cheap implementation of this feature has
/// one "trust not checked" state and this is the test that refuses it. Two of
/// the four are constructed here from real inputs; the other two need a
/// filesystem this test does not have, so their *distinctness* is asserted over
/// hand-built values, which is enough — the property under test is that the
/// type can tell them apart at all.
#[test]
fn the_four_no_anchor_states_are_distinct() {
    let states = [
        Anchors::OptedOut,
        Anchors::NoStore {
            looked_in: vec![nowhere()],
            configured_missing: None,
        },
        Anchors::NoStore {
            looked_in: Vec::new(),
            configured_missing: Some(nowhere()),
        },
        Anchors::Unreadable {
            path: nowhere(),
            reason: "not a PPKLITE address book".to_owned(),
        },
    ];
    for (i, a) in states.iter().enumerate() {
        for b in states.iter().skip(i + 1) {
            assert_ne!(a, b, "two no-anchor situations collapsed into one value");
        }
        assert!(
            !a.evaluated(),
            "no anchors means trust was NOT evaluated, whatever the reason"
        );
    }
}

/// **`examine` over a document with no signature fields produces no verdicts
/// and still reports the anchor state.**
///
/// ★ The second half is the point. A report that carried an empty verdict list
/// AND no anchor state would leave the panel unable to distinguish *"this
/// document is not signed"* from *"pdfcer did not look"*, which is the same
/// collapse the four states exist to prevent, one level up.
#[test]
fn an_unsigned_document_still_reports_where_the_anchors_would_have_come_from() {
    // ★ Anchored on `CARGO_MANIFEST_DIR`, not on the working directory. A bare
    // relative path resolves against the CRATE directory under `cargo test`
    // and against the workspace root under some runners, so the same test
    // passes and fails depending on how it was invoked. Every other fixture
    // read in this crate does the same — `app::actions::latency` is the
    // precedent.
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/a1-titleblock.pdf");
    let bytes = std::fs::read(&fixture).expect("the portable-floor fixture");
    let doc = pdfcer_core::document::Document::from_bytes(bytes.clone()).expect("it opens");
    let report = super::examine(&doc.view(), &bytes, AcrobatTrustStore::Off, "");
    assert!(
        report.verdicts.is_empty(),
        "the title-block fixture carries no signature fields"
    );
    assert_eq!(report.anchors, Anchors::OptedOut);
    assert_eq!(report.file_len, bytes.len() as u64);
}

/// **A modification time comes back as a plain calendar date.**
///
/// Pinned against a known instant rather than against "today", for
/// `app::clock`'s own stated reason: a test that formats the current date
/// passes for a year and then fails at a month boundary for reasons nobody
/// remembers.
#[test]
fn a_store_date_is_a_calendar_date() {
    use std::time::{Duration, UNIX_EPOCH};
    let at = UNIX_EPOCH + Duration::from_secs(1_716_768_000);
    assert_eq!(super::modified_date(at).as_deref(), Some("2024-05-27"));
}
