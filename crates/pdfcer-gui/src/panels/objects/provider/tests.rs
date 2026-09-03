//! # `provider::tests` — the object model, proved against real content streams
//!
//! Every test that was in `provider.rs`'s inline `mod tests`, moved out
//! unchanged, plus the eight that arrived with form-XObject descent.
//!
//! ## Why it is a file rather than an inline module
//!
//! R2. `provider.rs` reached 1,846 lines when the deep hit test and its tests
//! landed, and the rule this project was founded on is that the limit is the
//! signal to find the seam, not to raise the limit — the GUI being replaced
//! reached 25,005 lines in one `main.rs`, and two independent regressions of
//! the same key landed two days apart without either noticing the other.
//!
//! The seam here is the obvious one and the crate already uses it in three
//! other places (`canvas::selection::tests`, `app::state::tests`,
//! `app::actions::apply::tests`): a `#[cfg(test)] mod tests;` declared in the
//! parent, living in its own file, with `use super::*` giving it exactly the
//! access an inline module had. Nothing about visibility changes, so nothing
//! about what these tests can reach changes.
//!
//! ## ★ The two fixtures, and the thing they exist to make possible
//!
//! Most tests here build their model with
//! [`pdfcer_core::vector::decompose`] over a hand-written content stream,
//! whose resolver seam is `NoXObjects` — so `PageObjects::leaves` is **always
//! empty** and not one of them can see whether this provider descends into a
//! form.
//!
//! That is not a hypothetical limitation. The deep hit test landed with the
//! entire workspace suite green, which was a suite reporting nothing about the
//! change that had just been made. The form tests use
//! [`ObjectModelProvider::build_or_reason`] against a real `Document`, which
//! is the only entry point that has a `DocumentView` to descend with, and they
//! were falsified in both directions before being believed: with the shallow
//! `hit_test_point_all` restored, three of them go red.
//!
//! ## `#![cfg(test)]` at the top, and why it is the marker rather than the name
//!
//! Two gates recognise the **inner attribute** as meaning *"none of this is in
//! the shipped binary"* — `check-ui-strings.sh` and `check-theme-colors.sh`
//! — and both state why they match on that rather than on a filename: the
//! property that earns the exemption is not being in the binary, and a
//! filename is a restatement of it that goes stale the moment a third such
//! module is written.
//!
//! Without it, `check-ui-strings` reports every `assert!` message here as
//! un-catalogued operator copy. That happened once already, on 2026-08-18,
//! when `canvas::selection::tests` was split out under the same rule and the
//! gate produced 28 false hits. The noise is the actual hazard: most of
//! pdfcer's old string-gate floor was test assertions, and a split that
//! reintroduced them would train people to ignore the report.
//!
//! ★ **The line gate still counts these lines.** `check-file-size.sh` counts
//! total lines, tests included, on purpose — its own header says so — so
//! this split is not a way of hiding lines from R2. It is the split R2 asked
//! for, taken on the seam that was already there.

#![cfg(test)]

use super::*;
use pdfcer_core::content::ContentStream;
use pdfcer_core::vector::{NoXObjects, decompose};

/// A provider over a content stream, with an identity canvas transform
/// (so canvas space == PDF space and the assertions read directly).
fn provider(src: &[u8]) -> ObjectModelProvider {
    let cs = ContentStream::parse(src.to_vec()).expect("parse");
    let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
    ObjectModelProvider::from_parts(0, objects, Transform::identity())
}

// -----------------------------------------------------------------
// Form XObjects: the operator's case
//
// Everything above builds its model with `decompose`, whose resolver seam
// is `NoXObjects` -- so `PageObjects::leaves` is empty in every one of
// those tests and **not one of them can see** whether this provider
// descends into a form. That is not a hypothetical: the deep hit test
// landed with the whole suite green, which is a suite reporting nothing.
//
// These use `decompose_page` against a real `Document`, which is the only
// entry point that has a `DocumentView` to descend with. The fixtures come
// from the engine's own synthetic tree, through `engine_fixture` -- the
// convention 43 other tests in this crate already use, and the reason a copy
// was not committed here: a duplicate would be one more thing to reconcile at
// fold-in, and the engine's `forms-xobject` directory exists *because* of this
// project's report, so drift between the two would be exactly the wrong drift.
// -----------------------------------------------------------------

