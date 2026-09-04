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

        let mut report_rect = |name: &str, rect: egui::Rect| {
            crate::diag::ui_rect(name, rect);
        };

        // Tokens collected inside the dock body and dispatched after it, for
        // the same reason actions are applied after the frame: the closure
        // holds borrows the dispatcher needs back.
        let mut tokens = Vec::new();
        // A second `Vec`, and it has to be: `tokens` is already captured
        // mutably by the body closure below, so the tab-menu handler cannot
        // also borrow it.
        let mut tab_tokens = Vec::new();
        let mut tab_menu = |tab: &mut egui_shell::dock::TabMenu<'_>| {
            // The dock hands out the tab's `Response`; what a right-click on
            // it offers is the application's business, which is the whole
            // point of the seam. Supplying a handler also takes the dock's
            // built-in Close off that tab — deliberately, because two menus
            // on one `Response` are two writers of one popup id.
            tab_tokens.extend(
                host.iter()
                    .flat_map(|h| h.attach(tab.response(), crate::shell::menus::DOCK_TAB)),
            );
        };
        let report = egui_shell::dock::Dock::new()
            .with_registry(panel_registry)
            .with_tab_menu(&mut tab_menu)
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

        for token in tokens.into_iter().chain(tab_tokens) {
            self.dispatch_token(ui.ctx(), token, actions);
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
