//! Rendered-ribbon tests: a real [`Ribbon`] drawn into a real
//! `egui::Context`, asserted on what came back.
//!
//! # Why these are in their own file
//!
//! Two reasons, and the second is the interesting one.
//!
//! 1. R2 caps a source file at 1,500 lines
//!    (`tools/gates/check-file-size.sh`), and `mod.rs` reached 1,455 with
//!    two thirds of it being this. A gate that fires on the next feature
//!    is a gate that gets worked around.
//! 2. **These tests and `super::width_tests` are not the same kind of
//!    test, and filing them together made that easy to miss.** Everything
//!    here asserts *structure* — a caption exists, a token came back, a
//!    contextual tab retired cleanly — and is correct whatever text
//!    measures. Everything in `width_tests` asserts *geometry* and is
//!    meaningless unless text has a width. Keeping them apart is what
//!    stops a width assertion being written here, where it would be
//!    vacuous under one of the two commands below.
//!
//! # ★ A WARNING ABOUT WIDTHS IN THIS FILE
//!
//! `egui = { default-features = false }` means this crate has no font data
//! *when it is built alone*, so every galley measures near zero. That is
//! precisely why [`super::plan::MIN_ITEM_WIDTH`] exists: the arithmetic
//! stays meaningful with no fonts, and so do these tests.
//!
//! But it also means the tests below measure DIFFERENT TEXT depending on
//! how they are invoked. Cargo unifies features across a workspace build,
//! and `pdfcer-gui` depends on `eframe`, which turns on
//! `egui/default_fonts`:
//!
//! ```text
//! cargo test -p egui-shell --lib   → no fonts → widths ≈ 0
//! cargo test --workspace           → fonts    → real widths
//! ```
//!
//! Three defects have now lived in that gap and were invisible to the
//! first command. A width assertion added here is therefore NOT a width
//! assertion — it is a width assertion under whichever font set the caller
//! happened to bring. Put it in [`super::width_tests`] instead, which
//! installs a synthetic proportional face this crate builds itself and so
//! measures the same text under both commands.
//!
//! The fixtures [`shell`] and [`registry`] are `pub(super)` because
//! `width_tests` renders the same manifest; one manifest, asserted two
//! ways, is what makes the two files comparable.

use super::*;
use crate::commands::{Command, CommandRegistry, ConditionSet, HandlerToken};
use crate::manifest::{Group, Item, Mode, Shell, Tab};
use egui::{Pos2, Rect, Vec2};

// -----------------------------------------------------------------
// Headless harness
//
// Every test here renders a REAL ribbon into a REAL `egui::Context`
// and asserts on what came back. No widget is stubbed, because the
// defects this module exists to prevent — a caption that is not
// drawn, an overflow control that cannot be reached — are properties
// of what was drawn and are invisible to a test that inspects a plan.
//
// ★ A WARNING ABOUT WIDTHS IN THIS MODULE.
//
// `default-features = false` means this crate has no font data *when
// it is built alone*, so every galley measures near zero. That is
// precisely why `plan`'s `MIN_ITEM_WIDTH` floor exists: the arithmetic
// stays meaningful with no fonts, and so do these tests.
//
// But it also means the tests below measure DIFFERENT TEXT depending
// on how they are invoked. Cargo unifies features across a workspace
// build, and `pdfcer-gui` depends on `eframe`, which turns on
// `egui/default_fonts`:
//
//     cargo test -p egui-shell --lib   → no fonts → widths ≈ 0
//     cargo test --workspace           → fonts    → real widths
//
// Two defects lived in that gap and were invisible to the first
// command. A width assertion added here is therefore NOT a width
// assertion — it is a width assertion under whichever font set the
// caller happened to bring. Put it in `super::width_tests` instead,
// which installs a synthetic proportional face this crate builds
// itself and so measures the same text under both commands.
// -----------------------------------------------------------------

/// One frame, at a given viewport width.
fn frame(ctx: &egui::Context, width: f32, mut body: impl FnMut(&mut egui::Ui)) {
    frame_with_input(ctx, width, egui::RawInput::default(), &mut body);
}

/// `egui` 0.35 replaced `Context::run` with `Context::run_ui`, which
/// hands the callback a root `Ui` covering the whole content rect —
/// so no panel container is needed, and the ribbon is laid out
/// against a width this harness controls exactly.
fn frame_with_input(
    ctx: &egui::Context,
    width: f32,
    mut input: egui::RawInput,
    body: &mut dyn FnMut(&mut egui::Ui),
) {
    input.screen_rect = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 400.0)));
    let _ = ctx.run_ui(input, |ui| body(ui));
}

pub(super) fn registry() -> CommandRegistry {
    let mut r = CommandRegistry::new();
    r.register_all([
        Command::new("file.open", "Open…", HandlerToken::new(10))
            .with_icon("open")
            .with_tooltip("Open a document"),
        Command::new("file.save_copy", "Save a copy…", HandlerToken::new(11)),
        Command::new("view.single", "Single page", HandlerToken::new(20)),
        Command::new("view.continuous", "Continuous", HandlerToken::new(21)),
        Command::new("view.facing", "Facing", HandlerToken::new(22)),
        Command::new("view.fullscreen", "Full screen", HandlerToken::new(23)),
        Command::new("view.thin_lines", "Thin lines", HandlerToken::new(24)),
        Command::new("view.reset_layout", "Reset layout", HandlerToken::new(25)),
        Command::new("format.colour", "Colour", HandlerToken::new(30)),
    ])
    .expect("distinct ids");
    r
}

