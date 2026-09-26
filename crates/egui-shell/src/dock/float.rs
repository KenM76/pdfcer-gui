//! # `dock::float` — a panel torn out of the dock into a window of its own
//!
//! This module owns the **value** and the **state machine**. Nothing here
//! draws anything and nothing here touches `egui::Context`; the window is
//! [`super::floatwin`]'s subject. The split is the same one
//! [`super::plan`] makes against [`super::mod`]: a rule you can test with
//! no window open is worth more than a rule you can only watch.
//!
//! ## The gesture is a command, not a drag
//!
//! `MODES_AND_PANELS.md`'s tear-out section states the reasoning:
//! starting from a stationary command rather than a drag captures most of
//! the value at a fraction of the cost, dodges the focus-gated `StartDrag`
//! primitive, and sidesteps failure mode #1 (ambiguous drag handle), the
//! most-reported docking complaint in the benchmarked product.
//!
//! A drag-to-tear gesture can be layered on top of this model without
//! changing a byte of it, because the model's question is *"where is this
//! panel and where did it come from"* and a drag is only one more way of
//! answering it.
//!
//! [`super::tab_menu`]'s own header names `"Float this panel"` as the
//! canonical example of a verb *"the shell cannot know"* — and it is
//! right, which is why nothing in this module knows the words. The shell
//! owns the **mechanism**; the application owns the row that raises it.
//!
//! ## The one decision the rest of this module follows from
//!
//! > **A floated panel remembers where it came from, and docking it back
//! > puts it there.**
//!
//! The alternative — dock it back "somewhere sensible", i.e. wherever
//! [`super::DockLayout::mount`]'s permissive clamp lands it — is what
//! every product in `MODES_AND_PANELS.md`'s benchmark table does, and it
//! is why re-docking in those products is a thing operators avoid. A
//! panel that came from the second column of the left dock and returns to
//! the first column of whatever side happens to have room has not been
//! docked; it has been re-mounted, and the operator's arrangement has
//! been quietly edited by a command whose whole promise was to put
//! something back.
//!
//! So [`FloatingPanel`] carries a [`DockHome`], the home is recorded at
//! the instant of the float, it is **serialized with the rest of the
//! layout**, and it survives a restart. That last clause is the one that
//! costs something — see "Why the home is an address and not a handle".
//!
//! ## Why the home is an ADDRESS and not a handle
//!
//! `DockHome` is four `usize`s. It is emphatically **not** a stable
//! identity for a stack, and it cannot be, because
//! [`super::DockLayout`] has no stable identity for a stack — a column is
//! whatever index it currently occupies, which is the same property that
//! makes an arena handle unusable for persistence —
//! `MODES_AND_PANELS.md` §"It is built on `egui` directly, not on
//! `egui_tiles`" carries that argument in full.
//!
//! An address can therefore go stale: the operator floats the Layers
//! panel out of `left[1][0]`, then closes every other panel in column 1,
//! and [`super::DockLayout::normalize`] prunes the column. The address
//! now names a column that does not exist.
//!
//! ⇒ **That is handled by rebuilding, not by refusing and not by
//! clamping.** [`DockLayout::dock_back`] inserts a column at
//! `home.column` and a stack at `home.stack` when they are missing, and
//! clamps only the parts that are genuinely past the end. A stale home
//! therefore lands the panel in a compartment of its own as near its old
//! place as the arrangement allows, and the operator gets their panel
//! back **with its compartment**.
//!
//! Delegating to [`super::DockLayout::mount`] and its permissive
//! clamp instead reads correct and is not: **floating a panel that was
//! alone in its stack prunes that stack**, so the home is out of range one
//! frame later with nothing having moved on, and the clamp merges the
//! panel into its neighbour without saying so.
//! [`tests::float_then_dock_puts_the_panel_back_where_it_was`] is the
//! guard, and the general rule is that *a tolerant fallback hides the case
//! it was not written for*.
//!
//! Refusing — a `Result` — was never the alternative. It would push a
//! decision into every caller that none of them can make better than this
//! one, and the honest answer at every one of those call sites would be
//! "put it back anyway".
//!
//! ## The state machine, in full
//!
//! Four states, and every edge is listed because the ones that are
//! *missing* are as load-bearing as the ones that are here.
//!
//! ```text
//!                      float()
//!        ┌──────────┐ ─────────► ┌──────────┐
//!        │  DOCKED  │            │ FLOATING │
//!        └──────────┘ ◄───────── └──────────┘
//!             │        dock_back()     │
//!             │ close()                │ close()
//!             ▼                        ▼
//!        ┌──────────────────────────────────┐
//!        │            ABSENT                │
//!        │  (not in the layout at all)      │
//!        └──────────────────────────────────┘
//!                        │
//!                        │ DockLayout::mount()  ← View ▸ Panels
//!                        ▼
//!                     DOCKED
//! ```
//!
//! | From | Verb | To | Note |
//! |---|---|---|---|
//! | Docked | [`DockLayout::float`] | Floating | Home recorded from the address it had. |
//! | Floating | [`DockLayout::dock_back`] | Docked | The home is rebuilt, not clamped into. |
//! | Docked | `close` | Absent | [`super::DockLayout::close`]; prunes what it empties. |
//! | Floating | `close` | Absent | Same verb — see below. |
//!
//! | Absent | `mount` | Docked | The application's View ▸ Panels group. |
//! | Absent | — | Floating | **Deliberately not an edge.** |
//!
//! **The last row is a decision.** "Open this panel" from a menu always
//! opens it *docked*, even if the last thing the operator did was float
//! it and then close the window. Two reasons, and the second is the one
//! that decides it:
//!
//! 1. A window that appears on a monitor the operator is not looking at
//!    is indistinguishable from a command that did nothing. A docked
//!    panel appears inside the window they are already looking at.
//! 2. **There is nowhere to remember it.** Closing removes the panel from
//!    the layout entirely, including its float entry — so a
//!    "reopen floating" would have to keep a fourth, invisible state
//!    (*absent, but was floating, at this position*) whose only observable
//!    effect is a surprise. `MODES_AND_PANELS.md` failure mode #11 is the
//!    general form: a surface that reopens showing something other than
//!    what the operator last saw is read as a defect.
//!
//! **Closing is one verb for both states**, and that is also a
//! decision. It would be easy to give a float window a "close" that means
//! *dock it back and then close it*, so that a later reopen finds the old
//! home. It is rejected: the operator pressed a close button on a window,
//! and a close that silently rearranges the dock behind that window is a
//! side effect nothing announced. Close means gone; [`dock_back`] means
//! back.
//!
//! ## What a float remembers, and where
//!
//! | Fact | Where | Survives restart |
//! |---|---|---|
//! | that it is floating | [`super::DockLayout::floating`] | ✅ |
//! | its home | [`DockHome`] | ✅ |
//! | its size | [`FloatingPanel::size_pts`] | ✅ |
//! | its desktop position | [`FloatingPanel::pos_pts`] | ✅ |
//! | which monitor that is | **nothing** | ❌ — see [`honour_position`] |
//!
//! All four persisted facts ride in the same `layout.ron` the docked
//! width already rides in, under the same per-mode workspace key, saved by
//! the same debounce on the same [`super::DockFrameReport::layout_changed`]
//! signal. **No new file, no new key, no new save path** — which is the
//! property that makes "it remembers, per mode, the way docked width
//! already does" true by construction rather than by a second
//! implementation that has to be kept in step.
//!
//! Design and rationale: `docs/modules/egui-shell/dock/float.md`.

