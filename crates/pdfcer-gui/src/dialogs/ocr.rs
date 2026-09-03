//! # `dialogs::ocr` — the Recognise-text transaction
//!
//! One dialog, three states, and a shape that is chosen rather than
//! conventional. It is the surface for `file.ocr`, and it is also the
//! **enforcement point for two rules** that would otherwise have nowhere to
//! live in this build.
//!
//! ## The three states
//!
//! | state | what the operator sees | what exists |
//! |---|---|---|
//! | **ready** | what OCR does to the page, and one button that starts it | nothing |
//! | **working** | *Recognising…* | a thread |
//! | **answered** | the disclosure, then a Save-as button — or a named refusal | bytes, in memory |
//!
//! ## ★ Why the recognition is disclosed BEFORE it is written, not after
//!
//! This is the whole reason the dialog has a third state instead of running
//! OCR and immediately opening a file picker.
//!
//! Project rule 4 is *"fuzzy, never sneaky"*, and `pdfcer-core`'s own OCR
//! header sharpens it for exactly this feature: **every word an OCR layer
//! contains is a guess**, and this engine reports no confidence for any of
//! them. A surface that recognised a page and dropped the finished file in
//! front of the operator would be technically disclosive — the report would be
//! *somewhere* — while being, in practice, a program that silently inserted
//! several hundred unreviewed inferences into a document. `DEFECTS.md` and
//! `HANDOFF.md` both record that this project's characteristic failure is a
//! surface that is *correct* and *unreadable at the moment it matters*.
//!
//! So the order is: recognise, **show what was inferred**, and only then offer
//! to write it. The operator reads the disclosure while holding the one thing
//! that gives it force — the ability to not save. That is not a nicety; it is
//! the difference between a disclosure and a receipt.
//!
//! ## ★ Why the write is a Save-as, in every mode
//!
//! The operator's standing rule, 2026-08-14: *"Read may produce a new
//! document; it may not modify this one"*, with the enforcement at the **save**
//! rather than at the operation.
//!
//! `HANDOFF.md` §3 records that rule as currently **vacuous**, on the grounds
//! that `file.save_copy` never overwrites the original. That was true and it
//! understated the position: when this dialog was built, `file.save_copy` had
//! **no dispatch arm at all** (`app/dispatch.rs` fell through to
//! `command-unimplemented`), so **this shell could not write a file of any
//! kind.**
//!
//! ★ **`file.save_copy` was wired on 2026-08-14** and the rule stays vacuous
//! for the reason it always was: that command asks for a destination too, and
//! `crate::app::save::suggested_path` guarantees the *suggestion* is never the
//! file that was opened, exactly as [`suggested_path`] below does here. The two
//! surfaces now share one picker, `crate::app::files::pick_save_path`, and the
//! only thing that differs between them is the dialog's title.
//!
//! This dialog was nevertheless the first write to disk pdfcer-gui performed, and
//! therefore the first place the rule could bite. It bites here in the only way that is
//! honest: the destination is a path the operator names, so the rule holds in
//! Read **and** in Edit **and** in Review by construction rather than by a mode
//! check. Nothing here consults the mode, and nothing here should — the rule is
//! about what a save may overwrite, not about who is asking.
//!
//! What is deliberately **not** done: no second save command, no in-place path,
//! no `Save`-labelled control anywhere. The day in-place `Save` lands it will
//! need its own Read-mode gate, and that gate belongs beside it rather than
//! being invented here in advance against a command that does not exist.
//!
//! ## ★ Why OCR is available in Read, with no capability flag
//!
//! `app::modes::capability` governs **gestures** — what a press on the canvas
//! means. OCR is not a gesture; it is a command with a dialog, and it changes
//! no document that is open. Adding a capability flag for it would put a rule
//! about *saving* into the machinery that decides what a drag does, where the
//! next reader would neither look for it nor believe it.
//!
//! Read is therefore offered OCR exactly as Edit is, and that is the operator's
//! instruction rather than an omission.
//!
//! ## Why this dialog does not push an `Action`
//!
//! [`super`]'s rule: a dialog uses the action funnel when it edits **this**
//! document, and this one never does. The recognised bytes are a *new*
//! document; the open one is untouched, its `edit_epoch` does not move, and
//! there is nothing to order against or to undo. What the funnel's reasoning
//! does still demand is that irreversible work not happen part-way through a
//! layout pass — and it does not: the button sets a flag, and the file is
//! written after the window's closure returns.
//!
//! ## What is document-scoped about it
//!
//! Everything. A recognition is of *this page* of *this file*, so
//! [`super::DialogsState`] holds it in the document-scoped group and closing
//! the document closes it. A finished-but-unsaved recognition is discarded
//! with it, which is the right answer: writing it afterwards would produce a
//! file derived from a document the operator has already put away.

