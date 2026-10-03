//! # `dialogs::ocr` — the Recognise-text transaction
//!
//! One dialog, three states, and a shape that is chosen rather than
//! conventional. It is the surface for `file.ocr`, and it is also the
//! **enforcement point for two rules** that would otherwise have nowhere to
//! live in this build.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/ocr.md`.

use std::path::{Path, PathBuf};

use egui_shell::theme::Theme;

use crate::app::state::{OpenDoc, Status};
use crate::ocr::{self, Dictionary, EngineId, Job, Refusal, Request};
use crate::text::ocr as t;

// ---------------------------------------------------------------------------
// Named regions
//
// `crate::diag::ui_rect` publishes where a control actually got drawn, so
// `tools/ui-verify` can aim a real click at it. These names are matched
// LITERALLY by `tools/ui-verify/src/checks/ocr.rs`, so renaming one silently
// un-aims the check that measures it.
//
// Why a dialog needs them at all, when the ribbon's controls get theirs for
// free: `egui_shell::ribbon` declares a rect per band control centrally, and
// nothing does that for a window this crate draws itself. Without these, the
// only way a harness could reach the Recognise button would be to guess a
// fraction of a centred window -- which goes stale the first time a sentence
// in the dialog wraps differently.
// ---------------------------------------------------------------------------

/// The whole window.
const REGION_DIALOG: &str = "ocr-dialog"; // ui-text-exempt: trace region name, never displayed

/// The page-scope group, so a driven check can find it.
const REGION_SCOPE: &str = "ocr-scope"; // ui-text-exempt: trace region name, never displayed

/// The skip-existing-text toggle.
const REGION_SKIP: &str = "ocr-skip"; // ui-text-exempt: trace region name, never displayed

/// The control that starts recognition.
const REGION_RUN: &str = "ocr-run"; // ui-text-exempt: trace region name, never displayed
/// The live progress line, drawn once a page has finished.
const REGION_PROGRESS: &str = "ocr-progress"; // ui-text-exempt: trace region name
/// The control that finishes the page in hand and keeps everything.
const REGION_STOP: &str = "ocr-stop"; // ui-text-exempt: trace region name
/// The control that abandons the run.
const REGION_CANCEL: &str = "ocr-cancel"; // ui-text-exempt: trace region name

/// Where one Recognise-text transaction has got to.
#[derive(Debug, Default)]
enum Phase {
    /// Nothing has been asked for yet.
    #[default]
    Ready,
    /// A thread is recognising.
    Working(Job),
    /// Recognition finished and **the words are in the open document.**
    ///
    /// `EditSession::add_ocr_layer` puts the layer straight into the session as
    /// one undoable edit, so this phase carries no bytes and there is no
    /// transaction to complete — what is left to report is the outcome.
    ///
    /// It carries the run's counts rather than the words, which have already
    /// been handed to the session by the time this phase is entered.
    Applied {
        /// How many pages got words.
        written: usize,
        /// How many were visited and produced none.
        skipped: usize,
        /// How many words in total.
        words: usize,
        /// `Some((attempted, of))` when the operator pressed **Stop**.
        ///
        /// The whole reason this field exists: a stopped run is a success
        /// with a caveat, and the caveat must be SAID. Without it, somebody who
        /// ended a 200-page recognition at page 40 is left believing the
        /// document is done — and finds out months later, searching for a word
        /// on page 150 that is not in the layer.
        stopped_at: Option<(usize, usize)>,
        /// The character dictionary the recogniser read through, when it
        /// has one.
        dictionary: Option<Dictionary>,
    },
    /// **The operator pressed Cancel.** Nothing was kept and nothing written.
    ///
    /// Its own phase rather than a `Refused`, because it is not a refusal:
    /// nothing went wrong and there is nothing to diagnose. The sentence it
    /// draws says what was discarded and offers to start again, where a refusal
    /// explains why the run could not happen.
    Cancelled {
        /// Pages attempted before the press, so the sentence can say what was
        /// thrown away rather than only that something was.
        attempted: usize,
    },
    /// Recognition did not happen, for a named reason.
    Refused(Refusal),
}

