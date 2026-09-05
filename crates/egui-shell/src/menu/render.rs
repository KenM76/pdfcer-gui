//! Drawing a context menu — and the two decisions that make it a menu
//! rather than a popup with buttons in it.
//!
//! # ★ THE SEAM: the shell reports intent; the application dispatches
//!
//! Identical to [`crate::ribbon::render`]'s, deliberately and to the
//! letter. [`Menu::show`] and [`Menu::attach`] return
//! `Vec<HandlerToken>` — the tokens of the commands the operator chose.
//! **They run nothing.** There is no handler in this crate and no path by
//! which drawing a menu can change an application's document.
//!
//! ```no_run
//! # use egui_shell::{CommandRegistry, ConditionSet, Shell, menu::Menu};
//! # fn dispatch(_: egui_shell::HandlerToken) {}
//! # fn on_canvas(response: &egui::Response, shell: &Shell,
//! #              registry: &CommandRegistry, conditions: &ConditionSet) {
//! for token in Menu::attach(response, shell, registry, "canvas.object", conditions) {
//!     dispatch(token); // ← the application's one choke point, the same one the ribbon uses
//! }
//! # }
//! ```
//!
//! The payoff is concrete here: `RIBBON_IA.md` §5 says the context menu
//! *"carries the same commands again"* as the Format tab. Because both
//! surfaces return tokens into the same `match`, "the same commands" is
//! literally true — the confirmation gate, the undo entry and the "this
//! document is encrypted" refusal are written once and cover both. A
//! registry of closures would have made the menu a second set of call
//! sites, and the fifth one somebody added would be the one that forgot
//! the undo entry.
//!
//! # ★ Decision 1: a menu with nothing to offer is never opened
//!
//! Not "opened and then closed", and not "opened showing five greyed
//! rows". [`Menu::attach`] resolves the menu **before** it asks `egui` for
//! a popup, and if [`plan::offers_anything`] says no it does two things:
//!
//! 1. never calls `Popup::show`, so the right-click does nothing at all;
//! 2. **closes the popup id if it is open** — because a menu whose offer
//!    evaporates while it is on screen (the selection is deleted by a
//!    keystroke, say) must vanish, and `egui` remembers a popup as open in
//!    memory even on a frame nobody drew it. Without step 2 the menu would
//!    reappear, at the old pointer position, the moment the offer came
//!    back.
//!
//! The reasoning for the rule itself is in [`plan`]'s header. What lives
//! here is the enforcement, and the enforcement is *ordering*: the
//! decision is taken before `egui` is involved, so there is no state for a
//! later branch to get wrong.
//!
//! # ★ Decision 2: the menu measures itself before it draws
//!
//! A menu row is `[icon?] [label] [grow] [chord]`. The `grow` atom is what
//! right-aligns the chord, and it right-aligns it *within the button*, so
//! the two columns only line up if every button is the same width — which
//! in a `top_down_justified` popup means the popup's width has to be
//! decided first.
//!
//! Left to itself `egui` sizes the popup to the widest row's **intrinsic**
//! width, at which the widest row's label and chord are separated by one
//! 4 pt atom gap and read as one run of text. So the body computes its own
//! width from real font metrics ([`plan::RowWidths`]), adds
//! [`plan::COLUMN_GAP`], and calls `ui.set_min_width`. `egui`'s
//! `Placer::set_min_width` expands the `Ui`'s **`max_rect`** as well as its
//! `min_rect` (`egui-0.35.0/src/placer.rs`), so the justified rows pick the
//! new width up in the same pass rather than a frame later.
//!
//! # ★ Decision 3: the icon column belongs to the menu, the glyph to the
//! command
//!
//! Added 2026-09-04. A menu either has an icon column or it does not, and
//! the answer is one property of the whole list rather than of each row:
//! [`plan::reserves_icon_column`] is true iff *some* surviving command
//! names an icon key. Every command row in a reserving menu then lays out
//! a slot — [`plan::IconSlot::Glyph`] for the ones with a key,
//! [`plan::IconSlot::Blank`] for the ones without — so the labels start at
//! one x and the eye can scan them.
//!
//! **A blank slot paints nothing.** Not a placeholder box, not a dimmed
//! outline, not a "?" — the painter is simply never called for that row.
//! An indent is not a hole; it is the same left margin its neighbour has.
//!
//! This decision must be taken **before** the width is measured, because
//! a `Blank` slot costs exactly what a `Glyph` slot costs and the widest
//! row is what sets the body width. [`plan::RowWidths::icon`] spells out
//! the failure the other ordering produces.
//!
//! Nothing changes for a menu whose commands have no icons at all — no
//! slot, no indent, no extra width — which is what makes the rule safe to
//! apply to every menu unconditionally.
//!
//! # What is *not* here
//!
//! **Submenus.** A nested menu is a second popup, a hover-intent timer and
//! a keyboard model of its own, and none of it is needed by any menu
//! `RIBBON_IA.md` §6 describes — those are flat lists of six to ten verbs.
//! Adding it speculatively would be the item vocabulary growing a variant
//! for a case nobody has.
//!
//! **Chord *handling*.** The menu draws the chord the keymap binds; it
//! does not consume input for it. Chord dispatch belongs with the
//! application's input pass, which is the only party that knows what has
//! focus — the same boundary [`crate::ribbon`] draws.

