//! # `canvas::gesture::outcome` — what a gesture PRODUCED
//!
//! Split out of `canvas/gesture/mod.rs` on 2026-08-18 under rule R2, when the
//! text-annotation band made a fifth `DragKind` and took that file past the
//! 1,500-line ceiling.
//!
//! ## The seam
//!
//! `super` owns the **machine**: [`super::GestureState`], which watches a
//! pointer across frames and decides when a press has become a drag, when a
//! drag has ended, and when a click is a click. This file owns the
//! **vocabulary that machine speaks in** — the phase of a gesture, the outcome
//! it reports, and the in-flight `Drag` that turns one into the other.
//!
//! They change for different reasons, which is the test for a seam rather than
//! a line count. A new *kind* of thing the canvas can do is a change here: a
//! variant, its fields, and the arm in [`Drag::outcome`] that shapes it. A new
//! rule about *when* a press counts as a drag is a change in `super`, and
//! touches nothing in this file.

use super::{DragKind, Grip, MarqueeIntent};
use crate::canvas::markup::MarkupKind;
use egui::{Pos2, Rect, Vec2};

/// Whether a drag is still happening or has just finished.
///
/// Both matter and they mean different things: an in-flight drag draws a
/// rubber-band or a ghost outline (a pre-commit affordance — the cursor
/// describing what is about to happen), while a completed one changes the
/// selection or raises an action. Collapsing them into one signal is how a
/// marquee ends up committing on every frame it is dragged across.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// The pointer is still down. Draw, do not commit.
    InFlight,
    /// The pointer has been released. Commit.
    Complete,
}

