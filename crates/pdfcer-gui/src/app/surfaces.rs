//! # `app::surfaces` — the three regions the application draws, and nothing
//! else
//!
//! Split out of [`crate::app`] on 2026-08-20, when that file crossed rule R2's
//! 1,500-line ceiling for the fifth time. The four earlier splits produced
//! `dispatch.rs` (*what does this verb do*), `conditions.rs` (*what is true
//! right now*), `gating.rs` (*what is this mode allowed to do*) and `frame.rs`
//! (*what happens, in what order, sixty times a second*).
//!
//! ## The seam, which `app/mod.rs`'s own header had already drawn
//!
//! It read, before this split:
//!
//! > `app/mod.rs` answers *"what is the application, and how is it built?"* —
//! > the state, its fields, its one constructor, and the two surfaces
//! > (`ribbon_band`, `docks`) that are pure layout.
//!
//! Two subjects in one sentence, joined by an "and". This file is the second
//! of them, plus the third surface (`central`) that had been living below
//! `configure_context` for no reason anybody had written down.
//!
//! What each of the three answers:
//!
//! | | question |
//! |---|---|
//! | `ribbon_band` | *what does the manifest say goes at the top, and what did the operator invoke there?* |
//! | `docks` | *where are the panels, and what did they report?* |
//! | `central` | *what goes in the space that is left — a page, or a sentence about why there is no page?* |
//!
//! All three are **layout**: they draw, they collect intent, and they apply
//! nothing. The ordering constraints *between* them are `frame.rs`'s, which is
//! the file to read when the question is *when*; this one answers *what*.

use eframe::egui;

use super::actions::Action;
use super::state::Status;
use super::{DOCK_SLOT, PdfcerApp, REGION_STATUS_MESSAGE, actions};

impl PdfcerApp {
    /// Draw the ribbon and translate what the operator invoked.
    ///
    /// # ★ The one custom item, and why it is not a command
    ///
    /// `Item::Custom` is `egui-shell`'s extension point for a control that is
    /// not a button — its own doc names *"a split button with a gallery"* —
    /// and the Recent menu is one: a `Command` item can only render as a
    /// button, and a button cannot ask *which* of ten documents. The renderer
    /// therefore draws and reports, nothing else: the path is parked in
    /// [`Self::recent_choice`] and the `file.recent` token is returned, so the
    /// command goes through [`Self::dispatch_command`] exactly as a ribbon
    /// click does. See [`crate::app::recent::menu`] for the control itself and
    /// [`crate::shell::manifest::CUSTOM_BACKED`] for why the command is on no
    /// tab. An unknown `kind` draws **nothing** and returns `None`, which is
    /// why the manifest's unbuilt `colour_swatch` leaves a gap rather than a
    /// mystery widget.
    pub(super) fn ribbon_band(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        let Some(shell) = self.shell.as_ref() else {
            return;
        };
        let conditions = self.conditions(ui.ctx());

        // The rect sink. Every group caption and mode segment publishes its
        // rect under a stable name, which is what lets `ui-verify` assert
        // legibility on the frame the rect was measured on instead of
        // hard-coding fractions that go stale the first time a panel moves
        // (`PROJECT_PLAN.md` §4.2 prerequisite 1).
        //
        // ★★ **`ui_rect`, and not `ui_rect_visible` — unlike the dock's sink in
        // `Self::docks`, and the asymmetry is a decision rather than an
        // omission.**
        //
        // A ribbon rect is *content*: a group's rectangle is whatever its
        // controls laid out to, a caption's is a galley. Content-sized regions
        // are the exact shape the RAG entry
        // `a_visibility_gated_region_disappears_when_the_section_is_taller_than_its_slot`
        // warns about — a visibility gate deletes them from the trace precisely
        // when they overflow, which is when a check most wants to see them. A
        // group caption 40 % clipped by a narrow window is still a legitimate
        // thing to assert legibility on and still the answer to *did this group
        // draw at all*; a dock panel 40 % visible is not reachable, and that is
        // the difference.
        //
        // The band also already carries one documented silent-drop mechanism —
        // `a_ribbon_group_that_collapses_at_the_default_window_width_makes_a_driven_check_skip_forever`,
        // where a collapsed group stops publishing `ribbon.item.*` and a check
        // SKIPs forever without ever going red. Stacking a second silent filter
        // on the same stream multiplies the ways a check can quietly stop
        // running. If a specific ribbon check ever needs the stronger claim,
        // the shell's `RectReport` is the shape to copy and the decision has to
        // be made per region name.
        let mut report_rect = |name: &str, rect: egui::Rect| {
            crate::diag::ui_rect(name, rect);
        };

        // Resolved before the closure, because inside it `self` is not
        // reachable: the closure borrows `self.recent` mutably while
        // `&self.commands` is handed to the ribbon. `Option` rather than an
        // expectation — a build whose registry has no `file.recent` draws no
        // menu instead of panicking in the paint loop.
        let recent_token = self
            .commands
            .get(crate::shell::commands::FILE_RECENT)
            .map(|c| c.handler);
        let recent = &mut self.recent;
        let mut chosen: Option<std::path::PathBuf> = None;
        // ★ The pen, borrowed for the closure. `Pen` is `Copy`, so the
        // closure takes a `&mut` to the field rather than a copy — a copy would
        // let the operator move a slider and have the change discarded when the
        // frame ended, which is the shape of bug that produces "the control
        // does nothing" reports.
        let pen = &mut self.pen;
        // ★ The Format ▸ Font group's three custom controls, borrowed as four
        // disjoint fields. `&self.status` is shared while `&mut self.panels`
        // and `&mut self.font_change` are exclusive, which the borrow checker
        // allows only because they are named separately here — a
        // `&mut self` inside the closure would not compile against the
        // `&self.commands` the ribbon is holding.
        //
        // ★★ `panels.text_style_mut()` is the SAME draft the Properties
        // panel's *This text* section uses, deliberately. It carries a
        // `(page, run, epoch)` stamp and the read behind it costs 392 ms on the
        // operator's benchmark sheet, so a second draft for the ribbon would
        // pay that twice on every selection change. Whichever surface draws
        // first in the frame pays; the other gets a stamp hit. `ribbon_band`
        // runs before `docks`, so on a frame where the selection moved it is
        // this one.
        let doc = match &self.status {
            Status::Open(doc) => Some(&**doc),
            _ => None,
        };
        let font_draft = self.panels.text_style_mut();
        let font_change = &mut self.font_change;
        let registry = &self.commands;
        let mut custom = |ui: &mut egui::Ui, item: &egui_shell::ribbon::CustomItem<'_>| {
            // ★ The Markup ▸ Style controls. They return `None` — no handler
            // token — because they invoke no command: they edit the pen the
            // next gesture will use, which is application state with no undo
            // log to order against. `None` is what tells the ribbon nothing was
            // invoked, which is true.
            if item.kind == crate::shell::manifest::COLOUR_SWATCH {
                crate::canvas::markup::swatch::show(ui, pen);
                return None;
            }
            // ★★ The Font group's face chooser, size field and colour swatch.
            //
            // These DO return a token, where the pen's swatch above does not,
            // and the difference is what the control acts on: the pen is
            // application state with no undo log, while these three rewrite a
            // content stream and land in the engine's command log. A capability
            // that edits the document is a registered command (R8), and a
            // registered command is invoked through `dispatch_command` — the
            // same choke point a chord and a context-menu row reach — rather
            // than by a panel-side shortcut into the action queue.
            //
            // The operand is parked in `self.font_change` for the length of
            // this frame, in `file.recent`'s shape and for its reason: a
            // `HandlerToken` has no room for "Helvetica-Bold".
            if let Some(token) = crate::app::fontband::draw(
                ui,
                item.kind,
                registry,
                &conditions,
                doc,
                font_draft,
                font_change,
            ) {
                return Some(token);
            }
            if item.kind != crate::shell::manifest::RECENT_FILES {
                return None;
            }
            // A build whose registry has no `file.recent` draws no menu at
            // all, rather than a menu whose choice nothing could act on. Same
            // posture as `R8`: a capability that is not compiled in renders
            // nothing.
            let token = recent_token?;
            let picked = crate::app::recent::menu(ui, recent, std::time::Instant::now())?;
            chosen = Some(picked);
            Some(token)
        };

