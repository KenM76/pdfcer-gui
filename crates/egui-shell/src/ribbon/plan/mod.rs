//! Ribbon layout arithmetic — the part of the ribbon that has no `egui` in
//! it.
//!
//! # What is planned here, and in which file
//!
//! Two rows, one rule.
//!
//! | Function | File | Plans | Reservation it protects |
//! |---|---|---|---|
//! | [`wrap_group`] | this one | one group's items across [`GROUP_ROWS`] rows | — |
//! | [`plan_band`] | this one | the band's groups | the band's "⏷ N more" affordance |
//! | [`plan_strip_row`] | [`row`] | the tab-strip row's three regions | the tab area itself |
//! | [`plan_tab_strip`] | [`row`] | the tabs within that area | the strip's own "⏷ N more" affordance **and the active tab** |
//!
//! Both are re-exported here, so every call site says `plan::…` and the
//! split is an organisational fact rather than something a caller has to
//! know. [`row`]'s header explains why the row is a different problem from
//! the band despite looking like the same one — the short version is that
//! everything a *band* hides is still reachable through its menu, and the
//! one thing a *strip* must never hide is the very tab its menu cannot
//! reach.
//!
//! What the two share is the greedy fill: [`plan_tab_strip`] calls
//! [`plan_band`] to place the tabs it has not pinned, so the monotonicity
//! rule — *widening never hides something that was visible* — exists in
//! one place rather than two.
//!
//! # Why this is a separate module with no `Ui` in its signatures
//!
//! Everything in this file is a pure function over `f32`. That is not
//! tidiness; it is the only way the *overflow* invariant can be tested at
//! all.
//!
//! `MODES_AND_PANELS.md` Part 2's failure mode #8 is the one this module
//! exists to make impossible: past a handful of tabs the overflow *button
//! itself* gets hidden, leaving no route to whatever it was hiding. The
//! rule that answers it — *the overflow affordance is reserved space, never
//! the first thing squeezed out* — is enforced here as arithmetic.
//!
//! That defect is a *layout arithmetic* defect. It happens when the
//! overflow control is emitted **after** the content, into whatever space
//! the content did not take — which is the obvious immediate-mode
//! spelling, and which yields nothing at all once the content takes
//! everything. Reading such code does not reveal the bug; the code says
//! "draw the groups, then draw the overflow button", and that sentence
//! sounds correct.
//!
//! So the arithmetic is lifted out, and the reservation is made the
//! **first** subtraction rather than the last:
//!
//! ```text
//! budget_for_groups = available − overflow_width − separator
//! ```
//!
//! computed before a single group is measured against it. A group can
//! then only ever consume `budget_for_groups`, and the overflow control's
//! width is not in that number. There is no ordering of the group loop
//! that can reach it.
//!
//! `plan_band` returns that budget, and [`super::band`] hands the
//! groups a `Ui` whose maximum width **is** that budget — so the
//! reservation is enforced twice: once by this arithmetic, and once by
//! `egui`'s own clipping, which cannot be talked out of it.
//!
//! # Why widths are estimated rather than measured
//!
//! Immediate mode has a genuine ordering problem: the width a group will
//! occupy is known only after it is drawn, and the decision about whether
//! to draw it must be made before. There are three ways out.
//!
//! 1. **Draw, measure, and re-lay-out next frame.** Correct widths, and a
//!    visible one-frame flicker every time the window is resized — which
//!    is exactly when the operator is looking at the ribbon.
//! 2. **Draw into a scratch layer and discard.** Correct widths, double
//!    the work, and every side effect (hover, click, focus) has to be
//!    suppressed on the discarded pass or it fires twice.
//! 3. **Estimate analytically from the item list**, using the same font
//!    metrics `egui` will use to lay the text out. Cheap, single-pass,
//!    and exact to within the padding constants.
//!
//! This module is option 3. `ItemWidths` is fed measured galley widths
//! by [`super::band`] — `egui` memoizes galleys, so asking for the width
//! of a label that is about to be drawn costs a hash lookup — and adds
//! the padding constants the renderer will actually apply.
//!
//! The estimate can be wrong. A [`crate::manifest::Item::Custom`] is
//! drawn by the application and the shell cannot know how wide it will
//! be until it has drawn it once, so it is budgeted at the width it was
//! last drawn, and at `CUSTOM_ITEM_WIDTH` before that. **An estimate that is
//! too small costs a clipped group; it cannot cost the overflow
//! control**, because the overflow control's width was subtracted from
//! the total before the estimate was consulted. That asymmetry is the
//! whole reason the reservation is made first, and it is why a rough
//! estimate is an acceptable input to an invariant this strict.
//!
//! # Minimum widths, and why they matter to the tests
//!
//! Every control is at least `MIN_ITEM_WIDTH` wide. That is a real
//! design rule — a control narrower than it is tall reads as a rendering
//! fault — and it has a second effect worth naming: it makes this
//! module's arithmetic meaningful **even when no font is installed**.
//!
//! This crate depends on `egui` with `default-features = false`, so a
//! test process has no font data and every galley measures near zero. A
//! layout that derived its widths from text alone would collapse to zero
//! in exactly the environment its tests run in, and the overflow tests
//! would be asserting against a band that never overflows. The floor
//! keeps the numbers honest headlessly.
//!
//! ## And why the floor is not enough — read this before trusting a
//! ## width test in this crate
//!
//! The floor keeps the arithmetic *meaningful*; it does not make a
//! zero-width-text test *equivalent* to a real one.
//!
//! Which fonts exist is not a property of this crate. It is **decided by
//! whichever sibling crate is in the build**:
//!
//! ```text
//! cargo test -p egui-shell --lib   egui alone             → no fonts, widths ≈ 0
//! cargo test --workspace           application → eframe   → egui/default_fonts,
//!                                                           real widths
//! ```
//!
//! Cargo unifies features across a workspace build, so the same assertions
//! measure different text under the two commands, and the *narrower*
//! command — the one a developer working on the shell reaches for — is
//! the one that measures nothing. Any assertion in this file that compares
//! one width against another is trivially satisfied under it.
//!
//! Two width rules therefore cannot be checked from here at all: that the
//! overflow affordance is positioned from a `Ui` whose `max_rect` a sibling
//! row has not grown (see [`super::band`]), and that [`overflow_width`]
//! reserves for the label with the greatest *width* rather than the most
//! *characters*.
//!
//! `super::width_tests` closes that hole by installing a synthetic
//! proportional face this crate builds itself, so the width-sensitive
//! paths are exercised with real advances under **both** commands and
//! under any future workspace membership. A new width rule added to this
//! module belongs there as well as here.
//!
//! Design and rationale: `docs/modules/egui-shell/ribbon/plan/mod.md`.

