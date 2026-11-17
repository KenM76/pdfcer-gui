//! # `canvas::selection::groupmates` — a click on a grouped markup selects
//! its whole group
//!
//! A markup group (ISO 32000-1 §12.5.6.2) is a primary plus the markups whose
//! `/IRT` names it with `/RT /Group`. The one clicked is selected last, so it
//! is the primary of the *selection* and the one the handles are drawn from.

use egui::Rect;
use pdfcer_core::object::ObjId;

use crate::app::dispatch::group::group_primary;
use crate::app::state::OpenDoc;
use crate::canvas::selection::{AnnotSelection, SelectionState};

/// The other selectable markups on the hit's page that share its group, in
/// paint order. Empty when it is in no group.
#[must_use]
pub fn of(doc: &OpenDoc, hit: &AnnotSelection) -> Vec<AnnotSelection> {
    let root = |id: ObjId| group_primary(doc, id).unwrap_or(id);
    let mine = root(hit.target.id);
    super::annot::in_band(doc, hit.target.page, Rect::EVERYTHING, true)
        .into_iter()
        .filter(|a| a.target.id != hit.target.id && root(a.target.id) == mine)
        .collect()
}

/// Select `hit`, and with it every markup grouped with it.
pub fn select(doc: &OpenDoc, selection: &mut SelectionState, hit: AnnotSelection) {
    let mates = of(doc, &hit);
    let Some((first, rest)) = mates.split_first() else {
        selection.select_annot(hit);
        return;
    };
    selection.select_annot(first.clone());
    for mate in rest {
        selection.toggle_annot(mate.clone());
    }
    let (page, id) = (hit.target.page, hit.target.id.num);
    selection.toggle_annot(hit);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "annot-group-select page={page} id={id} mates={} selected={}",
            mates.len(),
            selection.annots().len()
        )
    });
}
