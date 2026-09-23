//! # `canvas::moving::refusal` — why a move drag committed nothing, in words and in a token
//!
//! The vocabulary a refused drag is reported in, and nothing else: the
//! [`Refusal`] enum, the stable token a driven check greps for, and the
//! sentence the status bar owes the operator.
//!
//! ## The contract this file holds
//!
//! **A token is a distinguishable CAUSE, not an enum variant.** Two arms of one
//! variant that put different sentences on the status bar are two tokens, so a
//! check asserting *which* refusal fired cannot be satisfied by the wrong one.
//! [`Refusal::token`] carries the argument in full.
//!
//! **[`Refusal::worded`] is exhaustive on purpose.** A new variant is a compile
//! error until somebody answers *does this one owe the operator a sentence?* —
//! the question that was never asked about `NoVerbForPart`, at the cost of a
//! drag across the sheet with nothing happening and no sentence anywhere.
//!
//! Separate from [`super`] because it is a closed vocabulary with its own
//! obligations to two different readers — a machine and a person — while the
//! parent module is the gesture machinery that raises it.

use crate::app::actions::CanvasDecline;
use crate::panels::objects::provider::{PartKind, RunMoveBlock};

/// Why a move drag committed nothing.
///
/// Reported rather than silently absorbed, and reported with enough detail to
/// act on, because *"nothing happened"* has several causes with opposite
/// responses: a drag that ended where it started is correct behaviour, a text
/// object at the Part rung is a missing verb, and a degenerate page is a
/// broken document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The page has no readable object model, so nothing can be verified and
    /// nothing may be promised. Reachable when the page failed to decompose.
    NoObjectModel,
    /// Nothing is selected on this page.
    NothingSelected,
    /// The selection names an object index that does not fit a `usize`.
    ///
    /// Structurally unreachable on any real document — [`TargetId`] is a `u64`
    /// and a paint-order index is bounded by the page's operator count — and
    /// refused rather than truncated because a truncating cast would address a
    /// *different* object, which is the one outcome
    /// `docs/core-api/02` §1.10.1 marks as dangerous.
    ///
    /// [`TargetId`]: crate::canvas::target::TargetId
    UnaddressableObject,
    /// **The selection is inside a form XObject**, so no page paint-order
    /// verb can address it.
    ///
    /// ★ Distinct from [`Self::NothingSelected`], and the distinction is the
    /// whole point: something *is* selected, the operator can see its outline,
    /// and answering "nothing selected" would be a flat contradiction of what
    /// is on screen. This is the refusal that has an explanation to give, and
    /// [`crate::app::status::decline`] gives it.
    ///
    ///
    /// It said: *"[`pdfcer_core::vector::FormLeaf::is_editable`] is `false` for
    /// every leaf the engine produces"*, dated 2026-08-27 against
    /// `pdfcer-core` v0.14.0, and ended *"when editing-through-recursion
    /// lands, the remedy is to route to the form-scoped verb"*.
    ///
    /// **Editing-through-recursion landed at `Pass 188.0`, this shell routed
    /// to all six form-scoped verbs on 2026-09-01, and the sentence stayed.**
    /// `is_editable` did not change signature, so nothing broke and nothing
    /// warned; it changed MEANING. It now answers *"is this leaf a path"*,
    /// and the engine's own doc says in as many words that a shell greying
    /// out the whole container on `false` is now wrong.
    ///
    ///
    /// # What actually reaches this variant
    ///
    /// Exactly one condition, in the Part and Node arms of `eligible`: the
    /// entered thing is inside a form **and its part kind could not be read
    /// as a path**. The page-order twin of that state refuses with
    /// [`Self::NotAPath`], carrying the page-object index a leaf does not
    /// have. So the fact is *not a path*, and containment is only how it got
    /// here — which is why the sentence the operator reads is
    /// [`crate::text::status::InsideFormRefusal::NotAPath`] and not a
    /// statement about forms at all.
    InsideForm,
    /// A selected object is not a path, so the whole move is refused. Carries
    /// its paint-order index.
    NotAPath(usize),
    /// The Part rung is entered but no part is named — an inconsistent state
    /// [`crate::canvas::selection::SelectionState`] does not produce, refused rather than guessed at.
    NoPartEntered,
    /// The entered part has no move verb.
    ///
    /// **Not the Part rung's answer for text.** [`super::eligible`] routes a movable
    /// line to [`super::MoveSubject::TextLine`] and an unmovable one to
    /// [`Self::TextRunCannotMove`], which carries the engine's reason. What is
    /// left here is the **Node** rung: a text line has no anchors, so there is
    /// no `move_node` to reach, and `PartKind::TextLine` arriving at that rung
    /// is an inconsistent selection rather than a missing capability.
    ///
    /// Kept rather than deleted because the Node arm genuinely needs an arm,
    /// and because [`Self::token`] still distinguishes the two part kinds — a
    /// driven check that could not tell them apart would be asserting the
    /// wrong cause.
    NoVerbForPart(PartKind),
    /// ★★★ **The engine would refuse to move this line, and it said so before
    /// the operator let go of the mouse.**
    ///
    /// `OPERATOR_REQUESTS.md` O188. Distinct from [`Self::NoVerbForPart`] in
    /// the way that matters to whoever reads the sentence: that one meant
    /// *pdfcer cannot do this at all*, this one means *pdfcer cannot do it to
    /// THIS line, because of how the file was written*. The remedy is the
    /// same — select the whole block and drag that — but the fact is not,
    /// and a sentence that got them the wrong way round would send the
    /// operator looking for a setting that does not exist.
    ///
    /// ★★ **Raised from [`super::eligible`], not from the edit funnel**, which is
    /// the placement that makes it honest. The engine would refuse this move
    /// too, and would do it after the gesture — so the operator would have
    /// watched an outline slide across the sheet and then snap back. Asking
    /// [`ObjectModelProvider::text_line_move_refusal_of`] first means no ghost
    /// is ever drawn for a move that will not happen, which is obligation 3
    /// in this module's header.
    TextRunCannotMove(RunMoveBlock),
    /// The Node rung is entered but no anchor is named. Same nature as
    /// [`Self::NoPartEntered`].
    NoNodeEntered,
    /// The entered anchor is not in the object's current anchor list — the
    /// selection out-ran a decomposition. Carries the object-scoped index.
    NodeNotFound(usize),
    /// The gesture ended where it began. See [`super::PageDelta::is_travel`].
    NoTravel,
    /// The page's device transform is not invertible, so there is no
    /// well-defined page-space displacement. Declining is the only honest
    /// answer; authoring garbage geometry is not.
    DegeneratePage,
}

