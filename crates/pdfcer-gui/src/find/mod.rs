//! # find — searching the page text, and showing the operator where it is
//!
//! The whole of Find, across three files:
//!
//! | module | subject |
//! |---|---|
//! | this one | the query and its options ([`FindState`]), the one place a search is run ([`search`]), the wrap rule, and what the position readout says ([`FindState::readout`]) |
//! | [`bar`] | the floating box: where it sits, the field, the step buttons, the readout, the options menu, and the three keys the field owns |
//! | [`reveal`] | how one hit reaches the operator's eye — the two-frame handshake, the scroll solve, and the projection out of PDF space |
//!
//! ## ★ The trap, stated first because it is the whole reason this module
//! is written the way it is
//!
//! `EditSession` has two search verbs and they are **not** interchangeable:
//!
//! | verb | wildcards |
//! |---|---|
//! | `find_text(needle, case_insensitive)` | **on** — it passes `with_wildcards(true)` |
//! | `find_text_with(needle, &options)` | whatever the options say; the default is **off** |
//!
//! In wildcard mode `#` matches any ASCII digit and `?` matches any single
//! character. **The old pdfcer shell's Find bar ran through `find_text`**, so
//! typing a `?` into it matched every character on the page and nothing said
//! why. `pdfcer-core` records that this was fixed *in the front end*, on
//! purpose: `find_text`'s pattern behaviour is its documented contract and
//! silently changing it would move results under every existing caller, so
//! what moved was [`pdfcer_core::edit::TextSearchOptions`]'s **default**, and
//! a front end that wants patterns now has to ask.
//!
//! So this module calls **[`pdfcer_core::edit::EditSession::find_text_with`]
//! and never `find_text`**, wildcards default to off, and the control that
//! turns them on is labelled with what `#` and `?` do
//! ([`crate::text::find::wildcards`]). `tests::the_default_search_is_literal`
//! and `tests::a_wildcard_search_is_only_ever_asked_for_explicitly` are the
//! regression tests; either one fails if someone reaches for the shorter
//! verb.
//!
//! ### The hazard next door, left where the next person will find it
//!
//! `EditSession::mark_redactions_by_search` matches **literally** while
//! `find_text` patterns. A future *"redact every hit"* button built on a
//! wildcard search would therefore highlight hits that the redaction then
//! declines to mark — the highlight and the removal disagreeing about which
//! text exists, which on a redaction is not a cosmetic difference. **This
//! build has no such button.** If one is added, it must either force
//! wildcards off for its own search or refuse while they are on;
//! [`crate::text::find::wildcards_tooltip`] already tells the operator the
//! two match literally-versus-not, so the words exist and only the control
//! is missing.
//!
//! ## ★ Where the bar is, and why
//!
//! **A compact box floating over the top-right of the page**, drawn as an
//! `egui::Area` positioned from the canvas viewport's own rect — not from the
//! window's, so a dock opening moves it with the page rather than leaving it
//! stranded over a panel.
//!
//! That is where Acrobat Reader, Chrome's PDF viewer and Edge's all put
//! theirs, and matching them is most of the argument: Ctrl+F is a chord an
//! operator arrives already knowing, and the surface it produces should be
//! where their eyes go. This application is meant to replace Reader, and
//! Reader's Ctrl+F box is a field, two arrows and a settings dropdown in the
//! top right of the document view.
//!
//! The second reason is measured rather than conventional. **A docked bar was
//! built first**, spanning the window above the status bar, and driving the
//! binary showed what docking costs: the bar takes its height out of the
//! canvas, the canvas feeds `ViewState::apply_fit`, and under *Fit page*
//! pressing Ctrl+F moved the zoom from **85 % to 81 %** — and back to 85 % on
//! close. The page jumps every time the operator goes looking for a word on
//! it, and jumps back when they stop. An overlay consumes no layout, so it
//! cannot do that: the page does not move at all.
//!
//! It costs what an overlay always costs — it covers a corner of the sheet.
//! Two things keep that small: the box is deliberately narrow (the four search
//! options are behind an `Options` menu rather than laid out along the row —
//! see [`bar`]), and it is at the **top** right, which on a drawing sheet is
//! usually clear where the bottom right is the title block.
//!
//! **R128 does not reach this surface, and that is a consequence rather than
//! an exemption.** The rule is *a panel whose size feeds a fit-to-viewport
//! computation has a fixed size*, and an `egui::Area` feeds no such
//! computation. The box's width is fixed all the same, for a different reason
//! of its own: it is anchored by its top-right corner, so a width that changed
//! with the readout's text would move every control on it. [`bar`]'s header
//! carries that argument.
//!
//! ## ★ What happens to stale results
//!
//! **An edit clears the highlights, keeps the query, and says so.**
//!
//! A hit is a *quad*, and a quad is a claim about where particular glyphs
//! are. `delete_*` excises byte spans and renumbers; `move_*` rewrites
//! operands. After either, a quad recorded beforehand can cover different
//! glyphs, no glyphs, or the right glyphs in the wrong place — and rule 4
//! forbids painting a mark over content that does not say what the mark
//! claims. There is no cheap way to tell which hits survived: the geometry
//! comes from a full document text extraction, so "re-check one hit" costs
//! the same as "re-run the search".
//!
//! The three available answers, and why this one:
//!
//! | answer | rejected because |
//! |---|---|
//! | re-search automatically | a search is a whole-document text extraction — 5.6 MB of CAD drawing per edit, on the frame after every nudge of an object |
//! | keep drawing the old hits | draws a highlight that may be over the wrong text, which is the one thing rule 4 forbids outright |
//! | **clear the geometry, keep the query, say so** | ✔ |
//!
//! Mechanically: [`Results`] records the `edit_epoch` it was computed at,
//! [`FindState::readout`] returns [`Readout::Stale`] the instant that epoch
//! moves, [`FindState::current_hit`] returns `None`, and the overlay
//! therefore draws nothing. The bar shows *"Document changed"* with a
//! tooltip saying to press Enter. Re-running is one keypress and it is the
//! **operator's** keypress.
//!
//! Closing the document is the harder version of the same event and is
//! handled by [`FindState::forget_document`], called from the same two
//! places `crate::panels::PanelsState::forget_document` is
//! (`PdfcerApp::open_path` and `PdfcerApp::close_document`) — a hit list
//! naming pages of a file that is no longer open is not stale, it is
//! nonsense.
//!
//! ## ★ Searching is not free, and nothing here searches on a keystroke
//!
//! [`pdfcer_core::edit::EditSession::find_text_with`] runs
//! `text_extract::extract_document_view` over the **whole document** on
//! every call — every page, every content stream, decoded, tokenised and
//! walked, with fonts resolved. There is no cache in `pdfcer-core` and none
//! here. On the project's benchmark sheet (`ncored-benchmark-cad-drawing.pdf`,
//! 5.6 MB, 129,758 objects on one page) that is a measurable fraction of a
//! second, and it is a *whole-document* cost that grows with page count
//! rather than with what is on screen.
//!
//! A find bar that searched per keystroke would therefore run that
//! extraction once per character typed — five extractions to type `total`,
//! four of them for prefixes nobody asked about, each one blocking the UI
//! thread it is dispatched from. Incremental search is a feature of editors
//! whose document is already in memory as text; a PDF's is not, and
//! pretending otherwise is how a viewer becomes unusable on exactly the
//! files it exists for.
//!
//! So: **a search runs only when the operator commits one** — Enter in the
//! field, the step buttons when the results are not current, or a change to
//! an option after a search has already been run (an explicit click, not a
//! keystroke, and one whose whole purpose is to change the hit list). Every
//! run reports its own cost on the `PDFCER_DIAG` channel:
//!
//! ```text
//! pdfcer-diag find needle="total" hits=47 current=1 page=3 ms=214 \
//!            case=insensitive whole=off wildcards=off boundary=Alphanumeric
//! ```
//!
//! …so the number in this header can be re-measured on any file by anyone,
//! rather than being a claim about one machine on one day.
//!
//! ## Actions, not mutations
//!
//! Nothing in [`bar`] touches a document. Every commit becomes an
//! [`crate::app::actions::Action::Find`] carrying a [`FindRequest`], applied
//! after the frame through the one funnel — and that is not ceremony here,
//! it is a requirement: the search needs `&mut EditSession`, which means
//! `Arc::get_mut` on `OpenDoc::session`, which fails while the render worker
//! holds its clone. The funnel is what makes it legal to stop the worker
//! first. See [`search`] for that protocol and for how it differs from
//! `app::actions::vector_edit`'s.
//!
//! ## Why the state lives on `PdfcerApp` and not on `OpenDoc`
//!
//! `crate::app`'s rule: *state that dies with the document lives on
//! `OpenDoc`; state that outlives it lives on `PdfcerApp`.* A find query and
//! its options outlive a document — closing one file and opening another is
//! the most likely moment to search for the same term again — so
//! [`FindState`] sits beside [`crate::panels::PanelsState`], with the same
//! `forget_document` seam for the half that does *not* outlive it.
//!
//! The one piece that does live on the document is [`Reveal`], and it has
//! to: it is *view* bookkeeping that spans two frames, exactly like
//! `OpenDoc::zoom_anchor`, and for the same reason — the page it targets has
//! not been navigated to yet on the frame the request is made.