/// A manifest with three modes, two ordinary tabs, one contextual
/// tab, and — deliberately — **one group with no caption at all**.
pub(super) fn shell() -> Shell {
    Shell::new()
        .with_mode(Mode::new("read", "Read", ["file", "view"]))
        .with_mode(Mode::new("review", "Review", ["file", "view"]))
        .with_mode(Mode::new("edit", "Edit", ["file", "view"]))
        .with_tab(
            Tab::new("file", "File").with_groups([Group::new("file", "File").with_items([
                Item::command("file.open"),
                Item::Separator,
                Item::command("file.save_copy"),
            ])]),
        )
        .with_tab(
            Tab::new("view", "View")
                .with_question("What is on my screen?")
                .with_groups([
                    Group::new("page_display", "Page display").with_items([
                        Item::command("view.single"),
                        Item::command("view.continuous"),
                        Item::command("view.facing"),
                    ]),
                    Group::new("render", "Render").with_items([
                        Item::command("view.thin_lines"),
                        Item::custom("quality_slider"),
                    ]),
                    // ★ No caption. `validate` would refuse this
                    // manifest; the renderer must survive it and must
                    // still emit a caption, because a band of
                    // unlabelled controls is the defect this whole
                    // module exists to prevent.
                    Group::patch("window").with_items([
                        Item::command("view.fullscreen"),
                        Item::command("view.reset_layout"),
                    ]),
                ]),
        )
        .with_contextual_tab(
            Tab::new("format", "Format")
                .with_visible_when("selection.any")
                .with_groups([
                    Group::new("style", "Style").with_items([Item::command("format.colour")])
                ]),
        )
        .with_qat(["file.open", "file.save_copy"])
}

/// Render one frame of the View tab at `width`, collecting every
/// reported rect.
fn render_view_tab(width: f32) -> (RibbonState, Vec<(String, Rect)>, Vec<HandlerToken>) {
    let ctx = egui::Context::default();
    let shell = shell();
    let registry = registry();
    let mut state = RibbonState::new();
    state.set_active_tab("view");
    let mut seen: Vec<(String, Rect)> = Vec::new();
    let mut invoked = Vec::new();

    // Two frames: `egui` resolves some geometry a frame late, and a
    // harness assertion about the SECOND frame is the honest one.
    for pass in 0..2 {
        seen.clear();
        let mut sink = |name: &str, rect: Rect| seen.push((name.to_owned(), rect));
        let mut ribbon_state = state.clone();
        let mut out = Vec::new();
        {
            frame(&ctx, width, |ui| {
                out = Ribbon::new().reporting_rects_to(&mut sink).render(
                    ui,
                    &shell,
                    &registry,
                    &mut ribbon_state,
                );
            });
        }
        state = ribbon_state;
        if pass == 1 {
            invoked = out;
        }
    }
    (state, seen, invoked)
}

/// **★ Every rendered group emits a caption — including a group the
/// manifest forgot to caption.**
///
/// This is the invariant the band exists to make structural. Two
/// caption-less groups shipped in the salvage source and were caught
/// by a screenshot, not by a test: nothing was *wrong*, two call
/// sites simply did not follow a convention.
///
/// The test asserts it three ways, on purpose, because each catches a
/// different regression:
///
/// 1. **The counts match.** Catches a second drawing path being added
///    that skips the caption.
/// 2. **A caption rect is published for every group id, inside that
///    group's own rect.** Catches a caption drawn somewhere other
///    than under its own group — which is what an inline caption
///    looks like geometrically, and inline captions are the shape
///    that made the grouping invisible in the 2026-08-08 capture.
///
///    It deliberately does **not** assert a positive height. This
///    crate depends on `egui` with `default-features = false`, so a
///    test process has no font data and every galley measures zero;
///    a height assertion here would be measuring the absence of a
///    font, not the presence of a caption. Legibility is what
///    `ui-verify` asserts against a real window, on the frame the
///    rect was measured on — which is the entire reason the rects
///    are published rather than assumed.
/// 3. **The uncaptioned group still produced one.** Catches the
///    fallback in [`band::caption_text`] being "simplified" to an
///    empty string, which would reintroduce the original defect for
///    exactly the manifests most likely to have it.
#[test]
fn every_rendered_group_emits_a_caption() {
    let (state, seen, _) = render_view_tab(1600.0);
    let report = state.last_frame();

    assert!(report.groups_rendered >= 3, "the View tab has three groups");
    assert_eq!(
        report.groups_rendered, report.captions_emitted,
        "a group was drawn without a caption; every group must go through \
         `band::captioned_group`, which is the only function that draws one"
    );
    assert!(
        !report.overflow_visible,
        "at 1600 pt nothing should overflow, or this test is measuring the menu"
    );

    for group in ["page_display", "render", "window"] {
        let caption_name = report::group_caption("view", group);
        let (_, caption) = seen
            .iter()
            .find(|(n, _)| n == &caption_name)
            .unwrap_or_else(|| panic!("no caption was published for `{group}`; saw {seen:?}"));
        let group_name = report::group("view", group);
        let (_, whole) = seen
            .iter()
            .find(|(n, _)| n == &group_name)
            .unwrap_or_else(|| panic!("no rect was published for `{group}`"));

        assert!(
            caption.is_finite(),
            "`{group}`'s caption was emitted at {caption:?}"
        );
        assert!(
            whole.contains_rect(*caption),
            "`{group}`'s caption at {caption:?} is outside its own group at \
             {whole:?}, so it is captioning something else"
        );
    }

    // 3: the uncaptioned group falls back to its id rather than to a
    // blank, which is both non-empty and diagnostic.
    assert_eq!(
        band::caption_text(&Group::patch("window")),
        "window",
        "the fallback must name the group whose manifest entry needs fixing"
    );
}

