//! Dock layout arithmetic — the part of the panel host that has no `egui`
//! in it.
//!
//! # Why this is a separate module with no `Ui` in its signatures
//!
//! Everything in this file is a pure function over `f32` and `usize`.
//! That is not tidiness; it is the only way the failure modes below can be
//! *tested* at all, because each of them is a statement about a number
//! that was computed, not about a pixel that was painted. The numbers are
//! `MODES_AND_PANELS.md` Part 2's:
//!
//! | # | Failure | The function that makes it impossible |
//! |---|---|---|
//! | 3 | **Widest hidden tab dictates minimum width** — an inactive tab you cannot see holds the whole dock open. | [`MIN_COLUMN_WIDTH`] is a *constant*. No function here ever consults a tab label when computing a minimum. |
//! | 6 | **Layout not stable under window resize** — un-maximise and re-maximise loses panel proportions. | `resolve_spans` is a pure function of `(shares, total)`. It has no memory, writes nothing back, and is therefore idempotent under any sequence of totals. |
//! | 7 | **Coupled splitters** — dragging one divider resizes every column. | `drag_boundary` touches exactly two entries of its slice, by construction. |
//! | 8 | **Tab overflow has no escape** — past a handful of tabs the overflow *button itself* gets hidden. | `plan_tabs` subtracts the reservation **before** the first tab is measured against the remainder. |
//!
//! ## The ordering trap that #8 actually is
//!
//! Failure mode #8 is a *layout arithmetic* defect, not a rendering one.
//! It happens when the overflow control is emitted **after** the content,
//! into whatever space the content did not take — which is the obvious
//! immediate-mode spelling, and which yields nothing at all once the
//! content takes everything. Reading such code does not reveal the bug;
//! the code says "draw the tabs, then draw the overflow button", and that
//! sentence sounds correct.
//!
//! So the reservation is made the **first** subtraction rather than the
//! last:
//!
//! ```text
//! tab_budget = available − overflow_width − gap
//! ```
//!
//! computed before a single tab is measured against it. The tab loop can
//! then only ever consume `tab_budget`, and the overflow control's width
//! is not in that number. There is no ordering of the loop that can reach
//! it. [`super::tabs`] enforces the same reservation a second time by
//! handing the tabs a rectangle that **is** `tab_budget` wide, so
//! `egui`'s own clipping backs up the arithmetic.
//!
//! This module is a deliberate sibling of [`crate::ribbon::plan`], which
//! solves the identical problem for ribbon *groups*. The two are not
//! merged because the shapes differ in one load-bearing way — see
//! `plan_tabs`' note on why a tab plan is a **window** and a band plan
//! is a **prefix** — and a single function with a flag for that would be
//! the sort of false economy that makes both harder to reason about.
//!
//! # Why widths are estimated rather than measured
//!
//! Immediate mode has a genuine ordering problem: the width a tab will
//! occupy is known only after it is drawn, and the decision about whether
//! to draw it must be made before. The three ways out are (1) draw,
//! measure, re-lay-out next frame — correct, with a visible flicker on
//! every resize; (2) draw into a scratch layer and discard — correct,
//! double the work, and every side effect has to be suppressed; or (3)
//! estimate analytically from the label list using the same font metrics
//! `egui` will use. This module is option 3, and `tab_width` is where
//! the estimate is made.
//!
//! An estimate that is too small costs a **clipped tab label**; it cannot
//! cost the overflow control, because the control's width was subtracted
//! from the total before the estimate was consulted. That asymmetry is
//! the whole reason the reservation is first.
//!
//! # Minimum sizes, and why they are constants rather than measurements
//!
//! [`MIN_COLUMN_WIDTH`], [`MIN_STACK_HEIGHT`] and [`MIN_SIDE_WIDTH`] are
//! constants. That is failure mode #3's design rule stated as code:
//!
//! > Size a container to its **active** child; let inactive children
//! > scroll.
//!
//! The defect it names is a *hidden* tab whose label is wide enough to
//! hold the whole dock open — you cannot see it, you cannot narrow the
//! dock, and the only cure is to close a panel you did not know was there.
//! That can only happen if some minimum-width computation walks the tab
//! list. Nothing here does, and
//! `the_minimum_column_width_ignores_tab_labels_entirely` is the test
//! that keeps it that way.
//!
//! They also make this module's arithmetic meaningful **when no font is
//! installed**. This crate depends on `egui` with
//! `default-features = false`, so a test process may have no font data
//! and every galley measures near zero; a layout that derived its sizes
//! from text alone would collapse to zero in exactly the environment its
//! tests run in. See `dock::width_tests` for why the floor is
//! necessary but *not sufficient*, and what is done about that.
//!
//! Design and rationale: `docs/modules/egui-shell/dock/plan.md`.

