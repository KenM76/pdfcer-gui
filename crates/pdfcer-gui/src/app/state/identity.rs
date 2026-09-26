//! # `app::state::identity` — the two small types that say *which thing*
//!
//! [`Origin`] says which file a document came from — or that it came from
//! none. [`SelectedField`] says which form field, and which of its boxes, the
//! operator clicked. Both are plain data with no behaviour; neither touches the
//! large document record in [`super`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/state/identity.md`.

/// **Whether an open document has a file behind it.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Loaded from [`OpenDoc::path`], which names a file that existed.
    Opened,
    /// Made by `file.new` from `crate::app::blank::TEMPLATE`.
    ///
    /// [`OpenDoc::path`] is a **name** — `crate::text::files::untitled` — and
    /// nothing is at it. Anything that would write to, read from, or remember
    /// something *about a file* must consult [`OpenDoc::stored_under`] first.
    Created,
}

/// Which form field the operator clicked, and which of its widgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedField {
    /// The field's fully-qualified name.
    pub field: String,
    /// Which of the field's widgets was clicked.
    pub widget: usize,
    /// The 0-based page that widget is on.
    pub page: usize,
}
