//! Layout tests that run the dock against **real text metrics**.
//!
//! # ★ Why this file exists, and why the tests next door are not enough
//!
//! `egui-shell` depends on `egui` with `default-features = false`, so a
//! test process building this crate alone has **no font data** and every
//! galley measures ≈ 0. Under those conditions every width-sensitive path
//! in this module is trivially satisfied:
//!
//! - every tab is exactly [`plan::MIN_TAB_WIDTH`] wide, so a bar
//!   overflows only when a test forces it;
//! - the overflow reservation is exactly [`plan::MIN_TAB_WIDTH`] too,
//!   so a reservation sized for the wrong label is indistinguishable
//!   from a correct one;
//! - `"⏷ 8 more"` and `"⏷ 9 more"` are the same width, so the
//!   non-monotonicity [`plan::overflow_width`] exists to handle never
//!   occurs;
//! - and no row can overflow its parent, so the `max_rect` inflation
//!   trap below never fires.
//!
//! Worse, the situation is not merely *absent* — it is **decided by
//! whichever sibling crate is in the build**:
//!
//! ```text
//! cargo test -p egui-shell --lib   → egui alone         → no fonts
//! cargo test --workspace           → pdfcer-gui → eframe → egui/default_fonts
//! ```
//!
//! `D:/dev/rag/rust/a_crate_tested_alone_and_in_a_workspace_gets_different_features_so_layout_tests_can_be_vacuous.md`
//! records this crate's own ribbon suite passing 116 tests over a width
//! layer that had never been shown a non-zero width — and two real
//! defects living in the gap, one of them an arithmetic underflow
//! reachable from day one. *"The suite was not weak. It was vacuous."*
//!
//! So every test here installs the crate's synthetic **proportional**
//! face and asserts it took effect before asserting anything else. The
//! numbers below are then identical under both commands and cannot be
//! changed by any feature any sibling crate turns on.
//!
//! # The trap these tests are pointed at
//!
//! `D:/dev/rag/egui/a_sibling_row_that_overflows_grows_the_parent_max_rect_so_available_width_is_not_the_window.md`
//! — a child that lays out past its parent's `max_rect` **grows it**, so
//! a later query for available space returns a rectangle extending past
//! the window edge. Measured there, in the ribbon, hours before this
//! module was written: a reservation taken from that rectangle landed
//! **78 pt off screen**, correct arithmetic applied to a width the window
//! never had. The entry's closing advice is what shapes the tests below:
//!
//! > A sweep across widths is **too coarse**. Binary-search the exact
//! > width at which the layout's own estimate flips from "fits" to "does
//! > not fit", and assert there — measured, an 11 pt sweep step walked
//! > straight over an 8 pt estimation error.
//!
//! # What is asserted, and what deliberately is not
//!
//! These are **geometric** assertions: what was drawn, where, and whether
//! it lies inside the thing that is supposed to contain it. They are not
//! pixel or legibility assertions — that is `tools/ui-verify`'s job
//! against a real window, and it is why the rects are published in the
//! first place. `MODES_AND_PANELS.md` is explicit that
//! *"layout/clipping defects have exactly one oracle: a rendered
//! screenshot"*, and nothing here claims otherwise.

use egui::{Pos2, Rect, Vec2};

use super::model::{Column, DockLayout, PanelId, PanelInfo, PanelRegistry, SideLayout, Stack};
use super::report::RectReport;
use super::{Dock, DockState, plan, testfont};

/// Tolerance, in points, for "inside" and "does not overlap".
///
/// `egui` rounds widget rectangles to whole physical pixels, so an edge
/// can land a fraction of a point beyond an exact arithmetic boundary
/// without anything being wrong. One point is well below anything a
/// person could see and well above the rounding.
const SLACK: f32 = 1.0;

