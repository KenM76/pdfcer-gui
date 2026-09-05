//! # `app::frame` — the per-frame update, in the one order it may happen in
//!
//! `eframe`'s entry point and nothing else. Split out of [`crate::app`] on
//! 2026-08-17, when that file crossed rule R2's 1,500-line ceiling.
//!
//! ## The seam is a real one, not a line count
//!
//! `app/mod.rs` answers *"what is the application, and how is it built?"* —
//! the state, its fields, its one constructor, and the two surfaces
//! (`ribbon_band`, `docks`) that are pure layout. This file answers *"what
//! happens, in what order, sixty times a second?"*, which is the question with
//! the ordering constraints in it.
//!
//! Those constraints are the reason the split is worth making rather than
//! merely necessary. Almost every comment in [`PdfcerApp::ui`] is about
//! **sequence** — the theme before any widget, the keyboard before any widget
//! can consume a key, the dialogs after the docks so they are painted over
//! rather than under, the zoom anchor after the commands that raise one, the
//! rasterize last so it measures a settled frame. A reader auditing that order
//! now has it in one file with nothing else in it, which is the condition
//! under which an ordering bug is visible at all.
//!
//! The old shell is the argument: two independent regressions of the same key
//! landed two days apart in a 25,005-line `main.rs`, and neither noticed the
//! other.

use eframe::egui;

use super::actions::Action;
use super::state::Status;
use super::{PdfcerApp, REGION_CENTRAL_PANEL, keyboard, modes, window};

/// **A command to invoke once, from `PDFCER_DIAG_INVOKE`.**
///
/// Consumed on the first frame that reads it and `None` for ever after, so a
/// scripted invocation happens exactly once rather than sixty times a second.
///
/// # ★★★ Why this seam exists, and it is R1 rather than convenience
///
/// R1 says a phase is not done until its behaviour is asserted by **driving the
/// running binary**. `tools/ui-verify` does that by moving the operator's real
/// mouse and keyboard — which means it cannot run while he is at the machine,
/// and this project's own memory records him saying *"I'm working on the pc"*
/// mid-session and everything after it having to be headless.
///
/// Two features have already needed a seam of exactly this shape and got one:
/// `PDFCER_DIAG_OPEN_PATH` (a native file picker is a hard wall for synthetic
/// input) and `PDFCER_DIAG_DROP_PATH` (a drop originates in Explorer and cannot
/// be synthesised at all). `app::dropped`'s note is the argument, and it
/// generalises:
///
/// > *"without this, drag-and-drop would be the one feature in this shell that
/// > R1 could not reach — implemented, unit-tested, and never once exercised in
/// > a running window, which is exactly the state R1 exists to forbid."*
///
/// This one generalises it one step further. **An offscreen window cannot be
/// driven by OS input at all** — `D:/dev/rag/egui/postmessage_to_offscreen_eframe_window_drops_pointer_button.md`
/// — so a headless run can launch the application and read its trace and can
/// press nothing. `PDFCER_DIAG_VIEWPORT` already gives a real, laid-out,
/// invisible window; this gives it something to do.
///
/// It landed with `dialogs::host` on 2026-08-20 because that change had no
/// other honest oracle: *"a dialog opened in its own OS window"* is a fact
/// about a second viewport that no unit test can observe and no screenshot of
/// the main window contains.
///
/// # ★ It reaches the same choke point an operator's chord does
///
/// Deliberately. `dispatch_command` is where mode gating, the decline
/// retirement and the command registry all live, and a seam that went round it
/// would prove that a *different* path works. What this substitutes is the
/// keystroke, not the dispatch.
///
/// # Why a list of ids and not a script
///
/// Because a grammar is a language and these are doorbells. The variable takes
/// a comma-separated list, rung one per frame in order — `mode.edit,
/// edit.form_text_field` — and that is the whole of it: no arguments, no
/// conditionals, no state. Each id is one the command registry already
/// publishes, dispatched through the same choke point a chord reaches. `diag`'s own header
/// records that the old shell's 800-line `PDFCER_DIAG_SCRIPT` harness was
/// deliberately not salvaged — *"salvaging a script grammar before there is a
/// harness to run it would be shipping a language with no speakers"* — and
/// `tools/ui-verify` is that harness now. What it lacks is a way in on a
/// machine whose desktop is occupied, and one command id is the whole of that.
fn scripted_invoke() -> Option<String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    /// ★ How many of the listed commands have been rung.
    ///
    /// Was an `AtomicBool` while the variable held one id. It became a counter
    /// on 2026-08-26 for the reason in the header's *"one command and not a
    /// script"* section, which is still the governing argument and is not
    /// weakened by this: **a list of doorbells is not a grammar.** There is no
    /// syntax to learn, no arguments, no conditionals and no state — the ids
    /// are the same ids the registry already publishes, and each is dispatched
    /// through the same `dispatch_command` a keystroke reaches.
    ///
    /// What forced it: **a capability can take two commands to reach.** Arming
    /// a form-field tool needs Edit mode first, because the arm declines
    /// without `edit_content`. With a single-shot variable, every feature gated
    /// behind a mode was unreachable headlessly — implemented, unit-tested and
    /// never once exercised in a running window, which is precisely the state
    /// R1 exists to forbid.
    static RUNG: AtomicUsize = AtomicUsize::new(0);
    if !crate::diag::enabled() {
        return None;
    }
    let list = std::env::var("PDFCER_DIAG_INVOKE")
        .ok()
        .filter(|s| !s.is_empty())?;
    // ★ ONE PER FRAME, not all at once. The commands are ordered because they
    // depend on each other — mode, then the tool the mode permits — and a mode
    // change is applied by draining the action queue, which happens at the end
    // of the frame that raised it. Ringing both in one frame would ask the
    // second command a question the first has not yet answered, and it would
    // decline for a reason that looks exactly like the feature being broken.
    let n = RUNG.load(Ordering::Relaxed);
    let id = list
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .nth(n)?;
    RUNG.store(n.saturating_add(1), Ordering::Relaxed);
    Some(id.to_owned())
}

