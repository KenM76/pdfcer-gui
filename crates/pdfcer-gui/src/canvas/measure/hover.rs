//! # `canvas::measure::hover` — showing what a measuring click will pick,
//! before it picks it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/measure/hover.md`.

use egui::{Pos2, Shape, Stroke};
use pdfcer_core::vector::PageObjects;
use pdfcer_core::vector::Point;
use pdfcer_core::vector::hit::{HitTarget, hit_test_point_deep};
use pdfcer_core::vector::linepick::pick_line_of;

/// What the pointer is over, resolved while the decomposition is borrowed.
///
/// `Copy`, so it can ride inside [`super::Resolved`] without changing that
/// type's shape — which matters because `Resolved`'s whole contract is that it
/// is one cheap value passed from the resolve pass to the paint pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::canvas) struct Entity {
    /// **Which list, and which entry in it**, the highlight is about — carried
    /// for the trace so a reader can tie a highlight to the object the pick
    /// will name.
    ///
    pub target: HitTarget,
    /// The straight run the pick would use, page space, when there is one.
    pub segment: Option<(Point, Point)>,
    /// The object's page-space bounds, as `(min, max)`.
    ///
    /// Always present. Drawn only when [`Self::segment`] is `None` — see the
    /// module header's order of preference — but carried in both cases because
    /// it costs nothing and a future *"which object?"* disclosure will want it.
    pub bounds: (Point, Point),
}

/// Find the entity under `query`.
///
/// `tolerance` is the same page-space catch radius the snap query uses, so the
/// highlight and the snap agree about what "near" means. Handing them different
/// radii would produce the state this whole module exists to prevent: a marker
/// on one line and a highlight on another.
///
/// # Why `hit_test_point` rather than the snap candidate's `source_object`
///
/// Because a snap candidate is often **not** owned by one object, and the two
/// questions genuinely differ. `SnapCandidate::source_object` is documented as
/// `None` for *"a page-axis or grid candidate, or a segment–segment
/// intersection between two different objects"* — and an intersection is
/// exactly the case where an operator most needs to be told which line they are
/// about to take, since by construction there are two.
///
/// So the entity is resolved from the **pointer**, independently. When the two
/// agree, the operator sees a node on a highlighted line and everything is
/// obvious. When they disagree — an intersection — they see the node at the
/// crossing and the highlight on the line the click will pick, which is the
/// information that was missing.
pub(in crate::canvas) fn resolve(
    model: &PageObjects,
    query: Point,
    tolerance: f64,
) -> Option<Entity> {
    //
    // The consequence for THIS module is worse than for selection, and it is
    // the reason the change had to come here too rather than only to the pick:
    // a hover highlight is a **promise about what the next click will take**,
    // and `measure::pick` has used the leaf-aware `pick_line_in_page` since
    // the same day. A shallow highlight over a deep pick is the exact state
    // this module's header exists to prevent — a marker on one line and a
    // highlight on another.
    let target = *hit_test_point_deep(model, query, tolerance).first()?;
    // One lookup, both lists. The path and the bbox come from the same entry,
    // so they cannot describe different objects.
    let object = match target {
        HitTarget::Object(i) => model.objects.get(i)?,
        HitTarget::Leaf(i) => &model.leaves.get(i)?.object,
    };
    let bbox = object.page_bbox();
    // `pick_line_of` answers `None` for a non-path, for a curve, and for a path
    // whose nearest run is not within tolerance. All three mean the same thing
    // here — *there is an entity and it is not a straight run* — so they share
    // the bounds-only branch rather than being distinguished.
    //
    // `pick_line_of` rather than `pick_line`: the latter takes an index into
    // `objects` and so cannot be asked about a leaf. The engine split them for
    // that reason — *"the geometry never needed the index; only the lookup
    // did"* — and the provenance is this caller's knowledge, so this caller
    // states it.
    let segment = match object {
        pdfcer_core::vector::VectorObject::Path(path) => {
            pick_line_of(path, target, query, tolerance).map(|l| (l.start, l.end))
        }
        _ => None,
    };
    Some(Entity {
        target,
        segment,
        bounds: (bbox.min, bbox.max),
    })
}

/// How much wider than a hairline the highlight is drawn, in points.
///
/// Deliberately heavier than the geometry it sits on. A CAD drawing's lines
/// are hairlines, and a highlight the same weight as its subject is a line that
/// changed colour — which on a monochrome drawing viewed at a distance is not a
/// change at all. It is a screen-space width, so it does not thicken with zoom.
const HIGHLIGHT_WIDTH_PT: f32 = 3.0;

/// How transparent the highlight is.
///
/// Under 1.0 for a reason rule 4 cares about: the operator must still be able
/// to **see the line underneath**. A solid overlay would replace the geometry
/// with a coloured bar, and *"is this the line I meant"* is a question about the
/// geometry, not about the bar.
const HIGHLIGHT_ALPHA: u8 = 150;

