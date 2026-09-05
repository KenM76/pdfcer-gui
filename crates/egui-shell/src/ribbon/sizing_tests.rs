//! Rendered geometry for the three item sizes and the `visible_when` filter.
//!
//! `RIBBON_SCALING.md`, `OPERATOR_REQUESTS.md` O31.
//!
//! # Why these are here and not in [`super::sizing`]'s own test module
//!
//! Because they measure **what was drawn**, and two of them need a font. This
//! crate depends on `egui` with `default-features = false`, so a plain test
//! process has no font data and every galley measures zero — which would make
//! *"an icon-only control is narrower than a labelled one"* pass against an
//! implementation that had never dropped the label, since both would measure
//! the icon and nothing else.
//!
//! [`super::testfont`] is the answer, and it is why [`super::width_tests`]
//! exists at all. This file borrows its harness rather than duplicating it:
//! one synthetic face, installed one way, asserted to actually measure
//! something before any test relies on it.
//!
//! ★ It is a **separate file** from `width_tests` for R2's reason and no
//! other: that one is 1,433 lines against a 1,500-line limit, and a rule that
//! is obeyed by writing the new tests somewhere else is a rule that is
//! working.

use egui::Rect;

use crate::commands::{Command, CommandRegistry, ConditionSet, HandlerToken};
use crate::manifest::{Group, Item, ItemSize, Mode, Shell, Tab};

use super::width_tests::context;

/// Two commands, both fully equipped, so `Small` is **earned** and the tests
/// below measure the size rule rather than the fallback.
///
/// ★ Each carries an icon **and** a tooltip. A fixture missing either would
/// make every `Small` in this file silently render as `Medium`, and the tests
/// asserting a narrower control would fail for a reason that has nothing to do
/// with what they are about — see [`super::sizing::resolved`].
fn registry() -> CommandRegistry {
    let mut r = CommandRegistry::new();
    r.register_all([
        Command::new("a.one", "Alpha command", HandlerToken::new(1))
            .with_icon("k1")
            .with_tooltip("The first"),
        Command::new("a.two", "Beta command", HandlerToken::new(2))
            .with_icon("k2")
            .with_tooltip("The second"),
    ])
    .expect("distinct ids");
    r
}

/// A one-tab, one-group manifest holding exactly `items`.
///
/// Deliberately minimal: every test below compares two renders that differ in
/// one property, and anything else on the tab would be width the comparison
/// has to reason about.
fn shell(items: impl IntoIterator<Item = Item>) -> Shell {
    Shell::new()
        .with_mode(Mode::new("only", "Only", ["t"]))
        .with_tab(Tab::new("t", "Tab").with_groups([Group::new("g", "Group").with_items(items)]))
}

/// The group's own rect, from the reported regions.
fn group_rect(rendered: &[(String, Rect)]) -> Option<Rect> {
    rendered
        .iter()
        .find(|(name, _)| name == "ribbon.group.t.g")
        .map(|(_, r)| *r)
}

/// **The area the group's ITEMS occupy** — the union of every `ribbon.item.*`
/// rect the frame published, in square points.
///
/// ★★★ **THE ORACLE FOR "SPACE IS RECLAIMED", AND THE GROUP'S WIDTH IS NOT.**
///
/// The two reclaim tests below used to compare `group_rect(..).width()`, and
/// that oracle is only valid while a group lays its items out on ONE ROW. On
/// 2026-09-05 `band::measure_group_rows` began asking every group for the
/// band's full row ceiling, so two equal-width controls stack into a column of
/// two — and the column is exactly as WIDE with one item as with two. Both
/// tests failed printing `126.71875 vs 126.71875`: **the same number on both
/// sides, which is the tell that the measurement had stopped being able to see
/// the property rather than that the property had gone.**
///
/// A hole — an item measured but not drawn — is still a hole under either
/// layout, and under either layout it shows up as **area**: two items occupy
/// twice one item's area whether they sit side by side or one above the other.
/// So the assertion is layout-independent, which is what the original was
/// silently not.
///
/// ⇒ **When a layout change turns a passing assertion red, ask whether the
/// assertion was measuring the rule or the arrangement.** This project has
/// recorded the same shape against `ui-verify` repeatedly (*ask what the check
/// SAMPLED*); it applies to a unit test's choice of dimension identically.
fn item_area(rendered: &[(String, Rect)]) -> f32 {
    rendered
        .iter()
        .filter(|(name, _)| name.starts_with("ribbon.item."))
        .map(|(_, r)| r.width() * r.height())
        .sum()
}