/// The narrowest a dock column may become by dragging.
pub const MIN_COLUMN_WIDTH: f32 = 140.0;

/// The shortest a vertical stack within a column may become by dragging.
pub const MIN_STACK_HEIGHT: f32 = 80.0;

/// The narrowest a whole dock side may become.
///
/// Deliberately close to [`MIN_COLUMN_WIDTH`]: a side holding one column
/// should be able to shrink to that column's minimum and no further.
pub const MIN_SIDE_WIDTH: f32 = 160.0;

/// The widest a dock side may be *drawn*, as a fraction of the window.
pub const MAX_SIDE_FRACTION: f32 = 0.45;

/// The thickness of a draggable splitter, in points, and therefore the
/// gap between two adjacent columns or stacks.
pub const SPLITTER_THICKNESS: f32 = 6.0;

/// The narrowest a tab may be drawn before it is dropped into the
/// overflow menu instead.
pub const MIN_TAB_WIDTH: f32 = 44.0;

/// The widest a single tab may be, however long its label.
pub const MAX_TAB_WIDTH: f32 = 160.0;

/// Horizontal padding inside a tab, total across both sides.
pub const TAB_PADDING: f32 = 14.0;

/// The gap between two adjacent tabs, and between the last tab and the
/// overflow affordance.
pub const TAB_GAP: f32 = 2.0;

/// The height of a stack's tab bar, in points.
pub const TAB_BAR_HEIGHT: f32 = 24.0;

/// Clamp a value that came from a layout pass into a usable length.
#[must_use]
pub(crate) fn sane_length(v: f32) -> f32 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

// ---------------------------------------------------------------------
// Proportional spans with pinned minimums
// ---------------------------------------------------------------------

/// Resolve a list of proportional shares into concrete spans.
#[must_use]
pub(crate) fn resolve_spans(shares: &[f32], total: f32, min: f32, gap: f32) -> Vec<f32> {
    let n = shares.len();
    if n == 0 {
        return Vec::new();
    }
    let total = sane_length(total);
    let gap = sane_length(gap);
    let min = sane_length(min);

    let content = (total - gap * (n as f32 - 1.0)).max(0.0);

    // Step 2: no assignment can satisfy the minimums. Split equally
    // rather than letting a child fall off the end of the container.
    if content <= min * n as f32 {
        return vec![content / n as f32; n];
    }

    let weights: Vec<f32> = shares.iter().map(|s| sanitize_share(*s)).collect();

    let mut spans = vec![0.0_f32; n];
    let mut pinned = vec![false; n];

    loop {
        let free: f32 = content
            - pinned
                .iter()
                .zip(&spans)
                .filter(|(p, _)| **p)
                .map(|(_, s)| *s)
                .sum::<f32>();
        let weight_sum: f32 = weights
            .iter()
            .zip(&pinned)
            .filter(|(_, p)| !**p)
            .map(|(w, _)| *w)
            .sum();

        // Every child is pinned; nothing left to distribute.
        if weight_sum <= 0.0 {
            break;
        }

        let mut newly_pinned = false;
        for i in 0..n {
            if pinned[i] {
                continue;
            }
            let span = free * weights[i] / weight_sum;
            if span < min {
                spans[i] = min;
                pinned[i] = true;
                newly_pinned = true;
            } else {
                spans[i] = span;
            }
        }
        if !newly_pinned {
            break;
        }
    }

    spans
}

/// The smallest weight a share may hold.
///
/// Not zero: see [`resolve_spans`]' argument notes. A child with no width
/// has no splitter, and a splitter is the only way back.
pub(crate) const MIN_SHARE: f32 = 0.01;

