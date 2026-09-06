//! # `app::dispatch` — one command in, one effect out
//!
//! The routing table. Every operator gesture that names a command — a
//! ribbon click, a quick-access button, a context-menu item, a keyboard
//! chord, a custom item reporting its own token — arrives here, and this is
//! the **one** place that decides what the application does about it.
//!
//! ## Why one choke point rather than a closure per command
//!
//! `egui_shell` stores an opaque `HandlerToken` and hands it back; it never
//! interprets it. That is what keeps the shell reusable — a registry of
//! closures would force it to name pdfcer's state type. The consequence on
//! this side is a single `match`, and the consequence of *that* is the
//! property worth protecting: **a confirmation gate, an undo entry or a
//! trace has exactly one place to go.** Scatter dispatch across as many
//! sites as there are commands and each of those becomes something somebody
//! has to remember at every site.
//!
//! ## Why this is separate from `app::mod`
//!
//! Split out at Phase 3, when `app/mod.rs` reached 1,638 lines against the
//! 1,500-line gate (R2). The seam is a real one rather than arithmetic:
//! `mod.rs` composes a frame — panels in order, canvas, dialogs, then apply
//! — while this file answers *what does this verb do*. The two change for
//! different reasons and are read at different times.
//!
//! The gate's own rationale is the argument for splitting here rather than
//! anywhere that merely counts: the GUI this project replaces reached 25,005
//! lines in one `main.rs`, and two of the defects in `DEFECTS.md` are pairs
//! of lines thousands of lines apart that no reviewer could have been
//! expected to see together.
//!
//! ## ★ The arms route; they do not compute
//!
//! Almost every arm is one line: push an [`Action`], or call the one
//! function in the module that owns the rule. Zoom anchoring lives in
//! `crate::canvas::zoom`, the tool in `crate::canvas::tool`, the print
//! dialog in `crate::dialogs`. The moment an arm starts working out *how* to
//! do something, that rule exists in two places and only one of them will be
//! the one that gets fixed.
//!
//! The few exceptions are marked where they occur, and each is a routing
//! decision rather than a rule: `file.recent` chooses between a parked
//! operand and the newest reachable entry; the panel commands map an id to a
//! panel.

// The Pages tab's arms, split out under R2. See its header for the seam.
/// ★★ **Cut, copy and the two pastes** — four ids over three operand kinds.
/// Split out on 2026-08-29 when `edit.paste_duplicate` pushed this file past
/// R2's ceiling for the fourth time. Its header carries the three-rung fork
/// that decides whether a `Ctrl+C` is about text, a form field or an object.
pub(crate) mod clipboard;
pub(crate) mod format;
/// ★ `edit.insert_image`'s four steps — pick, read, import, refuse or open.
///
/// A module rather than a match arm because it is a **sequence**, not a verb,
/// and a ninety-line sequence inside a `match` is how this file crossed 1,500
/// lines. Its header carries the two decisions worth questioning: why the
/// import happens before the window, and why the refusal is passed through in
/// the engine's own words.
pub(crate) mod images;
/// ★ Every `measure.*` command — the three tools, Finish, and the two windows.
///
/// A module rather than four arms because three of them resolve the **active
/// authoring group** the same way, and that resolution was written twice here
/// before the move. Its header carries why the fallback is traced rather than
/// silent.
mod measure;
/// **View ▸ Navigate** — the four canvas tools and the Smart-select switch.
/// Extracted 2026-08-31 under R2; see its header for what makes them one
/// subject rather than five arms that happened to be adjacent.
pub(crate) mod navigate;
/// ★★ **Cut, copy and paste of whole PAGES** — O59 item 2. Its header carries
/// the decision that shapes it: these are named commands rather than `Ctrl+C`,
/// because the `pages.*` operand rule always resolves and a chord rung reading
/// it would take the clipboard from the canvas for ever.
pub(crate) mod pageclip;
pub(crate) mod pages;
/// ★ **The two text-copy verbs** — the page's words and the whole document's,
/// onto the clipboard.
///
/// A module rather than two arms because their bodies are longer than most
/// whole tabs and their subject is its own; the third application of the seam
/// [`images`] and [`pages`] were split along, on the day this file crossed
/// R2's ceiling for the third time. Its header carries why both read the
/// **same** extraction a canvas selection reads, and why neither raises an
/// `Action`.
pub(crate) mod textcopy;
/// ★ Every `view.zoom_*` command — the two framing verbs, actual size and the
/// three fit modes.
///
/// A module rather than six arms because the family carries the most
/// reasoning per arm in the whole match, and because O29's third fit mode was
/// the line that took this file past 1,500. Its header carries the seam and
/// what deliberately did not move with it.
pub(crate) mod zoom;

use super::PdfcerApp;
/// The commands that perform nothing and point somewhere else — three of them,
/// each raising `Action::Command` and doing no more. Split out under R2 when
/// this file crossed 1,500 lines for the fourth time; see its header for why
/// the seam is a subject rather than a size.
/// ★ The Tools ▸ Batch band — `OPERATOR_REQUESTS.md` O68. Its own module under
/// R2, on the same argument as the six splits above it.
pub(crate) mod batch;
pub(crate) mod fonts;
mod forms;
/// **The four verbs whose subject is a PANEL** — float it, dock it back,
/// close it, and bring every floating one home. Its header carries the
/// operand problem (a command id is a verb with no noun) and the
/// park-and-drain shape that answers it.
pub(crate) mod panels;
pub(crate) mod routes;
/// The File > Security band's commands — O119 plus signing; see its header.
pub(crate) mod security;
pub(crate) mod settings;
/// **The three commands whose subject is a text caret** — the two that arm it
/// and the reflow that acts on the paragraph it is in. The arming pair moved
/// there from this file on 2026-08-28 so the subject lives in one place.
pub(crate) mod text;

use super::actions::Action;
use super::state::Status;

impl PdfcerApp {
    /// Turn one invoked command into whatever the application does about it.
    ///
    /// Resolved token → id → [`Self::dispatch_command`], rather than matching
    /// on the raw token number. The numbers are assigned in per-tab blocks in
    /// `crate::shell::commands` and are meaningful only there; duplicating
    /// them here would create a second place to keep in step, and a silent
    /// mis-dispatch is the failure that would result.
    ///
    /// **The id is cloned rather than borrowed**, and it is not an
    /// oversight: the arms below need `&mut self` — a panel to activate, a
    /// dock to reset, a mode to select — and a `&str` borrowed out of
    /// `self.commands` would hold `self` shared for the whole match. One
    /// short allocation per *invoked command* (an operator click, not a
    /// frame) is the right price for arms that can act on the application.
    /// `pub(super)` rather than private: this method moved out of
    /// `app/mod.rs` and its callers stayed. It is deliberately NOT `pub` —
    /// nothing outside `app` may dispatch a command, because the choke
    /// point's whole value is that there is exactly one way in.
    pub(super) fn dispatch_token(
        &mut self,
        ctx: &egui::Context,
        token: egui_shell::commands::HandlerToken,
        actions: &mut Vec<Action>,
    ) {
        let Some(id) = self
            .commands
            .iter()
            .find(|c| c.handler == token)
            .map(|c| c.id.clone())
        else {
            return;
        };
        self.dispatch_command(ctx, &id, actions);
    }

