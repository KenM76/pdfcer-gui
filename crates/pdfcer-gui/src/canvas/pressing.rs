//! # `canvas::pressing` — **what a press would land on, and what it would mean**
//!
//! ## Why this is its own file
//!
//! R2, on 2026-08-19, when the Node tool and the Bézier-handle hit test pushed
//! `canvas::interact` past 1,500 lines. It is a real seam rather than a
//! convenient cut: everything here answers one question — *if the primary
//! button went down at this point, right now, what would happen?* — and nothing
//! here changes anything. `interact`'s remaining sections advance a gesture,
//! route a click and paint; this one only looks.
//!
//! ## ★★ The precedence, in one place, because it was learned three times
//!
//! Four different things can be under the pointer at once, and the order they
//! are asked in is the whole behaviour:
//!
//! | # | claimant | why it outranks the next |
//! |---|---|---|
//! | 1 | a **Bézier handle** of a selected anchor | it sits *inside* the selection box, so anything asked before it would swallow every press on one |
//! | 2 | an **anchor**, reached through the inflated move box | an anchor sitting on the bounding box's edge is half outside it |
//! | 3 | a **resize grip** — Object rung only | it scales the whole object, which is the wrong subject at an inner rung |
//! | 4 | the **selection body** → move | the least specific claim, so it answers last |
//!
//! **The most specific thing under the pointer wins, and specificity is depth
//! down the selection ladder.** That sentence is the rule, and each clause of
//! it was paid for separately on 2026-08-19:
//!
//! 1. the corner resize grips covered the corner **anchors**, so the end points
//!    of every path were undraggable while the middle ones worked — fixed by
//!    confining the eight grips to the Object rung;
//! 2. the move box still did not *reach* an anchor sitting on its own edge,
//!    which is the same defect from the other side and needed an **inflation**
//!    rather than another suppression;
//! 3. `grip_at` answered `Move` for every press on a **handle**, so every
//!    attempt to shape a curve moved the whole object — and that one is entirely
//!    plausible from a chair, because the object *did* move.
//!
//! ## ★ Everything here reads `press_origin`, not the current pointer
//!
//! `egui` does not call an interaction a drag until the pointer has travelled a
//! threshold, so by the frame it says so the pointer is **already that far from
//! where it went down** — measured at 94 PDF points of error on an A1 sheet at
//! 0.21× zoom. A grip is an 8 pt square and a handle mark is 7 pt across.
//! Reading the current position misses both, and the miss is silent: the
//! gesture becomes a marquee, which *clears the selection the operator was
//! trying to resize*.

use egui::Pos2;

use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::gesture::{self, PressMeaning};
use crate::canvas::handles::Grip;
use crate::canvas::mapping::PageMapping;
use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::canvas::tool::CanvasTool;
use crate::canvas::{annotdrag, dimdrag, handledrag, handles, overlay, widgetdrag, zoom};

