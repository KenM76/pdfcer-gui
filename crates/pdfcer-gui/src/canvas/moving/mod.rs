//! # `canvas::moving` — dragging a selection, and the four things that make it honest
//!
//! ## What this module is for
//!
//! A press inside the selection, a drag, a release: the object moves. That is
//! one sentence and four separate obligations, and this module exists because
//! each of them is a place the gesture can go quietly wrong.
//!
//! 1. **One gesture is ONE command.** A multi-select moves through
//!    `EditSession::move_objects`, which takes a *slice*, resolves and
//!    type-checks every index before planning anything, and refuses the whole
//!    call rather than moving the prefix that happened to qualify. Emitting one
//!    `move_object` per selected entry would be N undo entries for one drag and
//!    — worse — N content-stream re-splices, each planned against byte offsets
//!    the previous one already invalidated. `docs/core-api/02` states the rule
//!    in a box: *"Never loop the singular verbs over a selection."*
//! 2. **The delta is PAGE space, never screen pixels.** See
//!    [`page_delta`] and the whole of [`crate::canvas::mapping`]'s header. A
//!    drag measured on screen and handed to a page-space verb compiles, runs,
//!    and merely scales with magnification — the same silent class as the
//!    hit-tolerance defect that module was built to make unavailable.
//! 3. **The preview must describe something that will actually happen.**
//!    `D:\Dev\FeatureRequests\pdfce_FeatureRequests\README.md` rule 4 welcomes
//!    a pre-commit affordance — *"a snap indicator, a hover highlight, a
//!    rubber-band, a selection handle — these are the cursor; they describe
//!    what is about to happen"* — and forbids marking content that has already
//!    been applied. A ghost outline is squarely in the first category, **as
//!    long as the move it describes is one the engine will accept.** So the
//!    ghost is drawn only when [`eligible`] has already said yes; a ghost over
//!    a text object at the Part rung, where no `move_*` verb applies, would be
//!    what `overlay`'s own note calls *"a lie with a low alpha"*.
//! 4. **The rung decides the verb, and a rung with no verb declines out loud.**
//!    Object → `move_objects`; Part → `move_subpath`; Node → `move_node`. The
//!    Part rung of a *text* object has no move verb at all (a show operator is
//!    not a subpath), so it refuses and traces, exactly as Delete refuses at
//!    the Part rung today.
//!
//! ## ★ Why the selection needs no invalidation across a move
//!
//! Because a move **does not renumber**. This was an open question that
//! blocked the whole feature, was asked as `request_stable_object_identity.md`,
//! and came back measured rather than asserted — the proof is
//! `crates/pdfcer-core/tests/object_identity_across_edits.rs`, which decomposes,
//! edits, and decomposes *again*:
//!
//! | family | mechanism | renumbers? |
//! |---|---|---|
//! | `move_object` · `move_objects` · `move_subpath` · `move_node` · `move_nodes` · `move_handle` | rewrites operator **operands** in place | **NO** |
//! | `delete_object` · `delete_objects` · `delete_subpath` · `delete_node` · `delete_text_run` | excises byte **spans** | **YES** |
//!
//! A move changes numbers *inside* existing operators. No operator is added or
//! removed, so a second decomposition walks the same operators in the same
//! order and yields the same objects at the same indices — asserted directly by
//! that test, and asserted to be non-vacuous (the moved object demonstrably
//! moved).
//!
//! So [`crate::canvas::selection::Selection`] — `{ page, object, subpath, node }`
//! — survives a move **unchanged**, with no durable token and no invalidation
//! pass. What *does* change is the geometry, and that is already handled by the
//! machinery invariant 3 built for a delete: the action bumps
//! `OpenDoc::edit_epoch`, `SelectionState::needs_resolve` sees the key move,
//! and the outlines are recomputed from the fresh decomposition on the next
//! frame. [`tests::a_move_never_alters_the_selection`] pins both halves.
//!
//! ## What is deliberately NOT here: resize
//!
//! `EditSession` has the entire `move_*` family and **no scale or resize verb
//! of any kind**. The eight grips are drawn, and a drag on one is *consumed*
//! (so it cannot fall through to a marquee and silently replace the selection
//! the operator was aiming at), and it commits nothing — see
//! [`crate::canvas::handles`]. Wiring a ghost to a resize grip would be an
//! affordance for something that cannot happen, which is the no-placeholders
//! invariant, and it is a separate change for the day the verb exists.
//!
//! ## The split between the pure rules and the wiring
//!
//! [`eligible`], [`action`] and [`page_delta`] are pure functions of plain
//! data, so every rule above is testable with no window, no document and no
//! decomposition — the same discipline that makes
//! [`crate::canvas::selection::SelectionState::click`] a pure function of a
//! [`ClickHit`](crate::canvas::selection::ClickHit). [`drag`] is the one
//! function that touches the live provider, and it does nothing except gather
//! those inputs, call the pure functions in order, and trace what happened.
//!
//! ## conventions: drag-moves
//!
//! Corpus: `ui-conventions/drag-moves.md`.
//!
//! - D1 live-preview: the ghost is drawn every in-flight frame, offset by the
//!   canvas delta.
//! - D2 derived-from-commit: `eligible` is consulted twice — once per frame to
//!   decide whether a ghost may be drawn at all, once on release to build the
//!   command — so the ghost appears if and only if the release would commit.
//! - D3 escape-cancels: the gesture machine drops the drag; nothing is written
//!   before `Complete`.
//! - D4 one-undo-entry: `move_objects` takes a slice, so a drag of forty objects
//!   is one command and one Ctrl+Z.
//! - D5 modifiers-constrain: **Shift locks the move to one axis**, applied by
//!   `canvas::interact` to the delta *before* it reaches this module — so the
//!   ghost and the commit read one filtered value and cannot disagree. The
//!   arithmetic, the re-decide-every-frame rule and the announcement are
//!   [`crate::canvas::constrain`].
//! - D6 snapping: **GAP** — a content move does not snap to guides, the grid or
//!   other geometry, while the measure tools snap to all three.
//! - D7 no-op-is-not-an-edit: a zero-travel drag is deliberately still
//!   *eligible* (it names a real verb on real operands) and commits nothing —
//!   the split is stated in `eligible`'s own docs and is what keeps the ghost
//!   visible when the pointer passes back over the press point.
//! - D8 grab-point: a delta, not an absolute position, so the grab is preserved.
//! - D9 disclosure: WAIVED — a translation changes no measured value and the
//!   operator can see where the objects went. There is nothing they cannot
//!   reconstruct by looking.