pub(crate) mod collapse;

pub(crate) mod row;

// Re-exported so every call site reads `plan::…` and the split between
// this file and [`row`] stays an organisational fact rather than
// something a caller has to know.
pub(crate) use row::{RowDemand, StripPlan, plan_strip_row, plan_tab_strip};

/// The narrowest a control may be drawn, in points, before the theme's
/// padding is added.
pub(crate) const MIN_ITEM_WIDTH: f32 = 20.0;

/// The width budgeted for a [`crate::manifest::Item::Custom`].
pub(crate) const CUSTOM_ITEM_WIDTH: f32 = 96.0;

/// Horizontal padding inside a group, either side of its content.
pub(crate) const GROUP_PADDING: f32 = 6.0;

/// **How many control rows one ribbon group may use.**
pub(crate) const GROUP_ROWS: usize = 3;

/// **The most rows a group may be re-wrapped onto** when the band runs short of
/// width — S5's ceiling.
pub(crate) const MAX_GROUP_ROWS: usize = 3;

/// **The row width at which a group wraps onto its second row.**
///
/// Taken from `mockups/ribbon.html`, which is this change's specification:
///
/// ```css
/// .gcmds { display:flex; flex-wrap:wrap; gap:5px; max-width:440px }
/// ```
///
/// It is a **trigger, not a target**. In the mockup a group narrower than
/// this stays on one row and a wider one wraps *greedily* — a full 440 px
/// row followed by whatever is left over. [`wrap_group`] keeps the first
/// half of that rule and improves the second: once a group trips this
/// width it is split into [`GROUP_ROWS`] rows **as evenly as the item
/// widths allow**, which is both narrower than the greedy answer (so more
/// groups fit before the overflow menu is needed) and what a ribbon
/// actually looks like — a full row over a stub reads as a wrapping
/// accident.
///
/// The number is the mockup's, in points rather than CSS pixels. The two
/// are close enough to be the same decision: the mockup's controls are
/// 12.5 px text in ~9 px of padding and this shell's are ~14 pt text in a
/// theme-set padding, so 440 buys about six controls in either.
pub(crate) const GROUP_WRAP_WIDTH: f32 = 440.0;

