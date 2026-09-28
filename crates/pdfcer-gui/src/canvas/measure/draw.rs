//! **What the measure tools draw**, and the one projection they draw through.
//!
//! The parent decides what a click *means* — which point is picked, whether the
//! gesture is finishable, what action it commits. This module decides what the
//! operator sees before any of that has happened: the snap indicator, the
//! hover highlight, the rubber-band, the committed-pick rings and the fitted
//! circle. R8b calls those the cursor, and they are the one class of mark the
//! canvas is allowed to carry that is not content.
//!
//! ## Why the projection lives here rather than with the picking
//!
//! [`page_to_screen`] is the only place in `canvas/measure/` that crosses from
//! PDF user space to the painter's space, and every mark this module makes goes
//! through it. Picking never needs it: a pick is resolved in page space and
//! compared in page space. So the conversion is a property of drawing, and its
//! own header records the defect that follows from splitting it — a mark
//! authored in the right place and drawn in the wrong one, invisible to every
//! test about *which* point is picked.
//!
//! ## Reach
//!
//! `Preview`, `preview`, `SNAP_MARKER_PT` and `page_to_screen` are spelled
//! `pub(in crate::canvas)` rather than `pub(super)`, and the distinction is
//! load-bearing: `super` here is `crate::canvas::measure`, one level narrower
//! than it was in the parent, and `canvas::painting` — which calls all four —
//! sits outside it. The parent re-exports them so `measure::preview` and
//! `measure::page_to_screen` stay the spelling every caller and every citation
//! uses.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/measure/draw.md`.

use egui::{Pos2, Ui};
use pdfcer_core::dimension::{DimensionKind, StyleOverrides, author_dimension, resolve_style};
use pdfcer_core::vector::Point;

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::snap;
use crate::viewer;

use super::state::MeasureState;
use super::{MeasureKind, Resolved, hover, pick, read};

/// **Draw what the next click would commit.**
pub(in crate::canvas) struct Preview<'a> {
    /// The open document, for the page transform.
    pub doc: &'a OpenDoc,
    /// The page being drawn on.
    pub page_index: usize,
    /// Which measure tool is armed.
    pub kind: MeasureKind,
    /// The frame's screen ⟷ canvas map. **The projection every mark here goes
    /// through**, and the reason this struct exists — see [`preview`].
    pub map: &'a PageMapping,
    /// Where the pointer would pick, resolved once for the frame.
    pub hover: Option<Resolved>,
}

