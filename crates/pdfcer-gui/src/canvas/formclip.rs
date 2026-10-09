//! Copy and cut parts of a placed drawing: `EditSession::copy_objects_in_form`
//! into the same `Clipped::Selection` a page-object copy makes, so the paste
//! path is shared.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/formclip.md`.

use std::sync::Arc;

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

/// Copy `leaves` on `page` to the clipboard. The engine bakes the drawing's
/// placement into each item, so the clip pastes as page content where and as
/// large as it was drawn.
pub fn copy(
    ctx: &egui::Context,
    doc: &mut OpenDoc,
    page: usize,
    leaves: &[usize],
) -> Result<Clipped, Refusal> {
    // `&mut` only because the engine reads its cached page model; a holder of
    // the session elsewhere refuses the copy rather than waiting.
    let Some(session) = Arc::get_mut(&mut doc.session) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "clipboard-copy-refused reason=session-borrowed".to_owned()
        });
        return Err(Refusal::EngineRefused);
    };
    let clip = session
        .copy_objects_in_form(page, leaves)
        .map_err(|error| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("clipboard-copy-refused reason=engine detail={error}")
            });
            Refusal::EngineRefused
        })?;
    let clipped = Clipped::Selection {
        count: clip.len(),
        annotations: 0,
        annot_ids: Vec::new(),
        left_behind: Vec::new(),
        thin: 0,
        in_form_left: 0,
        bytes: clip.to_bytes(),
        page,
        anchor: super::clipboard::anchor_of(&clip),
    };
    super::clipboard::store(ctx, clipped.clone());
    super::clipboard::publish(ctx, &clip, leaves.len(), 0);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "clipboard-copy kind=form-leaves page={page} leaves={} items={} bytes={}",
            crate::diag::index_list(leaves),
            clip.len(),
            clip.to_bytes().len()
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
    actions.push(Action::Vector(VectorAction::DeleteLeavesInForm {
        page,
        leaves,
    }));
    Ok(clipped)
}
