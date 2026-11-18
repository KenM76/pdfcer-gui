//! # `groupaction` — Group and Ungroup, for page objects and for markups
//!
//! Raised by `pdfcer_gui::app::dispatch::group`; applied by
//! `pdfcer_gui::app::actions::group`. Each is one `EditSession` call and one
//! undo entry.

use pdfcer_core::object::ObjId;

/// A grouping verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupAction {
    /// **Gather page objects into one form XObject**:
    /// `EditSession::group_objects`.
    Objects {
        /// 0-based page index.
        page: usize,
        /// Paint-order indices into the page's objects.
        objects: Vec<usize>,
    },
    /// **Gather objects inside one placed drawing into a group within it**:
    /// `EditSession::group_objects_in_form`.
    ObjectsInForm {
        /// 0-based page index.
        page: usize,
        /// Indices into the page's leaves, all in one invocation of one form.
        leaves: Vec<usize>,
    },
    /// **Dissolve the form a leaf is directly inside**:
    /// `EditSession::ungroup_form_in_form`.
    UngroupInForm {
        /// 0-based page index.
        page: usize,
        /// The leaf's index in the page's leaves.
        leaf: usize,
    },
    /// **Inline a form XObject at this `Do`**: `EditSession::ungroup_form`.
    UngroupForm {
        /// 0-based page index.
        page: usize,
        /// The form's paint-order index.
        object: usize,
    },
    /// **Group markups under a primary** (`/IRT` + `/RT /Group`), each
    /// selected markup bringing the group it is in, recorded so Ungroup puts
    /// those groups back: `EditSession::merge_annotation_groups`.
    Markups {
        /// The page all of them are on.
        page: usize,
        /// The group's primary: other readers show its comment and author.
        primary: ObjId,
        /// The others.
        members: Vec<ObjId>,
    },
    /// **Dissolve the group a markup is in, one level**: recorded earlier
    /// groups come back. `EditSession::ungroup_annotations`.
    UngroupMarkups {
        /// The page it is on.
        page: usize,
        /// The primary or any member.
        markup: ObjId,
    },
}
