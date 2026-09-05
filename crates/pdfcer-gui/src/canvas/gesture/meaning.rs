//! # `canvas::gesture::meaning` — what a press MEANS, decided once and then remembered
//!
//! One pure function, [`press_kind`], and the two enums it decides between:
//! [`DragKind`], which says what a drag is going to *do*, and [`MarqueeIntent`],
//! which says what a rubber band does when it is released. Nothing in this file
//! holds state, touches egui, or knows that frames exist — a press is
//! `(tool, grip, zoom_armed, capabilities)` in and one meaning, or `None`, out.
//! That is what makes the whole precedence testable as a table.
//!
//! The state machine that *carries* a meaning across a press, a drag and a
//! release — [`PointerFrame`](super::PointerFrame),
//! [`GestureState`](super::GestureState) and
//! [`GestureOutcome`] — is the parent module, [`super`].
//! It calls this function on exactly one frame per gesture, the press frame,
//! and then never asks again.
//!
//! The precedence itself — which meaning wins when two are available, and which
//! presses a mode refuses outright — is documented on [`press_kind`], because it
//! *is* the rule rather than a note about it.
//!
//! [`press_kind`] deliberately has no case for the hand tool, and the absence is
//! load-bearing: `canvas::interact` hands the state machine a **blank** frame
//! while the hand is active, so no press ever arrives here to be classified.
//! That rule — *one state machine, one meaning per frame* — is stated in full in
//! [`super`]'s header, under "Marquee versus pan".
//!
//! ## ★ Marquee-select versus marquee-zoom: one rubber band, two releases
//!
//! Phase 3.4 adds a marquee that *zooms* to what it encloses. It is
//! deliberately **the same gesture**: same press, same in-flight rect, same
//! pixels on screen ([`crate::canvas::overlay::draw_marquee`] is not
//! duplicated), same normalisation, same Escape. What differs is one thing —
//! *what happens on release* — so what is carried is one value, [`MarqueeIntent`].
//!
//! It is sampled **at the press**, exactly as `shift` is, and for the identical
//! reason: the one-shot arming is retired when the drag completes, and an
//! intent re-read at release would be read after something else had already
//! consumed it. A gesture means what it meant when it started.

use crate::app::modes::Capabilities;
use crate::canvas::formfield::FormFieldKind;
use crate::canvas::handles::Grip;
use crate::canvas::markup::MarkupKind;
use crate::canvas::tool::CanvasTool;

// Rustdoc-only: the doc comments below link to items that live in the parent
// module. The link targets have to be nameable here for the reference to
// resolve, but nothing in this file's *code* needs them — so the import is
// compiled only when rustdoc is running, and never costs an unused-import
// warning in a normal build.
#[cfg(doc)]
use super::GestureOutcome;

/// What a press landed on — decided once, on the press frame, by the caller.
///
/// # Why it is decided at press time and then remembered
///
/// A drag that began on a grip stays a resize even when the pointer wanders
/// off the grip, off the object, and off the page. Re-deciding per frame
/// would turn a resize into a marquee the instant the operator's hand moved
/// faster than the box, which is exactly when they are dragging hardest.
///
/// This is also what makes the grips *consume* their drags. See
/// [`crate::canvas::handles`]: without it, a drag aimed at a resize grip
/// would fall through to a marquee and silently replace the selection the
/// operator was trying to resize.
/// What a completed rubber-band does — **the only difference between
/// marquee-select and marquee-zoom.**
///
/// See the module docs. Carried by [`DragKind::Marquee`] and echoed back on
/// [`GestureOutcome::Marquee`] so the release arm can branch on it without
/// asking the world what mode it is in — the world may have changed since the
/// press, and the press is when the operator decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarqueeIntent {
    /// Select everything the band fully encloses. The default, and what an
    /// un-armed canvas does.
    #[default]
    Select,
    /// Zoom the view to the band. Armed by
    /// [`crate::canvas::zoom::arm_region_zoom`] and retired on release.
    Zoom,
}

/// **What a press on a selected ce dimension landed on.**
///
/// Resolved by [`crate::canvas::pressing`] while it has the document and the
/// mapping in hand, so that the meaning function stays free of geometry — the
/// same division `grip` and `handle` already follow.
///
/// `None` at the call site means *no ce dimension is selected, or the press
/// missed it*, and the press falls through to the ordinary rungs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionPress {
    /// On one of a perimeter's corner handles, by index.
    ///
    /// Outranks [`Self::Body`] because a corner handle sits ON the shape, so
    /// every press that hits a handle also hits the body — and of the two
    /// readings, the one the operator aimed at is the small square they can
    /// see.
    Vertex(usize),
    /// Inside the dimension's own box, but not on a handle.
    Body,
}

