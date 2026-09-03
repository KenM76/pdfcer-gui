//! # `app::panels` — putting a panel on screen, and taking it off again
//!
//! Two methods, and the distinction between them is the whole subject of this
//! file: **not every command that shows a panel is a toggle.**
//!
//! | | [`PdfcerApp::toggle_panel`] | [`PdfcerApp::show_panel`] |
//! |---|---|---|
//! | the control asks | *"is this panel open?"* | *"tell me about this thing"* |
//! | pressing it when open | **closes** it | shows it again, idempotently |
//! | callers | `view.panel_*`, `file.fonts` | `file.properties`, `markup.comments` |
//!
//! ## Why this is its own file
//!
//! Split from `app/mod.rs` when that file crossed the 1,500-line gate for the
//! fourth time — the earlier splits produced `dispatch.rs`, `conditions.rs`
//! and `gating.rs`. The seam is the same shape as all three: `mod.rs` composes
//! a frame, `dispatch.rs` answers *what does this verb do*, `conditions.rs`
//! answers *what is true right now*, `gating.rs` answers *what is this mode
//! allowed to do*, and this file answers *where does a panel go, and what does
//! pressing its control mean*.
//!
//! ## ★ The distinction is load-bearing, and a test proved it
//!
//! The toggle's first draft changed `show_panel` itself, which turned **every**
//! panel command into a toggle — including `file.properties`. That is offered
//! by the **Objects row context menu** to describe the row just clicked, so
//! right-clicking a second row and choosing Properties would have *closed* the
//! description instead of re-pointing it at the new row.
//!
//! `app::tests::the_properties_command_puts_the_panel_on_screen_in_every_mode`
//! caught it, and its comment had said so in advance:
//!
//! > *Idempotent: asking twice is not a toggle. The `objects.row` context menu
//! > offers this command to describe the row just clicked, and a second
//! > invocation that hid the description would be actively hostile.*
//!
//! It is worth recording that the argument was already written down and was
//! still walked into. The lesson is not "read the tests" — it is that a change
//! phrased as *"make panel commands toggles"* silently assumes every command
//! that reaches a panel is a panel **control**, and two of them are not.

use crate::app::PdfcerApp;