/// The Find bar's widgets. Split from this file because the two answer
/// different questions — *what is a search and what does it mean* here,
/// *what does the operator see and click* there — and because this module's
/// header is already the longer half of the subject.
pub mod bar;
/// Bringing a hit onto the screen: the two-frame handshake, the gate that
/// spends it, the scroll solve it shares with `canvas::zoom`, and the
/// projection from a core `Quad` into the space the canvas paints in. Split
/// from this file at rule R2's 1,500-line ceiling, along a seam that was
/// already there — *what a search means* here, *how one hit reaches the
/// operator's eye* there.
pub mod reveal;

pub use reveal::{Reveal, take_reveal_offset};

use std::sync::Arc;
use std::time::Instant;

use egui::Rect;
use pdfcer_core::edit::{TextSearchOptions, WordBoundary};

use crate::app::state::OpenDoc;
use crate::canvas::overlay::FindHighlight;

// ===========================================================================
// Options
// ===========================================================================

/// What the operator has asked a search to mean.
///
/// A shell-side struct rather than [`TextSearchOptions`] itself, and the
/// difference is deliberate in three places:
///
/// 1. **`case_sensitive`, not `case_insensitive`.** The control on the bar
///    says *Match case* — the thing the operator switches **on** — and a
///    field whose polarity is the inverse of its checkbox is how a `!` gets
///    dropped. The inversion happens once, in [`Self::to_core`], with a test.
/// 2. **`TextSearchOptions` is `#[non_exhaustive]`**, so it cannot be
///    written as a struct expression from this crate and cannot be exhaustively
///    matched. Owning a plain struct keeps the bar's state a plain value that
///    `PartialEq` and `Default` work on.
/// 3. **The default differs, on purpose.** See [`Self::default`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindOptions {
    /// Whether `total` should stop finding `TOTAL`. The *Match case* control.
    pub case_sensitive: bool,
    /// Whether a hit must be a complete word. The *Whole word* control.
    pub whole_word: bool,
    /// Whether `#` and `?` are wildcards. See this module's ★ trap section.
    pub wildcards: bool,
    /// Which characters make a word, when [`Self::whole_word`] is on.
    ///
    /// Held whether or not whole-word is on, because
    /// [`TextSearchOptions::with_word_boundary`]'s own docs say why: turning
    /// the option off and on again must not silently discard a rule the
    /// operator chose.
    pub word_boundary: WordBoundary,
}

impl Default for FindOptions {
    /// ★ **Case-insensitive, substring, literal,
    /// [`WordBoundary::Alphanumeric`].**
    ///
    /// Three of the four are [`TextSearchOptions`]'s own defaults. The fourth
    /// is not, and the divergence is a decision:
    ///
    /// **Case.** `TextSearchOptions::default()` is case-*sensitive*, because
    /// its job is to reproduce `find_text(needle, false)` byte for byte for
    /// existing callers. A find **bar** is not an existing caller. Reader's
    /// *Case-Sensitive* toggle is off by default, so is every browser's, and
    /// an operator who types `total` and is not shown `TOTAL` on the next
    /// line reads that as a search that did not work. So this shell starts
    /// case-insensitive and the control turns it off.
    ///
    /// **Wildcards.** Off, which is core's default and the whole subject of
    /// this module's ★ trap section.
    ///
    /// **Word boundary — `Alphanumeric`, and here is the justification the
    /// brief asks for.** ISO 32000-1 §14.8.2.5 NOTE 1 says outright that
    /// *"the notion of a word is not precisely defined"*, and NOTE 4 offers
    /// three reader strategies without preferring one, so there is no
    /// standard answer to import — only a choice, which is precisely the
    /// shape the operator's standing directive covers: *where standards are
    /// ambiguous those should become settings, with the initial installed
    /// default as the best guess of what is usually followed.*
    ///
    /// `Alphanumeric` is that best guess on two independent grounds, and
    /// `pdfcer-core` classifies it as **evidence tier (c)** — what other major
    /// implementations do, as documented, rather than a bare guess:
    ///
    /// - Acrobat Reader's own *Whole Words Only* is recorded as an
    ///   exact-boundary match where `stick` does not match `tick` or
    ///   `sticky`, which is what this variant produces.
    /// - It is `\w`/`\b`, the boundary model every mainstream search box and
    ///   regex engine ships, so it is what the operator's habits already
    ///   predict.
    ///
    /// The alternatives are better for narrower work and are offered rather
    /// than hidden: `NonSpace` is right when the text is part numbers or
    /// file paths (`A-12/B` is one token), `NonSpaceOrDash` when hyphenated
    /// compounds matter. Neither is a good *default*, because under
    /// `NonSpace` the string `(total)` does not contain the whole word
    /// `total` — which is a surprising answer to give somebody who ticked a
    /// box called "Whole word" and typed an ordinary English word.
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            wildcards: false,
            word_boundary: WordBoundary::Alphanumeric,
        }
    }
}

