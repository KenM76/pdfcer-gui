//! # `app::floats` — the panels that are not in the dock
//!
//! The dock's **second** per-frame call. `app::surfaces` draws the three
//! regions inside the application window; this file draws the windows outside
//! it, one per floating panel, each in its own child viewport.
//!
//! It is a file of its own for the reason a child viewport is a call of its
//! own: everything here is conditioned on being in a different window from the
//! one the layout was measured in. Which viewport a rect belongs to, which
//! origin a pointer is reported against, and which closure the shell will call
//! before the body — all three are questions `surfaces.rs` never has to ask
//! and this file cannot stop asking.
//!
//! Ordering against the docked half is `frame.rs`'s subject, and the invariant
//! it turns on is stated on [`PdfcerApp::floating_panels`]: the docked panels
//! draw first, so a floating panel reads state one frame later than a docked
//! one would.

use eframe::egui;

use super::PdfcerApp;
use super::actions::Action;
use super::state::Status;

impl PdfcerApp {
    /// **Draw every floating panel's window.**
    ///
    /// The second of the dock's two per-frame calls. It takes an
    /// `&egui::Context` rather than a `&mut Ui` because it opens child
    /// viewports, and a child viewport must be opened from the top of the
    /// frame rather than from inside a half-composed side panel — see
    /// `egui_shell::dock::floatwin`'s header, and `crate::dialogs::host`,
    /// which is called from the same place for the same reason.
    ///
    /// # The body closure is the SAME ONE `docks` uses
    ///
    /// Not a similar one — the same expression, resolving the same
    /// `PanelId` through the same `Panel::from_command_id` and calling the
    /// same `Panel::show`. That is the property `MODES_AND_PANELS.md`
    /// identified as the thing that makes tear-out cheap here:
    /// `show_viewport_immediate` takes `FnMut` with **no** `Send + Sync +
    /// 'static` bound, so a torn-out panel keeps the docked signature and
    /// there is no second rendering path to keep in step.
    ///
    /// A previous float-or-dock dual mode is on record as costing *"two
    /// code paths for the same content, each duplicating open-state,
    /// position/size and focus handling"*. This has one.
    ///
    /// # Every rect is tagged with its own viewport
    ///
    /// A child viewport's coordinates start at **its** origin, so an
    /// untagged `ui-rect` from a float window reads to a harness as a
    /// position in the application window — plausible numbers naming a
    /// different place on the desktop, which
    /// `D:/dev/rag/egui/a_child_viewports_ui_rects_are_relative_to_ITS_origin…`
    /// records as a harness aiming hundreds of points away. The shell has
    /// no diagnostic channel of its own, so the scope is entered here,
    /// inside the body, from the very id the shell used — recovered
    /// through `floatwin::viewport_id`, which is public for this.
    ///
    /// # The draw-order invariant, and why it HOLDS
    ///
    /// `D:/dev/rag/egui/moving_a_surface_into_a_child_viewport_breaks_the_draw_order_invariant_containment_gave_it_free.md`
    /// records the hazard this change is the exact shape of: *"containment
    /// silently provides a draw order, and moving a surface into a sibling
    /// viewport turns that invariant into the order of your two call
    /// sites."* A panel that writes into shared state while it draws, and
    /// something drawn **after** it that reads that state, is ordered for
    /// free while both are inside one window and by nothing at all once one
    /// of them is a child viewport.
    ///
    /// It is checked here rather than assumed, and the answer is that
    /// **tear-out cannot change any reader's side of the fence**, because
    /// every parent reader is ordered before *both* halves of the dock:
    ///
    /// | `PdfcerApp::ui` line | what it does |
    /// |---:|---|
    /// | 132 | the ribbon's Font group takes `panels.text_style_mut()` |
    /// | 668 | the ribbon band draws |
    /// | 691 | `dimension_groups.take_scale_request()` |
    /// | **831** | **`docks` — the DOCKED panel bodies** |
    /// | 938 | the dialogs (which read no panel state) |
    /// | **970** | **`floating_panels` — the FLOATED panel bodies** |
    ///
    /// A panel that moves from 831 to 970 is still after 132, 668 and 691,
    /// so a reader that saw last frame's write when the panel was docked
    /// sees last frame's write when it is floating. The one-frame lag is
    /// pre-existing, identical in both states, and not this capability's.
    ///
    /// **That is a fact about the current call order, not a guarantee.**
    /// Move any reader of panel-written state to between `docks` and here
    /// and the two states diverge — the docked panel would be read this
    /// frame and the floated one next frame, which presents as a control
    /// that is correct until you tear its panel out. The status bar's layer
    /// clause is deliberately NOT such a reader:
    /// `app::status::selected::with_layer` recomputes through
    /// `panels::layers::highlight::resolve(doc)`, a pure function of the
    /// document, rather than reading anything the Layers panel cached while
    /// drawing.
    pub(super) fn floating_panels(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) {
        if self.dock.layout().floating.is_empty() {
            return;
        }
        let conditions = self.conditions(ctx);
        let Self {
            status,
            shell,
            commands,
            panel_registry,
            dock,
            panels,
            ..
        } = self;
        let doc = match status {
            Status::Open(doc) => Some(&**doc),
            _ => None,
        };
        let host = shell
            .as_ref()
            .map(|s| crate::shell::menus::MenuHost::new(s, commands, &conditions));

        let mut tokens: Vec<(
            egui_shell::dock::PanelId,
            egui_shell::commands::HandlerToken,
        )> = Vec::new();
        let mut header_tokens: Vec<(
            egui_shell::dock::PanelId,
            egui_shell::commands::HandlerToken,
        )> = Vec::new();
        // The header strip's menu is the dock tab's menu, with the two
        // per-panel conditions set the other way round: everything drawn
        // here is floating by construction, so Dock is offered and Float is
        // not.
        let mut header_menu = |tab: &mut egui_shell::dock::TabMenu<'_>| {
            let panel = tab.panel().clone();
            // **Where the header strip is**, published from the only place in
            // the program that can answer.
            //
            // The strip is the grab handle for carrying a float window back
            // over the dock, so a driven check has to aim at it — and nothing
            // else knows where it is. `egui_shell::dock::floatwin` derives it
            // from `BODY_MARGIN_PTS` and `split_header` and has no diagnostic
            // channel to say so (R7); a harness that re-derived it from those
            // two constants would be encoding the shell's geometry in the
            // instrument, and would go on aiming confidently at the old place
            // the day either constant moved.
            //
            // The scope is entered here as well as in the body closure because
            // this runs *before* it: an untagged region carries the
            // application window's origin, and a click converted against the
            // wrong origin lands somewhere plausible and wrong — the failure
            // `crate::diag::viewport_inner` exists to prevent.
            if crate::diag::enabled() {
                let _regions = crate::diag::ViewportScope::enter(
                    egui_shell::dock::floatwin::viewport_id(&panel),
                );
                // ui-text-exempt: diagnostic region name, never displayed.
                crate::diag::ui_rect(&format!("float.header.{panel}"), tab.response().rect);
            }
            for h in host.iter() {
                let conditions = h.with_conditions(&[
                    (crate::shell::menus::PANEL_DOCKED, false),
                    (crate::shell::menus::PANEL_FLOATING, true),
                ]);
                header_tokens.extend(
                    h.attach_with(tab.response(), crate::shell::menus::DOCK_TAB, &conditions)
                        .into_iter()
                        .map(|t| (panel.clone(), t)),
                );
            }
        };

