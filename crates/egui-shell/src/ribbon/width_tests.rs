//! Layout tests that run the ribbon against **real text metrics**.
//!
//! # ★ Why this file exists, and why the tests next door were not enough
//!
//! Every width-sensitive path in this module — [`super::plan`]'s
//! reservation, [`super::band`]'s budget, [`super::mode_selector`]'s track
//! — had, until this file was written, only ever been executed against
//! text of **zero width**.
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
//! Every one of those four sentences hid a real defect. Two of them were
//! shipped defects (`the_overflow_control_is_hit_testable_at_a_width_that_hides_groups`
//! and `groups_in_the_overflow_menu_are_captioned_too` both failed the
//! moment fonts appeared); the other two were latent.
//!
//! # The trap this file is built to close
//!
//! The font situation was not merely absent — it was **inconsistent**:
//!
//! ```text
//! cargo test -p egui-shell --lib   → egui alone         → no fonts  → 116 pass
//! cargo test --workspace           → pdfcer-gui → eframe → fonts     → 2 fail
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
use crate::manifest::{Group, Item, Mode, Shell, Tab};

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
    /// **`Option`, and it matters.** `HANDOFF.md` §10: a layout test can be
    /// entirely vacuous under one of the two test commands, and an
    /// assertion about a number nobody produced passes exactly like an
    /// assertion about a number that was right. `None` means the closure
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
fn render_shell(
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

/// The ids of [`strip_shell`]'s ordinary tabs, in manifest order.
///
/// Seven, which is the count `MODES_AND_PANELS.md` failure mode #8 names
/// (*"past ~6 tabs the overflow button itself gets hidden"*), and enough
/// that no realistic test window fits them all.
const STRIP_TABS: [&str; 7] = [
    "file", "view", "pages", "edit", "markup", "measure", "tools",
];

/// A manifest built to make the **tab strip** overflow, rather than the
/// band.
///
/// Seven ordinary tabs with deliberately unequal label widths, one
/// contextual tab, the same three modes and the same two-control QAT as
/// [`super::tests::shell`] — so the row has all four claimants on it
/// (QAT, tabs, tab affordance, selector) and the reservation order is
/// actually under test rather than assumed.
///
/// Each tab carries **one small group**, so the band never overflows at
/// the widths these tests use. That is deliberate: a frame in which both
/// the strip and the band have an affordance is a frame in which a test
/// asserting "the affordance is on screen" might be reading the wrong one,
/// and `report::tab_overflow()` and `report::overflow()` exist as separate
/// names precisely so it cannot.
fn strip_shell() -> Shell {
    let mut shell = Shell::new()
        .with_mode(Mode::new("read", "Read", STRIP_TABS))
        .with_mode(Mode::new("review", "Review", STRIP_TABS))
        .with_mode(Mode::new("edit", "Edit", STRIP_TABS));
    for (i, id) in STRIP_TABS.iter().enumerate() {
        // Unequal labels: "File", "View page", "Pages page page", … so the
        // greedy fill has to make a real decision rather than dividing a
        // uniform row.
        let label = std::iter::once(capitalised(id))
            .chain(std::iter::repeat_n("page".to_owned(), i))
            .collect::<Vec<_>>()
            .join(" ");
        shell = shell
            .with_tab(Tab::new(*id, label).with_groups([
                Group::new("g", "Group").with_items([Item::command("view.single")]),
            ]));
    }
    shell
        .with_contextual_tab(
            Tab::new("format", "Format")
                .with_visible_when("selection.any")
                .with_groups([
                    Group::new("style", "Style").with_items([Item::command("format.colour")])
                ]),
        )
        .with_qat(["file.open", "file.save_copy"])
}

/// `"pages"` → `"Pages"`. A test fixture's label, not a UI string.
fn capitalised(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// **★ The ribbon really is measuring real text.**
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

    assert!(
        page_display.width() > 100.0,
        "a group of three labelled controls measured {} pt — that is the \
         no-font floor, not real text",
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

/// **★ Failure mode #8, swept: the overflow affordance is on screen at
/// every width, with real text.**
///
/// The defect this replaces was not a mistake in the reservation
/// arithmetic — that arithmetic was and is correct. It was that the band
/// asked a `Ui` for its width *after* the tab-strip row above it had
/// overflowed and grown that `Ui`'s `max_rect`, so the reservation was
/// taken from a right edge 78 pt off screen. The affordance existed, had
/// area, was reported, and could not be clicked.
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

/// **★ No visible group reaches into the reserved space.**
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

/// **★ When the plan says everything fits, everything actually fits.**
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
/// # Its sensitivity floor, and why it improved on 2026-08-14
///
/// [`super::plan::GROUP_PADDING`] contributes 2 × 6 pt to every group's
/// planned width. Until 2026-08-14 **nothing drew it**, so every band was
/// over-planned by 12 pt per group and that surplus acted as an accidental
/// safety margin: an under-estimate smaller than it was absorbed and this
/// test could not see it. Measured at the time — cutting the item padding by
/// 20 % was invisible here; removing it entirely failed at the transition
/// width.
///
/// [`super::band::captioned_group`] now insets the group by that same
/// constant, so the surplus is spent on ink rather than on slack and this
/// test is 12 pt per group **more** sensitive than it was. Nothing about it
/// had to change: it asserts that a band claiming to fit really fits, which
/// is a claim about the renderer, and the renderer now consumes what the plan
/// reserves. It is worth knowing that the floor moved, because a future
/// under-estimate this test starts catching will look like a new defect and
/// will in fact be an old one that finally became visible.
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

/// **★ A band narrower than the affordance itself keeps the affordance.**
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

/// **★ The affordance can actually be hit, at widths where it is
/// crowded.**
///
/// A rectangle proves something was allocated; only `egui`'s own hit test
/// proves it can be reached, because that is what accounts for clipping,
/// for occlusion by a later widget and for a zero-area interact rect.
///
/// Checked at three widths: one where a group still fits beside the
/// affordance, one where none does, and one narrower than the affordance
/// itself.
#[test]
fn the_affordance_is_hit_testable_under_real_metrics() {
    for width in [400.0_f32, 180.0, 40.0] {
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

/// **★ The mode selector stays on screen when the row cannot hold it.**
///
/// [`super`]'s header states the rule — *"two things on this ribbon must
/// never be squeezed out by content: the mode selector and the overflow
/// affordance"* — and laying the selector out first, from the right edge,
/// delivers it against content. It delivers nothing when the selector
/// alone is wider than the row: `egui` answers an over-wide
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

/// **★ The overflow reservation is wide enough for the label it will
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

// =====================================================================
// The tab-strip row
//
// Everything below is `MODES_AND_PANELS.md` failure mode #8 one row up.
// The defect these replace, measured with the synthetic face and the
// two-tab fixture before `super::strip` existed:
//
//     window  QAT             tabs                selector      verdict
//      500    0..166          188..265            322..500      correct
//      320    0..166          188..265            142..320      tabs UNDER selector
//      180   -6..160          182..259              2..180      both tabs off screen
//
// Reserving the right island protected the right island; `egui` does not
// clip children to `max_rect`, so everything else simply ran off the edge.
// =====================================================================

/// Every rect the tab strip published this frame, as
/// `(name, rect)` pairs, excluding the band's.
fn strip_rects(frame: &Rendered) -> Vec<(String, Rect)> {
    frame
        .rects
        .iter()
        .filter(|(n, _)| {
            n.starts_with("ribbon.tab.")
                || n.starts_with("ribbon.qat.")
                || n == report::tab_overflow()
                || n == report::mode_selector()
        })
        .cloned()
        .collect()
}

/// **★ Failure mode #8 on the tab strip, swept: nothing on the row is
/// ever off screen, at any width.**
///
/// The single assertion the whole of [`super::strip`] exists to make true.
/// It covers all four claimants at once — the QAT, every tab, the strip's
/// affordance and the mode selector — because the defect was not in any
/// one of them: it was that only *one* of them was reserved and the rest
/// were laid out into whatever was left, which at 180 pt was a negative
/// coordinate.
///
/// The sweep step is 7 pt, which is deliberately finer than a "few
/// samples" test and deliberately **not** relied on for the estimate
/// check — see `the_strip_that_claims_to_fit_really_does_fit`, which
/// binary-searches because a sweep at any step coarser than the estimation
/// error walks straight over it.
#[test]
fn nothing_on_the_tab_strip_row_is_ever_off_screen() {
    let ctx = context();
    let shell = strip_shell();
    for width in (40..1600).step_by(7).map(|w| w as f32) {
        let frame = render_shell(&ctx, &shell, "file", &ConditionSet::new(), width);
        let published = strip_rects(&frame);
        assert!(
            !published.is_empty(),
            "at {width} pt the tab-strip row published nothing at all"
        );
        for (name, rect) in published {
            assert!(
                rect.left() >= -SLACK && rect.right() <= width + SLACK,
                "at {width} pt `{name}` is at {rect:?}, outside the window. A \
                 control drawn off the edge is still allocated, still reported and \
                 still has a `Response` — it simply cannot be seen or clicked, \
                 which is exactly what failure mode #8 describes"
            );
            assert!(
                rect.width() >= 0.0,
                "at {width} pt `{name}` was allocated an inverted rect {rect:?}"
            );
        }
    }
}

/// **★ No tab is ever drawn under the mode selector.**
///
/// The specific shape the old layout failed in at 320 pt: the tabs ran
/// from 188 to 265 while the selector ran from 142 to 320, so the two
/// overlapped by 77 pt and the tabs were the ones underneath. Nothing was
/// off screen and nothing looked wrong in the reported rects; the tabs
/// were simply unreachable.
///
/// Asserted as a geometric relation (`tab.right ≤ selector.left`) rather
/// than as coordinates, so it survives a fourth mode, a reworded label and
/// a theme change — which is the whole reason the rects are published.
///
/// The QAT is included on the same principle: the row is ordered
/// `QAT → tabs → affordance → selector` and every adjacent pair must
/// respect it.
#[test]
fn the_tab_strip_never_runs_under_the_mode_selector_or_the_qat() {
    let ctx = context();
    let shell = strip_shell();
    for width in (40..1600).step_by(11).map(|w| w as f32) {
        let frame = render_shell(&ctx, &shell, "file", &ConditionSet::new(), width);
        // The selector may be absent: at a width narrower than the tabs'
        // own floor there is nothing left for it, and a track drawn into a
        // zero-width region would be laid out at its full natural size,
        // over the tabs. See `strip::render`.
        let Some(selector) = frame.rect(report::mode_selector()) else {
            continue;
        };
        let qat_right = frame
            .all("ribbon.qat.")
            .into_iter()
            .fold(f32::NEG_INFINITY, |acc, r| acc.max(r.right()));

        let affordance = frame.rect(report::tab_overflow());
        if let Some(a) = affordance {
            assert!(
                a.right() <= selector.left() + SLACK,
                "at {width} pt the strip's affordance at {a:?} runs under the mode \
                 selector at {selector:?}"
            );
        }

        for (name, tab) in frame
            .rects
            .iter()
            .filter(|(n, _)| n.starts_with("ribbon.tab."))
        {
            // Tabs in the OPEN overflow menu are drawn in a popup layer and
            // are not on this row; the menu is shut on these frames, so
            // every published tab is a strip tab.
            assert!(
                tab.right() <= selector.left() + SLACK,
                "at {width} pt `{name}` at {tab:?} is drawn under the mode selector \
                 at {selector:?} — present, reported, and unclickable"
            );
            if let Some(a) = affordance {
                assert!(
                    tab.right() <= a.left() + SLACK,
                    "at {width} pt `{name}` at {tab:?} reaches into the space \
                     reserved for the strip's affordance at {a:?}"
                );
            }
            if qat_right.is_finite() {
                assert!(
                    tab.left() >= qat_right - SLACK,
                    "at {width} pt `{name}` at {tab:?} is drawn over the QAT, which \
                     ends at {qat_right}"
                );
            }
        }
    }
}

/// **★ The active tab is pinned: it is on screen at every width the strip
/// can hold a tab at all, whichever tab it is.**
///
/// Requirement 2, asserted through the renderer rather than through the
/// plan. The interesting case is the **last** tab being active — the one a
/// prefix-filling planner drops first — so every tab in turn is made
/// active and swept.
///
/// What makes this a *rendered* claim rather than a repeat of
/// `the_active_tab_is_always_shown_and_never_hidden` is the second
/// assertion: the tab is not merely in the plan's `shown` list, its
/// published rectangle is inside the window and has area. A plan that
/// pinned a tab into a zero-width slot would satisfy the pure test and
/// fail here.
///
/// # The one exception, asserted rather than skipped
///
/// Below about 47 pt of tab area the strip **collapses**: it cannot hold a
/// tab and an affordance at sizes `egui` will draw, and #8 decides which
/// survives — see [`super::plan::plan_tab_strip`]'s collapse section. The
/// branch below does not quietly `continue` past that; it asserts the
/// collapsed contract instead, because "the active tab is not in the
/// strip" is only acceptable while *every* tab is reachable through the
/// affordance, and a bug that collapsed the strip at 900 pt would
/// otherwise slip through as a skipped iteration.
#[test]
fn the_active_tab_is_on_screen_at_every_width_whichever_tab_it_is() {
    let ctx = context();
    let shell = strip_shell();
    for active in STRIP_TABS {
        for width in (40..900).step_by(13).map(|w| w as f32) {
            let frame = render_shell(&ctx, &shell, active, &ConditionSet::new(), width);
            let report = frame.state.last_frame();
            assert_eq!(
                frame.state.active_tab(),
                Some(active),
                "at {width} pt the ribbon lost the active tab entirely"
            );

            if report.tab_strip_collapsed {
                assert!(
                    report.tab_overflow_visible,
                    "at {width} pt the strip collapsed with no affordance, so not one \
                     of its {} tabs can be reached — failure mode #8 exactly",
                    report.tabs_visible
                );
                assert_eq!(
                    report.tabs_overflowed, report.tabs_visible,
                    "at {width} pt the strip collapsed, so the menu must hold every \
                     tab — the active one included"
                );
                continue;
            }

            let rect = frame.rect(&report::tab(active)).unwrap_or_else(|| {
                panic!(
                    "at {width} pt the active tab `{active}` published no rect, and \
                     the strip did not collapse — it is either in the overflow menu, \
                     which is the one place a pinned tab must never be, or it was not \
                     drawn at all"
                )
            });
            assert!(
                rect.left() >= -SLACK && rect.right() <= width + SLACK,
                "at {width} pt the active tab `{active}` is at {rect:?}, off screen"
            );
            assert!(
                rect.width() > 0.0 && rect.height() > 0.0,
                "at {width} pt the active tab `{active}` was pinned into a slot with \
                 no area: {rect:?}. A tab that truncates to nothing has disappeared, \
                 which is what the pin exists to prevent"
            );
        }
    }
}

/// **★ The collapse happens only where it must, and only downwards.**
///
/// The guard on the exception the test above carves out. A collapse is a
/// real loss — the strip stops showing which tab is current — so it must
/// be confined to widths where the alternative is worse, and it must be
/// **monotonic**: once the window is wide enough for a tab strip, widening
/// it further can never take the strip away again.
///
/// Without this, "the strip collapsed" would be an escape hatch that a
/// regression could widen indefinitely while every other test kept
/// passing by taking the collapsed branch.
#[test]
fn the_strip_collapses_only_at_widths_too_narrow_to_hold_a_tab() {
    let ctx = context();
    let shell = strip_shell();
    let mut narrowest_uncollapsed = f32::INFINITY;
    let mut widest_collapsed = f32::NEG_INFINITY;

    for width in (40..1600).step_by(3).map(|w| w as f32) {
        let frame = render_shell(&ctx, &shell, "file", &ConditionSet::new(), width);
        let report = frame.state.last_frame();
        if report.tab_strip_collapsed {
            widest_collapsed = widest_collapsed.max(width);
            assert_eq!(
                report.tabs_in_strip, 0,
                "at {width} pt the strip reported a collapse and drew {} tabs anyway",
                report.tabs_in_strip
            );
        } else {
            narrowest_uncollapsed = narrowest_uncollapsed.min(width);
        }
    }

    assert!(
        widest_collapsed < narrowest_uncollapsed,
        "the collapse is not monotonic in width: the strip showed tabs at \
         {narrowest_uncollapsed} pt and then collapsed again at {widest_collapsed} pt, \
         so widening the window can take the tab strip away"
    );
    assert!(
        widest_collapsed < 200.0,
        "the strip collapsed at {widest_collapsed} pt. Collapsing is the one state in \
         which the active tab is not visible, and it is justified only where a tab and \
         an affordance cannot both be drawn — around 47 pt of tab area, not a fifth of \
         a realistic window"
    );
}

/// **★ The strip's affordance can actually be hit, at widths where it is
/// crowded.**
///
/// A rectangle proves something was allocated; only `egui`'s own hit test
/// proves it can be reached, because that is what accounts for clipping,
/// for occlusion by a later widget and for a zero-area interact rect. The
/// band's affordance has the same test for the same reason
/// (`the_affordance_is_hit_testable_under_real_metrics`); this is its
/// counterpart one row up, and the row up is the one where the old code
/// drew controls at negative coordinates.
///
/// Three widths: one where several tabs still fit beside it, one where
/// almost none do, and one narrower than the affordance itself — the case
/// where [`super::plan::plan_tab_strip`] has to divide the shortfall
/// between the affordance and the pinned tab.
#[test]
fn the_tab_overflow_affordance_is_hit_testable_under_real_metrics() {
    let shell = strip_shell();
    for width in [500.0_f32, 260.0, 120.0] {
        let ctx = context();
        let frame = render_shell(&ctx, &shell, "file", &ConditionSet::new(), width);
        let report = frame.state.last_frame().clone();
        assert!(
            report.tab_overflow_visible,
            "at {width} pt seven tabs, a QAT and a three-position selector cannot \
             all fit, so this test is no longer exercising the affordance"
        );
        let rect = frame
            .rect(report::tab_overflow())
            .expect("a visible affordance publishes its rect");
        let id = report
            .tab_overflow_id
            .expect("a visible affordance publishes its id");

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
            "at {width} pt the strip's affordance is reported at {rect:?} but cannot \
             be hit at its own centre {:?} — which is the state failure mode #8 \
             describes",
            rect.center()
        );
    }
}

/// **★ When the strip claims everything fits, everything actually fits.**
///
/// The estimate-accuracy check for the tab strip, asked through the
/// renderer, and the one that catches an **under**-estimate — the
/// dangerous direction, because it means the plan believes nothing is
/// hidden while a tab is drawn off the edge with no affordance offering
/// it.
///
/// # Why this binary-searches rather than sweeps
///
/// The only widths at which an under-estimate is *visible* are those
/// between "the plan stopped overflowing" and "the content genuinely
/// fits". An estimate short by 8 pt makes that window 8 pt wide, and a
/// sweep at any step coarser than the error walks straight over it — which
/// was measured on the band, not assumed: with the item padding
/// deliberately cut to a fifth, an 11 pt sweep still reported success.
///
/// So the transition width is found exactly (the counts are monotonic in
/// width, which `the_active_tab_is_on_screen_at_every_width_whichever_tab_it_is`
/// and the pure `widening_the_strip_never_hides_a_tab_that_was_visible`
/// pin separately) and the assertion is made **there**, where any
/// shortfall at all is visible, plus at the two widths above it.
///
/// # What it reports
///
/// The transition width is printed on failure rather than hard-coded,
/// because it is a property of the synthetic face and the fixture and
/// would become a maintenance burden the moment either changed. What is
/// asserted is the *relationship* at that width, not the number.
#[test]
fn the_strip_that_claims_to_fit_really_does_fit() {
    let ctx = context();
    let shell = strip_shell();
    let none = ConditionSet::new();

    let fits = |w: i32| {
        !render_shell(&ctx, &shell, "file", &none, w as f32)
            .state
            .last_frame()
            .tab_overflow_visible
    };
    let (mut lo, mut hi) = (40_i32, 2000_i32);
    assert!(
        !fits(lo),
        "the search needs a width at which the strip does overflow"
    );
    assert!(
        fits(hi),
        "the search needs a width at which the strip does not overflow; if seven \
         tabs, a QAT and a selector do not fit in 2000 pt, the fixture or the \
         measurement has changed"
    );
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        if fits(mid) { hi = mid } else { lo = mid }
    }

    for width in [hi, hi + 1, hi + 2].map(|w| w as f32) {
        let frame = render_shell(&ctx, &shell, "file", &none, width);
        let report = frame.state.last_frame();
        assert!(
            !report.tab_overflow_visible,
            "at {width} pt — at or above the transition at {hi} pt — the strip \
             should claim to fit"
        );
        assert_eq!(
            report.tabs_in_strip,
            STRIP_TABS.len(),
            "at {width} pt no affordance was drawn, so every tab must be in the strip"
        );

        let selector = frame
            .rect(report::mode_selector())
            .expect("the selector publishes its track");
        for id in STRIP_TABS {
            let rect = frame
                .rect(&report::tab(id))
                .unwrap_or_else(|| panic!("at {width} pt tab `{id}` published no rect"));
            assert!(
                rect.right() <= width + SLACK && rect.left() >= -SLACK,
                "at {width} pt — the narrowest width at which the plan concludes all \
                 seven tabs fit — tab `{id}` was drawn at {rect:?}, outside the \
                 window. The width estimate is smaller than what a tab actually \
                 occupies, which is the direction that loses a tab with no \
                 affordance to reach it"
            );
            assert!(
                rect.right() <= selector.left() + SLACK,
                "at {width} pt tab `{id}` at {rect:?} is under the mode selector at \
                 {selector:?}, so the strip's estimate is short by at least {} pt",
                rect.right() - selector.left()
            );
        }
    }
}

/// **★ Requirement 3, rendered: a contextual tab arriving into a full
/// strip goes into the menu, and does not displace the active one.**
///
/// The same frame twice, once with `selection.any` set and once without,
/// at a width where the strip is already full. What changes must be
/// exactly one thing: the affordance's count.
///
/// The `.count()` on the affordance is what "announced" means here, and it
/// is why [`super::plan::overflow_label`] puts the number in the label
/// rather than drawing a bare chevron — see [`super::a11y`] for what
/// `egui` 0.35 cannot express beyond that.
#[test]
fn a_contextual_tab_arriving_into_a_full_strip_is_announced_by_the_count() {
    let ctx = context();
    let shell = strip_shell();
    let width = 320.0;

    let without = render_shell(&ctx, &shell, "file", &ConditionSet::new(), width);
    let with = render_shell(
        &ctx,
        &shell,
        "file",
        &ConditionSet::new().with("selection.any"),
        width,
    );

    assert!(
        without.state.last_frame().tab_overflow_visible,
        "this test needs a strip that is already full at {width} pt"
    );
    assert_eq!(
        with.state.last_frame().tabs_visible,
        without.state.last_frame().tabs_visible + 1,
        "the Format tab did not appear"
    );
    assert_eq!(
        with.state.last_frame().tabs_in_strip,
        without.state.last_frame().tabs_in_strip,
        "a contextual tab appearing pushed a tab out of the strip; it must go into \
         the menu, not displace what is already drawn"
    );
    assert_eq!(
        with.state.last_frame().tabs_overflowed,
        without.state.last_frame().tabs_overflowed + 1,
        "the contextual tab is announced by the menu's count going up, and it did not"
    );
    assert_eq!(
        with.state.active_tab(),
        Some("file"),
        "a contextual tab must never displace the active one"
    );
    assert!(
        with.rect(&report::tab("format")).is_none(),
        "the Format tab was drawn in the strip at a width where the strip is full"
    );
}

/// **★ No tab is lost between the strip and its menu, at any width.**
///
/// The counting form of failure mode #8, and the cheapest possible
/// tripwire on it: a tab that is in neither place is a tab the operator
/// cannot reach at all.
///
/// Also asserts the biconditional — the affordance is drawn exactly when
/// something is behind it. Both directions are real defects: an affordance
/// with nothing behind it opens an empty menu, and something hidden with
/// no affordance is #8 itself.
#[test]
fn no_tab_is_lost_between_the_strip_and_its_menu() {
    let ctx = context();
    let shell = strip_shell();
    for width in (40..1600).step_by(9).map(|w| w as f32) {
        let frame = render_shell(&ctx, &shell, "view", &ConditionSet::new(), width);
        let report = frame.state.last_frame();
        assert_eq!(
            report.tabs_in_strip + report.tabs_overflowed,
            report.tabs_visible,
            "at {width} pt {} tabs are visible but {} are in the strip and {} in the \
             menu — the difference is unreachable",
            report.tabs_visible,
            report.tabs_in_strip,
            report.tabs_overflowed
        );
        assert_eq!(
            report.tabs_overflowed > 0,
            report.tab_overflow_visible,
            "at {width} pt: {} hidden, affordance visible = {}",
            report.tabs_overflowed,
            report.tab_overflow_visible
        );
        assert!(
            report.tabs_in_strip >= 1 || report.tab_strip_collapsed,
            "at {width} pt the strip drew no tab at all and did not report a \
             collapse, so the active tab is simply missing"
        );
    }
}

/// **★ Requirement 4: the QAT never starts at a negative x.**
///
/// The measured symptom of the old layout, stated as the narrowest
/// possible assertion. At 180 pt the first QAT control ran from −6 to a
/// point past the tabs; it was drawn, it was reported, and it could not be
/// clicked.
///
/// The QAT has no overflow menu of its own — it is a fixed cost, and
/// `RIBBON_IA.md` treats its contents as the handful of things an operator
/// uses constantly — so the answers available when it does not fit are
/// "truncate", "drop the ones that will not fit", and "draw off the edge".
/// This asserts that the third never happens.
///
/// # Two claims, and the second is the one with teeth
///
/// 1. **Containment, at every width.** Whatever is drawn is inside the
///    window.
/// 2. **Presence, above a realistic width.** Containment alone is
///    satisfied by a QAT that draws nothing at all, so the sweep also
///    pins that the whole QAT really is there at ordinary widths. Without
///    it, a regression that dropped every control would pass claim 1
///    perfectly.
#[test]
fn the_qat_stays_inside_the_row_at_every_width() {
    let ctx = context();
    let shell = strip_shell();
    let full = shell
        .qat
        .as_ref()
        .expect("the fixture has a QAT")
        .ids()
        .len();

    for width in (40..900).step_by(5).map(|w| w as f32) {
        let frame = render_shell(&ctx, &shell, "file", &ConditionSet::new(), width);
        let controls = frame.all("ribbon.qat.");
        for rect in &controls {
            assert!(
                rect.left() >= -SLACK,
                "at {width} pt a QAT control starts at {} — the exact defect \
                 measured before `strip` existed, where the first control began at \
                 x = −6 and everything to its right was pushed off the window",
                rect.left()
            );
            assert!(
                rect.right() <= width + SLACK,
                "at {width} pt a QAT control ends at {}, past the right edge",
                rect.right()
            );
        }
        if width >= 600.0 {
            assert_eq!(
                controls.len(),
                full,
                "at {width} pt every QAT control must be drawn. Dropping one is the \
                 last resort for a row that cannot hold it (see `qat::render`), not \
                 something a comfortable window should ever reach"
            );
        }
    }
}

/// **The overflow menu is a route to a hidden tab, not a place to look at
/// one.**
///
/// Opening the menu and clicking a hidden tab must make that tab active
/// **and** put it in the strip — the second half is what the pin
/// guarantees, and without it the operator would pick a tab out of the
/// menu and watch it stay in the menu.
///
/// Driven the way an operator would: hover the affordance, press, release,
/// let the popup render, then click the entry.
#[test]
fn picking_a_hidden_tab_from_the_menu_brings_it_into_the_strip() {
    let ctx = context();
    let shell = strip_shell();
    let registry = registry();
    let width = 320.0;

    let frame = render_shell(&ctx, &shell, "file", &ConditionSet::new(), width);
    assert!(
        frame.state.last_frame().tab_overflow_visible,
        "this test needs a full strip at {width} pt"
    );
    let affordance = frame
        .rect(report::tab_overflow())
        .expect("a visible affordance publishes its rect");
    let mut state = frame.state;

    let click_at = |state: &mut RibbonState, at: egui::Pos2, rects: &mut Vec<(String, Rect)>| {
        rects.clear();
        let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 400.0))),
            events: vec![
                egui::Event::PointerMoved(at),
                egui::Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let _ = Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, state);
        });
    };

    // Open the menu, then let it lay out so its entries publish rects.
    let mut rects = Vec::new();
    click_at(&mut state, affordance.center(), &mut rects);
    rects.clear();
    {
        let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 400.0))),
            events: vec![egui::Event::PointerMoved(affordance.center())],
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let _ = Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, &mut state);
        });
    }

    // The last tab is the one a full strip is guaranteed to have hidden.
    let hidden_id = STRIP_TABS[STRIP_TABS.len() - 1];
    let entry = rects
        .iter()
        .find(|(n, _)| n == &report::tab(hidden_id))
        .map(|(_, r)| *r)
        .unwrap_or_else(|| {
            panic!(
                "the open menu published no entry for the hidden tab `{hidden_id}`; \
                 it published {:?}. A menu whose contents cannot be found is not a \
                 route to anything",
                rects.iter().map(|(n, _)| n).collect::<Vec<_>>()
            )
        });

    let mut more = Vec::new();
    click_at(&mut state, entry.center(), &mut more);
    assert_eq!(
        state.active_tab(),
        Some(hidden_id),
        "clicking a tab in the overflow menu must activate it"
    );

    // And the pin puts it in the strip on the next frame.
    let after = render_shell(&ctx, &shell, hidden_id, &ConditionSet::new(), width);
    assert!(
        after.rect(&report::tab(hidden_id)).is_some(),
        "the tab the operator just picked out of the menu is still in the menu"
    );
    assert_eq!(after.state.active_tab(), Some(hidden_id));
}