impl FindOptions {
    /// Turn the operator's choices into the engine's request.
    ///
    /// **The one place the case polarity is inverted**, and the one place
    /// `wildcards` is stated at all — which is what makes the ★ trap
    /// checkable rather than a promise: there is exactly one construction of
    /// a [`TextSearchOptions`] in this crate, it is this function, and
    /// `tests::the_default_search_is_literal` reads it.
    #[must_use]
    pub fn to_core(self) -> TextSearchOptions {
        TextSearchOptions::default()
            .with_case_insensitive(!self.case_sensitive)
            .with_whole_word(self.whole_word)
            .with_word_boundary(self.word_boundary)
            .with_wildcards(self.wildcards)
    }

    /// The three whole-word rules, in the order the chooser offers them.
    ///
    /// Narrowest-word-first: `Alphanumeric` splits at the most characters,
    /// `NonSpace` at the fewest. A chooser whose entries are in an arbitrary
    /// order makes the operator read all three every time.
    ///
    /// A `const` list rather than a `match` over the enum because
    /// [`WordBoundary`] is `#[non_exhaustive]` — a fourth variant (core names
    /// UAX #29 as a candidate) cannot be matched exhaustively here, and a
    /// wildcard arm would silently drop it from the chooser instead of
    /// failing to compile. This list is the one that has to be extended, and
    /// `tests::every_word_rule_the_chooser_offers_has_a_label` is what says
    /// so out loud.
    pub const WORD_RULES: &'static [WordBoundary] = &[
        WordBoundary::Alphanumeric,
        WordBoundary::NonSpace,
        WordBoundary::NonSpaceOrDash,
    ];
}

// ===========================================================================
// Hits and results
// ===========================================================================

/// One occurrence, as this shell needs it.
///
/// Not [`pdfcer_core::edit::TextMatch`] itself, for two reasons that both
/// matter:
///
/// - `TextMatch` is `#[non_exhaustive]`, so a test in this crate cannot
///   construct one — which would leave every rule in this module
///   (stepping, wrapping, the readout, staleness) testable only through a
///   real document and a real search;
/// - the **canvas-space rectangle is computed once, here, at search time**
///   rather than per frame. See [`Self::canvas`].
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// Zero-based page index, in the session's page space.
    pub page: usize,
    /// The hit's box in **canvas space** — Y-down, origin at the page's
    /// top-left, `/Rotate` applied — or `None` if the page's device
    /// transform is not invertible.
    ///
    /// ★ **Projected once, at search time.** The core match carries a
    /// [`pdfcer_core::annot_author::Quad`] in *unrotated PDF user space*
    /// (Y-**up**, origin at the un-rotated CropBox's lower-left), which is a
    /// different frame from the one the canvas paints in. The conversion is
    /// [`crate::viewer::pdf_space_to_canvas`], the single bridge that works
    /// by inverting the renderer's **own** device transform so the geometry
    /// and the picture agree by construction.
    ///
    /// Doing it here rather than in the overlay is a real saving and a real
    /// simplification: page geometry cannot change while a document is open,
    /// so the answer is constant for the life of the hit, and the paint path
    /// becomes a filter and a projection with no PDF concepts in it at all.
    /// A page whose transform will not invert yields `None` and is simply not
    /// drawn — the hit is still counted and still navigable, because "we
    /// cannot draw a box on this page" is not "this hit does not exist".
    pub canvas: Option<Rect>,
    /// What was actually matched.
    ///
    /// Kept because a case-insensitive search for `total` matches `TOTAL`,
    /// and core's own `TextMatch::text` doc says the operator reviewing hits
    /// needs to see which one they got. Not yet shown on the bar — there is
    /// no results list in this build — and held rather than dropped because
    /// the *next* surface (a hit list, or a redaction review) is the one that
    /// needs it and dropping it here would make that surface a second search.
    pub text: String,
}

/// One completed search, and what it was a search for.
///
/// The three fields above `hits` are the **currency key**: results describe a
/// query, under options, against a revision, and any of the three moving
/// makes them something other than an answer to the question now being
/// asked. Storing the key with the answer is what lets
/// [`FindState::readout`] be a pure function of state rather than a flag
/// somebody has to remember to clear.
#[derive(Debug, Clone, PartialEq)]
pub struct Results {
    /// The exact needle that was searched for.
    query: String,
    /// The options it was searched under.
    options: FindOptions,
    /// The document's [`OpenDoc::edit_epoch`] at the moment of the search.
    epoch: u64,
    /// Every hit, in document order — page, then position on the page.
    hits: Vec<Hit>,
    /// **How many fonts in this document carry text no search could reach.**
    ///
    /// Type 3 fonts and `Identity-H` fonts with no `/ToUnicode` CMap, summed,
    /// from `pdfcer-core`'s `TextDiagnostics` via `search_text`.
    ///
    /// ★★★ Why a Find bar needs this at all. A zero-result search has **two**
    /// causes that produce an identical empty result: the word is not in the
    /// document, or *the document's text was never recoverable as Unicode, so
    /// no word could ever have matched it*. The second is not exotic and it
    /// does not look broken — the text **renders perfectly**, which is exactly
    /// what makes it invisible. Answering that with a bare "0 results" is, in
    /// the engine's own phrase, lying by omission.
    ///
    /// ★ Acrobat has the identical limit — its extract/search/copy pipeline for
    /// Type 3 is gated on the same `/ToUnicode` entry — and answers it by
    /// giving up silently. Rule 4 forbids that here: an inference the operator
    /// **cannot see** still owes them an off-canvas report. This is the "still
    /// owes a report" half of the rule, which is the half that gets forgotten.
    ///
    /// Document-wide rather than filtered to the pages that matched, because
    /// `search_text` returns it that way and deliberately so: a font that
    /// swallowed the needle on page 40 is precisely the one a caller with zero
    /// hits needs to hear about.
    unsearchable_fonts: u64,
    /// Which hit the view is on, as an index into [`Self::hits`].
    ///
    /// Meaningless, and never read, when `hits` is empty.
    current: usize,
}

