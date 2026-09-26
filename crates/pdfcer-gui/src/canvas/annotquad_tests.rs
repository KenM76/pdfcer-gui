//! # `canvas::annotquad` tests — what is left to test once the engine owns the
//! algorithm
//!
//! ## THESE SHRANK ON PURPOSE, AND THE DELETION IS THE FINDING
//!
//!
//! **They were deleted with their subject, not ported.** Re-asserting the
//! engine's own contract from here would be this shell keeping a private
//! opinion about somebody else's normative algorithm — which is exactly the
//! divergence the workaround was filed to end. The engine has its own tests for
//! each of those cases (five in `pdfcer-render`'s `appearance_placement.rs`),
//! and it pins something this shell could not: that the bearing of the first
//! placed edge equals `appearance_rotation_degrees()` on the same annotation.
//!
//! ⇒ **What is worth testing here is what this module still decides**, and that
//! is now two things: the `is_upright` predicate the canvas branches on, and
//! that a real turned annotation arrives through the adapter with sane corners
//! and an angle at all. Plus the tripwire, inverted.
//!
//! ## What these still cannot prove
//!
//! That the operator sees an angled outline. Every test here calls a function
//! directly; whether the painter maps those corners through the right
//! projection is `tools/ui-verify`'s `rotating_a_markup_turns_it`. R1 is not
//! relaxed.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/annotquad/tests.md`.

#![cfg(test)]

use crate::canvas::annotquad::*;
use pdfcer_core::annot_author::{Color, MarkupSpec};
use pdfcer_core::object::ObjId;

/// The authored rectangle: deliberately **not square**, so a bug that swapped
/// width for height, or that returned an axis-aligned bounding box, is visible
/// rather than accidentally correct.
const W: f64 = 140.0;
const H: f64 = 60.0;
const X0: f64 = 200.0;
const Y0: f64 = 400.0;

/// The centre of that rectangle — the pivot every rotation here turns about.
const PIVOT: (f64, f64) = (X0 + W / 2.0, Y0 + H / 2.0);

/// A real document with one `/Square` markup authored on page 1, turned by
/// `degrees`, and its id.
fn turned(degrees: f64) -> (crate::app::state::OpenDoc, ObjId) {
    let mut doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
    let session =
        std::sync::Arc::get_mut(&mut doc.session).expect("the fixture is this test's sole owner");
    let id = session
        .add_markup(
            0,
            &MarkupSpec::Square {
                rect: pdfcer_core::page_tree::Rect {
                    llx: X0,
                    lly: Y0,
                    urx: X0 + W,
                    ury: Y0 + H,
                },
                border: Some(Color::Rgb(0.0, 0.0, 0.0)),
                interior: None,
                border_width: 2.0,
                border_effect: None,
            },
        )
        .expect("the markup is authored");
    if degrees != 0.0 {
        session
            .rotate_annotation(id, PIVOT, degrees)
            .expect("a `/Square` rotates");
    }
    (doc, id)
}

/// Longest and shortest edge of a quadrilateral — what lets a test assert on
/// **the artwork's own dimensions** rather than on its axis-aligned bound,
/// which is the distinction this whole module exists to make.
fn edges(corners: [(f64, f64); 4]) -> (f64, f64) {
    let mut lengths: Vec<f64> = (0..4)
        .map(|i| {
            let (ax, ay) = corners[i];
            let (bx, by) = corners[(i + 1) % 4];
            (bx - ax).hypot(by - ay)
        })
        .collect();
    lengths.sort_by(|a, b| a.partial_cmp(b).expect("every edge length is finite"));
    (lengths[3], lengths[0])
}

/// **A 30° turn arrives through the adapter with the ARTWORK's own
/// dimensions**, not with its bounding box's.
#[test]
fn a_thirty_degree_turn_arrives_with_the_artworks_own_dimensions() {
    let (doc, id) = turned(30.0);
    let quad = oriented_by_id(&doc.session.view(), &doc.pages[0], id)
        .expect("a `/Square` pdfcer authored has an appearance to place");
    let (long, short) = edges(quad.corners);

    assert!(
        (long - W).abs() < 0.5,
        "the long edge is {long:.2} and the artwork is {W} wide — corners {:?}",
        quad.corners
    );
    assert!(
        (short - H).abs() < 0.5,
        "the short edge is {short:.2} and the artwork is {H} tall — corners {:?}",
        quad.corners
    );

    // And the upright bound really did grow, so the two assertions above are
    // not passing because there was nothing to notice.
    let xs = quad.corners.map(|c| c.0);
    let bound = xs.iter().fold(f64::NEG_INFINITY, |a, b| a.max(*b))
        - xs.iter().fold(f64::INFINITY, |a, b| a.min(*b));
    assert!(
        bound > W + 5.0,
        "the upright bound is {bound:.2}, barely wider than the artwork — this fixture is not \
         exercising the case the module exists for"
    );

    assert!(
        (quad.degrees.expect("a rotation is an angle") - 30.0).abs() < 0.05,
        "the adapter read back {:?} for a 30 degree turn",
        quad.degrees
    );
    assert!(
        !quad.is_upright(),
        "a 30 degree turn reported itself upright"
    );
}

/// **An unturned annotation is upright and reports 0°, not `None`.**
#[test]
fn an_unturned_annotation_is_upright_and_reports_zero() {
    let (doc, id) = turned(0.0);
    let quad = oriented_by_id(&doc.session.view(), &doc.pages[0], id).expect("placed");
    assert_eq!(quad.degrees.map(|d| d.round()), Some(0.0));
    assert!(quad.is_upright());
}

