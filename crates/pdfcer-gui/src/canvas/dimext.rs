//! # `canvas::dimext` — the extension-line grips of a selected linear ce dimension
//!
//! A selected linear ce dimension shows one grip at the near end of each
//! extension line — the end nearest the measured point. Dragging it along the
//! line lengthens or shortens that extension line by changing its gap, and
//! the release commits `EditSession::set_dimension_extension_gap`. The number
//! the dimension prints never changes.
//!
//! The live preview is the engine's own bake of the dimension with the new
//! gap, through [`crate::canvas::dimpreview`], so what is shown mid-drag is
//! what the release writes.
//!
//! Coordinate spaces: grips are published in CANVAS space; the gap is in page
//! space, points, measured along the extension line from the picked point.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/dimext.md`.

use pdfcer_core::dimension::{DimensionEnd, DimensionId, DimensionKind, resolve_style};
use pdfcer_core::vector::Point;

use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::OpenDoc;
use crate::canvas::dimdrag::Placed;
use crate::canvas::gesture::Phase;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;

/// The trace region each grip is published under, suffixed with the end's
/// index — `canvas.dimension-extension.0` (end A) and `.1` (end B).
pub const GRIP_REGION: &str = "canvas.dimension-extension"; // ui-text-exempt: trace region name

const TRACE: &str = "dimension-extension-gap"; // ui-text-exempt: diagnostic trace name

/// The smallest distance, in points, a grip is kept short of the dimension
/// line. The engine draws no extension line at all once the gap reaches the
/// reach, so the drag stops just before it.
const MIN_LINE_PT: f64 = 0.5;

/// The selected linear ce dimension, with the style its extension lines are
/// drawn under.
fn selected(
    doc: &OpenDoc,
    selection: &SelectionState,
) -> Option<(
    DimensionId,
    DimensionKind,
    pdfcer_core::dimension::DimensionStyle,
)> {
    let (id, kind) = crate::canvas::dimdrag::selected(doc, selection)?;
    if !matches!(kind, DimensionKind::Linear { .. }) {
        return None;
    }
    let model = doc.session.dimension_model();
    let record = model.dimensions().iter().find(|r| r.id == id)?;
    let group = model.group(record.group)?;
    Some((id, kind, resolve_style(group, &record.style)))
}

fn end_of(index: usize) -> DimensionEnd {
    if index == 0 {
        DimensionEnd::A
    } else {
        DimensionEnd::B
    }
}

/// A grip being dragged: where it is this pass, in CANVAS space. Stamped
/// with the pass that wrote it so a drag that ended without a release frame
/// leaves nothing behind.
#[derive(Clone, Copy)]
struct Live {
    pass: u64,
    end: DimensionEnd,
    at: egui::Pos2,
}

fn live_id() -> egui::Id {
    egui::Id::new("canvas.dimext.live")
}

/// Each drawn extension line's grip, in CANVAS space, with its end. An end
/// whose extension line is not drawn (its gap already reaches the dimension
/// line) has no grip.
#[must_use]
pub fn grips(doc: &OpenDoc, selection: &SelectionState) -> Vec<(DimensionEnd, egui::Pos2)> {
    let Some((_, kind, style)) = selected(doc, selection) else {
        return Vec::new();
    };
    segment_starts(doc, &kind, style)
}

/// [`grips`], with a grip being dragged this pass drawn where the drag has
/// it. The page raster still shows the committed line under the preview, so
/// a shortening drag is visible only through the grip moving with it.
#[must_use]
pub fn grips_drawn(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selection: &SelectionState,
) -> Vec<(DimensionEnd, egui::Pos2)> {
    let mut out = grips(doc, selection);
    let live = ctx.data(|d| d.get_temp::<Live>(live_id()));
    if let Some(live) = live.filter(|l| l.pass == ctx.cumulative_pass_nr()) {
        for (end, at) in &mut out {
            if *end == live.end {
                *at = live.at;
            }
        }
    }
    out
}

fn segment_starts(
    doc: &OpenDoc,
    kind: &DimensionKind,
    style: pdfcer_core::dimension::DimensionStyle,
) -> Vec<(DimensionEnd, egui::Pos2)> {
    let (Some(segments), Some(page)) = (
        kind.extension_segments(style),
        doc.pages.get(doc.view.page_index),
    ) else {
        return Vec::new();
    };
    segments
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let (start, _) = (*s)?;
            #[allow(clippy::cast_possible_truncation)]
            let at = egui::Pos2::new(start.x as f32, start.y as f32);
            Some((end_of(i), crate::viewer::pdf_space_to_canvas(at, page)?))
        })
        .collect()
}

/// Which grip a press at `screen` landed on, if any.
#[must_use]
pub fn grip_at(
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
    screen: egui::Pos2,
) -> Option<DimensionEnd> {
    let tolerance = crate::canvas::dimdrag::VERTEX_HANDLE_PT / 2.0 + 3.0;
    grips(doc, selection)
        .into_iter()
        .rfind(|(_, canvas)| map.to_screen(*canvas).distance(screen) <= tolerance)
        .map(|(end, _)| end)
}