/// One item's rect.
fn item_rect(rendered: &[(String, Rect)], id: &str) -> Option<Rect> {
    let want = format!("ribbon.item.{id}");
    rendered
        .iter()
        .find(|(name, _)| *name == want)
        .map(|(_, r)| *r)
}

/// Render a manifest at a comfortable width, **with an icon painter
/// installed**, and report every rect.
///
/// ★★★ The painter is the whole reason this file has its own render function
/// instead of calling [`render_shell_with`] like its neighbours. `Small` is
/// **earned** — it needs an icon, a tooltip *and* an installed painter — and
/// the shared harness installs no painter, so every `Small` in every test here
/// would silently render as `Medium` and the assertions would fail against a
/// perfectly correct implementation.
///
/// That is not a flaw in the shared harness: a ribbon with no icon painter is
/// a working ribbon, and its tests are right not to invent one. It is a
/// property of what this file measures.
///
/// The painter draws **nothing**. It exists to be `Some`. What is being
/// measured is the space a control reserves, and a painter that filled its
/// rect would be measuring `egui`'s compositor.
fn render_with_icons(
    items: impl IntoIterator<Item = Item>,
    registry: &CommandRegistry,
    conditions: &ConditionSet,
) -> Vec<(String, Rect)> {
    let ctx = context();
    let shell = shell(items);
    let mut state = crate::ribbon::RibbonState::new();
    state.set_active_tab("t");
    let mut rects = Vec::new();
    // Two frames, because `egui` resolves some geometry a frame late and the
    // second is the honest one — the same reason every other harness in this
    // crate runs twice.
    for _ in 0..2 {
        rects.clear();
        let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
        let mut paint = |_: &egui::Painter, _: &crate::ribbon::IconRequest<'_>| {};
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(1400.0, 400.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let _ = crate::ribbon::Ribbon::new()
                .with_conditions(conditions)
                .with_icon_painter(&mut paint)
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, registry, &mut state);
        });
    }
    rects
}

/// [`render_with_icons`] against the shared fixture registry.
fn render(items: impl IntoIterator<Item = Item>, conditions: &ConditionSet) -> Vec<(String, Rect)> {
    render_with_icons(items, &registry(), conditions)
}

/// ★★★ **An icon-only control is narrower than the same control labelled.**
///
/// The whole point of `Small`, and the measurement that moved the 884-point
/// number in `RIBBON_SCALING.md` §2. Asserted as a *comparison* between two
/// renders of the same command rather than against a number, because the
/// absolute width depends on the synthetic face's metrics and a literal here
/// would be pinning the fixture rather than the rule.
#[test]
fn an_icon_only_control_is_narrower_than_a_labelled_one() {
    let none = ConditionSet::new();
    let labelled = render([Item::command("a.one")], &none);
    let icon_only = render([Item::command("a.one").sized(ItemSize::Small)], &none);

    let wide = item_rect(&labelled, "a.one").expect("the labelled control drew");
    let narrow = item_rect(&icon_only, "a.one").expect("the icon-only control drew");
    assert!(
        narrow.width() < wide.width(),
        "icon-only must be narrower: {} vs {}",
        narrow.width(),
        wide.width()
    );
    // ★ And the GROUP narrowed with it. A control that shrank inside a group
    // whose width did not would have saved nothing — the band's plan is made
    // of group widths, and that is the number the operator feels.
    let wide_group = group_rect(&labelled).expect("group drew");
    let narrow_group = group_rect(&icon_only).expect("group drew");
    assert!(
        narrow_group.width() < wide_group.width(),
        "the group must narrow too: {} vs {}",
        narrow_group.width(),
        wide_group.width()
    );
}

