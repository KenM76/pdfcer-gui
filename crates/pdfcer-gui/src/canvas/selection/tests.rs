//! # `canvas::selection::tests` — the selection algebra, asserted
//!
//! Split out of [`super`] under **R2** on 2026-08-18, when annotation
//! selection took the module past 1,500 lines.
//!
//! ## Why the TESTS moved and not the code
//!
//! R2's usual answer is *find the seam and split the subject*, and the honest
//! reading here is that [`super`] does not have one worth taking. Its subject
//! is a single value — what is selected and how a gesture changes it — and the
//! candidate seams (the outline cache, the ladder) are all things that exist
//! **because of** that value rather than beside it. Splitting them would give
//! two files that had to be read together, which is worse than one long file
//! and is what R2 is trying to prevent, not what it asks for.
//!
//! Tests are a different subject by construction: they change when a *claim*
//! about the algebra changes, where the code changes when the algebra does.
//! `egui-shell`'s ribbon has taken this route three times already
//! (`ribbon/tests.rs`, `ribbon/height_tests.rs`, `ribbon/width_tests.rs`), so
//! it is a pattern this codebase already uses rather than one invented to get
//! under a limit.
//!
//! **The gate counts total lines, tests included, on purpose** — its own
//! header says so — so this is not a way of hiding lines from it. It is the
//! split the limit asked for, taken on the seam that is actually there.

#![cfg(test)]

/// ★★ **One canvas, one selection** — content and annotation are mutually
/// exclusive, in both directions.
///
/// The invariant this type owns, and the reason the annotation lives here
/// as a field rather than beside this state on `OpenDoc`. A build where
/// both could be non-empty would draw two kinds of outline at once and
/// leave `format.delete` and the Delete key with two plausible meanings —
/// one of which removes page content the operator did not point at, which
/// is the loss `deletable_objects_on`'s own guard calls *"one line and the
/// whole view"*.
///
/// Both directions, because the two writers are different code and a build
/// that cleared only one way is the more likely mistake.
#[test]
fn selecting_an_annotation_and_selecting_content_replace_each_other() {
    use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};
    use egui::{Pos2, Rect};

    let stamp = AnnotSelection {
        target: AnnotTarget {
            page: 0,
            id: pdfcer_core::object::ObjId::new(12, 0),
            kind: AnnotKind::Markup,
            subtype: "Stamp".to_owned(),
            locked: false,
        },
        outline: Rect::from_min_size(Pos2::ZERO, egui::vec2(20.0, 10.0)),
    };

    // content ▸ annotation
    let mut state = SelectionState::default();
    state.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(4)),
            part: None,
            node: None,
        },
        false,
        false,
    );
    assert_eq!(state.len(), 1, "the content click selected something");
    state.select_annot(stamp.clone());
    assert!(
        state.entries().is_empty(),
        "selecting an annotation must drop the content selection"
    );
    assert!(state.annot().is_some());
    assert!(!state.is_empty(), "a selected annotation IS a selection");

    // annotation ▸ content
    state.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(4)),
            part: None,
            node: None,
        },
        false,
        false,
    );
    assert!(
        state.annot().is_none(),
        "selecting content must drop the annotation selection"
    );
    assert_eq!(state.len(), 1);
}

/// ★ `is_empty` answers about **both**, and the Format tab depends on it.
///
/// Pinned separately from the exclusion test because it is a different
/// failure: a build that kept the two exclusive and still answered
/// `is_empty() == true` over a selected stamp would hide the contextual
/// Format tab — the one surface the whole feature exists to reach — while
/// the outline sat on the page saying something was selected.
#[test]
fn a_selected_annotation_is_not_an_empty_selection() {
    use crate::canvas::selection::annot::{AnnotKind, AnnotSelection, AnnotTarget};
    use egui::{Pos2, Rect};

    let mut state = SelectionState::default();
    assert!(state.is_empty(), "nothing selected to begin with");
    state.select_annot(AnnotSelection {
        target: AnnotTarget {
            page: 2,
            id: pdfcer_core::object::ObjId::new(9, 0),
            kind: AnnotKind::CeDimension,
            subtype: "Line".to_owned(),
            locked: false,
        },
        outline: Rect::from_min_size(Pos2::ZERO, egui::vec2(5.0, 5.0)),
    });
    assert!(!state.is_empty());
    assert!(state.clear_annot(), "…and reports that it dropped one");
    assert!(state.is_empty());
    assert!(
        !state.clear_annot(),
        "a second drop reports nothing to drop"
    );
}
use super::*;
use crate::canvas::target::StubTargets;
use egui::{Pos2, Rect};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h))
}

