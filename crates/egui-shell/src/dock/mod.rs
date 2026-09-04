//! # `dock` — the panel host
//!
//! Multiple columns per side, vertical stacks within a column, tabbed
//! groups within a stack, user-draggable splitters, an overflow menu that
//! cannot be squeezed out, and a layout that is a plain value the
//! application can save, name, restore and reset ([`crate::layout`]).
//!
//! ## What the application supplies, and what it never has to
//!
//! ```no_run
//! use egui_shell::dock::{Dock, DockState, DockLayout, SideLayout, Column, Stack,
//!                        PanelRegistry, PanelInfo};
//!
//! // 1. Say what can be docked. Three strings per panel; no types, no enum.
//! let mut panels = PanelRegistry::new();
//! panels.register(PanelInfo::new("pages", "Pages")
//!     .with_tooltip("Pages — jump to a page, reorder or rotate sheets"));
//! panels.register(PanelInfo::new("layers", "Layers")
//!     .with_tooltip("Layers — show or hide the document's optional content"));
//!
//! // 2. Say where they start. This is a value; it round-trips to a file.
//! let layout = DockLayout::new(
//!     SideLayout::new([Column::new([Stack::tabbed(["pages", "layers"])])]),
//!     SideLayout::none(),
//! );
//! let mut state = DockState::new(layout);
//!
//! // 3. Draw. Once per frame, with one callback for every panel body.
//! # fn frame(ui: &mut egui::Ui, state: &mut DockState, panels: &PanelRegistry) {
//! Dock::new().with_registry(panels).show(ui, state, |panel, ui| {
//!     match panel.as_str() {
//!         "pages"  => { ui.label("thumbnails go here"); }
//!         "layers" => { ui.label("the layer tree goes here"); }
//!         _ => {}
//!     }
//! });
//! # }
//! ```
//!
//! Nothing in that list mentions a document, and nothing in this module
//! knows what a "page thumbnail" is. A [`PanelId`] is an opaque string;
//! `SHELL_FRAMEWORK.md` §2 states the rule and
//! `tools/gates/check-shell-purity.sh` enforces its negative half.
//!
//! ## The single dispatcher
//!
//! There is **one** callback for every panel body, on every side, docked
//! or overflowed. That is carried across deliberately from the previous
//! implementation, whose standing rule R80 required exactly one, and
//! whose own notes explain what the alternative costs: a float-or-dock
//! dual mode meant *"two code paths for the same content, each
//! duplicating open-state, position/size and focus handling, for zero
//! operator benefit at this scale."*
//!
//! It also keeps a future tear-out honest. `MODES_AND_PANELS.md` records
//! the finding that `show_viewport_immediate` takes `FnMut` with no
//! `Send + Sync + 'static` bound, *"so a torn-out panel therefore keeps
//! the identical `panel_body(...)` signature as the docked one — the
//! one-dispatcher rule survives intact."* Nothing here forecloses that.
//!
//! ## Why this is built on `egui` directly
//!
//! `SHELL_FRAMEWORK.md` §3 lists this module as a *"panel host over
//! `egui_tiles`"*, and it is not, for a reason worth stating plainly
//! rather than leaving to be discovered: **`egui_tiles` is not a declared
//! dependency of this crate**, and its manifest was not this agent's to
//! edit at the time this landed. That is the constraint. What follows is
//! why the outcome is nonetheless the right one on the merits, and what
//! it costs.
//!
//! Every requirement that made the engine attractive is present here:
//! columns, stacks, tabs, draggable splitters, per-group active tabs. And
//! four of the twelve failure modes are answered *better* by owning the
//! layout than by wrapping a library:
//!
//! | Failure mode | With a general tiling engine | Here |
//! |---|---|---|
//! | #8 tab overflow | The engine hides overflowing tabs behind scroll arrows with `ScrollBarVisibility::AlwaysHidden` — the same class of failure. The previous implementation capped default groups at **two panes** to dodge it. | [`tabs`] reserves the affordance before the first tab is measured, and the cap is retired. |
//! | #3 hidden tab dictates width | Avoided by that engine too, but by a property of *its* internals (`min_size` is a global scalar) — true today, and not a contract. | [`plan::MIN_COLUMN_WIDTH`] is a constant this crate owns, and a test asserts no minimum consults a label. |
//! | #7 coupled splitters | Whatever the engine's share algebra does. | `plan::drag_boundary` writes to exactly two slice entries. |
//! | (f) persistence | The engine's `Tree` derives serde behind its **default** feature, which this workspace disables — so the persistence had to be hand-written over an owned schema regardless. | The schema *is* the model; there is nothing to translate. |
//!
//! **What is genuinely given up**, stated rather than implied away:
//!
//! - **Drag-and-drop rearrangement of panels between compartments.** The
//!   engine ships it; this does not. Tabs can be selected, closed and
//!   restored; they cannot yet be dragged into a different stack. The
//!   layout *model* expresses every arrangement, so this is a gesture
//!   gap, not a capability gap — an operator reaches those arrangements
//!   through a saved workspace rather than by dragging. It is the next
//!   thing to build here.
//! - **Free tiling of arbitrary depth.** Four levels are fixed: side,
//!   column, stack, tab. The engine allows arbitrary nesting. Nothing in
//!   `MODES_AND_PANELS.md`'s target behaviour needs it, and a fixed depth
//!   is what makes the serialized form readable and diffable.
//!
//! If the dependency is added later, the migration is [`crate::layout`]'s
//! loader building an engine tree instead of a [`DockLayout`] — the
//! persisted form does not change, which is exactly the property owning
//! the schema was supposed to buy.
//!
//! ## R128 — the fit-to-viewport feedback loop, and how it is closed here
//!
//! `D:/dev/rag/egui/bottom_panel_height_change_retriggers_fit_to_viewport_zoom.md`
//! records a measured 230 % → 224 % → 215 % zoom drift caused by a panel
//! whose height was **content-driven** sitting next to a per-frame
//! fit-to-viewport zoom. The rule it produced is pdfcer's R128: *a panel
//! whose size feeds a fit-to-viewport computation has a fixed size.*
//!
//! A user-resizable dock looks like a direct violation. It is not, and
//! the distinction is the one the RAG entry itself draws: the loop is
//! driven by **content**, not by **the operator**.
//!
//! - The dock's outer width is drawn with
//!   [`egui::Panel::exact_size`], from a number stored in the layout.
//!   The RAG entry is explicit that this is the API that closes the loop
//!   and that `default_width`, `min_width`/`max_width` and
//!   `resizable(false)` all fail to: *"Only `exact_size` closes it."*
//! - Nothing a panel body draws can change that number. A body that
//!   overflows is clipped, not accommodated.
//! - The number changes only when the operator drags a splitter — an
//!   explicit, discrete gesture, which is exactly the *"explicit
//!   trigger"* the other half of the RAG entry's advice names.
//!
//! And the harder half of R128 is respected by omission: **the
//! application's content area is not inside a dock compartment.** The
//! dock draws side panels; the application draws its canvas in whatever
//! remains. `MODES_AND_PANELS.md` recommends the single wide tree
//! spanning left ▸ canvas ▸ right as *"the real unlock"* for cross-dock
//! dragging, and immediately adds that it *"puts the canvas inside a
//! resizable pane and fires R128 directly. So the fit-zoom must be
//! converted to cached-recompute-on-explicit-trigger first, as its own
//! landing, before the wide tree is attempted."* That landing has not
//! happened, so the canvas stays outside.
//!
//! ## Module map
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`model`] | The layout value: sides, columns, stacks, tabs, shares. Serializable, ownable, diffable. |
//! | [`plan`] | Every number. Pure functions, no `egui`, swept across hundreds of widths by unit tests. |
//! | [`tabs`] | One stack's tab bar and its reserved overflow menu. |
//! | [`tab_menu`] | The seam an application uses to own a tab's secondary click: [`Dock::with_tab_menu`] and the [`TabMenu`] it hands out. |
//! | [`splitter`] | The draggable boundary, its cursor and its feedback. |
//! | [`report`] | The rect stream a verification harness reads. |
//! | `ctx` | The per-frame context and the intent queue. |
//! | `width_tests` | Layout tests against **real** font metrics. |

