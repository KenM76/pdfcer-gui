//! # `provider::node_rung_tests` — the Part and Node rungs, on real geometry
//!
//! The second of `provider.rs`'s two inline test modules, moved out for R2
//! with its contents unchanged. Kept **separate** from
//! [`super::tests`](crate::panels::objects::provider::tests) rather than
//! merged, because it was separate before the move and merging two test
//! modules while relocating them would make a review of the move
//! indistinguishable from a review of a rewrite.
//!
//! Its subject is the two rungs below the object: which subpath a click lands
//! in, which anchor is nearest, and — the law it exists for — that node
//! indices stay **object-scoped** across a part boundary, because that is the
//! space `vector::anchor_count` reports and `pdfcer node-move --node N`
//! addresses. A second numbering would make the number pdfcer shows disagree
//! with the number the operator can act on.
//!
//! ★ Everything here addresses the **page's** paint order. There is no
//! form-interior equivalent, and that is deliberate rather than missing: the
//! part and node rungs exist to act on geometry, `part_hits`, `part_bounds`
//! and `nearest_node` all index `PageObjects::objects`, and
//! `FormLeaf::is_editable` is `false` for every leaf the engine produces. The
//! ladder stops at the Object rung for a leaf, and it stops there because the
//! address space runs out — see `TargetId::page_object_index`.
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

fn provider(src: &[u8]) -> ObjectModelProvider {
    let cs = ContentStream::parse(src.to_vec()).expect("parse");
    let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
    ObjectModelProvider::from_parts(0, objects, Transform::identity())
}

/// **Node indices stay OBJECT-scoped across a subpath boundary.**
///
/// This is decision 025 §1.3(b) made testable. The pick set is scoped to
/// one part, but the numbering is not — because the number pdfcer shows
/// and the number `pdfcer node-move --node N` addresses have to be the
/// same number. A subpath-scoped index would restart at 0 on the second
/// part and quietly address a point in the first.
///
/// The Objects panel's point rows print these numbers, which is what
/// makes this a live invariant at S3 rather than an S4 one.
#[test]
fn the_second_parts_points_keep_counting_from_the_first() {
    // Two parts of two anchors each: indices 0,1 then 2,3.
    let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
    let first: Vec<usize> = p
        .subpath_node_points(0, 0)
        .into_iter()
        .map(|(i, _)| i)
        .collect();
    let second: Vec<usize> = p
        .subpath_node_points(0, 1)
        .into_iter()
        .map(|(i, _)| i)
        .collect();
    assert_eq!(first, vec![0, 1]);
    assert_eq!(
        second,
        vec![2, 3],
        "the second part must continue the object's numbering, not restart"
    );
}

/// The whole object's flat list agrees with the per-part lists
/// concatenated.
///
/// Two functions walk the same anchors in the same order and both hand
/// out object-scoped indices. If they ever disagreed, a multi-node drag
/// would move a different point from the one the panel row named — and
/// nothing about that looks wrong at the moment it happens.
#[test]
fn the_object_wide_point_list_matches_the_parts_concatenated() {
    let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
    let mut per_part = Vec::new();
    for part in 0..p.subpath_count(0) {
        per_part.extend(p.subpath_node_points(0, part));
    }
    assert_eq!(p.object_node_points(0), per_part);
}

/// The pick set contains ONLY the named part's points.
///
/// The whole reason the rung exists: a measured CAD object holds 6,681
/// anchors, and offering all of them as a grab target is what made the
/// old ungated gesture unpredictable.
#[test]
fn a_parts_pick_set_excludes_every_other_part() {
    let p = provider(b"0 0 m 10 0 l 100 5 m 110 5 l S");
    let pts: Vec<Point> = p
        .subpath_node_points(0, 1)
        .into_iter()
        .map(|(_, q)| q)
        .collect();
    assert_eq!(pts.len(), 2);
    assert!(
        pts.iter().all(|q| q.x >= 100.0),
        "part 1's pick set must not contain part 0's points: {pts:?}"
    );
}

