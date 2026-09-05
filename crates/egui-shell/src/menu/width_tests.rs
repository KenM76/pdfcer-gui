//! Layout tests against **real** font metrics.
//!
//! # ★ Why this file is separate, and why it would otherwise be worthless
//!
//! `egui-shell` depends on `egui` with `default-features = false`, so in a
//! `cargo test -p egui-shell --lib` build there is **no font data at all**
//! and every galley measures ≈ 0 × 0. Every width assertion about a menu —
//! that the chord column is separated, that a short menu still reaches its
//! floor, that a long label clamps — would then be satisfied by text that
//! occupies no space.
//!
//! Worse, the failure is conditional on who is building:
//!
//! | Command | What these tests would get |
//! |---|---|
//! | `cargo test -p egui-shell --lib` | `egui` alone → no fonts → zero widths |
//! | `cargo test --workspace` | a sibling pulls `egui/default_fonts` → **real** widths |
//!
//! The full account is in [`super::testfont`] and in
//! `D:/dev/rag/rust/a_crate_tested_alone_and_in_a_workspace_gets_different_features_so_layout_tests_can_be_vacuous.md`.
//! Every test here installs the synthetic proportional face, and
//! `testfont::install` asserts that it took effect — so a font that failed
//! to load fails the suite rather than quietly restoring the vacuum.
//!
//! # ★ The harness: a justified scope, not a popup
//!
//! Most tests here render into a `Ui` built with
//! `Layout::top_down_justified(Align::Min)` rather than through a real
//! right-click. That layout is **not a convenience** — it is the layout
//! `egui::Popup::menu` installs
//! (`egui-0.35.0/src/containers/popup.rs`), and it is the reason the
//! chord column lines up at all: `Atom::grow` right-aligns *within a
//! button*, so two columns only exist if every button is the same width.
//!
//! Using it directly buys determinism. A real popup runs an invisible
//! **sizing pass** first, during which `egui` switches `cross_justify`
//! off (`egui-0.35.0/src/ui.rs`), so the first frame's geometry is
//! deliberately not the geometry anybody sees. A geometry test that
//! measured it would be measuring a frame that is never displayed.
//!
//! `the_popup_path_justifies_every_row_to_the_same_width` closes the loop
//! by asserting the same property through the real right-click, so the
//! shortcut cannot hide a difference between the harness and the shell.

use egui::{Align, Layout, Rect, UiBuilder, vec2};

use crate::commands::{Command, CommandRegistry, ConditionSet, HandlerToken};
use crate::manifest::{Item, Keymap};
use crate::menu::{ContextMenu, Menus, Shortcuts};
use crate::ribbon::measure::{button_padding, text_width};
use crate::theme::Theme;

use super::plan;
use super::testfont;
use super::tests;

/// The context id these tests render.
const CONTEXT: &str = "canvas.object";

/// A context with the synthetic face installed and proven to measure.
fn context() -> egui::Context {
    let ctx = egui::Context::default();
    testfont::install(&ctx);
    ctx
}

/// Build a registry from `(id, label, icon?)`, all commands enabled.
fn registry(rows: &[(&str, &str, bool)]) -> CommandRegistry {
    let mut r = CommandRegistry::new();
    for (n, (id, label, icon)) in rows.iter().enumerate() {
        let mut c = Command::new(*id, *label, HandlerToken::new(n as u64 + 1));
        if *icon {
            c = c.with_icon("glyph");
        }
        r.register(c).expect("distinct ids");
    }
    r
}

/// A one-menu document naming every command in `rows`, in order.
fn document(rows: &[(&str, &str, bool)]) -> Menus {
    Menus::new().with(
        super::Menu::new(CONTEXT).with_items(rows.iter().map(|(id, _, _)| Item::command(*id))),
    )
}

fn shortcuts(pairs: &[(&str, &str)]) -> Shortcuts {
    Shortcuts::from_keymap(&Keymap(
        pairs
            .iter()
            .map(|(chord, id)| ((*chord).to_owned(), (*id).to_owned()))
            .collect(),
    ))
}

