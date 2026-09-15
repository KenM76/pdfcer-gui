//! # `app::status::decline::canvas` — what a refused canvas gesture may say
//!
//! ★★★ **Split out of [`super`] on 2026-09-15 with `OPERATOR_REQUESTS.md`
//! O188**, which is the fifth time that file has met R2's 1,500-line ceiling.
//! It stood at 1,367 lines and O188's variant is fifty-seven of them; adding
//! the vocabulary and its writer as well would have landed at ~1,498, and a
//! file with two lines of headroom is a file that forbids its own next edit.
//!
//! ## Why a file and not two more functions in `record`
//!
//! `record.rs`'s own header states its seam: it holds *“a family of twenty-odd
//! functions whose whole content is a one-line write plus the argument for
//! where it is called from”*, and it holds them because that argument is the
//! same argument twenty times over.
//!
//! This is the other kind. `clipboard.rs` and `textedit.rs` were cut out of
//! that family for carrying **an argument of their own**, and so does this: the
//! question here is not *which phase writes* but **what a surface holding
//! `&OpenDoc` is permitted to assert**, which is a question about the shape of
//! the boundary rather than about timing. Among twenty one-line writers it
//! would be unfindable.
//!
//! ## The whole mechanism in four steps
//!
//! 1. a gesture in `crate::canvas` refuses, holding `&OpenDoc` and no `&mut`;
//! 2. it raises [`crate::app::actions::Action::DeclineOnCanvas`] carrying one
//!    [`CanvasDecline`];
//! 3. the apply phase calls [`record_canvas`], the only mapping from that
//!    vocabulary to [`super::Declined`];
//! 4. the status bar reads it through `super::live`, which applies the
//!    retirement filter.
//!
//! ⇒ Nothing here writes a sentence. The wording lives in `crate::text::*`
//! catalogs, per this crate's standing rule, and the two arms below name which
//! catalog entry rather than quoting one.

/// **Everything a refused canvas gesture is allowed to put on the status bar.**
///
/// Two arms, and the arity is the point rather than an accident of how many
/// have been written so far.
///
/// # ★★★ Why this type exists when [`super::Declined`] is right there
///
/// The canvas holds `&OpenDoc` and not `&mut`, so a gesture with something to
/// declare cannot write the store; it raises an action and the apply phase
/// writes. That much is this crate's standing shape, and it is not the
/// interesting part.
///
/// The interesting part is what the action may **carry**, and the answer is not
/// a [`super::Declined`]. That enum holds a variant for very nearly every
/// surface in the application, and nearly every one of them is
/// a sentence some other surface owns — a save that failed, a bookmark that
/// would not move, a reflow the engine refused. An action able to carry any of
/// them would let the canvas assert facts it has no way to establish, and would
/// turn one choke point into a general *print any sentence* channel.
///
/// `Action::DeclineInsideForm` named that failure mode in its own docs and set
/// the condition for widening itself: *“Adding an `InsideFormRefusal` payload
/// would be milder and is still wrong today” — “Add it the day the canvas can
/// raise the second one, not before.”* O188 is the day the canvas can raise the
/// second one. This enum is the *milder* widening that block described, made
/// one degree milder still: not the engine's refusal taxonomy, but a list of
/// sentences written here.
///
/// ⇒ **A new arm is a deliberate act with a visible cost.** It means naming a
/// sentence the canvas is allowed to say, in a type whose whole documented
/// purpose is to stay short, and then answering [`super::Declined::still_true`]
/// for it. That friction is what this type is for.
/// # ★★ Why `pub`, when everything around it is `pub(crate)`
///
/// Because it is half of [`crate::app::actions::Action::DeclineOnCanvas`]'s
/// signature and `Action` is `pub` and genuinely reachable. A `pub(crate)`
/// payload on a `pub` variant is what the `private_interfaces` lint is for,
/// and under `-D warnings` that is a build failure rather than a note. Both
/// of `Action`'s existing payload sub-enums, `VectorAction` and
/// `RedactAction`, are `pub` for the same reason.
///
/// `app::prefs` records the other way out of this collision — keep the type
/// `pub(crate)` and hold it on a `pub(crate)` **field** of a `pub` struct —
/// and it is not available here. A variant's payload is the variant; there
/// is no field to demote.
///
/// ⇒ **The module did not widen, and neither did the store.**
/// `app::status` still declares `pub(super) mod decline`, so the path
/// `crate::app::status::decline` is nameable only from inside `crate::app`,
/// exactly as before. [`record_canvas`] below is still `pub(crate)`,
/// [`super::LAST`] is still private, and `super::record_inside_form` is
/// unchanged. What crossed the boundary is one **type**, through one named
/// re-export at `crate::app::actions::CanvasDecline`.
///
/// ★★★ That distinction is the whole justification. The rule
/// `pub(super)` enforces is *“a decline is written by the one dispatcher and
/// read by the one bar”* — a rule about **who may write**. A two-armed list
/// of which sentence a refused gesture may ask for is vocabulary, and asking
/// is what the action is. `Action` itself is already re-exported for the
/// canvas to name, on precisely this reasoning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasDecline {
    /// A part or a node was entered inside a form XObject whose kind is not a
    /// path, so no geometry verb applies.
    ///
    /// ★ [`record_canvas`] maps this to
    /// [`crate::text::status::InsideFormRefusal::NotAPath`] as a **constant**,
    /// because the canvas only ever meets that one of the two arms. The other,
    /// `NoContainingForm`, is written directly by `app::dispatch::format` from a
    /// place that has established a fact the canvas cannot.
    InsideFormNotAPath,
    /// ★★★ **A drag on one line inside a block of text** — O188.
    ///
    /// [`super::Declined::TextRunCannotMoveAlone`] carries the argument for why
    /// this refusal earns a sentence when nine of its ten siblings in
    /// `canvas::moving::Refusal` do not.
    TextRunCannotMoveAlone,
}

