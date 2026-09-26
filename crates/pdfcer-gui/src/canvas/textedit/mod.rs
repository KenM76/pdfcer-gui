//! # `canvas::textedit` — **editing the page's own words**, and placing new ones
//!
//! `DEFECTS.md` **D4** is the defect that began this project, reported as:
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/mod.md`.
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
//!   act either side of it. Without an index a draft can only append, and the
//!   painter can only draw its line at the right edge of the run.
//! - T3 graphemes-not-bytes: **PARTIAL** — characters, not bytes, so `é` takes
//!   one keystroke. Not grapheme clusters, so a combining mark or an emoji
//!   sequence still takes two. `unicode-segmentation` is already in the tree.
//! - T4 clamp-never-assert: every operation clamps on entry. A panic in a caret
//!   would take the whole window down over a keystroke.
//! - T5 composer-owns-the-keyboard: `composing` is the one predicate, and
//!   `tools/gates/check-typing-guard.sh` fails the build on a second copy. This
//!   caret is not an `egui::TextEdit`, so egui's own predicate cannot see it:
//!   any page key answered by a bare `text_edit_focused()` is a key taken out
//!   of a word the operator is visibly typing — Delete, and the space bar the
//!   pan tool claims.
//! - T6 every-exit-commits: Enter and Escape both write the draft, **and Enter
//!   has two meanings**.
//!   Inside a dragged text BOX a plain Enter is a paragraph break and
//!   `Ctrl+Enter` commits; everywhere else Enter commits. That is why
//!   [`Anchor::Box`] is a variant rather than a flag: the keystroke handler has
//!   to know which gesture started the draft, and asking the TEXT would make
//!   the first Enter commit and every one after it insert. A draft identical to
//!   what it replaces still raises no action.
//! - T7 no-control-characters: [`caret::insert`] filters them, on the ground
//!   that *a control character arriving in a `Text` event is something this
//!   shell has no meaning for* — true of typed text, and not true of the draft
//!   as a whole, which may hold a deliberate paragraph break. So the filter
//!   stays and the newline has its own door, [`caret::newline`]: relaxing
//!   `insert` would let a stray `\t` or `\r` from a paste into a show string as
//!   well.
//! - T8 selection: **GAP** — no Shift+arrow, no Ctrl+A, no drag-select within a
//!   draft. Named rather than left implied, because a highlight that some keys
//!   respect and others silently ignore is worse than none.

/// **The page's lines, reassembled into paragraphs** — and the arrow keys
/// that walk between them. Its header carries the four lines the behaviour is
/// modelled on and why the reassembly is `pdfcer-core`'s rather than this
/// shell's.
pub mod blocks;
/// The caret's own arithmetic — insert, delete, and the four movements. Pure
/// functions of a `&str` and an index, with no window in them; its header says
/// why that is a seam and not a cut.
pub mod caret;
/// Where the pointer is in relation to the editor box, published by `paint`
/// and read by everything that has to decide whether a press belongs to the
/// draft or to the page.
pub mod hit;
/// What every key means inside a draft — the keystroke contract.
pub mod keys;
/// **The caret's arithmetic inside a draft that holds more than one line**
/// — `OPERATOR_REQUESTS.md` **O127**, defect 2.
pub mod lines;
/// What a draft looks like on the page — the in-place editor and its caret. Its
/// header carries the standing rule that the text and the caret are measured
/// from ONE layout.
pub mod paint;
/// **Where a press puts the caret** — the three gestures that start a draft,
/// and the refusals each of them can raise. Its header carries why a text BOX
/// must be a drag rather than a click.
pub mod place;
/// Which paragraph the caret is in — the one question `reflow_block` needs and
/// the shell has to answer. Its header carries the refusal that shapes the
/// whole feature: a reflow is planned against the BASE document, so a page
/// already edited this session is refused by name.
pub mod reflow;
/// **What an edit report is worth telling anyone** — which of
/// `EditReport`'s eleven fields reach the operator, which reach the diagnostic
/// channel, and which reach neither. Its header carries the rule and why the
/// middle row of it exists.
pub mod report;
pub use caret::{backspace, delete_forward, insert, word_left, word_right};
/// **Naming the exact show operator, and the exact buffer it lives in** — the
/// one producer of `(pinned_span, EditTarget)` in this shell, shared by the
/// caret's `edit_text` and the restyle verbs' `format_text`.
pub mod pin;
/// **The alphabet the caret's run will accept, measured once when the caret
/// lands** — the shell side of `EditSession::run_repertoire`, and the reason a
/// key the run's font cannot spell is declined as it is pressed instead of
/// costing the operator the whole word at commit.
pub mod repertoire;

pub use place::{Click, begin_box, click};
pub mod disposition;
// The byte-level proof that the untouched tail did not move, with an
// `EditOptions::default()` run beside it as the falsifier. `#[cfg(test)]`
// inside; it compiles to nothing in a release build.
mod proof;
// The experiment that decides whose defect O141's last step is: ONE
// `EditSession`, `format_text` then `edit_text`, located by find text alone so
// no operand this shell computes is in the request. It refuses; the same pair
// with a reopen between them succeeds. `#[cfg(test)]` inside.
mod facewall;
/// **Planning the commit** — the whole of what a text edit decides, from a
/// caret and two strings to one `EditRequest`. `plan` and `Plan` are
/// re-exported below, so every caller reaches them through this module.
mod plan;
pub use plan::{Plan, plan};
// O142 — a typo in a run written one glyph per show operator, which only
// a spanning match can reach, and the guard that keeps the spanning match
// addressed to the occurrence the operator clicked. Two fixtures: one where the
// run is unique, one where the same text appears twice and the edit must land
// on the clicked one. `#[cfg(test)]` inside.
mod glyphwall;
// The per-keystroke re-measure measurement `DEFECTS.md` D4b's fix would need,
// and the reason it is not wired. `#[ignore]`d; run it and read the numbers.
mod cost;
/// The face, size and colour NEW page text is written in — the Phase 5 row
/// that read *"choosing what those three controls are is a decision, not an
/// omission"*. The decision is in that module's header, along with why it lives
/// in `egui::Memory` where the markup pen does not.
pub mod pen;

