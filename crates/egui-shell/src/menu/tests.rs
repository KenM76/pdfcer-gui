//! Behavioural tests: a real `egui::Context`, a real right-click, a real
//! popup — and assertions about what came back.
//!
//! # Why these are not unit tests of [`super::plan`]
//!
//! [`super::plan`] already asserts the *rules* — unregistered is absent,
//! disabled is greyed, nothing enabled means no menu — on pure values,
//! exhaustively, with no window. Those tests are the ones that will still
//! be here in five years.
//!
//! What they cannot say is whether the renderer **obeys** them. "The menu
//! does not open" is a claim about `egui`'s popup memory, and the only
//! honest way to check it is to right-click something and ask `egui`
//! whether a popup is open. That is what this file does: it drives frames
//! with synthetic pointer events and reads back three things the
//! application would read —
//!
//! 1. **`egui::Popup::is_id_open`** — did a menu appear at all;
//! 2. **the reported rectangles** ([`super::report`]) — which rows were
//!    drawn, and where;
//! 3. **`platform_output.events`** — what a screen reader was told, which
//!    is the only way to observe [`super::a11y`] end to end.
//!
//! # The frame model these tests rely on
//!
//! `egui` hit-tests against the widget rectangles of the **previous**
//! pass. So every interaction here takes at least two frames, and the
//! sequence is always the same:
//!
//! | Frame | What happens |
//! |---|---|
//! | 1 | the target widget is laid out; nothing is clicked |
//! | 2 | a secondary click at the target opens the menu and draws its rows |
//! | 3 | a primary click at a row's centre invokes it |
//!
//! Written out because a test that silently needed a fourth frame would
//! look like a broken assertion rather than like a frame-ordering
//! mistake.

use egui::{Event, Modifiers, PointerButton, Pos2, Rect, vec2};

use crate::commands::{Command, CommandRegistry, ConditionSet, HandlerToken};
use crate::manifest::Item;
use crate::menu::{ContextMenu, Menu, MenuCustomItem, Menus, Shortcuts};
use crate::ribbon::IconRequest;

use super::testfont;

/// The context id every test in this file right-clicks.
pub(super) const CONTEXT: &str = "canvas.object";

/// Handler tokens, so an assertion names a command rather than a number.
const CUT: HandlerToken = HandlerToken::new(1);
const COPY: HandlerToken = HandlerToken::new(2);
const PASTE: HandlerToken = HandlerToken::new(3);
const DELETE: HandlerToken = HandlerToken::new(4);
const SWATCH: HandlerToken = HandlerToken::new(99);

pub(super) fn registry() -> CommandRegistry {
    let mut r = CommandRegistry::new();
    r.register_all([
        Command::new("edit.cut", "Cut", CUT)
            .enabled_when("selection.any")
            .with_icon("cut")
            .with_tooltip("Move the selection to the clipboard"),
        Command::new("edit.copy", "Copy", COPY).enabled_when("selection.any"),
        Command::new("edit.paste", "Paste", PASTE)
            .enabled_when("clipboard.any")
            .with_tooltip("Nothing has been copied yet"),
        Command::new("edit.delete", "Delete", DELETE).enabled_when("selection.any"),
    ])
    .expect("distinct ids");
    r
}

/// The document under test: Cut/Copy, a rule, Delete, and a stale id the
/// build does not have.
pub(super) fn menus() -> Menus {
    Menus::new().with(Menu::new(CONTEXT).with_items([
        Item::command("edit.cut"),
        Item::command("edit.copy"),
        Item::command("edit.paste"),
        Item::Separator,
        Item::command("edit.rasterize"), // never registered anywhere
        Item::command("edit.delete"),
    ]))
}

pub(super) fn shortcuts() -> Shortcuts {
    Shortcuts::from_keymap(&crate::manifest::Keymap(
        [
            ("Ctrl+X".to_owned(), "edit.cut".to_owned()),
            ("Ctrl+C".to_owned(), "edit.copy".to_owned()),
            ("Del".to_owned(), "edit.delete".to_owned()),
        ]
        .into_iter()
        .collect(),
    ))
}

