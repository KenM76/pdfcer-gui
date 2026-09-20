//! # `annotation_rotation_grows` — rotation composes, and the one annotation
//! where it cannot
//!
//! `pdfcer-core`'s `rotate_annotation` derives the new `/Rect` from the mark's
//! own artwork: the transformed appearance `/BBox` where there is an `/AP`, the
//! bounded rotated geometry keys where there is not. Both routes compose — N
//! turns totalling θ draw the same picture as one turn of θ — and the outcome
//! names the route it took through `RectDerivation`.
//!
//! An annotation with **neither** an appearance stream nor rotatable geometry
//! has only `RectDerivation::PreviousRect` available, which bounds the previous
//! bound and therefore compounds. §12.5.2 requires `/Rect` upright and such an
//! annotation has nowhere else to record an orientation, so no rule can do
//! better; what the shell owes is disclosure, not a fix.
//!
//! This file pins both ends of that fork. The first test measures that the
//! composing route composes; the second that the non-composing route is still
//! reachable, still grows, and still says which rule it took — because a
//! disclosure whose trigger never fires is indistinguishable from one that is
//! broken.
//!
//! ## Why the oracle is an A/B and not an expected number
//!
//! *"A rotated rectangle's bounding box is larger, and that is normative"* is a
//! true sentence that can be used to justify almost any measurement. It cannot
//! justify **two different pictures from the same total rotation**. One 60°
//! turn and four 15° turns must produce the same file to within float noise,
//! and no argument about bounding boxes can absorb their differing.
//!
//! ## Why the measurement is a DIFF against a bare render
//!
//! Reading `/Rect` off the file would measure the rectangle, and the rectangle
//! is not the complaint: §12.5.5 step (c) scales the appearance box to fill
//! `/Rect` exactly, so it is the *placement matrix* that decides whether the
//! artwork is drawn at its true size. Only a raster shows that. Diffing against
//! a render of the same page with no annotation isolates the mark without the
//! test needing to know anything about the page's own content, and it fails
//! loudly — see `diff_box` — when the mark was not drawn at all.

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
/// Through `render_page_with_view` on the **session's** view, not on the
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

/// The regression net: same total angle, same picture, at 1, 4 and 24 turns.
///
/// | | drawn size (device px @ scale 2) |
/// |---|---|
/// | authored, unrotated | 279 × 120 — the 140 × 60 pt artwork, correct |
/// | one 60° turn | **243 × 302** — 140·cos60 + 60·sin60 = 121.96 pt |
/// | four 15° turns | **243 × 302** — identical, which is the property |
/// | twenty-four 2.5° turns | **243 × 302** — identical |
///
/// # Why 24 turns and not just 4
///
/// A non-composing derivation compounds **multiplicatively**, so a per-turn
/// residual too small for a four-turn tolerance to see is unmissable by
/// twenty-four: 1 % per turn is 4 % at four turns — inside a two-pixel window
/// on a 243-pixel shape — and 80 % at twenty-four.
///
/// # The unrotated control is not decoration
///
/// It is asserted **exactly**, and it is what makes the two equalities below it
/// mean anything: a build that rotated **nothing at all** would satisfy every
/// *"these two are equal"* assertion in this file perfectly.
///
/// So this asserts three things and needs all three: the instrument reads the
/// right number on an untouched mark, one turn moves it to the trigonometric
/// answer, and repeating the turn does not change the answer.
///
/// # The shape is load-bearing
///
/// A shape carrying `/Vertices` falls through to the geometry rule, which also
/// composes — so an A/B built on a `/Polygon` stays green with the artwork rule
/// sabotaged, measuring *"some rule composes"* while its name claims it
/// measures the artwork one. A `/Square` has no rotatable geometry keys, so it
/// can only be exercising `RectDerivation::Artwork`. **If the shape is ever
/// changed, pin the route** — the second test in this file is what pins the
/// other end of that fork.
#[test]
fn the_same_total_rotation_draws_the_same_picture_however_many_turns_it_takes() {
    let unturned = drawn_size(0, 0.0);
    let once = drawn_size(1, 60.0);
    let four = drawn_size(4, 15.0);
    let twenty_four = drawn_size(24, 2.5);

    // The control. 140 × 60 pt at 2 px/pt, to within one pixel of antialiasing
    // at each edge.
    assert!(
        unturned.0.abs_diff(280) <= 2 && unturned.1.abs_diff(120) <= 2,
        "the unrotated mark measured {unturned:?} and should be about 280 x 120 — the harness is \
         not measuring what it thinks it is, and nothing below this line means anything"
    );

    // And one turn lands where trigonometry says, so "equal" below is equal to
    // the RIGHT number rather than equal to each other and wrong.
    assert!(
        once.0.abs_diff(244) <= 3 && once.1.abs_diff(302) <= 3,
        "one 60 degree turn measured {once:?} and trigonometry says about 244 x 302"
    );

    // THE PROPERTY. `rotate_annotation` derives `/Rect` from the artwork rather
    // than from the previous `/Rect`, so this composes.
    for (label, measured) in [
        ("four 15 degree turns", four),
        ("24 2.5 degree turns", twenty_four),
    ] {
        assert_eq!(
            measured, once,
            "{label} drew {measured:?} and one 60 degree turn drew {once:?}.\n\
             \n\
             A rotation must compose: N turns totalling theta draw the same picture as one turn \n\
             of theta. If these differ, `/Rect` has gone back to being derived from the PREVIOUS \n\
             rectangle instead of from the artwork, and the placement rule then scales the \n\
             artwork up to fill an oversized box — the operator's own report, \n\
             \"the object gets larger with each enactment of the tool\".\n\
             \n\
             Read `rect_derived=` off a `set-annotation-rotation-applied` trace line, or run \n\
             `pdfcer rotate-annotation` and read its second line. `artwork` and `geometry` both \n\
             compose; `previous-rect` does not and cannot, and this fixture must never take \n\
             that route — a `/Square` authored by `add_markup` has an appearance stream."
        );
    }
}

