//! Tests for **the drop grammar** — [`super::DropTarget`] and
//! [`super::model::DockLayout::move_panel`].
//!
//! # Why these open nothing
//!
//! The grammar is a pure value: a layout in, a layout out. Its sibling
//! [`super::drag_tests`] drives a real `egui::Context` because the gesture it
//! measures only exists inside one. Nothing here does, and a windowed harness
//! would add a second thing that can fail without measuring anything more.
//!
//! # ★ The fuzz is the point, and it counts what it swept
//!
//! [`every_drop_leaves_a_layout_the_dock_can_draw`] generates layouts and
//! targets — including out-of-range ones — and asserts the invariants after
//! each. A generator that declined every case would satisfy every invariant
//! vacuously, so the test also asserts that each of the three variants landed,
//! and that a meaningful share of the cases both landed and were declined. A
//! sweep that does not say what it swept is a claim about the generator.

use std::collections::BTreeSet;

use super::geometry::{ColumnAddr, StackAddr};
use super::{Column, DockLayout, DockSide, DropTarget, PanelId, SideLayout, Stack};

/// Left: `[pages | bookmarks]` in column 0, `[layers]` in column 1.
/// Right: `[props]`.
fn two_columns() -> DockLayout {
    DockLayout::new(
        SideLayout::new([
            Column::new([Stack::tabbed(["pages", "bookmarks"])]),
            Column::new([Stack::new("layers")]),
        ]),
        SideLayout::new([Column::new([Stack::new("props")])]),
    )
}

/// The stack holding `layers`, alone in the left side's second column.
const LAYERS: StackAddr = StackAddr {
    side: DockSide::Left,
    column: 1,
    stack: 0,
};

/// The stack holding `pages` and `bookmarks`.
const PAGES: StackAddr = StackAddr {
    side: DockSide::Left,
    column: 0,
    stack: 0,
};

/// The tabs of one stack, by address, as plain strings.
fn tabs_of(layout: &DockLayout, addr: StackAddr) -> Vec<&str> {
    layout.side(addr.side).columns[addr.column].stacks[addr.stack]
        .tabs
        .iter()
        .map(PanelId::as_str)
        .collect()
}

#[test]
fn a_panel_dropped_on_another_stacks_tab_bar_joins_it_at_the_boundary() {
    let mut layout = two_columns();
    assert!(layout.move_panel(
        &PanelId::new("pages"),
        DropTarget::Tab {
            stack: LAYERS,
            gap: 0,
        },
    ));
    assert_eq!(tabs_of(&layout, LAYERS), ["pages", "layers"]);
    assert_eq!(tabs_of(&layout, PAGES), ["bookmarks"]);
}

#[test]
fn the_boundary_past_the_last_tab_puts_the_panel_last() {
    let mut layout = two_columns();
    assert!(layout.move_panel(
        &PanelId::new("pages"),
        DropTarget::Tab {
            stack: LAYERS,
            gap: 1,
        },
    ));
    assert_eq!(tabs_of(&layout, LAYERS), ["layers", "pages"]);
}

#[test]
fn the_dropped_panel_becomes_the_active_tab_of_the_stack_it_joins() {
    let mut layout = two_columns();
    assert!(layout.move_panel(
        &PanelId::new("pages"),
        DropTarget::Tab {
            stack: LAYERS,
            gap: 1,
        },
    ));
    assert!(layout.is_active(&PanelId::new("pages")));
}

/// The hole is why this passes. `pages` is alone in the left side's stack 0, so
/// removing it empties that stack; if the empty stack were pruned before the
/// insertion, stack index 2 would name a different compartment — or none.
#[test]
fn a_target_later_in_the_source_column_still_names_that_stack_after_the_take() {
    let mut layout = DockLayout::new(
        SideLayout::new([Column::new([
            Stack::new("pages"),
            Stack::new("bookmarks"),
            Stack::tabbed(["layers", "search"]),
        ])]),
        SideLayout::default(),
    );
    let third = StackAddr::new(DockSide::Left, 0, 2);
    assert!(layout.move_panel(
        &PanelId::new("pages"),
        DropTarget::Tab {
            stack: third,
            gap: 2
        }
    ));

    // One stack fewer, and the pruning happened after the insertion rather than
    // before it, so the target still meant the stack the operator pointed at.
    let column = &layout.left.columns[0];
    assert_eq!(column.stacks.len(), 2);
    assert_eq!(
        tabs_of(&layout, StackAddr::new(DockSide::Left, 0, 1)),
        ["layers", "search", "pages"]
    );
}

