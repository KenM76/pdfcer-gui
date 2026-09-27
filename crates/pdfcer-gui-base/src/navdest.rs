//! # `navdest` — a place on a page a bookmark asked the view to arrive at

/// **A place on a page a bookmark asked the view to arrive at.**
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PendingDestination {
    /// Put this PDF-space point at the view's top-left.
    ///
    /// `None` on an axis is §12.3.2.2's null — *"leave this one as it is"*. A
    /// literal `0.0` is a real coordinate and is NOT that: Table 151 states the
    /// zero-means-null equivalence for `zoom` alone.
    Point {
        /// The page the coordinates belong to.
        page: usize,
        /// PDF-space left edge.
        left: Option<f64>,
        /// PDF-space top edge.
        top: Option<f64>,
    },
    /// Frame this PDF-space rectangle, `(left, bottom, right, top)`.
    Rect {
        /// The page the rectangle belongs to.
        page: usize,
        /// The rectangle.
        rect: (f64, f64, f64, f64),
    },
}
impl PendingDestination {
    /// **The page this destination is about** — the one field both variants
    /// share, and the one `pdfcer_gui::canvas::destination::arrive_step` is a decision about.
    #[must_use]
    pub const fn page(&self) -> usize {
        match self {
            Self::Point { page, .. } | Self::Rect { page, .. } => *page,
        }
    }
}