/// **A Large control is taller than a Medium one**, because it spans the
/// band's rows rather than sitting in one of them.
///
/// Height needs no font, so this one would pass without the synthetic face —
/// it is here because it is the same subject, and because a reader comparing
/// the three sizes wants the three assertions together.
#[test]
fn a_large_control_spans_the_rows_a_medium_one_sits_in() {
    let none = ConditionSet::new();
    let medium = render([Item::command("a.one")], &none);
    let large = render([Item::command("a.one").sized(ItemSize::Large)], &none);

    let short = item_rect(&medium, "a.one").expect("the medium control drew");
    let tall = item_rect(&large, "a.one").expect("the large control drew");
    assert!(
        tall.height() > short.height(),
        "a large control must span the rows: {} vs {}",
        tall.height(),
        short.height()
    );
}

/// ★★★ **A hidden item is not drawn, and its space is reclaimed.**
///
/// Both halves, because only the first is obvious and only the second is the
/// operator's ask. A `visible_when` applied at draw time would satisfy the
/// first and leave a hole: the group would still be measured at its full
/// width, the groups to its right would not move left, and *"shift the space
/// used depending on what exists"* would be false.
#[test]
fn a_hidden_item_is_not_drawn_and_its_space_is_reclaimed() {
    let mut on = ConditionSet::new();
    on.set("show.two");
    let off = ConditionSet::new();
    let items = || {
        [
            Item::command("a.one"),
            Item::command("a.two").shown_when("show.two"),
        ]
    };

    let both = render(items(), &on);
    let one = render(items(), &off);

    assert!(
        item_rect(&both, "a.two").is_some(),
        "the conditioned item must draw while its condition holds"
    );
    assert!(
        item_rect(&one, "a.two").is_none(),
        "and must not draw when it does not"
    );
    assert!(
        item_rect(&one, "a.one").is_some(),
        "its neighbour is unaffected"
    );

    // The GROUP still draws in both cases; what must shrink is the space its
    // items take. See `item_area` for why that, and not the group's width, is
    // the oracle a stacking layout leaves standing.
    assert!(
        group_rect(&both).is_some() && group_rect(&one).is_some(),
        "group drew"
    );
    let wide = item_area(&both);
    let narrow = item_area(&one);
    assert!(
        narrow < wide,
        "the group must give the hidden item's space back: {narrow} vs {wide} sq pt"
    );
}

/// **A group whose every item is hidden is not drawn at all** — R9, and the
/// end of the same rule.
///
/// ★ Not "drawn empty", and not "drawn with just its caption". A caption over
/// nothing is a promise of a control that is not there, and the separator
/// beside it is a rule between two things with nothing between them.
#[test]
fn a_group_with_nothing_left_is_not_drawn() {
    let off = ConditionSet::new();
    let rendered = render(
        [
            Item::command("a.one").shown_when("never"),
            Item::command("a.two").shown_when("never"),
        ],
        &off,
    );
    assert!(
        group_rect(&rendered).is_none(),
        "an emptied group must vanish, caption and all: {:?}",
        rendered.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );
}

/// A `Small` that has not earned icon-only rendering draws at `Medium` width
/// — the fallback, measured rather than asserted about the resolver.
///
/// ★ This is the guard that lets a manifest ask for `Small` freely. Without
/// it, marking a tooltip-less command `Small` would ship an unlabelled
/// rectangle, and the author would have no way to know except by looking.
#[test]
fn a_small_that_has_not_earned_it_renders_at_medium_width() {
    let mut r = CommandRegistry::new();
    // Icon, but no tooltip — so no accessible name, so no icon-only.
    r.register_all([Command::new("a.one", "Alpha command", HandlerToken::new(1)).with_icon("k1")])
        .expect("distinct ids");
    let none = ConditionSet::new();
    let asked_small = render_with_icons([Item::command("a.one").sized(ItemSize::Small)], &r, &none);
    let plain = render_with_icons([Item::command("a.one")], &r, &none);

    let a = item_rect(&asked_small, "a.one").expect("drew");
    let b = item_rect(&plain, "a.one").expect("drew");
    assert!(
        (a.width() - b.width()).abs() < 0.5,
        "an unearned Small must fall back to the labelled width: {} vs {}",
        a.width(),
        b.width()
    );
}

