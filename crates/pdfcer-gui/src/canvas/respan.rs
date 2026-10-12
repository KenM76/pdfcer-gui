//! # `canvas::respan` — **a text markup's two end grips, and re-spanning it**
//!
//! A selected `/Highlight`, `/Underline`, `/StrikeOut` or `/Squiggly` shows a
//! grip at each end of the text it covers. Dragging one re-runs the text
//! selection from the other end to the pointer, and the release hands the
//! resulting quads to `EditSession::respan_text_markup` — one undo entry.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/respan.md`.
//!
//! The grips ride the markup-node route: `pressing` resolves a press on one to
//! `DragKind::MarkupVertex { index }` (0 = start, 1 = end) and
//! `vertexroute::dragged` hands the drag here whenever [`ends`] answers, so
//! the two families never both claim one selection.

use pdfcer_core::annot_author::{MarkupSpec, Quad};
use pdfcer_core::object::ObjId;
use pdfcer_core::vector::Point;

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{AnnotKind, SelectionState};

/// The region each end grip is published under — `canvas.markup-end.0` is the
/// start, `.1` the end.
pub const END_REGION: &str = "canvas.markup-end"; // ui-text-exempt: trace region name

/// `markup-respan id=… index=… quads=…` — the release was understood and the
/// edit raised. `quads=` is the count the edit will write.
pub const TRACE_RESPAN: &str = "markup-respan"; // ui-text-exempt: diagnostic trace name

/// `markup-respan-declined id=… index=… reason=no-text` — the release landed
/// where the selection could not follow text, so nothing was raised.
pub const TRACE_DECLINED: &str = "markup-respan-declined"; // ui-text-exempt: diagnostic trace name

/// How far inside the fixed end the re-run selection is anchored, in points,
/// so the caret hit lands on the first covered glyph rather than on the
/// boundary shared with its neighbour.
const ANCHOR_INSET_PT: f64 = 1.0;

/// **The covered span of the selected text markup**, page space.
#[derive(Debug, Clone, PartialEq)]
pub struct Ends {
    /// The annotation.
    pub id: ObjId,
    /// The page it is on.
    pub page: usize,
    /// Its `/QuadPoints`, as the engine's reader returns them.
    pub quads: Vec<Quad>,
}

/// The four text-markup subtypes — the ones `respan_text_markup` accepts.
fn is_text_markup(subtype: &str) -> bool {
    matches!(
        subtype,
        "Highlight" | "Underline" | "StrikeOut" | "Squiggly"
    )
}

/// **The selected text markup's quads**, or `None` for any other selection,
/// a locked mark, a mark off the page in view, or one whose quads the engine
/// cannot read.
#[must_use]
pub fn ends(doc: &OpenDoc, selection: &SelectionState) -> Option<Ends> {
    use pdfcer_core::object::Object;
    let annot = selection.annot()?;
    let target = &annot.target;
    if target.kind != AnnotKind::Markup || target.locked || !is_text_markup(&target.subtype) {
        return None;
    }
    // `OpenDoc::page_text` extracts the page in view and no other, so a mark
    // on any other page has no text to re-run a selection against.
    if target.page != doc.view.page_index {
        return None;
    }
    let graph = doc.session.graph();
    let Some(Object::Dict(dict)) = doc.session.value(target.id) else {
        return None;
    };
    let MarkupSpec::TextMarkup { quads, .. } =
        pdfcer_core::annot_author::spec_from_dict(&graph, dict).ok()?
    else {
        return None;
    };
    (!quads.is_empty()).then_some(Ends {
        id: target.id,
        page: target.page,
        quads,
    })
}

fn mid(a: (f64, f64), b: (f64, f64)) -> Point {
    Point::new(f64::midpoint(a.0, b.0), f64::midpoint(a.1, b.1))
}

