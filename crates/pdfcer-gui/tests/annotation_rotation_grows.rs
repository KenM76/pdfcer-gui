//! # `annotation_rotation_grows` — the operator's *"the object gets larger with
//! each enactment of the tool"*, reduced to pixels
//!
//! ## ★★★ STATUS: THIS TEST ASSERTS A DEFECT, AND IT IS SUPPOSED TO PASS
//!
//! **If it goes RED, the engine has fixed `rotate_annotation` and this file's
//! job has changed.** Turn every assertion round to state the correct behaviour
//! — *the same total angle draws the same size* — and close
//! `request_rotate_annotation_grows_the_artwork_when_applied_twice.md` in
//! `D:\Dev\FeatureRequests\pdfce_FeatureRequests\`. That is exactly what
//! `engine_overlay_skew.rs` in this same directory did on 2026-08-31, and its
//! header records why the shape is right: a claim about somebody else's crate
//! had better carry a reproduction, and a reproduction that lives in a request
//! file is not run by anything.
//!
//! ## What the operator said, 2026-09-07
//!
//! > *"fixed the rotate bug in the review objects where the object gets larger
//! > with each enactment of the tool."*
//!
//! He is right, and the effect is **not** the one this shell already discloses.
//! `pdfcer-core`'s `rotate_annotation` doc comment says *"`/Rect` grows, and
//! that is correct … **The artwork does not grow**; only the rectangle that
//! bounds it does."* That sentence is true of the **first** rotation and false
//! of the second. Measured below: a mark turned 15° four times is drawn
//! **1.93× wider and 1.42× taller** than the same mark turned 60° once.
//!
//! ## The mechanism, from the engine's own source (`edit.rs` at pin `e1bdb6c`)
//!
//! Two individually-correct things that diverge after one application:
//!
//! 1. `edit.rs:24997` sets the new `/Rect` to the upright bound of **the four
//!    corners of the current `/Rect`**, rotated. §12.5.2 requires `/Rect`
//!    upright, so bounding is unavoidable — but bounding the *previous bound*
//!    compounds.
//! 2. `edit.rs:25041` composes the rotation into the appearance's own
//!    `/Matrix`, which is exactly right and is what makes rotation work on
//!    artwork pdfcer did not draw.
//!
//! After turn 1 the appearance's transformed `/BBox` bounds to precisely the
//! new `/Rect`, so §12.5.5 step (c)'s placement matrix **A** is a pure
//! translation and the artwork is drawn at its true size. After turn 2 the
//! `/Rect` has been bounded twice while the `/Matrix` has only accumulated to
//! 2θ — so the transformed `/BBox` is *smaller* than the rectangle, and step
//! (c) **scales it up to fit**, because §12.5.5 requires the appearance box to
//! fill `/Rect` exactly. The picture grows. Every further turn multiplies it.
//!
//! ## ★★ Why the oracle is an A/B and not an expected number
//!
//! Because *"a rotated rectangle's bounding box is larger, and that is
//! normative"* is a true sentence that can be used to justify almost any
//! measurement. It cannot justify **two different pictures from the same total
//! rotation**. One 60° turn and four 15° turns must produce the same file to
//! within float noise; that they do not is a claim no argument about bounding
//! boxes can absorb.
//!
//! ## ★ And why the measurement is a DIFF against a bare render
//!
//! The fixture page carries text. A naive ink-bounding-box over the whole page
//! measures the page, and the first draft of this test did exactly that: it
//! reported the same box for both arms and **passed**, while the underlying
//! defect was a 3.7 % change in the dark-pixel count that nothing was looking
//! at. Rendering the page once without the annotation and diffing pixel for
//! pixel isolates the mark and nothing else — which is the difference between a
//! test of the defect and a test of the fixture.

use pdfcer_core::annot_author::{Color, MarkupSpec};
use pdfcer_core::edit::EditSession;

/// The authored rectangle. Deliberately not square, and deliberately in clear
/// space on the fixture's first page.
const W: f64 = 140.0;
const H: f64 = 60.0;
const X0: f64 = 200.0;
const Y0: f64 = 400.0;

/// The pivot: the rectangle's own centre, which is what the shell's rotate grip
/// uses (`Grip::Rotate.pivot` answers the centre, and its doc comment says
/// why).
const PIVOT: (f64, f64) = (X0 + W / 2.0, Y0 + H / 2.0);

/// Device scale for the measurement. Two device pixels per point is enough that
/// a one-point error in a 140 pt edge is two pixels — well outside the
/// antialiasing noise the diff picks up — without making the render slow.
const SCALE: f32 = 2.0;

fn fixture() -> pdfcer_core::document::Document {
    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/four-pages.pdf");
    assert!(
        path.exists(),
        "the pinned fixture is missing at {} — this measurement needs a page with clear space at \
         (200, 400)",
        path.display()
    );
    pdfcer_core::document::Document::load(&path).expect("the fixture loads")
}

