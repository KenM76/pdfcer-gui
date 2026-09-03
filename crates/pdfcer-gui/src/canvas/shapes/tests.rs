//! Tests for [`super`] — the live geometry preview.
//!
//! ## What is worth asserting here, and what is not
//!
//! The **painting** is not tested here. A stroke width and a colour are pixels,
//! and this project's standing rule R1 is that pixels have exactly one oracle:
//! a rendered screenshot from `tools/ui-verify`. A unit test that asserted the
//! painter was called would assert that the code calls itself.
//!
//! What is tested is the part a screenshot cannot check cheaply and a future
//! edit can silently break:
//!
//! 1. **The anchor walk agrees with the provider's**, so a node index means the
//!    same thing here as it does to `move_nodes`;
//! 2. **The transform reaches the geometry**, so a preview cannot silently be
//!    the unmoved shape;
//! 3. **The caps fire and say so**, so a big selection degrades to the bounding
//!    outline instead of to a stall.

// ★ The marker `check-ui-strings.sh` and `check-theme-colors.sh` both read: a
// whole file gated out of release builds. Every string below is an assertion
// message, and the property that earns the exemption is "not in the shipped
// binary" rather than anything about the filename.
#![cfg(test)]

use std::collections::BTreeSet;

use super::*;
use crate::app::state::{OpenDoc, open_local_fixture};
use crate::panels::objects::provider::TargetId;

/// The local fixture with real path geometry to walk.
///
/// `polyline-nodes.pdf` exists in this repository precisely because no engine
/// fixture carried a node-draggable polyline — see `open_local_fixture`'s
/// header on why there are two fixture roots.
fn polyline() -> OpenDoc {
    open_local_fixture("polyline-nodes.pdf")
}

/// The first path object on page 0, if there is one.
fn first_path(doc: &OpenDoc) -> Option<usize> {
    let provider = doc.page_objects()?;
    let model = provider.page_objects();
    (0..model.objects.len()).find(|i| matches!(model.objects.get(*i), Some(VectorObject::Path(_))))
}

/// ★★★ A node index means the same thing here as it does to `move_nodes`.
///
/// # Why this test is the important one in this file
///
/// [`super::with_nodes_moved`] displaces *anchor number N* by walking
/// `page_subpaths()` and counting `start` then one per segment. The provider
/// counts the same way, through `Subpath::anchors()`, and the engine's
/// `move_node` counts the same way again.
///
/// **Three implementations of one enumeration.** R74's rule is that a matching
/// rule must not be re-derived in the shell; index arithmetic that has to agree
/// with another module's is the same hazard in smaller clothes, and the failure
/// it produces is the worst kind — the preview bends the line at one end and the
/// commit bends it at the other, and *both look deliberate*.
///
/// ⇒ So the agreement is asserted rather than reasoned. If the engine ever
/// changes what an anchor index counts, this goes red here rather than shipping
/// a preview that lies.
#[test]
fn the_walk_agrees_with_the_providers_anchor_numbering() {
    let doc = polyline();
    let Some(object) = first_path(&doc) else {
        // Not a failure: a fixture without a path has nothing to disagree about.
        // Said out loud rather than passed silently — a test that can only
        // report one outcome cannot detect the thing it was added to detect.
        panic!("polyline-nodes.pdf carries no path object, so this test measured nothing");
    };

    let points: Vec<(usize, Point)> = {
        let provider = doc.page_objects().expect("a decomposition");
        provider.object_node_points(object)
    };
    assert!(
        !points.is_empty(),
        "the provider reports no anchors for object {object}, so the walk below has nothing to \
         agree with"
    );

    // Move exactly one anchor, one at a time, and check the moved point lands
    // where the provider said that anchor was.
    for (index, expected) in &points {
        let mut only = BTreeSet::new();
        only.insert(*index);
        let preview = with_nodes_moved(
            &doc.page_objects().expect("a decomposition"),
            TargetId::Object(object as u64),
            &only,
            10.0,
            20.0,
        )
        .expect("the preview builds");
        let moved: Vec<Point> = preview.shapes[0]
            .subpaths
            .iter()
            .flat_map(|sp| sp.anchors().collect::<Vec<_>>())
            .collect();
        let want = Point {
            x: expected.x + 10.0,
            y: expected.y + 20.0,
        };
        assert!(
            moved
                .iter()
                .any(|p| (p.x - want.x).abs() < 1e-6 && (p.y - want.y).abs() < 1e-6),
            "anchor {index} sits at {expected:?} per the provider, so moving it by (10, 20) must \
             put a point at {want:?} — and the preview's anchors are {moved:?}. The shell's walk \
             and the provider's disagree about what anchor {index} IS, which means the preview \
             would bend one end of the line and the commit the other."
        );
    }
}