/// ★★★ **A Large control in the OVERFLOW MENU is still tall enough to click.**
///
/// The regression this file exists to hold, and the one thing here that was a
/// shipped defect rather than a hypothetical.
///
/// A group drawn in the menu uses `GroupBox::NATURAL`, whose row height is
/// `0.0` **on purpose** — so a one-row group in the popup has no hole beneath
/// it. The first `render_large` allocated exactly the height it was handed, so
/// a Large control in the menu got a rect of **zero height**: it painted (the
/// icon and label are placed from the rect's centre, which still exists), it
/// reported its rect as required, and it **could not be clicked**.
///
/// ★ Every unit test passed, because the band path hands a real row height and
/// only the menu path does not. `ui-verify`'s `print_dialog_reaches_the_spooler`
/// found it, at the width the harness drives, and said exactly the right
/// thing: `ribbon.item.file.print` declared at `y 148.0 .. 148.0`, *"which has
/// no usable area — the control is laid out and not on screen"*.
///
/// This drives the same path: a band too narrow for the group, a click on the
/// affordance, and an assertion about the rect the menu reported.
#[test]
fn a_large_control_in_a_popup_is_tall_enough_to_click() {
    let ctx = context();
    let registry = registry();
    // ★★ RETARGETED 2026-08-25 from the `⏷ N more` dropdown to a COLLAPSED
    // GROUP's popup, which S4 left as the only popup that renders groups. The
    // defect guarded is unchanged and is one of the sharpest in this crate: a
    // Large control handed `GroupBox::NATURAL` — whose `rows` is 0.0 — used to
    // allocate a rect of ZERO HEIGHT. It painted, it published its rect, and it
    // was not clickable, because a zero-height rect has no area to hit.
    // `ui-verify` found it in the honest way, reporting `ribbon.item.file.print`
    // at `y 148.0 .. 148.0`. Every unit test passed, because only the popup
    // path passes a zero.
    let mut shell = shell([Item::command("a.one").sized(ItemSize::Large)]);
    if let Some(g) = shell
        .tabs
        .iter_mut()
        .flatten()
        .flat_map(|t| t.groups.iter_mut().flatten())
        .next()
    {
        g.collapse = Some(1);
    }
    let mut state = crate::ribbon::RibbonState::new();
    state.set_active_tab("t");
    // Narrow enough that the only group cannot fit beside the affordance.
    let narrow = 60.0_f32;

    let render = |ctx: &egui::Context,
                  state: &mut crate::ribbon::RibbonState,
                  input: egui::RawInput,
                  rects: &mut Vec<(String, Rect)>| {
        rects.clear();
        let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
        let mut input = input;
        input.screen_rect = Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::Vec2::new(narrow, 400.0),
        ));
        let _ = ctx.run_ui(input, |ui| {
            let _ = crate::ribbon::Ribbon::new()
                .reporting_rects_to(&mut sink)
                .render(ui, &shell, &registry, state);
        });
    };

    let mut rects = Vec::new();
    render(&ctx, &mut state, egui::RawInput::default(), &mut rects);
    let affordance = rects
        .iter()
        .find(|(n, _)| n == "ribbon.group.t.g.collapsed")
        .map(|(_, r)| *r)
        .expect("the band is too narrow for the group, so it must have collapsed");
    assert!(
        item_rect(&rects, "a.one").is_none(),
        "with the popup closed the control must not be on the band, or the assertion below could be satisfied without the popup ever opening"
    );

    // Click the affordance, then let the popup render and settle.
    let at = affordance.center();
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
    render(&ctx, &mut state, input, &mut rects);
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::PointerMoved(at));
    render(&ctx, &mut state, input, &mut rects);

    let drawn = item_rect(&rects, "a.one")
        .expect("with the menu open the group's control must have been drawn");
    assert!(
        drawn.height() > 0.0,
        "a Large control in the overflow menu must have a clickable height, got {:?}",
        drawn
    );
}

