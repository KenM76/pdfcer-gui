//! # `panels::properties::toolhighlight` — the highlighter's Highlight block
//!
//! While the highlighter is armed, Tool Options offers what one drag draws:
//! Text, Area, Ellipse, Freehand or Polygon
//! (`canvas::markup::highlight::Shape`), with the shape's hint under it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/toolhighlight.md`.

use egui::Ui;

use crate::canvas::markup::highlight::{self, Shape};
use crate::text::highlight as t;

/// The block.
pub const HIGHLIGHT_BLOCK: &str = "properties.tool.highlight"; // ui-text-exempt: trace region name, never displayed
/// The prefix of each shape choice; the shape's index in `Shape::ALL` is appended.
pub const HIGHLIGHT_SHAPE_PREFIX: &str = "properties.tool.highlight.shape."; // ui-text-exempt: trace region name, never displayed

/// Draw the block.
pub(super) fn section(ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let chosen = highlight::shape(&ctx);
    ui.label(t::heading());
    let mut picked = chosen;
    ui.horizontal_wrapped(|ui| {
        ui.label(t::draws());
        for (i, ((shape, label), hint)) in Shape::ALL
            .into_iter()
            .zip(t::shapes())
            .zip(t::shape_hints())
            .enumerate()
        {
            let choice = ui
                .radio_value(&mut picked, shape, label)
                .on_hover_text(hint);
            crate::diag::ui_rect_visible(
                &format!("{HIGHLIGHT_SHAPE_PREFIX}{i}"),
                choice.rect,
                ui.clip_rect(),
            );
        }
    });
    if picked != chosen {
        highlight::set_shape(&ctx, picked);
    }
    let note = match picked {
        Shape::Text => Some(t::alt_hint()),
        Shape::Polygon => Some(t::polygon_hint()),
        Shape::Area | Shape::Ellipse | Shape::Freehand => None,
    };
    if let Some(note) = note {
        ui.label(egui::RichText::new(note).small().weak());
    }
    crate::diag::ui_rect_visible(HIGHLIGHT_BLOCK, ui.min_rect(), ui.clip_rect());
}