/// The two controls that minimise a side and bring it back — split out under
/// R2 on 2026-08-20. Its header carries the operator's ask and the argument for
/// why a collapsed side must leave something on screen.
pub mod banner;
mod collapse;
pub mod ctx;
pub mod model;
pub mod plan;
pub mod report;
pub mod splitter;
pub mod tab_menu;
pub mod tabs;

#[cfg(test)]
mod width_tests;

// ★ The synthetic proportional face, borrowed rather than duplicated.
//
// `crate::ribbon::testfont` is `mod testfont;` — private to the ribbon —
// so it cannot be reached by a path from here, and `ribbon/` is not this
// module's to edit. Including the file by `#[path]` under `#[cfg(test)]`
// compiles the same source a second time, in test builds only, with no
// edit to the ribbon and no 600-line copy in this tree.
//
// Why it must be here at all is not a detail:
// `D:/dev/rag/rust/a_crate_tested_alone_and_in_a_workspace_gets_different_features_so_layout_tests_can_be_vacuous.md`
// records this crate's own suite passing 116 tests over a width layer
// that had never been shown a non-zero width, because `egui` with
// `default-features = false` supplies no font data and every galley
// measures ≈ 0. Every width assertion in `width_tests` would be
// satisfied by nothing at all without this.
#[cfg(test)]
#[path = "../ribbon/testfont.rs"]
// `clippy::duplicate_mod` is exactly right in general and wrong here: it
// warns that one file compiled as two modules gives two distinct types
// that look identical. That is the intent. Nothing crosses between the
// ribbon's copy and this one — each is used only by its own sibling
// tests — and the alternative clippy would prefer, a shared module, is
// what `ribbon/` not being this module's to edit rules out.
#[allow(clippy::duplicate_mod)]
mod testfont;

use egui::{Align, Layout, Rect, UiBuilder, Vec2};

use ctx::{Ctx, Intent};
use splitter::Axis;

pub use banner::BannerHandler;
pub use model::{
    AnyPanel, Column, DockLayout, DockSide, PanelAddress, PanelCatalog, PanelId, PanelInfo,
    PanelRegistry, SideLayout, Stack,
};
pub use report::{RectReport, RectSink};
pub use tab_menu::{TabMenu, TabMenuHandler};

/// What one frame of the dock drew and what the operator did to it.
///
/// Returned by [`Dock::show`] and also kept on [`DockState`], because two
/// different callers want it: the frame's own caller, and a diagnostic
/// surface that runs later in the same frame and has no access to the
/// return value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DockFrameReport {
    /// Every panel whose body was drawn — the active tab of every stack
    /// on every visible side.
    ///
    /// **This is the honest answer to "what is on screen".** A panel
    /// behind another tab is not in this list, and neither is one on a
    /// hidden side. An application deriving a toolbar toggle's selected
    /// state should read this rather than keeping a boolean of its own;
    /// the previous implementation records a `properties_open` flag that
    /// *"could disagree with what was on screen"*, which is the defect
    /// the field exists to make unnecessary.
    pub panels_drawn: Vec<PanelId>,
    /// How many tabs were moved into an overflow menu, across every
    /// stack.
    pub panels_overflowed: usize,
    /// How many overflow affordances were drawn.
    pub overflow_menus: usize,
    /// Which sides drew anything.
    pub sides_drawn: Vec<DockSide>,
    /// The panel whose tab the operator selected this frame, if any.
    pub activated: Option<PanelId>,
    /// The panel the operator closed this frame, if any.
    pub closed: Option<PanelId>,
    /// Whether the layout changed this frame and is therefore worth
    /// saving.
    ///
    /// An application persists on this rather than on a timer: writing a
    /// layout file on every frame is what makes the benchmarked
    /// application's own layout file *"rewritten on every exit"* and its
    /// community's workaround — copying the file aside and back — as
    /// awkward as `MODES_AND_PANELS.md` records it being.
    pub layout_changed: bool,
}

/// The dock's live state: the arrangement, plus what the last frame did.
///
/// Held by the application across frames. `Clone` so a workspace can be
/// snapshotted; `PartialEq` so a test can assert a frame changed nothing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DockState {
    layout: DockLayout,
    last_frame: DockFrameReport,
}

impl DockState {
    /// Start from an arrangement.
    ///
    /// The arrangement is normalized on the way in, so an application's
    /// built-in default cannot ship a stack with no tabs or a panel
    /// mounted twice. An application that wants to *know* whether its
    /// default needed repair asserts [`DockLayout::is_normalized`] in its
    /// own test suite — the same posture `manifest` takes towards its
    /// built-in layer, and for the same reason: a defect in a compiled-in
    /// constant should fail a test, not be quietly patched on every
    /// machine that runs it.
    #[must_use]
    pub fn new(layout: DockLayout) -> Self {
        let mut layout = layout;
        layout.normalize();
        Self {
            layout,
            last_frame: DockFrameReport::default(),
        }
    }

    /// The current arrangement.
    #[must_use]
    pub fn layout(&self) -> &DockLayout {
        &self.layout
    }

    /// The current arrangement, mutably.
    ///
    /// The application's route to everything [`DockLayout`] can do —
    /// mounting a panel, hiding a side, applying a workspace, resetting a
    /// scope. Deliberately **not** normalized on the way out: a caller
    /// making several edits should not pay a repair pass per edit. Call
    /// [`Self::normalize`] when the edits are finished, or let the next
    /// [`Dock::show`] do it.
    pub fn layout_mut(&mut self) -> &mut DockLayout {
        &mut self.layout
    }

    /// Replace the arrangement wholesale — how a named workspace is
    /// applied.
    pub fn set_layout(&mut self, layout: DockLayout) {
        self.layout = layout;
        self.layout.normalize();
    }