use std::path::{Path, PathBuf};

use egui_shell::theme::Theme;

use crate::app::state::{OpenDoc, Status};
use crate::ocr::{self, Job, Refusal, Request};
use crate::text::ocr as t;

// ---------------------------------------------------------------------------
// Named regions
//
// `crate::diag::ui_rect` publishes where a control actually got drawn, so
// `tools/ui-verify` can aim a real click at it. These names are matched
// LITERALLY by `tools/ui-verify/src/checks/ocr.rs`, so renaming one silently
// un-aims the check that measures it.
//
// ★ Why a dialog needs them at all, when the ribbon's controls get theirs for
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
///
/// Declared **only while it exists**, which is itself the assertion a harness
/// wants: this control is drawn if and only if the dialog is in its ready
/// state with a resolvable page scope, so its absence from the trace is
/// evidence that the run could not have been started rather than that the
/// harness missed it.
///
/// ★ There is no region for a save control any more, and its removal is the
/// change: recognition became an edit to the open session on 2026-08-27, so
/// there is no transaction to complete and nothing for a second button to do.
const REGION_RUN: &str = "ocr-run"; // ui-text-exempt: trace region name, never displayed
/// The live progress line, drawn once a page has finished.
///
/// ★ Published so a driven check can assert the operator can SEE the run
/// moving. The whole request was *"so that the user can see that it is doing
/// something and hasn't frozen"*, and a feature whose entire purpose is to be
/// visible needs an oracle that is about visibility.
const REGION_PROGRESS: &str = "ocr-progress"; // ui-text-exempt: trace region name
/// The control that finishes the page in hand and keeps everything.
const REGION_STOP: &str = "ocr-stop"; // ui-text-exempt: trace region name
/// The control that abandons the run.
const REGION_CANCEL: &str = "ocr-cancel"; // ui-text-exempt: trace region name

