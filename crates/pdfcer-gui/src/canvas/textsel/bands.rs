//! # `canvas::textsel::bands` — from glyph cells to the boxes a selection shows
//!
//! The accumulation half of `textsel`'s §5 promise: **what is highlighted is
//! what is copied**. [`super::resolve`] walks a range once and, for each glyph
//! it covers, decides which band the glyph belongs to and grows that band by the
//! glyph's cell. This module is the two types that make "grow" mean the right
//! thing in each of the two frames the shell now has to work in.
//!
//! ## Why two frames exist at all
//!
//! `pdfcer-core` publishes a glyph's advance as a **length** and never publishes
//! its **direction** — see [`super::writing`], which recovers the direction from
//! the glyphs themselves, and the operator report that made it necessary. So a
//! page can carry lines running along x, which the engine groups correctly, and
//! lines running at 90° to it, which the engine splits at every letter.
//!
//! A selection over the first kind is a union of axis-aligned rectangles. A
//! selection over the second kind is a union taken **in the line's own axes**,
//! because a band across rotated text is a rotated band and its bounding
//! rectangle in page axes is not the same shape.
//!
//! [`Band`] names which of the two a glyph is in; [`Accum`] does the arithmetic
//! for that one. They are separate types because the question *"which band"* is
//! answered once per glyph from three maps, and the question *"how does this
//! band grow"* is answered from the band's own frame — and merging them would
//! mean the second question could be asked of a glyph whose band was never
//! settled.
//!
//! ## ★ The invariant this file is built around
//!
//! **Only equal bands are merged, and a band's identity fixes its variant.** A
//! [`Band::Engine`] glyph always produces an [`Accum::Page`] and a
//! [`Band::Rotated`] glyph always produces an [`Accum::Frame`], so
//! [`Accum::absorb`] is never called on a mismatched pair. That is why its
//! mismatch arm does nothing rather than panicking: the state is unreachable,
//! and a drag that killed the application would be a far worse outcome than a
//! selection one box short.
//!
//! ## What this module does NOT do
//!
//! It does not project into canvas space, does not read the page's `/Rotate`,
//! and does not know what a selection is for. [`Accum::quad`] hands back PDF
//! user space corners and [`super::resolve`] takes them through
//! `find::reveal::quad_to_canvas` — the same function Find projects its hits
//! with, which is what makes a selected word and a found word land in the same
//! place on a rotated page.

use pdfcer_core::annot_author::Quad;
use pdfcer_core::page_tree::Rect as PdfRect;

/// **Which band a glyph's cell joins**, and therefore which frame it is
/// measured in.
///
/// One enum rather than two parallel maps because the whole point is that a
/// glyph belongs to exactly one of these: a rotated line, an engine line, or
/// neither. Two maps would admit the state where a glyph is in both, and the
/// box drawn from it would be whichever [`super::resolve`] happened to read first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Band {
    /// A line [`writing`] recovered — measured in that line's own axes. §8.
    Rotated(usize),
    /// A line the engine derived — measured in the page's axes, exactly as
    /// before this distinction existed.
    Engine(usize),
    /// A glyph no line claimed, identified by itself so it merges with nothing.
    Loose(usize, usize),
}

impl Band {
    /// Whether other glyphs may join this band.
    ///
    /// False only for [`Band::Loose`], which is per-glyph by construction: a
    /// shared "unclaimed" key would merge every orphan on the page into one
    /// box spanning the sheet, which is the failure the previous `usize::MAX`
    /// sentinel guarded against by hand.
    pub(super) const fn merges(self) -> bool {
        !matches!(self, Self::Loose(..))
    }
}

