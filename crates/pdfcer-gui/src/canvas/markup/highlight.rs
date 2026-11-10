//! # `canvas::markup::highlight` — what the highlighter draws
//!
//! The Highlight tool draws in one of five shapes, chosen in Tool Options:
//! following **text** (falling back to a box over blank paper), a plain
//! **area** box that never follows text, or a washed **ellipse**, **freehand**
//! stroke or **polygon**. A shape is drawn by the gesture of its own markup
//! kind and authored as that kind with the pen [`Pen::washed`]: highlighter
//! colour, no outline, multiplied into the page. Holding Alt while dragging
//! in Text mode draws an area box.
//!
//! The choice is read in two places and nowhere else: [`drawn_as`] maps the
//! armed tool to the tool the canvas behaves as, and [`pen_for`] maps the pen.
//! The armed tool itself stays `Markup(Highlight)`, so the ribbon shows the
//! highlighter pressed whatever shape it draws.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/markup/highlight.md`.

use super::MarkupKind;
use super::pen::Pen;
use crate::canvas::tool::CanvasTool;

const SHAPE_MEMORY_KEY: &str = "pdfcer-highlight-shape"; // ui-text-exempt: internal memory id, never displayed

pub use pdfcer_gui_base::prefs::markuptools::HighlightShape as Shape;

/// The chosen shape.
#[must_use]
pub fn shape(ctx: &egui::Context) -> Shape {
    ctx.data(|d| d.get_temp::<Shape>(egui::Id::new(SHAPE_MEMORY_KEY)))
        .unwrap_or_default()
}

/// Choose a shape. A vertex run in progress is abandoned, since the polygon
/// it was drawing is no longer what the highlighter draws.
pub fn set_shape(ctx: &egui::Context, shape: Shape) {
    if self::shape(ctx) == shape {
        return;
    }
    super::vertex::abandon(ctx);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(SHAPE_MEMORY_KEY), shape));
    crate::diag::trace(|| format!("highlight-shape shape={shape:?}"));
}

/// Set the shape remembered from the preferences file, at launch. Unlike
/// [`set_shape`] it abandons nothing and traces nothing: no tool is drawing.
pub fn seed(ctx: &egui::Context, shape: Shape) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(SHAPE_MEMORY_KEY), shape));
}

/// The tool the canvas behaves as when `tool` is armed: the highlighter's
/// shape kind for `Markup(Highlight)`, `tool` itself otherwise.
#[must_use]
pub fn drawn_as(ctx: &egui::Context, tool: CanvasTool) -> CanvasTool {
    if tool == CanvasTool::Markup(MarkupKind::Highlight) {
        CanvasTool::Markup(shape(ctx).kind())
    } else {
        tool
    }
}

/// The pen the canvas draws with this frame: `pen` washed for a highlighter
/// shape, marked as an area box for Area mode or Alt held, `pen` otherwise.
#[must_use]
pub fn pen_for(ctx: &egui::Context, pen: Pen) -> Pen {
    if crate::canvas::tool::selected(ctx) != CanvasTool::Markup(MarkupKind::Highlight) {
        return pen;
    }
    match shape(ctx) {
        Shape::Text if ctx.input(|i| i.modifiers.alt) => pen.area(),
        Shape::Text => pen,
        Shape::Area => pen.area(),
        Shape::Ellipse | Shape::Freehand | Shape::Polygon => pen.washed(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlighter_draws_its_shape_and_other_tools_are_untouched() {
        let ctx = egui::Context::default();
        let hl = CanvasTool::Markup(MarkupKind::Highlight);
        assert_eq!(drawn_as(&ctx, hl), hl);
        set_shape(&ctx, Shape::Polygon);
        assert_eq!(drawn_as(&ctx, hl), CanvasTool::Markup(MarkupKind::Polygon));
        let rect = CanvasTool::Markup(MarkupKind::Rectangle);
        assert_eq!(drawn_as(&ctx, rect), rect);
        assert_eq!(drawn_as(&ctx, CanvasTool::Select), CanvasTool::Select);
    }

    #[test]
    fn the_pen_washes_only_with_the_highlighter_armed() {
        use super::super::pen::Wash;
        let ctx = egui::Context::default();
        set_shape(&ctx, Shape::Ellipse);
        assert_eq!(pen_for(&ctx, Pen::default()).wash, Wash::Off);
        crate::canvas::tool::select(&ctx, CanvasTool::Markup(MarkupKind::Highlight));
        let pen = pen_for(&ctx, Pen::default());
        assert_eq!(pen.wash, Wash::Shape);
        assert_eq!(pen.ink, Pen::default().highlighter);
        set_shape(&ctx, Shape::Area);
        assert_eq!(pen_for(&ctx, Pen::default()).wash, Wash::Area);
    }
}
