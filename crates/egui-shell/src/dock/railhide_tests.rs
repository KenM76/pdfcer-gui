//! Tests for the two things that happen at a dock's outer edge — the rail's
//! **auto-hide**, and the **tab strip it lets a stack do without**.
//!
//! Both landed on 2026-09-05, both from one message, and they are tested in one
//! file because the dangerous interaction is between them: a rail that can
//! disappear, drawn beside a stack that has given up its own switch, is exactly
//! the arrangement in which a panel becomes unreachable. This project has
//! shipped three unreachable panels with every gate green
//! (`SHELL_LAYOUT_PROPOSAL.md` §5), and the pair of features here is the
//! first arrangement since that could do it again.
//!
//! # ★★★ What each test is pointed at, and the shape it refuses
//!
//! | Test | The build it would fail against |
//! |---|---|
//! | [`a_stack_whose_panels_are_all_on_the_rail_draws_no_tab_strip`] | one that suppressed nothing — the feature not built |
//! | [`a_panel_the_rail_cannot_raise_keeps_the_strip_for_the_whole_stack`] | one that suppressed on `any` instead of `all`, which is the plausible off-by-one and leaves exactly one panel unreachable |
//! | [`a_side_too_narrow_for_the_rail_keeps_its_tab_strip_at_every_width`] | one that asked *"is a rail configured"* rather than *"was one drawn"* |
//! | [`the_panel_beside_a_hiding_rail_is_the_same_width_revealed_and_hidden`] | one that reclaimed the sliver on reveal — R128, the panel reflowing under the pointer that revealed it |
//! | [`a_hiding_rail_always_publishes_a_trigger_wide_enough_to_hit`] | one whose sliver was thinner than a pointer can find, i.e. a rail with no way back |
//!
//! # ★★ Fonts, and why these tests do not need the synthetic face
//!
//! [`super::width_tests`]' header is the authority on the trap: this crate
//! builds `egui` without `default_fonts`, so a label measures zero and any test
//! whose subject is a *width derived from text* is vacuous. Nothing here is.
//! Every number these tests read is either a constant ([`super::rail::WIDTH_PTS`],
//! [`super::rail::PEEK_WIDTH_PTS`]) or a rectangle carved from the window, and
//! the panel bodies are empty closures. A tab **strip**'s height is
//! [`super::plan::TAB_BAR_HEIGHT`], also a constant — which is what lets
//! "was a strip drawn" be asked as a geometry question rather than as a
//! text-measurement one.

use egui::{Rect, Vec2};

use super::{Column, Dock, DockLayout, DockSide, DockState, PanelId, SideLayout, Stack, rail};
use crate::peek::{AutoHide, Show};

/// A left dock holding one stack of three panels, and an empty right dock.
fn three_on_the_left() -> DockLayout {
    DockLayout::new(
        SideLayout::new([Column::new([Stack::tabbed([
            "pages",
            "bookmarks",
            "layers",
        ])])]),
        SideLayout::default(),
    )
}

/// What one rendered frame said about the rail and the tabs.
struct Rendered {
    report: super::DockFrameReport,
    rects: Vec<(String, Rect)>,
    bodies: Vec<PanelId>,
}

impl Rendered {
    fn rect(&self, name: &str) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, r)| *r)
    }

    fn any_named(&self, prefix: &str) -> bool {
        self.rects.iter().any(|(n, _)| n.starts_with(prefix))
    }
}

/// Render one frame with a left rail, a reach predicate, and a window size.
///
/// `reachable` is the set the application claims the rail can raise —
/// deliberately a parameter rather than "everything", because the `all` vs
/// `any` distinction below is the whole safety argument and a helper that
/// could only express "everything" would make it untestable.
fn frame(
    state: &mut DockState,
    window: Vec2,
    reachable: &[&str],
    left_width: Option<f32>,
) -> Rendered {
    if let Some(w) = left_width {
        state.layout_mut().left.width_pts = w;
    }
    let owned: Vec<String> = reachable.iter().map(|s| (*s).to_string()).collect();
    let mut reach = |panel: &PanelId| owned.iter().any(|id| id == panel.as_str());
    let mut strip = |ui: &mut egui::Ui| {
        // A rail that draws SOMETHING, because a handler that returned early
        // would leave `rail_drawn` true and every claim below about a drawn
        // rail vacuous.
        ui.allocate_space(Vec2::new(rail::WIDTH_PTS - 8.0, 24.0));
    };
    let mut rects: Vec<(String, Rect)> = Vec::new();
    let mut bodies: Vec<PanelId> = Vec::new();
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, window)),
        ..Default::default()
    };
    let mut report = super::DockFrameReport::default();
    let mut sink = |r: &super::report::RectReport<'_>| rects.push((r.name.to_string(), r.rect));
    let _ = ctx.run_ui(input, |ui| {
        report = Dock::new()
            .with_side_rail(DockSide::Left, &mut strip)
            .with_rail_reach(&mut reach)
            .reporting_rects_to(&mut sink)
            .show(ui, state, |panel, _ui| bodies.push(panel.clone()));
    });
    Rendered {
        report,
        rects,
        bodies,
    }
}

