//! # `canvas::dimlabel` — sliding a linear ce dimension's text along its line
//!
//! A press on the value text of a selected linear ce dimension moves the text
//! alone: it slides along the dimension line and the line stays where it is.
//! A press anywhere else on the dimension is `canvas::dimdrag`'s, which moves
//! line and text together.
//!
//! The text's box is the engine's `DimensionPreview::appearance.label_quad`
//! for the dimension as it stands, so the hit test aims at the text the page
//! draws. The drag changes only `text_along`; the release commits
//! `place_dimension` with the stored `offset`, one engine command and so one
//! undo entry. The live preview is the engine's bake of the moved dimension
//! through [`crate::canvas::dimpreview`].
//!
//! Coordinate spaces: the quad and the placement are page space, in points;
//! the published region and the hit test are screen space.

use egui::Pos2;
use pdfcer_core::dimension::{DimensionId, DimensionKind};

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::canvas::dimdrag::Placed;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;

/// The trace region the text's screen bounds are published under while a
/// linear ce dimension is selected.
pub const REGION: &str = "canvas.dimension-label"; // ui-text-exempt: trace region name

/// `dimension-label id=… offset=… text_along=…` — the release.
const TRACE: &str = "dimension-label"; // ui-text-exempt: diagnostic trace name

/// How far outside the text a press still counts as on it, in screen points.
const SLACK_PT: f32 = 2.0;

/// The selected ce dimension, when it is linear.
fn selected(doc: &OpenDoc, selection: &SelectionState) -> Option<(DimensionId, DimensionKind)> {
    let (id, kind) = crate::canvas::dimdrag::selected(doc, selection)?;
    matches!(kind, DimensionKind::Linear { .. }).then_some((id, kind))
}

/// The last quad asked of the engine, keyed on the geometry it was asked for,
/// so a selected dimension is baked once rather than every frame.
#[derive(Clone)]
struct Cached {
    id: DimensionId,
    kind: DimensionKind,
    quad: [pdfcer_core::vector::Point; 4],
}

fn cache_id() -> egui::Id {
    egui::Id::new("canvas.dimlabel.quad")
}

/// The page-space corners of the selected linear ce dimension's text.
fn page_quad(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selection: &SelectionState,
) -> Option<[pdfcer_core::vector::Point; 4]> {
    let (id, kind) = selected(doc, selection)?;
    let hit = ctx.data(|d| d.get_temp::<Cached>(cache_id()));
    if let Some(c) = hit.filter(|c| c.id == id && c.kind == kind) {
        return Some(c.quad);
    }
    let quad = doc
        .session
        .dimension_preview(id, &kind)
        .ok()?
        .appearance
        .label_quad;
    ctx.data_mut(|d| d.insert_temp(cache_id(), Cached { id, kind, quad }));
    Some(quad)
}

/// The text's corners in screen space, in order around the box.
#[must_use]
pub fn screen_quad(
    ctx: &egui::Context,
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
) -> Option<[Pos2; 4]> {
    let quad = page_quad(ctx, doc, selection)?;
    let page = doc.current_page()?;
    let mut out = [Pos2::ZERO; 4];
    for (slot, p) in out.iter_mut().zip(quad) {
        #[allow(clippy::cast_possible_truncation)]
        let at = Pos2::new(p.x as f32, p.y as f32);
        *slot = map.to_screen(crate::viewer::pdf_space_to_canvas(at, page)?);
    }
    Some(out)
}

/// The canvas-space corners of the text of the ce dimension that `annot`
/// draws, as it stands; any kind. One engine bake, so asked per click and
/// only of a dimension whose `/Rect` already holds the click.
#[must_use]
pub fn canvas_quad_of(
    doc: &OpenDoc,
    page: &pdfcer_core::page_tree::Page,
    annot: pdfcer_core::object::ObjId,
) -> Option<[Pos2; 4]> {
    let model = doc.session.dimension_model();
    let record = model.dimensions().iter().find(|r| r.annot == Some(annot))?;
    let quad = doc
        .session
        .dimension_preview(record.id, &record.kind)
        .ok()?
        .appearance
        .label_quad;
    let mut out = [Pos2::ZERO; 4];
    for (slot, p) in out.iter_mut().zip(quad) {
        #[allow(clippy::cast_possible_truncation)]
        let at = Pos2::new(p.x as f32, p.y as f32);
        *slot = crate::viewer::pdf_space_to_canvas(at, page)?;
    }
    Some(out)
}