use egui::{Pos2, Vec2};
use pdfcer_core::page_tree::Page;
use pdfcer_core::vector::Point;

use crate::app::actions::{Action, VectorAction};
use crate::canvas::gesture::Phase;
use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::panels::objects::provider::{ObjectModelProvider, PartKind};
use crate::viewer;

/// A drag displacement in **PDF page space** — the frame every `move_*` verb
/// consumes.
///
/// A distinct type rather than a bare `(f64, f64)` so a canvas-space `Vec2`
/// cannot be handed to a page-space verb by a call that happens to typecheck.
/// The only way to build one is [`page_delta`], which is the only place in
/// `canvas/` that crosses into PDF space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageDelta {
    /// Horizontal displacement, PDF user-space units.
    pub dx: f64,
    /// Vertical displacement, PDF user-space units. **Y is up** here — the
    /// opposite of the canvas-space `Vec2` it was derived from.
    pub dy: f64,
}

impl PageDelta {
    /// Whether this displacement is a real move.
    ///
    /// # Why the threshold is exactly zero, and not a nudge more
    ///
    /// egui already applies the only distance threshold this gesture needs:
    /// a press-and-release that does not exceed the drag threshold is reported
    /// as `clicked`, never as a drag, so a shaky hand cannot reach here at all
    /// (see [`crate::canvas::gesture`]'s header). Adding a second threshold
    /// *in page space* would make it zoom-dependent in the wrong direction —
    /// at 16× a deliberate quarter-point nudge is a 4 px screen drag the
    /// operator meant, and swallowing it would read as "the drag did not
    /// take". So the only thing refused here is a gesture that ended exactly
    /// where it began (a drag out and back), which must not put a no-op
    /// command on the undo stack.
    ///
    /// Non-finite is refused for the obvious reason: it would author NaN
    /// operands into a content stream.
    #[must_use]
    pub fn is_travel(self) -> bool {
        self.dx.is_finite() && self.dy.is_finite() && (self.dx != 0.0 || self.dy != 0.0)
    }
}

/// Which core verb a completed move drag on this selection would reach, with
/// its operands already resolved.
///
/// One variant per rung of the selection ladder, because that is the whole
/// rule: the rung the operator is standing on decides which of the `move_*`
/// family the gesture means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveSubject {
    /// Every selected object, moved by a **matrix** rather than by rewriting
    /// coordinates — the rung a selection containing a text run, an image, a
    /// form XObject or an inline image takes.
    ///
    /// Reaches `EditSession::transform_objects` with `Matrix::translate(dx, dy)`
    /// in PAGE space. See [`eligible`]'s Object arm for why this is a second
    /// rung beside [`Self::Objects`] rather than a replacement for it, and
    /// `VectorAction::TransformObjects.into()` for the page-space contract.
    Transform {
        /// The 0-based page.
        page: usize,
        /// Paint-order indices, ascending and de-duplicated.
        objects: Vec<usize>,
    },
    /// The Object rung: `move_objects`, one command for the whole selection.
    Objects {
        /// The page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique — the clean operand list
        /// `move_objects` needs in order to succeed rather than refuse.
        objects: Vec<usize>,
    },
    /// ★★★ **The Object rung for things painted INSIDE a form XObject**:
    /// `move_objects_in_form`.
    ///
    /// `OPERATOR_REQUESTS.md` O70's second slice, and until `pdfcer-core`
    /// Pass 188.0 (2026-08-31) there was no verb to route it to — this was a
    /// worded refusal, [`Refusal::InsideForm`], because a leaf has no
    /// paint-order index and every geometry verb addressed one.
    ///
    /// ## ★★ A separate variant, not `Objects` with leaf indices in it
    ///
    /// Because the indices are in **different address spaces** and the whole
    /// safety property of `TargetId` is that they cannot be confused. `objects`
    /// are positions in `PageObjects::objects`; `leaves` are positions in
    /// `PageObjects::leaves`, which on the operator's benchmark drawing holds
    /// 10,256 entries against 129,758 — the same integer means two things and
    /// only the variant says which.
    ///
    /// ⇒ `TargetId`'s own header names the failure this avoids: *"in range and
    /// wrong is the dangerous combination"*. A merged variant would put the
    /// shell back in the business of remembering which numbering it is holding.
    LeavesInForm {
        /// The page the leaves are on.
        page: usize,
        /// Leaf indices, ascending and unique — the same clean operand shape
        /// every other verb here needs, and for the engine's own reason: it
        /// resolves every index before planning anything, so one stale entry
        /// refuses the whole batch.
        leaves: Vec<usize>,
    },
    /// ★★ The Part rung **inside a form XObject**: `move_subpath_in_form`.
    ///
    /// Its own variant beside [`Self::Subpath`] for [`Self::LeavesInForm`]'s
    /// reason: the enclosing object is a leaf index, and the two spaces must
    /// not share a field.
    SubpathInForm {
        /// The page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
    },
    /// ★★ The Node rung inside a form, one anchor: `move_node_in_form`.
    NodeInForm {
        /// The page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The anchor, object-scoped.
        node: usize,
    },
    /// ★★ The Node rung inside a form, several anchors: `move_nodes_in_form`.
    NodesInForm {
        /// The page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The anchors, object-scoped, ascending and unique.
        nodes: Vec<usize>,
    },
    /// The Part rung of a **path** object: `move_subpath`.
    Subpath {
        /// The page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
    },
    /// The Node rung with exactly **one** anchor selected: `move_node`.
    Node {
        /// The page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchor, **object-scoped** — the space `vector::anchor_count`
        /// reports and `pdfcer node-move --node N` addresses.
        node: usize,
    },
    /// The Node rung with **several** anchors selected: `move_nodes`.
    ///
    /// # ★ Why this is a second variant and not `Node` with a `Vec`
    ///
    /// Because the singular case has a verb of its own in `EditSession`, and
    /// `docs/core-api/02`'s rule cuts the other way too: the plural verb is
    /// correct for a set and the singular one is correct for a member, and
    /// collapsing them would mean either routing one node through a slice
    /// (losing the singular verb's own planner) or routing a set through a
    /// loop (which is the thing the rule forbids by name — N undo entries, and
    /// each planned against byte offsets the previous one invalidated).
    ///
    /// Both are the same gesture from the operator's side. The distinction is
    /// which engine verb the shell is entitled to call, which is exactly the
    /// kind of thing that belongs in a type rather than in an `if`.
    Nodes {
        /// The page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchors, object-scoped, ascending and unique.
        ///
        /// Never empty and never of length one — [`super::moving::subject`]
        /// produces [`Self::Node`] for the singular case, so a reader of this
        /// variant may assume two or more without checking.
        nodes: Vec<usize>,
    },
}