impl eframe::App for PdfcerApp {
    /// ★★★ **Flush what the debounce is still holding, before the process
    /// goes** — `OPERATOR_REQUESTS.md` O80.
    ///
    /// The operator: *"it should remember my page display preferences from my
    /// last closing of the program."*
    ///
    /// # What was wrong, and it is the shape worth naming
    ///
    /// [`crate::app::persistence::LayoutStore::flush`] exists, is documented,
    /// is tested — and had **no production caller**. Its own doc comment says
    /// what it is for in as many words: *"For an exit path, which must not
    /// lose the last change to a debounce that had not yet expired."* There
    /// was no exit path. `impl eframe::App for PdfcerApp` implemented `ui` and
    /// nothing else, `run_native` installed no exit callback, and there is no
    /// `Drop`.
    ///
    /// So the layout is written 750 ms after it changes, with a 5 s ceiling —
    /// and **a change made in the last 750 ms before the window closes was
    /// silently thrown away.** The debounce was correct, the ceiling was
    /// correct, and the last write of every session was a coin toss.
    ///
    /// ★★ It reaches the operator through page display, which is why it lands
    /// under O80 rather than as a housekeeping note. The active ribbon **mode**
    /// rides in `layout.ron`, and the mode is what picks
    /// `PageDisplay::default_for_mode` for a document with no remembered
    /// entry. Switch to Edit, close the program within three quarters of a
    /// second, reopen: the mode is Read again, Read defaults to continuous,
    /// and from his chair the program forgot which way it was showing pages.
    ///
    /// # Why `on_exit` and not `save`
    ///
    /// `save` is only called when eframe's `persistence` feature is enabled,
    /// and this application deliberately does its own persistence — see
    /// [`crate::app::persistence`]'s header on why the location is
    /// `pdfcer-core`'s decision rather than the platform's. `on_exit` is
    /// unconditional and runs after `save`, so it is the hook that is actually
    /// there.
    ///
    /// The `glow` form of the signature, because that is the backend this
    /// workspace's `eframe` features select. The parameter is unused: this
    /// flushes a file, it does not touch the GPU.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // ★ `flush` is a no-op when nothing is pending, so this costs an
        // `Option` read on the common exit and writes only when the debounce
        // was genuinely still holding something.
        let wrote = self.layout.flush();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            //
            // Traced even when nothing was written, because "the exit hook ran
            // and had nothing to do" and "the exit hook never ran" are the two
            // states this defect was hiding between, and a line that only
            // appeared on the write would not tell them apart.
            format!("exit-flush layout-written={wrote}")
        });
    }

    /// eframe 0.35's entry point is `ui`, **not** `update`.
    ///
    /// The trait hands a root [`egui::Ui`] rather than a [`egui::Context`]
    /// (`eframe-0.35.0/src/epi.rs:176`), and panels are added *inside* that
    /// `Ui` — `CentralPanel::show(ui, …)`, not `show(ctx, …)`. Anyone
    /// arriving from an older eframe, or from a code sample, will write
    /// `update` and get a "not a member of trait" error whose message does
    /// not say what to write instead; hence this note.
    ///
    /// The context is cloned out at the top because the raster bookkeeping
    /// needs it after the panel closure has ended, and `Ui::ctx()` borrows.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // ★ The window every dialog is owned BY, published once a frame.
        // See `dialogs::host::set_owner` for why it travels this way, and the
        // host's G3 section for what ownership buys.
        crate::dialogs::host::set_owner(&ctx, self.window);
        crate::diag::trace_on_change("root-focus", || {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("focused={:?}", ctx.input(|i| i.viewport().focused))
        });

        // ★ Step 0 — install the theme. See `DEFECTS.md` D10.
        //
        // **This call did not exist until 2026-08-14**, and the whole theme
        // subsystem — three presets, a palette, a role per colour, a
        // rendered-pair contrast gate over five widget states, and a gate
        // self-test — was compiled into the binary and never handed to the
        // `Context`. Every colour an operator has ever seen in this shell was
        // `egui`'s stock light style. Found by a `ui-verify` check sampling a
        // pressed ribbon button and getting `egui`'s `selection.bg_fill`
        // instead of the preset's.
        //
        // Two things are installed, and the second is the one whose absence
        // was invisible: `apply` writes the palette into **both** of egui's
        // light and dark `Style`s, and it stashes the whole `Theme` in
        // `ctx.data` where `Theme::of` retrieves it. `egui-shell`'s ribbon,
        // dock and splitter all call `Theme::of` for roles that have nowhere
        // to live in an `egui::Style` — the content backdrop, the label
        // plate. Without the stash they silently got the DEFAULT theme, so
        // the framework's chrome and egui's widgets painted from two
        // different palettes. `apply`'s own doc comment names that failure
        // and calls it the thing the module exists to prevent.
        //
        // **Per frame, not once at startup**, which is what that doc
        // prescribes: a theme change then takes effect immediately, with no
        // restart and no cache to invalidate. It is a handful of field writes
        // against a struct egui already owns.
        //
        // ★ The preset comes from the operator's settings — 2026-08-17, and
        // this is the second half of `DEFECTS.md` D10.
        //
        // The first half was fixed on 2026-08-14 by calling `apply` at all.
        // What that note said next, and what stayed true until now:
        //
        // > There is also **no way to choose a preset**: the settings dialog is
        // > one of the unsalvaged Class-B surfaces, so even once `apply` is
        // > wired, the preset is whatever the code picks until that dialog
        // > lands.
        //
        // # ★ The DRAFT wins over the live settings, and only for the theme
        //
        // Every other setting in that window is draft-until-Save. A theme
        // cannot be judged from a radio label — you choose it by *seeing* it —
        // so while the window is open the draft's token is the one installed.
        // The draft still governs what is SAVED; it just no longer governs
        // what is SHOWN, and the window's own radius line says so.
        //
        // Cancel drops the draft, so the look reverts with it. That is why
        // this is a two-line lookup rather than a separate "preview theme"
        // field with its own lifecycle: there is nothing to undo and nothing
        // that can get out of step with what will be written.
        //
        // # `unwrap_or_default`, and what it is covering
        //
        // A token this build does not recognise — from a settings file written
        // by a NEWER pdfcer — falls back to the default preset and the token is
        // **kept, not overwritten**. The window says so, quoting the name. The
        // alternative, silently rewriting it to `quiet` on the next save, would
        // destroy a setting the operator made in a different version of the
        // program they also run from the same folder.
        let theme_token = self
            .settings_draft
            .as_ref()
            .map_or(self.settings.theme.as_str(), |draft| {
                draft.working.theme.as_str()
            });
        let preset = egui_shell::theme::Preset::from_key(theme_token).unwrap_or_default();
        let theme = egui_shell::theme::Theme::new(preset);
        theme.apply(&ctx);
        // ★ Step 0a-bis — publish the application's own colour roles.
        //
        // Beside `apply` rather than in `configure_context`, for `apply`'s own
        // reason: the operator can change the preset from the Settings window,
        // and a one-time install would mean a restart to see the effect.
        //
        // `FEATURES.md` carried the absence of this call as a ⬜ row for a whole
        // phase — `snap_indicator_tint` returned `None` on every frame and the
        // snap marker silently fell back to the selection stroke, which is the
        // shape of failure `Overlays::get`'s `Option` makes invisible: nothing
        // looks broken, the cue is simply not there.
        crate::canvas::overlays::install(&ctx, &theme);

        // ★ Step 0b — install the UI scale. The theme's twin, added 2026-08-17.
        //
        // # Why it is here and not in `configure_context`
        //
        // Same reason the theme is: the operator can change it, so a one-time
        // call at start-up would mean a restart to see the effect.
        //
        // # ★ Why the epsilon guard, given that egui already guards
        //
        // `Context::set_zoom_factor` (`context.rs:2269-2280` in 0.35) does test
        // before acting — but on **exact float equality**. A bit-identical
        // `f32` handed straight back is absorbed and costs nothing, so this
        // guard is not covering a naive upstream.
        //
        // What `!=` misses is every *derived* value, which is what a
        // continuous, operator-settable quantity actually produces: a
        // percentage that has been formatted and re-parsed through the
        // preferences file, a slider mid-drag, anything that has been through
        // `normalise_ui_scale`. Those land a hair off the stored value, egui's
        // equality test sees a change, and the cost is real — a trip requests a
        // repaint on **every viewport** and re-derives `screen_rect` on the
        // next pass (`context.rs:431-443`). An epsilon is the right *kind* of
        // guard for a quantity with no exact representation; exact equality is
        // the right kind for a token.
        //
        // Note also that `zoom_factor()` does not reflect a set until the pass
        // **ends** (`context.rs:2258`), so this read returns what the previous
        // frame settled on — which is exactly what "has it changed since last
        // frame?" wants, and is why a set-then-read-back within one frame would
        // prove nothing.
        //
        // # ★ The draft wins, exactly as it does for the theme
        //
        // These two are the only settings in the window that take effect
        // before Save, and the argument is identical in both cases: **you
        // cannot judge either from a label.** A theme is chosen by seeing it;
        // a scale is chosen by seeing whether you can read the ribbon at it.
        // The draft still governs what is SAVED — Cancel drops it and the size
        // reverts with it, with no separate preview state to get out of step.
        //
        // The settings window's own radius line says so, so this is disclosed
        // rather than merely true.
        //
        // # What it does NOT do, and why that is right
        //
        // It does not touch the page. `set_zoom_factor` moves
        // `ctx.pixels_per_point`, which `viewer::raster_scale` already reads —
        // so the canvas re-rasterises at the new device density and the
        // document stays exactly the same size **relative to the window**. A
        // bigger UI genuinely does mean a smaller visible page, because the
        // ribbon and the panels take more of the window; that is the honest
        // consequence of the setting and not something to compensate for.
        //
        // The rasters keyed on `pixels_per_point` — the page texture, the
        // strip, the Pages thumbnails — invalidate through their own existing
        // keys, so nothing here has to know about them.
        let ui_scale = self
            .settings_draft
            .as_ref()
            .map_or(self.prefs.ui_scale, |draft| draft.working_prefs.ui_scale);
        // A tenth of a step: finer than any change an operator can make with
        // the control, coarse enough that float noise never trips the setter.
        if (ctx.zoom_factor() - ui_scale).abs() > crate::app::prefs::UI_SCALE_STEP / 10.0 {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "ui-scale from={:.2} to={ui_scale:.2} ppp={:.3}",
                    ctx.zoom_factor(),
                    ctx.pixels_per_point(),
                )
            });
            ctx.set_zoom_factor(ui_scale);
        }

        // ★★ Step 0b² — **carry the persisted Smart-Selector answer into the
        // canvas's live copy** — `OPERATOR_REQUESTS.md` O70.
        //
        // One direction only, and written every frame rather than seeded once:
        // `Prefs` is where the answer survives a restart and `egui::Memory` is
        // where the click path can read it, so this is a mirror rather than a
        // second opinion. Writing it every frame means a change made anywhere
        // that touches `Prefs` — the file on disk, a future Settings row —
        // reaches the canvas on the next frame without that surface having to
        // know this mechanism exists.
        //
        // ★ `sync`, not `set_enabled`: the latter also LEAVES whatever
        // container the operator is inside, which is right for a deliberate
        // press and wrong for a mirror that runs sixty times a second.
        crate::canvas::smart::sync(&ctx, self.prefs.smart_select);

        // ★★ Step 0b² bis — **measure the page's content digest**, on the one
        // frame-level `&mut` this shell has.
        //
        // `app::cache::OpenDoc::page_objects_revision` keys the decomposition
        // on it, which is what stops a 469 ms rebuild after every annotation
        // edit. The accessor is `&mut self` (2026-09-01, `pdfcer-core`
        // `6e2b69e`), and the cache that reads it is `&self` behind an `Arc`
        // the render worker shares — so the measurement happens here and the
        // reader takes the number.
        //
        // ★ Silent when a render is in flight and the `Arc` is shared. That is
        // safe rather than lucky: the digest is stored with the epoch it was
        // measured at, and is ignored once that epoch is stale.
        if let Status::Open(doc) = &mut self.status {
            doc.refresh_content_generation();
        }

        // ★ Step 0b³ — **publish whether this mode edits page content**, for
        // the canvas helpers that have no `Capabilities` to hand (O71).
        //
        // Before anything draws, so no surface can read last frame's answer.
        // `capability::publish_edit_content`'s docs carry why this is a
        // published value rather than a fifth parameter threaded through two
        // call chains — and the obligation that comes with it: one writer, and
        // it is this line.
        let caps = self.capabilities();
        crate::app::modes::capability::publish_edit_content(&ctx, caps.edit_content);
        // ★★★ AND THE WHOLE SET, EVERY FRAME — 2026-09-04, and its absence was a
        // defect I introduced the day before.
        //
        // `canvas::tool::store_capabilities` was called from exactly one place:
        // `on_mode_capabilities_changed`, which runs **when the mode CHANGES**.
        // Its own comment argued that was the right home — *"this is the ONE
        // function that runs when the answer changes"* — and for a value read
        // only after a mode switch it was.
        //
        // ⇒ **On a fresh launch nothing has changed.** The application starts in
        // Read, no mode switch has happened, nothing was ever stored, and
        // `canvas::tool::capabilities` returns its `Capabilities::FULL`
        // fallback — a deliberately permissive default, chosen so a unit test
        // with a bare `Context` is not silently gated.
        //
        // That was harmless while the only reader was `panels::tool::idle`,
        // which uses it to pick a *sentence*. It stopped being harmless the
        // moment `panels::bookmarks` used it to decide whether to draw the
        // AUTHORING half of a panel: on first launch, in Read, the panel read
        // FULL and drew Add, Rename, Remove, Copy and Cut — the exact defect
        // that gating was written to remove, surviving in the one state an
        // operator always starts in.
        //
        // ★★ The lesson is not "publish more". It is that a value stored **on
        // change** has no value **before the first change**, and a permissive
        // fallback turns that gap into a silently-ungated surface. A gate whose
        // default is "allow" must be published unconditionally or not read at
        // all.
        //
        // Beside `publish_edit_content` and for its stated reason: before
        // anything draws, so no surface can read last frame's answer, and one
        // writer. `store_capabilities` is an `insert_temp` of a `Copy` struct —
        // the per-frame cost is a hash-map write.
        //
        // `on_mode_capabilities_changed` keeps its job: RETIRING what a new
        // mode forbids. That is genuinely a change-triggered act and must not
        // run every frame.
        crate::canvas::tool::store_capabilities(&ctx, caps);

        // ★ Step 0c — clear any bitmap cursor, BEFORE anything draws.
        //
        // `egui::PlatformOutput::take` keeps `cursor_image` across frames —
        // *"sticky between frames"*, in its own comment — and `egui-winit`'s
        // `apply_cursor` prefers the image over `cursor_icon` whenever one is
        // present. So a bitmap set once by the canvas outlives every later
        // `set_cursor_icon` from anywhere in the application: the crosshair
        // would follow the pointer onto the ribbon, into the panels, over the
        // scrollbars, and stay there after the document was closed.
        //
        // One place resets and one place asks. `canvas::interact` re-asserts it
        // on the frames it wants it, which is strictly later in this function,
        // so a frame where the canvas does not run — no document, or a full
        // overlay — cannot leave a stale cursor behind. That last case is the
        // one a "clear it when the tool retires" version would miss.
        //
        // Costs one `Option` write per frame. See `canvas::cursor` for why the
        // application supplies its own crosshair at all.
        ctx.set_cursor_image(None);

        // ★ Step 0d — **publish which document every surface is drawing**,
        // before any of them draws.
        //
        // One writer, at a known point in the frame, before anything reads —
        // which is the property that makes `crate::pagedrag::ActiveDocument`
        // safe to keep in the context rather than thread through three
        // signatures. `egui_shell::theme::Theme::of` is the precedent and the
        // same shape.
        //
        // Cleared when nothing is open, rather than left stale: a page drag
        // that outlived its document would otherwise name a slot that is no
        // longer there.
        match &self.status {
            Status::Empty => crate::pagedrag::clear_active(&ctx),
            _ => {
                let label = self
                    .active_path()
                    .map(|p| crate::text::doctabs::tab_label(p, false))
                    .unwrap_or_default();
                crate::pagedrag::publish_active(&ctx, self.active_slot, label);
            }
        }

        // ★ Step 0e — rotate the page drag's landing slots, before any
        // surface can write one.
        //
        // `crate::pagedrag::begin_frame` is the single owner of the clear, and
        // its docs carry the argument: two surfaces can resolve a landing and
        // neither is in a position to clear the other's, so the clear cannot
        // belong to a surface at all.
        crate::pagedrag::begin_frame(&ctx);

        // ★ Step 0f — **the window title**, from what is open.
        //
        // The only surface that reaches an operator who is not looking at the
        // application: Alt-Tab, the taskbar and the accessibility window list
        // all read it, and none of them can see the tab strip. See
        // `crate::text::doctabs::window_title` for the three forms and for why
        // the count is in it.
        //
        // Sent only when it changes — see `last_window_title`.
        let title = crate::text::doctabs::window_title(self.active_path(), self.document_count());
        if title != self.last_window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("window-title {title:?}")
            });
            self.last_window_title = title;
        }

        // Step 1 — keyboard, before any widget can consume a key.
        let page_count = match &self.status {
            Status::Open(doc) => Some(doc.pages.len()),
            _ => None,
        };
        let mut actions = keyboard::collect(&ctx, page_count);

        // ★★ Step 1 — **files dragged onto the window**, read before anything
        // is drawn.
        //
        // `egui` reports drops on the CONTEXT, not on a widget: `RawInput`
        // carries `dropped_files` for the whole window and nothing narrows it to
        // a rect. So this is the only correct place for it — reading it inside
        // the canvas would miss a drop on the ribbon or on a dock panel, and the
        // operator would learn that the program accepts drops *sometimes*, which
        // is worse than never.
        //
        // Nothing in this shell read that field at all until 2026-08-19. The
        // operator's report — *"can't drag and drop a jpg file onto a new
        // pdf"* — was entirely true, and it made a WORKING Insert-image button
        // look broken, because both were tried in the same minute and only one
        // of them told him anything.
        //
        // ★★ **Read here, ACTED ON at the end of the frame** — changed
        // 2026-08-31 for `OPERATOR_REQUESTS.md` O67.
        //
        // `crate::app::filedrag` records the drop and the point it landed on,
        // and any surface drawn later this frame may CLAIM it: the Pages panel
        // claims a document dropped onto its thumbnails and inserts the pages
        // at the gap under the pointer. Whatever nobody claims falls through to
        // `dropped::resolve` below, unchanged — which is why the reading and
        // the acting are now at opposite ends of the frame.
        crate::app::filedrag::poll(&ctx);

        // Step 1a — the chords the MANIFEST binds.
        //
        // ★ This is the second half of the one-owner-per-chord fix. The
        // keymap is data — `egui-shell` deliberately does not dispatch it,
        // because "the application owns the question of what has focus and
        // what a chord means" — so until now every binding in it was a
        // documented promise with nothing behind it, and `keyboard::collect`
        // quietly bound two of the same chords to something else.
        //
        // `keyboard::commands` returns command *ids*, which go through the
        // same dispatcher a ribbon click does. That is what makes a chord and
        // its button incapable of disagreeing.
        //
        // Owned rather than borrowed (`Vec<String>`) because dispatching
        // needs `&mut self` and the keymap lives in `self.shell`. It is empty
        // on all but the handful of frames where a chord was actually
        // pressed.
        //
        // ★ **Filtered by the active mode**, which is the keymap's share of
        // the mode gate. Operator decision, 2026-08-14.
        //
        // The ribbon hides a tab and the canvas asks `Capabilities`; between
        // them sat this, dispatching by id and consulting neither — so Read
        // hid the Edit tab and `Ctrl+E` still reached `edit.text`. The rule
        // lives in `modes::capability::offers_command`, beside the other
        // statement of what a mode permits, rather than here: this is the
        // choke point that *applies* it, and a second copy of the rule at the
        // point of application is how the two come to disagree.
        //
        // Filtered rather than refused inside `dispatch_command`, because a
        // command the mode does not offer is not a command that *failed* —
        // there is nothing to report and nothing to trace as declined. The
        // chord simply is not bound in this mode, which is what the operator
        // sees: no tab, no button, no effect.
        let chord_commands =
            keyboard::commands(&ctx, self.shell.as_ref().and_then(|s| s.keymap.as_ref()));
        let mode = self.ribbon.mode().map(str::to_owned);
        for id in chord_commands {
            if !modes::capability::offers_command(self.shell.as_ref(), mode.as_deref(), &id) {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "chord-not-offered id={id} mode={}",
                        mode.as_deref().unwrap_or("-")
                    )
                });
                continue;
            }
            self.dispatch_command(&ctx, &id, &mut actions);
        }

        // ★★★ Step 1a½ — THE SCRIPTED INVOCATION, once, for a headless run.
        //
        // See [`scripted_invoke`]. It is here rather than earlier because it
        // must reach the SAME choke point a chord reaches, one line above:
        // a seam that bypassed `dispatch_command` would be exercising a path
        // no operator has, which is the failure this whole channel exists to
        // avoid.
        if let Some(id) = scripted_invoke() {
            // ★★★ **An OPERAND, for the commands that need one** — added
            // 2026-09-04 with the panel-layout verbs.
            //
            // `view.panel_float`, `_dock` and `_close` act on *the panel the
            // operator right-clicked*, which arrives through
            // `PdfcerApp::dock_menu_panel` — see `dispatch::panels`' header.
            // A harness has no pointer and no menu, so without this the three
            // commands were **unreachable headlessly**: the seam would fire
            // them, they would find no parked panel, and they would correctly
            // do nothing — which is symptom-identical to the feature being
            // broken.
            //
            // ★★ `D:/dev/rag/egui/a_harness_seam_that_fires_one_command_cannot_reach_anything_behind_a_mode.md`
            // is the same finding one turn earlier: that one added the comma
            // list because a capability could take two commands to reach; this
            // adds the operand because a capability can take a command **and a
            // noun**. Both are the same rule — a seam that can express less
            // than the interface can leaves part of the interface unverifiable
            // — and the answer is to widen the seam rather than to write a
            // check that asserts something easier.
            //
            // Spelled `view.panel_float@view.panel_layers`: the id, an `@`,
            // and the PanelId. `@` because it appears in no command id and no
            // panel id, so the split cannot be ambiguous.
            let (id, operand) = match id.split_once('@') {
                Some((id, panel)) => (id.to_owned(), Some(panel.to_owned())),
                None => (id, None),
            };
            self.dock_menu_panel = operand.as_deref().map(egui_shell::dock::PanelId::new);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("diag-invoke id={id} operand={operand:?}")
            });
            self.dispatch_command(&ctx, &id, &mut actions);
            // Drained whether or not the command read it, so a seam-supplied
            // operand cannot survive into the next frame's dispatch — the same
            // one-call window `dispatch::panels::take_menu_panel` enforces for
            // the menu path.
            self.dock_menu_panel = None;
        }

        // Step 1b — the ribbon, above the canvas.
        //
        // Added before the `CentralPanel` because panel composition order is
        // load-bearing for both geometry and Tab focus: a panel added later
        // carves its space from what is left, so the canvas must be last or
        // it takes the whole window and the ribbon draws over it.
        //
        // The shell executes nothing. `show` returns the handler tokens the
        // operator invoked this frame, and they are translated into
        // `Action`s here — the same one-choke-point discipline every other
        // surface follows. A token with no arm yet is not an error; at S2
        // most of the ribbon is scaffolding for behaviour that lands later,
        // and `dispatch_token` says so per token rather than silently.
        // ★ Read mode's whole effect is here: the ribbon and the docks are not
        // added to the frame. Why it is not `mode.read`, why the status bar
        // below stays and how the operator gets back out are all in [`window`];
        // a composition step deciding any of that would be a second rule.
        let chrome = window::draws_chrome(&ctx);
        if chrome {
            self.ribbon_band(ui, &mut actions);
        }

        // ★ Step 1b¹ — the DOCUMENT TAB STRIP, under the ribbon and over
        // everything else.
        //
        // Composition order, not preference, and the same rule the ribbon and
        // the status bar are placed by: a full-width bar must be added before
        // any side panel or it starts at the dock's edge instead of spanning
        // the window.
        //
        // `exact_size`, never `default_height`, for the reason the status bar
        // carries at greater length: a chrome surface whose height follows its
        // content, sitting above a viewport that fits a page to itself, is a
        // measured feedback loop (R128). The strip's height is a constant in
        // `egui_shell` and this is where that constant is honoured.
        //
        // Behind `chrome` with the ribbon and the docks. Read mode's stated
        // purpose is the largest possible page, and an operator who has asked
        // for that has not asked to keep a tab strip — `Ctrl+Tab` still
        // switches documents, exactly as every other command still works from
        // the keymap while its button is hidden.
        //
        // Drawn even with ONE document open. Chrome, VS Code and Acrobat all
        // do, and the alternative costs the feature its discoverability: an
        // operator who has never seen a tab has no reason to believe a second
        // document is possible. `app::doctabs` §2 carries the argument.
        if chrome && self.document_count() > 0 {
            egui::Panel::top("document-tabs")
                .exact_size(egui_shell::tabstrip::STRIP_HEIGHT)
                .show(ui, |ui| {
                    self.document_tabs(ui, &mut actions);
                });
        }

        // Step 1b² — the status bar, before the docks.
        //
        // **Order, not preference.** This module's own header states the rule
        // the old shell was bitten by: *"a full-width bar must be added
        // before any side panel, or it starts at the side panel's edge
        // instead of spanning the window."* A status bar that stops at the
        // dock is not a status bar.
        //
        // `exact_size`, never `default_height`: a content-driven status
        // height and a per-frame fit-to-viewport zoom form a measured
        // feedback loop — 230 % → 224 % → 215 % drift from a status line
        // that grew (R128, `D:\dev\rag\egui\bottom_panel_height_...md`).
        egui::Panel::bottom("status")
            // ★ Theme-derived since 2026-08-26. The constant this replaced
            // assumed 24-point controls and the shipped theme's are 28, so the
            // bar's own zoom stepper and Find toggle were clipped by two points
            // at every UI scale. See `status::height_for`.
            .exact_size(crate::app::status::height_for(&theme))
            .show(ui, |ui| {
                // Three disjoint field borrows through `self`, as at the
                // canvas call site below: the bar reads the status and
                // writes the Find toggle's and the selection filter's own
                // state.
                //
                // ★ The filter is compared before and after rather than
                // reporting its own change. `PickFilter` is `Copy` and
                // eleven bytes wide, so a snapshot costs less than the
                // dirty flag it replaces — and, more to the point, it
                // cannot be forgotten by a future row added to the popup.
                // A control that mutated the filter without setting a flag
                // would persist nothing and look completely correct.
                let filter_before = self.pick_filter;
                // ★ Same comparison seam as the filter, for the same reason:
                // a snapshot of a `Copy` value cannot be forgotten by a future
                // control added to the popup, where a dirty flag can.
                let max_zoom_before = self.prefs.max_zoom_percent;
                // ★ And the wheel-paging choice, snapshotted for the same
                // reason and saved by the same branch — O30. Two preferences
                // reachable from one bar, and the file is written whole, so
                // one comparison and one save covers both. Adding a second
                // save call here would write the file twice on the frame a
                // future control changed both.
                let wheel_before = self.prefs.wheel_paging;
                crate::app::status::show(
                    ui,
                    &self.status,
                    &mut self.find,
                    &mut self.pick_filter,
                    &mut self.prefs.max_zoom_percent,
                    &mut self.prefs.wheel_paging,
                    &mut actions,
                );
                let wheel_now = self.prefs.wheel_paging;
                // ★★★ THE WHEEL CHOICE REACHES THE OPEN DOCUMENTS AT ONCE —
                // O30, and it is the one preference that must not wait for a
                // Settings apply.
                //
                // `OpenDoc::prefs` is a SNAPSHOT, adopted when the Settings
                // window is applied, and its contract is about values baked
                // into cached rasters — `render_quality` is the reason it
                // exists. `wheel_paging` bakes into nothing; it is a live
                // preference about an input gesture, and the operator who
                // just pressed the toggle expects the very NEXT notch to obey.
                // Left to the snapshot, the control would look correct, write
                // the file correctly, and change nothing on screen until the
                // Settings window was opened and applied — which is exactly
                // the shape of a silently-inert control this project has
                // shipped before.
                if self.prefs.wheel_paging != wheel_before {
                    let adopt = |status: &mut crate::app::state::Status| {
                        if let crate::app::state::Status::Open(doc) = status {
                            doc.prefs.wheel_paging = wheel_now;
                        }
                    };
                    adopt(&mut self.status);
                    for parked in &mut self.parked {
                        adopt(parked);
                    }
                }
                if (self.prefs.max_zoom_percent - max_zoom_before).abs() > f32::EPSILON
                    || self.prefs.wheel_paging != wheel_before
                {
                    // The preferences file is written whole, and the error is
                    // traced rather than shown: losing a maximum-zoom choice
                    // across a restart is an inconvenience, where a modal about
                    // a preferences file raised at the moment he picked from a
                    // menu would be worse than what it reports.
                    if let Err(err) = self.prefs.save() {
                        crate::diag::trace(|| {
                            format!(
                                // ui-text-exempt: diagnostic trace, never displayed.
                                "prefs-save-failed after=max-zoom err={err}"
                            )
                        });
                    }
                }
                if self.pick_filter != filter_before {
                    // Immediately, not debounced: a filter can only change
                    // on a discrete click, so one change is already one
                    // operator decision. `pickstore`'s header carries the
                    // contrast with the dock layout, which needs a delay
                    // because a splitter drag reports one per frame.
                    //
                    // The error is traced and not shown. Losing a filter
                    // across a restart is an inconvenience; a modal about a
                    // preferences file, raised at the moment the operator
                    // ticked a checkbox, would be worse than what it
                    // reports.
                    if let Err(err) = crate::app::pickstore::save(self.pick_filter) {
                        crate::diag::trace(|| {
                            format!(
                                // ui-text-exempt: diagnostic trace, never displayed.
                                "pick-filter-save-failed kind={}",
                                err.kind()
                            )
                        });
                    }
                }
            });

        // Step 1c — the docks, between the ribbon and the canvas.
        //
        // Order is load-bearing twice over. The ribbon is a full-width bar
        // and must be added *before* any side panel, or it would start at
        // the dock's edge instead of spanning the window. The canvas must
        // be added *after* both, because a `CentralPanel` takes whatever is
        // left and there must be something left for it to take.
        if chrome {
            self.docks(ui, &mut actions);
        }

        // Step 1c² — the debounced workspace write, dock drawn or not. ★ Moved
        // out of `Self::docks` when read mode landed; [`window`] §3 has why the
        // debounce belongs to the frame and what quitting from read mode would
        // otherwise lose.
        if let Some(after) = self.layout.tick(std::time::Instant::now()) {
            ctx.request_repaint_after(after);
        }

        // Step 2 — compose. Nothing here mutates a document; surfaces push
        // onto `actions`.
        egui::CentralPanel::default().show(ui, |ui| {
            // Declare the panel's own rect before drawing into it. This is
            // the outermost named region the application owns, and it is the
            // one a screenshot oracle uses to tell "the control is drawn but
            // clipped out of its pane" from "the control is not drawn" —
            // `PROJECT_PLAN.md` §4.2 prerequisite 2 records two cases where a
            // traced rect was correct and the control was still clipped.
            crate::diag::ui_rect(REGION_CENTRAL_PANEL, ui.max_rect());
            self.central(ui, &mut actions);
        });

        // Step 2a² — the FIND OVERLAY, over the page.
        //
        // ★ After the canvas, and the order IS the placement. The box is an
        // `egui::Area` positioned from the CANVAS VIEWPORT's rect, which
        // `canvas::show` records through `zoom::remember_frame` as the last
        // thing it does — so drawing it before the canvas would position this
        // frame's box from last frame's layout, visible as a one-frame lag
        // every time a dock splitter is dragged.
        //
        // Before the dialogs, because a modal takes the frame and must be over
        // everything, this included.
        //
        // It draws nothing when the bar is closed and nothing when no document
        // is open, so on the overwhelming majority of frames this line costs
        // one boolean. Two disjoint field borrows through `self`, as at the
        // canvas call site: `&mut self.find` and `&self.status`.
        crate::find::bar::show(ui, &mut self.find, &self.status, &mut actions);

        // Step 2a³ — drain any `Action::Command` raised by a surface that is
        // not the ribbon, and route it through the one dispatch choke point.
        //
        // ★ Here, and not in the apply phase, and the position is the design.
        //
        // The Find bar's OCR offer is the first control outside the ribbon that
        // means an existing *command* rather than a document change. Wiring it
        // straight to `DialogsState::open_ocr` would have been one line and
        // would have put `file.ocr`'s guards in two places — the failure this
        // module's "one choke point for dispatch" invariant exists to prevent.
        //
        // It has to run **now** rather than at step 3 for two reasons, both
        // hard rather than stylistic: `dispatch_command` needs an
        // `&egui::Context` and the apply phase is deliberately given none, and
        // a dialog opened by the dispatch must be drawn by `DialogsState::show`
        // three lines below — on this frame, not the next one.
        //
        // The drain is unconditional and cheap: on the overwhelming majority of
        // frames `actions` is empty and this is one `iter().any`. Dispatched
        // commands may themselves raise actions, which is why the loop pushes
        // into the same vector the apply phase will read.
        if actions.iter().any(|a| matches!(a, Action::Command(_))) {
            let mut invoked: Vec<String> = Vec::new();
            actions.retain(|a| match a {
                Action::Command(id) => {
                    invoked.push(id.clone());
                    false
                }
                _ => true,
            });
            for id in invoked {
                self.dispatch_command(&ctx, &id, &mut actions);
            }
        }

        // Step 2b — modal dialogs, LAST among the surfaces.
        //
        // After the canvas and the docks, because egui draws in call order
        // and a dialog shown before them would be painted under the very
        // content it is modal over. It takes `&self.status` so it can close
        // itself when the document does — a print dialog outliving its
        // document would offer to print pages that are gone.
        // ★ The keymap and the registry are threaded in for ONE window: the
        // keyboard reference derives every row from them rather than holding a
        // list. That is `DEFECTS.md` D5 made unrepresentable — see
        // `dialogs::shortcuts` — and it is why this call takes two arguments
        // that no other dialog reads.
        // ★★★ The password prompt is driven by the document's STATE, not by an
        // event, and that is what makes it robust.
        //
        // `Status::NeedsPassword` is a state a document sits in until it is
        // resolved, so asking here — every frame, idempotently on the path —
        // means the prompt appears whether the document arrived from the Open
        // dialog, from a command-line argument, from a recent-files entry or
        // from a tab the operator switched back to. An event-driven prompt
        // would have to be raised at each of those sites and would be missing
        // from whichever one was added last.
        //
        // `ask_for_password` returns immediately when it is already asking for
        // this path, so calling it unconditionally costs one comparison.
        if let Status::NeedsPassword { path } = &self.status {
            let path = path.clone();
            self.dialogs.ask_for_password(&path);
        }
        let keymap = self.shell.as_ref().and_then(|s| s.keymap.as_ref());
        self.dialogs.show(
            &ctx,
            &self.status,
            &mut actions,
            self.window,
            keymap,
            &self.commands,
        );

        // ★★★ **Step 4a — the FLOATING PANELS' own windows.**
        //
        // The dock's second per-frame call, and it is here rather than
        // inside `Self::docks` for the reason `egui_shell::dock::floatwin`'s
        // header states: opening a child viewport runs a complete nested
        // pass for another window, and doing that from inside a
        // half-composed side panel would make the rest of this frame's
        // layout depend on what a different window did. The dialogs are
        // hosted from this same point, for this same reason.
        //
        // ★★ After the dialogs, deliberately. A modal dialog takes the
        // frame; a panel window is a peer surface the operator can work in
        // while a dialog is up. Drawing the panels first would put them
        // above a modal in the composition order for no benefit and one
        // real cost — a dialog raised *from* a panel window would be behind
        // the window that raised it.
        //
        // ⚠ **Forgetting this call is a silent failure.** Every floating
        // panel would stay in the layout, report as on screen, and be
        // drawn nowhere — the exact class of defect this project shipped on
        // 2026-08-10 with three unreachable panels and every gate green.
        // `DockFrameReport::floats_undrawn` is the number that catches it,
        // and `crate::app::surfaces`' own test asserts it is zero.
        self.floating_panels(&ctx, &mut actions);

        // ★★ The unsaved-edits answer, drained IMMEDIATELY after the dialogs
        // draw and before anything else in this frame reads the document.
        //
        // Here rather than in `dispatch` for the reason the calibration round
        // trip below is: it is not a command. It is a frame-level observation
        // that a window the operator was looking at has been answered — and
        // the act it authorises (close, open, replace) belongs to the
        // application rather than to a dialog, which is why the window parks an
        // answer instead of calling `close_document` itself.
        //
        // Before the calibration edges, not after, and the ordering is real:
        // both of those manipulate a window that describes the open document,
        // and this line may have just replaced it.
        self.resume_after_unsaved();

        // ★★ The signature warning's answer, drained on the same line of
        // reasoning and immediately after — a frame-level observation that a
        // window has been answered, whose act (a write, possibly over the
        // operator's own file) belongs to the application rather than to a
        // dialog.
        //
        // AFTER the unsaved drain, and `resume_after_signature`'s own header
        // carries why: that drain may close or replace the open document, and
        // a save resumed before it could then write a document the operator
        // was never asked about. The two questions cannot currently be live at
        // once; the order is fixed in the direction that stays correct if that
        // ever changes.
        self.resume_after_signature();

        // ★★ O122's answer, drained on the same line of reasoning as its two
        // neighbours and immediately after them — a frame-level observation
        // that a window has been answered, whose acts (a write, a process
        // launch and a close) belong to the application rather than to a
        // dialog.
        //
        // ★ LAST of the three, and the order is real rather than incidental.
        // This drain can **close the document**, and both of the drains above
        // read it: running it first would let a signature warning resume over a
        // document that had already been handed to Acrobat and closed. The
        // three questions cannot currently be live at once; the order is fixed
        // in the direction that stays correct if that ever changes.
        self.resume_after_open_in_acrobat();

        // ★★★ **THE WINDOW'S ✕, AND THE QUIT CYCLE** —
        // `OPERATOR_REQUESTS.md` O102. Read here, after both drains, because
        // an answer given this frame may have cleaned or closed the very
        // document the cycle would otherwise ask about next — and asking twice
        // about a document the operator has just dealt with is the one thing
        // this cycle must not do.
        self.step_quit_cycle(&ctx);

        // ★ The calibration round trip: dialog -> canvas gesture -> dialog.
        //
        // Two edges, read once per frame, in this order.
        //
        // FIRST, "the operator pressed *Measure it on the drawing*": close the
        // window so it is not over the page they are about to click, and arm
        // the two-point pick. Read-and-clear, so a request cannot re-arm on
        // every subsequent frame.
        //
        // SECOND, "the pick just completed": put the window back with the
        // measured length in it, and disarm, so the next click is an ordinary
        // one rather than the start of another reference line.
        //
        // Here rather than in `dispatch` because neither edge is a command —
        // one is a button inside a dialog and the other is the canvas noticing
        // its own state machine finished. Both are frame-level observations,
        // which is what this function is for.
        // ★★★ **The placement round trip** — `OPERATOR_REQUESTS.md` O66, and
        // it is THREE edges where the scale round trip below has two. The third
        // is the difference that matters: a placement can be abandoned, and the
        // window has to come back when it is.
        //
        // ★ Edge 3 is an explicit no-op arm rather than an absent one, because
        // the sentence it carries is the whole design: **nothing happens on a
        // cancel, because the dialog un-hides itself.** Being hidden is derived
        // from the pending record, so clearing that record IS the un-hide.
        {
            let page = match &self.status {
                crate::app::state::Status::Open(doc) => doc.view.page_index,
                _ => 0,
            };
            if let Some((kind, page)) = self.dialogs.take_place_request(page) {
                crate::canvas::placing::arm(&ctx, kind, page);
            }
            if let Some((kind, rect)) = crate::canvas::placing::take_result(&ctx) {
                self.dialogs.deliver_placement(kind, rect);
            }
            if let Some(kind) = crate::canvas::placing::take_cancelled(&ctx) {
                // Deliberately nothing. See above.
                let _ = kind;
            }
            // ★★ …and the invariant that closes every route nobody enumerated:
            // a placement pending for a window that has gone. Closing the
            // document drops the dialog, and without this the canvas would wait
            // in a placement tool for a window that no longer exists. One
            // `Option` read per frame.
            if crate::canvas::placing::pending(&ctx)
                .is_some_and(|p| !self.dialogs.has_requester(p.kind))
            {
                crate::canvas::placing::cancel(&ctx);
            }
        }
        if self.dialogs.take_scale_calibrate_request() {
            self.dialogs.close_scale();
            crate::canvas::tool::select(
                &ctx,
                crate::canvas::tool::CanvasTool::Measure(
                    crate::canvas::measure::MeasureKind::Scale,
                ),
            );
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "scale-calibrate armed=true".to_owned()
            });
        }
        // ★ Both halves must be present, and the group is the one that can be
        // absent. `active_group` answers `None` when no measure state exists —
        // which cannot happen on the frame a pick completes, since completing
        // one requires the state. Handled rather than unwrapped anyway: an
        // `expect` here would turn an impossible ordering into a crash in the
        // one gesture whose whole output is a number the operator is trusting.
        if let Some(measured) = crate::canvas::measure::take_completed_scale_line(&ctx)
            && let Some(group) = crate::canvas::measure::active_group(&ctx)
        {
            self.dialogs
                .open_scale_calibrated(&self.status, group, measured);
            crate::canvas::tool::select(&ctx, crate::canvas::tool::CanvasTool::Select);
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "scale-calibrate measured_pt={measured:.3}"
                )
            });
        }

        // Step 2b(ii) — the Settings window.
        //
        // Drawn beside the other dialogs but held separately, because its draft
        // has to be readable at the TOP of the frame where the theme is
        // installed. See `crate::dialogs::settings`' header.
        self.settings_window(&ctx);

        // ★★ Step 2b½ — **the drop nobody claimed**, which is every drop that
        // is not a document landing on the thumbnails.
        //
        // Last among the surfaces on purpose: a claim is a statement that some
        // surface knows what this file means *there*, and it can only be made
        // once that surface has laid itself out. The fallback is
        // unconditional, so a surface that forgets to claim costs a feature
        // and never a file.
        if let Some(landing) = crate::app::filedrag::unclaimed(&ctx)
            && let Some(dropped_image) =
                crate::app::dropped::resolve(&landing.paths, page_count.is_some(), &mut actions)
        {
            // The image goes straight into the placement window — the same one
            // `edit.insert_image` opens, through the same import. See
            // `dispatch::images::insert_path` for why that split exists.
            crate::app::dispatch::images::insert_path(
                &mut self.dialogs,
                &self.status,
                &dropped_image,
            );
        }

        // Step 2c — give every pending zoom an anchor, in ONE place.
        //
        // `ZoomIn`, `ZoomOut` and `ZoomTo` are raised from five call sites —
        // `view.zoom_actual` in the dispatcher, the keyboard, and three
        // status-bar controls. Anchoring them where they are raised would
        // mean the same rule spelled six times, and a seventh surface added
        // later would silently zoom to the top-left corner: the exact defect
        // this closes, which is why it is one statement here rather than six
        // there.
        //
        // The rule it applies lives in `canvas::zoom::anchor_point` and
        // nowhere else: **a zoom holds one page point still, and that point
        // is where the operator is looking** — the pointer when it is over
        // the canvas, the viewport's centre when it is not.
        //
        // It skips an action whose anchor is already armed, so the framing
        // verbs (fit, zoom-to-selection, region zoom) and the Ctrl+wheel keep
        // the anchors they set deliberately.
        if let Status::Open(doc) = &mut self.status {
            crate::canvas::zoom::arm_for_actions(&ctx, doc, &actions);
        }

        // Step 3 — apply, after the frame is drawn.
        let pixels_per_point = ctx.pixels_per_point();
        self.apply_actions(actions, pixels_per_point);

        // Step 4 — decide whether the picture on screen still matches the
        // state that was just updated, and start a render if not.
        self.settle_and_rasterize(&ctx, pixels_per_point);

        // ★ LAST — close the frame's region census.
        //
        // Every `diag::ui_rect` call for this frame has happened by now, so
        // this is the first moment at which "which regions were NOT drawn this
        // frame?" has an answer. It emits `ui-rect-gone` for each, and
        // `crate::diag::end_ui_frame` carries the argument for why a trace
        // that only reports appearances is not merely incomplete but
        // actively misleading.
        //
        // After `settle_and_rasterize` rather than before it, because that
        // call can still declare regions — and a census closed one line early
        // would retire whatever it was about to draw, every frame, forever.
        crate::diag::end_ui_frame();
    }
}