/// **Which family the selected annotation under a rotate handle belongs to.**
///
/// ★★★ Carried as a *variant* rather than as a bool, for
/// `canvas::selection::annot::AnnotKind`'s own stated reason and for a second
/// one this function needs: the two families are **gated by different
/// capabilities**, and a bool would have to be paired with a second bool
/// saying which gate to ask.
///
/// | | verb | capability | why that one |
/// |---|---|---|---|
/// | [`Self::Markup`] | `rotate_annotation` | `author_markup` | markup is authored in **Review**, where `edit_content` is false — an operator who has just drawn a shape there and wants to turn it is in the mode the content branch does not run in |
/// | [`Self::CeDimension`] | `rotate_dimension` | `author_measure` | turning a dimension is a **measure** edit: it writes the sidecar and one annotation and touches no page content. The same ruling the vertex drag already ships under, and for the same reason — a mode that may author a dimension may adjust the one it just authored |
///
/// ★★ Resolved by [`crate::canvas::pressing`] while it has the document and the
/// mapping in hand, so this module stays free of geometry — the same division
/// `grip`, `handle` and [`DimensionPress`] already follow. **It is `Some` only
/// when the press origin is actually on the handle**, because it is derived
/// from the very `grip` this function reads, through the very `GripSet` the
/// painter uses. One predicate; see [`crate::canvas::handles::GripSet`] H7.
///
/// ⇒ That last property is the guard against **the hazard this canvas has
/// produced four times**: a working gesture aimed at the wrong verb. The most
/// recent was a `covers()` that tested the selection's *move box* alone — and
/// the rotate handle sits OUTSIDE that box, so a press on it selected the
/// object underneath and the rotate became a select-and-move. Nothing here asks
/// a second question about where the pointer is; it asks
/// `handles::grip_at`, which is the function the gesture machine asks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotatableAnnot {
    /// Ordinary markup — a shape, a note, a stamp, a text markup.
    Markup,
    /// A **ce dimension**: a `/Line` with `/IT /LineDimension` and a sidecar
    /// record.
    CeDimension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragKind {
    /// The press was on empty paper, or on unselected content: rubber-band,
    /// doing whatever [`MarqueeIntent`] says on release.
    Marquee(MarqueeIntent),
    /// The press was inside the selection's body: move it.
    Move,
    /// ★★ The press began a **text box** — a rectangle to type a paragraph
    /// into.
    ///
    /// Carries nothing: the rectangle is the drag's own two endpoints, which
    /// [`Drag::outcome`] already has, and there is no per-press decision to
    /// sample. Compare [`Self::Markup`], which carries a `MarkupKind` because
    /// *which shape* was armed decides what the release authors.
    ///
    /// # ★ Why it is not `Markup(MarkupKind::Rectangle)` with a flag
    ///
    /// Because what it authors is **page content**, not an annotation. A markup
    /// rectangle is a `/Square` the operator can select, restyle and delete as a
    /// comment; this writes glyphs into the page's own stream through
    /// `add_text`. They look identical while the band is being dragged and have
    /// nothing else in common.
    TextBox,
    /// The press was on one of the eight resize grips.
    Resize(Grip),
    /// ★★ The press was on the **rotate handle**, above the top edge.
    ///
    /// Carries nothing, and that is the difference from [`Self::Resize`]: a
    /// resize needs to know WHICH grip, because each one pivots about a
    /// different corner and scales a different pair of axes. There is one
    /// rotate handle, it always turns about the selection's centre, and what
    /// the drag reads is a bearing rather than a displacement — so the press
    /// has nothing to sample that the frame does not already have.
    ///
    /// # ★ Why it is a drag kind rather than `Resize(Grip::Rotate)`
    ///
    /// Because everything downstream of `Resize` computes scale factors from a
    /// delta and a box. Routing the handle there would have produced a
    /// perfectly working *resize* on the ninth grip — an object that got bigger
    /// when the operator meant to turn it, which looks like a deliberate
    /// feature. `Grip::is_resize` is the predicate that keeps them apart and it
    /// is enumerated rather than negated for exactly this reason.
    Rotate,
    /// The press was on a **Bézier handle** of a selected anchor.
    ///
    /// Carries the anchor it belongs to, object-scoped, and which side of it —
    /// arriving or leaving. Both are sampled at the press for the reason
    /// [`MarqueeIntent`] gives: *a gesture means what it meant when it
    /// started*. Re-deriving the handle from the pointer each frame would let a
    /// drag that passed near the other handle silently switch to dragging that
    /// one instead.
    ///
    /// # ★ Why this outranks `Move` in [`press_kind`]
    ///
    /// A handle sits **inside** the selection's box, so `grip_at` would answer
    /// `Grip::Move` for every press on one and the handle would be undraggable
    /// — the same collision that made the corner ANCHORS undraggable until the
    /// eight scale grips were confined to the Object rung. The rule that falls
    /// out of both: **the most specific thing under the pointer wins**, and
    /// specificity here is depth down the selection ladder.
    Handle {
        /// The anchor the handle belongs to, object-scoped.
        node: usize,
        /// Arriving or leaving.
        handle: pdfcer_core::vector::Handle,
    },
    /// The press was on a **vertex of a selected perimeter ce dimension**:
    /// drag that corner and re-measure the shape.
    ///
    /// # ★ Why the index is sampled at the PRESS
    ///
    /// The same rule [`MarqueeIntent`] states — *a gesture means what it meant
    /// when it began*. A vertex drag moves the vertex, which moves every other
    /// vertex's screen position not at all but does change what is nearest the
    /// pointer; resolving "which vertex" per frame would let a drag hop onto a
    /// neighbour it passed over. Carrying it here means one press picks one
    /// corner and keeps it.
    ///
    /// # ★★ And why it is a THIRD drag kind rather than a mode of `Handle`
    ///
    /// A Bézier handle belongs to a path node and commits `move_handle`; this
    /// belongs to a sidecar dimension record and commits
    /// `move_dimension_vertex`. They are the same *gesture* and different
    /// *subjects*, and the one thing that must never happen is a drag reaching
    /// the wrong verb — which is exactly what one shared variant with a
    /// discriminator inside it invites.
    DimensionVertex {
        /// Which vertex of the selected perimeter, by index into its points.
        index: usize,
    },
    /// The press was on the page with the **text tool armed**, or in a mode that
    /// cannot select its content: **sweep a range of text**.
    ///
    /// Carries nothing, unlike the three above, because a text drag's whole
    /// state is its two endpoints and those already travel on
    /// [`GestureOutcome::TextSelect`]. There is no per-drag choice to sample at
    /// the press — no kind, no intent, no grip.
    ///
    /// ★ That emptiness used to carry an extra claim: *"which is itself the
    /// reason the gate for it is a mode question rather than an armed-tool one."*
    /// The inference was wrong and is corrected rather than deleted, because it
    /// is a tempting one. Carrying no per-drag state says nothing about **who
    /// decides** the drag happens; it says only that the deciding does not have
    /// to be *remembered*. Since 2026-08-14 the gate is both — an armed
    /// [`CanvasTool::Text`], or the pre-existing mode rule — and this variant
    /// still carries nothing, because
    /// [`crate::canvas::tool::CanvasTool::Text`] itself carries nothing either.
    /// See [`crate::canvas::textsel`]'s header §3.
    TextSelect,
    /// The markup tool was armed: **draw**, in the carried shape.
    ///
    /// The kind is carried on the drag rather than re-read at the release, for
    /// the identical reason [`MarqueeIntent`] is — *a gesture means what it
    /// meant when it started*. It also gives the markup tool, for free, the
    /// property the old shell had to write code for: changing the armed kind
    /// mid-drag cannot reach a drag already in flight, so there is no
    /// in-progress gesture to discard.
    Markup(MarkupKind),
    /// Dragging out the **rectangle a text-bearing annotation will occupy** —
    /// a text box or a stamp.
    ///
    /// # ★ Its own variant rather than `Markup(kind)`, and the reason is the
    /// completion rule
    ///
    /// A `Markup` drag authors on release. This one does not: the release
    /// opens a dialog and the operator types, and nothing reaches the document
    /// until they accept. Sharing the variant would make every arm that asks
    /// *"does this release author?"* need a second predicate for the exception.
    ///
    /// The sticky note is absent because it is not dragged at all — its rect is
    /// discarded by the format, so it is placed with a click and takes the
    /// click branch beside the measure and caret tools.
    TextAnnot(crate::canvas::textannot::TextAnnotKind),
    /// Dragging out the **rectangle a form control will occupy**.
    ///
    /// [`Self::TextAnnot`]'s twin in shape and in completion rule: the release
    /// opens a dialog and authors nothing. It is a separate variant for the
    /// same reason that one is — an arm asking *"does this release author?"*
    /// gets one answer per variant rather than a predicate — and separate from
    /// **it** because the dialog is a different dialog and the kind is a
    /// different enum.
    ///
    /// ★ There is no click/drag split here, unlike the text-annotation family:
    /// **every** form kind is placed either way. A click means "the default
    /// size for this kind, here", which is a real answer because a form control
    /// has a conventional size — see [`FormFieldKind::default_size_pt`]. A
    /// text box's default size, by contrast, would be a number nobody chose.
    Form(FormFieldKind),
    /// ★★★ **A window stepped aside and is waiting for a box** —
    /// `OPERATOR_REQUESTS.md` O66.
    ///
    /// Like [`Self::Form`] and unlike the text-annotation family there is no
    /// click/drag split: a click and a drag are both answers, and they say
    /// different things. A **click** gives a corner and leaves the size to the
    /// dialog — which already has one, typed or defaulted. A **drag** gives
    /// the whole box.
    ///
    /// ★ The commit does not live on this canvas. It is the requesting
    /// dialog's own Insert, which is why `canvas::placing` writes the answer to
    /// `egui::Memory` for `app::frame` to hand back rather than raising an
    /// `Action` — the operator has not pressed Insert yet and may still change
    /// the numbers.
    Place(crate::canvas::placing::PlaceKind),
}