use egui::{Atoms, RichText, TextStyle, UiKind, Vec2, vec2};

use crate::commands::{CommandRegistry, ConditionSet, HandlerToken};
use crate::ribbon::measure::{button_padding, text_width};
use crate::ribbon::report::{RectSink, Reporter};
use crate::ribbon::{IconPainter, IconRequest};
use crate::theme::Theme;

use super::ctx::{Ctx, MenuCustomItem, MenuCustomRenderer};
use super::model::{Menu, MenuLookup};
use super::plan::Slot;
use super::shortcut::Shortcuts;
use super::{a11y, plan, report};

/// Draws a context menu. See this module's header for the seam it sits on.
///
/// The plain entry points are [`Menu::attach`] (the one an application
/// wants) and [`Menu::show`] (for an application that owns its own popup).
/// This builder exists for the four optional capabilities, each of which
/// is a seam that keeps a domain concern out of the shell:
///
/// | Builder method | Supplies | Why the shell cannot do it itself |
/// |---|---|---|
/// | [`Self::with_icon_painter`] | how to draw an icon key | An icon set is a licensing and rasterization decision. |
/// | [`Self::with_custom_items`] | how to draw a non-button row | Otherwise the item vocabulary grows a variant per widget. |
/// | [`Self::reporting_rects_to`] | where to publish drawn rects | Only the harness knows what it wants to assert. |
/// | [`Self::with_shortcuts`] | chord hints from outside the manifest | An application may hold its accelerators in a platform table rather than in the keymap. |
///
/// All four are optional; without them a menu draws labels, no glyphs, the
/// manifest's own chords, and publishes nothing.
#[derive(Default)]
pub struct ContextMenu<'a> {
    icons: Option<&'a mut IconPainter<'a>>,
    custom: Option<&'a mut MenuCustomRenderer<'a>>,
    rects: Option<&'a mut RectSink<'a>>,
    shortcuts: Option<Shortcuts>,
}

