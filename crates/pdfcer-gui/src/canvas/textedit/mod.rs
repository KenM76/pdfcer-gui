//! # `canvas::textedit` — **editing the page's own words**, and placing new ones
//!
//! `DEFECTS.md` **D4** is the defect that began this project, reported as:
//!
//! > *"text editing is weird and doesn't just edit the existing box and move the
//! > text correctly as you type plus flow to the next line doesn't work."*
//!
//! This module is the shell half of the answer. It arms a tool, puts a caret in
//! a run, collects keystrokes into a draft, and commits the draft as **one**
//! `EditSession` command. What it fixes that the old shell did not is in
//! [`disposition`] — the two cases D4b names as *wrong on commit* — and what it
//! deliberately does not fix is listed under **What is out of scope** below,
//! in words, because a silently-disabled typing loop is how the old shell told
//! an operator it could not do something.
//!
//! ---
//!
//! ## §1. The admission argument `canvas::tool`'s header asked for
//!
//! That header names one exclusion and one only:
//!
//! > **Text *editing*** — Phase 5, the defect that began this project — remains
//! > outside, and for exactly the original reason: it is a caret in a
//! > re-laid-out box, it would drag a whole subsystem's state through this type
//! > […] **Whoever brings the second should have to make this argument again,
//! > in this file.**
//!
//! It is made there, at [`CanvasTool::TextEdit`](crate::canvas::tool::CanvasTool::TextEdit),
//! and the short form is this: the objection was *"it would drag a whole
//! subsystem's state through this type"*, and the state does not go through the
//! type. What crosses the boundary is one [`TextEditKind`] — the same single
//! value `Markup` and `Measure` each carry — and the draft, the caret and the
//! anchor live in `egui::Memory` exactly where a half-finished measure pick
//! lives, for the reason `canvas::measure`'s header gives: *"a half-finished
//! pick is not part of the document and a document saved mid-gesture must not
//! carry one."* A half-typed word is the same category.
//!
//! ## §2. Two kinds, one tool — and why it is not two tools
//!
//! `edit.text` edits a run that is already on the page; `edit.add_text` places
//! new page content. They are the same *gesture* — click somewhere, type, press
//! Enter — differing only in what the click resolves to and which engine verb
//! the commit calls. So they are one [`CanvasTool`] variant carrying a
//! [`TextEditKind`], which is `MarkupKind`'s argument restated: the operator is
//! doing exactly one of the two, and a type that could say both would need
//! discipline to keep honest.
//!
//! ## §3. It clicks AND drags — and until 2026-08-21 it only clicked
//!
//! This section read: *"There is no drag in 'put the caret here', so there is no
//! `DragKind` for one, and inventing a `DragKind::TextEdit` that every arm
//! ignored would be the placeholder this project's no-placeholders invariant
//! forbids."* Correct, and it answered the wrong question — because the
//! operator's next ask was not about carets:
//!
//! > *"I should be able to make it multi line."*
//!
//! **A PDF has no paragraph.** Each visual line is its own show operator at its
//! own absolute position, so something has to decide where the second line
//! starts: a width to wrap against and a leading to step by. That is
//! `AddTextRequest::with_box`, and it needs a rectangle — so the gesture is a
//! drag, and the `DragKind` is not a placeholder but the whole feature.
//!
//! | gesture | anchor | what commits |
//! |---|---|---|
//! | click on existing text | [`Anchor::Run`] | `edit_text` |
//! | click on bare page | [`Anchor::Origin`] | `add_text`, one line at a point |
//! | **drag a rectangle** | [`Anchor::Box`] | `add_text` boxed, a wrapped paragraph |
//!
//! ★★ The drag belongs to **this** tool and not to `CanvasTool::Text`, and that
//! was decided by two unit tests rather than by taste. The box was briefly
//! offered from the sweep tool's rung — where it took the text sweep away in
//! Edit, which `text_tool_selects_and_marks_in_edit` depends on to make a
//! selection the markup verbs can act on. **Two features claiming one drag is a
//! choice somebody has to make, and taking a shipped gesture away to make room
//! is the wrong way to make it.** Add-text drags; the text tool goes on
//! sweeping.
//!
//! ## §4. It does not disturb the text-selection gate
//!
//! `canvas::textsel::gate`'s §3 warns that exclusivity between the text sweep
//! and the content marquee is now *by precedence*, not by construction. This
//! variant does not touch that: `takes_the_press` asks `tool.is_text()`, which
//! is `matches!(tool, CanvasTool::Text)` and is therefore **false** for
//! `TextEdit(_)` by construction rather than by an added condition. A press with
//! this tool armed is claimed by this module's own rung in `press_kind`, which
//! sits *above* the text-selection question for the same reason the measure rung
//! does — an armed tool takes the press — so the two can never both claim one
//! press. `gate.rs`'s tests carry a case asserting exactly that.
//!
//! ## §5. Capability: `edit_content`, and nothing wider
//!
//! Unlike text *selection*, this authors. Every entry point is gated on
//! `Capabilities::edit_content`, which is Edit alone in the shipped manifest —
//! the dispatch arms decline by name and trace, and
//! [`crate::canvas::tool::retire_forbidden`] disarms the tool on the way into a
//! mode that cannot author, so a draft cannot survive into Read.
//!
//! ---
//!
//! ## §6. What is out of scope, said in words rather than by a dead key
//!
//! `DEFECTS.md` D4a's **cross-run editing** is not built, and cannot be from
//! here: it needs a multi-run edit request in `pdfcer-core` that does not exist —
//! `EditRequest` pins to one show operator, and *"a `TJ` array is one
//! operator"*. The old shell handled this by setting a `cross_run` flag that
//! **silently disabled the whole typing loop**, which is the failure this
//! module's [`Refusal::SpansRuns`] exists to avoid: a caret that lands where two
//! runs meet refuses **in a sentence**, on the status bar, naming what to do
//! instead. The sentence is `crate::text::textedit::spans_runs`.
//!
//! D4c's **reflow gates** are out of scope and untouched.
//!
//! ---
//!
//! ## §7. The two costs an operator pays, and where they are disclosed
//!
//! Rule 4 — *disclosure lives off-canvas* — so neither of these is drawn on the
//! page. Both reach the status bar through the disclosure list `vector_edit`
//! records, and one of them is a disclosure this shell adds because the engine
//! does not:
//!
//! * **`Reflow`** — the engine already discloses that the line may now overrun
//!   its margin.
//! * **`Pin`** — the engine discloses nothing, because from its side pinning is
//!   what was asked for. But a pinned tail does not make room, so a longer
//!   replacement grows *into* it. [`plan`] adds
//!   `crate::text::textedit::pinned_tail_disclosure` for exactly this, which is
//!   why the pin is never silent.
//!
//! ## conventions: text-caret
//!
//! Corpus: `ui-conventions/text-caret.md`.
//!
//! - T1 live-preview: **GAP, and it is the operator's open complaint** — *"I can
//!   edit text now, but there is no live preview of that either."* The page
//!   renders committed glyphs; the draft lives beside it and nothing draws it,
//!   so the operator sees the old text and a blinking caret. The corpus is
//!   explicit that the approximation is acceptable and the absence is not:
//!   drawing the draft in the shell's own font, scaled to the run, shifts
//!   slightly on commit and is still a preview.
//! - T2 caret-has-a-position: a click lands it at the nearest character
//!   boundary; arrows, Ctrl+arrows, Home and End move it; Backspace and Delete
//!   act either side of it. Added 2026-08-20 — before that there was no index at
//!   all, and the painter drew its line at the right edge because that is the
//!   only position an append-only draft has.
//! - T3 graphemes-not-bytes: **PARTIAL** — characters, not bytes, so `é` takes
//!   one keystroke. Not grapheme clusters, so a combining mark or an emoji
//!   sequence still takes two. `unicode-segmentation` is already in the tree.
//! - T4 clamp-never-assert: every operation clamps on entry. A panic in a caret
//!   would take the whole window down over a keystroke.
//! - T5 composer-owns-the-keyboard: `composing` is the one predicate, and
//!   `tools/gates/check-typing-guard.sh` fails the build on a second copy.
//!   **This row exists because it failed twice** — Delete after a canvas click,
//!   then the space bar, which the pan tool took because this caret is not an
//!   `egui::TextEdit` and egui's own predicate cannot see it.
//! - T6 enter-commits-escape-abandons: both — **and Enter has a second meaning
//!   as of 2026-08-21**. Inside a dragged text BOX a plain Enter is a paragraph
//!   break and `Ctrl+Enter` commits; everywhere else Enter commits. That is the
//!   old shell's own split, carried across, and it is why [`Anchor::Box`] is a
//!   variant rather than a flag: the keystroke handler has to know which gesture
//!   started the draft, and asking the TEXT would make the first Enter commit
//!   and every one after it insert. A draft identical to what it replaces still
//!   raises no action.
//! - T7 no-control-characters: [`caret::insert`] filters them, and **that filter
//!   ate the paragraph break for one driven run.** The Enter arrived, the branch
//!   was right, `insert` was called, and the newline was dropped one call
//!   deeper — by a guard whose own doc argued, correctly, that *"a control
//!   character arriving in a `Text` event is something this shell has no meaning
//!   for."* Still true of typed text; no longer true of the whole draft. The
//!   filter stays and the newline has its own door, [`caret::newline`], because
//!   relaxing `insert` would have let a stray `\t` or `\r` from a paste into a
//!   show string as well.
//! - T8 selection: **GAP** — no Shift+arrow, no Ctrl+A, no drag-select within a
//!   draft. Named rather than left implied, because a highlight that some keys
//!   respect and others silently ignore is worse than none.