    /// Repair every structural invariant. See [`DockLayout::normalize`].
    pub fn normalize(&mut self) {
        self.layout.normalize();
    }

    /// What the last frame drew.
    #[must_use]
    pub fn last_frame(&self) -> &DockFrameReport {
        &self.last_frame
    }

    /// Bring a panel to the front of its stack, revealing its side.
    ///
    /// Returns `false` if the panel is not mounted — never an error, see
    /// [`DockLayout::activate`].
    pub fn activate(&mut self, panel: &PanelId) -> bool {
        self.layout.activate(panel)
    }

    /// Whether a panel's body is actually being drawn.
    #[must_use]
    pub fn is_on_screen(&self, panel: &PanelId) -> bool {
        self.layout.is_on_screen(panel)
    }
}

/// The panel host. Built per frame, cheap, holds no state of its own.
///
/// Mirrors [`crate::ribbon::Ribbon`]'s shape so an application configures
/// both surfaces the same way.
#[derive(Default)]
pub struct Dock<'a> {
    registry: Option<&'a PanelRegistry>,
    sink: Option<&'a mut RectSink<'a>>,
    id_salt: Option<egui::Id>,
    tab_menu: Option<&'a mut TabMenuHandler<'a>>,
    /// The permanent strip above one side's columns, if the application
    /// asked for one. Built by [`Dock::with_side_banner`], which lives in
    /// [`banner`] beside the geometry it belongs to.
    banner: Option<(DockSide, f32, &'a mut BannerHandler<'a>)>,
}

impl<'a> Dock<'a> {
    /// A dock with no registry and no rect sink.
    ///
    /// Usable as-is: with no registry every tab is labelled with its own
    /// id, which is ugly and truthful. See `ctx::Ctx::describe` on why the
    /// fallback is a fallback rather than a skip.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Supply the panels the application can draw.
    #[must_use]
    pub fn with_registry(mut self, registry: &'a PanelRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Publish every drawn rectangle to a sink. See [`report`].
    ///
    /// The sink is handed a [`report::RectReport`]: the region's name, the
    /// rectangle it was laid out at, **and the clip rectangle in force where
    /// it was drawn**.
    ///
    /// ```no_run
    /// # use egui_shell::dock::{Dock, DockState, RectReport};
    /// # fn frame(ui: &mut egui::Ui, state: &mut DockState) {
    /// let mut sink = |r: &RectReport<'_>| {
    ///     // `r.rect` alone answers "where was this laid out".
    ///     // `r.rect` with `r.clip` answers "can the operator see it".
    ///     let shown = r.clip.intersect(r.rect);
    ///     let _visible = (shown.width() * shown.height()) / (r.rect.width() * r.rect.height());
    /// };
    /// Dock::new().reporting_rects_to(&mut sink).show(ui, state, |_p, _ui| {});
    /// # }
    /// ```
    ///
    /// # Why the dock reports a clip and the ribbon does not
    ///
    /// [`crate::ribbon::RectSink`] and [`crate::menu`]'s — which is the
    /// ribbon's, shared — are still `FnMut(&str, Rect)`. That divergence is a
    /// decision, not an unfinished migration, and the whole argument is on
    /// [`report::RectSink`] under *"Why the dock reports a clip and the
    /// ribbon does not"*. The short of it: **the dock's rects are
    /// compartments and the question asked of them is reachability; the
    /// ribbon's are content and the question asked of them is whether they
    /// drew at all.**
    #[must_use]
    pub fn reporting_rects_to(
        mut self,
        sink: &'a mut (impl FnMut(&report::RectReport<'_>) + 'a),
    ) -> Self {
        self.sink = Some(sink);
        self
    }