/// What the pointer may grab, and which grips that thing offers.
///
/// ★★★ ONE value, read by the hit test, the painter and the drag — which is
/// rule **H7**, and the reason this is a struct rather than two functions.
///
/// That row exists because it failed on 2026-08-20: a dimension's vertex
/// handles were painted from the selection and hit-tested behind a capability
/// the mode did not have, so they were **visible and untouchable in the very
/// mode that authors dimensions**. A handle painted and not hit-tested is the
/// "visible control, silently inert" failure; one hit-tested and not painted is
/// worse — an invisible target that steals the press aimed at what is under it.
#[derive(Debug, Clone, Copy)]
pub struct Grabbable {
    /// The box, in screen space, or `None` when nothing is grabbable.
    pub bounds: Option<egui::Rect>,
    /// Which grips it offers, because it has a verb behind each.
    pub offer: handles::GripSet,
    /// ★★★ **Whether the box is a BOUNDING box or the subject itself** —
    /// `OPERATOR_REQUESTS.md` O72.
    ///
    /// `true` only for the last row of the table below: page **content**,
    /// whose box is `selection.outline_union()` and is therefore mostly empty
    /// space. `false` for a ce dimension, a markup annotation and a form
    /// field's widget, every one of which is a `/Rect` — for those three the
    /// rectangle **is** the object, and a press anywhere inside it genuinely
    /// lands on the thing.
    ///
    /// # What it is for
    ///
    /// The operator: *"Click and hold shouldn't select an object - it should
    /// allow me to draw a box around objects to select."*
    ///
    /// A marquee starts when a press finds no grip. `handles::grip_at` ends
    /// `bounds.contains(pointer).then_some(Grip::Move)`, so while anything is
    /// selected, a press **anywhere inside its union bounding box** became a
    /// move. Select a title-block border once — a hollow rectangle spanning
    /// the sheet — and the marquee became unreachable on the entire drawing,
    /// because every press was inside the box and none of them was on the
    /// object.
    ///
    /// ⇒ Where this is `true`, [`look`] confirms the press actually lands on
    /// a selected object before honouring `Grip::Move`. Where it is `false`
    /// nothing changes, which is why the flag exists rather than the check
    /// running unconditionally.
    pub content: bool,
    /// ★★★ **Whether this box DESCRIBES the subject, and may therefore be
    /// stroked** — `OPERATOR_REQUESTS.md` O69.
    ///
    /// The operator: *"If we are at a point where we are showing the nodes in
    /// an editable state there shouldn't be a bounding box around the
    /// objects."*
    ///
    /// `true` for the three `/Rect` kinds — a ce dimension, a markup, a form
    /// field's widget — because their box **is** the object. `true` for page
    /// content at the **Object** rung, where the box is the thing selected.
    /// `false` at the **Part** and **Node** rungs, where the subject is a
    /// subpath or a set of anchors and the box is a bound around something the
    /// operator is working *inside*.
    ///
    /// # ★★ Three things it is deliberately NOT
    ///
    /// - **Not [`Self::content`].** That is `true` at every content rung
    ///   including Object, and answers a different question — *is this box
    ///   mostly empty paper?*, which decides whether a press on it is a move
    ///   (O72). No single boolean expresses both.
    /// - **Not a [`SelectionLevel`].** Three of the four rows of
    ///   [`grabbable`]'s table have no rung at all — an annotation is not on
    ///   the content ladder — so a rung field would force this function to
    ///   invent one for a markup.
    /// - **Not `offer.resize` in disguise.** A ce dimension offers no resize
    ///   (`rotate_only`) and a locked annotation offers nothing, and both keep
    ///   their outline. *"Draw the box iff you can scale it"* is not the rule,
    ///   and inferring it would break silently the day an inner rung gains a
    ///   grip.
    ///
    /// ★ It exists here rather than being re-derived in the painter because
    /// [`grabbable`] already computes `at_object_rung`, uses it twice, and
    /// threw it away — and a second spelling of one predicate is the failure
    /// this module's own header condemns.
    pub outline: bool,
}

