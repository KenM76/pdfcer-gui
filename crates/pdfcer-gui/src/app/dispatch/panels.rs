//! `app::dispatch::panels` — the four layout verbs that act on a panel.
//!
//! Float it, dock it back, close it, and bring every floating one home.
//! Split out of [`super`] under **R2** on 2026-09-04: `dispatch.rs` was at
//! 1,460 lines and four arms carrying their own reasoning would not fit
//! under the 1,500-line ceiling. The seam is a real one — these four are
//! the only commands in the program whose operand is *a panel* rather than
//! the document — so this is a module and not a spill file.
//!
//! # ★★★ The operand problem, which is the whole reason this file has a
//! shape at all
//!
//! Three of the four verbs act on **the panel the operator right-clicked**.
//! Nothing in [`super::PdfcerApp::dispatch_command`]'s signature carries
//! that: it is handed a command id and the application, and a command id
//! is a verb with no noun.
//!
//! Three ways to supply the noun were considered.
//!
//! | | Why not |
//! |---|---|
//! | Four commands per panel (`view.panel_float.layers`, …) | Twelve panels × three verbs is thirty-six registered commands whose only difference is a suffix, each needing a `CommandText`, a handler token that can never be reused, and a row in the reachability register. The registry would be mostly this. |
//! | A `HandlerToken` that carries data | A token is an integer the operator's saved key bindings are written against (`shell::commands::catalog`'s per-tab hundreds). Making it a payload makes a keybinding file un-writable. |
//! | **Park the panel beside the dispatch** | What this does. |
//!
//! ⇒ [`crate::app::PdfcerApp::dock_menu_panel`] is set on the line before
//! `dispatch_token` and read by the arms here. It is the same "park an
//! answer, drain it immediately" shape `crate::dialogs`' scale hand-over
//! uses, and `crate::app::surfaces`' own comment on that one states the
//! rule it follows: the parking site and the draining site are **adjacent**,
//! so the value cannot be stale.
//!
//! ★★ **It is set from the token's own origin, not from a hover.** The
//! tab-menu handler runs once per drawn tab per frame — for *every* tab,
//! whether or not anything was clicked — so a naive `dock_menu_panel =
//! tab.panel()` inside the handler would leave the field naming whichever
//! tab happened to be drawn last. Instead `surfaces` collects
//! `(PanelId, HandlerToken)` **pairs**, because a token only ever comes
//! back from the one tab whose menu row was actually chosen. The pairing is
//! exact rather than nearly right, which for a command that closes things
//! is the difference that matters.
//!
//! # Why none of these raises an `Action`
//!
//! `crate::app::actions`' funnel exists for **document** state: the things
//! an undo log holds and a save writes. A panel arrangement is neither. It
//! is chrome, it is per-operator rather than per-document, it is persisted
//! to `layout.ron` by an entirely separate debounce, and it survives
//! closing every document.
//!
//! ⇒ These four mutate `self.dock` directly, exactly as
//! `view.reset_layout` and [`crate::app::PdfcerApp::toggle_panel`] already
//! do. **No new `Action` variant was needed for any of this**, which is
//! also why `app/actions/action.rs` did not have to be split — it is at
//! 1,500 lines exactly, and adding a variant would have required it.
//!
//! # The persistence, which is the part that is easy to leave out
//!
//! A float, a dock-back and a close all change the layout, and a layout
//! change is worth saving. The dock reports its own edits through
//! [`egui_shell::dock::DockFrameReport::layout_changed`] and
//! `crate::app::surfaces` records those — but **that path only sees
//! changes the dock made**, i.e. ones that arrived as an `Intent` during
//! `Dock::show`. A command dispatched from a menu is not one of those: it
//! runs after the dock has drawn, straight against `DockState::layout_mut`.
//!
//! So every arm here calls [`record`], which is the same
//! `Modes::record_layout` the dock's own path calls. Without it the
//! operator floats a panel, quits, and finds it docked again — with no
//! error, no trace, and nothing to blame.