/// The Recognise-text dialog.
#[derive(Debug)]
pub struct OcrDialog {
    /// The page this transaction is about, captured when the dialog opened.
    ///
    /// **Captured, not read per frame**, and that is a correctness
    /// requirement rather than an optimisation. The operator can page the
    /// document while the dialog is open; a `Save` that read the *current*
    /// page index would label bytes recognised from page 3 as belonging to
    /// whatever page they had scrolled to. The recognition is of one page and
    /// the dialog remembers which.
    page_index: usize,
    /// **Which pages to recognise.**
    ///
    /// Recognising only [`Self::page_index`] is not an engine limitation:
    /// `add_ocr_layer`'s output is a complete PDF that can be fed back in, so
    /// pages chain. That was measured before this was built, because a wrong
    /// answer would have corrupted a file.
    scope: Scope,
    /// **The rail's page selection, captured when the dialog opened** —
    /// `OPERATOR_REQUESTS.md` O79.
    ///
    /// Zero-based, ascending, and possibly empty — empty is a defined answer
    /// meaning *nothing is picked*, in which case [`Scope::Picked`] is not
    /// offered at all (R9: an option with no operand renders nothing).
    ///
    /// # Captured rather than read live, and this is the decision worth
    /// arguing
    ///
    /// The rail is on screen beside this window and the operator can work it
    /// while the dialog is up. Reading the selection live would mean the
    /// label's number changed under them mid-read, and the run would cover a
    /// set they had stopped thinking about — the same failure
    /// [`Self::page_index`] already documents for *"the current page"*, which
    /// is why both are captured and neither is polled.
    ///
    /// The cost is one snapshot per dialog opening, of a `BTreeSet` that on a
    /// 36-sheet document holds at most 36 `usize`.
    picked: Vec<usize>,
    /// The range the operator typed, when [`Self::scope`] is [`Scope::Range`].
    ///
    /// Kept as **text**, not as a parsed list, so that a half-typed `1-` is a
    /// state the field can hold. Parsed on every frame by
    /// `dialogs::print::tabs::parse_page_range` — the same parser the print
    /// dialog uses, which is the point: two page-range parsers would accept
    /// different things on two surfaces of one program, and the operator would
    /// have to learn which.
    range: String,
    /// Leave alone any page that already draws text. On by default.
    ///
    /// See [`crate::ocr::Refusal::AlreadyHasText`] for the measurement that
    /// makes this the default rather than an option: a second pass over a
    /// recognised page **adds a second invisible layer**, doubling every search
    /// hit and every copy.
    skip_pages_with_text: bool,
    /// The models on offer and the one chosen.
    models: super::ocr_model::ModelPicker,
    /// The recogniser of the run this dialog started, for its report.
    engine: EngineId,
    /// The model a run just started with; [`Self::show`] stores it and its
    /// engine as the preferences, because only `show` holds them.
    remember: Option<String>,
    /// The page list the last `ocr-scope` line reported.
    ///
    /// Kept so the trace fires on a change rather than on a frame. See
    /// [`Self::scope_group`].
    traced_scope: Vec<usize>,
    /// The `attempted` count the last `ocr-progress` line reported.
    ///
    /// **Why the numbers are traced at all, when the rect already is.**
    ///
    /// `crate::diag::ui_rect(REGION_PROGRESS, …)` says *a label was drawn
    /// there*. That is enough to prove something is on screen and is **not**
    /// enough to prove the thing the operator asked for. His words were *"gives
    /// feedback on what it is doing … so that the user can see that it is doing
    /// something and hasn't frozen"* — the subject of that sentence is the
    /// numbers **moving**. A label reading `Page 1 of 8` for the whole run
    /// declares a perfectly substantial rect on every frame and is exactly the
    /// stall he is afraid of.
    ///
    /// So the driven check needs an oracle for the *content*, and a rect cannot
    /// carry one. This line does: `ocr-progress attempted=… of=… words=…
    /// chars=…`, and a check asserts that two of them differ.
    ///
    /// Traced on **change**, for the reason [`Self::traced_scope`] already
    /// gives at length: an identical line per frame for twenty seconds is a
    /// haystack, not a diagnostic. `usize::MAX` is the "nothing traced yet"
    /// sentinel rather than `0`, because `attempted == 0` is a real state the
    /// label deliberately does not draw and a `0` sentinel would make the first
    /// genuine `attempted = 0` unreportable if that policy ever changed.
    traced_progress: usize,
    /// The transaction's state.
    phase: Phase,
    /// Set by the Close button, consumed by [`Self::show`].
    ///
    /// The two-step every dialog here uses: a widget drawn from the state
    /// cannot drop the state it is being drawn from, so it records the request
    /// and the caller acts after the closure returns.
    close_requested: bool,
}

/// Which pages a recognition run covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Scope {
    /// Every page of the document. The default.
    All,
    /// Only the page the operator was looking at when the dialog opened.
    ///
    /// **When it opened**, not now — the operator can page the document while
    /// this window is up, and a run that read the *current* index would
    /// recognise a page they were no longer thinking about. That capture is the
    /// same argument [`OcrDialog::page_index`] already carries.
    CurrentPage,
    /// **The pages picked in the thumbnail rail** —
    /// `OPERATOR_REQUESTS.md` O79.
    ///
    /// The operator: *"the pages I have selected in the thumbnails."*
    ///
    /// # Why this is not the same as [`Self::Range`] with the numbers typed in
    ///
    /// Because on his documents it is the difference between a feature and a
    /// chore. A 36-sheet SolidWorks set where four sheets are scans and the
    /// rest are vector is exactly the case where **All** is minutes of wasted
    /// work, **This page** is four separate runs, and a typed range is him
    /// reading page numbers off the rail and retyping them into a text field
    /// six inches away.
    ///
    /// The rail's selection is already the operand for delete, extract,
    /// rotate and the page clipboard — `PanelsState::selected_pages`, whose
    /// own doc comment says those verbs *"respect the thumbnail rail's
    /// selection when there is one"*. OCR was the one page-scoped verb that
    /// ignored it.
    ///
    /// # Captured at OPEN, like [`Self::CurrentPage`], and for the same reason
    ///
    /// The operator can work the rail while this window is up. A run that read
    /// the selection as it is *now* would recognise a set they were no longer
    /// thinking about, and — worse — the label would have said a different
    /// number when they read it. See [`OcrDialog::picked`].
    Picked,
    /// The pages named in [`OcrDialog::range`].
    Range,
}