/// A page with two objects, the first of which has two parts.
fn stub(page: usize) -> StubTargets {
    StubTargets::new(
        page,
        [rect(0.0, 0.0, 100.0, 100.0), rect(200.0, 200.0, 50.0, 50.0)],
    )
    .with_parts(
        0,
        [rect(0.0, 0.0, 40.0, 40.0), rect(60.0, 60.0, 40.0, 40.0)],
    )
}

fn hit_object(index: u64) -> ClickHit {
    ClickHit {
        object: Some(TargetId::Object(index)),
        ..ClickHit::default()
    }
}

// -----------------------------------------------------------------
// ★ Invariant 1 — selection is identity, never position
// -----------------------------------------------------------------

/// ★ **Navigation never alters the selection.**
///
/// The acceptance criterion, as close to literally as a headless test can
/// state it: select a node, then perform every navigation the roadmap
/// names — zoom out three rungs, pan, change fit mode, rotate the view,
/// change page-display mode, switch ribbon tab — and assert the selection
/// is byte-identical afterwards.
///
/// # ★ Phase 3's gestures were added to THIS sweep, not to a parallel test
///
/// The hand-tool pan, the anchored discrete zoom, the marquee zoom and
/// zoom-to-selection are navigation, so they belong to the invariant that
/// already governs navigation. A second test asserting the same property
/// about four more operations would be a second place for the property to
/// be stated — and the first one to be forgotten when a fifth arrives.
///
/// **Zoom-to-selection is the interesting addition**, because it is the
/// only navigation that *reads* the selection: it resolves the selection's
/// bounds and frames them. Reading is exactly where a "helpful" edit —
/// normalise the entries, collapse to the outlined ones, drop what has no
/// bounds — would creep in, and it would be invisible until the operator
/// zoomed to a node and found they had selected the object instead.
///
/// What this cannot reach is the *wiring*: that a released
/// `MarqueeIntent::Zoom` never calls [`SelectionState::marquee`] at all.
/// That is structural in `canvas::interact` — the two intents are separate
/// match arms over an exhaustive enum, and only one of them names the
/// selection — and it is asserted from the gesture side by
/// `canvas::gesture`'s `a_zoom_marquee_is_the_same_band_with_the_other_intent`.
///
/// It is expressed as *"drive the view state and then compare"* because
/// that is the honest model of what navigation is: those operations act
/// on [`crate::viewer::ViewState`], and the property being asserted is
/// that no route exists from there to here. The test would fail the
/// moment somebody gave `SelectionState` a screen coordinate to keep in
/// step, which is the defect it guards.
#[test]
fn navigating_the_view_never_alters_the_selection() {
    use crate::viewer::{FitMode, MAX_ZOOM, ViewState};

    let targets = stub(0);
    let mut sel = SelectionState::default();
    // Select a node: click, double-click into the part, double-click
    // again to reach the anchor.
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: None,
        },
        false,
        true,
    );
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: Some(4),
        },
        false,
        true,
    );
    sel.resolve(Some(&targets), 0, 0);
    assert_eq!(sel.level(), SelectionLevel::Node);
    assert_eq!(sel.entries()[0].node, Some(4));
    let before = sel.clone();

    // Every navigation the invariant names, in one sweep.
    let mut view = ViewState::default();
    for _ in 0..3 {
        view.zoom_out(MAX_ZOOM);
    }
    view.set_fit(FitMode::Width);
    view.apply_fit((612.0, 792.0), (300.0, 900.0), MAX_ZOOM);
    view.set_fit(FitMode::Page);
    view.apply_fit((612.0, 792.0), (1_600.0, 400.0), MAX_ZOOM);
    view.zoom_by(1.37, MAX_ZOOM);
    view.next_page(4);
    view.prev_page(4);

    // ---- Phase 3's navigation gestures, in the same sweep -------------
    use crate::canvas::geometry;
    use crate::canvas::mapping::PageMapping;
    use crate::canvas::zoom::{self, ZoomOutcome};

    let extent = (200.0_f32, 300.0_f32);
    let frame = zoom::CanvasFrame {
        map: PageMapping::new(
            Rect::from_min_size(Pos2::new(12.0, 7.0), egui::vec2(extent.0, extent.1)),
            extent,
            1.0,
        ),
        extent,
        display: (extent.0, extent.1),
        viewport: (400.0, 400.0),
        // No scroll bar in a test world, so the outer and inner sizes agree.
        // See `CanvasFrame::outer` for when they do not.
        outer: (400.0, 400.0),
        viewport_rect: Rect::from_min_size(Pos2::new(10.0, 5.0), egui::vec2(400.0, 400.0)),
        offset: (0.0, 0.0),
        // The single-page world every test in here builds: one page,
        // at the strip origin. See `ZoomAnchor::page`.
        page: 0,
    };

    // A hand-tool / space-bar pan. The same arithmetic the middle drag
    // uses, and it must move the view — a pan that clamped to a no-op
    // would make the assertion below vacuous.
    let panned = geometry::pan_offset(
        (120.0, 80.0),
        (30.0, -20.0),
        (1_600.0, 1_600.0),
        (800.0, 800.0),
    );
    assert_ne!(panned, (120.0, 80.0), "the pan must actually move the view");

    // An anchored discrete zoom: arm on a page point, step the ladder,
    // solve. This is Ctrl+Plus, end to end, minus the `egui::Context`.
    let anchor = zoom::hold(zoom::frac_of(Pos2::new(50.0, 50.0), extent), &frame);
    view.zoom_in(MAX_ZOOM);
    let _ = geometry::zoom_anchor_offset(
        anchor.offset_before,
        anchor.display_before,
        (extent.0 * view.zoom, extent.1 * view.zoom),
        anchor.viewport,
        anchor.frac,
    );

    // A marquee zoom to a region of the page.
    let region = Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(90.0, 120.0));
    if let ZoomOutcome::Zoomed { applied, .. } = zoom::plan_framing(
        &frame,
        region,
        16.0,
        1.0,
        crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT,
    )
    .outcome
    {
        view.set_zoom(applied, MAX_ZOOM);
    }

    // ★ Zoom to the selection — the one navigation that reads it.
    let bounds = sel
        .outline_union()
        .expect("a resolved selection has bounds to frame");
    if let ZoomOutcome::Zoomed { applied, .. } = zoom::plan_framing(
        &frame,
        bounds,
        16.0,
        1.0,
        crate::app::prefs::DEFAULT_MAX_ZOOM_PERCENT,
    )
    .outcome
    {
        view.set_zoom(applied, MAX_ZOOM);
    }

    // ---- Phase 4's page-display modes, in the same sweep ---------------
    //
    // ★ Added HERE rather than in a parallel test, for the reason this
    // test's header already gives about Phase 3's gestures: a page-display
    // change is navigation, and navigation is governed by this invariant.
    // A second test asserting the same property about a fifth operation
    // would be a second place for the property to live and the first one
    // to be forgotten.
    //
    // `FEATURES.md` names "page-display mode" in the list of things the
    // selection is asserted byte-identical across, and until Phase 4 there
    // was only one mode — so the clause was true and untested. It is now
    // exercised: every arrangement, including the two that put several
    // pages on screen at once, and a full strip laid out for each so the
    // geometry the mode produces is real rather than nominal.
    use crate::viewer::{PageDisplay, strip::Strip};
    use pdfcer_core::object::{Dict, ObjId};
    use pdfcer_core::page_tree::{Page, Rect as PageRect};

    let pages: Vec<Page> = (0..4)
        .map(|_| Page {
            id: ObjId::new(1, 0),
            resources: Dict::new(),
            media_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
            crop_box: PageRect::from_corners(0.0, 0.0, 612.0, 792.0),
            rotate: 0,
            contents: Vec::new(),
            contents_unresolved: 0,
            contents_flattened: 0,
        })
        .collect();
    for &display in PageDisplay::ALL {
        view.display = display;
        let strip = Strip::new(&pages, display, view.page_index, view.zoom);
        // The mode really does change the layout, or the loop asserts
        // nothing about the modes it iterates.
        assert!(!strip.is_empty());
        let metrics = crate::viewer::strip::row_metrics(&pages, display, view.page_index, 1.0);
        view.apply_fit(metrics.extent, (900.0, 700.0), metrics.max_zoom);
    }
    assert!(
        Strip::new(&pages, PageDisplay::Continuous, 0, 1.0).size().y
            > Strip::new(&pages, PageDisplay::Single, 0, 1.0).size().y,
        "the continuous strip must be taller than one page, or the sweep \
         above passed through four modes that all laid out the same thing"
    );

    // The provider is rebuilt on the way — that is what a page step, a
    // page-display change and a ribbon-tab change do — and the selection
    // must come through it.
    sel.resolve(Some(&stub(0)), 0, 0);

    assert_eq!(
        sel, before,
        "a view change reached the selection; it must not be able to"
    );
}

