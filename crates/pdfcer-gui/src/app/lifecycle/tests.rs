//! # `app::lifecycle::tests` — what is guaranteed about a document arriving
//! and leaving
//!
//! Split out of [`super`] on 2026-09-10 under rule **R2**, when the
//! re-read-under-a-different-reading route (`ENGINE_BACKLOG.md`'s
//! *"Re-open a self-contradicting file under a chosen load policy"*) needed
//! lines in a file that stood at **exactly 1,500** — the ceiling
//! `tools/gates/check-file-size.sh` enforces. **Nothing moved but its
//! address.**
//!
//! ★ The seam is the one [`super`]'s own header already draws twice.
//! `lifecycle.rs` answers *"what happens when a document arrives, and what has
//! to be forgotten when it leaves?"* and grows when a loading verb or a
//! forgetting step is added. This file answers *"what must never be true after
//! one of those transitions?"* and grows when a way of getting that wrong is
//! found. Different rate, different reader: the second is what somebody opens
//! after a panel described the previous document.
//!
//! ★★ The suite's centre of gravity is the **forgetting**. Three of its
//! assertions exist because state that outlives a document — panel expansion
//! sets, the Properties focus, the mid-navigation flag — is invisible when it
//! is wrong: the new document simply shows somebody else's rows, and nothing
//! anywhere says so.

// ★ The INNER `#![cfg(test)]` is redundant — the module is declared
// `#[cfg(test)] mod tests;` — and it is here anyway, because
// `tools/gates/check-ui-strings.sh` exclusion 2 recognises a test-only FILE by
// exactly this attribute. Without it every assertion message below is read as
// operator copy outside the catalog.
#![cfg(test)]

use super::*;
use crate::app::state::{FOUR_PAGES, open_fixture};
use crate::panels::objects::test_support::engine_fixture;

// =======================================================================
// Opening a document is what forgets the panels' state
//
// Moved here with `open_path` when `state.rs` was split under R2. They are
// the test for whether that split was along a seam: every one of them is
// about the **transition**, and none reads a field of `OpenDoc` except to
// check it was reset.
// =======================================================================

/// **★ Opening a document forgets the panels' view state.**
///
/// The second half of the `DocKey` deletion. Expansion sets and the
/// Properties focus are paint-order indices that live on `PdfcerApp`, so
/// they genuinely do outlive a document. The old answer was to compare a
/// document identity every frame; the answer here is that documents are
/// opened in exactly one place, so forgetting is one statement at the one
/// moment it is true.
///
/// Without it, opening a second document leaves the Objects panel with
/// rows expanded for a page that no longer exists and the Properties
/// panel describing whatever object lands at that index in the new
/// file.
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
// Phase 4 — which arrangement a document opens in
// =======================================================================

/// ★ **Read mode opens a document continuous; every other mode opens it
/// single page.**
///
/// `MODES_AND_PANELS.md`'s table and the operator decision of 2026-08-13,
/// asserted through the **open path** rather than through
/// `PageDisplay::default_for_mode` — which is already tested in its own
/// module. What this adds is that `open_path` actually consults it: the
/// rule existing and the rule being applied are two different facts, and
/// the second is the one an operator experiences.
///
/// Driven with no remembered choice for the fixture (nothing has ever set
/// one for a path under the engine fixtures directory), so what is measured
/// is the mode default and not a leftover.
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
///
/// `tracked_page` starting anywhere but at `view.page_index` would make
/// the canvas scroll a continuous strip on the first frame after an open,
/// which the operator did not ask for and which would fight a saved scroll
/// position the moment there is one.
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

/// ★ **`file.new` raises `Action::New`, and applying it makes a document.**
///
/// Driven through the real token lookup rather than by calling the arm,
/// exactly as `the_close_command_empties_the_shell` is, so a command that
/// stopped being registered fails here instead of silently taking the
/// `command-unimplemented` path — which is the failure `file.open` and
/// `file.close` both shipped with, and which no test that called the
/// function directly could ever have caught.
///
/// The starting state is `Empty`, which is the state New exists for: an
/// operator who has just launched pdfcer with no argument.
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

/// ★ **New replaces what is open, and forgets what belonged to it.**
///
/// The reason [`PdfcerApp::adopt`] was extracted rather than copied. A New
/// that left the panels' paint-order indices behind would show the Objects
/// panel expanded over rows of a four-page drawing that is no longer open,
/// on a document that has one blank page — and every test of `open_path`
/// would still pass, because `open_path` would still be doing it correctly.
///
/// The page count moving from four to one is what makes "replaced" a
/// measurement rather than an assumption.
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

/// ★ **Successive new documents are numbered, and the number is visible.**
///
/// `crate::text::files::untitled`'s own test pins that the *function*
/// numbers; this pins that the **application** advances the ordinal, which
/// is a different fact and the one that breaks if the increment is dropped
/// or placed after the name is built. Without it both documents would be
/// `Untitled 1.pdf`, the forms cache would key two different documents the
/// same way, and the trace of a driven run could not tell a second New
/// from a New that did nothing.
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

/// ★ **A document with no file gets no Recent row — and one with a file
/// still does.**
///
/// Both halves, because the interesting failure is not "New was skipped"
/// but "the guard was written the wrong way round and now nothing is ever
/// remembered". A Recent menu offering `Untitled 1.pdf` is a row that
/// cannot be opened, on a surface whose whole promise is *this worked
/// before*.
///
/// `PdfcerApp::new()` under `cfg(test)` builds a `RecentFiles` that points
/// nowhere and writes nothing, so this reads the in-memory list and leaves
/// the operator's own recent file untouched.
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

/// ★ **`stored_under` is the whole of the difference, in both directions.**
///
/// The predicate three call sites consult. Asserted as a pair rather than
/// one at a time, because a version that answered `None` for everything
/// would satisfy every assertion about created documents in this file and
/// would silently stop persisting page-display and guide choices for real
/// ones — a regression with no visible symptom until the next session.
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

/// ★ **A new document lands in the mode's default arrangement, not in a
/// remembered one.**
///
/// The sibling of `read_mode_opens_a_document_continuous_and_the_others_paged`,
/// and it asserts something that test cannot: a created document reaches
/// the *second* source every time, because `stored_under` answers `None`
/// and there is nothing to recall. New therefore inherits the mode the
/// operator is in rather than changing it — see `new_document`'s own note
/// on why it does not switch to Edit.
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

/// …and a FAILED open forgets it too.
///
/// Whatever was showing is gone either way, and stale expansion state
/// over a document that could not be read is the worse of the two states
/// to leave behind: the panel would look populated while the shell says
/// the file is damaged.
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