        // ★ The icon painter, and what supplying it actually changes.
        //
        // `egui_shell::ribbon::qat`'s `shows_label` draws a control icon-only
        // only when three things hold: the command names an icon, it has a
        // tooltip to be that icon's accessible name, and **the application
        // supplied a painter**. The third clause exists because an earlier
        // build registered icon keys, supplied no painter, and produced a row
        // of blank boxes.
        //
        // So until this line the whole ribbon fell back to text buttons —
        // which is exactly what the first packaged build did, and the trace
        // said so plainly: `ribbon.qat.file.open` was 73 pt wide where an
        // icon-only control is about 18. The icons had landed, the painter
        // existed and was tested, and nothing drew a glyph, because the one
        // line that connects them was never written.
        //
        // A plain `fn` item satisfies the `FnMut` bound, so there is no
        // closure and no captured state — which is the property worth
        // keeping: a painter with no state cannot be the thing that goes
        // stale.
        let mut icons = crate::icons::paint_ribbon_icon;

        let tokens = egui::Panel::top("ribbon")
            .show(ui, |ui| {
                egui_shell::ribbon::Ribbon::new()
                    .with_conditions(&conditions)
                    .reporting_rects_to(&mut report_rect)
                    .with_custom_items(&mut custom)
                    .with_icon_painter(&mut icons)
                    .render(ui, shell, &self.commands, &mut self.ribbon)
            })
            .inner;

        // Park the operand BEFORE dispatching the token that consumes it.
        // Reversing these two lines would make the `file.recent` arm find an
        // empty slot and fall back to the newest entry — which is a defined
        // behaviour, and therefore a defect with no symptom except that the
        // operator's third choice opened their first.
        if chosen.is_some() {
            self.recent_choice = chosen;
        }