/// ★ **A selection on another page survives a provider for a different
/// page** — the half of invariant 3 the acceptance criterion turns on.
///
/// Paging away rebuilds the provider for the new page. A `resolve` that
/// pruned everything it could not find would wipe the selection on the
/// way past, and coming back would find nothing.
#[test]
fn paging_away_and_back_keeps_the_selection() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(1), false, false);
    sel.resolve(Some(&stub(0)), 0, 0);
    assert_eq!(sel.len(), 1);
    assert_eq!(sel.outlines().len(), 1);

    // Page 1: a provider that knows nothing about page 0's objects.
    sel.resolve(Some(&stub(1)), 1, 0);
    assert_eq!(sel.len(), 1, "the entry for page 0 must survive");
    assert!(
        sel.outlines().is_empty(),
        "nothing on page 1 is selected, so nothing on page 1 is outlined"
    );

    // …and back.
    sel.resolve(Some(&stub(0)), 0, 0);
    assert_eq!(sel.entries(), [Selection::object(0, TargetId::Object(1))]);
    assert_eq!(sel.outlines().len(), 1, "the outline comes back with it");
}

/// A rebuild at the same revision is a no-op that costs one comparison —
/// which is what makes "resolve every frame" affordable and therefore
/// what makes the invariant cheap enough to actually hold.
#[test]
fn re_resolving_at_the_same_revision_changes_nothing() {
    let targets = stub(0);
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.resolve(Some(&targets), 0, 0);
    let after_first = sel.clone();
    for _ in 0..50 {
        sel.resolve(Some(&targets), 0, 0);
    }
    assert_eq!(sel, after_first);
}