impl Scope {
    /// The pages this scope names, zero-based and in order.
    pub(super) fn pages(
        self,
        current: usize,
        count: usize,
        range: &str,
        picked: &[usize],
    ) -> Option<Vec<usize>> {
        match self {
            Self::All => (count > 0).then(|| (0..count).collect()),
            Self::CurrentPage => (current < count).then(|| vec![current]),
            // Filtered against the page count rather than trusted (O79). The
            // selection was captured when the dialog opened and the document
            // can be edited underneath it — a page deleted from the rail while
            // this window is up would otherwise hand the engine an index past
            // the end. `None` for an empty result, exactly as an unresolvable
            // range gives `None`, so the Recognise button greys by the path
            // that already exists.
            Self::Picked => {
                let pages: Vec<usize> = picked.iter().copied().filter(|p| *p < count).collect();
                (!pages.is_empty()).then_some(pages)
            }
            // The PRINT dialog's parser, deliberately. Two page-range parsers
            // in one program would accept different things on two surfaces and
            // the operator would have to learn which one they were talking to.
            Self::Range => crate::dialogs::print::tabs::parse_page_range(range, count)
                .filter(|pages| !pages.is_empty()),
        }
    }
}

impl OcrDialog {
    /// Build the dialog for the page `doc` is showing.
    #[must_use]
    pub(super) fn open(
        doc: &OpenDoc,
        picked: Vec<usize>,
        prefs: &crate::app::prefs::Prefs,
    ) -> Self {
        let models = super::ocr_model::ModelPicker::open(prefs);
        let engine = models
            .chosen()
            .and_then(|c| c.engine)
            .unwrap_or(EngineId::Ocrs);
        Self {
            page_index: doc.view.page_index,
            // The rail's selection, captured once — see `Self::picked` for
            // why it is a snapshot rather than a live read (O79).
            picked,
            // **All pages by default**, which is what every surveyed OCR tool
            // defaults to and what the operator was asking for. The old
            // behaviour — this page only — is still one click away and is the
            // right answer when he is checking one sheet, but it is the
            // unusual want and it should not be the assumption.
            scope: Scope::All,
            range: String::new(),
            skip_pages_with_text: true,
            models,
            engine,
            remember: None,
            traced_scope: Vec::new(),
            traced_progress: usize::MAX,
            phase: Phase::Ready,
            close_requested: false,
        }
    }

