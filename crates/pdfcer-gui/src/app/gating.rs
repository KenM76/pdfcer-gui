//! # `app::gating` — the mode gate, on the application side
//!
//! Two methods on [`PdfcerApp`], and they are the seam between *what a mode
//! is* and *what the canvas is allowed to do*.
//!
//! [`crate::app::modes::capability`] answers the question in the abstract —
//! given a manifest and a mode id, which capabilities does that mode carry —
//! and is pure. This file is where the running application asks it, and where
//! the answer is **acted on**: once per frame to hand the canvas its
//! [`Capabilities`], and once per *mode change* to retire the things that
//! would otherwise survive into a mode that forbids them.
//!
//! ## Why this is its own file
//!
//! Split from `app/mod.rs` when that file crossed the 1,500-line gate for the
//! third time — the earlier splits produced `app/dispatch.rs` and
//! `app/conditions.rs`. The seam is the same shape as both: `mod.rs` composes
//! a frame, `dispatch.rs` answers *what does this verb do*, `conditions.rs`
//! answers *what is true right now*, and this file answers *what is this mode
//! allowed to do, and what has to be put down on the way in*.
//!
//! ## ★ The two halves, and why neither is sufficient alone
//!
//! The gate is genuinely two mechanisms, and the reason is that a mode change
//! is a moment while a capability is a state:
//!
//! | | mechanism | covers |
//! |---|---|---|
//! | **refuse** | [`crate::canvas::gesture::press_kind`] returns `None` | anything the operator tries to *start* while in the mode |
//! | **retire** | [`PdfcerApp::on_mode_capabilities_changed`] | anything that was *already there* when the mode was entered |
//!
//! Refusal alone leaves an armed pen drawing a crosshair over a page it
//! cannot draw on, and eight resize handles on a selection nothing will move.
//! Retirement alone leaves every gesture available for as long as the
//! operator stays put. Both, and the canvas tells the truth in both tenses.

use crate::app::PdfcerApp;
use crate::app::modes::Capabilities;
use crate::app::state::Status;

impl PdfcerApp {
    /// **What the mode the operator is in lets the canvas do to the
    /// document.**
    ///
    /// One expression, in one place, so that every consumer — the canvas
    /// gestures, the Delete key, the context menus, the tool arming — reads
    /// the same answer rather than re-deriving it. Cheap enough to call per
    /// frame per consumer (a linear scan of at most a handful of modes over a
    /// handful of tab ids) and `Copy`, so nothing has to cache it and no
    /// cache can go stale.
    ///
    /// The ribbon rather than `self.modes` is asked for the active mode,
    /// deliberately: the ribbon is where the operator's click lands, and
    /// `self.modes` catches up with it later in the same frame (see the
    /// mode-change arm in [`Self::dock_area`]). Reading the laggard would put
    /// the canvas one frame behind the selector on the frame the mode
    /// changes — which is precisely the frame a stray click is most likely,
    /// because the pointer is already down over the chrome.
    ///
    /// See `app::modes::capability` for the derivation and for why an
    /// unrecognised mode gets everything.
    #[must_use]
    pub fn capabilities(&self) -> Capabilities {
        Capabilities::for_mode(self.shell.as_ref(), self.ribbon.mode())
    }

