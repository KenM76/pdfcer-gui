//! # `panels::align::handles` — on-canvas alignment handles.
//!
//! With *On-canvas alignment* ticked and two or more objects selected at the
//! Object rung, nine handles sit inside the selection box — one per corner,
//! one per edge, one in the middle — as in Inkscape's align mode. A click
//! aligns every selected object to that side of the selection area (a corner
//! is both sides, the middle is both centres); Shift+click aligns them just
//! outside it. Each press goes through [`super::run`], so it is one undo step
//! and is traced as an ordinary Align press.
//!
//! The canvas records the selection box it drew this pass ([`publish_box`]);
//! [`show`] runs after the canvas and draws in the foreground layer, so a
//! press on a handle never reaches the canvas beneath. The handles sit inside
//! the box so they stay clear of the eight resize grips on its corners and
//! edges and of the rotate handle above it.

use egui::{Context, Id, Pos2, Rect, Sense, Stroke, Vec2};
use pdfcer_gui_base::alignlayout::{Axis, Edge, RelativeTo};

use super::{AlignUi, Op};
use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::panels::align as t;

/// The handles' regions, row by row from the top-left, in screen reading
/// order.
pub const HANDLE_REGIONS: [&str; 9] = [
    "align.handle.0",
    "align.handle.1",
    "align.handle.2",
    "align.handle.3",
    "align.handle.4",
    "align.handle.5",
    "align.handle.6",
    "align.handle.7",
    "align.handle.8", // ui-text-exempt: diagnostic region names
];

/// A handle's side, in screen pixels.
const SIZE_PX: f32 = 12.0;
/// A handle's inset from the box edge to its centre, in screen pixels.
const INSET_PX: f32 = 12.0;

fn key() -> Id {
    Id::new("align.handles.box") // ui-text-exempt: memory key
}

/// Record the Object-rung selection box the canvas drew this pass, in screen
/// points.
pub fn publish_box(ctx: &Context, rect: Rect) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| d.insert_temp(key(), (rect, pass)));
}

/// The three stops along one axis: min, centre, max. A box too small to hold
/// three handles apart spreads them from its centre instead.
fn stops(min: f32, max: f32) -> [f32; 3] {
    let c = f32::midpoint(min, max);
    let half = ((max - min) / 2.0 - INSET_PX).max(SIZE_PX + 2.0);
    [c - half, c, c + half]
}

/// The ops handle `(col, row)` presses: 0 = min side, 1 = centre, 2 = max
/// side; `outside` is Shift held.
fn ops(col: usize, row: usize, outside: bool) -> Vec<Op> {
    let edge = |i: usize| match (i, outside) {
        (0, false) => Edge::Min,
        (0, true) => Edge::MaxToMin,
        (2, false) => Edge::Max,
        (2, true) => Edge::MinToMax,
        _ => Edge::Centre,
    };
    let mut out = Vec::with_capacity(2);
    if col != 1 || row == 1 {
        out.push(Op::Align(Axis::X, edge(col)));
    }
    if row != 1 || col == 1 {
        out.push(Op::Align(Axis::Y, edge(row)));
    }
    out
}

/// Draw the handles when they apply, and press the one clicked.
pub fn show(ctx: &Context, doc: &OpenDoc, settings: AlignUi, actions: &mut Vec<Action>) {
    if !settings.on_canvas {
        return;
    }
    let Some((rect, pass)) = ctx.data(|d| d.get_temp::<(Rect, u64)>(key())) else {
        return;
    };
    if pass != ctx.cumulative_pass_nr() || super::measure(doc).is_none_or(|m| m.objects.len() < 2) {
        return;
    }
    let xs = stops(rect.min.x, rect.max.x);
    let ys = stops(rect.min.y, rect.max.y);
    let shift = ctx.input(|i| i.modifiers.shift);
    egui::Area::new(Id::new("align.handles")) // ui-text-exempt: widget id
        .order(egui::Order::Foreground)
        .fixed_pos(Pos2::ZERO)
        .interactable(true)
        .show(ctx, |ui| {
            let visuals = ui.visuals().clone();
            let ink = visuals.strong_text_color();
            for (row, &y) in ys.iter().enumerate() {
                for (col, &x) in xs.iter().enumerate() {
                    let index = row * 3 + col;
                    let centre = Pos2::new(x, y);
                    let r = Rect::from_center_size(centre, Vec2::splat(SIZE_PX));
                    let response = ui
                        .interact(r, Id::new(HANDLE_REGIONS[index]), Sense::click())
                        .on_hover_text(t::handle_tip());
                    let fill = if response.hovered() {
                        visuals.widgets.hovered.weak_bg_fill
                    } else {
                        visuals.window_fill
                    };
                    let painter = ui.painter();
                    painter.circle(centre, SIZE_PX / 2.0, fill, Stroke::new(1.0, ink));
                    // A tick toward the side the handle aligns to.
                    let dir = Vec2::new(col as f32 - 1.0, row as f32 - 1.0);
                    if dir != Vec2::ZERO {
                        let tip = centre + dir.normalized() * (SIZE_PX / 2.0 - 1.5);
                        painter.line_segment([centre, tip], Stroke::new(1.5, ink));
                    } else {
                        painter.circle_filled(centre, 1.5, ink);
                    }
                    crate::diag::ui_rect(HANDLE_REGIONS[index], r);
                    if response.clicked() {
                        let ops = ops(col, row, shift);
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed.
                            format!("align-handle index={index} outside={shift} ops={ops:?}")
                        });
                        let at = AlignUi {
                            multi: RelativeTo::Selection,
                            as_group: false,
                            ..settings
                        };
                        super::run(doc, &ops, at, actions);
                    }
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_press_both_axes_and_edges_one() {
        assert_eq!(
            ops(0, 0, false),
            vec![Op::Align(Axis::X, Edge::Min), Op::Align(Axis::Y, Edge::Min)]
        );
        assert_eq!(ops(1, 0, false), vec![Op::Align(Axis::Y, Edge::Min)]);
        assert_eq!(ops(2, 1, false), vec![Op::Align(Axis::X, Edge::Max)]);
        assert_eq!(
            ops(1, 1, false),
            vec![
                Op::Align(Axis::X, Edge::Centre),
                Op::Align(Axis::Y, Edge::Centre)
            ]
        );
    }

    #[test]
    fn shift_aligns_outside() {
        assert_eq!(ops(0, 1, true), vec![Op::Align(Axis::X, Edge::MaxToMin)]);
        assert_eq!(ops(1, 2, true), vec![Op::Align(Axis::Y, Edge::MinToMax)]);
    }

    #[test]
    fn a_small_box_keeps_its_handles_apart() {
        let [a, b, c] = stops(0.0, 10.0);
        assert!(b - a >= SIZE_PX && c - b >= SIZE_PX);
    }
}