/// What a press means, given the tool, what it landed on and what is armed —
/// **the whole precedence, in one pure function.**
///
/// Lifted out of `canvas::interact` when the markup tool arrived, because it
/// stopped being a two-case question the moment there were three tools and it
/// is exactly the kind of rule this module exists to hold: it is a decision
/// about what the pointer means, it is drivable with no window, and leaving it
/// as a `match` in the middle of the wiring is how the ordering below becomes
/// three separate opinions.
///
/// # The order is the rule
///
/// 1. **An armed markup tool outranks everything**, including the grips. A
///    markup drag that started on a selected object's resize handle must draw a
///    shape, not resize — the operator armed a pen, and grips belong to a
///    selection they are not currently acting on. (There is no resize verb to
///    reach anyway; see [`crate::canvas::handles`].) It outranks the region
///    zoom for the same reason: only one of the two can own the primary drag,
///    and the one the operator armed *last* is not knowable here — but the one
///    that authors content is the one whose loss would be silent.
///
///    ★ This rung sees only the **band** and **freehand** kinds. The two
///    vertex kinds are answered by an early return above, beside the measure
///    tools, because their gesture is clicks and they have no drag at all —
///    [`crate::canvas::markup::MarkupKind::is_vertex`], and the comment at the
///    branch itself.
/// 2. **An armed text tool**, which sweeps a range in *every* mode — including
///    the ones whose primary button is otherwise the content marquee. It sits
///    here, above the content branch, because that branch is total: below it this
///    rung would be unreachable in exactly the mode the tool was built for. It
///    yields to an armed region zoom, and only to that; see the comment at the
///    branch itself for why the ordering is borrowed from the reading-mode text
///    row rather than decided afresh.
/// 3. **A grip** — resize on the six that resize, move on the two that do not.
/// 4. **An armed region zoom**, which turns the marquee's release into a zoom.
/// 5. **A plain marquee**, which is what an un-armed canvas does.
///
/// The hand tool is deliberately **absent** from this list, and its absence is
/// load-bearing: `canvas::interact` hands the gesture machine a *blank* frame
/// while the hand is active, so no press ever reaches this function to be
/// classified. One state machine, one meaning per frame — see the module
/// header.
///
/// # ★ The mode gate lives here, and it is two answers rather than one
///
/// The mode's [`Capabilities`] are applied **here**, at the point where a press
/// is given its meaning, rather than at the several places that act on one.
/// That ordering is the whole design: a press whose meaning is forbidden never
/// becomes a drag, so there is no band to draw, no ghost to preview, no
/// release to refuse and no half-gesture to explain.
///
/// [`PressMeaning`] carries **two** answers because the canvas has two kinds of
/// tool and they take the primary button differently — see that type's header.
/// A single `Option<DragKind>` was the first shape of this gate and it was
/// wrong in a way that would not have shown up until Review mode was used in
/// anger: it made "a drag means nothing here" and "a click means nothing here"
/// the same fact, which is exactly false for the measure tools, whose entire
/// gesture is clicks.
///
/// Refusing at the *press* is also what keeps the safety rule intact
/// (`MODES_AND_PANELS.md`: *"It never makes a visible control silently
/// inert"*). Nothing visible is refused, because in a mode that cannot select
/// there is no selection, hence no handles and no outline — see
/// `app::modes::capability` §5 and `PdfcerApp::on_mode_capabilities_changed`,
/// which clears the selection on the way in precisely so that this function
/// never has to refuse a grip the operator can see.
///
/// Which capability each meaning needs:
///
/// | Meaning | Needs |
/// |---|---|
/// | [`DragKind::Markup`] | `author_markup` |
/// | a **vertex-markup** click — PolyLine, Polygon | `author_markup`, and it is the same flag on purpose: these author a comment, so a mode that draws rectangles draws polygons |
/// | a measure **click** | `author_measure` |
/// | [`DragKind::Resize`], [`DragKind::Move`] | `edit_content` |
/// | [`DragKind::Marquee`] with [`MarqueeIntent::Select`] | `edit_content` |
/// | a selecting **click** | `edit_content` |
/// | [`DragKind::Marquee`] with [`MarqueeIntent::Zoom`] | **nothing** — it is a navigation gesture that reads the document and changes none of it, so it is offered in every mode, Read included |
/// | [`DragKind::TextSelect`], and the click that goes with it | **nothing** — either because the operator armed the text tool, or *because* `edit_content` is absent, which is the one row here that reads backwards |
///
/// # ★ The text row, and why it is not an inconsistency
///
/// Every other row above asks *"does this mode permit the gesture?"*. The text
/// row asks *"is the primary button free, or has the operator claimed it?"* —
/// which is a different question that happens to read the same flag in one of
/// its two halves. That is not a capability inverted.
///
/// Selecting text authors nothing — it reads the page and writes to the
/// clipboard, which is the operator's own *copying is not authoring* ruling of
/// 2026-08-14 — so there is nothing here to permit, in either half. What there
/// is, is a collision: in a mode that selects page content the primary drag is
/// already the marquee. `crate::canvas::textsel::takes_the_press` is the one
/// place that collision is resolved, this function asks it, and
/// `canvas::interact` asks the same function again when it routes the click — so
/// the two cannot disagree about what a press meant. That module's header §3
/// carries the full argument, including why a `select_text` capability would
/// have been the wrong shape.
///
/// ★ **Since 2026-08-14 the predicate has two disjuncts**, and the second one
/// changes where the exclusivity comes from rather than whether there is any:
///
/// * **un-armed** (`CanvasTool::Select` in a mode that cannot select content) —
///   exclusive **by construction**, one flag on both sides of one branch, which
///   is how it shipped;
/// * **armed** (`CanvasTool::Text`, in any mode) — exclusive **by precedence**,
///   at rung 2 above, which is the rule `DragKind::Markup` has always used.
///
/// The property an operator can feel is untouched either way: this function
/// returns one [`DragKind`], so one press has one meaning. What a reader has to
/// know is that in Edit *both underlying facts* can now be true at once — the
/// mode can select content, and the operator has asked for text — so the order
/// of the branches below is load-bearing where it previously was not.
/// **What the primary button may do this frame** — the answer [`press_kind`]
/// returns and [`super::GestureState::update`] acts on.
///
/// # ★ Why two fields rather than one `Option<DragKind>`
///
/// Because the canvas has two kinds of authoring tool and they take the primary
/// button in genuinely different ways:
///
/// | tool | the gesture is | uses |
/// |---|---|---|
/// | markup — rectangle, ellipse, arrow, highlight | press, drag out a shape, release | the **drag** |
/// | measure — linear, radius/diameter, two-line, scale | click point A, click point B, click where the dimension sits | the **click** |
/// | select | either: click to select, drag to marquee | both |
/// | text | either: drag to sweep a range, click to take a word / a line / extend / clear | **both**, and this is the row that shows why the two fields cannot be collapsed even for a tool with no state — three of the gesture's four meanings are clicks (`canvas::textsel` §1) |
///
/// A single `Option<DragKind>` cannot express that, and the gate's first
/// version proved it: it suppressed the click whenever it suppressed the drag,
/// which is right for Read (neither means anything) and **wrong for Review**,
/// where a dimension must be placeable and page content must not be
/// selectable. The two facts have to be separable because a mode really does
/// grant one without the other.
///
/// Keeping them in one value rather than as two returns is what stops them
/// drifting apart: there is exactly one function that decides what a press
/// means, and it decides both halves in one pass over the same inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PressMeaning {
    /// What a **drag** starting now would mean, or `None` if a drag means
    /// nothing here — either because the mode forbids it, or because the armed
    /// tool simply has no drag gesture.
    ///
    /// The two reasons are deliberately not distinguished. Nothing downstream
    /// wants to tell them apart: in both cases no drag starts, no band is
    /// drawn, and nothing is committed. A reader who needs to know *why* is
    /// asking a question about the tool and the mode, which is what
    /// [`press_kind`] is for.
    pub drag: Option<DragKind>,
    /// Whether a completed **click** is reported at all.
    ///
    /// `false` makes [`super::GestureState::update`] swallow the click and the
    /// double-click, which is what stops a click in Read reaching the
    /// selection. It is asked separately from [`Self::drag`] because a measure
    /// tool needs the click while having no drag, and a mode that cannot select
    /// content must still let that click through.
    pub click: bool,
}