/// One page-sized form holding three separate 40 x 40 squares: **one** page
/// object, **three** leaves.
///
/// The engine's own reproduction of *"when I click on one of the objects
/// all I get is the page selected"*. The three squares sit at 10,10-50,50,
/// 80,80-120,120 and 150,150-190,190 in PDF user space, and the gaps between
/// them matter as much as the squares: a click in a gap is a click *inside
/// the form* that must select nothing.
const PAGE_SIZED_FORM: &str = "forms-xobject/page-sized-form.pdf";

/// Form A holds form B holds one square -- containment depth **2**, and an
/// intermediate form that is deliberately not itself a leaf.
const NESTED_FORMS: &str = "forms-xobject/nested-forms.pdf";

/// A provider over page 1 of a real document, with the page's own device
/// transform.
///
/// The transform is `page_device_geometry`'s, not the identity, because
/// that is what `build_or_reason` uses live -- so a canvas point in these
/// tests is a canvas point in the running program. The fixture pages are
/// 200 x 200 with no `/Rotate`, so the map is the plain Y-flip and
/// `canvas_y = 200 - pdf_y`.
fn provider_over(fixture: &str) -> ObjectModelProvider {
    let bytes = std::fs::read(crate::panels::objects::test_support::engine_fixture(
        fixture,
    ))
    .expect("the fixture is readable");
    let doc = pdfcer_core::document::Document::from_bytes(bytes).expect("the fixture parses");
    let view = doc.view();
    let pages = pdfcer_core::page_tree::pages(&doc).expect("the fixture has a page tree");
    ObjectModelProvider::build_or_reason(&view, &pages[0], 0).expect("the page decomposes")
}

/// PDF user space to this fixture's canvas space: a 200 pt page, no
/// rotation, so the Y axis simply flips.
fn canvas(x: f32, y: f32) -> Pos2 {
    Pos2::new(x, 200.0 - y)
}

/// The fixture really does have the shape the assertions below assume.
///
/// Stated as its own test so that a fixture that stopped having a form
/// fails **here**, with a message about the fixture, rather than turning
/// every test under it into a confusing report about hit testing.
#[test]
fn the_form_fixture_has_one_page_object_and_three_leaves() {
    let p = provider_over(PAGE_SIZED_FORM);
    let model = p.page_objects();
    assert_eq!(
        model.objects.len(),
        1,
        "the page's own content stream paints exactly one thing: the form"
    );
    assert_eq!(
        model.leaves.len(),
        3,
        "and three squares are painted from inside it"
    );
    assert_eq!(
        model.diagnostics.form_depth_overflows + model.diagnostics.form_cycles,
        0,
        "nothing was left undescended, so `leaves` is the whole interior"
    );
}

/// THE DEFECT, ASSERTED.
///
/// A click on a square inside the page-sized form selects **that square**,
/// as a leaf -- not the form, and not nothing.
///
/// Falsified before it was believed: with `hit_test_point_all` in place of
/// `hit_test_point_deep` this returns `TargetId::Object(0)`, the form,
/// which is the operator's report reproduced in one line.
#[test]
fn a_click_inside_a_page_sized_form_selects_what_is_drawn_there() {
    let p = provider_over(PAGE_SIZED_FORM);
    // The middle square is 80,80 -> 120,120 in PDF space.
    let hit = p.hit_test(0, canvas(100.0, 100.0), 1.0);
    assert_eq!(
        hit,
        Some(TargetId::Leaf(1)),
        "the square inside the form, addressed in the leaf index space"
    );
    assert!(
        hit.expect("a hit").page_object_index().is_none(),
        "and it carries no page paint-order index, so no edit verb can take it"
    );
}