/// What one driven frame produced.
#[derive(Debug)]
pub(super) struct Frame {
    /// The rectangle of the right-clickable target.
    pub(super) target: Rect,
    /// The `egui::Id` the context menu's popup is stored under.
    pub(super) popup: egui::Id,
    /// Whether `egui` believes that popup is open at the end of the frame.
    pub(super) open: bool,
    /// The tokens the shell reported.
    pub(super) invoked: Vec<HandlerToken>,
    /// Every rectangle the menu published, by name.
    pub(super) rects: Vec<(String, Rect)>,
    /// The accessibility/output events `egui` emitted.
    pub(super) events: Vec<egui::output::OutputEvent>,
}

impl Default for Frame {
    /// `egui::Rect` and `egui::Id` have no `Default`, so the empty frame
    /// is spelled out. `Rect::NOTHING` rather than `Rect::ZERO`, because a
    /// frame whose target was never laid out should not look like a target
    /// at the origin.
    fn default() -> Self {
        Self {
            target: Rect::NOTHING,
            popup: egui::Id::new("menu-test-unset"),
            open: false,
            invoked: Vec::new(),
            rects: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl Frame {
    /// The rect published under a name, if the menu drew it.
    pub(super) fn rect(&self, name: &str) -> Option<Rect> {
        self.rects.iter().find(|(n, _)| n == name).map(|(_, r)| *r)
    }

    /// The names published, in order — a compact way to assert *which*
    /// rows exist without asserting where they are.
    pub(super) fn names(&self) -> Vec<&str> {
        self.rects.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// The label a click announced, if a click was announced.
    ///
    /// # ★ Why the *last* one
    ///
    /// One click produces **two** `OutputEvent::Clicked`. `egui::Button`
    /// publishes its own default info from inside `atom_ui` — the atoms
    /// flattened into text, which for a menu row reads `"Copy Ctrl+C"` —
    /// and the shell then publishes the real one
    /// ([`super::a11y::describe_item`]) immediately afterwards.
    ///
    /// The shell's is the one that counts, and not only by being second:
    /// `Response::widget_info` also calls `register_widget_info`, whose
    /// later value **replaces** the earlier one, so the accesskit node an
    /// assistive technology actually reads carries the shell's name. The
    /// duplicated output *event* is `egui`'s behaviour for any widget that
    /// refines its own info; the ribbon's band has it too.
    pub(super) fn announced(&self) -> Option<String> {
        self.events.iter().rev().find_map(|e| match e {
            egui::output::OutputEvent::Clicked(info) => info.label.clone(),
            _ => None,
        })
    }
}

/// Everything a driven frame is allowed to vary.
pub(super) struct Scene<'a> {
    menus: &'a Menus,
    registry: &'a CommandRegistry,
    conditions: &'a ConditionSet,
    shortcuts: Shortcuts,
    /// Called for every icon the menu asks to have painted.
    icon_keys: Vec<String>,
    /// Whether to supply a custom-row renderer, and what it returns.
    custom: Option<HandlerToken>,
}

impl<'a> Scene<'a> {
    pub(super) fn new(
        menus: &'a Menus,
        registry: &'a CommandRegistry,
        conditions: &'a ConditionSet,
    ) -> Self {
        Self {
            menus,
            registry,
            conditions,
            shortcuts: shortcuts(),
            icon_keys: Vec::new(),
            custom: None,
        }
    }

    /// Drive one frame with the given pointer events.
    pub(super) fn frame(&mut self, ctx: &egui::Context, events: Vec<Event>) -> Frame {
        let mut out = Frame::default();
        let mut rects: Vec<(String, Rect)> = Vec::new();
        let mut icon_keys: Vec<String> = Vec::new();
        let custom_token = self.custom;

        let mut input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(600.0, 400.0))),
            events,
            ..Default::default()
        };
        input.max_texture_side = Some(2048);

        let full = ctx.run_ui(input, |ui| {
            let target = ui.allocate_response(vec2(160.0, 48.0), egui::Sense::click());
            out.target = target.rect;
            out.popup = egui::Popup::default_response_id(&target);

            let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
            let mut painter = |_: &egui::Painter, req: &IconRequest<'_>| {
                icon_keys.push(req.key.to_owned());
            };
            let mut custom = |ui: &mut egui::Ui, item: &MenuCustomItem<'_>| {
                let r = ui.add(egui::Button::new(format!("[{}]", item.kind)));
                custom_token.filter(|_| r.clicked())
            };

            let mut menu = ContextMenu::new()
                .reporting_rects_to(&mut sink)
                .with_icon_painter(&mut painter)
                .with_shortcuts(self.shortcuts.clone());
            if custom_token.is_some() {
                menu = menu.with_custom_items(&mut custom);
            }
            out.invoked = menu.attach(&target, self.menus, self.registry, CONTEXT, self.conditions);
        });

        out.open = egui::Popup::is_id_open(ctx, out.popup);
        out.rects = rects;
        out.events = full.platform_output.events;
        self.icon_keys = icon_keys;
        out
    }
}