/// The one case that does not compose, asserted so it is a known limit rather
/// than a surprise.
///
/// `RectDerivation::PreviousRect`: an annotation with **neither** an appearance
/// stream **nor** rotatable geometry. Its artwork *is* its rectangle, §12.5.2
/// requires that rectangle upright, so there is **nowhere in the annotation an
/// orientation could be recorded** and no rule can do better. The engine says
/// so explicitly and warns that a shell ignoring it *"re-introduces the
/// operator's bug one level up, on exactly the annotations that cannot be
/// fixed."*
///
/// ⇒ This shell honours it by **disclosing**:
/// `text::rotating::rect_still_grows` fires on this rule and only on this rule.
/// The test that the sentence exists is in that module; this is the test that
/// the *condition* is real, because a disclosure whose trigger never occurs is
/// indistinguishable from one that is broken.
///
/// # The fixture
///
/// `fixtures/square-no-appearance.pdf` — a 479-byte hand-written PDF with one
/// `/Square` annotation carrying a `/Rect`, a `/C` and nothing else. It has to
/// be hand-written rather than authored: `add_markup` always writes an
/// appearance stream (correctly — R43 means a mark with no `/AP` is named and
/// not painted), so there is no route to this case through the engine's own
/// authoring API. It is exactly the document a producer that leaves appearance
/// generation to the reader would write.
#[test]
fn an_annotation_with_no_artwork_and_no_geometry_still_grows_and_says_which_rule() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/square-no-appearance.pdf");
    let doc = pdfcer_core::document::Document::load(&path).expect("the fixture loads");
    let mut session = EditSession::new(doc);
    let pages = pdfcer_core::page_tree::pages_in(&session.graph()).expect("a page tree");
    let id = pdfcer_core::annot::page_annotations(&session.graph(), pages[0].id)
        .into_iter()
        .find(|a| a.subtype == b"Square")
        .and_then(|a| a.id)
        .expect("the fixture carries one `/Square`");

    let first = session
        .rotate_annotation(id, PIVOT, 15.0)
        .expect("a `/Square` with no appearance still rotates");
    assert_eq!(
        first.rect_derived_from,
        pdfcer_core::edit::RectDerivation::PreviousRect,
        "a `/Square` with no `/AP` and no geometry keys must fall through to the rule that \
         cannot compose. If it did not, this test no longer exercises the case \
         `text::rotating::rect_still_grows` discloses, and that sentence is now unreachable"
    );

    let second = session
        .rotate_annotation(id, PIVOT, 15.0)
        .expect("and again");
    let (first_w, second_w) = (first.to.urx - first.to.llx, second.to.urx - second.to.llx);
    assert!(
        second_w > first_w + 1.0,
        "the rectangle did not grow on the second turn ({first_w:.1} then {second_w:.1}). \
         Either the engine found a better rule for this case — in which case DELETE the \
         disclosure in `text::rotating::rect_still_grows` and this test with it — or the \
         fixture stopped reaching `PreviousRect`."
    );
}
