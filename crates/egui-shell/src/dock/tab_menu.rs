//! The tab-menu seam — what an application is handed when it wants to own
//! a panel tab's secondary click.
//!
//! # THE SEAM: the dock hands out a `Response`; the application decides
//! what a right-click means
//!
//! A dock's job is *layout*: which panels are on which side, in which
//! column, in which stack, and which tab of that stack is in front. **What
//! a right-click on a tab offers is not a layout question.** It is the
//! same argument that makes [`super::PanelId`] an opaque string: the shell
//! cannot know that "Reset layout", "Float this panel" or "Hide the
//! Objects group" are the verbs *this* application wants on *this* tab,
//! and the moment it tries to guess it has learned something about the
//! application it is supposed to be reusable across.
//!
//! So the dock stops at the boundary it can actually defend. It draws the
//! tab, senses the click, publishes the accessible name, and then — if the
//! application asked for it — hands the tab's [`egui::Response`] out
//! together with the [`super::PanelId`] the response belongs to. What
//! happens next is the application's business.
//!
//! ```no_run
//! use egui_shell::dock::{Dock, DockState, PanelRegistry, TabMenu};
//! use egui_shell::menu::Menu;
//! use egui_shell::{CommandRegistry, ConditionSet, HandlerToken, Menus};
//! # fn frame(ui: &mut egui::Ui, state: &mut DockState, panels: &PanelRegistry,
//! #          menus: &Menus, commands: &CommandRegistry, conditions: &ConditionSet) {
//! // Tokens are collected here and dispatched after `show` returns — the
//! // same "record now, act later" discipline the dock applies to its own
//! // intents, and for the same borrow-checker reason.
//! let mut chosen: Vec<HandlerToken> = Vec::new();
//! let mut tab_menu = |tab: &mut TabMenu<'_>| {
//!     chosen.extend(Menu::attach(tab.response(), menus, commands, "dock.tab", conditions));
//! };
//!
//! Dock::new()
//!     .with_registry(panels)
//!     .with_tab_menu(&mut tab_menu)
//!     .show(ui, state, |_panel, _ui| { /* panel bodies */ });
//! # }
//! ```
//!
//! Nothing in this file — or anywhere else in [`super`] — names
//! [`crate::menu::Menu`], [`crate::commands::Command`] or
//! [`crate::manifest::Item`]. The dock does not learn what a command is;
//! it learns that *somebody else* would like the `Response`. A menu is one
//! thing an application might attach. A drag-to-tear-out gesture, a
//! double-click-to-maximize, a tooltip of its own and a diagnostic overlay
//! are others, and all of them arrive through this one seam without the
//! dock growing a concept per feature.
//!
//! # Why one `&mut TabMenu` rather than two arguments
//!
//! The obvious spelling is `FnMut(&PanelId, &egui::Response)`. It is
//! rejected for one concrete reason and one structural one.
//!
//! **The concrete reason: a two-argument handler has no way back in.** The
//! dock's whole design turns on *recording an intent rather than mutating*
//! (see [`super::ctx`]'s header). A handler whose only outputs are the
//! side effects it can reach through its own captures cannot ask the dock
//! to close a panel; it can only reach around the dock and mutate
//! [`super::DockState::layout_mut`] after the frame. That "works", and it
//! is exactly the shape of bug the intent queue exists to make impossible:
//! a close applied outside the queue does not appear in
//! [`super::DockFrameReport::closed`], does not set
//! [`super::DockFrameReport::layout_changed`], and therefore is not
//! persisted by an application that — correctly — saves on
//! `layout_changed`. The panel comes back on the next launch and nobody
//! can explain why. [`TabMenu::request_close`] is the seam's answer, and
//! it is only expressible if the handler is given something it can write
//! to.
//!
//! **The structural reason: one parameter is one edit.** Every capability
//! the seam grows later — "was this tab the active one?", "here is the
//! stack it belongs to", "ask the dock to activate it" — is a method on
//! this struct rather than a change to the signature of every application
//! that adopted the seam. The cost is one extra type in the public API;
//! the cost of the alternative is a breaking change per capability.
//!
//! # What happens to the dock's built-in "Close"
//!
//! | The application… | The tab's secondary click is owned by | "Close" |
//! |---|---|---|
//! | supplies **no** handler | the dock | **drawn** |
//! | supplies a handler | the application | offered as [`TabMenu::request_close`] |
//!
//! **Row 1 is a compatibility guarantee, not an implementation detail.**
//! The built-in Close is the only way to close a panel from its tab without
//! a handler, and a consumer that has not adopted the seam must not lose it
//! by standing still.
//! `the_built_in_close_still_closes_a_panel_with_no_handler` in
//! [`super::tabs`] is the test that says so.
//!
//! **Row 2 is not the dock being precious about its menu.** It is the one
//! technical constraint in this design that is not negotiable: a
//! context menu's popup id is `response.id.with("popup")`
//! (`egui::Popup::default_response_id`), so two menus attached to one
//! `Response` are two writers of one open/closed flag in `egui`'s memory.
//! The observable result is not "both menus appear" — it is one menu
//! flickering, or opening at the wrong pointer position, or refusing to
//! close, depending on which call ran second this frame. There is exactly
//! one popup per `Response`, so there is exactly one owner, and the owner
//! is whoever asked to be.
//!
//! An application that wants the shell's Close *and* its own rows does not
//! get a merged menu — it puts a close row in its own menu and calls
//! [`TabMenu::request_close`] when that row is chosen. The row is the
//! application's (its label, its icon, its position, its keyboard chord);
//! the *action* is the dock's, and goes through the dock's queue:
//!
//! ```no_run
//! # use egui_shell::dock::TabMenu;
//! # use egui_shell::{HandlerToken, menu::Menu, CommandRegistry, ConditionSet, Menus};
//! # fn wire(tab: &mut TabMenu<'_>, menus: &Menus, commands: &CommandRegistry,
//! #         conditions: &ConditionSet, close_panel: HandlerToken,
//! #         chosen: &mut Vec<HandlerToken>) {
//! for token in Menu::attach(tab.response(), menus, commands, "dock.tab", conditions) {
//!     if token == close_panel {
//!         tab.request_close(); // ← the dock's own path: Intent::Close, reported, persisted
//!     } else {
//!         chosen.push(token); // ← everything else is the application's
//!     }
//! }
//! # }
//! ```
//!
//! # Accessibility
//!
//! The `Response` handed out has **already published its `WidgetInfo`** —
//! the panel's purpose as the accessible name, and its selected state —
//! before the handler is called. See [`super::tabs`]'s header for the
//! wording and for the honest limitation (`egui` 0.35 has no tab role),
//! and `crate::ribbon::a11y` for the convention it follows.
//!
//! This ordering is deliberate and it is the reason the handler cannot
//! degrade the accessibility of the dock. A handler that ignores the
//! response entirely, panics, or attaches nothing still leaves a tab that
//! announces itself correctly, because the announcement was made before
//! control left the dock. The handler *adds* to the tab; it cannot
//! silently subtract from it.
//!
//! # What the handler is called for, and when
//!
//! Once per **drawn** tab, per frame, in bar order, whether or not the
//! operator clicked anything. That is not an accident of implementation —
//! it is what [`crate::menu::Menu::attach`] requires: it must run every
//! frame to decide whether to open, to keep an open menu drawn, and to
//! *close* a menu whose offer has evaporated. A handler that only wanted
//! to react to a click asks `tab.response().secondary_clicked()`.
//!
//! Tabs hidden behind the overflow affordance are **not** offered, because
//! they have no `Response` — nothing was drawn for them. A right-click on
//! an overflow *row* is likewise not offered: the overflow menu is itself a
//! popup, and a context menu inside a menu is a nested-popup design this
//! module deliberately does not open (see [`crate::menu::render`]'s
//! "What is *not* here").
//!
//! Design and rationale: `docs/modules/egui-shell/dock/tab_menu.md`.