    /// **Bring the canvas into line with a mode the operator has just entered.**
    ///
    /// Called once, from the mode-change arm in [`Self::dock_area`], *after*
    /// `Modes::on_mode_changed` has rearranged the panels — so the two halves
    /// of "what this mode is" land in one frame and in a fixed order.
    ///
    /// Three things, and each is a thing that would otherwise **survive** into
    /// a mode that forbids it. That is the shape of the whole problem: the
    /// gesture gate in `canvas::gesture::press_kind` stops anything *new* from
    /// starting, and cannot by itself retire what was already there.
    ///
    /// 1. **An armed tool** — see [`crate::canvas::tool::retire_forbidden`],
    ///    which carries the argument. Its visible symptom is the cursor.
    /// 2. **The selection.** A selection made in Edit outlives the switch,
    ///    because it lives on the document and `MODES_AND_PANELS.md` rule 1
    ///    forbids a mode change from destroying work. A selection is **not
    ///    work** — nothing about the document changes, the undo stack is
    ///    untouched, and it is re-made with one click on returning. Leaving it
    ///    would put eight resize handles and an outline on a page in Read:
    ///    controls the operator can see, aim at, and drag with no effect, which
    ///    is precisely the *"visible control, silently inert"* failure the mode
    ///    system exists to avoid. Clearing is what lets the gesture gate refuse
    ///    a grip **that is not there** rather than one the operator is looking
    ///    at.
    /// 3. **A gesture in flight.** Rule 1 again, and this time it is the rule's
    ///    own wording: *"If a mode change would hide a pending, uncommitted
    ///    gesture … that gesture is committed or cancelled first."* Cancelled,
    ///    not committed — the operator asked for a mode, not for the half-drawn
    ///    rectangle under their pointer, and `GestureOutcome::Cancelled`'s
    ///    contract is that nothing is written.
    ///
    /// Nothing here fires when capability did not change: `retire_forbidden`
    /// reports `false` for a permitted tool, a `clear` on an empty selection is
    /// a no-op, and Read → Review leaves an armed Rectangle armed.
    pub(crate) fn on_mode_capabilities_changed(&mut self, ctx: &egui::Context) {
        let caps = self.capabilities();
        // ★ Park it where a dock panel can read it. `crate::panels::tool` has
        // to answer "what does a press mean in this mode" and is handed no
        // `Capabilities` — see `canvas::tool::store_capabilities` for why this
        // is a park rather than a sixth parameter on `Panel::show`, and why it
        // must not be a second derivation.
        //
        // Here rather than anywhere else because this is the ONE function that
        // runs when the answer changes: the capability set is a property of the
        // mode, the mode changes in exactly one place, and this is what that
        // place calls.
        crate::canvas::tool::store_capabilities(ctx, caps);
        let retired = crate::canvas::tool::retire_forbidden(ctx, caps);
        let mut cleared = false;
        let mut abandoned = false;
        // ★ **The text selection is retired in the OTHER direction**, and that
        // is the whole reason it needs its own line here rather than joining the
        // `!caps.edit_content` block below.
        //
        // Every other retirement above is *"this mode forbids what you were
        // doing"*. A text selection is forbidden by nothing — it authors nothing
        // and needs no capability (`canvas::textsel`'s header §3) — but the
        // **gesture that answers it** disappears when a mode starts selecting
        // content: in Edit a click selects an object, Escape ascends the object
        // ladder, and the wash sits on the page with nothing that clears it.
        // That is the *"visible control, silently inert"* failure
        // `MODES_AND_PANELS.md` Part 1 forbids, arriving from the far side, and
        // it is exactly why refusal alone is never sufficient.
        //
        // The predicate is the gesture's own, asked of the tool the operator
        // actually has — not `!caps.edit_content` spelled a second time — so a
        // future `CanvasTool::Text` widens this and the press rule together or
        // widens neither.
        let mut cleared_text = false;
        if !crate::canvas::textsel::takes_the_press(crate::canvas::tool::selected(ctx), caps)
            && let Status::Open(doc) = &mut self.status
        {
            cleared_text = doc.text_selection.take().is_some();
        }
        if !caps.edit_content {
            abandoned = crate::canvas::input::abandon_gesture(ctx);
            if let Status::Open(doc) = &mut self.status {
                cleared = doc.selection.clear();
            }
        }
        // ★ The ANNOTATION selection retires on its own capability, and the
        // separation is the point rather than an accident of ordering.
        //
        // A selected stamp is governed by `author_markup`, which **Review
        // grants and Read does not**; a selected path is governed by
        // `edit_content`, which only Edit grants. Entering Review from Edit
        // must therefore drop the path and keep the stamp — and a single
        // "clear the selection" on `!edit_content` would have dropped both,
        // silently, in the mode the operator entered *in order to* work on
        // markup.
        //
        // That is the same shape as the duplex/tray split in the print dialog
        // and the same shape as this gate's own three capabilities: one
        // predicate per capability, never one predicate standing in for
        // several. `SelectionState::clear`'s own docs refuse to fold these
        // together for the same reason.
        let mut cleared_annot = false;
        if !caps.author_markup
            && let Status::Open(doc) = &mut self.status
        {
            cleared_annot = doc.selection.clear_annot();
        }
        if retired || cleared || abandoned || cleared_text || cleared_annot {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "mode-capabilities content={} markup={} measure={} retired_tool={retired} cleared_selection={cleared} abandoned_drag={abandoned} cleared_text={cleared_text} cleared_annot={cleared_annot}",
                    caps.edit_content, caps.author_markup, caps.author_measure,
                )
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::{opened, select_object};

    // ---------------------------------------------------------------
    // The mode gate — Read does not edit the document
    // ---------------------------------------------------------------

    /// The three shipped modes, read through the app rather than through the
    /// manifest, so this asserts what the *running* application computes.
    #[test]
    fn the_apps_capabilities_follow_its_ribbon_mode() {
        let ctx = egui::Context::default();
        let mut app = opened();

        app.dispatch_command(&ctx, "mode.read", &mut Vec::new());
        assert_eq!(app.capabilities(), Capabilities::NONE);

        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        assert_eq!(app.capabilities(), Capabilities::FULL);

        app.dispatch_command(&ctx, "mode.review", &mut Vec::new());
        let review = app.capabilities();
        assert!(
            !review.edit_content,
            "a reviewer does not edit page content"
        );
        assert!(review.author_markup && review.author_measure);
    }

    /// ★ **Entering Read drops a selection made in Edit.**
    ///
    /// The defect this closes is not "Delete works in Read" — it is the
    /// *outline and eight resize handles* left on the page, which are visible
    /// controls the operator can aim at and which would do nothing. See
    /// [`PdfcerApp::on_mode_capabilities_changed`] for why a selection is not
    /// "work" under `MODES_AND_PANELS.md` rule 1.
    #[test]
    fn entering_read_clears_a_selection_made_in_edit() {
        let ctx = egui::Context::default();
        let mut app = opened();
        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);

        select_object(&mut app, 1, false);
        let Status::Open(doc) = &app.status else {
            unreachable!()
        };
        assert!(!doc.selection.is_empty(), "the object is selected in Edit");

        app.dispatch_command(&ctx, "mode.read", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);

        let Status::Open(doc) = &app.status else {
            unreachable!()
        };
        assert!(
            doc.selection.is_empty(),
            "entering Read must leave no selection, and so no handles"
        );
    }

