//! The drawn segments of every markup on a page, canvas space, so a click
//! inside an unfilled box or beside a line reaches whatever lies under it.
//!
//! Contract: a markup is given a shape only when its ink is known from the
//! engine's own read of it. A filled one (an `/IC` the engine reads back) and
//! a cloudy square and any subtype not listed here keep no shape, so its `/Rect` claims the click,
//! which is the generous answer. A revision cloud keeps its rectangle too: its
//! scallops bulge past the vertices by an amount the spec does not state.

use std::collections::BTreeMap;

use egui::Pos2;
use pdfcer_core::annot_author::{MarkupSpec, spec_from_dict};
use pdfcer_core::object::{ObjId, Object};
use pdfcer_core::page_tree::Page;

use crate::app::state::OpenDoc;

/// Segments drawn round an unfilled ellipse; enough that the chord error stays
/// under a click tolerance on any sheet size a drawing reaches.
const ELLIPSE_SEGMENTS: usize = 48;

/// Every shaped markup on `page`, by id.
#[must_use]
pub fn markup_shapes(doc: &OpenDoc, page: &Page) -> BTreeMap<ObjId, Vec<(Pos2, Pos2)>> {
    let graph = doc.session.graph();
    let mut out = BTreeMap::new();
    for annot in pdfcer_core::annot::page_annotations(&graph, page.id) {
        let Some(id) = annot.id else { continue };
        let Some(Object::Dict(dict)) = doc.session.value(id) else {
            continue;
        };
        let Ok(spec) = spec_from_dict(&graph, dict) else {
            continue;
        };
        let page_points = page_segments(&spec);
        let segments: Vec<(Pos2, Pos2)> = page_points
            .into_iter()
            .filter_map(|(a, b)| Some((to_canvas(a, page)?, to_canvas(b, page)?)))
            .collect();
        if !segments.is_empty() {
            out.insert(id, segments);
        }
    }
    out
}

/// A page-space point onto the canvas.
fn to_canvas((x, y): (f64, f64), page: &Page) -> Option<Pos2> {
    #[allow(clippy::cast_possible_truncation)]
    crate::viewer::pdf_space_to_canvas(Pos2::new(x as f32, y as f32), page)
}

/// The ink of `spec`, page space, or nothing when the rectangle must stand.
fn page_segments(spec: &MarkupSpec) -> Vec<((f64, f64), (f64, f64))> {
    match spec {
        MarkupSpec::Line { start, end, .. } => vec![(*start, *end)],
        MarkupSpec::PolyLine { vertices, .. } => chain(vertices, false),
        MarkupSpec::Polygon {
            vertices,
            interior: None,
            ..
        } => chain(vertices, true),
        MarkupSpec::Ink { strokes, .. } => strokes.iter().flat_map(|s| chain(s, false)).collect(),
        MarkupSpec::Square {
            rect,
            interior: None,
            border_width,
            border_effect: None,
            ..
        } => {
            let [l, b, r, t] = inset(rect, *border_width);
            chain(&[(l, b), (r, b), (r, t), (l, t)], true)
        }
        MarkupSpec::Circle {
            rect,
            interior: None,
            border_width,
            ..
        } => ellipse(inset(rect, *border_width)),
        _ => Vec::new(),
    }
}

/// Consecutive pairs of `points`, closed back to the first when `closed`.
fn chain(points: &[(f64, f64)], closed: bool) -> Vec<((f64, f64), (f64, f64))> {
    let mut out: Vec<_> = points.windows(2).map(|w| (w[0], w[1])).collect();
    if closed && points.len() > 2 {
        out.push((points[points.len() - 1], points[0]));
    }
    out
}

/// `/Rect` drawn in by half the pen, where the stroke's centre line runs.
fn inset(rect: &pdfcer_core::page_tree::Rect, width: f64) -> [f64; 4] {
    let h = (width / 2.0)
        .min((rect.urx - rect.llx) / 2.0)
        .min((rect.ury - rect.lly) / 2.0)
        .max(0.0);
    [rect.llx + h, rect.lly + h, rect.urx - h, rect.ury - h]
}

/// The ellipse inscribed in `[l, b, r, t]`, as a closed polygon.
fn ellipse([l, b, r, t]: [f64; 4]) -> Vec<((f64, f64), (f64, f64))> {
    let (cx, cy, rx, ry) = ((l + r) / 2.0, (b + t) / 2.0, (r - l) / 2.0, (t - b) / 2.0);
    #[allow(clippy::cast_precision_loss)]
    let points: Vec<(f64, f64)> = (0..ELLIPSE_SEGMENTS)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / ELLIPSE_SEGMENTS as f64;
            (rx.mul_add(a.cos(), cx), ry.mul_add(a.sin(), cy))
        })
        .collect();
    chain(&points, true)
}
