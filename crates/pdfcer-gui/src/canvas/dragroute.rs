//! # `canvas::dragroute` — which of THREE move verbs one drag reaches
//!
//! Split out of [`super::interact`] under **R2** on 2026-08-28, when the
//! annotation drag took that file past 1,500 lines for the second time.
//!
//! ## ★★★ The seam is a subject, and it is the one this project got wrong
//!
//! One gesture — press inside the thing, drag it — reaches three different
//! engine verbs, and **which one is decided entirely by what is selected**:
//!
//! | selection | verb | why it is not the others |
//! |---|---|---|
//! | page content | `move_objects` / `move_nodes` | paint-order indices into a content stream |
//! | a **ce dimension** | `place_dimension` | moves where it is DRAWN and cannot alter the number it prints |
//! | ordinary **markup** | `move_annotation` | a stable `ObjId`, and two halves of geometry to write |
//!
//! ★★★ **A fork whose branches can all answer "not mine" eats the gesture**,
//! and that is exactly what this one did for ten days. The annotation branch
//! held only `dimdrag`, which answers `None` for anything that is not a ce
//! dimension; the content branch was in the `else`, unreachable behind an
//! annotation selection by construction. An operator pressed inside a stamp,
//! dragged it across the sheet, released, and nothing happened and nothing
//! declined.
//!
//! ⇒ The failure is worse than a missing feature, because a missing feature
//! usually refuses. Gathering the three into one function is what makes the
//! exhaustiveness visible: they are now adjacent, and a fourth kind arriving
//! has one place to be added rather than a fork to be noticed.
//!
//! ## ★★ Ordered, not exclusive-by-guard
//!
//! `dimdrag` is asked first because it is the **narrower** claim — it answers
//! only for `AnnotKind::CeDimension` — and `annotdrag` re-checks the kind for
//! itself rather than reading *"dimdrag said no"*. A module that decided what
//! it handles from another module's refusal would be one rename away from
//! claiming everything.
//!
//! ## ★ Shift is applied ONCE, above the fork
//!
//! `ui-conventions/drag-moves.md` D5. All three verbs receive the same
//! constrained delta from one filter, because two copies of *"what does Shift
//! mean"* is how two drags in one program come to disagree about it.

use egui::Vec2;

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::Phase;
use crate::canvas::selection::SelectionState;
use crate::canvas::{annotdrag, dimdrag, moving};

/// The previews one move frame produced — at most one is `Some`.
///
/// ★ Three fields rather than an `enum`, matching the seven preview slots
/// `interact` already carries and for their stated reason: the painter reads
/// each one independently, and folding them together would put a branch in the
/// paint loop for a value that is `None` on every frame nobody is dragging.
#[derive(Default)]
pub struct Previews {
    /// A content move's canvas-space displacement.
    pub ghost: Option<Vec2>,
    /// Where a dragged markup would land, in canvas space.
    pub annot: Option<egui::Rect>,
    /// Where a dragged form-field box would land, in canvas space.
    ///
    /// ★ A fourth field rather than sharing [`Self::annot`], even though the
    /// two are the same shape and can never both be `Some`. The painter reads
    /// each independently, and one rectangle whose meaning depends on which
    /// selection is live is a value the paint loop has to interrogate.
    pub widget: Option<egui::Rect>,
    /// ★★★ **The selection's own geometry at its new position**, in page space
    /// (`OPERATOR_REQUESTS.md` O63).
    ///
    /// `None` on every rung `canvas::shapes` cannot draw honestly — a text run,
    /// an image, a form XObject, a page that will not decompose, a selection
    /// past the cap — in which case [`Self::ghost`]'s bounding outline is the
    /// whole answer, exactly as it was before this field existed.
    pub shape: Option<crate::canvas::shapes::ShapePreview>,
    /// ★★★ The geometry to **hold** on screen after the gesture ends, until the
    /// page raster catches up (`OPERATOR_REQUESTS.md` O63).
    ///
    /// `Some` on exactly one frame per gesture: the one that raised the Action.
    /// [`Self::shape`] is `None` on that frame, so the two never both speak.
    pub hold: Option<crate::canvas::shapes::ShapePreview>,
    /// A ce dimension redrawn at its new placement, in page space.
    pub dimension: Option<Vec<(pdfcer_core::vector::Point, pdfcer_core::vector::Point)>>,
}

/// Everything one move frame needs that is not the delta.
///
/// ★ A struct because the argument list reached nine and clippy is right that
/// nine positional parameters is a call nobody can read — five of the six here
/// are borrows of similar-looking things, and transposing two would compile.
/// `dimdrag::Frame` and `annotdrag::Frame` take the same shape for the same
/// reason, so this is the local convention rather than an accommodation.
pub struct Frame<'a> {
    /// The egui context, for the Shift filter.
    pub ctx: &'a egui::Context,
    /// The open document — consulted only by the ce-dimension branch, which
    /// has to scan the dimension model to answer at all.
    pub doc: &'a OpenDoc,
    /// What is selected. **This is what decides the verb.**
    pub selection: &'a SelectionState,
    /// The page on screen.
    pub page_index: usize,
    /// The decomposition, for the content branch. `None` at a rung that does
    /// not need one.
    pub provider: Option<&'a crate::panels::objects::provider::ObjectModelProvider>,
    /// Whether Shift is held **this frame**.
    pub shift: bool,
}