impl<'a> ContextMenu<'a> {
    /// A menu with no optional capabilities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Supply the painter for [`crate::commands::Command::icon`] keys.
    ///
    /// The **same** callback type the ribbon takes, so an application
    /// wires its icon set once. Without one, rows draw their labels and no
    /// glyphs — a working menu, which is the point.
    #[must_use]
    pub fn with_icon_painter(
        mut self,
        painter: &'a mut (impl FnMut(&egui::Painter, &IconRequest<'_>) + 'a),
    ) -> Self {
        self.icons = Some(painter);
        self
    }

    /// Supply the renderer for [`crate::manifest::Item::Custom`] rows.
    #[must_use]
    pub fn with_custom_items(
        mut self,
        renderer: &'a mut (impl FnMut(&mut egui::Ui, &MenuCustomItem<'_>) -> Option<HandlerToken> + 'a),
    ) -> Self {
        self.custom = Some(renderer);
        self
    }

    /// Publish the body and every drawn row to `sink`, under the stable
    /// names in [`report`].
    #[must_use]
    pub fn reporting_rects_to(mut self, sink: &'a mut (impl FnMut(&str, egui::Rect) + 'a)) -> Self {
        self.rects = Some(sink);
        self
    }

    /// Override the chord hints, instead of deriving them from the
    /// document's keymap.
    ///
    /// For an application whose accelerators live outside the manifest —
    /// a platform menu table, an inherited binding scheme. It is an
    /// override rather than an addition, because two sources for one
    /// chord is precisely the drift this crate refuses elsewhere: one of
    /// them wins, and it should be the one the caller named.
    #[must_use]
    pub fn with_shortcuts(mut self, shortcuts: Shortcuts) -> Self {
        self.shortcuts = Some(shortcuts);
        self
    }

    /// Attach this menu to a widget's secondary click.
    ///
    /// **The entry point an application wants.** It owns the popup, which
    /// is what lets it honour decision 1 in the module header: a menu with
    /// nothing to offer is never opened, and an open menu whose offer
    /// evaporates is closed.
    ///
    /// `response` must come from a widget that senses clicks —
    /// `egui::Label` does not by default; `Label::new(..).sense(Sense::click())`
    /// does.
    pub fn attach<L: MenuLookup + ?Sized>(
        self,
        response: &egui::Response,
        catalog: &L,
        registry: &CommandRegistry,
        context_id: &str,
        conditions: &ConditionSet,
    ) -> Vec<HandlerToken> {
        let popup_id = egui::Popup::default_response_id(response);
        let ctx = response.ctx.clone();

        let shortcuts = self.resolved_shortcuts(catalog);
        let Some(menu) = catalog.menu_for(context_id) else {
            // No menu for this context at all. Not an error: an
            // application may attach the same helper to a surface that has
            // no menu yet, and the correct behaviour is that right-clicking
            // does nothing.
            close_if_open(&ctx, popup_id);
            if response.secondary_clicked() {
                crate::verify::event("menu-no-such-context")
                    .kv("context", context_id)
                    .emit();
            }
            return Vec::new();
        };

        let slots = plan::resolve(menu.items(), registry, conditions, &shortcuts, context_id);
        if !plan::offers_anything(&slots) {
            close_if_open(&ctx, popup_id);
            if response.secondary_clicked() {
                crate::verify::event("menu-not-opened-nothing-on-offer")
                    .kv("context", context_id)
                    .kv("rows", slots.len())
                    .emit();
            }
            return Vec::new();
        }

        egui::Popup::context_menu(response)
            .show(|ui| self.body(ui, context_id, &slots))
            .map(|inner| inner.inner)
            .unwrap_or_default()
    }

    /// Draw the menu's rows into a `Ui` the caller has already opened.
    ///
    /// For an application that owns its own popup, or that embeds a menu
    /// in a panel. Prefer [`Self::attach`]: this entry point cannot decide
    /// *not to open*, because by the time it runs the popup exists.
    ///
    /// It does the next best thing — if the menu turns out to have nothing
    /// on offer it draws nothing and asks the containing menu to close, so
    /// the worst case is one frame of an empty popup rather than a
    /// persistent one. A caller that wants the right-click to do nothing
    /// at all must ask [`Menu::would_open`] first, which is exactly what
    /// [`Self::attach`] does for it.
    pub fn render<L: MenuLookup + ?Sized>(
        self,
        ui: &mut egui::Ui,
        catalog: &L,
        registry: &CommandRegistry,
        context_id: &str,
        conditions: &ConditionSet,
    ) -> Vec<HandlerToken> {
        let shortcuts = self.resolved_shortcuts(catalog);
        let Some(menu) = catalog.menu_for(context_id) else {
            close_containing_menu(ui);
            return Vec::new();
        };
        let slots = plan::resolve(menu.items(), registry, conditions, &shortcuts, context_id);
        if !plan::offers_anything(&slots) {
            crate::verify::event("menu-body-empty")
                .kv("context", context_id)
                .emit();
            close_containing_menu(ui);
            return Vec::new();
        }
        self.body(ui, context_id, &slots)
    }

    /// The chords this menu will show: the override if one was given, the
    /// document's keymap otherwise.
    fn resolved_shortcuts<L: MenuLookup + ?Sized>(&self, catalog: &L) -> Shortcuts {
        self.shortcuts
            .clone()
            .unwrap_or_else(|| catalog.shortcuts())
    }

    /// Lay the resolved rows out. Consumes the builder, which holds `&mut`
    /// borrows of the application's callbacks for exactly this long.
    fn body(self, ui: &mut egui::Ui, context_id: &str, slots: &[Slot<'_>]) -> Vec<HandlerToken> {
        let mut ctx = Ctx {
            context: context_id.to_owned(),
            theme: Theme::of(ui.ctx()),
            reporter: Reporter::new(self.rects),
            icons: self.icons,
            custom: self.custom,
            base_id: ui.id().with("egui-shell-menu").with(context_id),
            invoked: Vec::new(),
        };

        // ★ Decision 3 (the icon column) is taken FIRST, because decision 2
        // depends on it: whether a row lays out an icon slot changes how
        // wide that row wants to be, and the widest row is what sets the
        // body width. Measuring before knowing this would under-measure
        // every icon-less row in a menu that has icons — see
        // `plan::RowWidths::icon`.
        //
        // One decision for the whole menu, taken once, and handed to both
        // the measurer and every row. `plan::reserves_icon_column` carries
        // the argument for why it is per-menu rather than per-row.
        let reserve_icons = plan::reserves_icon_column(slots);

        // Decision 2: measure, then set the width, then draw. Nothing
        // below may widen the body, because the two columns only line up
        // if every row was justified to the same number.
        let width = measure(ui, &ctx, slots, reserve_icons);
        if width.truncating {
            crate::verify::event("menu-width-clamped")
                .kv("context", context_id)
                .kv("limit", format!("{:.1}", plan::MAX_BODY_WIDTH))
                .emit();
        }
        // BOTH bounds, and the pair is load-bearing in opposite
        // directions:
        //
        // * `set_min_width` widens a body that would otherwise be as
        //   narrow as its widest row's intrinsic size, which is where the
        //   chord column comes from.
        // * `set_max_width` is what makes [`plan::MAX_BODY_WIDTH`] real.
        //   `Button::truncate` shortens a label to the room *available*,
        //   and without an upper bound the room available is whatever the
        //   popup's provisional area happened to be — so a pathological
        //   label would produce a banner and the clamp would be a constant
        //   nothing consults.
        //
        // Together they fix the body at exactly the planned width, which
        // is also what makes every row the same width and therefore what
        // makes the right-aligned chords a column rather than a diagonal.
        ui.set_min_width(width.points);
        ui.set_max_width(width.points);

        for slot in slots {
            match slot {
                Slot::Separator => {
                    ui.separator();
                }
                Slot::Command {
                    command,
                    enabled,
                    selected,
                    shortcut,
                } => command_row(
                    ui,
                    &mut ctx,
                    &RowPlan {
                        command,
                        enabled: *enabled,
                        selected: *selected,
                        shortcut: *shortcut,
                        truncating: width.truncating,
                        icon: plan::icon_slot(reserve_icons, command.icon.is_some()),
                    },
                ),
                Slot::Custom { kind, payload } => custom_row(ui, &mut ctx, kind, *payload),
            }
        }

        let body_rect = ui.min_rect();
        ctx.reporter.report(body_rect, || report::body(context_id));
        ctx.invoked
    }
}

