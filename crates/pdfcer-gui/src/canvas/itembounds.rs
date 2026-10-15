//! Where each selected form field and annotation is drawn, in canvas space:
//! one reading shared by the Align panel's measure and the group drag, so
//! what Align lines up and what a drag picks up are the same boxes.

use egui::Rect;

use crate::app::state::{OpenDoc, SelectedField};
use crate::canvas::selection::{AnnotTarget, SelectionState};

/// `fields`' widget boxes, in the order given, from the placement census the
/// canvas hit-tests (`canvas::forms::placed`); a field not placed this frame
/// is dropped.
pub fn fields(
    ctx: &egui::Context,
    doc: &OpenDoc,
    fields: impl IntoIterator<Item = SelectedField>,
) -> Vec<(SelectedField, Rect)> {
    let fields: Vec<_> = fields.into_iter().collect();
    if fields.is_empty() {
        return Vec::new();
    }
    let placed = crate::canvas::forms::placed(ctx, doc);
    fields
        .into_iter()
        .filter_map(|f| {
            let rect = placed
                .targets
                .iter()
                .find(|t| t.page == f.page && t.field == f.field && t.widget == f.widget)?
                .rect;
            Some((f, rect))
        })
        .collect()
}

/// The selected annotations' outlines, in selection order; only those on
/// `page` when one is given.
#[must_use]
pub fn annots(selection: &SelectionState, page: Option<usize>) -> Vec<(AnnotTarget, Rect)> {
    selection
        .annots()
        .into_iter()
        .filter(|a| page.is_none_or(|p| a.target.page == p))
        .map(|a| (a.target.clone(), a.outline))
        .collect()
}
