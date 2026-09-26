//! # `canvas::modelneed` — **does this frame need the page's object model?**
//!
//! One question, asked once, for the whole canvas frame. It decides whether
//! [`crate::app::state::OpenDoc::page_objects`] is called at all — and
//! therefore whether every consumer below it sees a decomposition or a `None`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/modelneed.md`.

use crate::canvas::gesture::{GestureOutcome, MarqueeIntent, Phase};
use crate::canvas::selection::{SelectionLevel, SelectionState};

/// Everything about this frame that bears on whether the decomposition is
/// wanted.
///
/// A struct rather than six positional arguments, for the reason
/// `canvas::keys::Keys` gives: the list has grown four times and every growth
/// was a defect being fixed, so the next one should read as a new named fact
/// rather than as a seventh `bool` nobody can order correctly at the call site.
pub struct Need<'a> {
    /// What this frame's pointer means, as the gesture machine reported it.
    pub outcome: &'a GestureOutcome,
    /// Whether the secondary button was clicked this frame.
    ///
    /// A right-click is **not** a gesture outcome — the machine does not model
    /// it — so it has always had to be its own term. It needs the model for a
    /// click's reason: the menu has to know what is under the pointer, and a
    /// menu about the wrong object is worse than no menu.
    pub secondary_clicked: bool,
    /// Whether a measure tool is armed.
    ///
    /// The one term that is true on **every** frame rather than on the frame
    /// of an event, and deliberately: the snap indicator has to appear while
    /// the operator is still deciding where to click, and an indicator that
    /// arrived only on the click it exists to guide would be useless. The cost
    /// is one cache hit per frame — see the module header's measurement — and
    /// an un-armed canvas pays nothing because the term is false.
    pub measure_armed: bool,
    /// Whether a Delete or Backspace is pressed on this frame.
    ///
    /// Read with `key_pressed`, which does **not** consume, so
    /// `canvas::keys` still reads the same key a few hundred lines later and
    /// this term cannot swallow the keystroke it is asking about.
    pub delete_pressed: bool,
    /// The canvas selection, for the rung it is on.
    pub selection: &'a SelectionState,
}

impl Need<'_> {
    /// **The whole answer**: does this frame want the page's object model?
    ///
    /// The four terms are OR-ed and each is documented on its own field or
    /// function. Nothing here short-circuits for cost reasons — every term is a
    /// field read or a `matches!`, and the expensive thing is what this decides
    /// to call, not the deciding.
    #[must_use]
    pub fn wanted(&self) -> bool {
        self.secondary_clicked
            || self.measure_armed
            || self.delete_at_a_deeper_rung()
            || gesture_needs_model(self.outcome)
    }

    /// **The term a list of gesture outcomes structurally cannot hold.**
    ///
    /// Delete at the Part or Node rung reaches
    /// [`crate::canvas::deleting::subject`], which needs the decomposition to
    /// answer *what kind of part is this* — a subpath and a show operator wear
    /// the same `subpath: Some(n)` field on a
    /// [`crate::canvas::selection::Selection`] and reach **different engine
    /// verbs** (`delete_subpath` and `delete_text_run`). Without a model it
    /// declines `NoObjectModel`, which is the correct refusal for *"the page
    /// would not decompose"* and was, for one commit, reporting *"nobody asked
    /// for it"*.
    ///
    /// # Why the rung is part of the condition
    ///
    /// The **Object** rung answers from the selection alone: an entry already
    /// holds a resolved `TargetId` and `object_indices_on` is a filter over
    /// four integers. Asking for a decomposition on every Delete would make
    /// the commonest destructive keystroke in the program pay 531 ms after
    /// each content edit for a value that arm never reads. `subject`'s own
    /// signature encodes the same asymmetry and says why.
    ///
    /// # Why it is deliberately over-broad in the other direction
    ///
    /// This does not replicate `canvas::keys`' guards — a focused text widget,
    /// a draft in flight, a mode without `edit_content`, a form field or an
    /// annotation claiming the key first. Every one of those would be a second
    /// statement of a rule that already exists three hundred lines away, free
    /// to drift from it; and being wrong in this direction costs a **cache
    /// hit**, while being wrong in the other direction is the defect this
    /// module exists to end. Over-asking is the safe error and it is chosen on
    /// purpose.
    #[must_use]
    pub fn delete_at_a_deeper_rung(&self) -> bool {
        self.delete_pressed && self.selection.level() != SelectionLevel::Object
    }
}