impl Refusal {
    /// The stable identifier this refusal is **traced** under.
    ///
    /// # ★★★ Why a token, when `{reason:?}` was already printing something
    ///
    /// Because `Debug` renders the **source**, not a contract. The trace line
    /// below is grepped by `tools/ui-verify`, and a `{:?}` field moves whenever
    /// a variant is renamed or gains a payload, with no compiler diagnostic and
    /// no failing test — the check simply stops matching and goes quietly green
    /// on an absence. Two of the eleven variants below already put a `usize`
    /// inside the field.
    ///
    /// ★★ **The `Debug` rendering is kept, beside this rather than instead of
    /// it**, in the trace's `detail=` field. [`Self::NotAPath`] and
    /// [`Self::NodeNotFound`] carry the only indication of *which object*, and
    /// dropping it to gain stability would have traded one loss for another.
    /// One field a machine reads, one field a human reads.
    ///
    /// ★ Kebab-case, one word per concept, on
    /// [`crate::canvas::pick::PickClass::token`]'s precedent — including the
    /// per-arm `// ui-text-exempt:` comment, which `check-ui-strings.sh`
    /// requires arm by arm rather than once per function.
    ///
    /// ⇒ **The unit is a distinguishable cause, not a variant**, which is why
    /// there are sixteen tokens for twelve variants: `NoVerbForPart` splits by
    /// part kind and `TextRunCannotMove` splits by block, because in each case
    /// one variant is worn by causes a harness must tell apart — and one that
    /// could not would assert the wrong cause while looking green.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::NoObjectModel => "no-object-model",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::NothingSelected => "nothing-selected",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::UnaddressableObject => "unaddressable-object",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::InsideForm => "inside-form",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::NotAPath(_) => "not-a-path",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::NoPartEntered => "no-part-entered",
            // ★★★ Split by part kind. **The unit a token names is a
            // distinguishable CAUSE, not an enum variant.** Both of these are
            // silent to the operator, so the status bar cannot tell them
            // apart and only the trace can: the text-line case is a selection
            // that reached the Node rung, the subpath case is unreachable.
            // Those are different faults to go looking for, and `detail=` is
            // the `Debug` field a check must not parse to make up the
            // difference.
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::NoVerbForPart(PartKind::TextLine) => "no-verb-for-text-line",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::NoVerbForPart(PartKind::Subpath) => "no-verb-for-subpath",
            // ★★★ Split by BLOCK, for the reason the pair above is split by
            // part kind: these are three distinguishable causes wearing one
            // variant, and two of them put different sentences on the status
            // bar. A driven check asserting "the drag was refused" learns
            // nothing; one asserting WHICH refusal fired is the only kind that
            // can tell a correct decline from a decline for the wrong reason.
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::TextRunCannotMove(RunMoveBlock::NoPositionOfItsOwn) => "run-has-no-position",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::TextRunCannotMove(RunMoveBlock::WouldMoveNextRun) => "run-would-move-next",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::TextRunCannotMove(RunMoveBlock::NotThere) => "run-not-there",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::NoNodeEntered => "no-node-entered",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::NodeNotFound(_) => "node-not-found",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::NoTravel => "no-travel",
            // ui-text-exempt: stable diagnostic tokens, never displayed.
            Self::DegeneratePage => "degenerate-page",
        }
    }

    /// What this refusal owes the operator in words, or `None` if the answer
    /// is silence.
    ///
    /// # ★★★ Exhaustive on purpose — a `_ => None` would be the defect
    ///
    /// Four arms return a sentence and the rest return `None`, and it would be
    /// shorter to write the four and catch the rest with a wildcard. That
    /// shorter version has one property this one does not: **a thirteenth
    /// refusal would join the silent ones without anybody deciding that it
    /// should.**
    ///
    /// Written out, adding a variant to [`Refusal`] is a compile error until
    /// somebody answers *does this one owe the operator a sentence?* — which is
    /// the question that was never asked about `NoVerbForPart`, and
    /// `OPERATOR_REQUESTS.md` **O188** is what that costs: a box drawn round one
    /// label, a drag across the sheet, and nothing happening with no sentence
    /// anywhere.
    ///
    /// # Why the other nine stay silent, and why it is still the right default
    ///
    /// They describe states the operator put themselves in and can see: nothing
    /// selected, a drag that travelled no distance, a rung entered with nothing
    /// named in it. A status bar that narrates the obvious is a status bar that
    /// stops being read, which would cost the two sentences that matter. The
    /// test is not *is this a refusal* but **can the operator see the cause**.
    ///
    /// ★★ [`PartKind::Subpath`] is answered explicitly rather than folded in
    /// with `NoVerbForPart(_)`. `eligible` routes a subpath at the Part rung to
    /// `move_subpath`, so that combination is unreachable today — and an
    /// unreachable arm that is written down is a claim the next reader can
    /// check, where one hidden behind a wildcard is an assumption.
    pub(crate) const fn worded(self) -> Option<CanvasDecline> {
        match self {
            Self::InsideForm => Some(CanvasDecline::InsideFormNotAPath),
            //
            // Both halves of the Part rung for text reach a verb or a REASON,
            // and neither is this one: a movable line goes to
            // `MoveSubject::TextLine`, an unmovable one to
            // `Self::TextRunCannotMove` below. What is left of
            // `NoVerbForPart(TextLine)` is the Node rung, where a line has no
            // anchors and the state is inconsistent rather than unsupported —
            // the operator has not been refused a capability, so there is
            // nothing to tell them.
            Self::NoVerbForPart(PartKind::TextLine) => None,
            // Unreachable: `eligible` sends a subpath at the Part rung to
            // `move_subpath`. Named anyway — see the docs above.
            Self::NoVerbForPart(PartKind::Subpath) => None,
            // ★★★ **The replacement pair, and they say opposite things about
            // WHICH line is the problem** — which is exactly why they are two
            // sentences and not one parameterised one. `NoPositionOfItsOwn` is
            // about the line the operator grabbed; `WouldMoveNextRun` is about
            // the line under it, which looks fine and is the reason the grabbed
            // one cannot go. An operator told the wrong one of those goes
            // looking for a fault in the wrong place.
            Self::TextRunCannotMove(RunMoveBlock::NoPositionOfItsOwn) => {
                Some(CanvasDecline::TextRunHasNoPositionOfItsOwn)
            }
            Self::TextRunCannotMove(RunMoveBlock::WouldMoveNextRun) => {
                Some(CanvasDecline::TextRunWouldDragTheNextLine)
            }
            // ★ Silence, deliberately. The selection named a run the object
            // does not have — it outlived an edit — and the operator did
            // nothing wrong and has nothing to do differently. Saying so would
            // be the bar narrating this program's own bookkeeping.
            Self::TextRunCannotMove(RunMoveBlock::NotThere) => None,
            Self::NoObjectModel
            | Self::NothingSelected
            | Self::UnaddressableObject
            | Self::NotAPath(_)
            | Self::NoPartEntered
            | Self::NoNodeEntered
            | Self::NodeNotFound(_)
            | Self::NoTravel
            | Self::DegeneratePage => None,
        }
    }
}