        let report = egui_shell::dock::Dock::new()
            .with_registry(panel_registry)
            .with_tab_menu(&mut header_menu)
            .show_floating(ctx, dock, |panel_id, ui| {
                let vp = egui_shell::dock::floatwin::viewport_id(panel_id);
                let _regions = crate::diag::ViewportScope::enter(vp);
                // **THE WINDOW HAS TO SAY WHERE IT IS.**
                //
                // The three STATE transitions a float goes through
                // (`panel-float moved=true`, `panel-dock moved=true`,
                // `panel-close closed=true`) say the panel tore out, docked
                // back and closed. Not one of them says a WINDOW ever
                // appeared, so without this line a check like
                // `panels_float_close_and_dock` can only report *"no
                // `viewport-inner`"* and cannot tell which half is broken.
                //
                // ⇒ That is the exact hole `diag::viewport_inner`'s own doc
                // comment describes for dialogs: *"the only way a check can
                // assert that a dialog opened in its own window at all … a
                // build that reverted to an in-viewport panel emits no
                // `viewport-inner` line, and its absence is the failure."* A
                // floated panel is the same act by the same mechanism, and it
                // was publishing the same nothing.
                //
                // It is also the coordinate every `ui-rect` below is
                // relative to. `ViewportScope` tags them with this viewport, but
                // a tag is not an origin — without this line a check that
                // resolves a region inside a floated panel aims at the
                // APPLICATION window's origin, hundreds of points away, and
                // clicks whatever happens to be there. `D:/dev/rag/egui/`
                // records that failure twice; both cost days and both presented
                // as *"the click lands somewhere else"*.
                //
                // Read from `ViewportInfo`, on change only, exactly as
                // `dialogs::host` does — one line, one mechanism, so a window
                // and a dialog cannot come to report their geometry two
                // different ways.
                //
                // The OUTER rectangle beside it, because the two answer
                // different questions and only one of them is the question the
                // tear-out affordance raises. `floatwin` places the window with
                // `with_position`, which takes the **outer** corner, from the
                // same `at_pts` the outline was drawn at; so *"did the window
                // open where the operator was shown it would"* is a claim about
                // the outer origin. Comparing it against the client rectangle
                // instead would be off by the host's border and title bar, and
                // reconciling that in the harness would put Windows' chrome
                // inside the instrument.
                let (inner, outer) = ui
                    .ctx()
                    .input(|i| (i.viewport().inner_rect, i.viewport().outer_rect));
                if let Some(inner) = inner {
                    crate::diag::viewport_inner(vp, inner);
                }
                if let Some(outer) = outer {
                    crate::diag::viewport_outer(vp, outer);
                }
                // **THE TWO REGIONS THAT MAKE AN EMPTY WINDOW
                // DISTINGUISHABLE FROM A FULL ONE.**
                //
                // A check asking *"does a floated panel open an OS window
                // and draw nothing inside it?"* on the presence of **any**
                // `ui-rect` carrying a `viewport=` tag is a blind oracle
                // rather than a defect detector, because without these two
                // lines nothing publishes one. Not
                // `egui_shell::dock::floatwin` — which has no diagnostic
                // channel and must not grow one (R7) — and not the Layers
                // panel it floats, whose only regions are its search field
                // and that field's clear button, and which draws NEITHER on
                // a document with no optional content: **no fixture in
                // `fixtures/` carries an `/OCProperties`**.
                //
                // ⇒ These two lines are the fix in the instrument. They are
                // published here, from inside the body closure, because this
                // is the only place in the program where all three facts are
                // in scope at once: the panel's identity, its `Ui`, and the
                // `ViewportScope` that makes the coordinates mean something.
                //
                // | region | answers |
                // |---|---|
                // | `float.body.<panel>` | the shell gave the panel a compartment, and **where** — the float twin of `dock.body.<panel>` |
                // | `float.content.<panel>` | how much of it the panel FILLED. A window whose body allocated nothing publishes a **zero-sized** rect here, and that is the empty-window defect stated as a number |
                //
                // `float.content` is published AFTER the body draws, and
                // it must be: `min_rect` before the draw is the empty
                // rectangle the `Ui` was built with, so publishing it early
                // would report every window empty. The docked path has no
                // equivalent because a docked panel that draws nothing leaves
                // a compartment the operator can see is blank, next to the
                // panel's own tab; a blank OS WINDOW names nothing and reads
                // as a crash.
                //
                // Named and formatted only when the channel is on, for
                // `egui_shell::dock::report::Reporter::is_listening`'s
                // reason: this runs per float per frame for ever.
                let listening = crate::diag::enabled();
                if listening {
                    // ui-text-exempt: diagnostic region name, never displayed.
                    crate::diag::ui_rect(&format!("float.body.{panel_id}"), ui.max_rect());
                }
                match crate::panels::Panel::from_command_id(panel_id.as_str()) {
                    Some(panel) => {
                        tokens.extend(
                            panel
                                .show(ui, doc, panels, host.as_ref(), actions)
                                .into_iter()
                                .map(|t| (panel_id.clone(), t)),
                        );
                    }
                    None => {
                        ui.label(crate::text::panels::panel_unknown());
                    }
                }
                if listening {
                    // ui-text-exempt: diagnostic region name, never displayed.
                    crate::diag::ui_rect(&format!("float.content.{panel_id}"), ui.min_rect());
                }
            });