use serde::{Deserialize, Serialize};

use super::model::{DockLayout, DockSide, PanelId};

/// The size a panel floats at when it has never been floated before.
pub const DEFAULT_SIZE_PTS: [f32; 2] = [320.0, 480.0];

/// The smallest a float window may be remembered at.
pub const MIN_SIZE_PTS: [f32; 2] = [200.0, 140.0];

/// The largest a float window may be remembered at.
pub const MAX_SIZE_PTS: [f32; 2] = [4000.0, 4000.0];

/// How far each successive float is offset from the one before, when
/// neither has a remembered position.
pub const CASCADE_PTS: f32 = 28.0;

/// How far in from the application window the first unremembered float
/// opens.
pub const OPEN_INSET_PTS: f32 = 48.0;

/// How many monitor-widths from the application window a remembered
/// position may be before it is treated as unremembered.
pub const OFFSCREEN_REACH_MONITORS: f32 = 3.0;

/// The reach used when the platform has not told us how big the
/// application's monitor is.
pub const OFFSCREEN_REACH_FALLBACK_PTS: f32 = 4000.0;

/// Where a floated panel came from, so it can be put back there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockHome {
    /// The side it was docked on.
    pub side: DockSide,
    /// The column index within that side, at the moment it was floated.
    pub column: usize,
    /// The stack index within that column, at the moment it was floated.
    pub stack: usize,
    /// The tab index within that stack, at the moment it was floated.
    ///
    /// Honoured by [`DockLayout::dock_back`], clamped to the stack's
    /// current length. It is what stops floating the *first* of three
    /// tabbed panels and docking it back from silently reordering the
    /// other two — see that method's docs for why appending is not good
    /// enough.
    pub tab: usize,
}

