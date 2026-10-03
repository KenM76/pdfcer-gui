//! # `snapshotbox` — the box View ▸ Snapshot leaves on a page
//!
//! Held in PDF user space, so zooming, scrolling and resizing the window move
//! the box with the page rather than with the screen. One box per document, on
//! one page; it is view state and is never saved.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/snapshotbox.md`.

use pdfcer_core::page_tree::Rect;

/// A snapshot box: a page and a rectangle on it, in PDF points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapshotBox {
    /// Zero-based page index.
    pub page: usize,
    /// The box in PDF user space, normalised.
    pub rect: Rect,
}

impl SnapshotBox {
    /// The smallest side a box may have, in points; a release closer than this
    /// to its press is a slip, not a box.
    pub const MIN_SIDE_PT: f64 = 2.0;

    /// The box two corners span, in either order; `None` when a side is
    /// shorter than [`Self::MIN_SIDE_PT`].
    #[must_use]
    pub fn from_corners(page: usize, a: (f64, f64), b: (f64, f64)) -> Option<Self> {
        let rect = Rect {
            llx: a.0.min(b.0),
            lly: a.1.min(b.1),
            urx: a.0.max(b.0),
            ury: a.1.max(b.1),
        };
        let big_enough =
            rect.urx - rect.llx >= Self::MIN_SIDE_PT && rect.ury - rect.lly >= Self::MIN_SIDE_PT;
        big_enough.then_some(Self { page, rect })
    }

    /// The trace fields `snapshot-box` carries: page and corners to 0.1 pt.
    #[must_use]
    pub fn trace_fields(&self) -> String {
        let r = self.rect;
        format!(
            // ui-text-exempt: diagnostic trace fields, never displayed.
            "page={} llx={:.1} lly={:.1} urx={:.1} ury={:.1}",
            self.page, r.llx, r.lly, r.urx, r.ury
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_in_either_order_give_the_same_box() {
        let a = SnapshotBox::from_corners(2, (10.0, 90.0), (60.0, 20.0));
        let b = SnapshotBox::from_corners(2, (60.0, 20.0), (10.0, 90.0));
        assert_eq!(a, b);
        let r = a.map(|s| s.rect);
        assert_eq!(
            r,
            Some(Rect {
                llx: 10.0,
                lly: 20.0,
                urx: 60.0,
                ury: 90.0
            })
        );
    }

    #[test]
    fn a_slip_makes_no_box() {
        assert_eq!(
            SnapshotBox::from_corners(0, (10.0, 10.0), (11.0, 50.0)),
            None
        );
        assert_eq!(
            SnapshotBox::from_corners(0, (10.0, 10.0), (50.0, 11.0)),
            None
        );
    }

    #[test]
    fn the_trace_names_the_page_and_corners() {
        let s = SnapshotBox::from_corners(1, (0.0, 0.0), (72.0, 36.0));
        assert_eq!(
            s.map(|s| s.trace_fields()).as_deref(),
            Some("page=1 llx=0.0 lly=0.0 urx=72.0 ury=36.0")
        );
    }
}