        for (panel, token) in tokens.into_iter().chain(header_tokens) {
            self.dock_menu_panel = Some(panel);
            self.dispatch_token(ctx, token, actions);
            self.dock_menu_panel = None;
        }

        // A close or a dock-back from a float window mutates the layout
        // outside `Dock::show`'s intent queue, so the mode's workspace has
        // to be told here — the same obligation `dispatch::panels` carries
        // and for the same reason. Without it the operator docks a window,
        // quits, and finds it floating again.
        if report.layout_changed {
            let layout = self.dock.layout().clone();
            self.modes.record_layout(&layout, &mut self.layout);
        }
        // `empty=` is the shell's own answer to *"is a window open with
        // nothing in it"*, and it is here rather than left to the region
        // stream because the two are independent witnesses of the same fact:
        // `float.content.<panel>` is measured by the APPLICATION from the
        // `Ui` it drew into, and `empty_bodies` is measured by the DOCK from
        // the `Ui` it handed over. A defect that silenced one would have to
        // silence the other separately, which is the property that makes the
        // pair worth its two lines. See
        // `egui_shell::dock::floatwin::FloatFrameReport::empty_bodies`.
        crate::diag::trace_changed("float-windows", || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "float-windows drawn={} real={} empty={} closed={:?} docked={:?}",
                report.drawn.len(),
                report.real_windows,
                report.empty_bodies.len(),
                report
                    .closed
                    .as_ref()
                    .map(egui_shell::dock::PanelId::as_str),
                report
                    .docked
                    .as_ref()
                    .map(egui_shell::dock::PanelId::as_str)
            )
        });
    }
}

