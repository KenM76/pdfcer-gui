//! # `canvas::annotquad` tests — the placement is asked of a REAL annotation,
//! and the pathological matrices are asked of a synthetic one
//!
//! ## ★★★ What these can and cannot prove
//!
//! **They cannot prove the operator sees an angled outline.** Every test here
//! calls [`super::oriented`] directly; whether the painter maps those corners
//! through the right projection, whether the selection layer carries them, and
//! whether a turned mark on screen ends up inside them are questions only
//! `tools/ui-verify` can answer. R1 is not relaxed here.
//!
//! What they do prove is the part that is pure arithmetic and therefore the part
//! a unit test genuinely is the right instrument for: **that this module
//! computes §12.5.5 rather than something that resembles it.**
//!
//! ## ★★ The split, and why it is not laziness in either direction
//!
//! | case | fixture | why |
//! |---|---|---|
//! | unrotated, 30°, 90°, 210° | a **real** `/Square` authored by `add_markup` and turned by `rotate_annotation` | these assert against the engine's *actual* matrix convention. A hand-built dictionary would let this file agree with itself while disagreeing with every file on disk — `annotnodes::tests` records that trap in the same words |
//! | shear, no `/AP`, unresolvable `/AS` | a **synthetic** three-object graph | **no pdfcer verb authors any of them.** They arrive from other people's files, and there is no route to one through the engine's writing API — so the choice is a synthetic graph or no coverage of the cases this module is most likely to be wrong about |
//!
//! The synthetic half is a `BTreeMap` behind [`ObjectGraph`], which is the same
//! trait the real session presents. The function under test cannot tell them
//! apart, which is exactly what makes the arrangement honest.
//!
//! ## ★ The 30° case is the one that matters
//!
//! At 0° an oriented box and an upright box are the same rectangle, so a module
//! that returned `/Rect` unchanged would pass that one. 30° is chosen because
//! every corner moves, the `/Rect` is measurably larger than the artwork, and
//! the expected result — *the edges are still 140 × 60* — follows from the fact
//! that a rotation is an isometry, which can be written down without reference
//! to the code under test.

#![cfg(test)]

use super::*;
use pdfcer_core::annot_author::{Color, MarkupSpec};
use pdfcer_core::object::Name;
use std::collections::BTreeMap;

/// The authored rectangle: deliberately **not square**, so a bug that swapped
/// width for height, or that returned an axis-aligned bounding box, is visible
/// rather than accidentally correct.
const W: f64 = 140.0;
const H: f64 = 60.0;
const X0: f64 = 200.0;
const Y0: f64 = 400.0;

/// The centre of that rectangle — the pivot every rotation here turns about.
const PIVOT: (f64, f64) = (X0 + W / 2.0, Y0 + H / 2.0);

/// A real document with one `/Square` markup authored on page 1, and its id.
///
/// ★★ Through `add_markup`, not through a hand-built dictionary: the appearance
/// stream, its `/BBox` and its `/Matrix` are then the ones the engine actually
/// writes, so a change in the engine's convention turns these red — which is
/// the notification this shell wants rather than a surprise on a real file.
fn authored() -> (crate::app::state::OpenDoc, ObjId) {
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
    (doc, id)
}

/// Turn the authored square by `degrees` and return the session's answer.
fn turned(degrees: f64) -> (crate::app::state::OpenDoc, ObjId) {
    let (mut doc, id) = authored();
    let session = std::sync::Arc::get_mut(&mut doc.session).expect("sole owner");
    session
        .rotate_annotation(id, PIVOT, degrees)
        .expect("a `/Square` rotates");
    (doc, id)
}

/// Longest and shortest edge of a quadrilateral.
///
/// This is what lets a test assert on **the artwork's own dimensions** rather
/// than on its axis-aligned bound — the distinction the whole module exists to
/// make, so measuring it directly is the point rather than a convenience.
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