/// What the canvas should do about the pointer this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureOutcome {
    /// Nothing to do. **This is what a press produces** — see invariant 2 in
    /// the module docs.
    Idle,
    /// A drag in flight was **abandoned by Escape** without committing.
    ///
    /// Distinct from [`Self::Idle`] on purpose, and the distinction is
    /// load-bearing in both directions:
    ///
    /// * it tells the caller the key was **consumed**, so the same press must
    ///   not also ascend the selection ladder — one press, one effect, which
    ///   is decision 025's L1 applied to the gesture layer;
    /// * it is raised only when a drag was genuinely in flight, so Escape with
    ///   an idle pointer falls straight through to the ladder and the operator
    ///   never has to press it twice to leave a rung.
    ///
    /// Nothing is committed and nothing is drawn. A cancelled move puts the
    /// object back where it was for the same reason an *interrupted* drag
    /// does — see [`GestureState::update`]'s last branch — except that this one
    /// is the operator asking, so it is reported rather than silent.
    Cancelled,
    /// A completed click with no drag: the only outcome that may change the
    /// selection by hit test, and the only one that may clear it.
    Click {
        /// Canvas-space position of the click.
        point: Pos2,
        /// Whether Shift was held (extend rather than replace).
        shift: bool,
        /// Whether this was the second click of a double-click (descend a
        /// rung rather than pick at the current one).
        double: bool,
        /// Whether this was the **third** click of a triple-click.
        ///
        /// Carried beside `double` rather than replacing it with a count,
        /// because the two consumers want different questions asked: the
        /// selection ladder and the measure tools ask *"was this a double?"* and
        /// must keep the answer they have, while the text gesture asks all
        /// three in order. Exactly one of the two flags can be set — see
        /// [`PointerFrame::triple_clicked`].
        triple: bool,
    },
    /// A rubber-band, in canvas space.
    Marquee {
        /// The band, normalised — dragged in any of four directions.
        rect: Rect,
        /// ★★★ **Was it dragged RIGHT TO LEFT?** — `OPERATOR_REQUESTS.md` O88.
        ///
        /// AutoCAD's names for the two bands: a left-to-right drag is a
        /// **window** and takes only what it completely surrounds; a
        /// right-to-left drag is a **crossing window** and takes anything it
        /// touches. SolidWorks drawings use the same rule, and it is the one a
        /// drawing-office hand already has.
        ///
        /// # Why one BOOLEAN and not the two raw endpoints
        ///
        /// [`Self::TextBox`], [`Self::Rotate`] and the markup band all carry
        /// their endpoints raw and normalise at the boundary, and the reason
        /// they give is real: `Rect::from_two_pos` discards which corner the
        /// press was at. This variant is the exception, and deliberately.
        ///
        /// **Every consumer of this band wants it normalised** — the hit test,
        /// the painter, the zoom. Exactly one bit is discarded and exactly one
        /// consumer needs it. Carrying `from`/`to` instead would let each
        /// consumer re-derive the direction, and *the direction decides what the
        /// gesture MEANS*: two consumers that disagreed about it would select one
        /// set of objects and paint the band for a different one.
        ///
        /// ⇒ Decided once, here, exactly as [`Self::Marquee::shift`] is sampled
        /// once at the press rather than re-read by whoever needs it.
        ///
        /// ★ The comparison is on **x only**. A drag that goes left and up is a
        /// crossing window; one that goes right and down is a window. AutoCAD's
        /// rule is horizontal, and a rule that also read y would give the four
        /// diagonal drags two meanings apiece with nothing on screen to say
        /// which.
        crossing: bool,
        /// Whether Shift was held at the press (extend rather than replace).
        shift: bool,
        /// What the release does: select what is enclosed, or zoom to it.
        /// Sampled at the press — see the module docs.
        intent: MarqueeIntent,
        /// Draw, or commit.
        phase: Phase,
    },
    /// A move drag of the current selection, as a canvas-space delta.
    Move {
        /// How far the pointer has travelled since the press.
        delta: Vec2,
        /// Draw the ghost, or commit the move.
        phase: Phase,
    },
    /// ★★ A **rotate** drag on the handle above the selection box.
    ///
    /// # Why this carries two POSITIONS and not a delta
    ///
    /// Because what the gesture reads is a **bearing**, and a bearing needs a
    /// ray. `from` is where the press landed and `at` is where the pointer is;
    /// the angle between the two rays *from the selection's centre* is the whole
    /// content of the drag.
    ///
    /// A delta cannot express it. The same displacement means a different turn
    /// depending on where round the box it happened — and, more sharply, the
    /// pointer's **distance** from the centre must mean nothing at all, which a
    /// delta has no way to discard. See [`crate::canvas::rotating`].
    ///
    /// The centre is not carried: it is a pure function of the selection's grip
    /// box, which the caller already has, and carrying it would be a second copy
    /// that could go stale between the frame that computed it and the frame that
    /// draws — `painting`'s own argument for re-reading `grip_box`.
    Rotate {
        /// Canvas-space position of the press — the first ray.
        from: Pos2,
        /// Canvas-space position of the pointer now — the second ray.
        at: Pos2,
        /// Draw the ghost, or commit.
        phase: Phase,
    },
    /// ★★ A **text box** being dragged out: the two raw endpoints, in canvas
    /// space.
    ///
    /// Raw and in drag order, for the reason [`Self::Markup`] states at length —
    /// `Rect::from_two_pos` has one normalised form and discards which corner
    /// the operator started at. It is normalised exactly once, at the point it
    /// becomes a page rectangle, so a preview and an authored box cannot come
    /// from two different normalisations.
    ///
    /// # What `Phase::Complete` means here, and why it is a separate variant
    ///
    /// It **opens a draft**, and authors nothing. The band is the cursor; the
    /// words decide whether anything is written, and an empty box committed by
    /// clicking away writes nothing at all. That is the same shape as
    /// [`Self::TextAnnot`] — which opens a dialog — and different from
    /// [`Self::Markup`], which authors on release.
    TextBox {
        /// Canvas-space position of the press — one corner.
        from: Pos2,
        /// Canvas-space position of the pointer now — the opposite corner.
        to: Pos2,
        /// Draw the band, or open the draft.
        phase: Phase,
    },
    /// A resize drag on one of the eight grips.
    ///
    /// Raised so the drag is **consumed** rather than falling through to a
    /// marquee. `pdfcer-core` has no scale verb for a vector object, so
    /// nothing commits on `Complete` yet — see [`crate::canvas::handles`] for
    /// the whole reasoning and the roadmap row that gives it a verb.
    Resize {
        /// Which grip is being dragged.
        grip: Grip,
        /// How far the pointer has travelled since the press.
        delta: Vec2,
        /// Draw, or commit.
        phase: Phase,
    },
    /// A **Bézier handle** drag.
    ///
    /// # ★ Why this carries the pointer's CANVAS POSITION and the others carry
    /// a delta
    ///
    /// Because `EditSession::move_handle` takes the control point's **new
    /// position**, not a displacement — and it takes it in PDF user space,
    /// which means the conversion has to happen against the frame's own
    /// mapping. A delta would make the caller reconstruct "where the handle
    /// started" in order to add to it, and the only thing that knows where it
    /// started is the decomposition the caller already has to ask for.
    ///
    /// So the outcome carries where the pointer *is*, the caller converts once,
    /// and the handle ends up under the cursor — which is also what the
    /// operator expects, and what a delta-based implementation gets subtly
    /// wrong the moment the press was a pixel or two off the handle's centre.
    Handle {
        /// The anchor, object-scoped.
        node: usize,
        /// Arriving or leaving.
        handle: pdfcer_core::vector::Handle,
        /// Where the pointer is now, in canvas space.
        at: egui::Pos2,
        /// Draw, or commit.
        phase: Phase,
    },
    /// A **perimeter ce dimension's vertex** being dragged.
    ///
    /// Carries the pointer's canvas position rather than a delta, for the
    /// reason [`Self::Handle`] gives for the same choice: the commit needs the
    /// page-space point the corner is going TO, and deriving it from a delta
    /// would mean adding it to a stored origin that the gesture machine already
    /// has and would then have to hand over anyway.
    DimensionVertex {
        /// Which vertex, sampled at the press.
        index: usize,
        /// Where the press landed, in canvas space.
        ///
        /// ★ Carried so the GRAB POINT can be preserved (`drag-moves` D8).
        /// Without it the only thing to do with `at` is assign it to the
        /// vertex, which teleports the corner under the cursor on the first
        /// frame — the operator grabbed a handle a few pixels off centre and
        /// the shape jumps before they have moved.
        from: egui::Pos2,
        /// Where the pointer is now, in canvas space.
        at: egui::Pos2,
        /// Draw, or commit.
        phase: Phase,
    },
    /// A **markup shape's node** being dragged — a `/Polygon`, `/PolyLine` or
    /// `/Line` the operator drew as a comment. `Pass 255.0`.
    ///
    /// The same three fields as [`Self::DimensionVertex`], for the same reasons,
    /// and a **separate variant** because it reaches a different family of
    /// engine verbs: `reshape_annotation` and its three wrappers, against
    /// `move_dimension_vertex` and its two. The one thing that must never
    /// happen on this canvas is a gesture aimed at the wrong verb, and a shared
    /// variant with a discriminator inside it is what invites exactly that —
    /// `DragKind::DimensionVertex`'s own note makes the argument against
    /// folding it into `Handle` in the same words.
    ///
    /// ★ R8b rule 15: this is a **markup shape**. A **ce dimension** is also a
    /// `/Line` and is claimed by the variant above; **pdf dimensions** are CAD
    /// page content and are not annotations at all.
    MarkupVertex {
        /// Which node, sampled at the press.
        index: usize,
        /// Where the press landed, in canvas space — so the grab point can be
        /// preserved (`drag-moves` D8).
        from: egui::Pos2,
        /// Where the pointer is now, in canvas space.
        at: egui::Pos2,
        /// Draw, or commit.
        phase: Phase,
    },
    /// A **text sweep**: the two raw endpoints of the drag, in canvas space.
    ///
    /// Raw and in drag order, for the reason [`Self::Markup`] states at length
    /// — `Rect::from_two_pos` has one normalised form and discards which corner
    /// the operator started at. For a text range that direction is not merely
    /// information, it is the **anchor**: a Shift+click after the drag extends
    /// from `from`, and a normalised pair would put the anchor at whichever end
    /// happened to be higher on the page.
    ///
    /// Unlike `Markup`, both phases are acted on rather than only the release:
    /// a text selection has to grow under the pointer while the button is down,
    /// which is the whole feedback of the gesture. Nothing is committed at
    /// either phase — see [`crate::canvas::textsel`]'s header §6 — so
    /// `Phase::Complete` differs from `Phase::InFlight` only in that it is the
    /// frame worth tracing.
    TextSelect {
        /// Canvas-space position of the press — the selection's **anchor**.
        from: Pos2,
        /// Canvas-space position of the pointer now — the selection's **focus**.
        to: Pos2,
        /// Grow the selection, or settle it.
        phase: Phase,
    },
    /// A markup band: the shape being authored, and its two **raw** endpoints
    /// in canvas space.
    ///
    /// # ★ Why this carries two points and not a `Rect`
    ///
    /// Because a `Rect` cannot express which corner the operator started at,
    /// and for an arrow that is the entire content of the gesture. `Rect` has
    /// exactly one normalised form; [`Rect::from_two_pos`] discards the drag
    /// direction on the way in, and no downstream code can recover it. An arrow
    /// built from a normalised rect points up-and-left for every drag,
    /// whichever way the operator went — silently, because the annotation that
    /// lands is a perfectly valid arrow.
    ///
    /// So the pair travels raw and the normalisation happens at the one place
    /// that needs a rectangle: [`crate::canvas::markup::spec`], per kind. This
    /// is the same shape of decision as `Marquee` carrying its `MarqueeIntent`
    /// — the release must not have to re-derive something the press knew.
    Markup {
        /// Which shape is being authored, sampled at the press.
        kind: MarkupKind,
        /// Canvas-space position of the press — the arrow's **tail**.
        from: Pos2,
        /// Canvas-space position of the pointer now — the arrow's **head**.
        to: Pos2,
        /// Draw the band, or commit the annotation.
        phase: Phase,
    },
    /// A **text-bearing** annotation's rectangle is being dragged out.
    ///
    /// The markup band's twin in every respect except what `Phase::Complete`
    /// means: there it authors, here it opens a dialog and authors nothing.
    /// See [`crate::canvas::tool::CanvasTool::TextAnnot`] for why that one
    /// difference earns a separate variant.
    TextAnnot {
        /// Which text-bearing kind, sampled at the press.
        kind: crate::canvas::textannot::TextAnnotKind,
        /// Canvas-space position of the press — one corner of the box.
        from: Pos2,
        /// Canvas-space position of the pointer now — the opposite corner.
        to: Pos2,
        /// Draw the band, or ask for the words.
        phase: Phase,
    },
    /// A **form control's** rectangle is being dragged out.
    ///
    /// [`Self::TextAnnot`]'s twin: the band draws identically and
    /// `Phase::Complete` opens a dialog rather than authoring. It is separate
    /// because the dialog and the kind are both different, and because a form
    /// control's release has to carry which of the five kinds it is.
    FormField {
        /// Which of the five kinds, sampled at the press.
        kind: crate::canvas::formfield::FormFieldKind,
        /// Canvas-space position of the press — one corner of the control.
        from: Pos2,
        /// Canvas-space position of the pointer now — the opposite corner.
        to: Pos2,
        /// Draw the band, or ask for the details.
        phase: Phase,
    },
    /// ★★★ **A window is waiting for a box** — `OPERATOR_REQUESTS.md` O66.
    ///
    /// Shaped exactly like [`Self::FormField`], and for the same reason: the
    /// two corners travel raw and in drag order, and whoever turns them into a
    /// page rect normalises once.
    ///
    /// ★ Unlike every other outcome here, **nothing on this canvas commits
    /// it.** `canvas::placing` writes the answer to `egui::Memory` and the
    /// requesting dialog reads it back through `app::frame`, because the
    /// operator has not pressed Insert yet and may still change the numbers.
    Place {
        /// Which window is waiting, sampled at the press.
        kind: crate::canvas::placing::PlaceKind,
        /// Where the drag began, in canvas space.
        from: Pos2,
        /// Where the pointer is now, in canvas space.
        to: Pos2,
        /// In flight, or released.
        phase: Phase,
    },
}