/// Where one Recognise-text transaction has got to.
///
/// A state machine rather than three `Option`s, because the states are
/// mutually exclusive and an `Option` triple has five nonsense combinations
/// that would all compile.
#[derive(Debug, Default)]
enum Phase {
    /// Nothing has been asked for yet.
    #[default]
    Ready,
    /// A thread is recognising.
    Working(Job),
    /// ★★ Recognition finished and **the words are in the open document.**
    ///
    /// This used to mean *"a complete PDF is sitting in memory and the operator
    /// must now choose a file for it"*, and it carried the bytes. Since the
    /// engine's `EditSession::add_ocr_layer` (Pass 135.0, 2026-08-27) the layer
    /// goes straight into the session as one undoable edit, so what is left to
    /// report is the outcome — not a transaction to complete.
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
        /// ★★★ The whole reason this field exists: a stopped run is a success
        /// with a caveat, and the caveat must be SAID. Without it, somebody who
        /// ended a 200-page recognition at page 40 is left believing the
        /// document is done — and finds out months later, searching for a word
        /// on page 150 that is not in the layer.
        stopped_at: Option<(usize, usize)>,
    },
    /// **The operator pressed Cancel.** Nothing was kept and nothing written.
    ///
    /// ★ Its own phase rather than a `Refused`, because it is not a refusal:
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
    /// ★ **Captured, not read per frame**, and that is a correctness
    /// requirement rather than an optimisation. The operator can page the
    /// document while the dialog is open; a `Save` that read the *current*
    /// page index would label bytes recognised from page 3 as belonging to
    /// whatever page they had scrolled to. The recognition is of one page and
    /// the dialog remembers which.
    page_index: usize,
    /// ★★★ **Which pages to recognise.**
    ///
    /// The operator, 2026-08-26: *"how do I OCR more than one page? Why does
    /// the tool stop at one? […] Where is the option to select more than one
    /// page?"*
    ///
    /// There was none. The dialog recognised [`Self::page_index`] and nothing
    /// else, and no engine limitation required that — `add_ocr_layer`'s output
    /// is a complete PDF that can be fed back in, so pages chain. Measured
    /// before this was built, because a wrong answer would have corrupted a
    /// file.
    scope: Scope,
    /// ★★★ **The rail's page selection, captured when the dialog opened** —
    /// `OPERATOR_REQUESTS.md` O79.
    ///
    /// Zero-based, ascending, and possibly empty — empty is a defined answer
    /// meaning *nothing is picked*, in which case [`Scope::Picked`] is not
    /// offered at all (R9: an option with no operand renders nothing).
    ///
    /// # ★★ Captured rather than read live, and this is the decision worth
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
    /// The page list the last `ocr-scope` line reported.
    ///
    /// Kept so the trace fires on a change rather than on a frame. See
    /// [`Self::scope_group`].
    traced_scope: Vec<usize>,
    /// The `attempted` count the last `ocr-progress` line reported.
    ///
    /// ★★★ **Why the numbers are traced at all, when the rect already is.**
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
    /// ★ Traced on **change**, for the reason [`Self::traced_scope`] already
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
///
/// Four options is what the surveyed tools converge on — Acrobat, ABBYY, Foxit
/// and PDF-XChange all offer all / current / a range in some wording — and the
/// order below is theirs: the broadest first, because it is both the default
/// and the one most runs want.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Scope {
    /// Every page of the document. The default.
    All,
    /// Only the page the operator was looking at when the dialog opened.
    ///
    /// ★ **When it opened**, not now — the operator can page the document while
    /// this window is up, and a run that read the *current* index would
    /// recognise a page they were no longer thinking about. That capture is the
    /// same argument [`OcrDialog::page_index`] already carries.
    CurrentPage,
    /// ★★★ **The pages picked in the thumbnail rail** —
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
    /// # ★ Captured at OPEN, like [`Self::CurrentPage`], and for the same reason
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
    ///
    /// `None` when the scope cannot be resolved — an unparseable or empty range
    /// — which the dialog renders as a disabled Recognise button rather than as
    /// an error, because a half-typed range is a normal state of a text field
    /// and not a mistake to be reported.
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
            // ★ Filtered against the page count rather than trusted (O79). The
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
    ///
    /// Nothing is recognised yet: opening the dialog is free, and the several
    /// seconds of work start on a press the operator makes after reading what
    /// the operation does. A dialog that started recognising on open would
    /// spend that time before the operator had decided they wanted it.
    #[must_use]
    pub(super) fn open(doc: &OpenDoc, picked: Vec<usize>) -> Self {
        Self {
            page_index: doc.view.page_index,
            // ★ The rail's selection, captured once — see `Self::picked` for
            // why it is a snapshot rather than a live read (O79).
            picked,
            // ★ **All pages by default**, which is what every surveyed OCR tool
            // defaults to and what the operator was asking for. The old
            // behaviour — this page only — is still one click away and is the
            // right answer when he is checking one sheet, but it is the
            // unusual want and it should not be the assumption.
            scope: Scope::All,
            range: String::new(),
            skip_pages_with_text: true,
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
    ) -> bool {
        self.poll_worker(actions);

        // ★ ITS OWN OS WINDOW as of 2026-08-21. OCR is the longest-running
        // thing in this program — a job an operator starts and then goes back
        // to work while it runs — and a progress window locked inside the
        // application frame is a window that has to be closed to keep working.
        //
        // ★ The dialog region is published from INSIDE the callback now. It
        // used to come from the `egui::Window` response rect, which no longer
        // exists; `ui.max_rect()` is the same rectangle in the coordinates the
        // harness converts, and `dialogs::host` tags it with this viewport.
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

        open && !std::mem::take(&mut self.close_requested)
    }

    /// Move a finished job into the phase that describes its answer.
    ///
    /// Separate from [`Self::body`] so the transition happens once per frame
    /// regardless of what the window drew, and so that a dialog scrolled out
    /// of view still notices its own worker finishing.
    fn poll_worker(&mut self, actions: &mut Vec<crate::app::actions::Action>) {
        let Phase::Working(job) = &mut self.phase else {
            return;
        };
        let Some(outcome) = job.poll() else {
            return;
        };
        // ★★★ Three endings, and the shell must keep them apart — 2026-09-01.
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
                        // ★ `recognised=` beside the page counts, on
                        // `HANDOFF.md` §2's own advice about the ink trail: a
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
                    // ★ Carried into the outcome so the sentence the operator
                    // reads afterwards can say the run ended early. A partial
                    // layer reported as a whole one is the failure this whole
                    // pair of buttons has to avoid.
                    stopped_at,
                };
                // ★★ **The edit is raised here, in the poll, rather than in the
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

        // ★ Filled by the `Working` arm and consumed after the match, rather
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
                // ★★★ **Ask for the next frame explicitly, and do not rely on
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
                // ★ It works today without this line, and that is the trap.
                // `egui::Spinner` calls `ui.request_repaint()` itself because
                // it is animated (egui 0.35, `widgets/spinner.rs:40`). So the
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
                    ui.spinner();
                    ui.label(t::working());
                });
                // ★★★ **WHAT IT IS DOING** — the operator's own ask, 2026-09-01:
                // *"so that the user can see that it is doing something and
                // hasn't frozen on large documents."*
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
                // ★★ STOP FIRST, and the order is the argument. It is the
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
            } => {
                // ★★★ **What this says now, and what it no longer has to.**
                //
                // Everything about choosing a destination is gone: the words
                // are in the document, `Ctrl+Z` takes them out and `Ctrl+S`
                // writes them. What is left is the outcome and the one
                // disclosure this surface owes.
                //
                // ★ The engine's per-page report is NOT re-rendered here. It
                // goes through `crate::app::actions`' edit-disclosure channel
                // with every other edit's, which is where the operator already
                // looks — a second, differently-worded copy on this window
                // would be two accounts of one run that could drift.
                ui.label(t::pages_outcome(*written, *skipped));
                // ★★★ THE CAVEAT, before the reassurance — 2026-09-01.
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
                Self::answered(ui, &theme, &[t::no_confidence().to_owned()]);
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!("ocr-applied written={written} skipped={skipped} words={words}")
                });
            }
            // ★ Cancelled draws its own sentence rather than a refusal's.
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

        // ★★ The progress line's CONTENT, traced on change. See
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
    ///
    /// ★ The order of the checks is the order of the questions, and it is not
    /// arbitrary. *Can this build look at all* comes before *are the files
    /// there to look with*, because asking them the other way round would
    /// report a missing model directory in a build that has no recogniser to
    /// use it — a true statement and the wrong diagnosis.
    fn ready(&mut self, ui: &mut egui::Ui, doc: &OpenDoc) {
        if let Some(refusal) = Self::preflight(doc) {
            ui.label(sentence(&refusal));
            return;
        }
        let count = doc.pages.len();
        self.scope_group(ui, count);
        ui.add_space(10.0);

        // ★★ **The button is unavailable when the scope names no page** —
        // greyed, not hidden, because this is R9's *temporarily* unavailable
        // case: the operator is mid-way through typing a range and the control
        // will come back on its own. Hiding it would make the dialog jump under
        // their hands as they type.
        let pages = self
            .scope
            .pages(self.page_index, count, &self.range, &self.picked);
        let run = ui
            .add_enabled(pages.is_some(), egui::Button::new(t::run()))
            .on_hover_text(t::run_tooltip())
            .on_disabled_hover_text(t::scope_range_unresolved());
        crate::diag::ui_rect(REGION_RUN, run.rect);
        if run.clicked() {
            self.start(doc);
        }
    }

    /// ★★★ **Which pages — the control the operator said was missing.**
    ///
    /// > *"Where is the option to select more than one page? How did we end up
    /// > with the most useless and un-userfriendly of options for the OCR?"*
    /// > — 2026-08-26
    ///
    /// # Why radios and not a dropdown
    ///
    /// Three choices, all short, all mutually exclusive, and one of them opens
    /// a text field. A dropdown would hide two of the three behind a click and
    /// would have nowhere sensible to put the range field; radios show the
    /// whole decision at once, which is what every surveyed recogniser does
    /// with the same three options.
    ///
    /// # Why the range field is always visible
    ///
    /// Rather than appearing when **Pages** is chosen. A field that appears
    /// changes the dialog's height mid-interaction, and the operator reaching
    /// for the radio has to then find where everything moved to. It is
    /// disabled instead when another scope is active — and clicking it selects
    /// [`Scope::Range`], because typing into a range field is an unambiguous
    /// statement of intent and making them click the radio first would be
    /// pedantry.
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
                // ★★★ **The pages picked in the rail** — `OPERATOR_REQUESTS.md`
                // O79 — drawn only when there ARE some.
                //
                // R9: with an empty rail selection this option has no operand,
                // and a greyed radio reading "Selected pages (0)" would be a
                // control explaining its own uselessness in a window that
                // already has three working answers. The remedy is not on this
                // surface — it is *go and pick some pages* — so there is
                // nothing a hover could usefully say either.
                //
                // ★ Positioned THIRD, between "this page" and a typed range,
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
        // ★ Traced on CHANGE, not every frame. The first version emitted a line
        // per frame for as long as the dialog was open — 90 of the 400 lines in
        // a driven capture, all identical — which is not a diagnostic, it is a
        // haystack. `HANDOFF.md` §2's rule about the ink trail cuts both ways:
        // a line nobody can find is the same as a line nobody wrote.
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
    /// Returns `None` when recognition may proceed. Pulled out of [`Self::ready`]
    /// so that the decision is a pure function of the document and is therefore
    /// reachable from a test — the button and the window are not.
    fn preflight(_doc: &OpenDoc) -> Option<Refusal> {
        if !ocr::engine_compiled_in() {
            return Some(Refusal::EngineAbsent);
        }
        // ★★★ Refused, not disclosed — and the comparison is against
        // `saved_epoch`, not against zero. 2026-08-26.
        //
        // `add_ocr_layer` reads the session's **base** revision, so a recognised
        // copy taken over unsaved edits would silently omit them. That refusal
        // is right and stays.
        //
        // What was wrong was the question. `edit_epoch != 0` asks *has anything
        // ever been edited*, and `edit_epoch` never comes back down — so OCR
        // died for the rest of the session the first time anyone edited and
        // saved anything, and said **"unsaved edits"** on the way out, which by
        // then was false. The operator met it and asked *"how did we end up with
        // the most useless and un-userfriendly of options for the OCR?"*
        //
        // ★★★ **AND THE GUARD IS GONE ENTIRELY, as of 2026-08-27.**
        //
        // It was correct and it was unfixable. `ocr::layer::add_ocr_layer` read
        // the document's **base** revision, so a recognised copy taken after an
        // edit silently omitted that edit — and silent omission is worse than a
        // refusal, so refusing was right. But a session never becomes clean
        // again, not even after a successful save, so this killed OCR for the
        // rest of the session the first time anything was edited and told the
        // operator something inaccurate on the way out.
        //
        // No guard could have fixed that, because the guard was not the
        // problem. `EditSession::add_ocr_layer` (engine Pass 135.0) plans
        // against the **session graph**, so the divergence the guard existed to
        // police no longer exists, and the guard has nothing left to guard.
        //
        // What remains here is the pair that is still real: a build with no
        // recogniser, and a build that cannot find its models.
        match ocr::resolve_models(ocr::exe_dir().as_deref(), user_data_dir().as_deref()) {
            Ok(_) => None,
            Err(e) => Some(Refusal::ModelsMissing(e.searched)),
        }
    }

    /// Spawn the worker.
    fn start(&mut self, doc: &OpenDoc) {
        let Ok(source) = ocr::resolve_models(ocr::exe_dir().as_deref(), user_data_dir().as_deref())
        else {
            // Unreachable behind `preflight`, and answered rather than
            // ignored: a button that did nothing would be indistinguishable
            // from a recognition that produced no words.
            self.phase = Phase::Refused(Refusal::ModelsMissing(Vec::new()));
            return;
        };
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed.
                "ocr-started page={} models={} source={}",
                self.page_index,
                source.path().display(),
                source.token()
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
            model_dir: source.path().to_path_buf(),
        }));
    }

    /// The disclosure block: the confidence statement, then the engine's own
    /// lines.
    ///
    /// ★ **The confidence sentence is drawn first and separately, above the
    /// list.** `OcrLayerReport::disclosures()` already contains a sentence
    /// making the same point, and this is deliberate duplication rather than an
    /// oversight: the engine's version sits fourth in a list of counts, and the
    /// one fact that must survive a skim is that **nothing here was scored**.
    /// A reader who takes in one line takes in that one.
    ///
    /// Drawn in the plain text role, never `.strong()` — `DEFECTS.md` D11
    /// records that role as unusable in this theme, and a named palette exists
    /// so a surface written later does not rediscover it.
    fn answered(ui: &mut egui::Ui, theme: &Theme, disclosures: &[String]) {
        ui.label(t::what_was_inferred());
        ui.add_space(6.0);
        ui.label(t::no_confidence());
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
///
/// See [`OcrDialog::answered`] — without a floor, a small window produces a
/// list that draws nothing at all and looks like a recognition that disclosed
/// nothing, which is the exact opposite of what this dialog is for.
const LIST_FLOOR: f32 = 48.0;

/// The operator-visible sentence for a refusal.
///
/// One place, so the dialog cannot word a refusal differently from anywhere
/// else that grows a need for it, and so `text::ocr`'s catalog is the only
/// thing that has to be read to know what pdfcer says when OCR declines.
fn sentence(refusal: &Refusal) -> String {
    match refusal {
        // ★★ Unreachable from the dialog, which turns a cancellation into
        // `Phase::Cancelled` before it ever reaches here — and worded anyway,
        // because a `match` arm that cannot be hit today is one line, while a
        // catch-all that swallowed a real refusal would be silent. Named so a
        // future caller that DOES reach it says something true.
        Refusal::Cancelled { attempted } => t::cancelled(*attempted),
        Refusal::EngineAbsent => t::engine_absent().to_owned(),
        // ★ The paths go to the catalog as a LIST, not as a pre-joined string.
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
///
/// ★ **Never the file that was opened.** The suffix is what makes the default
/// answer a new document, so an operator who accepts the suggestion without
/// reading it cannot overwrite their scan. That is the standing rule expressed
/// as a default rather than as a warning — a warning is something to click
/// past.
///
/// The extension is forced to `.pdf` rather than preserved: the bytes are a
/// PDF whatever the source was called, and a recognised copy of `scan.PDF`
/// landing as `scan-recognised.PDF` would be correct but is one more way for a
/// tool downstream to disagree about case.
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

/// Where a durable per-user model directory would live.
///
/// `None` today, and that is a statement rather than a stub: this shell has no
/// user-data directory of its own — `app::persistence` writes its layout beside
/// the executable — so there is no second place to look and reporting one would
/// name a path in a "searched here" list that was never searched.
///
/// It exists as a function because `ocr::resolve_models` takes the parameter
/// and the day a user-data location appears there is one call site to change
/// rather than three.
fn user_data_dir() -> Option<PathBuf> {
    None
}

/// Open the dialog for the document in `status`, if there is one.
///
/// The dispatch target for `file.ocr`. Lives here rather than in
/// [`super::DialogsState`] only because it needs [`OcrDialog::open`]'s private
/// constructor; the guard it applies is the one `open_print` documents — the
/// ribbon control is gated on `doc.pages`, a chord bound to the same id is not,
/// and both are fixed by refusing here at the one place the dialog is built.
pub(super) fn open_for(status: &Status, picked: Vec<usize>) -> Option<OcrDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(OcrDialog::open(doc, picked))
}

#[cfg(test)]
mod tests {
    /// ★★★ **An edited, unsaved document may now be recognised** — the last
    /// act of the operator's OCR complaint.
    ///
    /// # What this used to assert, and why the reversal is not a relaxation
    ///
    /// It asserted the opposite: that a document with unsaved edits was
    /// refused. That was correct at the time and for a real reason —
    /// `ocr::layer::add_ocr_layer` read the document's **base** revision, so a
    /// recognised copy taken after an edit silently omitted it.
    ///
    /// But the base never becomes current, not even after a save, so the
    /// refusal was permanent from the first edit onward and the operator was
    /// stuck in it. `EditSession::add_ocr_layer` (engine Pass 135.0) plans
    /// against the **session graph** instead, which removes the divergence
    /// rather than policing it.
    ///
    /// ★ So this test now pins the *absence* of the guard, and it is worth
    /// having as a test rather than as a deletion: the trap was re-introduced
    /// once already, in a different spelling, and a named assertion is what
    /// makes a third spelling fail rather than ship.
    #[test]
    fn an_unsaved_edit_no_longer_refuses_recognition() {
        let mut doc = crate::app::state::open_fixture(crate::app::state::FOUR_PAGES);
        // Whatever the verdict is on this machine — the models may genuinely be
        // absent — it must not depend on the epochs. That is the whole property:
        // recognition is an edit now, and an edit does not care what else is
        // unsaved.
        let untouched = OcrDialog::preflight(&doc);

        doc.edit_epoch = 7;
        assert_eq!(
            OcrDialog::preflight(&doc),
            untouched,
            "an unsaved edit must not change the answer"
        );

        doc.saved_epoch = 7;
        assert_eq!(OcrDialog::preflight(&doc), untouched, "nor must a save");

        doc.edit_epoch = 8;
        assert_eq!(
            OcrDialog::preflight(&doc),
            untouched,
            "nor an edit after a save — the state the operator was stuck in"
        );
    }

    use super::*;

    /// ★ **The suggested name is never the file that was opened.**
    ///
    /// The standing rule as a default. An operator who accepts the suggestion
    /// without reading it must not overwrite their scan, and this is the
    /// assertion that says so — the label and the tooltip say it in words, and
    /// words are not a mechanism.
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

    /// ★ **Every refusal produces a different sentence.**
    ///
    /// The property the whole `Refusal` enum exists for: `pdfcer-core`'s error
    /// types refuse by name because "OCR failed" is unactionable, and a shell
    /// that mapped four named causes onto one sentence would throw that away at
    /// the last step.
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
    ///
    /// `models::ModelsNotFound` carries them precisely so the operator learns
    /// where to put the files; dropping them at the display boundary would
    /// undo that in the last inch.
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
        assert!(open_for(&Status::Empty, Vec::new()).is_none());
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

    /// ★ **This page only means the page the dialog OPENED on.**
    ///
    /// The operator can page the document while the window is up. A scope that
    /// resolved against the live index would recognise a page they had moved
    /// away from — which is why `page_index` is captured at `open` and threaded
    /// here rather than read from the document.
    #[test]
    fn this_page_only_is_the_captured_page() {
        assert_eq!(Scope::CurrentPage.pages(2, 5, "", &[]), Some(vec![2]));
    }

    /// ★★★ **The rail's picked pages are the operand** —
    /// `OPERATOR_REQUESTS.md` O79.
    ///
    /// The operator: *"the pages I have selected in the thumbnails."*
    ///
    /// Asserted as passed through **unchanged and in order**, because the two
    /// tempting mistakes are both silent: re-deriving the set here would put a
    /// second page selection in the program, and sorting or de-duplicating it
    /// would hide a caller handing over something malformed.
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

    /// ★★ **A picked page the document no longer has is dropped**, and an
    /// empty result resolves to nothing.
    ///
    /// The selection is captured when the dialog opens and the document can be
    /// edited underneath it — deleting a sheet from the rail while this window
    /// is up would otherwise hand the engine an index past the end. Filtering
    /// rather than refusing, because the pages that survive are still the ones
    /// he asked for; refusing the whole run would punish him for an edit he
    /// made deliberately.
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

    /// ★★ **The range field speaks the PRINT dialog's dialect, not its own.**
    ///
    /// Two page-range parsers in one program would accept different things on
    /// two surfaces and the operator would have to learn which one they were
    /// talking to. This asserts they are the same parser rather than merely
    /// similar: the expectations below are `parse_page_range`'s own, taken
    /// from its behaviour rather than restated.
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