/// What the pointer may grab, given what is selected.
///
/// ★★ The chain is ordered by **narrowness** and every link answers `None`
/// unless its own kind is selected, so it is exhaustive rather than a precedence
/// anybody has to remember:
///
/// | selected | box | grips |
/// |---|---|---|
/// | a **ce dimension** | its `/Rect` | the **rotate handle alone** — `rotate_dimension` turns one and no verb scales one, and none ever will |
/// | a **markup** annotation | its `/Rect` | everything — `resize_annotation` scales it and `rotate_annotation` turns it |
/// | a **form field's** box | the widget's rect | the eight scale grips, through `edit_widget(… with_rect)`; a widget's rotation is `/MK /R` and is not built |
/// | page **content** | the selection's union | everything, and only at the Object rung |
///
/// ## ★★★ The last two rows were ONE row until 2026-08-28, and splitting them
/// was the whole of the wiring
///
/// A markup annotation and a form field's box shared a branch — `grab_box(…)
/// .or_else(widgetdrag::grab_box)` — because they offered the identical grip
/// set. They no longer do: `rotate_annotation` shipped on `Pass 155.0` and
/// nothing rotates a widget, so an `or_else` that answered `scale_only()` for
/// both would have left a markup with no rotate handle for the same reason a
/// widget has none — **which is not the same reason at all**, and the code
/// would have said it was.
///
/// ⇒ The dimension row moved the other way in the same change: from *no grips*
/// to *the ninth only*. Two rows, two directions, one day. See
/// [`handles::GripSet`]'s own header for the table of verbs behind them.
#[must_use]
pub fn grabbable(
    ctx: &egui::Context,
    doc: &OpenDoc,
    map: &PageMapping,
    selection: &SelectionState,
) -> Grabbable {
    if let Some(bounds) = dimdrag::grab_box(doc, map, selection) {
        // ★★ The rotate handle and NOT the eight. A ce dimension's extent is
        // its measurement, so `pdfcer-core` declines a scale by name and says it
        // will keep declining it; a rotation is an isometry, so the measured
        // value is identical either side of it by construction and turning one
        // is a legitimate drafting operation.
        //
        // ★ `dimdrag::grab_box` is the gate, unchanged, and it answers `Some`
        // only for a kind whose *placement* drag can finish — Linear and
        // Perimeter. An **angular** or **circular** dimension therefore gets no
        // box and so no rotate handle either, although `rotate_dimension` would
        // accept one. That is this shell's gap rather than the engine's, and it
        // is inherited deliberately rather than patched around: widening the
        // box here would offer a MOVE gesture on a kind whose move cannot
        // commit, which is the "visible control, silently inert" failure
        // swapped for a different one. Filed as owed, not hidden.
        return Grabbable {
            bounds: Some(bounds),
            offer: handles::GripSet::rotate_only(),
            // A ce dimension's box IS its `/Rect`, not a bound around
            // scattered geometry. See `Grabbable::content`.
            content: false,
            // …and for the same reason it is stroked: the box is the object.
            outline: true,
        };
    }
    if let Some(bounds) = annotdrag::grab_box(map, selection) {
        // ★★★ Everything, as of 2026-08-28. `resize_annotation` scales a markup
        // and `rotate_annotation` turns one, and the second is BETTER behaved
        // than the first rather than worse: a rotation composes into the
        // `/Matrix` a producer already wrote (§12.5.5 step (a)), so it works on
        // a stamp Acrobat made, where a resize has to refuse artwork pdfcer did
        // not draw. There is no distortion question and no confirmation step.
        //
        // ★ `annotdrag::grab_box` answers `None` for a **locked** annotation
        // (§12.5.3 bit 8) and for a ce dimension, so neither reaches this arm —
        // a locked markup is offered no handles at all rather than nine that
        // the file forbids.
        return Grabbable {
            bounds: Some(bounds),
            offer: handles::GripSet::all(),
            // A markup annotation's box IS its `/Rect`.
            content: false,
            outline: true,
        };
    }
    if let Some(bounds) = widgetdrag::grab_box(ctx, doc, map) {
        // ★★ The eight and NOT the ninth, and the asymmetry is §12.5.6.19's.
        // A widget's rotation is `/MK /R` — a quantised 0/90/180/270
        // *declaration* the field's appearance generator reads, not a
        // free-angle transform — and it is not built. `rotate_annotation`
        // refuses a widget by name and points at a verb that does not exist
        // yet.
        //
        // ⇒ **R9**: nothing is painted and nothing is hit-tested. A ninth
        // handle here would be a control that declines on release, which is the
        // defect this project exists to remove wearing the costume of a fix.
        return Grabbable {
            bounds: Some(bounds),
            offer: handles::GripSet::scale_only(),
            // A widget's box IS its `/Rect`.
            content: false,
            outline: true,
        };
    }
    let at_object_rung = selection.level() == SelectionLevel::Object;
    let bounds = overlay::grip_box(map, selection).map(|b| {
        if at_object_rung {
            b
        } else {
            b.expand(overlay::ANCHOR_PX)
        }
    });
    // ★★★ **…AND NOT IN A MODE THAT CANNOT EDIT CONTENT** — 2026-08-31, O71.
    //
    // A content selection is reachable in Read as of that row: the ordinary
    // pointer picks an IMAGE there so it can be copied out of pdfcer. Everything
    // else about that selection is the same, and the grips must not be — every
    // one of them commits a geometry edit the mode forbids, so offering them
    // would draw eight controls whose drag the funnel then refuses.
    //
    // ★ Read from `egui::Memory` rather than taken as a parameter, because this
    // function has four callers and only two of them hold a `Capabilities`.
    // `crate::app::modes::capability::edit_content_now` publishes it for
    // exactly this kind of read — the same shape `canvas::tool` uses for the
    // armed tool, and for the same reason: the alternative is a fifth argument
    // threaded through two call chains to answer one boolean.
    let editable = crate::app::modes::capability::edit_content_now(ctx);
    Grabbable {
        bounds,
        offer: if at_object_rung && editable {
            handles::GripSet::all()
        } else {
            handles::GripSet::default()
        },
        // ★ The one row where the box is a BOUND rather than the subject.
        // `overlay::grip_box` is `selection.outline_union()`, so on a CAD sheet
        // it is mostly empty paper. See `Grabbable::content`.
        content: true,
        // ★★★ …and the box is stroked only at the OBJECT rung (O69). At the
        // Part and Node rungs the anchors are the picture and the box is
        // clutter drawn on top of them. Note this is the same boolean the two
        // lines above already turn on: the rung decides the grips, the move-box
        // inflation AND the outline, which is one decision with three
        // consequences rather than three decisions.
        outline: at_object_rung,
    }
}