/// The upright width of a quadrilateral — used only to prove that a fixture is
/// exercising the case it claims to.
fn upright_width(corners: [(f64, f64); 4]) -> f64 {
    let xs = corners.map(|c| c.0);
    xs.iter().fold(f64::NEG_INFINITY, |a, b| a.max(*b))
        - xs.iter().fold(f64::INFINITY, |a, b| a.min(*b))
}

// ---------------------------------------------------------------------------
// The synthetic graph, for matrices no pdfcer verb can author.
// ---------------------------------------------------------------------------

/// A minimal [`ObjectGraph`] over a map — three objects is all these cases need.
///
/// `trailer_entry` answers `None` for everything, which is correct: nothing in
/// [`super::oriented`] reads the trailer, and a stub that invented a `/Root`
/// would be inviting a future reader to believe this fixture is a document.
struct Fake(BTreeMap<ObjId, Object>);

impl ObjectGraph for Fake {
    fn value(&self, id: ObjId) -> Option<&Object> {
        self.0.get(&id)
    }
    fn trailer_entry(&self, _key: &[u8]) -> Option<&Object> {
        None
    }
}

fn id(n: u32) -> ObjId {
    ObjId::new(n, 0)
}

fn reals(values: &[f64]) -> Object {
    Object::Array(values.iter().map(|v| Object::Real(*v)).collect())
}

fn dict(entries: &[(&[u8], Object)]) -> Dict {
    let mut d = Dict::new();
    for (key, value) in entries {
        d.insert(Name::from(*key), value.clone());
    }
    d
}

/// An annotation (object 1) whose `/AP` `/N` is a stream (object 2) carrying
/// `matrix`, with an optional `/AP` at all.
///
/// The `/BBox` is `[0 0 W H]` and the `/Rect` is the same box at the origin, so
/// step (c) of the placement is the identity and any difference in the answer is
/// attributable to the matrix alone. That is what makes the shear test's verdict
/// unambiguous.
fn synthetic(matrix: Option<&[f64]>, with_ap: bool) -> Fake {
    let mut objects = BTreeMap::new();
    let mut appearance = dict(&[(b"BBox", reals(&[0.0, 0.0, W, H]))]);
    if let Some(m) = matrix {
        appearance.insert(Name::from(b"Matrix".as_slice()), reals(m));
    }
    objects.insert(
        id(2),
        Object::Stream(pdfcer_core::object::Stream {
            dict: appearance,
            data_span: pdfcer_core::span::ByteSpan::new(0, 0),
        }),
    );
    let mut annot = dict(&[
        (b"Subtype", Object::Name(Name::from(b"Square".as_slice()))),
        (b"Rect", reals(&[0.0, 0.0, W, H])),
    ]);
    if with_ap {
        annot.insert(
            Name::from(b"AP".as_slice()),
            Object::Dict(dict(&[(b"N", Object::Reference(id(2)))])),
        );
    }
    objects.insert(id(1), Object::Dict(annot));
    Fake(objects)
}

// ---------------------------------------------------------------------------
// The real-annotation cases.
// ---------------------------------------------------------------------------

/// ★★★ **An UNROTATED annotation's quad is its `/Rect`, corner for corner.**
///
/// The base case, and it is what makes every other assertion here meaningful: if
/// this were wrong, a plausible-looking 30° result would be plausible for the
/// wrong reason.
///
/// It also pins the corner **order**, which the painter depends on to draw a
/// closed outline rather than a bow tie — corner 0 is the artwork's lower-left
/// and the four run anticlockwise.
#[test]
fn an_unrotated_square_is_bounded_by_its_own_rect() {
    let (doc, annot) = authored();
    let quad =
        oriented(&doc.session.graph(), annot).expect("a `/Square` pdfcer authored has an `/AP`");

    let want = [(X0, Y0), (X0 + W, Y0), (X0 + W, Y0 + H), (X0, Y0 + H)];
    for (got, want) in quad.corners.iter().zip(want.iter()) {
        assert!(
            (got.0 - want.0).abs() < 0.5 && (got.1 - want.1).abs() < 0.5,
            "corner {got:?} should have been {want:?} — the whole quad was {:?}",
            quad.corners
        );
    }
    assert_eq!(
        quad.degrees.map(|d| d.round()),
        Some(0.0),
        "an unrotated appearance is 0 degrees — not `None`, and not 360"
    );
    assert!(quad.is_upright(), "and it reports itself upright");
}