/// **A QUARTER TURN IS NOT UPRIGHT**, which is the one part of
/// [`OrientedBox::is_upright`] a reader is likely to think is a bug.
#[test]
fn a_quarter_turn_is_not_upright_even_though_its_rect_fits() {
    let (doc, id) = turned(90.0);
    let quad = oriented_by_id(&doc.session.view(), &doc.pages[0], id).expect("placed");
    assert!(
        !quad.is_upright(),
        "a quarter turn reported itself upright at {:?} degrees — the outline would be right and \
         the grips would be on the wrong corners",
        quad.degrees
    );
}

/// **An annotation the engine will not place answers `None`** — and that is
/// a correct answer rather than a failure, because the caller then keeps
/// `/Rect`, which for such an annotation is where the mark is.
///
#[test]
fn an_annotation_that_is_not_there_has_no_placement() {
    let (doc, _) = turned(0.0);
    let absent = ObjId::new(9_999, 0);
    assert!(
        oriented_by_id(&doc.session.view(), &doc.pages[0], absent).is_none(),
        "an id that names no annotation on this page was given a placement"
    );
}

// ---------------------------------------------------------------------------
// The tripwire, inverted.
// ---------------------------------------------------------------------------

/// **THE TRIPWIRE FIRED, AND THIS IS WHAT IT BECAME.**
#[test]
fn the_engine_owns_the_placement_and_this_module_only_projects() {
    let lock = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"),
    )
    .expect("this crate is in a workspace with a Cargo.lock");

    // `source = "git+file:///D:/Dev/pdfcer?branch=main#<40-hex>"` — the rev is
    // everything after the last `#`.
    let rev = lock
        .lines()
        .skip_while(|l| l.trim() != "name = \"pdfcer-core\"")
        .find_map(|l| l.trim().strip_prefix("source = ")?.rsplit('#').next())
        .map(|s| s.trim_matches('"').to_owned())
        .expect("`Cargo.lock` pins `pdfcer-core` to a git revision");

    let home = std::env::var("CARGO_HOME").map_or_else(
        |_| {
            std::path::PathBuf::from(
                std::env::var("USERPROFILE")
                    .or_else(|_| std::env::var("HOME"))
                    .expect("a home directory"),
            )
            .join(".cargo")
        },
        std::path::PathBuf::from,
    );
    let checkouts = home.join("git/checkouts");
    let entries = std::fs::read_dir(&checkouts)
        .unwrap_or_else(|e| panic!("cannot read {} — {e}", checkouts.display()));

    // Cargo names the revision directory with a short prefix of the rev, and
    // the length of that prefix is cargo's business, not ours — so match by
    // prefix rather than assuming seven characters.
    let mut source = None;
    for entry in entries.flatten() {
        let Ok(revs) = std::fs::read_dir(entry.path()) else {
            continue;
        };
        for r in revs.flatten() {
            let name = r.file_name();
            let name = name.to_string_lossy();
            if !name.is_empty() && rev.starts_with(name.as_ref()) {
                let candidate = r.path().join("crates/pdfcer-core/src/annot.rs");
                if candidate.exists() {
                    source = Some(candidate);
                }
            }
        }
    }
    let source = source.unwrap_or_else(|| {
        panic!(
            "no checkout of pdfcer-core at pinned revision {rev} was found under {}. This crate \
             compiles against that revision, so the source MUST be there — the locating in this \
             test is what is broken, and a tripwire that cannot find its subject is a green check \
             over an empty scan.",
            checkouts.display()
        )
    });

    let annot = std::fs::read_to_string(&source)
        .unwrap_or_else(|e| panic!("cannot read {} — {e}", source.display()));
    let start = annot
        .find("pub struct Annotation {")
        .expect("`pdfcer-core` still declares `pub struct Annotation`");
    let body = &annot[start..];
    let body = &body[..body.find("\n}").expect("the struct is closed")];

    assert!(
        body.contains("pub appearance_matrix:"),
        "★★★ `pdfcer_core::annot::Annotation` NO LONGER HAS `appearance_matrix`, at engine \
         revision {rev}.\n\
         \n\
         `canvas::annotquad` is a thin adapter over that field and over \n\
         `pdfcer_render::annot::appearance_placement`, both of which arrived in `Pass 155.2` \n\
         after this shell filed for them. Before this build compiled, they existed.\n\
         \n\
         DO NOT restore this shell's own matrix reader. It carried one for a single day and \n\
         the reasons it was wrong are in the module header: a second reader of a structure \n\
         `pdfcer-core` owns, and a second implementation of a normative algorithm, both of \n\
         which drift. File a request naming what was withdrawn and why this shell needs it, \n\
         the way `request_an_annotations_rotation_angle_cannot_be_read.md` did."
    );
}

/// **A CLOCKWISE TURN IS NORMALISED, AND THIS IS THE TEST THAT WAS MISSING.**
#[test]
fn a_clockwise_turn_is_normalised_into_the_same_range_as_an_anticlockwise_one() {
    for (turn, expected) in [(-89.15_f64, 270.85_f64), (89.15, 89.15)] {
        let (doc, id) = turned(turn);
        let quad = oriented_by_id(&doc.session.view(), &doc.pages[0], id).expect("placed");
        let read = quad.degrees.expect("a rotation is an angle");
        assert!(
            (read - expected).abs() < 0.05,
            "a {turn} degree turn read back as {read:.2} and must be {expected:.2} — the engine \
             returns a signed atan2 and this module normalises with rem_euclid"
        );
        assert!(
            !quad.is_upright(),
            "a {turn} degree turn reported itself UPRIGHT at {read:.2} degrees. That is the \
             2026-09-07 defect exactly: the range test in `is_upright` is `(0.1..=359.9)`, so an \
             un-normalised negative angle falls outside it and the canvas draws an axis-aligned \
             box around a turned mark."
        );
    }
}
