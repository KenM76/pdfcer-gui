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
//! `MODES_AND_PANELS.md` Part 2 lists twelve failure modes observed in a
//! shipping application, and number eight is the one this module exists
//! to make impossible:
//!
//! > **Tab overflow has no escape** — past ~6 tabs the overflow *button
//! > itself* gets hidden, leaving no route to the hidden tabs. → *The
//! > overflow affordance is reserved space, never the first thing
//! > squeezed out.*
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
//! be, so it is budgeted at `CUSTOM_ITEM_WIDTH`. **An estimate that is
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
//! ## ★ And why the floor is not enough — read this before trusting a
//! ## width test in this crate
//!
//! The floor keeps the arithmetic *meaningful*; it does not make a
//! zero-width-text test *equivalent* to a real one, and treating the two
//! as equivalent cost this module two defects.
//!
//! The font situation is not merely "absent in tests". It is **decided by
//! whichever sibling crate is in the build**:
//!
//! ```text
//! cargo test -p egui-shell --lib   egui alone            → no fonts, widths ≈ 0
//! cargo test --workspace           pdfcer-gui → eframe    → egui/default_fonts,
//!                                                          real widths
//! ```
//!
//! Cargo unifies features across a workspace build, so the same assertions
//! measure different text under the two commands, and the *narrower*
//! command — the one a developer working on the shell reaches for — is
//! the one that measures nothing. Everything below that compares a width
//! against another width was, for the whole of this module's life,
//! trivially satisfied under that command.
//!
//! Two consequences were real defects, both of which only appear with
//! metrics: the overflow affordance being positioned from a `Ui` whose
//! `max_rect` a sibling row had grown (see [`super::band`]), and
//! [`overflow_width`] reserving for the label with the most *characters*
//! rather than the most *width*.
//!
//! `super::width_tests` closes that hole by installing a synthetic
//! proportional face this crate builds itself, so the width-sensitive
//! paths are exercised with real advances under **both** commands and
//! under any future workspace membership. A new width rule added to this
//! module belongs there as well as here.

pub(crate) mod collapse;

pub(crate) mod row;

// Re-exported so every call site reads `plan::…` and the split between
// this file and [`row`] stays an organisational fact rather than
// something a caller has to know.
pub(crate) use row::{RowDemand, StripPlan, plan_strip_row, plan_tab_strip};

/// The narrowest a control may be drawn, in points, before the theme's
/// padding is added.
///
/// A control narrower than its own height reads as a clipping artefact
/// rather than as a button. This is also the floor that keeps the
/// arithmetic meaningful with no fonts installed — see the module header.
pub(crate) const MIN_ITEM_WIDTH: f32 = 20.0;

/// The width budgeted for a [`crate::manifest::Item::Custom`].
///
/// The shell does not draw custom items and cannot measure them: the
/// application is handed a `Ui` and draws a colour swatch, a zoom slider
/// or a gallery into it. Four control-widths is a generous guess at a
/// compound control.
///
/// Being wrong here costs a clipped group, never a lost overflow
/// affordance — see the module header on why that asymmetry is the point.
pub(crate) const CUSTOM_ITEM_WIDTH: f32 = 96.0;