/// ★★★ **A 30° turn produces a quad whose EDGES are still 140 × 60**, while its
/// upright bound has grown past 151.
///
/// This is the operator's sentence reduced to arithmetic. A module returning
/// `/Rect` would report a ~151 × 122 axis-aligned box; the artwork is 140 × 60
/// at an angle, and that is what the outline must hug.
///
/// The tolerance is half a point — a quarter of the border width the fixture is
/// drawn with. A wrong pivot, a transposed matrix, or a missing step (c) all
/// fail it.
#[test]
fn a_thirty_degree_turn_keeps_the_artworks_own_dimensions() {
    let (doc, annot) = turned(30.0);
    let quad = oriented(&doc.session.graph(), annot).expect("the appearance survived the rotation");
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
    // ★ And the upright bound really did grow, so the assertions above are not
    // passing because there was nothing to notice.
    let bound = upright_width(quad.corners);
    assert!(
        bound > W + 5.0,
        "the upright bound is {bound:.2}, barely wider than the artwork — this fixture is not \
         exercising the case the module exists for"
    );
    assert!(
        !quad.is_upright(),
        "a 30 degree turn reported itself upright"
    );
}

/// ★★ **The angle comes back anticlockwise and normalised.**
///
/// Three turns, each catching a different way of getting a rotation
/// decomposition wrong:
///
/// | turn | what it catches |
/// |---|---|
/// | 30° | a sign flip — that would read 330 |
/// | 90° | an `atan2` argument swap — that would read 0 |
/// | 210° | folding onto `[-90, 90]` through `atan` instead of `atan2` — that would read 30 |
#[test]
fn the_angle_comes_back_anticlockwise_and_normalised() {
    for turn in [30.0_f64, 90.0, 210.0] {
        let (doc, annot) = turned(turn);
        let quad = oriented(&doc.session.graph(), annot).expect("the appearance survived");
        let got = quad.degrees.expect("a rotation is an angle");
        assert!(
            (got - turn).abs() < 0.05,
            "asked for {turn} degrees and read back {got:.3}"
        );
    }
}

// ---------------------------------------------------------------------------
// The synthetic cases.
// ---------------------------------------------------------------------------

/// ★★★ **A matrix that is not a rotation reports NO ANGLE — and still reports
/// corners.**
///
/// Both halves are the point. A sheared appearance has a perfectly good
/// quadrilateral and an outline can and should be drawn round it; but it has no
/// angle, and a properties field showing one would be showing a number nothing
/// in the file means.
///
/// `[1 0 0.6 1 0 0]` is a horizontal shear: `a == d` holds, `b == −c` does not.
/// That is exactly the pair the test in `Mat::rotation_degrees` exists to
/// separate, and a naive `atan2(b, a)` would confidently answer **0°**.
#[test]
fn a_sheared_appearance_has_corners_but_no_angle() {
    let graph = synthetic(Some(&[1.0, 0.0, 0.6, 1.0, 0.0, 0.0]), true);
    let quad = oriented(&graph, id(1)).expect("a sheared appearance still has a placement");
    assert!(
        quad.degrees.is_none(),
        "a shear was reported as {:?} degrees",
        quad.degrees
    );
    assert!(
        quad.corners
            .iter()
            .all(|(x, y)| x.is_finite() && y.is_finite()),
        "and the corners must still be usable: {:?}",
        quad.corners
    );
}