/// ★★★ **A custom item obeys `visible_when` exactly as a command does** —
/// added 2026-08-27, with the field.
///
/// # Why this is asserted through the RENDERER rather than through a rect
///
/// A [`Item::Custom`] publishes no `ribbon.item.<id>` region: the shell does
/// not draw it and has no id to name it by. So *"was it drawn?"* cannot be
/// read out of the reported rects the way the command tests above read it, and
/// the honest observation is whether the application's renderer was **called**.
/// A count is that observation, and it is stronger than a rect would be: it
/// distinguishes *"the shell skipped the item"* from *"the shell called the
/// renderer and the renderer chose to draw nothing"*, which is exactly the
/// difference the new field exists to remove.
///
/// # And the group narrows, which is the half that is easy to leave out
///
/// `super::sizing::visible` runs **before measurement**, so a hidden custom
/// item must give back `plan::CUSTOM_ITEM_WIDTH` rather than leaving a hole
/// the band has already budgeted for. Drawing nothing into a reserved slot is
/// precisely what pdfcer would have had to do without this field, and the gap
/// it leaves is why the field was added instead.
#[test]
fn a_hidden_custom_item_is_never_offered_to_the_renderer_and_gives_its_width_back() {
    let mut on = ConditionSet::new();
    on.set("mode.editing");
    let off = ConditionSet::new();

    let render_counting = |conditions: &ConditionSet| -> (usize, Rect) {
        let ctx = context();
        // ★★★ `with_prefer_rows(1)` since 2026-09-05, and it is the FIXTURE
        // pinning the layout so that WIDTH is a valid oracle — not a change to
        // what is being tested.
        //
        // A custom item publishes no `ribbon.item.*` rect (that is this test's
        // own first paragraph), so `item_area` cannot see it and the only
        // observable is the group's box. Once `band::measure_group_rows` began
        // asking every group for the band's row ceiling, the group's two items
        // STACKED — and a column is as wide with one item as with two whenever
        // the hidden one is not the widest. `CUSTOM_ITEM_WIDTH` is 96 and
        // `a.one` labelled is ~115, so it never was. The test failed printing
        // `126.71875 vs 126.71875`: the same number twice, which says the
        // measurement stopped being able to see the property.
        //
        // One row is a state a manifest can legally declare, it is what makes
        // "gives its budgeted width back" a statement about width at all, and
        // it leaves the property under test — `sizing::visible` running BEFORE
        // measurement — exactly where it was.
        let shell = Shell::new()
            .with_mode(Mode::new("only", "Only", ["t"]))
            .with_tab(Tab::new("t", "Tab").with_groups([
                Group::new("g", "Group").with_prefer_rows(1).with_items([
                    Item::command("a.one"),
                    Item::custom("swatch").shown_when("mode.editing"),
                ]),
            ]));
        let registry = registry();
        let mut state = crate::ribbon::RibbonState::new();
        state.set_active_tab("t");
        let mut calls = 0usize;
        let mut rects: Vec<(String, Rect)> = Vec::new();
        // Two frames, for the reason `render_with_icons` runs twice.
        for _ in 0..2 {
            calls = 0;
            rects.clear();
            let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
            // ★ The renderer ALLOCATES. A renderer that drew nothing would
            // leave both groups the same width and the width half of this
            // test would pass against an implementation that never filtered
            // anything — which is what the first draft of it did.
            let mut custom = |ui: &mut egui::Ui, _: &crate::ribbon::CustomItem<'_>| {
                calls += 1;
                ui.allocate_space(egui::Vec2::new(60.0, 20.0));
                None
            };
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2::new(1400.0, 400.0),
                )),
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| {
                let _ = crate::ribbon::Ribbon::new()
                    .with_conditions(conditions)
                    .with_custom_items(&mut custom)
                    .reporting_rects_to(&mut sink)
                    .render(ui, &shell, &registry, &mut state);
            });
        }
        (calls, group_rect(&rects).expect("the group drew"))
    };

    let (shown_calls, wide) = render_counting(&on);
    let (hidden_calls, narrow) = render_counting(&off);

    assert_eq!(
        shown_calls, 1,
        "while its condition holds the renderer must be offered the item exactly once"
    );
    assert_eq!(
        hidden_calls, 0,
        "when the condition does not hold the renderer must never be asked to draw"
    );
    assert!(
        narrow.width() < wide.width(),
        "the group must give the hidden custom item's budgeted width back: {} vs {}",
        narrow.width(),
        wide.width()
    );
}

