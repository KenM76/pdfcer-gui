//! `dock::drop` — where a dragged panel may be released, and what releasing it
//! there does to the layout.
//!
//! A pure value grammar over [`DockLayout`]: no `egui::Context`, no pointer, no
//! rect, nothing to open. The overlay that offers these targets previews a drop
//! by cloning the layout, applying the candidate, and resolving the clone — so
//! what the operator is shown mid-drag is produced by the code that performs
//! the drop, rather than by a second description of it that can disagree.
//!
//! ## The take leaves a hole, and that is what keeps the target valid
//!
//! A drop target is named against the layout the operator can see. Removing the
//! dragged panel first can empty its stack, and pruning that stack shifts every
//! later index in its column — so a target resolved before the take may name a
//! different compartment after it. [`DockLayout::move_panel`] therefore removes
//! the tab and leaves its stack and column standing, inserts against the
//! unshifted indices, and lets [`DockLayout::normalize`] prune whatever is left
//! empty. The one boundary that still needs adjusting is a move *within* one
//! stack, where the removal shortens the very list the boundary counts; that
//! case has its own verb, [`DockLayout::reorder_tab`], and is delegated to it.
//!
//! Design and rationale: `docs/modules/egui-shell/dock/drop.md`.

use super::geometry::{ColumnAddr, StackAddr};
use super::model::{Column, DockLayout, DockSide, PanelAddress, PanelId, Stack};

/// **Where a dragged panel would be released.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropTarget {
    /// Join an existing stack, as a tab at boundary `gap` of its tab bar.
    Tab {
        /// The stack to join.
        stack: StackAddr,
        /// The boundary within that stack's tabs.
        gap: usize,
    },
    /// Split a column with a new stack of its own, at boundary `gap`.
    Stack {
        /// The column to split.
        column: ColumnAddr,
        /// The boundary within that column's stacks.
        gap: usize,
    },
    /// Split a side with a new column of its own, at boundary `gap`.
    Column {
        /// The side to split.
        side: DockSide,
        /// The boundary within that side's columns.
        gap: usize,
    },
}

impl DropTarget {
    /// Which side the panel would end up on.
    #[must_use]
    pub const fn side(&self) -> DockSide {
        match self {
            Self::Tab { stack, .. } => stack.side,
            Self::Stack { column, .. } => column.side,
            Self::Column { side, .. } => *side,
        }
    }
}

impl DockLayout {
    /// **Whether this layout could accept a drop at `target`.**
    #[must_use]
    pub fn accepts_drop(&self, target: DropTarget) -> bool {
        match target {
            DropTarget::Tab { stack, gap } => self
                .side(stack.side)
                .columns
                .get(stack.column)
                .and_then(|c| c.stacks.get(stack.stack))
                .is_some_and(|s| gap <= s.tabs.len()),
            DropTarget::Stack { column, gap } => self
                .side(column.side)
                .columns
                .get(column.column)
                .is_some_and(|c| gap <= c.stacks.len()),
            DropTarget::Column { side, gap } => gap <= self.side(side).columns.len(),
        }
    }

    /// **Move `panel` to `target`**, returning whether the layout changed.
    pub fn move_panel(&mut self, panel: &PanelId, target: DropTarget) -> bool {
        let from = self.find(panel);
        if from.is_none() && !self.is_floating(panel) {
            return false;
        }
        if let (Some(from), DropTarget::Tab { stack, gap }) = (from, target)
            && stack == StackAddr::from(from)
        {
            return self.reorder_tab(stack.side, stack.column, stack.stack, from.tab, gap);
        }
        if !self.accepts_drop(target) {
            return false;
        }
        let panel = panel.clone();
        if from.is_some() {
            self.take_panel(&panel);
        } else {
            self.floating.retain(|f| f.panel != panel);
        }
        let inserted = self.insert_at(panel, target);
        debug_assert!(
            inserted,
            "accepts_drop passed and leaving a hole cannot invalidate it"
        );
        self.side_mut(target.side()).visible = true;
        self.normalize();
        true
    }

    /// **Remove `panel` from its stack, leaving that stack and its column in
    /// place** — even when that leaves them empty.
    pub(super) fn take_panel(&mut self, panel: &PanelId) -> Option<PanelAddress> {
        let a = self.find(panel)?;
        let stack = &mut self.side_mut(a.side).columns[a.column].stacks[a.stack];
        stack.tabs.remove(a.tab);
        if stack.active >= a.tab && stack.active > 0 {
            stack.active -= 1;
        }
        Some(a)
    }

    /// Put `panel` where `target` says, reporting whether the target existed.
    fn insert_at(&mut self, panel: PanelId, target: DropTarget) -> bool {
        match target {
            DropTarget::Tab { stack, gap } => {
                let Some(st) = self
                    .side_mut(stack.side)
                    .columns
                    .get_mut(stack.column)
                    .and_then(|c| c.stacks.get_mut(stack.stack))
                else {
                    return false;
                };
                if gap > st.tabs.len() {
                    return false;
                }
                st.tabs.insert(gap, panel);
                st.active = gap;
                true
            }
            DropTarget::Stack { column, gap } => {
                let Some(col) = self.side_mut(column.side).columns.get_mut(column.column) else {
                    return false;
                };
                if gap > col.stacks.len() {
                    return false;
                }
                col.stacks.insert(gap, Stack::new(panel));
                true
            }
            DropTarget::Column { side, gap } => {
                let s = self.side_mut(side);
                if gap > s.columns.len() {
                    return false;
                }
                s.columns.insert(gap, Column::new([Stack::new(panel)]));
                true
            }
        }
    }
}