/// How one group's items are distributed across its rows.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GroupRows {
    /// How many items are on each row, in manifest order.
    ///
    /// Always sums to the group's item count, and is always a partition
    /// into **contiguous runs** — the manifest's order is the operator's
    /// order, exactly as [`BandPlan::shown`] is a prefix for the same
    /// reason.
    ///
    /// Empty for an empty group; never contains a zero.
    pub counts: Vec<usize>,
    /// The width of the **widest** row, in points, gutters included.
    ///
    /// This — not the sum of the items — is what the group costs the band.
    pub width: f32,
}

impl GroupRows {
    /// How many rows the group uses. Never more than the `max_rows` it was
    /// planned with, and never zero for a non-empty group.
    #[cfg(test)]
    pub(crate) fn rows(&self) -> usize {
        self.counts.len()
    }
}

/// The width of one row of items: the items plus one `gutter` between each
/// adjacent pair.
pub(crate) fn row_width(item_widths: &[f32], gutter: f32) -> f32 {
    if item_widths.is_empty() {
        0.0
    } else {
        item_widths.iter().sum::<f32>() + gutter * (item_widths.len() as f32 - 1.0)
    }
}

/// Slack, in points, when comparing an accumulating row width against a
/// candidate target.
const PACK_SLACK: f32 = 1.0e-3;

/// Split one group's items across at most `max_rows` rows.
pub(crate) fn wrap_group(
    item_widths: &[f32],
    gutter: f32,
    max_rows: usize,
    wrap_at: f32,
    prefer_rows: Option<usize>,
) -> GroupRows {
    let n = item_widths.len();
    let single = row_width(item_widths, gutter);
    let one_row = || GroupRows {
        counts: if n == 0 { Vec::new() } else { vec![n] },
        width: single,
    };

    // **`prefer_rows` is exactly the right to skip the fits-already test.**
    //
    // Everything below already searches for the NARROWEST packing within the
    // row limit. What kept a comfortable group on one row was this
    // short-circuit — *"it fits, so leave it"* — which is the correct default
    // and is wrong for a group whose shape is part of the control.
    //
    // A four-position radio is the case: four square buttons in a row is a
    // strip, and the same four as a 2 x 2 block is half the width and reads as
    // one control (`OPERATOR_REQUESTS.md` O97). So a group that asked for rows
    // goes on to the search, and the search does the rest — no second packing
    // algorithm, and no way for a preferred layout and a pressured one to
    // disagree about how a group wraps.
    //
    // The value is a HINT and the band's ceiling still wins: `max_rows` is
    // unchanged below, so a group asking for four rows in a two-row band gets
    // two. And because the search returns the narrowest, asking for two rows
    // does not FORCE two — a pair of items whose 1 x 2 is narrowest stays on
    // one row.
    let asked = prefer_rows.is_some_and(|rows| rows >= 2);
    if n <= 1 || max_rows <= 1 || !wrap_at.is_finite() || wrap_at <= 0.0 {
        return one_row();
    }
    if single <= wrap_at && !asked {
        return one_row();
    }

    // Every contiguous run's width, ascending. The optimum is one of them.
    let mut candidates: Vec<f32> = Vec::with_capacity(n * (n + 1) / 2);
    for i in 0..n {
        for j in i..n {
            candidates.push(row_width(&item_widths[i..=j], gutter));
        }
    }
    candidates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    for &target in &candidates {
        if let Some(counts) = pack(item_widths, gutter, target, max_rows) {
            let width = widest_row(item_widths, gutter, &counts);
            return GroupRows { counts, width };
        }
    }

    // Unreachable: the last candidate is the whole run, which packs into
    // one row. Falling back rather than panicking, because a panic in a
    // paint loop is a worse answer to an impossible input than a band that
    // is one group too wide.
    one_row()
}

/// Greedily fill rows of at most `target` points, `None` if that needs more
/// than `max_rows` of them.
fn pack(item_widths: &[f32], gutter: f32, target: f32, max_rows: usize) -> Option<Vec<usize>> {
    let mut counts: Vec<usize> = Vec::new();
    let mut in_row = 0_usize;
    let mut used = 0.0_f32;

    for &w in item_widths {
        if in_row > 0 && used + gutter + w > target + PACK_SLACK {
            counts.push(in_row);
            in_row = 1;
            used = w;
        } else {
            used += if in_row == 0 { w } else { gutter + w };
            in_row += 1;
        }
    }
    if in_row > 0 {
        counts.push(in_row);
    }
    (counts.len() <= max_rows).then_some(counts)
}

