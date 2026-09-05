//! Tab-strip **row** arithmetic — how the row is divided between the QAT,
//! the tabs and the mode selector, and how the tabs are then divided
//! between the strip and its overflow menu.
//!
//! # Why this is a separate file from [`super`]
//!
//! Not line count. The two files answer different questions about the same
//! rule, and reading either one should not require holding the other:
//!
//! | | [`super`] (the band) | this file (the row) |
//! |---|---|---|
//! | Claimants | one affordance vs. N groups | four: QAT, selector, affordance, tabs |
//! | Visible set | always a **prefix** | a prefix **plus a pinned member** |
//! | When space runs out | the affordance takes absolute priority | the affordance and the pinned tab share a floor |
//! | Everything hidden is reachable | yes — that is why groups may go to zero | **normally no** — the pinned tab is not in its own menu |
//!
//! The last row is the whole reason this is not simply another call to
//! [`super::plan_band`]. A band may legitimately degrade to *"no groups,
//! one working affordance"*, because every group is one click away and
//! nothing is lost. A strip may not: the active tab is the one thing its
//! menu is not a route to, so a strip that hid it would leave the operator
//! reading a band whose owner is invisible. (There is one width below
//! which even that has to give — see [`plan_tab_strip`]'s collapse
//! section.)
//!
//! What *is* shared is the greedy fill. [`plan_tab_strip`] calls
//! [`super::plan_band`] to place the non-pinned tabs, so the monotonicity
//! rule — *widening never hides something that was visible* — exists in
//! one place rather than two, and a change to it cannot apply to one row
//! and not the other.
//!
//! # The defect this file was written for
//!
//! `MODES_AND_PANELS.md` Part 2's failure mode #8, one row above the band:
//!
//! > **Tab overflow has no escape** — past ~6 tabs the overflow *button
//! > itself* gets hidden, leaving no route to the hidden tabs.
//!
//! Before this existed the row had no arithmetic at all. The mode selector
//! was emitted first from the right edge and everything else took what was
//! left — which is the "reserve first" shape, and it still failed, because
//! reserving the right island protects the right island and **`egui` does
//! not clip a `Ui`'s children to its `max_rect`**. Measured against the
//! synthetic proportional face of [`super::super::testfont`], with the
//! two-control QAT and two tabs of the test manifest:
//!
//! ```text
//! window  QAT             tabs                selector      verdict
//!  500    0..166          188..265            322..500      correct
//!  320    0..166          188..265            142..320      tabs UNDER the selector
//!  180   -6..160          182..259              2..180      both tabs off screen
//! ```
//!
//! At 180 pt the first QAT control started at **x = −6**. Everything here
//! is a pure function over `f32` for the reason [`super`]'s header gives:
//! that is the only way the invariant can be tested without a window, and
//! [`super::super::width_tests`] then asserts the same properties of the
//! rendered row.
//!
//! # ★ The one measured number everything here turns on
//!
//! [`super::super::measure::min_button_width`] records it:
//! **`Button::truncate()` stops shrinking** at padding-plus-ellipsis,
//! about 19.7 pt with this theme and face. A region granted less than that
//! does not get a crowded control; it gets a control drawn *outside its
//! own rectangle*, on top of its neighbour.
//!
//! That is why [`grant`] answers "all of it", "as much as is usable", or
//! **"none"**, and never a sliver — and why [`plan_tab_strip`] has a width
//! below which it stops trying to show a tab at all. Both rules exist
//! because being accommodating here reproduces the defect.

use super::plan_band;

/// What each claimant on the tab-strip row is asking for, and the floors
/// below which giving it anything is worse than giving it nothing.
///
/// A struct rather than five positional `f32`s because four of the five
/// are widths and a transposed pair would compile, run, and produce a row
/// that is subtly wrong at exactly the widths nobody checks by hand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RowDemand {
    /// The QAT's natural width, from [`super::super::qat::measure`].
    pub qat: f32,
    /// The narrowest a QAT worth drawing can be — its first control's own
    /// floor, from [`super::super::qat::min_width`].
    ///
    /// Separate from [`Self::button_floor`] because a QAT control may
    /// carry an **icon slot**, which `truncate()` cannot shrink at all: a
    /// labelled control with a 16 pt icon bottoms out around 40 pt, not
    /// 19.7. Granting the QAT the smaller figure produced exactly the
    /// defect the floors exist to prevent — the region was 21.5 pt wide,
    /// the control was drawn 39.7 pt wide, and it landed on top of the
    /// first tab.
    pub qat_floor: f32,
    /// The mode selector's natural track width, from
    /// [`super::super::mode_selector::measure_track`].
    pub selector: f32,
    /// The narrowest a selector worth drawing can be: one control's width
    /// per position. Below this its labels are unreadable and its
    /// positions unhittable, and `MODES_AND_PANELS.md` Part 1's *"all
    /// three labels visible"* has stopped being true whatever the
    /// arithmetic says.
    ///
    /// Held back from the **QAT's** share rather than merely clamped
    /// afterwards, which is what stops a wide QAT compressing the selector
    /// to a sliver. Without it a 166 pt QAT in a 180 pt row left the
    /// three-position selector 19.7 pt — 6.6 pt per position.
    pub selector_floor: f32,
    /// What the tab area needs to keep both its promises: a pinned tab
    /// **and** an affordance beside it, each at [`Self::button_floor`],
    /// with the layout's gap between them.
    pub tabs_floor: f32,
    /// The narrowest a **text-only** `egui::Button` can be drawn — see
    /// [`super::super::measure::min_button_width`]. The single most important
    /// number on this row: below it, `truncate()` stops shrinking and the
    /// control overflows whatever rectangle it was given.
    ///
    /// Used for the tabs and the affordance, which are text-only. The QAT
    /// has [`Self::qat_floor`] instead.
    pub button_floor: f32,
    /// The trailing region's natural width, from
    /// [`super::super::trailing::measure`]. **Zero when there is nothing to
    /// draw there**, which is the ordinary case and is what makes the region
    /// cost nothing when it is not in use.
    pub trailing: f32,
    /// The narrowest a trailing region worth drawing can be — its first
    /// control's own floor, from [`super::super::trailing::min_width`].
    ///
    /// A separate figure from [`Self::button_floor`] for exactly the reason
    /// [`Self::qat_floor`] is: a control with an **icon slot** bottoms out
    /// around 40 pt rather than at `truncate()`'s 19.7, and granting the
    /// smaller figure produces a control drawn outside its own rectangle —
    /// here, over the mode selector.
    pub trailing_floor: f32,
}