/// What one rendered menu body looked like.
#[derive(Debug)]
struct Drawn {
    /// The body's rectangle, as published under `menu.body.<context>`.
    body: Rect,
    /// Every command row, in draw order, as `(command id, rect)`.
    rows: Vec<(String, Rect)>,
    /// The width the plan asked for, recomputed inside the same `Ui` from
    /// the same font metrics — so a disagreement between plan and render
    /// is visible rather than inferred.
    planned: plan::BodyWidth,
}

impl Drawn {
    fn row(&self, command_id: &str) -> Rect {
        self.rows
            .iter()
            .find(|(id, _)| id == command_id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("no row for `{command_id}` in {:?}", self.rows))
    }
}

/// Render a menu into a justified scope of `available` width and report
/// what it drew.
fn draw(
    ctx: &egui::Context,
    rows: &[(&str, &str, bool)],
    chords: &[(&str, &str)],
    available: f32,
) -> Drawn {
    let registry = registry(rows);
    let menus = document(rows);
    let shortcuts = shortcuts(chords);
    let conditions = ConditionSet::new();

    let mut published: Vec<(String, Rect)> = Vec::new();
    let mut planned = plan::BodyWidth {
        points: 0.0,
        truncating: false,
    };

    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(1200.0, 800.0))),
        ..Default::default()
    };
    let _ = ctx.run_ui(input, |ui| {
        // Recompute the plan's answer here, where the fonts are, using the
        // same three inputs the renderer uses.
        let theme = Theme::of(ui.ctx());
        let atom_gap = ui.spacing().icon_spacing;
        let padding = button_padding(ui);
        // ★ The icon column is decided ONCE for the menu, exactly as the
        // renderer decides it (`plan::reserves_icon_column`) — so a row
        // with no key in a menu that has one is measured *with* its blank
        // slot. Mirroring the old per-command rule here would make this
        // harness agree with a renderer that no longer exists, which is
        // the worst kind of green.
        let reserved = rows.iter().any(|(_, _, icon)| *icon);
        let totals: Vec<f32> = rows
            .iter()
            .map(|(id, label, icon)| {
                plan::RowWidths {
                    icon: if plan::icon_slot(reserved, *icon).is_reserved() {
                        theme.metrics.icon_pts
                    } else {
                        0.0
                    },
                    label: text_width(ui, label, &egui::TextStyle::Button),
                    shortcut: shortcuts
                        .get(id)
                        .map(|c| text_width(ui, c, &egui::TextStyle::Button))
                        .unwrap_or(0.0),
                }
                .total(atom_gap, padding)
            })
            .collect();
        planned = plan::body_width(&totals);

        let scope = UiBuilder::new()
            .max_rect(Rect::from_min_size(
                ui.max_rect().min,
                vec2(available, 600.0),
            ))
            .layout(Layout::top_down_justified(Align::Min));
        ui.scope_builder(scope, |ui| {
            let mut sink = |name: &str, rect: Rect| published.push((name.to_owned(), rect));
            let invoked = ContextMenu::new()
                .reporting_rects_to(&mut sink)
                .with_shortcuts(shortcuts.clone())
                .render(ui, &menus, &registry, CONTEXT, &conditions);
            assert!(invoked.is_empty(), "nothing was clicked");
        });
    });

    let prefix = format!("menu.item.{CONTEXT}.");
    Drawn {
        body: published
            .iter()
            .find(|(n, _)| n == &format!("menu.body.{CONTEXT}"))
            .map(|(_, r)| *r)
            .expect("the body publishes its rectangle"),
        rows: published
            .iter()
            .filter_map(|(n, r)| n.strip_prefix(&prefix).map(|id| (id.to_owned(), *r)))
            .collect(),
        planned,
    }
}

// =====================================================================