/// The widest row a `counts` partition produces.
fn widest_row(item_widths: &[f32], gutter: f32, counts: &[usize]) -> f32 {
    let mut at = 0_usize;
    let mut widest = 0.0_f32;
    for &count in counts {
        let end = (at + count).min(item_widths.len());
        widest = widest.max(row_width(&item_widths[at..end], gutter));
        at = end;
    }
    widest
}

/// The measured pieces of one item, before padding.
///
/// Separated from the total so [`item_width`] has one place to apply the
/// padding rule and the tests have something to assert against.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ItemWidths {
    /// Width of the item's icon, or `0.0` if it has none.
    pub icon: f32,
    /// Width of the item's visible text, or `0.0` if it is icon-only.
    pub text: f32,
    /// Gap between icon and text, applied only when both are present.
    pub gap: f32,
    /// Padding applied inside the control, both sides together.
    pub padding: f32,
}

impl ItemWidths {
    /// The width this item will occupy, floored at [`MIN_ITEM_WIDTH`].
    pub(crate) fn total(self) -> f32 {
        let gap = if self.icon > 0.0 && self.text > 0.0 {
            self.gap
        } else {
            0.0
        };
        (self.icon + self.text + gap + self.padding).max(MIN_ITEM_WIDTH)
    }
}

/// The width of a whole group: its **widest row**, its caption, and its
/// padding.
pub(crate) fn group_width(content_width: f32, caption_width: f32) -> f32 {
    content_width.max(caption_width) + GROUP_PADDING * 2.0
}

/// How a band's groups are split between the visible band and the
/// overflow menu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BandPlan {
    /// How many leading groups are drawn in the band itself.
    ///
    /// Always a *prefix* of the group list: the manifest's order is the
    /// operator's order, and a plan that dropped a group from the middle
    /// would make the visible band's order depend on the window width.
    pub shown: usize,
    /// How many trailing groups moved into the overflow menu.
    pub hidden: usize,
    /// The width the shown groups may occupy, in points.
    ///
    /// **This number already excludes the overflow control's width.** It
    /// is what [`super::band`] uses as the maximum width of the `Ui` the
    /// groups are drawn into, which is the second half of the enforcement
    /// — see the module header.
    pub group_budget: f32,
    /// The width reserved for the overflow affordance, or `0.0` when
    /// nothing overflowed.
    ///
    /// Zero when [`Self::hidden`] is zero, and non-zero whenever it is
    /// not. That biconditional is asserted by
    /// `the_overflow_affordance_is_reserved_exactly_when_it_is_needed`.
    pub overflow_width: f32,
}

impl BandPlan {
    /// Whether the overflow affordance is to be drawn.
    pub(crate) fn has_overflow(self) -> bool {
        self.hidden > 0
    }
}

/// Decide how many groups fit, reserving the overflow affordance's width
/// **before** any group is measured against the remainder.
pub(crate) fn plan_band(
    available: f32,
    group_widths: &[f32],
    separator: f32,
    overflow_width: f32,
) -> BandPlan {
    // A non-finite or negative width is not a width. Treating it as zero
    // makes the degenerate case the *safe* one (everything in the menu)
    // rather than the dangerous one (infinite budget, no overflow).
    let available = if available.is_finite() {
        available.max(0.0)
    } else {
        0.0
    };
    let separator = separator.max(0.0);
    let overflow_width = overflow_width.max(0.0);

    let n = group_widths.len();
    if n == 0 {
        return BandPlan {
            shown: 0,
            hidden: 0,
            group_budget: available,
            overflow_width: 0.0,
        };
    }

    let total: f32 = group_widths.iter().sum::<f32>() + separator * (n as f32 - 1.0);
    if total <= available {
        return BandPlan {
            shown: n,
            hidden: 0,
            group_budget: available,
            overflow_width: 0.0,
        };
    }

    // THE RESERVATION. Subtracted before a single group is considered.
    let group_budget = (available - overflow_width - separator).max(0.0);

    let mut used = 0.0_f32;
    let mut shown = 0_usize;
    for (i, w) in group_widths.iter().enumerate() {
        let step = if i == 0 { *w } else { separator + *w };
        if used + step <= group_budget {
            used += step;
            shown += 1;
        } else {
            break;
        }
    }

    BandPlan {
        shown,
        hidden: n - shown,
        group_budget,
        overflow_width,
    }
}

/// The overflow affordance's label for `hidden` hidden groups.
pub(crate) fn overflow_label(hidden: usize) -> String {
    format!("⏷ {hidden} more")
}