/// What the object model says about the entries a move would act on.
///
/// Assembled by [`drag`], which owns the provider, and handed to [`eligible`]
/// as plain data — the same shape, and for the same reason, as
/// [`ClickHit`](crate::canvas::selection::ClickHit): every rule below is then
/// a pure function of "what is selected" and "what kind of thing is it", with
/// no decomposition anywhere near the test that proves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoveContext {
    /// The paint-order index of the first selected object that is **not** a
    /// path, if there is one.
    ///
    /// Operand translation is path-only, and `move_objects` refuses the WHOLE
    /// call over a single non-path member rather than moving the paths and
    /// leaving a text object behind — a partial application that would read as
    /// a rendering fault rather than as a refusal. The *index* is carried
    /// rather than a bare `bool` because the engine's own error carries it for
    /// exactly this purpose: a refusal that cannot say which object refused is
    /// a refusal the operator cannot act on.
    pub non_path: Option<usize>,
    /// What kind of part the entered object decomposes into, at the Part and
    /// Node rungs. `None` for an object with no Part rung at all (an image).
    pub part_kind: Option<PartKind>,
}

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
    /// A leaf's geometry lives in the form's own content stream
    /// ([`pdfcer_core::vector::FormLeaf::stream`]) and
    /// [`pdfcer_core::vector::FormLeaf::is_editable`] is `false` for every leaf
    /// the engine produces, so this is a statement about what the engine can
    /// do today rather than a policy this shell chose. Dated 2026-08-27
    /// against `pdfcer-core` v0.14.0; when editing-through-recursion lands, the
    /// remedy is to route to the form-scoped verb, not to relax this.
    InsideForm,
    /// A selected object is not a path, so the whole move is refused. Carries
    /// its paint-order index.
    NotAPath(usize),
    /// The Part rung is entered but no part is named — an inconsistent state
    /// [`SelectionState`] does not produce, refused rather than guessed at.
    NoPartEntered,
    /// The entered part has no move verb: a text object's show operator is a
    /// "part", but `move_subpath` translates path construction operands and
    /// there is nothing for it to translate.
    NoVerbForPart(PartKind),
    /// The Node rung is entered but no anchor is named. Same nature as
    /// [`Self::NoPartEntered`].
    NoNodeEntered,
    /// The entered anchor is not in the object's current anchor list — the
    /// selection out-ran a decomposition. Carries the object-scoped index.
    NodeNotFound(usize),
    /// The gesture ended where it began. See [`PageDelta::is_travel`].
    NoTravel,
    /// The page's device transform is not invertible, so there is no
    /// well-defined page-space displacement. Declining is the only honest
    /// answer; authoring garbage geometry is not.
    DegeneratePage,
}