impl Ends {
    /// The grip at `index`, page space: 0 is the middle of the first quad's
    /// leading edge, 1 the middle of the last quad's trailing edge.
    #[must_use]
    pub fn grip(&self, index: usize) -> Option<Point> {
        match index {
            0 => self.quads.first().map(|q| mid(q.ul, q.ll)),
            1 => self.quads.last().map(|q| mid(q.ur, q.lr)),
            _ => None,
        }
    }

    /// Where the re-run selection is anchored when grip `moving` is dragged:
    /// the OTHER end, stepped [`ANCHOR_INSET_PT`] (at most a quarter of its
    /// quad's length) back into the covered text.
    fn anchor(&self, moving: usize) -> Option<Point> {
        let (edge, inner) = match moving {
            0 => {
                let q = self.quads.last()?;
                (mid(q.ur, q.lr), mid(q.ul, q.ll))
            }
            _ => {
                let q = self.quads.first()?;
                (mid(q.ul, q.ll), mid(q.ur, q.lr))
            }
        };
        let (dx, dy) = (inner.x - edge.x, inner.y - edge.y);
        let length = dx.hypot(dy);
        if length <= f64::EPSILON {
            return Some(edge);
        }
        let t = (ANCHOR_INSET_PT / length).min(0.25);
        Some(Point::new(edge.x + dx * t, edge.y + dy * t))
    }
}

/// The page-space outline of `quads`, as segments, for the preview painter.
#[must_use]
pub fn outline(quads: &[Quad]) -> Vec<(Point, Point)> {
    let p = |c: (f64, f64)| Point::new(c.0, c.1);
    quads
        .iter()
        .flat_map(|q| {
            [
                (p(q.ul), p(q.ur)),
                (p(q.ur), p(q.lr)),
                (p(q.lr), p(q.ll)),
                (p(q.ll), p(q.ul)),
            ]
        })
        .collect()
}

#[allow(clippy::cast_possible_truncation)]
fn to_canvas(point: Point, page: &pdfcer_core::page_tree::Page) -> Option<egui::Pos2> {
    crate::viewer::pdf_space_to_canvas(egui::Pos2::new(point.x as f32, point.y as f32), page)
}

/// **Both end grips of the selected text markup, in CANVAS space** — empty for
/// every other selection.
#[must_use]
pub fn grips(doc: &OpenDoc, selection: &SelectionState) -> Vec<egui::Pos2> {
    let Some(span) = ends(doc, selection) else {
        return Vec::new();
    };
    let Some(page) = doc.pages.get(span.page) else {
        return Vec::new();
    };
    (0..2)
        .filter_map(|i| to_canvas(span.grip(i)?, page))
        .collect()
}

/// **Which end grip a press at `screen` landed on**, with the node anchors'
/// tolerance. The later grip wins a tie, as `annotnodes::node_at`'s does.
#[must_use]
pub fn grip_at(
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    screen: egui::Pos2,
) -> Option<usize> {
    let tolerance = crate::canvas::annotnodes::grab_tolerance();
    grips(doc, selection)
        .into_iter()
        .enumerate()
        .rfind(|(_, canvas)| map.to_screen(*canvas).distance(screen) <= tolerance)
        .map(|(index, _)| index)
}

/// What one frame of an end-grip drag needs.
pub struct Frame<'a> {
    /// Which grip: 0 start, 1 end.
    pub index: usize,
    /// Where the press landed, canvas space.
    pub from: egui::Pos2,
    /// Where the pointer is now, canvas space.
    pub at: egui::Pos2,
    /// Draw, or commit.
    pub phase: Phase,
    /// The open document.
    pub doc: &'a OpenDoc,
    /// The selection naming the mark.
    pub selection: &'a SelectionState,
}