const ALL_THREE: [&str; 3] = ["pages", "bookmarks", "layers"];

/// ★★★ **The feature.** Three panels, all on the rail: no tab strip.
///
/// Two assertions rather than one, and the second is the one that matters. The
/// count says the dock *decided* to suppress; the absence of every
/// `dock.tab.*` region says it actually did. A build that incremented the
/// counter and drew the strip anyway would pass the first alone — and the
/// counter is the thing a later refactor is most likely to keep while moving
/// the drawing.
#[test]
fn a_stack_whose_panels_are_all_on_the_rail_draws_no_tab_strip() {
    let mut state = DockState::new(three_on_the_left());
    let r = frame(
        &mut state,
        Vec2::new(1400.0, 900.0),
        &ALL_THREE,
        Some(320.0),
    );

    assert_eq!(r.report.tab_strips_suppressed, 1);
    assert!(
        !r.any_named("dock.tab."),
        "a tab was drawn anyway: {:?}",
        r.rects
            .iter()
            .map(|(n, _)| n.as_str())
            .filter(|n| n.starts_with("dock.tab."))
            .collect::<Vec<_>>()
    );
    // ★ And the panel is still there. A "suppression" that also stopped
    // drawing the body would satisfy both assertions above.
    assert_eq!(
        r.bodies.len(),
        1,
        "the active panel's body is still drawn: {:?}",
        r.bodies
    );
    assert!(r.rect("dock.left.toolrail").is_some(), "the rail was drawn");
}

/// ★★★ **`all`, not `any`** — one panel the rail cannot raise keeps the strip
/// for the whole stack.
///
/// The plausible wrong implementation suppresses when the rail covers the
/// **active** panel, or when it covers *most* of them. Either leaves the
/// uncovered panel with no switch of any kind: it is not on the rail, and the
/// tab that was its only other route has just been taken away. That panel is
/// then reachable by nothing at all, which is the defect
/// `SHELL_LAYOUT_PROPOSAL.md` §5 made a precondition of this whole surface.
///
/// The fixture makes the uncovered panel the **active** one deliberately, so a
/// build that checked only the active tab would also be caught.
#[test]
fn a_panel_the_rail_cannot_raise_keeps_the_strip_for_the_whole_stack() {
    let mut state = DockState::new(three_on_the_left());
    let r = frame(
        &mut state,
        Vec2::new(1400.0, 900.0),
        &["bookmarks", "layers"],
        Some(320.0),
    );
    assert_eq!(
        r.report.tab_strips_suppressed, 0,
        "`pages` is not on the rail, so the strip is the only way to it"
    );
    assert!(r.any_named("dock.tab."), "and the strip was actually drawn");
}

