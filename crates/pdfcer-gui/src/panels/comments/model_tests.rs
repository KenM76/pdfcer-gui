//! Tests for `crate::panels::comments::model`, kept in the gui because they reach gui modules.

use crate::panels::comments::model::*;
use crate::panels::objects::test_support::engine_fixture;
use pdfcer_core::document::Document;
use pdfcer_core::edit::EditSession;
use pdfcer_core::page_tree::Page;

/// Load a fixture as an `EditSession` plus its page vector — the same two
/// things `crate::app::state::OpenDoc` carries, so a test exercises the
/// panel's real inputs rather than a convenient stand-in.
fn open(rel: &str) -> (EditSession, Vec<Page>) {
    let path = engine_fixture(rel);
    let doc = Document::load(&path).expect("the fixture loads");
    let pages = pdfcer_core::page_tree::pages(&doc).expect("a page tree");
    (EditSession::new(doc), pages)
}

/// Collect a fixture through the same path the panel body uses.
fn listing(rel: &str) -> Listing {
    let (session, pages) = open(rel);
    let ce = ce_dimension_annots(&session);
    collect(&session.view(), &pages, &ce)
}

/// **A pop-up is excluded, and it is counted rather than dropped.**
#[test]
fn a_popup_is_excluded_and_counted() {
    let l = listing("annot/popup-not-painted.pdf");
    assert!(
        l.rows.is_empty(),
        "the only annotation in this fixture is a pop-up: {:?}",
        l.rows
    );
    assert_eq!(l.excluded.popups, 1);
    assert_eq!(l.excluded.total(), 1);
    assert!(
        !l.every_row_lacks_note_text(),
        "an EMPTY listing must not claim its rows lack note text — there \
         are no rows, and the panel says so with a different sentence"
    );
}

/// **A form field is excluded — the Forms panel owns those.**
///
/// `Annotation::is_widget` is the exact predicate, reused rather than
/// re-derived: *"a second one would be a divergence waiting to happen."*
#[test]
fn a_widget_is_excluded_and_counted() {
    let l = listing("forms/demo-form.pdf");
    assert!(
        l.excluded.widgets > 0,
        "a form fixture must carry widgets, or this test proves nothing"
    );
    for row in &l.rows {
        assert_ne!(
            row.subtype, "Widget",
            "a widget reached the comment list: {row:?}"
        );
    }
}

/// **A `/TrapNet` is excluded.**
#[test]
fn a_trapnet_is_excluded_and_counted() {
    let l = listing("annot/undeletable.pdf");
    assert_eq!(l.excluded.trap_nets, 1);
    for row in &l.rows {
        assert_ne!(row.subtype, "TrapNet", "prepress state reached the list");
    }
    // …and the squares beside it are listed, so the filter is a filter and
    // not a wall.
    assert!(
        l.rows.iter().any(|r| r.subtype == "Square"),
        "the ordinary markup on this page must survive the filter: {:?}",
        l.rows
    );
}

/// **ce dimensions are NOT excluded, and are named as what they are.**
#[test]
fn a_ce_dimension_is_listed_and_recognised() {
    let (session, pages) = open("dimension/linear-dim.pdf");
    let ce = ce_dimension_annots(&session);
    assert!(
        !ce.is_empty(),
        "this fixture exists to carry a ce dimension; if the sidecar is \
         unreadable the rest of this test proves nothing"
    );
    let l = collect(&session.view(), &pages, &ce);
    let dims: Vec<&CommentRow> = l.rows.iter().filter(|r| r.is_ce_dimension).collect();
    assert_eq!(
        dims.len(),
        ce.len(),
        "every sidecar record's annotation must appear as a row: {:?}",
        l.rows
    );
    for d in dims {
        assert_eq!(
            d.subtype, "Line",
            "a ce dimension IS a /Line annotation — that is the whole \
             reason it cannot be filtered out by subtype"
        );
    }
}

/// **…and a document with no sidecar calls nothing a ce dimension.**
#[test]
fn a_document_with_no_sidecar_has_no_ce_dimensions() {
    let (session, _pages) = open("annot/demo-annotated.pdf");
    assert!(ce_dimension_annots(&session).is_empty());
    let l = listing("annot/demo-annotated.pdf");
    assert!(l.rows.iter().all(|r| !r.is_ce_dimension));
}

/// **The order is page order, then `/Annots` order.**
#[test]
fn rows_are_in_page_order() {
    let l = listing("annot/thread.pdf");
    assert!(!l.rows.is_empty());
    let mut last = 0usize;
    for row in &l.rows {
        assert!(
            row.page_index >= last,
            "the list left page {last} and came back to {}: {row:?}",
            row.page_index
        );
        last = row.page_index;
    }
}

/// **A reply is recognised through `effective_reply_type`.**
#[test]
fn a_threaded_annotation_is_recognised_as_a_relation() {
    let l = listing("annot/thread.pdf");
    let related: Vec<&CommentRow> = l.rows.iter().filter(|r| r.relation.is_some()).collect();
    assert!(
        !related.is_empty(),
        "this fixture exists to carry /IRT links: {:?}",
        l.rows
    );
    // Every relation is one of the three modelled kinds; `Other` is
    // reachable only from an `/RT` name pdfcer has never seen.
    for r in related {
        assert!(matches!(
            r.relation,
            Some(Relation::Reply | Relation::GroupMember | Relation::Other)
        ));
    }
    // An annotation with no `/IRT` has no relation at all — `None` is a
    // fourth state, not a synonym for "not a reply".
    assert!(
        l.rows.iter().any(|r| r.relation.is_none()),
        "the thread's own root must have no relation: {:?}",
        l.rows
    );
}