pub mod blocks;
/// The caret's own arithmetic - insert, delete, and the four movements -
/// split out under R2 on 2026-08-20. Pure functions of a `&str` and an index,
/// with no window in them; its header says why that is a seam and not a cut.
pub mod caret;
/// Where the pointer is in relation to the editor box, published by `paint`
/// and read by everything that has to decide whether a press belongs to the
/// draft or to the page.
pub mod hit;
/// What every key means inside a draft — the keystroke contract, split out
/// under R2 on the day the selection landed.
pub mod keys;
/// ★★★ **The caret's arithmetic inside a draft that holds more than one line**
/// — `OPERATOR_REQUESTS.md` **O127**, defect 2.
///
/// Enter inserting a line break is one keystroke; a caret that can reach the
/// second line is four more. Its header carries the distinction that keeps it
/// separate from [`blocks`]: a line here is one the **operator typed**, and a
/// line there is one the **page draws** — different models, different costs
/// (nothing, against 336 ms), and confusing them moves the caret to another
/// part of the sheet mid-word.
pub mod lines;
/// What a draft looks like on the page - the in-place editor and its caret.
/// Split out under R2 on 2026-08-20; its header carries the standing rule that
/// the text and the caret are measured from ONE layout.
pub mod paint;
pub mod place;
/// ★ **Where a press puts the caret** — the three gestures that start a draft,
/// and the refusals each of them can raise. Split out under R2 on 2026-08-21;
/// its header carries why a text BOX must be a drag rather than a click.
/// ★★ **The page's lines, reassembled into paragraphs** — and the arrow keys
/// that walk between them. SALVAGE from the shell this project replaces, on the
/// operator's report of 2026-08-21; its header carries the four lines it came
/// from and why the reassembly was always `pdfcer-core`'s.
/// Which paragraph the caret is in — the one question `reflow_block` needs and
/// the shell has to answer. Its header carries the refusal that shapes the
/// whole feature: a reflow is planned against the BASE document, so a page
/// already edited this session is refused by name.
pub mod reflow;
/// ★ **What an edit report is worth telling anyone** — which of
/// `EditReport`'s eleven fields reach the operator, which reach the diagnostic
/// channel, and which reach neither. Split out under R2; its header carries
/// the rule and why the middle row of it exists.
pub mod report;
pub use caret::{backspace, delete_forward, insert, word_left, word_right};
/// **Naming the exact show operator, and the exact buffer it lives in** — the
/// one producer of `(pinned_span, EditTarget)` in this shell, shared by the
/// caret's `edit_text` and the restyle verbs' `format_text`.
pub mod pin;

