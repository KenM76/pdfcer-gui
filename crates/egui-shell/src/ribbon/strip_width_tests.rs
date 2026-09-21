//! `MODES_AND_PANELS.md` failure mode #8, **one row up** — the tab strip.
//!
//! # The claim
//!
//! Reserving the right island protects the right island and nothing else:
//! `egui` does not clip children to `max_rect`, so every other claimant on
//! the row runs off the edge instead. Measured with the synthetic face on a
//! two-tab fixture, a row that reserves only the selector lays out like this:
//!
//! ```text
//! window  QAT             tabs                selector      verdict
//!  500    0..166          188..265            322..500      correct
//!  320    0..166          188..265            142..320      tabs UNDER selector
//!  180   -6..160          182..259              2..180      both tabs off screen
//! ```
//!
//! Every test here pins one of those rows shut, across all four claimants at
//! once: the QAT, every tab, the strip's affordance and the mode selector.
//!
//! # Why it is its own file
//!
//! [`super::width_tests`] makes the same shape of claim about the **band**, a
//! row down, and the two ladders are independent: a band that overflows
//! correctly says nothing about a strip that does, and the fixtures differ —
//! the band tests render the shared two-tab manifest, these render
//! [`strip_shell`], built to make the *strip* overflow while the band never
//! does. Splitting them is R2 with a real seam under it rather than a cut for
//! the line count.
//!
//! # The harness is [`super::width_tests`]', deliberately
//!
//! [`super::width_tests::context`], [`super::width_tests::render_shell`] and
//! [`super::width_tests::Rendered`] are imported rather than reproduced, for
//! the reason that file's header gives at length and that governs both: this
//! crate depends on `egui` with `default-features = false`, so a test that
//! installs no font measures text of **zero width**, under which the strip
//! always fits and every assertion here passes against numbers no glyph
//! produced. One synthetic face, installed one way, is what makes these
//! numbers identical under `cargo test -p egui-shell` and `cargo test
//! --workspace` alike. A second harness is a second thing to get wrong.

use egui::{Pos2, Rect, Vec2};

use crate::commands::ConditionSet;
use crate::manifest::{Group, Item, Mode, Shell, Tab};

use super::tests::registry;
use super::width_tests::{Rendered, SLACK, context, render_shell};
use super::{Ribbon, RibbonState, report};

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

/// **Failure mode #8 on the tab strip, swept: nothing on the row is
/// ever off screen, at any width.**
///
/// The single assertion the whole of [`super::strip`] exists to make true.
/// It covers all four claimants at once — the QAT, every tab, the strip's
/// affordance and the mode selector — because the failure is in none of
/// them individually: it is what happens when only *one* is reserved and the
/// rest are laid out into whatever is left, which at 180 pt is a negative
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

/// **No tab is ever drawn under the mode selector.**
///
/// The shape an unreserved row fails in, measured at 320 pt: tabs from 188
/// to 265 under a selector from 142 to 320 — a 77 pt overlap, with the tabs
/// underneath. Nothing is off screen and nothing looks wrong in the reported
/// rects; the tabs are simply unreachable, which is why a containment sweep
/// does not cover this on its own.
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

/// **The active tab is pinned: it is on screen at every width the strip
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

/// **The collapse happens only where it must, and only downwards.**
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

/// **The strip's affordance can actually be hit, at widths where it is
/// crowded.**
///
/// A rectangle proves something was allocated; only `egui`'s own hit test
/// proves it can be reached, because that is what accounts for clipping,
/// for occlusion by a later widget and for a zero-area interact rect. The
/// band's affordance has the same test for the same reason
/// (`the_affordance_is_hit_testable_under_real_metrics`); this is its
/// counterpart one row up, and the row up is where an unreserved layout
/// puts controls at negative coordinates.
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

/// **When the strip claims everything fits, everything actually fits.**
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

/// **Requirement 3, rendered: a contextual tab arriving into a full
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

/// **No tab is lost between the strip and its menu, at any width.**
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

/// **Requirement 4: the QAT never starts at a negative x.**
///
/// The narrowest possible assertion on a measured symptom: at 180 pt an
/// unreserved row puts the first QAT control from −6 to a point past the
/// tabs. It is drawn, it is reported, and it cannot be clicked.
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