/// A context with the synthetic proportional face installed, so every
/// width in these tests is measured against real advances. See
/// [`super::testfont`] — without it every galley measures ~0 and the
/// menu's width plan would be satisfied by nothing at all.
pub(super) fn context() -> egui::Context {
    let ctx = egui::Context::default();
    testfont::install(&ctx);
    ctx
}

/// A secondary (right) click at `at`.
fn right_click(at: Pos2) -> Vec<Event> {
    click_events(at, PointerButton::Secondary)
}

/// A primary (left) click at `at`.
fn left_click(at: Pos2) -> Vec<Event> {
    click_events(at, PointerButton::Primary)
}

fn click_events(at: Pos2, button: PointerButton) -> Vec<Event> {
    vec![
        Event::PointerMoved(at),
        Event::PointerButton {
            pos: at,
            button,
            pressed: true,
            modifiers: Modifiers::default(),
        },
        Event::PointerButton {
            pos: at,
            button,
            pressed: false,
            modifiers: Modifiers::default(),
        },
    ]
}

/// Frame 1 (lay the target out), frame 2 (right-click it), frame 3 (let
/// the popup settle), and return what frame 3 drew.
///
/// # ★ Why the settle frame is not optional
///
/// An `egui::Area` that has never been shown runs a **sizing pass**: it is
/// laid out invisibly, at a provisional size, purely to measure its
/// content (`egui-0.35.0/src/containers/area.rs`, `Area::begin`). Two
/// consequences bite a test that skips it:
///
/// - the row rectangles reported on the opening frame are the sizing
///   pass's, so a click aimed at one of them lands somewhere else and
///   nothing is invoked;
/// - `cross_justify` is switched off during a sizing pass
///   (`egui-0.35.0/src/ui.rs`), so the rows are at their intrinsic widths
///   and the chord column is not yet where it will end up.
///
/// This is not a defect in the menu; it is how `egui` sizes any
/// auto-sizing area, and the operator never sees the pass because it is
/// painted invisibly. But a test that asserted against it would be
/// asserting against a frame that is never shown to anybody.
pub(super) fn open_menu(ctx: &egui::Context, scene: &mut Scene<'_>) -> Frame {
    let first = scene.frame(ctx, Vec::new());
    let opened = scene.frame(ctx, right_click(first.target.center()));
    assert!(
        opened.open || opened.rects.is_empty(),
        "the popup must be open by the frame the right-click was delivered"
    );
    scene.frame(ctx, Vec::new())
}

// =====================================================================
// The invariant the whole module exists for
// =====================================================================

