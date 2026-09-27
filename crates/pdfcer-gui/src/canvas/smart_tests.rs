//! Tests for `crate::canvas::smart`, kept in the gui because they reach gui modules.

use crate::canvas::smart::*;
use crate::canvas::target::TargetId;
use crate::canvas::targetstub::StubTargets;

/// A stub whose leaves all belong to page object 0.
fn stub() -> StubTargets {
    StubTargets::new(
        0,
        [egui::Rect::from_min_max(
            egui::pos2(0.0, 0.0),
            egui::pos2(100.0, 100.0),
        )],
    )
    .with_leaves([egui::Rect::from_min_max(
        egui::pos2(10.0, 10.0),
        egui::pos2(20.0, 20.0),
    )])
    .with_containers([(0, 0)])
}

/// **A click on something inside a container selects the container.**
///
/// The feature, stated as the one substitution it performs.
#[test]
fn a_leaf_resolves_to_its_container() {
    let ctx = egui::Context::default();
    let targets = stub();
    assert_eq!(
        scope(&ctx, 0).resolve(&targets, 0, TargetId::Leaf(0)),
        TargetId::Object(0),
        "clicking a line inside a title block selects the title block"
    );
}

/// …**and once inside it, the same click selects the line.**
#[test]
fn inside_a_container_a_leaf_resolves_to_itself() {
    let ctx = egui::Context::default();
    let targets = stub();
    enter(
        &ctx,
        Entered {
            page: 0,
            form: 0,
            slot: 0,
        },
    );
    assert_eq!(
        scope(&ctx, 0).resolve(&targets, 0, TargetId::Leaf(0)),
        TargetId::Leaf(0)
    );
}

/// **The record is scoped to its page and its document**, and does not
/// have to be cleared by whoever changes either.
#[test]
fn the_scope_does_not_follow_you_to_another_page_or_document() {
    let ctx = egui::Context::default();
    let targets = stub();
    enter(
        &ctx,
        Entered {
            page: 0,
            form: 0,
            slot: 0,
        },
    );
    assert!(entered(&ctx, 0, 0).is_some());
    assert!(entered(&ctx, 1, 0).is_none(), "another page");
    assert!(entered(&ctx, 0, 1).is_none(), "another document");

    // And the consequence, which is the part worth asserting: in the
    // OTHER document the same click resolves to the container again. The
    // record being filtered out is the mechanism; this is the behaviour.
    //
    // Asked of page 0 deliberately — the stub answers `containing_form`
    // for its own page only, exactly as the live provider does (it
    // decomposes one page), so asking about page 1 would be testing the
    // stub's page guard rather than this module's scope.
    assert_eq!(
        Scope {
            enabled: true,
            entered: entered(&ctx, 0, 1).map(|e| e.form),
        }
        .resolve(&targets, 0, TargetId::Leaf(0)),
        TargetId::Object(0),
        "the scope belongs to the document it was entered in"
    );
}

/// Switched off, nothing is substituted.
#[test]
fn with_the_option_off_a_leaf_is_a_leaf() {
    let ctx = egui::Context::default();
    let targets = stub();
    set_enabled(&ctx, false);
    assert_eq!(
        scope(&ctx, 0).resolve(&targets, 0, TargetId::Leaf(0)),
        TargetId::Leaf(0)
    );
}

/// **Switching it off leaves the container too.**
///
/// A scope that outlived the mechanism that created it would make the next
/// click resolve by a rule that is no longer switched on.
#[test]
fn switching_it_off_leaves_the_container() {
    let ctx = egui::Context::default();
    enter(
        &ctx,
        Entered {
            page: 0,
            form: 0,
            slot: 0,
        },
    );
    set_enabled(&ctx, false);
    assert!(entered(&ctx, 0, 0).is_none());
}

/// `leave` reports whether it did anything, which is what makes Escape a
/// two-step gesture rather than one that clears everything at once.
#[test]
fn leaving_reports_whether_there_was_anything_to_leave() {
    let ctx = egui::Context::default();
    assert!(
        !leave(&ctx),
        "nothing entered, so Escape means something else"
    );
    enter(
        &ctx,
        Entered {
            page: 0,
            form: 0,
            slot: 0,
        },
    );
    assert!(leave(&ctx));
    assert!(!leave(&ctx));
}

/// A page object is never substituted, switched on or off.
#[test]
fn a_page_object_is_always_itself() {
    let ctx = egui::Context::default();
    let targets = stub();
    assert_eq!(
        scope(&ctx, 0).resolve(&targets, 0, TargetId::Object(0)),
        TargetId::Object(0)
    );
}