use super::model::PanelId;

/// The handler an application supplies to [`super::Dock::with_tab_menu`].
pub type TabMenuHandler<'a> = dyn FnMut(&mut TabMenu<'_>) + 'a;

/// One drawn tab, offered to the application's tab-menu handler.
pub struct TabMenu<'a> {
    /// Which panel's tab this is. The application's own identifier for it,
    /// echoed back unchanged — the dock never interprets it.
    panel: &'a PanelId,
    /// The tab button's response, with its accessible name already
    /// published. This is the thing the seam exists to hand out.
    response: &'a egui::Response,
    /// Whether the handler asked the dock to float this panel out.
    ///
    /// A separate flag from [`Self::close_requested`] rather than an
    /// enum, and the reason is the same one that made this a struct: a
    /// capability added later must not change the shape of anything an
    /// application already wrote. An enum would also make "asked for two
    /// things" unrepresentable, which sounds like a virtue until you
    /// notice that the dock is the right place to decide what happens
    /// then — and it does, at the call site, in a documented order.
    float_requested: bool,
    /// Whether the handler asked the dock to put this panel back.
    ///
    /// Meaningful only on a floating panel's header strip, and harmless
    /// on a tab: [`super::DockLayout::dock_back`] answers `false` for a
    /// panel that is not floating and changes nothing.
    dock_requested: bool,
    /// Whether the handler asked the dock to close this panel.
    ///
    /// A plain flag rather than a direct push into
    /// [`super::ctx::Ctx::intents`], because the handler is called with the
    /// context already mutably borrowed to reach the handler itself. The
    /// dock reads this the instant the handler returns and converts it to
    /// an [`super::ctx::Intent::Close`] — so the "record, then apply"
    /// discipline is preserved rather than bypassed, and a close asked for
    /// here is indistinguishable, downstream, from a close asked for by
    /// the built-in menu.
    close_requested: bool,
}