/// The gap a drag from `from` to `at` (canvas space) asks for, clamped so the
/// extension line is still drawn. Moves by the pointer's delta, so the grab
/// point is kept.
fn dragged_gap(
    kind: &DimensionKind,
    style: pdfcer_core::dimension::DimensionStyle,
    end: DimensionEnd,
    along: f64,
) -> Option<f64> {
    let (start, _) = kind.extension_segments(style)?[end.index()]?;
    let (_, _, pa, pb) = kind.linear_geometry()?;
    let p = if end == DimensionEnd::A { pa } else { pb };
    let was = (start.x - p.x).hypot(start.y - p.y);
    let reach = kind.extension_reach(end, style)?;
    let top = (reach - MIN_LINE_PT).max(0.0);
    Some((was + along).clamp(0.0, top))
}

/// The page-space distance the pointer moved along end `end`'s extension
/// line, positive towards the dimension line.
fn along_line(kind: &DimensionKind, end: DimensionEnd, delta: Point) -> Option<f64> {
    let (dim_a, dim_b, pa, pb) = kind.linear_geometry()?;
    let (p, d) = if end == DimensionEnd::A {
        (pa, dim_a)
    } else {
        (pb, dim_b)
    };
    let len = (d.x - p.x).hypot(d.y - p.y);
    if !len.is_finite() || len <= f64::EPSILON {
        return None;
    }
    Some((delta.x * (d.x - p.x) + delta.y * (d.y - p.y)) / len)
}

/// One frame of an extension-grip drag.
pub struct Frame<'a> {
    /// The frame's context, which carries the dragged grip to the painter.
    pub ctx: &'a egui::Context,
    /// Which extension line, sampled at the press.
    pub end: DimensionEnd,
    /// Where the press landed, in canvas space.
    pub from: egui::Pos2,
    /// Where the pointer is now, in canvas space.
    pub at: egui::Pos2,
    /// Draw, or commit.
    pub phase: Phase,
    /// The open document.
    pub doc: &'a OpenDoc,
    /// The selection naming the dimension.
    pub selection: &'a SelectionState,
}

/// Advance one frame: the preview while dragging, one
/// [`DimensionAction::SetExtensionGap`] on release. `None` when there is
/// nothing to preview.
pub fn drag(frame: Frame<'_>, actions: &mut Vec<Action>) -> Option<Placed> {
    let Frame {
        ctx,
        end,
        from,
        at,
        phase,
        doc,
        selection,
    } = frame;
    let (id, kind, style) = selected(doc, selection)?;
    let page = doc.pages.get(doc.view.page_index)?;
    let a = crate::viewer::canvas_to_pdf_space(from, page)?;
    let b = crate::viewer::canvas_to_pdf_space(at, page)?;
    let delta = Point::new(f64::from(b.x - a.x), f64::from(b.y - a.y));
    let gap = dragged_gap(&kind, style, end, along_line(&kind, end, delta)?)?;

    if phase == Phase::Complete {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("{TRACE} id={} end={} gap={gap:.2}", id.0, end.index())
        });
        actions.push(Action::Dimension(DimensionAction::SetExtensionGap {
            dimension: id,
            end,
            gap: Some(gap),
        }));
        return None;
    }
    let mut moved = kind;
    if let DimensionKind::Linear { extension_gap, .. } = &mut moved {
        extension_gap[end.index()] = Some(gap);
    }
    if let Some(&(_, at)) = segment_starts(doc, &moved, style)
        .iter()
        .find(|(e, _)| *e == end)
    {
        let pass = ctx.cumulative_pass_nr();
        ctx.data_mut(|d| d.insert_temp(live_id(), Live { pass, end, at }));
    }
    Some(Placed {
        segments: super::measure::pick::dimension_preview_segments(&moved),
        baked: super::dimpreview::bake(doc, id, &moved),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::dimension::{Group, GroupId, StyleOverrides, Unit};

    fn linear() -> DimensionKind {
        DimensionKind::Linear {
            a: Point::new(0.0, 0.0),
            b: Point::new(100.0, 0.0),
            constraint: pdfcer_core::vector::AxisConstraint::Aligned,
            offset: 40.0,
            text_along: 0.0,
            extension_gap: [None; 2],
        }
    }

    #[test]
    fn the_gap_follows_the_drag_along_the_line_and_stays_drawable() {
        let kind = linear();
        let group = Group::new(GroupId(0), "Plan", Unit::Millimeter);
        let style = resolve_style(&group, &StyleOverrides::default());
        let end = DimensionEnd::A;
        let along = along_line(&kind, end, Point::new(3.0, 5.0)).unwrap();
        // Only the component along the extension line counts.
        assert!((along - 5.0).abs() < 1e-9, "{along}");
        let reach = kind.extension_reach(end, style).unwrap();
        let far = dragged_gap(&kind, style, end, 1e6).unwrap();
        assert!(far < reach, "{far} must stay short of {reach}");
        assert_eq!(dragged_gap(&kind, style, end, -1e6), Some(0.0));
    }
}
