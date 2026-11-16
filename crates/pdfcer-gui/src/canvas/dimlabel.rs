//! # `canvas::dimlabel` — where a linear ce dimension's value text is
//!
//! The text's box is the engine's `DimensionPreview::appearance.label_quad`
//! for the dimension as it stands, so a hit test aims at the text the page
//! draws. A press on it drags the dimension exactly as a press on its line
//! does (`canvas::dimdrag`): the text can sit outside the line's grab box, so
//! it is asked about separately.
//!
//! Coordinate spaces: the quad is page space, in points; the published region
//! and the hit test are screen space.

use egui::Pos2;
use pdfcer_core::dimension::{DimensionId, DimensionKind};

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;

/// The trace region the text's screen bounds are published under while a
/// linear ce dimension is selected.
pub const REGION: &str = "canvas.dimension-label"; // ui-text-exempt: trace region name

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
