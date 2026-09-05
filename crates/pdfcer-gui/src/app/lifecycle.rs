//! # `app::lifecycle` — opening a document, closing it, and the three ways an open can fail
//!
//! Three methods on [`PdfcerApp`] and one predicate: what happens when a
//! document arrives, what happens when it leaves, and how a load failure is
//! told apart from a file pdfcer has not finished supporting.
//!
//! ## ★ Why this is its own file
//!
//! `app/state.rs` crossed the 1,500-line gate (rule R2) when canvas text
//! selection added the page-text cache and the text selection to [`OpenDoc`].
//! The rule's own justification is why the split is here rather than at
//! whichever line the count happened to reach: *"the value of the limit is that
//! the file has to have a single subject"*.
//!
//! `state.rs`'s subject is **what an open document is** — the fields, the
//! render keys derived from them, the view overrides, the caches that hang off
//! it. This file's subject is **the document's lifetime on the application**:
//! `self.status` moving between [`Status::Empty`], [`Status::Open`] and
//! [`Status::Failed`], and everything that has to be forgotten on the way. The
//! two change for entirely different reasons — a new per-document cache is a
//! `state.rs` change, a new thing to forget on close is a change here — and
//! they are read at different times.
//!
//! It is the same seam `app/mod.rs` has already been split along three times,
//! producing `dispatch.rs` (*what does this verb do*), `conditions.rs` (*what
//! is true right now*) and `gating.rs` (*what is this mode allowed to do*). The
//! test for whether a split was along a seam is whether the tests came with it,
//! and they did: the four below are all about the *transition*, and none of them
//! reads a field of [`OpenDoc`] except to check it was reset.
//!
//! ## The three ways an open fails, and why they are three
//!
//! `crate::text`'s header carries the copy argument — *the file is wrong*, *the
//! file is fine and pdfcer is not finished*, *the file is encrypted and pdfcer has
//! not been told the password*. What lives here is the **branch**, and its one
//! rule: it is made on **structured error data** from `pdfcer-core`, never by
//! inspecting a message string. [`is_unsupported_structure`] is that rule, in
//! one place, so a new refusal from the engine is added to a `matches!` rather
//! than to a substring search that decays silently.

use std::path::PathBuf;

use pdfcer_core::document::{DocError, Document};
use pdfcer_core::xref::XrefErrorKind;

use crate::app::PdfcerApp;
use crate::app::blank;
use crate::app::settings::SettingsExt;
use crate::app::state::{OpenDoc, Status};
use crate::viewer;