        for token in tokens {
            self.dispatch_token(ui.ctx(), token, actions);
        }
    }

    /// Draw the left and right docks and their panel bodies.
    ///
    /// The dock knows nothing about PDFs — it is handed opaque
    /// [`egui_shell::dock::PanelId`]s and hands them back, and this closure
    /// is the single place a `PanelId` becomes a `crate::panels::Panel`.
    /// One dispatcher, exactly as the ribbon has one: an id that does not
    /// resolve draws its own explanation rather than an empty pane, because
    /// an empty pane is indistinguishable from a panel that had nothing to
    /// say.
    /// **Draw every floating panel's window.**
    ///
    /// The second of the dock's two per-frame calls. It takes an
    /// `&egui::Context` rather than a `&mut Ui` because it opens child
    /// viewports, and a child viewport must be opened from the top of the
    /// frame rather than from inside a half-composed side panel — see
    /// `egui_shell::dock::floatwin`'s header, and `crate::dialogs::host`,
    /// which is called from the same place for the same reason.
    ///
    /// # ★★★ The body closure is the SAME ONE `docks` uses
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
    /// # ★★ Every rect is tagged with its own viewport
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
                // ★★★ **THE WINDOW HAS TO SAY WHERE IT IS, and until 2026-09-04
                // it did not.**
                //
                // `panels_float_close_and_dock` drove the real binary and
                // reported *"3 of the four panel-window properties failed: no
                // `viewport-inner`"* — while the three STATE transitions all
                // fired correctly (`panel-float moved=true`, `panel-dock
                // moved=true`, `panel-close closed=true`). The panel really was
                // tearing out, docking back and closing; what no harness could
                // see was whether a WINDOW ever appeared.
                //
                // ⇒ That is the exact hole `diag::viewport_inner`'s own doc
                // comment describes for dialogs: *"the only way a check can
                // assert that a dialog opened in its own window at all … a
                // build that reverted to an in-viewport panel emits no
                // `viewport-inner` line, and its absence is the failure."* A
                // floated panel is the same act by the same mechanism, and it
                // was publishing the same nothing.
                //
                // ★★ It is also the coordinate every `ui-rect` below is
                // relative to. `ViewportScope` tags them with this viewport, but
                // a tag is not an origin — without this line a check that
                // resolves a region inside a floated panel aims at the
                // APPLICATION window's origin, hundreds of points away, and
                // clicks whatever happens to be there. `D:/dev/rag/egui/`
                // records that failure twice; both cost days and both presented
                // as *"the click lands somewhere else"*.
                //
                // ★ Read from `ViewportInfo`, on change only, exactly as
                // `dialogs::host` does — one line, one mechanism, so a window
                // and a dialog cannot come to report their geometry two
                // different ways.
                if let Some(inner) = ui.ctx().input(|i| i.viewport().inner_rect) {
                    crate::diag::viewport_inner(vp, inner);
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
        crate::diag::trace_changed("float-windows", || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "float-windows drawn={} real={} closed={:?} docked={:?}",
                report.drawn.len(),
                report.real_windows,
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

    pub(super) fn docks(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        // Borrows split before the closure: the body needs `status` and
        // `panels` while `show` holds `dock` mutably, and the closure
        // cannot reach through `self` for them.
        // Conditions are computed before the destructure, because building
        // them needs `&self` and the destructure takes it apart.
        let conditions = self.conditions(ui.ctx());

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

        // The right-click host. `None` if the manifest failed to validate —
        // in which case there is no ribbon either, and a context menu
        // offering commands the ribbon cannot show would be the only route
        // to them, which is worse than none.
        let host = shell
            .as_ref()
            .map(|s| crate::shell::menus::MenuHost::new(s, commands, &conditions));

        // ★★★ **The dock's rects are published as VISIBILITY, not layout.**
        //
        // `crate::diag::ui_rect` states *"this region was laid out at these
        // coordinates"*. Until 2026-09-04 that is what every docked panel
        // published, and every driven check in `tools/ui-verify` that names a
        // `dock.…` region was reading it as *"the operator can get to this"*.
        // Those are two different claims, and the distance between them is the
        // whole of the defect this project shipped on 2026-08-10: Bookmarks,
        // Layers and Signatures unreachable in a real build, each with a rail
        // entry, each publishing a perfectly healthy rectangle, every gate
        // green. `tools/ui-verify/src/checks/preset_group_reachable.rs` spells
        // the general form out, and `SHELL_LAYOUT_PROPOSAL.md` §5 makes closing
        // it a precondition for the proposed panel rail — because no check
        // could distinguish a working rail from that defect.
        //
        // `crate::diag::ui_rect_visible` makes the stronger claim: it publishes
        // only when at least `VISIBLE_FRACTION` of the region survived the clip
        // rectangle in force where it was drawn. `egui_shell::dock` now hands
        // that clip over beside the rect (`RectReport`), so this line is the
        // one place the dock's stream is upgraded from layout to reachability.
        //
        // ★★ Why the *whole* dock stream and not a chosen subset. The RAG entry
        // `a_visibility_gated_region_disappears_when_the_section_is_taller_than_its_slot`
        // records the rule that governs this choice: gate what a check will
        // CLICK or SAMPLE, do not gate what a check asks a yes/no question
        // about, because a *content-sized* region is never 60 % inside its clip
        // and gating it deletes it exactly when it is interesting. That hazard
        // does not reach here, and the reason is structural rather than lucky:
        // **every name the dock publishes is a compartment, not content.**
        // `egui_shell::dock::plan::resolve_spans` degrades to an equal split
        // rather than letting a child overflow its container, and a panel body
        // is clipped into its stack before it is drawn — so no dock rectangle
        // is ever bigger than the thing containing it. The only way one fails
        // the fraction is the case we want reported: the dock itself is not on
        // screen.
        //
        // ⚠ And the failure mode if that reasoning is wrong is **silent**:
        // `ui_rect_visible` is deliberately quiet when a region misses the
        // threshold, so an over-applied filter turns working checks into SKIPs,
        // and a SKIP is not red. The guard against it is not this comment — it
        // is the before/after SKIP-set diff across the driven suite, which is
        // the one artefact that can catch a check that stopped running.
        //
        // The ribbon's sink, ~180 lines above, deliberately still calls
        // `ui_rect`. See `egui_shell::dock::Dock::reporting_rects_to` for the
        // per-surface argument in full.
        let mut report_rect = |r: &egui_shell::dock::RectReport<'_>| {
            publish_dock_rect(r);
        };

        // Tokens collected inside the dock body and dispatched after it, for
        // the same reason actions are applied after the frame: the closure
        // holds borrows the dispatcher needs back.
        let mut tokens = Vec::new();
        // A second `Vec`, and it has to be: `tokens` is already captured
        // mutably by the body closure below, so the tab-menu handler cannot
        // also borrow it.
        // ★★ `(PanelId, HandlerToken)` PAIRS, not bare tokens, since
        // 2026-09-04.
        //
        // Three of the four panel-layout verbs act on *the panel the
        // operator right-clicked*, and a `HandlerToken` carries no operand.
        // The handler below runs once per **drawn** tab per frame — for
        // every tab, clicked or not — so recording `tab.panel()` into a
        // field from inside it would leave that field naming whichever tab
        // was drawn last. A token, by contrast, only ever comes back from
        // the one tab whose menu row was actually chosen, so pairing it
        // with that tab's panel at the moment it is produced is **exact**
        // rather than nearly right. For a command that closes things, that
        // is the difference that matters.
        //
        // `crate::app::dispatch::panels`' header carries the two designs
        // rejected in favour of this one.
        let mut tab_tokens: Vec<(
            egui_shell::dock::PanelId,
            egui_shell::commands::HandlerToken,
        )> = Vec::new();
        // The float census, read once before the closure so the closure
        // borrows a plain value rather than the dock. Cheap: a `Vec` of the
        // ids of the panels in windows, which is empty in the overwhelmingly
        // common case.
        let floating: Vec<egui_shell::dock::PanelId> = dock
            .layout()
            .floating
            .iter()
            .map(|f| f.panel.clone())
            .collect();
        let mut tab_menu = |tab: &mut egui_shell::dock::TabMenu<'_>| {
            // The dock hands out the tab's `Response`; what a right-click on
            // it offers is the application's business, which is the whole
            // point of the seam. Supplying a handler also takes the dock's
            // built-in Close off that tab — deliberately, because two menus
            // on one `Response` are two writers of one popup id.
            //
            // ★★★ **The conditions are corrected PER TAB**, which is what
            // makes R9 hold on this menu: `view.panel_float` is
            // `shown_when("panel.docked")` and `view.panel_dock` is
            // `shown_when("panel.floating")`, so exactly one of the two is
            // drawn and the other renders NOTHING rather than a greyed row
            // the operator cannot explain.
            //
            // `MenuHost::with_conditions` is the sanctioned route and its
            // docs carry why this is not a second source of truth: it
            // corrects two named conditions to values computed one line
            // above, from the same `DockLayout` the frame's condition set
            // reads. There is no second rule for what "floating" means.
            let is_floating = floating.iter().any(|p| p == tab.panel());
            let panel = tab.panel().clone();
            for h in host.iter() {
                let conditions = h.with_conditions(&[
                    (crate::shell::menus::PANEL_DOCKED, !is_floating),
                    (crate::shell::menus::PANEL_FLOATING, is_floating),
                ]);
                tab_tokens.extend(
                    h.attach_with(tab.response(), crate::shell::menus::DOCK_TAB, &conditions)
                        .into_iter()
                        .map(|t| (panel.clone(), t)),
                );
            }
        };
        // ★★★ **The one-line tool status** — `OPERATOR_REQUESTS.md` O123.
        //
        // The strip the right dock reserves above its columns, in place of the
        // Tool panel's stack. It is a `FnMut` for the same borrow reason
        // `tab_menu` is one, and it captures only `host` and the shared `doc` —
        // deliberately nothing mutable, so it cannot compete with the body
        // closure below for `tokens`.
        //
        // ★ It draws through `crate::diag::ui_rect_visible`, and the dock
        // publishes `dock.right.banner` around it. Two regions rather than one,
        // and the pair is the point: the dock's says *the strip is on screen*,
        // the application's says *something was drawn into it*. A build whose
        // handler returned early would keep the first and lose the second,
        // which is exactly the distinction three unreachable panels shipped
        // without on 2026-08-10.
        let mut tool_banner = |ui: &mut egui::Ui| {
            crate::app::toolstatus::banner(ui, doc, host.as_ref());
        };
        // ★★★ **The left rail** — `OPERATOR_REQUESTS.md` O123 part 7 and O126.
        //
        // A third `Vec`, for `tab_tokens`' reason exactly: `tokens` is already
        // captured mutably by the body closure, so this handler cannot also
        // borrow it. Dispatched after `show` returns, in press order.
        //
        // ★ The rail's CONTENT is `shell.rail` — manifest data — so this line
        // connects a region to a document and knows nothing about what is in
        // it. `crate::app::rail` paints a row; `egui_shell::dock::rail` decides
        // which rows exist at this height; neither of them is here.
        let mut rail_tokens: Vec<egui_shell::commands::HandlerToken> = Vec::new();
        let rail_data = shell.as_ref().and_then(|s| s.rail.clone());
        let mut rail_strip = |ui: &mut egui::Ui| {
            if let Some(rail) = &rail_data {
                rail_tokens.extend(crate::app::rail::show(ui, rail, commands, &conditions));
            }
        };
        let report = egui_shell::dock::Dock::new()
            .with_registry(panel_registry)
            .with_tab_menu(&mut tab_menu)
            .with_side_banner(
                egui_shell::dock::DockSide::Right,
                crate::app::toolstatus::BANNER_HEIGHT_PTS,
                &mut tool_banner,
            )
            .with_side_rail(egui_shell::dock::DockSide::Left, &mut rail_strip)
            .reporting_rects_to(&mut report_rect)
            .show(
                ui,
                dock,
                |panel_id, ui| match crate::panels::Panel::from_command_id(panel_id.as_str()) {
                    Some(panel) => {
                        tokens.extend(panel.show(ui, doc, panels, host.as_ref(), actions));
                    }
                    None => {
                        ui.label(crate::text::panels::panel_unknown());
                    }
                },
            );

        // ★ The Dimension-groups panel's *Set scale…* hand-over.
        //
        // Read here, not inside the body, and that is a consequence of the seam
        // rather than a preference: a panel body is handed `&OpenDoc` and
        // `&mut PanelsState` and **nothing else**, so it cannot see
        // `DialogsState` and cannot open a window. It parks a `GroupId`
        // instead; this is the first line that can see both halves, because the
        // destructured borrows above end with the dock's closure.
        //
        // The same one-shot lived in `DialogsState::show` while that panel was
        // a window — see `crate::dialogs`' note where the draw used to be. The
        // guards did not move: `open_scale` still applies its own no-document
        // and already-open checks at the one place a `ScaleDialog` is built, so
        // a request arriving while one is open leaves a half-typed ratio alone.
        let scale_request = self.panels.dimension_groups.take_scale_request();

        for token in tokens {
            self.dispatch_token(ui.ctx(), token, actions);
        }
        // ★ The tab-menu tokens are dispatched SEPARATELY, and each one parks
        // its panel on the line before the dispatch. Adjacent by
        // construction: there is no statement between the write and the
        // read, which is what makes the parked operand impossible to leave
        // stale. `dispatch::panels` TAKES it rather than reading it, so it is
        // `None` again before the next iteration.
        // The rail's presses, dispatched like the ribbon's: they are ordinary
        // commands and reach the ordinary dispatcher. No new `Action` variant
        // exists for the rail and none is needed — every row on it is a
        // command the ribbon or a menu already invokes, which is what
        // `RIBBON_IA.md` P1a permits for a shortcut surface.
        for token in rail_tokens {
            self.dispatch_token(ui.ctx(), token, actions);
        }
        for (panel, token) in tab_tokens {
            self.dock_menu_panel = Some(panel);
            self.dispatch_token(ui.ctx(), token, actions);
            self.dock_menu_panel = None;
        }

        if let Some(group) = scale_request {
            self.dialogs.open_scale(&self.status, group);
        }

        // ★ Bind the mode selector to the dock, and the dock to disk.
        //
        // Two questions, deliberately in this order.
        //
        // 1. **Did the operator change mode?** Then the arrangement they are
        //    leaving is recorded and the one they are entering is applied.
        //    Compared against `modes`' own idea of the active mode rather
        //    than a flag, because the ribbon owns the selector and there is
        //    no event: it is drawn, the operator clicks, and the two differ
        //    on the next frame.
        // 2. **Did they rearrange it?** `layout_changed` is the dock's own
        //    report of a splitter drag, a tab move or a close — not a
        //    comparison this function has to re-derive.
        //
        // A third — *is a write due?* — used to be here and is now in
        // `Self::ui`, so read mode cannot hide a pending write with the dock.
        //
        // The order of 1 and 2 is load-bearing: recording after a mode change
        // would file the outgoing arrangement under the incoming mode.
        if let Some(mode) = self.ribbon.mode().map(str::to_owned)
            && self.modes.active() != Some(mode.as_str())
        {
            self.modes.on_mode_changed(
                &mode,
                &mut self.dock,
                &mut self.layout,
                &self.panel_registry,
            );
            self.on_mode_capabilities_changed(ui.ctx());
        }
        if report.layout_changed {
            let arrangement = self.dock.layout().clone();
            self.modes.record_layout(&arrangement, &mut self.layout);
        }

        // What the dock actually drew, not what the layout asked for. The
        // two differ whenever a saved layout names a panel this build does
        // not have, and the difference is the thing worth tracing.
        crate::diag::trace_changed(DOCK_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "dock panels={} overflowed={} changed={}",
                report.panels_drawn.len(),
                report.panels_overflowed,
                report.layout_changed,
            )
        });
    }
}