/// **★ Every row is the same width, which is what makes a chord column a
/// column.**
///
/// `Atom::grow` right-aligns the chord *inside its own button*. If the
/// buttons were not all the same width, each chord would be flush with a
/// different right edge and the "column" would be a ragged diagonal —
/// which looks like a rendering fault and is the reason
/// `set_min_width` is called before a single row is drawn.
#[test]
fn every_row_is_justified_to_the_same_width() {
    let ctx = context();
    let rows = [
        ("edit.cut", "Cut", true),
        ("edit.copy", "Copy", false),
        ("edit.paste", "Paste a much longer label", false),
        ("edit.delete", "Delete", false),
    ];
    let drawn = draw(
        &ctx,
        &rows,
        &[("Ctrl+X", "edit.cut"), ("Del", "edit.delete")],
        600.0,
    );

    let widths: Vec<f32> = drawn.rows.iter().map(|(_, r)| r.width()).collect();
    let first = widths[0];
    for (id, rect) in &drawn.rows {
        assert!(
            (rect.width() - first).abs() < 0.5,
            "`{id}` is {} wide but the first row is {first} — a ragged menu has no \
             chord column: {widths:?}",
            rect.width()
        );
    }
}

/// **★ The chord column is really reserved: the same menu is wider when
/// its commands have chords.**
///
/// The direct consequence of [`plan::COLUMN_GAP`] and of charging for the
/// chord's own text. With no font data this test is vacuous — both menus
/// measure the floor — which is precisely why [`testfont`] exists.
#[test]
fn a_menu_with_chords_is_wider_than_the_same_menu_without() {
    let ctx = context();
    let rows = [
        ("edit.cut", "Cut something reasonably long", false),
        ("edit.copy", "Copy something reasonably long", false),
    ];

    let bare = draw(&ctx, &rows, &[], 600.0);
    let chorded = draw(
        &ctx,
        &rows,
        &[("Ctrl+X", "edit.cut"), ("Ctrl+C", "edit.copy")],
        600.0,
    );

    assert!(
        chorded.body.width() > bare.body.width() + plan::COLUMN_GAP,
        "a chorded menu must pay for the chord text *and* the column gap: \
         bare {} vs chorded {}",
        bare.body.width(),
        chorded.body.width()
    );
}

/// **★ The rendered width is the width the plan asked for.**
///
/// The plan is pure arithmetic and the renderer is `egui`; they agree only
/// because the renderer measures with the plan's own inputs. This is the
/// tripwire on that: if a future edit changes the atom order, adds a
/// spacer, or measures the chord in a different `TextStyle`, the two
/// numbers separate and this fails — where otherwise the only symptom
/// would be a chord column that is slightly too tight, in one theme,
/// noticed by nobody.
#[test]
fn the_rendered_body_is_the_width_the_plan_computed() {
    let ctx = context();
    for rows in [
        &[("a.x", "Cut", false), ("a.y", "Copy", false)][..],
        &[("a.x", "Cut", true), ("a.y", "Copy", true)][..],
        &[("a.x", "A very much longer command label indeed", false)][..],
    ] {
        let chords: Vec<(&str, &str)> = rows
            .iter()
            .enumerate()
            .map(|(i, (id, _, _))| (["Ctrl+X", "Ctrl+Shift+C", "F11"][i % 3], *id))
            .collect();
        let drawn = draw(&ctx, rows, &chords, 600.0);
        assert!(
            (drawn.body.width() - drawn.planned.points).abs() < 1.0,
            "the plan asked for {} and the body came out {}: rows {rows:?}",
            drawn.planned.points,
            drawn.body.width()
        );
    }
}

/// A menu of two short verbs still reaches the floor, so it reads as a
/// menu rather than as a clickable tooltip.
#[test]
fn a_tiny_menu_still_reaches_the_floor() {
    let ctx = context();
    let drawn = draw(
        &ctx,
        &[("a.x", "Go", false), ("a.y", "Up", false)],
        &[],
        600.0,
    );
    assert!(
        drawn.body.width() >= plan::MIN_BODY_WIDTH - 0.5,
        "a two-verb menu came out {} wide, below the {} floor",
        drawn.body.width(),
        plan::MIN_BODY_WIDTH
    );
    assert!(!drawn.planned.truncating);
}