/// ★★★ **Does the press really land on something that is selected?** —
/// `OPERATOR_REQUESTS.md` O72.
///
/// The predicate [`look`] uses to decide whether a `Grip::Move` over a page
/// **content** selection is genuine, and the one `crate::canvas::presspick`
/// uses to decide whether the current selection already claims a point. One
/// function, two callers, because the two must not be able to disagree — see
/// the block comment at [`look`]'s grip downgrade for the incident that rule
/// comes from.
///
/// # What it asks
///
/// Whatever is topmost under `point`, and whether that thing is a member of
/// the selection. Deliberately the **same** hit test the canvas uses for
/// picking (`input::topmost`), so "the press landed on the object" here means
/// exactly what it means everywhere else, including its tolerance.
///
/// # Why `false` when there is no decomposition
///
/// A page whose objects have not decomposed cannot answer the question, and
/// the honest answer to *"is the press on the selection?"* is then no. That
/// direction is deliberate: `false` yields a marquee, which selects things;
/// `true` would yield a move of a selection the operator may not have pressed
/// on, which changes the document. When the question cannot be answered, the
/// gesture that cannot damage anything is the right default.
pub fn body_under(
    doc: &OpenDoc,
    selection: &SelectionState,
    map: &PageMapping,
    page_index: usize,
    point: Pos2,
    pick: crate::canvas::pick::PickFilter,
    scope: crate::canvas::smart::Scope,
) -> bool {
    let page_point = map.to_page(point);
    doc.page_objects()
        .and_then(|provider| {
            crate::canvas::input::topmost(&*provider, page_index, page_point, map, pick, scope)
        })
        .is_some_and(|hit| {
            selection
                .entries()
                .iter()
                .any(|e| e.page == page_index && e.object == hit)
        })
}

/// The pick filter [`body_under`] asks with.
///
/// ★ **Everything, deliberately, and not the operator's filter.** The question
/// is *"is the press on the thing I have selected?"*, and the answer must not
/// depend on whether that kind of object is currently pickable — an operator
/// who selects an image, then switches images off in the selection filter, has
/// not thereby asked for the image to become undraggable. The filter governs
/// what a press may *acquire*; this asks about what is already held.
fn pick_for_body() -> crate::canvas::pick::PickFilter {
    crate::canvas::pick::PickFilter::all()
}

/// Everything the frame learned by looking at where a press would land.
///
/// Returned as one struct rather than a tuple because four of the five members
/// are `Option`s of similar-looking types, and a caller that transposed two of
/// them would compile. Named fields make that a spelling error instead.
pub struct Press {
    /// The grip under the press origin, if any. `Grip::Move` for a press
    /// anywhere inside the (possibly inflated) selection box.
    pub grip: Option<Grip>,
    /// The Bézier handle under the press origin, as `(anchor, side)`.
    pub handle: Option<(usize, pdfcer_core::vector::Handle)>,
    /// Every drawn handle of the selected anchors, in canvas space.
    ///
    /// Carried out as well as used here, because the paint pass wants the same
    /// list and re-deriving it would mean a second `page_objects()` borrow and
    /// a second answer that could differ from this one.
    pub visible_handles: Vec<(usize, pdfcer_core::vector::Handle, Pos2)>,
    /// What a press means — the drag it would start, and whether a click has a
    /// meaning at all.
    pub meaning: PressMeaning,
}