impl CanvasDecline {
    /// The stable identifier this decline is **traced** under.
    ///
    /// # ★★★ Why a token and not `{self:?}`
    ///
    /// Because [`record_canvas`]'s trace line is read by a machine, and a
    /// `Debug` rendering is a property of how the variant is **spelled**. Rename
    /// an arm and every driven check keyed on it stops matching — silently, with
    /// no compiler anywhere in the chain, and the check then reports *the sentence
    /// never reached the operator* about a build in which it did. That is the
    /// worst shape a diagnostic can have: a confident false negative that quotes
    /// the truth in its own failure message.
    ///
    /// ⇒ These two strings are part of the harness contract, like
    /// `status-group:decline` itself. Changing one is changing an interface;
    /// `tools/ui-verify/src/checks/move_line_of_text.rs` matches them literally.
    ///
    /// ★★ Twin of `canvas::moving::Refusal::token`, written the same day and
    /// for the same reason. The pair is deliberate: the canvas names the cause,
    /// this names the sentence, and a driven check that reads both can tell a
    /// refusal that reached the store from one that was raised and dropped.
    #[must_use]
    pub(crate) const fn token(self) -> &'static str {
        match self {
            // ui-text-exempt: stable diagnostic token, never displayed.
            Self::InsideFormNotAPath => "inside-form-not-a-path",
            // ui-text-exempt: stable diagnostic token, never displayed.
            Self::TextRunCannotMoveAlone => "text-run-cannot-move-alone",
        }
    }
}

/// Record what a refused **canvas gesture** had to say — O188, 2026-09-15.
///
/// The one writer behind [`crate::app::actions::Action::DeclineOnCanvas`], and
/// the only place [`CanvasDecline`]'s arms become [`super::Declined`]s.
///
/// # ★★ Why this is an apply arm, when [`super::record_inside_form`] argues
/// # at length for the dispatcher
///
/// Because the canvas is not the dispatcher and cannot be made into one. That
/// function's argument is about a *command* refusing, where a dispatcher already
/// holds the answer and an apply phase would have to re-derive it — and both of
/// its callers are exactly that. A canvas **gesture** is not dispatched at all:
/// it happens under `&OpenDoc`, the store is written under `&mut`, and the
/// action is how a read-only surface asks. This is where the ask lands.
///
/// ★ Deliberately not folded into [`super::record_inside_form`], which keeps
/// both of its callers. One of them is `app::dispatch::format` writing directly
/// from a place that has established `NoContainingForm`; the canvas cannot
/// establish that, and folding the two would put a dispatcher's write behind a
/// name that says *canvas*.
///
/// ★ **A selection is still not an edit** — no `vector_edit`, no epoch bump,
/// no cache invalidation. Nothing about the document changed; a sentence was
/// written down.
///
/// # ★★★ It traces, and the trace is the middle link of a three-stage chain
///
/// The status bar publishes one region for every decline in the application,
/// `status-group:decline`. A driven check that asserted only *that region
/// drew* would be satisfied by a build that raised the **wrong** sentence, and
/// equally by one that left a stale sentence on the bar from an earlier
/// gesture. An assertion both outcomes satisfy is not a measurement of which
/// one shipped.
///
/// So the refusal is observable at three points, in three subsystems, and a
/// build that breaks any one link fails at that link:
///
/// ```text
/// canvas       canvas-move-declined level=Part sel=1 reason=no-verb-for-text-run
/// apply phase  canvas-decline-recorded what=text-run-cannot-move-alone
/// status bar   ui-rect name=status-group:decline
/// ```
///
/// ★★ That is not the application agreeing with itself. The first line is
/// written by `crate::canvas` holding `&OpenDoc`; this one by the apply phase
/// holding `&mut`, on the far side of the `Action` boundary the design exists
/// to keep; the third by the status bar on a later frame. A shell that raised
/// the action and never applied it writes the first and not the second.
///
/// ★ Once per decline, not once per frame. [`super::show`] runs 60 times a
/// second and a line there would bury the channel — the lesson `canvas-pointer`
/// taught, and the reason `canvas::moving::drag` traces its refusal only on
/// release.
pub(crate) fn record_canvas(what: CanvasDecline) {
    let declined = match what {
        CanvasDecline::InsideFormNotAPath => {
            super::Declined::InsideForm(crate::text::status::InsideFormRefusal::NotAPath)
        }
        CanvasDecline::TextRunCannotMoveAlone => super::Declined::TextRunCannotMoveAlone,
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("canvas-decline-recorded what={}", what.token())
    });
    super::LAST.with_borrow_mut(|slot| *slot = Some(declined));
}