#[test]
fn a_drop_against_a_columns_edge_splits_it_with_a_new_stack() {
    let mut layout = two_columns();
    assert!(layout.move_panel(
        &PanelId::new("props"),
        DropTarget::Stack {
            column: ColumnAddr::new(DockSide::Left, 0),
            gap: 0,
        },
    ));
    assert_eq!(layout.left.columns[0].stacks.len(), 2);
    assert_eq!(
        tabs_of(&layout, StackAddr::new(DockSide::Left, 0, 0)),
        ["props"]
    );
    assert!(layout.right.columns.is_empty());
}

#[test]
fn a_drop_past_the_last_column_starts_a_new_one() {
    let mut layout = two_columns();
    assert!(layout.move_panel(
        &PanelId::new("props"),
        DropTarget::Column {
            side: DockSide::Left,
            gap: 2,
        },
    ));
    assert_eq!(layout.left.columns.len(), 3);
    assert_eq!(
        tabs_of(&layout, StackAddr::new(DockSide::Left, 2, 0)),
        ["props"]
    );
}

#[test]
fn a_new_stack_and_a_new_column_take_the_default_share() {
    let mut layout = two_columns();
    layout.left.columns[0].share = 3.0;
    assert!(layout.move_panel(
        &PanelId::new("props"),
        DropTarget::Column {
            side: DockSide::Left,
            gap: 0,
        },
    ));
    assert!((layout.left.columns[0].share - Column::default().share).abs() < f32::EPSILON);
    // The column that was already there keeps the weight the operator gave it.
    assert!((layout.left.columns[1].share - 3.0).abs() < f32::EPSILON);
}

#[test]
fn a_panel_moved_to_a_side_that_draws_nothing_makes_that_side_visible() {
    let mut layout = DockLayout::new(
        SideLayout::new([Column::new([Stack::tabbed(["pages", "bookmarks"])])]),
        SideLayout::none(),
    );
    assert!(!layout.right.visible);
    assert!(layout.move_panel(
        &PanelId::new("pages"),
        DropTarget::Column {
            side: DockSide::Right,
            gap: 0,
        },
    ));
    assert!(layout.right.visible);
    assert!(layout.is_on_screen(&PanelId::new("pages")));
}

/// Within one stack the drop is a reorder, and a reorder does not change which
/// panel is on screen. Inserting as if the stack were a stranger would raise the
/// dragged tab instead, which is the observable that separates the two paths.
#[test]
fn a_move_within_one_stack_is_a_reorder_and_keeps_the_visible_panel() {
    let mut layout = DockLayout::new(
        SideLayout::new([Column::new([Stack::tabbed([
            "pages",
            "bookmarks",
            "layers",
        ])
        .with_active(&PanelId::new("bookmarks"))])]),
        SideLayout::default(),
    );
    assert!(layout.move_panel(
        &PanelId::new("pages"),
        DropTarget::Tab {
            stack: PAGES,
            gap: 3,
        },
    ));
    assert_eq!(tabs_of(&layout, PAGES), ["bookmarks", "layers", "pages"]);
    assert!(layout.is_active(&PanelId::new("bookmarks")));
}

#[test]
fn a_move_within_one_stack_that_permutes_nothing_is_declined() {
    let layout = two_columns();
    for gap in [0, 1] {
        let mut probe = layout.clone();
        assert!(!probe.move_panel(
            &PanelId::new("pages"),
            DropTarget::Tab { stack: PAGES, gap }
        ));
        assert_eq!(probe, layout);
    }
}

#[test]
fn a_panel_that_is_not_in_the_layout_is_declined_and_changes_nothing() {
    let layout = two_columns();
    let mut probe = layout.clone();
    assert!(!probe.move_panel(
        &PanelId::new("nothing-by-that-name"),
        DropTarget::Tab {
            stack: LAYERS,
            gap: 0,
        },
    ));
    assert_eq!(probe, layout);
}