/// **★ Every command control in the band publishes where it was drawn.**
///
/// # The gap this closes
///
/// The band reported its *groups* and their *captions* and nothing else,
/// so the forty controls an operator actually clicks were invisible to
/// anything outside the process. A caption rect answers "is this label
/// legible"; it cannot answer "did clicking this button do anything",
/// because nothing outside the window could find the button in order to
/// click it. The whole chain from a ribbon click to a control rendering
/// pressed was therefore covered only by unit tests, one per link — which
/// is exactly the state the icon painter was in when it shipped drawing
/// nothing: every part tested, the join untested, the join wrong.
///
/// # What is asserted, and why each part
///
/// 1. **One rect per `Item::Command` in the drawn tab**, under
///    [`report::band_item`]'s name. Catches the report being conditioned
///    on something — enabled, selected, having an icon — which would make
///    it go quiet in the states a harness most wants to inspect.
/// 2. **Inside its own group's rect.** Catches a control reported at a
///    rectangle that is not where it was drawn, which would aim a click
///    at empty chrome and make a working command look broken.
/// 3. **Nothing is published for `Item::Separator` or `Item::Custom`.**
///    Neither is a command and neither has an id to report under; a rect
///    appearing for one would mean the name is being built from
///    something other than a command id.
/// 4. **`ribbon.item.` and `ribbon.qat.` stay disjoint.** `file.open` is
///    on the File tab *and* on the quick-access toolbar in this fixture,
///    so the two surfaces genuinely draw the same command twice on one
///    frame. A harness filtering for band controls must not pick up the
///    QAT's copy, and `qat_icons`' aspect-ratio assertion must not pick
///    up the band's — a band control legitimately shows a label and is
///    legitimately wide.
///
/// Sizes are deliberately not asserted: this crate builds `egui` with
/// `default-features = false`, so a test process has no font data and
/// every galley measures zero. What a control *looks* like is
/// `ui-verify`'s question, against a real window, using precisely these
/// rects.
#[test]
fn every_band_command_publishes_the_rect_it_was_drawn_in() {
    let (_, seen, _) = render_view_tab(1600.0);

    // The View tab's three groups and every command in them, from
    // `shell()` above. Written out rather than derived from the manifest,
    // so a command silently disappearing from the fixture cannot make
    // this test vacuous.
    let expected: [(&str, &[&str]); 3] = [
        (
            "page_display",
            &["view.single", "view.continuous", "view.facing"],
        ),
        ("render", &["view.thin_lines"]),
        ("window", &["view.fullscreen", "view.reset_layout"]),
    ];

    for (group, commands) in expected {
        let group_name = report::group("view", group);
        let (_, whole) = seen
            .iter()
            .find(|(n, _)| n == &group_name)
            .unwrap_or_else(|| panic!("no rect was published for `{group}`"));
        for id in commands {
            let name = report::band_item(id);
            let (_, rect) = seen.iter().find(|(n, _)| n == &name).unwrap_or_else(|| {
                panic!(
                    "`{id}` was drawn in `{group}` and published no rect under `{name}`. \
                     A control nothing outside the process can locate is a control no \
                     check can click; see this test's header."
                )
            });
            assert!(
                whole.contains_rect(*rect),
                "`{id}` reported {rect:?}, which is outside its own group at {whole:?} — \
                 a click aimed at that rectangle would land on something else"
            );
        }
    }

    // 3: the two non-command item kinds report nothing. `render` holds a
    // `quality_slider` custom item and `file`'s group holds a separator;
    // neither has a command id, so neither may appear in this namespace.
    let items: Vec<&str> = seen
        .iter()
        .map(|(n, _)| n.as_str())
        .filter(|n| n.starts_with("ribbon.item."))
        .collect();
    assert_eq!(
        items.len(),
        6,
        "exactly the six commands on the View tab, and nothing for the custom item \
         or the separators: {items:?}"
    );

    // 4: the QAT's copy of a command is a different control on a
    // different surface, and the two namespaces must not overlap.
    assert!(
        seen.iter()
            .any(|(n, _)| n == &report::qat_item("file.open")),
        "the fixture puts `file.open` on the QAT, so this frame should carry both \
         namespaces and be able to prove they are disjoint"
    );
    assert!(
        !items.iter().any(|n| n.starts_with("ribbon.qat.")),
        "a QAT control leaked into the band namespace: {items:?}"
    );
}

