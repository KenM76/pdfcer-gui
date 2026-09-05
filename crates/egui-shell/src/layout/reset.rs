//! Resetting a layout — and why it has scopes.
//!
//! # The rule this module exists to obey
//!
//! `RIBBON_IA.md` records the requirement in one sentence, and it is the
//! whole design:
//!
//! > *"an operator who only wanted the right dock back must not lose
//! > their left one."*
//!
//! A single global "reset layout" is a command whose blast radius is
//! always larger than the problem. The operator has made a mess of one
//! dock — usually by dragging a splitter somewhere they cannot drag it
//! back from — and the only remedy on offer destroys the arrangement they
//! spent a month settling on the other side. So they do not use it, and
//! the mess stays.
//!
//! `MODES_AND_PANELS.md`'s peer table makes the same point from the
//! outside: the benchmarked application has **no in-app layout reset at
//! all** (*"the documented route is to quit and delete that file"*), and
//! of the products that do, the one singled out for having got it right
//! is the one with a **two-tier** reset. Failure mode #12 lists in-app
//! reset alongside named workspaces as *table stakes, not luxuries*.
//!
//! # What a reset restores, and what it deliberately does not
//!
//! A reset restores **the arrangement**: columns, stacks, tabs, shares,
//! widths, visibility — for the scope named, and for nothing else.
//!
//! It does **not** touch saved workspaces. That separation is the one
//! judgement call in this module, and it goes this way because the two
//! are different kinds of thing: an arrangement is scratch work, and a
//! named workspace is something the operator deliberately kept. A reset
//! that also emptied the workspace list would be a command whose name
//! promises one thing and whose effect is unrecoverable — and the
//! operator would discover the difference exactly once. An application
//! that wants "forget everything" builds it from
//! [`super::LayoutDocument::delete_workspace`], visibly, as its own
//! command.
//!
//! # Why the default is supplied rather than derived
//!
//! [`reset`] takes the arrangement to reset *to*. The shell has no
//! opinion about what a good starting arrangement is — it does not know
//! what any panel is for — so a `reset()` that invented one would be
//! inventing an application's information architecture. The application
//! passes its own built-in default, which is the same value it uses on a
//! fresh profile and the same value [`super::LayoutDocument::from_ron`]
//! falls back to. One constant, three uses, no drift.

use crate::dock::model::{DockLayout, DockSide};

/// How much of a layout a reset touches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetScope {
    /// Only the left dock. The right dock keeps its arrangement, its
    /// width and its visibility.
    Left,
    /// Only the right dock.
    Right,
    /// Both docks.
    ///
    /// Still not "everything": saved workspaces survive. See the module
    /// header on why that separation is deliberate.
    All,
}

impl ResetScope {
    /// Every scope, in the order a menu should offer them.
    ///
    /// Narrowest first. A destructive command's least destructive form
    /// should be the one nearest the pointer, and the one an operator
    /// reaches by accident.
    pub const ALL: [ResetScope; 3] = [ResetScope::Left, ResetScope::Right, ResetScope::All];

    /// A stable key, for a command id or a diagnostic.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            ResetScope::Left => "left",
            ResetScope::Right => "right",
            ResetScope::All => "all",
        }
    }

    /// Whether this scope touches the given side.
    #[must_use]
    pub const fn covers(self, side: DockSide) -> bool {
        matches!(
            (self, side),
            (ResetScope::All, _)
                | (ResetScope::Left, DockSide::Left)
                | (ResetScope::Right, DockSide::Right)
        )
    }
}

impl std::fmt::Display for ResetScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.key())
    }
}

