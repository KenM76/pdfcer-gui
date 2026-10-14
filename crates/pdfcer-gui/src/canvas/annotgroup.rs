//! # `canvas::annotgroup` — dragging several selected annotations together
//!
//! With two or more markups, links or ce dimensions selected, a press on any
//! unlocked member's box and a drag moves every member by the same delta. The
//! release raises one [`VectorAction::MoveMixed`] with gesture `drag`, which
//! `app::actions::movemixed` applies and folds into one undo step (G190 for
//! the missing batch verb).
//!
//! Locked members stay in the action so the engine refuses them by name and
//! the status line says how many moved; they get no ghost, because they will
//! not move. A ce dimension's ghost is its box translated, while the commit
//! places it by `dimdrag::placed`, so its box may land short of the ghost.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/annotgroup.md`.

use egui::Rect;
use pdfcer_core::page_tree::Page;

use crate::app::actions::Action;
use crate::app::actions::vector::VectorAction;
use crate::canvas::gesture::Phase;
use crate::canvas::selection::{AnnotSelection, SelectionState};

/// The selected annotations when there are several, else `None`.
#[must_use]
fn group(selection: &SelectionState) -> Option<Vec<&AnnotSelection>> {
    let all = selection.annots();
    (all.len() > 1).then_some(all)
}

/// Whether a press at `p` (screen space) lands inside an unlocked member of a
/// several-annotation selection, which makes the drag a group move.
#[must_use]
pub fn body_at(
    map: &crate::canvas::mapping::PageMapping,
    selection: &SelectionState,
    p: egui::Pos2,
) -> bool {
    group(selection).is_some_and(|all| {
        all.iter()
            .any(|a| !a.target.locked && map.rect_to_screen(a.outline).contains(p))
    })
}

/// Drive one frame of a group drag. `None` when the selection is not a group,
/// so the single-annotation route answers instead; otherwise the canvas-space
/// ghost of every member that will move (empty on the release frame).
pub fn drag(
    delta: egui::Vec2,
    phase: Phase,
    page: Option<&Page>,
    selection: &SelectionState,
    actions: &mut Vec<Action>,
) -> Option<Vec<Rect>> {
    let all = group(selection)?;
    if !all.iter().any(|a| !a.target.locked) {
        return None;
    }
    if phase != Phase::Complete {
        return Some(
            all.iter()
                .filter(|a| !a.target.locked)
                .map(|a| a.outline.translate(delta))
                .collect(),
        );
    }
    let d = super::moving::page_delta(delta, page?)?;
    // `toggle_annot` keeps a selection to one page.
    let page_index = all[0].target.page;
    let annots: Vec<_> = all.iter().map(|a| (a.target.clone(), d.dx, d.dy)).collect();
    actions.push(
        VectorAction::MoveMixed {
            page: page_index,
            objects: Vec::new(),
            leaves: Vec::new(),
            widgets: Vec::new(),
            annots,
            notes: Vec::new(),
            // ui-text-exempt: diagnostic gesture name, never displayed
            gesture: "drag",
        }
        .into(),
    );
    Some(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::selection::{AnnotKind, AnnotTarget};
    use pdfcer_core::object::ObjId;

    fn member(num: u32, x: f32, locked: bool) -> AnnotSelection {
        AnnotSelection {
            target: AnnotTarget {
                page: 0,
                id: ObjId::new(num, 0),
                kind: AnnotKind::Markup,
                // ui-text-exempt: a PDF /Subtype name in a test fixture.
                subtype: "Square".to_owned(),
                locked,
            },
            outline: Rect::from_min_size(egui::pos2(x, 20.0), egui::vec2(40.0, 30.0)),
            oriented: None,
        }
    }

    fn state(members: Vec<AnnotSelection>) -> SelectionState {
        let mut s = SelectionState::default();
        for (i, m) in members.into_iter().enumerate() {
            if i == 0 {
                s.select_annot(m);
            } else {
                s.toggle_annot(m);
            }
        }
        s
    }

    #[test]
    fn one_annotation_is_not_a_group() {
        let s = state(vec![member(5, 10.0, false)]);
        let mut actions = Vec::new();
        assert!(
            drag(
                egui::vec2(3.0, 4.0),
                Phase::InFlight,
                None,
                &s,
                &mut actions
            )
            .is_none()
        );
    }

    #[test]
    fn every_unlocked_member_gets_a_ghost() {
        let s = state(vec![
            member(5, 10.0, false),
            member(7, 100.0, true),
            member(9, 200.0, false),
        ]);
        let mut actions = Vec::new();
        let ghosts = drag(
            egui::vec2(3.0, 4.0),
            Phase::InFlight,
            None,
            &s,
            &mut actions,
        )
        .expect("a group");
        assert_eq!(ghosts.len(), 2);
        assert!(ghosts.iter().any(|r| r.min == egui::pos2(13.0, 24.0)));
        assert!(ghosts.iter().any(|r| r.min == egui::pos2(203.0, 24.0)));
        assert!(actions.is_empty());
    }

    #[test]
    fn an_all_locked_group_does_not_drag() {
        let s = state(vec![member(5, 10.0, true), member(7, 100.0, true)]);
        let mut actions = Vec::new();
        assert!(
            drag(
                egui::vec2(3.0, 4.0),
                Phase::InFlight,
                None,
                &s,
                &mut actions
            )
            .is_none()
        );
    }
}