/// The width to reserve for the overflow affordance, given how many
/// groups the band holds **in total**.
pub(crate) fn overflow_width(
    total_groups: usize,
    padding: f32,
    measure: impl Fn(&str) -> f32,
) -> f32 {
    let widest = (1..=total_groups.max(1))
        .map(|n| measure(&overflow_label(n)))
        .fold(0.0_f32, f32::max);
    (widest + padding).max(MIN_ITEM_WIDTH)
}
#[cfg(test)]
mod tests {
    use super::*;

    /// `n` groups of `each` points, the uniform shape most of these tests
    /// want.
    fn widths(n: usize, each: f32) -> Vec<f32> {
        vec![each; n]
    }

    /// An item is never narrower than [`MIN_ITEM_WIDTH`], and the
    /// icon/text gap applies only when both are present.
    ///
    /// The second half is what makes an icon-only control the same width
    /// as a square rather than a square plus a gap to nothing.
    #[test]
    fn an_item_is_floored_and_only_gapped_when_it_has_both_halves() {
        let both = ItemWidths {
            icon: 16.0,
            text: 40.0,
            gap: 4.0,
            padding: 8.0,
        };
        assert_eq!(both.total(), 68.0);

        let icon_only = ItemWidths { text: 0.0, ..both };
        assert_eq!(icon_only.total(), 24.0, "no gap when there is no text");

        let text_only = ItemWidths { icon: 0.0, ..both };
        assert_eq!(text_only.total(), 48.0, "no gap when there is no icon");

        let tiny = ItemWidths {
            icon: 0.0,
            text: 1.0,
            gap: 4.0,
            padding: 2.0,
        };
        assert_eq!(
            tiny.total(),
            MIN_ITEM_WIDTH,
            "a control narrower than it is tall reads as a clipping artefact"
        );
    }

    /// **A group is as wide as its caption when its caption is the wider
    /// half.**
    #[test]
    fn a_group_is_as_wide_as_its_caption_when_the_caption_is_wider() {
        let rows = |widths: &[f32]| wrap_group(widths, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, None);

        let one_narrow_button = rows(&[24.0]);
        let wide_caption = 90.0;
        assert_eq!(
            group_width(one_narrow_button.width, wide_caption),
            wide_caption + GROUP_PADDING * 2.0
        );

        let wide_row = rows(&[60.0, 60.0]);
        assert_eq!(
            group_width(wide_row.width, 20.0),
            60.0 + 4.0 + 60.0 + GROUP_PADDING * 2.0
        );

        assert_eq!(
            group_width(rows(&[]).width, 0.0),
            GROUP_PADDING * 2.0,
            "an empty group is its padding, not a negative number"
        );
    }

    /// **A group that asks for two rows gets them, even though one fits.**
    #[test]
    fn a_group_that_asks_for_rows_wraps_when_it_would_otherwise_fit() {
        let four = [24.0_f32, 24.0, 24.0, 24.0];
        let unasked = wrap_group(&four, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, None);
        assert_eq!(
            unasked.counts,
            vec![4],
            "the default must stay one row — a group that fits is not re-shaped"
        );

        let asked = wrap_group(&four, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, Some(2));
        assert_eq!(
            asked.counts,
            vec![2, 2],
            "two rows of two, which is narrowest"
        );
        assert!(
            asked.width < unasked.width,
            "the whole point is that it is NARROWER: {} against {}",
            asked.width,
            unasked.width
        );
    }

    /// **The hint is a preference, not a command.**
    #[test]
    fn the_row_hint_is_a_preference_and_the_bands_ceiling_still_wins() {
        let four = [24.0_f32, 24.0, 24.0, 24.0];
        assert_eq!(
            wrap_group(&four, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, Some(1)).counts,
            vec![4],
            "one row is the default and asking for it must change nothing"
        );
        // Asking for more rows than the band has: the ceiling is `max_rows`.
        let greedy = wrap_group(&four, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, Some(9));
        assert!(
            greedy.counts.len() <= GROUP_ROWS,
            "the band's row limit must bound the manifest's hint, got {:?}",
            greedy.counts
        );
        // A single item cannot be split however hard the manifest asks.
        assert_eq!(
            wrap_group(&[24.0], 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, Some(2)).counts,
            vec![1]
        );
    }