/// Horizontal padding inside a group, either side of its content.
///
/// The mockup's `.group { padding: 0 13px }`, and — since 2026-08-14 —
/// **both budgeted here and drawn** by [`super::band::captioned_group`].
///
/// # ★ The history, because the resolution is the interesting part
///
/// From 2026-08-13 this constant was budgeted and **not drawn**. The
/// renderer laid a group out as a bare `ui.vertical` with no horizontal
/// inset, so the 12 pt it adds to every group's planned width was space the
/// renderer never used. Measured against the synthetic face at the time: the
/// three View-tab groups planned at 213.9, 174.4 and 165.6 pt and drew at
/// 202.4, 165.1 and 157.5 pt. Measured in the running application at
/// 1,100 pt, the Markup tab's Text-markup group box began at x = 322.5 and so
/// did its first control — controls sat flush against the group boundary and
/// against the rule separating them from the next group, which is most of
/// what the operator meant by *"cluttered"*.
///
/// The old note called resolving it a **design** decision rather than an
/// arithmetic one, and that was right: either a ribbon group has internal
/// padding (draw it, the plan is already correct) or it does not (this
/// constant should be zero). The decision taken was the first, for the reason
/// the note itself gives — the plan had been reserving the space all along,
/// so **drawing it costs zero additional width budget**. Not one group moves
/// into the overflow menu that was not already there; the band spends on
/// padding exactly what it was already spending on nothing.
///
/// # ★ 6 pt here against the mockup's 13 px, and why they agree anyway
///
/// The two numbers look like a disagreement and are not, because the mockup
/// and this build decompose the space *between* two groups differently:
///
/// | | mockup | this build |
/// |---|---|---|
/// | group's own inset | 13 px each side | `GROUP_PADDING` = 6 pt each side |
/// | the rule between them | `border-right: 1px` — **costs no width** | `ui.separator()` — [`super::band::separator_width`] = 6 pt of line plus one `item_spacing.x` each side |
/// | total, group edge to group edge | 13 + 0 + 13 = **26 px** | 6 + 14 + 6 = **26 pt** at the quiet preset |
///
/// So the inter-group breathing room already matches the specification to the
/// point; what was missing was only that none of it was on the *inside* of
/// the boundary. Raising this constant to 13 would make that gap 40 pt, which
/// is not what the mockup shows — and it would cost 14 pt of planned width
/// **per group**, which is 70 pt across the File tab's five band groups and
/// is enough to push a group into the overflow menu. Overflow is a measured
/// property of this band (see `super::width_tests` and the counts recorded in
/// [`super::band`]'s header), so that is a regression, not a refinement.
///
/// The one place the two genuinely differ is the band's outer edge, where the
/// mockup adds its own `padding: … 10px` before the first group's 13. This
/// build's band has no horizontal padding of its own, so its first control
/// sits `GROUP_PADDING` from the band edge rather than 23 px. That is a
/// separate decision about the *band*, not about a group, and it is not taken
/// here.
pub(crate) const GROUP_PADDING: f32 = 6.0;

/// **How many control rows one ribbon group may use.**
///
/// # Why this is a constant and not a parameter
///
/// `PROJECT_PLAN.md`'s **R128** is this project's most-repeated bug: a
/// *content-driven* height next to a fit-to-viewport zoom is a feedback
/// loop, measured at 230 % → 224 % → 215 % zoom drift from a status line
/// that grew by one row. The ribbon sits in a top panel above the canvas,
/// so the band's height is exactly such a term — and a band that were one
/// row tall on File and two on Markup would re-fit the page on **every tab
/// click**.
///
/// A constant makes *"the band is the same height on every tab"* true by
/// construction rather than by an invariant somebody has to enforce. Every
/// alternative spelling reintroduces the loop or the enforcement:
///
/// | Spelling | What breaks |
/// |---|---|
/// | per-**group** row count | the tallest group on a tab decides the band, so the band is content-driven again — R128 exactly |
/// | per-**tab** row count | the manifest can now say "File is one row, Markup is two", which is the drift stated as a feature |
/// | per-**manifest** / theme field | safe for R128, but it is a knob with one sane value that every consumer must set identically, and the first one that does not gets the drift |
///
/// A [`crate::theme::Preset`] change *does* alter the band's height,
/// through [`crate::theme::Metrics::control_height`], and that is fine and
/// unavoidable: it is a deliberate, global, one-off event, not something
/// that happens when the operator clicks a tab.
///
/// # Why two, and why not three
///
/// Two is what `mockups/ribbon.html` specifies — `.gcmds` wraps
/// (`flex-wrap: wrap`) inside a band with a fixed `min-height: 86px`, which
/// is one control row plus a second plus a caption.
///
/// Three was considered and refused on two grounds. **Screen budget:** a
/// third row costs another `control_height + gutter` — 28 pt at the quiet
/// preset, 36 at the airy one — off the top of the canvas on every tab,
/// permanently, and `MODES_AND_PANELS.md` failure mode #4 is precisely a
/// chrome minimum that ate *"up to a third of my screen width"*. **The
/// caption:** the caption is what makes a group legible as a group (see
/// [`super::band`]'s header), and it is drawn `small()` and `weak()`; three
/// rows of controls put it far enough from the top row that it stops
/// reading as that row's label.
pub(crate) const GROUP_ROWS: usize = 2;