pub(in crate::canvas) fn preview(ui: &Ui, preview: Preview<'_>) {
    let Preview {
        doc,
        page_index,
        kind,
        map,
        hover,
    } = preview;
    let Some(page) = doc.current_page() else {
        return;
    };
    let ctx = ui.ctx();
    // The SECOND instance of the same bail, and it is why fixing
    // `resolve_hover` alone changed nothing.
    //
    // `read` returns `None` until the operator has clicked once, because
    // [`load`] builds a default and only the click paths [`store`] it. This
    // function returned early on that, so a freshly armed tool painted no snap
    // marker and no hover highlight — the exact state the operator reported,
    // and the exact state the comment forty lines below promises is handled.
    //
    // A driven run found `resolve_hover` producing `entity=1 snap=1` while
    // nothing was drawn, which is what separated the two: one instrument said
    // the answer existed and another said it was never painted. Neither alone
    // would have located it.
    //
    // The fallback is a value, not a write. Painting must not mutate shared
    // state, and `kind` is already on `Preview` because the caller knew what
    // was armed.
    let st = read(ctx)
        .filter(|s| s.page_index == page_index)
        .unwrap_or_else(|| MeasureState::for_kind(page_index, kind));
    let color = snap::snap_indicator_tint(ctx)
        .unwrap_or_else(|| egui_shell::theme::Theme::canvas_selection_ink(ctx));

    // The picked POINTS, marked, and drawn on EVERY frame the set is
    // non-empty — not only while the pointer is over the canvas.
    //
    // A `hover` of `None` means the pointer has left the widget, which for the
    // other tools means there is nothing to preview against. Here it means the
    // operator has moved to the Tool panel or the ribbon, which is exactly when
    // they most need to still be able to see what is in the set.
    //
    // The marker's GLYPH is the snap kind's, through the same
    // `snap::snap_marker_shapes` the hover indicator uses — so a point picked
    // on an endpoint is marked the way an endpoint is marked, and the operator
    // reads one vocabulary rather than two. A FREE point gets the endpoint
    // glyph, because that is what it is: a terminus the operator asserted.
    // Nothing distinguishes it ON THE CANVAS, deliberately — rule 4 puts that
    // disclosure off-canvas, in the Tool panel's list, where it can be read
    // rather than decoded.
    let painter = ui.painter();
    let stroke = egui::Stroke::new(1.5, color);
    for point in st.circular.points() {
        // `None` when the page transform refuses the point — the same bail
        // every other projection here takes, and the right one: one marker
        // fewer beats a panic in the frame that is trying to draw.
        let Some(at) = page_to_screen(point.at, page, map) else {
            continue;
        };
        let kind = match point.origin {
            pick::PickOrigin::Snapped(k) => k,
            pick::PickOrigin::Free => pdfcer_core::vector::snap::SnapKind::Endpoint,
        };
        painter.extend(snap::snap_marker_shapes(at, kind, color, SNAP_MARKER_PT));
        // …and a ring around it, which is the ONE thing that distinguishes
        // a committed pick from the hover marker under the pointer.
        //
        // Without it the two are the same glyph at the same size, and while the
        // pointer is over a picked point the operator cannot tell *"this is in
        // the fit"* from *"this is where a click would land"*. Those are
        // different claims — rule 4 calls the second one the cursor — and a
        // surface that renders both identically is one an operator has to
        // decode rather than read.
        //
        // A ring rather than a second colour: colour is how this canvas says
        // *kind*, and spending it on *state* would mean an endpoint pick and a
        // midpoint pick stopped being distinguishable to make room.
        painter.circle_stroke(at, SNAP_MARKER_PT * PICKED_RING_SCALE, stroke);
    }

    // The hovered entity, drawn UNDER the snap marker.
    //
    // Order is the whole of it: the highlight is a wide translucent stroke and
    // the marker is a small opaque glyph, so painting the highlight second
    // would put a coloured bar over the very node it is meant to accompany. The
    // node is the precise statement and must stay legible; the line is the
    // context.
    //
    // Drawn before the in-progress check for the same reason the marker is —
    // it does its work while the operator is deciding *where to click first*.
    if let Some(entity) = hover.and_then(|h| h.entity) {
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                // The list is named as well as the index. A page has two,
                // and a trace that cannot tell `objects[7]` from `leaves[7]`
                // is a trace that cannot be read back.
                "measure-hover-entity {}={} segment={}",
                match entity.target {
                    pdfcer_core::vector::hit::HitTarget::Object(_) => "object",
                    pdfcer_core::vector::hit::HitTarget::Leaf(_) => "leaf",
                },
                match entity.target {
                    pdfcer_core::vector::hit::HitTarget::Object(i)
                    | pdfcer_core::vector::hit::HitTarget::Leaf(i) => i,
                },
                u8::from(entity.segment.is_some())
            )
        });
        ui.painter().extend(hover::shapes(entity, color, |p| {
            page_to_screen(p, page, map)
        }));
    }

    // The snap indicator is drawn BEFORE the in-progress check, and that is
    // the point of it.
    //
    // It has to appear while the operator is still deciding *where to click
    // first* — that is when it does its work, by saying "this click will land
    // on that endpoint, not where your pointer is". Gating it on a gesture
    // already being in progress would show it only after the first pick, i.e.
    // everywhere except the place it is needed most.
    //
    // This is also what makes the snap honest rather than sneaky
    // (`pdfce_FeatureRequests/README.md` rule 4): the point is moved, and the
    // operator is told, before anything is committed. A snap that silently
    // relocated a click would be an inference applied without disclosure.
    if let Some(c) = hover.and_then(|h| h.candidate)
        && let Some(screen) = page_to_screen(c.point, page, map)
    {
        // The marker's screen position and the pointer's, on one line.
        //
        // The evidence for an invariant that is **true by the definition of
        // snapping**: a snap marker is never further from the pointer than the
        // snap tolerance. That is what "snap" means — the click is being moved
        // to something *near* where the operator is aiming.
        //
        // It exists because of the defect `resolve_hover`'s own docs record:
        // the preview was resolved against a raw SCREEN position while the
        // click used a converted CANVAS one, so the marker sat away from the
        // pointer by the scroll origin over the zoom. It survived four days and
        // no unit test could have seen it — both functions were individually
        // correct and the caller mixed two spaces that are the same type.
        //
        // `dx`/`dy` rather than a distance: a pure-x or pure-y offset says
        // "one axis of the conversion", and a reader chasing a regression
        // wants to know which.
        crate::diag::trace(|| {
            let p = ctx.pointer_latest_pos();
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "measure-snap-marker kind={:?} marker={:.1},{:.1} pointer={:?} dx={:.1} dy={:.1} tol={:.2}",
                c.kind,
                screen.x,
                screen.y,
                p.map(|q| (q.x, q.y)),
                p.map_or(f32::NAN, |q| screen.x - q.x),
                p.map_or(f32::NAN, |q| screen.y - q.y),
                map.snap_tolerance(),
            )
        });
        // Zoom-invariant: the marker is a screen-space affordance, so its size
        // is in points and does not grow with magnification.
        ui.painter().extend(snap::snap_marker_shapes(
            screen,
            c.kind,
            color,
            SNAP_MARKER_PT,
        ));
    }

    if !st.gesture_in_progress() {
        return;
    }

    // A measured dimension waiting for its placing click follows the pointer,
    // value text included, whatever tool measured it.
    if let Some(placing) = &st.placing {
        let Some(at) = hover.map(|h| h.at) else {
            return;
        };
        let kind = placing.at(at);
        // The engine's own bake, the pixels `add_dimension` will write; the
        // outline and label below stand in only when it refuses or is off
        // screen.
        let baked = crate::canvas::dimpreview::bake_new(doc, st.group, &kind).is_some_and(|b| {
            crate::canvas::dimpreview::paint(painter, doc, page, map, painter.clip_rect(), &b)
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!("measure-place-preview baked={}", u8::from(baked))
        });
        if !baked {
            draw_dimension(painter, stroke, &kind, page, map);
            draw_label(painter, doc, st.group, &kind, page, map, color);
        }
        return;
    }

    let segments: Vec<(Point, Point)> = match kind {
        // The reference line, drawn exactly as the linear tool draws its
        // measuring segment — because it IS one. `ScalePick::line` is a
        // `LinearPick`, so an operator calibrating sees the same constrained
        // A-to-pointer rubber band, the same snap markers and the same
        // H/V/aligned behaviour they already know from placing a dimension.
        //
        // Once both points are picked the segment stops following the pointer
        // and the dialog is up, so the line stays drawn while the operator
        // types what it represents — which is the picture that makes the
        // question answerable: *this* line is how long?
        MeasureKind::Scale => {
            if st.scale.drawn_pdf_length.is_some() {
                // Complete: hold the committed line still while the operator
                // types. `placing_preview` is fed the pointer only so it can
                // answer at all; with both points picked the segment it
                // returns is the picked line and does not follow the pointer.
                hover
                    .and_then(|h| st.scale.line.placing_preview(h.at))
                    .as_ref()
                    .map_or_else(Vec::new, pick::dimension_preview_segments)
            } else {
                let Some(at) = hover.map(|h| h.at) else {
                    return;
                };
                st.scale.line.preview_segment(at).into_iter().collect()
            }
        }
        MeasureKind::Linear => {
            // The placing preview needs the pointer; the measuring one does
            // too. With the pointer off the widget there is nothing honest to
            // draw, so nothing is.
            let Some(at) = hover.map(|h| h.at) else {
                return;
            };
            if let Some(authored) = st.linear.placing_preview(at) {
                // Placing: draw the dimension itself, exactly as it will land.
                draw_label(painter, doc, st.group, &authored, page, map, color);
                pick::dimension_preview_segments(&authored)
            } else {
                // Measuring: the constrained A→pointer segment.
                st.linear.preview_segment(at).into_iter().collect()
            }
        }
        // The fitted circle, from the value the commit would author.
        //
        // `author()` is `None` for a degenerate set — one arc, or three points
        // on a line — and that draws nothing, which is the correct picture: the
        // objects are outlined so the operator can see what is in the set, and
        // no circle appears because there is no circle. Drawing a guess would
        // promise a dimension `circular::commit` is about to refuse.
        MeasureKind::Circular => st
            .circular
            .author()
            .map(|kind| pick::dimension_preview_segments(&kind))
            .unwrap_or_default(),
        // The perimeter, drawn through the SAME segment function a committed
        // one is drawn from - the standing rule in this module, and the whole
        // of what makes a preview a preview rather than an illustration.
        //
        // `preview` appends the pointer as a provisional last vertex, so the
        // rubber band runs from the last committed pick to the cursor and says
        // *"this click would add this segment"*. It is deliberately never drawn
        // closed: the operator has not closed it, and showing the closing
        // segment early would promise a shape one segment longer than the one
        // the next click commits.
        MeasureKind::Perimeter | MeasureKind::PathLength => {
            let Some(at) = hover.map(|h| h.at) else {
                return;
            };
            st.perimeter
                .preview(at)
                .map(|kind| pick::dimension_preview_segments(&kind))
                .unwrap_or_default()
        }
        // A two-line pick has nothing to preview against the pointer: what it
        // has picked is a *line already on the page*, which the page is already
        // drawing. Highlighting the picked line is the affordance it wants, and
        // that needs the hover query this call site does not yet run — see the
        // module header's note on what remains.
        MeasureKind::TwoLine => Vec::new(),
    };
    if segments.is_empty() {
        return;
    }

    for (a, b) in segments {
        let (Some(sa), Some(sb)) = (page_to_screen(a, page, map), page_to_screen(b, page, map))
        else {
            continue;
        };
        painter.line_segment([sa, sb], stroke);
    }
}