use egui_shell::dock::PanelId;

use crate::app::PdfcerApp;

/// The four command ids this module claims.
///
/// ★★ A **free function** taking `id`, and that shape is required rather
/// than preferred. `shell::commands::reach` parses `dispatch.rs`'s syntax
/// tree to work out which commands each guard arm claims, and it can only
/// read a guard that calls a named function with `id` — a method call on
/// `self` is *"an expression that calls nothing with `id`"* to it, and the
/// arm becomes invisible to the reachability register. An arm the register
/// cannot see is an arm that stops proving anything, which is the whole
/// point of the register.
///
/// ⇒ So the guard is this, and the body calls the method. The pair is
/// pinned by [`tests::the_guard_and_the_dispatcher_claim_the_same_ids`], so
/// a fifth verb added to one and not the other fails a named test rather
/// than becoming a control that traces `command-unimplemented`.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    matches!(
        id,
        "view.panel_float" | "view.panel_dock" | "view.panel_close" | "view.dock_all_panels"
    )
}

impl PdfcerApp {
    /// Write the current arrangement to the active mode's workspace and
    /// mark it for the debounced save.
    ///
    /// A method rather than four copies of two lines, because the two
    /// lines are not the interesting part — *remembering to call them at
    /// all* is, and a named verb is what a reviewer can check for at each
    /// of four sites.
    fn record_panel_layout(&mut self) {
        let layout = self.dock.layout().clone();
        self.modes.record_layout(&layout, &mut self.layout);
    }

    /// The panel a `dock.tab` menu row was chosen on, if this dispatch
    /// came from one.
    ///
    /// Taken rather than read: a parked operand that survived its dispatch
    /// would be available to the *next* command, which is how a Close
    /// meant for one panel comes to act on another. The whole value of
    /// parking-and-draining is that the window is one call wide.
    fn take_menu_panel(&mut self) -> Option<PanelId> {
        self.dock_menu_panel.take()
    }

