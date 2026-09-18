//! `dock::preview` — where a compartment lands, and what a candidate drop
//! would do to that.
//!
//! Two things that must not be written twice live here.
//!
//! **The rect walk.** A side's columns and a column's stacks are placed by
//! [`plan::resolve_spans`] and then walked, laying each span down and stepping
//! over a splitter. [`Dock::show`](super::Dock::show) calls
//! [`columns_across`] and [`stacks_down`] to know where to draw; the preview
//! below calls the same two functions to know where a panel *would* land. One
//! definition, so the two cannot drift.
//!
//! **The replay.** [`DockLayout::preview_drop`] clones the layout, applies the
//! candidate [`DropTarget`] with the same [`DockLayout::move_panel`] the
//! release will call, finds the dragged panel in the result, and re-walks the
//! rects. What comes back is the compartment the operator will get — not a
//! restatement of the zone they are hovering, which is the disclosure defect
//! the compass exists to prevent: an overlay that highlights a *valid target*
//! rather than the *outcome* is showing one thing and committing another.
//!
//! The side's own rect is taken from the retained [`DockGeometry`] rather than
//! recomputed: it is set by `egui`'s panel reservation less the banner and the
//! rail, and no drop this grammar can express changes it. Every target
//! [`DockLayout::resolve_drop`](super::DockLayout::resolve_drop) produces names
//! a compartment on a side that is already drawn.

use egui::{Rect, Vec2};

use super::drop::DropTarget;
use super::geometry::{DockGeometry, StackAddr};
use super::model::{DockLayout, PanelId};
use super::plan;

/// The columns of a side, laid left-to-right across `area`.
///
/// `area` is the side's rect **less its width handle** — the rectangle
/// [`DockGeometry::side_rect`] reports.
pub(super) fn columns_across(area: Rect, shares: &[f32]) -> Vec<Rect> {
    walk(
        shares,
        area.width(),
        plan::MIN_COLUMN_WIDTH,
        |offset, span| {
            Rect::from_min_size(
                egui::pos2(area.left() + offset, area.top()),
                Vec2::new(span, area.height()),
            )
        },
    )
}

/// The stacks of a column, laid top-to-bottom down `area`.
pub(super) fn stacks_down(area: Rect, shares: &[f32]) -> Vec<Rect> {
    walk(
        shares,
        area.height(),
        plan::MIN_STACK_HEIGHT,
        |offset, span| {
            Rect::from_min_size(
                egui::pos2(area.left(), area.top() + offset),
                Vec2::new(area.width(), span),
            )
        },
    )
}

/// Resolve `shares` against `extent` and hand each span its offset.
///
/// The offset steps over a splitter after every span, including the last —
/// where nothing reads it, which is why this is the same walk the draw path
/// performs with the step inside an `if`.
fn walk(shares: &[f32], extent: f32, min: f32, place: impl Fn(f32, f32) -> Rect) -> Vec<Rect> {
    let spans = plan::resolve_spans(shares, extent, min, plan::SPLITTER_THICKNESS);
    let mut offset = 0.0;
    let mut out = Vec::with_capacity(spans.len());
    for span in spans {
        out.push(place(offset, span));
        offset += span + plan::SPLITTER_THICKNESS;
    }
    out
}

impl DockLayout {
    /// **The compartment `panel` would occupy if it were released at
    /// `target`**, in the coordinates `geometry` was recorded in.
    ///
    /// `None` when the panel is nowhere to be found, when the geometry has no
    /// rect for the side it would land on, or when the layout is empty. A
    /// *declined* move is not one of those: a target naming where the panel
    /// already is returns that compartment, so the overlay shows the honest
    /// no-op rather than blinking out.
    #[must_use]
    pub fn preview_drop(
        &self,
        geometry: &DockGeometry,
        panel: &PanelId,
        target: DropTarget,
    ) -> Option<Rect> {
        let mut after = self.clone();
        let at = if after.move_panel(panel, target) {
            after.find(panel)?
        } else {
            // Nothing moved, so the answer is where the panel is now — and
            // the *current* geometry already holds that rect exactly, with no
            // arithmetic to get wrong.
            return geometry.stack_rect(StackAddr::from(self.find(panel)?));
        };

        let side = after.side(at.side);
        let shares: Vec<f32> = side.columns.iter().map(|c| c.share).collect();
        let column = *columns_across(geometry.side_rect(at.side)?, &shares).get(at.column)?;

        let shares: Vec<f32> = side.columns[at.column]
            .stacks
            .iter()
            .map(|s| s.share)
            .collect();
        stacks_down(column, &shares).get(at.stack).copied()
    }