/// Whether `p` lies in the convex quad `q`, of either winding, widened by
/// `slack` on every side.
pub(crate) fn inside(q: [Pos2; 4], p: Pos2, slack: f32) -> bool {
    let mut distances = [0.0_f32; 4];
    for (i, d) in distances.iter_mut().enumerate() {
        let (a, b) = (q[i], q[(i + 1) % 4]);
        let edge = b - a;
        let len = edge.length();
        if len <= f32::EPSILON {
            return false;
        }
        // Signed distance of `p` from the edge's line.
        *d = (edge.x * (p.y - a.y) - edge.y * (p.x - a.x)) / len;
    }
    distances.iter().all(|d| *d >= -slack) || distances.iter().all(|d| *d <= slack)
}

/// Whether a press at `screen` landed on the selected linear ce dimension's
/// text.
#[must_use]
pub fn label_at(
    ctx: &egui::Context,
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    screen: Pos2,
) -> bool {
    screen_quad(ctx, doc, map, selection).is_some_and(|q| inside(q, screen, SLACK_PT))
}

/// **The rule.** A page-space delta projected onto the dimension line: the
/// text slides by its along-line part and the standoff is unchanged. Returns
/// the moved dimension with the `(offset, text_along)` pair the commit writes.
#[must_use]
pub fn slid(kind: &DimensionKind, dx: f64, dy: f64) -> Option<(DimensionKind, f64, f64)> {
    let (u, _) = kind.axis_frame()?;
    let mut moved = kind.clone();
    let DimensionKind::Linear {
        offset, text_along, ..
    } = &mut moved
    else {
        return None;
    };
    *text_along += dx * u.x + dy * u.y;
    let pair = (*offset, *text_along);
    Some((moved, pair.0, pair.1))
}

/// One frame of a text drag, in canvas space.
pub struct Frame<'a> {
    /// Where the press landed.
    pub from: Pos2,
    /// Where the pointer is now.
    pub at: Pos2,
    /// Draw the preview, or commit.
    pub phase: Phase,
    /// The open document.
    pub doc: &'a OpenDoc,
    /// The selection the press was made against.
    pub selection: &'a SelectionState,
}

/// Advance one frame of a text drag. Commits on [`Phase::Complete`] and
/// returns the preview on every other frame.
pub fn drag(frame: Frame<'_>, actions: &mut Vec<Action>) -> Option<Placed> {
    let Frame {
        from,
        at,
        phase,
        doc,
        selection,
    } = frame;
    let (id, kind) = selected(doc, selection)?;
    let d = crate::canvas::moving::page_delta(at - from, doc.current_page()?)?;
    let (moved, offset, text_along) = slid(&kind, d.dx, d.dy)?;
    if phase == Phase::Complete {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "{TRACE} id={} offset={offset:.2} text_along={text_along:.2}",
                id.0
            )
        });
        actions.push(Action::Dimension(DimensionAction::Place {
            dimension: id,
            offset,
            text_along,
        }));
        return None;
    }
    Some(Placed {
        segments: crate::canvas::measure::pick::dimension_preview_segments(&moved),
        baked: crate::canvas::dimpreview::bake(doc, id, &moved),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn horizontal() -> DimensionKind {
        DimensionKind::Linear {
            a: pdfcer_core::vector::Point::new(100.0, 200.0),
            b: pdfcer_core::vector::Point::new(300.0, 200.0),
            constraint: pdfcer_core::vector::AxisConstraint::Horizontal,
            offset: 10.0,
            text_along: 0.0,
            extension_gap: [None; 2],
        }
    }

    #[test]
    fn a_slide_keeps_the_standoff() {
        let (moved, offset, along) = slid(&horizontal(), 25.0, 40.0).unwrap();
        assert!((offset - 10.0).abs() < 1e-9, "offset unchanged: {offset}");
        assert!(
            (along - 25.0).abs() < 1e-9,
            "x goes along a horizontal line: {along}"
        );
        let DimensionKind::Linear { text_along, .. } = moved else {
            panic!("still linear")
        };
        assert!((text_along - 25.0).abs() < 1e-9);
    }

    #[test]
    fn a_perpendicular_drag_moves_nothing() {
        let (_, offset, along) = slid(&horizontal(), 0.0, 40.0).unwrap();
        assert!((offset - 10.0).abs() < 1e-9);
        assert!(along.abs() < 1e-9);
    }

    #[test]
    fn inside_either_winding() {
        let q = [
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(10.0, 5.0),
            Pos2::new(0.0, 5.0),
        ];
        let r = [q[3], q[2], q[1], q[0]];
        for quad in [q, r] {
            assert!(inside(quad, Pos2::new(5.0, 2.5), 0.0));
            assert!(inside(quad, Pos2::new(11.0, 2.5), 2.0));
            assert!(!inside(quad, Pos2::new(13.0, 2.5), 2.0));
        }
    }
}