/// **★ `MODES_AND_PANELS.md` failure mode #8: at a width narrow
/// enough to hide groups, the overflow control is still there and
/// still hit-testable.**
///
/// The observed defect: *"past ~6 tabs the overflow button itself
/// gets hidden, leaving no route to the hidden tabs."*
///
/// "Hit-testable" is asserted by **hovering it**, not by checking
/// that a rectangle exists. A rectangle proves something was
/// allocated; only `egui`'s own hit test proves it can be reached,
/// because that is what accounts for clipping, for occlusion by a
/// later widget, and for a zero-area interact rect.
///
/// The width is chosen below one group's worth of the band, which is
/// the case a naive implementation gets wrong: there is still *some*
/// room, so it spends it on a group and has nothing left for the
/// affordance.
#[test]
fn the_overflow_control_is_hit_testable_at_a_width_that_hides_groups() {
    let ctx = egui::Context::default();
    let shell = shell();
    let registry = registry();
    let mut state = RibbonState::new();
    state.set_active_tab("view");

    let narrow = 180.0;
    let mut rect_of_overflow = None;
    for _ in 0..2 {
        let mut seen = Vec::new();
        {
            let mut sink = |name: &str, rect: Rect| {
                if name == report::overflow() {
                    seen.push(rect);
                }
            };
            frame(&ctx, narrow, |ui| {
                let _ = Ribbon::new()
                    .reporting_rects_to(&mut sink)
                    .render(ui, &shell, &registry, &mut state);
            });
        }
        rect_of_overflow = seen.first().copied();
    }

    let report = state.last_frame().clone();
    assert!(
        report.groups_overflowed > 0,
        "at {narrow} pt the View tab's groups cannot all fit; if they do, \
         this test is no longer exercising failure mode #8"
    );
    assert!(
        report.overflow_visible,
        "groups were hidden with no affordance to reach them — failure mode #8"
    );

    let rect = rect_of_overflow.expect("the overflow affordance must publish its rect");
    assert!(
        rect.width() > 0.0 && rect.height() > 0.0,
        "the overflow affordance was allocated with no area: {rect:?}"
    );
    assert!(
        rect.right() <= narrow + 1.0 && rect.left() >= 0.0,
        "the overflow affordance was placed off-screen at {rect:?}, which is \
         failure mode #8 wearing a different hat"
    );

    // The real proof: ask `egui` to hit-test it.
    let id = report
        .overflow_id
        .expect("a visible overflow affordance must publish its id");
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::PointerMoved(rect.center()));
    frame_with_input(&ctx, narrow, input, &mut |ui| {
        let _ = Ribbon::new().render(ui, &shell, &registry, &mut state);
    });
    let response = ctx
        .read_response(id)
        .expect("the overflow affordance must be a widget egui knows about");
    assert!(
        response.hovered(),
        "the overflow affordance is on screen but cannot be hit at its own \
         centre {:?} — which is exactly the state failure mode #8 describes",
        rect.center()
    );
}

/// Widening the window retires the overflow menu and shows every
/// group again.
///
/// The other half of the previous test: an affordance that never went
/// away would also satisfy "always reachable", and would be a
/// permanent tax on a band that fits.
#[test]
fn a_wide_enough_band_shows_every_group_and_no_affordance() {
    let (narrow, _, _) = render_view_tab(180.0);
    let (wide, _, _) = render_view_tab(1600.0);
    assert!(narrow.last_frame().overflow_visible);
    assert!(!wide.last_frame().overflow_visible);
    assert_eq!(wide.last_frame().groups_overflowed, 0);
    assert_eq!(
        wide.last_frame().groups_rendered,
        3,
        "all three View groups are drawn in the band"
    );
}

/// **★ Every mode publishes its own labelled, positive-area segment.**
///
/// `MODES_AND_PANELS.md` Part 1 forbids *"a bare track with a knob,
/// where the available positions are invisible until you drag."* The
/// assertable form is: three modes, three segments, each with area.
/// A knob-and-track implementation publishes one rect and fails here.
#[test]
fn the_mode_selector_draws_every_position() {
    let (_, seen, _) = render_view_tab(1600.0);
    for mode in ["read", "review", "edit"] {
        let name = report::mode_segment(mode);
        let (_, rect) = seen
            .iter()
            .find(|(n, _)| n == &name)
            .unwrap_or_else(|| panic!("mode `{mode}` has no segment; saw {seen:?}"));
        assert!(
            rect.width() > 0.0 && rect.height() > 0.0,
            "mode `{mode}`'s segment has no area, so its label cannot be visible"
        );
    }
    let (_, track) = seen
        .iter()
        .find(|(n, _)| n == report::mode_selector())
        .expect("the selector publishes its whole track");
    assert!(track.width() > 0.0);
}

/// The mode selector is right-aligned on the tab-strip row, outboard
/// of the tabs.
///
/// Asserted as a geometric relation rather than as a coordinate, so
/// it survives a theme change, a font change and a fourth mode —
/// which is the whole reason rects are published rather than
/// hard-coded.
#[test]
fn the_mode_selector_sits_right_of_the_tab_strip() {
    let (_, seen, _) = render_view_tab(1600.0);
    let find = |name: &str| {
        seen.iter()
            .find(|(n, _)| n == name)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("`{name}` was not published; saw {seen:?}"))
    };
    let tabs_right = find(&report::tab("view")).right();
    let selector_left = find(report::mode_selector()).left();
    assert!(
        selector_left >= tabs_right,
        "the mode selector at {selector_left} overlaps the tab strip ending at \
         {tabs_right}"
    );
}