/// ★★ **A MIRROR is not an angle either**, and it is the case a looser test
/// would let through.
///
/// `[-1 0 0 1 0 0]` reflects in x. `b == −c` holds (both are zero); `a == d`
/// does not. A decomposition that checked only the `b`/`c` pair would report
/// **180°**, which is wrong in a way no operator could diagnose: a mirrored
/// stamp turned "back" by 180° would come out mirrored *and* upside down.
#[test]
fn a_mirrored_appearance_is_not_reported_as_a_half_turn() {
    let graph = synthetic(Some(&[-1.0, 0.0, 0.0, 1.0, 0.0, 0.0]), true);
    let quad = oriented(&graph, id(1)).expect("a mirrored appearance still has a placement");
    assert!(
        quad.degrees.is_none(),
        "a mirror was reported as {:?} degrees",
        quad.degrees
    );
}

/// ★ **A missing `/Matrix` is the identity**, per §8.10.2's default — not a
/// refusal. The overwhelming majority of appearance streams in the wild carry no
/// `/Matrix` at all, so an implementation that treated its absence as
/// unreadable would answer `None` for almost every annotation ever written.
#[test]
fn an_appearance_with_no_matrix_is_upright() {
    let graph = synthetic(None, true);
    let quad = oriented(&graph, id(1)).expect("no `/Matrix` is the identity, not a refusal");
    assert_eq!(quad.degrees.map(|d| d.round()), Some(0.0));
    assert!(quad.is_upright());
}

/// ★★ **A collapsed matrix has no orientation**, and answering `0°` for it would
/// be a confidently wrong number rather than an absent one.
///
/// `[0 0 0 0 0 0]` maps the whole `/BBox` to a point. `atan2(0.0, 0.0)` is
/// `0.0` in Rust — a perfectly finite answer that means nothing — which is why
/// the scale floor is tested before the angle is taken. The placement itself is
/// also degenerate, so the whole answer is `None`.
#[test]
fn a_collapsed_matrix_has_no_placement_at_all() {
    let graph = synthetic(Some(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.0]), true);
    assert!(
        oriented(&graph, id(1)).is_none(),
        "a matrix that collapses the appearance to a point was given a placement"
    );
}

/// ★★ **An annotation with no appearance stream answers `None`** — and that is
/// the correct answer rather than a failure. The caller keeps `/Rect`, which for
/// such an annotation *is* where the mark is.
///
/// The only difference from [`an_appearance_with_no_matrix_is_upright`] is the
/// presence of `/AP`, which is what makes this a test of the thing it names.
#[test]
fn no_appearance_means_no_oriented_box_rather_than_a_guess() {
    let graph = synthetic(None, false);
    assert!(
        oriented(&graph, id(1)).is_none(),
        "an annotation with no `/AP` was given an orientation it does not have"
    );
}

/// ★★★ **An unresolvable appearance STATE answers `None`, matching the engine's
/// refusal to guess.**
///
/// `/N` is a subdictionary with two entries and there is no `/AS`. §12.5.5
/// NOTE 3 permits *"reasonable behaviour such as displaying nothing"*, and
/// `pdfcer-core` models this as `Appearance::StateUnresolved` and **declines to
/// pick a first/`On`/`Off` key**, because real readers disagree about which and
/// guessing shows a state no other viewer shows.
///
/// ⇒ This shell must decline for the same reason plus one of its own: an outline
/// drawn from an appearance the renderer refused to paint is **a box around
/// nothing**, which is the "selection outline on blank paper" failure this
/// project has already fixed once, in `selection::annot`'s `/NoView` guard.
#[test]
fn an_unresolvable_appearance_state_is_declined_rather_than_guessed() {
    let mut graph = synthetic(None, true);
    let states = Object::Dict(dict(&[
        (b"Off", Object::Reference(id(2))),
        (b"Yes", Object::Reference(id(2))),
    ]));
    let Some(Object::Dict(annot)) = graph.0.get_mut(&id(1)) else {
        panic!("the synthetic annotation is a dictionary");
    };
    annot.insert(
        Name::from(b"AP".as_slice()),
        Object::Dict(dict(&[(b"N", states)])),
    );

    assert!(
        oriented(&graph, id(1)).is_none(),
        "an appearance state the renderer would not resolve was resolved here anyway"
    );
}