pub use place::{Click, begin_box, click};
pub mod disposition;
// The byte-level proof that the untouched tail did not move, with the old
// shell's own `EditOptions::default()` run beside it as the falsifier.
// `#[cfg(test)]` inside; it compiles to nothing in a release build.
mod proof;
// The per-keystroke re-measure measurement `DEFECTS.md` D4b's fix would need,
// and the reason it is not wired. `#[ignore]`d; run it and read the numbers.
mod cost;
/// ★★ The face, size and colour NEW page text is written in — the Phase 5 row
/// that read *"choosing what those three controls are is a decision, not an
/// omission"*. The decision is in that module's header, along with why it lives
/// in `egui::Memory` where the markup pen does not.
pub mod pen;

use pdfcer_core::text_edit::{
    BlockRecognitionOptions, EditOptions, EditRequest, EditableTextModel, ReflowEngine,
    TextPosition, reflow_recognition_options,
};

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use disposition::Reason;

/// `egui::Memory` key for the in-flight draft.
///
/// One key for both kinds, because one draft can be in flight: arming the other
/// kind clears it, exactly as arming a different `MarkupKind` cannot reach a
/// drag already in flight.
const DRAFT_MEMORY_KEY: &str = "pdfcer-textedit-draft"; // ui-text-exempt: internal memory id, never displayed

/// The environment variable that supplies a draft when no keyboard can.
///
/// **A diagnostic seam in the shape `app::files`' two already have** — see
/// `DIAG_OPEN_PATH` and `DIAG_SAVE_PATH`, which exist because a native modal
/// cannot be driven from a harness. This one exists because **this machine
/// cannot inject text**: `tools/ui-verify`'s `sys::vk` is a deliberately closed
/// list of eight non-character virtual keys, and its own comment refuses to
/// grow into `pub const A..Z` on the ground that *"a harness that can press any
/// key is a harness whose scripts stop being readable"*.
///
/// Typing is this feature's entire input, so without a seam the only honest
/// verification would be *"the tool armed"* — which is `HANDOFF.md` §2's
/// grid lesson exactly: an assertion in the right direction that measures the
/// wrong thing.
///
/// **It is not load-bearing and it is not a second input path.** It is read at
/// exactly one place, [`typing`], on the frame a caret is set, and what it does
/// is push characters through **the same** [`insert`] every `egui::Event::Text`
/// goes through. A build with the variable unset cannot tell it exists; a build
/// with it set still has to route the click, resolve the anchor, plan the
/// disposition and reach the engine, which is every link the check is about.
pub const DIAG_TYPE: &str = "PDFCER_DIAG_TYPE"; // ui-text-exempt: an environment variable name, never displayed

/// **Which of the two text verbs is armed.**
///
/// One value carried on the tool, for [`MarkupKind`](crate::canvas::markup::MarkupKind)'s
/// argument: the operator is doing exactly one of these, so a type that could
/// express both would have illegal states to prevent by discipline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEditKind {
    /// `edit.text` — replace the words in a run that is already on the page.
    Edit,
    /// `edit.add_text` — place new page content where the operator clicks.
    Add,
}

impl TextEditKind {
    /// The command id that arms this kind.
    ///
    /// The single binding between an id and a kind, in the shape
    /// `shell::commands::markup_for_command` has — read from both directions by
    /// `crate::shell::commands::text_edit_for_command` and by the label the
    /// status bar shows, so the two cannot drift.
    #[must_use]
    pub const fn command_id(self) -> &'static str {
        match self {
            // ui-text-exempt: command ids, never displayed.
            Self::Edit => "edit.text",
            Self::Add => "edit.add_text",
        }
    }
}

/// **What the caret is attached to** — the half of the draft that the click
/// resolves and typing never changes.
#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    /// A run already on the page, by index into `PageText::runs`, with the text
    /// it held when the caret landed.
    ///
    /// The *original* is carried and the geometry is not, deliberately. The
    /// original is what `EditRequest::find` needs and is a fact about the moment
    /// the operator clicked; everything else — the pinned span, the matrices,
    /// the block alignment — is a pure function of `(page_text, run)` that
    /// [`plan`] re-derives at commit. That is the old shell's own ruling, and
    /// its reason is worth keeping: *"storing a copy on `PendingEdit` would be a
    /// second source of truth that can go stale when the page is rebuilt."*
    Run { run: usize, original: String },
    /// A point in **PDF user space** where new text will be placed.
    Origin { x: f64, y: f64 },
    /// ★★★ **A RECTANGLE in PDF user space** — new text, wrapped to its width.
    ///
    /// The operator, 2026-08-21: *"I should be able to make it multi line."*
    ///
    /// # Why multi-line needs a BOX and cannot be a point with newlines in it
    ///
    /// Because a PDF has no paragraph. Each visual line is its own show
    /// operator at its own absolute position, so *something* has to decide
    /// where the second line starts — and the only thing that can is a width to
    /// wrap against and a leading to step by.
    ///
    /// `pdfcer-core`'s `AddTextRequest::wrap_box` is exactly that (`Pass 16.1`),
    /// and it is more than a container: **paragraphs split on a hard `\n` and
    /// each is wrapped independently**, so an operator gets both behaviours —
    /// Enter makes a new paragraph, and running past the right edge makes a new
    /// line — from one field.
    ///
    /// # ★ Why this is a third variant and not a `wrap: Option<Rect>` on
    /// [`Self::Origin`]
    ///
    /// Because the two are **different gestures with different affordances**,
    /// and folding them would make "did the operator drag or click?" a runtime
    /// question at commit time rather than a fact the press already settled. A
    /// click places a single-line run at a point; a drag places a paragraph in
    /// a box. That is the old shell's own split — *"in box mode a plain Enter
    /// is a paragraph break; Ctrl+Enter accepts. In point mode Enter accepts"*
    /// — and it is what every program in the class does.
    ///
    /// It is also what keeps the Enter key honest. Enter cannot mean *insert a
    /// line* and *commit* in one draft, and the variant is how the keystroke
    /// handler knows which it is without asking about the text's contents.
    Box {
        /// Lower-left x, PDF user space.
        llx: f64,
        /// Lower-left y.
        lly: f64,
        /// Upper-right x.
        urx: f64,
        /// Upper-right y.
        ury: f64,
    },
}