/// A band's accumulated extent, in whichever frame its [`Band`] chose.
///
/// Two variants rather than one general parallelogram because the page-axis
/// case is the overwhelming majority and reducing it to a degenerate rotated
/// frame would put every ordinary selection through trigonometry to produce the
/// number it already had.
#[derive(Debug, Clone, Copy)]
pub(super) enum Accum {
    /// Page axes: the union of glyph cells as an axis-aligned rectangle.
    Page(PdfRect),
    /// A rotated line's own frame — see [`super::writing`] §3.
    Frame {
        /// The line's unit writing direction in PDF user space.
        dir: (f32, f32),
        /// The frame's origin: the first covered glyph's own origin. Any fixed
        /// point on the line would do; this one needs no arithmetic to find.
        origin: (f32, f32),
        /// Extent along the writing direction, relative to `origin`.
        along: (f32, f32),
        /// Extent across it — descender to ascender.
        perp: (f32, f32),
    },
}

impl Accum {
    /// Grow this band to include `other`.
    ///
    /// The two are always the same variant, because [`Band`] decides the
    /// variant and only equal bands are merged. A mismatch is therefore
    /// impossible rather than merely unexpected, and is left as a no-op rather
    /// than a panic: a selection that silently drew one box short is a far
    /// smaller failure than a drag that killed the application.
    pub(super) fn absorb(&mut self, other: &Self) {
        match (self, other) {
            (Self::Page(a), Self::Page(b)) => {
                *a = PdfRect::from_corners(
                    a.llx.min(b.llx),
                    a.lly.min(b.lly),
                    a.urx.max(b.urx),
                    a.ury.max(b.ury),
                );
            }
            (
                Self::Frame {
                    dir,
                    origin,
                    along,
                    perp,
                },
                Self::Frame {
                    origin: other_origin,
                    along: other_along,
                    perp: other_perp,
                    ..
                },
            ) => {
                // The incoming cell is measured from ITS OWN origin, so it is
                // rebased onto this band's before the extremes are taken. That
                // projection is the whole of why a band is exact at any angle:
                // the offset between two glyphs on one line is almost entirely
                // `along`, and whatever `perp` it has is a real difference in
                // baseline that the band must cover.
                let d = (other_origin.0 - origin.0, other_origin.1 - origin.1);
                let shift_along = d.0 * dir.0 + d.1 * dir.1;
                let shift_perp = d.0.mul_add(-dir.1, d.1 * dir.0);
                along.0 = along.0.min(other_along.0 + shift_along);
                along.1 = along.1.max(other_along.1 + shift_along);
                perp.0 = perp.0.min(other_perp.0 + shift_perp);
                perp.1 = perp.1.max(other_perp.1 + shift_perp);
            }
            _ => {}
        }
    }

    /// The band as four PDF-user-space corners.
    ///
    /// ★ The corner naming is `/QuadPoints`' (§12.5.6.10) and is relative to
    /// **the text's own baseline**, not to the page: `ul`/`ur` are the ascender
    /// side and `ll`/`lr` the descender side, `ll`/`ul` the start of the text
    /// and `lr`/`ur` its end. For `dir = (1, 0)` that is exactly
    /// [`Quad::from_rect`]'s assignment, which is the check that the rotated
    /// construction below generalises rather than replaces it.
    pub(super) fn quad(self) -> Quad {
        match self {
            Self::Page(rect) => Quad::from_rect(rect),
            Self::Frame {
                dir,
                origin,
                along,
                perp,
            } => {
                // The frame's across-axis: the writing direction turned a
                // quarter turn towards the ascender. For `dir = (1, 0)` this is
                // `(0, 1)`, i.e. up the page, which is where an ascender is.
                let up = (-dir.1, dir.0);
                let at = |a: f32, p: f32| -> (f64, f64) {
                    (
                        f64::from(a.mul_add(dir.0, p.mul_add(up.0, origin.0))),
                        f64::from(a.mul_add(dir.1, p.mul_add(up.1, origin.1))),
                    )
                };
                Quad {
                    ul: at(along.0, perp.1),
                    ur: at(along.1, perp.1),
                    ll: at(along.0, perp.0),
                    lr: at(along.1, perp.0),
                }
            }
        }
    }
}
