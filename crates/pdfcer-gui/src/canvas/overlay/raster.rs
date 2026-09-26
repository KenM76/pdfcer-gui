//! The **raster ghost**: a translucent copy of the selection's own pixels,
//! lifted out of the page texture and painted at the displaced position while
//! a move is in flight.
//!
//! The outline ghost in [`super`] says WHERE a move lands; this says WHAT
//! lands there. It is a pre-commit affordance under Rule 4 - it draws the
//! cursor, never the document - and it disappears the instant the gesture
//! ends, whether the move was applied or abandoned.
//!
//! Contract: the caller owns the decision to paint. This module is handed the
//! page strip, the page it is painting, the projection, the selection and the
//! canvas-space displacement, and it answers by painting; it reads no
//! application state and mutates nothing.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/overlay/raster.md`.

use egui::{Color32, Painter, Rect};

use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::SelectionState;

/// How far you can see THROUGH the travelling copy, out of 255.
const RASTER_GHOST_ALPHA: u8 = 190;

/// Paint the **raster ghost**: a translucent copy of the selection's own
/// pixels, travelling with the pointer.
pub(in crate::canvas) fn draw_raster_ghost(
    painter: &Painter,
    pages: &[crate::canvas::strip::PageView],
    page_index: usize,
    mapping: &PageMapping,
    selection: &SelectionState,
    delta: egui::Vec2,
    // The same in-flight preview [`super::draw_move_ghost`] takes. See
    // [`raster_ghost_is_owed`] for why this one is not `outline`-sensitive.
    already_travelling: Option<&crate::canvas::shapes::ShapePreview>,
) {
    use crate::canvas::trace::RasterGhostReason as Why;

    if !raster_ghost_is_owed(already_travelling) {
        crate::canvas::trace::raster_ghost(0, 0, Why::GeometryTravels);
        return;
    }
    let source = pages
        .iter()
        .find(|view| view.page == page_index)
        .and_then(|view| view.raster.map(|texture| (texture, view.paint_rect)));
    let Some((texture, source)) = source else {
        crate::canvas::trace::raster_ghost(0, 0, Why::NoRaster);
        return;
    };
    if source.width() <= 0.0 || source.height() <= 0.0 {
        crate::canvas::trace::raster_ghost(0, 0, Why::NoRaster);
        return;
    }
    // NOT A THEME COLOUR: an alpha multiplier applied to DOCUMENT pixels, not
    // a choice of colour. White is the identity for a tint, so the hue that
    // arrives is the operator's own drawing and the only thing chosen here is
    // how far you can see through it. A palette role would recolour his page.
    let tint = Color32::from_white_alpha(RASTER_GHOST_ALPHA);
    let mut drawn = 0_usize;
    let mut clipped = 0_usize;
    for (_, page_rect) in selection.outlines() {
        let from = mapping.rect_to_screen(*page_rect);
        // The displacement is measured between the two PROJECTED rects rather
        // than by projecting the canvas-space delta on its own: at the deep
        // zoom tier the projection is not a plain scale, and a delta scaled
        // here would disagree with the outline ghost drawn on top of it.
        let shift = mapping.rect_to_screen(page_rect.translate(delta)).min - from.min;
        let Some(blit) = blit_of(from, source, shift) else {
            clipped += 1;
            continue;
        };
        if blit.cropped {
            clipped += 1;
        }
        painter.image(texture, blit.to, blit.uv, tint);
        drawn += 1;
    }
    let why = if drawn > 0 {
        Why::Drew
    } else if selection.outlines().is_empty() {
        Why::NoSelection
    } else {
        Why::OffRaster
    };
    crate::canvas::trace::raster_ghost(drawn, clipped, why);
}

/// One chunk's blit: where to put it, and which part of the page texture to
/// take.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::canvas) struct Blit {
    /// Where the piece lands, in screen points.
    pub to: Rect,
    /// The sub-rectangle of the page texture to sample, in 0..1 texture space.
    pub uv: Rect,
    /// Whether the source had to be cropped to the painted raster.
    pub cropped: bool,
}

/// Work out one chunk's blit: the piece of the page texture that stands under
/// `from`, moved by `shift`.
pub(in crate::canvas) fn blit_of(from: Rect, source: Rect, shift: egui::Vec2) -> Option<Blit> {
    if source.width() <= 0.0 || source.height() <= 0.0 {
        return None;
    }
    let visible = from.intersect(source);
    if visible.width() <= 0.0 || visible.height() <= 0.0 {
        return None;
    }
    let uv = Rect::from_min_max(
        egui::pos2(
            (visible.min.x - source.min.x) / source.width(),
            (visible.min.y - source.min.y) / source.height(),
        ),
        egui::pos2(
            (visible.max.x - source.min.x) / source.width(),
            (visible.max.y - source.min.y) / source.height(),
        ),
    );
    Some(Blit {
        to: visible.translate(shift),
        uv,
        cropped: visible != from,
    })
}

/// **Whether the travelling copy of the picture is owed** —
/// `OPERATOR_REQUESTS.md` O215 ask 5.
pub(in crate::canvas) fn raster_ghost_is_owed(
    already_travelling: Option<&crate::canvas::shapes::ShapePreview>,
) -> bool {
    !already_travelling.is_some_and(|preview| !preview.is_empty())
}