/// How the tab-strip row's width is divided between its three regions.
///
/// Returned by [`plan_strip_row`]. Widths, not rectangles — this module
/// has no coordinate system; [`super::super::strip`] turns these into
/// rects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RowPlan {
    /// Width granted to the quick-access toolbar, on the left.
    pub qat: f32,
    /// Width granted to the mode selector, on the right.
    pub selector: f32,
    /// Width left for the tabs and their overflow affordance.
    ///
    /// Never zero unless the whole row is — see [`plan_strip_row`].
    pub tabs: f32,
    /// Whether the QAT was granted less than it asked for.
    pub qat_truncated: bool,
    /// Whether the mode selector was granted less than it asked for.
    pub selector_truncated: bool,
    /// Width granted to the trailing region, at the far right — past the
    /// mode selector. Zero when there is nothing to draw, and **also** zero
    /// when the row is too narrow to hold it; see [`RowPlan::trailing_dropped`]
    /// for how the two are told apart.
    pub trailing: f32,
    /// Whether a trailing region that wanted space got none.
    ///
    /// ★ Its own flag rather than `trailing < demand.trailing`, because the
    /// two cases that produce a zero grant are *"there was nothing to draw"*
    /// and *"there was something and the row could not hold it"* — the first
    /// is the ordinary state of this region and must not be announced, the
    /// second is a control the operator cannot reach and must be.
    pub trailing_dropped: bool,
}

/// Divide the tab-strip row between the QAT, the mode selector and the
/// tabs, **outermost first**, with a floor under everything downstream.
///
/// # The order, and why it is an order rather than a competition
///
/// `MODES_AND_PANELS.md` failure mode #8 is about a control that gets
/// squeezed out by content. The row has three claimants and the same cure
/// applies to each: whatever must survive is subtracted *before* whatever
/// may shrink. So the row is divided in one pass, outermost first:
///
/// ```text
/// [ QAT ][ tabs … ⏷ N more ][ mode selector ][ trailing ]
///    1              4              2            3
/// ```
///
/// 1. **The QAT** is a fixed cost. It is neither content nor an
///    affordance: it is the row of controls an operator uses continuously,
///    and `RIBBON_IA.md`'s whole reason for having one is that it must not
///    sit behind a tab switch. It is therefore reserved first.
/// 2. **The mode selector** is reserved from the right edge, out of what
///    the QAT left.
/// 3. **The trailing region** — see [`RowDemand::trailing`] — is reserved from
///    the right edge past the selector, out of what the QAT and the floors
///    left. It is the one reserved region that may be granted **nothing**,
///    because it is an optional extra rather than a promise the interface has
///    already made; see the ★★ comment in the body.
/// 4. **The tabs** — and the affordance that reaches the ones that do not
///    fit — get the remainder.
///
/// # ★ The floors: no reservation may consume the row
///
/// Ordering alone is not enough, and the reason is the defect this
/// function was written for. A QAT wider than the window, reserved first
/// and unconditionally, leaves the tabs nothing at all — and `egui` does
/// not clip children to a `Ui`'s `max_rect`, so "nothing at all" is not
/// drawn as nothing: it is drawn *off the edge*. Requirement 4 of the S2
/// clean-up states it directly: *"the QAT is not allowed to consume the
/// strip."*
///
/// So each reservation is capped at *what leaves the regions after it
/// their floors*:
///
/// ```text
/// qat      = grant(demand.qat,      row − tabs_floor − selector_floor, qat_floor)
/// trailing = grant(demand.trailing, row − qat − tabs_floor − selector_floor, trailing_floor)
/// selector = min(demand.selector,   row − qat − trailing − tabs_floor)
/// tabs     = row − qat − selector − trailing   ← therefore ≥ tabs_floor
/// ```
///
/// Note what `selector_floor` does and does not promise. It is held back
/// from the **QAT's** share, so a wide QAT cannot compress the selector
/// below one control per position. It is *not* a guarantee the selector
/// gets that much: when the row itself is narrower than
/// `tabs_floor + selector_floor`, the selector compresses further, because
/// the tabs' floor outranks it and there is nothing left to take.
///
/// # ★ `grant`: a sliver is worse than nothing
///
/// The QAT goes through [`grant`] rather than a plain `min`, and that is
/// the correction that made this function work rather than merely look
/// right — see this module's header for the measurement.
///
/// The **selector** is exempt, deliberately: its positions are painted
/// rectangles rather than `egui::Button`s
/// ([`super::super::mode_selector`] draws them with a `Painter`), so it
/// has no such floor and compresses continuously. It is protected instead
/// by [`RowDemand::selector_floor`] being held back from the QAT's share.
///
/// # What "truncated" costs, and why it is the right thing to spend
///
/// A truncated region keeps its **position** and loses **characters**.
/// That is the same trade [`super::super::band`] makes for the overflow
/// affordance and [`super::super::mode_selector::fit_track`] makes for the
/// track: a control the operator cannot reach has failed completely,
/// whereas a control whose label is tight has failed cosmetically. Both
/// truncations are disclosed by [`super::super::strip`] through
/// [`crate::verify`], because a row that silently shrank is a different
/// fact from one that fitted.
///
/// A non-finite or negative `row` is treated as zero — see [`sane`].
pub(crate) fn plan_strip_row(row: f32, demand: RowDemand) -> RowPlan {
    let row = sane(row);
    let qat_wanted = sane(demand.qat);
    let selector_wanted = sane(demand.selector);
    let selector_floor = sane(demand.selector_floor).min(selector_wanted);
    let tabs_floor = sane(demand.tabs_floor);
    let qat_floor = sane(demand.qat_floor);

    let trailing_wanted = sane(demand.trailing);
    let trailing_floor = sane(demand.trailing_floor);

    let qat = grant(qat_wanted, row - tabs_floor - selector_floor, qat_floor);
    // ★★ THE TRAILING REGION IS RESERVED LAST AMONG THE RESERVED REGIONS, AND
    // IT IS THE ONLY ONE ALLOWED TO VANISH.
    //
    // The header's ordering argument is about controls that must SURVIVE, and
    // the QAT, the selector and the tabs' floor all qualify: an operator who
    // cannot reach them has lost something the interface promised. The
    // trailing region is different in kind — it holds an optional extra whose
    // absence is already a state the application handles, because R9 makes it
    // absent whenever the capability behind it is missing. So it takes what is
    // left after everything load-bearing has its floor, and takes NOTHING
    // rather than a sliver.
    //
    // `grant`, not a clamp: this region's controls are `egui::Button`s with an
    // icon slot, so the floor is real in exactly the way the selector's is not.
    let trailing = grant(
        trailing_wanted,
        row - qat - tabs_floor - selector_floor,
        trailing_floor,
    );
    // A plain clamp, not `grant`: see the header on why the selector has
    // no button floor.
    let selector = selector_wanted.min((row - qat - trailing - tabs_floor).max(0.0));
    let tabs = (row - qat - selector - trailing).max(0.0);

    RowPlan {
        qat,
        selector,
        tabs,
        qat_truncated: qat < qat_wanted,
        selector_truncated: selector < selector_wanted,
        trailing,
        trailing_dropped: trailing_wanted > 0.0 && trailing <= 0.0,
    }
}