/// A primary-button drag in flight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Drag {
    /// Canvas-space position of the press.
    pub(super) origin: Pos2,
    /// Canvas-space position of the most recent frame that had one.
    ///
    /// Held rather than re-read, so a frame in which the pointer left the
    /// window continues the gesture from where it was last seen instead of
    /// collapsing it to the origin — which would look like the object
    /// snapping back to where it started.
    pub(super) latest: Pos2,
    /// What the press landed on.
    pub(super) kind: DragKind,
    /// Whether Shift was held **at the press**.
    ///
    /// At the press, not at the release: an operator who lets go of Shift
    /// before the mouse button has still asked for an extending marquee, and
    /// sampling the modifier at the end would make the gesture's meaning
    /// depend on the order two fingers came up.
    pub(super) shift: bool,
}

impl Drag {
    /// This drag's outcome at `phase`.
    pub(super) fn outcome(self, phase: Phase) -> GestureOutcome {
        let delta = self.latest - self.origin;
        match self.kind {
            DragKind::Marquee(intent) => GestureOutcome::Marquee {
                rect: Rect::from_two_pos(self.origin, self.latest),
                // ★ Strictly less-than, so a perfectly vertical drag is a
                // WINDOW. It has to fall one way and the safe way is the
                // existing behaviour: a vertical band that silently became a
                // crossing window would take everything it grazed on a gesture
                // the operator has been making for weeks.
                crossing: self.latest.x < self.origin.x,
                shift: self.shift,
                intent,
                phase,
            },
            DragKind::Move => GestureOutcome::Move { delta, phase },
            // Raw and in drag order, exactly as the markup band and the
            // text-annotation band above — the rectangle is normalised once, at
            // the point it becomes a page rect.
            DragKind::TextBox => GestureOutcome::TextBox {
                from: self.origin,
                to: self.latest,
                phase,
            },
            DragKind::Resize(grip) => GestureOutcome::Resize { grip, delta, phase },
            // Raw, and in that order: `origin` is the ray the press established
            // and `latest` is the ray the pointer is on. Passing a delta here —
            // which is what every neighbouring arm does — would discard the
            // thing this gesture is made of.
            DragKind::Rotate => GestureOutcome::Rotate {
                from: self.origin,
                at: self.latest,
                phase,
            },
            DragKind::DimensionVertex { index } => GestureOutcome::DimensionVertex {
                index,
                from: self.origin,
                at: self.latest,
                phase,
            },
            DragKind::MarkupVertex { index } => GestureOutcome::MarkupVertex {
                index,
                from: self.origin,
                at: self.latest,
                phase,
            },
            DragKind::Handle { node, handle } => GestureOutcome::Handle {
                node,
                handle,
                // `latest`, not `origin + delta` — the same number by
                // construction, and the one that says what it means.
                at: self.latest,
                phase,
            },
            // Raw, and in that order: `origin` is where the press landed and
            // `latest` is where the pointer is. Passing them through
            // `Rect::from_two_pos` here — which is what the marquee above does
            // one line up, and what a reader tidying this file would reach for —
            // is exactly the reversal `GestureOutcome::Markup`'s docs describe.
            DragKind::Markup(kind) => GestureOutcome::Markup {
                kind,
                from: self.origin,
                to: self.latest,
                phase,
            },
            // ★ Raw and in drag order, exactly as the markup band above — the
            // rectangle is normalised once, at the point it becomes a page
            // rect, so a preview and an authored box cannot come from two
            // different normalisations.
            DragKind::TextAnnot(kind) => GestureOutcome::TextAnnot {
                kind,
                from: self.origin,
                to: self.latest,
                phase,
            },
            // Raw and in drag order, for the reason the two bands above give:
            // the rectangle is normalised exactly once, where it becomes a page
            // rect, so the band the operator watched and the control that is
            // authored cannot come from two different normalisations.
            DragKind::Form(kind) => GestureOutcome::FormField {
                kind,
                from: self.origin,
                to: self.latest,
                phase,
            },
            // Raw and in drag order, like every band above — `canvas::placing`
            // normalises once, at the point the two corners become a page rect,
            // so a band dragged up-and-left and one dragged down-and-right
            // produce the same answer without either of them being rewritten
            // mid-gesture. O66.
            DragKind::Place(kind) => GestureOutcome::Place {
                kind,
                from: self.origin,
                to: self.latest,
                phase,
            },
            // Raw and in drag order, for the same reason the markup band above
            // is: `origin` is the anchor the operator chose and `latest` is
            // where they have got to. Normalising here would silently move the
            // anchor to the top-left of the sweep, which a later Shift+click
            // would then extend from.
            DragKind::TextSelect => GestureOutcome::TextSelect {
                from: self.origin,
                to: self.latest,
                phase,
            },
        }
    }
}
