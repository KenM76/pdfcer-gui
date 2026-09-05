//! # `dock::floatwin` — the window a floated panel is drawn in
//!
//! [`super::float`] owns *whether* a panel is floating and *where it came
//! from*. This module owns the OS window: opening it, placing it,
//! remembering what the operator did to it, and drawing the one control
//! that offers the way back.
//!
//! ## ★★★ Why this is a second call and not part of `Dock::show`
//!
//! [`super::Dock::show`] is handed a `&mut egui::Ui` and is called from
//! *inside* the application's layout — between the ribbon and the canvas,
//! with side panels being composed around it. A child viewport must be
//! opened from the application's **top-level frame**, beside its dialogs,
//! for the same reason `dialogs::host` is called there: opening one runs a
//! complete nested pass for another window, and doing that from inside a
//! half-composed panel closure makes the parent's remaining layout depend
//! on what a different window did.
//!
//! So the application calls two things per frame:
//!
//! ```no_run
//! # use egui_shell::dock::{Dock, DockState, PanelId, PanelRegistry};
//! # fn frame(ui: &mut egui::Ui, ctx: &egui::Context, state: &mut DockState,
//! #          registry: &PanelRegistry,
//! #          mut body: impl FnMut(&PanelId, &mut egui::Ui)) {
//! // …between the ribbon and the canvas:
//! Dock::new().with_registry(registry).show(ui, state, &mut body);
//!
//! // …at the top of the frame, beside the dialogs:
//! Dock::new().with_registry(registry).show_floating(ctx, state, &mut body);
//! # }
//! ```
//!
//! **Forgetting the second one is a silent failure**, and it is precisely
//! the class this project has already shipped once: three panels laid out,
//! publishing correct rectangles, unreachable, with every gate green. The
//! answer is [`super::DockFrameReport::floats_undrawn`] — a number an
//! application asserts is zero — and its field documentation carries the
//! whole argument.
//!
//! ## ★★★ …and forgetting is not the only way a float window ends up blank
//!
//! **Added 2026-09-05, after a driven sweep reported *"a floated panel opens
//! an OS window and draws nothing inside it"*.**
//!
//! [`super::DockFrameReport::floats_undrawn`] catches the application that
//! never called [`super::Dock::show_floating`]. It cannot catch the case
//! one step along: the call IS made, the window IS opened, the background
//! IS painted, the header IS drawn — and `body` allocates nothing. Every
//! number this module produced before that date reports that frame as a
//! success, because every one of them is satisfied the moment the loop runs.
//!
//! ⚠ **A blank window is R9 broken at the scale of a whole window.** The
//! rule — *an unavailable capability renders nothing* — is about a control
//! inside a surface. An operator can read an absent button. Nobody can read
//! a blank window with a title bar; it is indistinguishable from a crash.
//!
//! ⇒ [`FloatFrameReport::empty_bodies`] is the second number, measured from
//! the body `Ui`'s [`egui::Ui::min_rect`] **after** `body` returns. Its
//! field documentation carries what it can and cannot see, and two tests in
//! this file falsify it in both directions.
//!
//! ★★ **The sweep's verdict was about the harness, not about this module.**
//! The check asserted *"a `ui-rect` tagged with the float's viewport"*, and
//! neither this module nor the panel it floated publishes one on the fixture
//! it was given — so the oracle could not have succeeded against a working
//! build either. The application publishes the tagged regions (see
//! `crate::app::surfaces::floating_panels` in the consuming crate); this
//! crate has no diagnostic channel and must not grow one, so what it can
//! honestly offer is a **value in a report**, which is this field. The
//! general lesson is in
//! `D:/dev/rag/egui/a_float_windows_emptiness_is_not_observable_from_any_number_the_dock_already_publishes.md`.
//!
//! ## ★★ The one-dispatcher rule survives, and that is the finding this
//! whole capability rests on
//!
//! `MODES_AND_PANELS.md` records it, and it is the reason tear-out is
//! cheap here and expensive elsewhere:
//!
//! > I expected `Send + Sync + 'static` on the viewport callback to force
//! > app state behind an `Arc<Mutex<…>>`, which would have been fatal.
//! > `show_viewport_deferred` does carry that bound — but
//! > **`show_viewport_immediate` does not.** It takes `FnMut` with no
//! > lifetime bound, so it can be called from inside `App::ui` capturing
//! > `&mut self`. A torn-out panel therefore keeps the identical
//! > `panel_body(&mut self, panel, ui, actions)` signature as the docked
//! > one.
//!
//! ⇒ [`super::Dock::show_floating`] takes **the same `body` closure**
//! `show` takes. There is no float-specific panel API, no second
//! rendering path, and no duplicated open-state — which
//! [`super::mod`]'s own header names as what a previous float-or-dock dual
//! mode cost: *"two code paths for the same content, each duplicating
//! open-state, position/size and focus handling."*
//!
//! ## `FnMut`, and the crash that proves it must be
//!
//! `egui` may call a viewport callback **more than once per frame**,
//! whenever anything inside asks for a re-run at a size it has just
//! learned. `D:/dev/rag/egui/show_viewport_immediate_may_run_its_callback_twice_per_frame_so_a_fnonce_body_aborts.md`
//! records this project taking a `FnOnce` and `expect`-ing on the second
//! call — which turned an ordinary `egui` behaviour into a process abort
//! that took the operator's open documents with it.
//!
//! A panel body is already re-runnable within a frame: it draws from state
//! it borrows rather than consumes, which is what immediate mode requires
//! of every widget anyway. Nothing here needs to change for that; it is
//! recorded so that a future body which *cannot* be run twice is fixed at
//! the body rather than by reinstating a panic.
//!
//! ## Three things the window must do that a naive one would not
//!
//! | | Why |
//! |---|---|
//! | **Paint its own background** | A viewport callback's `Ui` is the child window's ROOT and nothing has painted it. `D:/dev/rag/egui/a_viewport_callbacks_ui_is_the_child_windows_ROOT_so_nothing_paints_its_background.md` records eight dialogs shipping as dark text on near-black, invisible to every oracle except a screenshot. |
//! | **Assert its position only on the frame it opens** | `show_viewport_immediate` diffs the builder against last frame's and turns each change into a `ViewportCommand`. A position clause that runs every frame re-asserts a position read back from the window one frame late, which is a `SetWindowPos` per frame and drags the window back toward where the program thinks it is. |
//! | **Tag its rectangles with its own viewport** | A child viewport's `ui_rect`s are relative to *its* origin. Untagged, a harness reads them as the application window's and aims hundreds of points away. The shell has no diagnostic channel of its own, so the application does this **inside its own `body` closure** — it can recover the very same id from [`viewport_id`], which is public for exactly that. No new seam, and the `body` signature stays identical to the docked one, which is the property this whole capability rests on. |
//!
//! ## What the header strip is for
//!
//! A docked panel offers float / close / dock from its **tab**. A floated
//! panel has no tab, so the same three verbs need somewhere to live, and
//! "wherever else a user would reach for it" is the top of the window.
//!
//! The strip carries the panel's name and hands its `Response` to the same
//! [`super::TabMenu`] seam a tab does — so an application writes **one**
//! menu handler and both surfaces get it, with the same rows, the same
//! command ids and the same conditions. `Close` is additionally the OS
//! window's own close button, because that is the control every operator
//! reaches for first and a window that ignored it would be a window that
//! could not be shut.