/// Grant a region all of what it wants, as much of `spare` as is still
/// usable, or nothing.
///
/// The three-way answer is the whole point — see this module's header on
/// why a width between zero and `floor` is not a smaller region but a
/// control drawn outside it.
fn grant(wanted: f32, spare: f32, floor: f32) -> f32 {
    if wanted <= 0.0 {
        0.0
    } else if spare >= wanted {
        wanted
    } else if spare >= floor {
        spare
    } else {
        0.0
    }
}

/// A width, or zero if it is not one.
///
/// `egui` hands out `f32::INFINITY` for available width inside an
/// unbounded container, and `INFINITY − anything` is still infinity — a
/// region granted an infinite width would be laid out somewhere no
/// coordinate system contains. `NaN` and negatives get the same answer for
/// the same reason: the degenerate case must be the *safe* one.
fn sane(v: f32) -> f32 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

/// How the tabs are split between the visible strip and the strip's own
/// overflow menu.
///
/// Returned by [`plan_tab_strip`]. `shown` and `hidden` are **indices**
/// rather than counts because, unlike a band, the visible set is not a
/// prefix — the active tab is pinned into it wherever it sits. Both are
/// ascending, so the strip and the menu each keep the manifest's order.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StripPlan {
    /// Indices of the tabs drawn in the strip, ascending.
    pub shown: Vec<usize>,
    /// Indices of the tabs in the overflow menu, ascending.
    pub hidden: Vec<usize>,
    /// The width the shown tabs may occupy.
    ///
    /// Already excludes the affordance's reservation and the gap before
    /// it, exactly as [`super::BandPlan::group_budget`] does.
    pub tab_budget: f32,
    /// The width reserved for the strip's overflow affordance, or `0.0`
    /// when nothing overflowed.
    pub overflow_width: f32,
    /// Whether the affordance was granted less than it asked for.
    pub overflow_truncated: bool,
    /// Whether the pinned tab was granted less than its natural width.
    pub active_truncated: bool,
    /// Whether the strip gave up the pin — see [`plan_tab_strip`]'s
    /// collapse section.
    ///
    /// `true` means the strip is the affordance and nothing else, and the
    /// menu holds **every** tab including the active one.
    pub collapsed: bool,
}

impl StripPlan {
    /// Whether the overflow affordance is to be drawn.
    pub(crate) fn has_overflow(&self) -> bool {
        self.overflow_width > 0.0
    }
}

