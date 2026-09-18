//! `dock::geometry` — where every compartment was drawn, asked by address.
//!
//! ## What this is for
//!
//! [`super::plan`] is scalar: it resolves spans and plans tab strips and never
//! sees an `egui::Rect`. The rects themselves are locals in [`super::mod`] and
//! [`super::tabs`], and until this module existed the only way one left the
//! dock was [`super::report`]'s stringly-named [`super::RectReport`], whose
//! names a consumer would have to parse back into indices.
//!
//! That is enough for a harness reading a trace. It is not enough for a
//! **gesture**, which has to ask the opposite question: *given this point,
//! which compartment is under it, and where exactly would a panel dropped here
//! land?* Every drag in the dock — reorder, move between stacks, tear out,
//! dock back — is that question and nothing else.
//!
//! ## ★★★ It is filled during the draw and consumed in the same frame
//!
//! A pointer gesture over compartment B is processed while compartment A is
//! being drawn, so a naive implementation reads the rect B had **last** frame.
//! This one does not have to: [`super::Dock::show`] accumulates into a
//! `DockGeometry` as it draws, and anything that needs the whole picture —
//! the drop overlay, the release settlement — runs after both sides are drawn
//! and therefore reads a complete, current record.
//!
//! The finished record is then kept on [`super::DockState`] for two consumers
//! that genuinely run later: [`super::Dock::show_floating`], which is a second
//! call at a different point in the frame, and a test or harness asking where
//! something was. Those two read a record one frame old, and the distinction
//! matters because a rect that is one frame old is indistinguishable from a
//! current one by inspection. It is rebuilt from nothing every frame, so it
//! never holds a **fossil** — a rect for something that has stopped being
//! drawn — which is the failure mode a change-log rect trace has and this does
//! not.
//!
//! ## What is deliberately not here
//!
//! No clipping and no reachability. A rect in this record says *"this is where
//! the compartment was laid out"*, which is exactly what
//! `D:\dev\rag\egui\a_dock_rect_stream_reports_layout_and_a_reachability_claim_needs_the_clip_beside_it.md`
//! warns is not the same as *"the operator can reach it"*. A drop target must
//! be a laid-out rect — the pointer is over it, so it is on screen by
//! construction — but nothing here should be read as a reachability claim, and
//! `report`'s clipped channel remains the surface that answers that.

use egui::Rect;

use super::model::{DockSide, PanelAddress};

/// Which stack, by position — the three-index prefix of a
/// [`PanelAddress`].
///
/// Positional for [`PanelAddress`]'s reason: this model has no stable handle
/// for a stack and deliberately does not want one. The consequence is the same
/// one [`super::float::DockHome`] carries — an address can go stale across an
/// edit — and the answer is the same: addresses are used **within** a frame, or
/// rebuilt against the layout before they are trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackAddr {
    /// Which dock side.
    pub side: DockSide,
    /// Index of the column within the side.
    pub column: usize,
    /// Index of the stack within the column.
    pub stack: usize,
}

impl StackAddr {
    /// Build one.
    #[must_use]
    pub const fn new(side: DockSide, column: usize, stack: usize) -> Self {
        Self {
            side,
            column,
            stack,
        }
    }

    /// The tab at `tab` in this stack.
    #[must_use]
    pub const fn tab(self, tab: usize) -> PanelAddress {
        PanelAddress {
            side: self.side,
            column: self.column,
            stack: self.stack,
            tab,
        }
    }
}

impl From<PanelAddress> for StackAddr {
    fn from(a: PanelAddress) -> Self {
        Self::new(a.side, a.column, a.stack)
    }
}

/// Which column, by position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColumnAddr {
    /// Which dock side.
    pub side: DockSide,
    /// Index of the column within the side.
    pub column: usize,
}

impl ColumnAddr {
    /// Build one.
    #[must_use]
    pub const fn new(side: DockSide, column: usize) -> Self {
        Self { side, column }
    }
}

/// Where every compartment of the dock was drawn.
///
/// Rebuilt from nothing every frame. See the module header for what it is for,
/// when it is current, and what it deliberately does not claim.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DockGeometry {
    sides: Vec<(DockSide, Rect)>,
    columns: Vec<(ColumnAddr, Rect)>,
    stacks: Vec<(StackAddr, Rect)>,
    strips: Vec<(StackAddr, Rect)>,
    tabs: Vec<(PanelAddress, Rect)>,
}