impl DockHome {
    /// The home a panel that has never been docked would have.
    #[must_use]
    pub const fn origin(side: DockSide) -> Self {
        Self {
            side,
            column: 0,
            stack: 0,
            tab: 0,
        }
    }
}

/// One panel that is not in either dock, because it is in a window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloatingPanel {
    /// Which panel. The application's opaque id, exactly as in a stack.
    pub panel: PanelId,
    /// Where it was docked when it was floated. See [`DockHome`].
    pub home: DockHome,
    /// Where the operator last left the window, in **desktop** points.
    ///
    /// `None` means *"has not been placed yet"*, and that is a different
    /// thing from `Some([0.0, 0.0])` — which is a real position, the
    /// top-left corner of the primary monitor, and somewhere a window can
    /// legitimately be. Collapsing the two would make "the operator
    /// dragged it to the corner" and "we have never seen this window"
    /// the same value, which is the exact mistake
    /// `dialogs::host::placement` records under A16c.
    ///
    /// Desktop points and not application-window points, because that is
    /// what `egui::ViewportBuilder::with_position` speaks and because a
    /// float is *allowed* to be outside the application window — that
    /// being most of the reason to float one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pos_pts: Option<[f32; 2]>,
    /// How big the window is, in points. Always present: a window has a
    /// size from the moment it exists, and [`DEFAULT_SIZE_PTS`] is the
    /// answer before that.
    pub size_pts: [f32; 2],
}

impl FloatingPanel {
    /// A fresh float of `panel`, from `home`, at the default size and with
    /// no remembered position.
    #[must_use]
    pub fn new(panel: PanelId, home: DockHome) -> Self {
        Self {
            panel,
            home,
            pos_pts: None,
            size_pts: DEFAULT_SIZE_PTS,
        }
    }
}

impl DockLayout {
    /// Whether `panel` is floating.
    #[must_use]
    pub fn is_floating(&self, panel: &PanelId) -> bool {
        self.floating.iter().any(|f| &f.panel == panel)
    }

    /// The float entry for `panel`, if it is floating.
    #[must_use]
    pub fn float_of(&self, panel: &PanelId) -> Option<&FloatingPanel> {
        self.floating.iter().find(|f| &f.panel == panel)
    }

    /// **Tear `panel` out of the dock into a window of its own.**
    pub fn float(&mut self, panel: &PanelId) -> bool {
        let Some(at) = self.find(panel) else {
            return false;
        };
        let home = DockHome {
            side: at.side,
            column: at.column,
            stack: at.stack,
            tab: at.tab,
        };
        // `close` is the right verb even though this is not a close: it is
        // the one place that removes a tab AND prunes what the removal
        // empties, and a float that left a dead one-tall column behind
        // would be the "dead column" failure by another route.
        self.close(panel);
        self.floating.push(FloatingPanel::new(panel.clone(), home));
        true
    }