/// **The most rows a group may be re-wrapped onto** when the band runs short of
/// width — S5's ceiling.
///
/// Three, from Word: its Font group is two rows at 1900 pt and **three** at
/// 1000 pt (`evidence/word-ribbon/`). It never goes to four at any width in the
/// series, including 460, where it collapses instead.
///
/// ★ Why the ceiling is not simply "as many as it takes": the band's HEIGHT is
/// fixed and must stay fixed — R128, and the reason [`GROUP_ROWS`]'s own doc
/// gives for stopping at two in the first place. A fourth row would either grow
/// the band, which moves the canvas under a fit-to-page zoom, or shrink the
/// controls below the size at which their icons are legible. Word reached the
/// same answer, and reaching it independently is worth more than copying it.
///
/// ★★ Note this is a **ceiling, not a target**. `wrap_group` searches for the
/// narrowest packing that fits within the row limit it is given, so a group
/// that reaches its narrowest at two rows stays at two even when three are
/// permitted, and `collapse::Candidate::gains_from` then declines to spend a
/// rung of the ladder on it.
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
///
/// Produced by [`wrap_group`] and consumed **twice**: once by
/// [`super::band::measure_group`], which needs the width the wrapped group
/// will occupy, and once by [`super::band::captioned_group`], which needs
/// the same split in order to draw it. Passing the split rather than
/// recomputing it at the second site is the point — two greedy fills that
/// agreed *by construction today* would be a plan and a renderer that
/// disagree about the band's width the first time either is edited, and
/// that disagreement is a clipped group.
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
///
/// `pub(crate)` because [`super::band`]'s measurement and this module's
/// wrap have to agree on what a row costs down to the last gutter; two
/// spellings of the same sum is how a plan and a renderer drift apart.
pub(crate) fn row_width(item_widths: &[f32], gutter: f32) -> f32 {
    if item_widths.is_empty() {
        0.0
    } else {
        item_widths.iter().sum::<f32>() + gutter * (item_widths.len() as f32 - 1.0)
    }
}

/// Slack, in points, when comparing an accumulating row width against a
/// candidate target.
///
/// [`wrap_group`] builds its candidate widths by summing a slice and packs
/// by accumulating item by item. The two orders of addition are equal in
/// exact arithmetic and can differ by an ULP or two in `f32`, which would
/// make the intended candidate look infeasible by a hair and push the
/// answer onto the next-widest one. A thousandth of a point is far below a
/// physical pixel and far above the rounding.
const PACK_SLACK: f32 = 1.0e-3;