/// A registry of `n` panels with realistic, differently-shaped labels.
///
/// Deliberately **not** `"Panel 0" … "Panel 8"`: labels of equal length
/// make every tab the same width, which is the one property real
/// proportional text does not have and the one the arithmetic must not
/// assume.
fn registry(n: usize) -> PanelRegistry {
    const LABELS: [&str; 9] = [
        "Pages",
        "Layers",
        "Bookmarks",
        "Attachments",
        "Signatures",
        "Fonts",
        "Comments",
        "Objects",
        "Ill",
    ];
    let mut r = PanelRegistry::new();
    for i in 0..n {
        let label = LABELS[i % LABELS.len()];
        r.register(
            PanelInfo::new(format!("p{i}"), label)
                .with_tooltip(format!("{label} — what to reach for when you need it")),
        );
    }
    r
}

/// One stack of `n` tabs in a left dock of the given width.
fn layout(n: usize, dock_width: f32) -> DockLayout {
    DockLayout::new(
        SideLayout::new([Column::new([Stack::tabbed(
            (0..n).map(|i| format!("p{i}")).collect::<Vec<_>>(),
        )])])
        .with_width(dock_width),
        SideLayout::none(),
    )
}

/// One rendered frame: every rect the dock published.
struct Rendered {
    rects: Vec<(String, Rect)>,
    window: Rect,
}

impl Rendered {
    fn suffixed(&self, suffix: &str) -> Option<Rect> {
        self.rects
            .iter()
            .find(|(n, _)| n.ends_with(suffix))
            .map(|(_, r)| *r)
    }

    fn tabs(&self) -> Vec<(&str, Rect)> {
        self.rects
            .iter()
            .filter(|(n, _)| n.starts_with("dock.tab."))
            .map(|(n, r)| (n.as_str(), *r))
            .collect()
    }
}

/// Render one frame with a **real proportional font installed**, at the
/// given dock width and window size.
fn render(n: usize, dock_width: f32, window: Vec2) -> Rendered {
    let ctx = egui::Context::default();
    // ★ Asserts internally that the face took effect. Without it every
    // width comparison below is satisfied by text that occupies no space.
    testfont::install(&ctx);

    let registry = registry(n);
    let mut state = DockState::new(layout(n, dock_width));
    let mut rects: Vec<(String, Rect)> = Vec::new();
    let window_rect = Rect::from_min_size(Pos2::ZERO, window);
    {
        let mut sink = |r: &RectReport<'_>| rects.push((r.name.to_owned(), r.rect));
        let input = egui::RawInput {
            screen_rect: Some(window_rect),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            Dock::new()
                .with_registry(&registry)
                .reporting_rects_to(&mut sink)
                .show(ui, &mut state, |_panel, ui| {
                    ui.label("body");
                });
        });
    }
    Rendered {
        rects,
        window: window_rect,
    }
}

/// Measure a label the way the dock will, with the synthetic face
/// installed. Used to compute the arithmetic flip point the geometric
/// tests then aim at.
fn measured_tab_widths(ctx: &egui::Context, n: usize) -> Vec<f32> {
    let registry = registry(n);
    (0..n)
        .map(|i| {
            let label = registry
                .get(&format!("p{i}"))
                .expect("registered")
                .label
                .clone();
            plan::tab_width(text_width(ctx, &label))
        })
        .collect()
}

/// Measure exactly as [`super::tabs`] does.
///
/// **The text style matters, and getting it wrong is silent.** The tab
/// bar resolves [`egui::TextStyle::Button`] against the live style; a
/// test that measured at a hard-coded `proportional(14.0)` would compute
/// a *different* flip point from the one the renderer uses, and the
/// binary-searched assertion below would then be aimed a few points away
/// from the boundary it exists to probe — passing, while testing an
/// ordinary width. Found exactly that way: the first draft searched at
/// 14 pt, the renderer draws at the style's button size, and the test
/// reported "nothing overflowed" at a width where nothing was supposed
/// to.
fn text_width(ctx: &egui::Context, text: &str) -> f32 {
    let font_id = egui::TextStyle::Button.resolve(&ctx.style_of(egui::Theme::Light));
    ctx.fonts_mut(|f| {
        f.layout_no_wrap(text.to_owned(), font_id, egui::Color32::PLACEHOLDER)
            .size()
            .x
    })
}

