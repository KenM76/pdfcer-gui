//! Copy and cut parts of a placed drawing: `EditSession::copy_objects_in_form`
//! into the same `Clipped::Selection` a page-object copy makes, so the paste
//! path is shared.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/formclip.md`.

use super::clipboard::{Clipped, Refusal};
use crate::app::actions::{Action, VectorAction};
use crate::app::state::OpenDoc;

/// The page and form leaves a copy acts on: the selection holds leaves and no
/// page object and no annotation. `None` sends the caller to the page copy.
#[must_use]
pub fn leaves_only(doc: &OpenDoc) -> Option<(usize, Vec<usize>)> {
    if doc.selection.annot().is_some() {
        return None;
    }
    let page = doc.selection.entries().first()?.page;
    if !doc.selection.object_indices_on(page).is_empty() {
        return None;
    }
    let leaves = doc.selection.leaf_indices_on(page);
    (!leaves.is_empty()).then_some((page, leaves))
}

/// Copy `leaves` on `page` to the clipboard, lifted by
/// [`super::take::leaves_of`].
pub fn copy(
    ctx: &egui::Context,
    doc: &mut OpenDoc,
    page: usize,
    leaves: &[usize],
) -> Result<Clipped, Refusal> {
    let taken = super::take::leaves_of(doc, page, leaves)?;
    let clipped = taken.clipped();
    super::clipboard::store(ctx, clipped.clone());
    super::clipboard::publish(ctx, &taken.clip, leaves.len(), 0);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "clipboard-copy kind=form-leaves page={page} leaves={} items={} bytes={}",
            crate::diag::index_list(leaves),
            taken.clip.len(),
            taken.clip.to_bytes().len()
        )
    });
    Ok(clipped)
}

/// Copy `leaves`, then queue their deletion as one undo entry. The copy comes
/// first, so a refused copy deletes nothing.
pub fn cut(
    ctx: &egui::Context,
    doc: &mut OpenDoc,
    page: usize,
    leaves: Vec<usize>,
    actions: &mut Vec<Action>,
) -> Result<Clipped, Refusal> {
    let clipped = copy(ctx, doc, page, &leaves)?;
    actions.push(Action::Vector(VectorAction::DeleteParts {
        page,
        objects: Vec::new(),
        leaves,
    }));
    Ok(clipped)
}