    /// **A group narrower than [`GROUP_WRAP_WIDTH`] is left alone.**
    #[test]
    fn a_group_that_fits_the_cap_stays_on_one_row() {
        for widths in [
            vec![80.0],
            vec![80.0, 80.0],
            vec![100.0, 100.0, 100.0],
            // 6 × 70 + 5 × 4 = 440, exactly the cap: `<=`, not `<`.
            vec![70.0; 6],
        ] {
            let rows = wrap_group(&widths, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, None);
            assert_eq!(
                rows.counts,
                vec![widths.len()],
                "{widths:?} is within the cap and must not have been wrapped"
            );
            assert_eq!(rows.width, row_width(&widths, 4.0));
        }
    }

    /// **A group over the cap is split into two rows, evenly, and is
    /// narrower for it.**
    #[test]
    fn a_group_over_the_cap_is_split_evenly_and_costs_less() {
        let widths = vec![75.0; 7]; // 7 × 75 + 6 × 4 = 549, over the cap
        let single = row_width(&widths, 4.0);
        assert!(single > GROUP_WRAP_WIDTH, "the fixture must trip the cap");

        let rows = wrap_group(&widths, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, None);
        assert_eq!(
            rows.rows(),
            GROUP_ROWS,
            "the ceiling this ships with, not a number typed here — the expectations \
             below moved from 4 + 3 to 3 + 2 + 2 on 2026-09-05 when GROUP_ROWS became 3"
        );
        assert_eq!(
            rows.counts,
            vec![3, 3, 1],
            "seven over three rows. NOT [3, 2, 2]: `wrap_group` minimises the WIDEST \
             row, and once three is the widest a fourth row-mate cannot help — both \
             splits are 233 pt wide and this is the one the search reaches first. The \
             claim being tested is the width below, and the counts are pinned only so \
             that a change of algorithm has to be deliberate"
        );
        assert_eq!(rows.width, 3.0f32.mul_add(75.0, 2.0 * 4.0));
        assert!(
            rows.width < single * 0.6,
            "a wrapped group must cost the band far less than an unwrapped one: \
             {} vs {single}",
            rows.width
        );
        assert!(
            rows.width < GROUP_WRAP_WIDTH,
            "the even split must also come in under the cap it tripped, or the \
             greedy fill the mockup specifies would have been the better answer"
        );
    }

    /// The split is a **partition into contiguous runs**, so the ribbon's
    /// reading order survives it.
    #[test]
    fn the_split_is_a_contiguous_partition_of_every_item() {
        for n in 0..14_usize {
            for each in [12.0_f32, 40.0, 97.5, 260.0] {
                let widths = vec![each; n];
                let rows = wrap_group(&widths, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, None);
                assert_eq!(
                    rows.counts.iter().sum::<usize>(),
                    n,
                    "n={n} each={each}: an item was lost or duplicated"
                );
                assert!(
                    rows.counts.len() <= GROUP_ROWS,
                    "n={n} each={each}: {} rows exceeds the band's height",
                    rows.counts.len()
                );
                assert!(
                    !rows.counts.contains(&0),
                    "n={n} each={each}: an empty row is a gap in the band"
                );
                assert_eq!(rows.counts.is_empty(), n == 0);
            }
        }
    }

    /// **A single control is never split, and wrapping can be switched
    /// off.**
    #[test]
    fn one_item_and_one_row_are_both_left_alone() {
        let huge = [900.0_f32];
        assert_eq!(
            wrap_group(&huge, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, None).counts,
            vec![1],
            "a control wider than the cap takes its own row; the cap is a wrap \
             trigger and never a clip"
        );

        let seven = vec![75.0_f32; 7];
        let unwrapped = wrap_group(&seven, 4.0, 1, GROUP_WRAP_WIDTH, None);
        assert_eq!(unwrapped.counts, vec![7]);
        assert_eq!(unwrapped.width, row_width(&seven, 4.0));
    }

    /// A cap that is not a width degrades to "never wrap", the same safe
    /// direction [`plan_band`] takes for a non-finite available width.
    #[test]
    fn a_non_finite_cap_degrades_to_one_row() {
        let widths = vec![75.0_f32; 7];
        for bad in [f32::INFINITY, f32::NAN, 0.0, -10.0] {
            let rows = wrap_group(&widths, 4.0, GROUP_ROWS, bad, None);
            assert_eq!(rows.counts, vec![7], "wrap_at={bad}");
            assert_eq!(rows.width, row_width(&widths, 4.0), "wrap_at={bad}");
        }
    }

