//! # `panels::objects::provider_node_rung_tests` — the Part and Node rungs, on real geometry
//!
//! The second of `provider.rs`'s two inline test modules, moved out for R2
//! with its contents unchanged. Kept **separate** from
//! `panels::objects::provider_tests` rather than
//! merged, because it was separate before the move and merging two test
//! modules while relocating them would make a review of the move
//! indistinguishable from a review of a rewrite.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/objects/provider_node_rung_tests.md`.

#![cfg(test)]

use crate::panels::objects::provider::*;
use egui::Pos2;
use pdfcer_core::vector::{Matrix, Point};
use pdfcer_render::tiny_skia::Transform;

use pdfcer_core::content::ContentStream;
use pdfcer_core::vector::{Handle, NoXObjects, decompose};

fn provider(src: &[u8]) -> ObjectModelProvider {
    let cs = ContentStream::parse(src.to_vec()).expect("parse");
    let objects = decompose(&cs, Matrix::IDENTITY, &NoXObjects);
    ObjectModelProvider::from_parts(0, objects, Transform::identity())
}

/// **Node indices stay OBJECT-scoped across a subpath boundary.**
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
#[test]
fn a_straight_part_has_no_handles_at_all() {
    let p = provider(b"0 0 m 10 0 l 20 0 l S");
    assert!(p.subpath_handle_points(0, 0).is_empty());
}

/// `v` and `y` resolve to explicit control points before they get here.
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