impl std::fmt::Debug for ContextMenu<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenu")
            .field("icons", &self.icons.is_some())
            .field("custom", &self.custom.is_some())
            .field("rects", &self.rects.is_some())
            .field("shortcuts", &self.shortcuts.is_some())
            .finish()
    }
}

impl Menu {
    /// **Whether right-clicking would produce a menu at all.**
    ///
    /// Pure, cheap, and the same question [`ContextMenu::attach`] asks
    /// itself. Public because an application may want to answer it for
    /// another reason — deciding whether to draw a "⋯" affordance beside a
    /// row, say, which should appear exactly when a right-click would do
    /// something.
    ///
    /// `false` when the context has no menu, when every command it names
    /// is missing from this build, or when every command it names is
    /// disabled. See [`plan`]'s rule 2.
    #[must_use]
    pub fn would_open<L: MenuLookup + ?Sized>(
        catalog: &L,
        registry: &CommandRegistry,
        context_id: &str,
        conditions: &ConditionSet,
    ) -> bool {
        let Some(menu) = catalog.menu_for(context_id) else {
            return false;
        };
        let shortcuts = catalog.shortcuts();
        let slots = plan::resolve(menu.items(), registry, conditions, &shortcuts, context_id);
        plan::offers_anything(&slots)
    }