    /// **Tear `panel` out into a window at a position the operator chose.**
    pub fn float_at(&mut self, panel: &PanelId, at: [f32; 2]) -> bool {
        if !self.float(panel) {
            return false;
        }
        // The size is the one `float` just gave it. Passing it back through
        // rather than reading it is what keeps this from being a second
        // opinion about how big a fresh float is.
        let size = self
            .float_of(panel)
            .map_or(DEFAULT_SIZE_PTS, |f| f.size_pts);
        self.set_float_geometry(panel, Some(at), size);
        true
    }

    /// **Put a floating panel back where it came from.**
    pub fn dock_back(&mut self, panel: &PanelId) -> bool {
        let Some(i) = self.floating.iter().position(|f| &f.panel == panel) else {
            return false;
        };
        let home = self.floating.remove(i).home;
        let side = self.side_mut(home.side);
        // Rebuild, do not clamp. Each `min` is the genuine
        // beyond-the-end case: a home column of 4 in a side that now has
        // one column becomes column 1, i.e. "a new column at the end",
        // which is as near as the arrangement allows.
        let ci = home.column.min(side.columns.len());
        if ci == side.columns.len() {
            side.columns.push(super::model::Column::default());
        }
        let column = &mut side.columns[ci];
        let si = home.stack.min(column.stacks.len());
        if si == column.stacks.len() {
            column.stacks.push(super::model::Stack::default());
        }
        let stack = &mut column.stacks[si];
        let ti = home.tab.min(stack.tabs.len());
        stack.tabs.insert(ti, panel.clone());
        // The side must be visible, or the panel has been docked into a
        // collapsed compartment and the window simply vanished. This is
        // what `activate` does for the docked case, and it is why the call
        // below is not merely cosmetic.
        self.activate(panel);
        true
    }

    /// **Put every floating panel back**, and say how many there were.
    pub fn dock_all_floating(&mut self) -> usize {
        let panels: Vec<PanelId> = self.floating.iter().map(|f| f.panel.clone()).collect();
        for panel in &panels {
            self.dock_back(panel);
        }
        panels.len()
    }

    /// Record where the operator has left a float window.
    pub fn set_float_geometry(
        &mut self,
        panel: &PanelId,
        pos_pts: Option<[f32; 2]>,
        size_pts: [f32; 2],
    ) -> bool {
        let Some(f) = self.floating.iter_mut().find(|f| &f.panel == panel) else {
            return false;
        };
        let pos = pos_pts.filter(|p| p[0].is_finite() && p[1].is_finite());
        let size = clamp_size(size_pts);
        if f.pos_pts == pos && f.size_pts == size {
            return false;
        }
        f.pos_pts = pos;
        f.size_pts = size;
        true
    }
}

/// Pull a stored size back inside [`MIN_SIZE_PTS`] and [`MAX_SIZE_PTS`],
/// answering [`DEFAULT_SIZE_PTS`] for anything that is not a number.
#[must_use]
pub fn clamp_size(size: [f32; 2]) -> [f32; 2] {
    let axis = |v: f32, min: f32, max: f32, fallback: f32| {
        if v.is_finite() {
            v.clamp(min, max)
        } else {
            fallback
        }
    };
    [
        axis(
            size[0],
            MIN_SIZE_PTS[0],
            MAX_SIZE_PTS[0],
            DEFAULT_SIZE_PTS[0],
        ),
        axis(
            size[1],
            MIN_SIZE_PTS[1],
            MAX_SIZE_PTS[1],
            DEFAULT_SIZE_PTS[1],
        ),
    ]
}