    /// Draw one frame. Returns `false` when the dialog should close.
    pub(super) fn show(
        &mut self,
        ctx: &egui::Context,
        doc: &OpenDoc,
        actions: &mut Vec<crate::app::actions::Action>,
        prefs: &mut crate::app::prefs::Prefs,
    ) -> bool {
        self.poll_worker(actions);

        // ITS OWN OS WINDOW. OCR is the longest-running thing in this program
        // — a job an operator starts and then goes back to work while it runs —
        // and a progress window locked inside the application frame is a window
        // that has to be closed to keep working.
        //
        // The dialog region is published from INSIDE the callback, because
        // there is no `egui::Window` response rect to take it from;
        // `ui.max_rect()` is the same rectangle in the coordinates the harness
        // converts, and `dialogs::host` tags it with this viewport.
        let (frame, ()) = crate::dialogs::host::Host::new(
            "ocr", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(560.0, 420.0),
            // A floor, on the print and About dialogs' own reasoning: a
            // resizable window with no minimum can be dragged down to a title
            // bar and a scrollbar, which is a state with no way out but
            // closing.
            egui::vec2(420.0, 260.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_DIALOG, ui.max_rect());
            self.body(ui, doc);
        });
        let open = !frame.closed;
        if let Some(model) = self.remember.take()
            && (prefs.ocr_engine != Some(self.engine)
                || prefs.ocr_models.model.as_deref() != Some(model.as_str()))
        {
            prefs.ocr_engine = Some(self.engine);
            prefs.ocr_models.model = Some(model.clone());
            let saved = prefs.save().is_ok();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "ocr-engine-remembered engine={} model={model} saved={saved}",
                    self.engine.key()
                )
            });
        }

        open && !std::mem::take(&mut self.close_requested)
    }

    /// Move a finished job into the phase that describes its answer.
    fn poll_worker(&mut self, actions: &mut Vec<crate::app::actions::Action>) {
        let Phase::Working(job) = &mut self.phase else {
            return;
        };
        let Some(outcome) = job.poll() else {
            return;
        };
        // Three endings, and the shell must keep them apart.
        //
        // `Complete` is the run finishing on its own. `Stopped` is the operator
        // asking for what had been done so far, which is a SUCCESS with a
        // caveat that must be said. `Cancelled` is the operator asking for
        // none of it, which touches the document not at all.
        //
        // Collapsing Stopped into Complete would leave somebody who ended a
        // 200-page run at page 40 believing the whole document was recognised;
        // collapsing Cancelled into "nothing was recognised" would tell them
        // the recogniser could not read their scan, which is a different
        // sentence with a different remedy.
        let (outcome, stopped_at) = match *outcome {
            crate::ocr::progress::Outcome::Complete(result) => (result, None),
            crate::ocr::progress::Outcome::Stopped {
                result,
                attempted,
                of,
            } => (result, Some((attempted, of))),
            crate::ocr::progress::Outcome::Cancelled { attempted } => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("ocr-cancelled attempted={attempted}")
                });
                self.phase = Phase::Cancelled { attempted };
                return;
            }
        };
        self.phase = match outcome {
            Ok(recognised) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed.
                        //
                        // `recognised=` beside the page counts, because a
                        // count of pages alone is not an ink trail: a
                        // build whose placement silently dropped every word
                        // would emit an otherwise identical line, and the pair
                        // is what makes the numbers comparable from a trace
                        // alone. What was WRITTEN is reported separately by
                        // the edit itself — see `Action::ApplyOcr` — because
                        // that is now a different subsystem's answer.
                        "ocr-recognised pages={} skipped={} recognised={} dpi={:.0}",
                        recognised.pages_written,
                        recognised.pages_skipped,
                        recognised.words_recognised,
                        recognised.effective_dpi,
                    )
                });
                let phase = Phase::Applied {
                    written: recognised.pages_written,
                    skipped: recognised.pages_skipped,
                    words: recognised.words_recognised,
                    dictionary: recognised.dictionary.clone(),
                    // Carried into the outcome so the sentence the operator
                    // reads afterwards can say the run ended early. A partial
                    // layer reported as a whole one is the failure this whole
                    // pair of buttons has to avoid.
                    stopped_at,
                };
                // **The edit is raised here, in the poll, rather than in the
                // window body.**
                //
                // Two reasons, and the second is the one that matters. A dialog
                // scrolled behind another window still polls, so the operator's
                // recognition lands whether or not this window drew — the same
                // argument that put `poll_worker` outside `body` in the first
                // place. And a body that raised an edit would raise it on
                // *every* frame it drew this phase, which is forty undo entries
                // for one recognition.
                actions.push(crate::app::actions::Action::ApplyOcr {
                    pages: recognised.pages,
                    engine: recognised.engine,
                });
                phase
            }
            Err(refusal) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("ocr-refused reason={refusal:?}")
                });
                Phase::Refused(refusal)
            }
        };
    }

    /// Everything inside the window.
    fn body(&mut self, ui: &mut egui::Ui, doc: &OpenDoc) {
        let theme = Theme::of(ui.ctx());
        ui.label(t::intro());
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        // Filled by the `Working` arm and consumed after the match, rather
        // than traced where it is read.
        //
        // The match borrows `self.phase` immutably for the whole of its body,
        // so nothing inside it may touch `self.traced_progress`. Threading the
        // tally out through a local is the smallest way to keep both — the
        // alternative, destructuring `self` into disjoint fields, would have to
        // account for `self.ready(…)` in the arm above, which needs the whole
        // of `self`.
        let mut progress_seen: Option<(usize, usize, usize, usize)> = None;

        match &self.phase {
            Phase::Ready => self.ready(ui, doc),
            Phase::Working(job) => {
                // **Ask for the next frame explicitly, and do not rely on
                // the spinner to do it.**
                //
                // The worker is on another thread and nothing it does generates
                // an input event. egui is immediate-mode and idle: with no
                // input and no repaint request, this window would draw ONCE at
                // the moment the run started and then hold that frame until the
                // operator moved the mouse over it — a stopped spinner above a
                // progress line reading `Page 1 of 8`, which is a **pixel-exact
                // rendition of the frozen application he asked us to rule out.**
                //
                // It works today without this line, and that is the trap.
                // `egui::Spinner` calls `ui.request_repaint()` itself because
                // it is animated (`egui-0.35.0/src/widgets/spinner.rs:40`). So the
                // whole visibility of this feature currently rests on a
                // side effect of a decorative widget, and anyone replacing the
                // spinner with a progress bar — a completely reasonable change,
                // and one this dialog will plausibly get — would silently take
                // the live progress with it and leave every unit test green.
                //
                // One redundant call per frame during a run costs nothing.
                // Stating the dependency is the point.
                ui.ctx().request_repaint();
                ui.horizontal(|ui| {
                    // **A BARE `ui.spinner()` IS INVISIBLE IN ALL THREE
                    // PRESETS — A15f, which the widened contrast gate catches,
                    // and it is the funniest defect in the tree because this
                    // control exists to prove the program has not frozen.**
                    //
                    // `egui::Spinner` resolves its own colour from
                    // `visuals.strong_text_color()` (egui-0.35
                    // `egui-0.35.0/src/widgets/spinner.rs:44`) and draws it on
                    // `window_fill`. That accessor **is**
                    // `widgets.active.fg_stroke.color` — the ink meant for the
                    // accent FILL — so on a dialog background the luminance gap
                    // measures **Quiet 17.9 / Airy 5.0 / Dark 29.1** against a
                    // readable floor of 90. Airy is white on white to within
                    // five levels.
                    //
                    // Those are the same three numbers as `DEFECTS.md` D2,
                    // because it is the same pair: a plate colour used against
                    // a background nobody paired it with. The fourth
                    // recurrence.
                    //
                    // And `check-strong-text.sh` structurally cannot see it.
                    // That gate greps for a `.strong()` or a colour named at a
                    // call site; **a bare `ui.spinner()` names no colour at
                    // all**. It took a gate that enumerates what a `Style` will
                    // RENDER rather than what a source file SAYS.
                    //
                    // ⇒ Stated at the call site, in the body-text role, because
                    // that is what this is: a spinner beside "Working…" is
                    // content at the same weight as the sentence next to it,
                    // not an emphasis and not a selection. Measured on
                    // `window_fill`: Quiet 204.0 / Airy 217.1 / Dark 183.1.
                    ui.add(
                        egui::Spinner::new().color(egui_shell::Theme::of(ui.ctx()).palette.text),
                    );
                    ui.label(t::working());
                });
                // **WHAT IT IS DOING** — so the operator can see that the
                // program is doing something and has not frozen on a large
                // document.
                //
                // Drawn only once a page has finished. Before that the tally is
                // all zeros, and "Page 0 of 36 — 0 words" beside a spinner says
                // less than the spinner alone while looking like a stall on the
                // very first page, which is the longest wait in the run.
                let tally = job.tally();
                if tally.attempted > 0 {
                    ui.add_space(4.0);
                    let said = ui.label(t::working_progress(
                        tally.attempted,
                        tally.of,
                        tally.words,
                        tally.chars,
                    ));
                    crate::diag::ui_rect(REGION_PROGRESS, said.rect);
                    // The numbers the label is showing, carried out of the
                    // borrow so they can be traced. See `traced_progress`.
                    progress_seen = Some((tally.attempted, tally.of, tally.words, tally.chars));
                }
                ui.add_space(8.0);
                // STOP FIRST, and the order is the argument. It is the
                // non-destructive one, and this project's standing rule for a
                // row of controls is least-destructive-first — the same reading
                // that orders the Format tab's group. An operator reaching in a
                // hurry meets the button that keeps their work.
                ui.horizontal(|ui| {
                    let stop = ui.button(t::stop_button()).on_hover_text(t::stop_tooltip());
                    crate::diag::ui_rect(REGION_STOP, stop.rect);
                    if stop.clicked() {
                        job.stop();
                    }
                    let cancel = ui
                        .button(t::cancel_button())
                        .on_hover_text(t::cancel_tooltip());
                    crate::diag::ui_rect(REGION_CANCEL, cancel.rect);
                    if cancel.clicked() {
                        job.cancel();
                    }
                });
            }
            Phase::Applied {
                written,
                skipped,
                words,
                stopped_at,
                dictionary,
            } => {
                // **What this says now, and what it no longer has to.**
                //
                // Everything about choosing a destination is gone: the words
                // are in the document, `Ctrl+Z` takes them out and `Ctrl+S`
                // writes them. What is left is the outcome and the one
                // disclosure this surface owes.
                //
                // The engine's per-page report is NOT re-rendered here. It
                // goes through `crate::app::actions`' edit-disclosure channel
                // with every other edit's, which is where the operator already
                // looks — a second, differently-worded copy on this window
                // would be two accounts of one run that could drift.
                ui.label(t::pages_outcome(*written, *skipped));
                // THE CAVEAT, before the reassurance.
                //
                // A stopped run is a success and an incomplete one, and the
                // order these two sentences appear in decides which the
                // operator remembers. *"It is in your document"* directly under
                // *"40 of 200 pages"* reads as a whole job done; the other way
                // round it reads as what it is.
                if let Some((attempted, of)) = stopped_at {
                    ui.add_space(6.0);
                    ui.label(t::stopped_early(*attempted, *of));
                }
                ui.add_space(6.0);
                ui.label(t::applied_to_document());
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);
                // The confidence sentence stays, and stays prominent. It is the
                // one fact a reader who skims must not miss, and it is about
                // the RECOGNITION rather than about the edit — so it does not
                // belong on the disclosure channel with the counts.
                let confidence = confidence_sentence(self.engine);
                let mut disclosures = vec![confidence.to_owned()];
                disclosures.extend(dictionary.as_ref().map(dictionary_sentence));
                Self::answered(ui, &theme, confidence, &disclosures);
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("ocr-applied written={written} skipped={skipped} words={words}")
                });
            }
            // Cancelled draws its own sentence rather than a refusal's.
            // Nothing went wrong, so there is nothing to diagnose — the
            // sentence says what was discarded and the ordinary Recognise
            // button below is the way to start again.
            Phase::Cancelled { attempted } => {
                ui.label(t::cancelled(*attempted));
            }
            Phase::Refused(refusal) => {
                ui.label(sentence(refusal));
            }
        }

        // The progress line's CONTENT, traced on change. See
        // [`Self::traced_progress`] for why a rect alone is not an oracle for
        // *"the user can see it is doing something"*.
        if let Some((attempted, of, words, chars)) = progress_seen
            && attempted != self.traced_progress
        {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("ocr-progress attempted={attempted} of={of} words={words} chars={chars}")
            });
            self.traced_progress = attempted;
        }

        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::close()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// The pre-run state: one button, and the refusals that can be answered
    /// without running anything.
    fn ready(&mut self, ui: &mut egui::Ui, doc: &OpenDoc) {
        if let Some(refusal) = self.preflight() {
            ui.label(sentence(&refusal));
            self.models.show(ui);
            return;
        }
        self.models.show(ui);
        let count = doc.pages.len();
        self.scope_group(ui, count);
        ui.add_space(10.0);

        // **The button is unavailable when the scope names no page** —
        // greyed, not hidden, because this is R9's *temporarily* unavailable
        // case: the operator is mid-way through typing a range and the control
        // will come back on its own. Hiding it would make the dialog jump under
        // their hands as they type.
        let pages = self
            .scope
            .pages(self.page_index, count, &self.range, &self.picked);
        let chosen = self.models.chosen().is_some();
        let run = ui
            .add_enabled(pages.is_some() && chosen, egui::Button::new(t::run()))
            .on_hover_text(t::run_tooltip())
            .on_disabled_hover_text(if chosen {
                t::scope_range_unresolved()
            } else {
                crate::text::ocrmodels::choose_first()
            });
        crate::diag::ui_rect(REGION_RUN, run.rect);
        if run.clicked() {
            self.start(doc);
        }
    }

    /// **Which pages.**
    fn scope_group(&mut self, ui: &mut egui::Ui, count: usize) {
        ui.label(t::scope_heading());
        ui.add_space(4.0);
        let group = ui
            .vertical(|ui| {
                ui.radio_value(&mut self.scope, Scope::All, t::scope_all());
                ui.radio_value(
                    &mut self.scope,
                    Scope::CurrentPage,
                    t::scope_current(self.page_index + 1),
                );
                // **The pages picked in the rail** — `OPERATOR_REQUESTS.md`
                // O79 — drawn only when there ARE some.
                //
                // R9: with an empty rail selection this option has no operand,
                // and a greyed radio reading "Selected pages (0)" would be a
                // control explaining its own uselessness in a window that
                // already has three working answers. The remedy is not on this
                // surface — it is *go and pick some pages* — so there is
                // nothing a hover could usefully say either.
                //
                // Positioned THIRD, between "this page" and a typed range,
                // which is the order of how much the operator had to do to
                // express the operand: nothing, one page, a set they picked, a
                // set they typed.
                if !self.picked.is_empty() {
                    ui.radio_value(
                        &mut self.scope,
                        Scope::Picked,
                        t::scope_picked(self.picked.len()),
                    );
                }
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.scope, Scope::Range, t::scope_range());
                    let field = ui.add(
                        // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
                        // every field in this window: the first press leaves the box, the second
                        // cancels.
                        egui::TextEdit::singleline(&mut self.range)
                            .desired_width(140.0)
                            .hint_text(t::scope_range_hint()),
                    );
                    // Typing IS the choice. See the doc comment.
                    if field.gained_focus() || field.changed() {
                        self.scope = Scope::Range;
                    }
                });
            })
            .response;
        crate::diag::ui_rect(REGION_SCOPE, group.rect);

        ui.add_space(8.0);
        let skip = ui
            .checkbox(&mut self.skip_pages_with_text, t::skip_pages_with_text())
            .on_hover_text(t::skip_pages_with_text_tooltip());
        crate::diag::ui_rect(REGION_SKIP, skip.rect);

        // What the current answer actually resolves to, in pages. Not a
        // rephrasing of the radio — it is the ONLY place a typed range is
        // confirmed to have been understood, and it is off in a status line
        // rather than in the field, per rule 4's disclosure clause.
        //
        // Traced on CHANGE, not every frame. A line per frame for as long as
        // the dialog is open is 90 of the 400 lines in a driven capture, all
        // identical — which is not a diagnostic, it is a haystack. The ink-trail
        // rule cuts both ways: a line nobody can find is the same as a line
        // nobody wrote.
        let resolved = self
            .scope
            .pages(self.page_index, count, &self.range, &self.picked)
            .unwrap_or_default();
        if resolved != self.traced_scope {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "ocr-scope pages={} first={:?} last={:?}",
                    resolved.len(),
                    resolved.first(),
                    resolved.last()
                )
            });
            self.traced_scope = resolved;
        }
    }

    /// Everything that can be refused before a thread is spawned.
    ///
    /// There is no unsaved-edits guard: `EditSession::add_ocr_layer` plans
    /// against the session graph, so a recognised copy carries unsaved edits.
    fn preflight(&self) -> Option<Refusal> {
        if ocr::available().is_empty() {
            return Some(Refusal::EngineAbsent);
        }
        (!self.models.any_runnable()).then(|| Refusal::ModelsMissing(self.models.roots().to_vec()))
    }

    /// Spawn the worker on the chosen model.
    fn start(&mut self, doc: &OpenDoc) {
        let Some((name, engine, folder)) = self
            .models
            .chosen()
            .and_then(|c| Some((c.name.clone(), c.engine?, c.folder.clone())))
        else {
            return;
        };
        self.engine = engine;
        let bundled = ocr::exe_dir().map(|d| d.join(ocr::catalog::BUNDLED_DIR));
        let source = if bundled.is_some_and(|b| folder.starts_with(b)) {
            "bundled" // ui-text-exempt: trace token
        } else {
            "extra-folder" // ui-text-exempt: trace token
        };
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "ocr-started engine={} page={} models={} source={source} model={name}",
                engine.key(),
                self.page_index,
                folder.display(),
            )
        });
        self.phase = Phase::Working(Job::spawn(Request {
            session: std::sync::Arc::clone(&doc.session),
            pages: self
                .scope
                .pages(self.page_index, doc.pages.len(), &self.range, &self.picked)
                .unwrap_or_default(),
            skip_pages_with_text: self.skip_pages_with_text,
            // Through the funnel. See `ocr::Request::extract_options`.
            extract_options: {
                use crate::app::settings::SettingsExt as _;
                doc.settings.extract_options()
            },
            engine,
            model_dir: folder,
        }));
        self.remember = Some(name);
    }

    /// The disclosure block: the confidence statement, then the engine's own
    /// lines.
    fn answered(ui: &mut egui::Ui, theme: &Theme, confidence: &str, disclosures: &[String]) {
        ui.label(t::what_was_inferred());
        ui.add_space(6.0);
        ui.label(confidence);
        ui.add_space(8.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            // The same negative-height trap the About dialog documents: a
            // window shorter than its own header makes `available_height()`
            // minus a reservation go negative, and a negative `max_height` is
            // a scroll area that silently draws nothing rather than an error.
            .max_height((ui.available_height() - FOOTER_RESERVE).max(LIST_FLOOR))
            .show(ui, |ui| {
                for line in disclosures {
                    ui.label(egui::RichText::new(line).color(theme.palette.text_muted));
                    ui.add_space(4.0);
                }
            });
    }
}