/// ★★ A transform must reach the geometry.
///
/// The failure this catches is a preview that builds, traces, paints, and shows
/// the shape exactly where it already was — which looks like "the preview is not
/// working" and is indistinguishable from the feature being absent.
#[test]
fn a_translation_moves_every_point_by_exactly_the_translation() {
    let doc = polyline();
    let Some(object) = first_path(&doc) else {
        panic!("polyline-nodes.pdf carries no path object");
    };
    let before = {
        let provider = doc.page_objects().expect("a decomposition");
        provider.object_node_points(object)
    };

    let preview = transformed(
        &doc.page_objects().expect("a decomposition"),
        &[TargetId::Object(object as u64)],
        Matrix::translate(7.0, -3.0),
    )
    .expect("the preview builds");
    let after: Vec<Point> = preview.shapes[0]
        .subpaths
        .iter()
        .flat_map(|sp| sp.anchors().collect::<Vec<_>>())
        .collect();

    assert_eq!(
        before.len(),
        after.len(),
        "the preview has a different number of anchors from the object it previews"
    );
    for ((_, was), now) in before.iter().zip(&after) {
        assert!(
            (now.x - (was.x + 7.0)).abs() < 1e-6 && (now.y - (was.y - 3.0)).abs() < 1e-6,
            "a point at {was:?} translated by (7, -3) must land at ({}, {}), not {now:?}",
            was.x + 7.0,
            was.y - 3.0
        );
    }
}

/// ★ An identity transform is a preview of the object exactly as it is.
///
/// Not a tautology: it is the assertion that `page_subpaths()` and the
/// provider's own numbers describe the same shape, so a preview built with no
/// gesture in progress would sit precisely on top of the rendered object rather
/// than a fraction away from it. A preview that is half a point out looks like a
/// rendering bug.
#[test]
fn an_identity_transform_lands_on_the_object() {
    let doc = polyline();
    let Some(object) = first_path(&doc) else {
        panic!("polyline-nodes.pdf carries no path object");
    };
    let before = {
        let provider = doc.page_objects().expect("a decomposition");
        provider.object_node_points(object)
    };
    let preview = transformed(
        &doc.page_objects().expect("a decomposition"),
        &[TargetId::Object(object as u64)],
        Matrix::IDENTITY,
    )
    .expect("the preview builds");
    let after: Vec<Point> = preview.shapes[0]
        .subpaths
        .iter()
        .flat_map(|sp| sp.anchors().collect::<Vec<_>>())
        .collect();
    for ((_, was), now) in before.iter().zip(&after) {
        assert!(
            (now.x - was.x).abs() < 1e-6 && (now.y - was.y).abs() < 1e-6,
            "the identity preview of a point at {was:?} is {now:?}"
        );
    }
}

/// ★★ Asking for more objects than the cap allows returns a **bounded** preview
/// that says it was bounded — never an unbounded one, and never nothing.
///
/// Both halves matter. An unbounded preview turns the gesture this feature
/// exists to smooth into the slowest thing in the program; a preview that
/// silently returned nothing would read as the feature being broken on exactly
/// the drawings it was built for.
#[test]
fn a_selection_past_the_cap_is_bounded_and_says_so() {
    let doc = polyline();
    let count = doc
        .page_objects()
        .map_or(0, |p| p.page_objects().objects.len());
    // Ask for far more indices than the page has; the out-of-range ones are
    // skipped, and the cap is what this asserts.
    let asked: Vec<TargetId> = (0..MAX_OBJECTS + 10)
        .map(|i| TargetId::Object(i as u64))
        .collect();
    let preview = transformed(
        &doc.page_objects().expect("a decomposition"),
        &asked,
        Matrix::IDENTITY,
    )
    .expect("the preview builds");
    assert!(
        preview.capped,
        "asking for {} objects on a page of {count} must report `capped`, or a marquee across a \
         CAD sheet will paint thousands of paths every frame with nothing saying why it is slow",
        asked.len()
    );
    assert!(
        preview.shapes.len() <= MAX_OBJECTS,
        "the cap did not bound the preview: {} shapes",
        preview.shapes.len()
    );
}

/// The stroke width follows a scale, and does not collapse under a rotation.
///
/// ★ The rotation half is the one worth having. Reading `a` and `d` off the
/// matrix — the obvious implementation — reports a shape rotated by 90° as
/// having zero width, so the preview of a rotate gesture would fade out as it
/// turned. `average_scale` uses the axis lengths instead.
#[test]
fn the_stroke_width_survives_a_rotation_and_follows_a_scale() {
    let quarter = std::f64::consts::FRAC_PI_2;
    let rotate = Matrix {
        a: quarter.cos(),
        b: quarter.sin(),
        c: -quarter.sin(),
        d: quarter.cos(),
        e: 0.0,
        f: 0.0,
    };
    assert!(
        (average_scale(rotate) - 1.0).abs() < 1e-9,
        "a pure rotation must not change the stroke width, and this one scales it by {}",
        average_scale(rotate)
    );
    assert!(
        (average_scale(Matrix::scale(2.0, 4.0)) - 3.0).abs() < 1e-9,
        "a 2x/4x scale should report the mean, 3"
    );
    // Degenerate input falls back to 1 rather than to 0 or NaN: a preview with a
    // zero-width stroke is invisible, which is the one outcome worse than a
    // slightly wrong one.
    assert!((average_scale(Matrix::scale(0.0, 0.0)) - 1.0).abs() < 1e-9);
}