/// **★ Right-clicking something with nothing to offer does nothing.**
///
/// Not "opens and closes", not "flashes", not "shows three greyed rows" —
/// nothing. The menu here is the real one; the only difference from the
/// test below is that no condition is set, so every command's predicate is
/// false.
///
/// This is the assertion that cannot be made in [`super::plan`], because
/// it is about `egui`'s popup memory rather than about a list of slots.
#[test]
fn right_clicking_something_with_nothing_to_offer_opens_no_menu() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new(); // nothing is true: every row disabled
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let frame = open_menu(&ctx, &mut scene);

    assert!(
        !frame.open,
        "a menu of nothing but greyed rows must never appear: it costs a click \
         to dismiss and teaches the operator that right-clicking here is useless"
    );
    assert!(
        frame.rects.is_empty(),
        "nothing was drawn: {:?}",
        frame.names()
    );
    assert!(frame.invoked.is_empty());
}

/// The mirror image: one enabled command is enough, and the menu appears.
///
/// Together with the test above this pins that the difference is the
/// *offer* and nothing else — same document, same registry, same
/// right-click, one condition flipped.
#[test]
fn right_clicking_something_with_one_enabled_command_opens_a_menu() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let frame = open_menu(&ctx, &mut scene);

    assert!(frame.open, "the menu must appear");
    let body = frame
        .rect("menu.body.canvas.object")
        .expect("the body must publish its rectangle");
    assert!(
        body.width() > 0.0 && body.height() > 0.0,
        "a menu with no area is a menu nobody can click: {body:?}"
    );
}

/// **★ An open menu closes when its offer evaporates.**
///
/// The half of decision 1 that is easy to leave out. `egui` remembers a
/// popup as open in memory, not by whether anyone drew it — so a renderer
/// that merely *stopped drawing* would leave the menu ready to reappear at
/// the old pointer position the moment the offer came back, with no
/// right-click behind it.
///
/// The scenario is ordinary: the menu is open, and the selection is
/// deleted by a keystroke.
#[test]
fn an_open_menu_closes_when_its_offer_evaporates() {
    let ctx = context();
    let menus = menus();
    let registry = registry();

    let selected = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &selected);
    let opened = open_menu(&ctx, &mut scene);
    assert!(opened.open, "precondition: the menu is open");

    // The selection goes away. No pointer event at all — this is a
    // keystroke elsewhere in the application.
    let nothing = ConditionSet::new();
    let mut scene = Scene::new(&menus, &registry, &nothing);
    let after = scene.frame(&ctx, Vec::new());

    assert!(
        !after.open,
        "the menu must be closed, not merely undrawn: an undrawn-but-open \
         popup springs back the next time it has something to say"
    );
}

/// A context with no menu defined behaves the same way: a right-click on a
/// surface the document says nothing about does nothing.
#[test]
fn right_clicking_a_context_with_no_menu_does_nothing() {
    let ctx = context();
    let menus = Menus::new(); // the document defines no menus at all
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let frame = open_menu(&ctx, &mut scene);
    assert!(!frame.open);
    assert!(frame.invoked.is_empty());
}

// =====================================================================
// What gets drawn
// =====================================================================

/// **★ Disabled is drawn; unregistered is not.**
///
/// The rendered half of [`super::plan`]'s rule 1. `edit.paste` is
/// registered and its predicate is false, so it is a row; `edit.rasterize`
/// is not registered at all, so there is no row — and no gap where one
/// would have been.
#[test]
fn a_disabled_command_is_drawn_and_an_unregistered_one_is_not() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let frame = open_menu(&ctx, &mut scene);
    let names = frame.names();

    assert!(
        names.contains(&"menu.item.canvas.object.edit.paste"),
        "a registered command whose predicate is false must still be a row: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.contains("rasterize")),
        "a command this build does not have must leave no row at all — a greyed \
         row for it would be a placeholder for a promise the build cannot keep: {names:?}"
    );
    for id in ["edit.cut", "edit.copy", "edit.delete"] {
        assert!(
            names.contains(&format!("menu.item.canvas.object.{id}").as_str()),
            "{id} is missing from {names:?}"
        );
    }
}

/// Every row publishes a rectangle with a positive area, so a harness can
/// assert legibility against something real.
#[test]
fn every_row_publishes_a_rectangle_with_area() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let frame = open_menu(&ctx, &mut scene);
    assert!(!frame.rects.is_empty());
    for (name, rect) in &frame.rects {
        assert!(
            rect.width() > 0.0 && rect.height() > 0.0,
            "`{name}` was published with no area: {rect:?}"
        );
    }
}