use egui::{Pos2, Rect, Vec2, ViewportBuilder, ViewportClass, ViewportId};

use super::ctx::Intent;
use super::float;
use super::model::{DockLayout, PanelId};
use super::tab_menu::TabMenu;
use super::{Dock, DockState};
use crate::theme::Theme;

/// The padding between a floated panel's body and its window edge, in
/// points.
///
/// The same 12 pt `dialogs::host` settled on, and for the same reason
/// stated there: the `Ui` a viewport callback receives is the window's
/// root and nothing pads it, so without this every control in the panel
/// touches the frame. A constant rather than a theme metric because it
/// participates in nothing that could feed back into it.
pub const BODY_MARGIN_PTS: f32 = 10.0;

/// The height of the strip above a floated panel's body.
pub const HEADER_HEIGHT_PTS: f32 = 22.0;

/// What one frame of the float windows drew and what the operator did to
/// them.
///
/// Deliberately a separate type from [`super::DockFrameReport`] rather
/// than more fields on it: the two are produced by two calls at two points
/// in the frame, and a single struct would have half its fields stale
/// whichever of the two a caller happened to read.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FloatFrameReport {
    /// Every panel a window was actually drawn for.
    ///
    /// **The fact**, against [`super::DockFrameReport::floating`]'s claim.
    /// A panel here is one the operator can see; a panel in the claim and
    /// not here is one that is in the layout and nowhere on the desktop.
    pub drawn: Vec<PanelId>,
    /// How many of those windows the platform gave a real OS window rather
    /// than falling back to an embedded one.
    ///
    /// `egui` falls back silently when the backend has no multi-viewport
    /// support — and, importantly, in a headless test context. Counted
    /// rather than assumed so that "it floated" and "it drew inside the
    /// main window because there is no window system" are distinguishable
    /// in a report rather than only in a screenshot.
    pub real_windows: usize,
    /// ★★★ **Every panel whose window was drawn and whose BODY laid out
    /// nothing** — an open window with an empty panel in it.
    ///
    /// # Why this is a second number and not an implication of the first
    ///
    /// [`Self::drawn`] answers *"was a window opened for this panel"*, and
    /// [`super::DockFrameReport::floats_undrawn`] answers *"is a panel in
    /// the layout with no window at all"*. Neither of them can tell an
    /// **empty** window from a full one, because both are satisfied the
    /// moment [`Dock::show_floating`] runs — the window opens, the
    /// background is painted, the header draws, and `body` is called. If
    /// `body` then draws nothing, every count above still reports success
    /// and the operator is looking at a blank rectangle with a title bar.
    ///
    /// ⚠ That is R9 (*an unavailable capability renders **nothing***)
    /// broken by accident in its worst possible form: the rule says a panel
    /// with nothing to say must draw nothing, and this is a **whole
    /// window** with nothing in it. The rule was written about a control
    /// inside a surface, not about the surface itself, and a window is not
    /// something an operator can read as "deliberately blank".
    ///
    /// # What is measured, exactly, and what it cannot see
    ///
    /// The body `Ui`'s [`egui::Ui::min_rect`] after `body` returns — the
    /// extent of everything the panel **allocated**. A `Ui` that has been
    /// handed to a body and had nothing allocated in it keeps the empty
    /// rectangle it was constructed with, so a zero width or height is
    /// exact rather than heuristic.
    ///
    /// ★ It measures **allocation**, not paint. A body that only calls
    /// `ui.painter()` without allocating is reported here as empty even
    /// though pixels reached the window. That is the honest bound of a
    /// layout-level measurement, and it is the right direction to be wrong
    /// in: it over-reports rather than under-reports, so it can never call
    /// a genuinely blank window full.
    pub empty_bodies: Vec<PanelId>,
    /// The panel whose window the operator closed this frame, if any.
    pub closed: Option<PanelId>,
    /// The panel the operator docked back this frame, if any.
    pub docked: Option<PanelId>,
    /// Whether the layout changed and is therefore worth saving.
    ///
    /// True for a dock-back or a close; **also** true when a window was
    /// moved or resized, which is the change an operator makes most often
    /// and the one that would otherwise be forgotten on quit.
    pub layout_changed: bool,
}