impl<'a> TabMenu<'a> {
    /// Wrap a freshly drawn tab. Called only by [`super::tabs`].
    pub(crate) fn new(panel: &'a PanelId, response: &'a egui::Response) -> Self {
        Self {
            panel,
            response,
            float_requested: false,
            dock_requested: false,
            close_requested: false,
        }
    }

    /// Which panel this tab belongs to.
    #[must_use]
    pub fn panel(&self) -> &PanelId {
        self.panel
    }

    /// The tab button's response.
    #[must_use]
    pub fn response(&self) -> &egui::Response {
        self.response
    }

    /// Ask the dock to close this panel.
    pub fn request_close(&mut self) {
        self.close_requested = true;
    }

    /// Whether [`Self::request_close`] has been called on this tab.
    #[must_use]
    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    /// **Ask the dock to tear this panel out into a window of its own.**
    pub fn request_float(&mut self) {
        self.float_requested = true;
    }

    /// Whether [`Self::request_float`] has been called.
    #[must_use]
    pub fn float_requested(&self) -> bool {
        self.float_requested
    }

    /// **Ask the dock to put this panel back where it came from.**
    pub fn request_dock(&mut self) {
        self.dock_requested = true;
    }

    /// Whether [`Self::request_dock`] has been called.
    #[must_use]
    pub fn dock_requested(&self) -> bool {
        self.dock_requested
    }
}

impl std::fmt::Debug for TabMenu<'_> {
    /// Prints the panel and the request flag, not the `Response`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TabMenu")
            .field("panel", &self.panel.as_str())
            .field("float_requested", &self.float_requested)
            .field("dock_requested", &self.dock_requested)
            .field("close_requested", &self.close_requested)
            .field("response_id", &self.response.id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Rect, Sense, Vec2};

    /// Build a real `Response` by running one frame, and hand it to `f`
    /// wrapped in a `TabMenu` for the panel `id`.
    fn with_tab(id: &str, mut f: impl FnMut(&mut TabMenu<'_>)) {
        let panel = PanelId::new(id);
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(400.0, 200.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let response = ui.allocate_response(Vec2::new(80.0, 24.0), Sense::click());
            let mut tab = TabMenu::new(&panel, &response);
            f(&mut tab);
        });
    }

    /// A fresh tab has asked for nothing.
    #[test]
    fn a_fresh_tab_menu_has_requested_nothing() {
        with_tab("pages", |tab| {
            assert!(!tab.close_requested());
            assert_eq!(tab.panel().as_str(), "pages");
        });
    }

    /// Asking twice is asking once.
    #[test]
    fn requesting_a_close_twice_is_the_same_as_requesting_it_once() {
        with_tab("layers", |tab| {
            tab.request_close();
            assert!(tab.close_requested());
            tab.request_close();
            assert!(tab.close_requested());
        });
    }

    /// The response handed out is the one that was drawn — same rect, and
    /// it senses clicks, which is what
    /// [`crate::menu::Menu::attach`] requires of it.
    #[test]
    fn the_response_handed_out_is_the_drawn_widget() {
        with_tab("objects", |tab| {
            assert_eq!(tab.response().rect.size(), Vec2::new(80.0, 24.0));
            assert!(tab.response().sense.senses_click());
        });
    }

    /// The `Debug` impl names the tab and what was asked of it, so a
    /// failing assertion in an application's own test suite reads.
    #[test]
    fn debug_prints_the_panel_and_the_request() {
        with_tab("pages", |tab| {
            tab.request_close();
            let s = format!("{tab:?}");
            assert!(s.contains("pages"), "{s}");
            assert!(s.contains("close_requested: true"), "{s}");
        });
    }
}