/// The icon painter is asked for exactly the rows that have an icon key,
/// through the **ribbon's own** [`IconRequest`] — one icon set, wired
/// once, serving both surfaces.
#[test]
fn the_icon_painter_is_asked_for_each_row_that_has_an_icon() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let frame = open_menu(&ctx, &mut scene);
    assert!(frame.open);
    assert_eq!(
        scene.icon_keys,
        ["cut"],
        "only `edit.cut` was registered with an icon key"
    );
}

/// **★★★ A painted icon slot publishes its rectangle, and a blank one does
/// not.**
///
/// The only signal a *driven* check has on this surface, and the reason it
/// exists is in [`super::report`]'s header: a menu row is justified to the
/// body width, so it measures the same whether its slot holds a glyph,
/// holds a blank, or does not exist. The QAT's trick — an icon-only
/// control is square and a text button is a word wide — has no menu
/// equivalent, so without this name a harness cannot tell an application
/// that wired an icon painter from one that did not. Which is exactly the
/// state this project shipped in, undetectably, for the whole life of the
/// menu engine.
///
/// The fixture is the mixed case on purpose: `edit.cut` has a key and the
/// other three do not, so the menu reserves a column and three rows draw a
/// blank into it. Publishing four names here would make the signal mean
/// "a slot was reserved" — true of a blank — and it would be worthless.
///
/// The containment assertion is the second half: the published rectangle
/// has to be *inside the row it belongs to*, or a check that clicked or
/// sampled it would be aiming at somewhere else on the menu.
#[test]
fn only_a_painted_icon_slot_publishes_a_rectangle() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let frame = open_menu(&ctx, &mut scene);
    let icons: Vec<&str> = frame
        .rects
        .iter()
        .filter(|(name, _)| name.starts_with("menu.icon."))
        .map(|(name, _)| name.as_str())
        .collect();
    assert_eq!(
        icons,
        ["menu.icon.canvas.object.edit.cut"],
        "exactly the rows whose glyph was painted publish a slot — Copy, Paste \
         and Delete reserve a blank and must publish nothing, or the name means \
         `a slot exists` and says nothing about whether anything draws"
    );

    let slot = frame
        .rects
        .iter()
        .find(|(name, _)| name == "menu.icon.canvas.object.edit.cut")
        .map(|(_, r)| *r)
        .expect("just asserted present");
    let row = frame
        .rects
        .iter()
        .find(|(name, _)| name == "menu.item.canvas.object.edit.cut")
        .map(|(_, r)| *r)
        .expect("every row publishes a rectangle");
    assert!(
        row.contains_rect(slot.shrink(0.5)),
        "the icon slot {slot:?} must lie inside its own row {row:?}"
    );
    assert!(
        slot.width() > 0.0 && slot.height() > 0.0,
        "a slot with no area is one nothing could have been drawn into: {slot:?}"
    );
}

// =====================================================================
// The seam
// =====================================================================

/// **★ Choosing a row reports a token and runs nothing.**
///
/// The seam, end to end: the shell hands back
/// [`crate::commands::HandlerToken`] and the application dispatches. There
/// is no handler anywhere in this crate for a test to have to stub.
#[test]
fn choosing_a_row_reports_its_token() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let opened = open_menu(&ctx, &mut scene);
    let row = opened
        .rect("menu.item.canvas.object.edit.copy")
        .expect("Copy is on offer");

    let chosen = scene.frame(&ctx, left_click(row.center()));
    assert_eq!(
        chosen.invoked,
        [COPY],
        "the operator chose Copy; the shell must report its token and nothing else"
    );
}