    /// Attach a context menu to a widget's secondary click, with no
    /// optional capabilities, and report the commands the operator chose.
    ///
    /// The shell **executes nothing**: the returned tokens are intent, and
    /// the application dispatches them at its own choke point.
    pub fn attach<L: MenuLookup + ?Sized>(
        response: &egui::Response,
        catalog: &L,
        registry: &CommandRegistry,
        context_id: &str,
        conditions: &ConditionSet,
    ) -> Vec<HandlerToken> {
        ContextMenu::new().attach(response, catalog, registry, context_id, conditions)
    }

    /// Draw a menu's rows into an already-open `Ui`, and report the
    /// commands the operator chose.
    ///
    /// `catalog` is normally the [`crate::Shell`] — see
    /// [`MenuLookup`] on why the parameter is a trait rather than that
    /// type.
    pub fn show<L: MenuLookup + ?Sized>(
        ui: &mut egui::Ui,
        catalog: &L,
        registry: &CommandRegistry,
        context_id: &str,
        conditions: &ConditionSet,
    ) -> Vec<HandlerToken> {
        ContextMenu::new().render(ui, catalog, registry, context_id, conditions)
    }
}

// ---------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------

/// Everything one command row needs to draw itself.
///
/// A struct rather than eight parameters, and not only for the lint: every
/// field here is a *decision already taken* — by [`plan::resolve`], by
/// [`measure`], by [`plan::icon_slot`] — and a positional argument list of
/// six `bool`-ish values is the shape where two of them get swapped and
/// the result still compiles.
struct RowPlan<'a> {
    /// The registration: label, tooltip, icon key, handler token.
    command: &'a crate::commands::Command,
    /// Whether the command's `Enable` predicate holds. `false` greys it.
    enabled: bool,
    /// Whether the command is currently *on* (a checkable item).
    selected: bool,
    /// The chord to show, right-aligned, if the keymap binds one.
    shortcut: Option<&'a str>,
    /// Whether the body was clamped to [`plan::MAX_BODY_WIDTH`] and labels
    /// must therefore truncate rather than overflow.
    truncating: bool,
    /// What this row does with its icon slot — **the same value
    /// [`measure`] budgeted width for.** The two must agree; they are
    /// derived from one function for that reason.
    icon: plan::IconSlot,
}