    /// Do whatever this build does about the command named `id`.
    ///
    /// **The one dispatcher.** A ribbon click, a QAT click, a context-menu
    /// click and a keyboard chord all arrive here, which is what makes it
    /// impossible for a chord and a button that share a command to do
    /// different things — the defect `crate::app::keyboard`'s header is
    /// about, closed structurally rather than by agreement.
    ///
    /// **A command with no arm is not an error.** At S2 most of the ribbon is
    /// scaffolding for behaviour that lands at S3 and later, and the
    /// honest thing is to say so once per invocation in the trace rather
    /// than to pretend the click did something. Where a command is *known*
    /// not to be implementable yet, its arm says why in the trace rather
    /// than falling through to the generic line — a reader of a trace from a
    /// machine they cannot see should not have to guess which kind of
    /// nothing happened.
    /// `pub(super)` rather than private: this method moved out of
    /// `app/mod.rs` and its callers stayed. It is deliberately NOT `pub` —
    /// nothing outside `app` may dispatch a command, because the choke
    /// point's whole value is that there is exactly one way in.
    pub(super) fn dispatch_command(
        &mut self,
        ctx: &egui::Context,
        id: &str,
        actions: &mut Vec<Action>,
    ) {
        // ★ **The operator's next act retires the last worded decline.**
        //
        // A decline — "Nothing to zoom to" — is a sentence about *one*
        // gesture, and this is the one place in the application that knows an
        // operator has just invoked something. That makes it the only honest
        // lifetime available: the sentence stands until the next thing they
        // do, and then it is gone.
        //
        // It is deliberately **not** keyed on `edit_epoch` the way the two
        // rule-4 disclosure lines beside it are. A decline changes no
        // document, so the epoch never moves and an epoch-keyed sentence would
        // never retire; and a decline must be **repeatable**, which an epoch
        // key cannot express because nothing changed between the two presses.
        // `crate::app::status::decline`'s header carries the whole argument.
        //
        // Placement above the match is what makes the repeat work: pressing
        // the declining chord twice retires the first sentence here and the
        // arm below records a second one, so two presses are two events rather
        // than one press and one swallowed keystroke.
        //
        // This is still routing rather than computing. The arm hands over a
        // value; it does not decide what a decline is, how long one lives, or
        // what it says — all three live in the module that owns them.
        crate::app::status::decline::retire();

        match id {
            // ★ Open. The command that makes pdfcer a reader rather than a
            // viewer of one file.
            //
            // It was registered, drawn on the File tab, drawn on the QAT,
            // bound to Ctrl+O in the keymap — and had no arm, so the only way
            // to open a document was `argv`. That is defect D1's shape with
            // the most consequential verb in the application behind it.
            //
            // The dialog runs HERE, during dispatch, and only its *result*
            // becomes an action. See `crate::app::files` for why that line is
            // where it is, and for the `PDFCER_DIAG_OPEN_PATH` seam that lets
            // a scripted harness answer the dialog without a human — a native
            // dialog is a hard wall for synthetic input, and substituting the
            // answer is the only thing that gets past it.
            // ★ New. One line, and the whole of the arm — which is what a
            // routing table looks like when the rule lives somewhere else.
            //
            // Everything a reader is likely to want is in
            // `crate::app::PdfcerApp::new_document` and `crate::app::blank`:
            // where the bytes come from, why the engine has no way to make a
            // document and never will, why the page is A4, why the mode is
            // left alone, and why the dirty-document question is `save_pending`
            // rather than a second rule.
            //
            // Note what it does NOT do: open a dialog. Two of the three
            // reference applications create immediately from a default and
            // only SolidWorks asks — and what SolidWorks asks is *which kind
            // of document*, which pdfcer has no analogue for. See
            // `crate::app::blank` §3.
            "file.new" => actions.push(Action::New),
            // ★ Insert another PDF's pages after the current one. Gated here
            // rather than at the control, on the shell's own pattern — *"push
            // the chord blind, gate the effect in dispatch"* — because a chord
            // reaches any command from any state. The picker blocks and runs
            // between frames, as `file.open`'s does, and has its OWN
            // diagnostic seam so a harness can drive an insert without also
            // answering an open.
            // ★★★ `pages.merge_into` — the audit's first pick, and shell work
            // only since `merge_document` landed on 2026-08-18.
            //
            // ★ It opens **no dialog**, where its neighbour below opens one.
            // That is the difference between the verbs rather than an economy:
            // an insert asks *which pages* and *where*, and a merge takes the
            // whole document and appends it, so there is nothing left to ask.
            // A merge that opened a position dialog would be the insert command
            // wearing a different label.
            "pages.merge_into" => {
                if !self.capabilities().edit_content {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("command-declined id={id} reason=mode-cannot-edit-content")
                    });
                } else if let crate::app::files::Picked::Path(path) =
                    crate::app::files::pick_insert_source()
                {
                    actions.push(Action::Page(
                        crate::app::actions::pages::PageAction::MergeIntoDocument { path },
                    ));
                }
            }
            "pages.insert_from_file" => {
                if !self.capabilities().edit_content {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("command-declined id={id} reason=mode-cannot-edit-content")
                    });
                } else if let Status::Open(doc) = &self.status {
                    let current = doc.view.page_index;
                    if let crate::app::files::Picked::Path(path) =
                        crate::app::files::pick_insert_source()
                    {
                        // ★ Count the source's pages HERE, before the dialog
                        // opens, so it can say "4 pages" rather than asking the
                        // operator to commit to a file they have not seen
                        // inside. A file that will not open is reported now
                        // instead of after a dialog they filled in.
                        self.dialogs.open_insert_pages(path, current);
                    }
                }
            }
            // ★ Opens a WINDOW rather than pushing an action, which is the
            // difference between the two New commands and the whole reason
            // there are two. A command cannot ask a question; a dialog can.
            // The action it eventually raises is `Action::NewSized`, from
            // `crate::dialogs::new_document`, behind the same `save_pending`
            // guard `Action::New` takes.
            "file.new_from_template" => self.dialogs.open_new_document(),
            "file.open" => crate::app::files::raise(crate::app::files::pick_document(), actions),
            // ★★★ O122 — the control beside the mode selector. A literal arm
            // rather than a routed one: it has its own `Action`, because what
            // follows is a sequence (save, launch, close) that no existing verb
            // performs and that must not be reachable without the confirmation
            // in front of it.
            "file.open_in_acrobat" => actions.push(Action::OpenInAcrobat),
            // ★ Close. `doc.open` gates the control, so the no-document case
            // is unreachable from the ribbon — and the action handles it
            // anyway, because a customized keymap can reach any command from
            // any state.
            // ★★ **One command, two operands, decided by where it was
            // invoked.**
            //
            // From the ribbon, the quick-access toolbar or `Ctrl+W` this means
            // *the document on screen*, which is what Close means everywhere.
            // From a **document tab's context menu** it means the tab that was
            // right-clicked, which is what a tab menu means everywhere.
            //
            // The alternative was a second command — `window.close_document` —
            // and two of this project's own gates refused it in the same run:
            // `no_two_commands_share_a_label` (it would carry `file.close`'s
            // label and tooltip, because it does `file.close`'s job) and
            // `every_menu_command_is_also_reachable_from_the_ribbon` (its only
            // route would have been the right-click). Between them they are
            // right. A command whose *meaning* is unchanged and whose
            // **operand** comes from the surface it was invoked on is one
            // command, and `tab_menu_target` is how the surface says so.
            "file.close" => match self.tab_menu_target {
                Some(slot) => actions.push(Action::CloseDocument(slot)),
                None => actions.push(Action::Close),
            },
            // ★ Its operand comes from the same place, and falls back the same
            // way: from a tab's menu it keeps that tab, from the ribbon it
            // keeps the one on screen.
            "view.close_other_documents" => {
                let keep = self.tab_menu_target.unwrap_or(self.active_slot);
                actions.push(Action::CloseOtherDocuments(keep));
            }
            // ★ Applied here rather than raised as an `Action`, which is the
            // same call `crate::app::doctabs` makes for a tab click and for
            // the same reason: switching documents destroys nothing, asks
            // nobody, and is a control the operator is watching. The funnel
            // exists to give one choke point to the things that change a
            // document; this changes *which* document, which is the question
            // the funnel is downstream of.
            "view.next_document" => self.cycle_document(true),
            "view.previous_document" => self.cycle_document(false),
            // ★ **Save a copy.** Registered, on the quick-access toolbar, bound
            // to `Ctrl+S`, printing "(Ctrl+S)" in its own tooltip — and until
            // 2026-08-14 it had **no arm**, so it traced `command-unimplemented`
            // and nothing this shell could author could be written to disk at
            // all. D1's shape, with the one verb that makes an editor an editor.
            //
            // One line, because the rule lives in `crate::app::save`: what the
            // copy is called, which mode the bytes are written in and why it is
            // not up for renegotiation, which `SaveOptions` were chosen, what
            // happens to `edit_epoch` (nothing, in both directions), and what
            // the operator sees when it fails.
            //
            // ★ **It does NOT open the picker here**, and that is the one thing
            // worth knowing at this site — `file.open` two arms above does. The
            // difference is not inconsistency: `crate::app::files::pick_save_path`
            // carries a **frame-timing requirement** that dispatch cannot
            // guarantee, because `PdfcerApp::central` dispatches the canvas's
            // context-menu tokens from inside `egui::CentralPanel::show`, and a
            // native modal opened mid-layout blocks the frame it is being drawn
            // in. The apply phase is always outside every closure, so the picker
            // runs there. `Action::SaveCopy`'s own docs carry the full argument,
            // including why it needs no operand where `Action::Open` needs one.
            // ★ Save-in-place, with the one case that must NOT be silent: a
            // blank document created in this shell has a placeholder path and
            // no file behind it, so saving over it would write somewhere the
            // operator never chose. That routes to the picker instead - the
            // same behaviour every other program has for a never-saved
            // document, and the reason this is a branch rather than an arm.
            "file.save" => actions.push(
                if matches!(&self.status, Status::Open(d) if crate::app::save::has_a_file(d)) {
                    Action::Save
                } else {
                    Action::SaveCopy
                },
            ),
            "file.save_copy" => actions.push(Action::SaveCopy),
            // ★ Beside its sibling, and a different action: see
            // `Action::SaveAs` for why the two are separate acts rather than
            // one act with a flag.
            "file.save_as" => actions.push(Action::SaveAs),
            // ★ **Undo and redo.** Registered since the ribbon landed, on the
            // quick-access toolbar in **every** mode, bound to `Ctrl+Z`,
            // `Ctrl+Y` and `Ctrl+Shift+Z` — and until 2026-08-14 they had **no
            // arm**, so every press traced `command-unimplemented` and did
            // nothing. That is `file.save_copy`'s defect one arm above, at the
            // other end of the same day: v0.1.0 shipped with dimensions, seven
            // markup kinds, text marks and form fills all reachable, and no way
            // to take any of them back.
            //
            // One line each, because the rule lives in
            // `crate::app::actions::apply`'s `history_step`: which engine verb
            // runs, why it goes through `vector_edit` like every other document
            // change, what the epoch bump and the texture drop are for, and what
            // an empty stack does.
            //
            // **The empty-stack check is emphatically not here.** `undo.available`
            // greys both controls, so a click cannot reach an empty log — but a
            // chord can, from any mode, and the answer to that is a decision
            // about the document made where the session is held. An arm that
            // consulted the session would be the second place this question is
            // asked (`crate::app::conditions` is the first), and the two would
            // eventually disagree — which shows up as a control that is greyed
            // while the status bar says something else.
            // ★ No enable predicate is consulted here, like every other arm —
            // greying is the ribbon's hint and the arm is the answer. A Select
            // All on a document with no page objects selects nothing and says
            // so through the trace, which is the honest outcome rather than a
            // refusal.
            "edit.select_all" => actions.push(Action::SelectAllOnPage),
            "edit.undo" => actions.push(Action::Undo),
            "edit.redo" => actions.push(Action::Redo),
            // ★ Print, and the one command in this match that raises no
            // action.
            //
            // Everything else here funnels through `Action` so that a
            // mutation is applied once, after the frame, in one place that
            // an undo log can be built from. Printing mutates nothing — it
            // has nothing to contribute to that log, and an action variant
            // could only be serviced by reaching back into the dialog for
            // the state it holds anyway.
            //
            // The funnel's actual *reason* — do no irreversible work in the
            // middle of laying out a frame — is honoured inside the dialog:
            // the commit button sets a flag, and the spool runs after the
            // window's closure has returned. Paper is as irreversible as it
            // gets; it is the rule being kept, not the mechanism.
            "file.print" => self.dialogs.open_print(&self.status),
            // ★★★ Save a compacted copy — `OPERATOR_REQUESTS.md` O48.
            //
            // The only `open_*` on this list that can fail rather than decline:
            // `pdfcer-core` refuses a full rewrite of a hybrid-reference file by
            // name, and of one whose object numbering is too sparse for §7.5.4's
            // single-section table. Both are facts about the operator's file and
            // both are recorded rather than swallowed — `app::save`'s header
            // states the rule this obeys: *"the honest response is to refuse and
            // say so, not to fall back to a full rewrite"*, read here in the
            // other direction.
            "file.save_compacted" => {
                let epoch = match &self.status {
                    Status::Open(doc) => doc.edit_epoch,
                    _ => return,
                };
                if let Some(refusal) = self.dialogs.open_compact(&self.status) {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("compact-refused detail={refusal}")
                    });
                    crate::app::actions::record_note(epoch, refusal);
                }
            }
            // About. Takes no `&self.status`, unlike every other dialog on
            // this list, and that asymmetry is the point rather than an
            // omission: it describes the program, so it opens with nothing
            // loaded and stays open when the document closes. The guard that
            // would make it document-scoped is deliberately absent at BOTH
            // ends — here and in `DialogsState::show` — because a guard in
            // one place and not the other is how a dialog ends up opening and
            // vanishing on the same frame.
            "file.about" => self.dialogs.open_about(),
            id if text::handles(id) => text::dispatch(self, ctx, id, actions),
            id if settings::handles(id) => {
                settings::dispatch(id, &mut self.settings_draft, &self.settings, &self.prefs)
            }
            // ★ Recognise text. A dialog rather than an immediate action, and
            // rather than the `file.copy_document_text` shape one arm below.
            //
            // Three things had to be true of this arm and none of them can be
            // true of a `match` limb that just does the work:
            //
            // 1. **It must not block.** Copying the document's text blocks the
            //    UI thread on purpose — 331–449 ms, a stutter. Recognition
            //    rasterizes a page at 300 DPI and runs two neural networks over
            //    it, which is *seconds*, and a window frozen for seconds is
            //    indistinguishable from a hung program. The work is on a thread;
            //    see `crate::ocr::Job`.
            // 2. **It must disclose before it writes.** Every word OCR produces
            //    is a guess and this recogniser scores none of them, so the
            //    operator reads what it inferred while still holding the ability
            //    not to save it. That needs a surface with three states.
            // 3. **It must ask where the result goes.** The operator's rule is
            //    that Read may produce a new document and may not modify this
            //    one, enforced at the save — and this is the first write to disk
            //    this shell performs, so it is the first place that can bite.
            //
            // No mode check, deliberately. OCR is offered in Read exactly as in
            // Edit: `app::modes::capability` governs canvas *gestures*, and OCR
            // is not a gesture. The rule it has to honour is about what a save
            // may overwrite, and that is enforced by the destination being a
            // path the operator names — which holds in every mode without any
            // mode being consulted.
            // ★ The thumbnail rail's page selection travels with the open —
            // `OPERATOR_REQUESTS.md` O79. Read HERE rather than inside the
            // dialog because `PanelsState` and `DialogsState` are two fields of
            // `PdfcerApp` and the dialog must not learn to reach across; the
            // dispatcher is the one place that already holds both.
            //
            // Ascending, from the panel's own `BTreeSet`, so the operand order
            // is the sheet order and not the order he happened to click.
            "file.ocr" => {
                let picked: Vec<usize> = self.panels.selected_pages().iter().copied().collect();
                self.dialogs.open_ocr(&self.status, picked);
            }
            // ★ **Apply redactions.** A dialog, in `file.ocr`'s shape one arm
            // up, and for two of its three reasons plus one of its own.
            //
            // 2 and 3 hold unchanged and harder: it **must disclose before it
            // writes**, because the disclosure is the list of things pdfcer could
            // *not* remove and it is the whole reason a redacted file can be
            // trusted; and it **must ask where the result goes**, because the
            // file the operator opened is the only remaining copy of the content
            // being removed and overwriting it would be the most damaging single
            // act this shell could perform.
            //
            // 1 does **not** hold, and the difference is deliberate. The removal
            // runs synchronously, inside this dispatch, so the report and the
            // bytes are one consistent snapshot of the marks as they were when
            // the operator clicked. `crate::dialogs::redact` §2 carries the
            // argument; the short version is that OCR can tolerate a worker
            // because it refuses outright on `edit_epoch != 0`, and a redaction
            // cannot, because the marks being applied are the ones the operator
            // has just made.
            //
            // The reason of its own: **this is the only irreversible operation
            // in the program**, so the arm opens a surface with two
            // acknowledgements rather than doing anything. It raises no
            // `Action`, because applying changes no document — the session keeps
            // its marks, its undo log and its epoch, and the redacted document
            // is a new file. `crate::app::actions`' redaction block says the
            // same thing from the other side, and says why an
            // `Action::ApplyRedactions` would be wrong rather than merely
            // unnecessary.
            //
            // No mode check, and none is needed: the command sits on the Edit
            // tab, so `app::modes::capability` already keeps Read and Review
            // away from it through the tab list — the same mechanism that makes
            // `crate::panels::redact` unreachable from Read, and the reason
            // neither surface carries a gate of its own.
            // ★★ The third marking route — O60. One arm, because the geometry
            // is the selection's and `actions::redactsel` owns turning it into
            // quads. The appearance comes from the panel's own default, the
            // same one the search and whole-page routes use, so three routes
            // cannot produce three differently-coloured marks.
            // ★ The panel's OWN chosen appearance, not a fresh default. Three
            // marking routes, one look: an operator who set the fill to grey in
            // the panel and then marked a selection from the ribbon must not get
            // a black one. `state.redact_mut()` is where that choice lives and
            // where the other two read it.
            "edit.redact_selection" => {
                let appearance = self.panels.redact_mut().appearance.to_core();
                actions.push(crate::app::actions::Action::Redact(
                    crate::app::actions::RedactAction::Selection { appearance },
                ));
            }
            "edit.redact_apply" => self.dialogs.open_redact(&self.status),
            // ★ Recent. The operand comes from the `recent_files` custom item
            // (see `Self::ribbon_band`), which parked it before returning this
            // command's token.
            //
            // Reaching this arm with nothing parked is not an error and not
            // unreachable: an operator may bind a chord to `file.recent` or
            // put it on their quick-access toolbar, neither of which draws a
            // menu. The defined answer is **the newest document that can be
            // seen right now** — which is what "recent" means with no further
            // qualification, and which skips an entry on a drive that is not
            // connected rather than reporting a failure the operator did not
            // cause.
            crate::shell::commands::FILE_RECENT => {
                let path = self
                    .recent_choice
                    .take()
                    .or_else(|| self.recent.newest_present(std::time::Instant::now()));
                match path {
                    Some(path) => actions.push(Action::Open(path)),
                    None => crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "recent-declined reason=nothing-reachable".to_owned()
                    }),
                }
            }
            // The zoom verbs, and the three fit modes, split out under R2 --
            // see `dispatch::zoom`'s header for the seam. `view.tool_hand`
            // below is the third of the Phase 3 navigation verbs the two
            // zoom-framing arms used to sit beside; each is one call because
            // the rule each obeys lives in `canvas::zoom` or `canvas::tool`,
            // not here. This match is a routing table; the moment it starts
            // computing an anchor or deciding what a drag means, the same rule
            // exists in two places and one of them will be the one that gets
            // fixed.
            id if crate::app::dispatch::zoom::handles(id) => {
                crate::app::dispatch::zoom::dispatch(self, ctx, id, actions);
            }

            // ★★ **The two text-EDITING verbs**, and they are the defect this
            // project was started for (`DEFECTS.md` D4).
            //
            // Two literal arms rather than a seventh guard function, deliberately.
            // The guard shape earns its keep when the id-to-kind map has four or
            // more members and would otherwise be four arms to forget the fifth
            // in; with exactly two, a `text_edit_for_command` would be a third
            // place — beside `reach::EVALUATED_GUARDS` and `reach::guard_claiming`
            // — that has to learn about a mapping already stated on
            // `TextEditKind::command_id`, and `shell::commands::mapping`'s own
            // tests exist to keep such a map honest in both directions. Two arms,
            // one binding, no new machinery.
            //
            // ★ **The capability check is `edit_content`, and unlike
            // `view.tool_text` immediately above, its presence is the decision.**
            // That arm has none because selecting text authors nothing — the
            // operator's own *copying is not authoring* ruling. Typing into the
            // page's content stream is authoring by every reading of that
            // sentence, so this declines by name in a mode that cannot author,
            // and `canvas::tool::retire_forbidden` disarms the tool (and abandons
            // any draft) from the other end. Reachable only by a chord or a
            // customized manifest, exactly like the markup and measure arms
            // below: the shipped manifest shows the Edit tab in Edit alone.
            // ★ The two pointer tools, armed directly rather than toggled.
            //
            // `view.tool_hand` and `view.tool_text` TOGGLE — press twice to
            // return to Select — because each was a single control with no
            // sibling, so "press it again" was the only way back. These do not:
            // they are members of a **row**, and in a row the way back to Select
            // is to press Select. Toggling a member of a radio group is the
            // behaviour that makes an operator press a button and watch a
            // different one light up.
            // ★★ **The clipboard family**, routed rather than inlined: four ids
            // (`edit.cut`, `edit.copy`, `edit.paste`, `edit.paste_duplicate`),
            // three operand kinds, and two chords that mean different things to
            // the same field. Moved out of this file on 2026-08-29 under R2 when
            // the fourth id arrived; `dispatch::clipboard`'s header carries the
            // fork that decides which module answers.
            id if clipboard::handles(id) => clipboard::dispatch(self, ctx, id, actions),
            id if pageclip::handles(id) => pageclip::dispatch(self, ctx, id, actions),
            // ★★ **The Navigate row — five controls, their own module.**
            //
            // Moved out of this file on 2026-08-31 under R2, when
            // `view.smart_select` (`OPERATOR_REQUESTS.md` O70) made it 1,506
            // lines. The seam was already there: four of the five arm a tool
            // and the fifth changes what the first one selects, which is one
            // subject — *what a press on the page means* — and it is the row
            // the ribbon draws them in.
            id if navigate::handles(id) => navigate::dispatch(self, ctx, id),

            // ★ **The markup shape tools — one arm for all four.**
            //
            // The same shape as the page-display radio below, for the same
            // reason: the id *is* the operand, and
            // `crate::shell::commands::markup_for_command` is the single
            // binding between an id and a kind. Four literal arms would be
            // four places to forget the fifth.
            //
            // **It arms a tool; it authors nothing.** The canvas draws the
            // band, the release raises `Action::CommitMarkup`, and pressing
            // the armed button again puts the pen down — `arm_markup`
            // toggles on the same kind and re-arms on a different one, so a
            // second press of Rectangle leaves the select tool rather than
            // arming Rectangle twice.
            //
            // The returned tool is discarded for the reason the `tool_hand`
            // arm above states: the pressed state is published from
            // `conditions` by asking `tool::selected`, never from a copy
            // kept on the app. A shadow copy is how a ribbon comes to say
            // Rectangle while the canvas is selecting.
            // ★ …and it declines in a mode that does not author markup.
            //
            // Unreachable through the shipped manifest — Read is shown File and
            // View alone, and no chord binds a `markup.*` id — so this is the
            // belt to `retire_forbidden`'s braces, which covers only the
            // *transition* into such a mode and cannot cover an arming that
            // happens while already in one. A customized manifest that binds a
            // chord to Rectangle is all it takes to reach this.
            //
            // Declining rather than arming-and-refusing is what keeps the
            // cursor honest: an armed markup tool paints a crosshair over every
            // page, which promises a drawing gesture `press_kind` has already
            // decided not to give. The same argument `retire_forbidden` makes,
            // at the other end of the tool's life.
            //
            // This is still routing rather than computing: the arm asks one
            // published predicate and either calls the one function or does
            // not. It does not work out *what* a markup is, and the trace
            // spelling matches the `mode.*` arm below, which already declines.
            // ★ **Every `measure.*` command**, routed. The arms are in
            // `dispatch::measure`, beside `dispatch::pages` and
            // `dispatch::images`, and for a reason those two do not share:
            // three of the four resolve the **active authoring group** out of
            // `egui::Memory` with the same traced fallback, and that resolution
            // was written twice here before the move. One module, one function,
            // one place for the fallback's trace to say which command asked.
            // ★ **A second ROUTE to `file.properties`, and deliberately not a
            // second implementation of it.**
            //
            // `Action::Command` exists for exactly this: `crate::app::actions`'
            // own docs say it is there *"so a second route to an existing
            // command cannot become a second implementation of it"*, with the
            // Find bar's OCR offer as the precedent. Calling
            // `panels::show(Panel::Properties)` here instead would put the
            // panel's opening guards — mode gating, dock state, the
            // already-open case — in two places, and the two would drift.
            //
            // The two ids exist because the shell enforces one command, one
            // tab, and the placements answer different questions: File ▸
            // Document is "tell me about this file", Format is "tell me about
            // the thing I just clicked". `format.properties` is the second
            // question's button and the first question's command is what it
            // presses.
            // ★ **Insert an image.** Its body is in `dispatch::images`, beside
            // `dispatch::pages`, and for that module's reason: it is a
            // *sequence* — pick, read, import, refuse or open — rather than a
            // verb, and a ninety-line sequence inside a match arm is how this
            // file crossed 1,500 lines.
            "edit.insert_image" => {
                if !self.capabilities().edit_content {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("command-declined id={id} reason=mode-cannot-edit-content")
                    });
                    return;
                }
                images::insert(&mut self.dialogs, &self.status);
            }
            // ★ **Export to DXF.** The FIRST entry in `reach`'s scaffold list
            // and one of three whose recorded reason was *"No recorded reason
            // anywhere. Scaffolded by omission, not by decision."* —
            // `pdfcer-core`'s `export::dxf` had shipped the whole time, and the
            // old shell has the feature, so `FEATURES.md`'s `gui` column made
            // it a REGRESSION rather than a gap.
            //
            // Gated on `doc.pages` through the registry rather than on a
            // capability: an export reads the document and writes elsewhere, so
            // there is no mode in which it should be refused. Read mode
            // exporting a drawing is exactly what a reading stance is for.
            // File > Security: Encrypt…, Permissions…, Sign…. See that module.
            id if security::claims(id) => self.dispatch_security(id),
            "file.export_dxf" => self.dialogs.open_export_dxf(&self.status),
            // ★★★ **Export image — `OPERATOR_REQUESTS.md` O120, wired
            // 2026-09-04.** The operator asked the ENGINE side for it on
            // 2026-09-03; the engine shipped all of it the same day and sent a
            // note marked *"informational, no reply needed"*, which nothing
            // here was required to read. There was no row on this side until a
            // session happened to read the request channel looking for
            // something else.
            //
            // ★ Gated through the registry on `doc.pages` rather than on a
            // capability, exactly as its DXF neighbour is and for that arm's
            // reason: an export reads the document and writes elsewhere, so
            // there is no mode in which it should be refused. Read mode
            // exporting a drawing is what a reading stance is FOR.
            "file.export_image" => self.dialogs.open_export_image(&self.status),
            // ★★★ **Export text — wired 2026-09-04**, on the operator's ask:
            // *"also the engine can export PDFs as text. we should have
            // export/import for that."* A dialog rather than a bare picker,
            // unlike `file.export_form_data` below, because four decisions have
            // to be made before the bytes exist and none is recoverable from a
            // save picker.
            //
            // ★★ **There is no `file.import_text` beside it**, and that is a
            // recorded finding rather than an omission — see
            // `crate::app::actions::exporttext`'s header for the three things
            // "import text" could mean and why the engine offers none of them.
            // R9: an absence is honest; a control that declines when pressed is
            // a promise the program cannot keep.
            "file.export_text" => self.dialogs.open_export_text(&self.status),
            // ★★★ **`file.export_form_data` — registered, drawn on File ▸
            // Export, and inert for the whole life of the project.**
            //
            // Its `SCAFFOLDED` reason said *"blocked on a writer that does not
            // exist"* and cited a `FEATURES.md` row that was itself stale.
            // Three writers exist and two have since `Pass 7.1`:
            // `fdf::FormData::{to_fdf, to_xfdf}` and `formcsv::to_csv`. The
            // **sixth** stale blocker this project has found, and the second in
            // one evening — both citations of citations, and nothing had
            // re-read either.
            //
            // ★ No dialog, unlike its DXF neighbour one line above: the format
            // is the extension the operator types in the save picker, which is
            // how every application on this desktop does it and is one modal
            // window rather than two. See `actions::export::form_data`.
            // ★★★ The SECOND-ROUTE arms live in `dispatch::routes` — three of
            // them as of 2026-08-28, and they moved out under R2 when this file
            // crossed 1,500 lines for the fourth time.
            //
            // The seam is a real subject rather than a size-driven cut, and it
            // is the property every member shares and no other arm here does:
            // **they raise `Action::Command`, so they perform nothing.** A
            // second route to an existing command must not become a second
            // implementation of it, which is the whole of what they are for and
            // is the one thing a reader has to check about any of them.
            id if fonts::handles(id) => {
                fonts::dispatch(id, &mut self.dialogs, &self.status, &self.prefs);
            }
            // ★★★ **`tools.merge_files`, wired 2026-08-31 — O68.** It was
            // registered, drawn, enabled at startup, and had no arm at all:
            // the operator pressed it and the program traced
            // `command-unimplemented` and did nothing he could see. The engine
            // verb (`pageops::merge`) had been complete and uncalled the whole
            // time; the recorded blocker named a missing PANEL, which is the
            // weak-blocker shape `reach::register` retired an entry for three
            // days earlier, in the same const, ten lines below this one.
            id if batch::handles(id) => batch::dispatch(self, id, actions),
            id if routes::handles(id) => routes::dispatch(id, actions),
            "file.export_form_data" => actions.push(Action::Write(
                crate::app::actions::write::WriteAction::FormData,
            )),
            // ★ The picker runs HERE, before the action, where the export's runs
            // inside the apply phase. Both are right for their case: an export
            // computes the bytes before it can honestly ask where they go, and
            // an import has nothing to compute until it knows which file.
            //
            // `dispatch_command` is not a layout pass — it runs between frames,
            // from the drained token queue — so a modal here blocks nothing
            // egui is part-way through.
            "file.import_form_data" => {
                if let crate::app::files::Picked::Path(path) =
                    crate::app::files::pick_form_data_source()
                {
                    actions.push(Action::Field(
                        crate::app::actions::forms::FieldAction::Import { path },
                    ));
                }
            }
            // ★ **The keyboard reference.** Its scaffold entry did not merely
            // say blocked — it carried the design, from `SALVAGE.md`: *"Fix
            // `shortcuts_reference()` — it omits six live bindings
            // (DEFECTS.md D5) — and derive it from the keyboard map so it
            // cannot drift again."* Nothing was salvaged; the window holds no
            // list.
            //
            // No document guard, unlike every other dialog on this tab: key
            // bindings exist whether or not a file is open, and refusing this
            // on an empty canvas would hide it from exactly the operator most
            // likely to be looking for it.
            "file.shortcuts" => self.dialogs.open_shortcuts(),
            // The Format tab's arms — Delete, Properties and Select-the-form.
            // Split out under R2 on 2026-08-27; see `dispatch::format`'s header
            // for the seam.
            id if format::handles(id) => format::dispatch(self, id, actions),
            id if measure::handles(id) => {
                measure::dispatch(self, ctx, id, actions);
            }
            // ★ **The three text-markup commands — one arm for all three.**
            //
            // The same one-arm shape the two families below have, and for the
            // same reason: the id IS the operand, and
            // `crate::shell::commands::text_mark_for_command` is the single
            // binding between an id and a `TextMarkKind`.
            //
            // ★ **It authors immediately; it arms nothing.** That is the whole
            // difference from the `markup_for_command` arm below it, and it is
            // the interaction decision recorded at
            // `canvas::markup::text`'s header §1: these kinds mark **an existing
            // text selection**, which is Acrobat's answer and needs no tool, no
            // gesture and no `CanvasTool` variant. The operand is on the
            // document, visible as a wash, at the moment the button is pressed.
            //
            // It must sit **ahead** of the `markup_for_command` arm. Both are
            // guard arms on `markup.*` ids and `match` takes the first that
            // matches; the two mappings are asserted disjoint in both directions
            // (`shell::commands::mapping`), so the order is belt to that
            // braces — but the order is also the cheaper of the two guarantees
            // and costs nothing to state.
            //
            // Two refusals, traced separately, because they have different
            // answers and a reader of a trace from a machine they cannot see
            // should not have to guess which nothing happened:
            //
            // * **the mode cannot author markup** — unreachable through the
            //   shipped manifest (Read is shown File and View alone), and
            //   reachable from a chord in a customized one, exactly as the
            //   markup and measure arms below;
            // * **there was nothing markable** — no selection, or one made
            //   against a revision that has since moved. The ribbon control is
            //   greyed in both cases, by the same `selection.text` condition the
            //   rule here asks about, so this is reachable only by a chord.
            //
            // The arm still routes rather than computes: `markup::text::mark` is
            // a pure function that owns every rule about which selection is
            // eligible and what a stale one means, and this reads one published
            // capability, calls it once, and pushes what comes back.
            id if crate::shell::commands::text_mark_for_command(id).is_some() => {
                let Some(kind) = crate::shell::commands::text_mark_for_command(id) else {
                    return;
                };
                if !self.capabilities().author_markup {
                    crate::diag::trace(|| {
                        format!(
                            // ui-text-exempt: diagnostic trace, never displayed.
                            "command-declined id={id} reason=mode-cannot-author-markup"
                        )
                    });
                    return;
                }
                // Every state but `Open` is *no document*, and therefore no
                // selection to mark and no page for the action to name — the
                // same hazard `measure.finishable` is published inside the
                // `Status::Open` arm to avoid. Written as an `if let` with one
                // fallback rather than an exhaustive `match`, so that a sixth
                // failure state (`Unsupported`, `NeedsPassword`, …) does not
                // arrive here asking to be classified: none of them holds a
                // document, and that is the only property this arm reads.
                let selected = if let Status::Open(doc) = &self.status {
                    crate::canvas::markup::text::mark(
                        kind,
                        doc.text_selection.as_ref(),
                        doc.edit_epoch,
                        // The live pen, sampled now. `canvas::interact` samples
                        // it at the same moment for the drag kinds — the start
                        // of the gesture — and for a ribbon command the whole
                        // gesture is this press.
                        self.pen,
                    )
                } else {
                    Err(crate::canvas::markup::text::Refusal::NoSelection)
                };
                match selected {
                    Ok(action) => {
                        if let Action::CommitTextMarkup { page, quads, .. } = &action {
                            crate::canvas::markup::text::trace_commit(kind, *page, quads.len());
                        }
                        actions.push(action);
                    }
                    Err(reason) => crate::canvas::markup::text::decline(kind, reason),
                }
            }
            // ★ **Finish** — the ribbon half of the vertex tools' ending, and the
            // one `markup.*` command that is neither a tool nor a mark.
            //
            // It is `measure.finish`'s twin, deliberately down to the shape of
            // this arm, because it answers the identical problem: PolyLine and
            // Polygon are runs of clicks with no natural end, exactly as the
            // radius/diameter pick set has none, and the operator settled that
            // on 2026-08-14 with **two endings through one commit path**. A
            // double-click on the canvas is the other half and is the one most
            // operators will use; this is the discoverable one, and the one that
            // works when the last vertex sits somewhere awkward to double-click.
            //
            // It must sit ahead of the `markup_for_command` arm below rather
            // than inside it, for the reason `measure.finish` states in its own
            // words: that mapping takes ids to *kinds*, this id names no kind,
            // and if it ever did, pressing Finish would toggle the tool off
            // (`arm_markup`'s same-kind-retires rule) instead of committing.
            //
            // The arm routes and does not compute. Everything about what a
            // finish *is* — whether the run is long enough for its kind, which
            // page it belongs to, emptying it afterwards — lives in
            // `canvas::markup::vertex::finish`, which is the same commit path
            // the canvas's double-click reaches. One commit path, two entrances;
            // a second derivation here is exactly how the two endings would come
            // to author different annotations.
            //
            // Both refusals are traced separately, because "the mode says no"
            // and "there was nothing to finish" are different facts with
            // different answers, and a reader of a trace from a machine they
            // cannot see should not have to guess which nothing happened.
            "markup.finish" => {
                if !self.capabilities().author_markup {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        format!("command-declined id={id} reason=mode-cannot-author-markup")
                    });
                } else if !crate::canvas::markup::vertex::finish(ctx, actions, self.pen) {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed.
                        //
                        // Reachable only by a chord or a customized manifest:
                        // the ribbon control is greyed unless there is a run
                        // long enough for its kind, by the same predicate
                        // `finish` itself asks.
                        format!("command-declined id={id} reason=no-vertex-run-to-finish")
                    });
                }
            }
            // ★ The three text-bearing kinds, ABOVE the geometric markup arm.
            //
            // Ordering is a statement rather than a tie-break — the two
            // families claim disjoint ids, and `mapping`'s own tests assert
            // that no `markup.*` id answers to both. It is written first
            // because a reader scanning for "what arms a markup tool?" should
            // meet both families together, and because if the two ever DID
            // overlap the failure would be silent: this arm would win and the
            // geometric one would simply stop being reached.
            //
            // The capability is `author_markup`, the same one the geometric
            // kinds gate on. A callout is markup — it sits over the page, it
            // appears in the Comments panel beside a rectangle, and a mode that
            // may not author one has no business authoring the other.
            id if crate::canvas::textannot::TextAnnotKind::from_command(id).is_some() => {
                if !self.capabilities().author_markup {
                    crate::diag::trace(|| {
                        format!(
                            // ui-text-exempt: diagnostic trace, never displayed.
                            "command-declined id={id} reason=mode-cannot-author-markup"
                        )
                    });
                } else if let Some(kind) = crate::canvas::textannot::TextAnnotKind::from_command(id)
                {
                    let _ = crate::canvas::tool::arm_text_annot(ctx, kind);
                }
            }
            // ★★★ THE FIVE FORM-FIELD COMMANDS, in `dispatch::forms`.
            //
            // The route is one line; what is in that file is the *reasoning* —
            // in particular why a greyed command declines in words here rather
            // than being swallowed by a blanket guard at the top of this
            // function. That guard was written, and two tests refused it.
            id if crate::shell::commands::form_for_command(id).is_some() => {
                // The `let else` rather than an `expect`: the guard one line
                // above already matched, so this cannot fail, and a panic
                // message would be an unreachable string in the catalog's way.
                let Some(kind) = crate::shell::commands::form_for_command(id) else {
                    return;
                };
                forms::arm(self, ctx, id, kind);
            }
            // ★★★ **`edit.form_flatten` — a drawn control that did nothing, for
            // the whole life of the project.**
            //
            // The capability shipped with the Forms panel: `flatten_fields` is
            // called from `panels::forms::edit`, and the panel draws a Flatten
            // button that raises the identical `FormEdit::Flatten`. What was
            // missing was only this line, and what kept it missing was a
            // SCAFFOLDED entry whose stated reason — *"the third of the unbuilt
            // forms-authoring verbs … irreversible on the document"* — had
            // become false in both halves without anything being able to fail.
            //
            // The manifest's own comment beside the item said what that costs:
            // *"a command buried in a panel is reachable only by someone who
            // already opened the panel."*
            "edit.form_flatten" => forms::flatten(self, id, actions),
            id if crate::shell::commands::markup_for_command(id).is_some() => {
                if !self.capabilities().author_markup {
                    crate::diag::trace(|| {
                        format!(
                            // ui-text-exempt: diagnostic trace, never displayed.
                            "command-declined id={id} reason=mode-cannot-author-markup"
                        )
                    });
                } else if let Some(kind) = crate::shell::commands::markup_for_command(id) {
                    let _ = crate::canvas::tool::arm_markup(ctx, kind);
                }
            }
            // ★ **The four positions of View ▸ Page display.**
            //
            // One arm for the whole radio, because the id *is* the operand:
            // `crate::shell::commands::page_display_for_command` is the single
            // binding between a command id and a
            // `crate::viewer::PageDisplay`, and its inverse is what publishes
            // the `selected:` condition that renders the active position
            // pressed. Four arms would be four places for that mapping to be
            // spelled, and the fifth mode would be added to three of them.
            //
            // An id the mapping does not know cannot reach here — the match is
            // gated on the same function — so there is no "unknown mode" arm
            // to write.
            id if crate::shell::commands::page_display_for_command(id).is_some() => {
                if let Some(display) = crate::shell::commands::page_display_for_command(id) {
                    // ★★★ **The press is also a standing preference** —
                    // `OPERATOR_REQUESTS.md` O80: *"it should remember my page
                    // display preferences from my last closing of the program.
                    // Example if I press show one page at a time."*
                    //
                    // Pressing this records the arrangement in three places
                    // with three lifetimes, and all three are wanted:
                    //
                    // | where | lifetime | set by |
                    // |---|---|---|
                    // | `doc.view.display` | this session | `Action::SetPageDisplay` |
                    // | `viewer::remembered` | this DOCUMENT, for ever | `Action::SetPageDisplay` |
                    // | `Prefs::default_page_display` | every document he has not arranged | **here** |
                    //
                    // ★★ It is written HERE rather than in the apply arm for a
                    // borrow reason and it is worth naming: `apply` takes
                    // `&mut self.status` for the whole of the document arms, so
                    // `self.prefs` is unreachable from inside one. `Action::Find`
                    // splits the same borrow the same way, one arm above, and
                    // its comment carries the general form.
                    //
                    // ★ Only when it CHANGES, and the guard is not an
                    // optimisation: `Prefs::save` is a whole-file write, and the
                    // ribbon raises this on every click including a click on the
                    // position that is already active.
                    if self.prefs.default_page_display != Some(display) {
                        self.prefs.default_page_display = Some(display);
                        let saved = self.prefs.save();
                        crate::diag::trace(|| {
                            // ui-text-exempt: diagnostic trace, never displayed.
                            format!(
                                "page-display-default mode={} saved={}",
                                display.id(),
                                // ★ The outcome, not the error text. A failed
                                // write is a real event — a read-only settings
                                // folder — and naming it here is what tells a
                                // reader of a trace that the preference was
                                // taken and not kept.
                                if saved.is_ok() { "yes" } else { "no" }
                            )
                        });
                    }
                    actions.push(Action::SetPageDisplay(display));
                }
            }
            // ★ **The three View ▸ Display chrome toggles**, one arm, for the
            // identical reason the page-display radio has one: the id IS the
            // operand, `chrome_for_command` is the single binding between an
            // id and a `ViewChrome`, and its inverse is what publishes the
            // `selected:` condition that renders each one pressed. Three arms
            // would be three places to spell one mapping.
            //
            // Unlike the radio these are independent — a click means "flip
            // this one", not "select this position" — so the action carries
            // which toggle and the apply reads the current value. Reading it
            // *there* rather than here is what keeps the dispatcher free of
            // `self.status`: a chord can reach this command with no document
            // open, and the arm must not have to decide what that means.
            id if crate::shell::commands::chrome_for_command(id).is_some() => {
                if let Some(chrome) = crate::shell::commands::chrome_for_command(id) {
                    actions.push(Action::ToggleViewChrome(chrome));
                }
            }
            // This control was drawn and enabled from the moment the ribbon
            // landed, and did nothing — a live instance of D1's shape: an
            // affordance that looks available and is inert. It became
            // wirable when `RenderKey` gained `annotations`.
            "view.show_annotations" => actions.push(Action::ToggleAnnotations),
            // The page verbs — rotate, delete, extract, move. Their arms and
            // the operand rule they share live in `dispatch::pages`, split out
            // under R2; see its header for the seam and for the three page
            // commands that still have no arm.
            id if crate::app::dispatch::pages::handles(id) => {
                crate::app::dispatch::pages::dispatch(self, id, actions);
            }
            // ★ **The three page commands that have no arm, and why each one
            // is absent rather than forgotten.**
            //
            // All three are registered, drawn on the Pages tab and reachable.
            // None of them is in the page tile's context menu — which is the
            // one place `RIBBON_IA.md` P3's "render nothing rather than a
            // control that fails" rule would be breached by their absence, and
            // `panels::pages`' own test records the exclusion as deliberate:
            // they *"are document-level verbs that act on the whole file rather
            // than on the sheets pointed at, and both open a dialog this build
            // has not built."*
            //
            // | command | engine verb | what is missing |
            // |---|---|---|
            // | `pages.split` | `pdfcer_core::pageops::split` | a **boundary chooser**. `plan_split` takes a `SplitPlan` — every N pages, at bookmarks, at an explicit list — and a destination *directory* plus a name template. There is no honest default: splitting a 36-sheet drawing set into 36 files because nobody was asked is not a lesser version of the feature |
            // | `pages.merge_into` | `EditSession::merge_document` | ★★ **NOT what this row used to say.** It read: *"`insert` returns the bytes of a NEW document rather than mutating the session … wiring it means replacing `OpenDoc::session` wholesale, which discards the command log."* That was true and stopped being true on 2026-08-18, when the engine answered the filed request with `merge_document` — in-session, one undo entry, field collisions renamed. What is left is a **file picker and an insertion point**, both of which `pages.insert_from_file` already has |
            //
            // ★★★ `pages.insert_from_file` was a THIRD ROW of this table until
            // 2026-08-28, claiming *"the same two things, for the same
            // reason."* **It has had a dispatch arm since 2026-08-18** — in
            // this same file, two hundred lines above. A table describing a
            // command as unimplemented, in the file that implements it, is the
            // clearest possible statement of why a reason is prose and prose is
            // not checked by anything. Found in an audit of eleven blocker
            // reasons, of which six were stale.
            //
            // They therefore fall through to `command-unimplemented`, which is
            // the honest report and is what the trace has always said about
            // them. Deliberately NOT given arms that trace a prettier decline:
            // a command that says "not yet" is still a command that does
            // nothing, and dressing it up would make the trace harder to grep
            // for what is genuinely unwired.
            // ★ The two text-copy verbs moved to `dispatch::textcopy` on
            // 2026-08-20, when this file crossed R2's ceiling for the third
            // time. The seam is the one `dispatch::images` and
            // `dispatch::pages` were drawn along: a family whose bodies are
            // longer than most whole tabs and whose subject — *read text out
            // of this document and put it on the clipboard* — is its own.
            // That module's header carries the argument, including why
            // neither of them raises an `Action`.
            id if textcopy::handles(id) => textcopy::dispatch(self, ctx, id),
            // ★ Find. `Ctrl+F`, and the status bar's Find toggle.
            //
            // A **toggle**, not a show: Ctrl+F is the chord every application
            // in the class uses to open a find bar, and the operator whose
            // fingers already know it expects the second press to put the
            // canvas back. That is the opposite of `file.properties`
            // immediately below, which is deliberately *idempotent* — and the
            // difference is not inconsistency. Properties is offered from a
            // context menu to *describe the row just clicked*, so a second
            // invocation that hid the description would be actively hostile;
            // Find is offered from a chord whose whole idiom is a toggle.
            //
            // Raises **no action**. Opening a bar changes no document and
            // needs no frame boundary, exactly as mounting a panel does not —
            // the funnel is for work that touches a document or that must not
            // happen mid-frame, and this is neither. What *does* go through
            // the funnel is the search itself; see `crate::find`.
            //
            // The command is gated on `doc.pages`, so the ribbon and the
            // status bar cannot reach it without a document. A customized
            // keymap can, which is why the no-document case is answered here
            // rather than assumed away: the bar would draw nothing over an
            // empty shell, so opening it would be a control the operator
            // cannot see, and a trace line is the honest response.
            "edit.find" => {
                if matches!(self.status, Status::Open(_)) {
                    let open = self.find.toggle();
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!("find-toggled open={open}")
                    });
                } else {
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "find-declined reason=no-document".to_owned()
                    });
                }
            }
            // ★ The Properties panel. See [`Self::show_panel`] for the
            // mount-versus-nothing decision, which is the only interesting
            // part of this.
            //
            // `file.properties` rather than a `view.panel_*` id because
            // `RIBBON_IA.md` §5.1 puts Properties in File ▸ Document — it
            // describes the document, not the screen — and
            // `crate::panels::Panel::command_id` is the one place that
            // binding is written down.
            "file.properties" => self.show_panel(crate::panels::Panel::Properties),
            // ★ The Comments panel, and its id is the interesting part.
            //
            // `markup.comments` rather than `view.panel_comments`, which is
            // what `crate::app::modes::defaults` named for the whole time the
            // panel did not exist. `RIBBON_IA.md` §5.2 lists Comments among
            // View ▸ Panels' toggles AND §5.5 gives Markup a `Comments` group;
            // §7's migration map settles it by naming the control —
            // `Review ▸ Comments ▸ Comments` → `Markup ▸ Comments`. A ruling
            // about one control beats a list that merely contains its name.
            //
            // Unlike `view.panel_forms`, this needed no move and no tab
            // argument: Comments mounts in Review and Edit only, and both are
            // shown the `markup` tab, so no mode can mount this panel without
            // being able to reopen it. Forms had the opposite problem.
            //
            // The command has been registered and drawn since the Markup tab
            // was built (`shell::commands`, token 540) with nothing behind it.
            // This arm is the body arriving, which is why none of the five
            // registration obligations apply here.
            "markup.comments" => self.show_panel(crate::panels::Panel::Comments),
            // ★ **The panel toggles — one arm for the whole family.**
            //
            // `view.panel_bookmarks|_layers|_signatures|_objects|_forms` and
            // `file.fonts`. Registered and drawn since the ribbon landed with
            // **nothing behind them** — this arm is the body arriving, which is
            // why none of the five registration obligations apply.
            //
            // `Panel::from_command_id` is the single binding between an id and
            // a panel, exactly as `markup_for_command` and `measure_for_command`
            // are for their families, so there is no second table here to drift.
            //
            // # ★ Placement below the two literal arms is load-bearing
            //
            // `from_command_id` also answers for `file.properties` and
            // `markup.comments`, because they name panels too. A `match` takes
            // the first arm that matches, so those two are claimed above by
            // their own literals and never reach this guard — which is exactly
            // right, because **they are not toggles**. `file.properties` is
            // offered by the Objects row context menu to describe the row just
            // clicked, and a second invocation that closed the description
            // would be, in that test's own word, hostile. See
            // [`Self::toggle_panel`] for the rule this distinction expresses:
            // a control asking *"is this panel open?"* toggles; a control
            // asking *"tell me about this thing"* shows.
            //
            // Moving this arm above either of them would silently turn both
            // into toggles, which is why the ordering is written down rather
            // than left to be noticed.
            id if crate::panels::Panel::from_command_id(id).is_some() => {
                if let Some(panel) = crate::panels::Panel::from_command_id(id) {
                    self.toggle_panel(panel);
                }
            }
            // ★ Reset layout. `ResetScope::All`, and the scope is a decision.
            //
            // `RIBBON_IA.md`'s rule is why a scope exists at all: *"an
            // operator who only wanted the right dock back must not lose
            // their left one."* Honouring that properly needs a **chooser**,
            // and this build has no modal, no popup and no split-button
            // affordance to put one in — see the note on
            // `crate::text::commands::view_reset_layout`, which used to
            // promise the choice in its tooltip and no longer does.
            //
            // Given one button and no chooser, `All` is the only scope whose
            // behaviour matches the words on it: a control named "Reset
            // layout" that reset half the layout would be the more surprising
            // of the two failures. It is also the least destructive it looks:
            // `Modes::reset` restores *this mode's* default and leaves every
            // other mode's saved workspace alone.
            //
            // What a chooser needs, so the next hand does not have to
            // re-derive it: three commands (`view.reset_layout_left`,
            // `_right`, `_all`) with their own `CommandText`, an
            // `egui_shell::manifest::Item` kind that renders a split button
            // or a submenu, and this arm becoming three that pass the
            // matching `ResetScope`. `ResetScope::ALL` already lists them
            // narrowest-first, in the order such a menu should offer them.
            // ★ **The two View ▸ Window verbs**, and the pair
            // `RIBBON_IA.md` §3 named as *"the single most confusing thing in
            // the current ribbon"*: registered, glyphed, grouped, bound to
            // `Ctrl+H` and `F11`, listed in the shortcuts reference, and with no
            // arm at all until 2026-08-15. `file.save_copy`'s defect twice more.
            //
            // Both are one line, because everything they decide lives in
            // `crate::app::window`: why `view.read_mode` is **not** a duplicate
            // of `mode.read` (chrome versus capability, and three of three
            // reference applications separate them), what read mode hides and
            // why the status bar is deliberately not on that list, why Escape
            // was declined as a second way out, and why one state lives in
            // `egui::Memory` while the other is read back off the viewport
            // rather than shadowed on the application.
            //
            // Neither raises an `Action`, for `edit.find`'s reason: nothing
            // about the document changes, so there is nothing for the undo log
            // to hold and nothing to order against. Neither *could* go through
            // the funnel without weakening it, either — the apply phase is
            // deliberately handed no `egui::Context`, and both of these need
            // one.
            //
            // No capability check and no mode check. A window is not a document:
            // Read may hide its own chrome and Edit may fill the display, and
            // `app::modes::capability` governs what may be *authored*, which is
            // neither of these.
            "view.read_mode" => {
                let on = crate::app::window::toggle_read_mode(ctx);
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("read-mode on={on}")
                });
            }
            // `asked=` rather than `on=`, and the difference is not pedantry: a
            // viewport command is queued and answered by the windowing backend,
            // so `ViewportInfo::fullscreen` still reports the old value on this
            // frame. A trace line claiming the window *is* full screen on the
            // strength of a request would be the harness's only evidence, and it
            // would be wrong exactly when the backend refused.
            "view.fullscreen" => {
                let asked = crate::app::window::toggle_fullscreen(ctx);
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("fullscreen asked={asked}")
                });
            }
            // ★ **Render diagnostics** — the last of the four wired on
            // 2026-08-15, and the one `shell::commands::reach` called the least
            // defensible on its list, *because the work behind it was already
            // done*: the renderer has produced this report since S0 and the
            // status bar has shown a one-line summary of it since S2.
            //
            // A dialog, in the shape `file.print`, `file.ocr` and `file.about`
            // established, because `shell::manifest::tools`' argument for the
            // command is an argument about placement and it names a dialog's
            // properties exactly: *"a thing you go and look at when something is
            // wrong"*, with *"room to be more than one line"*. See
            // `crate::dialogs::diagnostics` for why not a panel, why the status
            // bar keeps its line, and why the two cannot disagree.
            //
            // Both guards — no document, already open — are inside
            // `open_diagnostics`, at the one place the dialog is ever built, for
            // the reason `DialogsState::open_print` documents: the ribbon
            // control is gated on `doc.open` and a chord bound to the same id is
            // not.
            "tools.render_diagnostics" => self.dialogs.open_diagnostics(&self.status),
            // ★★ **The four panel-layout verbs**, 2026-09-04.
            //
            // A guard arm rather than four literals, and it sits ABOVE
            // `view.reset_layout` for no ordering reason at all — the two sets
            // are disjoint by construction, since `dispatch_panel_layout`
            // matches four exact ids and returns `false` for everything else.
            // It is here because it is the same subject.
            //
            // ★ It must stay BELOW the `Panel::from_command_id` guard above,
            // and that one IS an ordering constraint: that guard matches any
            // id a panel claims, and these four are not panel ids — but a
            // future panel command id beginning `view.panel_` would be, and
            // the toggle is the more surprising thing to lose.
            id if panels::claims(id) => {
                self.dispatch_panel_layout(id);
            }
            "view.reset_layout" => {
                let scope = egui_shell::layout::ResetScope::All;
                let changed = self.modes.reset(
                    scope,
                    &mut self.dock,
                    &mut self.layout,
                    &self.panel_registry,
                );
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "layout-reset scope={scope} changed={changed}"
                    )
                });
            }
            // ★ The mode selector's three keyboard positions.
            //
            // `RibbonState::set_mode`'s own doc commissions exactly this:
            // *"This is what an application calls when the operator presses
            // the Ctrl+1 its manifest bound to `mode.read` — the shell
            // reports the command's token, the application dispatches it,
            // and dispatching it means calling this."*
            //
            // Wired here because the keymap route now reaches it. The mode
            // ids are the command ids without their `mode.` prefix, which is
            // the manifest's own convention — see
            // `crate::shell::manifest::built_in`'s mode list beside its
            // keymap — and an id the manifest does not declare is declined
            // rather than adopted, so a customized keymap naming a fourth
            // mode cannot put the ribbon into a state it has no tab list for.
            //
            // Nothing else happens here: the dock follows on the same frame,
            // in `Self::docks`, which compares `ribbon.mode()` against
            // `modes.active()` and moves the workspace across. One place does
            // that, and it must stay one place — see its ★ comment on why the
            // order of *record* and *restore* is load-bearing.
            "mode.read" | "mode.review" | "mode.edit" => {
                if let Some(mode) = id.strip_prefix("mode.") {
                    if self.modes.is_known(mode) {
                        self.ribbon.set_mode(mode.to_owned());
                    } else {
                        crate::diag::trace(|| {
                            format!(
                                // ui-text-exempt: diagnostic trace, never displayed.
                                "command-declined id={id} reason=mode-not-declared"
                            )
                        });
                    }
                }
            }
            other => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        "command-unimplemented id={other}"
                    )
                });
            }
        }
    }

    /// **The pages every `pages.*` verb acts on**, or `None` when there is
    /// nothing to act on at all.
    ///
    /// One helper for five arms, so the operand cannot be derived two ways —
    /// and derived it must be, because the answer is a *join* of two pieces of
    /// state this struct owns separately: the Pages panel's multi-select
    /// ([`crate::panels::PanelsState::selected_pages`]) and the open document's
    /// current page.
    ///
    /// The **rule** is not here. [`crate::panels::pages::ops::operands`] owns
    /// it, is pure, and is unit-tested against the three cases that matter
    /// (nothing picked, something picked, a pick left stale by an edit the
    /// panel has not yet reconciled). This method's whole job is to go and get
    /// the two facts, which is what keeps it a routing helper rather than a
    /// second statement of a rule — `HANDOFF.md` §6.
    ///
    /// # Why `Option` rather than an empty `Vec`
    ///
    /// So a caller cannot accidentally raise an action with an empty operand
    /// list that the engine would then have to refuse. `None` means *no
    /// document, or a document with no pages* — the second of which is a legal
    /// PDF (`/Count 0`) that pdfcer opens and says so about. Both are
    /// unreachable from the ribbon, which gates every one of these on
    /// `doc.pages`, and both are reachable from a **chord**, because a keymap
    /// reaches any command from any state.
    ///
    /// # Returns
    ///
    /// Ascending, de-duplicated and in range — which is what
    /// `EditSession::delete_pages` and `rotate_pages` need in order to succeed
    /// rather than refuse the whole batch over one bad index.
    fn page_operands(&self) -> Option<Vec<usize>> {
        let Status::Open(doc) = &self.status else {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "pages-declined reason=no-document".to_owned()
            });
            return None;
        };
        let pages = crate::panels::pages::ops::operands(
            self.panels.selected_pages(),
            doc.view.page_index,
            doc.pages.len(),
        );
        if pages.is_empty() {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                "pages-declined reason=no-pages".to_owned()
            });
            return None;
        }
        Some(pages)
    }
}