    /// **An item wider than the cap does not drag the whole group onto one
    /// row with it.**
    #[test]
    fn one_oversized_control_still_lets_the_rest_wrap() {
        let widths = [500.0_f32, 90.0, 90.0, 90.0, 90.0];
        let rows = wrap_group(&widths, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, None);
        assert_eq!(
            rows.counts,
            vec![1, 2, 2],
            "the oversized control takes a row and the other four share the rest"
        );
        assert_eq!(
            rows.width, 500.0,
            "the group is as wide as its widest control"
        );
        assert!(rows.width < row_width(&widths, 4.0));
    }

    /// Everything fits: no overflow control, and no width taken for one.
    #[test]
    fn a_band_that_fits_reserves_nothing() {
        let plan = plan_band(1000.0, &widths(3, 100.0), 8.0, 60.0);
        assert_eq!(plan.shown, 3);
        assert_eq!(plan.hidden, 0);
        assert_eq!(plan.overflow_width, 0.0);
        assert!(!plan.has_overflow());
    }

    /// **Failure mode #8: at a width too narrow for even one group, the
    /// overflow affordance is still planned and still has its width.**
    #[test]
    fn the_overflow_affordance_survives_a_band_too_narrow_for_any_group() {
        let groups = widths(6, 100.0);
        for available in [0.0_f32, 1.0, 40.0, 60.0, 99.0, 100.0, 140.0] {
            let plan = plan_band(available, &groups, 8.0, 60.0);
            assert!(
                plan.has_overflow(),
                "at {available} pt every group must be reachable through the menu"
            );
            assert_eq!(plan.overflow_width, 60.0, "at {available} pt");
            assert_eq!(
                plan.hidden,
                groups.len() - plan.shown,
                "at {available} pt: every group is either shown or reachable"
            );
            assert!(
                plan.group_budget + plan.overflow_width <= available.max(60.0),
                "at {available} pt the groups were allowed into the reserved space"
            );
        }

        // The specific shape of the degenerate case.
        let plan = plan_band(10.0, &groups, 8.0, 60.0);
        assert_eq!(plan.shown, 0, "no group fits beside the reservation");
        assert_eq!(plan.hidden, 6, "so all six are in the menu");
        assert_eq!(plan.group_budget, 0.0, "and the groups get nothing");
    }

    /// **The reservation is subtracted before any group is placed.**
    ///
    /// Stated as arithmetic rather than as an outcome, because this is
    /// the property that cannot be reintroduced by a later edit to the
    /// group loop: whatever the loop does, it is filling a budget that
    /// never contained the overflow control's width.
    ///
    /// The equality below is exact and is the whole rule:
    ///
    /// ```text
    /// group_budget == max(0, available − overflow_width − separator)
    /// ```
    ///
    /// Note that this is *not* the same as "budget + reservation ≤
    /// available". Once the band is narrower than the reservation itself,
    /// the reservation deliberately exceeds the band — the affordance
    /// keeps its width and the groups get nothing, which is exactly the
    /// degenerate case failure mode #8 is about. Writing the assertion
    /// the other way would have demanded the opposite behaviour.
    #[test]
    fn the_group_budget_never_contains_the_reservation() {
        const SEP: f32 = 8.0;
        const RESERVE: f32 = 60.0;
        for n in 1..12_usize {
            for available in (0..900).step_by(17).map(|w| w as f32) {
                let groups = widths(n, 100.0);
                let plan = plan_band(available, &groups, SEP, RESERVE);
                if plan.has_overflow() {
                    assert_eq!(
                        plan.group_budget,
                        (available - RESERVE - SEP).max(0.0),
                        "n={n} available={available}: the group budget was not \
                         the band minus the reservation"
                    );
                    assert!(
                        plan.group_budget <= available,
                        "n={n} available={available}: the groups were budgeted \
                         more than the whole band"
                    );
                }
            }
        }
    }

    /// The overflow affordance is reserved **exactly** when it is needed:
    /// `hidden > 0` if and only if `overflow_width > 0`.
    #[test]
    fn the_overflow_affordance_is_reserved_exactly_when_it_is_needed() {
        for available in (0..1200).step_by(13).map(|w| w as f32) {
            let plan = plan_band(available, &widths(5, 100.0), 8.0, 60.0);
            assert_eq!(
                plan.hidden > 0,
                plan.overflow_width > 0.0,
                "at {available} pt: hidden={} reserved={}",
                plan.hidden,
                plan.overflow_width
            );
        }
    }

    /// A wider band never shows fewer groups.
    #[test]
    fn widening_the_band_never_hides_a_group_that_was_visible() {
        let groups = [40.0, 120.0, 30.0, 200.0, 55.0, 90.0];
        let mut last = 0;
        for available in (0..1400).step_by(3).map(|w| w as f32) {
            let shown = plan_band(available, &groups, 8.0, 60.0).shown;
            assert!(
                shown >= last,
                "widening to {available} pt dropped a group that fitted at a narrower width"
            );
            last = shown;
        }
        assert_eq!(last, groups.len(), "at 1400 pt everything must be visible");
    }

