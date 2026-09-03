//! # `app::state::identity` — the two small types that say *which thing*
//!
//! Split out of [`super`] on 2026-08-26, when adding form-field selection
//! pushed that file past R2's 1,500-line limit. The seam is not arbitrary:
//! both types here answer the question *which one?* about something the shell
//! is pointing at, and neither has anything to do with the large document
//! record that fills the rest of the file.
//!
//! [`Origin`] says which file a document came from — or that it came from
//! none. [`SelectedField`] says which form field, and which of its boxes, the
//! operator clicked. Both are plain data with no behaviour, which is what makes
//! them the cheapest thing in that file to move and the easiest to find here.

/// **Whether an open document has a file behind it.**
///
/// Two variants rather than an `Option<PathBuf>` on [`OpenDoc::path`], and the
/// choice is deliberate. Every document — created or opened — needs a
/// *identity* that is path-shaped: the forms cache keys on it, the Pages panel
/// captions from it, the trace names it, and a save suggestion would be built
/// from it. Making the path optional would push an `unwrap_or_default()` into
/// each of those, and `""` is the identity every unnamed document would then
/// share. What actually varies is one much narrower fact — *is there a file
/// there* — so that is what is stored, and [`OpenDoc::stored_under`] is the
/// only place it is asked.
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
///
/// ★ Both halves are needed and neither is redundant. The **name** is what
/// every field verb takes — `rename_field`, `delete_field` — because a field is
/// identified by name and not by object id. The **widget index** is what
/// `delete_widget` takes, and is the only way to say *"the box on page 3"* when
/// one field is drawn in two places.
///
/// The page is carried so the properties panel can say where the clicked box is
/// without re-walking the form to find out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedField {
    /// The field's fully-qualified name.
    pub field: String,
    /// Which of the field's widgets was clicked.
    pub widget: usize,
    /// The 0-based page that widget is on.
    pub page: usize,
}