/// Repair the float list: drop anything that is also docked, drop
/// duplicates, and clamp every stored size.
pub(super) fn normalize_floats(layout: &mut DockLayout) {
    let mut seen: std::collections::BTreeSet<PanelId> = std::collections::BTreeSet::new();
    // Collected first because the retain closure below needs to ask
    // whether the panel is in the tree, and the tree is behind the same
    // `&mut` as the list being retained.
    let docked: std::collections::BTreeSet<PanelId> = DockSide::ALL
        .into_iter()
        .flat_map(|s| layout.side(s).panels())
        .cloned()
        .collect();
    layout
        .floating
        .retain(|f| !docked.contains(&f.panel) && seen.insert(f.panel.clone()));
    for f in &mut layout.floating {
        f.size_pts = clamp_size(f.size_pts);
        f.pos_pts = f.pos_pts.filter(|p| p[0].is_finite() && p[1].is_finite());
    }
}

/// **Should a remembered position be honoured, or has the desktop it
/// described stopped existing?**
#[must_use]
pub fn honour_position(
    remembered: Option<[f32; 2]>,
    size: [f32; 2],
    app_outer: egui::Rect,
    monitor: Option<egui::Vec2>,
) -> Option<egui::Pos2> {
    let at = remembered?;
    if !at[0].is_finite() || !at[1].is_finite() {
        return None;
    }
    let size = clamp_size(size);
    let reach = match monitor {
        Some(m) if m.x.is_finite() && m.y.is_finite() && m.x > 0.0 && m.y > 0.0 => {
            egui::vec2(m.x, m.y) * OFFSCREEN_REACH_MONITORS
        }
        _ => egui::Vec2::splat(OFFSCREEN_REACH_FALLBACK_PTS),
    };
    let envelope = app_outer.expand2(reach);
    let window = egui::Rect::from_min_size(egui::pos2(at[0], at[1]), egui::vec2(size[0], size[1]));
    // `intersects` and not `contains`: a window half off the envelope's
    // edge is still a window the operator can grab, and requiring full
    // containment would drop a legitimately-placed float whose far corner
    // happens to run past a bound that is itself a guess.
    envelope.intersects(window).then_some(window.min)
}

/// Where the `nth` unremembered float of this frame should open.
#[must_use]
pub fn opening_position(app_outer: egui::Rect, nth: usize) -> egui::Pos2 {
    let step = CASCADE_PTS * (nth.min(CASCADE_MAX) as f32);
    app_outer.min + egui::Vec2::splat(OPEN_INSET_PTS + step)
}