/// An edit that removed a selected object drops **that** entry and keeps
/// the rest, rather than clearing the selection or leaving a hole a
/// batched delete would refuse.
#[test]
fn an_edit_that_removed_an_object_drops_only_that_entry() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(0, hit_object(1), true, false);
    sel.resolve(Some(&stub(0)), 0, 0);
    assert_eq!(sel.len(), 2);

    // The page now holds one object: index 1 is gone.
    let after_edit = StubTargets::new(0, [rect(0.0, 0.0, 100.0, 100.0)]);
    sel.resolve(Some(&after_edit), 0, 1);
    assert_eq!(sel.entries(), [Selection::object(0, TargetId::Object(0))]);
}

/// An undecodable page loses its outlines and keeps its selection. The
/// two states are different and must not be conflated — and the failure
/// is recorded, so the decomposition that would not decode is not retried
/// on every frame.
#[test]
fn losing_the_provider_does_not_lose_the_selection() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.resolve(Some(&stub(0)), 0, 0);
    assert_eq!(sel.outlines().len(), 1);

    // The operator pages to a sheet whose content will not decode.
    assert!(sel.needs_resolve(1, 0));
    sel.resolve(None, 1, 0);
    assert!(sel.outlines().is_empty());
    assert_eq!(sel.len(), 1, "the selection is not the outline");
    assert!(
        !sel.needs_resolve(1, 0),
        "a failed decomposition must not be retried every frame"
    );
}

/// ★ **Delete's operand list is empty at every rung below Object** — the
/// one statement of that rule, asserted where it lives.
///
/// The canvas keys and the ribbon's `format.delete` both read
/// [`SelectionState::deletable_objects_on`], so this test covers both. It
/// is the destructive case: the only wired verb removes whole objects, and
/// one measured CAD export holds an entire drawing view as a single path
/// object with 1,194 subpaths.
///
/// The page filter is asserted too, because a paint-order index is a
/// position on **one** page and handing `delete_objects` an index from
/// another one would remove whatever happens to sit at that slot.
#[test]
fn only_the_object_rung_offers_anything_to_delete() {
    let mut sel = SelectionState::default();
    assert!(sel.deletable_objects_on(0).is_empty(), "nothing selected");

    sel.click(0, hit_object(1), false, false);
    sel.click(0, hit_object(0), true, false);
    assert_eq!(
        sel.deletable_objects_on(0),
        vec![0, 1],
        "ascending and de-duplicated, or `delete_objects` refuses the batch"
    );
    assert!(
        sel.deletable_objects_on(1).is_empty(),
        "an index is a position on ONE page"
    );

    // Descend into the object: the rung has no delete verb.
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: None,
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Part);
    assert!(
        sel.deletable_objects_on(0).is_empty(),
        "deleting the enclosing object because one subpath was selected is \
         a destructive wrong action, not a convenience"
    );
    assert_eq!(sel.len(), 1, "and asking does not change the selection");

    // …and back out again, which restores the operand list.
    assert_eq!(
        sel.escape(),
        EscapeOutcome::LeftLevel(SelectionLevel::Object)
    );
    assert_eq!(sel.deletable_objects_on(0), vec![0]);
}