/// A dimension's lines, as the preview draws them.
fn draw_dimension(
    painter: &egui::Painter,
    stroke: egui::Stroke,
    kind: &DimensionKind,
    page: &pdfcer_core::page_tree::Page,
    map: &PageMapping,
) {
    for (a, b) in pick::dimension_preview_segments(kind) {
        if let (Some(sa), Some(sb)) = (page_to_screen(a, page, map), page_to_screen(b, page, map)) {
            painter.line_segment([sa, sb], stroke);
        }
    }
}

/// A dimension's value text, where and how large the baker would put it.
///
/// The text and its box come from `author_dimension` under the group's style,
/// so the preview says the value the commit writes. The box's corners run
/// baseline-left (at the descender), baseline-right, cap-right, cap-left; the
/// font size is the box height over 1.3, the descender-to-cap span the baker
/// sizes it from.
fn draw_label(
    painter: &egui::Painter,
    doc: &OpenDoc,
    group: pdfcer_core::dimension::GroupId,
    kind: &DimensionKind,
    page: &pdfcer_core::page_tree::Page,
    map: &PageMapping,
    color: egui::Color32,
) {
    let model = doc.session.dimension_model();
    let Some(group) = model.group(group) else {
        return;
    };
    let style = resolve_style(group, &StyleOverrides::default());
    let authored = author_dimension(kind, style);
    let [q0, q1, _, q3] = authored.label_quad;
    let (Some(s0), Some(s1), Some(s3)) = (
        page_to_screen(q0, page, map),
        page_to_screen(q1, page, map),
        page_to_screen(q3, page, map),
    ) else {
        return;
    };
    let size = s0.distance(s3) / 1.3;
    if !size.is_finite() || size < 1.0 {
        return;
    }
    let galley = painter.layout_no_wrap(authored.label, egui::FontId::proportional(size), color);
    let angle = (s1.y - s0.y).atan2(s1.x - s0.x);
    painter.add(egui::epaint::TextShape::new(s3, galley, color).with_angle(angle));
}