/// THE HALF THAT IS EASY TO LOSE: a click on blank paper inside the form
/// selects **nothing**.
///
/// This is the assertion that forbids the tempting "fall back to the
/// shallow hit test when the deep one is empty" fix. That fallback would
/// answer this click with the page-sized form -- the operator's original
/// complaint, restored, for the case that produces it most often.
#[test]
fn a_click_on_blank_paper_inside_a_form_selects_nothing() {
    let p = provider_over(PAGE_SIZED_FORM);
    // 65,65 is between the first square (ends at 50) and the second
    // (starts at 80), and inside the form's page-sized /BBox.
    assert_eq!(
        p.hit_test(0, canvas(65.0, 65.0), 1.0),
        None,
        "a /BBox is a clipping extent, not a claim about ink"
    );
}

/// The form is not a candidate, ever -- not even when it is the only thing
/// the page's own stream paints and the click is inside its box.
#[test]
fn a_form_is_never_a_hit_candidate() {
    let p = provider_over(PAGE_SIZED_FORM);
    for (x, y) in [(100.0_f32, 100.0_f32), (65.0, 65.0), (30.0, 30.0)] {
        for target in p.hit_test_all(0, canvas(x, y), 1.0) {
            assert!(
                target.is_leaf(),
                "a page object came back from ({x},{y}); the only page object here is the form"
            );
        }
    }
}

/// A leaf resolves to the box the operator sees outlined -- **the
/// square's**, not the form's page-sized one.
///
/// The outline is the whole visible evidence of what got selected. A leaf
/// whose bounds fell back to its container would draw the page-edge
/// rectangle that made the original defect look like "the page is
/// selected", while having actually selected the right thing -- a fix that
/// is invisible is not a fix.
#[test]
fn a_leaf_outlines_its_own_square_and_not_the_form() {
    let p = provider_over(PAGE_SIZED_FORM);
    let r = p
        .bounds(0, TargetId::Leaf(1))
        .expect("the middle square has bounds");
    assert!(
        (r.width() - 40.0).abs() < 0.01 && (r.height() - 40.0).abs() < 0.01,
        "expected the 40x40 square, got {r:?}"
    );
    // A stale leaf index is `None`, not a panic -- the same contract the
    // page-object side has, for the same reason.
    assert_eq!(p.bounds(0, TargetId::Leaf(99)), None);
    assert_eq!(p.bounds(1, TargetId::Leaf(1)), None, "another page");
}

/// The container is reachable as a deliberate second act, and it lands on
/// the **page** index space -- which is what makes it editable.
#[test]
fn the_containing_form_resolves_to_an_editable_page_object() {
    let p = provider_over(PAGE_SIZED_FORM);
    let form = p
        .containing_form(0, TargetId::Leaf(1))
        .expect("the leaf is inside a form");
    assert_eq!(form, TargetId::Object(0));
    assert_eq!(
        form.page_object_index(),
        Some(0),
        "and the container IS an edit operand -- that is the point of offering it"
    );
    // A page object has no container, and a stale leaf has none either.
    assert_eq!(p.containing_form(0, TargetId::Object(0)), None);
    assert_eq!(p.containing_form(0, TargetId::Leaf(99)), None);
    assert_eq!(p.containing_form(1, TargetId::Leaf(1)), None);
}