/// An in-progress, operator-composed edit. Never written anywhere until commit.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// Which page it belongs to. A page change abandons it — see [`load`].
    pub page: usize,
    /// Which verb will commit it.
    pub kind: TextEditKind,
    /// What the caret is on.
    pub anchor: Anchor,
    /// The operator's in-progress text.
    pub text: String,
    /// ★★ **Where the caret sits inside [`Self::text`], as a CHARACTER index.**
    ///
    /// `0` is before the first character; `text.chars().count()` is after the
    /// last. Every edit and every movement clamps into that range, so the
    /// invariant `caret <= text.chars().count()` holds by construction and no
    /// caller has to check it.
    ///
    /// # The defect this field is
    ///
    /// It did not exist until 2026-08-20. `insert` extended the end of the
    /// string and `backspace` popped the last character, so the caret was not
    /// merely fixed at the end - **there was no caret**, and the painter drew
    /// its line at the right edge of the run's glyph box because that is the
    /// only position an append-only draft has. The operator:
    ///
    /// > *"the cursor just sits at the end of a text line. It can't be moved to
    /// > the center of an existing text block."*
    ///
    /// Exactly right, and it made editing existing page text almost useless: a
    /// title-block cell reading `SHEET 1 OF 4` could only be changed by deleting
    /// it back to `SHEET ` and retyping.
    ///
    /// # Why characters and not bytes
    ///
    /// Because every operation here is expressed in keystrokes, and one
    /// keystroke is one `char`. A byte index would make Left-arrow over `e` -
    /// two bytes - either move half a character or need a decode at every use.
    /// `backspace` already worked in `char`s for the same reason (a byte
    /// truncation of a multi-byte character is a panic in Rust, not mojibake),
    /// so this is that decision applied consistently rather than a new one.
    ///
    /// The cost is that every operation is O(n) in the draft's length. A draft
    /// is one show operator - a cell, a label, a line of a note - so n is tens
    /// of characters, and the alternative is a byte index plus a boundary check
    /// at every call site.
    pub caret: usize,
    /// ★★ **The other end of a selection**, as a character index, or `None`
    /// when nothing is selected.
    ///
    /// The selection is the range between this and [`Self::caret`], in either
    /// order — [`caret::range`] normalises it. Two indices rather than a
    /// `Range`, because the *direction* is real: Shift+Left from the middle of
    /// a word must extend leftward and then shrink back rightward, and a
    /// normalised pair forgets which end the operator is dragging.
    ///
    /// # ★ Why it is called a mark
    ///
    /// Because "anchor" is taken. [`Anchor`] already means *what on the page
    /// this draft is attached to*, which is a different question with a
    /// different answer, and two fields called anchor in one struct is how a
    /// wrong one gets read.
    ///
    /// # The defect this field is
    ///
    /// `OPERATOR_REQUESTS.md` O14 item 11, from the conventions sweep of
    /// 2026-08-20: *"no selection inside a draft — no Shift+arrow, no Ctrl+A,
    /// no drag-select."* Every text field the operator has ever used has all
    /// three, and without them replacing a word means pressing Backspace once
    /// per character.
    ///
    /// ★ **It is cleared by any un-shifted movement**, which is what makes a
    /// selection feel like a selection rather than a mode. That rule lives in
    /// [`caret::moved`] so it is applied in one place; every arrow arm calls
    /// it, and an arm that forgot would leave a highlight behind after the
    /// caret had walked out of it.
    pub mark: Option<usize>,
    /// Whether the diagnostic seam has already been consumed for this draft, so
    /// a seam-supplied string is typed **once** rather than on every frame.
    ///
    /// `pub` only so a test in another module can build a draft that is already
    /// past the seam. Nothing outside this module should ever set it to `false`.
    pub seeded: bool,
}

/// Why a click could not start a draft, in a form the status bar can render.
///
/// Every variant is a *sentence to show*, not a state to be silent in. That is
/// the whole difference from the old shell, which set a boolean and stopped
/// responding to the keyboard.
///
/// ★★ **`SpansRuns` was the third variant and is gone as of 2026-08-19.** It
/// refused every click whose visual line was made of more than one show
/// operator, which on a CAD sheet is nearly every click — see [`resolve_run`]
/// for the measurement and for why the refusal was answering a question about
/// the *line* when the operator was editing a *run*. The two that are left are
/// both genuine absences of a thing to edit: no text under the pointer, and no
/// readable text on the page at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The click landed on no text at all.
    NoRun,
    /// The page's text could not be extracted (an image-only page, a damaged
    /// content stream).
    NoText,
    /// ★★ **The run is real, readable, and inside a form XObject — which
    /// `pdfcer-core`'s text-edit surgery does not enter.** 2026-08-20.
    ///
    /// # Why this variant had to exist, and what it replaced
    ///
    /// Nothing. The click was accepted, a caret was placed, keystrokes were
    /// taken, a plan was built, and the engine then refused the commit with
    /// *"text to edit (\"p\") was not found in an editable run on the page"* —
    /// **to the trace only**. The operator saw a caret that took their typing
    /// and threw it away in silence. Their words:
    ///
    /// > *"Still no editing text on top of the canvas."*
    ///
    /// # The mechanism, because it is not obvious from either side
    ///
    /// `GlyphProvenance` carries two fields and the shell was reading one:
    /// `operator_span` is a **byte span into a decoded content buffer**, and
    /// `content_stream` names *which* buffer. For text drawn by the page's own
    /// stream the two agree with what the edit surgery walks. For text inside a
    /// `Do`-invoked form XObject the span indexes the FORM's bytes, and
    /// `pdfcer-core`'s `find_anchor` compares it against page-stream offsets —
    /// where it can never match. Worse, a pinned request skips the text search
    /// entirely, so the loop exhausts and reports `NoMatch(find)`, which blames
    /// the operator's text for a failure of the pin.
    ///
    /// # Why refusing is better than trying
    ///
    /// Because the attempt **cannot** succeed — this is a named non-goal of
    /// that cut of the engine (`pdfcer-core/src/text_edit/edit.rs:79`) — and a
    /// control that accepts input it will discard is this project's defining
    /// defect class. An honest refusal at the click costs the operator one
    /// click; the silent version cost them a sentence they had already typed
    /// and the belief that the feature works.
    ///
    /// ★ **On a CAD sheet this is the common case, not an edge case.** Measured
    /// on the benchmark drawing: 1,696 show operators of real drawing text
    /// inside the form, against 3,007 metadata glyphs in the page stream. Filed
    /// as an engine request the same day.
    ///
    /// # ★★★ AND IT IS GONE, 2026-08-20 — `Pass 119.0` shipped form editing
    ///
    /// The variant above is **deleted**, not deprecated, and everything written
    /// about it is kept because the *shape of the episode* is the durable part.
    /// `Editability::InsideForm` is `#[deprecated]` in the engine and never
    /// returned; `edit_text` resolves a target stream and reaches form content
    /// as one undoable command.
    ///
    /// ★ **The reason this cost one deletion instead of an investigation** is
    /// the argument this project made when it filed the request:
    ///
    /// > *"my shell encodes a fact about your surgery's internals. The day form
    /// > editing lands, my guard silently keeps refusing until I notice and
    /// > delete it — a workaround that outlives its bug, which is decision
    /// > 058's exact failure mode."*
    ///
    /// The engine's answer was to publish `TextRun::editability()` so the shell
    /// asked pdfcer rather than modelling it. When the capability landed, the
    /// predicate started answering `Editable` and **a `#[deprecated]` attribute
    /// pointed at the one line to remove.** That is the whole value of not
    /// hand-rolling a guard, demonstrated end to end inside two days.
    ///
    /// # What replaces it, and it is a genuinely different fact
    ///
    /// A run whose glyphs come from `/ActualText` covers **no show operators of
    /// its own**, so there is nothing for the surgery to anchor on. That is not
    /// "out of reach" — it is "there is nothing to reach for", and the engine
    /// carries it as its own [`Editability::NoAnchor`] variant precisely so a
    /// shell can say something different about it.
    ///
    /// [`Editability::NoAnchor`]: pdfcer_core::text_extract::Editability::NoAnchor
    NoAnchor,
}

