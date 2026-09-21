//! Layout tests that run the **band** against real text metrics.
//!
//! The tab strip, one row up, is the same shape of claim over an independent
//! set of claimants and lives in [`super::strip_width_tests`], which imports
//! [`context`], [`render_shell`] and [`Rendered`] from here rather than
//! standing up a second synthetic face. The argument below governs both files.
//!
//! # Why this file exists, and why the tests next door are not enough
//!
//! Every width-sensitive path in this module — [`super::plan`]'s
//! reservation, [`super::band`]'s budget, [`super::mode_selector`]'s track
//! — is a claim about measured text, and a test that installs no font
//! measures text of **zero width**.
//!
//! `egui-shell` depends on `egui` with `default-features = false`, so a
//! test process building this crate alone has no font data and every
//! galley measures ≈ 0. Under those conditions:
//!
//! - every group is as wide as [`super::plan::MIN_ITEM_WIDTH`] times its
//!   item count, so nothing ever overflows unless a test forces it;
//! - the tab-strip row always fits, so it never grows the enclosing `Ui`
//!   and the band is never told a width the window did not have;
//! - the mode selector's track is always narrower than the row;
//! - `"⏷ 8 more"` and `"⏷ 9 more"` are the same width, so a reservation
//!   sized for the wrong one of them is indistinguishable from a correct
//!   one.
//!
//! Each of those four conditions hides a class of real defect, and hides it
//! in the direction that reads as success: the test runs, asserts, and
//! passes against numbers no glyph produced.
//!
//! # The trap this file is built to close
//!
//! The font situation was not merely absent — it was **inconsistent**:
//!
//! ```text
//! cargo test -p egui-shell --lib   → egui alone         → no fonts
//! cargo test --workspace           → pdfcer-gui → eframe → fonts
//! ```
//!
//! Cargo unifies features across a workspace build, so `pdfcer-gui`'s
//! dependency on `eframe` (which enables `egui/default_fonts`) silently
//! changed what *this crate's own tests* measured. The same source, the
//! same assertions, two different answers, and the narrower command — the
//! one a developer working on the shell would naturally run — was the one
//! that reported success.
//!
//! Every test here therefore installs [`super::testfont`], a synthetic
//! TrueType face built in memory by this crate, and asserts that it took
//! effect before asserting anything else. The numbers below are then
//! **identical under both commands** and cannot be changed by any feature
//! any sibling crate turns on. If someone removes `pdfcer-gui` from the
//! workspace, or adds a crate that pulls in a different font set, these
//! tests measure exactly what they measured today.
//!
//! # What is asserted, and what deliberately is not
//!
//! These are *geometric* assertions: what was drawn, where, and whether it
//! can be clicked. They are not pixel or legibility assertions — that is
//! `ui-verify`'s job against a real window, and it is why the rects are
//! published in the first place. The property under test throughout is
//! `MODES_AND_PANELS.md` Part 2's failure mode #8:
//!
//! > The overflow affordance is reserved space, never the first thing
//! > squeezed out.

use egui::{Pos2, Rect, Vec2};

use crate::commands::{CommandRegistry, ConditionSet};
use crate::manifest::Shell;

use super::tests::{registry, shell};
use super::{Ribbon, RibbonState, report, testfont};

/// Tolerance, in points, for "on screen" and "does not overlap".
///
/// `egui` rounds widget rectangles to whole physical pixels, so an edge
/// can land a fraction of a point beyond an exact arithmetic boundary
/// without anything being wrong. One point is well below anything a person
/// could see and well above the rounding.
pub(super) const SLACK: f32 = 1.0;

/// One rendered ribbon: the state after the frame, and every rect the
/// frame published.
pub(super) struct Rendered {
    pub(super) state: RibbonState,
    pub(super) rects: Vec<(String, Rect)>,
    /// The height the whole ribbon occupied in the `Ui` it was handed,
    /// read back after [`Ribbon::render`] returned.
    ///
    /// **`Option`, and it matters.** A layout test can be entirely vacuous
    /// under one of the two commands above, and an assertion about a number
    /// nobody produced passes exactly like an assertion about a number that
    /// was right. `None` means the closure
    /// that measures never ran, which is a different failure from "the
    /// height was wrong" and gets a different message.
    pub(super) ribbon_height: Option<f32>,
    /// The whole rectangle the ribbon occupied, same frame and same
    /// `Option` discipline as [`Self::ribbon_height`].
    ///
    /// Kept alongside the height rather than replacing it because the two
    /// answer different questions and only one of them is R128's. R128 is
    /// about the *extent* — does the number the canvas sees change when the
    /// tab does. [`super::height_tests::the_band_leaves_clear_space_beneath_its_captions`]
    /// is about the **bottom edge**: how far below the last caption the
    /// ribbon stops, which an extent cannot answer without also knowing where
    /// it started.
    pub(super) ribbon_rect: Option<Rect>,
}