    /// **Whether releasing `panel` at `target` would change this layout.**
    ///
    /// The overlay knocks its highlight back where the answer is `false` — the
    /// releases that are legal and permute nothing, which are common: a panel
    /// dropped back into the middle of the group it already leads, or against
    /// the edge of a column it is already alone in.
    ///
    /// Asked by applying the same verb the release will apply, to a clone, so
    /// the dimming cannot disagree with the outcome. A predicate written out by
    /// hand would be a second description of the drop grammar, and the
    /// grammar's exceptions — a reorder inside one stack, an insertion into a
    /// column the take has just emptied — are exactly what it would get wrong.
    #[must_use]
    pub fn drop_lands(&self, panel: &PanelId, target: DropTarget) -> bool {
        self.clone().move_panel(panel, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::model::{Column, DockSide, PanelInfo, PanelRegistry, SideLayout, Stack};
    use crate::dock::{ColumnAddr, Dock, DockState};

    /// One panel per compartment, so every rect has exactly one occupant.
    fn registry() -> PanelRegistry {
        let mut r = PanelRegistry::new();
        for (id, label) in [
            ("pages", "Pages"),
            ("layers", "Layers"),
            ("fields", "Fields"),
        ] {
            r.register(PanelInfo::new(id, label));
        }
        r
    }

    /// A left side of two columns: a lone stack, and a column split in two.
    ///
    /// The lone stack is the one the tests drag, because taking it out
    /// **removes its column** — so a preview that merely echoed the target's
    /// current rect would be wrong by half the side's width.
    fn layout() -> DockLayout {
        DockLayout::new(
            SideLayout::new([
                Column::new([Stack::new("pages")]),
                Column::new([Stack::new("layers"), Stack::new("fields")]),
            ])
            .with_width(400.0),
            SideLayout::none(),
        )
    }

    const WINDOW: Vec2 = Vec2::new(1200.0, 800.0);

    /// Render one frame, returning the state with its geometry filled in.
    fn rendered() -> DockState {
        let ctx = egui::Context::default();
        let registry = registry();
        let mut state = DockState::new(layout());
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, WINDOW)),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            Dock::new()
                .with_registry(&registry)
                .show(ui, &mut state, |_panel, ui| {
                    ui.label("body");
                });
        });
        state
    }

    fn close(a: Rect, b: Rect) -> bool {
        (a.min - b.min).abs().max_elem() < 0.5 && (a.max - b.max).abs().max_elem() < 0.5
    }

    /// ★ **The one thing the calibration above cannot see.**
    ///
    /// It compares the walk against a frame that was drawn *by the walk*, so
    /// an error in the step — dropping the splitter, double-counting it —
    /// moves both together and every rect still matches. The step is
    /// therefore asserted directly: compartments sit one splitter apart and
    /// together fill the area they divide, with nothing left over at either
    /// end.
    #[test]
    fn the_walk_leaves_exactly_one_splitter_between_compartments_and_no_slack() {
        let area = Rect::from_min_size(egui::pos2(10.0, 20.0), Vec2::new(600.0, 500.0));
        let shares = [1.0, 2.0, 1.0];

        let columns = columns_across(area, &shares);
        assert_eq!(columns.len(), 3);
        for pair in columns.windows(2) {
            assert!(
                (pair[1].left() - pair[0].right() - plan::SPLITTER_THICKNESS).abs() < 0.001,
                "columns are {} apart, not one splitter: {pair:?}",
                pair[1].left() - pair[0].right()
            );
        }
        assert!(
            (columns[0].left() - area.left()).abs() < 0.001,
            "{columns:?}"
        );
        assert!(
            (columns[2].right() - area.right()).abs() < 0.001,
            "the columns do not reach the far edge: {columns:?} in {area:?}"
        );

        let stacks = stacks_down(area, &shares);
        for pair in stacks.windows(2) {
            assert!(
                (pair[1].top() - pair[0].bottom() - plan::SPLITTER_THICKNESS).abs() < 0.001,
                "stacks are {} apart, not one splitter: {pair:?}",
                pair[1].top() - pair[0].bottom()
            );
        }
        assert!((stacks[0].top() - area.top()).abs() < 0.001, "{stacks:?}");
        assert!(
            (stacks[2].bottom() - area.bottom()).abs() < 0.001,
            "the stacks do not reach the bottom edge: {stacks:?} in {area:?}"
        );
    }

    /// A lone compartment takes the whole area — no splitter is stepped over
    /// after the last span.
    #[test]
    fn a_lone_compartment_fills_the_area_it_was_given() {
        let area = Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(300.0, 400.0));
        assert_eq!(columns_across(area, &[1.0]), vec![area]);
        assert_eq!(stacks_down(area, &[1.0]), vec![area]);
    }

    /// ★ **The calibration, and the reason the replay can be trusted.**
    ///
    /// Everything below asks the walk where a compartment *would* be. This
    /// asks whether the walk agrees with where the dock *did* draw one — so
    /// if the draw path ever stops calling [`columns_across`] and
    /// [`stacks_down`], the divergence is red here rather than shown to an
    /// operator mid-drag.
    #[test]
    fn the_frame_draws_every_compartment_where_the_walk_says_it_does() {
        let state = rendered();
        let geometry = state.geometry();
        let layout = state.layout();
        let side = DockSide::Left;
        let area = geometry.side_rect(side).expect("the side drew");

        let shares: Vec<f32> = layout.side(side).columns.iter().map(|c| c.share).collect();
        let columns = columns_across(area, &shares);
        assert_eq!(columns.len(), 2, "the fixture has two columns");

        let mut checked = 0;
        for (ci, column) in columns.iter().enumerate() {
            let shares: Vec<f32> = layout.side(side).columns[ci]
                .stacks
                .iter()
                .map(|s| s.share)
                .collect();
            for (si, stack) in stacks_down(*column, &shares).into_iter().enumerate() {
                let drawn = geometry
                    .stack_rect(StackAddr::new(side, ci, si))
                    .expect("every stack of a visible side is recorded");
                assert!(
                    close(stack, drawn),
                    "column {ci} stack {si}: the walk says {stack:?}, the frame drew {drawn:?}"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 3, "the fixture has three stacks");
    }

    /// ★ The property a preview built out of the *target's current rect*
    /// cannot have: emptying a column widens the one the panel lands in.
    #[test]
    fn the_preview_accounts_for_the_column_the_drag_empties() {
        let state = rendered();
        let destination = StackAddr::new(DockSide::Left, 1, 0);
        let before = state
            .geometry()
            .stack_rect(destination)
            .expect("the destination drew");

        let after = state
            .layout()
            .preview_drop(
                state.geometry(),
                &PanelId::from("pages"),
                DropTarget::Tab {
                    stack: destination,
                    gap: 1,
                },
            )
            .expect("a panel joining a group has a compartment");

        let area = state.geometry().side_rect(DockSide::Left).expect("drawn");
        assert!(
            after.width() > before.width() + plan::MIN_COLUMN_WIDTH,
            "the emptied column's width was not handed over: {before:?} -> {after:?}"
        );
        assert!(
            (after.width() - area.width()).abs() < 0.5,
            "one column left should span the side: {after:?} in {area:?}"
        );
    }

    /// A bottom-edge release splits the compartment it was made over.
    #[test]
    fn a_release_below_a_stack_previews_the_lower_part_of_that_column() {
        let state = rendered();
        let split = StackAddr::new(DockSide::Left, 1, 1);
        let before = state.geometry().stack_rect(split).expect("drawn");

        let after = state
            .layout()
            .preview_drop(
                state.geometry(),
                &PanelId::from("pages"),
                DropTarget::Stack {
                    column: ColumnAddr::new(DockSide::Left, 1),
                    gap: 2,
                },
            )
            .expect("a new stack has a compartment");

        assert!(
            after.top() > before.top(),
            "a drop below the lowest stack landed above it: {after:?} vs {before:?}"
        );
        assert!(
            after.height() < before.height(),
            "the column was not divided: {after:?} vs {before:?}"
        );
    }

    /// A left-edge release starts a column beside the one it split.
    #[test]
    fn a_release_against_an_edge_previews_a_column_beside_the_one_it_split() {
        let state = rendered();
        let after = state
            .layout()
            .preview_drop(
                state.geometry(),
                &PanelId::from("fields"),
                DropTarget::Column {
                    side: DockSide::Left,
                    gap: 0,
                },
            )
            .expect("a new column has a compartment");

        let area = state.geometry().side_rect(DockSide::Left).expect("drawn");
        assert!(
            (after.left() - area.left()).abs() < 0.5,
            "a column at boundary 0 does not start at the side's left edge: {after:?} in {area:?}"
        );
        assert!(
            after.height() > area.height() - 0.5,
            "a lone stack in a new column should have the column's full height: {after:?}"
        );
    }

    /// ★ A target that moves nothing still has an answer, and it is the
    /// compartment the panel is already in. Returning `None` would blink the
    /// highlight out over exactly the release the operator is most likely to
    /// make by accident.
    #[test]
    fn a_target_that_changes_nothing_previews_where_the_panel_already_is() {
        let state = rendered();
        let home = StackAddr::new(DockSide::Left, 0, 0);
        let after = state
            .layout()
            .preview_drop(
                state.geometry(),
                &PanelId::from("pages"),
                DropTarget::Tab {
                    stack: home,
                    gap: 0,
                },
            )
            .expect("a no-op still has a compartment");
        assert_eq!(
            Some(after),
            state.geometry().stack_rect(home),
            "a no-op previewed somewhere other than home"
        );
    }

    #[test]
    fn a_panel_the_layout_has_never_heard_of_has_no_preview() {
        let state = rendered();
        assert_eq!(
            state.layout().preview_drop(
                state.geometry(),
                &PanelId::from("nothing"),
                DropTarget::Tab {
                    stack: StackAddr::new(DockSide::Left, 1, 0),
                    gap: 0,
                },
            ),
            None
        );
    }

    /// ★ The geometry is a record of a frame that happened. Before the first
    /// one there is nothing to divide, and inventing a rect from the window
    /// would be a second answer to the question the panel reservation already
    /// settles.
    #[test]
    fn a_preview_taken_before_the_first_frame_is_nothing() {
        let layout = layout();
        assert_eq!(
            layout.preview_drop(
                &DockGeometry::default(),
                &PanelId::from("pages"),
                DropTarget::Tab {
                    stack: StackAddr::new(DockSide::Left, 1, 0),
                    gap: 0,
                },
            ),
            None
        );
    }
}