/// Look at the pointer and answer what a press would do.
///
/// Changes nothing. See the module header for the precedence and for why every
/// hit test reads `press_origin`.
///
/// # Why eight arguments rather than a `Frame` struct
///
/// Because every one of them is a borrow the caller already holds and none is a
/// *product* of anything here — the same call `canvas::painting::draw` makes
/// when it takes `ui` and `doc` alongside its `Frame`. A struct assembled at the
/// one call site, passed once, and destructured immediately would be a grouping
/// that groups nothing.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn look(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selection: &SelectionState,
    map: &PageMapping,
    page_index: usize,
    screen_pos: Option<Pos2>,
    active_tool: CanvasTool,
    caps: Capabilities,
) -> Press {
    // ★★ At an inner rung the move-hit box is INFLATED by an anchor mark's
    // width, and without it the outermost anchors of every path are undraggable.
    //
    // An anchor mark is centred on its point, so an anchor sitting on the
    // object's bounding box is half outside it — and `grip_at` answers `Move`
    // only for a press *inside* the box. Confining the eight scale grips to the
    // Object rung stopped them CLAIMING that press; this is the other half,
    // which makes the move claim it. The operator's version of the pair is
    // *"I can drag the middle points and not the end ones"*.
    // ★★ A selected ce dimension supplies its OWN move box, and it comes
    // first.
    //
    // `overlay::grip_box` derives its answer from the selection's cached
    // content outlines, which `select_annot` clears - an annotation is not
    // content and has nothing decomposed to cache. So over a selected dimension
    // it answers `None`, the press fell through to a marquee, and pressing on
    // the dimension REPLACED the selection the operator was trying to drag.
    // That is the operator's report of 2026-08-20 in its mechanical form.
    //
    // `dimdrag::grab_box` is `Some` only for a dimension a placement drag can
    // actually finish, so no gesture is ever started that could not commit.
    let Grabbable {
        bounds: grip_box,
        offer,
        content,
        // Not read here — this function decides what a press MEANS, and
        // whether a box is stroked is a question for the painter. Named
        // rather than wildcarded so a fifth field has to be ruled on.
        outline: _,
    } = grabbable(ctx, doc, map, selection);
    let origin = ctx.input(|i| i.pointer.press_origin()).or(screen_pos);

    let grip = grip_box
        .zip(origin)
        .and_then(|(bounds, p)| handles::grip_at(bounds, p, offer));

    // ★★★ **A press on empty paper inside the selection's BOUNDING box is not
    // a press on the selection** — `OPERATOR_REQUESTS.md` O72.
    //
    // The operator: *"Click and hold shouldn't select an object - it should
    // allow me to draw a box around objects to select."*
    //
    // A marquee already exists and is already the default meaning of a press
    // that finds no grip (`gesture::meaning`, `(None, None) => Marquee`). What
    // made it unreachable is one line in `handles::grip_at`:
    // `bounds.contains(pointer).then_some(Grip::Move)`. For page content
    // `bounds` is `selection.outline_union()` — a rectangle around scattered
    // geometry, mostly empty. Select a title-block border once, which on a CAD
    // sheet is a hollow rectangle spanning the drawing, and **every** press on
    // the sheet fell inside that box and became a move. The band could not be
    // started anywhere.
    //
    // ⇒ Where the box is a bound rather than the subject (`Grabbable::content`),
    // confirm the press really lands on something selected before honouring
    // `Grip::Move`. It does not, so the grip falls to `None`, and
    // `press_kind`'s existing `(None, None) => Marquee(Select)` arm runs. No
    // new gesture, no new state, no new arm to audit.
    //
    // # ★ Only `Grip::Move`, never a resize grip and never Rotate
    //
    // Those eight-plus-one are **drawn**. The operator can see them, they sit
    // on the box's edges and corners, and a press on one is unambiguous — so
    // second-guessing it would break a gesture that is working. `Grip::Move`
    // is the only member of the set with no visible affordance of its own: it
    // is "anywhere inside", which is exactly why it is the one that can be
    // claimed by mistake.
    //
    // # ★★ And it is the same predicate `presspick::covers` asks
    //
    // Not a similar one. That guard exists to agree with this function, and its
    // own header records what happened the last time the two computed the same
    // answer separately — *"a second opinion computed differently here would
    // disagree with the gesture machine at the margins, and every disagreement
    // is a press that selects when it should have transformed."* So the check
    // lives in one exported function and both callers call it.
    let grip = match grip {
        Some(Grip::Move)
            if content
                && !origin.is_some_and(|p| {
                    body_under(
                        doc,
                        selection,
                        map,
                        page_index,
                        p,
                        pick_for_body(),
                        // ★ The same scope the click path resolves in — O70.
                        // Asking this question outside the operator's current
                        // container would answer about a different object from
                        // the one their next click will select.
                        crate::canvas::smart::scope(ctx, page_index),
                    )
                }) =>
        {
            None
        }
        other => other,
    };

    // ★★★ **THE ANNOTATION UNDER THE ROTATE HANDLE, AND IT IS DERIVED FROM
    // `grip` RATHER THAN FROM A SECOND HIT TEST.**
    //
    // This is the whole guard against the failure this canvas has produced four
    // times — *a working gesture aimed at the wrong verb*, which never looks
    // broken from a chair because something moves. The most recent instance
    // (`canvas::presspick`) is exactly this shape: `covers()` asked its own
    // question about where the pointer was, tested the selection's **move box**
    // alone, and the rotate handle sits OUTSIDE that box — so a press on the
    // handle selected the object underneath and the rotate became a
    // select-and-move.
    //
    // ⇒ **Nothing here asks a second question.** `grip` above came from
    // `handles::grip_at`, over `grabbable`'s box, with `grabbable`'s `GripSet`
    // — the same function, the same box and the same predicate the painter uses
    // (H7). If the handle was painted, this is `Some`; if it was not, this is
    // `None`. There is no third answer for the two to disagree about.
    //
    // ★★ And `grip` reads **`press_origin`**, not the current pointer, because
    // `origin` above does. That is this module's header rule and it is
    // load-bearing here in particular: egui does not call an interaction a drag
    // until the pointer has travelled a threshold, so by the frame it says so
    // the pointer is already ~20 pt from an 8 pt handle. A build that read the
    // live pointer would find `None` on every real rotate drag and the gesture
    // would fall through — to a marquee, which CLEARS the selection the
    // operator was trying to turn.
    //
    // ★ The kind is carried through rather than re-derived downstream, so
    // `gesture::press_kind` routes on a variant the compiler makes it handle:
    // a markup goes to `rotate_annotation` and a ce dimension to
    // `rotate_dimension`, and the engine refuses the first verb a dimension by
    // name. `canvas::selection::annot::AnnotKind`'s header states why that
    // distinction lives in the type.
    let annot_rotate = (grip == Some(Grip::Rotate))
        .then(|| selection.annot())
        .flatten()
        .map(|annot| match annot.target.kind {
            crate::canvas::selection::AnnotKind::Markup => gesture::RotatableAnnot::Markup,
            crate::canvas::selection::AnnotKind::CeDimension => {
                gesture::RotatableAnnot::CeDimension
            }
        });

    // The provider is asked for only at an inner rung — `handledrag::visible`
    // returns empty above it — so the ordinary case pays one `entered_object()`
    // and one `subpath` check.
    let visible_handles = doc
        .page_objects()
        .zip(doc.pages.get(page_index))
        .map(|(provider, page)| handledrag::visible(selection, &provider, page, page_index))
        .unwrap_or_default();

    let handle = origin.and_then(|p| handledrag::at(&visible_handles, map, p));

    // ★★ What a press on a selected ce dimension landed on — a corner handle,
    // its body, or nothing.
    //
    // Sampled here with the other two hit tests so that a press has one meaning
    // decided in one place (this module's header), and resolved to a VALUE
    // rather than left as two booleans, so `gesture::press_kind` stays free of
    // geometry.
    //
    // ★ A corner outranks the body, and it must: a handle sits ON the shape, so
    // every press that hits a handle also hits the body. Of the two readings,
    // the one the operator aimed at is the small square they can see.
    //
    // Cheap to ask — it answers `None` immediately unless an annotation is
    // selected and its sidecar record is a draggable kind.
    let dimension = origin.and_then(|p| {
        dimdrag::vertex_at(doc, map, selection, p)
            .map(gesture::DimensionPress::Vertex)
            .or_else(|| {
                // ★ Asked of `dimdrag` directly rather than of `grabbable`'s
                // box, because the two answer different questions: `grabbable`
                // says *what may be grabbed* and this says *is the thing under
                // the pointer a ce dimension*. They coincide today and would
                // stop coinciding the moment anything else offered no grips.
                dimdrag::grab_box(doc, map, selection)
                    .filter(|b| b.contains(p))
                    .map(|_| gesture::DimensionPress::Body)
            })
    });

    // Whether the press landed inside a selected MARKUP annotation's own box.
    //
    // ★ Sampled here with the other hit tests, resolved to a bool rather than
    // left for `press_kind` to compute, so that function stays free of geometry
    // — this module's stated contract. `annotdrag::grab_box` answers `None`
    // unless the selection is a markup this shell can actually move, so no
    // gesture is started that could not commit.
    let markup_body =
        origin.is_some_and(|p| annotdrag::grab_box(map, selection).is_some_and(|b| b.contains(p)));

    // Whether the press landed inside the selected FORM FIELD's box.
    //
    // ★ Same shape as `markup_body` above and the same contract:
    // `widgetdrag::grab_box` answers `None` unless a widget is selected and
    // still present in the form, so no gesture is started that could not
    // commit. The target list it consults is cached on `(path, edit_epoch)`,
    // so asking every frame costs a map lookup rather than a form walk.
    let widget_body =
        origin.is_some_and(|p| widgetdrag::grab_box(ctx, doc, map).is_some_and(|b| b.contains(p)));

    let meaning = gesture::press_kind(
        gesture::Press {
            tool: active_tool,
            grip,
            handle,
            dimension,
            annot_rotate,
            markup_body,
            widget_body,
            zoom_armed: zoom::region_zoom_armed(ctx),
        },
        caps,
    );

    Press {
        grip,
        handle,
        visible_handles,
        meaning,
    }
}