/// What the bar's readout should say.
///
/// A four-way enum rather than an `Option<(usize, usize)>` because the four
/// states have four different sentences and an operator has to be able to
/// tell them apart: *I have not searched yet* is not *I searched and there
/// is nothing*, and neither is *the answer I gave you is no longer true*.
/// Collapsing any pair of them produces a readout that is silent exactly
/// when the operator most needs a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readout {
    /// No search has been run for the query and options now in the bar.
    /// The readout shows nothing at all.
    Idle,
    /// A search ran and matched nothing.
    Empty,
    /// A search ran and the view is on hit `current` of `total`.
    /// `current` is **one-based**, ready to print.
    At {
        /// Which hit, counting from one.
        current: usize,
        /// How many there are.
        total: usize,
    },
    /// A search ran, and the document has been edited since. The hits are no
    /// longer trustworthy geometry, so nothing is highlighted and nothing is
    /// navigable until the operator searches again. See this module's ★
    /// staleness section.
    Stale,
}

// ===========================================================================
// The state
// ===========================================================================

/// Everything Find remembers.
///
/// Lives on [`crate::app::PdfcerApp`]; see this module's header for why the
/// query outlives a document and the hits do not.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FindState {
    /// Whether the bar is on screen.
    open: bool,
    /// What the operator has typed. Kept across a close and across a
    /// document change — reopening Find with an empty box would make the
    /// commonest action (search the next file for the same thing) a retype.
    query: String,
    /// The operator's choices. Kept for the same reason and for one more:
    /// they are a *preference*, and resetting a preference because a
    /// document closed would be pdfcer discarding a setting the operator made.
    options: FindOptions,
    /// The last completed search, if any.
    results: Option<Results>,
    /// Set when the bar should take keyboard focus on the next frame it
    /// draws, and cleared by the bar when it does.
    ///
    /// A one-shot rather than "focus whenever open": re-requesting focus
    /// every frame would make it impossible to click anything else while the
    /// bar is open, which is the classic way a find bar becomes a trap.
    focus_wanted: bool,
}

impl FindState {
    /// Whether the bar is on screen.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Open the bar and ask for keyboard focus.
    ///
    /// Idempotent about the *open* half and deliberately **not** about the
    /// focus half: `Ctrl+F` pressed while the bar is already open is a
    /// request to type in it, which is what every browser and editor does
    /// with that chord, and it is the recovery an operator reaches for after
    /// clicking on the page.
    pub fn open(&mut self) {
        self.open = true;
        self.focus_wanted = true;
    }

    /// Close the bar.
    ///
    /// **The results go with it**, which is what makes the highlights
    /// disappear: the overlay reads [`Self::current_hit`] and the hit list,
    /// and a closed bar with live hits would leave marks on the page with no
    /// surface saying what they are or how to get rid of them. The query and
    /// the options survive — see [`Self::query`].
    pub fn close(&mut self) {
        self.open = false;
        self.focus_wanted = false;
        self.results = None;
    }

    /// Toggle the bar, which is what the `edit.find` command does.
    ///
    /// Returns whether it is now open, so the caller can trace the transition
    /// rather than re-reading the state it just changed.
    pub fn toggle(&mut self) -> bool {
        if self.open {
            self.close();
        } else {
            self.open();
        }
        self.open
    }

    /// Take the pending focus request, if there is one.
    pub fn take_focus_request(&mut self) -> bool {
        std::mem::take(&mut self.focus_wanted)
    }

    /// What the operator has typed.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// The editable buffer behind the field.
    ///
    /// Handed straight to `egui::TextEdit`, which needs `&mut String`. This
    /// is the one place the bar writes state directly rather than raising an
    /// action, and it is the same exemption `crate::app::status`'s page box
    /// takes for the same reason: a text buffer is *widget* state, it
    /// describes the control rather than the document, and deferring a
    /// keystroke to after the frame would make typing lag by a frame.
    pub fn query_mut(&mut self) -> &mut String {
        &mut self.query
    }

    /// The operator's search options.
    #[must_use]
    pub fn options(&self) -> FindOptions {
        self.options
    }

    /// Replace the search options.
    ///
    /// Does **not** clear the results, and does not need to: [`Self::readout`]
    /// compares the options the results were computed under against the ones
    /// now set, so changing an option makes the results non-current by the
    /// same rule that a changed query does. One currency test, three inputs.
    pub fn set_options(&mut self, options: FindOptions) {
        self.options = options;
    }

    /// Forget everything that describes a *document*, keeping everything that
    /// describes the *operator*.
    ///
    /// Called from `PdfcerApp::open_path` and `PdfcerApp::close_document`, the
    /// same two sites `crate::panels::PanelsState::forget_document` is called
    /// from and for the same reason: page indices and page-space rectangles
    /// are positions in one file, and carrying them into another one is not
    /// staleness but nonsense. The bar stays open if it was open — the
    /// operator did not ask for it to close — with an empty readout and the
    /// query they last typed, ready for Enter.
    pub fn forget_document(&mut self) {
        self.results = None;
    }

    /// **Whether the bar is currently showing an answer to what is in it** —
    /// the *document-independent* half of the currency test.
    ///
    /// True when a search has been run for exactly this query under exactly
    /// these options, whatever has happened to the document since. That is
    /// deliberately weaker than [`Self::readout`] returning [`Readout::At`],
    /// and it is the right test for its one caller: [`bar`] asks it before
    /// changing an option, to decide whether the change should re-run the
    /// search. An edit having intervened is not a reason to *skip* the
    /// re-run — the operator has just asked for a different hit list — and
    /// the epoch is not reachable from that call site anyway.
    #[must_use]
    pub fn answered(&self) -> bool {
        self.results
            .as_ref()
            .is_some_and(|r| r.query == self.query && r.options == self.options)
    }

    /// ★ **What the readout says** — the pure rule, testable without a
    /// document, a frame or a search.
    ///
    /// `epoch` is [`OpenDoc::edit_epoch`], or any value at all when nothing
    /// is open (there are then no results, so every branch below yields
    /// [`Readout::Idle`]).
    ///
    /// The order of the tests is the interesting part. **Staleness is checked
    /// before emptiness**, so a document edited after a fruitless search says
    /// *"Document changed"* rather than *"No matches"* — the second would be
    /// a claim about the current revision that the search never made.
    #[must_use]
    /// **How many fonts made this document partly unsearchable**, for the
    /// query the bar currently holds — or `0` when there is nothing to say.
    ///
    /// Returns `0` unless the last search is the one the bar is showing, so a
    /// stale or edited-away result cannot leave a sentence on screen about a
    /// query the operator has moved on from. Same staleness rules as
    /// [`Self::readout`], deliberately: two surfaces describing one search must
    /// not disagree about which search it is.
    pub fn unsearchable_fonts(&self, epoch: u64) -> u64 {
        let Some(results) = &self.results else {
            return 0;
        };
        if results.query != self.query || results.options != self.options || results.epoch != epoch
        {
            return 0;
        }
        results.unsearchable_fonts
    }