/// A marquee that encloses a square inside a form selects it, so the two
/// gestures that both mean "select this" agree.
#[test]
fn a_marquee_encloses_objects_inside_a_form() {
    let p = provider_over(PAGE_SIZED_FORM);
    // A canvas rect covering PDF 70..130 in both axes: the middle square
    // only.
    let rect = Rect::from_min_max(canvas(70.0, 130.0), canvas(130.0, 70.0));
    assert_eq!(
        p.hit_test_rect(0, rect, MarqueeMode::Enclosed),
        vec![TargetId::Leaf(1)]
    );
    // And a marquee that only grazes it takes nothing -- enclosure, not
    // touching, on both index spaces.
    let grazing = Rect::from_min_max(canvas(100.0, 130.0), canvas(130.0, 100.0));
    assert!(
        p.hit_test_rect(0, grazing, MarqueeMode::Enclosed)
            .is_empty()
    );
    // ★ …and the SAME grazing band as a crossing window takes it. Added
    // 2026-09-02 with O88: without this the enclosure assertion above is
    // half a claim, because it cannot distinguish "enclosure is enforced"
    // from "the deep index space is unreachable by any marquee at all".
    assert_eq!(
        p.hit_test_rect(0, grazing, MarqueeMode::Touched),
        vec![TargetId::Object(0), TargetId::Leaf(1)],
        "a crossing window must reach a form's interior, or the two index spaces \
         disagree about what a right-to-left drag means"
    );
    // ★★ …and `Object(0)` — the page-sized WRAPPER — comes with it, which is
    // the honest answer from THIS function and is NOT what the operator gets.
    //
    // A crossing band touches a page-sized form wherever it is drawn, so on a
    // wrapped drawing every right-to-left drag would otherwise include the whole
    // sheet. It is dropped one layer up, by
    // `canvas::marquee::without_page_wrappers`, using the same
    // `container_is_worth_selecting` rule the click ladder already applies --
    // and it is dropped THERE rather than here because this function is a hit
    // test and that is a selection policy.
    //
    // ★ Asserting the raw answer keeps the seam visible. An expectation written
    // as the filtered result would pass whether the filter existed or not.
}

/// Nesting is reported by depth, so a shell can say "three wrappers down"
/// rather than only "inside something".
///
/// `nested-forms.pdf` is form A holding form B holding one square. The
/// intermediate form is deliberately **not** a leaf: it is a container,
/// and counting it as content would make "how many objects are in here"
/// wrong by one per level.
#[test]
fn a_nested_leaf_reports_its_full_containment_chain() {
    let p = provider_over(NESTED_FORMS);
    let model = p.page_objects();
    assert_eq!(model.leaves.len(), 1, "one square, two wrappers");
    assert_eq!(
        model.leaves[0].containment.len(),
        2,
        "outermost first, ending with the form the square is directly in"
    );
    // The container offered is the OUTERMOST one, because that is the one
    // with an index in the page's paint order.
    assert_eq!(
        p.containing_form(0, TargetId::Leaf(0)),
        Some(TargetId::Object(model.leaves[0].paint_order as u64))
    );
}

#[test]
fn click_inside_a_filled_rectangle_returns_its_target() {
    // One filled rectangle 10..90 square; a click at its centre hits it.
    let p = provider(b"10 10 80 80 re f");
    let hit = p.hit_test(0, Pos2::new(50.0, 50.0), 3.0);
    assert_eq!(hit, Some(TargetId::Object(0)));
    // A click on empty canvas misses.
    assert_eq!(p.hit_test(0, Pos2::new(200.0, 200.0), 3.0), None);
    // A query for a different page misses regardless.
    assert_eq!(p.hit_test(1, Pos2::new(50.0, 50.0), 3.0), None);
}

/// The regression test for the zoom-inverted-tolerance bug: a click that
/// misses a hairline stroke by 4 canvas units must MISS at a tight
/// tolerance and HIT at a forgiving one.
///
/// This is what makes the fix meaningful rather than cosmetic. Before it,
/// the tolerance was hard-coded at 3.0 canvas units at every zoom, so at
/// "Fit page" (~0.5x on a letter page in a typical window) the operator's
/// real on-screen catch radius was ~1.5 px and thin geometry could not be
/// clicked at all. The tolerance now arrives from the caller, scaled by
/// `1 / zoom`, which keeps the on-screen radius constant.
///
/// The *other half* of that law — that the caller's conversion really is
/// `1 / zoom` — is asserted in `canvas/` at S4, where the conversion
/// lives. See this module's header, "What changed at salvage" §4.
#[test]
fn selection_tolerance_is_honoured_per_query_not_baked_in() {
    // A zero-width horizontal line at y=20; click 4 units above it.
    let p = provider(b"10 20 m 100 20 l S");
    let near_miss = Pos2::new(50.0, 24.0);

    // Tight tolerance (the old zoomed-out effective radius): a miss.
    assert_eq!(p.hit_test(0, near_miss, 1.5), None);
    // Forgiving tolerance (what a zoomed-out click now supplies): a hit.
    assert_eq!(p.hit_test(0, near_miss, 6.0), Some(TargetId::Object(0)));

    // A degenerate tolerance must NOT silently disable selection — it
    // falls back to the fixed canvas-space value, so a click within
    // 3.0 units still lands.
    assert_eq!(
        p.hit_test(0, Pos2::new(50.0, 22.0), 0.0),
        Some(TargetId::Object(0))
    );
    assert_eq!(
        p.hit_test(0, Pos2::new(50.0, 22.0), f64::NAN),
        Some(TargetId::Object(0))
    );
    assert_eq!(
        p.hit_test(0, Pos2::new(50.0, 22.0), -1.0),
        Some(TargetId::Object(0)),
        "a negative tolerance is degenerate too, and must fall back"
    );
}