/// The shapes for a hovered entity, in screen space.
///
/// Returns an empty vector rather than `Option` so a caller can `extend` a
/// painter unconditionally — the same shape [`super::snap::snap_marker_shapes`]
/// uses, for the same reason.
///
/// `to_screen` converts a page point, returning `None` when the point does not
/// map (off-page, or a degenerate transform). A segment with one unmappable end
/// is dropped rather than half-drawn: a highlight from a real endpoint to an
/// arbitrary fallback would be pointing at something that is not there.
pub(in crate::canvas) fn shapes(
    entity: Entity,
    color: egui::Color32,
    to_screen: impl Fn(Point) -> Option<Pos2>,
) -> Vec<Shape> {
    let tint = color.gamma_multiply(f32::from(HIGHLIGHT_ALPHA) / 255.0);
    let stroke = Stroke::new(HIGHLIGHT_WIDTH_PT, tint);

    if let Some((a, b)) = entity.segment
        && let (Some(pa), Some(pb)) = (to_screen(a), to_screen(b))
    {
        return vec![Shape::line_segment([pa, pb], stroke)];
    }

    // No straight run: outline the object instead. A rectangle rather than a
    // filled tint, because a fill over a text run or an image would hide the
    // very thing the operator is trying to identify.
    let (min, max) = entity.bounds;
    let (Some(p0), Some(p1)) = (to_screen(min), to_screen(max)) else {
        return Vec::new();
    };
    vec![Shape::rect_stroke(
        egui::Rect::from_two_pos(p0, p1),
        0.0,
        stroke,
        egui::StrokeKind::Outside,
    )]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(segment: Option<(Point, Point)>) -> Entity {
        Entity {
            target: HitTarget::Object(3),
            segment,
            bounds: (Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 50.0 }),
        }
    }

    /// A straight run is drawn as the run, not as its bounding box.
    ///
    /// The distinction is the whole feature for a diagonal. A box around a
    /// 45° line highlights a square region containing every other line that
    /// crosses it, which on a CAD drawing is most of them — it would answer
    /// *"somewhere around here"* to a question that means *"which one"*.
    #[test]
    fn a_segment_is_highlighted_as_a_line() {
        let shapes = shapes(
            entity(Some((Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }))),
            egui::Color32::RED, // NOT A THEME COLOUR: a test probe, never drawn
            |p| {
                Some(Pos2::new(
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        p.x as f32
                    },
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        p.y as f32
                    },
                ))
            },
        );
        assert_eq!(shapes.len(), 1);
        assert!(
            matches!(shapes[0], Shape::LineSegment { .. }),
            "a straight run must be drawn as a line: {:?}",
            shapes[0]
        );
    }

    /// An entity with no straight run falls back to its outline.
    ///
    /// A curve, a text run or an image. Saying nothing at all would be worse:
    /// the operator would move the pointer over something, see no response, and
    /// conclude the tool had stopped working.
    #[test]
    fn an_entity_with_no_straight_run_is_outlined() {
        // NOT A THEME COLOUR: a test probe, never drawn
        let shapes = shapes(entity(None), egui::Color32::RED, |p| {
            Some(Pos2::new(
                #[allow(clippy::cast_possible_truncation)]
                {
                    p.x as f32
                },
                #[allow(clippy::cast_possible_truncation)]
                {
                    p.y as f32
                },
            ))
        });
        assert_eq!(shapes.len(), 1);
        assert!(
            matches!(shapes[0], Shape::Rect(_)),
            "a non-linear entity must be outlined: {:?}",
            shapes[0]
        );
    }

    /// A segment with an unmappable end draws NOTHING rather than half a line.
    ///
    /// The failure this prevents is not a missing highlight, it is a
    /// **misleading** one: a line drawn from a real endpoint to a fallback
    /// position points at geometry that is not there, and the operator would
    /// aim at it.
    #[test]
    fn a_half_mappable_segment_draws_nothing() {
        let shapes = shapes(
            entity(Some((Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }))),
            egui::Color32::RED, // NOT A THEME COLOUR: a test probe, never drawn
            |p| (p.x == 0.0).then_some(Pos2::ZERO),
        );
        assert!(
            shapes.is_empty(),
            "half a segment is worse than none: {shapes:?}"
        );
    }

    /// The highlight is translucent, so the line underneath stays visible.
    ///
    /// *"Is this the line I meant"* is a question about the geometry. A solid
    /// bar over it replaces the evidence with the affordance.
    #[test]
    fn the_highlight_does_not_hide_what_it_marks() {
        // Asserted on the RENDERED stroke rather than on the constants.
        //
        // `assert!(HIGHLIGHT_ALPHA < 255)` is a constant comparison — clippy
        // says so, and clippy is right that it tests the source rather than the
        // behaviour. What matters is that the colour actually handed to the
        // painter is translucent and the width actually used is heavier than a
        // hairline, which is one `shapes` call away.
        let shapes = shapes(
            entity(Some((Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 0.0 }))),
            // NOT A THEME COLOUR: a test probe, never drawn
            egui::Color32::WHITE,
            |p| {
                Some(Pos2::new(
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        p.x as f32
                    },
                    0.0,
                ))
            },
        );
        let Some(Shape::LineSegment { stroke, .. }) = shapes.first() else {
            panic!("expected one line segment: {shapes:?}");
        };
        assert!(
            stroke.color.a() < 255,
            "an opaque highlight hides the line it is identifying: {:?}",
            stroke.color
        );
        assert!(
            stroke.width > 1.0,
            "a hairline highlight on hairline geometry is a colour change, not a highlight"
        );
    }
}
