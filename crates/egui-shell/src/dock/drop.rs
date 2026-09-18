//! `dock::drop` — where a dragged panel may be released, and what releasing it
//! there does to the layout.
//!
//! A pure value grammar over [`DockLayout`]: no `egui::Context`, no pointer, no
//! rect, nothing to open. The overlay that offers these targets previews a drop
//! by cloning the layout, applying the candidate, and resolving the clone — so
//! what the operator is shown mid-drag is produced by the code that performs
//! the drop, rather than by a second description of it that can disagree.
//!
//! ## ★ The take leaves a hole, and that is what keeps the target valid
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

use super::geometry::{ColumnAddr, StackAddr};
use super::model::{Column, DockLayout, DockSide, PanelAddress, PanelId, Stack};

/// **Where a dragged panel would be released.**
///
/// Every variant names a **boundary**, not a destination index: `0` is before
/// the first item and `len` is after the last. The three variants are the three
/// things a dock can do with a panel — join a group, split a column, or start a
/// column.
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
    ///
    /// Purely a question about the target's indices: whether the compartment it
    /// names exists, and whether the boundary is within range. It says nothing
    /// about which panel is being dropped, so an overlay can ask it once per
    /// candidate zone while laying the zones out.
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
    ///
    /// The one verb behind every drop: a tab dragged onto another tab bar, a
    /// panel dropped against a compartment's edge to split it, and a floating
    /// panel dragged back over the dock all arrive here. It is total — an
    /// unknown panel or an out-of-range target is declined, and **a declined
    /// move mutates nothing**, which is what makes it safe to call with a target
    /// that was resolved against a snapshot a frame old.
    ///
    /// # What a drop does besides moving the panel
    ///
    /// - **The panel becomes the active tab of the stack it joins.** Dropping a
    ///   panel into a group and leaving it behind a sibling's tab is
    ///   indistinguishable, on screen, from the drop having done nothing.
    /// - **The destination side is made visible**, for the same reason
    ///   [`DockLayout::activate`] does it: a panel moved somewhere that draws
    ///   nothing has been hidden, not moved.
    /// - **A new stack or column is created at the default share**, so the
    ///   compartments around it keep their proportions and the newcomer takes an
    ///   even split. The share the panel's old compartment had is not carried
    ///   across, because a share is a weight against *its own* parent's siblings
    ///   and means nothing under a different parent.
    ///
    /// A move within a single stack is a reorder, delegated to
    /// [`DockLayout::reorder_tab`] — which preserves the visible panel by
    /// identity, so rearranging a tab bar does not change what is on screen.
    /// That is also the one case where the boundary is counted in a list the
    /// removal shortens; see the module header.
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
    ///
    /// The half of a close that is not the pruning, shared so the rule for what
    /// becomes active afterwards exists once: removing the active tab selects
    /// the **previous** one, which keeps a run of closes moving leftwards along
    /// the bar instead of marching through tabs the operator has not touched.
    ///
    /// Returns where the panel was, or `None` if it was not docked. The layout
    /// is left un-normalized deliberately — [`DockLayout::close`] normalizes
    /// immediately after calling this, and [`DockLayout::move_panel`] needs the
    /// hole to survive until it has inserted.
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
    ///
    /// Total rather than panicking on a bad index, but not a second gate:
    /// [`DockLayout::move_panel`] has already asked
    /// [`DockLayout::accepts_drop`], and a `false` from here means those two
    /// answers disagree.
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