/// **Does this gesture outcome need the page's object model?**
///
/// An exhaustive `match` with **no wildcard arm**, and that is the whole
/// point of the function. A new [`GestureOutcome`] variant is a compile error
/// here until somebody answers this question for it — where the `matches!`
/// this replaces would have answered `false` in silence, which is exactly how
/// `Resize`, `Handle` and `DimensionVertex` each shipped needing the model and
/// not asking for it.
///
/// ⚠ **Do not add a `_ =>` arm.** It compiles, it looks tidy, and it restores
/// the defect this function was written to remove. If a variant genuinely does
/// not need the model, say so by name — the `false` arms below are a list of
/// deliberate answers, not a default.
#[must_use]
pub fn gesture_needs_model(outcome: &GestureOutcome) -> bool {
    match outcome {
        // ---- needs it: the hit test ---------------------------------------
        //
        // A click has to know what is under the pointer in order to select it.
        GestureOutcome::Click { .. } => true,

        // A **move drag** is in the set at either phase, and it is the one
        // member that is not a hit test — which is why the flag this feeds is
        // named for what it gates rather than for what most of its members do.
        // It needs the model to answer two questions the selection alone
        // cannot: *is every selected object a path* (a non-path refuses the
        // whole move, and a ghost drawn over one would promise a move that
        // gets refused), and, at the Node rung, *where is the anchor now*
        // (`move_node` takes a destination, not a delta).
        //
        // Asking on every frame of an in-flight drag is affordable because the
        // answer is already built: the selection cannot have outlines to drag
        // without a decomposition, so this is a cache hit for the whole
        // gesture.
        GestureOutcome::Move { .. } => true,

        // `Resize` joined this set on 2026-08-19, and its absence was the
        // second defect the first driven resize found. The decomposition is
        // what `canvas::resizing` reads every node position out of, so without
        // it the commit declined with `NoObjectModel` — a refusal that is
        // correct for *"the model could not be read"* and was here reporting
        // *"nobody asked for it"*. The list was written when a resize
        // committed nothing, so there was genuinely nothing for it to need.
        GestureOutcome::Resize { .. }
        // Same reason as `Resize`, and it was learned there: the commit
        // needs the object model to refuse a stale index, and a gesture on a
        // canvas that never asked for a provider gets `None` and declines. The
        // resize spent a whole driving session on exactly this.
        | GestureOutcome::Handle { .. }
        | GestureOutcome::DimensionVertex { .. }
        // …and `Rotate`, for `Resize`'s reason: the commit resolves
        // paint-order indices, and a gesture on a canvas that never asked for
        // a provider would address indices nothing has verified.
        | GestureOutcome::Rotate { .. } => true,

        // ---- the marquee, which is two different gestures ------------------
        //
        // A **zoom** marquee is deliberately NOT in the set. It selects
        // nothing, so it hit-tests nothing, so it decomposes nothing — a
        // region zoom over a 129,758-object drawing costs one scroll offset.
        // That falls out of the intent being carried on the outcome rather
        // than being asked for at the release, and it is the concrete payoff
        // for sampling it at the press.
        //
        // An **in-flight** select marquee is out too: the band is drawn in
        // canvas space and nothing is resolved until it is let go.
        GestureOutcome::Marquee { phase, intent, .. } => {
            *phase == Phase::Complete && *intent == MarqueeIntent::Select
        }

        // ---- does not need it ---------------------------------------------
        //
        // Each of these is answered by name rather than by a default arm. The
        // authoring gestures below all end in a rectangle or a path in page
        // space and none of them asks what was already on the page: a band is
        // normalised into a page rect and handed to a verb that creates
        // something new.
        GestureOutcome::Idle
        | GestureOutcome::Cancelled
        | GestureOutcome::TextBox { .. }
        | GestureOutcome::MarkupVertex { .. }
        | GestureOutcome::TextSelect { .. }
        | GestureOutcome::Markup { .. }
        | GestureOutcome::TextAnnot { .. }
        | GestureOutcome::FormField { .. }
        | GestureOutcome::Place { .. } => false,
    }
}

#[cfg(test)]
mod tests;