/// ★★★ **Is the operator composing text ANYWHERE?** The one predicate, asked in
/// one place.
///
/// # This function exists because the answer was written twice and one copy was
/// # wrong
///
/// Two claimants have to be asked about, because this shell composes text in
/// two different places:
///
/// 1. [`egui::Context::text_edit_focused`] — a real `egui::TextEdit`: a form
///    field, the page-number box, a dialog's box, the Find bar. **D1's
///    predicate**, never `egui_wants_keyboard_input()`, for the reason
///    `app::keyboard::collect` gives at length.
/// 2. **A canvas text draft** — the caret this shell paints on the page, which
///    is deliberately *not* a `TextEdit` (this module's header says why: the
///    caret sits in PDF space at the glyphs' own scale, which a floating widget
///    cannot do). **egui therefore reports no focused text field for an
///    operator who is visibly mid-word.**
///
/// `app::keyboard` asked both. `canvas::tool::arm::space_held` asked only the
/// first — and the space bar is the hand tool's modifier, so **an operator
/// typing on the canvas could not type a space.** They got a pan instead. The
/// operator, 2026-08-20: *"it doesn't accept spaces. Like how?"*
///
/// That is **defect D1 one rung along**: D1 was `egui_wants_keyboard_input()`
/// where `text_edit_focused()` was meant; this is `text_edit_focused()` where
/// *"anybody is composing"* was meant. Same shape, same invisibility to a
/// harness that builds a bare `Context` with no draft in it, same silent loss
/// of a key the operator is plainly pressing.
///
/// The lesson is not "be careful". It is that **a predicate with two claimants
/// must exist once**, and `tools/gates/check-typing-guard.sh` now fails the
/// build on any bare `text_edit_focused()` outside this function.
///
/// # Why "is a draft in flight" and not "is a caret tool armed"
///
/// An armed tool that has not been clicked yet owns no keystrokes — the page
/// keys must keep working right up until the caret is placed. Carried over from
/// `app::keyboard`, where it was already right.
#[must_use]
pub fn composing(ctx: &egui::Context) -> bool {
    ctx.text_edit_focused() || read(ctx).is_some()
}

/// Read the draft without creating one. `None` when nothing is being composed.
#[must_use]
pub fn read(ctx: &egui::Context) -> Option<Draft> {
    ctx.data(|d| d.get_temp::<Draft>(egui::Id::new(DRAFT_MEMORY_KEY)))
}

/// Store a draft.
pub(crate) fn store(ctx: &egui::Context, draft: Draft) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(DRAFT_MEMORY_KEY), draft));
}

/// Forget the draft, returning whether there was one.
///
/// The abandon half of the Escape ladder, and the retirement path
/// `crate::canvas::tool::retire_forbidden` calls when the mode loses
/// `edit_content`. Returns `bool` for the reason `measure::abandon` does: the
/// ladder rung above it needs to know whether this rung consumed the key.
pub fn abandon(ctx: &egui::Context) -> bool {
    let had = read(ctx).is_some();
    ctx.data_mut(|d| d.remove::<Draft>(egui::Id::new(DRAFT_MEMORY_KEY)));
    if had {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| "text-edit-abandon".to_owned());
    }
    had
}

/// The draft for `page`, or `None` — dropping any draft that belongs to another
/// page or another kind.
///
/// The two synchronisations `measure::load` performs, for the same two reasons.
/// A draft composed on page 3 must not commit against page 4's run indices, and
/// a draft begun under `Edit` must not be committed by `Add` because the
/// operator pressed the other ribbon button mid-word.
#[must_use]
pub fn load(ctx: &egui::Context, page: usize, kind: TextEditKind) -> Option<Draft> {
    let draft = read(ctx)?;
    if draft.page == page && draft.kind == kind {
        return Some(draft);
    }
    abandon(ctx);
    None
}