/// Convert a **canvas-space** drag delta into a **PDF page-space** one.
///
/// # ★ Why this is the only zoom-safe way to do it, and why no zoom appears
///
/// The zoom has already been divided out, once, before this is ever called.
/// `canvas/mod.rs` builds the frame's [`PageMapping`] and converts the
/// pointer's position through [`PageMapping::to_page`] *before* handing it to
/// the gesture machine, so [`crate::canvas::gesture::GestureOutcome::Move`]'s
/// `delta` is already a difference of two **canvas-space** points. There is
/// therefore no zoom left in it, and nothing here divides by one —
/// [`PageMapping`] deliberately has no `zoom()` accessor for exactly that
/// reason. A drag of the same screen distance yields the same page delta at
/// every magnification, which is
/// [`tests::a_drag_between_two_page_points_moves_the_same_distance_at_every_zoom`].
///
/// # Why two point conversions and a subtraction
///
/// [`viewer::canvas_to_pdf_space`] maps *points*, and a displacement is the
/// difference of two points. Taking that difference is what cancels the
/// transform's translation and leaves its linear part — the rotation and the
/// Y-flip — applied exactly once. Writing the linear part out by hand here
/// instead would be a second derivation of the page transform, which is the
/// precise failure `viewer`'s header warns about: *"PDF user space is y-UP;
/// canvas and screen are y-DOWN. The failure is silent — the page looks
/// perfect until someone selects a line and gets a different one."*
///
/// The subtraction is widened to `f64` *before* it is taken so no precision is
/// lost to an intermediate `f32` difference.
///
/// Returns `None` for a page whose device transform cannot be inverted, which
/// is the same condition under which both halves of the `viewer` bridge
/// decline. Callers refuse the move rather than authoring a fabricated delta.
///
/// [`PageMapping`]: crate::canvas::mapping::PageMapping
/// [`PageMapping::to_page`]: crate::canvas::mapping::PageMapping::to_page
#[must_use]
pub fn page_delta(canvas: Vec2, page: &Page) -> Option<PageDelta> {
    let origin = viewer::canvas_to_pdf_space(Pos2::ZERO, page)?;
    let moved = viewer::canvas_to_pdf_space(Pos2::ZERO + canvas, page)?;
    Some(PageDelta {
        dx: f64::from(moved.x) - f64::from(origin.x),
        dy: f64::from(moved.y) - f64::from(origin.y),
    })
}

/// Which verb a move drag on this selection would reach, or why it reaches
/// none.
///
/// Consulted **twice per drag**: once per frame while the drag is in flight,
/// to decide whether a ghost may be drawn at all, and once on release, to
/// build the command. Asking the same question both times is the mechanism
/// behind obligation 3 in the module docs — a ghost is drawn if and only if
/// the release would commit, so the preview cannot promise a move the engine
/// is going to refuse.
///
/// Deliberately says nothing about the *distance* dragged: a zero-travel drag
/// is eligible (it names a real verb on real operands), it simply has nothing
/// to commit, and that is [`action`]'s call. Splitting it this way is what
/// keeps the ghost visible during the frames where the pointer happens to pass
/// back over the press point.
pub fn eligible(
    selection: &SelectionState,
    page: usize,
    ctx: MoveContext,
) -> Result<MoveSubject, Refusal> {
    match selection.level() {
        SelectionLevel::Object => {
            // The same clean, ascending, de-duplicated operand list Delete
            // uses, and for the identical reason: `move_objects` resolves
            // EVERY index before planning anything, so one duplicate or stale
            // entry refuses the whole batch.
            let objects = selection.object_indices_on(page);
            if objects.is_empty() {
                // ★★ **An empty operand list is not the same as an empty
                // selection**, and saying so was a flat contradiction of what
                // was on screen.
                //
                // `object_indices_on` answers about the page's own paint order
                // and drops every target drawn inside a form XObject —
                // correctly, because no paint-order verb can address one. Since
                // 2026-08-27 an ordinary click can produce a selection made
                // entirely of those, and this arm reported it as *"nothing
                // selected"* while the operator was looking at an outline round
                // the thing they were dragging.
                // ★★★ **A pure form-interior selection MOVES, as of
                // 2026-09-01** — O70's second slice.
                //
                // This arm returned `Refusal::InsideForm` for the life of the
                // shell, and the refusal was honest: no geometry verb could
                // address a leaf. `pdfcer-core` Pass 188.0 shipped six that can,
                // and this is the first of them to be wired.
                //
                // ★ The refusal is KEPT for the case it is still true of — an
                // empty selection is still nothing selected, and a leaf list
                // the engine later declines still produces a worded decline
                // from the apply arm rather than silence.
                let leaves = selection.leaf_indices_on(page);
                return if leaves.is_empty() {
                    Err(Refusal::NothingSelected)
                } else {
                    Ok(MoveSubject::LeavesInForm { page, leaves })
                };
            }
            // ★★★ THE REFUSAL BECAME A FORK — 2026-08-20, and it is the
            // operator's *"can I please please please have the capability to
            // move the text after?"*
            //
            // This read:
            //
            //     if let Some(index) = ctx.non_path {
            //         return Err(Refusal::NotAPath(index));
            //     }
            //
            // and it was right: `move_objects` rewrites numeric **operands**,
            // and a text run and an image carry no coordinate operands at all.
            // `Pass 113.0`'s `transform_objects` wraps the object's operator run
            // in `q <cm> … Q`, which never looks at an operand — so it moves
            // anything.
            //
            // ★★ WHY THIS IS A FORK AND NOT A REPLACEMENT, which is the part a
            // reader will want to argue with.
            //
            // Both verbs move things and both are one command and one undo
            // entry, so the obvious tidy is to route everything through the
            // transform. That would be worse, and the reason is the FILE rather
            // than the API: `move_objects` rewrites coordinates in place and
            // adds nothing, while a transform adds a `q`, a `cm` and a `Q` per
            // object per gesture. On this operator's drawings a nudge is
            // something he does dozens of times to hundreds of objects, and the
            // wrapping accumulates in a file he then sends to somebody.
            //
            // So: **the lighter verb where it can express the gesture, the
            // general one where it cannot.** The predicate is unchanged — it is
            // the same `ctx.non_path` that used to refuse — which is what makes
            // this a fork rather than a second notion of "is this a path".
            match ctx.non_path {
                None => Ok(MoveSubject::Objects { page, objects }),
                Some(_) => Ok(MoveSubject::Transform { page, objects }),
            }
        }
        SelectionLevel::Part => {
            let entry = entered_entry(selection, page)?;
            let subpath = entry.subpath.ok_or(Refusal::NoPartEntered)?;
            // ★★★ **Inside a form, the same rung reaches a different verb** —
            // `OPERATOR_REQUESTS.md` O70, 2026-09-01. Asked before the kind
            // match because the address space decides which family of verbs
            // exists, and the kind decides which member of it.
            if let Some(leaf) = entry.object.leaf_index() {
                return match ctx.part_kind {
                    Some(PartKind::Subpath) => Ok(MoveSubject::SubpathInForm {
                        page,
                        leaf,
                        subpath,
                    }),
                    // A text run inside a form has no move verb either — the
                    // engine's six are node and object verbs — so it declines
                    // by the same name it would on the page.
                    Some(other) => Err(Refusal::NoVerbForPart(other)),
                    None => Err(Refusal::InsideForm),
                };
            }
            let object = entry
                .object
                .page_object_index()
                .ok_or(Refusal::UnaddressableObject)?;
            match ctx.part_kind {
                Some(PartKind::Subpath) => Ok(MoveSubject::Subpath {
                    page,
                    object,
                    subpath,
                }),
                // A text run IS a part, and it has no move verb. Declining
                // here rather than letting `move_subpath` refuse downstream is
                // what keeps the ghost truthful — the engine's refusal arrives
                // after the operator has already watched an outline slide.
                Some(other) => Err(Refusal::NoVerbForPart(other)),
                None => Err(Refusal::NotAPath(object)),
            }
        }
        SelectionLevel::Node => {
            let entry = entered_entry(selection, page)?;
            let node = entry.node.ok_or(Refusal::NoNodeEntered)?;
            // ★★★ …and the Node rung, for the Part rung's reason.
            if let Some(leaf) = entry.object.leaf_index() {
                let nodes = selection.selected_nodes_on(page, entry.object);
                return match ctx.part_kind {
                    Some(PartKind::Subpath) if nodes.len() > 1 => {
                        Ok(MoveSubject::NodesInForm { page, leaf, nodes })
                    }
                    Some(PartKind::Subpath) => Ok(MoveSubject::NodeInForm { page, leaf, node }),
                    Some(other) => Err(Refusal::NoVerbForPart(other)),
                    None => Err(Refusal::InsideForm),
                };
            }
            let object = entry
                .object
                .page_object_index()
                .ok_or(Refusal::UnaddressableObject)?;
            // ★★ **Every selected anchor on the entered object, not just the
            // entered one.** `SelectionState::pick_within` has always added a
            // Shift-clicked anchor as its own entry — the model could hold a
            // multi-node selection from the day the Node rung landed — and this
            // function read `entered_object()`, which is the FIRST entry. So an
            // operator could Shift-click four anchors, watch four highlight,
            // drag, and move one.
            //
            // That is the defect `pdfcer`'s own `gui` column ticked `[x]` for
            // months (their note of 2026-08-19: "multi-node select-and-move —
            // objects move together; nodes one at a time"), and it is one of
            // the six rows that were true of the OLD in-repo shell and became
            // false when the column's referent moved to this build.
            let nodes = selection.selected_nodes_on(page, entry.object);
            match ctx.part_kind {
                Some(PartKind::Subpath) if nodes.len() > 1 => Ok(MoveSubject::Nodes {
                    page,
                    object,
                    nodes,
                }),
                Some(PartKind::Subpath) => Ok(MoveSubject::Node { page, object, node }),
                Some(other) => Err(Refusal::NoVerbForPart(other)),
                None => Err(Refusal::NotAPath(object)),
            }
        }
    }
}