/// Height kept clear below the disclosure list for the save and close rows.
const FOOTER_RESERVE: f32 = 96.0;

/// The least height the disclosure list may be given.
const LIST_FLOOR: f32 = 48.0;

/// The sentence naming the character dictionary a run read through.
fn dictionary_sentence(dictionary: &Dictionary) -> String {
    match dictionary {
        Dictionary::File(path) => t::dictionary_file(&path.display().to_string()),
        Dictionary::Embedded => t::dictionary_embedded().to_owned(),
    }
}

/// The operator-visible sentence for a refusal.
fn sentence(refusal: &Refusal) -> String {
    match refusal {
        // Unreachable from the dialog, which turns a cancellation into
        // `Phase::Cancelled` before it ever reaches here — and worded anyway,
        // because a `match` arm that cannot be hit today is one line, while a
        // catch-all that swallowed a real refusal would be silent. Named so a
        // future caller that DOES reach it says something true.
        Refusal::Cancelled { attempted } => t::cancelled(*attempted),
        Refusal::EngineAbsent => t::engine_absent().to_owned(),
        // The paths go to the catalog as a LIST, not as a pre-joined string.
        //
        // The separator between them is copy: it is punctuation an operator
        // reads, and `tools/gates/check-ui-strings.sh` caught a `", "` sitting
        // here, correctly. Joining inside `text::ocr::models_missing` puts the
        // whole sentence — wording, punctuation and all — in the one file that
        // is allowed to decide how pdfcer phrases things, which is what rule R1
        // is actually asking for rather than a technicality about where a comma
        // lives.
        Refusal::ModelsMissing(searched) => {
            let paths: Vec<String> = searched.iter().map(|p| p.display().to_string()).collect();
            t::models_missing(&paths)
        }
        Refusal::NothingRecognised => t::nothing_recognised().to_owned(),
        Refusal::AlreadyHasText => t::already_has_text().to_owned(),
        // A page index past the end and a page with no area are both
        // structural impossibilities from a dialog opened on a page the canvas
        // is showing. They are worded through the engine's own channel rather
        // than given catalog entries of their own: inventing operator copy for
        // a state nothing can reach is how a catalog fills with sentences
        // nobody has ever seen.
        Refusal::NoSuchPage(i) => t::failed(&(i + 1).to_string()),
        Refusal::EmptyPage => t::failed(&(0).to_string()),
        Refusal::Engine(reason) => t::failed(reason),
    }
}