// ---------------------------------------------------------------------
// The font itself
// ---------------------------------------------------------------------

/// **★ The harness is not vacuous.**
///
/// Everything below is worthless if this is not true, and "worthless"
/// here means "passing" — which is why it is asserted rather than
/// assumed.
#[test]
fn the_dock_measures_real_proportional_text() {
    let ctx = egui::Context::default();
    testfont::install(&ctx);

    let narrow = plan::tab_width(text_width(&ctx, "Ill"));
    let wide = plan::tab_width(text_width(&ctx, "Attachments"));
    assert!(
        wide > narrow,
        "tabs are all the same width — the face is not being measured"
    );
    assert!(
        narrow > 0.0 && wide < plan::MAX_TAB_WIDTH + SLACK,
        "{narrow} .. {wide} are not believable tab widths"
    );
}

/// ★ **The reservation is sized for the widest label the control can ever
/// show, and with real metrics that is not the longest one.**
///
/// `"⏷ 8 more"` is wider than `"⏷ 9 more"` in any face whose digits are
/// not tabular. With no font installed the two measure the same and this
/// test is vacuous, which is exactly why it lives in this file.
#[test]
fn the_reservation_covers_the_widest_label_with_real_metrics() {
    let ctx = egui::Context::default();
    testfont::install(&ctx);
    let measure = |s: &str| text_width(&ctx, s);

    let reserved = plan::overflow_width(12, plan::TAB_PADDING, measure);
    for hidden in 1..=12 {
        let label = plan::overflow_label(hidden);
        let needed = measure(&label) + plan::TAB_PADDING;
        assert!(
            needed <= reserved + 0.01,
            "showing {label:?} needs {needed} pt but only {reserved} pt was reserved"
        );
    }
}

// ---------------------------------------------------------------------
// Failure mode #8, against a rendered frame, at the exact flip point
// ---------------------------------------------------------------------

/// ★ **Binary-searched to the exact width at which a tab first has to
/// hide — and the affordance is inside the bar there.**
///
/// A sweep is too coarse: the RAG entry this test is written from records
/// an 11 pt sweep step walking straight over an 8 pt estimation error.
/// The flip point is where the arithmetic and the drawing are most likely
/// to disagree, because it is the only width at which a one-point error
/// changes the answer.
#[test]
fn the_affordance_is_inside_the_bar_at_the_exact_width_where_tabs_start_hiding() {
    let ctx = egui::Context::default();
    testfont::install(&ctx);
    let n = 6;
    let widths = measured_tab_widths(&ctx, n);
    let overflow_w = plan::overflow_width(n, plan::TAB_PADDING, |s| text_width(&ctx, s));

    // The dock width whose tab bar is exactly wide enough for everything.
    // The bar is the dock width less the side splitter.
    let fits = |dock_width: f32| {
        let bar = dock_width - plan::SPLITTER_THICKNESS;
        !plan::plan_tabs(&widths, 0, bar, plan::TAB_GAP, overflow_w).has_overflow()
    };

    let (mut too_narrow, mut wide_enough) = (plan::MIN_SIDE_WIDTH, 1200.0_f32);
    assert!(
        !fits(too_narrow) && fits(wide_enough),
        "the search is bracketed"
    );
    while wide_enough - too_narrow > 0.05 {
        let mid = (too_narrow + wide_enough) / 2.0;
        if fits(mid) {
            wide_enough = mid;
        } else {
            too_narrow = mid;
        }
    }

    // One tenth of a point narrower than the flip: something must hide,
    // and the affordance must be inside the bar it belongs to.
    let rendered = render(n, too_narrow - 0.5, Vec2::new(1600.0, 900.0));
    let bar = rendered
        .suffixed(".tabbar")
        .expect("a tab bar was published");
    let affordance = rendered
        .suffixed(".overflow")
        .unwrap_or_else(|| panic!("at {too_narrow} pt — just below the flip — nothing overflowed"));
    assert!(
        affordance.right() <= bar.right() + SLACK && affordance.left() >= bar.left() - SLACK,
        "the affordance {affordance:?} is not inside the bar {bar:?}"
    );
    for (name, rect) in rendered.tabs() {
        assert!(
            rect.right() <= affordance.left() + SLACK,
            "tab {name} at {rect:?} runs into the affordance at {affordance:?}"
        );
    }

    // And one step wider: everything fits, so nothing is reserved.
    let rendered = render(n, wide_enough + 1.0, Vec2::new(1600.0, 900.0));
    assert!(
        rendered.suffixed(".overflow").is_none(),
        "an affordance was drawn at a width where every tab fits"
    );
    assert_eq!(rendered.tabs().len(), n, "every tab should be drawn");
}

