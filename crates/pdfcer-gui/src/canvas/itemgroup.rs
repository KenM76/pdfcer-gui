//! Dragging several selected form fields, or several selected annotations,
//! together. A press on any member that will move picks the group up; the
//! release raises one [`VectorAction::MoveMixed`] with gesture `drag`, which
//! `app::actions::movemixed` applies as one undo step (batch verbs owed:
//! G189 for widgets, G190 for annotations).
//!
//! A locked annotation stays in the action so the engine refuses it by name
//! and the status line says how many moved; it gets no ghost and cannot pick
//! the group up. Selected fields on a page other than the primary's do not
//! move: the operator cannot see them follow. A ce dimension's ghost is its
//! box translated, while the commit places it by `dimdrag::placed`.
//!
//! Design: `docs/modules/pdfcer-gui/canvas/itemgroup.md`.

use egui::Rect;
use pdfcer_core::page_tree::Page;

use crate::app::actions::Action;
use crate::app::actions::vector::VectorAction;
use crate::app::state::{OpenDoc, SelectedField};
use crate::canvas::gesture::Phase;
use crate::canvas::itembounds;
use crate::canvas::selection::{AnnotTarget, SelectionState};

/// One member of a group.
enum Member {
    Widget(SelectedField),
    Annot(AnnotTarget),
}

impl Member {
    fn moves(&self) -> bool {
        match self {
            Self::Widget(_) => true,
            Self::Annot(t) => !t.locked,
        }
    }
}

/// Several selected items that a drag moves together, all on one page, each
/// with its canvas-space box.
pub struct Group {
    /// The page the members are on.
    pub page: usize,
    members: Vec<(Member, Rect)>,
}

impl Group {
    /// The selected annotations, when there are several and one will move.
    /// `toggle_annot` keeps the selection to one page.
    #[must_use]
    pub fn annots(selection: &SelectionState) -> Option<Self> {
        let all = itembounds::annots(selection, None);
        if all.len() < 2 || all.iter().all(|(t, _)| t.locked) {
            return None;
        }
        Some(Self {
            page: all[0].0.page,
            members: all
                .into_iter()
                .map(|(t, r)| (Member::Annot(t), r))
                .collect(),
        })
    }

    /// The selected fields' boxes on the primary's page and placed this
    /// frame, when several fields are selected and one of them is there.
    #[must_use]
    pub fn fields(ctx: &egui::Context, doc: &OpenDoc) -> Option<Self> {
        let primary = doc.selected_field.as_ref()?;
        let all = doc.field_set.all(Some(primary));
        if all.len() < 2 {
            return None;
        }
        let on_page = all.into_iter().filter(|f| f.page == primary.page);
        let members: Vec<_> = itembounds::fields(ctx, doc, on_page)
            .into_iter()
            .map(|(f, r)| (Member::Widget(f), r))
            .collect();
        (!members.is_empty()).then_some(Self {
            page: primary.page,
            members,
        })
    }

    /// Whether a press at `p` (screen space) lands inside a member that will
    /// move, which makes the drag a group move.
    #[must_use]
    pub fn body_at(&self, map: &crate::canvas::mapping::PageMapping, p: egui::Pos2) -> bool {
        self.members
            .iter()
            .any(|(m, r)| m.moves() && map.rect_to_screen(*r).contains(p))
    }

    /// Drive one frame of the drag: the canvas-space ghost of every member
    /// that will move, empty on the release frame. `sheet` converts the delta
    /// to page space; without one, or with a delta it cannot convert, the
    /// release declines with `None`.
    pub fn drag(
        self,
        delta: egui::Vec2,
        phase: Phase,
        sheet: Option<&Page>,
        actions: &mut Vec<Action>,
    ) -> Option<Vec<Rect>> {
        if phase != Phase::Complete {
            return Some(
                self.members
                    .iter()
                    .filter(|(m, _)| m.moves())
                    .map(|(_, r)| r.translate(delta))
                    .collect(),
            );
        }
        let d = super::moving::page_delta(delta, sheet?)?;
        let (mut widgets, mut annots) = (Vec::new(), Vec::new());
        for (m, _) in self.members {
            match m {
                Member::Widget(f) => widgets.push((f, d.dx, d.dy)),
                Member::Annot(t) => annots.push((t, d.dx, d.dy)),
            }
        }
        actions.push(
            VectorAction::MoveMixed {
                page: self.page,
                objects: Vec::new(),
                leaves: Vec::new(),
                widgets,
                annots,
                notes: Vec::new(),
                // ui-text-exempt: diagnostic gesture name, never displayed
                gesture: "drag",
            }
            .into(),
        );
        Some(Vec::new())
    }
}

#[cfg(test)]
mod tests;