    /// Take over what a right-click on a panel tab offers.
    ///
    /// `handler` is called once per **drawn** tab, per frame, in bar
    /// order, whether or not the operator clicked anything — which is what
    /// [`crate::menu::Menu::attach`] needs in order to decide whether to
    /// open, to keep an open menu drawn, and to close one whose offer has
    /// evaporated. It receives a [`TabMenu`]: the [`PanelId`] and the tab
    /// button's [`egui::Response`], with its accessible name already
    /// published.
    ///
    /// ```no_run
    /// # use egui_shell::dock::{Dock, DockState, PanelRegistry, TabMenu};
    /// # use egui_shell::{CommandRegistry, ConditionSet, HandlerToken, Menus, menu::Menu};
    /// # fn frame(ui: &mut egui::Ui, state: &mut DockState, panels: &PanelRegistry,
    /// #          menus: &Menus, commands: &CommandRegistry, conditions: &ConditionSet) {
    /// let mut chosen: Vec<HandlerToken> = Vec::new();
    /// let mut tab_menu = |tab: &mut TabMenu<'_>| {
    ///     chosen.extend(Menu::attach(tab.response(), menus, commands, "dock.tab", conditions));
    /// };
    /// Dock::new()
    ///     .with_registry(panels)
    ///     .with_tab_menu(&mut tab_menu)
    ///     .show(ui, state, |_panel, _ui| {});
    /// # }
    /// ```
    ///
    /// # What this costs and what it gives back
    ///
    /// **The dock's built-in Close menu is not drawn on a tab whose click
    /// the application has taken.** Not a policy — `egui` derives a
    /// context menu's popup id from the response
    /// (`Popup::default_response_id` is `response.id.with("popup")`), so
    /// two menus on one tab are two writers of one open/closed flag and
    /// the visible result is a menu that flickers, mispositions or refuses
    /// to close. One response, one popup, one owner.
    ///
    /// Closing is given back as [`TabMenu::request_close`], which records
    /// the dock's own `Intent::Close` — reported in
    /// [`DockFrameReport::closed`], counted in
    /// [`DockFrameReport::layout_changed`], applied after the frame like
    /// every other intent. An application that wants a close row keeps one
    /// in its own menu and calls that method when the row is chosen; the
    /// row is the application's, the action stays the dock's.
    ///
    /// An application that supplies **no** handler is unaffected: the
    /// built-in Close is drawn exactly as it always was. See
    /// [`tab_menu`] for the full account.
    ///
    /// # Borrowing
    ///
    /// `handler` and the `body` closure passed to [`Self::show`] are two
    /// separate `FnMut`s that both live across the call, so they cannot
    /// both capture `&mut` to the same thing. The pattern that works — and
    /// the one the example uses — is for the handler to *record* into a
    /// local `Vec` and for the application to dispatch after `show`
    /// returns, which is the same discipline the dock applies to its own
    /// intents and is what [`crate::menu`]'s token seam is shaped for.
    #[must_use]
    pub fn with_tab_menu(mut self, handler: &'a mut (impl FnMut(&mut TabMenu<'_>) + 'a)) -> Self {
        self.tab_menu = Some(handler);
        self
    }

    /// Distinguish this dock from another in the same window.
    ///
    /// Only needed by an application hosting two independent docks — a
    /// document window and a preview window, say. Without it every
    /// interactive element in both would share ids, and `egui` would
    /// treat a click on one as a click on the other.
    #[must_use]
    pub fn with_id_salt(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.id_salt = Some(egui::Id::new(salt));
        self
    }

    /// Draw both docks and call `body` once per visible panel.
    ///
    /// # The order of the three phases, and why it is that order
    ///
    /// 1. **Snapshot.** The layout is cloned. Everything drawn this frame
    ///    is drawn from the snapshot, so one frame shows one truth.
    /// 2. **Draw**, recording `Intent`s. No `&mut` to the layout exists
    ///    anywhere in this phase, which is what makes it structurally
    ///    impossible for a resize pass to write a *computed* span back
    ///    into a stored share — failure mode #6, closed by construction
    ///    rather than by care. See `ctx`'s header for the full argument.
    /// 3. **Apply.** The intents are applied, in order, in one place.
    ///
    /// The cost is one frame of latency on a splitter drag, during a
    /// gesture `egui` is already repainting continuously for.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut DockState,
        mut body: impl FnMut(&PanelId, &mut egui::Ui),
    ) -> DockFrameReport {
        // ★ Read the entitlement BEFORE anything lays out.
        //
        // `D:/dev/rag/egui/a_sibling_row_that_overflows_grows_the_parent_max_rect_...md`:
        // a child that lays out past its parent's `max_rect` GROWS it,
        // and a later query for available space then returns a rectangle
        // extending past the window edge. Measured there: a reservation
        // taken from that rectangle landed 78 pt off screen, correct
        // arithmetic applied to a width the window never had. Everything
        // below is derived from this value, captured while nothing can
        // have inflated it.
        let entitled = ui.max_rect();
        let window_width = entitled.width();

        let snapshot = state.layout.clone();
        let mut report = DockFrameReport::default();

        let mut ctx = Ctx {
            registry: self.registry,
            theme: crate::theme::Theme::of(ui.ctx()),
            reporter: report::Reporter::new(self.sink.take()),
            id_salt: self
                .id_salt
                .unwrap_or_else(|| egui::Id::new("egui-shell-dock")),
            // `take`, like the sink above: both are `&'a mut`, and a
            // `Dock` is built per frame, so moving them into the frame's
            // context rather than reborrowing keeps `show` free of a
            // second lifetime parameter. A `Dock` shown twice draws its
            // tabs the second time with the built-in menu — the same
            // pre-existing quirk the sink has, and the same answer:
            // build one per frame, which every caller does.
            tab_menu: self.tab_menu.take(),
            intents: Vec::new(),
        };

        for side in DockSide::ALL {
            let s = snapshot.side(side);
            // ★ A side with no panels in it draws nothing at all — not even a
            // rail. There is nothing to bring back, and a control that opened
            // an empty compartment would be the no-placeholders rule broken:
            // an affordance for something that cannot happen.
            if s.is_empty() {
                continue;
            }
            if s.visible {
                report.sides_drawn.push(side);
                let width = snapshot.drawn_side_width(side, window_width);
                self.draw_side(ui, &mut ctx, &snapshot, side, width, &mut report, &mut body);
            } else {
                // ★★ The RAIL — the way back from a collapsed side.
                //
                // Before 2026-08-20 a hidden side drew nothing, so the only
                // route back was a ribbon command the operator had to know
                // existed. A collapsed panel with no visible handle is a panel
                // that has been lost rather than minimised, which is the
                // difference the operator was asking about.
                collapse::draw_collapsed_rail(ui, &mut ctx, side, &mut report);
            }
        }

        // Phase 3: apply. The one place the layout is mutable.
        report.layout_changed = apply(&mut state.layout, &ctx.intents, &mut report);
        state.last_frame = report.clone();
        report
    }

    /// Draw one side.
    #[allow(clippy::too_many_arguments)]
    fn draw_side(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut Ctx<'_>,
        layout: &DockLayout,
        side: DockSide,
        width: f32,
        report: &mut DockFrameReport,
        body: &mut impl FnMut(&PanelId, &mut egui::Ui),
    ) {
        let panel = match side {
            DockSide::Left => egui::Panel::left(egui::Id::new(("egui-shell-dock", side.key()))),
            DockSide::Right => egui::Panel::right(egui::Id::new(("egui-shell-dock", side.key()))),
        };

        panel
            // ★ `exact_size`, and only `exact_size`. See the module
            // header's R128 section: this is the one API that makes the
            // dock's outer width content-independent, and therefore the
            // one that keeps a fit-to-viewport zoom next door from
            // oscillating. `resizable` is off because the dock supplies
            // its own splitter, whose drag writes to the layout the
            // application saves — `egui`'s own resize handle would store
            // the width in `egui`'s memory, where nothing can persist it.
            .exact_size(width)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::NONE.fill(ctx.theme.palette.panel))
            .show(ui, |ui| {
                // Any `ScrollArea` a panel body creates inherits this.
                //
                // `D:/dev/rag/egui/scrollstyle_solid_draws_the_handle_in_bg_fill_which_is_invisible_on_a_light_panel.md`:
                // the default is `floating()` — a 2 pt sliver with
                // `dormant_handle_opacity: 0.0`, i.e. fully transparent
                // until hovered — and `solid()` alone still draws the
                // handle from `widgets.inactive.bg_fill`, which on a
                // light panel is near-white on near-white. Both halves
                // are fixed here, once, for every panel body, rather
                // than being rediscovered per panel.
                let mut scroll = egui::style::ScrollStyle::solid();
                scroll.foreground_color = true;
                scroll.bar_width = 10.0;
                ui.style_mut().spacing.scroll = scroll;

                let area = ui.max_rect();
                ctx.reporter.report(ui, area, || report::side(side));
                // ★ The banner is reserved off the TOP before the columns are
                // resolved, so a side with one takes its height from the
                // stacks once rather than painting over them. See
                // [`banner`]'s header for why it is chrome and not a stack.
                let area = banner::draw(ui, ctx, side, area, self.banner.as_mut());
                self.draw_side_contents(ui, ctx, layout, side, area, report, body);
                // ★ The collapse chevron, drawn LAST and OVER the columns.
                //
                // Over rather than inside, because inserting it into the column
                // layout would take height from a panel body on every frame —
                // and it is dock chrome, not a panel's content.
                collapse::draw_collapse(ctx, ui, side, area);
            });
    }

    /// Lay out the side splitter and the columns within a side's rect.
    #[allow(clippy::too_many_arguments)]
    fn draw_side_contents(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut Ctx<'_>,
        layout: &DockLayout,
        side: DockSide,
        area: Rect,
        report: &mut DockFrameReport,
        body: &mut impl FnMut(&PanelId, &mut egui::Ui),
    ) {
        // The side's own width handle sits on the INNER edge — the one
        // facing the application's content — because that is the edge the
        // operator is dragging *against*. A handle on the window edge
        // would sit under the OS resize border, which is failure mode #1
        // (*never nest a drag handle under OS chrome that looks the
        // same*) reproduced exactly.
        let t = plan::SPLITTER_THICKNESS;
        let (columns_rect, handle_rect) = match side {
            DockSide::Left => (
                Rect::from_min_max(area.min, egui::pos2(area.right() - t, area.bottom())),
                Rect::from_min_max(egui::pos2(area.right() - t, area.top()), area.max),
            ),
            DockSide::Right => (
                Rect::from_min_max(egui::pos2(area.left() + t, area.top()), area.max),
                Rect::from_min_max(area.min, egui::pos2(area.left() + t, area.bottom())),
            ),
        };

        let outcome = splitter::splitter(
            ui,
            ctx.id("side-split", side, 0, 0),
            handle_rect,
            Axis::Horizontal,
            &ctx.theme,
        );
        ctx.reporter
            .report(ui, handle_rect, || report::side_splitter(side));
        if outcome.changed() {
            // Dragging the right dock's handle rightwards makes it
            // NARROWER. Getting this sign wrong produces a dock that runs
            // away from the pointer, which is the kind of defect that is
            // obvious in one second of use and invisible to every
            // headless test — hence the sign lives here, once, beside its
            // reason.
            let delta = match side {
                DockSide::Left => outcome.delta,
                DockSide::Right => -outcome.delta,
            };
            ctx.intents.push(Intent::DragSide { side, delta });
        }

        let side_layout = layout.side(side);
        let shares: Vec<f32> = side_layout.columns.iter().map(|c| c.share).collect();
        let spans = plan::resolve_spans(
            &shares,
            columns_rect.width(),
            plan::MIN_COLUMN_WIDTH,
            plan::SPLITTER_THICKNESS,
        );

        let mut x = columns_rect.left();
        for (i, span) in spans.iter().enumerate() {
            let rect = Rect::from_min_size(
                egui::pos2(x, columns_rect.top()),
                Vec2::new(*span, columns_rect.height()),
            );
            ctx.reporter.report(ui, rect, || report::column(side, i));
            self.draw_column(ui, ctx, layout, side, i, rect, report, body);
            x += span;

            if i + 1 < spans.len() {
                let split = Rect::from_min_size(
                    egui::pos2(x, columns_rect.top()),
                    Vec2::new(plan::SPLITTER_THICKNESS, columns_rect.height()),
                );
                let outcome = splitter::splitter(
                    ui,
                    ctx.id("col-split", side, i, 0),
                    split,
                    Axis::Horizontal,
                    &ctx.theme,
                );
                ctx.reporter
                    .report(ui, split, || report::column_splitter(side, i));
                // A double-click and a drag are different gestures, so
                // they are different intents — never both in one frame.
                if outcome.equalize {
                    ctx.intents
                        .push(Intent::EqualizeColumns { side, boundary: i });
                } else if outcome.changed() {
                    ctx.intents.push(Intent::DragColumns {
                        side,
                        boundary: i,
                        delta: outcome.delta,
                    });
                }
                x += plan::SPLITTER_THICKNESS;
            }
        }
    }

    /// Lay out the stacks within one column's rect.
    #[allow(clippy::too_many_arguments)]
    fn draw_column(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut Ctx<'_>,
        layout: &DockLayout,
        side: DockSide,
        column: usize,
        rect: Rect,
        report: &mut DockFrameReport,
        body: &mut impl FnMut(&PanelId, &mut egui::Ui),
    ) {
        let stacks = &layout.side(side).columns[column].stacks;
        let shares: Vec<f32> = stacks.iter().map(|s| s.share).collect();
        let spans = plan::resolve_spans(
            &shares,
            rect.height(),
            plan::MIN_STACK_HEIGHT,
            plan::SPLITTER_THICKNESS,
        );

        let mut y = rect.top();
        for (i, span) in spans.iter().enumerate() {
            let stack_rect =
                Rect::from_min_size(egui::pos2(rect.left(), y), Vec2::new(rect.width(), *span));
            ctx.reporter
                .report(ui, stack_rect, || report::stack(side, column, i));
            self.draw_stack(
                ui, ctx, side, column, i, &stacks[i], stack_rect, report, body,
            );
            y += span;

            if i + 1 < spans.len() {
                let split = Rect::from_min_size(
                    egui::pos2(rect.left(), y),
                    Vec2::new(rect.width(), plan::SPLITTER_THICKNESS),
                );
                let outcome = splitter::splitter(
                    ui,
                    ctx.id("row-split", side, column, i),
                    split,
                    Axis::Vertical,
                    &ctx.theme,
                );
                ctx.reporter
                    .report(ui, split, || report::stack_splitter(side, column, i));
                if outcome.equalize {
                    ctx.intents.push(Intent::EqualizeStacks {
                        side,
                        column,
                        boundary: i,
                    });
                } else if outcome.changed() {
                    ctx.intents.push(Intent::DragStacks {
                        side,
                        column,
                        boundary: i,
                        delta: outcome.delta,
                    });
                }
                y += plan::SPLITTER_THICKNESS;
            }
        }
    }

    /// Draw one stack: its tab bar, then its **active** panel's body.
    #[allow(clippy::too_many_arguments)]
    fn draw_stack(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut Ctx<'_>,
        side: DockSide,
        column: usize,
        index: usize,
        stack: &Stack,
        rect: Rect,
        report: &mut DockFrameReport,
        body: &mut impl FnMut(&PanelId, &mut egui::Ui),
    ) {
        let bar_height = plan::TAB_BAR_HEIGHT.min(rect.height());
        let bar_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), bar_height));
        let body_rect =
            Rect::from_min_max(egui::pos2(rect.left(), rect.top() + bar_height), rect.max);

        let outcome = tabs::tab_bar(ui, ctx, side, column, index, stack, bar_rect);
        report.panels_overflowed += outcome.hidden;
        if outcome.overflow_drawn {
            report.overflow_menus += 1;
        }

        // ★ ONE body, the active tab's. Failure mode #3's design rule —
        // *size a container to its active child* — is honoured by there
        // being nothing else to size it to: an inactive tab's body is
        // never constructed, so it can neither impose a width nor consume
        // a frame's work.
        //
        // The RAG entry
        // `only_the_active_tab_is_emitted_so_scripted_harnesses_cannot_reach_other_tabs.md`
        // names the consequence honestly, and it is a consequence worth
        // paying for: a harness CANNOT observe a backgrounded panel, and
        // must select its tab first. The alternative — emit everything
        // and hide it — *"converts a keyboard-navigation improvement into
        // a keyboard-navigation regression with no visual symptom at
        // all"*, because every hidden control re-enters the focus chain.
        // The harness verb is [`DockState::activate`], and
        // [`DockFrameReport::panels_drawn`] is what tells a harness which
        // panels it can currently see.
        let Some(panel) = stack.active_panel().cloned() else {
            return;
        };
        if body_rect.height() <= 0.0 || body_rect.width() <= 0.0 {
            return;
        }

        ui.scope_builder(
            UiBuilder::new()
                .id_salt(ctx.id("body", side, column, index))
                .max_rect(body_rect)
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                // Clip to the compartment. A body that draws more than
                // fits is truncated, never accommodated — accommodating
                // it is what makes a panel content-driven, and a
                // content-driven panel next to a fit-to-viewport zoom is
                // the R128 feedback loop.
                ui.set_clip_rect(body_rect.intersect(ui.clip_rect()));
                body(&panel, ui);
            },
        );

        ctx.reporter.report(ui, body_rect, || report::body(&panel));
        report.panels_drawn.push(panel);
    }
}