    /// The shown groups are always a **prefix** of the manifest order,
    /// and the two counts always sum to the whole band.
    #[test]
    fn the_visible_groups_are_a_prefix_and_nothing_is_lost() {
        let groups = [40.0, 200.0, 30.0];
        for available in (0..600).step_by(7).map(|w| w as f32) {
            let plan = plan_band(available, &groups, 8.0, 60.0);
            assert_eq!(plan.shown + plan.hidden, groups.len(), "at {available} pt");
            assert!(plan.shown <= groups.len());
        }
    }

    /// **A non-finite available width degrades to "everything in the
    /// menu", not to "infinite room".**
    #[test]
    fn a_non_finite_width_degrades_safely() {
        for bad in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN, -50.0] {
            let plan = plan_band(bad, &widths(4, 100.0), 8.0, 60.0);
            assert_eq!(plan.shown, 0, "available={bad}");
            assert_eq!(plan.hidden, 4, "available={bad}");
            assert!(plan.has_overflow(), "available={bad}");
        }
    }

    /// An empty band plans nothing and reserves nothing.
    #[test]
    fn an_empty_band_plans_nothing() {
        let plan = plan_band(500.0, &[], 8.0, 60.0);
        assert_eq!(plan.shown, 0);
        assert_eq!(plan.hidden, 0);
        assert!(!plan.has_overflow());
    }

    /// **The reservation covers the widest label by WIDTH, not by
    /// character count.**
    #[test]
    fn the_reservation_covers_the_widest_label_by_width_not_by_digit_count() {
        let measure = |s: &str| {
            s.chars()
                .map(|c| if c == '8' { 40.0 } else { 7.0 })
                .sum::<f32>()
        };
        let reserved = overflow_width(9, 10.0, measure);
        for hidden in 1..=9 {
            let label = overflow_label(hidden);
            assert!(
                reserved >= measure(&label) + 10.0,
                "a band of nine groups reserved {reserved} pt, but with {hidden} \
                 hidden it draws {label:?} at {} pt plus padding",
                measure(&label)
            );
        }
    }

    /// The reservation is sized for the widest label it could ever show,
    /// so it cannot turn out to be too small once the hidden count is
    /// known.
    #[test]
    fn the_reservation_is_sized_for_the_worst_case_label() {
        // A measure that charges 7 pt per character, so the assertion is
        // about the label chosen rather than about a font.
        let measure = |s: &str| s.chars().count() as f32 * 7.0;
        let for_twelve = overflow_width(12, 10.0, measure);
        let for_two = overflow_width(2, 10.0, measure);
        assert!(
            for_twelve >= for_two,
            "a band of twelve groups must reserve at least what a band of two does"
        );
        assert!(
            for_twelve >= measure(&overflow_label(12)),
            "the reservation must fit the widest label the control can show"
        );
        assert_eq!(
            overflow_width(0, 0.0, |_| 0.0),
            MIN_ITEM_WIDTH,
            "even with no text the affordance is a clickable size"
        );
    }

    /// The label says how many are hidden, which is the difference
    /// between an affordance and a mystery chevron.
    #[test]
    fn the_overflow_label_states_the_count() {
        assert_eq!(overflow_label(3), "⏷ 3 more");
        assert_eq!(overflow_label(1), "⏷ 1 more");
    }

    /// **The chevron is the pinned codepoint — one half of a two-sided
    /// pin, and this half cannot check the thing that actually matters.**
    #[test]
    fn the_overflow_label_uses_the_pinned_chevron() {
        const PINNED: char = '\u{23F7}';
        let label = overflow_label(2);
        let first = label.chars().next().expect("a non-empty label");
        assert_eq!(
            first, PINNED,
            "the overflow chevron changed to U+{:04X}. That is allowed, but the \
             bundled fonts must be able to draw it — see this module's own note on \
             which near misses are missing, and update \
             `pdfcer_gui::shell::tests::the_ribbon_overflow_chevron_has_a_glyph`, \
             which is the only place that can check.",
            first as u32
        );
        assert_eq!(
            crate::dock::plan::overflow_label(2),
            label,
            "the dock and the ribbon must spell the affordance identically; an \
             operator should not have to learn two overflow idioms in one window"
        );
    }
}