impl PdfcerApp {
    /// The central area: the canvas when a document is open, and an
    /// explanation when one is not.
    ///
    /// Each non-open state renders **one sentence and nothing else**. There
    /// is deliberately no "Open…" button, no retry, and no password field:
    /// S0 opens the file named on the command line and has no other way to
    /// open anything, so a control here would either not exist or not work.
    /// Saying plainly what happened, and what the operator can do about it
    /// outside the application, is the honest version of that.
    pub(super) fn central(&mut self, ui: &mut egui::Ui, actions: &mut Vec<actions::Action>) {
        // The status message's own rect, for whichever non-open arm draws
        // one. Declared through `ui_rect` so a legibility check measures the
        // text the application actually laid out rather than a fraction of
        // the window written into the harness — see `crate::diag::ui_rect`.
        //
        // `.inner.rect` and not `.response.rect`: `centered_and_justified`
        // returns the JUSTIFIED CONTAINER as its response, which is the whole
        // available area, while the label is drawn centred inside it.
        // Reporting the container would name a region that is mostly empty
        // background, and a contrast measurement over it would be dominated
        // by pixels the sentence never touched — a real measurement of the
        // wrong thing, which is the failure mode this whole mechanism exists
        // to prevent.
        // Built before the `&mut self.status` borrow, because the host reads
        // the shell and the registry that live beside it.
        let conditions = self.conditions(ui.ctx());
        let host = self
            .shell
            .as_ref()
            .map(|s| crate::shell::menus::MenuHost::new(s, &self.commands, &conditions));

        // The canvas is drawn first and dispatched second, in two statements
        // rather than one match arm, because `dispatch_token` needs `&self` —
        // `format.delete` reads the selection off the open document — and the
        // arm holds `&mut self.status`. Letting the borrow end at the `if let`
        // is the whole reason this is not simply the first arm below.
        // ★ Two disjoint field borrows, and they have to be taken through
        // `self` in one expression: the canvas needs `&mut` on the open
        // document (it writes the three documented bookkeeping fields) and
        // `&` on the find state (it reads the hits to draw them). Binding
        // either one first and then reaching for the other through `self`
        // would be a second borrow of the whole struct.
        // Sampled before the `&mut self.status` borrow, and by value: it reads
        // `self.shell` and `self.ribbon`, both of which are disjoint from the
        // document but not provably so through a single `self`.
        let caps = self.capabilities();
        // ★ Sampled by value here for the same reason `caps` and `pen` are:
        // `PickFilter` is `Copy`, and taking a snapshot before the `&mut
        // self.status` borrow is what lets the canvas see one consistent
        // filter for the whole frame without a second borrow of `self`.
        let pick = self.pick_filter;
        let pen = self.pen;
        // The three per-frame samples, bundled — see `canvas::Sampled` for why
        // they belong together and why they are read here rather than inside.
        let max_zoom_percent = self.prefs.max_zoom_percent;
        let sampled = crate::canvas::Sampled {
            caps,
            pick,
            max_zoom_percent,
            pen,
        };
        let find = &self.find;
        if let Status::Open(doc) = &mut self.status {
            let tokens = crate::canvas::show(ui, doc, host.as_ref(), find, sampled, actions);
            // ★★★ **The note pop-ups, in their own floating layer, after the
            // canvas has laid itself out.**
            //
            // One statement, and it is here rather than inside `canvas::show`
            // for a reason `crate::canvas::painting`'s header states: that
            // module's layer order is its content, every position in it is an
            // argument, and **a pop-up has no position in it** — it is an
            // `egui::Area`, drawn above the page raster in a layer of its own
            // and composited into nothing that is ever saved.
            //
            // ★ It runs *after* `canvas::show` because it reads that call's
            // own published mapping (`canvas::zoom::last_frame`), which
            // `canvas::present` records before it hands the frame to
            // `interact`. Running it first would draw every window against the
            // previous frame's zoom and pan — a one-frame lag visible as
            // windows sliding after the page during a drag.
            crate::canvas::notepopup::show(ui.ctx(), doc, caps, actions);
            // Dispatched here rather than inside `show`: the canvas reports
            // intent and the application decides, which is the same seam the
            // ribbon and the dock already use.
            for token in tokens {
                self.dispatch_token(ui.ctx(), token, actions);
            }
            return;
        }

        let message = match &self.status {
            // ui-text-exempt: a panic message, read from a stack trace by
            // whoever broke the two-statement structure above. Never rendered.
            Status::Open(_) => unreachable!("handled above, before the borrow ended"),
            Status::Empty => {
                ui.centered_and_justified(|ui| ui.label(crate::text::canvas_no_document()))
            }
            Status::Failed { path, message } => {
                let text = crate::text::open_failed(path, message);
                ui.centered_and_justified(|ui| ui.colored_label(ui.visuals().error_fg_color, text))
            }
            Status::Unsupported { path, message } => {
                // Deliberately NOT `error_fg_color`. "pdfcer cannot do this
                // yet" is not an error in the operator's document, and
                // painting it red would say it was — the same conflation the
                // three-way status split exists to avoid.
                let text = crate::text::open_unsupported(path, message);
                ui.centered_and_justified(|ui| ui.label(text))
            }
            // ★★★ THE CANVAS SAYS THE SAME THING THE DIALOG IS ASKING, and
            // for one afternoon it said the OPPOSITE — 2026-09-03.
            //
            // `Status::NeedsPassword` has two readers in the same frame: this
            // one, and `app::frame`'s `ask_for_password`, which opens a real
            // password dialog. The canvas's sentence still claimed *"this build
            // cannot yet prompt for a password"* — written when that was true,
            // never re-read after `dialogs::password` shipped.
            //
            // ★ It is kept rather than removed, and that is deliberate. The
            // dialog is a separate OS window: it can be dragged onto another
            // monitor, or hidden behind the application by a click on the main
            // window. A blank canvas behind it would leave an operator who did
            // exactly that with no statement anywhere about why the document is
            // not open. The rule this follows is the project's own — the canvas
            // says what STATE the document is in; the dialog is where the
            // question is answered — so the two must agree and must not
            // duplicate the ask.
            Status::NeedsPassword { path } => {
                let text = crate::text::open_needs_password(path);
                ui.centered_and_justified(|ui| ui.label(text))
            }
        };
        crate::diag::ui_rect(REGION_STATUS_MESSAGE, message.inner.rect);
    }
}