/// Decide which tabs are drawn in the strip and which move into its
/// overflow menu, **with the active tab pinned**.
///
/// # Arguments
///
/// - `available` — the width the tabs and their affordance share, i.e.
///   [`RowPlan::tabs`].
/// - `tab_widths` — each visible tab's planned width, in display order
///   (the mode's ordinary tabs, then any contextual ones — see
///   [`super::super::tabs::visible_tabs`]).
/// - `active` — the index of the active tab, if there is one. Out of range
///   is treated as absent.
/// - `gap` — the layout's `item_spacing`: between two adjacent tabs, and
///   between the last tab and the affordance.
/// - `overflow_width` — what the affordance wants, from
///   [`super::overflow_width`].
/// - `button_floor` — the narrowest an `egui::Button` can be drawn; see
///   [`super::super::measure::min_button_width`].
///
/// # ★ Why the active tab is pinned, and why a band needs no such rule
///
/// A band and a strip look like the same problem and differ in exactly one
/// respect: **everything a band hides is still reachable through its
/// menu.** So a band may legitimately degrade to "no groups, one working
/// affordance" — every group is one click away and nothing is lost.
///
/// A strip normally cannot. The active tab is not a thing you go and get;
/// it is the thing you are looking at, and the band below the strip is
/// *its* band. A strip that moved the active tab into the menu would leave
/// the operator reading a band whose owner is invisible, with the strip
/// showing a set of tabs none of which is current — which reads as "the
/// ribbon has lost its selection", not as "the window is narrow".
/// Requirement 2 of the S2 clean-up states it as a rule: *a strip that
/// hides the tab you are looking at is worse than one that hides the
/// others.*
///
/// So the active tab's width is charged to the budget **before**
/// [`plan_band`] measures anything else against the remainder. If it alone
/// does not fit, it keeps its slot and its label truncates
/// ([`StripPlan::active_truncated`] says so), because the same trade
/// applies here as everywhere else on this row: position is not
/// negotiable, characters are.
///
/// A **contextual** tab therefore needs no special case at all, which is
/// requirement 3. [`super::super::tabs::visible_tabs`] appends contextual
/// tabs last, so a contextual tab arriving into a full strip is simply the
/// next thing the greedy fill cannot place: it goes into the menu like any
/// other tab, it cannot displace the active one (that one is already paid
/// for), and it is announced by the affordance's count going up.
///
/// # ★ When the area is too narrow for both: the strip collapses
///
/// The pin has a hard limit, and pretending otherwise would reintroduce
/// the defect it exists to prevent. Below `2 × button_floor + gap` — about
/// 47 pt with this theme and face — the area cannot hold a tab **and** an
/// affordance at sizes `egui` will actually draw. Exactly two outcomes are
/// available, and only one is defensible:
///
/// | | Strip shows | Consequence |
/// |---|---|---|
/// | keep the pin | the active tab, no affordance | every other tab is **unreachable** — failure mode #8, exactly |
/// | drop the pin | "⏷ N more", nothing else | every tab reachable, including the active one |
///
/// So the strip **collapses**: [`StripPlan::collapsed`] is set, `shown` is
/// empty, and the menu holds every tab — the active one included, drawn
/// with its active cues so the operator can still see which it is. This is
/// the band's own degradation ("one working affordance that still reaches
/// everything") applied one row up, and it is reached only where the
/// alternative is a control nobody can click. It is disclosed as
/// `ribbon-tab-strip-collapsed`.
///
/// Above that width both survive with a floor each:
///
/// ```text
/// reserve    = clamp(overflow_width, button_floor, available − button_floor − gap)
/// tab_budget = available − reserve − gap        ← therefore ≥ button_floor
/// ```
///
/// At any ordinary width `reserve == overflow_width` and the tabs get the
/// rest, which is the band's behaviour exactly. As the area narrows the
/// affordance is the first to yield — down to, but never past, one
/// button's width — and both then truncate together.
///
/// # Reusing [`plan_band`]
///
/// The greedy prefix fill over the non-pinned tabs *is* [`plan_band`], and
/// it is called with `overflow_width = 0.0` — not because there is no
/// affordance, but because its width has **already** been taken out of
/// `tab_budget` above. Passing it again would reserve it twice.
///
/// That is safe because `plan_band` cannot then wrongly conclude "nothing
/// is hidden": reaching this point means the tabs did not all fit in
/// `available`, and if the remainder fitted in the budget then
/// `total ≤ pinned + gap + rest_available = tab_budget ≤ available`,
/// contradicting that. The `debug_assert` below is the tripwire on that
/// argument, and the `hidden.is_empty()` branch after it is the one case
/// the argument does not cover: a single pinned tab with no remainder at
/// all.
pub(crate) fn plan_tab_strip(
    available: f32,
    tab_widths: &[f32],
    active: Option<usize>,
    gap: f32,
    overflow_width: f32,
    button_floor: f32,
) -> StripPlan {
    let available = sane(available);
    let gap = sane(gap);
    let overflow_width = sane(overflow_width);
    let button_floor = sane(button_floor);
    let n = tab_widths.len();

    let nothing_hidden = |shown: Vec<usize>, active_truncated: bool| StripPlan {
        shown,
        hidden: Vec::new(),
        tab_budget: available,
        overflow_width: 0.0,
        overflow_truncated: false,
        active_truncated,
        collapsed: false,
    };

    if n == 0 {
        return nothing_hidden(Vec::new(), false);
    }

    let total: f32 = tab_widths.iter().sum::<f32>() + gap * (n as f32 - 1.0);
    if total <= available {
        return nothing_hidden((0..n).collect(), false);
    }

    // ★ THE COLLAPSE. See the header: below this width the only choice is
    // between "one tab and no route to the rest" and "a route to
    // everything", and #8 decides it.
    if available < 2.0 * button_floor + gap {
        let reserve = if available >= button_floor {
            overflow_width.max(button_floor).min(available)
        } else {
            // Narrower than one button. Nothing can be drawn here at all,
            // and an affordance would be drawn on top of the mode
            // selector rather than inside this area — there is no row for
            // it to be unreachable in.
            0.0
        };
        return StripPlan {
            shown: Vec::new(),
            hidden: (0..n).collect(),
            tab_budget: 0.0,
            overflow_width: reserve,
            overflow_truncated: reserve < overflow_width,
            active_truncated: false,
            collapsed: true,
        };
    }

    // ★ THE SPLIT. Both the affordance and the pinned tab keep a floor;
    // see the header on why this differs from the band's "affordance takes
    // absolute priority".
    let reserve = overflow_width
        .max(button_floor)
        .min(available - button_floor - gap);
    let tab_budget = (available - reserve - gap).max(0.0);

    // ★ THE PIN. Charged before any other tab is measured.
    let active = active.filter(|&i| i < n);
    let pinned_wanted = active.map_or(0.0, |i| sane(tab_widths[i]));
    let pinned = pinned_wanted.min(tab_budget);

    let rest: Vec<usize> = (0..n).filter(|i| Some(*i) != active).collect();
    let rest_widths: Vec<f32> = rest.iter().map(|&i| tab_widths[i]).collect();
    let rest_available = match active {
        // The pinned tab and its neighbour need a gap between them, so the
        // gap is charged with the pin rather than after it.
        Some(_) => (tab_budget - pinned - gap).max(0.0),
        None => tab_budget,
    };

    let inner = plan_band(rest_available, &rest_widths, gap, 0.0);
    debug_assert!(
        inner.hidden > 0 || rest.is_empty(),
        "plan_tab_strip reserved {reserve} pt for an affordance with nothing \
         behind it: available={available} total={total} tab_budget={tab_budget}"
    );

    let mut shown: Vec<usize> = rest[..inner.shown].to_vec();
    if let Some(i) = active {
        shown.push(i);
        shown.sort_unstable();
    }
    let hidden: Vec<usize> = rest[inner.shown..].to_vec();

    if hidden.is_empty() {
        // One tab, pinned, wider than the area. Nothing is hidden, so
        // nothing may be reserved: a "⏷ 0 more" button opens an empty
        // menu. The tab gets the whole area and truncates into it.
        return nothing_hidden(shown, true);
    }

    StripPlan {
        shown,
        hidden,
        tab_budget,
        overflow_width: reserve,
        overflow_truncated: reserve < overflow_width,
        active_truncated: pinned < pinned_wanted,
        collapsed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The theme-and-face numbers the rendered ribbon actually produces,
    /// so these pure tests exercise the same regime the renderer does.
    ///
    /// Measured, not invented: `button_floor` is
    /// `button_padding (8) + "…" (11.6875)` against the synthetic face of
    /// [`super::super::super::testfont`], and `gap` is `egui`'s default
    /// `item_spacing.x`. See [`super::super::super::measure::min_button_width`].
    const FLOOR: f32 = 19.6875;
    const GAP: f32 = 8.0;

    /// A QAT control with an icon slot bottoms out far above a text-only
    /// button: 8 pt of padding, a 16 pt icon, 4 pt of `icon_spacing` and
    /// the ellipsis. Roughly the figure that caught the defect described
    /// on [`RowDemand::qat_floor`].
    const QAT_FLOOR: f32 = 39.6875;

    /// A demand shaped like the real one: a two-control QAT, a
    /// three-position selector, both wider than a narrow window.
    fn demand(qat: f32, selector: f32, positions: usize) -> RowDemand {
        RowDemand {
            qat,
            // A floor can never exceed the demand it guards: a QAT whose
            // whole natural width is 10 pt does not have a 40 pt control
            // in it. `qat::min_width` derives both from the same command
            // list and cannot disagree; the fixture must not either.
            qat_floor: QAT_FLOOR.min(qat),
            selector,
            selector_floor: selector.min(positions as f32 * FLOOR),
            tabs_floor: 2.0 * FLOOR + GAP,
            button_floor: FLOOR,
            // No trailing region, which is the ordinary case: every existing
            // assertion in this file is about a row that has none, and the
            // arithmetic must be unchanged for it. `trailing_demand` below is
            // the fixture for a row that has one.
            trailing: 0.0,
            trailing_floor: 0.0,
        }
    }

    /// The same demand, with a trailing region of `trailing` points whose
    /// single control bottoms out at [`QAT_FLOOR`] — a trailing control is a
    /// button with an icon slot, exactly as a QAT control is.
    fn trailing_demand(qat: f32, selector: f32, positions: usize, trailing: f32) -> RowDemand {
        RowDemand {
            trailing,
            trailing_floor: QAT_FLOOR.min(trailing),
            ..demand(qat, selector, positions)
        }
    }

    /// **★ Requirement 4: the QAT is not allowed to consume the strip.**
    ///
    /// The observed defect, measured against the synthetic face at a
    /// 180 pt viewport, was a QAT running from x = −6 to x = 160 with both
    /// tabs entirely off screen. The arithmetic form of "must be
    /// impossible" is this: **whatever the QAT asks for, the tabs are left
    /// something usable**, at every row width above zero.
    ///
    /// Swept over the whole plausible range of both, because the
    /// interesting case is not "the QAT is enormous" — that one is easy to
    /// spot — but the widths where it is *nearly* the whole row and a
    /// subtraction would leave a small positive number that looks fine and
    /// is not.
    #[test]
    fn no_reservation_may_leave_the_tabs_with_nothing() {
        for row in (1..900).step_by(7).map(|w| w as f32) {
            for qat in [0.0_f32, 10.0, 90.0, 166.0, 400.0, 5_000.0] {
                for selector in [0.0_f32, 60.0, 189.0, 900.0] {
                    let plan = plan_strip_row(row, demand(qat, selector, 3));
                    assert!(
                        plan.tabs > 0.0,
                        "row={row} qat={qat} selector={selector}: the tabs were left \
                         {} pt, so no tab and no overflow affordance can be drawn at \
                         all — which is failure mode #8 with the whole strip missing",
                        plan.tabs
                    );
                    assert!(
                        plan.tabs >= row.min(2.0 * FLOOR + GAP) - 0.001,
                        "row={row} qat={qat} selector={selector}: the tabs got {} pt, \
                         below the floor a pinned tab and an affordance need",
                        plan.tabs
                    );
                    assert!(
                        plan.qat == 0.0 || plan.qat >= QAT_FLOOR.min(qat) - 0.001,
                        "row={row} qat={qat}: the QAT got {} pt — a sliver narrower \
                         than one of its controls, which `egui` answers by drawing \
                         the control outside it",
                        plan.qat
                    );
                    assert!(
                        (plan.qat + plan.selector + plan.tabs - row).abs() < 0.001,
                        "row={row} qat={qat} selector={selector}: the three regions \
                         must tile the row exactly, not overlap ({plan:?})"
                    );
                }
            }
        }
    }

    /// A row with room for everything grants everything, and grants
    /// **exactly** what was asked for.
    ///
    /// The other half of the rule above: a floor that also applied when
    /// there was plenty of space would be a permanent tax, and a
    /// reservation that rounded up would push the tabs left for no reason.
    #[test]
    fn a_row_with_room_grants_every_reservation_untouched() {
        let plan = plan_strip_row(1000.0, demand(166.0, 189.0, 3));
        assert_eq!(plan.qat, 166.0);
        assert_eq!(plan.selector, 189.0);
        assert_eq!(plan.tabs, 1000.0 - 166.0 - 189.0);
        assert!(!plan.qat_truncated && !plan.selector_truncated);

        // A manifest with no QAT and no modes costs the row nothing.
        let bare = plan_strip_row(500.0, demand(0.0, 0.0, 0));
        assert_eq!(bare.tabs, 500.0);
        assert!(!bare.qat_truncated && !bare.selector_truncated);
    }

    /// **★ A wide QAT cannot compress the mode selector to a sliver.**
    ///
    /// The QAT is reserved first, so without
    /// [`RowDemand::selector_floor`] it takes everything the tabs do not
    /// need and the three-position selector is left whatever remains. At a
    /// 180 pt row that was 19.7 pt — 6.6 pt per position, which
    /// `MODES_AND_PANELS.md` Part 1 forbids in the plainest terms it uses
    /// anywhere: *"all three labels visible"*.
    ///
    /// Holding one button's width per position back from the QAT's share
    /// is what makes the ordering a *priority* rather than a licence.
    ///
    /// The claim is carefully **relative**, and the difference matters: it
    /// is *"an enormous QAT leaves the selector no less than a
    /// zero-width QAT would"*, not *"the selector always gets its floor"*.
    /// The second is not true and must not be asserted, because when the
    /// row itself is narrower than `tabs_floor + selector_floor` the
    /// selector compresses further — the tabs' floor outranks it and there
    /// is nothing left to take.
    #[test]
    fn a_wide_qat_cannot_squeeze_the_selector_below_one_control_per_position() {
        for row in (30..600).step_by(3).map(|w| w as f32) {
            let greedy = plan_strip_row(row, demand(5_000.0, 189.0, 3));
            let alone = plan_strip_row(row, demand(0.0, 189.0, 3));
            let floor = 189.0_f32.min(3.0 * FLOOR);
            assert!(
                greedy.selector >= alone.selector.min(floor) - 0.001,
                "row={row}: an enormous QAT left the selector {} pt where a QAT of \
                 nothing would have left it {} pt, taking it below the {floor} pt its \
                 three positions need ({greedy:?})",
                greedy.selector,
                alone.selector
            );
        }
    }

    /// Truncation is reported, in both directions and only when real.
    ///
    /// The flags are what [`super::super::strip`] turns into
    /// `ribbon-qat-truncated`, and they are what makes a silently shrunken
    /// row a different fact from a row that fitted. A flag that merely
    /// meant "the row is narrow" would fire on every narrow window and be
    /// ignored within a day.
    #[test]
    fn a_truncated_reservation_says_so_and_an_untouched_one_does_not() {
        // Roomy: nothing is touched.
        let roomy = plan_strip_row(600.0, demand(166.0, 189.0, 3));
        assert!(!roomy.qat_truncated && !roomy.selector_truncated);

        // Tight: both are cut, and both say so.
        let tight = plan_strip_row(220.0, demand(166.0, 189.0, 3));
        assert!(tight.qat_truncated || tight.selector_truncated, "{tight:?}");
        assert!(
            tight.qat + tight.selector + tight.tabs <= 220.0 + 0.001,
            "{tight:?}"
        );

        // Too narrow for the QAT to be usable at all: it is dropped, and a
        // dropped region is a truncated one.
        let starved = plan_strip_row(60.0, demand(166.0, 189.0, 3));
        assert_eq!(starved.qat, 0.0, "{starved:?}");
        assert!(starved.qat_truncated);
    }

    /// A degenerate row is answered with zeros rather than with a panic or
    /// an infinity.
    ///
    /// `egui` hands out `f32::INFINITY` for available width inside an
    /// unbounded container; `INFINITY − anything` is still infinity, and a
    /// region granted an infinite width would be laid out somewhere no
    /// coordinate system contains.
    #[test]
    fn a_degenerate_row_is_answered_with_zeros() {
        for bad in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN, -10.0, 0.0] {
            let plan = plan_strip_row(bad, demand(100.0, 100.0, 3));
            assert_eq!(plan.qat, 0.0, "row={bad}");
            assert_eq!(plan.selector, 0.0, "row={bad}");
            assert_eq!(plan.tabs, 0.0, "row={bad}");
        }
        // And a non-finite *demand* is not a demand.
        let plan = plan_strip_row(600.0, demand(f32::NAN, f32::INFINITY, 3));
        assert_eq!(plan.qat, 0.0);
        assert_eq!(plan.selector, 0.0);
        assert_eq!(plan.tabs, 600.0);
    }

    // -----------------------------------------------------------------
    // The tabs within the tab area
    // -----------------------------------------------------------------

    /// Five tabs of unequal width, the shape most of the strip tests want.
    fn tabs(n: usize) -> Vec<f32> {
        (0..n).map(|i| 60.0 + i as f32 * 5.0).collect()
    }

    /// `plan_tab_strip` with the measured floor and gap.
    fn plan(available: f32, widths: &[f32], active: Option<usize>) -> StripPlan {
        plan_tab_strip(available, widths, active, GAP, 55.0, FLOOR)
    }

    /// The narrowest area in which a pinned tab and an affordance can both
    /// be drawn. Below it [`plan_tab_strip`] collapses — see its header.
    const BOTH: f32 = 2.0 * FLOOR + GAP;

    /// **★ Requirement 2: the active tab is never in the overflow menu.**
    ///
    /// The rule the whole pin exists for. A band may legitimately degrade
    /// to "no groups, one working affordance" because everything it hid is
    /// one click away; a strip cannot, because the tab the operator is
    /// looking at is the one thing its menu is not a route to. A strip
    /// that hid it would show a band whose owner is invisible.
    ///
    /// Swept across every width **and** every choice of active tab,
    /// because the interesting case is the *last* tab being active — the
    /// one a prefix-filling planner would drop first — and a test that
    /// only pinned tab 0 would pass with the pin removed entirely.
    ///
    /// Above the collapse width only. Below it the strip has no tab slot
    /// at all and the menu holds everything, which
    /// `a_strip_too_narrow_for_both_collapses_to_the_affordance` covers
    /// and this test must not contradict.
    #[test]
    fn the_active_tab_is_always_shown_and_never_hidden() {
        let widths = tabs(5);
        for active in 0..5 {
            for available in (48..700).step_by(3).map(|w| w as f32) {
                let p = plan(available, &widths, Some(active));
                assert!(!p.collapsed, "at {available} pt, above the collapse width");
                assert!(
                    p.shown.contains(&active),
                    "at {available} pt with tab {active} active, the strip drew \
                     {:?} — the tab the operator is looking at is not on screen",
                    p.shown
                );
                assert!(
                    !p.hidden.contains(&active),
                    "at {available} pt the active tab {active} was put in the \
                     overflow menu, which is the one place it must never be"
                );
                assert!(
                    p.tab_budget >= FLOOR - 0.001,
                    "at {available} pt the pinned tab was given {} pt, less than \
                     `egui` will draw a button in — so it would be drawn outside \
                     its own slot",
                    p.tab_budget
                );
            }
        }
    }

    /// **★ Failure mode #8 for the strip: nothing is ever lost.**
    ///
    /// Every tab is either drawn in the strip or reachable through the
    /// menu, and the menu exists exactly when it has something in it.
    ///
    /// The biconditional is asserted from one button's width upward, which
    /// is a real boundary rather than a fudge: below it nothing can be
    /// drawn in the area at all, and "the affordance is missing" describes
    /// a strip that does not exist. See [`plan_tab_strip`]'s header.
    #[test]
    fn every_tab_is_either_shown_or_reachable_through_the_menu() {
        let widths = tabs(7);
        for available in (20..900).step_by(3).map(|w| w as f32) {
            let p = plan(available, &widths, Some(3));

            let mut all: Vec<usize> = p.shown.iter().chain(p.hidden.iter()).copied().collect();
            all.sort_unstable();
            assert_eq!(
                all,
                (0..7).collect::<Vec<_>>(),
                "at {available} pt a tab was neither in the strip nor in the menu, \
                 so it cannot be reached at all: shown={:?} hidden={:?}",
                p.shown,
                p.hidden
            );

            assert_eq!(
                !p.hidden.is_empty(),
                p.has_overflow(),
                "at {available} pt: hidden={} reserved={} — either tabs were hidden \
                 with nothing to reach them (failure mode #8) or the row is paying \
                 for an affordance with nothing behind it",
                p.hidden.len(),
                p.overflow_width
            );

            assert!(
                p.overflow_width <= available + 0.001,
                "at {available} pt the affordance was reserved {} pt, more than the \
                 whole area it sits in",
                p.overflow_width
            );
            assert!(
                p.overflow_width == 0.0 || p.overflow_width >= FLOOR - 0.001,
                "at {available} pt the affordance was reserved {} pt, below the \
                 width `egui` will draw a button in — so it would be drawn outside \
                 its own rectangle",
                p.overflow_width
            );
        }
    }

    /// **Both the strip and its menu keep the manifest's order.**
    ///
    /// The visible set is *not* a prefix — the pin makes sure of that —
    /// but it is still ascending, and so is the menu. A strip whose
    /// left-to-right order depended on the window width would move every
    /// target under the operator's cursor as they resized.
    #[test]
    fn the_strip_and_the_menu_both_keep_the_manifest_order() {
        let widths = tabs(6);
        for available in (20..600).step_by(5).map(|w| w as f32) {
            let p = plan(available, &widths, Some(5));
            assert!(
                p.shown.windows(2).all(|w| w[0] < w[1]),
                "at {available} pt the strip's order is {:?}",
                p.shown
            );
            assert!(
                p.hidden.windows(2).all(|w| w[0] < w[1]),
                "at {available} pt the menu's order is {:?}",
                p.hidden
            );
        }

        // And the pin really does break the prefix property, which is the
        // point: tab 5 is active and on screen while 4 is not.
        let p = plan(150.0, &widths, Some(5));
        assert_eq!(p.shown.last(), Some(&5));
        assert!(p.hidden.contains(&4), "{p:?}");
    }

    /// **★ Requirement 3: a contextual tab arriving into a full strip goes
    /// into the menu and does not displace the active one.**
    ///
    /// [`super::super::tabs::visible_tabs`] appends contextual tabs last,
    /// so this is stated as "the tab that appeared at the end". Three
    /// claims, and the middle one is the requirement:
    ///
    /// 1. Adding it does not change which *other* tabs are shown when the
    ///    strip was already full — the appearance of a Format tab must not
    ///    reshuffle the strip under the operator's cursor.
    /// 2. The active tab is still shown afterwards.
    /// 3. The menu's count went up by one, which is how the new tab is
    ///    announced — [`super::overflow_label`] puts that count in the
    ///    affordance.
    #[test]
    fn a_contextual_tab_arriving_into_a_full_strip_goes_to_the_menu() {
        let ordinary = tabs(5);
        let mut with_contextual = ordinary.clone();
        with_contextual.push(70.0); // the contextual tab, appended last

        // A width at which the strip is already full.
        let room = 200.0;
        let before = plan(room, &ordinary, Some(1));
        let after = plan(room, &with_contextual, Some(1));

        assert!(
            !before.hidden.is_empty(),
            "this test needs a strip that is already full at {room} pt"
        );
        assert_eq!(
            before.shown, after.shown,
            "a contextual tab appearing reshuffled the strip: {:?} became {:?}",
            before.shown, after.shown
        );
        assert!(
            after.shown.contains(&1),
            "the contextual tab displaced the active one"
        );
        assert!(
            after.hidden.contains(&5),
            "the contextual tab must go into the menu, not off the edge: {after:?}"
        );
        assert_eq!(
            after.hidden.len(),
            before.hidden.len() + 1,
            "the menu's count is how the new tab is announced, and it did not go up"
        );
    }

    /// **Widening the area never hides a tab that was visible.**
    ///
    /// Monotonicity is what makes a window resize feel like a resize
    /// rather than like a reshuffle. It is inherited from [`plan_band`]'s
    /// greedy fill, but the pin, the collapse and the shared affordance
    /// floor all sit on top of it and any of them could break it.
    ///
    /// The collapse is the interesting boundary: crossing it upward must
    /// *gain* the pinned tab, never lose one.
    #[test]
    fn widening_the_strip_never_hides_a_tab_that_was_visible() {
        let widths = tabs(6);
        for active in [0_usize, 3, 5] {
            let mut previous = 0;
            for available in (1..800).map(|w| w as f32) {
                let shown = plan(available, &widths, Some(active)).shown.len();
                assert!(
                    shown >= previous,
                    "active={active}: widening to {available} pt dropped a tab that \
                     fitted at a narrower width ({shown} shown, was {previous})"
                );
                previous = shown;
            }
            assert_eq!(
                previous, 6,
                "active={active}: at 800 pt everything must fit"
            );
        }
    }

    /// **★ Above the collapse width, the pinned tab and the affordance
    /// share the shortfall and neither is reduced below what `egui` will
    /// draw.**
    ///
    /// The place this planner deliberately differs from [`plan_band`]. In
    /// the band the affordance takes absolute priority and the groups may
    /// get zero, because every group is still reachable. Here both must
    /// survive: the affordance is the only route to the hidden tabs, and
    /// the pinned tab is not reachable through it.
    #[test]
    fn a_crowded_strip_keeps_both_the_pinned_tab_and_the_affordance() {
        let widths = tabs(4);
        for available in (48..140).map(|w| w as f32) {
            let p = plan(available, &widths, Some(0));
            assert!(
                p.has_overflow(),
                "at {available} pt three tabs are hidden with no route to them"
            );
            assert!(
                p.overflow_width >= FLOOR - 0.001 && p.tab_budget >= FLOOR - 0.001,
                "at {available} pt the affordance got {} pt and the pinned tab {} pt; \
                 `egui` will not draw a button in less than {FLOOR}, so whichever is \
                 short would be drawn outside its own rectangle",
                p.overflow_width,
                p.tab_budget
            );
            assert!(
                p.shown.contains(&0),
                "at {available} pt the pinned tab was squeezed out by the affordance"
            );
            assert!(
                p.tab_budget + GAP + p.overflow_width <= available + 0.001,
                "at {available} pt the tab slot and the affordance overlap: {p:?}"
            );
        }

        // Just above the collapse width the affordance is the one that has
        // yielded, and it says so.
        let p = plan(BOTH, &widths, Some(0));
        assert!(
            p.overflow_truncated,
            "a 55 pt affordance in a {BOTH} pt area must report that it was crowded: \
             {p:?}"
        );
        assert!(
            p.active_truncated,
            "a 60 pt tab in what is left of a {BOTH} pt area must report that it lost \
             characters: {p:?}"
        );
    }

    /// **★ Below the collapse width the strip becomes the affordance, and
    /// the affordance reaches every tab — the active one included.**
    ///
    /// The one place requirement 2's pin is deliberately given up, and the
    /// reasoning is in [`plan_tab_strip`]'s header: the alternative is one
    /// visible tab and no route at all to the other six, which is failure
    /// mode #8 in its original form. Reachability wins over pinning
    /// because an unreachable tab is a lost capability and a hidden active
    /// tab is a confusing one.
    #[test]
    fn a_strip_too_narrow_for_both_collapses_to_the_affordance() {
        let widths = tabs(4);
        for available in (FLOOR.ceil() as i32..BOTH.floor() as i32).map(|w| w as f32) {
            let p = plan(available, &widths, Some(2));
            assert!(p.collapsed, "at {available} pt: {p:?}");
            assert!(
                p.shown.is_empty(),
                "at {available} pt there is not room for a tab and an affordance, so \
                 the strip must show the affordance alone: {p:?}"
            );
            assert_eq!(
                p.hidden,
                vec![0, 1, 2, 3],
                "at {available} pt every tab — the active one included — must be \
                 reachable through the menu, or something is lost"
            );
            assert!(
                p.overflow_width >= FLOOR - 0.001 && p.overflow_width <= available + 0.001,
                "at {available} pt the affordance got {} pt",
                p.overflow_width
            );
        }
    }

    /// A strip that fits reserves nothing, and a strip with no tabs plans
    /// nothing.
    ///
    /// The first is the "no permanent tax" half of the biconditional; the
    /// second is the first frame of an application that has not built its
    /// manifest yet.
    #[test]
    fn a_strip_that_fits_reserves_nothing_and_an_empty_one_plans_nothing() {
        let widths = tabs(3);
        let p = plan(1000.0, &widths, Some(1));
        assert_eq!(p.shown, vec![0, 1, 2]);
        assert!(p.hidden.is_empty());
        assert_eq!(p.overflow_width, 0.0);
        assert!(!p.has_overflow());
        assert!(!p.active_truncated && !p.overflow_truncated && !p.collapsed);

        let empty = plan(500.0, &[], None);
        assert!(empty.shown.is_empty() && empty.hidden.is_empty());
        assert!(!empty.has_overflow());
    }

    /// **A single tab, too wide for the area, keeps its place and loses
    /// characters — and grows no affordance.**
    ///
    /// The corner where the pin and the biconditional could contradict
    /// each other. There is exactly one tab, it does not fit, and it is
    /// active. Nothing is hidden, so nothing may be reserved: a
    /// "⏷ 0 more" button would be a control that opens an empty menu.
    #[test]
    fn one_over_wide_tab_truncates_rather_than_growing_an_empty_menu() {
        let p = plan(60.0, &[200.0], Some(0));
        assert_eq!(p.shown, vec![0]);
        assert!(p.hidden.is_empty());
        assert_eq!(
            p.overflow_width, 0.0,
            "an affordance with nothing behind it is a button that opens nothing"
        );
        assert!(p.active_truncated, "{p:?}");
        assert_eq!(
            p.tab_budget, 60.0,
            "with no affordance to make room for, the tab gets the whole area"
        );
    }

    /// With **no** active tab the plan is an ordinary prefix fill.
    ///
    /// An empty manifest, or the frame before `resolve_active` has run.
    /// The pin is the only thing that makes the visible set non-prefix, so
    /// without one the strip must behave exactly like a band.
    #[test]
    fn with_no_active_tab_the_strip_fills_from_the_front() {
        let widths = tabs(5);
        for available in (48..600).step_by(7).map(|w| w as f32) {
            let p = plan(available, &widths, None);
            let expected: Vec<usize> = (0..p.shown.len()).collect();
            assert_eq!(
                p.shown, expected,
                "at {available} pt a strip with no active tab must be a prefix"
            );
        }
        // An out-of-range index is treated as "no active tab" rather than
        // as a panic: this runs in the paint loop against state an
        // application supplied.
        let p = plan(150.0, &widths, Some(99));
        assert_eq!(p.shown, (0..p.shown.len()).collect::<Vec<_>>());
    }

    /// A non-finite available width degrades to "everything in the menu".
    ///
    /// The same guard [`plan_band`] carries, for the same reason, and it
    /// has to be re-asserted here because this function does its own
    /// arithmetic before delegating. At zero width the area collapses, so
    /// the honest answer is that everything is in the menu and there is no
    /// room to draw even the affordance.
    #[test]
    fn a_non_finite_strip_width_degrades_safely() {
        for bad in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN, -50.0] {
            let p = plan(bad, &tabs(4), Some(2));
            assert!(p.collapsed, "available={bad}: {p:?}");
            assert_eq!(p.hidden.len(), 4, "available={bad}");
            assert_eq!(
                p.overflow_width, 0.0,
                "available={bad}: there is no width to draw an affordance in"
            );
        }
    }

    /// **★ A row with no trailing region is arithmetically unchanged.**
    ///
    /// The first thing to establish about a new claimant on a shared budget:
    /// it costs nothing when it is not there. Every other assertion in this
    /// file was written against the three-region row, and if adding a fourth
    /// moved any of them the fourth is wrong rather than the third.
    #[test]
    fn a_row_with_no_trailing_region_divides_exactly_as_it_did_before() {
        for row in (1..600).map(|r| r as f32) {
            let without = plan_strip_row(row, demand(166.0, 189.0, 3));
            let with_empty = plan_strip_row(row, trailing_demand(166.0, 189.0, 3, 0.0));
            assert_eq!(
                (without.qat, without.selector, without.tabs),
                (with_empty.qat, with_empty.selector, with_empty.tabs),
                "row={row}: an absent trailing region changed the division"
            );
            assert_eq!(without.trailing, 0.0, "row={row}");
            assert!(
                !without.trailing_dropped,
                "row={row}: nothing was asked for, so nothing was dropped"
            );
        }
    }

    /// **★★ The trailing region never eats the selector, the QAT or the tabs'
    /// floor — at any width.**
    ///
    /// This is `the_qat_is_not_allowed_to_consume_the_strip` restated for the
    /// new claimant, and it is the assertion that makes reserving a fourth
    /// region safe. A trailing control four times wider than the whole window
    /// must still leave every load-bearing region what it had; the only thing
    /// that may give is the trailing region itself.
    #[test]
    fn a_trailing_region_never_consumes_the_row() {
        for row in (1..800).map(|r| r as f32) {
            let baseline = plan_strip_row(row, demand(166.0, 189.0, 3));
            let greedy = plan_strip_row(row, trailing_demand(166.0, 189.0, 3, 4.0 * row));

            assert!(
                greedy.trailing <= row,
                "row={row}: the trailing region took more than the row has"
            );
            assert!(
                greedy.tabs >= baseline.tabs.min(greedy.tabs) - 0.001,
                "row={row}"
            );
            assert!(
                (greedy.qat + greedy.selector + greedy.tabs + greedy.trailing - row).abs() < 0.001,
                "row={row}: the four regions must tile the row exactly"
            );
            // The load-bearing regions keep what they had. A trailing region
            // that could shrink the QAT would be able to hide Undo behind a
            // button that opens another program, which is the wrong way round.
            assert_eq!(greedy.qat, baseline.qat, "row={row}: the QAT gave ground");
            assert!(
                greedy.selector >= baseline.selector.min(greedy.selector) - 0.001,
                "row={row}"
            );
        }
    }

    /// **★★ A trailing region that cannot have a whole control gets NOTHING,
    /// and says so.**
    ///
    /// The three-way `grant` rule, which this module's header measures: a
    /// width between zero and the floor does not produce a smaller button, it
    /// produces a button drawn *outside its own rectangle* — here, on top of
    /// the mode selector, where a misplaced click changes the operator's mode
    /// instead of opening their document elsewhere.
    ///
    /// And the drop is announced. `trailing_dropped` is a separate flag from
    /// `trailing == 0.0` precisely because the ordinary state of this region
    /// is to be empty, and announcing that every frame would bury the one
    /// case worth reading.
    #[test]
    fn a_trailing_region_below_its_floor_is_dropped_whole_and_disclosed() {
        // Roomy: it gets exactly what it asked for and nothing is disclosed.
        let roomy = plan_strip_row(900.0, trailing_demand(166.0, 189.0, 3, 60.0));
        assert_eq!(roomy.trailing, 60.0);
        assert!(!roomy.trailing_dropped);

        // Tight: below the floor, so nothing — not a sliver.
        let tight = plan_strip_row(260.0, trailing_demand(166.0, 189.0, 3, 60.0));
        assert_eq!(
            tight.trailing, 0.0,
            "a region that cannot hold a whole control must hold none"
        );
        assert!(
            tight.trailing_dropped,
            "a control the operator cannot reach must be disclosed"
        );

        // Never a width between zero and the floor, at any row width.
        for row in (1..900).map(|r| r as f32) {
            let plan = plan_strip_row(row, trailing_demand(166.0, 189.0, 3, 60.0));
            assert!(
                plan.trailing == 0.0 || plan.trailing >= QAT_FLOOR.min(60.0) - 0.001,
                "row={row}: granted a sliver of {}",
                plan.trailing
            );
            assert_eq!(
                plan.trailing_dropped,
                plan.trailing <= 0.0,
                "row={row}: the flag and the grant disagree"
            );
        }
    }
}