/// The entered entry of a deeper rung, **whichever index space it names**.
///
/// Refuses an entry that belongs to a different page rather than addressing
/// page A's index space with page B's number — the same class of error the
/// [`TargetId`](crate::canvas::target::TargetId) newtype exists to prevent,
/// and one comparison to rule out.
///
/// ★ It stopped resolving the index on 2026-09-01 (O70). It used to answer
/// `(usize, Selection)` and refuse a leaf with `Refusal::InsideForm` on the
/// way — which was right while no verb could address one, and is now a
/// decision the CALLER makes, because the two arms above route to two families
/// of verb rather than to one.
fn entered_entry(
    selection: &SelectionState,
    page: usize,
) -> Result<crate::canvas::selection::Selection, Refusal> {
    let entry = selection
        .entered_object()
        .ok_or(Refusal::NothingSelected)
        .and_then(|e| {
            (e.page == page)
                .then_some(e)
                .ok_or(Refusal::NothingSelected)
        })?;
    Ok(entry)
}

/// The ONE action a completed move drag becomes.
///
/// `node_at` is the entered anchor's **current** page-space position, and is
/// consulted only by [`MoveSubject::Node`]. It is needed because `move_node`
/// takes an absolute destination rather than a displacement — the operand it
/// rewrites is a coordinate pair, and expressing the drag as "where the point
/// ends up" is what lets the planner map one point through the object's CTM
/// inverse instead of decomposing a translation into a space it would have to
/// re-derive.
pub fn action(
    subject: MoveSubject,
    delta: PageDelta,
    node_at: Option<Point>,
    points: &[(usize, Point)],
) -> Result<Action, Refusal> {
    if !delta.is_travel() {
        return Err(Refusal::NoTravel);
    }
    match subject {
        // ★ `translate` in PAGE space, which is what `PageDelta` already is —
        // `page_delta` did the one canvas → page conversion and this is the
        // same pair of numbers `MoveSelection` below hands to `move_objects`.
        // Two rungs, one displacement, no second derivation.
        MoveSubject::Transform { page, objects } => Ok(VectorAction::TransformObjects {
            page,
            objects,
            matrix: pdfcer_core::vector::Matrix::translate(delta.dx, delta.dy),
        }
        .into()),
        MoveSubject::Objects { page, objects } => Ok(VectorAction::MoveSelection {
            page,
            objects,
            dx: delta.dx,
            dy: delta.dy,
        }
        .into()),
        MoveSubject::SubpathInForm {
            page,
            leaf,
            subpath,
        } => Ok(VectorAction::MoveSubpathInForm {
            page,
            leaf,
            subpath,
            dx: delta.dx,
            dy: delta.dy,
        }
        .into()),
        MoveSubject::NodeInForm { page, leaf, node } => {
            // ★ Absolute, exactly as the page-level arm below: the verb takes
            // where the point IS GOING, not how far it moved, because the
            // operand it rewrites is a coordinate pair. `node_at` is the
            // anchor's current page-space position and comes from the same
            // provider the ghost was drawn from.
            let from = node_at.ok_or(Refusal::NodeNotFound(node))?;
            Ok(VectorAction::MoveNodeInForm {
                page,
                leaf,
                node,
                to: Point::new(from.x + delta.dx, from.y + delta.dy),
            }
            .into())
        }
        MoveSubject::NodesInForm { page, leaf, nodes } => {
            // ★ A selected anchor the current decomposition does not have
            // refuses the WHOLE drag, for the page-level arm's reason: a
            // partial application reads as a rendering fault rather than as a
            // refusal, and the operator cannot tell which anchor was dropped.
            let mut moves = Vec::with_capacity(nodes.len());
            for node in nodes {
                let from = points
                    .iter()
                    .find_map(|(i, p)| (*i == node).then_some(*p))
                    .ok_or(Refusal::NodeNotFound(node))?;
                moves.push((node, Point::new(from.x + delta.dx, from.y + delta.dy)));
            }
            Ok(VectorAction::MoveNodesInForm { page, leaf, moves }.into())
        }
        MoveSubject::LeavesInForm { page, leaves } => Ok(VectorAction::MoveLeavesInForm {
            page,
            leaves,
            dx: delta.dx,
            dy: delta.dy,
        }
        .into()),
        MoveSubject::Subpath {
            page,
            object,
            subpath,
        } => Ok(VectorAction::MoveSubpath {
            page,
            object,
            subpath,
            dx: delta.dx,
            dy: delta.dy,
        }
        .into()),
        MoveSubject::Node { page, object, node } => {
            let from = node_at.ok_or(Refusal::NodeNotFound(node))?;
            Ok(VectorAction::MoveNode {
                page,
                object,
                node,
                to: Point::new(from.x + delta.dx, from.y + delta.dy),
            }
            .into())
        }
        MoveSubject::Nodes {
            page,
            object,
            nodes,
        } => {
            // ★ A selected anchor that the current decomposition does not have
            // refuses the WHOLE drag rather than moving the ones it recognises.
            // The same call `move_objects` makes over a non-path member, and
            // for the same reason: a partial application reads as a rendering
            // fault, not as a refusal, and the operator has no way to tell
            // which of their four anchors was silently dropped.
            let mut moves = Vec::with_capacity(nodes.len());
            for node in nodes {
                let from = points
                    .iter()
                    .find_map(|(i, p)| (*i == node).then_some(*p))
                    .ok_or(Refusal::NodeNotFound(node))?;
                moves.push((node, Point::new(from.x + delta.dx, from.y + delta.dy)));
            }
            Ok(VectorAction::MoveNodes {
                page,
                object,
                moves,
            }
            .into())
        }
    }
}