/// **A disabled row cannot be invoked, however hard it is clicked.**
///
/// `SHELL_FRAMEWORK.md` §5: *"predicates are safety, not decoration."* A
/// greyed row that still fired would make the predicate a suggestion.
#[test]
fn a_disabled_row_cannot_be_invoked() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    // `selection.any` enables Cut/Copy/Delete; `clipboard.any` stays
    // false, so Paste is drawn and disabled.
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let opened = open_menu(&ctx, &mut scene);
    let paste = opened
        .rect("menu.item.canvas.object.edit.paste")
        .expect("Paste is drawn, greyed");

    let clicked = scene.frame(&ctx, left_click(paste.center()));
    assert!(
        clicked.invoked.is_empty(),
        "a disabled row must report nothing; got {:?}",
        clicked.invoked
    );
}

/// A custom row is the application's, and reports through the **same**
/// channel as a command — so an application never grows a second dispatch
/// path for a control it drew itself.
#[test]
fn a_custom_row_reports_through_the_same_channel() {
    let ctx = context();
    let menus = Menus::new().with(
        Menu::new(CONTEXT).with_items([Item::command("edit.copy"), Item::custom("colour_swatch")]),
    );
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);
    scene.custom = Some(SWATCH);

    let opened = open_menu(&ctx, &mut scene);
    let swatch = opened
        .rect("menu.custom.canvas.object.colour_swatch")
        .expect("the custom row publishes its rectangle too");

    let clicked = scene.frame(&ctx, left_click(swatch.center()));
    assert_eq!(clicked.invoked, [SWATCH]);
}

/// **A menu whose only offer is a custom row still opens.**
///
/// The shell cannot evaluate an application's own control. Refusing to
/// open would silently delete a control the application asked for, which
/// is a worse failure than opening a menu that turns out to be useless.
#[test]
fn a_menu_whose_only_offer_is_a_custom_row_opens() {
    let ctx = context();
    let menus = Menus::new().with(Menu::new(CONTEXT).with_items([
        Item::command("edit.copy"), // registered, and disabled here
        Item::custom("colour_swatch"),
    ]));
    let registry = registry();
    let conditions = ConditionSet::new(); // nothing enabled
    let mut scene = Scene::new(&menus, &registry, &conditions);
    scene.custom = Some(SWATCH);

    let frame = open_menu(&ctx, &mut scene);
    assert!(frame.open);
}

// =====================================================================
// Accessibility, observed
// =====================================================================

/// **★ A screen reader hears the label *and* the chord.**
///
/// [`super::a11y`] argues from `egui` 0.35's source that the chord has
/// nowhere else to go. This is the end-to-end proof: click Copy and read
/// the `OutputEvent` `egui` actually emitted.
///
/// If this ever fails while `super::a11y`'s unit tests pass, the wiring
/// broke rather than the rule — which is exactly the distinction worth
/// being able to make.
#[test]
fn a_chosen_row_announces_its_label_and_its_chord() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);

    let opened = open_menu(&ctx, &mut scene);
    let row = opened
        .rect("menu.item.canvas.object.edit.copy")
        .expect("Copy is on offer");

    let chosen = scene.frame(&ctx, left_click(row.center()));
    assert_eq!(
        chosen.announced().as_deref(),
        Some("Copy, Ctrl+C"),
        "the announced name must carry both halves of what the row shows"
    );
}

/// A row with no binding announces its label alone — no trailing comma,
/// no invented chord.
#[test]
fn a_row_with_no_binding_announces_its_label_alone() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new()
        .with("selection.any")
        .with("clipboard.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);
    // Paste is deliberately unbound in `shortcuts()`.
    let opened = open_menu(&ctx, &mut scene);
    let row = opened
        .rect("menu.item.canvas.object.edit.paste")
        .expect("Paste is enabled here");

    let chosen = scene.frame(&ctx, left_click(row.center()));
    assert_eq!(chosen.announced().as_deref(), Some("Paste"));
}

// =====================================================================
// Customization, end to end
// =====================================================================