// ===========================================================================
// ★★★ THE MOCKUP'S `Large` CONTROL — 2026-09-04
//
// `mockups/pdfcer-shell.html` specifies a Large control as
//
//     .rb.big     { height: 56px; gap: 4px; padding: 5px 8px 2px;
//                   min-width: 52px }
//     .rb.big .lb { font-size: 11px; max-width: 76px; white-space: normal }
//     svg.g.big   { width: 24px; height: 24px }
//
// and the operator's second complaint was about exactly this control: *"the
// mock's `New` is a large item — glyph above, label beneath, centred in its
// own column. The real one is a small row."* Half of that is a manifest
// change (which items are Large) and half is this file's subject: what a
// Large control looks like once it is one.
//
// Three properties are pinned, and each is a thing the shipped control got
// wrong rather than a restatement of the CSS:
//
//   · the label WRAPS, so a long-labelled Large control is a button and not
//     a letterbox;
//   · a short-labelled one does not collapse below the mockup's floor;
//   · a Large control is SHORTER than the row area it sits in, which is what
//     stops a group of them reading as one solid block.
// ===========================================================================

/// A registry whose one command has a label far wider than
/// `sizing::LARGE_LABEL_WRAP`, and one whose label is far narrower.
///
/// Both fully equipped (icon + tooltip) for [`registry`]'s stated reason, so
/// nothing here is measuring a size that silently fell back.
fn wrapping_registry() -> CommandRegistry {
    let mut r = CommandRegistry::new();
    r.register_all([
        Command::new(
            "a.long",
            "Save a compacted copy of this document",
            HandlerToken::new(1),
        )
        .with_icon("k1")
        .with_tooltip("The long one"),
        Command::new("a.short", "New", HandlerToken::new(2))
            .with_icon("k2")
            .with_tooltip("The short one"),
    ])
    .expect("distinct ids");
    r
}

/// Render `items` against [`wrapping_registry`] and report the rects.
///
/// Goes through the same `render_with_icons` the rest of this file uses, so a
/// change to the harness cannot make these three tests measure something the
/// others do not.
fn render_wrapping(items: impl IntoIterator<Item = Item>) -> Vec<(String, Rect)> {
    render_with_icons(items, &wrapping_registry(), &ConditionSet::new())
}