// -----------------------------------------------------------------
// Click semantics
// -----------------------------------------------------------------

#[test]
fn a_plain_click_replaces_and_a_shift_click_toggles() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    assert_eq!(sel.entries(), [Selection::object(0, TargetId::Object(0))]);

    sel.click(0, hit_object(1), true, false);
    assert_eq!(sel.len(), 2, "shift adds");

    sel.click(0, hit_object(1), true, false);
    assert_eq!(sel.len(), 1, "shift on a selected entry removes it");

    sel.click(0, hit_object(1), false, false);
    assert_eq!(
        sel.entries(),
        [Selection::object(0, TargetId::Object(1))],
        "a plain click replaces rather than adding"
    );
}

/// ★★★ **SHIFT-PICKING A SECOND ANCHOR ADDS IT** — the model half of the
/// driven check `multi_node_move_moves_every_picked_anchor`.
///
/// # Why this test was written
///
/// That check FAILED on 2026-08-21, reproducibly and in isolation:
///
/// > *"TWO MARKED ANCHORS WERE CLICKED, THE SECOND WITH SHIFT, AND 1 ENDED UP
/// > SELECTED."*
///
/// It had SKIPPED on every earlier run, for want of a `--doc-point` whose
/// subpath carried more than one anchor, so the rung had never actually been
/// exercised — by it or by anything here. `a_plain_click_replaces_and_a_shift_
/// click_toggles` covers the **Object** rung only, and nothing covered this one.
///
/// So the question was whether the *model* is wrong or the driven path is, and
/// the two need very different fixes. This is the cheap half of that question.
///
/// ★ It asserts through [`SelectionState::selected_nodes_on`] as well as
/// through the entry count, because that accessor is what
/// `canvas::moving`'s multi-node drag actually reads. A model that held two
/// entries but reported one node would satisfy a length check and still fail
/// the operator.
#[test]
fn shift_picking_a_second_anchor_adds_it_rather_than_replacing() {
    let targets = stub(0);
    let mut sel = SelectionState::default();

    // Descend to the Node rung on object 0, part 1, anchor 4 — the same route
    // the invariant test above takes.
    sel.click(0, hit_object(0), false, true);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: Some(4),
        },
        false,
        true,
    );
    sel.resolve(Some(&targets), 0, 0);
    assert_eq!(
        sel.level(),
        SelectionLevel::Node,
        "the rung must be entered"
    );
    assert_eq!(sel.selected_nodes_on(0, TargetId::Object(0)), vec![4]);

    // Shift-click a DIFFERENT anchor on the same subpath.
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: Some(7),
        },
        true,
        false,
    );

    assert_eq!(
        sel.selected_nodes_on(0, TargetId::Object(0)),
        vec![4, 7],
        "shift on a second anchor must ADD it — this is what the multi-node \
         move carries, and a selection the program shows and does not honour is \
         worse than not offering one"
    );
    assert_eq!(sel.len(), 2);
    assert_eq!(
        sel.level(),
        SelectionLevel::Node,
        "adding a second anchor must not fall back to the Object rung"
    );
}

/// ★★ …and shift on an anchor that is already picked REMOVES it, which is the
/// other half of a toggle and the half a naive fix breaks.
#[test]
fn shift_picking_a_selected_anchor_removes_it() {
    let targets = stub(0);
    let mut sel = SelectionState::default();

    sel.click(0, hit_object(0), false, true);
    let at = |node| ClickHit {
        object: Some(TargetId::Object(0)),
        part: Some(1),
        node: Some(node),
    };
    sel.click(0, at(4), false, true);
    sel.resolve(Some(&targets), 0, 0);
    sel.click(0, at(7), true, false);
    assert_eq!(sel.selected_nodes_on(0, TargetId::Object(0)), vec![4, 7]);

    sel.click(0, at(7), true, false);
    assert_eq!(
        sel.selected_nodes_on(0, TargetId::Object(0)),
        vec![4],
        "shift on an anchor that is already picked takes it back out"
    );
}

/// A plain click on empty paper clears; a shift click on empty paper does
/// not. The asymmetry is deliberate — an over-shot shift-click must not
/// destroy a set that took five clicks to build.
#[test]
fn a_miss_clears_only_without_shift() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(0, ClickHit::default(), true, false);
    assert_eq!(sel.len(), 1, "shift+miss leaves the selection alone");
    sel.click(0, ClickHit::default(), false, false);
    assert!(sel.is_empty(), "a plain click on empty paper clears");
}