use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;

/// `egui::Memory` key for the in-flight draft.
const DRAFT_MEMORY_KEY: &str = "pdfcer-textedit-draft"; // ui-text-exempt: internal memory id, never displayed

/// The environment variable that supplies a draft when no keyboard can.
pub const DIAG_TYPE: &str = "PDFCER_DIAG_TYPE"; // ui-text-exempt: an environment variable name, never displayed

/// **Which of the two text verbs is armed.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEditKind {
    /// `edit.text` — replace the words in a run that is already on the page.
    Edit,
    /// `edit.add_text` — place new page content where the operator clicks.
    Add,
}

impl TextEditKind {
    /// The command id that arms this kind.
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
    /// **A RECTANGLE in PDF user space** — new text, wrapped to its width.
    ///
    /// The operator: *"I should be able to make it multi line."*
    ///
    /// # Why multi-line needs a BOX and cannot be a point with newlines in it
    ///
    /// Because a PDF has no paragraph. Each visual line is its own show
    /// operator at its own absolute position, so *something* has to decide
    /// where the second line starts — and the only thing that can is a width to
    /// wrap against and a leading to step by.
    ///
    /// `pdfcer-core`'s `AddTextRequest::wrap_box` is exactly that, and it is
    /// more than a container: **paragraphs split on a hard `\n` and
    /// each is wrapped independently**, so an operator gets both behaviours —
    /// Enter makes a new paragraph, and running past the right edge makes a new
    /// line — from one field.
    ///
    /// # Why this is a third variant and not a `wrap: Option<Rect>` on
    /// [`Self::Origin`]
    ///
    /// Because the two are **different gestures with different affordances**,
    /// and folding them would make "did the operator drag or click?" a runtime
    /// question at commit time rather than a fact the press already settled. A
    /// click places a single-line run at a point; a drag places a paragraph in
    /// a box. In box mode a plain Enter is a paragraph break and `Ctrl+Enter`
    /// accepts; in point mode Enter accepts — which is what every program in
    /// the class does.
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
    /// Which page it belongs to, and the page the commit is addressed to. A
    /// draft survives a page change: it commits against this index whenever it
    /// is settled, so navigating away and back does not move the text.
    pub page: usize,
    /// Which verb will commit it.
    pub kind: TextEditKind,
    /// What the caret is on.
    pub anchor: Anchor,
    /// The operator's in-progress text.
    pub text: String,
    /// **Where the caret sits inside [`Self::text`], as a CHARACTER index.**
    ///
    /// `0` is before the first character; `text.chars().count()` is after the
    /// last. Every edit and every movement clamps into that range, so the
    /// invariant `caret <= text.chars().count()` holds by construction and no
    /// caller has to check it.
    ///
    /// # Why an index and not an append point
    ///
    /// Without one, `insert` can only extend the end of the string and
    /// `backspace` can only pop the last character — the caret is not merely
    /// fixed at the end, **there is no caret**, and the painter can only draw
    /// its line at the right edge of the run's glyph box. The operator:
    ///
    /// > *"the cursor just sits at the end of a text line. It can't be moved to
    /// > the center of an existing text block."*
    ///
    /// That makes editing existing page text almost useless: a title-block cell
    /// reading `SHEET 1 OF 4` can only be changed by deleting it back to
    /// `SHEET ` and retyping.
    ///
    /// # Why characters and not bytes
    ///
    /// Because every operation here is expressed in keystrokes, and one
    /// keystroke is one `char`. A byte index would make Left-arrow over `é` —
    /// two bytes — either move half a character or need a decode at every use.
    /// `backspace` already worked in `char`s for the same reason (a byte
    /// truncation of a multi-byte character is a panic in Rust, not mojibake),
    /// so this is that decision applied consistently rather than a new one.
    ///
    /// The cost is that every operation is O(n) in the draft's length. A draft
    /// is one show operator — a cell, a label, a line of a note — so n is tens
    /// of characters, and the alternative is a byte index plus a boundary check
    /// at every call site.
    pub caret: usize,
    /// **The other end of a selection**, as a character index, or `None`
    /// when nothing is selected.
    ///
    /// The selection is the range between this and [`Self::caret`], in either
    /// order — [`caret::range`] normalises it. Two indices rather than a
    /// `Range`, because the *direction* is real: Shift+Left from the middle of
    /// a word must extend leftward and then shrink back rightward, and a
    /// normalised pair forgets which end the operator is dragging.
    ///
    /// # Why it is called a mark
    ///
    /// Because "anchor" is taken. [`Anchor`] already means *what on the page
    /// this draft is attached to*, which is a different question with a
    /// different answer, and two fields called anchor in one struct is how a
    /// wrong one gets read.
    ///
    /// # Why the field exists at all
    ///
    /// `OPERATOR_REQUESTS.md` O14 item 11: *"no selection inside a draft — no
    /// Shift+arrow, no Ctrl+A, no drag-select."* Every text field the operator
    /// has ever used has all three, and without them replacing a word means
    /// pressing Backspace once per character.
    ///
    /// **It is cleared by any un-shifted movement**, which is what makes a
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The click landed on no text at all.
    NoRun,
    /// The page's text could not be extracted (an image-only page, a damaged
    /// content stream).
    NoText,
    /// **The run is real and readable, and it covers no show operator the
    /// surgery could anchor on** — its glyphs come from `/ActualText`, which
    /// supplies the text without drawing it.
    ///
    /// This is not *"out of reach"*; it is *"there is nothing to reach for"*,
    /// and the engine carries it as its own [`Editability::NoAnchor`] variant
    /// precisely so a shell can say something different about it. The shell
    /// asks that predicate rather than modelling the surgery's internals — a
    /// guard built here out of what the engine happens not to support yet is a
    /// workaround that goes on refusing after the support lands.
    ///
    /// # Why refusing at the click rather than at the commit
    ///
    /// Because the attempt cannot succeed, and a control that accepts input it
    /// will discard is this project's defining defect class. Without this, the
    /// click is accepted, a caret placed, keystrokes taken, a plan built, and
    /// the engine refuses the commit **to the trace only** — so the operator
    /// sees a caret that took their sentence and threw it away in silence. An
    /// honest refusal at the click costs them one click instead.
    ///
    /// [`Editability::NoAnchor`]: pdfcer_core::text_extract::Editability::NoAnchor
    NoAnchor,
    /// **The run is real, addressable, and its font can spell nothing at
    /// all** — so the caret declines to open rather than open and refuse every
    /// key.
    ///
    /// # The engine built this distinction because this shell asked for it
    ///
    /// `EditSession::run_repertoire` could have reported an un-invertible font
    /// as an `Err`. It does not: it answers an **empty**
    /// `RunRepertoire` with `RunRepertoire::reason` set, and the engine's own
    /// rustdoc says why in as many words — *"so an editor can decline to open
    /// rather than open and refuse every key. The requesting shell asked for
    /// that distinction by name."*
    ///
    /// A run that cannot be **located** is still an `Err` and lands on
    /// [`Self::NoRun`] or [`Self::NoText`]. The two are not interchangeable:
    /// one says *there is nothing here*, this one says *this text is here and
    /// pdfcer cannot spell into it*.
    ///
    /// # Why a refusal and not a caret that greys out
    ///
    /// Because there is no key it would accept. [`Self::NoAnchor`]'s argument
    /// applies unchanged and is the module's oldest lesson: *a control that
    /// accepts input it will discard is this project's defining defect class.*
    /// A caret in a run with an empty repertoire is exactly that control — it
    /// blinks, it takes clicks, it moves with the arrow keys, and every printable
    /// key does nothing. One honest refusal at the click is cheaper than
    /// discovering it one keystroke at a time.
    ///
    /// # ⚠ The engine's `reason` string does NOT reach the operator
    ///
    /// It is good prose and it is in the engine's vocabulary: *"none of the 217
    /// character(s) this font addresses can be shown by this run — an embedded
    /// subset carries only the codes already drawn on this page (R-INV-1)"*.
    /// `text::redact`'s wording rule, generalised: **say where in HIS
    /// vocabulary, never in the engine's.** So the `reason` goes to
    /// `PDFCER_DIAG` — where the clause number is exactly what a reader wants
    /// — and `crate::text::textedit::refusal` says the same fact in his terms.
    ///
    /// It carries no payload for that reason. A `String` here would be the
    /// engine's sentence travelling towards a surface that must not show it,
    /// and `Refusal` would stop being `Copy` to carry it.
    NoUsableEncoding,
}