/// Gather what the object model says about the selection a move would act on.
///
/// Returns `None` when there is no object model for the page at all, which is
/// distinct from "the model says no": nothing can be verified, so nothing may
/// be promised, and [`drag`] turns it into [`Refusal::NoObjectModel`].
///
/// The Object-rung scan asks [`ObjectModelProvider::part_kind`] once per
/// selected entry, which is a `Vec::get` and a match. It runs on every frame
/// of an in-flight drag, and that is affordable for the reason the whole
/// preview is affordable: the decomposition is already built and cached (the
/// selection could not have outlines to drag without it), so this walks a
/// slice rather than a content stream.
fn context(
    selection: &SelectionState,
    page: usize,
    provider: Option<&ObjectModelProvider>,
) -> Option<MoveContext> {
    let provider = provider?;
    // ★★★ **Asked of the TARGET, not of a page index** — O70, 2026-09-01.
    //
    // This read `.and_then(|e| e.object.page_object_index())`, with a comment
    // saying a form-interior target *"has no `part_kind` to ask about"*. It had
    // one; nothing could answer. `provider::geometry::part_kind_of` can, and
    // the consequence of the old line was subtle enough to be worth recording:
    // `eligible`'s Part arm reached its `None` branch and declined
    // `InsideForm` **after** the descent had already succeeded, so the operator
    // could enter the rung, watch the anchors draw, drag, and be refused.
    //
    // Found by driving it — `the_ladder_goes_as_deep_inside_a_container_as_
    // outside_one` reported `canvas-move-declined level=Part reason=InsideForm`
    // with every other line in the trace looking correct.
    let entered = selection.entered_object().map(|e| e.object);
    Some(MoveContext {
        non_path: selection
            .object_indices_on(page)
            .into_iter()
            .find(|&i| provider.part_kind(i) != Some(PartKind::Subpath)),
        part_kind: entered.and_then(|target| provider.part_kind_of(target)),
    })
}