impl PdfcerApp {
    /// Open `path`, replacing whatever was open.
    ///
    /// The document is loaded **read-only**: `Document::load` maps the
    /// bytes, `page_tree::pages` flattens the page tree, and nothing here
    /// writes. S0 is a viewer.
    ///
    /// Note the deliberate structure of the match: each `Err` arm is chosen
    /// by *structured* error data, never by inspecting a message. See the
    /// module docs on the three-way failure distinction.
    pub fn open_path(&mut self, path: PathBuf) {
        // ★ **Already open? Show that tab instead of opening it twice.**
        //
        // `crate::app::documents` §3 carries the argument, and it is a
        // correctness one rather than a convenience: two tabs over one path
        // would be two `EditSession`s with two undo stacks, and a save from
        // either would silently discard the other's work.
        if let Some(slot) = self.slot_of_path(&path) {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "open-already-open slot={slot} path={path:?}"
                )
            });
            self.activate_slot(slot);
            return;
        }
        self.open_path_inner(path, None);
    }

    /// **Open a document that needs a password, with one** — `Action::OpenWithPassword`.
    ///
    /// `OPERATOR_REQUESTS.md` O108. The retry half of [`Self::open_path`], and
    /// its only caller is the dispatch of the action
    /// [`crate::dialogs::password::PasswordDialog`] raises.
    ///
    /// # ★★ Why the two are one function underneath
    ///
    /// Because everything after the load is identical — the page-tree check, the
    /// three-way failure branch, the settings funnel, the new tab, the adopt —
    /// and the *only* difference is which loading verb is called. Two copies
    /// would be two places to update when the failure branch grows a fourth
    /// case, and this shell has already paid once for a predicate with two
    /// claimants (`text_edit_focused`, which cost the Delete key and then the
    /// space bar).
    ///
    /// # ★★★ `Some(pw)` and `None` are different requests, not a defaulted one
    ///
    /// `Document::load(path)` means *"try the empty user password, then give
    /// up"* — which every conforming reader does silently before prompting.
    /// `load_with_password(path, Some(pw))` means *"try this one"*. So a caller
    /// with nothing to offer passes `None` and gets the silent attempt; the
    /// prompt refuses an empty box locally rather than passing `Some(b"")`,
    /// because that would ask the engine a question it has already answered and
    /// return a rejection the operator reads as *"my password was wrong"*.
    ///
    /// # Returns
    ///
    /// Why the password did not work, or `None` when the document opened.
    pub fn open_path_with_password(
        &mut self,
        path: PathBuf,
        password: &crate::secret::Secret,
    ) -> Option<crate::dialogs::password::Rejection> {
        // ★★ The `NeedsPassword` TAB is closed first, and it must be.
        //
        // `slot_of_path` matches `NeedsPassword` deliberately — a failed open
        // still occupies a tab so the operator can see why — and
        // `open_path_inner` adds a NEW slot. Without this the successful retry
        // would leave two tabs over one path: one showing the document and one
        // still saying it needs a password. `open_path`'s guard cannot be reused
        // here for the same reason: it would find that tab and "activate" it,
        // which shows the operator the failure they are trying to get past.
        if let Some(slot) = self.slot_of_path(&path) {
            self.close_slot(slot);
        }
        self.open_path_inner(path, Some(password))
    }

    /// The shared body of [`Self::open_path`] and [`Self::open_path_with_password`].
    ///
    /// See the latter for why the two share one, and for what `None` means as
    /// distinct from `Some` of an empty password.
    ///
    /// # Returns
    ///
    /// Why the supplied password did not work, when one was supplied and it did
    /// not. `None` on success **and** on every failure that is not about the
    /// password, because the prompt has nothing to say about a damaged file.
    ///
    /// ★★ The two password failures are carried out separately rather than
    /// collapsed into "it did not open", and `pdfcer-core` went to some trouble
    /// to make that possible: `PasswordRequiresNormalisation` exists, in its own
    /// words, *"so that failure does not masquerade as `PasswordRequired`'s 'you
    /// typed it wrong', which would send the operator to re-check a password
    /// that was correct."* Flattening them here would undo that on the last
    /// step, which is the only step the operator sees.
    fn open_path_inner(
        &mut self,
        path: PathBuf,
        password: Option<&crate::secret::Secret>,
    ) -> Option<crate::dialogs::password::Rejection> {
        let loaded = match password {
            Some(pw) => Document::load_with_password(&path, Some(pw.expose())),
            None => Document::load(&path),
        };
        // Captured before the `match` consumes the error, because the branch
        // below folds both password errors into one `Status` — which is right
        // for the tab and loses the distinction the prompt needs.
        let rejection = match (&loaded, password.is_some()) {
            (Err(DocError::PasswordRequired), true) => {
                Some(crate::dialogs::password::Rejection::Wrong)
            }
            (Err(DocError::PasswordRequiresNormalisation), true) => {
                Some(crate::dialogs::password::Rejection::NeedsNormalisation)
            }
            _ => None,
        };
        let incoming = match loaded {
            Ok(doc) => match pdfcer_core::page_tree::pages(&doc) {
                Ok(pages) => {
                    // ★ `open_session`, not `EditSession::new` — the settings
                    // funnel. A bare `new` takes the engine's defaults and
                    // silently discards the operator's `quad_point_order`,
                    // which is what it did here until 2026-08-28.
                    // `app::settings`' fourth funnel carries the argument.
                    Status::Open(Box::new(OpenDoc::new(
                        path,
                        self.settings.open_session(doc),
                        pages,
                    )))
                }
                // The header and cross-reference table were fine and the
                // page tree is not. That is a damaged file, not an
                // unimplemented feature.
                Err(err) => Status::Failed {
                    path,
                    message: err.to_string(),
                },
            },
            // §7.6: pdfcer CAN decrypt this one and has not been told how.
            // Neither damaged nor unsupported — a third thing.
            Err(DocError::PasswordRequired | DocError::PasswordRequiresNormalisation) => {
                Status::NeedsPassword { path }
            }
            Err(err) if is_unsupported_structure(&err) => Status::Unsupported {
                path,
                message: err.to_string(),
            },
            Err(err) => Status::Failed {
                path,
                message: err.to_string(),
            },
        };
        // ★ A **new tab**, since 2026-08-19, rather than a replacement.
        //
        // Note what did not have to change: `adopt` below is unchanged and
        // still runs on exactly the same schedule, because `park_and_adopt`
        // leaves the incoming document in `self.status` — which is what every
        // statement in `adopt` reads.
        self.park_and_adopt(incoming);
        self.adopt();
        rejection
    }

    /// **Make a blank document and show it, in a tab of its own.**
    ///
    /// The `file.new` half of this module, and the third member of the family
    /// whose other two are [`Self::open_path`] and [`Self::close_document`].
    ///
    /// It is a sibling of `open_path` rather than a branch inside it because
    /// the two answer different questions — *load this file* against *make a
    /// document* — and they share the only part that is genuinely common, the
    /// [`Self::adopt`] tail. What differs is one line: where the bytes come
    /// from.
    ///
    /// # Where the bytes come from, and why not from the engine
    ///
    /// [`crate::app::blank`] carries the whole argument. In one sentence:
    /// `pdfcer-core` has no way to create a document and states in
    /// `document.rs:10-19` that it never will (*"No separate
    /// builder/generation model may ever be introduced"*), so New parses a
    /// 443-byte template that ships as an asset — which makes it an **open**,
    /// which is the thing this shell already does well.
    ///
    /// # Failure is a build defect, not an operator's
    ///
    /// The `Err` arm is unreachable in a correct build —
    /// `crate::app::blank::tests` pins that the compiled-in bytes parse and
    /// hold one page — and it still produces [`Status::Failed`] rather than an
    /// `expect`. The state it describes is *"this binary was built with a
    /// corrupt asset"*, and an operator who somehow meets it gets the shell's
    /// ordinary explanatory sentence instead of a process that vanishes.
    ///
    /// # What it does NOT do
    ///
    /// **It does not change the mode.** An operator in Read who presses
    /// `Ctrl+N` gets a blank sheet they can look at and not author on, and
    /// stays in Read. None of the three reference applications has a mode
    /// system to consult, so standing instruction 4's head-count is empty here
    /// and this shell's own rule decides: the chord/mode gate
    /// (`crate::app::modes::capability::offers_command`, operator decision
    /// 2026-08-14) **refuses** a command a mode does not offer rather than
    /// switching modes to allow it. Silently moving the operator's workspace
    /// out from under them would be the same decision made the other way, in
    /// the one place it is least expected.
    ///
    /// Read is nevertheless the right mode to offer this in, and not by
    /// tolerance: standing instruction 5 is *"Read may produce a new document;
    /// it may not modify this one"*, and `file.new` is the most literal
    /// instance of that rule there could be.
    pub fn new_document(&mut self) {
        self.adopt_created(blank::document());
    }

    /// `file.new_from_template` — a blank document at a **chosen** sheet size.
    ///
    /// The other half of `RIBBON_IA.md` §5.1's *New (blank / from template)*
    /// row, and Inkscape's own split: `Ctrl+N` makes a document, this one asks
    /// what kind. Reached from [`crate::dialogs::new_document`], which is where
    /// the size, the orientation and the custom-size validation live.
    ///
    /// # ★ It is not "New, then resize"
    ///
    /// [`blank::document_sized`] serializes and re-parses, so what arrives here
    /// is an ordinary freshly-parsed document that simply is that size —
    /// nothing pending, nothing undoable. See that function's own header for
    /// why handing over an edited session would have been wrong.
    ///
    /// Everything else about it is `file.new`: same name sequence, same
    /// `Untitled` naming, same [`Self::adopt`], and the same rule that it does
    /// not change the mode.
    pub fn new_document_sized(&mut self, rect: pdfcer_core::page_tree::Rect) {
        self.adopt_created(blank::document_sized(rect));
    }

    /// The half of the two New verbs that is not about *what* was created.
    ///
    /// Extracted when the size chooser arrived, for the reason [`Self::adopt`]
    /// itself was extracted: every statement here is something that must be
    /// true of a created document, and two copies would eventually agree about
    /// four of the five. The naming, the counter, the trace and the failure
    /// arm are identical for both verbs; only the bytes differ, and the bytes
    /// arrive already made.
    fn adopt_created(
        &mut self,
        made: Result<(Document, Vec<pdfcer_core::page_tree::Page>), String>,
    ) {
        // Incremented before the name is built, so the first document of a
        // session is `Untitled 1` rather than `Untitled 0`. Per session and
        // never persisted: the number distinguishes this run's documents from
        // each other, which is all it is for.
        self.created_documents = self.created_documents.saturating_add(1);
        let name = PathBuf::from(crate::text::files::untitled(self.created_documents));

        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "new-document name={name:?} template-bytes={} made={}",
                blank::TEMPLATE.len(),
                made.is_ok(),
            )
        });

        let incoming = match made {
            Ok((doc, pages)) => Status::Open(Box::new(OpenDoc::created(
                name,
                // The funnel, as on the open path above.
                self.settings.open_session(doc),
                pages,
            ))),
            Err(message) => Status::Failed {
                path: name,
                message,
            },
        };
        // A created document is a new tab exactly as an opened one is —
        // `park_and_adopt`, then the unchanged `adopt` tail. There is no
        // already-open check here for the reason `documents` §3 gives: a
        // created document's path is a *name*, and the counter never repeats.
        self.park_and_adopt(incoming);
        self.adopt();
    }

    /// **Everything that happens to the application once `self.status` has
    /// been replaced**, whichever of the two ways replaced it.
    ///
    /// Extracted when `file.new` arrived, and the extraction is the point
    /// rather than a tidy-up: every statement below is something that has to be
    /// **forgotten or re-derived because the open document changed**, and
    /// leaving them inside `open_path` would have meant `new_document` either
    /// duplicating five of them or silently skipping one. The panels keeping a
    /// previous document's expanded rows after a New is the same defect as
    /// keeping them after an Open, and it would have been found later and by an
    /// operator.
    fn adopt(&mut self) {
        // ★ Give the document the operator's settings — FIRST, before anything
        // below can cause a render or an extraction.
        //
        // `OpenDoc::assemble` starts every document on the *shipped defaults*,
        // because it cannot reach `PdfcerApp`. Without this line an operator who
        // has configured anything would open a file and see it rendered under
        // pdfcer's answers rather than their own — and the settings window would
        // still show their choices, correctly, which is the worst combination:
        // a control that reads back what you set and does not do it.
        //
        // Here rather than at the two `Status::Open(...)` construction sites,
        // because this function's own header states why it exists: *documents
        // are opened in exactly one place*, so a thing that must be true of
        // every newly opened document is one statement at the one moment it is
        // true. A third open path added later inherits it.
        //
        // Unconditional, exactly as the two `forget_document` calls below are.
        // On a failed open there is no document to adopt into and the call does
        // nothing; branching on the status here would be a condition whose only
        // effect is to make the next reader check what it guards.
        self.adopt_settings();

        // ★ Apply the OPENING preferences — how this page is fitted, and which
        // overlays are already on.
        //
        // Here rather than inside `adopt_settings`, and the distinction is the
        // whole reason this is a second statement. `adopt_settings` runs on
        // **every settings Save** as well as on every open, because it is what
        // hands the new configuration to the document and drops the caches
        // derived under the old one. Seeding the view from inside it would mean
        // that pressing Save in the Settings window snapped the operator's page
        // back to fit-page and switched their rulers off — an edit to the view
        // they are looking at, caused by a preference about the *next* document
        // they open. `Prefs::opening_fit`'s own docs state the rule: read once,
        // never consulted again.
        //
        // After `adopt_settings` rather than before, because that is the call
        // that puts the operator's preferences on the document at all;
        // `OpenDoc::assemble` starts every document on the shipped defaults.
        // Seeding first would seed from those.
        //
        // Unconditional in the same sense as the call above: a failed open has
        // no document and the `let else` falls through.
        if let crate::app::state::Status::Open(doc) = &mut self.status {
            // Cloned rather than borrowed: `seed_view` takes `&self` on the
            // preferences and `&mut` on the view, and both live on `doc` once
            // `adopt_settings` has copied them across. A four-field `Copy`-able
            // struct is cheaper to clone than the borrow split is to argue.
            let prefs = doc.prefs.clone();
            prefs.seed_view(&mut doc.view);
            // Kept in step with the seeded zoom for the reason `assemble` sets
            // it from `view.zoom` in the first place: a document whose
            // `observed_zoom` disagreed with its `zoom` before the first frame
            // reads as one whose zoom was changed by something, and the settle
            // machinery would commit a rasterisation nobody asked for.
            doc.observed_zoom = doc.view.zoom;
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "opening-view fit={:?} zoom={:.3} rulers={} grid={} guides={}",
                    doc.view.fit, doc.view.zoom, doc.view.rulers, doc.view.grid, doc.view.guides,
                )
            });

            // ★★★ **DOES THIS DOCUMENT REACH OUTSIDE ITSELF?** — asked once,
            // here, because this is the moment the operator has the file and
            // has not yet acted on it.
            //
            // pdfcer runs none of these (NF4 is standing: actions are recognised
            // and round-tripped, never executed), so nothing is about to
            // happen — which is exactly why this is a sentence and not a
            // dialog. What the operator can do is KNOW, before they hand the
            // drawing on or press a button in a viewer that does run them.
            //
            // ★ Silent on the overwhelming majority of documents. See
            // `reachout::ReachOut::worth_saying` for why an ordinary
            // calculating form must produce nothing at all.
            let reach = crate::app::reachout::scan(&doc.session);
            if reach.worth_saying() {
                let sentence = crate::text::reachout::disclosure(reach);
                // ★★ The SENTENCE is traced, not merely the fact that one was
                // recorded. `record_note` puts prose on the status bar and
                // traces nothing, so without this a driven check could prove
                // the scan ran and could not prove the operator was told —
                // which is the entire subject. The wording is the feature here:
                // a disclosure that named the action and omitted *"pdfcer never
                // does any of that"* would be an alarm about something that
                // cannot happen in this program.
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed. It
                    // carries the operator-facing sentence so a check can read
                    // it; the sentence itself lives in `text::reachout`.
                    format!("reach-out-disclosed text={sentence:?}")
                });
                crate::app::actions::record_note(doc.edit_epoch, sentence);
            }
        }

        // ★ Forget the panels' own view state, because a NEW DOCUMENT is
        // open and none of it describes anything any more.
        //
        // This is the second half of deleting `panels::DocKey`. The caches it
        // used to guard now live on `OpenDoc` and die with it, but what is
        // left on `PanelsState` — which object rows are expanded, which row
        // the Properties panel is describing — hangs off the *application*
        // and therefore does outlive a document. Those are paint-order
        // indices: positions on one page of one revision, not identities.
        //
        // The old answer was to give the cache a document identity and
        // compare it every frame, which is what needed an `Arc` address and
        // carried the ABA hazard. The answer here is that documents are
        // opened in exactly one place — this function — so forgetting is a
        // single statement at the one moment it is true, and there is no
        // identity to key on at all.
        //
        // Unconditional, including on a failed open: whatever was showing is
        // gone either way, and stale expansion state over a document that
        // could not be read is the worse of the two states to leave behind.
        self.panels.forget_document();
        // ★ …and the search results, for a stronger version of the same
        // reason.
        //
        // A hit carries a page index and a page-space rectangle, both of which
        // are positions in ONE file. Carrying them into another is not
        // staleness — a freshly opened document's `edit_epoch` is 0, so the
        // epoch test that catches an edit would happily declare them current —
        // it is nonsense, and it would put highlights on whatever happens to
        // be at those coordinates in the new file. The query and the operator's
        // options survive; see `crate::find::FindState::forget_document`.
        self.find.forget_document();

        // ★ Remember the file — but only if it actually opened.
        //
        // The recent list is a list of documents the operator has *read*, and
        // offering one that cannot be opened invites the same failure again
        // from a surface whose whole promise is "this worked before". A file
        // that failed is not lost: it is still wherever the operator got it
        // from, and `Open…` reaches it.
        //
        // Placed here rather than in the `Action::Open` arm on purpose: this
        // is the one function that opens documents, and `argv` reaches it
        // without an action, so a caller-side call would miss the first
        // document of every session — the one an operator is most likely to
        // want back.
        //
        // `remember` absolutizes, de-duplicates, caps and writes; re-opening
        // what is already at the front of the list writes nothing at all.
        //
        // ★ …and only if the document HAS a file. `stored_under` is what says
        // so. A document made by `file.new` is called `Untitled 1.pdf` and
        // nothing is at that name, so a row for it would be a Recent entry
        // that cannot be reopened — on a menu whose entire promise is *"this
        // worked before"*. It is not an omission the operator loses anything
        // to: the document is on their screen, and the moment a save lands it
        // acquires a real path and joins the list through that.
        if let Status::Open(doc) = &self.status
            && let Some(path) = doc.stored_under()
        {
            let path = path.to_path_buf();
            self.recent.remember(&path);
        }

        // ★ **The page-display mode this document opens in.**
        //
        // Two sources, in this precedence, and the order is the operator's
        // requirement of 2026-08-12 rather than a convenience:
        //
        // 1. **what this document was last shown in**, from
        //    `viewer::remembered` — *"so a sheet set does not inherit a
        //    report's setting"*;
        // 2. failing that, **the ribbon mode's default**, from
        //    `PageDisplay::default_for_mode` — which is where
        //    `MODES_AND_PANELS.md`'s "Read defaults to continuous scroll;
        //    Review and Edit default to single page" lives.
        //
        // The two are genuinely different questions and the `Option` between
        // them carries the difference: `None` from the store means "nobody has
        // chosen for this document", which in Read mode must become
        // continuous. A store that returned `Single` for an unknown document
        // would silently invert the operator decision of 2026-08-13, and it is
        // exactly the collapse `remembered::recall`'s own docs refuse.
        //
        // Placed here, in the one function that opens documents, for the same
        // reason the recent-list call is: `argv` reaches this without an
        // action, so a caller-side version would miss the first document of
        // every session.
        //
        // The ribbon mode is read out first, as an owned `String`, so the
        // `&mut self.status` borrow below does not have to be interleaved with
        // a read of a sibling field inside a trace closure.
        //
        // ★ A **created** document reaches the second source every time, and
        // that is the third consequence of it having no file: `stored_under`
        // answers `None`, so nothing is recalled and the mode's default
        // applies. That is the correct answer rather than a fallback — nobody
        // has ever chosen an arrangement for a document that did not exist a
        // moment ago — and it is why `file.new` in Read shows the blank sheet
        // continuous while `file.new` in Edit shows it single-page, with no
        // code here saying anything about `file.new` at all.
        let ribbon_mode = self.ribbon.mode().unwrap_or_default().to_owned();
        // ★★★ **The middle tier, added 2026-08-31** — `OPERATOR_REQUESTS.md`
        // O80: *"it should remember my page display preferences from my last
        // closing of the program."*
        //
        // It already did, per document. What it could not do was answer for a
        // document it had never seen, so a choice made on one drawing meant
        // nothing on the next — which from his chair is forgetting.
        //
        // Three tiers, in order: this document's own record, then his standing
        // preference, then the mode's rule. Read from `self.prefs` here rather
        // than from `doc.prefs`, because `doc.prefs` is a snapshot taken when
        // the document opened and this decision is being made AS it opens.
        let default_display = self.prefs.default_page_display;
        if let Status::Open(doc) = &mut self.status {
            let remembered = doc.stored_under().and_then(viewer::remembered::recall);
            let display = remembered
                .or(default_display)
                .unwrap_or_else(|| viewer::PageDisplay::default_for_mode(&ribbon_mode));
            doc.view.display = display;
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "page-display mode={} source={} ribbon-mode={ribbon_mode}",
                    display.id(),
                    // ★★★ **THREE tiers, and this line reported TWO** until
                    // 2026-09-02.
                    //
                    // The resolution above is
                    // `remembered.or(default_display).unwrap_or(mode rule)` --
                    // document, then the operator's standing preference, then
                    // the mode. The disclosure tested only `remembered` and
                    // called everything else `mode-default`, so a display that
                    // came from the STANDING PREFERENCE was reported as having
                    // come from the mode's rule.
                    //
                    // ★★ That is not cosmetic. The standing preference is the
                    // whole of O80 — *"it should remember my page display
                    // preferences from my last closing of the program"* — and
                    // its two possible states are "the preference was honoured"
                    // and "there was no preference, so the mode decided". This
                    // line rendered them identically, which means a driven
                    // check of the feature had no oracle and would have passed
                    // against a build where the middle tier was never read.
                    //
                    // ★ Found while writing that check, which is the second
                    // time in three days: a trace that cannot separate the two
                    // states a check must tell apart is a trace that has not
                    // finished being written.
                    if remembered.is_some() {
                        "document" // ui-text-exempt: trace token, never displayed
                    } else if default_display.is_some() {
                        "preference" // ui-text-exempt: trace token, never displayed
                    } else {
                        "mode-default" // ui-text-exempt: trace token, never displayed
                    },
                )
            });
        }

        // Forget every de-duplicated trace slot, so this document gets its
        // own canvas line and its own region declarations rather than
        // inheriting the previous document's because the numbers happened to
        // match. §4.3 requirement 1 is "at least once per document open", and
        // a consumer is entitled to read that as a line about *this*
        // document. (Written before there was an Open command, when this
        // fired once per process, precisely because the SECOND open is the one
        // that would silently break it. There is an Open command now — this
        // function is reached from `argv`, from `file.open`'s picker and from
        // the Recent menu — so the second open happens routinely and the gate
        // reset is load-bearing rather than anticipatory.)
        crate::diag::reset_change_gates();
        crate::diag::trace(|| {
            let kind = match &self.status {
                Status::Empty => "empty",
                Status::Open(d) => {
                    // ★ The recovery counters ride along with the open line, so
                    // a trace records that this file's index was REBUILT rather
                    // than read. Without it the only evidence a document was
                    // repaired lives in a panel the operator may never open,
                    // and a support conversation about a drawing that "looks
                    // wrong" has nothing to go on. See
                    // `panels::docprops::recovery_note` for the
                    // operator-facing half and for why it is not a page badge.
                    let recovered = d
                        .session
                        .document()
                        .recovery()
                        .map_or_else(String::new, |r| {
                            format!(
                                " recovered=1 objects={} collisions={} repaired={}",
                                r.file_level_objects + r.objstm_objects,
                                r.last_wins_collisions,
                                r.stream_lengths_recovered + r.missing_endobj_recovered,
                            )
                        });
                    return format!(
                        "open ok pages={} path={:?}{recovered}",
                        d.pages.len(),
                        d.path
                    );
                }
                Status::Failed { .. } => "failed",
                Status::Unsupported { .. } => "unsupported",
                Status::NeedsPassword { .. } => "needs-password",
            };
            format!("open {kind}")
        });
    }

    /// **Close whatever is open and go back to [`Status::Empty`].**
    ///
    /// The other half of [`Self::open_path`], and it forgets exactly what
    /// that function forgets — which is the whole of why it exists as a
    /// sibling rather than as `self.status = Status::Empty` at the call site.
    ///
    /// # What closing has to forget, and why each thing is here
    ///
    /// - **The document itself.** Dropping the [`Status::Open`] box drops the
    ///   `Arc<EditSession>`, the page vector, the cached texture, the
    ///   decomposition and the font inventory, the selection, and the render
    ///   worker — every one of which lives *inside* `OpenDoc` precisely so
    ///   that this is a single move rather than a checklist. `OpenDoc::new`'s
    ///   own docs make the argument from the other direction: state that dies
    ///   with the document belongs on the document.
    /// - **The panels' view state**, through [`crate::panels::PanelsState::forget_document`].
    ///   Expansion sets and the Properties focus are paint-order indices —
    ///   positions on one page of one revision — and they hang off the
    ///   *application*, so they genuinely do outlive a document. Leaving them
    ///   behind means the Objects panel keeps rows expanded for a file that is
    ///   no longer open, which is the same staleness `open_path` forgets for
    ///   the same reason.
    /// - **The search results**, through
    ///   [`crate::find::FindState::forget_document`], for a stronger version
    ///   of that argument: a hit's page index and its page-space rectangle are
    ///   positions in one file, and the epoch test that catches an *edit*
    ///   cannot catch a *different document* — a freshly opened one's
    ///   `edit_epoch` is 0, so stale hits would read as current. The query and
    ///   the search options survive, because those describe the operator
    ///   rather than the document.
    /// - **The de-duplicated trace slots**, so the next document opened in
    ///   this session gets its own canvas line and its own region
    ///   declarations rather than inheriting these because the numbers
    ///   happened to match.
    ///
    /// # What it deliberately does NOT forget
    ///
    /// The **recent list**. Closing a document is not disowning it; it is the
    /// single most likely moment for an operator to reach for the one they
    /// had before it.
    ///
    /// The **dock arrangement** and the **mode**. Those belong to the
    /// operator and outlive every document, which is what
    /// [`crate::app::persistence`] exists to make true across restarts, let
    /// alone across a close.
    pub fn close_document(&mut self) {
        if self.document_count() == 0 {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            crate::diag::trace(|| "close nothing-open".to_owned());
            return;
        }
        // ★ **Closes the ACTIVE TAB**, since 2026-08-19, rather than emptying
        // the application.
        //
        // The forgetting this function used to do inline — the panels' view
        // state, the find hits, the de-duplicated trace slots — moved to
        // `crate::app::documents::PdfcerApp::close_slot`, because switching to
        // another document has to forget exactly the same three things and two
        // copies of that list is how one of them comes to be missed. That
        // module's §4 carries the table and the reason each entry is on it.
        //
        // Everything this function's own docs say about what closing must
        // *not* forget — the recent list, the dock arrangement, the mode — is
        // still true and still true for the same reasons, and is now also true
        // of the documents that stay open.
        self.close_slot(self.active_slot);
    }

    /// **Resume what the operator asked for, once they have said what to do
    /// about their unsaved edits.**
    ///
    /// Called from [`Self::ui`]'s frame, immediately after the dialogs draw.
    /// `crate::dialogs::unsaved`'s header carries the defect this closes; this
    /// function is the half that acts.
    ///
    /// # ★ Why the resume calls the lifecycle functions directly rather than
    /// re-raising the `Action`
    ///
    /// Re-raising would be the tidier-looking answer and it does not work:
    /// `Action::Close` and its three siblings now consult
    /// `DialogsState::ask_unsaved` at the top of their arms, so a re-raised
    /// action would be asked the same question again and the operator would be
    /// in a loop they can only leave by pressing Cancel. Adding a
    /// *"but not this time"* flag to the action would put a second, invisible
    /// meaning on a value the funnel's whole discipline says is plain data.
    ///
    /// So this calls `close_document`, `open_path`, `new_document` and
    /// `new_document_sized` — **the same four functions those arms call**, one
    /// line below their guards. There is no second implementation of any of
    /// them; what is skipped is exactly the guard that has just been answered.
    ///
    /// # The save branch, and why a cancelled picker cancels everything
    ///
    /// [`crate::app::save::save_copy`] answers `false` for a cancelled picker,
    /// an unavailable picker and a failed write, and this proceeds on `true`
    /// alone. Its own docs argue why none of the three is safe to proceed on;
    /// the operator-facing form is that **pressing Cancel in a file dialog must
    /// never be a way to destroy a document.**
    ///
    /// A cancelled save leaves the window closed and the document open, which
    /// is the state the operator is in the middle of anyway — they pressed
    /// Close, then thought better of naming a file. Re-asking would be the
    /// application insisting on finishing a transaction the operator abandoned.
    pub(super) fn resume_after_unsaved(&mut self) {
        let Some((intent, outcome)) = self.dialogs.take_unsaved_answer() else {
            return;
        };
        use crate::dialogs::unsaved::{Outcome, PendingIntent};

        // ★★★ **Two writing outcomes now, and BOTH gate the intent on a real
        // write** — `OPERATOR_REQUESTS.md` O65.
        //
        // The argument is the one this function's header already makes about
        // the copy and it transfers unchanged: a save that did not happen — a
        // cancelled picker, an unavailable picker, a failed write, a read-only
        // file — must never be a route to discarding the work it was supposed
        // to preserve. `pressing Cancel in a file dialog must never be a way to
        // destroy a document`, and now also: *a failed overwrite must never be
        // one either*.
        //
        // ★ `write_in_place` grew its `bool` for exactly this caller, which is
        // the same shape `save_copy` already had and for the same reason. It
        // had the value in hand and was discarding it.
        let written = match outcome {
            Outcome::SaveInPlace => Some(self.write_in_place()),
            Outcome::SaveCopy => Some(match &self.status {
                crate::app::state::Status::Open(doc) => crate::app::save::save_copy(doc),
                // Unreachable: the question is only asked over an open
                // document. Spelled rather than `unwrap`ped, because the
                // consequence of being wrong here is destroying a document to
                // satisfy a `match`.
                _ => false,
            }),
            // ★★★ **Save all** — `OPERATOR_REQUESTS.md` O102. Every dirty
            // document that has a file, written in place, then this one's
            // question is answered too.
            //
            // ★★ `Some(false)` when ANY of them failed, so the guard below
            // abandons the whole resume. That is the conservative direction and
            // it is the one this arm's neighbours already take: a save that did
            // not happen must never be a route to discarding the work it was
            // supposed to preserve, and on a quit the thing being resumed is
            // *closing the program*.
            //
            // ★ Documents with no file are NOT written and are not failures:
            // they need a destination, which is a question only the operator
            // can answer, so the cycle asks about them individually afterwards.
            // That is Word's behaviour and the only honest one.
            Outcome::SaveAll => Some(self.save_every_dirty_document()),
            Outcome::Discard => None,
        };
        if written == Some(false) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // Names the outcome, so a reader can tell a cancelled picker
                // from a refused overwrite — two failures with the same
                // consequence and very different remedies.
                format!("unsaved-resume-abandoned outcome={outcome:?} reason=not-written")
            });
            return;
        }

        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // Names BOTH halves. A trace saying only "resumed" could not
            // distinguish an operator who saved a copy first from one who threw
            // their work away, and those are the two outcomes a reader of this
            // log will most want to tell apart.
            format!("unsaved-resume outcome={outcome:?} intent={intent:?}")
        });

        // ★ Taken unconditionally, and consulted only on the `Close` arm.
        //
        // Taking it here rather than inside the arm is what bounds how long a
        // parked sequence can live: at most until the next answer of any kind.
        // A cancelled `Close others` produces no answer at all, so it would
        // otherwise sit there until some unrelated question was answered and
        // then close four documents nobody had asked about.
        let queued = self.closing_others.take();
        // Recorded before the match, which consumes `intent`.
        let was_close = matches!(intent, PendingIntent::Close);

        match intent {
            PendingIntent::Close => self.close_document(),
            PendingIntent::Open(path) => self.open_path(path),
            PendingIntent::New => self.new_document(),
            PendingIntent::NewSized {
                width_pt,
                height_pt,
            } => self.new_document_sized(pdfcer_core::page_tree::Rect::from_corners(
                0.0, 0.0, width_pt, height_pt,
            )),
        }

        // ★★ **And carry on closing, if this answer was one of a sequence.**
        //
        // `Close others` over four marked-up drawings asks four questions, and
        // the loop that asks them cannot run across a frame boundary — the
        // dialog needs frames to be answered in. So it parks the tab it is
        // keeping and returns, and this is where the rest happens.
        //
        // Only on the `Close` arm, because that is the only intent the
        // sequence can produce, and only when an answer actually arrived —
        // this function does not run at all on a cancel, which is exactly how
        // cancelling stops the sequence.
        //
        // AFTER the close above, not before: the document the operator just
        // answered about has to be gone before the loop counts what is left,
        // or it would immediately ask about it again.
        if let Some(keep) = queued
            && was_close
        {
            self.apply_close_other_documents(keep);
        }
    }

    /// **Save the open document over its own file, and record which revision
    /// is now on disk.**
    ///
    /// The body of the `Action::Save` arm, lifted out of
    /// `crate::app::actions::apply` on 2026-08-28 so that the signature
    /// warning's answer can resume **the same save** rather than re-raise the
    /// action. `resume_after_unsaved`'s own header carries the argument in its
    /// general form: re-raising would meet the guard again and put the
    /// operator in a loop they could only leave by pressing Cancel, and a
    /// *"but not this time"* flag on the action would put a second, invisible
    /// meaning on a value the funnel's whole discipline says is plain data.
    ///
    /// So there is one implementation with two callers, and what the second
    /// caller skips is exactly the guard it has just answered.
    ///
    /// # ★★★ The defect the move surfaced: `Action::Save` never returned
    ///
    /// Worth recording where the body now lives, because nothing about it is
    /// visible from the arm any more.
    ///
    /// `crate::app::actions::apply` matches a handful of actions **before** the
    /// guard that narrows `self.status` to an open document, and every one of
    /// those arms `return`s — because the `match` further down lists all of
    /// them together under
    /// `unreachable!("handled before the document guard")`. `Action::Save` was
    /// added between two arms that both return (`SaveCopy` and `Find`) and
    /// **did not return**, so it fell through the guard and into that
    /// `unreachable!`.
    ///
    /// The consequence: **every in-place save of a document that had a file
    /// behind it panicked** — which is the path `Ctrl+S` takes for every
    /// document opened from disk, since `crate::app::dispatch` routes
    /// `file.save` to `Action::Save` exactly when
    /// `crate::app::save::has_a_file` is true and to `Action::SaveCopy`
    /// otherwise.
    ///
    /// It is fixed as part of this change because the guard above it needed
    /// the same `return` and it would have been dishonest to add one and leave
    /// the other. The class is worth naming: a fall-through arm in a `match`
    /// whose *later* twin asserts unreachability is a defect the compiler
    /// cannot see, because both halves type-check and the panic is reached
    /// only at run time on one input.
    ///
    /// # ★★ The `bool` is READ here, where [`Self::write_copy_somewhere`]'s is
    /// discarded
    ///
    /// And the difference is the whole point: an in-place save that succeeded
    /// means the file on disk now holds this revision, and `OpenDoc::saved_epoch`
    /// is the only record of that. A failed save must not move it — the disk
    /// still holds the older bytes, and claiming otherwise would let
    /// `dialogs::ocr` read a file that does not have the operator's work in
    /// it.
    ///
    /// # Why the no-document case traces rather than dropping silently
    ///
    /// A keymap reaches any command from any state, and an operator who
    /// presses the save chord over an empty shell must not be
    /// indistinguishable, in the trace, from one whose keystroke never
    /// arrived. That is the same argument the arm this came from makes for
    /// sitting above the document guard in the first place.
    pub(super) fn write_in_place(&mut self) -> bool {
        match &mut self.status {
            crate::app::state::Status::Open(doc) => {
                if crate::app::save::save_in_place(doc) {
                    doc.saved_epoch = doc.edit_epoch;
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed
                        format!("save-epoch-recorded epoch={}", doc.saved_epoch)
                    });
                    true
                } else {
                    false
                }
            }
            _ => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    "save-declined reason=no-document".to_owned()
                });
                false
            }
        }
    }

    /// **Ask where a copy goes and write it there.**
    ///
    /// The body of the `Action::SaveCopy` arm, lifted out for
    /// [`Self::write_in_place`]'s reason and at the same time.
    ///
    /// ★ The `bool` is DISCARDED here, deliberately, and that is not the same
    /// as ignoring it. `crate::app::save::save_copy` answers *"did a file get
    /// written"* for exactly one caller — `crate::dialogs::unsaved`, which must
    /// not destroy a document on the strength of a save that did not happen.
    /// A plain `file.save_copy` has nothing waiting on the answer: it
    /// succeeded or it reported its own failure, and either way the next thing
    /// that happens is the operator's choice rather than this function's.
    pub(super) fn write_copy_somewhere(&mut self) {
        match &self.status {
            crate::app::state::Status::Open(doc) => {
                let _ = crate::app::save::save_copy(doc);
            }
            _ => crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "save-copy-declined reason=no-document".to_owned()
            }),
        }
    }

    /// **Save As** — write the document somewhere new and rebind it there.
    ///
    /// # ★★★ The rebinding is the command, and it is four statements
    ///
    /// `save::save_as` writes the bytes; everything that makes this a *move*
    /// rather than a copy is below, in one place, on purpose. A document whose
    /// path moved while something else did not is a document whose next
    /// `Ctrl+S` writes a file the operator is not looking at, so this is a
    /// list to be read as a whole rather than four changes scattered across the
    /// frame:
    ///
    /// 1. **`doc.path`** — the binding itself. The window title and the tab
    ///    label are both recomputed from it every frame
    ///    (`app::frame` and `app::doctabs`), so neither needs telling.
    /// 2. **`doc.saved_epoch = doc.edit_epoch`** — the new file contains every
    ///    edit, so the document is clean. Without this the tab keeps its unsaved
    ///    marker over a file that is on disk and complete, and the unsaved-close
    ///    guard would ask about a document with nothing outstanding.
    /// 3. **the recent list** — he will look for the new name there, not the
    ///    old one, and `Self::open_path` joins the list this same way.
    /// 4. **a receipt**, because the rebinding is otherwise **invisible until
    ///    the next save**, and by then the surprise has already happened.
    ///
    /// ★★ What is deliberately NOT here: any form of close or reopen. The
    /// session, its undo history and the operator's selection all continue —
    /// see [`crate::app::save::save_as`]'s own ★★ on why a round trip would be
    /// an unannounced data loss.
    ///
    /// ★ And nothing is written to `viewer::remembered` for the new path. The
    /// per-document page display belongs to a document the operator has
    /// *looked at*; inventing an entry for a file that has existed for a
    /// millisecond would put a record in that store for every Save As, and the
    /// standing preference already answers for a file with no entry.
    pub(super) fn save_as_somewhere(&mut self) {
        let crate::app::state::Status::Open(doc) = &self.status else {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "save-as-declined reason=no-document".to_owned()
            });
            return;
        };
        // The write first, and against `&*doc`: nothing is rebound until the
        // bytes are somewhere. A `None` here is a cancel, an unavailable picker
        // or a failed write, and `save_as` flattens the three for the reason
        // its own docs give — there is no member of that set on which it would
        // be safe to move the document.
        let Some(target) = crate::app::save::save_as(doc) else {
            return;
        };
        let name = target.file_name().map_or_else(
            || target.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        if let crate::app::state::Status::Open(doc) = &mut self.status {
            doc.path = target.clone();
            doc.saved_epoch = doc.edit_epoch;
            let epoch = doc.edit_epoch;
            crate::app::actions::record_note(epoch, crate::text::files::save_as_receipt(&name));
        }
        self.recent.remember(&target);
    }

    /// **Perform the save the operator has just authorised over their
    /// signature.**
    ///
    /// Called from [`Self::ui`]'s frame, immediately after the dialogs draw
    /// and immediately after [`Self::resume_after_unsaved`], for the reason
    /// that one is called there: it is not a command but a **frame-level
    /// observation** that a window the operator was looking at has been
    /// answered, and the act it authorises — a write over their own file —
    /// belongs to the application rather than to a dialog.
    ///
    /// # ★ Why it runs AFTER the unsaved drain and not before
    ///
    /// The two questions cannot be live at once — `crate::dialogs::signature`'s
    /// §7 records why the unsaved window's *Save a copy…* button deliberately
    /// does not raise this one — so the order cannot matter today. It is
    /// nonetheless fixed, in the direction that stays correct if that ever
    /// changes: the unsaved drain may **close or replace the open document**,
    /// and a save resumed after that would write the document the operator is
    /// now looking at rather than the one they were asked about. Running this
    /// second means a stale save can never outlive its subject; running it
    /// first would mean it could.
    ///
    /// # There is no cancel branch, and there does not need to be one
    ///
    /// `SignatureDialog::take_confirmation` answers `Some` only when the
    /// proceed button was pressed. A cancel, and the window's ✕, close the
    /// window and answer nothing — so this function simply does not run, and
    /// **no file is written**. That is the shape `crate::dialogs::unsaved`
    /// uses for the same reason: the destructive act does not happen until a
    /// button is pressed, which is a property of the control flow rather than
    /// of the window.
    pub(super) fn resume_after_signature(&mut self) {
        let Some(pending) = self.dialogs.take_signature_answer() else {
            return;
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // Names the save that was authorised, not merely that one was: an
            // in-place write and a copy have different consequences for the
            // operator's own file, and a reader of a trace from a machine they
            // cannot see should not have to infer which happened.
            format!("signature-confirmed pending={pending:?}")
        });
        match pending {
            // ★ The `bool` is discarded here, and the asymmetry with
            // `resume_after_unsaved` is the point rather than an omission:
            // nothing is waiting on this answer. A signature save that failed
            // has reported its own failure and the next thing that happens is
            // the operator's choice — where a failed save in the unsaved-edits
            // prompt would otherwise be followed by the document closing.
            crate::dialogs::signature::PendingSave::InPlace => {
                let _ = self.write_in_place();
            }
            crate::dialogs::signature::PendingSave::Copy => self.write_copy_somewhere(),
        }
    }

    /// **Whether a save is in flight, so an Open or a Close must wait.**
    ///
    /// # The rule, stated where it will be needed
    ///
    /// A document is written by appending an incremental update to a file the
    /// operator names. While that is happening, the bytes on disk are a
    /// partial revision and the `EditSession` the writer is reading from must
    /// not be dropped or replaced. So:
    ///
    /// > **An Open, a New or a Close must not proceed while a save is
    /// > pending.** The operator is asked what to do about it — wait, or
    /// > discard — and the action is applied afterwards or not at all. It is
    /// > never applied underneath the save.
    ///
    /// ★ `file.new` joined the list on 2026-08-14 by **reusing this
    /// predicate**, not by growing a second rule beside it. A New replaces the
    /// open document exactly as an Open does, so the question it has to ask is
    /// the same question, and the day this function reads a real save
    /// subsystem all three arms grow their confirmation together.
    ///
    /// # Why it answers `false`, and why that is not a stub
    ///
    /// ★ **`file.save_copy` was wired on 2026-08-14 and this still answers
    /// `false`**, which is worth stating explicitly because the obvious reading
    /// — "there is a save now, so this must sometimes be true" — is wrong.
    ///
    /// The predicate asks *"is a save **in flight**"*: is there a moment at
    /// which the bytes on disk are a partial revision and the `EditSession` the
    /// writer is reading from must not be dropped or replaced.
    /// [`crate::app::save::save_copy`] is **synchronous** — it is entered and
    /// finished inside one [`crate::app::PdfcerApp::apply`] call, and no frame is
    /// drawn while it is part-way through — so there is still no state in which
    /// this could be true, and `PROJECT_PLAN.md`'s no-placeholders invariant is
    /// explicit that the answer to that is **nothing**: not a confirmation
    /// dialog wired to a condition that cannot occur, and not an
    /// `unimplemented!()` waiting for an operator to find it.
    ///
    /// It is also **not** *"are there unsaved edits?"*, and conflating the two
    /// would be the expensive mistake here. A successful save-a-copy leaves the
    /// document exactly as unsaved as it was, at its own path: the copy went
    /// somewhere else. See [`crate::app::save`] §3, which carries the whole
    /// argument and the live consumer that would break —
    /// `dialogs::ocr`'s `UnsavedEdits` refusal reads `edit_epoch != 0`.
    ///
    /// What is still absent is an **asynchronous** save. `file.save` is in
    /// `crate::shell::manifest::PLANNED`, blocked on autosave and crash
    /// recovery, and it is the one that will make this predicate live.
    ///
    /// What this is instead is the **seam**: one predicate, consulted by
    /// [`crate::app::actions::Action::Open`], `Action::New` and
    /// [`crate::app::actions::Action::Close`], carrying the rule in its own
    /// docs. When the save lands, it reads that subsystem's state and the three
    /// arms grow their confirmation — in one place, already wired, rather
    /// than in three arms somebody has to remember to find. `file.close`'s own
    /// tooltip already promises the operator this behaviour
    /// ("You are asked what to do about unsaved edits first"), which is the
    /// other reason the rule is written down here rather than left implicit:
    /// the promise exists on an operator-visible surface today.
    #[must_use]
    pub fn save_pending(&self) -> bool {
        false
    }
}