impl Rendered {
    /// The single rect published under `name`, if any.
    pub(super) fn rect(&self, name: &str) -> Option<Rect> {
        self.rects.iter().find(|(n, _)| n == name).map(|(_, r)| *r)
    }

    /// Every rect whose name starts with `prefix`.
    pub(super) fn all(&self, prefix: &str) -> Vec<Rect> {
        self.rects
            .iter()
            .filter(|(n, _)| n.starts_with(prefix))
            .map(|(_, r)| *r)
            .collect()
    }

    /// The height of the **band**, measured from the groups it drew.
    ///
    /// Every group in a band is padded to the same height (see
    /// [`super::band::captioned_group`]'s `rows_height`), so any one of
    /// them reports it — and the maximum is taken rather than the first so
    /// that a group which somehow drew taller than the others is a failure
    /// rather than a coin toss.
    ///
    /// `None` when the band drew no group at all, which is a real state:
    /// at a width narrower than one group plus the overflow reservation,
    /// every group is in the menu. A caller that wants R128's claim at
    /// *those* widths has to ask [`Self::ribbon_height`] instead, and the
    /// two are deliberately separate so neither can be mistaken for the
    /// other.
    pub(super) fn band_height(&self, tab: &str) -> Option<f32> {
        let prefix = format!("ribbon.group.{tab}.");
        self.rects
            .iter()
            // A caption's rect is inside its group's, so it would never win
            // the maximum — but excluding it keeps the claim "this is a
            // group's height" literally true rather than true by luck.
            .filter(|(n, _)| n.starts_with(&prefix) && !n.ends_with(".caption"))
            .map(|(_, r)| r.height())
            .fold(None, |acc: Option<f32>, h| {
                Some(acc.map_or(h, |a| a.max(h)))
            })
    }
}

/// A context with the synthetic face installed and proven to work.
///
/// `install` asserts that text measures non-zero and that the face is
/// proportional, so a test built on this context cannot silently revert to
/// measuring nothing.
pub(super) fn context() -> egui::Context {
    let ctx = egui::Context::default();
    testfont::install(&ctx);
    ctx
}

/// Render the View tab twice at `width` and report the second frame.
///
/// Two frames because `egui` resolves some geometry a frame late; the
/// second is the honest one, exactly as the harness in [`super::tests`]
/// does it.
fn render_view_tab(ctx: &egui::Context, width: f32) -> Rendered {
    render_shell(ctx, &shell(), "view", &ConditionSet::new(), width)
}

/// Render any manifest twice at `width` and report the second frame.
///
/// The generalisation of [`render_view_tab`], for the tab-strip tests:
/// those need a manifest with enough tabs to overflow a strip, which the
/// two-tab fixture in [`super::tests`] deliberately is not.
pub(super) fn render_shell(
    ctx: &egui::Context,
    shell: &Shell,
    active_tab: &str,
    conditions: &ConditionSet,
    width: f32,
) -> Rendered {
    render_shell_with(ctx, shell, &registry(), active_tab, conditions, width)
}

/// [`render_shell`] against a caller-supplied [`CommandRegistry`].
///
/// The two-row tests need labels long enough to trip
/// [`super::plan::GROUP_WRAP_WIDTH`], and the shared fixture registry is
/// deliberately small and short-labelled — widening it would change the
/// measured numbers in every other test in this file for a reason that has
/// nothing to do with what they are about.
pub(super) fn render_shell_with(
    ctx: &egui::Context,
    shell: &Shell,
    registry: &CommandRegistry,
    active_tab: &str,
    conditions: &ConditionSet,
    width: f32,
) -> Rendered {
    let mut state = RibbonState::new();
    state.set_active_tab(active_tab);

    let mut rects = Vec::new();
    let mut ribbon_height = None;
    let mut ribbon_rect = None;
    for _ in 0..2 {
        rects.clear();
        ribbon_height = None;
        ribbon_rect = None;
        let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 400.0))),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let _ = Ribbon::new()
                .with_conditions(conditions)
                .reporting_rects_to(&mut sink)
                .render(ui, shell, registry, &mut state);
            // Read back what the ribbon actually consumed of the `Ui` it was
            // handed. This — not any rect the ribbon publishes — is the
            // number the canvas below it sees, and R128 is a claim about
            // that number.
            ribbon_height = Some(ui.min_rect().height());
            ribbon_rect = Some(ui.min_rect());
        });
    }
    Rendered {
        state,
        rects,
        ribbon_height,
        ribbon_rect,
    }
}

