//! # `formpreview` — what the Form preview window asks of the open document
//!
//! Both edits carry the window's reading choices, in display order, so the
//! form is merged and laid out as the window showed it.

/// One request from Form preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormPreviewAction {
    /// Use these pages: the computed pages replace the stand-in pages,
    /// with fillable fields when `fillable`.
    UsePages { choices: [bool; 5], fillable: bool },
    /// A value typed into the field at template `path` (child indices from
    /// the `template` element), written into the form's data.
    Fill {
        path: Vec<usize>,
        value: String,
        /// The field's `name`, for the status line.
        name: Option<String>,
        choices: [bool; 5],
    },
}