/// The name to suggest for the recognised copy.
#[must_use]
pub fn suggested_path(source: &Path) -> PathBuf {
    let stem = source.file_stem().map_or_else(
        || String::from("document"),
        |s| s.to_string_lossy().into_owned(),
    ); // ui-text-exempt: a filename fallback, not operator copy
    let name = format!("{stem}{}.pdf", t::suggested_suffix());
    source
        .parent()
        .map_or_else(|| PathBuf::from(&name), |dir| dir.join(&name))
}

/// The sentence saying what the engine's scores are, or that it has none.
fn confidence_sentence(engine: EngineId) -> &'static str {
    if engine.reports_confidence() {
        t::scored_confidence()
    } else {
        t::no_confidence()
    }
}

/// Open the dialog for the document in `status`, if there is one.
pub(super) fn open_for(
    status: &Status,
    picked: Vec<usize>,
    prefs: &crate::app::prefs::Prefs,
) -> Option<OcrDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(OcrDialog::open(doc, picked, prefs))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The suggested name is never the file that was opened.**
    #[test]
    fn the_suggested_name_is_never_the_source_file() {
        let source = PathBuf::from("D:\\scans\\survey.pdf");
        let suggested = suggested_path(&source);
        assert_ne!(suggested, source);
        assert_eq!(suggested, PathBuf::from("D:\\scans\\survey-recognised.pdf"));
        assert_eq!(
            suggested.parent(),
            source.parent(),
            "the copy should land beside the original, where the operator will look for it"
        );
    }

    /// A capitalised extension still produces a `.pdf`.
    #[test]
    fn the_suggested_name_always_ends_in_pdf() {
        for name in ["scan.PDF", "scan.pdf", "scan"] {
            let suggested = suggested_path(Path::new(name));
            assert!(
                suggested.to_string_lossy().ends_with(".pdf"),
                "{name} suggested {suggested:?}"
            );
        }
    }

    /// A source with no parent directory still yields a usable name.
    #[test]
    fn a_bare_filename_still_produces_a_suggestion() {
        assert_eq!(
            suggested_path(Path::new("scan.pdf")),
            PathBuf::from("scan-recognised.pdf")
        );
    }

    /// **Every refusal produces a different sentence.**
    #[test]
    fn each_named_refusal_says_something_different() {
        let all = [
            Refusal::EngineAbsent,
            Refusal::ModelsMissing(vec![PathBuf::from("C:\\a"), PathBuf::from("C:\\b")]),
            Refusal::NothingRecognised,
            Refusal::Engine("the runtime rejected the model".to_owned()),
        ];
        let mut seen: Vec<String> = Vec::new();
        for refusal in &all {
            let s = sentence(refusal);
            assert!(!s.is_empty(), "{refusal:?} produced no sentence");
            assert!(
                !seen.contains(&s),
                "{refusal:?} repeats a sentence another refusal already uses"
            );
            seen.push(s);
        }
    }

    /// The searched paths survive into the message an operator reads.
    #[test]
    fn a_missing_model_directory_names_every_place_that_was_tried() {
        let s = sentence(&Refusal::ModelsMissing(vec![
            PathBuf::from("C:\\app\\models\\ocrs"),
            PathBuf::from("C:\\users\\x\\models\\ocrs"),
        ]));
        assert!(s.contains("C:\\app\\models\\ocrs"));
        assert!(s.contains("C:\\users\\x\\models\\ocrs"));
    }

    /// A dialog opened with nothing loaded is not built at all.
    #[test]
    fn no_document_means_no_dialog() {
        assert!(
            open_for(
                &Status::Empty,
                Vec::new(),
                &crate::app::prefs::Prefs::default()
            )
            .is_none()
        );
    }
}