#[test]
fn bounds_round_trips_the_object_bbox_into_canvas_space() {
    let p = provider(b"10 10 80 80 re f");
    let r = p.bounds(0, TargetId::Object(0)).expect("bounds");
    // Under the identity transform the canvas rect is the PDF bbox.
    assert!((r.min.x - 10.0).abs() < 1e-3 && (r.min.y - 10.0).abs() < 1e-3);
    assert!((r.max.x - 90.0).abs() < 1e-3 && (r.max.y - 90.0).abs() < 1e-3);
    // A stale target id resolves to nothing rather than panicking.
    assert_eq!(p.bounds(0, TargetId::Object(99)), None);
}

#[test]
fn marquee_encloses_only_fully_contained_objects() {
    // Two rectangles; a marquee over the first only encloses it.
    let p = provider(b"10 10 20 20 re f 200 200 20 20 re f");
    let hits = p.hit_test_rect(
        0,
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0)),
        MarqueeMode::Enclosed,
    );
    assert_eq!(hits, vec![TargetId::Object(0)]);
    // A marquee spanning both encloses both.
    let both = p.hit_test_rect(
        0,
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(300.0, 300.0)),
        MarqueeMode::Enclosed,
    );
    assert_eq!(both, vec![TargetId::Object(0), TargetId::Object(1)]);
    // Wrong page: nothing.
    assert!(
        p.hit_test_rect(1, Rect::EVERYTHING, MarqueeMode::Enclosed)
            .is_empty()
    );
}

#[test]
fn a_text_object_is_selectable_by_its_bbox() {
    // A text object is bbox-only but still a valid target.
    let p = provider(b"BT /F1 12 Tf 40 40 Td (Hi) Tj ET");
    // The show origin (40,40) is inside the inflated text bbox.
    assert!(p.hit_test(0, Pos2::new(40.0, 40.0), 3.0).is_some());
}

/// Overlapping objects are all reported, front-most first, in CANVAS
/// space — the input click-through cycling steps through. Without this
/// the covered rectangle here is unselectable by any click.
#[test]
fn overlapping_objects_are_all_reported_front_most_first() {
    // A small filled rectangle painted first, then a big one over it.
    let p = provider(b"40 40 20 20 re f 0 0 100 100 re f");
    let hits = p.hit_test_all(0, Pos2::new(50.0, 50.0), 3.0);
    assert_eq!(hits, vec![TargetId::Object(1), TargetId::Object(0)]);
    // The topmost query is exactly that list's head.
    assert_eq!(
        p.hit_test(0, Pos2::new(50.0, 50.0), 3.0),
        Some(TargetId::Object(1))
    );
    // Only the cover is under a point outside the covered object.
    assert_eq!(
        p.hit_test_all(0, Pos2::new(5.0, 5.0), 3.0),
        vec![TargetId::Object(1)]
    );
    // A miss is an empty list, and a wrong page is too.
    assert!(p.hit_test_all(0, Pos2::new(500.0, 500.0), 3.0).is_empty());
    assert!(p.hit_test_all(1, Pos2::new(50.0, 50.0), 3.0).is_empty());
}

