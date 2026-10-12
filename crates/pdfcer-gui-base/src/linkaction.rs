//! # `linkaction` — the verbs that author and edit a `/Link`
//!
//! Raised by the Add-link window and the Properties link section; applied by
//! `pdfcer_gui::app::actions::links`. Each is one `EditSession` call and one
//! undo entry.

use pdfcer_core::edit::{LinkBorder, LinkTarget};
use pdfcer_core::object::ObjId;
use pdfcer_core::page_tree::Rect;

/// A link verb.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkAction {
    /// **Create a link** over `rect` on `page`: `EditSession::add_link`.
    Add {
        /// 0-based page index.
        page: usize,
        /// Unrotated page space, y-up, PDF points.
        rect: Rect,
        /// Where it goes.
        target: LinkTarget,
        /// Its visible border, or `None` for none.
        border: Option<LinkBorder>,
    },
    /// **Point a link somewhere else**: `EditSession::set_link_target`.
    SetTarget {
        /// The link, by stable object id.
        id: ObjId,
        /// Where it now goes.
        target: LinkTarget,
    },
    /// **Give a link a border, or take it away**:
    /// `EditSession::set_link_border`.
    SetBorder {
        /// The link, by stable object id.
        id: ObjId,
        /// The border, or `None` for none.
        border: Option<LinkBorder>,
    },
}
