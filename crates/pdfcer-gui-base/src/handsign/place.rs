//! # `handsign::place` — where a signature's ink sits in its box
//!
//! Contract: every rectangle here is y-down and in the box's own space (the
//! canvas, or a preview drawn to scale). [`fit_rect`] is the placement rule
//! every kind of signature starts from; [`allowed`] is the region a signature
//! may occupy, the box plus one more box height above it, which is the rule's
//! own rise limit; [`reshape`] is what a grip drag does, and [`clamp`] keeps
//! the result inside [`allowed`]. A [`Placement`] carries a chosen rectangle
//! relative to the box, so it survives the box being drawn at another scale.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/handsign/place.md`.

use egui::{Pos2, Rect, Vec2, pos2, vec2};

use super::{CENTRE_BELOW, FILL, INSET, RISE};
use crate::handles::Grip;

/// The smallest side a reshaped signature keeps, in the box's units.
pub const MIN_SIDE: f32 = 4.0;

/// Where ink of size `ink` (any unit; one side may be zero) lands in
/// `target`: scaled uniformly to 95 % of the box width or of twice its
/// height, whichever binds; left-aligned with a 3 % inset; centred vertically
/// when it fits in 90 % of the height, otherwise standing on the box's lower
/// edge and rising above it. `None` for ink with no extent or a degenerate
/// target.
#[must_use]
pub fn fit_rect(ink: Vec2, target: Rect) -> Option<Rect> {
    if !(target.width() > 0.0 && target.height() > 0.0) {
        return None;
    }
    let (w, h) = (target.width(), target.height());
    let by_width = (ink.x > 0.0).then(|| FILL * w / ink.x);
    let by_height = (ink.y > 0.0).then(|| FILL * RISE * h / ink.y);
    let scale = match (by_width, by_height) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => return None,
    };
    let placed = ink * scale;
    let top = if placed.y <= CENTRE_BELOW * h {
        target.min.y + (h - placed.y) / 2.0
    } else {
        target.max.y - (1.0 - FILL) * h - placed.y
    };
    Some(Rect::from_min_size(
        pos2(target.min.x + INSET * w, top),
        placed,
    ))
}

/// Where ink of size `ink` goes in `target`: the operator's `chosen`
/// placement kept inside [`allowed`], or [`fit_rect`]'s when there is none.
#[must_use]
pub fn ink_rect(ink: Vec2, target: Rect, chosen: Option<Placement>) -> Option<Rect> {
    match chosen {
        Some(placement) => Some(clamp(placement.in_box(target), target)),
        None => fit_rect(ink, target),
    }
}

/// The region a signature in `target` may occupy: the box, and one more box
/// height above it.
#[must_use]
pub fn allowed(target: Rect) -> Rect {
    Rect::from_min_max(
        pos2(target.min.x, target.min.y - (RISE - 1.0) * target.height()),
        target.max,
    )
}

/// `rect` kept inside [`allowed`]`(target)`: shrunk about its centre, aspect
/// kept, when it is larger than the region, then moved the least distance
/// that brings it inside.
#[must_use]
pub fn clamp(rect: Rect, target: Rect) -> Rect {
    let region = allowed(target);
    let shrink = (region.width() / rect.width().max(f32::EPSILON))
        .min(region.height() / rect.height().max(f32::EPSILON))
        .min(1.0);
    let size = rect.size() * shrink;
    let mut min = rect.center() - size / 2.0;
    min.x = min.x.clamp(region.min.x, region.max.x - size.x);
    min.y = min.y.clamp(region.min.y, region.max.y - size.y);
    Rect::from_min_size(min, size)
}

/// `original` after a drag of `delta` on `grip`. The body moves it; a corner
/// keeps its proportions unless `free`, an edge stretches one side unless
/// `!stretchable`, when it scales both. The opposite corner or edge stays
/// put, and no side falls below [`MIN_SIDE`].
#[must_use]
pub fn reshape(grip: Grip, original: Rect, delta: Vec2, free: bool, stretchable: bool) -> Rect {
    if grip == Grip::Move || !grip.is_resize() {
        return original.translate(delta);
    }
    let anchor = grip.opposite().anchor(original);
    let moved = grip.anchor(original) + delta;
    let (w, h) = (
        original.width().max(f32::EPSILON),
        original.height().max(f32::EPSILON),
    );
    let corner = matches!(
        grip,
        Grip::NorthWest | Grip::NorthEast | Grip::SouthEast | Grip::SouthWest
    );
    let horizontal = matches!(grip, Grip::East | Grip::West);
    let reach = (moved - anchor).abs();
    let size = if corner && free && stretchable {
        vec2(reach.x, reach.y)
    } else if corner {
        let s = (reach.x / w).max(reach.y / h);
        vec2(w * s, h * s)
    } else if stretchable {
        if horizontal {
            vec2(reach.x, h)
        } else {
            vec2(w, reach.y)
        }
    } else {
        let s = if horizontal { reach.x / w } else { reach.y / h };
        vec2(w * s, h * s)
    };
    let size = size.max(Vec2::splat(MIN_SIDE));
    // Where the anchor sits in the rectangle, as fractions of its sides.
    let at = vec2(
        ((anchor.x - original.min.x) / w).clamp(0.0, 1.0),
        ((anchor.y - original.min.y) / h).clamp(0.0, 1.0),
    );
    Rect::from_min_size(anchor - vec2(at.x * size.x, at.y * size.y), size)
}

/// A rectangle chosen for a signature, relative to its box: `(0, 0)` is the
/// box's top-left, `(1, 1)` its bottom-right, and negative `y` the rise above.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement(pub Rect);

impl Placement {
    /// `rect` (in `target`'s space) as a placement in `target`.
    #[must_use]
    pub fn of(rect: Rect, target: Rect) -> Self {
        let (w, h) = (
            target.width().max(f32::EPSILON),
            target.height().max(f32::EPSILON),
        );
        let rel = |p: Pos2| pos2((p.x - target.min.x) / w, (p.y - target.min.y) / h);
        Self(Rect::from_min_max(rel(rect.min), rel(rect.max)))
    }

    /// The placement in `target`'s space.
    #[must_use]
    pub fn in_box(self, target: Rect) -> Rect {
        let abs = |p: Pos2| {
            pos2(
                target.min.x + p.x * target.width(),
                target.min.y + p.y * target.height(),
            )
        };
        Rect::from_min_max(abs(self.0.min), abs(self.0.max))
    }
}

#[cfg(test)]
mod tests;