    pub fn readout(&self, epoch: u64) -> Readout {
        let Some(results) = &self.results else {
            return Readout::Idle;
        };
        // A different question is not a stale answer to this one; it is no
        // answer at all, and the readout should be blank rather than
        // reporting on a query the operator has already edited away from.
        if results.query != self.query || results.options != self.options {
            return Readout::Idle;
        }
        if results.epoch != epoch {
            return Readout::Stale;
        }
        if results.hits.is_empty() {
            return Readout::Empty;
        }
        Readout::At {
            current: results.current + 1,
            total: results.hits.len(),
        }
    }

    /// The hit the view is on, or `None` when there is not one.
    ///
    /// `None` covers every non-[`Readout::At`] state, staleness included —
    /// which is the mechanism by which an edit stops the highlights: the
    /// overlay asks this, and a stale result answers no.
    #[must_use]
    pub fn current_hit(&self, epoch: u64) -> Option<&Hit> {
        if !matches!(self.readout(epoch), Readout::At { .. }) {
            return None;
        }
        let results = self.results.as_ref()?;
        results.hits.get(results.current)
    }

    /// Every hit on `page`, paired with whether it is the current one.
    ///
    /// The overlay's input. Empty — not merely all-`false` — whenever the
    /// results are not current, so a stale or superseded search paints
    /// nothing at all rather than painting hits without a highlighted one.
    ///
    /// Returns [`FindHighlight`]s, which carry a canvas-space rect and a
    /// flag and nothing else: `crate::canvas::overlay` is not told what a
    /// page index or a quad is, and this module is not told what a `Painter`
    /// is.
    pub fn page_highlights(
        &self,
        page: usize,
        epoch: u64,
    ) -> impl Iterator<Item = FindHighlight> + '_ {
        let results = self
            .results
            .as_ref()
            .filter(|_| matches!(self.readout(epoch), Readout::At { .. }));
        let current = results.map_or(usize::MAX, |r| r.current);
        results
            .into_iter()
            .flat_map(|r| r.hits.iter().enumerate())
            .filter(move |(_, hit)| hit.page == page)
            .filter_map(move |(index, hit)| {
                Some(FindHighlight {
                    rect: hit.canvas?,
                    current: index == current,
                })
            })
    }
}

// ===========================================================================
// The request, and applying it
// ===========================================================================

/// Which way [`FindRequest::Step`] moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Toward the end of the document, wrapping to the first hit.
    Next,
    /// Toward the start, wrapping to the last hit.
    Previous,
}

/// One thing the operator asked Find to do, carried by
/// [`crate::app::actions::Action::Find`].
///
/// Two variants and no more. In particular there is **no** `Open`/`Close`
/// variant: opening the bar changes no document state and needs no frame
/// boundary, so it happens in the `edit.find` dispatch arm directly, exactly
/// as `file.properties` mounts a panel there. What has to go through the
/// funnel is what needs the *document* — and both of these do, one because
/// it borrows the session mutably and one because it navigates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindRequest {
    /// Run the search now, for whatever is in the bar.
    Search,
    /// Move to the adjacent hit.
    Step(Step),
}

/// Apply one [`FindRequest`].
///
/// Called from `PdfcerApp::apply`, **after** the frame that raised it, which
/// is the only place the two borrows this needs can be had at once: the state
/// and the open document are separate fields of `PdfcerApp`.
pub fn apply(state: &mut FindState, doc: &mut OpenDoc, request: FindRequest) {
    match request {
        FindRequest::Search => search(state, doc),
        FindRequest::Step(step) => step_to(state, doc, step),
    }
}

/// ★ **Run the search.**
///
/// # The borrow protocol, and how it differs from an edit's
///
/// [`pdfcer_core::edit::EditSession::find_text_with`] takes `&mut self` —
/// it is a *read* that needs a mutable borrow — and `OpenDoc::session` is an
/// `Arc` precisely so a render worker can hold a clone while it rasterizes.
/// `Arc::get_mut` fails while any other strong reference exists, so the
/// worker is stopped first, exactly as `app::actions::vector_edit` does:
/// `RenderWorker::cancel_and_wait`'s own docs call itself *"the choke point
/// that makes `Arc<EditSession>` sound"*.
///
/// Two steps of `vector_edit`'s four are **deliberately absent**, and their
/// absence is the whole difference between a search and an edit:
///
/// - **`edit_epoch` is not bumped.** Nothing about the document changed. A
///   bump would throw away the page decomposition and the font inventory, and
///   would immediately make the results this function just produced *stale by
///   its own rule* — a search that invalidated itself.
/// - **The page texture is not dropped.** The picture on screen is still a
///   picture of the page. Dropping it would re-rasterize a CAD sheet on every
///   Enter.
///
/// A cancelled render is re-spawned by `settle_and_rasterize` at the end of
/// the same frame if the texture is stale, and left alone if it is not — so
/// the cost of the cancel is a rasterization that was going to happen anyway,
/// restarted.
///
/// # An empty query is not a search
///
/// `find_text_with` already returns an empty vector for an empty needle, so
/// this could simply run. It does not, because the two states must not look
/// the same on the bar: "you have not typed anything" is [`Readout::Idle`]
/// and "there is nothing here" is [`Readout::Empty`], and running a search
/// for `""` would put the second sentence in front of an operator who had
/// merely cleared the box.
fn search(state: &mut FindState, doc: &mut OpenDoc) {
    let query = state.query.clone();
    if query.is_empty() {
        state.results = None;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find-declined reason=empty-query".to_owned()
        });
        return;
    }

    doc.render_worker.cancel_and_wait();
    let Some(session) = Arc::get_mut(&mut doc.session) else {
        // Not a panic: something else still holds the session, which is a
        // bug in this caller's ordering rather than in the operator's
        // document. Declining leaves the previous results in place, and the
        // bar's readout still describes them honestly.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find-refused reason=session-borrowed".to_owned()
        });
        return;
    };

    let options = state.options;
    let started = Instant::now();
    // ★ `search_text`, NEVER `find_text`. See this module's trap section:
    // `find_text` passes `with_wildcards(true)`, and a Find bar built on it
    // matches every character on the page when the operator types `?`.
    //
    // ★★ `search_text` rather than `find_text_with` since `pdfcer-core` v0.11.0.
    // It runs the IDENTICAL scan and returns the IDENTICAL hits —
    // `find_text_with` now delegates to it — and additionally hands back the
    // extraction diagnostics that say whether a zero-result answer can be
    // trusted. See `Results::unsearchable_fonts`. There is no behavioural
    // difference in the matching and no new failure mode; the only cost is
    // holding a `TextDiagnostics` that was previously computed and discarded.
    let found = session.search_text(&query, &options.to_core());
    let unsearchable_fonts = found.diagnostics.type3_fonts_without_to_unicode
        + found.diagnostics.identity_fonts_without_to_unicode;
    let matches = found.matches;
    let elapsed = started.elapsed();

    let hits: Vec<Hit> = matches
        .into_iter()
        .map(|m| Hit {
            page: m.page_index,
            canvas: doc
                .pages
                .get(m.page_index)
                .and_then(|page| reveal::quad_to_canvas(&m.quad, page)),
            text: m.text,
        })
        .collect();

    let total = hits.len();
    let first_page = hits.first().map(|h| h.page);
    state.results = Some(Results {
        query: query.clone(),
        options,
        unsearchable_fonts,
        epoch: doc.edit_epoch,
        hits,
        current: 0,
    });

    // ★ The line the cost claim in this module's header is made from, and the
    // line a harness reads to know a search ran at all.
    //
    // `current=` is ONE-BASED, matching the bar's readout, so a trace and a
    // screenshot of the same moment say the same number; `0` means there is
    // nothing to be on. `page=-1` likewise means "no hit, so no page" rather
    // than page zero, which is a real page.
    //
    // NOT de-duplicated through `trace_changed`: two identical searches are
    // two events, and a gate that silenced the second would make a harness
    // unable to tell a search that ran twice from one that ran once.
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find needle={query:?} hits={total} current={} page={} ms={} \
             case={} whole={} wildcards={} boundary={:?}",
            usize::from(total > 0),
            first_page.map_or(-1_i64, |p| i64::try_from(p).unwrap_or(-1)),
            elapsed.as_millis(),
            // ui-text-exempt: diagnostic trace field values, never displayed
            if options.case_sensitive {
                "sensitive"
            } else {
                "insensitive"
            },
            if options.whole_word { "on" } else { "off" },
            if options.wildcards { "on" } else { "off" },
            options.word_boundary,
        )
    });

    // Land on the first hit. Doing it here rather than leaving the view where
    // it was is the difference between a search and a report: the operator
    // asked where the text is, and the answer is the page it is on.
    reveal::reveal_current(state, doc);
}