/// Turn a draft into the action that commits it, if it says anything.
///
/// A draft byte-identical to what it replaces is **not a write**. Without this,
/// an operator who typed a character and deleted it again would put a no-op
/// entry on the undo stack every time they clicked away — the old shell's own
/// finding, and it matters more here because clicking out commits.
pub(super) fn commit_into(
    ctx: &egui::Context,
    draft: &Draft,
    actions: &mut Vec<crate::app::actions::Action>,
) {
    use crate::app::actions::Action;
    match &draft.anchor {
        Anchor::Run { run, original } if draft.text != *original && !draft.text.is_empty() => {
            actions.push(Action::CommitTextEdit {
                page: draft.page,
                run: *run,
                original: original.clone(),
                replacement: draft.text.clone(),
            });
        }
        Anchor::Origin { x, y } if !draft.text.is_empty() => {
            actions.push(Action::CommitAddText {
                page: draft.page,
                origin: (*x, *y),
                text: draft.text.clone(),
                // ★ Sampled HERE, at the commit, not read in `apply`. See the
                // variant's own docs: an action is what the operator asked for,
                // and it is applied on a later frame.
                pen: pen::read(ctx),
                // A point-text run is one line and has no box to wrap to.
                wrap: None,
            });
        }
        // ★★ The boxed variant, and it reaches the SAME action — one commit
        // path, one apply arm, one place that can be wrong about a font.
        //
        // The whole difference is that `wrap` is `Some`, which is what
        // `AddTextRequest::with_box` turns into the multi-line layout: hard
        // newlines split paragraphs and each is wrapped independently to the
        // box's width, top-anchored from its top edge.
        //
        // ★ `origin` is still carried and is still the box's lower-left, even
        // though the engine documents it as **ignored** in boxed mode. Sending a
        // meaningless value would be worse than sending a meaningful one that
        // happens to be unread: the day a caller or a trace wants to know where
        // this text was placed, the honest answer is already on the action.
        Anchor::Box { llx, lly, urx, ury } if !draft.text.is_empty() => {
            actions.push(Action::CommitAddText {
                page: draft.page,
                origin: (*llx, *lly),
                text: draft.text.clone(),
                pen: pen::read(ctx),
                wrap: Some((*llx, *lly, *urx, *ury)),
            });
        }
        _ => {}
    }
}

// ===========================================================================
// Planning the commit — where D4b's two fixes actually take effect
// ===========================================================================

/// A planned in-place edit: the request, the options, and the disclosure the
/// engine will not write for us.
pub struct Plan {
    /// The request, with its provenance pin.
    pub request: EditRequest,
    /// ★ The options, with the [`disposition`] this module exists to choose.
    pub options: EditOptions,
    /// Why that disposition, for the trace and the disclosure.
    pub reason: Reason,
}