/// **A pathological label clamps rather than producing a banner.**
///
/// Past [`plan::MAX_BODY_WIDTH`] the label gives way, not the position —
/// the same trade the ribbon's overflow affordance makes, and for the same
/// reason: characters are recoverable (the tooltip has the full text),
/// position is not.
#[test]
fn a_pathological_label_clamps_and_says_so() {
    let ctx = context();
    let long = "Delete every annotation on every page of this document, without asking again";
    let drawn = draw(
        &ctx,
        &[("a.x", long, false), ("a.y", "Copy", false)],
        &[],
        900.0,
    );

    assert!(
        drawn.planned.truncating,
        "a {long:?} label must trip the clamp, or the constant is meaningless \
         against real metrics"
    );
    assert!(
        drawn.body.width() <= plan::MAX_BODY_WIDTH + 0.5,
        "the body must be clamped to {}, got {}",
        plan::MAX_BODY_WIDTH,
        drawn.body.width()
    );
    // And the rows are still inside it — a clamped menu whose rows spill
    // out of it would be the clamp making things worse.
    for (id, rect) in &drawn.rows {
        assert!(
            rect.width() <= drawn.body.width() + 0.5,
            "`{id}` ({}) is wider than the clamped body ({})",
            rect.width(),
            drawn.body.width()
        );
    }
}

/// A row with an icon is wider than the same row without one — the icon
/// slot is really reserved rather than drawn on top of the label.
#[test]
fn an_icon_slot_is_reserved_rather_than_overlaid() {
    let ctx = context();
    let bare = draw(
        &ctx,
        &[("a.x", "Cut the current selection", false)],
        &[],
        600.0,
    );
    let iconed = draw(
        &ctx,
        &[("a.x", "Cut the current selection", true)],
        &[],
        600.0,
    );
    assert!(
        iconed.planned.points > bare.planned.points,
        "an icon must cost width: {} vs {}",
        bare.planned.points,
        iconed.planned.points
    );
}

/// **★★★ A menu with one icon costs the same as a menu with all icons,
/// and a menu with none costs nothing.**
///
/// The rendered proof of the icon-column rule, against real font metrics.
/// Three menus, identical labels and chords, differing only in which rows
/// name a glyph:
///
/// | Menu | Every row's slot | Expected body |
/// |---|---|---|
/// | all four have icons | `Glyph` | the reference |
/// | one has an icon | `Glyph`, then three `Blank` | **the same** |
/// | none has an icon | `Absent` | strictly narrower |
///
/// The first two being **equal** is the whole rule: a blank slot costs
/// what a glyph costs, so the label column starts at one x whatever the
/// mix. If the renderer had kept the per-command rule the middle menu
/// would come out narrower than the first and its labels would zig-zag —
/// and no width assertion that looked at one menu at a time would notice.
///
/// The third being narrower is the other half, and it is what keeps the
/// rule cheap: a menu whose commands have no icons is laid out exactly as
/// it was before the column existed.
#[test]
fn one_icon_costs_a_menu_the_same_column_as_four() {
    let ctx = context();
    // ★ The LONGEST label belongs to a row with NO icon, and that is the
    // whole design of this fixture. The body width is set by the widest
    // row, so if the widest row were the one carrying the glyph the
    // equality below would hold under the wrong rule too — a per-command
    // renderer would give that row its slot either way and the menu would
    // measure the same. Putting the long label on a bare row makes the
    // blank slot the thing being measured.
    let labels = [
        "Cut",
        "Copy",
        "Paste a very much longer command label indeed",
        "Delete",
    ];
    let ids = ["a.1", "a.2", "a.3", "a.4"];
    let build = |icons: [bool; 4]| -> Vec<(&'static str, &'static str, bool)> {
        (0..4).map(|i| (ids[i], labels[i], icons[i])).collect()
    };
    let chords = [("Ctrl+X", "a.1"), ("Del", "a.4")];

    let all = draw(&ctx, &build([true; 4]), &chords, 600.0);
    let one = draw(&ctx, &build([true, false, false, false]), &chords, 600.0);
    let none = draw(&ctx, &build([false; 4]), &chords, 600.0);

    assert!(
        (all.body.width() - one.body.width()).abs() < 0.5,
        "a menu where one row has a glyph reserves the same column as one where \
         every row does — all {} vs one {}",
        all.body.width(),
        one.body.width()
    );
    assert!(
        none.body.width() < one.body.width() - 1.0,
        "a menu with no icons must not pay for a column it has no use for: \
         none {} vs one {}",
        none.body.width(),
        one.body.width()
    );

    // ★ And the column costs no HEIGHT. The slot is `icon_pts` square and
    // the row is `control_height` tall, so a menu that gained a column
    // must not have grown taller — a taller menu is how "we added an icon
    // column" turns into "the menus all got bigger", which nobody asks for
    // and everybody notices.
    let row_height = |d: &Drawn| d.rows.first().expect("a row").1.height();
    assert!(
        (row_height(&one) - row_height(&none)).abs() < 0.5,
        "gaining an icon column changed the row height: {} with a column, {} without",
        row_height(&one),
        row_height(&none)
    );
    for (id, rect) in &one.rows {
        assert!(
            (rect.height() - row_height(&one)).abs() < 0.5,
            "`{id}` is {} tall against the first row's {}: a menu whose rows are \
             different heights reads as a rendering fault",
            rect.height(),
            row_height(&one)
        );
    }
}