/// Whether a load failure is "pdfcer is not finished" rather than "your file
/// is broken".
///
/// Matched on the structured error, never on its message. Today the live
/// case is an encryption configuration pdfcer will not decrypt (§7.6) —
/// reached either as the cross-reference layer's capability-gap refusal or
/// as a `crypto::EncryptionUnsupported` in its own right.
fn is_unsupported_structure(err: &DocError) -> bool {
    matches!(
        err,
        DocError::Xref(x) if matches!(x.kind, XrefErrorKind::EncryptionUnsupported)
    ) || matches!(err, DocError::Encryption(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, open_fixture};
    use crate::panels::objects::test_support::engine_fixture;

    // =======================================================================
    // Opening a document is what forgets the panels' state
    //
    // Moved here with `open_path` when `state.rs` was split under R2. They are
    // the test for whether that split was along a seam: every one of them is
    // about the **transition**, and none reads a field of `OpenDoc` except to
    // check it was reset.
    // =======================================================================

    /// **★ Opening a document forgets the panels' view state.**
    ///
    /// The second half of the `DocKey` deletion. Expansion sets and the
    /// Properties focus are paint-order indices that live on `PdfcerApp`, so
    /// they genuinely do outlive a document. The old answer was to compare a
    /// document identity every frame; the answer here is that documents are
    /// opened in exactly one place, so forgetting is one statement at the one
    /// moment it is true.
    ///
    /// Without it, opening a second document leaves the Objects panel with
    /// rows expanded for a page that no longer exists and the Properties
    /// panel describing whatever object lands at that index in the new
    /// file.
    #[test]
    fn opening_a_document_forgets_the_panels_focus_and_expansion() {
        let mut app = PdfcerApp::new();
        app.panels.set_focus(7);
        app.panels.tree_mut().toggle_object(7);
        assert_eq!(app.panels.focus(), Some(7));

        app.open_path(engine_fixture(FOUR_PAGES));
        assert!(matches!(app.status, Status::Open(_)), "the fixture opens");
        assert_eq!(
            app.panels.focus(),
            None,
            "a new document makes every paint-order index meaningless"
        );
        assert!(app.panels.tree_mut().objects_expanded.is_empty());
    }

    // =======================================================================
    // Phase 4 — which arrangement a document opens in
    // =======================================================================

    /// ★ **Read mode opens a document continuous; every other mode opens it
    /// single page.**
    ///
    /// `MODES_AND_PANELS.md`'s table and the operator decision of 2026-08-13,
    /// asserted through the **open path** rather than through
    /// `PageDisplay::default_for_mode` — which is already tested in its own
    /// module. What this adds is that `open_path` actually consults it: the
    /// rule existing and the rule being applied are two different facts, and
    /// the second is the one an operator experiences.
    ///
    /// Driven with no remembered choice for the fixture (nothing has ever set
    /// one for a path under the engine fixtures directory), so what is measured
    /// is the mode default and not a leftover.
    #[test]
    fn read_mode_opens_a_document_continuous_and_the_others_paged() {
        for (mode, expected) in [
            ("read", viewer::PageDisplay::Continuous),
            ("review", viewer::PageDisplay::Single),
            ("edit", viewer::PageDisplay::Single),
        ] {
            let mut app = PdfcerApp::new();
            app.ribbon.set_mode(mode.to_owned());
            app.open_path(engine_fixture(FOUR_PAGES));
            let Status::Open(doc) = &app.status else {
                panic!("the fixture opens");
            };
            assert_eq!(
                doc.view.display, expected,
                "{mode} mode opened the document in {:?}",
                doc.view.display
            );
        }
    }

    /// A freshly opened document is not mistaken for one that has been
    /// navigated to.
    ///
    /// `tracked_page` starting anywhere but at `view.page_index` would make
    /// the canvas scroll a continuous strip on the first frame after an open,
    /// which the operator did not ask for and which would fight a saved scroll
    /// position the moment there is one.
    #[test]
    fn a_freshly_opened_document_is_not_mid_navigation() {
        let doc = open_fixture(FOUR_PAGES);
        assert_eq!(doc.tracked_page, doc.view.page_index);
        assert!(doc.strip_visible.is_empty());
        assert!(doc.strip_rasters.is_empty());
        assert!(doc.render_in_flight.is_none());
    }

    // =======================================================================
    // `file.new` — making a document rather than opening one
    // =======================================================================

    /// The handler token the ribbon would raise for `id`.
    fn token_for(app: &PdfcerApp, id: &str) -> egui_shell::commands::HandlerToken {
        app.commands
            .get(id)
            .unwrap_or_else(|| panic!("`{id}` must be registered")) // ui-text-exempt: test panic, never displayed
            .handler
    }

    /// ★ **`file.new` raises `Action::New`, and applying it makes a document.**
    ///
    /// Driven through the real token lookup rather than by calling the arm,
    /// exactly as `the_close_command_empties_the_shell` is, so a command that
    /// stopped being registered fails here instead of silently taking the
    /// `command-unimplemented` path — which is the failure `file.open` and
    /// `file.close` both shipped with, and which no test that called the
    /// function directly could ever have caught.
    ///
    /// The starting state is `Empty`, which is the state New exists for: an
    /// operator who has just launched pdfcer with no argument.
    #[test]
    fn the_new_command_makes_a_blank_document_from_nothing() {
        // A bare context: this exercises the dispatcher, not a frame.
        let ctx = egui::Context::default();
        let mut app = PdfcerApp::new();
        assert!(matches!(app.status, Status::Empty));

        let mut actions = Vec::new();
        app.dispatch_token(&ctx, token_for(&app, "file.new"), &mut actions);
        assert_eq!(actions, vec![crate::app::actions::Action::New]);

        app.apply_actions(actions, 1.0);
        let Status::Open(doc) = &app.status else {
            panic!("New must leave a document open");
        };
        assert_eq!(doc.pages.len(), 1, "New makes a one-page document");
        assert_eq!(
            doc.origin,
            crate::app::state::Origin::Created,
            "a document New made has no file behind it"
        );
    }

    /// ★ **New replaces what is open, and forgets what belonged to it.**
    ///
    /// The reason [`PdfcerApp::adopt`] was extracted rather than copied. A New
    /// that left the panels' paint-order indices behind would show the Objects
    /// panel expanded over rows of a four-page drawing that is no longer open,
    /// on a document that has one blank page — and every test of `open_path`
    /// would still pass, because `open_path` would still be doing it correctly.
    ///
    /// The page count moving from four to one is what makes "replaced" a
    /// measurement rather than an assumption.
    #[test]
    fn new_replaces_the_open_document_and_forgets_its_panel_state() {
        let mut app = PdfcerApp::new();
        app.open_path(engine_fixture(FOUR_PAGES));
        app.panels.set_focus(3);
        app.panels.tree_mut().toggle_object(3);
        let Status::Open(doc) = &app.status else {
            panic!("the fixture opens");
        };
        assert_eq!(doc.pages.len(), 4, "the fixture is the four-page one");

        app.apply_actions(vec![crate::app::actions::Action::New], 1.0);

        let Status::Open(doc) = &app.status else {
            panic!("New must leave a document open");
        };
        assert_eq!(doc.pages.len(), 1, "the four-page document was replaced");
        assert_eq!(
            app.panels.focus(),
            None,
            "a paint-order index into the previous document means nothing here"
        );
        assert!(app.panels.tree_mut().objects_expanded.is_empty());
    }

    /// ★ **Successive new documents are numbered, and the number is visible.**
    ///
    /// `crate::text::files::untitled`'s own test pins that the *function*
    /// numbers; this pins that the **application** advances the ordinal, which
    /// is a different fact and the one that breaks if the increment is dropped
    /// or placed after the name is built. Without it both documents would be
    /// `Untitled 1.pdf`, the forms cache would key two different documents the
    /// same way, and the trace of a driven run could not tell a second New
    /// from a New that did nothing.
    #[test]
    fn each_new_document_is_numbered_from_one() {
        let mut app = PdfcerApp::new();

        app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
        let Status::Open(first) = &app.status else {
            panic!("New must leave a document open");
        };
        assert_eq!(first.path, PathBuf::from("Untitled 1.pdf"));

        app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
        let Status::Open(second) = &app.status else {
            panic!("New must leave a document open");
        };
        assert_eq!(second.path, PathBuf::from("Untitled 2.pdf"));
    }

    /// ★ **A document with no file gets no Recent row — and one with a file
    /// still does.**
    ///
    /// Both halves, because the interesting failure is not "New was skipped"
    /// but "the guard was written the wrong way round and now nothing is ever
    /// remembered". A Recent menu offering `Untitled 1.pdf` is a row that
    /// cannot be opened, on a surface whose whole promise is *this worked
    /// before*.
    ///
    /// `PdfcerApp::new()` under `cfg(test)` builds a `RecentFiles` that points
    /// nowhere and writes nothing, so this reads the in-memory list and leaves
    /// the operator's own recent file untouched.
    #[test]
    fn a_created_document_is_not_remembered_but_an_opened_one_is() {
        let mut app = PdfcerApp::new();

        app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
        assert!(
            app.recent.is_empty(),
            "`Untitled 1.pdf` is a name, not a file; a Recent row for it could never be opened"
        );

        app.open_path(engine_fixture(FOUR_PAGES));
        assert_eq!(
            app.recent.entries().len(),
            1,
            "the guard must not have turned the recent list off altogether"
        );

        // …and a New over the top of it does not add a second row, nor drop
        // the one that is there. Closing is not disowning, and neither is
        // replacing.
        app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
        assert_eq!(app.recent.entries().len(), 1);
    }

    /// ★ **`stored_under` is the whole of the difference, in both directions.**
    ///
    /// The predicate three call sites consult. Asserted as a pair rather than
    /// one at a time, because a version that answered `None` for everything
    /// would satisfy every assertion about created documents in this file and
    /// would silently stop persisting page-display and guide choices for real
    /// ones — a regression with no visible symptom until the next session.
    #[test]
    fn only_a_document_with_a_file_has_somewhere_to_store_its_preferences() {
        let mut app = PdfcerApp::new();

        app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
        let Status::Open(created) = &app.status else {
            panic!("New must leave a document open");
        };
        assert_eq!(created.stored_under(), None);

        let fixture = engine_fixture(FOUR_PAGES);
        app.open_path(fixture.clone());
        let Status::Open(opened) = &app.status else {
            panic!("the fixture opens");
        };
        assert_eq!(opened.stored_under(), Some(fixture.as_path()));
    }

    /// ★ **A new document lands in the mode's default arrangement, not in a
    /// remembered one.**
    ///
    /// The sibling of `read_mode_opens_a_document_continuous_and_the_others_paged`,
    /// and it asserts something that test cannot: a created document reaches
    /// the *second* source every time, because `stored_under` answers `None`
    /// and there is nothing to recall. New therefore inherits the mode the
    /// operator is in rather than changing it — see `new_document`'s own note
    /// on why it does not switch to Edit.
    #[test]
    fn a_new_document_takes_the_modes_default_arrangement() {
        for (mode, expected) in [
            ("read", viewer::PageDisplay::Continuous),
            ("review", viewer::PageDisplay::Single),
            ("edit", viewer::PageDisplay::Single),
        ] {
            let mut app = PdfcerApp::new();
            app.ribbon.set_mode(mode.to_owned());
            app.apply_actions(vec![crate::app::actions::Action::New], 1.0);
            let Status::Open(doc) = &app.status else {
                panic!("New must leave a document open");
            };
            assert_eq!(
                doc.view.display, expected,
                "{mode} mode made the new document {:?}",
                doc.view.display
            );
            assert_eq!(
                app.ribbon.mode(),
                Some(mode),
                "New must not move the operator to another mode"
            );
        }
    }

    /// …and a FAILED open forgets it too.
    ///
    /// Whatever was showing is gone either way, and stale expansion state
    /// over a document that could not be read is the worse of the two states
    /// to leave behind: the panel would look populated while the shell says
    /// the file is damaged.
    #[test]
    fn a_failed_open_forgets_the_panels_state_as_well() {
        let mut app = PdfcerApp::new();
        app.panels.set_focus(3);
        app.open_path(engine_fixture("not-a-pdf.bin"));
        assert!(
            matches!(app.status, Status::Failed { .. }),
            "this fixture must fail to open, or the test proves nothing"
        );
        assert_eq!(app.panels.focus(), None);
    }
}