/// How large the snap marker is drawn, in **points**.
pub(in crate::canvas) const SNAP_MARKER_PT: f32 = 6.0;

/// How much wider than the snap glyph the **committed-pick ring** is drawn.
const PICKED_RING_SCALE: f32 = 1.7;

/// **PDF user space → screen**, both hops, in one place.
pub(in crate::canvas) fn page_to_screen(
    p: Point,
    page: &pdfcer_core::page_tree::Page,
    map: &PageMapping,
) -> Option<Pos2> {
    let canvas = viewer::pdf_space_to_canvas(Pos2::new(p.x as f32, p.y as f32), page)?;
    Some(map.to_screen(canvas))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The preview is drawn in SCREEN space, through the frame's map.**
    #[test]
    fn the_preview_projects_page_space_all_the_way_to_the_screen() {
        let origin = egui::Pos2::new(37.0, 11.0);
        let zoom = 2.0_f32;
        let extent = (200.0_f32, 300.0_f32);
        let map = PageMapping::new(
            egui::Rect::from_min_size(origin, egui::vec2(extent.0 * zoom, extent.1 * zoom)),
            extent,
            zoom,
        );
        // Canvas (50, 60) is 50 across and 60 down from the page's top-left.
        let canvas = egui::Pos2::new(50.0, 60.0);
        let screen = map.to_screen(canvas);
        assert!(
            (screen.x - (origin.x + 100.0)).abs() < 1e-3
                && (screen.y - (origin.y + 120.0)).abs() < 1e-3,
            "the second hop must apply the page origin AND the zoom, got {screen:?}"
        );
        // …and the canvas coordinate on its own is neither, which is what made
        // the defect invisible at zoom 1 with the page at the window's origin.
        assert!(
            (canvas.x - screen.x).abs() > 1.0,
            "a canvas coordinate handed straight to the painter is the defect"
        );
    }
}