/// Rows stack downwards in document order and do not overlap.
///
/// Obvious, and worth pinning: a menu whose rows overlapped would still
/// pass every width assertion above.
#[test]
fn rows_stack_in_document_order_without_overlapping() {
    let ctx = context();
    let rows = [
        ("a.1", "One", false),
        ("a.2", "Two", false),
        ("a.3", "Three", false),
    ];
    let drawn = draw(&ctx, &rows, &[], 600.0);
    assert_eq!(drawn.rows.len(), 3);

    let one = drawn.row("a.1");
    let two = drawn.row("a.2");
    let three = drawn.row("a.3");
    assert!(one.bottom() <= two.top() + 0.5, "{one:?} then {two:?}");
    assert!(two.bottom() <= three.top() + 0.5, "{two:?} then {three:?}");
    assert!(
        drawn.body.contains_rect(three.shrink(0.5)),
        "the last row must be inside the published body: {three:?} vs {:?}",
        drawn.body
    );
}

/// **★ The real popup path justifies every row too.**
///
/// The harness above installs the popup's layout by hand for
/// determinism. This asserts the same property through an actual
/// right-click, so the shortcut cannot be hiding a difference between the
/// harness and the shell — the failure mode that makes a green geometry
/// suite worthless.
#[test]
fn the_popup_path_justifies_every_row_to_the_same_width() {
    let ctx = tests::context();
    let menus = tests::menus();
    let registry = tests::registry();
    let conditions = ConditionSet::new()
        .with("selection.any")
        .with("clipboard.any");
    let mut scene = tests::Scene::new(&menus, &registry, &conditions);

    let frame = tests::open_menu(&ctx, &mut scene);
    assert!(frame.open, "the menu must be open to have any geometry");

    let prefix = format!("menu.item.{CONTEXT}.");
    let rows: Vec<(&str, Rect)> = frame
        .rects
        .iter()
        .filter(|(n, _)| n.starts_with(&prefix))
        .map(|(n, r)| (n.as_str(), *r))
        .collect();
    assert!(rows.len() >= 3, "expected several rows, got {rows:?}");

    let first = rows[0].1.width();
    for (name, rect) in &rows {
        assert!(
            (rect.width() - first).abs() < 0.5,
            "`{name}` is {} wide but the first row is {first}",
            rect.width()
        );
    }

    let body = frame
        .rect(&format!("menu.body.{CONTEXT}"))
        .expect("the body publishes its rectangle");
    assert!(
        body.width() >= first - 0.5,
        "the body ({}) must be at least as wide as its rows ({first})",
        body.width()
    );
}