/// **A suppressed annotation is listed and flagged, never dropped.**
#[test]
fn a_hidden_annotation_is_listed_and_flagged() {
    for fixture in ["annot/flags-hidden.pdf", "annot/flags-noview.pdf"] {
        let l = listing(fixture);
        assert!(
            l.rows.iter().any(|r| r.suppressed),
            "{fixture} exists to carry a suppressed annotation, and the \
             listing shows none: {:?}",
            l.rows
        );
    }
    // …and the flag DISCRIMINATES, which is the half that makes the
    // marker mean something. Asserted within one document rather than
    // across two: `demo-annotated.pdf` looks like an ordinary fixture but
    // carries a suppressed `/Stamp` of its own, so no fixture in this
    // corpus may be assumed to flag nothing. A document with a mix is in
    // any case the shape this panel exists to disclose, and a predicate
    // reduced to `true` would satisfy the sweep above and fail here.
    let mixed = listing("annot/demo-annotated.pdf");
    assert!(
        mixed.rows.iter().any(|r| r.suppressed),
        "this fixture carries a suppressed stamp: {:?}",
        mixed.rows
    );
    assert!(
        mixed.rows.iter().any(|r| !r.suppressed),
        "…and ordinary annotations beside it, or the flag is not \
         discriminating: {:?}",
        mixed.rows
    );
}

/// **`/Contents` on a subtype that displays no text is a description.**
#[test]
fn the_five_non_text_subtypes_are_descriptions_and_nothing_else_is() {
    for s in ["Link", "Movie", "Widget", "PrinterMark", "TrapNet"] {
        assert!(
            contents_is_description(s),
            "{s} displays no text of its own"
        );
    }
    for s in [
        "Text",
        "FreeText",
        "Square",
        "Circle",
        "Line",
        "Polygon",
        "PolyLine",
        "Ink",
        "Highlight",
        "Underline",
        "StrikeOut",
        "Squiggly",
        "Stamp",
        "Caret",
        "FileAttachment",
        "(no Subtype)",
    ] {
        assert!(
            !contents_is_description(s),
            "{s} displays its /Contents; calling it an accessibility \
             description would tell an operator somebody's comment was \
             written for a screen reader"
        );
    }
}

/// **The subtype a note edit is judged on is the ENGINE's, and this
/// panel's copy of the same vocabulary must not drift from it.**
#[test]
fn the_panels_subtype_vocabulary_is_the_one_the_disclosure_asks() {
    use crate::text::textannot::paints_its_note;

    assert!(
        paints_its_note("FreeText"),
        "a /FreeText paints its /Contents, so a note edit on one can leave a \
         foreign appearance behind and the status line must be able to say so"
    );
    for s in [
        "Text",
        "Square",
        "Circle",
        "Line",
        "Polygon",
        "PolyLine",
        "Ink",
        "Highlight",
        "Underline",
        "StrikeOut",
        "Squiggly",
        "Stamp",
        "Caret",
        "FileAttachment",
        "(no Subtype)",
    ] {
        assert!(
            !contents_is_description(s),
            "the list this test shares with its neighbour drifted: {s}"
        );
        assert!(
            !paints_its_note(s),
            "/{s} does not paint its /Contents, so a note edit on one is \
             complete -- disclosing about it teaches the operator to ignore \
             the sentence that matters"
        );
    }
}

/// **An absent `/Contents` is [`Note::Absent`], and that drives the
/// document-wide disclosure.**
#[test]
fn the_all_without_notes_condition_is_not_vacuously_true() {
    assert!(!Listing::default().every_row_lacks_note_text());

    let mut l = Listing::default();
    l.rows.push(CommentRow {
        page_index: 0,
        id: None,
        subtype: "Square".to_owned(),
        is_ce_dimension: false,
        note: Note::Absent,
        author: None,
        modified: None,
        suppressed: false,
        appearance_unresolved: false,
        relation: None,
        in_reply_to: None,
    });
    assert!(l.every_row_lacks_note_text());
    assert_eq!(l.with_note_text(), 0);

    // An accessibility description is NOT note text — it is the document
    // describing a control to a screen reader, and counting it would
    // withhold the disclosure from a document that genuinely has no notes.
    l.rows[0].note = Note::Description("Opens the drawing index".to_owned());
    assert!(l.every_row_lacks_note_text());
    assert_eq!(l.with_note_text(), 0);

    l.rows[0].note = Note::Text("Check this weld".to_owned());
    assert!(!l.every_row_lacks_note_text());
    assert_eq!(l.with_note_text(), 1);
}

/// **Every listed row carries a page index inside the document.**
#[test]
fn every_row_can_be_navigated_to() {
    let (session, pages) = open("annot/thread.pdf");
    let ce = ce_dimension_annots(&session);
    let l = collect(&session.view(), &pages, &ce);
    assert!(!l.rows.is_empty());
    for row in &l.rows {
        assert!(row.page_index < pages.len(), "{row:?} is off the end");
    }
}