/// Restore `scope` of `layout` from `default`.
///
/// The sides outside the scope are **not read, not written, and not
/// normalized** — untouched is stronger than unchanged, and it is what
/// makes the guarantee in this module's header checkable by inspection.
///
/// Returns whether anything actually changed, so an application can skip
/// a redundant save and can tell the operator *"that was already the
/// default arrangement"* rather than silently doing nothing.
pub fn reset(layout: &mut DockLayout, scope: ResetScope, default: &DockLayout) -> bool {
    let before = layout.clone();
    for side in DockSide::ALL {
        if scope.covers(side) {
            *layout.side_mut(side) = default.side(side).clone();
        }
    }
    // ★★★ **A reset re-docks every float whose HOME is in scope**, and this
    // is the deterministic recovery for a window nobody can reach.
    //
    // A float has no side, so a scoped reset that only touched sides would
    // leave it floating — and a float window on a monitor that has since
    // been unplugged is a panel with no pointer route to it at all: it
    // cannot be dragged, cannot be closed, and (being in the layout) cannot
    // be floated again. `crate::dock::float::honour_position` is a
    // *heuristic* about that case and says so; this is the part that is
    // not a guess.
    //
    // ★★ Re-docked rather than dropped. The alternative — clearing
    // `floating` and letting the default arrangement decide — loses a panel
    // the operator floated *out of the side they are not resetting*, which
    // would make "reset the left dock" delete something on the right. This
    // way the panel comes back where it came from, and if that side is also
    // in scope the copy from `default` immediately governs anyway.
    //
    // ★ Scoped by HOME rather than by "all floats", so the promise in this
    // module's header — *"the sides outside the scope are not read, not
    // written"* — still holds for the floats too. An operator resetting the
    // right dock keeps the panel they floated out of the left one.
    let homeless: Vec<crate::dock::PanelId> = layout
        .floating
        .iter()
        .filter(|f| scope.covers(f.home.side))
        .map(|f| f.panel.clone())
        .collect();
    for panel in &homeless {
        layout.dock_back(panel);
    }
    // Normalizing only after the copy, and over the whole layout, because
    // a partial reset can duplicate a panel: the default may mount
    // `pages` on the left while the operator had dragged it to the right,
    // and after resetting the left alone it would be in both. The
    // duplicate rule keeps the FIRST mount, which is the freshly reset
    // one — the side the operator just asked to have back.
    layout.normalize();
    *layout != before
}