/// **Publish one dock region as a claim about VISIBILITY.**
///
/// Returns whether the region was published — `false` means the dock laid it
/// out somewhere the operator cannot see it, and the trace stays silent about
/// it on purpose.
///
/// A free function rather than the closure body it used to be, for one reason:
/// **the tests below have to be able to call the production line.** A closure
/// inlined in [`PdfcerApp::docks`] is reachable only by running the
/// application, and a change to a diagnostic channel that can only be checked
/// by running the application is a change that can be green and wrong — which
/// is the entire failure class this function exists to close.
pub(super) fn publish_dock_rect(r: &egui_shell::dock::RectReport<'_>) -> bool {
    crate::diag::ui_rect_visible(r.name, r.rect, r.clip)
}

#[cfg(test)]
mod dock_rect_tests {
    use super::publish_dock_rect;
    use eframe::egui;
    use egui_shell::dock::{
        Column, Dock, DockLayout, DockState, PanelInfo, PanelRegistry, RectReport, SideLayout,
        Stack,
    };

    /// One published region: its name, the verdict [`publish_dock_rect`]
    /// reached about it, and the two rectangles that produced the verdict.
    struct Row {
        name: String,
        published: bool,
        rect: egui::Rect,
        clip: egui::Rect,
    }

    /// Render one frame of a **real** dock in a window of the given size and
    /// return a [`Row`] per published region.
    ///
    /// ★ Deliberately the real [`Dock`], not a hand-built list of rectangles.
    /// The rectangles that matter here are the ones `egui` and the dock's own
    /// geometry produce together at a window size nobody laid out for, and a
    /// fixture written by hand could only contain the numbers its author
    /// already expected.
    fn rows(window: egui::Vec2) -> Vec<Row> {
        let ctx = egui::Context::default();
        let mut registry = PanelRegistry::new();
        registry.register(PanelInfo::new("p0", "Bookmarks").with_tooltip("Bookmarks — the tree"));
        registry.register(PanelInfo::new("p1", "Layers").with_tooltip("Layers — what shows"));
        let mut state = DockState::new(DockLayout::new(
            SideLayout::new([Column::new([Stack::tabbed(vec![
                "p0".to_string(),
                "p1".to_string(),
            ])])]),
            SideLayout::none(),
        ));
        let mut out = Vec::new();
        {
            let mut sink = |r: &RectReport<'_>| {
                out.push(Row {
                    name: r.name.to_owned(),
                    published: publish_dock_rect(r),
                    rect: r.rect,
                    clip: r.clip,
                });
            };
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, window)),
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| {
                Dock::new()
                    .with_registry(&registry)
                    .reporting_rects_to(&mut sink)
                    .show(ui, &mut state, |_panel, ui| {
                        ui.label("body");
                    });
            });
        }
        out
    }

    fn verdict(rows: &[Row], name: &str) -> Option<bool> {
        rows.iter().find(|r| r.name == name).map(|r| r.published)
    }

    /// Every row, formatted for a failure message. A verdict on its own is
    /// unreadable; the two rectangles say *why*.
    fn detail(rows: &[Row]) -> String {
        rows.iter()
            .map(|r| {
                format!(
                    "\n  {:32} published={} rect={:?} clip={:?}",
                    r.name, r.published, r.rect, r.clip
                )
            })
            .collect()
    }

    /// ★★★ **A dock control laid out past the window edge must publish
    /// nothing.**
    ///
    /// This is the test that fails on the behaviour this change replaced, and
    /// the state it drives is not contrived. `egui_shell::dock::plan`'s
    /// `MIN_SIDE_WIDTH` is a hard floor of 160 pt that wins over the window —
    /// `DockLayout::drawn_side_width` clamps *up* to it — and `egui::Panel`
    /// honours an `exact_size` wider than the space it was given. So in a
    /// window narrower than 160 pt the side is drawn 160 pt wide and clipped,
    /// and everything the dock puts at the side's **trailing edge** — the
    /// collapse chevron that minimises it, and the splitter that resizes it —
    /// lands outside the clip entirely.
    ///
    /// Measured, at a 120 pt window (the numbers this test asserts against):
    ///
    /// ```text
    /// dock.left             rect=[0..160]   clip=[0..120]   0.750 visible
    /// dock.left.split.side  rect=[154..160] clip=[0..120]   0.000 visible
    /// dock.left.collapse    rect=[138..154] clip=[0..120]   0.000 visible
    /// dock.body.p0          rect=[0..154]   clip=[0..120]   0.779 visible
    /// ```
    ///
    /// **Both zero-visibility regions used to publish an ordinary-looking
    /// rectangle**, and a driven check asserting "the collapse control is
    /// there" would have passed on a build where the operator could not reach
    /// it by any means. That is the shape of the 2026-08-10 defect —
    /// Bookmarks, Layers and Signatures shipping unreachable with a rail entry
    /// and every gate green — reproduced at unit scale.
    ///
    /// ★★ The assertions run in **both** directions on purpose. Asserting only
    /// the silences would be satisfied by a filter that dropped the whole dock
    /// stream, which is the over-application hazard: `ui_rect_visible` is
    /// deliberately silent, so a filter that is too aggressive turns working
    /// checks into SKIPs, and a SKIP is not red. The body and the tab are
    /// asserted PRESENT for that reason and no other.
    #[test]
    fn a_dock_control_laid_out_past_the_window_edge_publishes_nothing() {
        let rows = rows(egui::Vec2::new(120.0, 800.0));
        assert!(!rows.is_empty(), "the dock published nothing at all");

        for name in ["dock.left.collapse", "dock.left.split.side"] {
            assert_eq!(
                verdict(&rows, name),
                Some(false),
                "{name} is laid out entirely outside the clip and must NOT be \
                 published — a rect for it is a claim that the operator can \
                 reach a control which is off screen.{}",
                detail(&rows)
            );
        }

        for name in ["dock.left", "dock.body.p0", "dock.tab.p0"] {
            assert_eq!(
                verdict(&rows, name),
                Some(true),
                "{name} is mostly inside the clip and MUST still be published \
                 — dropping it would silently turn every check that names it \
                 into a SKIP, and a SKIP is not red.{}",
                detail(&rows)
            );
        }
    }

    /// At an ordinary window size **nothing** is filtered.
    ///
    /// The companion to the test above, and the one that catches
    /// over-application. Every region the dock publishes at 1280 × 800 is
    /// fully inside its clip, so the gate must be a no-op there. If this ever
    /// fails, the filter has begun eating regions that driven checks
    /// legitimately need — in silence, because that is what the gate does.
    #[test]
    fn at_an_ordinary_window_size_the_visibility_gate_drops_nothing() {
        let rows = rows(egui::Vec2::new(1280.0, 800.0));
        assert!(
            rows.len() >= 8,
            "too few regions to be a real frame{}",
            detail(&rows)
        );
        let dropped: Vec<&str> = rows
            .iter()
            .filter(|r| !r.published)
            .map(|r| r.name.as_str())
            .collect();
        assert!(
            dropped.is_empty(),
            "the gate dropped {dropped:?} in an ordinary window; every one of \
             those is a check that has silently stopped running{}",
            detail(&rows)
        );
    }
}

/// ★★★ **The float windows are drawn, and forgetting to draw them is
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
/// **That is the exact class of defect this project shipped on 2026-08-10**:
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

    /// ★★★ **An application that calls both halves reports nothing
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

    /// ★★★ **An application that forgets `show_floating` is caught.**
    ///
    /// The falsification, written as a test rather than performed by hand:
    /// the same fixture, the same frames, and the second call simply not
    /// made. If this ever reports zero, the guard has stopped guarding and
    /// the next tear-out consumer ships three unreachable panels the way this
    /// project already has once.
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

    /// ★★ **The docked half never draws a floating panel**, which is the
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

    /// ★ **A layout with no floats does not pay for the second call.**
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