#[cfg(test)]
mod o69_outline_tests {
    use super::*;
    use crate::canvas::handles::GripSet;

    /// ★★★ **The four rows of `grabbable`'s table, and what each says about
    /// the outline** — `OPERATOR_REQUESTS.md` O69.
    ///
    /// Asserted as `Grabbable` literals rather than by driving `grabbable`,
    /// which needs a `Context` and an `OpenDoc`. What is under test is the
    /// *relationship between the three flags*, which is where a mistake would
    /// hide: the whole point of `outline` is that it is not expressible as
    /// either of the other two.
    #[test]
    fn the_outline_flag_is_not_the_content_flag_nor_the_grip_set() {
        // Page content at the OBJECT rung: the box is the subject, it is
        // stroked, and it is also a bound around scattered geometry.
        let object = Grabbable {
            bounds: None,
            offer: GripSet::all(),
            content: true,
            outline: true,
        };
        // Page content at an INNER rung: still a bound, no longer the subject.
        let inner = Grabbable {
            bounds: None,
            offer: GripSet::default(),
            content: true,
            outline: false,
        };
        // A ce dimension: the `/Rect` IS the object, so it is stroked — and it
        // offers no resize. This is the pair that proves `outline` is not
        // `offer.resize` in disguise.
        let dimension = Grabbable {
            bounds: None,
            offer: GripSet::rotate_only(),
            content: false,
            outline: true,
        };

        assert_eq!(
            (object.content, inner.content),
            (true, true),
            "`content` cannot distinguish the rungs — it is true at both"
        );
        assert_ne!(
            object.outline, inner.outline,
            "…and `outline` must, which is why it is a second flag"
        );
        assert!(
            !dimension.offer.resize && dimension.outline,
            "a ce dimension is stroked and offers no resize, so `outline` is not `offer.resize`"
        );
        assert!(
            !dimension.content && dimension.outline,
            "…and it is not `content` either: the two disagree here"
        );
    }
}