/// **★ The shell reports intent and executes nothing.**
///
/// A synthetic click on a QAT control returns that command's handler
/// token — and returns *only* that. Nothing in this crate can act on
/// it, which is the seam the module header describes.
#[test]
fn a_click_reports_a_handler_token_and_nothing_else_happens() {
    let ctx = egui::Context::default();
    let shell = shell();
    let registry = registry();
    let mut state = RibbonState::new();
    state.set_active_tab("view");

    // Frame 1: lay out, and learn where the QAT's Open control is.
    let mut open_rect = None;
    {
        let mut sink = |name: &str, rect: Rect| {
            if name == report::qat_item("file.open") {
                open_rect = Some(rect);
            }
        };
        frame(&ctx, 1600.0, |ui| {
            let _ = Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, &mut state);
        });
    }
    let open_rect = open_rect.expect("the QAT publishes its controls");

    // Frame 2: click it.
    let mut input = egui::RawInput::default();
    input.events.extend([
        egui::Event::PointerMoved(open_rect.center()),
        egui::Event::PointerButton {
            pos: open_rect.center(),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Event::PointerButton {
            pos: open_rect.center(),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]);
    let mut invoked = Vec::new();
    frame_with_input(&ctx, 1600.0, input, &mut |ui| {
        invoked = Ribbon::new().render(ui, &shell, &registry, &mut state);
    });

    assert_eq!(
        invoked,
        vec![HandlerToken::new(10)],
        "a click must report exactly the clicked command's token"
    );
    assert_eq!(state.last_frame().commands_invoked, 1);
}

/// A disabled command reports nothing when clicked.
///
/// `SHELL_FRAMEWORK.md` §5: *predicates are safety, not decoration.*
/// The enable predicate has to hold at the point the intent is
/// reported, not only at the point the control is greyed — otherwise
/// a customized ribbon could route around it.
#[test]
fn a_disabled_command_reports_no_intent() {
    let ctx = egui::Context::default();
    let mut registry = CommandRegistry::new();
    registry
        .register(
            Command::new("edit.undo", "Undo", HandlerToken::new(99)).enabled_when("undo.available"),
        )
        .expect("fresh registry");
    let shell =
        Shell::new().with_tab(Tab::new("edit", "Edit").with_groups([
            Group::new("history", "History").with_items([Item::command("edit.undo")]),
        ]));
    let mut state = RibbonState::new();

    let mut undo_rect = None;
    {
        let mut sink = |name: &str, rect: Rect| {
            if name == report::group("edit", "history") {
                undo_rect = Some(rect);
            }
        };
        frame(&ctx, 800.0, |ui| {
            let _ = Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, &mut state);
        });
    }
    let rect = undo_rect.expect("the group publishes its rect");

    let mut input = egui::RawInput::default();
    let at = rect.center();
    input.events.extend([
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
    ]);
    let mut invoked = Vec::new();
    frame_with_input(&ctx, 800.0, input, &mut |ui| {
        invoked = Ribbon::new().render(ui, &shell, &registry, &mut state);
    });
    assert!(
        invoked.is_empty(),
        "a command whose predicate is false must not report intent when clicked"
    );
}

/// A contextual tab appears in the strip while its condition holds,
/// and its band renders like any other.
#[test]
fn a_contextual_tab_renders_when_its_condition_holds() {
    let ctx = egui::Context::default();
    let shell = shell();
    let registry = registry();
    let mut state = RibbonState::new();
    state.set_active_tab("format");
    let conditions = ConditionSet::new().with("selection.any");

    frame(&ctx, 1600.0, |ui| {
        let _ = Ribbon::new()
            .with_conditions(&conditions)
            .render(ui, &shell, &registry, &mut state);
    });

    assert_eq!(state.active_tab(), Some("format"));
    assert_eq!(state.last_frame().tabs_visible, 3, "File, View and Format");
    assert_eq!(state.last_frame().groups_rendered, 1);
    assert_eq!(state.last_frame().captions_emitted, 1);

    // And it retires cleanly when the selection goes away.
    frame(&ctx, 1600.0, |ui| {
        let _ = Ribbon::new().render(ui, &shell, &registry, &mut state);
    });
    assert_eq!(
        state.active_tab(),
        Some("file"),
        "losing the active contextual tab must fall back, not blank the band"
    );
    assert_eq!(state.last_frame().tabs_visible, 2);
}

/// **A custom item is handed back to the application, with its kind,
/// its payload and where it appeared.**
///
/// The extension point that keeps the item vocabulary from growing a
/// variant per widget an application happens to want — which is the
/// road by which a reusable shell acquires a `ColourSwatch` variant
/// and stops being reusable.
#[test]
fn a_custom_item_is_handed_to_the_application_with_its_context() {
    let ctx = egui::Context::default();
    let shell = shell();
    let registry = registry();
    let mut state = RibbonState::new();
    state.set_active_tab("view");

    let mut seen: Vec<(String, String, String)> = Vec::new();
    {
        let mut renderer = |ui: &mut egui::Ui, item: &CustomItem<'_>| {
            seen.push((
                item.kind.to_owned(),
                item.tab.to_owned(),
                item.group.to_owned(),
            ));
            ui.label("slider");
            Some(HandlerToken::new(777))
        };
        frame(&ctx, 1600.0, |ui| {
            let _ = Ribbon::new()
                .with_custom_items(&mut renderer)
                .render(ui, &shell, &registry, &mut state);
        });
    }

    assert_eq!(
        seen,
        vec![(
            "quality_slider".to_owned(),
            "view".to_owned(),
            "render".to_owned()
        )],
        "the shell must say WHICH custom item and WHERE, so one renderer can \
         serve a kind that appears in more than one place"
    );
}

/// **A manifest naming an unregistered command loses that control and
/// keeps the rest.**
///
/// `SHELL_FRAMEWORK.md` §4: an unknown id is a *disclosed skip*, not
/// a crash. Reaching the renderer with one means the application did
/// not validate its manifest — a real defect, whose correct penalty
/// is one missing control rather than a panic in the paint loop with
/// a document open.
#[test]
fn an_unregistered_command_loses_one_control_not_the_band() {
    let ctx = egui::Context::default();
    let registry = registry();
    let shell = Shell::new().with_tab(Tab::new("view", "View").with_groups([
        Group::new("display", "Display").with_items([
            Item::command("view.single"),
            Item::command("view.does_not_exist"),
            Item::command("view.facing"),
        ]),
    ]));
    let mut state = RibbonState::new();

    frame(&ctx, 1600.0, |ui| {
        let _ = Ribbon::new().render(ui, &shell, &registry, &mut state);
    });

    assert_eq!(state.last_frame().groups_rendered, 1);
    assert_eq!(
        state.last_frame().captions_emitted,
        1,
        "the group survives with its caption intact"
    );
}