/// **The ribbon really is measuring real text.**
///
/// The guard on every other test in this file. With no font data the
/// three View groups would each collapse to their item count times
/// [`super::plan::MIN_ITEM_WIDTH`] and would come out *identical* in
/// width; with a proportional face they differ, because "Page display"
/// over three buttons is not the same width as "Render" over two.
///
/// Asserted through the **published rects** rather than through
/// `layout_no_wrap` directly, because what needs proving is not that the
/// font loaded — [`testfont::install`] proves that — but that the font
/// reaches the ribbon's own measurement path.
#[test]
fn the_band_measures_real_text_and_not_a_floor() {
    let ctx = context();
    let wide = render_view_tab(&ctx, 1600.0);

    let page_display = wide
        .rect(&report::group("view", "page_display"))
        .expect("the band publishes its groups");
    let render = wide
        .rect(&report::group("view", "render"))
        .expect("the band publishes its groups");

    // The floor is DERIVED from the rule, never lifted from one arrangement.
    // What this test is for is that real glyph metrics reach the ribbon's
    // measurement path; with no font every control collapses to
    // [`super::plan::MIN_ITEM_WIDTH`], so that is the floor to clear — under
    // any arrangement, because a stacked group is at least one control wide
    // and a row of three is wider still. A literal taken from the one-row
    // arrangement instead (100 pt, these three controls side by side) accuses
    // a correct build of having lost its font the moment the group stacks
    // into columns and its widest control measures 86.94 pt. The `assert_ne!`
    // below carries the other half of the claim.
    let floor = super::plan::MIN_ITEM_WIDTH;
    assert!(
        page_display.width() > floor * 2.0,
        "a group of three labelled controls measured {} pt against a no-font \
         floor of {floor} pt per control — that is the floor, not real text",
        page_display.width()
    );
    assert_ne!(
        page_display.width(),
        render.width(),
        "two groups with different labels came out the same width, which only \
         happens when the text is not being measured"
    );

    let caption = wide
        .rect(&report::group_caption("view", "page_display"))
        .expect("every group publishes its caption");
    assert!(
        caption.width() > 10.0,
        "the caption \"Page display\" measured {} pt wide",
        caption.width()
    );
    assert!(
        page_display.contains_rect(caption),
        "the caption is outside its own group once text has a width: {caption:?} \
         is not inside {page_display:?}"
    );
}

/// **Failure mode #8, swept: the overflow affordance is on screen at
/// every width, with real text.**
///
/// Correct reservation arithmetic is not enough on its own. The band asks a
/// `Ui` for its width, and if the tab-strip row above it has overflowed and
/// grown that `Ui`'s `max_rect`, the reservation is taken from a right edge
/// off screen — measured at 78 pt past it. The affordance then exists, has
/// area, is reported, and cannot be clicked, which is a state no arithmetic
/// test can see.
///
/// The sweep matters. A single narrow width would have caught this one,
/// but the family of bugs it belongs to is "at *some* width the answer is
/// wrong", and the interesting widths are the ones where a naive
/// implementation still has *just* enough room for one more group and
/// spends the affordance's space on it.
#[test]
fn the_overflow_affordance_is_on_screen_at_every_width() {
    let ctx = context();
    for width in (60..1400).step_by(17).map(|w| w as f32) {
        let frame = render_view_tab(&ctx, width);
        let report = frame.state.last_frame();

        assert_eq!(
            report.groups_in_band + report.groups_overflowed,
            3,
            "at {width} pt a group was neither drawn in the band nor moved to the \
             menu, so it is unreachable"
        );

        if !report.overflow_visible {
            assert_eq!(
                report.groups_overflowed, 0,
                "at {width} pt groups were hidden with no affordance to reach them \
                 — failure mode #8 exactly"
            );
            continue;
        }

        let rect = frame
            .rect(report::overflow())
            .expect("a visible affordance publishes its rect");
        assert!(
            rect.width() > 0.0 && rect.height() > 0.0,
            "at {width} pt the affordance was allocated with no area: {rect:?}"
        );
        assert!(
            rect.left() >= -SLACK && rect.right() <= width + SLACK,
            "at {width} pt the affordance was placed off-screen at {rect:?}"
        );
    }
}