/// Split one group's items across at most `max_rows` rows.
///
/// # The rule, in two sentences
///
/// A group whose items fit within `wrap_at` on one row is left on one row.
/// Otherwise it is split into contiguous runs, at most `max_rows` of them,
/// chosen to make the **widest** run as narrow as possible.
///
/// # Why "minimise the widest row" is the right objective
///
/// Because the widest row is what the group *costs*: [`group_width`] takes
/// the maximum, [`plan_band`] spends the band's budget on it, and a group
/// that costs less is a group the band can fit beside another one. The
/// mockup's greedy fill optimises nothing — it fills the cap and lets the
/// remainder fall where it falls — so a seven-control group 548 pt wide
/// becomes 440 + 100 under the mockup's rule and about 280 + 270 under this
/// one. Both are two rows; only one of them gets a fifth group into a
/// 1,100 pt window.
///
/// # The algorithm, and why it is exhaustive rather than clever
///
/// The optimum's widest row is, necessarily, the width of *some contiguous
/// run of items* — it is one of the rows. So the candidate set is every
/// contiguous run, `n(n+1)/2` of them, and the answer is the narrowest
/// candidate that a greedy fill can meet within `max_rows`. Greedy is
/// monotone in the target width (a wider row never needs more rows), so the
/// first feasible candidate in ascending order is the optimum.
///
/// A ribbon group holds single digits of items — the widest in this
/// project's own manifest is seven — so `n²` candidates is at most a few
/// dozen f32 sums per group per frame, against a measurement pass that has
/// already asked `egui` for a galley per label. A binary search over a real
/// interval would be *less* exact for no measurable saving, and "exhaustive
/// over the runs" is a sentence a reader can check against the code.
///
/// # Degenerate inputs, all of which are reachable
///
/// - **Empty group.** `counts` is empty and `width` is zero; a manifest may
///   legally declare a group with no items (a layer patching a caption, for
///   instance), and it must not produce a row of nothing.
/// - **One item.** Never split — a column of one control per row is not a
///   ribbon group, it is a rendering fault that happens to be deliberate.
/// - **`max_rows <= 1`.** Wrapping disabled; one row, whatever its width.
///   This is the escape hatch that keeps the *old* behaviour expressible
///   and testable rather than merely deleted.
/// - **An item wider than `wrap_at` all by itself.** It takes a row of its
///   own and that row is wider than the cap. There is no other answer: the
///   shell cannot make a control narrower, and clipping it would hide a
///   command's name. The cap is a wrap trigger, never a clip.
/// - **A non-finite or negative `wrap_at`.** Treated as "never wrap", which
///   is the same safe direction [`plan_band`] takes for a non-finite width:
///   the group is as wide as it says it is and the band's overflow menu —
///   whose reservation is taken first — still reaches everything.
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

    // ★★★ **`prefer_rows` is exactly the right to skip the fits-already test.**
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
    // ★★ The value is a HINT and the band's ceiling still wins: `max_rows` is
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
///
/// An item wider than `target` on its own still gets placed — on a row by
/// itself, over budget — because refusing it would be refusing to draw a
/// control.
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
///
/// The caption is part of the *width*, not only of the height, and that
/// is deliberate. A group whose caption is wider than its single control
/// — "Page display" over one icon button — is as wide as its caption, and
/// planning it at the control's width would overflow the band by the
/// difference. `RIBBON_IA.md` §5 is full of two-word captions over
/// one-glyph controls, so this is the common case rather than the corner.
///
/// `content_width` is what the group's controls occupy: its **widest row**
/// for a group that wraps, plus the Large run that leads it if it has one.
/// A wrapped group costs its widest row and not the sum of its items — that
/// substitution is the entire width benefit of wrapping.
///
/// ★ It took a [`GroupRows`] until `ItemSize` landed, so that a caller could
/// not ask for a group's width without having first decided how it wraps.
/// That guard stopped being expressible once a group could also have a Large
/// run *beside* its rows: the content width is then a sum of two things, and
/// only the caller knows both. The guard it is replaced by is that
/// `measure_group` is the one caller, and it computes the content width
/// immediately above the call.
pub(crate) fn group_width(content_width: f32, caption_width: f32) -> f32 {
    content_width.max(caption_width) + GROUP_PADDING * 2.0
}