impl Dock<'_> {
    /// **Draw every floating panel's window**, calling `body` once per
    /// window with the same signature [`Dock::show`] uses.
    ///
    /// Call this from the application's top-level frame, beside its
    /// dialogs — see the module header for why it cannot live inside
    /// [`Dock::show`].
    ///
    /// # What it does per open float, in order
    ///
    /// 1. Decides the window's position, honouring a remembered one only
    ///    if it is still plausible ([`float::honour_position`]).
    /// 2. Opens or updates the viewport, asserting the position only on
    ///    the frame the window opens.
    /// 3. Paints the background, because nothing else will.
    /// 4. Draws the header strip and offers its `Response` to the
    ///    application's tab-menu handler — the same seam a tab uses.
    /// 5. Calls `body`.
    /// 6. Reads the window's geometry back and records it as an intent.
    /// 7. Applies every intent afterwards, in one place, exactly as
    ///    [`Dock::show`] does.
    pub fn show_floating(
        &mut self,
        ctx: &egui::Context,
        state: &mut DockState,
        mut body: impl FnMut(&PanelId, &mut egui::Ui),
    ) -> FloatFrameReport {
        let mut report = FloatFrameReport::default();
        if state.layout.floating.is_empty() {
            state.floats_drawn = 0;
            return report;
        }

        // Phase 1: snapshot, exactly as `show` does. Everything drawn this
        // frame is drawn from it, so one frame shows one truth even though
        // a header control may be asking to change it as we go.
        let snapshot = state.layout.clone();
        let app_outer = ctx
            .input(|i| i.viewport().outer_rect)
            // A window whose geometry the platform has not reported yet —
            // the first frame, and a headless harness. Placing relative to
            // the origin is wrong but bounded, and the position is
            // re-derived on the next frame when the real rectangle exists.
            .unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 800.0)));
        let monitor = ctx.input(|i| i.viewport().monitor_size);
        // ★ The panel the operator asked to bring forward, read from the
        // previous frame's dock report. `DockLayout::activate` deliberately
        // does nothing structural to a float — there is no tab to select —
        // so raising the window is this module's half of that verb, and it
        // is a `ViewportCommand` rather than a layout edit.
        let raise = state.last_frame.activated.clone();

        let mut intents: Vec<Intent> = Vec::new();
        let mut unplaced = 0usize;
        let mut tab_menu = self.tab_menu.take();

        for f in &snapshot.floating {
            let size = float::clamp_size(f.size_pts);
            let position = match float::honour_position(f.pos_pts, size, app_outer, monitor) {
                Some(at) => at,
                None => {
                    let at = float::opening_position(app_outer, unplaced);
                    unplaced += 1;
                    at
                }
            };
            let id = viewport_id(&f.panel);
            let opening = !ctx.data(|d| d.get_temp::<bool>(seen_key(&f.panel)).unwrap_or(false));
            ctx.data_mut(|d| d.insert_temp(seen_key(&f.panel), true));

            let mut builder = ViewportBuilder::default()
                .with_title(self.float_title(&f.panel))
                .with_inner_size(Vec2::new(size[0], size[1]))
                .with_min_inner_size(Vec2::new(float::MIN_SIZE_PTS[0], float::MIN_SIZE_PTS[1]))
                // ★ A panel window IS in the window list, unlike a dialog's
                // decision to be there for findability alone. A floated
                // panel is a place the operator works, not a transaction
                // they finish, so it minimises, it restores, and it is
                // reachable from the taskbar when it falls behind
                // something.
                .with_taskbar(true);
            // ★★★ ASSERTED ONCE, on the frame the window opens.
            //
            // `show_viewport_immediate` diffs this builder against the
            // previous frame's and issues a `ViewportCommand` per changed
            // property. A position asserted every frame is therefore a
            // `SetWindowPos` every frame, fed by a value read back out of
            // the window one frame late — which drags the window toward
            // where the program thinks it is and fights the operator's
            // drag. `dialogs::host` records the same finding and the hour
            // spent hunting it.
            if opening {
                builder = builder.with_position(position);
            }

            let mut closed = false;
            let mut dock_back = false;
            let mut geometry: Option<(Option<[f32; 2]>, [f32; 2])> = None;
            // ★★ Set inside the callback, read after it, for `geometry`'s
            // reason: `egui` may run a viewport callback twice in one frame,
            // and the LAST run is the one whose answers the frame keeps.
            let mut body_extent = Vec2::ZERO;

            let class = ctx.show_viewport_immediate(id, builder, |ui, class| {
                // The callback's return value is `show_viewport_immediate`'s,
                // and `egui` keeps the LAST call's — which is what makes this
                // safe under the twice-per-frame re-run described in the
                // header. The class is the honest answer to "did the platform
                // give us a real window", and it is only knowable in here.
                // ★★ Nothing has painted this. A viewport callback's `Ui`
                // is the child window's ROOT — the position
                // `eframe::App::ui` occupies for the main window — and the
                // application's `CentralPanel`, which is what fills the
                // background in the main window, is not here. Eight
                // dialogs shipped for an hour as dark text on near-black
                // for exactly this, invisible to every oracle but a
                // screenshot.
                let theme = Theme::of(ui.ctx());
                ui.painter()
                    .rect_filled(ui.max_rect(), 0.0, theme.palette.panel);
                // ★ Bring this window to the front when the application
                // activated its panel — "View ▸ Panels ▸ Layers" on a panel
                // that is already floating behind the main window. The layout
                // has nothing to change (there is no tab to select), so
                // `DockLayout::activate` returns `true` and does nothing, and
                // the raise is this: a viewport command, sent only on the
                // frame the activation was reported. Sending it every frame
                // would hold the window in front of anything the operator
                // switched to, including another application.
                //
                // Guarded on `Immediate` because an embedded fallback has no
                // window to raise and the command would go to the parent.
                if class == ViewportClass::Immediate
                    && raise.as_ref().is_some_and(|r| r == &f.panel)
                {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                let inner = ui.max_rect().shrink(BODY_MARGIN_PTS);
                let (header_rect, body_rect) = split_header(inner);
                if let Some(handler) = tab_menu.as_deref_mut() {
                    let response = self.draw_header(ui, header_rect, &f.panel, &theme);
                    let mut tab = TabMenu::new(&f.panel, &response);
                    handler(&mut tab);
                    if tab.close_requested() {
                        closed = true;
                    }
                    if tab.dock_requested() {
                        dock_back = true;
                    }
                } else {
                    // ★ No handler, so the shell owns the strip — and the
                    // built-in offer is **Dock**, not Close. The OS window
                    // already has a close button; it does not have a way
                    // back into the dock, and a float window with no route
                    // home is the state `MODES_AND_PANELS.md` failure mode
                    // #12 calls table stakes to avoid. This mirrors
                    // `tabs.rs`'s built-in "Close": a consumer that has not
                    // adopted the tab-menu seam still gets a usable
                    // surface.
                    if self.draw_builtin_header(ui, header_rect, &f.panel, &theme) {
                        dock_back = true;
                    }
                }
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(body_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                child.set_clip_rect(body_rect);
                body(&f.panel, &mut child);
                // ★★★ **How much of the window the panel actually filled.**
                //
                // `min_rect` on a `Ui` nothing has allocated into is the
                // empty rectangle it was built with, so this is an exact
                // answer to *"did the body draw"* rather than a threshold.
                // See [`FloatFrameReport::empty_bodies`] for why the
                // question needs asking at all: every other number this
                // module produces is already satisfied by an open, blank
                // window.
                body_extent = child.min_rect().size();

                if class == ViewportClass::Immediate {
                    let (outer, inner_rect) = ui
                        .ctx()
                        .input(|i| (i.viewport().outer_rect, i.viewport().inner_rect));
                    // ★ The OUTER position and the INNER size, and the
                    // pairing is not arbitrary: `with_position` takes the
                    // outer corner while `with_inner_size` takes the client
                    // extent, so storing the inner origin would walk the
                    // window up-left by its title bar on every reopen, and
                    // storing the outer size would grow it by its
                    // decoration on every one.
                    geometry = Some((
                        outer.map(|r| [r.min.x, r.min.y]),
                        inner_rect.map(|r| [r.width(), r.height()]).unwrap_or(size),
                    ));
                    if ui.ctx().input(|i| i.viewport().close_requested()) {
                        closed = true;
                    }
                }
                class
            });

            report.drawn.push(f.panel.clone());
            if body_extent.x <= 0.0 || body_extent.y <= 0.0 {
                report.empty_bodies.push(f.panel.clone());
            }
            if class == ViewportClass::Immediate {
                report.real_windows += 1;
            }
            if let Some((pos, sz)) = geometry {
                intents.push(Intent::FloatGeometry {
                    panel: f.panel.clone(),
                    pos,
                    size: sz,
                });
            }
            // Close before dock: a frame in which the operator managed
            // both can only honour one, and the close button is the more
            // emphatic instruction.
            if closed {
                intents.push(Intent::Close(f.panel.clone()));
            } else if dock_back {
                intents.push(Intent::Dock(f.panel.clone()));
            }
        }

        // Phase 3: apply, in one place, after everything has drawn — the
        // same discipline `Dock::show` holds and for the same reason.
        report.layout_changed = apply_float_intents(&mut state.layout, &intents, &mut report);
        // A window that has gone must forget that it was ever opened, or a
        // re-float would find `opening == false` and never assert its
        // position.
        for panel in snapshot.floating.iter().map(|f| &f.panel) {
            if !state.layout.is_floating(panel) {
                ctx.data_mut(|d| d.remove::<bool>(seen_key(panel)));
            }
        }
        state.floats_drawn = report.drawn.len();
        report
    }

    /// The window title for a floated panel: its registered label, or its
    /// raw id when nothing registered it.
    ///
    /// Falling back to the id rather than to a generic word, for
    /// [`super::ctx::Ctx::describe`]'s reason: a window called "Panel" is
    /// a window the operator cannot tell from another window called
    /// "Panel", and an unregistered panel is a bug whose symptom should
    /// name itself.
    fn float_title(&self, panel: &PanelId) -> String {
        self.registry
            .and_then(|r| r.get(panel.as_str()))
            .map_or_else(|| panel.as_str().to_owned(), |i| i.label.clone())
    }

    /// The header strip: the panel's name, sensed for a secondary click.
    ///
    /// Returns the `Response`, which goes straight to the application's
    /// tab-menu handler — so the float window's menu and the dock tab's
    /// menu are literally the same menu, resolved through the same
    /// registry against the same conditions.
    fn draw_header(
        &self,
        ui: &mut egui::Ui,
        rect: Rect,
        panel: &PanelId,
        theme: &Theme,
    ) -> egui::Response {
        let (label, announced) = match self.registry.and_then(|r| r.get(panel.as_str())) {
            Some(info) => (info.label.clone(), info.accessible_name().to_owned()),
            None => (panel.as_str().to_owned(), panel.as_str().to_owned()),
        };
        let response = ui.allocate_rect(rect, egui::Sense::click());
        ui.painter().text(
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            &label,
            egui::TextStyle::Body.resolve(ui.style()),
            // Explicit, because `.strong()` and every other implicit
            // emphasis resolves to the ACCENT-FILLED widget foreground and
            // this strip is on the panel fill. `DEFECTS.md` D11.
            theme.palette.text,
        );
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &announced));
        response
    }

    /// The strip an application that has adopted no tab-menu handler
    /// gets: a name and a **Dock** button.
    ///
    /// Returns whether the button was pressed. See the call site for why
    /// the built-in offer is Dock rather than Close.
    fn draw_builtin_header(
        &self,
        ui: &mut egui::Ui,
        rect: Rect,
        panel: &PanelId,
        theme: &Theme,
    ) -> bool {
        let mut docked = false;
        let mut strip = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        // The one hard-coded string in this module, and it is the same
        // compromise `tabs.rs`'s built-in "Close" makes: a shell that
        // shipped no fallback would leave a consumer with no route home,
        // and a shell with a text catalog would be a shell that knows a
        // language. An application that cares supplies a menu handler and
        // this is never drawn.
        docked |= strip.button("Dock").clicked();
        strip.painter().text(
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            self.float_title(panel),
            egui::TextStyle::Body.resolve(strip.style()),
            theme.palette.text,
        );
        docked
    }
}