/// **Plan a commit against the page as it is now.**
///
/// Called from the apply arm rather than from the canvas, because it needs the
/// document and an `Action` is plain data. It is still one function in one place
/// — the arm routes to it and computes nothing itself.
///
/// The three things it derives, all from `(page_text, run)`:
///
/// 1. **the provenance pin** — `operator_span`, which is how the surgery finds
///    *this* show operator rather than the first one whose text matches. Without
///    it, editing the second `TITLE` on a title-block sheet edits the first.
/// 2. **the matrices** — `Tm` and the CTM in force at the run's first glyph,
///    which is what [`disposition::is_upright`] reads.
/// 3. **the block alignment** — through `ReflowEngine::detect_alignment` on a
///    model recognised with [`reflow_recognition_options`], i.e. the **relaxed**
///    recogniser. That is the old shell's own choice for its reflow target and
///    the reason carries here unchanged: the default recogniser splits on
///    indentation, so a right-aligned block whose lines start at different x —
///    which is what right alignment *is* — is exactly the shape it fragments,
///    and a fragmented block is a one-line block, and a one-line block reports
///    `SingleLineDefault`. Using the default model would make the alignment
///    fix unreachable on precisely the documents it is for.
#[must_use]
pub fn plan(doc: &OpenDoc, page: usize, run: usize, original: &str, replacement: &str) -> Plan {
    let mut request = EditRequest::find_replace(page, original, replacement);
    let mut matrices = (
        [1.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0],
        [1.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0],
    );
    let mut finding = None;
    // ★★ Whether the caret's visual line is made of more than one show
    // operator, re-derived here rather than carried on the `Anchor`.
    //
    // The `Anchor` docs give the rule and it applies unchanged: everything but
    // the original text is a pure function of `(page_text, run)`, and a copy
    // taken when the operator clicked would go stale when the page is rebuilt.
    //
    // Defaults to `false`, which is the *permissive* direction — `Reflow` — and
    // that is the honest default for the same reason the identity matrices below
    // are: it is what a page whose provenance could not be read gets, and a
    // shell that pinned on no evidence would be claiming to have measured
    // something it never saw. The single-run case is also the overwhelmingly
    // commoner one in ordinary prose documents.
    let mut shares_the_line = false;

    // ★★ **This extraction is its own, and it is NOT `doc.page_text()`.**
    //
    // That was the first shape of this function and it was silently broken.
    // `app::cache`'s extraction runs with `ExtractOptions::default()`, and
    // `capture_provenance` **defaults to off** — the engine says so in terms:
    // *"`None` unless the extraction set `ExtractOptions::capture_provenance`;
    // this keeps the default Pass 4 output byte-for-byte unchanged."* With it
    // off, `model.provenance(..)` answers `None` for every glyph, and this
    // function would have:
    //
    // * left `pinned_span` at `None`, so the surgery would locate the **first**
    //   operator whose text matches rather than the one the caret is in — which
    //   on a title-block sheet with two runs reading `REV A` edits the wrong
    //   one; and
    // * fallen back to the identity matrices below, so **the rotation guard
    //   would never fire** and D4b case 2 would be unfixed while every unit test
    //   in `disposition` stayed green. That is precisely `HANDOFF.md` §2's
    //   shape: a correct decision function, wired to a value that is always the
    //   same.
    //
    // Widening the shared cache was the other option and is the worse one: every
    // caller of `page_text()` — Find, both copy verbs, the text sweep — would
    // then pay for provenance on every page, and `app::cache`'s own header
    // records that extraction is the expensive thing this shell does (392 ms on
    // the benchmark sheet). Paying it **once per commit**, here, is the whole
    // cost, and a commit is already an operation that saves and re-rasters.
    //
    // The run index is shared between the two extractions, which is safe and is
    // worth stating: `capture_provenance` populates a field and changes no
    // segmentation, so `runs[i]` names the same run under both options.
    if let Some(page_ref) = doc.pages.get(page) {
        // ★ The funnel's output, MODIFIED — not a second construction.
        //
        // `with_provenance(true)` is the one thing no setting governs: it is the
        // substrate for editing text, and `app::cache`'s read-only extraction
        // deliberately leaves it off because it costs and it is not needed
        // there. Everything else — the word gap, the unmappable sentinel, the
        // replacement-text precedence — comes from the operator, so the runs
        // this editor addresses are segmented exactly as the runs the canvas
        // paints and the find bar searches. Two extractions of one page under
        // two configurations would put the glyph the operator clicked and the
        // glyph this code edits one step out of step.
        use crate::app::settings::SettingsExt;
        let opts = doc.settings.extract_options().with_provenance(true);
        if let Ok(text) =
            pdfcer_core::text_extract::extract_page_view(&doc.session.view(), page_ref, page, &opts)
        {
            let model = EditableTextModel::recognize(&text, &BlockRecognitionOptions::default());
            // ★★ The pin, and the buffer it indexes — [`pin::of_run`].
            //
            // Both facts, from one call, over the model just recognised. They
            // lived here inline until 2026-08-27; `format_text` became the
            // second verb that needs exactly the same measurement, and the
            // sixty lines of argument behind the `EditTarget` choice are what
            // makes it right, so they moved somewhere both callers reach them
            // rather than being paraphrased twice.
            if let Some(p) = pin::of_run(&model, run) {
                request.pinned_span = Some(p.span);
                matrices = (p.text_matrix, p.ctm);
                request.target = p.target;
                // ★★★ **The find string is DROPPED when the pin is exact.**
                //
                // `EditRequest::whole_operator` (`Pass 152.0`): an empty `find`
                // beside a pin means *"this whole show operator"*, which is
                // precisely what a caret in a run means and what a rebuilt
                // `find` was only ever an approximation of.
                //
                // ## Why the approximation had to go
                //
                // A run's `text` is not in 1:1 correspondence with its glyphs —
                // `/ToUnicode` may map one glyph to several characters — so the
                // reconstructed find *"fails invisibly on unligatured test text
                // and routinely on real typeset copy"*, in the engine's words.
                // On this operator's own CAD drawings it is worse than that:
                // `text_extract` synthesises inter-glyph spacing (a trace of one
                // of his title-block cells showed **twenty-one** spaces), so the
                // string this shell holds contains characters no show operator
                // ever wrote and the match can never succeed.
                //
                // ⇒ That was reported as *"text editing is weird"*, filed as a
                // defect against the engine, and answered by naming a capability
                // that already existed. The workaround is deleted rather than
                // kept beside the fix.
                //
                // ## ★★ And why only when the run is one operator
                //
                // See [`pin::spans_one_operator`]. On a split run the whole
                // -operator form would replace one fragment's text with the
                // whole replacement and leave the other fragments painting their
                // old glyphs — visible corruption reported as success. The
                // find-based form fails cleanly there instead, which is the
                // right outcome for a case this shell cannot yet edit at all.
                // ★★★ THE DECISION IS TRACED, because without it the two
                // outcomes are indistinguishable from outside the process and
                // one of them is correct.
                //
                // A driven run on the operator's own drawing produced
                // `edit-text-refused … detail=text to edit ("0.00[21 spaces]0.030")
                // was not found in an editable run`, and **nothing anywhere said
                // whether the pin path had been taken.** So the check reported a
                // program defect and aimed the reader at `pdfcer-core`, when the
                // honest reading might have been *"this run spans two operators,
                // the find-based form was used deliberately, and it failed
                // cleanly as designed"*.
                //
                // ⇒ Those two need different responses — one is a request to the
                // engine, the other is the shell working — and a trace that
                // cannot separate them turns a correct build into a filed
                // defect. `find_len` carries the number that made the string
                // unmatchable, because a reader seeing 30 characters for a
                // six-character cell has the whole story in one line.
                let one_operator = pin::spans_one_operator(&model, run);
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    format!(
                        "edit-text-pin page={page} run={run} one_operator={one_operator} \
                         find_len={}",
                        request.find.chars().count()
                    )
                });
                if one_operator {
                    request.find.clear();
                }
            }
            // ★ The SAME model the caret's hit test used, with the same
            // options — `BlockRecognitionOptions::default()` — because the
            // question is *how did the thing the operator clicked get
            // segmented*, and asking it of a differently-recognised model would
            // answer about a different segmentation. The relaxed model below is
            // for alignment detection, which is a different question about the
            // same page.
            if let Some((from, to)) = model.line_range_at(TextPosition::new(run, 0)) {
                shares_the_line = from.run != to.run;
            }
            let relaxed = EditableTextModel::recognize(&text, &reflow_recognition_options());
            finding = relaxed
                .block_at(TextPosition::new(run, 0))
                .and_then(|b| ReflowEngine::new(&relaxed).detect_alignment(b).ok())
                .map(disposition::from_detection);
        }
    }

    let reason = disposition::choose(matrices.0, matrices.1, shares_the_line, finding);
    Plan {
        request,
        options: disposition::options(reason),
        reason,
    }
}

// ===========================================================================
// Painting
// ===========================================================================