impl PressMeaning {
    /// A press that means nothing at all — no drag, no click.
    ///
    /// What Read grants an un-armed canvas, and the value every test that is
    /// about *refusal* names rather than spelling two fields.
    pub const NOTHING: Self = Self {
        drag: None,
        click: false,
    };

    /// A press that starts `kind` on a drag and reports a click otherwise —
    /// what an ordinary permitted press means.
    ///
    /// A constructor rather than a literal because it is what almost every test
    /// of the state machine wants, and those tests are about press/drag/release
    /// rather than about modes.
    #[must_use]
    pub const fn dragging(kind: DragKind) -> Self {
        Self {
            drag: Some(kind),
            click: true,
        }
    }

    /// A press that reports a click and starts no drag — what an armed measure
    /// tool means.
    #[must_use]
    pub const fn clicking() -> Self {
        Self {
            drag: None,
            click: true,
        }
    }
}

/// Everything a press landed on, resolved by [`crate::canvas::pressing`] while
/// it has the geometry in hand.
///
/// ★★ A struct because the argument list reached eight, and clippy is right
/// that eight positional parameters is a call nobody can read — three of them
/// are now bare `bool`s, and transposing two would compile and produce a
/// gesture aimed at the wrong verb. `dimdrag::Frame`, `annotdrag::Frame` and
/// `dragroute::Frame` all took the same shape for the same reason, so this is
/// the local convention rather than an accommodation.
#[derive(Debug, Clone, Copy)]
pub struct Press {
    /// The tool the operator has armed.
    pub tool: CanvasTool,
    /// The resize/move/rotate grip under the pointer, if any.
    pub grip: Option<Grip>,
    /// The Bézier handle under the pointer, if any.
    pub handle: Option<(usize, pdfcer_core::vector::Handle)>,
    /// What a press on a selected ce dimension landed on.
    pub dimension: Option<DimensionPress>,
    /// **The selected annotation whose rotate handle this press is on**, if it
    /// is on one. See [`RotatableAnnot`].
    pub annot_rotate: Option<RotatableAnnot>,
    /// Whether it landed inside a selected **markup** annotation's box.
    pub markup_body: bool,
    /// Whether it landed on one of that annotation's **resize grips**, which
    /// lie half outside its box.
    ///
    /// ★★ A second flag rather than a wider `markup_body`, and the distinction
    /// is load-bearing: the arm below reads them for **different verbs** — the
    /// body means Move, a grip means Resize — so collapsing them would make a
    /// press on a corner ambiguous exactly where the precedence matters. See
    /// `canvas::pressing` for why a grip is outside the box at all.
    pub markup_grip: bool,
    /// Whether it landed inside the selected **form field's** box.
    pub widget_body: bool,
    /// Whether it landed on one of that field's box's **resize grips**.
    pub widget_grip: bool,
    /// Whether a one-shot region zoom is armed.
    pub zoom_armed: bool,
}