/// **No visible group reaches into the reserved space.**
///
/// The other half of the reservation: it is worth nothing if a group
/// drawn under real metrics overruns its budget and paints on top of the
/// affordance. `plan_band` subtracts the reservation before measuring, and
/// `render_band` hands the groups a `Ui` whose `max_rect` stops there —
/// this asserts that the two together actually hold once the widths are
/// real numbers rather than zeros.
#[test]
fn no_visible_group_overlaps_the_overflow_affordance() {
    let ctx = context();
    for width in (60..1400).step_by(23).map(|w| w as f32) {
        let frame = render_view_tab(&ctx, width);
        if !frame.state.last_frame().overflow_visible {
            continue;
        }
        let affordance = frame
            .rect(report::overflow())
            .expect("a visible affordance publishes its rect");

        for group in frame.all("ribbon.group.view.") {
            assert!(
                group.right() <= affordance.left() + SLACK,
                "at {width} pt a band group at {group:?} runs into the space \
                 reserved for the affordance at {affordance:?}"
            );
        }
    }
}

/// **When the plan says everything fits, everything actually fits.**
///
/// This is the estimate-accuracy check, asked through the renderer rather
/// than of the estimator, and it is the one that would catch an
/// **under**-estimate — the dangerous direction.
///
/// [`super::plan`] budgets groups analytically, from item labels plus
/// padding constants, because immediate mode cannot measure a group before
/// deciding whether to draw it. If those constants disagree with what
/// `egui` actually applies (the icon/label gap being `icon_spacing` rather
/// than the theme's `gutter` is exactly such a disagreement, and was one),
/// the plan concludes that three groups fit in a band that holds two and a
/// half, and the third is drawn off the right-hand edge with no affordance
/// offering it — because the plan does not think anything is hidden.
///
/// The property is checkable without knowing the estimate: on any frame
/// with **no** overflow, every group the band drew must lie inside the
/// window.
///
/// # Why this binary-searches rather than sweeps
///
/// The only widths at which an under-estimate is *visible* are the ones
/// between "the plan stopped overflowing" and "the content genuinely
/// fits". An estimate that is short by 8 pt makes that window 8 pt wide,
/// and a sweep at any step coarser than the error walks straight over it —
/// which was measured, not assumed: with the item padding deliberately
/// cut to a fifth, an 11 pt sweep still reported success.
///
/// So the test finds the transition width exactly (the counts are
/// monotonic in width, which
/// `widening_the_band_never_hides_a_group_under_real_metrics` pins
/// separately) and asserts **there**, where any shortfall at all is
/// visible, plus at the two widths above it.
///
/// # Its sensitivity floor
///
/// [`super::plan::GROUP_PADDING`] contributes 2 × 6 pt to every group's
/// planned width, and [`super::band::captioned_group`] insets the group by
/// that same constant — so the reservation is spent on ink rather than
/// standing as slack. That coupling is what gives this test its sensitivity.
/// A band that planned the padding without drawing it would carry 12 pt per
/// group of accidental safety margin, and any under-estimate smaller than
/// the margin would be absorbed and invisible here: measured against such a
/// band, cutting the item padding by 20 % changed nothing, and only removing
/// it outright failed at the transition width.
///
/// The test itself asserts only that a band claiming to fit really fits —
/// a claim about the renderer — so it needs no adjustment when the padding
/// moves. What moves with the padding is the smallest under-estimate it can
/// still see.
#[test]
fn a_band_that_claims_to_fit_really_does_fit() {
    let ctx = context();

    // Binary search for the narrowest whole-point width at which the band
    // claims everything fits. `lo` always overflows, `hi` never does.
    let fits = |w: i32| {
        !render_view_tab(&ctx, w as f32)
            .state
            .last_frame()
            .overflow_visible
    };
    let (mut lo, mut hi) = (60_i32, 1600_i32);
    assert!(!fits(lo), "the search needs a width that does overflow");
    assert!(fits(hi), "the search needs a width that does not");
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if fits(mid) { hi = mid } else { lo = mid }
    }

    for width in [hi, hi + 1, hi + 2].map(|w| w as f32) {
        let frame = render_view_tab(&ctx, width);
        let report = frame.state.last_frame();
        assert_eq!(
            report.groups_in_band, 3,
            "at {width} pt no affordance was drawn, so every group must be in the band"
        );
        assert!(
            !report.overflow_visible,
            "at {width} pt — at or above the transition — the band should claim to fit"
        );
        for group in frame.all("ribbon.group.view.") {
            assert!(
                group.right() <= width + SLACK && group.left() >= -SLACK,
                "at {width} pt — the narrowest width at which the plan concludes \
                 that all three groups fit — a group was drawn at {group:?}, outside \
                 the window. The width estimate is smaller than what the group \
                 actually occupies, which is the direction that loses a control with \
                 no affordance to reach it"
            );
        }
    }
}