/// ★★★ **Walked across the width series, never at two endpoints.**
///
/// [`rail::resolve_width`] returns zero when reserving 52 pt would leave the
/// panel body under [`super::plan::MIN_COLUMN_WIDTH`] — *absent rather than
/// squeezed*. So on a narrow side there is no rail, and a tab strip suppressed
/// there would leave the stack with no switch at all.
///
/// The assertion is therefore an **implication**, checked at every width in a
/// fine series: *suppressed ⇒ a rail was drawn*. A build that asked "is a rail
/// configured for this side" instead of "was one drawn" passes at 320 pt and
/// fails somewhere below it, which is precisely why a two-endpoint test would
/// have been worthless — the interesting widths are in the middle.
#[test]
fn a_side_too_narrow_for_the_rail_keeps_its_tab_strip_at_every_width() {
    let mut narrow_seen = false;
    let mut wide_seen = false;
    for tenths in 400..=3600u32 {
        if tenths % 40 != 0 {
            continue;
        }
        let width = f32::from(u16::try_from(tenths).expect("small")) / 10.0;
        let mut state = DockState::new(three_on_the_left());
        let r = frame(
            &mut state,
            Vec2::new(1600.0, 900.0),
            &ALL_THREE,
            Some(width),
        );
        let drew_rail = r.rect("dock.left.toolrail").is_some();
        if r.report.tab_strips_suppressed > 0 {
            assert!(
                drew_rail,
                "at a {width} pt side the tab strip was suppressed and NO rail was \
                 drawn — the stack has no switch of any kind and two of its three \
                 panels are unreachable"
            );
            wide_seen = true;
        } else if !drew_rail {
            assert!(
                r.any_named("dock.tab."),
                "at a {width} pt side there is neither a rail nor a tab strip"
            );
            narrow_seen = true;
        }
    }
    // ★ The series must actually straddle the threshold, or the implication
    // above is satisfied vacuously by a run in which the rail was never absent.
    assert!(
        narrow_seen,
        "no width in the series was too narrow for the rail, so the case this \
         test exists for was never reached"
    );
    assert!(wide_seen, "no width in the series suppressed anything");
}

/// ★★★ **THE NO-REFLOW GUARANTEE.** The panel beside a hiding rail is the same
/// width whether the rail is showing or not.
///
/// This is R128 for the rail, and it is the property that makes auto-hide
/// usable rather than nauseating: the operator's pointer is travelling towards
/// a control in the panel, and the panel must not move as the pointer passes
/// the rail's edge. It holds because [`rail::PEEK_WIDTH_PTS`] is reserved from
/// the SETTING, before the reveal is resolved, and the revealed strip is
/// painted into an `Area` that allocates nothing.
///
/// Read as body rectangles rather than as a claim about the code, at several
/// side widths, because the arithmetic is per-side and a build that reclaimed
/// the sliver would differ by exactly ten points — a difference invisible in a
/// screenshot and obvious in a number.
#[test]
fn the_panel_beside_a_hiding_rail_is_the_same_width_revealed_and_hidden() {
    for side_width in [260.0_f32, 300.0, 320.0, 420.0, 520.0] {
        let mut hidden = DockState::new(three_on_the_left());
        hidden.set_rail_auto_hide(AutoHide::OnHover);
        let a = frame(
            &mut hidden,
            Vec2::new(1600.0, 900.0),
            &ALL_THREE,
            Some(side_width),
        );
        assert_eq!(
            a.report.rail_show,
            Show::Hidden,
            "with no pointer in the frame the rail must be at rest"
        );
        let hidden_body = a
            .rect("dock.body.pages")
            .expect("the active panel's body region");

        // The same state, driven to the revealed side by a pointer sitting on
        // the sliver. Planted rather than waited for: `Peek` starts hidden, so
        // a test that only ever rendered the default would assert about one of
        // the two states this test is comparing.
        let mut shown = DockState::new(three_on_the_left());
        shown.set_rail_auto_hide(AutoHide::OnHover);
        let b = reveal(&mut shown, side_width);
        assert_eq!(
            b.report.rail_show,
            Show::Overlay,
            "the plant did not land: the pointer is on the sliver and the rail \
             is still hidden, so the comparison below is between two identical \
             frames"
        );
        let shown_body = b
            .rect("dock.body.pages")
            .expect("the active panel's body region");

        assert!(
            (hidden_body.width() - shown_body.width()).abs() < 0.01,
            "at a {side_width} pt side the panel is {} pt wide with the rail \
             hidden and {} pt with it revealed — it reflowed under the pointer \
             that revealed it",
            hidden_body.width(),
            shown_body.width()
        );
        assert!(
            (hidden_body.left() - shown_body.left()).abs() < 0.01,
            "at a {side_width} pt side the panel moved sideways"
        );
    }
}

/// Render two frames with the pointer parked on the rail's sliver, and return
/// the second.
///
/// Two, because [`crate::peek::Peek`] answers from the pointer position **and**
/// last frame's state: the first frame is the one that reveals, and the second
/// is the one that draws the revealed strip and reports the geometry.
fn reveal(state: &mut DockState, side_width: f32) -> Rendered {
    let mut last = frame_with_pointer(state, side_width);
    last = {
        let _ = last;
        frame_with_pointer(state, side_width)
    };
    last
}