/// Entries are held in document order however they were clicked, so the
/// outlines paint in a stable sequence.
#[test]
fn entries_are_ordered_and_unique_however_they_were_clicked() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(1), false, false);
    sel.click(0, hit_object(0), true, false);
    assert_eq!(
        sel.entries(),
        [
            Selection::object(0, TargetId::Object(0)),
            Selection::object(0, TargetId::Object(1))
        ]
    );
    assert_eq!(sel.object_indices_on(0), vec![0, 1]);
    assert!(sel.object_indices_on(1).is_empty());
}

// -----------------------------------------------------------------
// The ladder
// -----------------------------------------------------------------

#[test]
fn a_double_click_descends_one_rung_at_a_time() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    assert_eq!(sel.level(), SelectionLevel::Object);

    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: None,
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Part);
    assert_eq!(sel.entries()[0].subpath, Some(1));

    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: Some(6),
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Node);
    assert_eq!(sel.entries()[0].node, Some(6));

    // Nothing is below a point.
    let at_the_bottom = sel.clone();
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: Some(6),
        },
        false,
        true,
    );
    assert_eq!(sel, at_the_bottom);
}

/// ★ **Escape ascends exactly one rung per press.**
///
/// The old shell shipped Escape as "clear everything", so an operator two
/// rungs inside a drawing found one press putting them back at the page.
/// Asserted as a sequence of outcomes rather than a boolean, so a
/// regression that collapsed the ladder cannot pass by clearing on the
/// first press and reporting `true` three times.
#[test]
fn escape_ascends_one_rung_per_press() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: None,
        },
        false,
        true,
    );
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: Some(2),
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Node);

    assert_eq!(
        sel.escape(),
        EscapeOutcome::LeftLevel(SelectionLevel::Part),
        "the first press leaves the Node rung and nothing else"
    );
    assert_eq!(sel.len(), 1, "leaving a rung does not clear the selection");
    assert_eq!(sel.entries()[0].node, None);
    assert_eq!(sel.entries()[0].subpath, Some(0));

    assert_eq!(
        sel.escape(),
        EscapeOutcome::LeftLevel(SelectionLevel::Object)
    );
    assert_eq!(sel.entries()[0].subpath, None);
    assert_eq!(sel.len(), 1);

    assert_eq!(sel.escape(), EscapeOutcome::ClearedSelection);
    assert!(sel.is_empty());

    assert_eq!(
        sel.escape(),
        EscapeOutcome::Nothing,
        "with nothing selected the canvas must not consume Escape"
    );
}

/// A click that misses everything inside the entered object leaves the
/// object rather than stranding the operator at a rung.
#[test]
fn clicking_away_leaves_the_entered_object() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: None,
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Part);

    sel.click(0, ClickHit::default(), false, false);
    assert_eq!(sel.level(), SelectionLevel::Object);
    assert!(sel.is_empty());
}

/// A click on a *different* object while inside one leaves and selects
/// that object — PDF path objects do not nest, so there is nothing to
/// nest into.
#[test]
fn clicking_a_different_object_leaves_rather_than_nesting() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: None,
        },
        false,
        true,
    );
    sel.click(0, hit_object(1), false, false);
    assert_eq!(sel.level(), SelectionLevel::Object);
    assert_eq!(sel.entries(), [Selection::object(0, TargetId::Object(1))]);
}

/// At the Node rung, a click that misses every anchor but lands on a part
/// ascends one rung and re-picks — rather than doing nothing, which is
/// how an operator gets stuck at a rung whose targets they keep missing.
#[test]
fn missing_every_anchor_falls_back_to_the_part_rung() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: None,
        },
        false,
        true,
    );
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: Some(1),
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Node);

    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: None,
        },
        false,
        false,
    );
    assert_eq!(sel.level(), SelectionLevel::Part);
    assert_eq!(sel.entries()[0].subpath, Some(1));
    assert_eq!(sel.entries()[0].node, None);
}

// -----------------------------------------------------------------
// Marquee
// -----------------------------------------------------------------

/// **★★★ A subtracting band takes exactly its hits out and leaves the rest** —
/// `OPERATOR_REQUESTS.md` O104.
///
/// The operator, 2026-09-03: *"I can't unselect things once I have selected
/// them for redaction."* This is the half a click could not do — on a sheet of
/// overlapping strokes, removing one object from a selection of twenty by
/// clicking it precisely is often not practical, and a band is how the work is
/// actually done.
#[test]
fn a_subtracting_band_removes_only_what_it_hit() {
    let mut sel = SelectionState::default();
    sel.marquee(
        0,
        &[
            TargetId::Object(1),
            TargetId::Object(2),
            TargetId::Object(3),
        ],
        false,
    );
    assert_eq!(sel.len(), 3);

    sel.marquee_remove(0, &[TargetId::Object(2)]);
    assert_eq!(
        sel.entries(),
        [
            Selection::object(0, TargetId::Object(1)),
            Selection::object(0, TargetId::Object(3)),
        ]
    );
}