/// Draw one command row: optional icon, label, grow, right-aligned chord.
fn command_row(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, row: &RowPlan<'_>) {
    let RowPlan {
        command,
        enabled,
        selected,
        shortcut,
        truncating,
        icon,
    } = *row;

    // ★ The slot is laid out whenever the MENU reserves the column, even
    // when this command has no key — that blank is what keeps the label
    // column straight, and `plan::icon_slot` argues for it. The painter is
    // only asked for a `Glyph`, so an icon-less row draws literally
    // nothing in its slot: no placeholder, no outline, no dimmed
    // stand-in. R9 in the small.
    let slot_id = icon.is_reserved().then(|| ctx.id("icon", &command.id));

    let mut atoms = Atoms::default();
    if let Some(slot_id) = slot_id {
        atoms.push_right(egui::Atom::custom(
            slot_id,
            Vec2::splat(ctx.theme.metrics.icon_pts),
        ));
    }
    atoms.push_right(RichText::new(&command.label));
    if let Some(chord) = shortcut.filter(|c| !c.trim().is_empty()) {
        // The zero-width `grow` atom is what pushes the chord to the right
        // edge; the chord is `weak()` so it reads as an annotation rather
        // than as a second label. Both mirror `egui::Button::shortcut_text`,
        // which is spelled out here rather than called because the icon
        // slot has to be pushed before the label and `shortcut_text` can
        // only append.
        atoms.push_right(egui::Atom::grow());
        atoms.push_right(RichText::new(chord).weak());
    }

    let laid_out = ui
        .scope(|ui| {
            if !enabled {
                ui.disable();
            }
            let mut button = egui::Button::new(atoms);
            if selected {
                // Only when it *is* a toggle: opting an ordinary Delete
                // into toggle semantics would have it announce as a
                // pressed/not-pressed control it is not.
                button = button.selected(true);
            }
            if truncating {
                button = button.truncate();
            }
            button.atom_ui(ui)
        })
        .inner;

    if icon.draws()
        && let Some(key) = command.icon.as_deref()
        && let Some(slot_id) = slot_id
        && let Some(rect) = laid_out.rect(slot_id)
        && let Some(painter) = ctx.icons.take()
    {
        // ★ Published from INSIDE the painting branch, deliberately.
        // Reporting it beside the row's own rect would make the name mean
        // "a slot was reserved", which is true of a blank one too and
        // therefore says nothing about whether this build draws glyphs at
        // all. Here, the name's presence in a trace means a painter
        // existed and was handed this rectangle — which is the only signal
        // a driven check has on this surface. `menu::report`'s header
        // carries the argument.
        ctx.reporter
            .report(rect, || report::icon(&ctx.context, &command.id));
        let visuals = ui.style().interact(&laid_out.response);
        painter(
            ui.painter(),
            &IconRequest {
                key,
                rect,
                tint: visuals.fg_stroke.color,
                enabled,
                selected,
            },
        );
        ctx.icons = Some(painter);
    }

    let response = laid_out.response;
    a11y::describe_item(&response, command, shortcut, enabled, selected);
    ctx.reporter
        .report(response.rect, || report::item(&ctx.context, &command.id));

    let response = match (&command.tooltip, enabled) {
        (Some(tip), true) => response.on_hover_text(tip),
        // A disabled row is the one that most needs its tooltip: it is
        // where the operator asks "why can I not do this?".
        (Some(tip), false) => response.on_disabled_hover_text(tip),
        (None, _) => response,
    };

    if response.clicked() {
        ctx.invoke(command.handler);
        crate::verify::event("menu-command-invoked")
            .kv("context", &ctx.context)
            .kv("id", &command.id)
            .kv("handler", command.handler.get())
            .emit();
        // Choosing an item dismisses the menu. `egui` does not do this for
        // us — its own `Response::context_menu` example calls `ui.close()`
        // inside the click branch for the same reason.
        close_containing_menu(ui);
    }
}