/// ★ **A single-entry state subdictionary with no `/AS` IS taken**, because
/// there is nothing to choose between — and this is the boundary of the rule
/// above, scoped to match the engine's own wording (*"`/AS` is missing against a
/// **multi-entry** subdictionary"*).
///
/// Without this test the previous one would be satisfied by a module that
/// refused every subdictionary, which would silently drop the outline on every
/// stamp whose producer wrapped its single appearance in a state dictionary.
#[test]
fn a_single_state_with_no_as_is_not_ambiguous() {
    let mut graph = synthetic(None, true);
    let Some(Object::Dict(annot)) = graph.0.get_mut(&id(1)) else {
        panic!("the synthetic annotation is a dictionary");
    };
    annot.insert(
        Name::from(b"AP".as_slice()),
        Object::Dict(dict(&[(
            b"N",
            Object::Dict(dict(&[(b"Only", Object::Reference(id(2)))])),
        )])),
    );

    assert!(
        oriented(&graph, id(1)).is_some(),
        "a single-entry appearance state with no `/AS` was refused as ambiguous"
    );
}

/// ★★★ **A UNIFORM RESIZE COMMUTES WITH A ROTATION**, measured rather than
/// assumed — and it is what makes the turned grip frame safe to ship.
///
/// The design question this answers: if the eight scale grips move onto the
/// turned outline, does a drag on one still mean anything? A resize is applied
/// to `/Rect`, which is axis-aligned; a rotation lives in the appearance
/// `/Matrix`. There is no obvious reason those should compose cleanly, and if
/// they did not, moving the grips would be dressing up a gesture that shears
/// the mark.
///
/// They do. 140 × 60 at 30°, scaled 2× uniformly, comes back **280 × 120 at
/// 30°** with all four corners still square — the angle preserved to fourteen
/// decimal places and the shape still a rectangle.
///
/// ★★ **And the non-uniform case never reaches the question**, which is the
/// other half of the answer. `resize_annotation` refuses a `/Square` outright
/// with `ResizeAppearanceNotRebuildable { uniform: false }` — *"the scale is
/// non-uniform, so the drawn stroke becomes anisotropic and no scalar `/BS /W`
/// can describe it"* — so the mid-edge grips, the ones whose behaviour in a
/// turned frame would be genuinely ambiguous, decline before geometry is
/// reached. The engine's refusal is doing this shell's scoping for it.
///
/// ⚠ **What is still owed**, recorded here rather than left to be rediscovered:
/// `canvas::resizing::factors` derives its scale factors from a screen delta
/// against an **axis-aligned** box. In a turned frame those should be projected
/// onto the frame's own axes. Today a corner drag on a turned mark therefore
/// commits a factor computed on the page's axes — which is what it did before
/// this change too, so nothing regressed, but it is not yet right.
#[test]
fn a_uniform_resize_preserves_a_rotation() {
    let (mut doc, annot) = turned(30.0);
    let before = oriented(&doc.session.graph(), annot).expect("placed");
    let session = std::sync::Arc::get_mut(&mut doc.session).expect("sole owner");
    session
        .resize_annotation(
            annot,
            PIVOT,
            2.0,
            2.0,
            // The stroke must scale too, or the engine declines the uniform
            // case as well — see `canvas::scaling::Modifiers`, which is the
            // operator-facing switch for exactly this.
            &pdfcer_core::edit::ResizeOptions::new().with_scale_stroke_width(true),
        )
        .expect("a uniform resize of a turned `/Square` is accepted");

    let after = oriented(&doc.session.graph(), annot).expect("the appearance survived");
    let (long_before, short_before) = edges(before.corners);
    let (long_after, short_after) = edges(after.corners);

    assert!(
        (long_after - long_before * 2.0).abs() < 0.5
            && (short_after - short_before * 2.0).abs() < 0.5,
        "a 2x uniform resize took {long_before:.1}x{short_before:.1} to          {long_after:.1}x{short_after:.1}"
    );
    assert!(
        (after.degrees.expect("still an angle") - 30.0).abs() < 0.05,
        "the resize moved the angle to {:?}",
        after.degrees
    );
    // ★ Still a rectangle. A shear would leave the edge lengths plausible and
    // the corners oblique, so the lengths alone are not enough to notice it.
    for i in 0..4 {
        let (px, py) = after.corners[(i + 3) % 4];
        let (cx, cy) = after.corners[i];
        let (nx, ny) = after.corners[(i + 1) % 4];
        let (ux, uy) = (px - cx, py - cy);
        let (vx, vy) = (nx - cx, ny - cy);
        let cosine = (ux * vx + uy * vy) / (ux.hypot(uy) * vx.hypot(vy));
        assert!(
            cosine.abs() < 1e-6,
            "corner {i} is oblique (cos = {cosine:.6}) — the resize sheared the mark"
        );
    }
}