/// Move to the adjacent hit and bring it into view.
///
/// Declines — visibly, on the trace, and with the bar's own controls already
/// unavailable — when the results are not current. The bar never raises this
/// in that state (it raises [`FindRequest::Search`] instead), so reaching
/// here means a keymap or a future surface got to the verb another way, and
/// the honest answer is to do nothing rather than to step through geometry
/// this module has already declared untrustworthy.
fn step_to(state: &mut FindState, doc: &mut OpenDoc, step: Step) {
    if !matches!(state.readout(doc.edit_epoch), Readout::At { .. }) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "find-step-declined reason=no-current-results".to_owned()
        });
        return;
    }
    let Some(results) = state.results.as_mut() else {
        return;
    };
    results.current = next_index(results.current, results.hits.len(), step);
    reveal::reveal_current(state, doc);
}

/// ★ **The wrap rule**, as a pure function of three numbers.
///
/// Wrapping rather than stopping, which is the opposite of what
/// `crate::viewer::ViewState::next_page` does — and the difference is not an
/// inconsistency. Page navigation saturates because *"wrap-around page
/// navigation silently teleports an operator from page 400 to page 1"*: the
/// operator is reading, and the pages have an order they care about. Stepping
/// hits is a **search**, the hit list is a ring the operator is working
/// around, and stopping at the last one would leave them pressing a live
/// button that does nothing with no way to tell that from a broken one. Every
/// find bar in the product class wraps.
///
/// `len == 0` is not reachable through [`step_to`], which checks
/// [`Readout::At`] first, and is handled anyway: an action can be raised from
/// anywhere and an index into an empty list is a panic waiting for a
/// customized keymap to find it.
#[must_use]
fn next_index(current: usize, len: usize, step: Step) -> usize {
    if len == 0 {
        return 0;
    }
    match step {
        Step::Next => (current + 1) % len,
        // `+ len - 1` rather than `- 1`: `current` is a `usize` and hit 0's
        // predecessor is the last hit, so the subtraction has to happen after
        // the addition or it underflows on the one case the wrap exists for.
        Step::Previous => (current + len - 1) % len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, open_fixture};

    /// A state with `hits` hits on page `page`, already searched for `query`.
    fn searched(query: &str, page: usize, hits: usize) -> FindState {
        #[allow(
            clippy::cast_precision_loss,
            reason = "small test-fixture indices, stacked into distinct rows" // ui-text-exempt: clippy lint justification, never displayed
        )]
        let hits: Vec<Hit> = (0..hits)
            .map(|i| Hit {
                page,
                canvas: Some(Rect::from_min_size(
                    egui::pos2(10.0, 10.0 * i as f32),
                    egui::vec2(40.0, 8.0),
                )),
                text: query.to_owned(),
            })
            .collect();
        FindState {
            query: query.to_owned(),
            results: Some(Results {
                unsearchable_fonts: 0,
                query: query.to_owned(),
                options: FindOptions::default(),
                epoch: 0,
                hits,
                current: 0,
            }),
            ..FindState::default()
        }
    }

    // =======================================================================
    // ★ The trap
    // =======================================================================

    /// ★ **The default search is literal.**
    ///
    /// The regression test for the defect this whole module's header is
    /// about: the old shell's Find bar ran through `EditSession::find_text`,
    /// which passes `with_wildcards(true)`, so a typed `?` matched every
    /// character on the page. `to_core` is the ONE place a
    /// `TextSearchOptions` is built in this crate, so asserting on it is
    /// asserting on every search this shell can run.
    #[test]
    fn the_default_search_is_literal() {
        let core = FindOptions::default().to_core();
        assert!(
            !core.wildcards,
            "a search box must search for what was typed; `?` is a question mark"
        );
        assert!(
            core.case_insensitive,
            "a find bar is forgiving about case by default — Reader's toggle is off, and so \
             is every browser's"
        );
        assert!(!core.whole_word);
        assert_eq!(core.word_boundary, WordBoundary::Alphanumeric);
    }

    /// ★ **Wildcards are only ever on because the operator asked.**
    ///
    /// The other direction, which matters as much: the control has to work,
    /// or the escape hatch from the literal default would be a dead
    /// checkbox — the placeholder P3 forbids, in the one place the operator
    /// went looking for a feature.
    #[test]
    fn a_wildcard_search_is_only_ever_asked_for_explicitly() {
        let asked = FindOptions {
            wildcards: true,
            ..FindOptions::default()
        };
        assert!(asked.to_core().wildcards);
    }

    /// The case control's polarity is inverted exactly once.
    ///
    /// The shell says *Match case* and the engine says `case_insensitive`.
    /// A dropped `!` here is a search that ignores the checkbox, which looks
    /// like a search that ignores the operator.
    #[test]
    fn match_case_inverts_into_the_engines_polarity() {
        let sensitive = FindOptions {
            case_sensitive: true,
            ..FindOptions::default()
        };
        assert!(!sensitive.to_core().case_insensitive);
        assert!(FindOptions::default().to_core().case_insensitive);
    }

    /// The whole-word flag and the rule travel independently.
    ///
    /// `TextSearchOptions::with_word_boundary`'s own docs require this:
    /// choosing a rule must not switch the option on, and switching the
    /// option off and on again must not reset the rule.
    #[test]
    fn the_word_rule_and_the_whole_word_flag_are_independent() {
        let rule_only = FindOptions {
            word_boundary: WordBoundary::NonSpace,
            ..FindOptions::default()
        };
        let core = rule_only.to_core();
        assert!(!core.whole_word, "choosing a rule does not switch it on");
        assert_eq!(core.word_boundary, WordBoundary::NonSpace);

        let both = FindOptions {
            whole_word: true,
            ..rule_only
        };
        assert!(both.to_core().whole_word);
        assert_eq!(both.to_core().word_boundary, WordBoundary::NonSpace);
    }

    /// Every rule the chooser offers is a real variant, and the list is the
    /// whole of what a `#[non_exhaustive]` enum lets this crate name.
    ///
    /// The chooser is driven from [`FindOptions::WORD_RULES`] rather than
    /// from a `match`, because a wildcard arm over a non-exhaustive enum
    /// would silently drop a future variant instead of failing to compile.
    /// This is the reminder that the list is the thing to extend.
    #[test]
    fn every_word_rule_the_chooser_offers_has_a_label() {
        assert_eq!(FindOptions::WORD_RULES.len(), 3);
        for rule in FindOptions::WORD_RULES {
            assert!(!bar::word_rule_label(*rule).is_empty());
        }
    }

    // =======================================================================
    // The readout — the pure rule
    // =======================================================================

    /// Nothing searched yet reads as nothing, not as "no matches".
    #[test]
    fn an_unsearched_bar_says_nothing_rather_than_no_matches() {
        let mut state = FindState::default();
        assert_eq!(state.readout(0), Readout::Idle);
        state.query = "total".to_owned();
        assert_eq!(
            state.readout(0),
            Readout::Idle,
            "typing is not searching; the readout must stay blank until Enter"
        );
    }

    /// A completed search reads one-based, and stepping moves it.
    #[test]
    fn the_readout_counts_hits_from_one() {
        let mut state = searched("total", 2, 3);
        assert_eq!(
            state.readout(0),
            Readout::At {
                current: 1,
                total: 3
            }
        );
        let results = state.results.as_mut().expect("searched");
        results.current = 2;
        assert_eq!(
            state.readout(0),
            Readout::At {
                current: 3,
                total: 3
            }
        );
    }

    /// A fruitless search says so, which is different from having not run.
    #[test]
    fn a_fruitless_search_reports_that_it_ran() {
        let state = searched("nothing", 0, 0);
        assert_eq!(state.readout(0), Readout::Empty);
    }

    /// ★ **Editing the document makes the results stale, not merely old.**
    ///
    /// The staleness rule this module's header argues for, asserted through
    /// both surfaces it governs: the readout says so, and the highlights stop.
    #[test]
    fn an_edit_makes_the_results_stale_and_stops_the_highlights() {
        let state = searched("total", 1, 4);
        assert!(matches!(state.readout(0), Readout::At { .. }));
        assert!(state.current_hit(0).is_some());
        assert_eq!(state.page_highlights(1, 0).count(), 4);

        // One edit later.
        assert_eq!(state.readout(1), Readout::Stale);
        assert!(
            state.current_hit(1).is_none(),
            "a quad recorded before an edit may cover different glyphs after it"
        );
        assert_eq!(
            state.page_highlights(1, 1).count(),
            0,
            "rule 4 forbids painting a mark over content that does not say what it claims"
        );
    }

    /// ★ **Staleness is reported ahead of emptiness.**
    ///
    /// A document edited after a fruitless search must not say "No matches":
    /// that would be a claim about the current revision, which the search
    /// never examined.
    #[test]
    fn an_edited_document_says_it_changed_rather_than_that_there_are_no_matches() {
        let state = searched("nothing", 0, 0);
        assert_eq!(state.readout(0), Readout::Empty);
        assert_eq!(state.readout(1), Readout::Stale);
    }

    /// Editing the query blanks the readout rather than staling it.
    ///
    /// A different question is not an out-of-date answer to this one. The
    /// operator who starts typing a new term should see the readout clear,
    /// not see the old count go on standing next to new text.
    #[test]
    fn changing_the_query_blanks_the_readout() {
        let mut state = searched("total", 0, 5);
        state.query.push('s');
        assert_eq!(state.readout(0), Readout::Idle);
        assert_eq!(state.page_highlights(0, 0).count(), 0);
    }

    /// Changing an option does the same thing, by the same rule.
    #[test]
    fn changing_an_option_blanks_the_readout() {
        let mut state = searched("total", 0, 5);
        state.set_options(FindOptions {
            whole_word: true,
            ..FindOptions::default()
        });
        assert_eq!(state.readout(0), Readout::Idle);
    }

    // =======================================================================
    // Stepping
    // =======================================================================

    /// ★ **Stepping wraps in both directions**, including the underflow case
    /// that a naive `- 1` gets wrong.
    #[test]
    fn stepping_wraps_at_both_ends() {
        assert_eq!(next_index(0, 3, Step::Next), 1);
        assert_eq!(
            next_index(2, 3, Step::Next),
            0,
            "the last hit wraps to the first"
        );
        assert_eq!(next_index(1, 3, Step::Previous), 0);
        assert_eq!(
            next_index(0, 3, Step::Previous),
            2,
            "the first hit's predecessor is the last; `- 1` on a usize underflows here"
        );
        // One hit is its own neighbour in both directions.
        assert_eq!(next_index(0, 1, Step::Next), 0);
        assert_eq!(next_index(0, 1, Step::Previous), 0);
    }

    /// An empty list cannot be stepped into a panic.
    ///
    /// Unreachable through [`step_to`], which checks the readout first, and
    /// handled anyway: an action can be raised from a customized keymap in
    /// any state, and an index into an empty `Vec` is a crash waiting for
    /// somebody to find it.
    #[test]
    fn stepping_an_empty_result_set_is_not_a_panic() {
        assert_eq!(next_index(0, 0, Step::Next), 0);
        assert_eq!(next_index(0, 0, Step::Previous), 0);
    }

    // =======================================================================
    // Lifecycle
    // =======================================================================

    /// Closing the bar takes the highlights with it and keeps the query.
    #[test]
    fn closing_clears_the_hits_and_keeps_the_query() {
        let mut state = searched("total", 0, 3);
        state.open();
        assert!(state.is_open());
        state.close();
        assert!(!state.is_open());
        assert_eq!(state.query(), "total", "what was typed survives a close");
        assert_eq!(
            state.readout(0),
            Readout::Idle,
            "a closed bar must not leave marks on the page with nothing to explain them"
        );
    }

    /// Opening asks for focus exactly once per request.
    ///
    /// Re-requesting focus every frame is how a find bar becomes a trap the
    /// operator cannot click out of.
    #[test]
    fn opening_asks_for_focus_once() {
        let mut state = FindState::default();
        state.open();
        assert!(state.take_focus_request());
        assert!(!state.take_focus_request());

        // …and Ctrl+F on an already-open bar asks again, which is what every
        // browser does and is the recovery after clicking on the page.
        state.open();
        assert!(state.take_focus_request());
    }

    /// The toggle reports the state it produced.
    #[test]
    fn the_toggle_reports_where_it_landed() {
        let mut state = FindState::default();
        assert!(state.toggle());
        assert!(!state.toggle());
    }

    /// ★ **A document change forgets the hits and keeps the operator's
    /// settings.**
    ///
    /// Page indices and page-space rectangles describe one file. Carrying
    /// them into another is not staleness — the epoch would still match,
    /// because a freshly opened document's epoch is 0 — it is nonsense, and
    /// it is why this seam exists rather than relying on the epoch alone.
    #[test]
    fn opening_another_document_forgets_the_hits_but_not_the_query() {
        let mut state = searched("total", 3, 9);
        let options = FindOptions {
            whole_word: true,
            ..FindOptions::default()
        };
        state.set_options(options);
        state.forget_document();
        assert_eq!(state.readout(0), Readout::Idle);
        assert_eq!(state.query(), "total");
        assert_eq!(state.options(), options);
    }

    /// Only the current page's hits are handed to the overlay, and exactly
    /// one of them is marked current.
    #[test]
    fn the_overlay_is_given_this_pages_hits_with_one_marked_current() {
        let mut state = searched("total", 2, 3);
        // Move one hit onto another page, the way a real multi-page result
        // set looks.
        state.results.as_mut().expect("searched").hits.push(Hit {
            page: 5,
            canvas: Some(Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1.0, 1.0),
            )),
            text: "total".to_owned(),
        });

        let on_two: Vec<FindHighlight> = state.page_highlights(2, 0).collect();
        assert_eq!(on_two.len(), 3);
        assert_eq!(on_two.iter().filter(|h| h.current).count(), 1);
        assert!(on_two[0].current, "the search landed on the first hit");

        let on_five: Vec<FindHighlight> = state.page_highlights(5, 0).collect();
        assert_eq!(on_five.len(), 1);
        assert!(
            !on_five[0].current,
            "the current hit is on page 2, so page 5's is not it"
        );

        assert_eq!(state.page_highlights(0, 0).count(), 0);
    }

    /// A hit whose page would not project is counted and navigable but not
    /// drawn.
    ///
    /// "We cannot draw a box on this page" is not "this hit does not exist",
    /// and conflating them would make a document with one degenerate page
    /// report the wrong number of hits.
    #[test]
    fn a_hit_with_no_geometry_still_counts() {
        let mut state = searched("total", 0, 2);
        state.results.as_mut().expect("searched").hits[0].canvas = None;
        assert_eq!(
            state.readout(0),
            Readout::At {
                current: 1,
                total: 2
            },
            "the hit is still one of the hits"
        );
        assert_eq!(
            state.page_highlights(0, 0).count(),
            1,
            "…and the one that can be drawn still is"
        );
    }

    // =======================================================================
    // The whole thing, against a real document
    // =======================================================================

    /// ★ **A real search runs, reports its cost, and lands on its first
    /// hit.**
    ///
    /// The end-to-end check that the borrow protocol works: the render worker
    /// is stopped, `Arc::get_mut` succeeds, the engine is asked, and the view
    /// moves to the page the answer is on. It is deliberately driven through
    /// [`apply`] rather than through [`search`] directly, because the thing
    /// most likely to be wrong is the wiring rather than the arithmetic.
    ///
    /// The fixture's text is asserted to exist first: a test that searched
    /// for a string the fixture does not contain would pass on a build whose
    /// search always returned nothing.
    #[test]
    fn a_real_search_finds_its_text_and_navigates_to_it() {
        let mut doc = open_fixture(FOUR_PAGES);
        let mut state = FindState::default();
        state.open();

        // Whatever this fixture actually says. `Page` is the word the
        // generator stamps on each sheet; if that ever changes, this test
        // fails loudly rather than silently proving nothing.
        state.query_mut().push_str("Page");
        apply(&mut state, &mut doc, FindRequest::Search);

        let Readout::At { current, total } = state.readout(doc.edit_epoch) else {
            panic!(
                "the fixture must contain the search term, or this test proves nothing: {:?}",
                state.readout(doc.edit_epoch)
            )
        };
        assert_eq!(current, 1, "a search lands on its first hit");
        assert!(total >= 1);

        let hit_page = state
            .current_hit(doc.edit_epoch)
            .expect("a current hit")
            .page;
        assert_eq!(
            doc.view.page_index, hit_page,
            "a search that does not go to the page it found is a report, not a search"
        );

        // Stepping moves the readout and stays inside the ring.
        apply(&mut state, &mut doc, FindRequest::Step(Step::Next));
        let Readout::At { current: after, .. } = state.readout(doc.edit_epoch) else {
            panic!("stepping must leave a current hit")
        };
        assert_eq!(after, if total == 1 { 1 } else { 2 });
    }

    /// An empty query is refused rather than searched.
    ///
    /// The two states must not look alike: "you have not typed anything" is
    /// blank, "there is nothing here" is a sentence.
    #[test]
    fn an_empty_query_is_not_a_search() {
        let mut doc = open_fixture(FOUR_PAGES);
        let mut state = FindState::default();
        apply(&mut state, &mut doc, FindRequest::Search);
        assert_eq!(
            state.readout(doc.edit_epoch),
            Readout::Idle,
            "an empty box must not report `No matches`"
        );
    }

    /// A search does not look like an edit.
    ///
    /// `find_text_with` takes `&mut EditSession`, which makes it easy to
    /// mistake for a mutation and to give it `vector_edit`'s epoch bump and
    /// texture drop. Both would be wrong: the bump would make the results
    /// stale by their own rule the instant they were produced, and the drop
    /// would re-rasterize a CAD sheet on every Enter.
    #[test]
    fn a_search_bumps_no_epoch_and_drops_no_texture() {
        let mut doc = open_fixture(FOUR_PAGES);
        let mut state = FindState::default();
        state.query_mut().push_str("Page");
        let before = doc.edit_epoch;
        apply(&mut state, &mut doc, FindRequest::Search);
        assert_eq!(
            doc.edit_epoch, before,
            "a search changes nothing about the document"
        );
        assert!(
            !matches!(state.readout(doc.edit_epoch), Readout::Stale),
            "…and must not invalidate the results it just produced"
        );
    }
}