/// Split the window's inner rectangle into the header strip and the body.
///
/// A pure function so the arithmetic can be asserted without a window, and
/// so the degenerate case has a named answer: a window too short to hold
/// both gives the header nothing and the body everything. **The body
/// wins**, because a header with no body is a window showing nothing at
/// all, whereas a body with no header is still a panel — and the operator
/// can still close it, because the OS window's close button is not drawn
/// by us.
#[must_use]
pub fn split_header(inner: Rect) -> (Rect, Rect) {
    if inner.height() <= HEADER_HEIGHT_PTS * 2.0 {
        return (Rect::NOTHING, inner);
    }
    let header = Rect::from_min_size(inner.min, Vec2::new(inner.width(), HEADER_HEIGHT_PTS));
    let body = Rect::from_min_max(Pos2::new(inner.min.x, header.max.y), inner.max);
    (header, body)
}

/// Apply the float windows' intents. The one place `show_floating` takes
/// `&mut DockLayout`.
///
/// Mirrors [`super::apply`]'s shape exactly — clone, mutate, compare —
/// so "did anything change" has one implementation per call and cannot be
/// forgotten by a new intent.
fn apply_float_intents(
    layout: &mut DockLayout,
    intents: &[Intent],
    report: &mut FloatFrameReport,
) -> bool {
    if intents.is_empty() {
        return false;
    }
    let before = layout.clone();
    for intent in intents {
        match intent {
            Intent::Close(panel) => {
                if layout.close(panel) {
                    report.closed = Some(panel.clone());
                }
            }
            Intent::Dock(panel) => {
                if layout.dock_back(panel) {
                    report.docked = Some(panel.clone());
                }
            }
            Intent::FloatGeometry { panel, pos, size } => {
                layout.set_float_geometry(panel, *pos, *size);
            }
            // Every other intent is raised by the docked surfaces and
            // cannot reach here. Listed as a catch-all rather than
            // enumerated, because the alternative is seven arms that all
            // say `unreachable!()` and one of them being wrong one day.
            _ => {}
        }
    }
    layout.normalize();
    *layout != before
}