#[cfg(test)]
mod scope_tests {
    use super::*;

    /// All pages means all of them, in order, zero-based.
    #[test]
    fn all_pages_is_every_page_in_order() {
        assert_eq!(
            Scope::All.pages(3, 5, "", &[]),
            Some(vec![0, 1, 2, 3, 4]),
            "the current page has no bearing on All"
        );
    }

    /// **This page only means the page the dialog OPENED on.**
    #[test]
    fn this_page_only_is_the_captured_page() {
        assert_eq!(Scope::CurrentPage.pages(2, 5, "", &[]), Some(vec![2]));
    }

    /// **The rail's picked pages are the operand** —
    /// `OPERATOR_REQUESTS.md` O79.
    #[test]
    fn the_picked_pages_are_the_pages_picked() {
        assert_eq!(
            Scope::Picked.pages(0, 36, "", &[3, 7, 11, 12]),
            Some(vec![3, 7, 11, 12]),
            "the rail's selection is the operand, verbatim"
        );
        assert_eq!(
            Scope::Picked.pages(0, 36, "1-4", &[9]),
            Some(vec![9]),
            "a typed range in the field has no bearing on the picked scope"
        );
    }

    /// **A picked page the document no longer has is dropped**, and an
    /// empty result resolves to nothing.
    #[test]
    fn a_picked_page_the_document_lost_is_dropped() {
        assert_eq!(
            Scope::Picked.pages(0, 5, "", &[1, 4, 9, 20]),
            Some(vec![1, 4]),
            "indices past the end are dropped, the rest stand"
        );
        assert_eq!(
            Scope::Picked.pages(0, 5, "", &[9, 20]),
            None,
            "nothing left is nothing to run, which greys the button by the existing path"
        );
        assert_eq!(
            Scope::Picked.pages(0, 5, "", &[]),
            None,
            "an empty rail selection names no page"
        );
    }

    /// A scope naming no page resolves to nothing, which the dialog renders as
    /// an unavailable button rather than as an error.
    #[test]
    fn a_scope_that_names_no_page_resolves_to_nothing() {
        assert_eq!(Scope::Range.pages(0, 5, "", &[]), None, "an empty range");
        assert_eq!(Scope::Range.pages(0, 5, "  ", &[]), None, "whitespace");
        assert_eq!(Scope::Range.pages(0, 5, "9-12", &[]), None, "past the end");
        assert_eq!(
            Scope::All.pages(0, 0, "", &[]),
            None,
            "a document with no pages"
        );
        assert_eq!(
            Scope::CurrentPage.pages(7, 5, "", &[]),
            None,
            "a captured index the document no longer has"
        );
    }

    /// **The range field speaks the PRINT dialog's dialect, not its own.**
    #[test]
    fn the_range_is_parsed_by_the_print_dialogs_parser() {
        for input in ["1-3", "2,4", "1-2, 5", "3"] {
            assert_eq!(
                Scope::Range.pages(0, 5, input, &[]),
                crate::dialogs::print::tabs::parse_page_range(input, 5).filter(|p| !p.is_empty()),
                "the two must agree on {input:?}"
            );
        }
    }
}