/// Coerce one stored share into a usable weight.
fn sanitize_share(s: f32) -> f32 {
    if s.is_finite() && s > MIN_SHARE {
        s
    } else {
        MIN_SHARE
    }
}

/// Move the boundary between children `i` and `i + 1` by `delta` points.
pub(crate) fn drag_boundary(spans: &mut [f32], boundary: usize, delta: f32, min: f32) -> f32 {
    if boundary + 1 >= spans.len() || !delta.is_finite() {
        return 0.0;
    }
    let min = sane_length(min);
    let (a, b) = (spans[boundary], spans[boundary + 1]);
    // The clamp is what pins the minimums: `a` may not shrink below
    // `min`, and neither may `b`. If either is ALREADY below `min` — a
    // window too small to satisfy them, per `resolve_spans` step 2 — the
    // corresponding bound is zero-width and the drag simply cannot move
    // in that direction, which is the honest behaviour.
    let low = (min - a).min(0.0);
    let high = (b - min).max(0.0);
    let applied = delta.clamp(low, high);
    spans[boundary] = a + applied;
    spans[boundary + 1] = b - applied;
    applied
}

/// Convert resolved spans back into stored shares.
#[must_use]
pub(crate) fn spans_to_shares(spans: &[f32]) -> Vec<f32> {
    let sum: f32 = spans.iter().map(|s| sane_length(*s)).sum();
    if sum <= 0.0 {
        return vec![1.0 / spans.len().max(1) as f32; spans.len()];
    }
    spans
        .iter()
        .map(|s| (sane_length(*s) / sum).max(MIN_SHARE))
        .collect()
}

// ---------------------------------------------------------------------
// Tab bar planning
// ---------------------------------------------------------------------

/// One tab's planned width, given its measured label.
#[must_use]
pub(crate) fn tab_width(label_width: f32) -> f32 {
    (sane_length(label_width) + TAB_PADDING).clamp(MIN_TAB_WIDTH, MAX_TAB_WIDTH)
}

/// The overflow affordance's label for `hidden` hidden tabs.
#[must_use]
pub(crate) fn overflow_label(hidden: usize) -> String {
    format!("⏷ {hidden} more")
}

/// The width to reserve for the overflow affordance, given how many tabs
/// the stack holds **in total**.
#[must_use]
pub(crate) fn overflow_width(total: usize, padding: f32, measure: impl Fn(&str) -> f32) -> f32 {
    let widest = (1..=total.max(1))
        .map(|n| measure(&overflow_label(n)))
        .fold(0.0_f32, f32::max);
    (widest + padding).max(MIN_TAB_WIDTH)
}

/// How a stack's tabs are split between the visible bar and the overflow
/// menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TabPlan {
    /// Index of the first visible tab.
    pub start: usize,
    /// How many consecutive tabs are visible, starting at [`Self::start`].
    ///
    /// May be **zero**: at a width narrower than the reservation plus one
    /// minimum tab, the bar degrades to a lone "⏷ N more" control that
    /// still reaches every panel. That is the correct answer and it is
    /// the case the reservation exists for — a bar of clipped tabs with
    /// no route to the rest is the defect.
    pub shown: usize,
    /// How many tabs are in the overflow menu — those before
    /// [`Self::start`] and those after the visible window.
    pub hidden: usize,
    /// The width the visible tabs may occupy, in points. **Already
    /// excludes the overflow control's width.**
    pub tab_budget: f32,
    /// The width reserved for the overflow affordance, or `0.0` when
    /// nothing overflowed.
    ///
    /// Zero exactly when [`Self::hidden`] is zero. That biconditional is
    /// asserted by
    /// `the_overflow_affordance_is_reserved_exactly_when_it_is_needed`.
    pub overflow_width: f32,
}

impl TabPlan {
    /// Whether the overflow affordance is to be drawn.
    #[must_use]
    pub(crate) fn has_overflow(self) -> bool {
        self.hidden > 0
    }

    /// Whether tab `i` is in the visible window.
    #[must_use]
    pub(crate) fn is_visible(self, i: usize) -> bool {
        i >= self.start && i < self.start + self.shown
    }
}

