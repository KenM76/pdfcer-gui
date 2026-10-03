//! # `app::lifecycle` — opening a document, closing it, and the three ways an open can fail
//!
//! Three methods on [`PdfcerApp`] and one predicate: what happens when a
//! document arrives, what happens when it leaves, and how a load failure is
//! told apart from a file pdfcer has not finished supporting.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/lifecycle.md`.

use std::path::PathBuf;

use pdfcer_core::document::{DocError, Document, LoadOptions};
use pdfcer_core::xref::XrefErrorKind;

use crate::app::PdfcerApp;
use crate::app::blank;
use crate::app::settings::SettingsExt;
use crate::app::state::{OpenDoc, Status};
use crate::viewer;

impl PdfcerApp {
    /// Open `path`, replacing whatever was open.
    pub fn open_path(&mut self, path: PathBuf) {
        // **Already open? Show that tab instead of opening it twice.**
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
        // `LoadOptions::new()`, spelled rather than defaulted. It is the
        // ordinary open: decide the contradictions, report them, do not ask.
        // See `Self::reread_active_document` for the one caller that passes
        // something else, and `crate::app::state::OpenDoc::load_options` for
        // why the value is then remembered.
        self.open_path_inner(path, None, LoadOptions::new());
    }

    /// **Open a document that needs a password, with one** — `Action::OpenWithPassword`.
    pub fn open_path_with_password(
        &mut self,
        path: PathBuf,
        password: &crate::secret::Secret,
    ) -> Option<crate::dialogs::password::Rejection> {
        // The `NeedsPassword` TAB is closed first, and it must be.
        //
        // `slot_of_path` matches `NeedsPassword` deliberately — a failed open
        // still occupies a tab so the operator can see why — and
        // `open_path_inner` adds a NEW slot. Without this the successful retry
        // would leave two tabs over one path: one showing the document and one
        // still saying it needs a password. `open_path`'s guard cannot be reused
        // here for the same reason: it would find that tab and "activate" it,
        // which shows the operator the failure they are trying to get past.
        //
        // The slot's own [`LoadOptions`] are read BEFORE it is closed, and
        // that is the whole of how the operator's chosen reading survives a
        // password prompt. `Status::NeedsPassword` carries them for exactly
        // this step; its field doc has the argument.
        let options = match self.slot_of_path(&path) {
            Some(slot) => {
                let carried = match self.slot(slot) {
                    Some(Status::NeedsPassword { options, .. }) => *options,
                    // Any other status at this path is not a password retry —
                    // the ordinary reading is right, and is what the prompt
                    // would have produced before this field existed.
                    _ => LoadOptions::new(),
                };
                self.close_slot(slot);
                carried
            }
            None => LoadOptions::new(),
        };
        self.open_path_inner(path, Some(password), options)
    }

