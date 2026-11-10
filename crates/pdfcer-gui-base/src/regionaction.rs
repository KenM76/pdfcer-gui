//! # `regionaction` — what the snapshot region's commands ask of the document
//!
//! The region is the snapshot box ([`crate::snapshotbox::SnapshotBox`]): one
//! rectangle on one page, in PDF user space. Each action carries the region
//! already resolved, so applying one reads no view state.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/regionaction.md`.

use pdfcer_core::annot_author::RedactAppearance;
use pdfcer_core::page_tree::Rect;

/// One command on the snapshot region.
#[derive(Debug, Clone, PartialEq)]
pub enum RegionAction {
    /// Mark `rect` on `page` for redaction, as one `/Redact` mark. Anything the
    /// rect covers or crosses goes on apply, whatever Delete takes is set to.
    Redact {
        /// 0-based page index.
        page: usize,
        /// The region, PDF user space.
        rect: Rect,
        /// How the mark looks once applied.
        appearance: RedactAppearance,
    },
    /// Mark everything outside each page's own rect for redaction, one mark per
    /// page, as one undo step.
    RedactOutside {
        /// `(page, keep)` pairs, ascending by page, each rect in that page's
        /// user space.
        keeps: Vec<(usize, Rect)>,
        /// Pages in the operator's scope left out because their size differs
        /// from the region page's.
        skipped: usize,
        /// How the marks look once applied.
        appearance: RedactAppearance,
    },
    /// Delete what the region took, then name both counts on the status line.
    Delete {
        /// The delete, as the Delete key would raise it for the same objects.
        delete: crate::subactions::VectorAction,
        /// How many objects `delete` removes.
        deleted: usize,
        /// Objects the region touches but does not enclose, which it keeps.
        crossing_kept: usize,
    },
}