/// Apply one frame's intents to the layout.
///
/// The **only** function in this module that takes `&mut DockLayout`.
/// Returns whether anything changed, which the application uses to decide
/// whether the layout is worth persisting.
///
/// Splitter drags are applied by resolving the *current* spans, moving
/// one boundary with [`plan::drag_boundary`], and converting back — which
/// is the one place [`plan::spans_to_shares`] may be called, and its
/// documentation says why.
fn apply(layout: &mut DockLayout, intents: &[Intent], report: &mut DockFrameReport) -> bool {
    if intents.is_empty() {
        return false;
    }
    let before = layout.clone();

    for intent in intents {
        match intent {
            // ★★ Collapse a side, or bring it back. One toggle, two controls:
            // the chevron on an open side and the rail on a shut one, neither
            // of which can be pressed in the state the other lives in.
            Intent::ToggleSide(side) => {
                let s = match side {
                    DockSide::Left => &mut layout.left,
                    DockSide::Right => &mut layout.right,
                };
                s.visible = !s.visible;
                // The arrangement is UNTOUCHED. Collapsing is a view state, not
                // a structural edit — which is the whole difference between
                // "collapse the dock" and "reset the dock", and the reason an
                // operator can minimise a side and get their columns back
                // exactly as they left them.
                //
                // No `changed` flag: this function compares the whole layout
                // against a clone taken before the loop, so a mutation IS the
                // signal. One place decides "did anything move", which is what
                // stops a new intent forgetting to say so.
            }
            Intent::Activate(panel) => {
                if layout.activate(panel) {
                    report.activated = Some(panel.clone());
                }
            }
            Intent::Close(panel) => {
                if layout.close(panel) {
                    report.closed = Some(panel.clone());
                }
            }
            Intent::DragSide { side, delta } => {
                let s = layout.side_mut(*side);
                s.width_pts = (s.width_pts + delta).max(plan::MIN_SIDE_WIDTH);
            }
            Intent::DragColumns {
                side,
                boundary,
                delta,
            } => {
                let s = layout.side_mut(*side);
                let shares: Vec<f32> = s.columns.iter().map(|c| c.share).collect();
                // Resolved against a nominal total rather than the real
                // one. The real width is not available here — this runs
                // after the frame — and it does not need to be: a drag
                // of `delta` points is a fraction of the side's width,
                // and the side's width is `width_pts`. Using it keeps the
                // delta in the same units the operator moved the pointer
                // in.
                let total = s.width_pts.max(plan::MIN_SIDE_WIDTH);
                let mut spans = plan::resolve_spans(
                    &shares,
                    total,
                    plan::MIN_COLUMN_WIDTH,
                    plan::SPLITTER_THICKNESS,
                );
                plan::drag_boundary(&mut spans, *boundary, *delta, plan::MIN_COLUMN_WIDTH);
                for (c, share) in s.columns.iter_mut().zip(plan::spans_to_shares(&spans)) {
                    c.share = share;
                }
            }
            Intent::EqualizeColumns { side, boundary } => {
                let s = layout.side_mut(*side);
                equalize(
                    &mut s
                        .columns
                        .iter_mut()
                        .map(|c| &mut c.share)
                        .collect::<Vec<_>>(),
                    *boundary,
                );
            }
            Intent::DragStacks {
                side,
                column,
                boundary,
                delta,
            } => {
                let s = layout.side_mut(*side);
                let Some(col) = s.columns.get_mut(*column) else {
                    continue;
                };
                let shares: Vec<f32> = col.stacks.iter().map(|s| s.share).collect();
                // The column's height in points is not known here either.
                // A nominal total works because `drag_boundary`'s delta
                // and the minimums are both in points and the conversion
                // back to shares is scale-free; the only visible effect
                // of a nominal total is that a drag near the minimum
                // resists slightly sooner or later than the pixel it was
                // drawn at. A real height would have to be smuggled out
                // of the draw phase, which is exactly the `&mut`-during-
                // draw this design refuses.
                let total = NOMINAL_COLUMN_HEIGHT;
                let mut spans = plan::resolve_spans(
                    &shares,
                    total,
                    plan::MIN_STACK_HEIGHT,
                    plan::SPLITTER_THICKNESS,
                );
                plan::drag_boundary(&mut spans, *boundary, *delta, plan::MIN_STACK_HEIGHT);
                for (st, share) in col.stacks.iter_mut().zip(plan::spans_to_shares(&spans)) {
                    st.share = share;
                }
            }
            Intent::EqualizeStacks {
                side,
                column,
                boundary,
            } => {
                let s = layout.side_mut(*side);
                let Some(col) = s.columns.get_mut(*column) else {
                    continue;
                };
                equalize(
                    &mut col
                        .stacks
                        .iter_mut()
                        .map(|s| &mut s.share)
                        .collect::<Vec<_>>(),
                    *boundary,
                );
            }
        }
    }

    layout.normalize();
    *layout != before
}