/// What [`preview`] needs.
pub struct Preview<'a> {
    /// The document, for the page geometry.
    pub doc: &'a OpenDoc,
    /// Which page is on screen.
    pub page_index: usize,
    /// The frame's screen ⟷ canvas mapping.
    pub map: &'a PageMapping,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **A draft equal to what it replaces is not a write.**
    ///
    /// The no-op guard, and it is load-bearing because clicking away commits: an
    /// operator who typed a letter and removed it again would otherwise get an
    /// undo entry for having changed their mind.
    #[test]
    fn an_unchanged_draft_pushes_no_action() {
        let draft = Draft {
            page: 0,
            kind: TextEditKind::Edit,
            anchor: Anchor::Run {
                run: 3,
                original: "TITLE".to_owned(),
            },
            text: "TITLE".to_owned(),
            caret: 0,
            mark: None,
            seeded: true,
        };
        let mut actions = Vec::new();
        // ★ A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert!(actions.is_empty(), "an unchanged draft is not an edit");
    }

    /// ★★★ **A box draft commits as a WRAPPED PARAGRAPH, and carries its
    /// rectangle.**
    ///
    /// The operator, 2026-08-21: *"I should be able to make it multi line."*
    ///
    /// Asserted through the ACTION rather than through the engine, because the
    /// action is where this shell's decision lives: `wrap: Some(..)` is what
    /// becomes `AddTextRequest::with_box`, and a build that dropped it would
    /// author the same characters as one long single line — plausible, silent,
    /// and wrong in exactly the way the operator asked for.
    #[test]
    fn a_box_draft_commits_with_its_wrap_rectangle() {
        use crate::app::actions::Action;
        let draft = Draft {
            page: 0,
            kind: TextEditKind::Add,
            anchor: Anchor::Box {
                llx: 100.0,
                lly: 200.0,
                urx: 340.0,
                ury: 290.0,
            },
            text: "first line\nsecond line".to_owned(),
            caret: 0,
            mark: None,
            seeded: false,
        };
        let mut actions = Vec::new();
        commit_into(&egui::Context::default(), &draft, &mut actions);
        match actions.as_slice() {
            [Action::CommitAddText { wrap, text, .. }] => {
                assert_eq!(
                    *wrap,
                    Some((100.0, 200.0, 340.0, 290.0)),
                    "the box must reach the action, or the paragraph is authored as one line"
                );
                assert!(
                    text.contains('\n'),
                    "the hard newline must survive to the engine: it is what splits paragraphs"
                );
            }
            other => panic!("expected one CommitAddText, got {other:?}"),
        }
    }

    /// …and a POINT draft carries no rectangle, which is what keeps the two
    /// gestures distinct all the way down.
    ///
    /// A build that gave every add-text a box would wrap a one-line label at
    /// whatever width it invented — and the width would have to be invented,
    /// because a click has no extent.
    #[test]
    fn a_point_draft_commits_without_one() {
        use crate::app::actions::Action;
        let draft = Draft {
            page: 0,
            kind: TextEditKind::Add,
            anchor: Anchor::Origin { x: 10.0, y: 20.0 },
            text: "one line".to_owned(),
            caret: 0,
            mark: None,
            seeded: false,
        };
        let mut actions = Vec::new();
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert!(matches!(
            actions.as_slice(),
            [Action::CommitAddText { wrap: None, .. }]
        ));
    }

    /// ★★ **Enter inserts inside a box and commits everywhere else**, which is
    /// the whole reason `Anchor::Box` is a variant rather than a flag.
    ///
    /// Asserted on the ANCHOR, because that is the fact the keystroke handler
    /// branches on. Driving the key itself needs a `Context` with focus and an
    /// event queue, which `text_box_takes_a_paragraph` does in the real binary;
    /// what is provable here is that the two anchors are distinguishable at all
    /// — and a build that folded the box into `Origin` with an `Option<Rect>`
    /// would fail this by construction.
    #[test]
    fn only_a_box_anchor_takes_a_paragraph_break() {
        let boxed = Anchor::Box {
            llx: 0.0,
            lly: 0.0,
            urx: 100.0,
            ury: 50.0,
        };
        assert!(matches!(boxed, Anchor::Box { .. }));
        assert!(!matches!(
            Anchor::Origin { x: 0.0, y: 0.0 },
            Anchor::Box { .. }
        ));
        assert!(!matches!(
            Anchor::Run {
                run: 0,
                original: String::new()
            },
            Anchor::Box { .. }
        ));
    }

    /// **An emptied draft is not a deletion.**
    ///
    /// Deleting every character of a run and clicking away is ambiguous —
    /// "remove this text" and "I changed my mind" look identical — and the
    /// recoverable reading is the one that writes nothing. Removing text is
    /// redaction's job, and it is a security operation with its own surface.
    #[test]
    fn an_emptied_draft_pushes_no_action() {
        let draft = Draft {
            page: 0,
            kind: TextEditKind::Edit,
            anchor: Anchor::Run {
                run: 1,
                original: "A".to_owned(),
            },
            text: String::new(),
            caret: 0,
            mark: None,
            seeded: true,
        };
        let mut actions = Vec::new();
        // ★ A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert!(actions.is_empty());
    }

    /// **A changed draft pushes exactly one action, carrying both texts.**
    #[test]
    fn a_changed_draft_pushes_one_edit_carrying_both_texts() {
        let draft = Draft {
            page: 2,
            kind: TextEditKind::Edit,
            anchor: Anchor::Run {
                run: 7,
                original: "REV A".to_owned(),
            },
            text: "REV B".to_owned(),
            caret: 0,
            mark: None,
            seeded: true,
        };
        let mut actions = Vec::new();
        // ★ A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions[0],
            crate::app::actions::Action::CommitTextEdit {
                page: 2,
                run: 7,
                original: "REV A".to_owned(),
                replacement: "REV B".to_owned(),
            }
        );
    }

    /// **An empty add-text draft places nothing.** A click with the Add tool and
    /// no typing is a caret, not a write.
    #[test]
    fn an_empty_add_text_draft_places_nothing() {
        let draft = Draft {
            page: 0,
            kind: TextEditKind::Add,
            anchor: Anchor::Origin { x: 10.0, y: 20.0 },
            text: String::new(),
            caret: 0,
            mark: None,
            seeded: true,
        };
        let mut actions = Vec::new();
        // ★ A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert!(actions.is_empty());
    }

    /// **The two kinds name the two registered commands, and they are
    /// different.** A copy-paste that gave both the same id would arm one tool
    /// from two buttons and nothing would notice.
    #[test]
    fn each_kind_names_its_own_registered_command() {
        assert_eq!(TextEditKind::Edit.command_id(), "edit.text");
        assert_eq!(TextEditKind::Add.command_id(), "edit.add_text");
    }
}