/// **★ An operator's customization reaches the screen.**
///
/// The claim `SHELL_FRAMEWORK.md` §1 makes for the whole design — *"the
/// shell is data … not code that has to be recompiled to change"* —
/// exercised through the surface rather than through the document: a RON
/// layer an operator could have typed, applied with
/// [`Menus::overlay`], and then right-clicked.
#[test]
fn a_customization_layer_changes_what_the_right_click_offers() {
    let ctx = context();
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");

    let mut menus = menus();
    menus.overlay(
        &Menus::from_ron(
            r#"[
                // "I never use Cut. Put Delete at the top."
                Menu(context: "canvas.object", items: [
                    Command(id: "edit.delete"),
                    Separator,
                    Command(id: "edit.copy"),
                ]),
            ]"#,
        )
        .expect("an operator's file parses"),
    );

    let mut scene = Scene::new(&menus, &registry, &conditions);
    let frame = open_menu(&ctx, &mut scene);
    let names = frame.names();

    assert!(
        !names.iter().any(|n| n.contains("edit.cut")),
        "the operator removed Cut: {names:?}"
    );
    let delete = frame
        .rect("menu.item.canvas.object.edit.delete")
        .expect("Delete survived");
    let copy = frame
        .rect("menu.item.canvas.object.edit.copy")
        .expect("Copy survived");
    assert!(
        delete.top() < copy.top(),
        "the operator put Delete first; the menu must be in their order, not \
         the built-in one"
    );
}

// =====================================================================
// The `show` entry point
// =====================================================================

/// [`Menu::show`] draws into a `Ui` the caller owns, and reports the same
/// way.
///
/// Its documented weakness is asserted here too: it cannot decline to
/// open, because by the time it runs the popup exists. What it *can* do is
/// draw nothing, which is what an application that skips
/// [`Menu::would_open`] gets.
#[test]
fn show_draws_into_a_caller_owned_ui_and_declines_to_draw_an_empty_menu() {
    let ctx = context();
    let menus = menus();
    let registry = registry();

    for (conditions, expect_rows) in [
        (ConditionSet::new().with("selection.any"), true),
        (ConditionSet::new(), false),
    ] {
        let mut rects: Vec<(String, Rect)> = Vec::new();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(600.0, 400.0))),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let mut sink = |name: &str, rect: Rect| rects.push((name.to_owned(), rect));
            let invoked = ContextMenu::new().reporting_rects_to(&mut sink).render(
                ui,
                &menus,
                &registry,
                CONTEXT,
                &conditions,
            );
            assert!(invoked.is_empty(), "nothing was clicked");
        });
        assert_eq!(
            !rects.is_empty(),
            expect_rows,
            "with conditions {conditions:?} the body should{} have drawn; got {rects:?}",
            if expect_rows { "" } else { " not" }
        );
    }
}

/// [`Menu::would_open`] answers the same question the renderer asks
/// itself, so an application can use it to decide whether to draw a "⋯"
/// affordance beside a row.
#[test]
fn would_open_agrees_with_what_the_renderer_does() {
    let ctx = context();
    let menus = menus();
    let registry = registry();

    for conditions in [
        ConditionSet::new(),
        ConditionSet::new().with("selection.any"),
        ConditionSet::new().with("clipboard.any"),
    ] {
        let predicted = Menu::would_open(&menus, &registry, CONTEXT, &conditions);
        let mut scene = Scene::new(&menus, &registry, &conditions);
        let actual = open_menu(&ctx, &mut scene).open;
        assert_eq!(
            predicted, actual,
            "`would_open` said {predicted} and the renderer did {actual} for {conditions:?}"
        );
        // A fresh context per iteration, so a popup left open by the
        // previous case cannot make the next one look right.
        egui::Popup::close_all(&ctx);
    }
}

/// The document, the registry and the theme are all that a menu needs: no
/// application state, no window, and no icon set. Asserted by the fact
/// that every test above runs headless — this one just states it.
#[test]
fn a_menu_needs_no_application_at_all() {
    let ctx = context();
    let menus = menus();
    let registry = registry();
    let conditions = ConditionSet::new().with("selection.any");
    let mut scene = Scene::new(&menus, &registry, &conditions);
    scene.shortcuts = Shortcuts::none();

    let frame = open_menu(&ctx, &mut scene);
    assert!(
        frame.open,
        "no keymap, no icons, no application — still a menu"
    );
    assert!(frame.rect("menu.body.canvas.object").is_some());
}