/// ★ **The affordance is on screen at every dock width, including ones
/// narrower than the affordance itself.**
///
/// This is the `max_rect`-inflation trap stated as an assertion. A
/// control positioned by subtraction (`right − width`) lands at a
/// negative x the moment the bar is narrower than its reservation; it is
/// still laid out, still allocated, still reported with a plausible
/// `Rect` — and painted where nobody can see or click it. Nothing errors
/// and nothing warns. Only a comparison against the **window** catches
/// it.
#[test]
fn the_affordance_is_always_within_the_window() {
    for dock_width in [plan::MIN_SIDE_WIDTH, 170.0, 200.0, 260.0, 340.0, 520.0] {
        for window in [Vec2::new(1280.0, 800.0), Vec2::new(640.0, 480.0)] {
            let rendered = render(9, dock_width, window);
            let Some(affordance) = rendered.suffixed(".overflow") else {
                continue;
            };
            assert!(
                affordance.left() >= rendered.window.left() - SLACK
                    && affordance.right() <= rendered.window.right() + SLACK,
                "at dock {dock_width} pt in a {window:?} window the affordance was drawn \
                 at {affordance:?}, outside the window {:?}",
                rendered.window
            );
            assert!(
                affordance.width() > 1.0,
                "the affordance collapsed to {affordance:?} and cannot be clicked"
            );
        }
    }
}

/// ★ **Every panel stays reachable at every width**: a tab that is not
/// drawn is in the menu, and the menu's affordance is drawn.
///
/// This is the property the previous implementation's two-pane cap was a
/// proxy for, asserted directly across the whole width range instead.
#[test]
fn every_panel_is_reachable_at_every_dock_width() {
    let n = 9;
    for dock_width in (160..900).step_by(13).map(|w| w as f32) {
        let rendered = render(n, dock_width, Vec2::new(2000.0, 900.0));
        let drawn = rendered.tabs().len();
        if drawn == n {
            continue;
        }
        assert!(
            rendered.suffixed(".overflow").is_some(),
            "at {dock_width} pt only {drawn} of {n} tabs were drawn and there is no \
             affordance — {} panels are unreachable",
            n - drawn
        );
    }
}