/// **Advance one frame of an end-grip drag.** Returns the page-space outline
/// to preview — the new span while text follows the pointer, the current one
/// while it does not — or `None` on the committing frame.
pub fn drag(frame: Frame<'_>, actions: &mut Vec<Action>) -> Option<Vec<(Point, Point)>> {
    let Frame {
        index,
        from,
        at,
        phase,
        doc,
        selection,
    } = frame;
    let span = ends(doc, selection)?;
    let page = doc.pages.get(span.page)?;
    // The grab point survives: the grip moves by the pointer's delta.
    let moved = to_canvas(span.grip(index)?, page)? + (at - from);
    let anchor = to_canvas(span.anchor(index)?, page)?;
    let quads = respanned(doc, span.page, anchor, moved);
    if phase != Phase::Complete {
        return Some(outline(quads.as_deref().unwrap_or(&span.quads)));
    }
    match quads {
        Some(quads) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                format!(
                    "{TRACE_RESPAN} id={} index={index} quads={}->{}",
                    span.id.num,
                    span.quads.len(),
                    quads.len()
                )
            });
            actions.push(Action::Annot(AnnotAction::RespanTextMarkup {
                id: span.id,
                quads,
            }));
        }
        None => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                format!(
                    "{TRACE_DECLINED} id={} index={index} reason=no-text",
                    span.id.num
                )
            });
            actions.push(Action::Annot(AnnotAction::DeclineNodeEdit {
                why: crate::text::markup::NodeEditRefusal::NoTextThere,
            }));
        }
    }
    None
}

/// The quads a text selection from `anchor` to `focus` (canvas space) covers
/// on `page_index`, or `None` where it covers no text.
fn respanned(
    doc: &OpenDoc,
    page_index: usize,
    anchor: egui::Pos2,
    focus: egui::Pos2,
) -> Option<Vec<Quad>> {
    let text = doc.page_text()?;
    let page = doc.pages.get(page_index)?;
    let ctx = crate::canvas::textsel::PageContext {
        text: &text,
        page,
        index: page_index,
        epoch: doc.edit_epoch,
    };
    let picked = crate::canvas::textsel::drag(&ctx, anchor, focus)?;
    let quads = picked.marks(doc.edit_epoch);
    (!quads.is_empty()).then(|| quads.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(x0: f64, x1: f64, y0: f64, y1: f64) -> Quad {
        Quad {
            ul: (x0, y1),
            ur: (x1, y1),
            ll: (x0, y0),
            lr: (x1, y0),
        }
    }

    fn span(quads: Vec<Quad>) -> Ends {
        Ends {
            id: ObjId::new(5, 0),
            page: 0,
            quads,
        }
    }

    #[test]
    fn the_grips_sit_on_the_leading_edge_of_the_first_quad_and_the_trailing_edge_of_the_last() {
        let s = span(vec![
            quad(80.0, 120.0, 298.0, 308.0),
            quad(40.0, 90.0, 278.0, 288.0),
        ]);
        assert_eq!(s.grip(0), Some(Point::new(80.0, 303.0)));
        assert_eq!(s.grip(1), Some(Point::new(90.0, 283.0)));
        assert_eq!(s.grip(2), None);
    }

    #[test]
    fn the_anchor_is_the_other_end_stepped_into_the_text() {
        let s = span(vec![quad(80.0, 120.0, 298.0, 308.0)]);
        assert_eq!(s.anchor(1), Some(Point::new(81.0, 303.0)));
        assert_eq!(s.anchor(0), Some(Point::new(119.0, 303.0)));
        // A quad shorter than four points steps a quarter of its length.
        let narrow = span(vec![quad(80.0, 82.0, 298.0, 308.0)]);
        assert_eq!(narrow.anchor(1), Some(Point::new(80.5, 303.0)));
    }

    #[test]
    fn the_outline_closes_every_quad() {
        let segments = outline(&[quad(0.0, 10.0, 0.0, 5.0)]);
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].0, segments[3].1);
    }

    #[test]
    fn only_the_four_text_markups_take_the_verb() {
        for s in ["Highlight", "Underline", "StrikeOut", "Squiggly"] {
            assert!(is_text_markup(s), "{s}");
        }
        for s in ["Line", "Ink", "Square", "FreeText"] {
            assert!(!is_text_markup(s), "{s}");
        }
    }
}