#[test]
fn an_out_of_range_target_is_declined_and_changes_nothing() {
    let layout = two_columns();
    let targets = [
        DropTarget::Tab {
            stack: StackAddr::new(DockSide::Left, 9, 0),
            gap: 0,
        },
        DropTarget::Tab {
            stack: StackAddr::new(DockSide::Left, 1, 9),
            gap: 0,
        },
        DropTarget::Tab {
            stack: LAYERS,
            gap: 9,
        },
        DropTarget::Stack {
            column: ColumnAddr::new(DockSide::Left, 9),
            gap: 0,
        },
        DropTarget::Stack {
            column: ColumnAddr::new(DockSide::Left, 0),
            gap: 9,
        },
        DropTarget::Column {
            side: DockSide::Left,
            gap: 9,
        },
    ];
    for target in targets {
        let mut probe = layout.clone();
        assert!(!probe.accepts_drop(target));
        assert!(!probe.move_panel(&PanelId::new("pages"), target));
        assert_eq!(probe, layout, "{target:?} mutated a layout it declined");
    }
}

#[test]
fn a_floating_panel_dropped_into_the_dock_stops_floating() {
    let mut layout = two_columns();
    assert!(layout.float(&PanelId::new("props")));
    assert!(layout.is_floating(&PanelId::new("props")));
    assert!(layout.move_panel(
        &PanelId::new("props"),
        DropTarget::Tab {
            stack: LAYERS,
            gap: 0,
        },
    ));
    assert!(!layout.is_floating(&PanelId::new("props")));
    assert_eq!(tabs_of(&layout, LAYERS), ["props", "layers"]);
}

/// [`super::model::DockLayout::close`] delegates its removal to
/// [`super::model::DockLayout::take_panel`]; this is the rule that delegation
/// must not have changed.
///
/// ★ **The closed tab is in the middle, and that is the whole test.** Closing
/// the *last* tab selects its predecessor whether the rule exists or not,
/// because [`super::model::DockLayout::normalize`] clamps a stale index to the
/// end of the list and the end of the list is the predecessor. Only a tab with
/// something after it separates "the previous one" from "whatever slid into
/// this index" — closing `bookmarks` selects `pages` under the rule and
/// `layers` without it.
#[test]
fn closing_the_active_tab_selects_the_one_before_it_not_the_one_after() {
    let mut layout = DockLayout::new(
        SideLayout::new([Column::new([Stack::tabbed([
            "pages",
            "bookmarks",
            "layers",
        ])
        .with_active(&PanelId::new("bookmarks"))])]),
        SideLayout::default(),
    );
    assert!(layout.close(&PanelId::new("bookmarks")));
    assert!(layout.is_active(&PanelId::new("pages")));
}

// ---------------------------------------------------------------------------
// The fuzz
// ---------------------------------------------------------------------------

/// xorshift64*, seeded per case. Deterministic on purpose: a failing case is
/// reproduced by its seed, and a suite whose input changes between runs cannot
/// be bisected.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// A value in `0..n`.
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// Weights a generated compartment may carry — one below the default, the
/// default, and one above, so a drop that resets a share is visible.
const SHARES: [f32; 3] = [0.5, 1.0, 2.0];

/// A layout of up to two columns a side, two stacks a column and three tabs a
/// stack, with every panel named once. Normalized, so the pre-state is one the
/// dock could actually have drawn.
fn random_layout(rng: &mut Rng) -> DockLayout {
    let mut next = 0usize;
    let mut sides = Vec::new();
    for _ in 0..2 {
        let mut columns = Vec::new();
        for _ in 0..rng.below(3) {
            let mut stacks = Vec::new();
            for _ in 0..=rng.below(2) {
                let tabs: Vec<PanelId> = (0..=rng.below(3))
                    .map(|_| {
                        next += 1;
                        PanelId::new(format!("p{next}"))
                    })
                    .collect();
                let active = rng.below(tabs.len());
                stacks.push(Stack {
                    tabs,
                    active,
                    share: SHARES[rng.below(SHARES.len())],
                });
            }
            columns.push(Column {
                stacks,
                share: SHARES[rng.below(SHARES.len())],
            });
        }
        sides.push(SideLayout::new(columns));
    }
    let right = sides.pop().expect("two sides were pushed");
    let left = sides.pop().expect("two sides were pushed");
    let mut layout = DockLayout::new(left, right);
    layout.normalize();
    layout
}