fn frame_with_pointer(state: &mut DockState, side_width: f32) -> Rendered {
    state.layout_mut().left.width_pts = side_width;
    let owned: Vec<String> = ALL_THREE.iter().map(|s| (*s).to_string()).collect();
    let mut reach = |panel: &PanelId| owned.iter().any(|id| id == panel.as_str());
    let mut strip = |ui: &mut egui::Ui| {
        ui.allocate_space(Vec2::new(rail::WIDTH_PTS - 8.0, 24.0));
    };
    let mut rects: Vec<(String, Rect)> = Vec::new();
    let mut bodies: Vec<PanelId> = Vec::new();
    let ctx = egui::Context::default();
    // On the sliver: the left edge of the window, half way down.
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(1600.0, 900.0),
        )),
        events: vec![egui::Event::PointerMoved(egui::pos2(
            rail::PEEK_WIDTH_PTS / 2.0,
            450.0,
        ))],
        ..Default::default()
    };
    let mut report = super::DockFrameReport::default();
    let mut sink = |r: &super::report::RectReport<'_>| rects.push((r.name.to_string(), r.rect));
    let _ = ctx.run_ui(input, |ui| {
        report = Dock::new()
            .with_side_rail(DockSide::Left, &mut strip)
            .with_rail_reach(&mut reach)
            .reporting_rects_to(&mut sink)
            .show(ui, state, |panel, _ui| bodies.push(panel.clone()));
    });
    Rendered {
        report,
        rects,
        bodies,
    }
}

/// ★★★ **There is always a way back, and it is big enough to hit.**
///
/// The trigger region is published on every frame the side is drawn — hidden or
/// not — and is never thinner than [`crate::peek::Peek::MIN_TRIGGER_PTS`]. That
/// is the entire reason it is safe to suppress a tab strip beside a rail that
/// can hide: the rail is never *gone*, only narrow.
///
/// ⚠ Asserted against the **width of the published rectangle**, not against the
/// constant. A build that reserved ten points and then published a rectangle
/// clipped to nothing would satisfy a constants-only test and would strand
/// every panel on the side.
#[test]
fn a_hiding_rail_always_publishes_a_trigger_wide_enough_to_hit() {
    for side_width in [260.0_f32, 300.0, 320.0, 420.0, 520.0, 700.0] {
        let mut state = DockState::new(three_on_the_left());
        state.set_rail_auto_hide(AutoHide::OnHover);
        let r = frame(
            &mut state,
            Vec2::new(1600.0, 900.0),
            &ALL_THREE,
            Some(side_width),
        );
        let trigger = r.rect("dock.left.railtrigger").unwrap_or_else(|| {
            panic!(
                "at a {side_width} pt side a hiding rail published NO trigger, so \
                 there is no rectangle for the operator to find it with"
            )
        });
        assert!(
            trigger.width() >= crate::peek::Peek::MIN_TRIGGER_PTS,
            "at a {side_width} pt side the way back to the rail is {} pt wide, \
             under the {} pt floor",
            trigger.width(),
            crate::peek::Peek::MIN_TRIGGER_PTS
        );
        assert!(
            trigger.height() > 100.0,
            "the trigger is the whole height of the side; {} pt is not",
            trigger.height()
        );
    }
}

/// A rail that is not hiding reports [`Show::Inline`] and reserves its full
/// width — the setting's off position, asserted so that a build which hid
/// unconditionally would be caught by something other than a screenshot.
#[test]
fn a_rail_that_is_not_hiding_takes_its_whole_width() {
    let mut state = DockState::new(three_on_the_left());
    let r = frame(
        &mut state,
        Vec2::new(1600.0, 900.0),
        &ALL_THREE,
        Some(320.0),
    );
    assert_eq!(r.report.rail_show, Show::Inline);
    let trigger = r
        .rect("dock.left.railtrigger")
        .expect("published either way");
    assert!(
        (trigger.width() - rail::WIDTH_PTS).abs() < 0.01,
        "an inline rail's trigger IS the rail: {} pt against {} pt",
        trigger.width(),
        rail::WIDTH_PTS
    );
}