/// **The float windows are drawn, and forgetting to draw them is
/// detectable.**
///
/// Floating is the one dock capability that needs **two** calls per frame:
/// [`egui_shell::dock::Dock::show`] for the docked panels and
/// [`egui_shell::dock::Dock::show_floating`] for the windows. The second
/// cannot live inside the first — a child viewport must be opened from the
/// top of the frame rather than from inside a half-composed side panel — so
/// an application can forget it, and the symptom is a panel that is in the
/// layout, reports as on screen, and is drawn nowhere.
///
/// **That is the exact class of defect this project has shipped before**:
/// three panels laid out, publishing correct rectangles, unreachable, with
/// every gate green. `crate::diag::ui_rect_visible` is the answer for a
/// surface that has a rect; this is the answer for one whose window was
/// never opened, and it is why
/// [`egui_shell::dock::DockFrameReport::floats_undrawn`] exists at all.
#[cfg(test)]
mod float_window_tests {
    use eframe::egui;
    use egui_shell::dock::{Column, Dock, DockLayout, DockState, PanelId, SideLayout, Stack};

    /// A layout with `layers` floated out of a two-tab left stack.
    fn floated() -> DockState {
        let mut layout = DockLayout::new(
            SideLayout::new([Column::new([Stack::tabbed(vec![
                "pages".to_string(),
                "layers".to_string(),
            ])])]),
            SideLayout::none(),
        );
        assert!(
            layout.float(&PanelId::new("layers")),
            "the fixture must float"
        );
        DockState::new(layout)
    }