/// The entered anchor's current page-space position, or `None` if the object's
/// anchor list no longer holds that index.
///
/// [`ObjectModelProvider::object_node_points`] is the whole-object list
/// precisely so a caller does not have to re-derive which subpath an
/// object-scoped index falls in — that offset arithmetic lives in one place,
/// in the provider, and duplicating it here is how the number pdfcer shows
/// starts disagreeing with the number the operator can act on.
fn node_point(provider: &ObjectModelProvider, object: usize, node: usize) -> Option<Point> {
    provider
        .object_node_points(object)
        .into_iter()
        .find(|(index, _)| *index == node)
        .map(|(_, point)| point)
}

/// **What one frame of a move drag gives the painter.**
///
/// Two values rather than one, since `OPERATOR_REQUESTS.md` **O63**, and they
/// answer different questions:
///
/// | field | question |
/// |---|---|
/// | [`Self::ghost`] | *where is the selection going?* — the bounding outline, which is the SELECTION indicator |
/// | [`Self::shape`] | *what will it look like?* — the real geometry, which is what the operator asked for |
///
/// ★★ The second is `None` on every rung the shell cannot draw honestly: a text
/// run, an image, a form XObject, a page that will not decompose, or a selection
/// past `canvas::shapes`' cap. In every one of those cases the outline alone is
/// drawn, which is exactly what this canvas did before the shape preview
/// existed — so the fallback is a known-good behaviour rather than a degraded
/// one.
///
/// ★ Not folded into one enum. `dragroute::Previews` gives the argument and it
/// applies here: the painter reads each independently, and one value whose
/// meaning depends on which rung is live is a value the paint loop has to
/// interrogate.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MovePreview {
    /// The canvas-space displacement, for the bounding ghost.
    pub ghost: Option<Vec2>,
    /// The selection's own geometry at its new position, in page space.
    pub shape: Option<crate::canvas::shapes::ShapePreview>,
    /// ★★★ **The same geometry, on the frame the gesture RELEASED**, for the
    /// canvas to hold until the page raster catches up (O63's third piece).
    ///
    /// `Some` on exactly one frame per gesture — the one that pushed the
    /// `Action` — and `None` on every other, which is what makes it unambiguous
    /// to the caller. [`Self::shape`] is the in-flight value and is `None` on
    /// that same frame, so the two never both speak.
    ///
    /// # Why it is a third field rather than reusing [`Self::shape`]
    ///
    /// Because they mean different things to the caller: one is *draw this now*
    /// and the other is *keep drawing this until the picture is right*. A caller
    /// that had to infer which from the phase would be inferring something this
    /// function already knows.
    pub hold: Option<crate::canvas::shapes::ShapePreview>,
}