/// Draw one application-owned row, or disclose that nobody drew it.
fn custom_row(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, kind: &str, payload: Option<&str>) {
    let request = MenuCustomItem {
        kind,
        payload,
        context: &ctx.context,
    };
    // `take` so the borrow of `ctx.custom` does not conflict with
    // `ctx.invoke`; put back immediately, because a renderer that vanished
    // after the first custom row would be a very confusing bug.
    if let Some(renderer) = ctx.custom.take() {
        // A `scope` rather than measuring `ui.min_rect()` either side: the
        // `Ui` handed to `body` is the *whole menu*, so its `min_rect`
        // before and after this row differ by the row **and by everything
        // above it**. A child scope's response rect is the row alone,
        // which is what a harness asking "where is the colour swatch"
        // needs — and what a click on the published rectangle has to land
        // inside.
        let laid_out = ui.scope(|ui| renderer(ui, &request));
        ctx.custom = Some(renderer);
        ctx.reporter.report(laid_out.response.rect, || {
            report::custom(&ctx.context, kind)
        });
        if let Some(token) = laid_out.inner {
            ctx.invoke(token);
            close_containing_menu(ui);
        }
    } else {
        // No renderer: reserve the row the width plan already budgeted for
        // it, so the gap is visible rather than silently closing up. An
        // application that put a custom item in its document and supplied
        // no renderer has a defect, and a hole is how it finds out.
        crate::verify::event("menu-custom-item-unrendered")
            .kv("context", &ctx.context)
            .kv("kind", kind)
            .emit();
        ui.allocate_space(vec2(ui.available_width(), ctx.theme.metrics.control_height));
    }
}

// ---------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------

/// The width this body will be laid out at.
///
/// Measured with [`crate::ribbon::measure::text_width`] and
/// [`crate::ribbon::measure::button_padding`] — **the ribbon's own
/// functions**, not copies. A menu row and a band control are both
/// `egui::Button`s, and two surfaces that measured the same text with
/// different constants would disagree about how wide the same command is
/// for no reason a reader could find.
fn measure(
    ui: &egui::Ui,
    ctx: &Ctx<'_>,
    slots: &[Slot<'_>],
    reserve_icons: bool,
) -> plan::BodyWidth {
    let atom_gap = ui.spacing().icon_spacing;
    let padding = button_padding(ui);
    let totals: Vec<f32> = slots
        .iter()
        .filter_map(|slot| match slot {
            Slot::Command {
                command, shortcut, ..
            } => Some(
                plan::RowWidths {
                    // ★ `icon_slot`, not `command.icon.is_some()` — an
                    // icon-less row in a menu that reserves the column
                    // still spends the width, and measuring it as though
                    // it did not is how the widest row comes to truncate
                    // its own label.
                    icon: if plan::icon_slot(reserve_icons, command.icon.is_some()).is_reserved() {
                        ctx.theme.metrics.icon_pts
                    } else {
                        0.0
                    },
                    label: text_width(ui, &command.label, &TextStyle::Button),
                    shortcut: shortcut
                        .map(|c| text_width(ui, c, &TextStyle::Button))
                        .unwrap_or(0.0),
                }
                .total(atom_gap, padding),
            ),
            // A separator has no width of its own, and a custom row's
            // width is the application's business — the shell budgets it
            // the body width rather than the other way round.
            Slot::Separator | Slot::Custom { .. } => None,
        })
        .collect();
    plan::body_width(&totals)
}

// ---------------------------------------------------------------------
// Popup plumbing
// ---------------------------------------------------------------------

/// Close a popup id if `egui` currently believes it is open.
///
/// The second half of decision 1. `egui` tracks a popup's open state in
/// memory, not by whether anyone drew it, so a menu that stops being drawn
/// stays "open" and reappears the moment it is drawn again — at the old
/// pointer position, with no right-click behind it.
fn close_if_open(ctx: &egui::Context, popup_id: egui::Id) {
    if egui::Popup::is_id_open(ctx, popup_id) {
        egui::Popup::close_id(ctx, popup_id);
    }
}

/// Ask the containing menu popup to close, if there is one.
///
/// Guarded rather than calling [`egui::Ui::close`] unconditionally,
/// because `close` logs a warning when there is no closable parent — and
/// [`ContextMenu::render`] is explicitly allowed to be called on a `Ui`
/// that is not a popup at all. A warning on a supported use is a warning
/// that teaches people to ignore warnings.
fn close_containing_menu(ui: &egui::Ui) {
    if ui
        .stack()
        .iter()
        .any(|frame| frame.kind() == Some(UiKind::Menu))
    {
        ui.close();
    }
}