impl PdfcerApp {
    /// **Show `panel`, or close it if it is already on screen.**
    ///
    /// What the *panel toggle controls* call — `view.panel_*` and
    /// `file.fonts`. Operator decision, 2026-08-14: pressing the control for a
    /// panel that is open closes it, which is what Acrobat, VS Code and
    /// Inkscape all do and therefore what the standing *"make it work the way
    /// other programs do"* tie-breaker asks for. Before this they were
    /// show-only, so such a control rendered **pressed and did nothing** — a
    /// visible control that is silently inert, which `RIBBON_IA.md` P3 does not
    /// excuse.
    ///
    /// # ★ Why this is a second entry point rather than a change to
    /// [`Self::show_panel`]
    ///
    /// Because **not every command that shows a panel is a toggle**, and the
    /// distinction is load-bearing rather than tidy. `file.properties` is
    /// offered by the **Objects row context menu** to describe the row just
    /// clicked; if it toggled, right-clicking a second row and choosing
    /// Properties would *close* the description instead of re-pointing it —
    /// which `app::tests::the_properties_command_puts_the_panel_on_screen_in_every_mode`
    /// calls "actively hostile", and it is right. That test caught this
    /// function's first draft, which toggled everything.
    ///
    /// So the rule is about the **control**, not the panel: a control whose
    /// job is *"is this panel open?"* toggles; a control whose job is *"tell me
    /// about this thing"* shows. Both end at the same mounting code.
    ///
    /// # The three states, and why the middle one is not a close
    ///
    /// | the panel is… | pressing it |
    /// |---|---|
    /// | **on screen** — mounted, the active tab of its stack, its side visible | **closes** it |
    /// | **mounted but behind a sibling tab**, or on a hidden side | **raises** it |
    /// | not mounted at all | mounts and raises it |
    ///
    /// The middle row is the one that would be easy to get wrong, and getting
    /// it wrong is worse than not building the toggle: a panel behind another
    /// tab is *not* on screen, so the operator pressing its control means
    /// "show me that", and closing it would unmount the thing they asked to
    /// see. [`egui_shell::dock::DockState::is_on_screen`] is the dock's own
    /// predicate for exactly that distinction — it asks both that the panel is
    /// the active tab **and** that its side is visible — so this reads the
    /// dock's answer rather than deriving a second one.
    ///
    /// Closing goes through `DockLayout::close`, the same path the dock's own
    /// tab Close takes (`Intent::Close`). Two ways to close a panel that did
    /// different things would eventually disagree about what "closed" means
    /// for persistence.
    pub(super) fn toggle_panel(&mut self, panel: crate::panels::Panel) {
        let id = egui_shell::dock::PanelId::new(panel.command_id());
        if !self.dock.is_on_screen(&id) {
            self.show_panel(panel);
            return;
        }
        let closed = self.dock.layout_mut().close(&id);
        self.dock.normalize();
        self.modes
            .record_layout(self.dock.layout(), &mut self.layout);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "panel-closed id={} closed={closed}",
                id.as_str()
            )
        });
    }

    /// **Put `panel` on screen, mounting it first if the operator's
    /// arrangement no longer holds it.**
    ///
    /// # The decision: mount, rather than do nothing
    ///
    /// `DockState::activate` returns `false` for a panel that is not mounted,
    /// and its own docs say what that means: *"the caller's fallback is to
    /// mount it or to restore a default arrangement, not to refuse."* Both
    /// are defensible. This mounts, for three reasons:
    ///
    /// 1. **Read mode has no Properties panel by design** — `app::modes`'
    ///    `spec("read")` gives it none, because "an inspector in a mode with
    ///    no edit verbs is a panel whose every row is a fact you cannot act
    ///    on". But File ▸ Properties is on the File tab, and the File tab is
    ///    in *every* mode. Doing nothing would make a visible, enabled
    ///    control inert in the mode the application **opens in** — defect
    ///    D1's exact shape, and the thing `PROJECT_PLAN.md`'s no-placeholders
    ///    invariant forbids.
    /// 2. **There is no other route.** Properties is not on View ▸ Panels
    ///    (it is `file.properties`, not `view.panel_properties`), so an
    ///    operator who closes its tab has no second way back. A command that
    ///    is the only route to a surface must be able to produce it.
    /// 3. **A mode default is a starting arrangement, not a prohibition.**
    ///    The dock belongs to the operator; asking for a panel is a
    ///    rearrangement, and rearrangements are what the layout store exists
    ///    to remember.
    ///
    /// # Where it lands
    ///
    /// Wherever the **Edit mode default** puts it, because that arrangement
    /// names every panel this build has and agrees with the other modes about
    /// which side each one belongs on (Properties is on the right in Review
    /// and in Edit). Reading the placement out of `app::modes` rather than
    /// writing a side into this function is what stops "where does Properties
    /// live" from acquiring a second answer. The right dock is the fallback
    /// if that lookup ever fails, which it cannot today.
    ///
    /// `DockLayout::mount` is deliberately permissive about out-of-range
    /// column and stack indices — it clamps — so a live arrangement with
    /// fewer columns than the default needs no special case here.
    ///
    /// # Why the arrangement is recorded
    ///
    /// A mount is a layout change, and `Dock::show`'s `layout_changed` report
    /// describes what the *operator* did to the dock during the frame — it
    /// does not see a programmatic edit. Without this call the panel would
    /// appear and then vanish on the next restart, which reads as a bug in
    /// persistence rather than as a design choice about mounting.
    pub(super) fn show_panel(&mut self, panel: crate::panels::Panel) {
        let id = egui_shell::dock::PanelId::new(panel.command_id());

        if self.dock.activate(&id) {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "panel-shown id={} mounted=already",
                    id.as_str()
                )
            });
            return;
        }

        let home = crate::app::modes::layout_for("edit").find(&id);
        let (side, column, stack) = home.map_or((egui_shell::dock::DockSide::Right, 0, 0), |a| {
            (a.side, a.column, a.stack)
        });
        self.dock
            .layout_mut()
            .mount(side, column, stack, id.clone());
        self.dock.normalize();
        let shown = self.dock.activate(&id);
        self.modes
            .record_layout(self.dock.layout(), &mut self.layout);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "panel-shown id={} mounted=now side={} shown={shown}",
                id.as_str(),
                side.key(),
            )
        });
    }
}
