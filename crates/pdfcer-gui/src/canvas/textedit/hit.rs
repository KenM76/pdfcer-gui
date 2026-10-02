//! # `canvas::textedit::hit` — where the pointer is inside the editor box
//!
//! ## What this is
//!
//! One fact, published once a frame by [`super::paint`] and read by everything
//! that needs to know whether a pointer event belongs to the draft: **the
//! editor box's rectangle, and how a point inside it becomes a caret slot.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/hit.md`.

use std::sync::Arc;

use egui::{Galley, Pos2, Rect};

/// Where the layout is parked between the frame that draws it and the frame
/// that hit-tests it.
const KEY: &str = "textedit-hit-layout"; // ui-text-exempt: a memory key, never displayed.

/// **How the drawn text maps screen points to character slots.**
#[derive(Clone)]
pub enum Caret {
    /// The shell-font box: a galley, and where its origin sits on screen.
    /// An `Arc` because egui hands galleys out that way; cloning is a refcount.
    Galley { origin: Pos2, galley: Arc<Galley> },
    /// The in-font draft ([`super::shaped`]): one screen point per caret slot,
    /// `chars + 1` of them, in character order.
    Stops(Vec<Pos2>),
}

/// **The editor box as it was last drawn.**
#[derive(Clone)]
pub struct Layout {
    /// The box, in **screen** coordinates — what a raw pointer position is in.
    pub body: Rect,
    /// The same box in **canvas** coordinates, for the click ladder, which
    /// works in page space and never sees a screen point.
    ///
    /// Both, rather than one and a conversion at the call site: the two
    /// callers live in different coordinate spaces and neither has the other's
    /// map to hand at the moment it asks. Publishing both puts the one
    /// conversion in the one place that owns the map.
    pub body_canvas: Rect,
    /// How a screen point inside the box becomes a character index.
    pub caret: Caret,
}

impl Layout {
    /// **Which character index is under `screen`**, as a character offset into
    /// the draft's text.
    #[must_use]
    pub fn index_at(&self, screen: Pos2) -> usize {
        match &self.caret {
            Caret::Galley { origin, galley } => {
                galley.cursor_from_pos(screen - *origin).index.into()
            }
            Caret::Stops(stops) => stops
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.distance_sq(screen).total_cmp(&b.1.distance_sq(screen)))
                .map_or(0, |(i, _)| i),
        }
    }

    /// **Where a sweep reaching `screen` ends**: [`Self::index_at`], except
    /// that a point past the box on the side the next line would be takes the
    /// end of the text, and one past the side the previous line would be its
    /// start, as a word processor's sweep off a last or first line does.
    /// "Next line" is across the reading direction, so rotated text sweeps the
    /// same way.
    #[must_use]
    pub fn swept_index_at(&self, screen: Pos2, len: usize) -> usize {
        let (first, along) = match &self.caret {
            Caret::Galley { origin, .. } => (*origin, egui::vec2(1.0, 0.0)),
            Caret::Stops(stops) => match (stops.first(), stops.last()) {
                (Some(a), Some(b)) if a.distance(*b) > f32::EPSILON => (*a, (*b - *a).normalized()),
                _ => return self.index_at(screen),
            },
        };
        let across = egui::vec2(-along.y, along.x);
        let depth = |p: Pos2| (p - first).dot(across);
        let corners = [
            self.body.left_top(),
            self.body.right_top(),
            self.body.left_bottom(),
            self.body.right_bottom(),
        ];
        let (lo, hi) = corners
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), c| {
                (lo.min(depth(*c)), hi.max(depth(*c)))
            });
        match depth(screen) {
            d if d > hi => len,
            d if d < lo => 0,
            _ => self.index_at(screen),
        }
    }

    /// The screen x of the caret slot before character `i`, clamped to the
    /// last slot.
    #[must_use]
    pub fn x_at(&self, i: usize) -> f32 {
        match &self.caret {
            Caret::Galley { origin, galley } => {
                origin.x + galley.pos_from_cursor(egui::text::CCursor::new(i)).min.x
            }
            Caret::Stops(stops) => stops
                .get(i)
                .or_else(|| stops.last())
                .map_or(self.body.min.x, |p| p.x),
        }
    }
}

/// Publish this frame's editor box. Called by [`super::paint`] only.
pub fn publish(ctx: &egui::Context, layout: Layout) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(KEY), layout));
}

/// The editor box as of the last frame that drew one, if any.
#[must_use]
pub fn read(ctx: &egui::Context) -> Option<Layout> {
    ctx.data(|d| d.get_temp::<Layout>(egui::Id::new(KEY)))
}

/// **Is `screen` inside the editor box a live draft is being composed in?**
///
/// `false` when there is no draft, which is what makes this safe to ask from
/// the click ladder on every press.
#[must_use]
pub fn owns_screen(ctx: &egui::Context, screen: Pos2) -> bool {
    super::read(ctx).is_some() && read(ctx).is_some_and(|l| l.body.contains(screen))
}

/// [`owns_screen`] for a caller that only has a canvas-space point.
#[must_use]
pub fn owns_canvas(ctx: &egui::Context, canvas: Pos2) -> bool {
    super::read(ctx).is_some() && read(ctx).is_some_and(|l| l.body_canvas.contains(canvas))
}

#[cfg(test)]
mod tests {
    use super::{Caret, Layout};
    use egui::{Pos2, Rect, pos2};

    fn layout(stops: &[Pos2], body: Rect) -> Layout {
        Layout {
            body,
            body_canvas: body,
            caret: Caret::Stops(stops.to_vec()),
        }
    }

    #[test]
    fn a_sweep_off_the_line_reaches_its_end_or_start() {
        let flat = layout(
            &[pos2(0.0, 0.0), pos2(10.0, 0.0), pos2(20.0, 0.0)],
            Rect::from_min_max(pos2(0.0, -8.0), pos2(20.0, 2.0)),
        );
        assert_eq!(flat.swept_index_at(pos2(5.0, 20.0), 2), 2);
        assert_eq!(flat.swept_index_at(pos2(5.0, -20.0), 2), 0);
        assert_eq!(flat.swept_index_at(pos2(12.0, 0.0), 2), 1);
    }

    #[test]
    fn a_sweep_off_rotated_text_goes_across_its_reading_direction() {
        let upward = layout(
            &[pos2(0.0, 0.0), pos2(0.0, -10.0), pos2(0.0, -20.0)],
            Rect::from_min_max(pos2(-8.0, -20.0), pos2(2.0, 0.0)),
        );
        assert_eq!(upward.swept_index_at(pos2(20.0, -5.0), 2), 2);
        assert_eq!(upward.swept_index_at(pos2(-20.0, -5.0), 2), 0);
        assert_eq!(upward.swept_index_at(pos2(0.0, 30.0), 2), 0);
    }
}