/// ★★★ **A Large control wraps its label instead of running on.**
///
/// The defect this pins is not subtle once it is drawn: `Save a compacted
/// copy of this document` laid out on one line is a control roughly 200 pt
/// wide and 56 pt tall — a letterbox with a small picture floating in the
/// middle of it, which is not what a Large control looks like in Word, in
/// Acrobat, or in the mockup. It also pushes every group to its right off the
/// band, so the first visible symptom is *"why is Print in the overflow
/// menu"*.
///
/// ★ The vacuity guard is the second assertion and it is doing real work.
/// Without it the test passes trivially against any implementation whose
/// labels happen to be short — including one that never wraps — because the
/// bound would never be approached. So the unwrapped width is measured too,
/// and the fixture is required to be a case that actually needs wrapping.
#[test]
fn a_large_control_wraps_a_long_label_instead_of_running_on() {
    let drawn = render_wrapping([Item::command("a.long").sized(ItemSize::Large)]);
    let large = item_rect(&drawn, "a.long").expect("the large control drew");

    // What the same label would measure with no wrap, through the same font.
    let ctx = context();
    let mut unwrapped = 0.0_f32;
    let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
        unwrapped = super::measure::text_width(
            ui,
            "Save a compacted copy of this document",
            &egui::TextStyle::Button,
        );
    });
    assert!(
        unwrapped > super::sizing::LARGE_LABEL_WRAP,
        "the fixture label measures {unwrapped} pt unwrapped, which already fits \
         inside the {} pt wrap width — so the assertion below would hold against an \
         implementation that never wraps anything, and this test would be measuring \
         nothing",
        super::sizing::LARGE_LABEL_WRAP
    );

    let ceiling = super::sizing::LARGE_LABEL_WRAP + super::sizing::LARGE_SIDE_PADDING * 2.0;
    assert!(
        large.width() <= ceiling + super::width_tests::SLACK,
        "a Large control with a long label drew {} pt wide, against a ceiling of \
         {ceiling} pt ({} of wrapped label plus {} of padding each side). The label \
         ran on instead of wrapping, and the control is a letterbox",
        large.width(),
        super::sizing::LARGE_LABEL_WRAP,
        super::sizing::LARGE_SIDE_PADDING
    );
}

/// ★ **…and a short-labelled one does not collapse below the floor.**
///
/// `.rb.big { min-width: 52px }`. Without it a run of Large controls is a
/// ragged fence — `New` measures `max(24 pt glyph, 21 pt label) + 16 = 40`,
/// `Open…` measures rather more — and a row of buttons of visibly unequal
/// width is the thing a ribbon is not.
///
/// The pair with the test above is the point: one asserts a ceiling, the
/// other a floor, and an implementation that satisfied only one of them would
/// be broken in a way the other could not see.
#[test]
fn a_large_control_never_narrows_below_the_mockups_floor() {
    let drawn = render_wrapping([Item::command("a.short").sized(ItemSize::Large)]);
    let large = item_rect(&drawn, "a.short").expect("the large control drew");
    assert!(
        large.width() >= super::sizing::LARGE_MIN_WIDTH - super::width_tests::SLACK,
        "a Large control with a short label drew {} pt wide, under the {} pt floor \
         the mockup pins. A band of Large controls whose widths track their labels \
         reads as a ragged fence",
        large.width(),
        super::sizing::LARGE_MIN_WIDTH
    );
}

/// ★★ **A Large control is SHORTER than the band's row area, not equal to it.**
///
/// The mockup draws `.rb.big` at 56 px inside a 68 px row area, top-aligned
/// by `.grp .items { align-items: flex-start }`. Until 2026-09-04 a Large
/// control simply *was* the row area, and the difference is visible the
/// moment a group holds nothing else: Pages ▸ Clipboard is three Large
/// controls, and three full-height plates side by side read as one block of
/// chrome rather than as three buttons.
///
/// ★ Asserted as a **relationship between the two metrics and the drawn
/// rect**, not against 56. A literal would pass under `Quiet` and say nothing
/// about `Airy`, whose own pair is 64 in 84.
#[test]
fn a_large_control_is_shorter_than_the_row_area_it_sits_in() {
    let ctx = context();
    let m = crate::theme::Theme::of(&ctx).metrics;
    assert!(
        m.ribbon_large_pts < m.ribbon_rows,
        "this preset's Large control ({} pt) is not shorter than its row area ({} \
         pt), so the assertion below is vacuous",
        m.ribbon_large_pts,
        m.ribbon_rows
    );

    let drawn = render_wrapping([Item::command("a.short").sized(ItemSize::Large)]);
    let large = item_rect(&drawn, "a.short").expect("the large control drew");
    assert!(
        (large.height() - m.ribbon_large_pts).abs() <= super::width_tests::SLACK,
        "a Large control drew {} pt tall against the {} pt the theme states. If it \
         drew {} pt it is still spanning the whole row area, which is the shape the \
         mockup replaced",
        large.height(),
        m.ribbon_large_pts,
        m.ribbon_rows
    );
}