/// **Is the operator composing text ANYWHERE?** The one predicate, asked in
/// one place.
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
pub fn abandon(ctx: &egui::Context) -> bool {
    let had = read(ctx).is_some();
    ctx.data_mut(|d| d.remove::<Draft>(egui::Id::new(DRAFT_MEMORY_KEY)));
    // The draft's run alphabet dies with the draft. `repertoire::of_run`'s key
    // check would reject it anyway on the next caret; this is the belt to that
    // pair of braces, and the reason is in `repertoire::forget`'s own docs — a
    // slot that outlives its subject is a fossil a later reader will trust.
    repertoire::forget(ctx);
    if had {
        // ui-text-exempt: diagnostic trace, never displayed.
        crate::diag::trace(|| "text-edit-abandon".to_owned());
    }
    had
}

/// Turn a draft into the action that commits it, if it says anything.
pub(super) fn commit_into(
    ctx: &egui::Context,
    draft: &Draft,
    actions: &mut Vec<crate::app::actions::Action>,
) {
    use crate::app::actions::Action;
    match &draft.anchor {
        Anchor::Run { run, original } if draft.text != *original => {
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
                // Sampled HERE, at the commit, not read in `apply`. See the
                // variant's own docs: an action is what the operator asked for,
                // and it is applied on a later frame.
                pen: pen::read(ctx),
                // A point-text run is one line and has no box to wrap to.
                wrap: None,
            });
        }
        // The boxed variant, and it reaches the SAME action — one commit
        // path, one apply arm, one place that can be wrong about a font.
        //
        // The whole difference is that `wrap` is `Some`, which is what
        // `AddTextRequest::with_box` turns into the multi-line layout: hard
        // newlines split paragraphs and each is wrapped independently to the
        // box's width, top-anchored from its top edge.
        //
        // `origin` is still carried and is still the box's lower-left, even
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