/// Decide which tabs fit, reserving the overflow affordance's width
/// **before** any tab is measured against the remainder.
#[must_use]
pub(crate) fn plan_tabs(
    widths: &[f32],
    active: usize,
    available: f32,
    gap: f32,
    overflow_width: f32,
) -> TabPlan {
    let n = widths.len();
    let available = sane_length(available);
    let gap = sane_length(gap);
    let overflow_width = sane_length(overflow_width);

    if n == 0 {
        return TabPlan {
            start: 0,
            shown: 0,
            hidden: 0,
            tab_budget: available,
            overflow_width: 0.0,
        };
    }
    let active = active.min(n - 1);

    let total: f32 = widths.iter().map(|w| sane_length(*w)).sum::<f32>() + gap * (n as f32 - 1.0);
    if total <= available {
        return TabPlan {
            start: 0,
            shown: n,
            hidden: 0,
            tab_budget: available,
            overflow_width: 0.0,
        };
    }

    // THE RESERVATION. Subtracted before a single tab is considered.
    let tab_budget = (available - overflow_width - gap).max(0.0);

    // Step 3: the window that starts at zero.
    let prefix = fill_forward(widths, 0, tab_budget, gap);
    let (start, shown) = if prefix > 0 && active < prefix {
        (0, prefix)
    } else {
        // Step 4: backwards from the active tab, then forwards.
        let start = fill_backward(widths, active, tab_budget, gap);
        let shown = fill_forward(widths, start, tab_budget, gap);
        (start, shown)
    };

    TabPlan {
        start,
        shown,
        hidden: n - shown,
        tab_budget,
        overflow_width,
    }
}

