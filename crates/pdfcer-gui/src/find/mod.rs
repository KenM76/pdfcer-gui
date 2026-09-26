//! # find — searching the page text, and showing the operator where it is
//!
//! The whole of Find, across three files:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/find/mod.md`.

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

/// Preparing a typed query for the engine: the trim decision, the
/// predicate the bar's disclosure is built on, and the argument for why
/// interior whitespace is left alone. Its own file because that decision,
/// the preference attached to it and the sentence that discloses it must
/// agree, and three things that must agree drift when they live apart.
pub mod query;

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
    /// Whether `#` and `?` are wildcards. See this module's trap section.
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
    /// **Case-insensitive, substring, literal,
    /// [`WordBoundary::Alphanumeric`].**
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
    /// `wildcards` is stated at all — which is what makes the trap
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
    /// **Projected once, at search time.** The core match carries a
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
    /// Why a Find bar needs this at all. A zero-result search has **two**
    /// causes that produce an identical empty result: the word is not in the
    /// document, or *the document's text was never recoverable as Unicode, so
    /// no word could ever have matched it*. The second is not exotic and it
    /// does not look broken — the text **renders perfectly**, which is exactly
    /// what makes it invisible. Answering that with a bare "0 results" is, in
    /// the engine's own phrase, lying by omission.
    ///
    /// Acrobat has the identical limit — its extract/search/copy pipeline for
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
    /// navigable until the operator searches again. See this module's
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
#[derive(Debug, Clone, PartialEq)]
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
    /// **Whether going to a hit is allowed to move the view** — the
    /// operator's *Zoom* control, `OPERATOR_REQUESTS.md` **O163**, 2026-09-09,
    /// widened by **O179**, 2026-09-12.
    ///
    /// “Move the view”, not “change the zoom”, and the correction is the
    /// substance of O179. As shipped this governed only the fit drop in
    /// [`reveal::hold_the_zoom_if_asked`], so with the control off the hit was
    /// still scrolled to the centre of the canvas on every step. It now also
    /// governs whether [`reveal::reveal_current`] arms `doc.find_reveal` at
    /// all, which is the scrolling half.
    ///
    /// Deliberately **not** a field of [`FindOptions`], and the separation is
    /// the point: [`bar`]'s options menu re-runs the search whenever a
    /// [`FindOptions`] field changes, because every one of them changes *what
    /// matches*. This one changes nothing about the answer — only what the
    /// view does with it — so re-running would be a whole-document text
    /// extraction charged for a navigation preference. It is also why
    /// [`Results`]' currency test does not read it: results computed before
    /// the operator toggled this are still exactly correct.
    ///
    /// **`true` is the shipped default**, and that is the *old* behaviour
    /// rather than a new preference: nothing in this module has ever set a
    /// zoom, so `true` means *"do not intervene"* and `false` means
    /// *"intervene, to hold the zoom still"*. See [`reveal::reveal_current`]
    /// for what the intervention is and why a **fit** is the thing being held
    /// off.
    ///
    /// The **persisted** half lives in [`crate::app::prefs::Prefs`]
    /// (`find_zoom_on_jump`) and is mirrored into here once, at startup — the
    /// same two-homes design `view.smart_select` documents, minus the
    /// `egui::Memory` hop, because this value is only ever read from a place
    /// that already holds the [`FindState`].
    zoom_on_jump: bool,
    /// **Whether whitespace at either end of the query is ignored** —
    /// `OPERATOR_REQUESTS.md` **O180**, 2026-09-12.
    ///
    /// His report: *“trailing spaces/tabs/etc stops a search from finding
    /// text on the page that doesn't have these symbols … copy pasting from
    /// excel seems to give a trailing space that I have to remove to
    /// search.”* See [`query`] for what is trimmed, what is deliberately
    /// not, and why the bar discloses it either way.
    ///
    /// **Unlike [`Self::zoom_on_jump`], this one DOES change what
    /// matches**, and the difference decides where it is read. It is still
    /// not a [`FindOptions`] field — those are the bar's own menu and this
    /// is a Settings-window preference, which is where he asked for it
    /// (*“this should be an option in the settings”*) — but a change to it
    /// makes a standing result set wrong in exactly the way a changed query
    /// does, so the operator searches again. That is honest rather than
    /// awkward: a settings change is a deliberate act, not a keystroke.
    ///
    /// The **persisted** half is [`crate::app::prefs::Prefs::find_trim_query`],
    /// mirrored in once at startup, exactly as [`Self::zoom_on_jump`] is.
    trim_query: bool,
}