    /// …and Review clears it too, which is the row that proves the rule is
    /// *content editing* rather than *Read*. Review authors markup and
    /// dimensions; the page's own content is still not the reviewer's.
    #[test]
    fn entering_review_also_clears_a_content_selection() {
        let ctx = egui::Context::default();
        let mut app = opened();
        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);
        select_object(&mut app, 1, false);

        app.dispatch_command(&ctx, "mode.review", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);

        let Status::Open(doc) = &app.status else {
            unreachable!()
        };
        assert!(doc.selection.is_empty());
    }

    /// ★ **An armed markup tool does not survive into Read**, so the crosshair
    /// does not promise a gesture the canvas has decided not to give.
    #[test]
    fn entering_read_retires_an_armed_markup_tool() {
        use crate::canvas::markup::MarkupKind;
        use crate::canvas::tool::{self, CanvasTool};

        let ctx = egui::Context::default();
        let mut app = opened();
        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);

        tool::arm_markup(&ctx, MarkupKind::Rectangle);
        assert_eq!(
            tool::selected(&ctx),
            CanvasTool::Markup(MarkupKind::Rectangle)
        );

        app.dispatch_command(&ctx, "mode.read", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);
        assert_eq!(
            tool::selected(&ctx),
            CanvasTool::Select,
            "Read has no pen, so the pen is put down"
        );
    }

    /// …and Review keeps it, because Review is where markup is authored. The
    /// previous test alone would pass on a build that retired the tool on
    /// **every** mode change.
    #[test]
    fn entering_review_keeps_an_armed_markup_tool() {
        use crate::canvas::markup::MarkupKind;
        use crate::canvas::tool::{self, CanvasTool};

        let ctx = egui::Context::default();
        let mut app = opened();
        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);
        tool::arm_markup(&ctx, MarkupKind::Arrow);

        app.dispatch_command(&ctx, "mode.review", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);
        assert_eq!(
            tool::selected(&ctx),
            CanvasTool::Markup(MarkupKind::Arrow),
            "a reviewer's pen is still theirs"
        );
    }

    /// ★ **A `markup.*` command declines in a mode that does not author
    /// markup** — the belt to `retire_forbidden`'s braces, for an arming that
    /// happens while already in the mode.
    #[test]
    fn a_markup_command_declines_in_read() {
        use crate::canvas::tool::{self, CanvasTool};

        let ctx = egui::Context::default();
        let mut app = opened();
        app.dispatch_command(&ctx, "mode.read", &mut Vec::new());

        app.dispatch_command(&ctx, "markup.rectangle", &mut Vec::new());
        assert_eq!(
            tool::selected(&ctx),
            CanvasTool::Select,
            "the pen is never picked up in Read"
        );

        // …and the identical dispatch in Edit does arm it, so the test above
        // is not passing because the command id is wrong.
        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        app.dispatch_command(&ctx, "markup.rectangle", &mut Vec::new());
        assert!(
            tool::selected(&ctx).markup_kind().is_some(),
            "the same command arms in Edit"
        );
    }

    // ---------------------------------------------------------------
    // Panel controls are toggles
    // ---------------------------------------------------------------

    /// ★ **Pressing an open panel's control closes it** — operator
    /// decision, 2026-08-14.
    ///
    /// Before this, `show_panel` was show-only, so the control for a panel
    /// already on screen rendered *pressed* and did nothing.
    #[test]
    fn a_panel_control_closes_a_panel_that_is_on_screen() {
        use crate::panels::Panel;
        let ctx = egui::Context::default();
        let mut app = opened();
        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);

        let panel = Panel::Objects;
        let id = egui_shell::dock::PanelId::new(panel.command_id());

        app.toggle_panel(panel);
        assert!(
            app.dock.is_on_screen(&id),
            "the first press must put the panel on screen"
        );

        app.toggle_panel(panel);
        assert!(
            !app.dock.is_on_screen(&id),
            "the second press must close it — that is the toggle"
        );

        // …and a third press brings it back, so the toggle is a toggle rather
        // than a one-way close.
        app.toggle_panel(panel);
        assert!(app.dock.is_on_screen(&id), "the third press reopens it");
    }

    /// ★ **A panel mounted but hidden behind a sibling tab is RAISED, not
    /// closed.**
    ///
    /// The middle state, and the one that would be easy to get wrong. Getting
    /// it wrong is worse than not building the toggle at all: the operator
    /// pressing the control for a panel they cannot see means *"show me
    /// that"*, and closing it would unmount the very thing they asked for.
    #[test]
    fn a_panel_behind_a_sibling_tab_is_raised_rather_than_closed() {
        use crate::panels::Panel;
        let ctx = egui::Context::default();
        let mut app = opened();
        app.dispatch_command(&ctx, "mode.edit", &mut Vec::new());
        app.on_mode_capabilities_changed(&ctx);

        // Two panels that share a stack: showing the second leaves the first
        // mounted but no longer active.
        app.toggle_panel(Panel::Objects);
        let objects = egui_shell::dock::PanelId::new(Panel::Objects.command_id());
        let properties = egui_shell::dock::PanelId::new(Panel::Properties.command_id());
        app.toggle_panel(Panel::Properties);

        if !app.dock.layout().contains(&objects) || app.dock.is_on_screen(&objects) {
            // The default arrangement did not stack these two, so this test has
            // nothing to say. Asserted rather than silently passing, because a
            // test that proves nothing must not read as evidence.
            assert!(
                app.dock.layout().contains(&objects),
                "Objects should still be mounted after Properties was shown"
            );
            return;
        }

        assert!(
            app.dock.layout().contains(&objects) && !app.dock.is_on_screen(&objects),
            "Objects is mounted and not on screen — the middle state"
        );
        app.toggle_panel(Panel::Objects);
        assert!(
            app.dock.is_on_screen(&objects),
            "pressing a hidden-but-mounted panel's control must raise it"
        );
        assert!(
            app.dock.layout().contains(&properties),
            "…and must not have closed the sibling"
        );
    }
}
