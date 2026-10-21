//! # `panels::objects::rowclick` — what a click on an object row selects
//!
//! The list conventions every file manager follows: a plain click selects the
//! row alone (and a plain click on the only selected row clears it), Ctrl+click
//! adds or removes one row, Shift+click selects the run of object rows from
//! the anchor to the clicked row, Ctrl+Shift+click adds that run. A
//! right-click on a row already in the selection keeps the selection, so a
//! context-menu command acts on every selected object; a right-click elsewhere
//! selects that row alone first.
//!
//! Raises `SelectionAction`s; never writes the selection itself.

use pdfcer_gui_base::canvastarget::TargetId;
use pdfcer_gui_base::subactions::SelectionAction;

use super::ObjectTreeRow;
use crate::app::actions::Action;

/// One click on the object row `index`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RowClick {
    /// The row's paint-order index.
    pub index: usize,
    /// The secondary button.
    pub secondary: bool,
    /// Ctrl (Cmd on macOS) held.
    pub command: bool,
    /// Shift held.
    pub shift: bool,
}

impl RowClick {
    /// The click `resp` carries on the row `index`, if any.
    pub fn read(resp: &egui::Response, index: usize) -> Option<Self> {
        let secondary = resp.secondary_clicked();
        if !resp.clicked() && !secondary {
            return None;
        }
        let mods = resp.ctx.input(|i| i.modifiers);
        Some(Self {
            index,
            secondary,
            command: mods.command,
            shift: mods.shift,
        })
    }
}

/// The selection change `click` asks for, given the object rows `selected` on
/// `page` and the tree's `rows`. Updates `anchor`.
pub(super) fn resolve(
    click: RowClick,
    page: usize,
    rows: &[ObjectTreeRow],
    selected: &[usize],
    anchor: &mut Option<usize>,
) -> Option<SelectionAction> {
    let RowClick {
        index,
        secondary,
        command,
        shift,
    } = click;
    let target = |i: usize| TargetId::Object(i as u64);
    let only = |i: usize| SelectionAction::SelectObject {
        page,
        object: Some(target(i)),
    };
    if secondary {
        if selected.contains(&index) {
            return None;
        }
        *anchor = Some(index);
        return Some(only(index));
    }
    if shift {
        let from = anchor.unwrap_or(index);
        return Some(SelectionAction::SetObjects {
            page,
            objects: object_run(rows, from, index)
                .into_iter()
                .map(target)
                .collect(),
            add: command,
        });
    }
    *anchor = Some(index);
    if command {
        return Some(if selected.contains(&index) {
            SelectionAction::RemoveObjects {
                page,
                objects: vec![target(index)],
            }
        } else {
            SelectionAction::SetObjects {
                page,
                objects: vec![target(index)],
                add: true,
            }
        });
    }
    if selected == [index] {
        return Some(SelectionAction::SelectObject { page, object: None });
    }
    Some(only(index))
}

/// The object rows from `from` to `to` inclusive, in the order the tree shows
/// them. Either end missing from the tree gives just `to`.
fn object_run(rows: &[ObjectTreeRow], from: usize, to: usize) -> Vec<usize> {
    let order: Vec<usize> = rows
        .iter()
        .filter_map(|r| match r {
            ObjectTreeRow::Object { index } => Some(*index),
            _ => None,
        })
        .collect();
    let at = |i: usize| order.iter().position(|&o| o == i);
    match (at(from), at(to)) {
        (Some(a), Some(b)) => order[a.min(b)..=a.max(b)].to_vec(),
        _ => vec![to],
    }
}

/// Raise the change `click` asks for.
pub(super) fn apply(
    click: RowClick,
    page: usize,
    rows: &[ObjectTreeRow],
    selected: &[usize],
    anchor: &mut Option<usize>,
    actions: &mut Vec<Action>,
) {
    if let Some(change) = resolve(click, page, rows, selected, anchor) {
        actions.push(Action::Selection(change));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(indices: &[usize]) -> Vec<ObjectTreeRow> {
        indices
            .iter()
            .map(|&index| ObjectTreeRow::Object { index })
            .collect()
    }

    fn click(index: usize, command: bool, shift: bool, secondary: bool) -> RowClick {
        RowClick {
            index,
            secondary,
            command,
            shift,
        }
    }

    fn ids(v: &[usize]) -> Vec<TargetId> {
        v.iter().map(|&i| TargetId::Object(i as u64)).collect()
    }

    #[test]
    fn ctrl_click_adds_and_removes_one_row() {
        let r = rows(&[0, 1, 2]);
        let mut anchor = None;
        assert_eq!(
            resolve(click(2, true, false, false), 0, &r, &[0], &mut anchor),
            Some(SelectionAction::SetObjects {
                page: 0,
                objects: ids(&[2]),
                add: true
            })
        );
        assert_eq!(
            resolve(click(0, true, false, false), 0, &r, &[0, 2], &mut anchor),
            Some(SelectionAction::RemoveObjects {
                page: 0,
                objects: ids(&[0])
            })
        );
    }

    /// The run follows the tree's order, which layer grouping can make
    /// differ from paint order.
    #[test]
    fn shift_click_selects_the_run_in_tree_order() {
        let r = rows(&[4, 1, 3, 0]);
        let mut anchor = Some(1);
        assert_eq!(
            resolve(click(0, false, true, false), 0, &r, &[1], &mut anchor),
            Some(SelectionAction::SetObjects {
                page: 0,
                objects: ids(&[1, 3, 0]),
                add: false
            })
        );
        assert_eq!(anchor, Some(1), "a Shift+click keeps the anchor");
    }

    #[test]
    fn a_right_click_inside_the_selection_keeps_it() {
        let r = rows(&[0, 1, 2]);
        let mut anchor = None;
        assert_eq!(
            resolve(click(1, false, false, true), 0, &r, &[0, 1], &mut anchor),
            None
        );
        assert_eq!(
            resolve(click(2, false, false, true), 0, &r, &[0, 1], &mut anchor),
            Some(SelectionAction::SelectObject {
                page: 0,
                object: Some(TargetId::Object(2))
            })
        );
    }

    #[test]
    fn a_plain_click_selects_alone_and_clears_the_only_selected_row() {
        let r = rows(&[0, 1]);
        let mut anchor = None;
        assert_eq!(
            resolve(click(1, false, false, false), 0, &r, &[0, 1], &mut anchor),
            Some(SelectionAction::SelectObject {
                page: 0,
                object: Some(TargetId::Object(1))
            })
        );
        assert_eq!(
            resolve(click(1, false, false, false), 0, &r, &[1], &mut anchor),
            Some(SelectionAction::SelectObject {
                page: 0,
                object: None
            })
        );
    }
}