/// **★★ An EMPTY subtracting band changes nothing — it must not clear.**
///
/// The asymmetry with a plain band is deliberate and is the whole safety of the
/// gesture. A plain band that encloses nothing means *"select nothing"*, which
/// is how every editor cancels a selection. A **subtracting** band that hits
/// nothing means *"remove nothing"* — clearing there would make a mis-aimed
/// Ctrl-drag destroy the very selection the operator was trying to refine,
/// which is the opposite of what they asked for.
#[test]
fn a_subtracting_band_that_hits_nothing_does_not_clear_the_selection() {
    let mut sel = SelectionState::default();
    sel.marquee(0, &[TargetId::Object(1), TargetId::Object(2)], false);

    sel.marquee_remove(0, &[]);
    assert_eq!(
        sel.len(),
        2,
        "an empty subtract must be a no-op, never a clear"
    );
}

#[test]
fn a_marquee_replaces_and_a_shift_marquee_extends() {
    let mut sel = SelectionState::default();
    sel.marquee(0, &[TargetId::Object(0)], false);
    assert_eq!(sel.entries(), [Selection::object(0, TargetId::Object(0))]);

    sel.marquee(0, &[TargetId::Object(1)], true);
    assert_eq!(sel.len(), 2);

    sel.marquee(0, &[TargetId::Object(1)], false);
    assert_eq!(sel.entries(), [Selection::object(0, TargetId::Object(1))]);
}

/// A marquee always lands at the Object rung and takes the operator out
/// of any object they were inside.
#[test]
fn a_marquee_ascends_to_the_object_rung() {
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(0),
            node: None,
        },
        false,
        true,
    );
    assert_eq!(sel.level(), SelectionLevel::Part);
    sel.marquee(0, &[TargetId::Object(0), TargetId::Object(1)], false);
    assert_eq!(sel.level(), SelectionLevel::Object);
    assert_eq!(sel.entries()[0].subpath, None);
}

// -----------------------------------------------------------------
// Outlines
// -----------------------------------------------------------------

/// The outline of an entered part is the **part's** box, not the
/// object's. An object-sized rectangle around a part tells the operator
/// they selected the whole thing again, which is the misunderstanding
/// entering the object exists to resolve.
#[test]
fn an_entered_parts_outline_is_the_parts_own_box() {
    let targets = stub(0);
    let mut sel = SelectionState::default();
    sel.click(0, hit_object(0), false, false);
    sel.resolve(Some(&targets), 0, 0);
    let whole = sel.outlines()[0].1;

    sel.click(
        0,
        ClickHit {
            object: Some(TargetId::Object(0)),
            part: Some(1),
            node: None,
        },
        false,
        true,
    );
    sel.resolve(Some(&targets), 0, 0);
    let part = sel.outlines()[0].1;
    assert!(part.width() < whole.width());
    assert!(whole.contains_rect(part));
}

/// The grips sit around the union of the selection, not around one
/// member of it.
#[test]
fn the_grip_box_is_the_union_of_the_selection() {
    let targets = stub(0);
    let mut sel = SelectionState::default();
    assert_eq!(sel.outline_union(), None);
    sel.click(0, hit_object(0), false, false);
    sel.click(0, hit_object(1), true, false);
    sel.resolve(Some(&targets), 0, 0);
    let union = sel
        .outline_union()
        .expect("two selected objects have a union");
    assert!(union.contains_rect(rect(0.0, 0.0, 100.0, 100.0)));
    assert!(union.contains_rect(rect(200.0, 200.0, 50.0, 50.0)));
}