/// **A band narrower than the affordance itself keeps the affordance.**
///
/// The degenerate case the reservation exists for, and the one the
/// arithmetic alone cannot answer: at 40 pt the band cannot fit the
/// control it is obliged to show. `MODES_AND_PANELS.md` #8 dictates what
/// gives — not the affordance. So the control is clamped into the band
/// rather than positioned by subtraction from the right edge, its label
/// truncates, and the shortfall is disclosed through the verification
/// channel as `ribbon-overflow-affordance-clamped`.
///
/// What is asserted is the part that matters to an operator: the control
/// is fully on screen, has area, and every group is reachable through it.
#[test]
fn a_band_narrower_than_the_affordance_still_shows_it() {
    let ctx = context();
    let narrow = 40.0;
    let frame = render_view_tab(&ctx, narrow);
    let report = frame.state.last_frame();

    assert!(
        report.overflow_visible,
        "at {narrow} pt nothing fits, so the affordance is the only route to any \
         group — it must be drawn"
    );
    assert_eq!(
        report.groups_in_band, 0,
        "no group can fit beside the reservation at {narrow} pt"
    );
    assert_eq!(report.groups_overflowed, 3, "so all three are in the menu");

    let rect = frame
        .rect(report::overflow())
        .expect("a visible affordance publishes its rect");
    assert!(
        rect.left() >= -SLACK && rect.right() <= narrow + SLACK,
        "the affordance hung off a band too narrow to hold it: {rect:?} in a \
         {narrow} pt band. The label is what gives, never the position"
    );
    assert!(
        rect.width() > 0.0 && rect.height() > 0.0,
        "the affordance was clamped out of existence: {rect:?}"
    );
}

/// **The affordance can actually be hit, at widths where it is
/// crowded.**
///
/// A rectangle proves something was allocated; only `egui`'s own hit test
/// proves it can be reached, because that is what accounts for clipping,
/// for occlusion by a later widget and for a zero-area interact rect.
///
/// Checked at three widths: one where a group still fits beside the
/// affordance, one where none does, and one narrower than the affordance
/// itself.
///
/// Those widths are a property of the fixture manifest and the synthetic
/// face, not constants with meaning of their own: `band::measure_group_rows`
/// asks every group for the band's row ceiling, so the View tab's groups
/// stack into columns and fit in less width than a single-row arrangement
/// needs. If the `overflow_visible` precondition goes red, the question is
/// *"do the groups still not fit?"* and the answer is a measurement, not a
/// smaller literal. That precondition is why a change to the band's
/// arrangement surfaces here as a red test naming its own vacuity, rather
/// than as a green test asserting nothing.
#[test]
fn the_affordance_is_hit_testable_under_real_metrics() {
    for width in [300.0_f32, 180.0, 40.0] {
        let ctx = context();
        let frame = render_view_tab(&ctx, width);
        let report = frame.state.last_frame().clone();
        assert!(
            report.overflow_visible,
            "at {width} pt the View tab cannot fit three groups, so this test is \
             no longer exercising the affordance"
        );
        let rect = frame
            .rect(report::overflow())
            .expect("a visible affordance publishes its rect");
        let id = report
            .overflow_id
            .expect("a visible affordance publishes its id");

        // Re-render with the pointer over the control's own centre.
        let shell = shell();
        let registry = registry();
        let mut state = frame.state;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 400.0))),
            events: vec![egui::Event::PointerMoved(rect.center())],
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let _ = Ribbon::new().render(ui, &shell, &registry, &mut state);
        });

        let response = ctx
            .read_response(id)
            .expect("the affordance must be a widget egui knows about");
        assert!(
            response.hovered(),
            "at {width} pt the affordance is reported at {rect:?} but cannot be hit \
             at its own centre {:?} — which is the state failure mode #8 describes",
            rect.center()
        );
    }
}