// ---------------------------------------------------------------------------
// The tripwire.
// ---------------------------------------------------------------------------

/// ★★★ **THE TRIPWIRE. When this goes red, DELETE THIS MODULE.**
///
/// It reads the **pinned** engine's `annot.rs` and fails the moment
/// `pub struct Annotation` grows any of [`super::AWAITED_ENGINE_FIELDS`].
///
/// # Why it reads the cargo checkout and not `D:/Dev/pdfcer`
///
/// Because `D:/Dev/pdfcer` is the engine session's **working tree** and moves
/// several times a day, often ahead of what this build compiles. A tripwire
/// keyed on it would fire on work that is not in our binary, which is the same
/// class of wrongness as not firing at all. The cargo checkout under
/// `~/.cargo/git/checkouts/` is the exact bytes `rustc` read, and the directory
/// is named for the revision — so locating it *through `Cargo.lock`* makes the
/// instrument follow the pin automatically.
///
/// ⚠ **It FAILS rather than skips when it cannot find the source.** A hard-coded
/// external path turning a rename into a green check over an empty scan is a
/// mistake this project has already made once, and the recovery cost more than
/// the false red would have. If the checkout is missing, the crate cannot have
/// compiled, so a red here means the *locating* is broken and needs fixing —
/// never ignoring.
///
/// # What a match means
///
/// The whole of `canvas::annotquad` becomes dead weight: take the angle from
/// the read model, take the placement from whatever shipped with it, delete
/// this file and its consumers' fallbacks, and close
/// `request_an_annotations_rotation_angle_cannot_be_read.md`.
#[test]
fn the_engine_still_has_no_rotation_field() {
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

    for field in super::AWAITED_ENGINE_FIELDS {
        let needle = format!("pub {field}:");
        assert!(
            !body.contains(&needle),
            "★★★ `pdfcer_core::annot::Annotation` now has a `{field}` field, at engine revision \
             {rev}.\n\
             \n\
             THAT IS WHAT `crates/pdfcer-gui/src/canvas/annotquad.rs` HAS BEEN WAITING FOR. It is \n\
             a declared workaround — a second reader of the appearance `/Matrix` and a second \n\
             implementation of ISO 32000-1 §12.5.5 — and it should now be DELETED, not adapted.\n\
             \n\
               1. Take the angle from the read model in `panels::properties::geometry`.\n\
               2. Take the placement from whatever shipped alongside it, in \n\
                  `canvas::selection::annot`.\n\
               3. Delete `canvas::annotquad` and this test.\n\
               4. Close `request_an_annotations_rotation_angle_cannot_be_read.md` in \n\
                  `D:/Dev/FeatureRequests/pdfce_FeatureRequests/`."
        );
    }
}