impl super::LayoutDocument {
    /// Reset the live arrangement. See [`reset`].
    ///
    /// Named workspaces are untouched, deliberately — see this module's
    /// header.
    pub fn reset(&mut self, scope: ResetScope, default: &DockLayout) -> bool {
        reset(&mut self.active, scope, default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::model::{Column, PanelId, SideLayout, Stack};
    use crate::layout::LayoutDocument;

    fn default_layout() -> DockLayout {
        DockLayout::new(
            SideLayout::new([Column::new([Stack::new("pages")])]).with_width(280.0),
            SideLayout::new([Column::new([Stack::new("objects")])]).with_width(300.0),
        )
    }

    /// An arrangement the operator has thoroughly rearranged, on both
    /// sides.
    fn rearranged() -> DockLayout {
        DockLayout::new(
            SideLayout::new([
                Column::new([Stack::tabbed(["pages", "layers"])]),
                Column::new([Stack::new("tools")]),
            ])
            .with_width(520.0),
            SideLayout::new([Column::new([Stack::tabbed(["objects", "properties"])])])
                .with_width(410.0),
        )
    }

    /// ★★★ **Reset recovers a floated panel** — the deterministic half of
    /// the off-screen-window answer.
    ///
    /// `crate::dock::float::honour_position` is a *heuristic* about a
    /// window whose monitor has been unplugged, and its own docs say so.
    /// This is the part that is not a guess: whatever the desktop looks
    /// like, a reset puts the panel back in the dock, where it is on the
    /// same monitor as the application window by construction.
    #[test]
    fn resetting_re_docks_a_floated_panel() {
        let mut layout = rearranged();
        assert!(layout.float(&PanelId::new("layers")));
        // A position on a monitor that no longer exists.
        layout.set_float_geometry(
            &PanelId::new("layers"),
            Some([9000.0, 9000.0]),
            [320.0, 480.0],
        );
        assert!(reset(&mut layout, ResetScope::All, &default_layout()));
        assert!(
            layout.floating.is_empty(),
            "a reset must leave nothing floating, or the unreachable window survives the remedy"
        );
        assert!(layout.is_normalized());
    }

    /// ★★ **A scoped reset only re-docks the floats whose HOME is in
    /// scope.**
    ///
    /// The module header promises that a side outside the scope is *"not
    /// read, not written"*. A float has no side, so without this the
    /// promise would quietly stop covering half the layout: resetting the
    /// left dock would yank back a panel the operator had floated out of
    /// the right one.
    #[test]
    fn a_scoped_reset_leaves_a_float_from_the_other_side_alone() {
        let mut layout = rearranged();
        assert!(
            layout.float(&PanelId::new("objects")),
            "objects is on the right"
        );
        assert!(reset(&mut layout, ResetScope::Left, &default_layout()));
        assert_eq!(
            layout.floating.len(),
            1,
            "resetting the LEFT dock must not re-dock a panel floated from the right"
        );
        assert!(reset(&mut layout, ResetScope::Right, &default_layout()));
        assert!(layout.floating.is_empty(), "resetting the right dock does");
    }

    /// **A reset with nothing floating still reports honestly.**
    ///
    /// The float clause must not make an already-default layout claim it
    /// changed, or an application that saves on `reset`'s return value
    /// writes a file on every press of a command that did nothing.
    #[test]
    fn resetting_an_already_default_layout_with_no_floats_reports_no_change() {
        let mut layout = default_layout();
        assert!(!reset(&mut layout, ResetScope::All, &default_layout()));
    }

    /// ★ **The rule, asserted directly: resetting the right dock leaves
    /// the left one bit-identical.**
    ///
    /// *"An operator who only wanted the right dock back must not lose
    /// their left one."* Equality on the whole `SideLayout` — its
    /// columns, its shares, its width and its visibility — because a
    /// reset that preserved the panels and lost the widths would satisfy
    /// a weaker assertion and still be the defect.
    #[test]
    fn resetting_one_side_leaves_the_other_bit_identical() {
        let mut layout = rearranged();
        let untouched = layout.left.clone();
        assert!(reset(&mut layout, ResetScope::Right, &default_layout()));

        assert_eq!(layout.left, untouched, "the left dock was disturbed");
        assert_eq!(layout.right, default_layout().right);
        assert!((layout.right.width_pts - 300.0).abs() < 0.01);
    }

    /// And symmetrically, because a reset that only works one way round
    /// is a reset with a typo in it.
    #[test]
    fn resetting_the_left_side_leaves_the_right_bit_identical() {
        let mut layout = rearranged();
        let untouched = layout.right.clone();
        assert!(reset(&mut layout, ResetScope::Left, &default_layout()));
        assert_eq!(layout.right, untouched);
        assert_eq!(layout.left, default_layout().left);
    }

    /// `All` restores both sides.
    #[test]
    fn resetting_everything_restores_both_sides() {
        let mut layout = rearranged();
        assert!(reset(&mut layout, ResetScope::All, &default_layout()));
        assert_eq!(layout, default_layout());
    }

    /// Resetting an already-default arrangement reports no change, so an
    /// application can say so rather than pretending it did something.
    #[test]
    fn resetting_an_untouched_layout_reports_no_change() {
        let mut layout = default_layout();
        assert!(!reset(&mut layout, ResetScope::All, &default_layout()));
        assert!(!reset(&mut layout, ResetScope::Left, &default_layout()));
    }

    /// ★ **A partial reset cannot leave a panel mounted twice.**
    ///
    /// The operator dragged `pages` — which the default mounts on the
    /// left — over to the right, then reset the left. Without the
    /// normalization pass the panel would now be in both docks, which is
    /// the state-drift bug [`DockLayout::normalize`] exists to forbid.
    /// The freshly reset side keeps it, which is the side the operator
    /// just asked to have back.
    #[test]
    fn resetting_one_side_cannot_leave_a_panel_in_both() {
        let mut layout = DockLayout::new(
            SideLayout::new([Column::new([Stack::new("tools")])]),
            SideLayout::new([Column::new([Stack::tabbed(["objects", "pages"])])]),
        );
        reset(&mut layout, ResetScope::Left, &default_layout());
        let pages = PanelId::new("pages");
        assert!(
            layout.left.panels().any(|p| *p == pages),
            "the reset side has it"
        );
        assert!(
            !layout.right.panels().any(|p| *p == pages),
            "and the other side no longer does"
        );
        assert_eq!(layout.panels().filter(|p| **p == pages).count(), 1);
    }

    /// ★ **A reset never destroys a named workspace.**
    ///
    /// The judgement call in this module's header, asserted so that a
    /// later "make reset thorough" edit fails a test instead of costing
    /// an operator work they deliberately kept.
    #[test]
    fn a_reset_never_touches_a_saved_workspace() {
        let mut doc = LayoutDocument::new(rearranged());
        doc.save_workspace("Marking up", rearranged());
        doc.reset(ResetScope::All, &default_layout());
        assert_eq!(doc.active, default_layout());
        assert_eq!(doc.workspace_names(), vec!["Marking up"]);
        assert_eq!(doc.workspace("Marking up"), Some(&rearranged()));
    }

    /// Every scope covers exactly the sides its name says.
    #[test]
    fn each_scope_covers_exactly_what_it_names() {
        assert!(ResetScope::Left.covers(DockSide::Left));
        assert!(!ResetScope::Left.covers(DockSide::Right));
        assert!(ResetScope::Right.covers(DockSide::Right));
        assert!(!ResetScope::Right.covers(DockSide::Left));
        assert!(DockSide::ALL.iter().all(|s| ResetScope::All.covers(*s)));
    }

    /// The narrowest scope is offered first — a destructive command's
    /// least destructive form is the one an operator reaches by accident.
    #[test]
    fn the_narrowest_scope_is_offered_first() {
        assert_eq!(ResetScope::ALL[2], ResetScope::All);
        assert_eq!(ResetScope::ALL.len(), 3);
    }
}