/// A target that may or may not exist: every index range runs one past what
/// [`random_layout`] can produce, because the decline path is half of what is
/// under test and an index that is always valid would never exercise it.
fn random_target(rng: &mut Rng) -> DropTarget {
    let side = if rng.below(2) == 0 {
        DockSide::Left
    } else {
        DockSide::Right
    };
    match rng.below(3) {
        0 => DropTarget::Tab {
            stack: StackAddr::new(side, rng.below(3), rng.below(3)),
            gap: rng.below(5),
        },
        1 => DropTarget::Stack {
            column: ColumnAddr::new(side, rng.below(3)),
            gap: rng.below(4),
        },
        _ => DropTarget::Column {
            side,
            gap: rng.below(4),
        },
    }
}

/// Every panel the layout holds, docked or floating.
fn panel_set(layout: &DockLayout) -> BTreeSet<PanelId> {
    layout.panels().cloned().collect()
}

/// How many times `panel` appears across every tab of both sides.
fn occurrences(layout: &DockLayout, panel: &PanelId) -> usize {
    DockSide::ALL
        .iter()
        .flat_map(|side| layout.side(*side).columns.iter())
        .flat_map(|column| column.stacks.iter())
        .flat_map(|stack| stack.tabs.iter())
        .filter(|p| *p == panel)
        .count()
}

/// How many of each [`DropTarget`] variant the sweep must actually land, in the
/// enum's own order. Roughly half of what the generator produces today, so a
/// change that quietly halves coverage of one arm fails rather than thinning
/// it: the tab arm is the rarest, and is also the one the hole rule lives in.
const LANDINGS_AT_LEAST: [usize; 3] = [150, 400, 900];

#[test]
fn every_drop_leaves_a_layout_the_dock_can_draw() {
    const CASES: usize = 12_000;
    let mut landed = [0usize; 3];
    let mut declined = 0usize;
    let mut skipped = 0usize;

    for seed in 1..=CASES as u64 {
        let rng = &mut Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let layout = random_layout(rng);
        let panels: Vec<PanelId> = layout.panels().cloned().collect();
        if panels.is_empty() {
            skipped += 1;
            continue;
        }
        let panel = panels[rng.below(panels.len())].clone();
        let target = random_target(rng);

        let before = layout.clone();
        let mut after = layout;
        if !after.move_panel(&panel, target) {
            declined += 1;
            assert_eq!(
                after, before,
                "seed {seed}: a declined move mutated the layout"
            );
            continue;
        }

        landed[match target {
            DropTarget::Tab { .. } => 0,
            DropTarget::Stack { .. } => 1,
            DropTarget::Column { .. } => 2,
        }] += 1;

        assert_eq!(panel_set(&after), panel_set(&before), "seed {seed}: panels");
        assert_eq!(occurrences(&after, &panel), 1, "seed {seed}: moved panel");
        assert!(
            after.is_normalized(),
            "seed {seed}: needs normalizing again"
        );
        for side in DockSide::ALL {
            for column in &after.side(side).columns {
                assert!(!column.stacks.is_empty(), "seed {seed}: empty column");
                assert!(column.share.is_finite(), "seed {seed}: column share");
                for stack in &column.stacks {
                    assert!(!stack.tabs.is_empty(), "seed {seed}: empty stack");
                    assert!(stack.active < stack.tabs.len(), "seed {seed}: active");
                    assert!(stack.share.is_finite(), "seed {seed}: stack share");
                }
            }
        }
    }

    // What the sweep actually swept. Without this a generator that declined
    // every case, or never produced one of the three targets, would report the
    // invariants as holding over work it never did.
    assert!(declined > CASES / 4, "declines: {declined}");
    assert!(
        skipped < CASES / 4,
        "cases with no panel to move: {skipped}"
    );
    for (variant, floor) in LANDINGS_AT_LEAST.iter().enumerate() {
        assert!(
            landed[variant] >= *floor,
            "target variant {variant} landed {} times, floor {floor}",
            landed[variant]
        );
    }
}