/// Route one frame of a move drag to the verb the selection names.
///
/// See the module header for the ordering and for what the absence of the
/// third branch cost.
pub fn moved(frame: &Frame<'_>, delta: Vec2, phase: Phase, actions: &mut Vec<Action>) -> Previews {
    let &Frame {
        ctx,
        doc,
        selection,
        page_index,
        provider,
        shift,
    } = frame;
    let mut out = Previews::default();
    // ★★ SHIFT LOCKS THE MOVE TO ONE AXIS — once, above the fork
    // below, so both verbs get the same constrained delta from one
    // filter. `ui-conventions/drag-moves.md` D5.
    //
    // ★ `shift` is THIS FRAME's modifier, not the press-time flag the
    // gesture machine carries. See `resizing::Frame::constrain` for why
    // those are two different facts that happen to read one key.
    let delta = crate::canvas::constrain::translate(ctx, shift, delta);
    // ★★ Two different verbs share one gesture, and the selection
    // decides which.
    //
    // A content move reaches `move_objects` / `move_nodes`; a ce
    // dimension reaches `place_dimension`, which changes only where the
    // dimension is DRAWN and cannot alter the number it prints. They
    // are the same gesture to the operator - press inside the thing,
    // drag it - and that is why they share `DragKind::Move` rather than
    // getting a mode or a modifier. See `canvas::dimdrag`'s header for
    // why placement, and not translation, is what dragging a dimension
    // means.
    //
    // Mutually exclusive by construction: the two annotation modules
    // answer only for an annotation selection and `moving::eligible`
    // only for content, so the `else` is a statement of that rather
    // than a precedence.
    //
    // ★★★ THREE verbs share one gesture as of 2026-08-28, not two, and
    // the third is why the annotation branch stopped being a dead end.
    //
    // `dimdrag` answers for a ce dimension and `None` for everything
    // else -- so before `annotdrag` existed, an annotation selection
    // took this branch, got `None`, and the content branch below was
    // unreachable BY CONSTRUCTION. An operator pressed inside a stamp,
    // dragged it, let go, and nothing happened anywhere. The gesture
    // was consumed rather than declined, which is the worst of the
    // three possible outcomes: it reads as a broken program.
    //
    // => A fork whose branches can BOTH answer "not mine" needs a
    // third arm or an explicit decline. This one had neither for ten
    // days, and no test could see it -- every assertion about
    // annotations asked whether one could be selected, restyled or
    // deleted, and all three were true.
    if selection.annot().is_some() {
        out.dimension = dimdrag::drag(
            dimdrag::Frame {
                delta,
                phase,
                page: doc.current_page(),
            },
            doc,
            selection,
            actions,
        );
        // ★★ Ordered, not exclusive-by-guard, and the order is the
        // safe one: `dimdrag` is the NARROWER claim -- it answers only
        // for `AnnotKind::CeDimension` -- so asking it first and
        // falling through means a kind neither module claims moves
        // nothing rather than being moved by the wrong verb.
        //
        // `annotdrag::eligible` re-checks the kind rather than reading
        // "dimdrag said no", because a module that decided what it
        // handles from another module's refusal would be one rename
        // away from claiming everything.
        if out.dimension.is_none() {
            out.annot = annotdrag::drag(
                &annotdrag::Frame { delta, phase },
                doc.current_page(),
                selection,
                actions,
            );
        }
    // ★★★ A FORM FIELD's box, and it needs its own top-level arm because a
    // widget is not an annotation selection.
    //
    // `canvas::selection::annot` excludes `/Widget` outright — *"the form field
    // surface owns it; a click there focuses an editor, and two owners of one
    // press is how a field becomes unfillable"* — so a selected widget lives on
    // `doc.selected_field` and the branch above never sees it. Before this arm
    // existed the press fell into the CONTENT branch below, where
    // `moving::eligible` found nothing and the gesture was eaten, which is the
    // same defect the annotation branch had for ten days and is why it was
    // looked for here.
    } else if doc.selected_field.is_some() {
        out.widget = crate::canvas::widgetdrag::drag(
            &crate::canvas::widgetdrag::Frame { delta, phase },
            ctx,
            doc,
            actions,
        );
    } else {
        let preview = moving::drag(
            delta,
            phase,
            selection,
            page_index,
            provider,
            doc.current_page(),
            actions,
        );
        out.ghost = preview.ghost;
        // ★★★ O63: the selection's own geometry, moving with the pointer.
        //
        // Carried through unchanged. `moving::drag` decides whether there is one
        // — it is the only place that knows what the release will commit — and
        // this is a wire, not a decision.
        out.shape = preview.shape;
        // ★★★ O63's third piece: the geometry to keep drawing after the gesture
        // ends, until the page raster carries the edit. `Some` on exactly one
        // frame per gesture — see `MovePreview::hold`.
        out.hold = preview.hold;
    }
    out
}
