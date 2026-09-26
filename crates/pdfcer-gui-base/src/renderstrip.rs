//! # `renderstrip` — several pages at once, and what an undrawn one says
//!
//! Phase 4's continuous modes put more than one page on screen. Everything
//! about rasterization up to this point assumed exactly one — one texture, one
//! [`RenderKey`], one single-slot worker — and this module is the whole of what
//! changes, kept in one place so the single-page path is provably untouched.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/renderstrip.md`.

use egui::{Color32, FontId, Painter, Rect, Stroke, Visuals};

pub use crate::stripcache::{MAX_CACHED_TEXELS, PageRaster, StripRasters};

/// Why a page has no picture on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageState {
    /// A render for this page is running now.
    Drawing,
    /// This page is visible and has not been started yet — the renderer is
    /// working through the strip and has not reached it.
    Waiting,
    /// **This page's whole-sheet raster is larger than the renderer can
    /// allocate at this zoom, and a strip page has no region tier to fall back
    /// on.** O186, 2026-09-12.
    ///
    /// Nothing was ordered and nothing failed: `render::settle::fill_strip`
    /// declined to place an order it knew could not be filled. See
    /// `render::strategy::whole_page_raster_fits` for the whole report,
    /// including the operator's `50411508x32619210` and why the page he was
    /// *on* was never the one that could not be drawn.
    ///
    /// Distinct from [`Self::Refused`] because the distinction is the operator's
    /// next move: a refusal is a fault to report, this is a zoom to undo.
    BeyondRaster,
    /// This page will not draw, and this is the renderer's own reason.
    Refused(String),
}

/// **Draw a page that has no raster — honestly.**
pub fn draw_page_state(
    painter: &Painter,
    visuals: &Visuals,
    rect: Rect,
    visible: Rect,
    page_number: usize,
    state: &PageState,
) {
    painter.rect_filled(rect, 0.0, undrawn_fill(visuals));
    // The boundary is a real fact about the document, so it is drawn at full
    // strength rather than as a hint.
    painter.rect_stroke(
        rect,
        0.0,
        boundary_stroke(visuals),
        egui::StrokeKind::Inside,
    );

    let (message, colour) = match state {
        PageState::Drawing => (
            crate::text::canvas_page_drawing(page_number),
            visuals.text_color(),
        ),
        PageState::Waiting => (
            crate::text::canvas_page_waiting(page_number),
            visuals.text_color(),
        ),
        // The ORDINARY text colour, not the error colour, and that is the
        // whole point of the state existing. Nothing failed — the operator has
        // simply zoomed in past the point where a neighbour sheet can be
        // rastered whole, and his own sheet is drawing perfectly through the
        // region tier. Painting this in `error_fg_color` would re-introduce
        // exactly the alarm O186 was reported as.
        PageState::BeyondRaster => (
            crate::text::canvas_page_beyond_raster(page_number),
            visuals.text_color(),
        ),
        // A refusal is a different kind of statement and gets the theme's
        // error colour — the same distinction `canvas_render_failed` already
        // draws for the single-page case.
        PageState::Refused(detail) => (
            crate::text::canvas_page_refused(page_number, detail),
            visuals.error_fg_color,
        ),
    };

    let font = FontId::proportional(14.0);
    let galley = painter.layout(
        message,
        font,
        colour,
        // Wrap to the page's width, less a margin, so a long refusal reason
        // becomes several lines rather than one line off both edges.
        (rect.width() - TEXT_MARGIN * 2.0).max(1.0),
    );
    // Centred in the part of the page that is ON SCREEN, not in the page.
    //
    // **Found by screenshotting a driven scroll**, not by a test. A continuous
    // strip almost always has a page whose top few centimetres are showing and
    // whose middle is far below the viewport — and a sentence centred in the
    // page is then centred off screen, so the page draws as an empty outlined
    // rectangle saying nothing at all. That is exactly the blank-paper reading
    // this whole function exists to prevent, arriving through the placement
    // rather than through the fill.
    //
    // The intersection is the honest region: it is where the page and the
    // viewport agree, so a label centred in it is on screen whenever any part
    // of the page is.
    let on_screen = rect.intersect(visible);
    if galley.size().x + TEXT_MARGIN * 2.0 <= on_screen.width()
        && galley.size().y + TEXT_MARGIN * 2.0 <= on_screen.height()
    {
        painter.galley(
            on_screen.center() - galley.size() / 2.0,
            galley,
            // Already coloured by `layout`; this is the fallback for any run
            // the galley did not colour, which for a plain string is none.
            colour,
        );
    }
}

/// The gap between the page's edge and its state sentence, in points.
const TEXT_MARGIN: f32 = 8.0;

/// **The fill an undrawn page is painted with, and why it is not
/// `faint_bg_color`.**
#[must_use]
pub fn undrawn_fill(visuals: &Visuals) -> Color32 {
    visuals.widgets.inactive.bg_fill
}

/// The stroke an undrawn page's boundary is drawn with, exposed so a test can
/// assert it is theme-derived rather than a literal.
#[must_use]
pub fn boundary_stroke(visuals: &Visuals) -> Stroke {
    visuals.widgets.noninteractive.bg_stroke
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three page states are distinguishable, because the operator's
    /// response to each differs.
    #[test]
    fn a_refusal_is_a_different_state_from_waiting() {
        assert_ne!(PageState::Drawing, PageState::Waiting);
        assert_ne!(
            PageState::Waiting,
            PageState::Refused("no".to_owned()),
            "'wait' and 'this will never draw' must not be one state"
        );
    }
}