/// **A cubic's two control points belong to DIFFERENT nodes** — the thing
/// most likely to be implemented backwards.
///
/// Segment k runs from anchor k to anchor k+1, so `c1` shapes the curve
/// LEAVING anchor k and `c2` shapes the curve ARRIVING at anchor k+1.
/// Assigning both to one node would look plausible, draw two handles in
/// roughly the right place, and make every handle drag move the wrong end
/// of the curve.
#[test]
fn a_cubics_two_handles_belong_to_the_nodes_at_its_two_ends() {
    // m(0,0) then c with c1=(10,40) c2=(60,40) to=(70,0).
    // Anchors: 0 -> (0,0), 1 -> (70,0).
    let p = provider(b"0 0 m 10 40 60 40 70 0 c S");
    let hs = p.subpath_handle_points(0, 0);
    assert_eq!(hs.len(), 2, "one cubic contributes exactly two handles");

    let outgoing = hs
        .iter()
        .find(|(_, s, _)| *s == Handle::Outgoing)
        .expect("c1");
    assert_eq!(outgoing.0, 0, "c1 shapes the curve LEAVING anchor 0");
    assert_eq!(outgoing.2, Point::new(10.0, 40.0));

    let incoming = hs
        .iter()
        .find(|(_, s, _)| *s == Handle::Incoming)
        .expect("c2");
    assert_eq!(incoming.0, 1, "c2 shapes the curve ARRIVING at anchor 1");
    assert_eq!(incoming.2, Point::new(60.0, 40.0));
}

/// **A straight segment contributes no handle, and none is invented.**
///
/// pdfcer refuses to turn a line into a curve without being asked, so the
/// absence must show up as nothing drawn — not as a placeholder sitting
/// on the node, which would advertise an edit that will be refused.
#[test]
fn a_straight_part_has_no_handles_at_all() {
    let p = provider(b"0 0 m 10 0 l 20 0 l S");
    assert!(p.subpath_handle_points(0, 0).is_empty());
}

/// `v` and `y` resolve to explicit control points before they get here.
///
/// Worth pinning because the GUI would otherwise need to know about the
/// short spellings, and getting `v` (first control = current point) and
/// `y` (second control = endpoint) confused is the classic error in this
/// operator family.
#[test]
fn the_short_curve_spellings_still_yield_two_handles() {
    // `v`: c1 is implicitly the current point (0,0), c2 = (60,40).
    let p = provider(b"0 0 m 60 40 70 0 v S");
    let hs = p.subpath_handle_points(0, 0);
    assert_eq!(hs.len(), 2, "`v` is a cubic and has both handles resolved");
    let outgoing = hs
        .iter()
        .find(|(_, s, _)| *s == Handle::Outgoing)
        .expect("c1");
    assert_eq!(
        outgoing.2,
        Point::new(0.0, 0.0),
        "`v`'s first control point IS the current point"
    );
}

/// A handle grab resolves to the node it belongs to, not to the nearest
/// node in space.
#[test]
fn grabbing_a_handle_names_its_own_node() {
    let p = provider(b"0 0 m 10 40 60 40 70 0 c S");
    // Press right on c2 = (60,40), which is far nearer anchor 1 (70,0)
    // than anchor 0 — and is c2, so it must report node 1 / Incoming.
    let hit = p.nearest_handle(0, 0, Point::new(60.0, 40.0), 2.0);
    assert_eq!(hit, Some((1, Handle::Incoming)));
}

/// A node pick resolves to the nearest anchor within tolerance, and
/// ties go to the lower index.
#[test]
fn a_node_pick_takes_the_nearest_anchor_and_ties_go_low() {
    let p = provider(b"0 0 m 100 0 l S");
    assert_eq!(p.nearest_node(0, 0, Pos2::new(2.0, 0.0), 5.0), Some(0));
    assert_eq!(p.nearest_node(0, 0, Pos2::new(98.0, 0.0), 5.0), Some(1));
    // Exactly halfway: the lower index wins.
    assert_eq!(p.nearest_node(0, 0, Pos2::new(50.0, 0.0), 60.0), Some(0));
    // Out of tolerance: nothing, rather than the nearest regardless.
    assert_eq!(p.nearest_node(0, 0, Pos2::new(50.0, 0.0), 5.0), None);
}

/// An out-of-range part yields nothing rather than panicking or wrapping.
#[test]
fn an_out_of_range_part_yields_no_points_or_handles() {
    let p = provider(b"0 0 m 10 0 l S");
    assert!(p.subpath_node_points(0, 9).is_empty());
    assert!(p.subpath_handle_points(0, 9).is_empty());
    assert!(p.object_node_points(9).is_empty());
    assert!(p.object_sample_points(9).is_empty());
}
