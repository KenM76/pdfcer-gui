//! # `canvas::fieldgroup` — dragging several selected form fields together
//!
//! With two or more fields selected, a press on any selected box on the
//! primary's page and a drag moves every such box by the same delta. The
//! release raises one [`VectorAction::MoveMixed`] with gesture `drag`, which
//! `app::actions::movemixed` applies and folds into one undo step (G189 for
//! the missing batch verb). Selected boxes on other pages do not move: the
//! operator cannot see them follow.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/fieldgroup.md`.

use egui::Rect;

use crate::app::actions::Action;
use crate::app::actions::vector::VectorAction;
use crate::app::state::{OpenDoc, SelectedField};
use crate::canvas::gesture::Phase;

/// The selected boxes on the primary's page with their canvas-space rects,
/// when several fields are selected; else `None`.
fn group(ctx: &egui::Context, doc: &OpenDoc) -> Option<Vec<(SelectedField, Rect)>> {
    let primary = doc.selected_field.as_ref()?;
    let all = doc.field_set.all(Some(primary));
    if all.len() < 2 {
        return None;
    }
    let placed = crate::canvas::forms::placed(ctx, doc);
    let members: Vec<_> = all
        .into_iter()
        .filter(|f| f.page == primary.page)
        .filter_map(|f| {
            let rect = placed
                .targets
                .iter()
                .find(|t| t.page == f.page && t.field == f.field && t.widget == f.widget)?
                .rect;
            Some((f, rect))
        })
        .collect();
    (!members.is_empty()).then_some(members)
}

/// Whether a press at `p` (screen space) lands inside a member of a
/// several-field selection, which makes the drag a group move.
#[must_use]
pub fn body_at(
    ctx: &egui::Context,
    doc: &OpenDoc,
    map: &crate::canvas::mapping::PageMapping,
    p: egui::Pos2,
) -> bool {
    group(ctx, doc).is_some_and(|all| all.iter().any(|(_, r)| map.rect_to_screen(*r).contains(p)))
}

/// Drive one frame of a group drag. `None` when fewer than two fields are
/// selected, so the single-field route answers instead; otherwise the
/// canvas-space ghost of every box that will move (empty on the release
/// frame).
pub fn drag(
    delta: egui::Vec2,
    phase: Phase,
    ctx: &egui::Context,
    doc: &OpenDoc,
    actions: &mut Vec<Action>,
) -> Option<Vec<Rect>> {
    let all = group(ctx, doc)?;
    if phase != Phase::Complete {
        return Some(all.iter().map(|(_, r)| r.translate(delta)).collect());
    }
    let page = all[0].0.page;
    let d = super::moving::page_delta(delta, doc.pages.get(page)?)?;
    let widgets: Vec<_> = all.into_iter().map(|(f, _)| (f, d.dx, d.dy)).collect();
    actions.push(
        VectorAction::MoveMixed {
            page,
            objects: Vec::new(),
            in_form: false,
            widgets,
            annots: Vec::new(),
            // ui-text-exempt: diagnostic gesture name, never displayed
            gesture: "drag",
        }
        .into(),
    );
    Some(Vec::new())
}