/// The height a stack drag is resolved against when the real one is not
/// available. See the call site.
const NOMINAL_COLUMN_HEIGHT: f32 = 800.0;

/// Give the two children either side of `boundary` equal share, leaving
/// every other child alone — failure mode #7 applied to the double-click
/// gesture as well as to the drag.
fn equalize(shares: &mut [&mut f32], boundary: usize) {
    if boundary + 1 >= shares.len() {
        return;
    }
    let mean = (*shares[boundary] + *shares[boundary + 1]) / 2.0;
    *shares[boundary] = mean;
    *shares[boundary + 1] = mean;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two columns on the left, one stack each; one column on the right.
    fn sample() -> DockLayout {
        DockLayout::new(
            SideLayout::new([
                Column::new([Stack::new("pages"), Stack::new("tools")]),
                Column::new([Stack::tabbed(["layers", "bookmarks"])]),
            ]),
            SideLayout::single("objects"),
        )
    }

    fn registry() -> PanelRegistry {
        let mut r = PanelRegistry::new();
        for (id, label) in [
            ("pages", "Pages"),
            ("tools", "Tools"),
            ("layers", "Layers"),
            ("bookmarks", "Bookmarks"),
            ("objects", "Objects"),
        ] {
            r.register(
                PanelInfo::new(id, label)
                    .with_tooltip(format!("{label} — the thing you reach for when working")),
            );
        }
        r
    }

    /// Render one frame at the given window size, returning the frame
    /// report and every panel body's `Ui` width.
    fn frame(state: &mut DockState, window: Vec2) -> (DockFrameReport, Vec<(PanelId, Rect)>) {
        let registry = registry();
        let mut bodies: Vec<(PanelId, Rect)> = Vec::new();
        let mut report = DockFrameReport::default();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, window)),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            report = Dock::new()
                .with_registry(&registry)
                .show(ui, state, |panel, ui| {
                    bodies.push((panel.clone(), ui.max_rect()));
                });
        });
        (report, bodies)
    }

    /// ★ **Only the ACTIVE tab's body is drawn.**
    ///
    /// Failure mode #3's design rule — *size a container to its active
    /// child* — stated as behaviour rather than as arithmetic. Four
    /// stacks hold five panels; four bodies are constructed.
    #[test]
    fn exactly_one_body_per_stack_is_drawn_and_it_is_the_active_one() {
        let mut state = DockState::new(sample());
        let (report, bodies) = frame(&mut state, Vec2::new(1280.0, 800.0));
        let drawn: Vec<&str> = bodies.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(drawn.len(), 4, "four stacks, four bodies: {drawn:?}");
        assert!(
            drawn.contains(&"layers"),
            "the active tab of the tabbed stack"
        );
        assert!(
            !drawn.contains(&"bookmarks"),
            "a backgrounded tab must not be constructed: {drawn:?}"
        );
        assert_eq!(report.panels_drawn.len(), 4);
    }

    /// ★★ **A collapsed side draws NO PANELS and still leaves a rail.**
    ///
    /// This test asserted `sides_drawn == [Left]` until 2026-08-20 — that a
    /// hidden side contributed nothing at all. That was the behaviour, and it
    /// was the defect:
    ///
    /// > *"add the little tabs that allow the left and right panels to be
    /// > minimized."* — the operator, 2026-08-20
    ///
    /// He was asking for the affordance in both directions, and the half that
    /// was missing is the way back. A side that drew nothing could only be
    /// restored from a ribbon command the operator had to know existed, so a
    /// panel collapsed by accident was a panel **lost** rather than minimised.
    /// Every program in the class leaves a rail: VS Code's activity bar, Visual
    /// Studio's auto-hide tabs, Photoshop's collapsed dock strip.
    ///
    /// So the report now lists the side — because a rail IS on screen and the
    /// report is the honest answer to *"what is on screen"* — and the
    /// assertions that matter are unchanged and are the real content of this
    /// test: **no panel body is constructed, and nothing on that side counts as
    /// on-screen.** A collapsed side must cost nothing but its rail.
    #[test]
    fn a_collapsed_side_draws_no_panels_and_leaves_a_rail() {
        let mut layout = sample();
        layout.right.visible = false;
        let mut state = DockState::new(layout);
        let (report, bodies) = frame(&mut state, Vec2::new(1280.0, 800.0));
        assert!(
            report.sides_drawn.contains(&DockSide::Right),
            "the rail is on screen, so the report must say so: {:?}",
            report.sides_drawn
        );
        assert!(
            !bodies.iter().any(|(p, _)| p.as_str() == "objects"),
            "a collapsed side must construct no panel body"
        );
        assert!(
            !state.is_on_screen(&PanelId::new("objects")),
            "and nothing on it is on screen"
        );
    }

    /// ★ **An EMPTY side draws no rail either**, and the distinction from a
    /// collapsed one is the whole of why this is a separate test.
    ///
    /// A collapsed side has panels waiting behind it, so a rail is a promise it
    /// can keep. An empty side has nothing to bring back, and a control that
    /// opened an empty compartment would be an affordance for something that
    /// cannot happen — the no-placeholders rule, which this crate holds to as
    /// strictly as its host does.
    #[test]
    fn an_empty_side_leaves_no_rail_because_there_is_nothing_to_bring_back() {
        let mut state = DockState::new(DockLayout::new(
            SideLayout::single("pages"),
            SideLayout::new([]),
        ));
        state.layout.right.visible = false;
        let (report, _) = frame(&mut state, Vec2::new(1280.0, 800.0));
        assert_eq!(
            report.sides_drawn,
            vec![DockSide::Left],
            "an empty collapsed side is not a rail, it is nothing"
        );
    }

    /// An empty side draws nothing — not an empty bordered stripe.
    #[test]
    fn an_empty_side_draws_nothing_at_all() {
        let mut state = DockState::new(DockLayout::new(
            SideLayout::single("pages"),
            SideLayout::new([]),
        ));
        let (report, _) = frame(&mut state, Vec2::new(1280.0, 800.0));
        assert_eq!(report.sides_drawn, vec![DockSide::Left]);
    }

    /// ★ **Failure mode #6, end to end: a round trip through a narrow
    /// window changes nothing.**
    ///
    /// The observed defect is that un-maximising and re-maximising loses
    /// the panel proportions. Here the whole `DockState` is compared for
    /// equality before and after three frames at three window sizes — so
    /// this catches a write-back anywhere in the module, not only in the
    /// span arithmetic.
    #[test]
    fn the_layout_survives_a_round_trip_through_a_narrow_window() {
        let mut state = DockState::new(sample());
        let before = state.layout().clone();
        frame(&mut state, Vec2::new(1920.0, 1080.0));
        frame(&mut state, Vec2::new(700.0, 500.0));
        frame(&mut state, Vec2::new(1920.0, 1080.0));
        assert_eq!(
            state.layout(),
            &before,
            "drawing at three window sizes perturbed the stored layout"
        );
    }

    /// A frame that does nothing reports no change, so an application
    /// persisting on `layout_changed` does not rewrite its file every
    /// frame.
    #[test]
    fn an_idle_frame_reports_no_change() {
        let mut state = DockState::new(sample());
        let (report, _) = frame(&mut state, Vec2::new(1280.0, 800.0));
        assert!(!report.layout_changed);
        assert!(report.activated.is_none());
        assert!(report.closed.is_none());
    }

    /// The dock's drawn width comes from the layout and is clamped to the
    /// window, and the body rects prove the clamp reached the drawing
    /// rather than only the arithmetic.
    #[test]
    fn a_dock_wider_than_the_window_allows_is_clamped_when_drawn() {
        let mut layout = sample();
        layout.left.width_pts = 4000.0;
        layout.right.visible = false;
        let mut state = DockState::new(layout);
        let (_, bodies) = frame(&mut state, Vec2::new(1000.0, 800.0));
        for (panel, rect) in &bodies {
            assert!(
                rect.right() <= 1000.0 * plan::MAX_SIDE_FRACTION + 1.0,
                "{panel} was drawn at {rect:?}, past the clamp"
            );
        }
        assert_eq!(
            state.layout().left.width_pts,
            4000.0,
            "the clamp must not be written back"
        );
    }

    /// Applying a side drag changes the width, and only that side's.
    #[test]
    fn a_side_drag_changes_one_sides_width_and_nothing_else() {
        let mut layout = sample();
        let before_right = layout.right.width_pts;
        let mut report = DockFrameReport::default();
        let changed = apply(
            &mut layout,
            &[Intent::DragSide {
                side: DockSide::Left,
                delta: 40.0,
            }],
            &mut report,
        );
        assert!(changed);
        assert!((layout.left.width_pts - 320.0).abs() < 0.01);
        assert_eq!(layout.right.width_pts, before_right);
    }

    /// A side cannot be dragged below its floor.
    #[test]
    fn a_side_drag_cannot_go_below_the_minimum() {
        let mut layout = sample();
        let mut report = DockFrameReport::default();
        apply(
            &mut layout,
            &[Intent::DragSide {
                side: DockSide::Left,
                delta: -5000.0,
            }],
            &mut report,
        );
        assert!((layout.left.width_pts - plan::MIN_SIDE_WIDTH).abs() < 0.01);
    }

    /// ★ **Failure mode #7 through the whole apply path: a column drag
    /// leaves a third column untouched.**
    ///
    /// [`plan::drag_boundary`]'s own test proves the slice arithmetic;
    /// this proves nothing between the intent and the model
    /// renormalises the others on the way past — which is exactly how
    /// coupled splitters happen, and is invisible to a reading of either
    /// half alone.
    #[test]
    fn a_column_drag_leaves_the_other_columns_shares_alone() {
        let mut layout = DockLayout::new(
            SideLayout::new([
                Column::new([Stack::new("a")]),
                Column::new([Stack::new("b")]),
                Column::new([Stack::new("c")]),
            ])
            .with_width(600.0),
            SideLayout::none(),
        );
        let mut report = DockFrameReport::default();
        apply(
            &mut layout,
            &[Intent::DragColumns {
                side: DockSide::Left,
                boundary: 0,
                delta: 30.0,
            }],
            &mut report,
        );
        let shares: Vec<f32> = layout.left.columns.iter().map(|c| c.share).collect();
        assert!(shares[0] > shares[1], "the dragged pair moved: {shares:?}");
        assert!(
            (shares[2] - 1.0 / 3.0).abs() < 0.02,
            "the third column's share moved: {shares:?}"
        );
    }

    /// A double-click equalises exactly two neighbours.
    #[test]
    fn equalising_touches_exactly_two_neighbours() {
        let mut a = 3.0_f32;
        let mut b = 1.0_f32;
        let mut c = 9.0_f32;
        equalize(&mut [&mut a, &mut b, &mut c], 0);
        assert!((a - 2.0).abs() < 0.01);
        assert!((b - 2.0).abs() < 0.01);
        assert!((c - 9.0).abs() < 0.01, "a distant child was equalised too");
    }

    /// Activating through an intent raises the tab and reports it.
    #[test]
    fn activating_a_backgrounded_tab_raises_it_and_is_reported() {
        let mut layout = sample();
        let mut report = DockFrameReport::default();
        let bookmarks = PanelId::new("bookmarks");
        let changed = apply(
            &mut layout,
            &[Intent::Activate(bookmarks.clone())],
            &mut report,
        );
        assert!(changed);
        assert!(layout.is_active(&bookmarks));
        assert_eq!(report.activated, Some(bookmarks));
    }

    /// Closing through an intent removes the panel and reports it.
    #[test]
    fn closing_a_panel_removes_it_and_is_reported() {
        let mut layout = sample();
        let mut report = DockFrameReport::default();
        let tools = PanelId::new("tools");
        assert!(apply(
            &mut layout,
            &[Intent::Close(tools.clone())],
            &mut report
        ));
        assert!(!layout.contains(&tools));
        assert_eq!(report.closed, Some(tools));
    }

    /// An intent naming a panel that is no longer mounted is a no-op, not
    /// a panic — it happens whenever a menu outlives the frame that
    /// opened it.
    #[test]
    fn an_intent_for_a_vanished_panel_is_a_no_op() {
        let mut layout = sample();
        let mut report = DockFrameReport::default();
        let changed = apply(
            &mut layout,
            &[
                Intent::Activate(PanelId::new("ghost")),
                Intent::Close(PanelId::new("ghost")),
                Intent::DragStacks {
                    side: DockSide::Right,
                    column: 40,
                    boundary: 0,
                    delta: 5.0,
                },
            ],
            &mut report,
        );
        assert!(!changed);
        assert!(report.activated.is_none());
    }

    /// ★ **Panel bodies inherit a scroll style whose handle is visible.**
    ///
    /// `D:/dev/rag/egui/scrollstyle_solid_draws_the_handle_in_bg_fill_...md`
    /// records two independent reasons a working `ScrollArea` shows no
    /// scrollbar — the default is `floating()` (transparent when the
    /// pointer is elsewhere), and `solid()` alone draws the handle in
    /// `widgets.inactive.bg_fill`, which on a light panel is near-white
    /// on near-white. Both were hit in sequence on one surface, and the
    /// first fix appeared not to work because the second was waiting
    /// behind it. Fixed once here, for every panel body, and asserted so
    /// it stays fixed.
    #[test]
    fn a_panel_body_inherits_a_visible_scrollbar_style() {
        let mut state = DockState::new(sample());
        let registry = registry();
        let mut styles = Vec::new();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1280.0, 800.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            Dock::new()
                .with_registry(&registry)
                .show(ui, &mut state, |_panel, ui| {
                    styles.push(ui.spacing().scroll);
                });
        });
        assert!(!styles.is_empty(), "no body was drawn");
        for scroll in styles {
            assert!(
                scroll.foreground_color,
                "the handle would be drawn in bg_fill — invisible on a light panel"
            );
            assert!(
                scroll.floating_allocated_width > 0.0 || !scroll.floating,
                "a floating bar is transparent until hovered"
            );
        }
    }

    /// A body is clipped to its compartment, so an over-eager panel
    /// cannot grow the dock — the content-driven half of R128.
    #[test]
    fn a_panel_body_is_clipped_to_its_compartment() {
        let mut state = DockState::new(DockLayout::new(
            SideLayout::single("pages").with_width(200.0),
            SideLayout::none(),
        ));
        let registry = registry();
        let mut clip = Rect::NOTHING;
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1280.0, 800.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            Dock::new()
                .with_registry(&registry)
                .show(ui, &mut state, |_panel, ui| {
                    ui.label("a label that is considerably wider than two hundred points");
                    clip = ui.clip_rect();
                });
        });
        assert!(
            clip.width() <= 200.0 + 1.0,
            "the body was not clipped to the dock: {clip:?}"
        );
        assert_eq!(
            state.layout().left.width_pts,
            200.0,
            "content must never change the dock's width"
        );
    }

    /// The application can drive the dock without a registry at all; tabs
    /// then carry their own ids, which is ugly and truthful.
    #[test]
    fn a_dock_with_no_registry_still_draws() {
        let mut state = DockState::new(sample());
        let mut drawn = 0;
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1280.0, 800.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            Dock::new().show(ui, &mut state, |_panel, _ui| drawn += 1);
        });
        assert_eq!(drawn, 4);
    }
}