/// How many consecutive tabs starting at `from` fit in `budget`.
fn fill_forward(widths: &[f32], from: usize, budget: f32, gap: f32) -> usize {
    let mut used = 0.0_f32;
    let mut count = 0_usize;
    for (i, w) in widths.iter().enumerate().skip(from) {
        let step = if i == from {
            sane_length(*w)
        } else {
            gap + sane_length(*w)
        };
        if used + step <= budget {
            used += step;
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// The smallest `start` such that the tabs `start..=active` fit in
/// `budget`.
fn fill_backward(widths: &[f32], active: usize, budget: f32, gap: f32) -> usize {
    let mut used = sane_length(widths[active]);
    if used > budget {
        return active;
    }
    let mut start = active;
    while start > 0 {
        let step = gap + sane_length(widths[start - 1]);
        if used + step > budget {
            break;
        }
        used += step;
        start -= 1;
    }
    start
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compare two floats with a tolerance that is well below anything a
    /// person could see and well above `f32` accumulation error.
    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    // -----------------------------------------------------------------
    // resolve_spans — failure mode #6
    // -----------------------------------------------------------------

    /// Equal shares divide the content equally, and the gaps come out of
    /// the total rather than out of the children's own accounting.
    #[test]
    fn equal_shares_divide_the_content_equally_after_the_gaps() {
        let spans = resolve_spans(&[1.0, 1.0, 1.0], 306.0, 10.0, 3.0);
        // 306 − 2 gaps of 3 = 300 of content, three ways.
        assert_eq!(spans.len(), 3);
        for s in &spans {
            assert!(close(*s, 100.0), "{spans:?}");
        }
    }

    /// Unequal shares divide in proportion, and the spans still sum to
    /// the content.
    #[test]
    fn unequal_shares_divide_in_proportion() {
        let spans = resolve_spans(&[3.0, 1.0], 100.0, 10.0, 0.0);
        assert!(close(spans[0], 75.0), "{spans:?}");
        assert!(close(spans[1], 25.0), "{spans:?}");
        assert!(close(spans.iter().sum::<f32>(), 100.0));
    }

    /// **A child that would fall below the minimum is pinned at it, and
    /// the remainder is redistributed among the others.**
    #[test]
    fn a_child_below_the_minimum_is_pinned_and_the_rest_redistribute() {
        let spans = resolve_spans(&[0.02, 1.0, 1.0], 300.0, 50.0, 0.0);
        assert!(close(spans[0], 50.0), "pinned at the minimum: {spans:?}");
        assert!(close(spans[1], 125.0), "{spans:?}");
        assert!(close(spans[2], 125.0), "{spans:?}");
        assert!(close(spans.iter().sum::<f32>(), 300.0));
    }

    /// Two children needing the pin both get it, in one settled result.
    #[test]
    fn several_children_can_be_pinned_in_the_same_resolve() {
        let spans = resolve_spans(&[0.01, 0.01, 1.0], 300.0, 60.0, 0.0);
        assert!(close(spans[0], 60.0), "{spans:?}");
        assert!(close(spans[1], 60.0), "{spans:?}");
        assert!(close(spans[2], 180.0), "{spans:?}");
    }

    /// When the container cannot satisfy every minimum, the split is
    /// **equal** — and, critically, still sums to the content, so no
    /// child is drawn outside the container.
    #[test]
    fn a_container_too_small_for_its_minimums_splits_equally_and_still_fits() {
        let spans = resolve_spans(&[5.0, 1.0], 60.0, 100.0, 0.0);
        assert!(close(spans[0], 30.0), "{spans:?}");
        assert!(close(spans[1], 30.0), "{spans:?}");
        assert!(close(spans.iter().sum::<f32>(), 60.0), "nothing overflows");
    }

    /// **Failure mode #6, asserted directly: resolving is idempotent
    /// under a round trip through a narrow window.**
    #[test]
    fn resolving_is_idempotent_under_a_round_trip_through_a_narrow_window() {
        let shares = [3.0_f32, 1.0, 2.0];
        let wide = resolve_spans(&shares, 900.0, MIN_COLUMN_WIDTH, SPLITTER_THICKNESS);
        let _narrow = resolve_spans(&shares, 200.0, MIN_COLUMN_WIDTH, SPLITTER_THICKNESS);
        let wide_again = resolve_spans(&shares, 900.0, MIN_COLUMN_WIDTH, SPLITTER_THICKNESS);
        assert_eq!(
            wide, wide_again,
            "the narrow pass perturbed the wide result"
        );
    }

    /// Degenerate inputs do not produce degenerate output: a `NaN` total
    /// yields zero-width children rather than `NaN` rectangles, which
    /// `egui` would paint in unpredictable places.
    #[test]
    fn non_finite_inputs_collapse_to_zero_rather_than_propagating() {
        let spans = resolve_spans(&[1.0, 1.0], f32::NAN, 10.0, 0.0);
        assert!(spans.iter().all(|s| s.is_finite()), "{spans:?}");
        let spans = resolve_spans(&[f32::NAN, 1.0], 200.0, 10.0, 0.0);
        assert!(spans.iter().all(|s| s.is_finite() && *s > 0.0), "{spans:?}");
    }

    /// An empty share list resolves to an empty span list rather than
    /// dividing by zero.
    #[test]
    fn no_children_resolves_to_no_spans() {
        assert!(resolve_spans(&[], 100.0, 10.0, 3.0).is_empty());
    }

    // -----------------------------------------------------------------
    // drag_boundary — failure mode #7
    // -----------------------------------------------------------------

    /// **Failure mode #7, asserted directly: a splitter affects its two
    /// neighbours only.**
    #[test]
    fn a_splitter_moves_exactly_two_neighbours_and_no_others() {
        let mut spans = [100.0_f32, 100.0, 100.0, 100.0];
        let before = spans;
        drag_boundary(&mut spans, 0, 30.0, 20.0);
        assert!(close(spans[0], 130.0));
        assert!(close(spans[1], 70.0));
        assert_eq!(spans[2], before[2], "a distant column moved");
        assert_eq!(spans[3], before[3], "a distant column moved");
    }

    /// The total is preserved exactly, which is why no renormalisation
    /// pass is needed — and a renormalisation pass is precisely what
    /// couples splitters together.
    #[test]
    fn a_drag_preserves_the_total() {
        let mut spans = [80.0_f32, 120.0, 200.0];
        let before: f32 = spans.iter().sum();
        drag_boundary(&mut spans, 1, -45.0, 20.0);
        assert!(close(spans.iter().sum::<f32>(), before));
    }

    /// A drag is clamped by BOTH neighbours' minimums, in both
    /// directions.
    #[test]
    fn a_drag_is_clamped_by_both_neighbours_minimums() {
        let mut spans = [100.0_f32, 100.0];
        let applied = drag_boundary(&mut spans, 0, 500.0, 40.0);
        assert!(close(spans[1], 40.0), "the right neighbour hit its floor");
        assert!(
            close(applied, 60.0),
            "the applied delta is reported: {applied}"
        );

        let mut spans = [100.0_f32, 100.0];
        drag_boundary(&mut spans, 0, -500.0, 40.0);
        assert!(close(spans[0], 40.0), "the left neighbour hit its floor");
    }

    /// A stale boundary index — a splitter dragged in the frame a panel
    /// was closed — is a no-op, never a panic.
    #[test]
    fn a_stale_boundary_index_is_a_no_op() {
        let mut spans = [100.0_f32, 100.0];
        assert_eq!(drag_boundary(&mut spans, 1, 10.0, 10.0), 0.0);
        assert_eq!(drag_boundary(&mut spans, 9, 10.0, 10.0), 0.0);
        assert_eq!(spans, [100.0, 100.0]);
        assert_eq!(drag_boundary(&mut [], 0, 10.0, 10.0), 0.0);
    }

    /// Shares round-trip through spans: dragging then re-resolving at the
    /// same total reproduces the dragged spans.
    #[test]
    fn a_dragged_layout_reproduces_itself_when_resolved_again() {
        let shares = [1.0_f32, 1.0, 1.0];
        let total = 306.0;
        let mut spans = resolve_spans(&shares, total, 20.0, 3.0);
        drag_boundary(&mut spans, 0, 40.0, 20.0);
        let new_shares = spans_to_shares(&spans);
        let again = resolve_spans(&new_shares, total, 20.0, 3.0);
        for (a, b) in spans.iter().zip(&again) {
            assert!(close(*a, *b), "{spans:?} vs {again:?}");
        }
    }

    // -----------------------------------------------------------------
    // Tab planning — failure mode #8
    // -----------------------------------------------------------------

    /// Tabs are floored and capped, so an empty label still has a
    /// grabbable tab and a very long one cannot evict its siblings.
    #[test]
    fn a_tab_is_floored_and_capped() {
        assert!(close(tab_width(0.0), MIN_TAB_WIDTH));
        assert!(close(tab_width(60.0), 60.0 + TAB_PADDING));
        assert!(close(tab_width(9000.0), MAX_TAB_WIDTH));
    }

    /// Nothing overflows, nothing is reserved. An overflow control that
    /// took space when it had nothing to show would tax every dock in the
    /// application permanently.
    #[test]
    fn everything_fitting_reserves_nothing() {
        let widths = [60.0_f32, 60.0];
        let plan = plan_tabs(&widths, 0, 400.0, TAB_GAP, 70.0);
        assert_eq!(plan.shown, 2);
        assert_eq!(plan.hidden, 0);
        assert_eq!(plan.overflow_width, 0.0);
        assert!(!plan.has_overflow());
    }

    /// **The reservation is exact and biconditional**: reserved
    /// whenever something is hidden, never when nothing is.
    #[test]
    fn the_overflow_affordance_is_reserved_exactly_when_it_is_needed() {
        let widths = [100.0_f32; 6];
        for available in [50.0_f32, 120.0, 260.0, 400.0, 620.0, 640.0, 2000.0] {
            let plan = plan_tabs(&widths, 0, available, TAB_GAP, 70.0);
            assert_eq!(
                plan.hidden > 0,
                plan.overflow_width > 0.0,
                "at {available} pt: hidden={} reserved={}",
                plan.hidden,
                plan.overflow_width
            );
        }
    }

    /// **Failure mode #8: the visible tabs plus the reservation never
    /// exceed the bar.**
    #[test]
    fn the_visible_tabs_never_encroach_on_the_reservation() {
        let widths = [90.0_f32, 70.0, 130.0, 55.0, 160.0, 44.0, 120.0];
        let reserved = 70.0_f32;
        for available in (40..900).step_by(7).map(|w| w as f32) {
            for active in 0..widths.len() {
                let plan = plan_tabs(&widths, active, available, TAB_GAP, reserved);
                if !plan.has_overflow() {
                    continue;
                }
                let used: f32 = widths[plan.start..plan.start + plan.shown]
                    .iter()
                    .sum::<f32>()
                    + TAB_GAP * plan.shown.saturating_sub(1) as f32;
                assert!(
                    used <= plan.tab_budget + 0.01,
                    "at {available} pt with active {active}: tabs used {used} of a \
                     {} budget",
                    plan.tab_budget
                );

                if available < reserved + TAB_GAP {
                    // The bar is narrower than the affordance ITSELF.
                    //
                    // There is no assignment in which everything fits, so
                    // the question becomes *what gives way* — and the
                    // answer this module exists to enforce is: the tabs.
                    // `shown == 0`, the affordance takes the whole bar and
                    // is truncated by the renderer rather than displaced,
                    // and every panel stays reachable through the menu.
                    // Asserting a sum that fits here would be asserting
                    // the impossible; asserting this is asserting the
                    // design rule.
                    assert_eq!(
                        plan.shown, 0,
                        "at {available} pt a tab was drawn beside an affordance that \
                         does not itself fit"
                    );
                    continue;
                }
                assert!(
                    used + TAB_GAP + plan.overflow_width <= available + 0.01,
                    "at {available} pt with active {active}: {used} + gap + {} > {available}",
                    plan.overflow_width
                );
            }
        }
    }

    /// **The active tab is always in the visible window** whenever the
    /// window is non-empty.
    #[test]
    fn the_active_tab_is_never_the_one_that_gets_hidden() {
        let widths = [90.0_f32, 70.0, 130.0, 55.0, 160.0, 44.0, 120.0];
        for available in (60..900).step_by(5).map(|w| w as f32) {
            for active in 0..widths.len() {
                let plan = plan_tabs(&widths, active, available, TAB_GAP, 70.0);
                if plan.shown == 0 {
                    continue;
                }
                assert!(
                    plan.is_visible(active),
                    "at {available} pt the active tab {active} fell outside the \
                     window {}..{}",
                    plan.start,
                    plan.start + plan.shown
                );
            }
        }
    }

    /// The visible window is contiguous and the counts add up — no tab is
    /// both shown and hidden, and none is neither.
    #[test]
    fn every_tab_is_either_visible_or_in_the_menu_exactly_once() {
        let widths = [90.0_f32, 70.0, 130.0, 55.0, 160.0];
        for available in (40..600).step_by(3).map(|w| w as f32) {
            let plan = plan_tabs(&widths, 3, available, TAB_GAP, 70.0);
            assert_eq!(plan.shown + plan.hidden, widths.len());
            assert!(plan.start + plan.shown <= widths.len());
        }
    }

    /// **At a width narrower than the reservation, the bar degrades to
    /// the affordance alone — the affordance is never what is squeezed
    /// out.**
    #[test]
    fn a_bar_narrower_than_its_reservation_keeps_the_affordance_and_drops_the_tabs() {
        let widths = [100.0_f32; 8];
        let plan = plan_tabs(&widths, 5, 60.0, TAB_GAP, 70.0);
        assert_eq!(plan.shown, 0, "no tab can fit beside the affordance");
        assert_eq!(plan.hidden, 8, "every tab is reachable from the menu");
        assert!(plan.has_overflow());
        assert!(
            close(plan.overflow_width, 70.0),
            "the affordance kept its width"
        );
    }

    /// A stack with one tab and a bar too small for it still yields a
    /// plan, not a panic or an empty menu.
    #[test]
    fn a_single_tab_too_wide_for_its_bar_goes_to_the_menu() {
        let plan = plan_tabs(&[300.0], 0, 100.0, TAB_GAP, 70.0);
        assert_eq!(plan.shown, 0);
        assert_eq!(plan.hidden, 1);
        assert!(plan.has_overflow());
    }

    /// An `active` index from a stale or hand-edited layout file is
    /// clamped, not trusted — fail-soft reaches the arithmetic too.
    #[test]
    fn an_out_of_range_active_index_is_clamped_rather_than_panicking() {
        let plan = plan_tabs(&[80.0, 80.0], 99, 60.0, TAB_GAP, 70.0);
        assert!(plan.shown == 0 || plan.is_visible(1));
    }

    /// An unbounded available width does not silently disable overflow.
    #[test]
    fn an_infinite_available_width_is_treated_as_none_at_all() {
        let plan = plan_tabs(&[100.0; 5], 0, f32::INFINITY, TAB_GAP, 70.0);
        assert_eq!(plan.shown, 0, "infinity is not a width");
        assert_eq!(plan.hidden, 5);
    }

    /// A stack with no tabs plans nothing and reserves nothing.
    #[test]
    fn an_empty_stack_plans_nothing() {
        let plan = plan_tabs(&[], 0, 300.0, TAB_GAP, 70.0);
        assert_eq!(plan.shown, 0);
        assert_eq!(plan.hidden, 0);
        assert!(!plan.has_overflow());
    }

    /// The reservation is the widest label the control can EVER show, not
    /// the label for the largest count.
    #[test]
    fn the_reservation_covers_the_widest_reachable_label_not_the_longest() {
        let measure = |s: &str| if s.contains('8') { 200.0 } else { 40.0 };
        let w = overflow_width(9, 8.0, measure);
        assert!(
            close(w, 208.0),
            "reserved {w}, but the control can show a 200 pt label"
        );
    }

    // -----------------------------------------------------------------
    // Failure mode #3, and the 1280-point budget of failure mode #4
    // -----------------------------------------------------------------

    /// **Failure mode #3: no minimum in this module is a function of a
    /// tab label.**
    #[test]
    fn the_minimum_column_width_ignores_tab_labels_entirely() {
        let modest = tab_width(40.0);
        let preposterous = tab_width(4000.0);
        assert!(preposterous > modest, "the tab itself does get wider");
        // …and yet:
        assert_eq!(MIN_COLUMN_WIDTH, 140.0);
        assert!(
            preposterous <= MAX_TAB_WIDTH,
            "even one tab is capped, so a bar cannot be monopolised"
        );
        // The container minimum is what a resize drag is clamped against,
        // and it is the same number regardless of what is in the stack.
        let mut spans = [MIN_COLUMN_WIDTH + 100.0, MIN_COLUMN_WIDTH + 100.0];
        drag_boundary(&mut spans, 0, -1000.0, MIN_COLUMN_WIDTH);
        assert!(close(spans[0], MIN_COLUMN_WIDTH));
    }

    /// **Failure mode #4, budgeted and tested at 1280 points wide.**
    #[test]
    fn both_docks_at_their_minimum_leave_most_of_a_1280_point_window() {
        let both = MIN_SIDE_WIDTH * 2.0;
        assert!(
            both <= 1280.0 * 0.3,
            "two docks at minimum take {both} pt of 1280, over 30 %"
        );
    }

    /// The presentation clamp keeps a huge restored width usable without
    /// the model ever learning about it.
    #[test]
    fn the_side_clamp_leaves_the_application_the_majority_of_the_window() {
        let fraction = MAX_SIDE_FRACTION;
        assert!(
            fraction > 0.0 && fraction < 0.5,
            "a clamp at {fraction} of the window is not a clamp"
        );
        let two_docks = 1280.0 * fraction * 2.0;
        assert!(
            two_docks < 1280.0,
            "two fully-expanded docks would leave no application"
        );
    }

    /// At [`MIN_COLUMN_WIDTH`] the tab bar still holds the overflow
    /// affordance plus at least one tab, so the narrowest reachable dock
    /// is still navigable rather than being a lone chevron.
    #[test]
    fn the_minimum_column_width_still_admits_the_overflow_affordance() {
        // The affordance at its floor, plus a minimum tab, plus the gap.
        let needed = MIN_TAB_WIDTH * 2.0 + TAB_GAP;
        assert!(
            MIN_COLUMN_WIDTH >= needed,
            "a column pinned at {MIN_COLUMN_WIDTH} cannot hold {needed} pt of bar"
        );
        let plan = plan_tabs(&[70.0; 5], 0, MIN_COLUMN_WIDTH, TAB_GAP, MIN_TAB_WIDTH);
        assert!(plan.shown >= 1, "the narrowest column shows no tab at all");
        assert!(plan.has_overflow());
    }
}
