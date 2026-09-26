//! Tests for `crate::app::status::anomalies`, kept in the gui because they reach gui modules.

use crate::app::status::anomalies::*;
use pdfcer_core::document::LoadAnomaly;
use pdfcer_core::object::{Name, ObjId, Object};

/// The file behind engine decision 145, as far as this shell can build it:
/// a doubled `/PageMode` in object 57, and a stream one object later whose
/// `/Length` was unusable.
fn the_operators_file() -> Vec<LoadAnomaly> {
    vec![
        LoadAnomaly::DuplicateDictKey {
            object: Some(ObjId::new(57, 0)),
            key: b"PageMode".to_vec(),
            kept: Object::Name(Name(b"UseOutlines".to_vec())),
            discarded: Object::Name(Name(b"UseOC".to_vec())),
        },
        LoadAnomaly::StreamLengthRecovered {
            object: ObjId::new(58, 0),
        },
    ]
}

/// A file with no contradiction produces no line at all.
///
/// The control the engine's notice names. The assertion is `None`, not an
/// empty string: an empty status line would still allocate a row.
#[test]
fn a_clean_file_discloses_nothing() {
    assert!(census(&[]).is_clean());
    assert_eq!(status_line(&[]), None);
    assert!(rows(&[]).is_empty());
}

/// The operator's own file reaches the bar as one line naming both defects.
#[test]
fn the_operators_file_reaches_the_status_line() {
    let line = status_line(&the_operators_file()).expect("two anomalies is not clean");
    assert!(line.contains("1 duplicate dictionary key"), "{line}");
    assert!(
        line.contains("1 stream whose declared length was wrong"),
        "{line}"
    );
    assert!(!line.contains('\n'), "the bar gets one line: {line}");
}

/// The census is ordered by consequence, and the order is pinned whole.
#[test]
fn the_census_is_ordered_by_consequence() {
    let anomalies = vec![
        LoadAnomaly::MissingEndobjRecovered {
            object: ObjId::new(1, 0),
        },
        LoadAnomaly::StreamLengthRecovered {
            object: ObjId::new(2, 0),
        },
        LoadAnomaly::DuplicateDictKey {
            object: None,
            key: b"Size".to_vec(),
            kept: Object::Integer(1),
            discarded: Object::Integer(2),
        },
        LoadAnomaly::ObjectUnreadable {
            object: ObjId::new(3, 0),
            reason: "parse error".to_owned(),
        },
    ];
    assert_eq!(
        clauses(&anomalies),
        vec![
            "1 object it could not read".to_owned(),
            "1 duplicate dictionary key".to_owned(),
            "1 stream whose declared length was wrong".to_owned(),
            "1 object with no end marker".to_owned(),
        ],
        "most consequential first; see `clauses`' ordering note"
    );
}

/// Every anomaly gets exactly one row, and the rows keep the engine's order.
///
/// The order half is the point: the two defects of one damaged region of the
/// file must stay adjacent.
#[test]
fn every_anomaly_gets_one_row_in_file_order() {
    let anomalies = the_operators_file();
    let rows = rows(&anomalies);
    assert_eq!(rows.len(), anomalies.len());
    assert!(rows[0].contains("/PageMode"), "{rows:?}");
    assert!(rows[1].contains("Object 58 0"), "{rows:?}");
}

/// Both values of a duplicate key survive all the way to the panel row.
#[test]
fn the_panel_row_shows_what_pdfcer_chose_between() {
    let rows = rows(&the_operators_file());
    let row = rows.first().expect("the duplicate key is first");
    assert!(row.contains("/UseOutlines"), "kept value missing: {row}");
    assert!(row.contains("/UseOC"), "discarded value missing: {row}");
}

/// An unreadable object's row carries the object id and the engine's reason.
#[test]
fn an_unreadable_object_names_itself_and_why() {
    let rows = rows(&[LoadAnomaly::ObjectUnreadable {
        object: ObjId::new(9, 0),
        reason: "parse error at byte 43992".to_owned(),
    }]);
    let row = rows.first().expect("one anomaly, one row");
    assert!(row.contains("Object 9 0"), "{row}");
    assert!(row.contains("parse error at byte 43992"), "{row}");
}

/// **A REAL FILE, THROUGH THE REAL LOADER** — the one test here that is
/// not this module talking to itself.
#[test]
fn the_contradicting_fixture_produces_exactly_one_anomaly_through_the_engine() {
    let doc = crate::app::state::open_local_fixture(crate::app::state::CONTRADICTS_ITSELF);
    let document = doc.session.document();
    let anomalies = document.load_anomalies();
    assert_eq!(
        anomalies.len(),
        1,
        "★ the fixture is authored for exactly one contradiction; the engine reported {}: {anomalies:?}",
        anomalies.len()
    );
    assert!(
        matches!(anomalies[0], LoadAnomaly::DuplicateDictKey { .. }),
        "★ the one anomaly should be the doubled /PageMode: {:?}",
        anomalies[0]
    );
    assert!(!census(anomalies).is_clean());

    let line = status_line(anomalies).expect("one anomaly is not clean");
    assert!(
        line.contains("1 duplicate dictionary key"),
        "the census clause should count exactly one: {line}"
    );

    let rows = rows(anomalies);
    let row = rows.first().expect("one anomaly, one row");
    assert!(row.contains("PageMode"), "the key is missing: {row}");
    assert!(
        row.contains("/UseOutlines"),
        "the KEPT value is missing (KeepLast should keep the second): {row}"
    );
    assert!(
        row.contains("/UseOC"),
        "the DISCARDED value is missing, which is the half that makes an override offerable: {row}"
    );

    assert!(
        document.recovery().is_none(),
        "★ this fixture's xref is sound by construction. A recovery report here means the file was rebuilt by scan, and a check reading the disclosure would be reading the wrong one."
    );
}

/// **The control: the fixtures a driven run uses as "a file that does NOT
/// contradict itself" really do not.**
#[test]
fn the_control_fixtures_a_driven_run_uses_are_genuinely_clean() {
    // The names are spelled literally rather than through
    // `crate::app::state::FOUR_PAGES`, which resolves against the ENGINE's
    // read-only corpus (`pageops/four-pages.pdf`) and not this repository's
    // `fixtures/`. `open_local_fixture` takes a path relative to THIS
    // repository, and the driven check launches the binary on the same two
    // paths, so the two instruments have to be naming the same bytes.
    for name in ["a1-titleblock.pdf", "four-pages.pdf"] {
        let doc = crate::app::state::open_local_fixture(name);
        let anomalies = doc.session.document().load_anomalies();
        assert!(
            anomalies.is_empty(),
            "{name} is a CONTROL for the driven disclosure check and it reported {} load \
             anomaly/anomalies: {anomalies:?}. Either pick a different control or accept \
             that this file is no longer one - do not widen the check.",
            anomalies.len()
        );
        assert!(
            census(anomalies).is_clean(),
            "{name} has no anomalies and the census still does not call it clean"
        );
        assert!(
            status_line(anomalies).is_none(),
            "{name} is clean and still produced a status line"
        );
        assert!(
            rows(anomalies).is_empty(),
            "{name} is clean and still produced Document-properties rows"
        );
    }
}