    /// **Read the open document's bytes again under a different reading** —
    /// the second half of `Pass 283.0`, and this shell's answer to the middle
    /// clause of the operator's own ruling.
    pub(crate) fn reread_active_document(&mut self, options: LoadOptions) -> bool {
        let Some(path) = self.active_document_path() else {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "reread-declined reason=no-file".to_owned()
            });
            return false;
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI.
            //
            // The policy is spelled as a STABLE token rather than left to
            // `{:?}` on the engine's enum. A driven check keys on this line,
            // and a `Debug` rendering is a formatting detail of somebody
            // else's crate — this project has already shipped one
            // machine-read field that changed meaning when a `Debug` impl did.
            format!(
                "reread-begin policy={} path={path:?}",
                crate::app::state::policy_token(options)
            )
        });
        if let Some(slot) = self.slot_of_path(&path) {
            self.close_slot(slot);
        }
        self.open_path_inner(path, None, options);
        true
    }

    /// The file behind the document on screen, or `None` when there is not one.
    fn active_document_path(&self) -> Option<PathBuf> {
        match &self.status {
            Status::Open(doc) => doc.stored_under().map(std::path::Path::to_path_buf),
            _ => None,
        }
    }

    /// The shared body of [`Self::open_path`] and [`Self::open_path_with_password`].
    fn open_path_inner(
        &mut self,
        path: PathBuf,
        password: Option<&crate::secret::Secret>,
        options: LoadOptions,
    ) -> Option<crate::dialogs::password::Rejection> {
        //
        // `Document::load(p)` is `load_with_options(p, None, LoadOptions::new())`
        // and `load_with_password(p, pw)` is the same with a password — the
        // engine says so at both definitions, and the equivalence is what makes
        // this collapse a *widening* rather than a behaviour change: every
        // existing caller reaches exactly the bytes it reached before, because
        // `open_path` supplies exactly the defaults those two verbs supplied.
        let loaded = Document::load_with_options(
            &path,
            password.map(crate::secret::Secret::expose),
            options,
        );
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
                    let mut open = OpenDoc::new(path, self.settings.open_session(doc), pages);
                    // Assigned here and nowhere else. `OpenDoc::assemble`
                    // starts it at `LoadOptions::new()` for the two dozen test
                    // constructors that neither know nor care; this is the one
                    // site that knows, and it is one line from the load that
                    // used the value.
                    open.load_options = options;
                    Status::Open(Box::new(open))
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
                // The reading travels with the tab. See the field's doc: an
                // encrypted file re-read under `KeepFirst` must not come back
                // under `KeepLast` because the password prompt forgot.
                Status::NeedsPassword { path, options }
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
    pub fn new_document(&mut self) {
        self.adopt_created(blank::document());
    }

    /// `file.new_from_template` — a blank document at a **chosen** sheet size.
    pub fn new_document_sized(&mut self, rect: pdfcer_core::page_tree::Rect) {
        self.adopt_created(blank::document_sized(rect));
    }

    /// The half of the two New verbs that is not about *what* was created.
    pub(in crate::app) fn adopt_created(
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
    fn adopt(&mut self) {
        // Give the document the operator's settings — FIRST, before anything
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

        // Apply the OPENING preferences — how this page is fitted, and which
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
            doc.frame.observed_zoom = doc.view.zoom;
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "opening-view fit={:?} zoom={:.3} rulers={} grid={} guides={}",
                    doc.view.fit, doc.view.zoom, doc.view.rulers, doc.view.grid, doc.view.guides,
                )
            });

            // **DOES THIS DOCUMENT REACH OUTSIDE ITSELF?** — asked once,
            // here, because this is the moment the operator has the file and
            // has not yet acted on it.
            //
            // pdfcer runs none of these (NF4 is standing: actions are recognised
            // and round-tripped, never executed), so nothing is about to
            // happen — which is exactly why this is a sentence and not a
            // dialog. What the operator can do is KNOW, before they hand the
            // drawing on or press a button in a viewer that does run them.
            //
            // Silent on the overwhelming majority of documents. See
            // `reachout::ReachOut::worth_saying` for why an ordinary
            // calculating form must produce nothing at all.
            let reach = crate::app::reachout::scan(&doc.session);
            if reach.worth_saying() {
                let sentence = crate::text::reachout::disclosure(reach);
                // The SENTENCE is traced, not merely the fact that one was
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

        // Forget the panels' own view state, because a NEW DOCUMENT is
        // open and none of it describes anything any more.
        //
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
        // …and the search results, for a stronger version of the same
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

        // Remember the file — but only if it actually opened.
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
        // …and only if the document HAS a file. `stored_under` is what says
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

        // **The page-display mode this document opens in.**
        //
        //
        // 1. **what this document was last shown in**, from
        //    `viewer::remembered` — *"so a sheet set does not inherit a
        //    report's setting"*;
        // 2. failing that, **the ribbon mode's default**, from
        //    `PageDisplay::default_for_mode` — which is where
        //    `MODES_AND_PANELS.md`'s "Read defaults to continuous scroll;
        //    Review and Edit default to single page" lives.
        //
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
        // A **created** document reaches the second source every time, and
        // that is the third consequence of it having no file: `stored_under`
        // answers `None`, so nothing is recalled and the mode's default
        // applies. That is the correct answer rather than a fallback — nobody
        // has ever chosen an arrangement for a document that did not exist a
        // moment ago — and it is why `file.new` in Read shows the blank sheet
        // continuous while `file.new` in Edit shows it single-page, with no
        // code here saying anything about `file.new` at all.
        let ribbon_mode = self.ribbon.mode().unwrap_or_default().to_owned();
        // **The middle tier, added 2026-08-31** — `OPERATOR_REQUESTS.md`
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
        // **Off-page display, resolved for the mode this document opens
        // in** — the operator's request of 2026-09-11: *"by default, read
        // doesn't show off page items, review and edit do show off page
        // items … their preference is remembered for each read review edit
        // modes."*
        //
        // TWO tiers, not three, and the missing one is the point:
        // `crate::app::prefs::offpage` holds the operator's answer PER MODE
        // and falls back to the mode's rule. There is deliberately no
        // per-document tier — see that module's *"What is NOT stored here"*
        // — because a document that remembered its own answer would fight
        // the mode's, and the operator asked for the mode's.
        //
        // Hoisted out of the borrow below for the same reason
        // `default_display` is: `self.prefs` and `self.status` are two
        // fields, but reading one inside a `&mut` of the other reads worse
        // than it compiles.
        let off_page = self.prefs.off_page.for_mode(&ribbon_mode);
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
                    //
                    // The resolution above is
                    // `remembered.or(default_display).unwrap_or(mode rule)` --
                    // document, then the operator's standing preference, then
                    // the mode. The disclosure tested only `remembered` and
                    // called everything else `mode-default`, so a display that
                    // came from the STANDING PREFERENCE was reported as having
                    // come from the mode's rule.
                    //
                    // That is not cosmetic. The standing preference is the
                    // whole of O80 — *"it should remember my page display
                    // preferences from my last closing of the program"* — and
                    // its two possible states are "the preference was honoured"
                    // and "there was no preference, so the mode decided". This
                    // line rendered them identically, which means a driven
                    // check of the feature had no oracle and would have passed
                    // against a build where the middle tier was never read.
                    //
                    // Found while writing that check, which is the second
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
            // Seeded here rather than in `ViewState::default`, because the
            // answer depends on the ribbon mode and a `ViewState` does not
            // know one. The default stays `false` so that a state built in a
            // test is the plain one; every state the OPERATOR sees passes
            // through here or through `prefs::offpage::apply_mode`, and those
            // are the only two moments the answer changes without a click.
            doc.view.off_page = off_page;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("off-page-seed mode={ribbon_mode} on={off_page}")
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
                    // The recovery counters ride along with the open line, so
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
    pub fn close_document(&mut self) {
        if self.document_count() == 0 {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            crate::diag::trace(|| "close nothing-open".to_owned());
            return;
        }
        //
        //
        // Everything this function's own docs say about what closing must
        // *not* forget — the recent list, the dock arrangement, the mode — is
        // still true and still true for the same reasons, and is now also true
        // of the documents that stay open.
        self.close_slot(self.active_slot);
    }

    /// **Resume what the operator asked for, once they have said what to do
    /// about their unsaved edits.**
    pub(super) fn resume_after_unsaved(&mut self) {
        let Some((intent, outcome)) = self.dialogs.take_unsaved_answer() else {
            return;
        };
        use crate::dialogs::unsaved::{Outcome, PendingIntent};

        // **Two writing outcomes now, and BOTH gate the intent on a real
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
        // `write_in_place` grew its `bool` for exactly this caller, which is
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
            // **Save all** — `OPERATOR_REQUESTS.md` O102. Every dirty
            // document that has a file, written in place, then this one's
            // question is answered too.
            //
            // `Some(false)` when ANY of them failed, so the guard below
            // abandons the whole resume. That is the conservative direction and
            // it is the one this arm's neighbours already take: a save that did
            // not happen must never be a route to discarding the work it was
            // supposed to preserve, and on a quit the thing being resumed is
            // *closing the program*.
            //
            // Documents with no file are NOT written and are not failures:
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

        // Taken unconditionally, and consulted only on the `Close` arm.
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
            // The reading travels the whole way. It was chosen in
            // `crate::panels::docprops`, carried through `Action`, parked in
            // the intent while the operator answered about their edits, and is
            // handed to the loader here — with no step in between able to
            // supply a default. That chain is the feature; a
            // `LoadOptions::new()` anywhere along it would be a control that
            // appears to work and quietly does the ordinary thing.
            //
            // The return value is dropped deliberately: `false` means there was
            // no file behind the document, and by this point there was one —
            // the control that raised the action is only drawn over a document
            // that reported load anomalies, which a document with no bytes
            // cannot have. `reread_active_document` traces either way, so the
            // impossible case is legible rather than silent.
            PendingIntent::Reread { options } => {
                let _ = self.reread_active_document(options);
            }
        }

        // **And carry on closing, if this answer was one of a sequence.**
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
            // The `bool` is discarded here, and the asymmetry with
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
    #[must_use]
    pub fn save_pending(&self) -> bool {
        false
    }
}

/// Whether a load failure is "pdfcer is not finished" rather than "your file
/// is broken".
fn is_unsupported_structure(err: &DocError) -> bool {
    matches!(
        err,
        DocError::Xref(x) if matches!(x.kind, XrefErrorKind::EncryptionUnsupported)
    ) || matches!(err, DocError::Encryption(_))
}

/// What must never be true after a document arrives or leaves, in its own
/// file since 2026-09-10 — see [`tests`]'s header for the seam.
#[cfg(test)]
mod tests;