/// How a band's groups are split between the visible band and the
/// overflow menu.
///
/// Returned by [`plan_band`]. Every field is a decision the renderer then
/// obeys without re-deriving anything, so that the arithmetic exists in
/// exactly one place and can be tested without a window.
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
///
/// # Arguments
///
/// - `available` — the band's usable width in points. A negative,
///   infinite or NaN value is treated as zero; a layout pass can hand out
///   `f32::INFINITY` for available width inside an unbounded container,
///   and `INFINITY - overflow_width` is still infinity, which would
///   silently disable overflow rather than show everything.
/// - `group_widths` — each group's planned width, in manifest order.
/// - `separator` — the width of the vertical rule drawn between adjacent
///   groups, including its own spacing.
/// - `overflow_width` — the width the overflow affordance needs. Computed
///   for the **widest label it could ever show** (see
///   [`overflow_label`]), so the reservation can never turn out to be too
///   small once the hidden count is known.
///
/// # The algorithm, and the one line that matters
///
/// 1. If everything fits, show everything and reserve nothing. An
///    overflow control that took space when there was nothing to overflow
///    into it would be a permanent tax on the band.
/// 2. Otherwise `group_budget = available − overflow_width − separator`,
///    **clamped at zero**. This is the line the whole module exists for.
///    The separator is subtracted too, because a rule is drawn between
///    the last visible group and the overflow control.
/// 3. Fill `group_budget` greedily from the front.
///
/// Step 3 can place **zero** groups — at a width narrower than one group
/// plus the reservation, `shown` is 0 and every group is in the menu.
/// That is the correct answer and it is the case the reservation exists
/// for: the band degrades to a single "⏷ N more" control that still
/// reaches everything, rather than to a band of clipped groups with no
/// route to the rest.
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

    // ★ THE RESERVATION. Subtracted before a single group is considered.
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
///
/// The chevron is part of the label rather than a separate glyph so the
/// control is one measurable string, and so a build with no icon set
/// still shows an affordance rather than an empty button.
///
/// ## ★ The chevron is `⏷` U+23F7, and the obvious choices are all tofu
///
/// This read `⌄` (U+2304, DOWNWARDS ARROWHEAD) until 2026-08-14 and
/// **rendered as an empty box in every shipped build** — `□ 1 more`,
/// `□ 2 more` — because egui's bundled font stack (Ubuntu-Light +
/// NotoEmoji + emoji-icon-font) has no face for it.
///
/// It is worth naming the near misses, because every one of them is what
/// somebody reaches for first and **four of them were already known to be
/// missing** by a test in the consuming application:
///
/// | codepoint | in the font? |
/// |---|---|
/// | `⌄` U+2304 | **no** — what this was |
/// | `▾` U+25BE, `▼` U+25BC, `⌃` U+2303, `˅` U+02C5 | **no** |
/// | `⏷` U+23F7 | **yes** — and its siblings `⏴` U+23F4 / `⏵` U+23F5 are already in use |
///
/// Measured with `Fonts::has_glyph`, not assumed.
///
/// **Why this crate cannot test it and the application must.** `cargo test
/// -p egui-shell` compiles without egui's `default_fonts`, so `has_glyph`
/// here would answer about a font set that does not exist in any real
/// build — the test would pass, vacuously, for the whole life of the
/// defect. The assertion therefore lives with the fonts, in the
/// application, beside the two that already guard the status bar and the
/// find bar. That is also why this shipped: the crate that owns the string
/// is structurally unable to check it.
pub(crate) fn overflow_label(hidden: usize) -> String {
    format!("⏷ {hidden} more")
}