/// The active tab is drawn at every width at which any tab is drawn —
/// there is never a panel body on screen with no tab naming it.
#[test]
fn the_active_tab_is_drawn_whenever_any_tab_is() {
    let n = 9;
    for active in [0_usize, 4, 8] {
        for dock_width in (170..700).step_by(29).map(|w| w as f32) {
            let ctx = egui::Context::default();
            testfont::install(&ctx);
            let registry = registry(n);
            let mut l = layout(n, dock_width);
            l.activate(&PanelId::new(format!("p{active}")));
            let mut state = DockState::new(l);
            let mut rects: Vec<(String, Rect)> = Vec::new();
            {
                let mut sink = |r: &RectReport<'_>| rects.push((r.name.to_owned(), r.rect));
                let input = egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(2000.0, 900.0))),
                    ..Default::default()
                };
                let _ = ctx.run_ui(input, |ui| {
                    Dock::new()
                        .with_registry(&registry)
                        .reporting_rects_to(&mut sink)
                        .show(ui, &mut state, |_p, _ui| {});
                });
            }
            let any_tab = rects.iter().any(|(n, _)| n.starts_with("dock.tab."));
            if !any_tab {
                continue;
            }
            assert!(
                rects
                    .iter()
                    .any(|(n, _)| n == &format!("dock.tab.p{active}")),
                "at {dock_width} pt with p{active} active, tabs were drawn but not that one"
            );
        }
    }
}

// ---------------------------------------------------------------------
// Failure mode #3, with real metrics
// ---------------------------------------------------------------------

/// ★ **A very wide hidden tab does not hold the dock open.**
///
/// The field report: *"an inactive tab you cannot see holds the whole
/// dock open; you must close it to narrow the dock."* Here one panel is
/// given a preposterous label and left **inactive**, and the dock is
/// still drawn at exactly the width the layout asked for. With no font
/// installed this test cannot fail, because the preposterous label
/// measures the same as every other one.
#[test]
fn an_inactive_tab_with_a_huge_label_does_not_widen_the_dock() {
    let ctx = egui::Context::default();
    testfont::install(&ctx);

    let mut registry = PanelRegistry::new();
    registry.register(PanelInfo::new("pages", "Pages"));
    registry.register(PanelInfo::new(
        "huge",
        "Digital signature validation and long-term archival report",
    ));

    let dock_width = 220.0_f32;
    let mut state = DockState::new(DockLayout::new(
        SideLayout::new([Column::new([Stack::tabbed(["pages", "huge"])])]).with_width(dock_width),
        SideLayout::none(),
    ));

    let mut body_rect = Rect::NOTHING;
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
        ..Default::default()
    };
    let _ = ctx.run_ui(input, |ui| {
        Dock::new()
            .with_registry(&registry)
            .show(ui, &mut state, |panel, ui| {
                assert_eq!(panel.as_str(), "pages", "the inactive tab must not draw");
                body_rect = ui.max_rect();
            });
    });

    assert!(
        body_rect.width() <= dock_width + SLACK,
        "the hidden tab widened the dock: body {body_rect:?} in a {dock_width} pt dock"
    );
    assert_eq!(
        state.layout().left.width_pts,
        dock_width,
        "and it must not have changed the stored width either"
    );
}

// ---------------------------------------------------------------------
// Failure mode #4, with real metrics
// ---------------------------------------------------------------------

/// ★ **Two full docks in a 1280-point window leave the application the
/// majority of it** — the width the design rule names, with real text.
#[test]
fn two_docks_in_a_1280_point_window_leave_the_application_most_of_it() {
    let ctx = egui::Context::default();
    testfont::install(&ctx);

    let mut registry = PanelRegistry::new();
    for id in ["pages", "objects"] {
        registry.register(PanelInfo::new(id, "Attachments"));
    }
    let mut state = DockState::new(DockLayout::new(
        SideLayout::single("pages").with_width(plan::MIN_SIDE_WIDTH),
        SideLayout::single("objects").with_width(plan::MIN_SIDE_WIDTH),
    ));

    let mut central = Rect::NOTHING;
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
        ..Default::default()
    };
    let _ = ctx.run_ui(input, |ui| {
        Dock::new()
            .with_registry(&registry)
            .show(ui, &mut state, |_p, _ui| {});
        central = ui.available_rect_before_wrap();
    });

    assert!(
        central.width() >= 1280.0 * 0.6,
        "two minimum docks left the application only {} pt of 1280",
        central.width()
    );
}