    /// Dispatch one of the four panel-layout commands.
    ///
    /// Returns `false` when `id` is not one of them, so
    /// [`super::PdfcerApp::dispatch_command`] can fall through to its
    /// other arms — the shape a guard arm needs when the set it claims is
    /// a fixed list of literals rather than a predicate.
    pub(in crate::app) fn dispatch_panel_layout(&mut self, id: &str) -> bool {
        match id {
            // ★★ **Float** — tear the right-clicked panel out into a
            // window.
            //
            // `DockLayout::float` answers `false` for a panel that is not
            // docked, which covers a stale operand and a panel that is
            // already floating. The trace records the verdict rather than
            // the attempt, because "the command ran" and "the panel
            // moved" are different facts and a harness needs the second.
            "view.panel_float" => {
                let panel = self.take_menu_panel();
                let moved = panel
                    .as_ref()
                    .is_some_and(|p| self.dock.layout_mut().float(p));
                if moved {
                    self.record_panel_layout();
                }
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "panel-float panel={:?} moved={moved}",
                        panel.as_ref().map(PanelId::as_str)
                    )
                });
                true
            }
            // **Dock back** — the mirror. See
            // `egui_shell::dock::float::DockLayout::dock_back` for why the
            // home is rebuilt rather than clamped into.
            "view.panel_dock" => {
                let panel = self.take_menu_panel();
                let moved = panel
                    .as_ref()
                    .is_some_and(|p| self.dock.layout_mut().dock_back(p));
                if moved {
                    self.record_panel_layout();
                }
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "panel-dock panel={:?} moved={moved}",
                        panel.as_ref().map(PanelId::as_str)
                    )
                });
                true
            }
            // ★ **Close** — one verb for both states.
            //
            // `DockLayout::close` handles a floating panel by removing its
            // float entry, and a docked one by removing its tab and
            // pruning whatever that empties. The application does not
            // branch on which, deliberately: the operator asked for the
            // panel to go away, and a close that behaved differently
            // depending on where the panel happened to be would be two
            // commands sharing a name.
            "view.panel_close" => {
                let panel = self.take_menu_panel();
                let closed = panel
                    .as_ref()
                    .is_some_and(|p| self.dock.layout_mut().close(p));
                if closed {
                    self.record_panel_layout();
                }
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "panel-close panel={:?} closed={closed}",
                        panel.as_ref().map(PanelId::as_str)
                    )
                });
                true
            }
            // ★★★ **Dock all** — the recovery verb, and the only one of
            // the four with no operand.
            //
            // It takes none deliberately: the state it exists to recover
            // from is one where the operator cannot point at the window,
            // so a version that needed them to name a panel would be
            // unusable exactly when it is needed. See
            // `crate::text::commands::view_dock_all_panels` for the whole
            // argument, and `egui_shell::dock::float::honour_position` for
            // what the heuristic half cannot promise and this can.
            "view.dock_all_panels" => {
                // The parked operand is dropped rather than read: this
                // command can be raised from the ribbon, where there is no
                // panel, and leaving a stale one parked would hand it to
                // whatever ran next.
                let _ = self.take_menu_panel();
                let docked = self.dock.layout_mut().dock_all_floating();
                if docked > 0 {
                    self.record_panel_layout();
                }
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("panels-dock-all docked={docked}")
                });
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use egui_shell::dock::{Column, DockLayout, DockSide, PanelId, SideLayout, Stack};

    /// The three panel ids this module's tests speak, as the application
    /// spells them — so a rename of a panel's command id breaks these
    /// rather than leaving them asserting about strings nothing uses.
    fn layers() -> PanelId {
        PanelId::new(crate::panels::Panel::Layers.command_id())
    }

    fn objects() -> PanelId {
        PanelId::new(crate::panels::Panel::Objects.command_id())
    }

    fn sample() -> DockLayout {
        DockLayout::new(
            SideLayout::new([Column::new([Stack::tabbed([
                crate::panels::Panel::Pages.command_id(),
                crate::panels::Panel::Layers.command_id(),
            ])])]),
            SideLayout::new([Column::new([Stack::new(
                crate::panels::Panel::Objects.command_id(),
            )])]),
        )
    }

    /// ★★★ **The guard and the dispatcher claim exactly the same ids.**
    ///
    /// Two lists, one obligation. `claims` is what
    /// `shell::commands::reach` reads out of `dispatch.rs` to decide that
    /// these four commands are routed; `dispatch_panel_layout`'s `match` is
    /// what actually routes them. A command in the first and not the second
    /// is a control that presses and does nothing while the register calls
    /// it reachable — which is `file.save_copy`'s defect exactly, and the
    /// register exists to end it.
    #[test]
    fn the_guard_and_the_dispatcher_claim_the_same_ids() {
        // Every id the guard claims must be one the dispatcher handles.
        for id in [
            "view.panel_float",
            "view.panel_dock",
            "view.panel_close",
            "view.dock_all_panels",
        ] {
            assert!(super::claims(id), "the guard must claim `{id}`");
        }
        // And nothing else may be claimed, or the guard would swallow ids
        // its `match` falls through on — which in a `match` guard means the
        // arm runs, does nothing, and the id never reaches the arms below.
        for id in ["view.reset_layout", "view.panel_layers", "file.open", ""] {
            assert!(!super::claims(id), "the guard must not claim `{id}`");
        }
    }

    /// ★★ **Every panel this build can draw can be floated and docked
    /// back**, and the round trip is the identity.
    ///
    /// A sweep rather than one case, because the failure this guards
    /// against is per-panel: a panel that is alone in its stack takes a
    /// different path through `dock_back` (the stack is pruned and has to
    /// be rebuilt) from one that is tabbed with a sibling, and a test that
    /// only exercised one shape would pass while the other silently
    /// merged compartments.
    #[test]
    fn every_panel_survives_a_float_and_dock_round_trip() {
        for panel in crate::panels::Panel::ALL {
            let id = PanelId::new(panel.command_id());
            let mut layout = sample();
            // Mount it somewhere if the sample does not already hold it,
            // so the sweep covers all twelve rather than the three the
            // sample names.
            if !layout.contains(&id) {
                layout.mount(DockSide::Left, 0, 0, id.clone());
            }
            let before = layout.clone();
            let addresses_before: Vec<_> = before
                .docked_panels()
                .map(|p| (p.clone(), before.find(p)))
                .collect();
            assert!(
                layout.float(&id),
                "{} could not be floated",
                panel.command_id()
            );
            assert!(layout.is_floating(&id));
            assert!(
                layout.dock_back(&id),
                "{} could not be docked back",
                panel.command_id()
            );
            // ★ Every panel's ADDRESS, not the whole value. The one field
            // that legitimately differs is `Stack::active`: `dock_back`
            // activates the panel it just returned, because docking a window
            // into a stack and leaving it behind another tab is a command
            // whose only visible effect is that a window vanished. Comparing
            // whole layouts would make that documented behaviour fail this
            // test, so the assertion is written against the thing actually
            // promised — the arrangement — plus the activation, below.
            for (p, was) in &addresses_before {
                assert_eq!(
                    layout.find(p),
                    *was,
                    "{}: floating {} moved {} to a different place",
                    panel.command_id(),
                    id,
                    p
                );
            }
            assert!(
                layout.is_active(&id),
                "{}: a docked-back panel must be the tab you are looking at",
                panel.command_id()
            );
            assert!(layout.floating.is_empty());
            assert!(layout.is_normalized());
        }
    }

    /// ★★★ **A floated panel is still reported as on screen**, which is
    /// what `PdfcerApp::toggle_panel` reads to decide whether choosing it
    /// from View ▸ Panels should open it or put it away.
    ///
    /// Without this the View menu would offer to "open" a panel that is
    /// already in a window in front of the operator, and choosing it would
    /// mount a second copy.
    #[test]
    fn a_floated_panel_reads_as_open_to_the_view_menu() {
        let mut layout = sample();
        layout.float(&layers());
        assert!(layout.is_on_screen(&layers()));
        assert!(
            layout.contains(&layers()),
            "and `toggle_panel`'s mount path must see it as already present"
        );
    }

    /// **Closing a floated panel leaves nothing behind**, so View ▸ Panels
    /// can mount it again from scratch.
    #[test]
    fn closing_a_floated_panel_makes_it_reopenable() {
        let mut layout = sample();
        layout.float(&layers());
        layout.close(&layers());
        assert!(!layout.contains(&layers()));
        assert!(!layout.is_on_screen(&layers()));
        layout.mount(DockSide::Left, 0, 0, layers());
        assert!(
            layout.contains_docked(&layers()),
            "a closed panel must be reopenable from the View tab"
        );
    }

    /// ★★ **Closing the last panel on a side leaves no dead column and no
    /// unreachable state.**
    ///
    /// The right dock holds exactly one panel in this sample. Closing it
    /// must prune the stack, the column and — as far as the drawing code
    /// is concerned — the side, so nothing draws an empty grey strip. The
    /// panel is still reachable, because View ▸ Panels mounts by id and
    /// not by address.
    #[test]
    fn closing_the_last_panel_on_a_side_prunes_the_side() {
        let mut layout = sample();
        assert!(layout.close(&objects()));
        assert!(
            layout.side(DockSide::Right).is_empty(),
            "the emptied side must report empty, so the dock draws neither a column nor a rail"
        );
        assert!(layout.is_normalized());
        layout.mount(DockSide::Right, 0, 0, objects());
        assert!(layout.contains_docked(&objects()), "and it comes back");
    }

    /// **Dock-all recovers every float**, whatever side they came from.
    #[test]
    fn dock_all_recovers_floats_from_both_sides() {
        let mut layout = sample();
        layout.float(&layers());
        layout.float(&objects());
        assert_eq!(layout.dock_all_floating(), 2);
        assert!(layout.contains_docked(&layers()));
        assert!(layout.contains_docked(&objects()));
    }
}