/// A manifest with no modes draws no selector at all.
///
/// A one-position segmented control is a control that cannot be
/// operated, and a zero-position one is a gap. An application without
/// modes gets neither.
#[test]
fn a_manifest_with_no_modes_draws_no_selector() {
    let ctx = egui::Context::default();
    let registry = registry();
    let shell = Shell::new().with_tab(Tab::new("view", "View").with_groups([
        Group::new("display", "Display").with_items([Item::command("view.single")]),
    ]));
    let mut state = RibbonState::new();
    let mut names: Vec<String> = Vec::new();
    {
        let mut sink = |name: &str, _: Rect| names.push(name.to_owned());
        frame(&ctx, 1600.0, |ui| {
            let _ = Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, &mut state);
        });
    }
    assert!(
        !names.iter().any(|n| n.starts_with("ribbon.mode")),
        "a manifest with no modes must produce no selector; got {names:?}"
    );
    assert!(names.iter().any(|n| n == "ribbon.tab.view"));
}

/// **The four-argument entry point is a complete, working ribbon.**
///
/// Every builder capability is optional. An application bringing the
/// shell up for the first time — no icon set, no custom items, no
/// harness — must get something that draws, and must not have to
/// discover four builder methods before it does.
#[test]
fn the_plain_entry_point_draws_a_working_ribbon() {
    let ctx = egui::Context::default();
    let shell = shell();
    let registry = registry();
    let mut state = RibbonState::new();
    frame(&ctx, 1600.0, |ui| {
        let invoked = Ribbon::show(ui, &shell, &registry, &mut state);
        assert!(invoked.is_empty(), "nothing was clicked");
    });
    assert_eq!(state.active_tab(), Some("file"));
    assert!(state.last_frame().groups_rendered > 0);
    assert_eq!(
        state.last_frame().groups_rendered,
        state.last_frame().captions_emitted
    );
}

/// An empty manifest draws an empty ribbon rather than panicking.
///
/// The first frame of an application that has not built its manifest
/// yet, and the last frame of one whose customization file emptied
/// itself. Neither is a reason to crash in the paint loop.
#[test]
fn an_empty_manifest_draws_an_empty_ribbon() {
    let ctx = egui::Context::default();
    let shell = Shell::new();
    let registry = CommandRegistry::new();
    let mut state = RibbonState::new();
    frame(&ctx, 800.0, |ui| {
        let invoked = Ribbon::show(ui, &shell, &registry, &mut state);
        assert!(invoked.is_empty());
    });
    assert_eq!(state.active_tab(), None);
    assert_eq!(state.last_frame().groups_rendered, 0);
    assert_eq!(state.last_frame().tabs_visible, 0);
}