/// Hand-written rather than derived, for exactly one field.
impl Default for FindState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            options: FindOptions::default(),
            results: None,
            focus_wanted: false,
            zoom_on_jump: true,
            trim_query: true,
        }
    }
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

    /// Whether going to a hit may change the zoom. See
    /// [`Self::zoom_on_jump`](FindState#structfield.zoom_on_jump).
    #[must_use]
    pub fn zoom_on_jump(&self) -> bool {
        self.zoom_on_jump
    }

    /// Set the *Zoom* control.
    ///
    /// **Does not touch the results, and must not.** The three
    /// [`FindOptions`] controls make the standing hit list wrong, which is why
    /// changing one re-runs the search; this one does not change which glyphs
    /// matched. Clearing the results here would throw away a correct answer
    /// and make the operator search again to get the same list back.
    ///
    /// Two callers: `PdfcerApp::new`, mirroring the persisted preference in at
    /// startup, and the [`PrefAction::FindZoom`](crate::app::actions::prefs::PrefAction::FindZoom)
    /// arm, carrying the operator's
    /// click. Both go through here rather than writing the field so there is
    /// one place to read when the value is wrong.
    pub fn set_zoom_on_jump(&mut self, on: bool) {
        self.zoom_on_jump = on;
    }

    /// Whether whitespace at either end of the query is ignored. See
    /// [`Self::trim_query`](FindState#structfield.trim_query).
    #[must_use]
    pub fn trim_query(&self) -> bool {
        self.trim_query
    }

    /// Set the trim preference.
    ///
    /// **Clears the results, unlike [`Self::set_zoom_on_jump`]**, and the
    /// asymmetry is the point: this preference changes which text matches,
    /// so a stored hit list computed under the old value is wrong rather
    /// than merely stale. Leaving it standing would let the operator step
    /// through hits for a needle the setting says is no longer the needle.
    ///
    /// Two callers, the same two [`Self::set_zoom_on_jump`] has:
    /// `PdfcerApp::new` mirroring the persisted value in at startup, and the
    /// `Action::SetFindTrim` arm carrying the operator's tick.
    pub fn set_trim_query(&mut self, on: bool) {
        if self.trim_query == on {
            return;
        }
        self.trim_query = on;
        self.results = None;
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

    /// **What the readout says** — the pure rule, testable without a
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

/// **Run the search.**
fn search(state: &mut FindState, doc: &mut OpenDoc) {
    //
    // `state.query` keeps exactly what was typed, so the box still shows it
    // and the caret still behaves; only the value handed to the engine is
    // prepared. `query::for_search` borrows, so the ordinary search - no edge
    // whitespace, or the setting off - allocates nothing new before the
    // `to_owned` below, which is needed anyway because `state` is borrowed
    // mutably through the rest of this function.
    //
    // The emptiness test below now sees the TRIMMED query, and that is
    // deliberate rather than incidental: a box holding one space reads as
    // `Readout::Idle` (*“you have not typed anything”*) instead of running a
    // search that confidently reports nothing found in a document full of
    // spaces.
    let query = query::for_search(&state.query, state.trim_query).to_owned();
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
    // `search_text`, NEVER `find_text`. See this module's trap section:
    // `find_text` passes `with_wildcards(true)`, and a Find bar built on it
    // matches every character on the page when the operator types `?`.
    //
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

    // The line the cost claim in this module's header is made from, and the
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

/// **The wrap rule**, as a pure function of three numbers.
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
#[path = "mod_tests.rs"]
mod tests;
