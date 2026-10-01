//! How a tab looks, shared by every tab row in the shell so they agree.
//!
//! A selected tab is the front of the surface beneath it: filled with that
//! surface's colour, rounded on top, outlined on top and sides, with an
//! `accent` rule along its top edge and no floor. The outline and the rule
//! are its two shape cues, so selection never rests on colour alone (R84).
//! An unselected tab has no frame; hovering lifts it with the same fill.
//!
//! A row of sub-pages inside a panel uses [`underline`] instead: plain text,
//! with an `accent` rule under the selected one.

use egui::{Color32, CornerRadius, Rect, Shape, Stroke, StrokeKind, pos2};

use crate::theme::{Palette, Theme};

/// Thickness of the accent rule on a selected tab, in points.
pub const RULE_PTS: f32 = 2.0;

/// The painted body of one tab in a tab bar.
///
/// `fill` is the colour of the surface the selected tab opens into. The
/// caller keeps the bar's baseline out from under a selected tab, so the
/// tab and the surface below read as one piece.
pub fn body(
    palette: &Palette,
    radius: u8,
    fill: Color32,
    rect: Rect,
    selected: bool,
    hovered: bool,
) -> Shape {
    let top = CornerRadius {
        nw: radius,
        ne: radius,
        sw: 0,
        se: 0,
    };
    if !selected {
        return if hovered {
            Shape::rect_filled(rect.shrink2(egui::vec2(0.0, 2.0)), top, fill)
        } else {
            Shape::Noop
        };
    }
    let r = f32::from(radius);
    let rule = Rect::from_min_max(
        pos2(rect.left() + r, rect.top()),
        pos2(rect.right() - r, rect.top() + RULE_PTS),
    );
    // Covers the stroke's bottom edge: the tab has no floor.
    let floor = Rect::from_min_max(
        pos2(rect.left() + 1.0, rect.bottom() - 1.5),
        pos2(rect.right() - 1.0, rect.bottom()),
    );
    Shape::Vec(vec![
        Shape::rect_filled(rect, top, fill),
        Shape::rect_stroke(
            rect,
            top,
            Stroke::new(1.0, palette.outline),
            StrokeKind::Inside,
        ),
        Shape::rect_filled(rule, 0.0, palette.accent),
        Shape::rect_filled(floor, 0.0, fill),
    ])
}

/// The label colour for a tab: `text` when selected, `text_muted` otherwise.
#[must_use]
pub fn label_colour(palette: &Palette, selected: bool) -> Color32 {
    if selected {
        palette.text
    } else {
        palette.text_muted
    }
}

/// One tab in a row of sub-pages inside a panel or dialog.
///
/// Frameless text; the selected one carries an `accent` rule beneath it, a
/// hovered one an `outline` rule.
pub fn underline(ui: &mut egui::Ui, selected: bool, label: &str) -> egui::Response {
    let palette = Theme::of(ui.ctx()).palette;
    let text = egui::RichText::new(label).color(label_colour(&palette, selected));
    let response = ui.add(
        egui::Button::new(text)
            .frame(false)
            .min_size(egui::vec2(0.0, ui.spacing().interact_size.y))
            .selected(selected),
    );
    let colour = if selected {
        Some(palette.accent)
    } else if response.hovered() {
        Some(palette.outline)
    } else {
        None
    };
    if let Some(colour) = colour {
        let rect = response.rect;
        let rule = Rect::from_min_max(
            pos2(rect.left(), rect.bottom() - RULE_PTS),
            rect.right_bottom(),
        );
        ui.painter().rect_filled(rule, 0.0, colour);
    }
    response
}