/// **★ The mode selector is operable from the keyboard.**
///
/// `MODES_AND_PANELS.md` Part 1, behavioural rule 6: *"the selector
/// is a real focusable control with arrow-key movement — not a
/// mouse-only affordance."*
///
/// Driven the way a keyboard user would: focus the control, press
/// Right, press Right again, press Right a third time at the end.
/// The third press must do nothing, because
/// [`mode_selector::move_index`] clamps — a wrap would take the most
/// capable stance straight to the least in one keystroke.
///
/// This asserts the whole path — focus registration, key
/// consumption, index movement, state write — rather than the pure
/// [`mode_selector::move_index`] that `arrow_movement_clamps_rather_than_wrapping`
/// covers on its own. Either alone would pass with the other broken.
#[test]
fn the_mode_selector_moves_with_the_arrow_keys() {
    let ctx = egui::Context::default();
    let shell = shell();
    let registry = registry();
    let mut state = RibbonState::new();

    frame(&ctx, 1600.0, |ui| {
        let _ = Ribbon::show(ui, &shell, &registry, &mut state);
    });
    // The selector resolves to the first mode with nothing chosen.
    assert_eq!(state.last_frame().mode.as_deref(), Some("read"));

    // Focus the control, as Tab would.
    ctx.memory_mut(|m| m.request_focus(state.mode_segment_id("read")));

    let press_right = |state: &mut RibbonState| {
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key {
            key: egui::Key::ArrowRight,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        frame_with_input(&ctx, 1600.0, input, &mut |ui| {
            let _ = Ribbon::show(ui, &shell, &registry, state);
        });
    };

    press_right(&mut state);
    assert_eq!(
        state.mode(),
        Some("review"),
        "one Right press must advance one position"
    );
    press_right(&mut state);
    assert_eq!(state.mode(), Some("edit"));
    press_right(&mut state);
    assert_eq!(
        state.mode(),
        Some("edit"),
        "at the last position a further Right must do nothing — a wrap \
         would take the most capable stance to the least in one keystroke"
    );
}

/// **★ Groups in the overflow menu are captioned too.**
///
/// The menu is the place a second, simpler drawing path would be
/// most tempting — it is a vertical list, the band's centring does
/// not obviously apply, and nobody looks at it in a screenshot. That
/// is exactly how the two caption-less groups in the salvage source
/// happened, so the menu is routed through the same
/// `band::captioned_group` closure as the band, and this asserts it.
///
/// The count is the whole point: with the menu open, **every** group
/// on the tab has been drawn and every one of them emitted a caption.
#[test]
fn a_group_in_a_popup_is_captioned_too() {
    let ctx = egui::Context::default();
    // ★★ RETARGETED 2026-08-25, not deleted. This test guarded *"a group drawn
    // inside a popup still gets its caption"* — the defect the salvage source
    // actually shipped, twice. Its subject used to be the `⏷ N more` dropdown;
    // S4 replaced that with a scroll arrow, so the only popup that renders
    // groups is now a COLLAPSED GROUP'S. The invariant is unchanged and still
    // reachable, so the test follows it rather than retiring with the
    // mechanism it happened to be written against.
    //
    // A local fixture, because giving the shared one a collapse priority
    // changes every width in this module and broke two unrelated tests when it
    // was tried.
    let shell = collapsing_shell();
    let registry = registry();
    let mut state = RibbonState::new();
    state.set_active_tab("view");

    let narrow = 180.0;
    let mut overflow_rect = None;
    {
        let mut sink = |name: &str, rect: Rect| {
            if name == "ribbon.group.view.page_display.collapsed" {
                overflow_rect = Some(rect);
            }
        };
        frame(&ctx, narrow, |ui| {
            let _ = Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, &mut state);
        });
    }
    let at = overflow_rect
        .expect("the collapsed group publishes its rect")
        .center();
    // ★ Zero is a legal value here and the first draft asserted otherwise.
    // A COLLAPSED group deliberately contributes to neither counter while its
    // popup is shut — see `collapsed`'s note on why there is no `count`
    // helper — so at a width where the only group on the band is the collapsed
    // one, nothing has been "rendered" yet. What makes the assertion below
    // meaningful is not that this is non-zero but that the popup RAISES it.
    let closed = state.last_frame().groups_rendered;

    // Click it, then let the popup render.
    let mut input = egui::RawInput::default();
    input.events.extend([
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
    ]);
    frame_with_input(&ctx, narrow, input, &mut |ui| {
        let _ = Ribbon::new().render(ui, &shell, &registry, &mut state);
    });
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::PointerMoved(at));
    frame_with_input(&ctx, narrow, input, &mut |ui| {
        let _ = Ribbon::new().render(ui, &shell, &registry, &mut state);
    });

    let report = state.last_frame();
    assert_eq!(
        report.groups_rendered, report.captions_emitted,
        "a group in a popup was drawn without a caption"
    );
    assert!(
        report.groups_rendered > closed,
        "the popup must actually have opened: {} groups drawn with it open against {closed} with it closed",
        report.groups_rendered
    );
}

/// A shell whose View tab has a group that collapses, so there is a popup to
/// open. See [`a_group_in_a_popup_is_captioned_too`].
fn collapsing_shell() -> Shell {
    let mut s = shell();
    if let Some(tab) = s.tabs.iter_mut().flatten().find(|t| t.id == "view")
        && let Some(g) = tab
            .groups
            .iter_mut()
            .flatten()
            .find(|g| g.id == "page_display")
    {
        g.collapse = Some(1);
    }
    s
}

/// The builder's borrows are ergonomic at a realistic call site: a
/// closure declared as a local is passed directly, with no
/// intermediate binding and no explicit lifetime anywhere.
///
/// Worth a test because the four capabilities are `&'a mut dyn`
/// borrows and an over-constrained signature would compile here in
/// this crate and fail in the *application*, at the one call site
/// that matters, with a lifetime error nobody can read.
#[test]
fn the_builder_takes_plain_local_closures() {
    let ctx = egui::Context::default();
    let shell = shell();
    let registry = registry();
    let mut state = RibbonState::new();
    let conditions = ConditionSet::new().with("selection.any");
    let mut seen = 0_usize;
    let mut rects = |_: &str, _: Rect| seen += 1;
    let mut icons = |_: &egui::Painter, _: &IconRequest<'_>| {};
    let mut custom = |ui: &mut egui::Ui, _: &CustomItem<'_>| {
        ui.label("x");
        None
    };
    frame(&ctx, 1200.0, |ui| {
        let _ = Ribbon::new()
            .with_conditions(&conditions)
            .reporting_rects_to(&mut rects)
            .with_icon_painter(&mut icons)
            .with_custom_items(&mut custom)
            .render(ui, &shell, &registry, &mut state);
    });
    assert!(seen > 0, "the sink received nothing");
}

/// [`IconRequest::selected`] reports the control's selected state, and
/// tracks the `selected:` condition rather than being wired to a
/// constant.
///
/// # Why this needs a test at all
///
/// A field that is always `false` compiles, renders, and looks correct.
/// The shell already draws selection with the button's frame, so an icon
/// set that ignored a permanently-false flag would produce a ribbon
/// nobody could tell was wrong by looking at it — and the whole reason
/// the field exists is to let an application keep the rule *selected
/// state is never colour alone*, which is precisely the rule whose
/// violations are invisible in the preset you happen to be using.
///
/// So both polarities are asserted in one run, from one registry, with
/// the only difference being the condition set. Asserting `true` alone
/// would pass against a field hard-wired to `true`.
#[test]
fn the_icon_request_reports_selected_state() {
    fn selected_flag_for(conditions: &ConditionSet) -> Option<bool> {
        let ctx = egui::Context::default();
        let shell = shell();
        let registry = registry();
        let mut state = RibbonState::new();
        // `file.open` is the one command in the test registry with an
        // icon, which is what puts a slot on screen for the painter to
        // be called with in the first place.
        let mut seen: Option<bool> = None;
        let mut icons = |_: &egui::Painter, req: &IconRequest<'_>| {
            if req.key == "open" {
                seen = Some(req.selected);
            }
        };
        frame(&ctx, 1200.0, |ui| {
            let _ = Ribbon::new()
                .with_conditions(conditions)
                .with_icon_painter(&mut icons)
                .render(ui, &shell, &registry, &mut state);
        });
        seen
    }

    let off = selected_flag_for(&ConditionSet::new());
    let on = selected_flag_for(&ConditionSet::new().with(selected_condition("file.open")));

    assert_eq!(
        off,
        Some(false),
        "an unselected command must report selected = false; \
         `None` means the painter was never called and the test proved nothing"
    );
    assert_eq!(
        on,
        Some(true),
        "the `selected:file.open` condition must reach IconRequest::selected"
    );
}

