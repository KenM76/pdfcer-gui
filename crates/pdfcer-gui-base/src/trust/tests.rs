//! # `trust::tests` — the decision table, and the two things a green test here
//! must not be read as proving
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/trust/tests.md`.
#![cfg(test)]

use std::path::PathBuf;

use pdfcer_core::settings::AcrobatTrustStore;

use super::{Anchors, Located, candidate_paths, describe_absence, locate};

/// A path this machine certainly does not have a file at.
fn nowhere() -> PathBuf {
    // temp-path-exempt: the whole point is a path with no file at it.
    std::env::temp_dir().join("pdfcer-trust-store-that-does-not-exist.acrodata")
}

/// **Every candidate is an address book, or there are none.**
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
#[test]
fn an_unsigned_document_still_reports_where_the_anchors_would_have_come_from() {
    // Anchored on `CARGO_MANIFEST_DIR`, not on the working directory. A bare
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
#[test]
fn a_store_date_is_a_calendar_date() {
    use std::time::{Duration, UNIX_EPOCH};
    let at = UNIX_EPOCH + Duration::from_secs(1_716_768_000);
    assert_eq!(super::modified_date(at).as_deref(), Some("2024-05-27"));
}
