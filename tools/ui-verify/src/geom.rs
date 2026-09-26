//! Points and rectangles.
//!
//! Three rectangle types, and the distinction between them is load-bearing:
//!
//! * [`LRect`] — **logical points**, the unit egui and the diagnostic trace
//!   speak in. A trace line's `rect=[[0.0 0.0] - [1600.0 1000.0]]` is one of
//!   these. Independent of display scaling.
//! * [`PixRect`] — **device pixels**, the unit a screenshot is measured in. A
//!   150% display makes these 1.5× the logical values, which is precisely the
//!   conversion that must not be done by hand at a call site.
//! * [`FracRect`] — **fractions of a window**, `0.0..=1.0`. This is the only
//!   rectangle a check is allowed to *write down*, and the reason is the same
//!   as the reason checks may not write down screen coordinates: a literal
//!   pixel rectangle stops meaning what it meant the moment the window is
//!   resized, the panel layout changes, or the harness runs on a machine with
//!   a different DPI — and it stops *silently*, by pointing at the wrong thing
//!   rather than at nothing.
//!
//! Keeping them as separate types rather than as three uses of `(f32, f32,
//! f32, f32)` costs a few conversions and buys the guarantee that a logical
//! rect can never be handed to the screenshot cropper by accident. That
//! mistake is invisible at 100% scaling — which is the developer's machine —
//! and wrong everywhere else.
//!
//! Design and rationale: `docs/modules/ui-verify/geom.md`.

/// A point in logical (egui) points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pt {
    /// Rightward.
    pub x: f32,
    /// Downward. Note the sign: egui's y grows *down*, PDF user space's y
    /// grows *up*, and [`crate::coords`] is where that flip happens exactly
    /// once.
    pub y: f32,
}

impl Pt {
    /// A point.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A rectangle in logical (egui) points, min/max form — the shape egui's
/// `Rect` prints in its `Debug`, which is what the trace carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LRect {
    /// Top-left.
    pub min: Pt,
    /// Bottom-right.
    pub max: Pt,
}

impl LRect {
    /// From corners.
    #[must_use]
    pub const fn new(min: Pt, max: Pt) -> Self {
        Self { min, max }
    }

    /// Does this rectangle wholly contain `other`?
    #[must_use]
    pub fn contains_rect(&self, other: Self) -> bool {
        other.min.x >= self.min.x
            && other.min.y >= self.min.y
            && other.max.x <= self.max.x
            && other.max.y <= self.max.y
    }

    /// Width in logical points.
    #[must_use]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Height in logical points.
    #[must_use]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Is this rectangle big enough to be a real laid-out area?
    #[must_use]
    pub fn is_substantial(&self) -> bool {
        self.width() > 1.0 && self.height() > 1.0
    }
}

/// A rectangle in device pixels, origin-plus-size form — the shape a
/// screenshot crop takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixRect {
    /// Left edge, pixels.
    pub x: u32,
    /// Top edge, pixels.
    pub y: u32,
    /// Width, pixels.
    pub w: u32,
    /// Height, pixels.
    pub h: u32,
}

impl PixRect {
    /// A pixel rectangle.
    #[must_use]
    pub const fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    /// Pixel count. Zero for a degenerate rect, which the oracles treat as
    /// "nothing was sampled" rather than dividing by it.
    #[must_use]
    pub fn area(&self) -> u32 {
        self.w.saturating_mul(self.h)
    }
}

/// A rectangle expressed as fractions of some containing surface, `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FracRect {
    /// Left edge as a fraction of width.
    pub x0: f32,
    /// Top edge as a fraction of height.
    pub y0: f32,
    /// Right edge as a fraction of width.
    pub x1: f32,
    /// Bottom edge as a fraction of height.
    pub y1: f32,
}

impl FracRect {
    /// A fractional rectangle. Values outside `0.0..=1.0` are permitted here
    /// and clamped at [`Self::resolve`] time, so a slightly-over-the-edge
    /// region clips instead of panicking.
    #[must_use]
    pub const fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self { x0, y0, x1, y1 }
    }

    /// Resolve against a surface of `width` × `height` device pixels.
    #[must_use]
    pub fn resolve(&self, width: u32, height: u32) -> PixRect {
        if width == 0 || height == 0 {
            return PixRect::new(0, 0, 0, 0);
        }
        let fw = width as f32;
        let fh = height as f32;
        let x0 = (self.x0.clamp(0.0, 1.0) * fw).floor() as u32;
        let y0 = (self.y0.clamp(0.0, 1.0) * fh).floor() as u32;
        let x1 = (self.x1.clamp(0.0, 1.0) * fw).ceil() as u32;
        let y1 = (self.y1.clamp(0.0, 1.0) * fh).ceil() as u32;
        let x0 = x0.min(width - 1);
        let y0 = y0.min(height - 1);
        let x1 = x1.clamp(x0 + 1, width);
        let y1 = y1.clamp(y0 + 1, height);
        PixRect::new(x0, y0, x1 - x0, y1 - y0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frac_rect_resolves_to_pixels() {
        let r = FracRect::new(0.25, 0.5, 0.75, 1.0).resolve(400, 200);
        assert_eq!(r, PixRect::new(100, 100, 200, 100));
    }

    #[test]
    fn frac_rect_never_resolves_to_an_empty_region() {
        // A region this thin would sample zero pixels, and a zero-pixel sample
        // reports contrast 1.0 — indistinguishable from invisible text. The
        // clamp turns that false FAIL into a one-pixel sample instead.
        let r = FracRect::new(0.5, 0.5, 0.5, 0.5).resolve(100, 100);
        assert!(r.w >= 1 && r.h >= 1, "resolved to an empty region: {r:?}");
    }

    #[test]
    fn frac_rect_clamps_outside_the_surface() {
        let r = FracRect::new(-0.5, -0.5, 2.0, 2.0).resolve(64, 32);
        assert_eq!(r, PixRect::new(0, 0, 64, 32));
    }

    #[test]
    fn a_degenerate_logical_rect_is_not_substantial() {
        let r = LRect::new(Pt::new(0.0, 0.0), Pt::new(0.0, 0.0));
        assert!(!r.is_substantial());
    }
}
