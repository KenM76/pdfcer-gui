//! # `app::lifecycle::tests` — what is guaranteed about a document arriving
//! and leaving
//!
//! The seam [`super`]'s own header draws: `lifecycle.rs` answers *"what happens
//! when a document arrives, and what has to be forgotten when it leaves?"* and
//! grows when a loading verb or a forgetting step is added. This file answers
//! *"what must never be true after one of those transitions?"* and grows when a
//! way of getting that wrong is found. Different rate, different reader: the
//! second is what somebody opens after a panel described the previous document.
//!
//! The suite's centre of gravity is the **forgetting**. State that outlives a
//! document — panel expansion sets, the Properties focus, the mid-navigation
//! flag — is invisible when it is wrong: the new document simply shows somebody
//! else's rows, and nothing anywhere says so.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/lifecycle/tests.md`.

// The inner `#![cfg(test)]` is redundant to the compiler — the module is
// declared `#[cfg(test)] mod tests;` — and is here anyway because
// `tools/gates/check-ui-strings.sh` exclusion 2b recognises a test-only *file*
// by exactly this attribute. Without it every assertion message below is read
// as operator copy outside the catalog.
#![cfg(test)]

use super::*;
use crate::app::state::{FOUR_PAGES, open_fixture};
use crate::panels::objects::test_support::engine_fixture;

// =======================================================================
// Opening a document is what forgets the panels' state
//
// Every assertion below is about the **transition**; none reads a field of
// `OpenDoc` except to check it was reset.
// =======================================================================

/// **Opening a document forgets the panels' view state.**
#[test]
fn opening_a_document_forgets_the_panels_focus_and_expansion() {
    let mut app = PdfcerApp::new();
    app.panels.set_focus(7);
    app.panels.tree_mut().toggle_object(7);
    assert_eq!(app.panels.focus(), Some(7));

    app.open_path(engine_fixture(FOUR_PAGES));
    assert!(matches!(app.status, Status::Open(_)), "the fixture opens");
    assert_eq!(
        app.panels.focus(),
        None,
        "a new document makes every paint-order index meaningless"
    );
    assert!(app.panels.tree_mut().objects_expanded.is_empty());
}

// =======================================================================
// Which arrangement a document opens in
// =======================================================================

/// **Read mode opens a document continuous; every other mode opens it
/// single page.**
#[test]
fn read_mode_opens_a_document_continuous_and_the_others_paged() {
    for (mode, expected) in [
        ("read", viewer::PageDisplay::Continuous),
        ("review", viewer::PageDisplay::Single),
        ("edit", viewer::PageDisplay::Single),
    ] {
        let mut app = PdfcerApp::new();
        app.ribbon.set_mode(mode.to_owned());
        app.open_path(engine_fixture(FOUR_PAGES));
        let Status::Open(doc) = &app.status else {
            panic!("the fixture opens");
        };
        assert_eq!(
            doc.view.display, expected,
            "{mode} mode opened the document in {:?}",
            doc.view.display
        );
    }
}

/// A freshly opened document is not mistaken for one that has been
/// navigated to.
#[test]
fn a_freshly_opened_document_is_not_mid_navigation() {
    let doc = open_fixture(FOUR_PAGES);
    assert_eq!(doc.tracked_page, doc.view.page_index);
    assert!(doc.strip_visible.is_empty());
    assert!(doc.strip_rasters.is_empty());
    assert!(doc.render_in_flight.is_none());
}

// =======================================================================
// `file.new` — making a document rather than opening one
// =======================================================================

/// The handler token the ribbon would raise for `id`.
fn token_for(app: &PdfcerApp, id: &str) -> egui_shell::commands::HandlerToken {
    app.commands
        .get(id)
        .unwrap_or_else(|| panic!("`{id}` must be registered")) // ui-text-exempt: test panic, never displayed
        .handler
}

/// **`file.new` raises `Action::New`, and applying it makes a document.**
#[test]
fn the_new_command_makes_a_blank_document_from_nothing() {
    // A bare context: this exercises the dispatcher, not a frame.
    let ctx = egui::Context::default();
    let mut app = PdfcerApp::new();
    assert!(matches!(app.status, Status::Empty));

    let mut actions = Vec::new();
    app.dispatch_token(&ctx, token_for(&app, "file.new"), &mut actions);
    assert_eq!(actions, vec![crate::app::actions::Action::New]);

    app.apply_actions(actions, 1.0);
    let Status::Open(doc) = &app.status else {
        panic!("New must leave a document open");
    };
    assert_eq!(doc.pages.len(), 1, "New makes a one-page document");
    assert_eq!(
        doc.origin,
        crate::app::state::Origin::Created,
        "a document New made has no file behind it"
    );
}