/// Two ribbons in one context do not share widget ids.
///
/// Without a distinct base id the symptom is that hovering a control
/// in one window highlights the corresponding control in the other —
/// a bug that is baffling until it is understood and trivial
/// afterwards.
#[test]
fn two_ribbons_can_coexist_with_distinct_id_salts() {
    let a = RibbonState::new().with_id_salt("left");
    let b = RibbonState::new().with_id_salt("right");
    assert_ne!(
        a.mode_segment_id("read"),
        b.mode_segment_id("read"),
        "two ribbons with different salts must not share widget ids"
    );
    assert_eq!(
        a.mode_segment_id("read"),
        RibbonState::new()
            .with_id_salt("left")
            .mode_segment_id("read"),
        "and the same salt must be stable across constructions"
    );
}

/// ★★★ **Pressing the right arrow scrolls the band, and the LEFT arrow then
/// appears** — the round trip, driven.
///
/// Written 2026-08-25, within the hour of S4 shipping, because the left arrow
/// had been **built and never observed**. Every other part of the scroll was
/// measured offscreen against the running application: the right arrow appears
/// at 1000 pt and not at 1200, the ladder compacts in the right order, the band
/// keeps its height. The left arrow only exists once something has scrolled,
/// nothing had scrolled, and so the one control on that surface with no
/// evidence behind it went out in a release.
///
/// *A check that cannot fail is not evidence*, and neither is a control that
/// has never been seen to draw. This is the falsification: press the right
/// arrow, and assert the band moved AND that the way back is on screen.
///
/// The second assertion is the one that matters. A scroll that advances with no
/// way back is not a scrolled band, it is a **trapped** one — every group left
/// of the fold unreachable for the life of the session, with the ribbon looking
/// entirely normal.
#[test]
fn scrolling_right_moves_the_band_and_offers_the_way_back() {
    let ctx = egui::Context::default();
    // ★★ A REAL FONT, pinned. Without it this test passed under
    // `cargo test -p egui-shell` and failed under `cargo test --workspace`,
    // because feature unification with `pdfcer-gui` changes the ambient font and
    // therefore every measured width — at which point the fixture's 180 pt was
    // too narrow to draw any group at all. A layout test whose verdict depends
    // on which crates happen to be in the build is not a layout test.
    // `width_tests` has installed this font for exactly this reason since it
    // was written; this one had to learn it.
    super::testfont::install(&ctx);
    let shell = shell();
    let registry = registry();
    let mut state = RibbonState::new();
    state.set_active_tab("view");

    // Narrow enough that the View tab's three groups cannot all fit, so the
    // right arrow is drawn and there is somewhere to scroll to.
    let narrow = 180.0;

    let mut rects: Vec<(String, Rect)> = Vec::new();
    {
        let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
        frame(&ctx, narrow, |ui| {
            let _ = Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, &mut state);
        });
    }

    let right = rects
        .iter()
        .find(|(n, _)| n == report::overflow())
        .map(|(_, r)| *r)
        .expect("at this width the band overflows, so the right arrow must be drawn");
    assert!(
        !rects.iter().any(|(n, _)| n == "ribbon.scroll.left"),
        "nothing has scrolled yet, so there is no way back to offer — and R9 says an inapplicable control renders NOTHING rather than greyed"
    );

    // Press it.
    let at = right.center();
    let mut input = egui::RawInput::default();
    input.events.extend([
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
    ]);
    frame_with_input(&ctx, narrow, input, &mut |ui| {
        let _ = Ribbon::new().render(ui, &shell, &registry, &mut state);
    });

    // And look again.
    let mut after: Vec<(String, Rect)> = Vec::new();
    {
        let mut sink = |name: &str, rect: Rect| after.push((name.to_owned(), rect));
        frame(&ctx, narrow, |ui| {
            let _ = Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, &mut state);
        });
    }

    // ★ The band having MOVED is not asserted separately, and does not need to
    // be: `ribbon.scroll.left` is drawn under exactly one condition, `scrolled
    // > 0`. Its presence IS the proof that the click advanced the band. The
    // first draft compared the leading group's name before and after, which
    // needed a group rect to exist and made the test depend on the fixture
    // being wide enough to draw one — a dependency that broke it under
    // workspace feature unification and proved nothing the line below does not.
    assert!(
        after.iter().any(|(n, _)| n == "ribbon.scroll.left"),
        "once the band has scrolled, the way back MUST be on screen. Without it the ribbon is not scrolled, it is trapped: every group left of the fold is unreachable for the rest of the session and the band looks entirely normal while it happens"
    );
}