/// ★★★ **A placed object arrives selected, at the Object rung, alone.**
///
/// The operator, 2026-08-26: *"if I add an image I Expect to click on it to
/// resize but dragging doesn't resize."* He was right about the symptom and it
/// was never the resize — a driven check had already proved a selected image
/// resizes from a corner grip (`resize-commit grip=SouthEast sx=0.6810`). It
/// arrived **unselected**, so his first press was a press on unselected paper,
/// which `gesture::meaning` reads as a marquee. He watched a rubber band.
///
/// Three properties, and each is a separate way to get this wrong: the new
/// object is selected; it is the ONLY thing selected; and the rung is Object,
/// not a rung inside it.
#[test]
fn a_placed_object_replaces_the_selection_at_the_object_rung() {
    let mut sel = SelectionState::default();
    sel.marquee(0, &[TargetId::Object(3), TargetId::Object(4)], false);
    assert_eq!(sel.len(), 2, "two things selected before the placement");

    sel.select_placed(0, TargetId::Object(9));

    assert_eq!(
        sel.entries(),
        &[Selection::object(0, TargetId::Object(9))],
        "the placed object must be selected, and must be the only thing selected: what was selected before is what the operator was working on BEFORE they placed this"
    );
    assert_eq!(
        sel.level(),
        SelectionLevel::Object,
        "a placement creates a whole object; arriving inside one is a rung nobody asked for"
    );
}

// ===========================================================================
// The two index spaces
//
// A selection can name a page object or a form-interior leaf, and the whole
// safety property of `TargetId` is that only the first reaches an edit verb.
// These assert the three accessors that make that decidable, because every
// refusal and every disclosure in the application is phrased in terms of them.
// ===========================================================================

/// ★★★ **An empty operand list is not an empty selection**, and the three
/// accessors say which is which.
///
/// This is the property every form-related refusal in the application rests
/// on. `canvas::moving` used to answer a leaf-only selection with
/// `Refusal::NothingSelected`, which contradicted the outline on screen;
/// `status::selected` would have gone silent on it. Both now ask
/// `leaf_indices_on` first, and both would be wrong again if these three
/// answers ever collapsed into each other.
#[test]
fn a_leaf_only_selection_is_not_an_empty_selection() {
    use crate::canvas::target::TargetId;
    let mut sel = SelectionState::default();
    sel.select_only(0, TargetId::Leaf(4), "test");

    assert!(!sel.is_empty(), "something IS selected");
    assert!(
        sel.object_indices_on(0).is_empty(),
        "and nothing in it is an edit operand"
    );
    assert_eq!(sel.leaf_indices_on(0), vec![4], "and this is why");
    assert_eq!(sel.targets_on(0), vec![TargetId::Leaf(4)]);
}

/// The two index spaces do not collide: `objects[4]` and `leaves[4]` are
/// different selections, and each accessor reports only its own.
///
/// A bare `u64` id could not express this at all — which is the argument for
/// `TargetId` being an enum, asserted rather than only written down.
#[test]
fn object_four_and_leaf_four_are_different_things() {
    use crate::canvas::target::TargetId;
    let mut sel = SelectionState::default();
    sel.select_only(0, TargetId::Object(4), "test");
    assert_eq!(sel.object_indices_on(0), vec![4]);
    assert!(sel.leaf_indices_on(0).is_empty());

    sel.select_only(0, TargetId::Leaf(4), "test");
    assert!(sel.object_indices_on(0).is_empty());
    assert_eq!(sel.leaf_indices_on(0), vec![4]);

    assert_ne!(
        Selection::object(0, TargetId::Object(4)),
        Selection::object(0, TargetId::Leaf(4)),
        "two selections that a bare index could not have told apart"
    );
}

/// A mixed selection hands the verbs only the half they can act on, and keeps
/// the other half visible to whatever has to explain the difference.
#[test]
fn a_mixed_selection_splits_into_operands_and_leaves() {
    use crate::canvas::target::TargetId;
    let mut sel = SelectionState::default();
    sel.marquee(
        0,
        &[
            TargetId::Object(7),
            TargetId::Leaf(2),
            TargetId::Object(1),
            TargetId::Leaf(9),
        ],
        false,
    );
    assert_eq!(sel.object_indices_on(0), vec![1, 7], "ascending and unique");
    assert_eq!(sel.leaf_indices_on(0), vec![2, 9]);
    assert_eq!(sel.targets_on(0).len(), 4, "the readout sees all four");
}

/// Deleting is refused for a leaf, structurally, at the one funnel that feeds
/// `EditSession::delete_objects`.
///
/// ★ The consequence if this ever returned the leaf's number: `delete_objects`
/// would resolve it against the **page's** paint order, find a real object
/// there, and delete the wrong thing — silently, because the index is in
/// range. That is the file-corruption failure `TargetId` exists to make
/// unrepresentable, asserted at the place it would have happened.
#[test]
fn a_leaf_is_never_a_delete_operand() {
    use crate::canvas::target::TargetId;
    let mut sel = SelectionState::default();
    sel.select_only(0, TargetId::Leaf(0), "test");
    assert!(sel.deletable_objects_on(0).is_empty());
}