/// **New replaces what is open, and forgets what belonged to it.**
#[test]
fn new_replaces_the_open_document_and_forgets_its_panel_state() {
    let mut app = PdfcerApp::new();
    app.open_path(engine_fixture(FOUR_PAGES));
    app.panels.set_focus(3);
    app.panels.tree_mut().toggle_object(3);
    let Status::Open(doc) = &app.status else {
        panic!("the fixture opens");
    };
    assert_eq!(doc.pages.len(), 4, "the fixture is the four-page one");

    app.apply_actions(vec![crate::app::actions::Action::New], 1.0);

    let Status::Open(doc) = &app.status else {
        panic!("New must leave a document open");
    };
    assert_eq!(doc.pages.len(), 1, "the four-page document was replaced");
    assert_eq!(
        app.panels.focus(),
        None,
        "a paint-order index into the previous document means nothing here"
    );
    assert!(app.panels.tree_mut().objects_expanded.is_empty());
}

/// **Successive new documents are numbered, and the number is visible.**
#[test]
fn each_new_document_is_numbered_from_one() {
    let mut app = PdfcerApp::new();

    app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
    let Status::Open(first) = &app.status else {
        panic!("New must leave a document open");
    };
    assert_eq!(first.path, PathBuf::from("Untitled 1.pdf"));

    app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
    let Status::Open(second) = &app.status else {
        panic!("New must leave a document open");
    };
    assert_eq!(second.path, PathBuf::from("Untitled 2.pdf"));
}

/// **A document with no file gets no Recent row — and one with a file
/// still does.**
#[test]
fn a_created_document_is_not_remembered_but_an_opened_one_is() {
    let mut app = PdfcerApp::new();

    app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
    assert!(
        app.recent.is_empty(),
        "`Untitled 1.pdf` is a name, not a file; a Recent row for it could never be opened"
    );

    app.open_path(engine_fixture(FOUR_PAGES));
    assert_eq!(
        app.recent.entries().len(),
        1,
        "the guard must not have turned the recent list off altogether"
    );

    // …and a New over the top of it does not add a second row, nor drop
    // the one that is there. Closing is not disowning, and neither is
    // replacing.
    app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
    assert_eq!(app.recent.entries().len(), 1);
}

/// **`stored_under` is the whole of the difference, in both directions.**
#[test]
fn only_a_document_with_a_file_has_somewhere_to_store_its_preferences() {
    let mut app = PdfcerApp::new();

    app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
    let Status::Open(created) = &app.status else {
        panic!("New must leave a document open");
    };
    assert_eq!(created.stored_under(), None);

    let fixture = engine_fixture(FOUR_PAGES);
    app.open_path(fixture.clone());
    let Status::Open(opened) = &app.status else {
        panic!("the fixture opens");
    };
    assert_eq!(opened.stored_under(), Some(fixture.as_path()));
}

/// **A new document lands in the mode's default arrangement, not in a
/// remembered one.**
#[test]
fn a_new_document_takes_the_modes_default_arrangement() {
    for (mode, expected) in [
        ("read", viewer::PageDisplay::Continuous),
        ("review", viewer::PageDisplay::Single),
        ("edit", viewer::PageDisplay::Single),
    ] {
        let mut app = PdfcerApp::new();
        app.ribbon.set_mode(mode.to_owned());
        app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
        let Status::Open(doc) = &app.status else {
            panic!("New must leave a document open");
        };
        assert_eq!(
            doc.view.display, expected,
            "{mode} mode made the new document {:?}",
            doc.view.display
        );
        assert_eq!(
            app.ribbon.mode(),
            Some(mode),
            "New must not move the operator to another mode"
        );
    }
}

/// …and a *failed* open forgets it too.
#[test]
fn a_failed_open_forgets_the_panels_state_as_well() {
    let mut app = PdfcerApp::new();
    app.panels.set_focus(3);
    app.open_path(engine_fixture("not-a-pdf.bin"));
    assert!(
        matches!(app.status, Status::Failed { .. }),
        "this fixture must fail to open, or the test proves nothing"
    );
    assert_eq!(app.panels.focus(), None);
}