    fn input() -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            ..Default::default()
        }
    }

    /// **An application that calls both halves reports nothing
    /// undrawn.**
    ///
    /// Two frames, and the second is the assertion. `Dock::show` measures
    /// `floats_undrawn` against the count `show_floating` recorded on the
    /// PREVIOUS frame — because `show` runs first — so a genuine first frame
    /// reports the float it is about to draw and the number settles on the
    /// next one. A harness drives two frames anyway; the test says so
    /// explicitly rather than leaving the one-frame lag to be discovered.
    #[test]
    fn drawing_both_halves_leaves_no_float_undrawn() {
        let ctx = egui::Context::default();
        let mut state = floated();
        let mut drew: Vec<String> = Vec::new();
        let mut last = 0usize;
        for _ in 0..2 {
            drew.clear();
            let _ = ctx.run_ui(input(), |ui| {
                let report = Dock::new().show(ui, &mut state, |_p, ui| {
                    ui.label("docked");
                });
                last = report.floats_undrawn;
                Dock::new().show_floating(ui.ctx(), &mut state, |panel, ui| {
                    drew.push(panel.as_str().to_owned());
                    ui.label("floated");
                });
            });
        }
        assert_eq!(
            drew,
            vec!["layers".to_string()],
            "the floated panel's body must be called exactly once per frame"
        );
        assert_eq!(
            last, 0,
            "an application that draws its float windows must report nothing undrawn"
        );
    }

    /// **An application that forgets `show_floating` is caught.**
    ///
    /// The falsification, written as a test rather than performed by hand:
    /// the same fixture, the same frames, and the second call simply not
    /// made. If this ever reports zero, the guard has stopped guarding and
    /// the next tear-out consumer ships unreachable panels with nothing red.
    #[test]
    fn forgetting_the_float_windows_is_reported_rather_than_silent() {
        let ctx = egui::Context::default();
        let mut state = floated();
        let mut last = 0usize;
        for _ in 0..2 {
            let _ = ctx.run_ui(input(), |ui| {
                let report = Dock::new().show(ui, &mut state, |_p, ui| {
                    ui.label("docked");
                });
                last = report.floats_undrawn;
                // …and `show_floating` is deliberately NOT called.
            });
        }
        assert_eq!(
            last, 1,
            "a floating panel nothing drew must be reported, not silently missing"
        );
    }

    /// **The docked half never draws a floating panel**, which is the
    /// invariant that stops one panel being drawn twice from two `Ui`s with
    /// the same widget ids.
    #[test]
    fn the_docked_half_does_not_draw_a_floating_panel() {
        let ctx = egui::Context::default();
        let mut state = floated();
        let mut docked: Vec<String> = Vec::new();
        let _ = ctx.run_ui(input(), |ui| {
            Dock::new().show(ui, &mut state, |panel, ui| {
                docked.push(panel.as_str().to_owned());
                ui.label("docked");
            });
        });
        assert_eq!(
            docked,
            vec!["pages".to_string()],
            "only the panel still in a stack may be drawn by the docked half"
        );
    }

    /// **A float window whose panel says ONE SENTENCE is not reported
    /// empty — and in THIS crate a sentence has a size.**
    ///
    /// Two claims in one test, and the second is the one worth the words.
    ///
    /// 1. `FloatFrameReport::empty_bodies` distinguishes an open window with
    ///    a panel in it from an open window with nothing in it. That is the
    ///    number a check asking *"does a floated panel open an OS window and
    ///    draw nothing inside it?"* needs and cannot otherwise get.
    /// 2. **`ui.label` measures a real rectangle here**, which is *not* true
    ///    one crate down. `egui-shell` pins `egui` with
    ///    `default-features = false` — its `Cargo.toml` says so, "so this
    ///    crate does not silently acquire fonts" — and in its tests every
    ///    galley is empty and every label is a **zero-sized** rect. The
    ///    twin of this test over there had to be written with
    ///    `allocate_space` for exactly that reason, and it says so at the
    ///    call. This crate links `eframe`, which brings the default fonts,
    ///    so the sentence an R9-correct empty panel draws is real content
    ///    and is measured as such.
    ///
    /// ⇒ Keeping both tests, one per crate, is deliberate: the pair is what
    /// says the measurement means the same thing in the shell's unit tests
    /// and in the running program, which is the only place it matters.
    #[test]
    fn a_float_window_whose_panel_says_one_sentence_is_not_reported_empty() {
        let ctx = egui::Context::default();
        let mut state = floated();
        let mut empty = vec!["unset".to_owned()];
        let _ = ctx.run_ui(input(), |ui| {
            let report = Dock::new().show_floating(ui.ctx(), &mut state, |_panel, ui| {
                // The sentence an R9-correct panel draws when the document
                // gives it nothing to list.
                ui.label("This document has no layers.");
            });
            empty = report
                .empty_bodies
                .iter()
                .map(|p| p.as_str().to_owned())
                .collect();
        });
        assert!(
            empty.is_empty(),
            "one sentence is content, and it must measure as content in the crate that has \
             fonts; got {empty:?}"
        );
    }

    /// **…and a float window whose panel draws NOTHING is named.**
    ///
    /// The falsification of the test above, in the crate the operator
    /// actually runs. Every other number the frame produces still reports
    /// success — the window is in `drawn`, `floats_undrawn` is zero — which
    /// is precisely why the empty case needed a number of its own.
    #[test]
    fn a_float_window_with_an_empty_body_is_named_rather_than_counted_a_success() {
        let ctx = egui::Context::default();
        let mut state = floated();
        let mut empty: Vec<String> = Vec::new();
        let mut drawn = 0usize;
        let _ = ctx.run_ui(input(), |ui| {
            let report = Dock::new().show_floating(ui.ctx(), &mut state, |_panel, _ui| {
                // The defect: a window is opened and the panel draws nothing.
            });
            drawn = report.drawn.len();
            empty = report
                .empty_bodies
                .iter()
                .map(|p| p.as_str().to_owned())
                .collect();
        });
        assert_eq!(drawn, 1, "the window was still opened, which is the trap");
        assert_eq!(
            empty,
            vec!["layers".to_string()],
            "and the panel with nothing in its window must be named"
        );
    }

    /// **A layout with no floats does not pay for the second call.**
    ///
    /// The common case, and the one that must stay free: an application that
    /// has never floated a panel calls `show_floating` on every frame
    /// forever, and it must return immediately.
    #[test]
    fn a_layout_with_no_floats_draws_no_windows() {
        let ctx = egui::Context::default();
        let mut state = DockState::new(DockLayout::new(
            SideLayout::new([Column::new([Stack::new("pages")])]),
            SideLayout::none(),
        ));
        let mut drew = 0usize;
        let _ = ctx.run_ui(input(), |ui| {
            let report = Dock::new().show_floating(ui.ctx(), &mut state, |_p, _ui| {
                drew += 1;
            });
            assert!(report.drawn.is_empty());
            assert!(!report.layout_changed);
        });
        assert_eq!(drew, 0);
    }
}