/// How many steps the cascade takes before it stops moving.
pub const CASCADE_MAX: usize = 8;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::model::{Column, SideLayout, Stack};

    /// Two columns on the left (two stacks in the first), one on the right.
    fn sample() -> DockLayout {
        DockLayout::new(
            SideLayout::new([
                Column::new([Stack::tabbed(["pages", "bookmarks"]), Stack::new("layers")]),
                Column::new([Stack::new("tools")]),
            ]),
            SideLayout::new([Column::new([Stack::tabbed(["objects", "properties"])])]),
        )
    }

    fn id(s: &str) -> PanelId {
        PanelId::new(s)
    }

    /// **Floating takes the panel out of the dock and records where it
    /// was.**
    #[test]
    fn floating_records_the_home_it_came_from() {
        let mut l = sample();
        assert!(l.float(&id("layers")));
        assert!(!l.contains_docked(&id("layers")), "still in a stack");
        assert!(l.is_floating(&id("layers")));
        let home = l.float_of(&id("layers")).expect("a float entry").home;
        assert_eq!(
            (home.side, home.column, home.stack, home.tab),
            (DockSide::Left, 0, 1, 0),
            "the home must be the address it had BEFORE the removal pruned anything"
        );
    }

    /// **Float, then dock, is the identity on the arrangement.**
    #[test]
    fn float_then_dock_puts_the_panel_back_where_it_was() {
        let before = sample();
        let mut l = before.clone();
        assert!(l.float(&id("layers")));
        assert!(l.dock_back(&id("layers")));
        assert_eq!(
            l.find(&id("layers")),
            before.find(&id("layers")),
            "docking back must restore the address, not merely the membership"
        );
        assert!(l.floating.is_empty());
    }

    /// **A float that came from a stack with siblings goes back into that
    /// stack**, rather than making a new one beside it.
    #[test]
    fn a_float_rejoins_the_stack_it_was_tabbed_into() {
        let mut l = sample();
        l.float(&id("bookmarks"));
        assert_eq!(
            l.side(DockSide::Left).columns[0].stacks[0].tabs,
            vec![id("pages")],
            "the sibling stays put"
        );
        l.dock_back(&id("bookmarks"));
        assert_eq!(
            l.side(DockSide::Left).columns[0].stacks[0].tabs,
            vec![id("pages"), id("bookmarks")],
            "it must rejoin the same stack, appended"
        );
    }

    /// **Docking back makes the panel the active tab.**
    #[test]
    fn docking_back_brings_the_panel_to_the_front_of_its_stack() {
        let mut l = sample();
        l.float(&id("bookmarks"));
        l.dock_back(&id("bookmarks"));
        assert!(
            l.is_active(&id("bookmarks")),
            "a docked-back panel must be the tab you are looking at"
        );
    }

    /// **Floating the last panel of a column prunes the column, and
    /// docking it back rebuilds one.**
    ///
    /// The "dead column" case, reached by the float path rather than by
    /// the close path. `tools` is alone in the left dock's second column.
    #[test]
    fn floating_the_only_panel_in_a_column_leaves_no_dead_column() {
        let mut l = sample();
        assert_eq!(l.side(DockSide::Left).columns.len(), 2);
        l.float(&id("tools"));
        assert_eq!(
            l.side(DockSide::Left).columns.len(),
            1,
            "the emptied column must be pruned, not left as a blank strip"
        );
        assert!(l.is_normalized());
        l.dock_back(&id("tools"));
        assert!(
            l.contains_docked(&id("tools")),
            "a stale home must still land the panel somewhere reachable"
        );
        assert!(l.is_normalized());
    }

    /// **A stale home does not lose the panel.**
    #[test]
    fn a_home_that_no_longer_exists_still_docks_the_panel() {
        let mut l = sample();
        l.float(&id("layers"));
        // Empty the whole left side except the column `layers` came from,
        // then empty that too.
        for p in ["pages", "bookmarks", "tools"] {
            l.close(&id(p));
        }
        assert!(l.side(DockSide::Left).columns.is_empty());
        assert!(l.dock_back(&id("layers")));
        assert!(
            l.contains_docked(&id("layers")),
            "the panel must come back even when its recorded column is gone"
        );
        assert!(l.is_normalized());
    }

    /// **Floating something that is not docked changes nothing.**
    #[test]
    fn floating_an_absent_or_already_floating_panel_is_a_no_op() {
        let mut l = sample();
        assert!(!l.float(&id("nonesuch")));
        assert_eq!(l, sample());
        l.float(&id("layers"));
        let after = l.clone();
        assert!(!l.float(&id("layers")), "already floating");
        assert_eq!(l, after);
    }

    /// **Docking something that is not floating changes nothing.**
    #[test]
    fn docking_a_panel_that_is_not_floating_is_a_no_op() {
        let mut l = sample();
        assert!(!l.dock_back(&id("layers")));
        assert_eq!(l, sample());
    }

    /// **Closing a floating panel removes it entirely** — it does not
    /// leave a float entry pointing at a panel that is nowhere.
    #[test]
    fn closing_a_floating_panel_removes_its_float_entry() {
        let mut l = sample();
        l.float(&id("layers"));
        assert!(l.close(&id("layers")), "close must report that it acted");
        assert!(!l.is_floating(&id("layers")));
        assert!(!l.contains(&id("layers")));
        assert!(l.floating.is_empty());
    }

    /// **A floating panel counts as present**, so the application
    /// cannot mount a second copy of it.
    #[test]
    fn a_floating_panel_cannot_be_mounted_a_second_time() {
        let mut l = sample();
        l.float(&id("layers"));
        assert!(l.contains(&id("layers")));
        l.mount(DockSide::Right, 0, 0, "layers");
        assert!(
            !l.contains_docked(&id("layers")),
            "mount must refuse a panel that is floating"
        );
        assert_eq!(l.floating.len(), 1);
    }

    /// **A floating panel is on screen**, which is what a toolbar
    /// toggle reads to decide whether it is lit.
    #[test]
    fn a_floating_panel_reports_as_on_screen() {
        let mut l = sample();
        l.float(&id("layers"));
        assert!(
            l.is_on_screen(&id("layers")),
            "it is in a window in front of the operator; \"not on screen\" would be a lie"
        );
        l.close(&id("layers"));
        assert!(!l.is_on_screen(&id("layers")));
    }

    /// **A float on a hidden side is still on screen.**
    #[test]
    fn collapsing_the_home_side_does_not_hide_a_float() {
        let mut l = sample();
        l.float(&id("layers"));
        l.side_mut(DockSide::Left).visible = false;
        assert!(l.is_on_screen(&id("layers")));
    }

    /// **Dock-all puts every float back and counts them.**
    #[test]
    fn docking_all_floats_returns_how_many_there_were() {
        let mut l = sample();
        l.float(&id("layers"));
        l.float(&id("tools"));
        assert_eq!(l.dock_all_floating(), 2);
        assert!(l.floating.is_empty());
        assert!(l.contains_docked(&id("layers")));
        assert!(l.contains_docked(&id("tools")));
        assert_eq!(l.dock_all_floating(), 0, "and it is idempotent");
    }

    /// **A panel that is both docked and floating is repaired by
    /// dropping the float.**
    #[test]
    fn normalize_drops_a_float_whose_panel_is_also_docked() {
        let mut l = sample();
        l.floating.push(FloatingPanel::new(
            id("layers"),
            DockHome::origin(DockSide::Left),
        ));
        l.normalize();
        assert!(l.floating.is_empty(), "the float is the copy that loses");
        assert!(
            l.contains_docked(&id("layers")),
            "and the docked mount is the copy that survives"
        );
    }

    /// **Two float entries for one panel become one.**
    #[test]
    fn normalize_drops_a_duplicate_float() {
        let mut l = sample();
        l.float(&id("layers"));
        let dup = l.floating[0].clone();
        l.floating.push(dup);
        l.normalize();
        assert_eq!(l.floating.len(), 1);
    }

    /// **A nonsense size from a file is replaced, not propagated.**
    #[test]
    fn normalize_repairs_a_stored_size() {
        let mut l = sample();
        l.float(&id("layers"));
        l.floating[0].size_pts = [f32::NAN, 99_999.0];
        l.floating[0].pos_pts = Some([f32::INFINITY, 0.0]);
        l.normalize();
        assert_eq!(l.floating[0].size_pts[0], DEFAULT_SIZE_PTS[0]);
        assert_eq!(l.floating[0].size_pts[1], MAX_SIZE_PTS[1]);
        assert_eq!(
            l.floating[0].pos_pts, None,
            "an unusable position must become \"never placed\", not a NaN window"
        );
        assert!(l.is_normalized(), "and the repair must be idempotent");
    }

    /// **A float survives a round trip through the on-disk form,
    /// with its home, its size and its position.**
    #[test]
    fn a_float_survives_serialization() {
        let mut l = sample();
        l.float(&id("layers"));
        l.set_float_geometry(&id("layers"), Some([1400.0, 260.0]), [360.0, 520.0]);
        let text = ron::ser::to_string(&l).expect("serialize");
        let back: DockLayout = ron::from_str(&text).expect("deserialize");
        assert_eq!(back, l);
        let f = back.float_of(&id("layers")).expect("the float came back");
        assert_eq!(f.pos_pts, Some([1400.0, 260.0]));
        assert_eq!(f.size_pts, [360.0, 520.0]);
        assert_eq!(f.home.side, DockSide::Left);
        assert_eq!(f.home.stack, 1);
    }

    /// **A layout written before floats existed still loads.**
    #[test]
    fn a_layout_with_no_floating_key_loads_with_no_floats() {
        let text = r#"(left: (columns: [(stacks: [(tabs: ["pages"])])]), right: (columns: []))"#;
        let back: DockLayout = ron::from_str(text).expect("an old layout must still load");
        assert!(back.floating.is_empty());
        assert!(back.contains_docked(&id("pages")));
    }

    /// **Geometry is only recorded when it changed**, so a still window
    /// does not mark the layout dirty every frame.
    #[test]
    fn recording_unchanged_geometry_reports_no_change() {
        let mut l = sample();
        l.float(&id("layers"));
        assert!(l.set_float_geometry(&id("layers"), Some([10.0, 20.0]), [300.0, 400.0]));
        assert!(
            !l.set_float_geometry(&id("layers"), Some([10.0, 20.0]), [300.0, 400.0]),
            "an unchanged geometry must not report a change, or the layout saves every frame"
        );
    }

    /// A 1,200 x 900 application window on a second monitor, so a test
    /// that assumed a desktop origin of zero would fail.
    fn app() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(1920.0, 100.0), egui::vec2(1200.0, 900.0))
    }

    /// **A position on the monitor next door is honoured.**
    #[test]
    fn a_second_monitor_position_is_kept() {
        let at = honour_position(
            Some([300.0, 200.0]),
            DEFAULT_SIZE_PTS,
            app(),
            Some(egui::vec2(1920.0, 1080.0)),
        );
        assert_eq!(at, Some(egui::pos2(300.0, 200.0)));
    }

    /// **A position far outside every plausible monitor is dropped.**
    ///
    /// The unplugged-monitor case as far as it is answerable here. See
    /// [`honour_position`]'s docs for what this can and cannot know.
    #[test]
    fn a_position_beyond_every_plausible_monitor_is_dropped() {
        let at = honour_position(
            Some([40_000.0, 40_000.0]),
            DEFAULT_SIZE_PTS,
            app(),
            Some(egui::vec2(1920.0, 1080.0)),
        );
        assert_eq!(at, None, "and the caller then places it as if it were new");
    }

    /// **A NaN position is dropped rather than opening a window nowhere.**
    #[test]
    fn a_nonsense_position_is_dropped() {
        assert_eq!(
            honour_position(Some([f32::NAN, 0.0]), DEFAULT_SIZE_PTS, app(), None),
            None
        );
        assert_eq!(honour_position(None, DEFAULT_SIZE_PTS, app(), None), None);
    }

    /// **With no monitor size reported, the fallback reach is used and
    /// a reasonable position still survives.**
    ///
    /// `monitor_size` is `None` on the first frame, which is exactly when
    /// a restored layout is being placed.
    #[test]
    fn an_unknown_monitor_size_still_honours_a_nearby_position() {
        let at = honour_position(Some([200.0, 200.0]), DEFAULT_SIZE_PTS, app(), None);
        assert_eq!(at, Some(egui::pos2(200.0, 200.0)));
    }

    /// **A zero or nonsense monitor size falls back rather than
    /// collapsing the envelope to the application window.**
    #[test]
    fn a_degenerate_monitor_size_falls_back() {
        let at = honour_position(
            Some([200.0, 200.0]),
            DEFAULT_SIZE_PTS,
            app(),
            Some(egui::vec2(0.0, 0.0)),
        );
        assert!(
            at.is_some(),
            "a monitor size of zero must not make every remembered position look off-screen"
        );
    }

    /// **Successive unplaced floats cascade, and the cascade stops.**
    #[test]
    fn unplaced_floats_cascade_and_the_cascade_is_bounded() {
        let a = opening_position(app(), 0);
        let b = opening_position(app(), 1);
        assert_ne!(a, b, "two windows at one point read as one window");
        assert_eq!(
            opening_position(app(), CASCADE_MAX),
            opening_position(app(), CASCADE_MAX + 40),
            "the cascade must stop rather than walk off the desktop"
        );
    }

    /// **Sizes are clamped, and a nonsense size becomes the default.**
    #[test]
    fn sizes_are_clamped_at_both_ends() {
        assert_eq!(clamp_size([1.0, 1.0]), MIN_SIZE_PTS);
        assert_eq!(clamp_size([99_999.0, 99_999.0]), MAX_SIZE_PTS);
        assert_eq!(clamp_size([f32::NAN, f32::NAN]), DEFAULT_SIZE_PTS);
    }
}