/// The viewport id for a panel's float window.
///
/// Derived from the panel id rather than counted, for `dialogs::host`'s
/// reason: `ViewportId` is what `egui` keys the OS window on, so two
/// panels sharing one would be two panels in one window, and a counter
/// would give a panel a different window depending on what else happened
/// to be floating when it was floated.
///
/// Salted with a prefix so a panel called `"print"` cannot collide with an
/// application dialog of the same name — the two id spaces are independent
/// and neither knows about the other.
#[must_use]
pub fn viewport_id(panel: &PanelId) -> ViewportId {
    ViewportId::from_hash_of(("egui-shell-float", panel.as_str()))
}

/// Where "has this window already been opened" is remembered, per panel.
fn seen_key(panel: &PanelId) -> egui::Id {
    egui::Id::new(("egui-shell-float-seen", panel.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock::model::{Column, DockSide, SideLayout, Stack};

    fn id(s: &str) -> PanelId {
        PanelId::new(s)
    }

    fn sample() -> DockLayout {
        DockLayout::new(
            SideLayout::new([Column::new([Stack::tabbed(["pages", "layers"])])]),
            SideLayout::new([Column::new([Stack::new("objects")])]),
        )
    }

    /// **The header takes its strip off the top and the body gets the
    /// rest**, with nothing overlapping.
    #[test]
    fn the_header_and_the_body_partition_the_window() {
        let inner = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 400.0));
        let (header, body) = split_header(inner);
        assert_eq!(header.height(), HEADER_HEIGHT_PTS);
        assert_eq!(header.max.y, body.min.y, "no gap and no overlap");
        assert_eq!(body.max, inner.max);
        assert!(body.height() > 0.0);
    }

    /// ★ **A window too short for both gives the body everything.**
    ///
    /// A header with no body shows nothing; a body with no header is still
    /// a panel, and the OS close button is not ours to lose.
    #[test]
    fn a_window_too_short_for_a_header_still_draws_the_body() {
        let inner = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 20.0));
        let (header, body) = split_header(inner);
        assert_eq!(header, Rect::NOTHING);
        assert_eq!(body, inner);
    }

    /// **Two panels get two different windows.**
    ///
    /// A shared `ViewportId` is a shared OS window, and the symptom would
    /// be one panel drawing over another rather than anything that looks
    /// like an id collision.
    #[test]
    fn each_panel_gets_its_own_viewport() {
        assert_ne!(viewport_id(&id("layers")), viewport_id(&id("objects")));
        assert_eq!(
            viewport_id(&id("layers")),
            viewport_id(&id("layers")),
            "and the id must be stable across frames, or the window is recreated every frame"
        );
    }

    /// **A float window's id does not collide with a dialog's.**
    ///
    /// Both spaces derive an id from a plain string and neither knows the
    /// other exists, so the salt is the whole defence.
    #[test]
    fn a_float_viewport_is_salted_away_from_a_bare_name() {
        assert_ne!(viewport_id(&id("print")), ViewportId::from_hash_of("print"));
    }

    /// ★★★ **The close intent removes the panel; the dock intent puts it
    /// back.** Both report, and both mark the layout worth saving.
    #[test]
    fn the_float_intents_apply_and_report() {
        let mut l = sample();
        l.float(&id("layers"));
        let mut report = FloatFrameReport::default();
        let changed = apply_float_intents(&mut l, &[Intent::Dock(id("layers"))], &mut report);
        assert!(changed);
        assert_eq!(report.docked, Some(id("layers")));
        assert!(l.contains_docked(&id("layers")));

        l.float(&id("layers"));
        let mut report = FloatFrameReport::default();
        let changed = apply_float_intents(&mut l, &[Intent::Close(id("layers"))], &mut report);
        assert!(changed);
        assert_eq!(report.closed, Some(id("layers")));
        assert!(!l.contains(&id("layers")));
    }

    /// ★★ **A window that has not moved does not mark the layout dirty.**
    ///
    /// The geometry intent is raised every frame for every open float. If
    /// it reported a change every time, `layout_changed` would be true on
    /// every frame a panel was floating and the application would rewrite
    /// `layout.ron` continuously — which is exactly the behaviour
    /// `MODES_AND_PANELS.md` records as the benchmarked application's own
    /// worst persistence defect.
    #[test]
    fn an_unmoved_float_window_does_not_mark_the_layout_dirty() {
        let mut l = sample();
        l.float(&id("layers"));
        let geometry = Intent::FloatGeometry {
            panel: id("layers"),
            pos: Some([100.0, 200.0]),
            size: [320.0, 480.0],
        };
        let mut report = FloatFrameReport::default();
        assert!(
            apply_float_intents(&mut l, std::slice::from_ref(&geometry), &mut report),
            "the first report of a position IS a change"
        );
        let mut report = FloatFrameReport::default();
        assert!(
            !apply_float_intents(&mut l, std::slice::from_ref(&geometry), &mut report),
            "the second must not be, or the layout file is rewritten every frame"
        );
    }

    /// A `DockState` with `layers` floated out of a two-tab left stack, and
    /// a window-sized frame to draw it in.
    fn floated_state() -> DockState {
        let mut layout = sample();
        assert!(layout.float(&id("layers")), "the fixture must float");
        DockState::new(layout)
    }

    fn frame_input() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
            ..Default::default()
        }
    }

    /// ★★★ **A float window whose panel allocates nothing is REPORTED, not
    /// silently counted as a success.**
    ///
    /// This is the falsification of [`FloatFrameReport::empty_bodies`]
    /// written as a test rather than performed by hand: the same fixture,
    /// the same call, and a body that does nothing at all. Every other
    /// number the frame produces still says the window is fine —
    /// `drawn` holds the panel and `DockFrameReport::floats_undrawn` is
    /// zero — which is exactly why this field had to exist.
    ///
    /// ⚠ If this ever reports an empty list, the guard has stopped
    /// guarding and a floated panel can ship as a blank window with a
    /// title bar, which is R9 broken at the scale of a whole window.
    #[test]
    fn a_float_window_whose_panel_draws_nothing_is_reported_empty() {
        let ctx = egui::Context::default();
        let mut state = floated_state();
        let mut empty: Vec<String> = Vec::new();
        let _ = ctx.run_ui(frame_input(), |ui| {
            let report = Dock::new().show_floating(ui.ctx(), &mut state, |_panel, _ui| {
                // Deliberately nothing. This is the defect.
            });
            assert_eq!(report.drawn.len(), 1, "a window was still opened for it");
            empty = report
                .empty_bodies
                .iter()
                .map(|p| p.as_str().to_owned())
                .collect();
        });
        assert_eq!(
            empty,
            vec!["layers".to_string()],
            "a window drawn around a body that allocated nothing must be named"
        );
    }

    /// ★★ **…and a panel that allocates one small rectangle is not
    /// reported**, which is what keeps the test above from being a check
    /// that always fires.
    ///
    /// One allocation deliberately, not a full panel: the honest floor is
    /// *anything at all was allocated*, because a panel with nothing to say
    /// is required by R9 to say so in a **sentence** rather than to draw a
    /// blank — so one sentence is the minimum legitimate content, and a
    /// measurement that called it empty would fail every correct panel on a
    /// document that gives it nothing to list.
    ///
    /// ★★★ The sentence cannot be spelled as a sentence *here*; see the
    /// comment at the allocation for the reason, which cost the first
    /// version of this test a red run against a working dock.
    #[test]
    fn a_float_window_whose_panel_allocates_anything_is_not_empty() {
        let ctx = egui::Context::default();
        let mut state = floated_state();
        let mut empty = 1usize;
        let _ = ctx.run_ui(frame_input(), |ui| {
            let report = Dock::new().show_floating(ui.ctx(), &mut state, |_panel, ui| {
                // ★★★ NOT `ui.label(...)`, and the reason is a trap this
                // crate is deliberately built into. `Cargo.toml` pins `egui`
                // with `default-features = false` precisely "so this crate
                // does not silently acquire fonts" — so in every test in
                // this crate a galley is EMPTY and `ui.label("anything")`
                // returns a **zero-sized** rect. A test written with a label
                // here would fail against a perfectly working dock, and,
                // worse, the mirror test would pass for the wrong reason.
                //
                // ⇒ The honest headless spelling of "the body drew" is an
                // allocation with a size of its own. In the application,
                // which links `eframe` and therefore real fonts, one label is
                // one real rectangle and the same measurement holds.
                // `D:/dev/rag/egui/` carries this as its own entry.
                let _ = ui.allocate_space(Vec2::new(120.0, 18.0));
            });
            empty = report.empty_bodies.len();
        });
        assert_eq!(
            empty, 0,
            "a body that allocated a rectangle must not be reported as an empty window"
        );
    }

    /// **A layout with no floats does nothing and reports nothing**, which
    /// is what every application that has never floated a panel does on
    /// every frame.
    #[test]
    fn a_layout_with_no_floats_reports_an_empty_frame() {
        let mut l = sample();
        let mut report = FloatFrameReport::default();
        assert!(!apply_float_intents(&mut l, &[], &mut report));
        assert_eq!(report, FloatFrameReport::default());
    }

    /// **Intents belonging to the docked surfaces are ignored here.**
    ///
    /// `apply_float_intents` shares an `Intent` enum with the docked path;
    /// a stray one must be a no-op rather than a second implementation of
    /// a splitter drag.
    #[test]
    fn a_docked_surfaces_intent_is_ignored_by_the_float_path() {
        let mut l = sample();
        l.float(&id("layers"));
        let before = l.clone();
        let mut report = FloatFrameReport::default();
        assert!(!apply_float_intents(
            &mut l,
            &[Intent::ToggleSide(DockSide::Left)],
            &mut report
        ));
        assert_eq!(l, before);
    }
}
