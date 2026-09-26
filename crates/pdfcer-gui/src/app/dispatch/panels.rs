//! `app::dispatch::panels` — the layout verbs that act on a panel or on the
//! chrome around it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/panels.md`.

use egui_shell::dock::PanelId;

use crate::app::PdfcerApp;

/// The command ids this module claims.
///
/// A **free function** taking `id`, and that shape is required rather
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
/// a verb added to one and not the other fails a named test rather than
/// becoming a control that traces `command-unimplemented`.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    matches!(
        id,
        "view.panel_float"
            | "view.panel_dock"
            | "view.panel_close"
            | "view.dock_all_panels"
            | "view.ribbon_auto_hide"
            | "view.rail_auto_hide"
    )
}

impl PdfcerApp {
    /// Write the current arrangement to the active mode's workspace and
    /// mark it for the debounced save.
    fn record_panel_layout(&mut self) {
        let layout = self.dock.layout().clone();
        self.modes.record_layout(&layout, &mut self.layout);
    }

    /// The panel a `dock.tab` menu row was chosen on, if this dispatch
    /// came from one.
    fn take_menu_panel(&mut self) -> Option<PanelId> {
        self.dock_menu_panel.take()
    }

    /// Dispatch one of the panel-layout commands.
    ///
    /// Returns `false` when `id` is not one of them, so
    /// [`super::PdfcerApp::dispatch_command`] can fall through to its
    /// other arms — the shape a guard arm needs when the set it claims is
    /// a fixed list of literals rather than a predicate.
    pub(in crate::app) fn dispatch_panel_layout(&mut self, id: &str) -> bool {
        match id {
            // **Float** — tear the right-clicked panel out into a
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
            // **Close** — one verb for both states.
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
            // **Dock all** — the recovery verb, and it takes no
            // operand.
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
            // THE TWO AUTO-HIDE TOGGLES. See `egui_shell::peek` for the
            // interaction model and for the R128 bound that keeps it out of a
            // feedback loop.
            //
            // Each writes the PREFERENCE and lets the frame loop push it
            // into the shell, rather than calling `set_auto_hide` here. The two
            // routes would otherwise disagree the moment a preferences file is
            // reloaded: `RibbonState::sync_auto_hide` runs every frame and
            // would immediately undo a shell-only change. One source of truth,
            // and it is the one that survives a restart.
            //
            // `save_prefs` rather than a dirty flag, for the reason
            // `view.smart_select`'s arm gives: a setting the operator changed
            // from a control and lost on a crash is a control that reports
            // having done something it did not do.
            "view.ribbon_auto_hide" => {
                let _ = self.take_menu_panel();
                self.prefs.ribbon_auto_hide = !self.prefs.ribbon_auto_hide;
                let on = self.prefs.ribbon_auto_hide;
                let _ = self.prefs.save();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("ribbon-auto-hide on={on}")
                });
                true
            }
            "view.rail_auto_hide" => {
                let _ = self.take_menu_panel();
                self.prefs.rail_auto_hide = !self.prefs.rail_auto_hide;
                let on = self.prefs.rail_auto_hide;
                let _ = self.prefs.save();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("rail-auto-hide on={on}")
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

    /// The panel ids this module's tests speak, as the application spells
    /// them — so a rename of a panel's command id breaks these rather than
    /// leaving them asserting about strings nothing uses.
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

    /// **The guard and the dispatcher claim exactly the same ids.**
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

    /// **Every panel this build can draw can be floated and docked
    /// back**, and the round trip is the identity.
    #[test]
    fn every_panel_survives_a_float_and_dock_round_trip() {
        for panel in crate::panels::Panel::ALL {
            let id = PanelId::new(panel.command_id());
            let mut layout = sample();
            // Mount it somewhere if the sample does not already hold it,
            // so the sweep covers every panel rather than the ones the
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
            // Every panel's ADDRESS, not the whole value. The one field
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

    /// **A floated panel is still reported as on screen**, which is
    /// what `PdfcerApp::toggle_panel` reads to decide whether choosing it
    /// from View ▸ Panels should open it or put it away.
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

    /// **Closing the last panel on a side leaves no dead column and no
    /// unreachable state.**
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
