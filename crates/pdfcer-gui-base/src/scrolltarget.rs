//! # `scrolltarget` — A scroll the view still owes: a destination point, a minimum region to reveal, or a find hit, each waiting for a frame that shows its page.

/// A destination that named a point, waiting for a frame that can solve it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DestScroll {
    /// The page the destination is on. Solved only on a frame showing it.
    pub page: usize,
    /// Where the point sits across the page's **canvas-space** width, as a
    /// fraction of the page extent — `None` for §12.3.2.2's null, *"leave this
    /// axis as it is"*.
    ///
    /// A canvas axis, not a PDF one, and the difference is `/Rotate`: on a
    /// 90°-rotated sheet a PDF *x* drives the canvas *y*, so a `/FitV` — which
    /// specifies a PDF left edge and nothing else — constrains the view
    /// vertically. `fracs_for` resolves that once, when the point is parked.
    pub frac_x: Option<f32>,
    /// See [`Self::frac_x`].
    pub frac_y: Option<f32>,
    /// The horizontal content offset the view had **before anything
    /// navigated** — see `pdfcer_gui::app::state::OpenDoc::dest_origin_x`. Both
    /// the visibility test and the hold-still answer are made against this
    /// rather than against the live offset, because on a cross-page
    /// destination the live offset is already the strip's horizontal centring
    /// of the new page.
    pub origin_x: f32,
    /// Frames spent waiting for [`Self::page`]. See `DEST_GRACE_FRAMES`.
    pub waited: u8,
}

/// A rectangle waiting for a frame that can scroll to it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinReveal {
    /// The page the rectangle is on. Solved only on a frame showing it.
    pub page: usize,
    /// The rectangle's top-left, as a fraction of the page's **canvas** extent.
    ///
    /// Canvas space, not PDF space: the page's `/Rotate` is already folded in
    /// by `viewer::pdf_space_to_canvas`, so a widget on a rotated sheet arrives
    /// here at the position the operator sees it. See
    /// `fracs_for_canvas_rect`.
    pub min: (f32, f32),
    /// The rectangle's bottom-right, in the same units as [`Self::min`].
    pub max: (f32, f32),
    /// How many frames this has waited for its page.
    pub waited: u8,
    /// Who asked, for the trace. Never displayed.
    pub why: &'static str,
}

/// A hit that has been navigated to and is waiting to be scrolled into view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reveal {
    /// The page the hit is on. The offset is solved only on a frame that is
    /// actually showing this page.
    pub page: usize,
    /// The hit's centre as a fraction of the page's extent.
    ///
    /// A **fraction**, not a canvas point, for exactly the reason
    /// `pdfcer_gui::app::state::ZoomAnchor` carries one: it is independent of
    /// the zoom, so it can be recorded before the frame that will spend it
    /// and stays correct if the operator zooms in between.
    pub frac: (f32, f32),
    /// How many frames this has waited for its page. See
    /// `REVEAL_GRACE_FRAMES`.
    pub waited: u8,
}