#[must_use]
pub fn press_kind(press: Press, caps: Capabilities) -> PressMeaning {
    let Press {
        tool,
        grip,
        handle,
        dimension,
        annot_rotate,
        markup_body,
        markup_grip,
        widget_body,
        widget_grip,
        zoom_armed,
    } = press;
    // ★ A measure tool takes the click and leaves the drag alone.
    //
    // Highest precedence, above the markup tool, for the same reason the
    // markup tool sits above the grips: the operator armed it, and it is the
    // claimant whose loss would be silent. It cannot actually contend with
    // markup — one tool is armed at a time, by construction — so the ordering
    // is a statement rather than a tie-break.
    //
    // `drag: None` is the honest answer and not a stub. A ce dimension is
    // authored by clicks: point A, point B, then a third click saying how far
    // off the geometry the dimension sits. There is no drag in that gesture, so
    // there is no `DragKind` for one, and inventing a `DragKind::Measure` that
    // every arm then ignored would be a placeholder — which this project's
    // no-placeholders invariant forbids, and which would put a rubber band on
    // screen promising a gesture nothing implements.
    if tool.measure_kind().is_some() {
        return PressMeaning {
            drag: None,
            click: caps.author_measure,
        };
    }
    // ★ …and the **caret** tool does the same third time, which is what makes
    // this rung a family rather than three special cases.
    //
    // A caret is placed, not dragged: one click says *where*, and the keyboard
    // says the rest. There is no drag in that gesture, so `drag: None` is the
    // honest answer and a `DragKind::TextEdit` that every arm ignored would put a
    // rubber band on screen promising a gesture nothing implements — the
    // placeholder this project's no-placeholders invariant forbids, and the
    // argument the two rungs around it make in their own words.
    //
    // The capability is `edit_content`, where the measure rung reads
    // `author_measure` and the vertex rung `author_markup`, and the difference is
    // the whole point: those two author a dimension and a comment, which sit
    // *over* the page. This rewrites the page's own show operators. So a mode
    // that offers Markup and not content editing — which is the row
    // `MODES_AND_PANELS.md`'s gesture table calls Review — places dimensions and
    // comments and still may not put a caret in a word.
    //
    // ★ It sits **above** the text-selection question below rather than beside
    // it, and that ordering is load-bearing in the direction that is easy to get
    // backwards. `textsel::takes_the_press` is false for this tool by
    // construction (it asks `is_text`, which is `matches!(tool, Text)`), so the
    // two cannot contend today — but `caps.edit_content` is *true* in the only
    // mode this tool arms in, so if that predicate ever grew a disjunct for the
    // caret tool the content branch below would answer first and a press would
    // marquee objects under an I-beam. Returning early is what makes that
    // unreachable rather than merely unlikely.
    // ★ …and a text-annotation tool splits BOTH ways, which is why it needs its
    // own rung rather than joining either family above.
    //
    // A text box and a stamp are **dragged** — the operator is choosing how
    // wide the words are — so they want a `DragKind`. A sticky note is
    // **clicked**, because its rect is fixed-size and `NoZoom` and the format
    // discards whatever was dragged; asking for a width would be asking for a
    // number nobody reads.
    //
    // `is_dragged` is the predicate the whole family branches on, so the two
    // shapes cannot drift apart here from the way they are authored — the same
    // welding `uses_gallery` does for the stamp's text.
    // ★★ A FORM tool, and it is placed BOTH ways at once — which is why it
    // needs no `is_dragged` predicate of its own.
    //
    // The operator, 2026-08-26: *"when I click one I should be able to click on
    // the canvas to place the position or drag a box for size"*. Both, for every
    // kind, and the click is not a degenerate drag that happens to be tolerated
    // — it is the primary gesture for a check box, whose conventional size is
    // 14 pt square and which nobody wants to drag out by hand.
    //
    // ★ It sits ABOVE the text-annotation rung rather than below, and the
    // ordering is currently unobservable: the two tools cannot both be armed,
    // because `CanvasTool` is one value. It is written in a fixed order anyway
    // so that the precedence is a decision on the page rather than an accident
    // of which rung someone appended to last.
    //
    // The capability is `edit_content`, NOT `author_markup`: a form field is a
    // change to the document's own content rather than an annotation over it,
    // which is the same reasoning that put these commands in Edit mode. Pairing
    // it with markup would let a reviewer author form fields, and authoring an
    // interactive control is not a review activity.
    // ★★★ **A pending placement takes the press before anything else** —
    // `OPERATOR_REQUESTS.md` O66.
    //
    // FIRST, and like the measure rung above it that is a statement rather
    // than a tie-break: one tool is armed at a time, so it cannot contend.
    // What the position DOES buy is three specific exclusions, and each of
    // them would be a real defect:
    //
    // * above the caret rung, so a placement press cannot reach
    //   `textsel::takes_the_press` and start sweeping text;
    // * above the content/reading split, so Edit mode's content marquee
    //   cannot claim it and rubber-band across the sheet instead;
    // * above the grips, so a placement over an existing selection places
    //   rather than moving what happens to be selected underneath.
    //
    // Both halves live, for the form field's stated reason: a click is the
    // primary gesture here, and answering `click: false` would drop a twitch
    // into whatever is beneath.
    //
    // The capability comes from the KIND rather than being named here, so the
    // mapping exists once — see `PlaceKind::capability`.
    if let CanvasTool::Place(kind) = tool {
        let permitted = kind.capability(caps);
        return PressMeaning {
            drag: permitted.then_some(DragKind::Place(kind)),
            click: permitted,
        };
    }
    if let CanvasTool::Form(kind) = tool {
        return PressMeaning {
            drag: caps.edit_content.then_some(DragKind::Form(kind)),
            click: caps.edit_content,
        };
    }
    if let CanvasTool::TextAnnot(kind) = tool {
        return PressMeaning {
            drag: kind.is_dragged().then_some(DragKind::TextAnnot(kind)),
            // The click is live for the sticky and, deliberately, ALSO for the
            // dragged kinds: a click with the text-box tool armed is a
            // zero-area drag, and answering `false` would make it fall through
            // to the marquee underneath — an operator who meant to place a
            // callout and twitched would select objects instead.
            click: caps.author_markup,
        };
    }
    if tool.text_edit_kind().is_some() {
        return PressMeaning {
            // ★★★ A DRAG WITH THE TEXT TOOL DRAWS A BOX TO TYPE IN — the
            // operator, 2026-08-21: *"I should be able to make it multi line."*
            //
            // It has to be a drag, and the reason is the file format rather
            // than a preference: a PDF has no paragraph, so each visual line is
            // its own show operator at its own absolute position, and something
            // must decide where the second line starts. A width to wrap against
            // is that something, and a width is a rectangle.
            //
            // ★ The CLICK still means what it meant, and both live at once.
            // Click for a single line at a point, drag for a paragraph in a
            // box — the same pair the markup band and the sticky note already
            // form one rung above, and the same pair the old shell had (*"in
            // box mode a plain Enter is a paragraph break … in point mode Enter
            // accepts"*).
            //
            // A zero-travel press is a click by the gesture machine's own rule,
            // so a twitch while aiming at a caret does not silently become an
            // empty box.
            drag: caps.edit_content.then_some(DragKind::TextBox),
            click: caps.edit_content,
        };
    }
    // ★ …and a **vertex** markup tool does exactly the same thing, for exactly
    // the same reason — which is why it is written here, immediately beside it,
    // rather than as a special case inside the markup rung below.
    //
    // PolyLine and Polygon are picked, not dragged: click each corner, then say
    // when. There is no drag in that gesture, so there is no `DragKind` for one,
    // and `drag: None` is the honest answer rather than a stub. Inventing a
    // `DragKind::Markup` that every arm then ignored would put a rubber band on
    // screen promising a gesture nothing implements — the placeholder this
    // project's no-placeholders invariant forbids, and the same argument the
    // measure branch above makes in its own words.
    //
    // The capability is `author_markup` where the measure branch reads
    // `author_measure`: these two kinds author a **comment**, not a dimension,
    // so a mode that offers Markup and not Measure must still place them. That
    // is the row `MODES_AND_PANELS.md`'s gesture table calls Review.
    //
    // It sits ABOVE the markup rung rather than inside it because this is a
    // question about the *shape* of the gesture and that rung is a question
    // about precedence. Folding it in would mean the markup rung returned a
    // `drag` for four kinds and `None` for two while a single `click` field was
    // decided fifteen lines further down for all six — which is exactly the
    // arrangement `PressMeaning`'s own header records as the gate's first,
    // wrong, shape.
    if let Some(kind) = tool.markup_kind()
        && kind.is_vertex()
    {
        return PressMeaning {
            drag: None,
            click: caps.author_markup,
        };
    }
    // ★ Does the press mean TEXT? Asked once, here, and asked again by
    // `canvas::interact` when it routes the click — through the same function,
    // so the drag's meaning and the click's routing cannot drift apart. See
    // this function's ★ section on the text row.
    let text = crate::canvas::textsel::takes_the_press(tool, caps);
    // ★ The two worlds, split on the one flag that separates them, rather than
    // one `match` with a capability test on every arm.
    //
    // It used to be the latter, and the text row is what showed why that was the
    // wrong shape: with `edit_content` false, **three of the four arms were
    // dead** — a grip is drawn only for a content selection, so in a mode that
    // cannot make one there is no grip to hover, no resize and no move. Writing
    // them as `caps.edit_content.then_some(…)` inside one match left those dead
    // arms answering `None` and swallowing the press, so a hypothetical grip in
    // Read produced *no meaning at all* where every other press in Read means
    // text. Unreachable, and incoherent — and the incoherence is the kind that
    // becomes reachable the day something else changes.
    //
    // Split, each branch says one thing. The content branch is byte-for-byte the
    // precedence that shipped: markup, grip, armed zoom, marquee. The reading
    // branch is the armed zoom and then text, and it has no grip arm because
    // there are no grips.
    let drag = if let Some(kind) = tool.markup_kind() {
        caps.author_markup.then_some(DragKind::Markup(kind))
    // ★ **The armed TEXT tool, and it sits above the content branch on purpose.**
    //
    // This is the one rung the text-tool work added, and its placement is the
    // whole of what it does. Below `caps.edit_content` it would be dead in Edit —
    // the mode the tool exists for — because that branch is total: every value of
    // `grip` and `zoom_armed` produces a meaning there, so nothing after it is
    // reachable while the mode can select content. A tool that armed, painted an
    // I-beam and marqueed objects is the *"visible control, silently inert"*
    // failure with an extra insult.
    //
    // Above it, the rule reads the same as every other armed tool's: **the press
    // belongs to whichever tool is armed.** That is not a rule invented here —
    // `Markup` above has relied on it since it landed, for the reason its own
    // rung states — and it is what replaces the by-construction exclusivity the
    // old single-disjunct rule had. `canvas::textsel` §3 records that move from
    // construction to precedence, including the consequence that an object
    // selection and a text selection can now both be non-empty in Edit.
    //
    // ★ The region zoom is the one thing it yields to, and the ordering is
    // **borrowed rather than decided**: the reading-mode text row four branches
    // below already yields to `zoom_armed`, on the argument that the zoom is a
    // one-shot the operator armed deliberately from the ribbon and that a text
    // sweep is back on the very next press. Nothing about that argument mentions
    // *why* the press means text, so applying it to the armed tool as well keeps
    // one rule where two would otherwise appear — and the alternative would be an
    // operator whose armed zoom silently stops working for as long as the text
    // tool is down, with the zoom control still rendering pressed on another tab
    // where they cannot see it. That is exactly the "spending an Escape on
    // something inert" hazard `canvas::keys`' header argues about, in the
    // pointer's tense.
    //
    // Note this differs from `Markup` above, which outranks the zoom. The
    // distinction is the one that rung already draws: markup **authors**, so the
    // loss of its drag would be a mark that was never made, while a text sweep
    // loses nothing an operator cannot re-make with one more drag.
    } else if tool.is_text() {
        Some(if zoom_armed {
            DragKind::Marquee(MarqueeIntent::Zoom)
        } else {
            DragKind::TextSelect
        })
    // ★★★ **THE ROTATE HANDLE OF A SELECTED ANNOTATION, AND IT IS THE HIGHEST
    // OF THE THREE ANNOTATION RUNGS.** 2026-08-28, `Pass 155.0` + `Pass 159.0`.
    //
    // ## Why it is a rung of its own rather than an arm of the two below
    //
    // Because **neither of the two below can see it**, and that is a geometric
    // fact rather than an oversight:
    //
    // * the dimension rung reads `dimension`, which `pressing` resolves from
    //   `dimdrag::vertex_at` and from the dimension's own `/Rect`. The rotate
    //   handle sits `ROTATE_STEM_PX` ABOVE that rect, so both answer `None`;
    // * the markup rung reads `markup_body`, which is `grab_box().contains(p)`
    //   over that same `/Rect`. The handle is outside it, so it is `false`.
    //
    // ⇒ A press on the handle therefore fell all the way through to
    // `caps.edit_content` — which is **false in Review**, the mode markup and
    // measurements are authored in. The handle would have been painted, been
    // grabbable, and produced nothing in the one mode that draws the things it
    // turns.
    //
    // ## ★★★ And in Edit it would have been WORSE than nothing
    //
    // In Edit `caps.edit_content` is true, so the press would have reached the
    // content branch's `(None, Some(Grip::Rotate)) => DragKind::Rotate` arm —
    // and `canvas::rotating` would then have rotated **the page content
    // selection**, which is empty beside an annotation selection… or, on the
    // build where it was not, some other object entirely.
    //
    // That is **the hazard this canvas has produced four times**: a working
    // gesture aimed at the wrong verb, which never looks broken from a chair
    // because *something moves*. The most recent instance is recorded in
    // `canvas::presspick`: `covers()` tested the selection's move box alone,
    // the rotate handle sits outside that box, so a press on it selected the
    // object underneath and the rotate became a select-and-move.
    //
    // ## Placement, stated rather than relied upon
    //
    // Above the dimension rung. It could not matter today — the handle is
    // outside every box those rungs test, so no press can satisfy two of them —
    // and it is stated anyway, because the day `dimdrag::grab_box` grows to
    // include the stem (a plausible change: it would make the whole affordance
    // one rectangle) a silent reordering would turn every rotate into a body
    // move. An ordering that is a *statement* survives that; one that is a
    // coincidence does not.
    //
    // ## The two capability gates, and why they are not one
    //
    // `author_markup` for a markup, `author_measure` for a ce dimension — the
    // same split the two rungs below already make, adopted rather than
    // re-derived. See [`RotatableAnnot`] for the table. A single gate would
    // have to be the union or the intersection, and both are wrong: the union
    // offers a dimension rotation in a mode that may not author measurements,
    // and the intersection withholds a markup rotation in Review.
    } else if grip == Some(Grip::Rotate)
        && annot_rotate.is_some_and(|kind| match kind {
            RotatableAnnot::Markup => caps.author_markup,
            RotatableAnnot::CeDimension => caps.author_measure,
        })
    {
        Some(DragKind::Rotate)
    // ★★★ **A SELECTED CE DIMENSION GETS ITS OWN RUNG, ABOVE `edit_content`.**
    //
    // 2026-08-20, and the placement is the whole of what this rung does.
    //
    // Everything below is gated on `caps.edit_content`, which **Review does not
    // have** — and Review is the mode a ce dimension is *made* in. So a
    // dimension authored in Review could not be dragged, reshaped or placed in
    // the mode that authored it, while its vertex handles were drawn the whole
    // time (the painter asks about the selection, not about a capability).
    // Handles you can see and cannot grab is the "visible control, silently
    // inert" failure, and it was one press away from shipping.
    //
    // The gate is `author_measure`, not `edit_content`, and that is the correct
    // capability rather than a convenient one: **reshaping a ce dimension is a
    // measure edit.** It writes the sidecar and one annotation; it touches no
    // page content. A mode that may author a dimension may adjust the one it
    // just authored — the alternative asks the operator to change mode to fix a
    // corner they misplaced two seconds ago.
    //
    // Above `edit_content` rather than inside it, because in **Edit** both
    // capabilities are present and this must still win: the operator has
    // selected a dimension and pressed on its corner, and there is no reading
    // of that press under which they meant to marquee the page behind it.
    } else if caps.author_measure
        && let Some(press) = dimension
    {
        Some(match press {
            // The corner. Reshapes, and re-measures — the one gesture in this
            // family that changes the number.
            DimensionPress::Vertex(index) => DragKind::DimensionVertex { index },
            // The body. Moves where the dimension is DRAWN and cannot alter
            // what it says; `canvas::dimdrag`'s header carries that argument.
            DimensionPress::Body => DragKind::Move,
        })
    // ★★★ A press inside a selected MARKUP annotation, in a mode that may author
    // markup. Added 2026-08-28, and it is the branch whose absence made the
    // whole annotation drag a dead end.
    //
    // Below the dimension branch and above `edit_content`, and both placements
    // are decisions:
    //
    // * **Below the dimension**, because the two are mutually exclusive by
    //   `AnnotKind` and the ordering is therefore a statement rather than a
    //   tie-break -- the same relationship the measure tool has to the markup
    //   tool at the top of this function.
    //
    // * **Above `edit_content`**, because it must fire in REVIEW, where
    //   `edit_content` is false. That is the whole reason it could not simply
    //   be another arm in the grip match below: markup is authored in Review,
    //   and an operator who has just drawn a shape there and wants to nudge it
    //   is in the mode where the content branch does not run at all.
    //
    // => The absence of this branch is what made the fork in `canvas::interact`
    // a dead end. `annotdrag` was reachable and never reached, because no press
    // on a markup ever became a `DragKind::Move` to route.
    // ★★★ A press inside a selected MARKUP annotation or a selected FORM
    // FIELD's box. Added 2026-08-28, ten days apart, and merged here because
    // they produce the same verb.
    //
    // Below the dimension branch and above `edit_content`, and both placements
    // are decisions:
    //
    // * **Below the dimension**, because the kinds are mutually exclusive by
    //   `AnnotKind` and the ordering is a statement rather than a tie-break.
    //
    // * **Above `edit_content`**, because the markup half must fire in REVIEW,
    //   where `edit_content` is false. That is why it could not be another arm
    //   in the grip match below: markup is authored in Review, and an operator
    //   who has just drawn a shape there and wants to nudge it is in the mode
    //   where the content branch does not run at all.
    //
    // ★★ TWO capability gates, one verb, and the asymmetry is the mode
    // selector's ruling rather than this function's. Markup is authored in
    // Review; a form field is only SELECTABLE in Edit, because `canvas::forms`
    // gives the selection surface to Edit and the fill surface to Read and
    // Review — *"the same click cannot both type a value and select the box to
    // rename it."* So a widget drag is only reachable in Edit, and gating it
    // any other way would be a second answer to a question that has one.
    //
    // ⇒ The absence of these two branches is what made the fork in
    // `canvas::dragroute` a dead end for its whole life: the modules were
    // reachable and never reached, because no press on a markup or a widget
    // ever became a `DragKind::Move` to route.
    // ★★★ **…AND ON ONE OF THEIR GRIPS, WHICH IS NOT THE SAME AS INSIDE THEIR
    // BOX** — 2026-09-05.
    //
    // `markup_body` is `grab_box().contains(p)`. A corner grip is *centred on*
    // a corner of that box, so half of its live area is outside it, and
    // `handles::grip_bounds` pushes the anchors further out again on a small
    // selection. A press on that outer half therefore had `grip == Some(NE)`
    // and `markup_body == false`, fell past this arm into `caps.edit_content`
    // — **false in Review** — and vanished. No resize, no decline, nothing in
    // the trace: `the_line_weight_switch_reaches_the_resize` FAILED on it for
    // the whole life of the feature, and `dragging_a_markup_moves_it` was
    // failing beside it for an unrelated reason, which is what made the pair
    // read as *"the annotation branch eats every gesture"*.
    //
    // ★★ The rotate handle is the control that shows why this was missed: it
    // sits obviously clear of the box and so was obviously given its own arm
    // above. The eight scale grips look as though they are on the edge, so a
    // body test looks sufficient — and it is, for every press an operator makes
    // one pixel too far in. `canvas::pressing` computes the two grip flags and
    // its comment carries the measurement.
    } else if (caps.author_markup && (markup_body || markup_grip))
        || (caps.edit_content && (widget_body || widget_grip))
    {
        // ★★★ A resize grip OUTRANKS the body, and the arm has to say so
        // itself rather than inherit it.
        //
        // The content branch below states this precedence in its own `match`,
        // and this branch sits ABOVE that branch — so a press on one of an
        // annotation's eight grips would otherwise be claimed here as a MOVE
        // before the grip match ever ran. The operator would grab a corner,
        // drag, and translate the shape instead of scaling it: a working
        // gesture aimed at the wrong verb, which is the failure mode
        // `Grip::Rotate` was given its own arm below to prevent.
        //
        // ★★ `is_resize()` rather than "not Move", enumerated for that same
        // reason. `Grip::Rotate` is not a resize and must not fall in here.
        //
        // ⚠ **The reason given here was stale and is corrected 2026-09-05.** It
        // said *"annotations are offered no rotate handle (`GripSet::scale_only`),
        // so it cannot arrive"* — false since `rotate_annotation` shipped on
        // Pass 155.0: `pressing::grabbable` hands a markup `GripSet::all()`,
        // and a rotate handle is exactly what it draws. What actually keeps
        // `Rotate` out of this arm is the **rotate arm above**, which claims
        // every `grip == Some(Grip::Rotate)` its capability allows, plus the
        // positive `is_resize()` test here — which is why matching on the
        // property rather than on "not Move" was right for a reason better than
        // the one written down.
        //
        // ★ `markup_grip` and `widget_grip` are `is_resize()`-gated at their
        // source (`canvas::pressing`), so a rotate press cannot enter this arm
        // through the new route either.
        match grip {
            Some(g) if g.is_resize() => Some(DragKind::Resize(g)),
            _ => Some(DragKind::Move),
        }
    } else if caps.edit_content {
        match (handle, grip) {
            // ★★ A Bézier handle outranks everything below it, and it has to.
            //
            // A handle sits INSIDE the selection's box, so `grip_at` answers
            // `Grip::Move` for every press on one — which would make handles
            // undraggable, exactly as the corner ANCHORS were undraggable until
            // the eight scale grips were confined to the Object rung.
            //
            // Both are the same rule: **the most specific thing under the
            // pointer wins**, and specificity is depth down the selection
            // ladder. A handle belongs to a selected anchor, which is one rung
            // deeper than anything a grip describes, so it wins.
            (Some((node, handle)), _) => Some(DragKind::Handle { node, handle }),
            (None, Some(grip)) if grip.is_resize() => Some(DragKind::Resize(grip)),
            // ★ Matched by NAME rather than falling into the `Move` arm below,
            // which is where it would have gone silently: `Grip::Move` and
            // `Grip::Rotate` are the two non-resize grips, so a wildcard that
            // meant "the body" now covers both. A press on the handle would
            // have MOVED the selection — a working gesture, aimed at the wrong
            // verb, with nothing to report.
            (None, Some(Grip::Rotate)) => Some(DragKind::Rotate),
            (None, Some(_)) => Some(DragKind::Move),
            (None, None) if zoom_armed => Some(DragKind::Marquee(MarqueeIntent::Zoom)),
            (None, None) => Some(DragKind::Marquee(MarqueeIntent::Select)),
        }
    // ★ An armed region zoom outranks a text sweep, and that ordering is the
    // operator's own arming decision rather than a preference: the zoom is a
    // one-shot they armed *deliberately* from the ribbon, and a reading mode is
    // exactly where they are most likely to have armed it. A text sweep is the
    // un-armed default and is back on the very next press.
    } else if zoom_armed {
        Some(DragKind::Marquee(MarqueeIntent::Zoom))
    } else if text {
        Some(DragKind::TextSelect)
    } else {
        // A reading mode with something armed that is neither — reachable only
        // through a manifest that grants markup or measure without the tab this
        // function reads. Nothing to offer, and saying so is better than
        // guessing.
        None
    };
    // Every remaining tool's click means *select what is under the pointer*,
    // which is the content capability. Including the markup tool's: a click
    // with a pen armed places nothing (a degenerate drag is refused by
    // `markup::drag`), so the click falls through to the selection exactly as
    // it did before this gate existed. That is behaviour carried across
    // deliberately rather than decided here — see `canvas::markup`.
    //
    // ★ …and the text press reports a click too, because a text gesture's
    // click carries three of its four meanings: double-click takes a word,
    // triple-click takes a line, Shift+click extends, and a plain click clears.
    // Suppressing it would leave a drag that selects and no way to unselect —
    // and it would leave the two most familiar text gestures in the product
    // class unreachable. Where that click is *routed* is `canvas::interact`'s
    // business; it asks `textsel::takes_the_press` again rather than inferring
    // it from this flag, because this flag is true for two different reasons.
    PressMeaning {
        drag,
        click: caps.edit_content || text,
    }
}

#[cfg(test)]
mod tests;