/// The tolerance fallback applies to the all-hits query as well: a
/// degenerate tolerance must not silently make cycling find nothing when
/// plain selection would still have found something.
#[test]
fn a_degenerate_tolerance_falls_back_for_the_all_hits_query_too() {
    let p = provider(b"10 20 m 100 20 l S");
    let near = Pos2::new(50.0, 22.0);
    assert_eq!(p.hit_test_all(0, near, 0.0), vec![TargetId::Object(0)]);
    assert_eq!(p.hit_test_all(0, near, f64::NAN), vec![TargetId::Object(0)]);
}

#[test]
fn page_objects_feeds_the_snap_engine_from_the_one_decomposition() {
    // The shared accessor: a consumer reads the provider's
    // already-decomposed objects (no second `decompose_page`) and
    // resolves a query in the same PDF/page space `PageObjects` stores.
    use pdfcer_core::vector::{Point, SnapConfig, SnapKind, snap_candidates};
    let p = provider(b"10 20 m 100 20 l S");
    let model = p.page_objects();
    let cands = snap_candidates(Point::new(11.0, 21.0), &SnapConfig::new(5.0), model);
    assert_eq!(cands[0].kind, SnapKind::Endpoint);
    assert_eq!(cands[0].point, Point::new(10.0, 20.0));
}

/// **The part rung dispatches by object kind, and images have none.**
///
/// This is what the Objects panel's tree builder relies on to decide
/// whether a row gets an expander. A path with subpaths expands, a text
/// object with runs expands, an image is a leaf — and the panel asks one
/// question rather than matching on `VectorObject` itself, which is the
/// duplicated-predicate drift [`ObjectModelProvider::part_hits`]'s own
/// docs warn about.
#[test]
fn part_kind_and_part_count_answer_for_every_object_kind() {
    let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
    assert_eq!(p.part_kind(0), Some(PartKind::Subpath));
    assert_eq!(p.part_count(0), 2);

    let t = provider(b"BT /F1 12 Tf 40 40 Td (Hi) Tj ET");
    assert_eq!(t.part_kind(0), Some(PartKind::Run));
    assert_eq!(t.part_count(0), t.text_run_count(0));
    assert_eq!(t.subpath_count(0), 0, "a text object has no subpaths");

    let i = provider(b"q 100 0 0 50 10 10 cm BI /W 1 /H 1 /CS /G /BPC 8 ID \x00 EI Q");
    assert_eq!(i.part_kind(0), None, "an image has no part rung");
    assert_eq!(i.part_count(0), 0);

    // Out of range is a leaf, not a panic.
    assert_eq!(p.part_kind(99), None);
    assert_eq!(p.part_count(99), 0);
}

/// A part hit dispatches to the right query for the object's kind.
#[test]
fn part_hits_dispatches_to_the_kind_specific_query() {
    let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
    // A press on the second part must name the second part.
    assert_eq!(p.part_hits(0, Pos2::new(105.0, 5.0), 3.0).first(), Some(&1));
    // An image has no parts, so a press anywhere over it names none.
    let i = provider(b"q 100 0 0 50 10 10 cm BI /W 1 /H 1 /CS /G /BPC 8 ID \x00 EI Q");
    assert!(i.part_hits(0, Pos2::new(50.0, 30.0), 3.0).is_empty());
}

/// A part's outline is the PART's box, not the object's.
///
/// The whole reason the rung exists: an object-sized rectangle around a
/// part tells the operator they selected the whole thing again.
#[test]
fn a_part_outline_is_smaller_than_its_objects() {
    let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
    let part = p.part_bounds_canvas(0, 1).expect("part 1 has bounds");
    let whole = p
        .bounds(0, TargetId::Object(0))
        .expect("the object has bounds");
    assert!(
        part.width() < whole.width(),
        "part {part:?} is not narrower than object {whole:?}"
    );
    assert!(whole.contains_rect(part));
    // An out-of-range part is `None`, never the object's own box —
    // returning the object there is how "the second level does nothing"
    // becomes "the second level lies".
    assert_eq!(p.part_bounds_canvas(0, 9), None);
}