/// **Finish a draft that is going out of scope: write what it says, then
/// tear it down.** Reports whether there was one.
pub fn settle(ctx: &egui::Context, actions: &mut Vec<crate::app::actions::Action>) -> bool {
    let Some(draft) = read(ctx) else {
        return false;
    };
    commit_into(ctx, &draft, actions);
    abandon(ctx);
    true
}

/// **What the text edit currently being applied is trying to write** — the four
/// operands of the `Action::CommitTextEdit` that [`plan`] was called for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Committing {
    /// The 0-based page the commit named.
    pub page: usize,
    /// The run index the caret was in.
    pub run: usize,
    /// The run's text when the caret landed on it — `EditRequest::find`, and
    /// the `original` operand of a re-raised `Action::CommitTextEdit`.
    pub original: String,
    /// **What the operator typed.** The operand that exists nowhere else once
    /// the draft has been abandoned.
    pub replacement: String,
}

thread_local! {
    /// See [`Committing`].
    static LAST_COMMIT: std::cell::RefCell<Option<Committing>> =
        const { std::cell::RefCell::new(None) };
}

/// **The operands of the text commit that is being applied right now**, or
/// `None` if no commit has been planned in this process yet.
#[must_use]
pub fn last_commit() -> Option<Committing> {
    LAST_COMMIT.with_borrow(Clone::clone)
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

    /// **A draft equal to what it replaces is not a write.**
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
        // A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert!(actions.is_empty(), "an unchanged draft is not an edit");
    }

    /// **A box draft commits as a WRAPPED PARAGRAPH, and carries its
    /// rectangle.**
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

    /// **Enter inserts inside a box and commits everywhere else**, which is
    /// the whole reason `Anchor::Box` is a variant rather than a flag.
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

    /// **An emptied run commits the emptying.**
    #[test]
    fn an_emptied_run_draft_commits_the_emptying() {
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
        // A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        commit_into(&egui::Context::default(), &draft, &mut actions);
        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions[0],
            crate::app::actions::Action::CommitTextEdit {
                page: 0,
                run: 1,
                original: "A".to_owned(),
                replacement: String::new(),
            },
            "emptying a run is an edit, not a change of mind"
        );
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
        // A bare `Context`, and it is the honest one for a pure-commit test:
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
    /// no typing is a caret, not a write, and a dragged box nobody typed into is
    /// the same thing with a size.
    #[test]
    fn an_empty_add_text_draft_places_nothing() {
        // A bare `Context`, and it is the honest one for a pure-commit test:
        // the pen it reads is whatever `TextPen::default()` is, which is the
        // engine's own default, so these assertions are about the ACTION's
        // shape and not about a pen nobody set.
        let ctx = egui::Context::default();

        for anchor in [
            Anchor::Origin { x: 10.0, y: 20.0 },
            Anchor::Box {
                llx: 10.0,
                lly: 20.0,
                urx: 210.0,
                ury: 90.0,
            },
        ] {
            let draft = Draft {
                page: 0,
                kind: TextEditKind::Add,
                anchor: anchor.clone(),
                text: String::new(),
                caret: 0,
                mark: None,
                seeded: true,
            };
            let mut actions = Vec::new();
            commit_into(&ctx, &draft, &mut actions);
            assert!(
                actions.is_empty(),
                "an Add draft with nothing typed is a caret, not a write: {anchor:?}"
            );
        }
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