/// Apply one frame of a move drag: draw the ghost, or commit the command.
///
/// The **only** function here that touches the live object model. It gathers
/// [`context`], asks [`eligible`], and then does one of two things:
///
/// * [`Phase::InFlight`] — returns the canvas-space delta for the ghost, and
///   changes nothing. Nothing is re-rasterized and nothing is decomposed: the
///   ghost is a translated copy of the outlines
///   [`SelectionState::outlines`] already caches in canvas space, which is
///   zoom-independent, so a preview costs one `Rect::translate` and one stroke
///   per selected entry.
/// * [`Phase::Complete`] — converts the delta to page space, resolves the node
///   position if the rung needs one, and pushes exactly one [`Action`].
///
/// Returns a [`MovePreview`] carrying the bounding ghost and, since
/// `OPERATOR_REQUESTS.md` O63, the selection's own **geometry** at its new
/// position. A drag that is not eligible draws nothing, which is the visible
/// half of obligation 3.
///
/// # Why the refusal is traced only on release
///
/// An in-flight drag is re-evaluated 60 times a second. Tracing a refusal per
/// frame would bury every other event on the channel — the lesson
/// `canvas-pointer` taught when a stationary pointer emitted fifty identical
/// lines in nine seconds. The release is one event, and it is the one a
/// harness reading the trace is asking about.
pub fn drag(
    delta: Vec2,
    phase: Phase,
    selection: &SelectionState,
    page_index: usize,
    provider: Option<&ObjectModelProvider>,
    page: Option<&Page>,
    actions: &mut Vec<Action>,
) -> MovePreview {
    let outcome = context(selection, page_index, provider)
        .ok_or(Refusal::NoObjectModel)
        .and_then(|ctx| eligible(selection, page_index, ctx));

    let subject = match outcome {
        Ok(subject) => subject,
        Err(reason) => {
            if phase == Phase::Complete {
                decline(selection, reason, actions);
            }
            return MovePreview::default();
        }
    };

    if phase == Phase::InFlight {
        // ★★★ THE LIVE SHAPE, `OPERATOR_REQUESTS.md` O63.
        //
        // **Ken, 2026-08-30:** *"if I moved the end of a line, it didn't show me
        // the shape change of the line, it just had a perimeter box around it …
        // there isn't a real preview like there is in inkscape."*
        //
        // Built HERE rather than in the painter, and that placement is the whole
        // guarantee: `subject` is the value the release hands to `EditSession`,
        // so there is no way to reach this line without having already decided
        // what the commit will do. Convention D2 — *derived from commit* —
        // enforced by control flow rather than by discipline.
        //
        // ★ The bounding ghost is returned as well, not instead. It is the
        // SELECTION indicator and it stays; what changes is that the shape now
        // moves with it. And on a rung the shape preview cannot serve — a text
        // run, an image, a form XObject, or a selection past the cap — the
        // outline is the whole answer, exactly as it was before this existed.
        let shape = page
            .and_then(|page| page_delta(delta, page))
            .zip(provider)
            .and_then(|(d, provider)| {
                crate::canvas::shapes::for_move_subject(provider, &subject, d.dx, d.dy)
            })
            .filter(|preview| !preview.is_empty());
        return MovePreview {
            ghost: Some(delta),
            shape,
            hold: None,
        };
    }

    // ---- commit ------------------------------------------------------
    let Some(page) = page else {
        decline(selection, Refusal::DegeneratePage, actions);
        return MovePreview::default();
    };
    let Some(delta) = page_delta(delta, page) else {
        decline(selection, Refusal::DegeneratePage, actions);
        return MovePreview::default();
    };
    // Only the Node rung needs a position, and asking for one costs an
    // allocation over every anchor of the object — 6,681 of them on one
    // measured CAD export — so it is asked for once, on release, and only for
    // the rung that consumes it.
    let node_at = match (&subject, provider) {
        (MoveSubject::Node { object, node, .. }, Some(provider)) => {
            node_point(provider, *object, *node)
        }
        _ => None,
    };
    // The plural rung needs every anchor's position, and that is the allocation
    // the singular rung's comment above is at pains to avoid — 6,681 anchors on
    // one measured CAD export. Asked for once, on release, and only when the
    // selection actually holds more than one node: the cost is paid by the
    // gesture that needs it and by no other.
    let points = match (&subject, provider) {
        (MoveSubject::Nodes { object, .. }, Some(provider)) => provider.object_node_points(*object),
        _ => Vec::new(),
    };

    // ★ Cloned before `action` consumes it, so the hold below describes the
    // SAME subject the Action carries. A hold rebuilt from the selection would
    // be a second derivation of what the release decided, and the two could
    // disagree on exactly the rung where a disagreement is invisible.
    let subject_for_hold = subject.clone();
    match action(subject, delta, node_at, &points) {
        Ok(raised) => {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "canvas-move page={page_index} level={:?} dx={:.4} dy={:.4} action={raised:?}",
                    selection.level(),
                    delta.dx,
                    delta.dy,
                )
            });
            actions.push(raised);
        }
        Err(reason) => {
            decline(selection, reason, actions);
            // ★★ A refused release holds NOTHING. The document did not change,
            // so there is nothing for a preview to be true about, and holding
            // one would show the operator a move that was declined — the
            // "placeholder" failure R9 forbids, in its most misleading form.
            return MovePreview::default();
        }
    }
    // ★★★ HOLD IT. `OPERATOR_REQUESTS.md` O63, third piece.
    //
    // **Ken, 2026-08-30:** *"the live preview should remain while the update to
    // the pdf structure runs in the background."*
    //
    // The gesture is over and the Action is raised, but the page raster
    // underneath still shows the object where it STARTED, for one to two
    // seconds on his own drawing. Discarding the preview here — which is what
    // this function did until today — makes the object appear to snap back to
    // its old position and then jump forward when the raster lands.
    //
    // ★ Built from the SAME `delta` and the SAME `subject` the Action carries,
    // one line below where it was pushed. There is no second computation to
    // drift, and a hold cannot describe a move the engine was not asked to make.
    let hold = provider
        .and_then(|provider| {
            crate::canvas::shapes::for_move_subject(provider, &subject_for_hold, delta.dx, delta.dy)
        })
        .filter(|preview| !preview.is_empty());
    MovePreview {
        ghost: None,
        shape: None,
        hold,
    }
}

/// Report a move that committed nothing, with the reason.
///
/// One trace shape for every refusal, so a harness reads `canvas-move-declined`
/// and finds the cause on the same line rather than inferring it from an
/// absence — the same contract `canvas-delete-declined` already honours.
fn decline(selection: &SelectionState, reason: Refusal, actions: &mut Vec<Action>) {
    // ★★ **One refusal out of the eight has something to say to the operator,
    // and it is the one they will meet without having made a mistake.**
    //
    // The rest describe states the operator put themselves in and can see:
    // nothing selected, a rung with no verb, a drag that travelled zero
    // distance. A sentence in the status bar for any of those would be a bar
    // that narrates the obvious, and a surface that narrates the obvious stops
    // being read.
    //
    // `InsideForm` is different in kind. The operator has an outline round the
    // thing they are dragging, the drag does nothing at all, and there is no
    // way on screen to learn why — from where they sit, dragging is broken.
    // Since 2026-08-27 that state is reachable by an ordinary click, so it is
    // no longer rare.
    //
    // ★ Recorded from the CANVAS, which no other decline in this application
    // does. It is sound for the reason `status::decline`'s header gives for the
    // store being a thread-local at all: `eframe`'s update loop is one thread,
    // the writer and the reader are the same thread, and this changes no
    // document. And the retirement rule still holds — `retire` runs at the top
    // of `dispatch_command`, so the operator's next command ends the sentence,
    // and `Declined::InsideForm::still_true` ends it the moment they select
    // something that is not in a form.
    // ★ Raised as an **action**, not written to the decline store directly.
    //
    // The store is `pub(super)` inside `crate::app` on purpose — *"a decline is
    // written by the one dispatcher and read by the one bar"* — and widening it
    // so the canvas could reach in would trade a real boundary for four fewer
    // lines. It would also cut against this crate's grain everywhere else: a
    // panel that wants to change something **asks**, because it holds
    // `&OpenDoc` and not `&mut`, and `Action` is that channel. The canvas has
    // the same relationship to the application state and gets the same answer.
    if reason == Refusal::InsideForm {
        actions.push(Action::DeclineInsideForm);
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-move-declined level={:?} sel={} reason={reason:?}",
            selection.level(),
            selection.len(),
        )
    });
}

// The move gesture's assertions. Split out under R2 on 2026-08-27; see its
// header for why the tests were the seam and the code was not.
#[cfg(test)]
mod tests;