impl DockGeometry {
    /// Whether anything was recorded at all.
    ///
    /// True on the first frame, and on any frame where both sides are empty or
    /// collapsed. A caller that hit-tests an empty geometry gets `None`
    /// everywhere, which is the correct answer rather than a special case.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stacks.is_empty()
    }

    /// Record a whole side's rect.
    pub(super) fn push_side(&mut self, side: DockSide, rect: Rect) {
        self.sides.push((side, rect));
    }

    /// Record a column's rect.
    pub(super) fn push_column(&mut self, addr: ColumnAddr, rect: Rect) {
        self.columns.push((addr, rect));
    }

    /// Record a stack's whole compartment — its tab strip and its body.
    pub(super) fn push_stack(&mut self, addr: StackAddr, rect: Rect) {
        self.stacks.push((addr, rect));
    }

    /// Record a stack's tab strip alone.
    pub(super) fn push_strip(&mut self, addr: StackAddr, rect: Rect) {
        self.strips.push((addr, rect));
    }

    /// Record one drawn tab.
    ///
    /// Only tabs that were **actually drawn** are recorded: a tab in the
    /// overflow menu has no rect on the strip, and inventing one from its index
    /// and width would be deriving a coordinate the layout already knows — the
    /// rule [`crate::tabstrip::TabStrip::drawn`] carries, for the reason that a
    /// harness computing a position from an index can be wrong in the same
    /// direction as the code under test.
    pub(super) fn push_tab(&mut self, addr: PanelAddress, rect: Rect) {
        self.tabs.push((addr, rect));
    }

    /// Which side is under `pos`, if either.
    #[must_use]
    pub fn side_at(&self, pos: egui::Pos2) -> Option<DockSide> {
        hit(&self.sides, pos)
    }

    /// Which column is under `pos`.
    #[must_use]
    pub fn column_at(&self, pos: egui::Pos2) -> Option<ColumnAddr> {
        hit(&self.columns, pos)
    }

    /// Which stack's compartment is under `pos`.
    #[must_use]
    pub fn stack_at(&self, pos: egui::Pos2) -> Option<StackAddr> {
        hit(&self.stacks, pos)
    }

    /// Which stack's **tab strip** is under `pos`.
    ///
    /// Separate from [`Self::stack_at`] because the two answer different
    /// questions for a drop: over the strip means *"between these two tabs"*,
    /// over the body means *"into this group, or splitting it"*.
    #[must_use]
    pub fn strip_at(&self, pos: egui::Pos2) -> Option<StackAddr> {
        hit(&self.strips, pos)
    }

    /// Which tab is under `pos`.
    #[must_use]
    pub fn tab_at(&self, pos: egui::Pos2) -> Option<PanelAddress> {
        hit(&self.tabs, pos)
    }

    /// Where a stack's compartment was drawn.
    #[must_use]
    pub fn stack_rect(&self, addr: StackAddr) -> Option<Rect> {
        rect_of(&self.stacks, addr)
    }

    /// Where a stack's tab strip was drawn.
    #[must_use]
    pub fn strip_rect(&self, addr: StackAddr) -> Option<Rect> {
        rect_of(&self.strips, addr)
    }

    /// Where one tab was drawn, if it was drawn at all.
    #[must_use]
    pub fn tab_rect(&self, addr: PanelAddress) -> Option<Rect> {
        rect_of(&self.tabs, addr)
    }

    /// Every drawn tab of one stack, in the order they were drawn.
    pub fn tabs_of(&self, addr: StackAddr) -> impl Iterator<Item = (usize, Rect)> + '_ {
        self.tabs
            .iter()
            .filter_map(move |(a, r)| (StackAddr::from(*a) == addr).then_some((a.tab, *r)))
    }

    /// **Which boundary of `addr`'s strip the point `pos` names.**
    ///
    /// `0` is before the first drawn tab and `n` is after the last, which is the
    /// vocabulary an insertion caret is drawn in and the one
    /// [`crate::tabstrip::TabIntent::Reorder`] and the dock's own reorder
    /// intent both use. It is deliberately not
    /// *"the index it ends up at"*: those differ by one whenever a tab moves
    /// rightward, because the tab is removed before it is re-inserted, and a
    /// caller that got the convention wrong would be off by one in one
    /// direction only — the hardest kind of off-by-one to see.
    ///
    /// ## ★ Resolved by CENTRES, not by edges
    ///
    /// A tab whose centre is left of the pointer is a tab the dragged one has
    /// passed. The boundary therefore flips when the pointer crosses the middle
    /// of a neighbour, which is what every tab strip on this desktop does, and
    /// which is what stops the caret jittering between two gaps while the
    /// pointer rests on the seam between two tabs.
    ///
    /// ## ★ Seeded from the FIRST DRAWN tab, not from zero
    ///
    /// A strip whose leading tabs are in the overflow menu starts at a non-zero
    /// index. Seeding at zero would let a pointer at the left edge report a
    /// boundary left of everything on screen — a caret drawn in one place and a
    /// move committed somewhere else.
    ///
    /// `None` when the stack drew no tabs at all.
    #[must_use]
    pub fn gap_in(&self, addr: StackAddr, pos: egui::Pos2) -> Option<usize> {
        let mut gap = None;
        for (i, r) in self.tabs_of(addr) {
            if gap.is_none() {
                gap = Some(i);
            }
            if pos.x > r.center().x {
                gap = Some(i + 1);
            }
        }
        gap
    }
}

/// The address whose rect contains `pos`, if any.
///
/// Last match wins: compartments are recorded in draw order and do not overlap,
/// so the choice is immaterial for stacks — but a later entry is the one drawn
/// on top, which is the answer a pointer gesture wants if that ever changes.
fn hit<A: Copy>(entries: &[(A, Rect)], pos: egui::Pos2) -> Option<A> {
    entries
        .iter()
        .rev()
        .find(|(_, r)| r.contains(pos))
        .map(|(a, _)| *a)
}