/// **Widening the window never hides a group that was visible.**
///
/// [`super::plan`] asserts this of the arithmetic. This asserts it of the
/// whole renderer with real text, where the group widths are not the
/// uniform 100 pt of the unit test and where a mis-measured group would
/// show up as a band that flickers a group in and out as the window is
/// dragged.
#[test]
fn widening_the_band_never_hides_a_group_under_real_metrics() {
    let ctx = context();
    let mut previous = 0;
    for width in (60..1400).step_by(19).map(|w| w as f32) {
        let frame = render_view_tab(&ctx, width);
        let shown = frame.state.last_frame().groups_in_band;
        assert!(
            shown >= previous,
            "widening to {width} pt dropped a group that fitted at a narrower width \
             ({shown} shown, was {previous})"
        );
        previous = shown;
    }
    assert_eq!(
        previous, 3,
        "at 1400 pt every View group must be in the band"
    );
}

/// **The mode selector stays on screen when the row cannot hold it.**
///
/// Two things on this ribbon must never be squeezed out by content: the
/// mode selector and the overflow affordance ([`super`]'s header owns that
/// rule). Laying the selector out first, from the right edge, delivers it
/// against content. It delivers nothing when the selector alone is wider
/// than the row: `egui` answers an over-wide
/// `allocate_exact_size` in a right-to-left layout by extending past the
/// container's left edge, silently.
///
/// With the synthetic face a three-position *Read · Review · Edit*
/// selector wants ~150 pt, so the sweep starts well below that and the
/// clamp in [`super::mode_selector::fit_track`] is what is under test. Its
/// failure mode without the clamp is a first position at a negative x —
/// drawn, reported, and unclickable.
#[test]
fn the_mode_selector_stays_within_the_row_at_every_width() {
    let ctx = context();
    for width in (60..900).step_by(13).map(|w| w as f32) {
        let frame = render_view_tab(&ctx, width);
        let track = frame
            .rect(report::mode_selector())
            .expect("a manifest with modes publishes its selector track");
        assert!(
            track.left() >= -SLACK && track.right() <= width + SLACK,
            "at {width} pt the mode selector's track is off screen at {track:?}"
        );

        for mode in ["read", "review", "edit"] {
            let segment = frame
                .rect(&report::mode_segment(mode))
                .unwrap_or_else(|| panic!("mode `{mode}` published no segment at {width} pt"));
            assert!(
                segment.left() >= -SLACK && segment.right() <= width + SLACK,
                "at {width} pt the `{mode}` position is off screen at {segment:?}, so \
                 it cannot be clicked"
            );
            assert!(
                segment.width() > 0.0,
                "at {width} pt the `{mode}` position has no width, so its label \
                 cannot be visible"
            );
        }
    }
}

/// **The overflow reservation is wide enough for the label it will
/// actually draw.**
///
/// The circularity in [`super::plan::overflow_width`] — the reservation is
/// needed before the hidden count is known — is broken by reserving for
/// the worst case. With no fonts, *every* label measures zero and the
/// worst case is a tautology; with a proportional face it is a real claim,
/// and the naive version of it ("reserve for the largest count, since it
/// has the most characters") is false in any face whose digits are not
/// tabular.
///
/// This asserts it directly: for a band of `n` groups, the reservation is
/// at least the width of every label the control could ever show.
#[test]
fn the_reservation_covers_every_label_the_control_could_show() {
    let ctx = context();
    // One empty frame so `fonts_mut` is available; `install` has already
    // run one, but the borrow below wants the current frame's fonts.
    let measure = |s: &str| {
        ctx.fonts_mut(|f| {
            f.layout_no_wrap(
                s.to_owned(),
                egui::FontId::proportional(14.0),
                egui::Color32::PLACEHOLDER,
            )
            .size()
            .x
        })
    };

    for total in 1..14_usize {
        let reserved = super::plan::overflow_width(total, 8.0, measure);
        for hidden in 1..=total {
            let label = super::plan::overflow_label(hidden);
            assert!(
                reserved >= measure(&label) + 8.0 - f32::EPSILON,
                "a band of {total} groups reserved {reserved} pt, but showing \
                 {hidden} hidden groups draws {label:?} at {} pt plus padding — \
                 the control would overhang its own reservation",
                measure(&label)
            );
        }
    }
}