/// Page 1 rendered at [`SCALE`], as raw RGB triples plus the row stride.
///
/// ★ Through `render_page_with_view` on the **session's** view, not on the
/// document, so what is rasterised is the session's unsaved state — the same
/// route `render::worker` takes. Rendering the loaded document would show the
/// page before any of these edits.
fn pixels(session: &EditSession) -> (u32, Vec<[u8; 3]>) {
    let view = session.view();
    let pages = pdfcer_core::page_tree::pages_in(&session.graph()).expect("a page tree");
    let options = pdfcer_render::RenderOptions::default();
    let rendered = pdfcer_render::render_page_with_view(&view, &pages[0], SCALE, &options)
        .expect("the page rasterizes");
    let px = rendered.pixmap;
    let width = px.width();
    (
        width,
        px.pixels()
            .iter()
            .map(|p| [p.red(), p.green(), p.blue()])
            .collect(),
    )
}

/// The bounding box, in device pixels, of every pixel that differs between two
/// renders of the same page — i.e. exactly the annotation.
fn diff_box(base: &(u32, Vec<[u8; 3]>), other: &(u32, Vec<[u8; 3]>)) -> (u32, u32, u32, u32) {
    let width = base.0;
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0_u32, 0_u32);
    let mut any = false;
    for (i, (a, b)) in base.1.iter().zip(other.1.iter()).enumerate() {
        if a != b {
            let (x, y) = (i as u32 % width, i as u32 / width);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
            any = true;
        }
    }
    assert!(
        any,
        "the two renders are identical — the annotation was not drawn at all, so this measurement \
         is of nothing"
    );
    (x0, y0, x1, y1)
}

fn square() -> MarkupSpec {
    MarkupSpec::Square {
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
    }
}

/// Author the square, apply `turns` rotations of `each` degrees, and return the
/// drawn size in device pixels.
fn drawn_size(turns: usize, each: f64) -> (u32, u32) {
    let bare = pixels(&EditSession::new(fixture()));
    let mut session = EditSession::new(fixture());
    let id = session
        .add_markup(0, &square())
        .expect("the markup is authored");
    for _ in 0..turns {
        session
            .rotate_annotation(id, PIVOT, each)
            .expect("a `/Square` rotates");
    }
    let (x0, y0, x1, y1) = diff_box(&bare, &pixels(&session));
    (x1 - x0, y1 - y0)
}

/// ★★★ **THE REPRODUCTION.** Same total angle, two different pictures.
///
/// Expected numbers, measured 2026-09-07 against `pdfcer-core` v0.44.0 at
/// `e1bdb6c`:
///
/// | | drawn size (device px @ scale 2) |
/// |---|---|
/// | authored, unrotated | 279 × 120 — i.e. the 140 × 60 pt artwork, correct |
/// | one 60° turn | **243 × 302** — correct: 140·cos60 + 60·sin60 = 121.96 pt |
/// | four 15° turns | **469 × 430** — 1.93× and 1.42× too big |
///
/// The single-turn arm is asserted **exactly**, and it is what makes the
/// defect arm meaningful: it proves the instrument reads the right number when
/// the engine gets it right, so the four-turn arm is measuring the engine and
/// not the harness.
#[test]
fn four_fifteen_degree_turns_draw_a_bigger_shape_than_one_sixty_degree_turn() {
    let unturned = drawn_size(0, 0.0);
    let once = drawn_size(1, 60.0);
    let four = drawn_size(4, 15.0);

    // The control. 140 × 60 pt at 2 px/pt, to within one pixel of antialiasing
    // at each edge.
    assert!(
        unturned.0.abs_diff(280) <= 2 && unturned.1.abs_diff(120) <= 2,
        "the unrotated mark measured {unturned:?} and should be about 280 x 120 — the harness is \
         not measuring what it thinks it is, and nothing below this line means anything"
    );

    // The single turn is exact, so the instrument is proven on a case the
    // engine gets right.
    assert!(
        once.0.abs_diff(244) <= 3 && once.1.abs_diff(302) <= 3,
        "one 60 degree turn measured {once:?} and trigonometry says about 244 x 302"
    );

    // ★★★ THE DEFECT. Delete this assertion and restore the equality below it
    // the day the engine ships the fix.
    assert!(
        four.0 > once.0 + 50 && four.1 > once.1 + 50,
        "four 15 degree turns measured {four:?} and one 60 degree turn measured {once:?}.\n\
         \n\
         IF THESE ARE NOW EQUAL, THE ENGINE HAS FIXED IT. That is good news and this test's \n\
         job has changed: replace this assertion with\n\
         \n\
             assert_eq!(four, once, \"the same total rotation must draw the same picture\");\n\
         \n\
         delete the `/Rect`-grows disclosure in `text::rotating::rect_grew` and the paragraph \n\
         at `app::actions::annots::rotate` that explains it, close \n\
         `request_rotate_annotation_grows_the_artwork_when_applied_twice.md`, and mark \n\
         OPERATOR_REQUESTS.md O145 done."
    );
}