/// The width to reserve for the overflow affordance, given how many
/// groups the band holds **in total**.
///
/// # Why every reachable label is measured, not just the largest count
///
/// The hidden count is not known until [`plan_band`] has run, and
/// [`plan_band`] needs this number as an input. The circularity has to be
/// broken by reserving for a label that has not been chosen yet, and the
/// only safe direction is the worst case.
///
/// An earlier version measured `"⏷ N more"` for `N = total_groups` alone,
/// reasoning that more hidden groups means a longer string. That is true
/// of the *character count* and false of the *width*: with no font
/// installed every label measures zero and the two agree, but with real
/// metrics `"⏷ 8 more"` is wider than `"⏷ 9 more"` in any face whose
/// digits are not tabular, and a band of nine groups showing one would
/// then draw a control wider than the space reserved for it — the
/// affordance overhanging the band's right edge, which is failure mode #8
/// with the control present but partly unclickable.
///
/// So the reservation is `max` over every label the control can ever
/// display: `1..=total_groups`. That makes the claim *"the drawn control
/// never exceeds its reservation"* a proof rather than an argument about
/// digit shapes. The cost is one memoized galley lookup per group per
/// frame — `egui` caches layout jobs, and a band has single-digit numbers
/// of groups.
///
/// `measure` is the caller's text-measuring function, so this stays free
/// of `egui`.
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

    /// Ten groups of 100 pt each, the shape most of these tests want.
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
    ///
    /// `RIBBON_IA.md` §5 is full of two-word captions over one-glyph
    /// controls — "Page display" over a single icon button — so this is
    /// the common case, not the corner. Planning such a group at its
    /// control's width would overflow the band by the difference, every
    /// time, and the symptom would be a clipped caption rather than
    /// anything that looks like a layout bug.
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

    /// **★ A group narrower than [`GROUP_WRAP_WIDTH`] is left alone.**
    ///
    /// The half of the mockup's rule that is easy to lose: `max-width` is a
    /// *trigger*. Wrapping every group would turn a two-control group into
    /// a column of one control per row, which is narrower and is not a
    /// ribbon. Most groups in a real manifest are under the cap, so this is
    /// the common path and not the corner.
    /// ★★★ **A group that asks for two rows gets them, even though one fits.**
    ///
    /// `OPERATOR_REQUESTS.md` O97 — four square icon buttons that are a strip in
    /// one row and a control in a 2 x 2 block. Without the hint the planner
    /// short-circuits on *"it fits"* and never looks for a narrower shape.
    ///
    /// The four widths are comfortably inside [`GROUP_WRAP_WIDTH`], so the
    /// `None` case is the regression guard as much as the `Some` case is the
    /// feature: **if the default ever starts wrapping, every group in every
    /// application changes shape at once.**
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

    /// ★★ **The hint is a preference, not a command.**
    ///
    /// Three properties, and each is a way the feature could have been built
    /// wrong:
    ///
    /// * asking for **one** row changes nothing — that is already the default,
    ///   and a `Some(1)` that started wrapping would be a footgun in a manifest;
    /// * the **band's ceiling wins** — a group asking for four rows in a
    ///   two-row band gets two, because the band's height is fixed (R128) and a
    ///   manifest must not be able to break it;
    /// * asking does not FORCE the count — the planner still returns the
    ///   narrowest packing, so two items whose one-row form is narrowest keep it.
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

    /// **★ A group over the cap is split into two rows, evenly, and is
    /// narrower for it.**
    ///
    /// The measurable claim of the whole change: a wrapped group *costs the
    /// band less*, which is what lets a fifth group fit where three used to
    /// be pushed into the overflow menu.
    ///
    /// The evenness is asserted as a bound rather than as an exact split,
    /// because the optimum depends on the item widths — seven items of 75
    /// divide 4/3, seven of 20 with one of 300 do not divide at all. The
    /// bound that always holds is *"the widest row is no wider than it has
    /// to be"*, and the strongest cheap statement of that is: no row may be
    /// wider than the widest single item plus the balanced share.
    #[test]
    fn a_group_over_the_cap_is_split_evenly_and_costs_less() {
        let widths = vec![75.0; 7]; // 7 × 75 + 6 × 4 = 549, over the cap
        let single = row_width(&widths, 4.0);
        assert!(single > GROUP_WRAP_WIDTH, "the fixture must trip the cap");

        let rows = wrap_group(&widths, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, None);
        assert_eq!(rows.rows(), 2, "two rows, not three and not one");
        assert_eq!(rows.counts, vec![4, 3], "4 + 3 is the even split of seven");
        assert_eq!(rows.width, 4.0f32.mul_add(75.0, 3.0 * 4.0));
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
    ///
    /// The manifest's order is the operator's order — the same rule
    /// [`BandPlan::shown`]'s prefix property exists for. A wrap that
    /// reordered items to pack them better would rearrange the ribbon as
    /// the theme's gutter changed, which is exactly the class of surprise
    /// this module refuses elsewhere.
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
    ///
    /// `max_rows == 1` is the old, one-row band expressed in the new
    /// arithmetic. Keeping it reachable is what makes "two rows" a decision
    /// this module records rather than a behaviour it merely has.
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
    ///
    /// The interesting shape, because the naive answer — "this cannot be
    /// split, give up" — would leave a group at the sum of every item when
    /// one oversized control is present, and oversized controls are exactly
    /// what a long label produces.
    #[test]
    fn one_oversized_control_still_lets_the_rest_wrap() {
        let widths = [500.0_f32, 90.0, 90.0, 90.0, 90.0];
        let rows = wrap_group(&widths, 4.0, GROUP_ROWS, GROUP_WRAP_WIDTH, None);
        assert_eq!(
            rows.counts,
            vec![1, 4],
            "the oversized control takes a row and the other four share the second"
        );
        assert_eq!(
            rows.width, 500.0,
            "the group is as wide as its widest control"
        );
        assert!(rows.width < row_width(&widths, 4.0));
    }

    /// Everything fits: no overflow control, and no width taken for one.
    ///
    /// The second half matters — a reservation that persisted when
    /// nothing was hidden would be a permanent tax on every band that
    /// fits.
    #[test]
    fn a_band_that_fits_reserves_nothing() {
        let plan = plan_band(1000.0, &widths(3, 100.0), 8.0, 60.0);
        assert_eq!(plan.shown, 3);
        assert_eq!(plan.hidden, 0);
        assert_eq!(plan.overflow_width, 0.0);
        assert!(!plan.has_overflow());
    }

    /// **★ Failure mode #8: at a width too narrow for even one group, the
    /// overflow affordance is still planned and still has its width.**
    ///
    /// This is the invariant the whole module exists for. The observed
    /// defect it guards against is `MODES_AND_PANELS.md` Part 2, #8: past
    /// a certain count the overflow button *itself* gets hidden, leaving
    /// no route to what it was hiding. The plan must degrade to "no
    /// groups, one working affordance", never to "some clipped groups, no
    /// affordance".
    ///
    /// Checked at three widths on the way down, because the interesting
    /// failure is not at zero — it is at the width where a naive
    /// implementation still has *just* enough room for a group and
    /// therefore spends the overflow control's space on it.
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
    ///
    /// A biconditional rather than one implication, because both
    /// directions are real defects. Reserving with nothing hidden wastes
    /// band width forever; hiding with nothing reserved is #8 itself.
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
    ///
    /// Monotonicity is what makes a window resize feel like a resize.
    /// A greedy fill has it by construction; a later "pack the widest
    /// first" optimisation would not, and this test is what would refuse
    /// that change.
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
    ///
    /// The prefix property is what stops the visible ordering of the
    /// ribbon from depending on the window width — the manifest's order
    /// is the operator's order, and a plan that dropped a middle group to
    /// fit a later narrow one would rearrange the ribbon as the window
    /// moved.
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
    ///
    /// `egui` hands out `f32::INFINITY` for available width inside an
    /// unbounded container. `INFINITY - overflow_width` is still
    /// infinity, so an unguarded implementation would conclude that
    /// everything fits and emit no affordance — the #8 defect arriving
    /// through arithmetic rather than through ordering.
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

    /// **★ The reservation covers the widest label by WIDTH, not by
    /// character count.**
    ///
    /// The trap this pins is the one real text springs and zero-width
    /// text cannot: in a face whose digits are not tabular, `"⏷ 8 more"`
    /// can be wider than `"⏷ 9 more"` even though the counts and the
    /// lengths say otherwise. Reserving for `N = total_groups` alone —
    /// which reads as obviously sufficient, and is what this function used
    /// to do — then draws a control wider than the space held for it, and
    /// the affordance overhangs the band's right edge.
    ///
    /// The `measure` below is deliberately perverse about exactly that:
    /// every character costs 7 pt except `8`, which costs 40. A
    /// reservation that consults only the largest count fails here; one
    /// that takes the maximum over every reachable label passes.
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

    /// ★ **The chevron is the pinned codepoint — one half of a two-sided
    /// pin, and this half cannot check the thing that actually matters.**
    ///
    /// Whether the character is *drawable* is asserted in the consuming
    /// application (`pdfcer_gui::shell`'s
    /// `the_ribbon_overflow_chevron_has_a_glyph`), because `cargo test -p
    /// egui-shell` compiles without egui's `default_fonts` — a `has_glyph`
    /// call here would answer about a font set no real build has, and would
    /// pass for the whole life of a defect. It did: `⌄` U+2304 shipped as a
    /// tofu box on every ribbon band and dock tab bar this project has
    /// produced.
    ///
    /// So this test does the half it *can* do honestly: pin the codepoint,
    /// so that changing it is a deliberate act which fails a named test and
    /// sends the next reader to the other half. Both docks and both rows
    /// are covered, because [`crate::dock::plan::overflow_label`] promises
    /// to stay identical and identical wording means identical codepoints.
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