/// The rect recorded for one address.
fn rect_of<A: Copy + PartialEq>(entries: &[(A, Rect)], addr: A) -> Option<Rect> {
    entries.iter().find(|(a, _)| *a == addr).map(|(_, r)| *r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    /// A strip of `n` tabs, each 100 wide and 20 tall, starting at x = 0.
    fn strip_of(n: usize, first: usize) -> DockGeometry {
        let addr = StackAddr::new(DockSide::Left, 0, 0);
        let mut g = DockGeometry::default();
        g.push_stack(
            addr,
            Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 400.0)),
        );
        g.push_strip(addr, Rect::from_min_size(pos2(0.0, 0.0), vec2(300.0, 20.0)));
        for k in 0..n {
            #[allow(clippy::cast_precision_loss)]
            let x = k as f32 * 100.0;
            g.push_tab(
                addr.tab(first + k),
                Rect::from_min_size(pos2(x, 0.0), vec2(100.0, 20.0)),
            );
        }
        g
    }

    #[test]
    fn an_empty_geometry_hits_nothing_rather_than_panicking() {
        let g = DockGeometry::default();
        assert!(g.is_empty());
        assert_eq!(g.stack_at(pos2(10.0, 10.0)), None);
        assert_eq!(g.tab_at(pos2(10.0, 10.0)), None);
        assert_eq!(
            g.gap_in(StackAddr::new(DockSide::Left, 0, 0), pos2(0.0, 0.0)),
            None
        );
    }

    #[test]
    fn a_point_inside_a_stack_names_that_stack() {
        let g = strip_of(3, 0);
        let addr = StackAddr::new(DockSide::Left, 0, 0);
        assert_eq!(g.stack_at(pos2(150.0, 200.0)), Some(addr));
        assert_eq!(g.stack_at(pos2(400.0, 200.0)), None);
        assert_eq!(g.strip_at(pos2(150.0, 10.0)), Some(addr));
        // Below the strip is the body, not the strip.
        assert_eq!(g.strip_at(pos2(150.0, 200.0)), None);
    }

    #[test]
    fn a_point_over_a_tab_names_that_tab() {
        let g = strip_of(3, 0);
        let addr = StackAddr::new(DockSide::Left, 0, 0);
        assert_eq!(g.tab_at(pos2(50.0, 10.0)), Some(addr.tab(0)));
        assert_eq!(g.tab_at(pos2(150.0, 10.0)), Some(addr.tab(1)));
        assert_eq!(g.tab_at(pos2(250.0, 10.0)), Some(addr.tab(2)));
        assert_eq!(g.tab_at(pos2(350.0, 10.0)), None);
    }

    /// ★ The whole contract of [`DockGeometry::gap_in`] in one sweep: the
    /// boundary flips at a tab's CENTRE, and the ends are reachable.
    #[test]
    fn the_gap_flips_at_each_tabs_centre() {
        let g = strip_of(3, 0);
        let addr = StackAddr::new(DockSide::Left, 0, 0);
        let gap = |x: f32| g.gap_in(addr, pos2(x, 10.0));
        assert_eq!(gap(0.0), Some(0), "left of everything");
        assert_eq!(gap(49.0), Some(0), "left half of tab 0");
        assert_eq!(gap(51.0), Some(1), "right half of tab 0");
        assert_eq!(gap(149.0), Some(1), "left half of tab 1");
        assert_eq!(gap(151.0), Some(2), "right half of tab 1");
        assert_eq!(gap(251.0), Some(3), "right half of the last tab");
        assert_eq!(gap(999.0), Some(3), "past the end is still the end");
    }

    /// ★ The seeding rule. With tabs 0 and 1 in the overflow menu, the leftmost
    /// boundary a pointer can name is 2 — not 0, which would be a caret drawn
    /// on screen and a move committed off it.
    #[test]
    fn a_strip_whose_leading_tabs_overflowed_cannot_report_a_gap_left_of_the_screen() {
        let g = strip_of(2, 2);
        let addr = StackAddr::new(DockSide::Left, 0, 0);
        assert_eq!(g.gap_in(addr, pos2(0.0, 10.0)), Some(2));
        assert_eq!(g.gap_in(addr, pos2(51.0, 10.0)), Some(3));
        assert_eq!(g.gap_in(addr, pos2(151.0, 10.0)), Some(4));
    }

    #[test]
    fn a_stack_reports_only_its_own_tabs() {
        let mut g = strip_of(2, 0);
        let other = StackAddr::new(DockSide::Right, 0, 0);
        g.push_tab(
            other.tab(0),
            Rect::from_min_size(pos2(500.0, 0.0), vec2(100.0, 20.0)),
        );
        let mine = StackAddr::new(DockSide::Left, 0, 0);
        assert_eq!(g.tabs_of(mine).count(), 2);
        assert_eq!(g.tabs_of(other).count(), 1);
        assert_eq!(g.tab_rect(other.tab(0)).map(|r| r.left()), Some(500.0));
    }
}
