//! # `app::actions::vector` — everything that changes page GEOMETRY
//!
//! ## Why this is its own file
//!
//! **R2**, and the seam [`super::action::Action`] already draws twice:
//! [`super::dimensions`] is *what happens to the dimensioning model*,
//! [`super::pages`] is *what happens to a page*, [`super::annots`] is *what
//! happens to an annotation that already exists*. This is **what happens to the
//! marks on the page**.
//!
//! It is a real subject rather than a size-driven cut, and the evidence is a
//! property every variant here shares and nothing elsewhere does: **each one
//! addresses paint-order indices into one page's content stream.** That makes
//! all of them subject to one rule nothing else in `actions` has to think
//! about — `docs/core-api/02-editing-and-saving.md` §1.10.1, *which verbs
//! RENUMBER* — and it is why the page travels on every variant rather than
//! being re-derived at apply time.
//!
//! ## ★★ The two verbs that both "move things", and why there are two
//!
//! [`VectorAction::MoveSelection`] reaches `move_objects`, which rewrites
//! numeric **operands** in place. [`VectorAction::TransformObjects`] reaches
//! `transform_objects`, which wraps each object's operator run in `q <cm> … Q`
//! and never looks at an operand at all.
//!
//! The second can express everything the first can and more — rotation, scale,
//! and *any object kind*, because text runs and images carry no coordinate
//! operands, which is exactly why `move_objects` is path-only by name. The
//! obvious tidy is therefore to route everything through the transform, and it
//! would be worse.
//!
//! **The reason is the FILE rather than the API.** `move_objects` adds nothing;
//! a transform adds a `q`, a `cm` and a `Q` per object per gesture. On this
//! operator's drawings a nudge is something done dozens of times to hundreds of
//! objects, and the wrapping accumulates in a file he then sends to somebody.
//! So: the lighter verb where it can express the gesture, the general one where
//! it cannot. `canvas::moving::eligible` is the one place that forks, on the
//! same predicate that used to *refuse* the general case.

use pdfcer_core::vector::{Handle, Matrix, Point};

impl From<VectorAction> for super::action::Action {
    /// ★ So a call site says what it MEANS and the wrapping is not its problem.
    ///
    /// Thirty-five places raise one of these, and almost all of them are a
    /// single `actions.push(…)` at the end of a gesture. Making each write
    /// `Action::Vector(VectorAction::MoveNodes { … })` would put the enum's
    /// filing system into every one of them — and the filing system is an R2
    /// artefact rather than something a drag needs to know about.
    ///
    /// `.into()` at the push, `From` here, one line.
    fn from(v: VectorAction) -> Self {
        Self::Vector(v)
    }
}

/// One change to the marks on a page.
///
/// Carried by [`super::action::Action::Vector`]. Every variant names a page and
/// paint-order indices into it; see the module header for why both travel
/// rather than being re-derived.
#[derive(Debug, Clone, PartialEq)]
pub enum VectorAction {
    /// Remove the canvas selection's objects from `page`, as **one**
    /// undoable command.
    ///
    /// Raised by the canvas when Delete or Backspace is pressed with a
    /// non-empty selection and no text field focused — the defect `DEFECTS.md`
    /// D1 is about, from the other end. D1's fix (`ctx.text_edit_focused()`
    /// rather than `ctx.egui_wants_keyboard_input()`) made the key *reachable*
    /// after a canvas click; this is the verb it reaches.
    ///
    /// # The operand list is already clean, and must be
    ///
    /// `objects` arrives ascending and de-duplicated from
    /// [`crate::canvas::selection::SelectionState::object_indices_on`],
    /// because `EditSession::delete_objects` resolves **every** index before
    /// planning anything: one stale or duplicated entry refuses the whole
    /// call. That refusal is the correct engine behaviour — the alternative
    /// is deleting the prefix that happened to resolve — so the shell's job
    /// is to hand it a list that can succeed.
    ///
    /// # Why the page travels with the list
    ///
    /// A paint-order index is a position on **one page**. Re-deriving the
    /// page here from `doc.view.page_index` would be a second source of truth
    /// that is right until the moment it matters: an action is applied after
    /// the frame that raised it, and a page step raised in the same frame is
    /// applied first if it was pushed first. Carrying the page makes the
    /// statement complete.
    DeleteSelection {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
    },
    /// Displace the canvas selection's objects on `page` by a **page-space**
    /// delta, as **one** undoable command.
    ///
    /// Raised by [`crate::canvas::moving::drag`] when a move drag that began
    /// inside the selection is released. The Object-rung member of the move
    /// family; its siblings are [`Self::MoveSubpath`] and [`Self::MoveNode`].
    ///
    /// # Why the whole list travels, exactly as it does for Delete
    ///
    /// `EditSession::move_objects` takes a **slice**, and resolves *and
    /// type-checks* every index before planning anything, so one non-path or
    /// one stale entry refuses the whole call rather than moving the prefix
    /// that happened to qualify. Emitting one `move_object` per selected
    /// object would be wrong twice over: N undo entries for one drag, and — the
    /// correctness half — each call re-splices the content stream, so the
    /// second index would be planned against byte offsets the first already
    /// invalidated. `docs/core-api/02` states it in a box: *"Never loop the
    /// singular verbs over a selection."*
    ///
    /// # ★ Why this does NOT invalidate the selection, and Delete does
    ///
    /// Because `move_*` **does not renumber**, and that is measured rather
    /// than assumed — `crates/pdfcer-core/tests/object_identity_across_edits.rs`
    /// decomposes, edits, and decomposes again. A move rewrites operands
    /// *inside* existing operators, so no operator is added or removed and the
    /// second decomposition yields the same objects at the same indices. The
    /// `delete_*` family excises byte **spans** and therefore does renumber,
    /// which is why `pdfcer_core::vector::remap_index_after_delete` exists and
    /// why nothing like it is needed here. See
    /// [`crate::canvas::moving`]'s header for the full table.
    ///
    /// # Units
    ///
    /// `dx`/`dy` are **PDF user-space** points, Y-**up** — produced by
    /// [`crate::canvas::moving::page_delta`], which is the one place a
    /// canvas-space drag crosses into page space. A screen-pixel delta here
    /// would compile, run, and scale the move with the magnification.
    MoveSelection {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// ★★★ Delete every selected object **inside a form XObject** —
    /// `EditSession::delete_objects_in_form`, `OPERATOR_REQUESTS.md` O70.
    ///
    /// Beside [`Self::MoveLeavesInForm`] and for its reason: the indices are a
    /// different address space from `DeleteSelection`'s, and the variant is
    /// what says which. One command however many leaves, exactly as its
    /// page-level twin.
    DeleteLeavesInForm {
        /// The 0-based page.
        page: usize,
        /// Leaf indices, ascending and unique.
        leaves: Vec<usize>,
    },
    /// ★★ Move one **Bézier control point of an object inside a form
    /// XObject** — `EditSession::move_handle_in_form`. O70.
    ///
    /// Absolute, as [`Self::MoveHandle`] is and for its reason: the operand is
    /// a coordinate pair.
    MoveHandleInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The anchor the handle serves, object-scoped.
        node: usize,
        /// Which of the two controls.
        handle: pdfcer_core::vector::Handle,
        /// Where it lands, in PDF user space.
        to: pdfcer_core::vector::Point,
    },
    /// ★★ Displace one **subpath of an object inside a form XObject** —
    /// `EditSession::move_subpath_in_form`. `OPERATOR_REQUESTS.md` O70.
    MoveSubpathInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// ★★ Move one **anchor of an object inside a form XObject** to an
    /// absolute page-space point — `EditSession::move_node_in_form`.
    ///
    /// Absolute rather than a delta, exactly as [`Self::MoveNode`]: the operand
    /// being rewritten is a coordinate pair, and expressing the drag as *"where
    /// the point ends up"* is what makes a refusal leave the document
    /// untouched rather than half-moved.
    MoveNodeInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The anchor, object-scoped.
        node: usize,
        /// Where it lands, in PDF user space.
        to: pdfcer_core::vector::Point,
    },
    /// ★★ Move **several anchors** of an object inside a form XObject, as one
    /// command — `EditSession::move_nodes_in_form`.
    MoveNodesInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// Each anchor and where it lands, object-scoped indices.
        moves: Vec<(usize, pdfcer_core::vector::Point)>,
    },
    /// ★★★ Displace every selected object **inside a form XObject** by a
    /// page-space delta — `EditSession::move_objects_in_form`.
    ///
    /// `OPERATOR_REQUESTS.md` O70's second slice, 2026-09-01. Its own variant
    /// beside [`Self::MoveSelection`] because the indices are a different
    /// address space; `canvas::moving::MoveSubject::LeavesInForm` carries the
    /// argument.
    ///
    /// ★★ The coordinates are **page space**, exactly as the page-level verbs
    /// take, and that is the engine's contract rather than this shell's choice:
    /// `FormLeaf` reports geometry already mapped out of the form's own space,
    /// so a caller never has to know the placement matrix. The one thing that
    /// differs is which list the index is a position in.
    MoveLeavesInForm {
        /// The 0-based page.
        page: usize,
        /// Leaf indices, ascending and unique.
        leaves: Vec<usize>,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Displace **one subpath** of one path object by a page-space delta, as
    /// one undoable command — the Part rung's move verb.
    ///
    /// Raised only when the entered object decomposes into *subpaths*. A text
    /// object's Part rung is a show-operator run, which `move_subpath` has
    /// nothing to translate, so the canvas declines and traces rather than
    /// borrowing the Object rung's verb — the same rule, and the same reason,
    /// as `SelectionState::deletable_objects_on`'s rung guard.
    MoveSubpath {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// Drag **one anchor** of one path object to an absolute page-space point
    /// — the Node rung's move verb.
    ///
    /// # Why a destination and not a displacement
    ///
    /// Because that is `EditSession::move_node`'s signature, and the signature
    /// is right: the operand being rewritten *is* a coordinate pair, and the
    /// planner maps the destination through the object's CTM affine inverse in
    /// one step. Expressing it as a delta would make the planner reconstruct
    /// the point it was given, in a space the caller would then have had to
    /// name. The canvas computes it as *"where the anchor is now, plus the
    /// drag"*, and refuses the move outright if the decomposition can no
    /// longer say where the anchor is — see
    /// [`crate::canvas::moving::Refusal::NodeNotFound`].
    ///
    /// `node` is **object-scoped**: the space `vector::anchor_count` reports
    /// and `pdfcer node-move --node N` addresses. A second numbering would
    /// make the number pdfcer shows disagree with the number the operator can
    /// act on.
    MoveNode {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchor, object-scoped.
        node: usize,
        /// Where the anchor ends up, in PDF user space.
        to: Point,
    },
    /// ★★ **Move many of one object's nodes at once** — what a RESIZE is, in
    /// the absence of a scale verb.
    ///
    /// `pdfcer-core` has no verb that scales a vector object. Re-derived against
    /// its source on 2026-08-19 rather than taken from a note, because two
    /// other blockers this project recorded had quietly expired: `grep "pub fn
    /// .*scale"` over `edit.rs` returns one hit and it is `set_group_scale`, a
    /// ce-dimension calibration.
    ///
    /// So a resize is expressed as what it *is* — every node of the path moved
    /// to `anchor + (p - anchor) * (sx, sy)` — and `EditSession::move_nodes`
    /// takes a slice, which makes the whole gesture **one command and one undo
    /// entry**. A per-node loop would be neither: N undo entries for one drag,
    /// and each move planned against byte offsets the previous one invalidated.
    ///
    /// The geometry is computed by `crate::canvas::resizing`, which is pure and
    /// tested; this variant carries the result and nothing else. That is the
    /// funnel's rule and it matters more here than usual — an action that
    /// carried a grip and two factors would put the arithmetic in `apply`,
    /// where it could not be tested without a document.
    /// **Drag one Bézier handle** — move a control point of `node`, leaving the
    /// on-curve anchor itself exactly where it is.
    ///
    /// # ★ Why this is a separate verb and not "move a node that happens to be
    /// a control point"
    ///
    /// Because the two change different things about the path, and the engine
    /// draws the distinction in the type. `move_node` moves a point the curve
    /// passes **through**; this moves a point that governs the curve's
    /// **shape** and that the curve never touches. A single "move a point" verb
    /// would have to infer which the operator meant from what they grabbed,
    /// which is exactly the inference `pdfcer-core`'s own `Handle` type exists
    /// to remove.
    ///
    /// # ★★ The disclosure it owes, and it is not the obvious one
    ///
    /// `EditSession::move_handle` returns a list of sentences that is **empty
    /// unless a `v`/`y` segment had to be re-spelled as `c`**. Table 59 gives a
    /// cubic three spellings and two of them omit a control point by making it
    /// equal to a point the segment already has; a handle that must hold its
    /// own value cannot be expressed in those, so the operator's drag rewrites
    /// the operator.
    ///
    /// The curve draws **identically**. Nothing on the page changes. What
    /// changes is that the original bytes are gone and dragging back does not
    /// restore them — which is precisely the class of thing rule 4's surviving
    /// half is about: *an inference the operator cannot see still owes an
    /// off-canvas report*. The apply arm forwards those sentences to the
    /// disclosure channel for that reason, and for no other.
    MoveHandle {
        /// The 0-based page.
        page: usize,
        /// The object whose handle moves, by paint-order index.
        object: usize,
        /// The anchor the handle belongs to, object-scoped.
        node: usize,
        /// Which side of the anchor — arriving or leaving.
        ///
        /// The engine's own enum rather than a `bool`, because "incoming" and
        /// "outgoing" have no natural true/false and a caller that got the
        /// polarity backwards would drag the neighbouring curve instead, which
        /// looks like a coordinate bug rather than an inverted flag.
        handle: Handle,
        /// Where the control point ends up, in PDF user space.
        to: Point,
    },
    MoveNodes {
        /// The 0-based page.
        page: usize,
        /// The object whose nodes move, by paint-order index.
        object: usize,
        /// Every node's new position, object-scoped, in PDF user space.
        ///
        /// Absolute destinations rather than displacements, matching
        /// [`Self::MoveNode`] and for the same reason its docs give: the
        /// operand the planner rewrites is a coordinate pair, so "where the
        /// point ends up" is what lets it map one point through the object's
        /// CTM inverse instead of decomposing a translation.
        moves: Vec<(usize, Point)>,
    },
    /// ★★★ **Move, resize or rotate any objects at all** — `Pass 113.0`,
    /// 2026-08-20, and the verb this shell had been waiting for since the eight
    /// resize grips were drawn at S4.
    ///
    /// # What it closes
    ///
    /// The operator, three times, escalating:
    ///
    /// > *"there was no way to reposition, resize, or rotate it on the screen.
    /// > Can I please please please have that too?"*
    /// > *"can I please please please have the capability to move the text
    /// > after?"*
    ///
    /// [`Self::MoveNodes`] and `move_objects` cannot answer either. They rewrite
    /// numeric **operands**, and a text run and an image carry no coordinate
    /// operands at all — which is why `move_objects` is path-only by name.
    ///
    /// # ★★ The one thing that must not be got wrong: the matrix is PAGE space
    ///
    /// `cm` composes into the CTM in force at that point in the stream — the
    /// object's **user** space, not the page's. The engine emits
    /// `X = CTM × M × CTM⁻¹` per object, from *that object's own* captured CTM,
    /// so a selection spanning two local spaces gets two different `cm`
    /// operands for one gesture and both land where the operator pointed.
    ///
    /// **This shell passes page space and nothing else.** There is no
    /// local-space variant and no flag. Had the engine emitted a caller's matrix
    /// directly it would have been right only where an object's CTM happens to
    /// be the identity and **silently wrong at every scale or slant the producer
    /// left in force** — the object landing twice as far as the pointer went,
    /// with nothing erroring.
    ///
    /// # Why it takes a SLICE, and why that retired a refusal
    ///
    /// One gesture is one command and one undo entry — this project's standing
    /// rule. `canvas::resizing` used to decline a multi-object resize by name
    /// (*"pdfcer resizes one shape at a time"*), because `move_nodes` is
    /// per-object and N objects would have been N commands. That refusal is
    /// **gone**: the transform takes every index at once and scales them all
    /// about one pivot, which is what every drawing application does.
    ///
    /// # ★ What the engine collapses, and why the count is not ours
    ///
    /// `TransformOutcome::objects_transformed` is **not necessarily the index
    /// count**. Duplicate indices, and an object whose byte span is *contained
    /// inside* another selected object's, are collapsed — because wrapping a
    /// contained span twice applies the transform to those marks twice, which is
    /// the one arithmetic error here that renders as *almost* right.
    /// ★★★ **Paste page content** — `Pass 120.0`, 2026-08-20, and the
    /// operator's oldest open request.
    ///
    /// > *"can you get cut copy and paste working for objects I select on the
    /// > canvas?"* — asked in the first week and repeatedly since.
    ///
    /// # Why the clip travels as BYTES
    ///
    /// Because that is what the shell is holding: `canvas::clipboard` parks an
    /// `ObjectClip::to_bytes` payload in `egui::Memory` so that the same
    /// representation serves the in-process clipboard and the OS one. See
    /// `Clipped::Content` for the three reasons, and for why the third decides
    /// it.
    ///
    /// The deserialisation therefore happens here, in the apply arm, and its
    /// refusals are the engine's own — `ClipError::NotAClip` is checked **before
    /// any length prefix is read**, so an unrelated payload the OS clipboard
    /// hands back is refused with a sentence rather than with whatever a length
    /// prefix read out of the wrong bytes.
    ///
    /// # ★★ `at` is a PAGE-SPACE matrix, exactly as [`Self::TransformObjects`]
    ///
    /// `Matrix::IDENTITY` is paste-in-place, `translate` is paste-with-offset,
    /// and `Matrix::about` gives paste-scaled and paste-rotated from the same
    /// verb. That is why the request asked for a matrix rather than a
    /// displacement: a future *paste special* is already built.
    PasteObjects {
        /// The 0-based page to paste onto.
        page: usize,
        /// `ObjectClip::to_bytes` — magic-prefixed, versioned, bit-exact.
        clip: Vec<u8>,
        /// Where it lands, **in PAGE space**.
        at: Matrix,
    },
    TransformObjects {
        /// The 0-based page.
        page: usize,
        /// Which objects, by paint-order index. Every kind is accepted — path,
        /// text, image, form XObject, inline image, in any mixture.
        objects: Vec<usize>,
        /// The transform, **in PAGE space**. See the variant's docs.
        matrix: Matrix,
    },
}

/// **Apply one geometry verb**, as one undoable command.
///
/// Routed here from `super::apply` rather than living there, which is the shape
/// [`super::dimensions::apply`] already sets: the family module owns both the
/// vocabulary and what the vocabulary does. `super::apply` stays a routing
/// table.
///
/// Every arm goes through `super::apply::vector_edit` — the four-step protocol
/// (cancel the render worker, mutate through `Arc::get_mut`, bump the epoch,
/// drop the texture) whose whole reason for existing is that seven hand-written
/// copies would be seven chances to omit a step.
pub(super) fn apply(doc: &mut crate::app::state::OpenDoc, action: VectorAction) {
    // ★★★ `vector_edit_on_page`, not `vector_edit` — `OPERATOR_REQUESTS.md`
    // O74. Every verb in this module addresses paint-order indices **into one
    // page's content stream**, which is the module header's own opening claim
    // and the property that makes the whole file a subject rather than a
    // size-driven cut. That property is exactly the one `EditScope::Page`
    // requires: it is true of the VERB, on every document and every operand,
    // not an observation about a particular call.
    //
    // ★ So this is the strongest narrowing available anywhere in `actions`,
    // and it is the one the operator asked about — "when I make edits … all of
    // the page previews get re-rendered". A node drag on sheet 12 now leaves
    // the other thirty-five thumbnails alone.
    //
    // It stays bounded even if that reasoning is ever wrong: `pages::resync`
    // runs after the bump and raises `bump_all` whenever the page SET moved, so
    // a mistake here can be about which page's content changed and never about
    // which sheet an index names.
    use super::apply::vector_edit_on_page;
    match action {
        VectorAction::DeleteSelection { page, objects } => {
            if !objects.is_empty() {
                // ★★★ THE FOOTPRINT, TAKEN BEFORE THE COMMIT DESTROYS IT.
                //
                // `OPERATOR_REQUESTS.md` O63. A deleted object stays on screen
                // until the page redraws — one to two seconds on a dense
                // drawing — with no gesture in flight to explain the wait. What
                // the operator sees is a delete that did nothing, and the
                // natural response is to press Delete again, which deletes
                // something else.
                //
                // ★★ The order is load-bearing. `page_objects` is keyed on
                // `(page, edit_epoch)` and `vector_edit` bumps the epoch, so the
                // geometry is thrown away by the very edit it describes. Built
                // here, and the `Ref` dropped before `vector_edit` takes
                // `&mut doc`.
                let preview = doc
                    .page_objects()
                    .and_then(|provider| crate::canvas::shapes::erased(&provider, &objects));
                // ★★★ HELD BEFORE THE COMMIT, and getting this backwards is a
                // silent quarter-second bug.
                //
                // `hold_preview` stamps `edit_epoch` AS IT IS WHEN CALLED, and
                // `held_preview_to_draw` reads "the epoch has not moved" as
                // "the commit has not landed yet" — a state it allows for only
                // 250 ms, because past that it means the engine REFUSED.
                //
                // So a hold taken after the bump looks permanently un-committed
                // and expires in a quarter of a second, which on the drawings
                // this exists for is a fifth of the wait. Taken before, the
                // bump moves the epoch past it and the normal rule applies:
                // draw until the raster carries the edit.
                //
                // ⇒ And the refusal case comes out right for free. If
                // `delete_objects` fails the epoch never moves, the grace
                // expires, and the operator does not spend four seconds looking
                // at a hole where their object still is.
                if let Some(preview) = preview {
                    doc.hold_preview(preview);
                }
                vector_edit_on_page(doc, "delete-objects", page, objects.len(), |session| {
                    session.delete_objects(page, &objects)
                });
            }
        }
        VectorAction::MoveSelection {
            page,
            objects,
            dx,
            dy,
        } => {
            if !objects.is_empty() {
                vector_edit_on_page(doc, "move-objects", page, objects.len(), |session| {
                    session.move_objects(page, &objects, dx, dy)
                });
            }
        }
        VectorAction::DeleteLeavesInForm { page, leaves } => {
            if !leaves.is_empty() {
                // ★ No shell-side invalidation here either — see
                // `MoveLeavesInForm`'s arm for the counter that stood in both
                // places for four hours and the engine commit that removed the
                // need for it.
                vector_edit_on_page(
                    doc,
                    "delete-leaves-in-form",
                    page,
                    leaves.len(),
                    |session| {
                        session
                            .delete_objects_in_form(page, &leaves)
                            .map(|outcome| outcome.disclosures)
                    },
                );
            }
        }
        VectorAction::MoveHandleInForm {
            page,
            leaf,
            node,
            handle,
            to,
        } => {
            vector_edit_on_page(doc, "move-handle-in-form", page, 1, |session| {
                session
                    .move_handle_in_form(page, leaf, node, handle, to)
                    .map(|outcome| outcome.disclosures)
            });
        }
        VectorAction::MoveSubpathInForm {
            page,
            leaf,
            subpath,
            dx,
            dy,
        } => {
            vector_edit_on_page(doc, "move-subpath-in-form", page, 1, |session| {
                session
                    .move_subpath_in_form(page, leaf, subpath, dx, dy)
                    .map(|outcome| outcome.disclosures)
            });
        }
        VectorAction::MoveNodeInForm {
            page,
            leaf,
            node,
            to,
        } => {
            vector_edit_on_page(doc, "move-node-in-form", page, 1, |session| {
                session
                    .move_node_in_form(page, leaf, node, to)
                    .map(|outcome| outcome.disclosures)
            });
        }
        VectorAction::MoveNodesInForm { page, leaf, moves } => {
            // ★ The count is the number of ANCHORS, which is what the funnel's
            // trace reports as the operand size — one object, many points, and
            // the number a reader comparing a trace against a drag wants.
            let n = moves.len();
            vector_edit_on_page(doc, "move-nodes-in-form", page, n, |session| {
                session
                    .move_nodes_in_form(page, leaf, &moves)
                    .map(|outcome| outcome.disclosures)
            });
        }
        VectorAction::MoveLeavesInForm {
            page,
            leaves,
            dx,
            dy,
        } => {
            if !leaves.is_empty() {
                // ★★ **A shell-side invalidation counter stood here for four
                // hours**, on 2026-09-01, because the engine's content digest
                // did not move for a form-stream rewrite — measured in
                // `tests/page_generation_covers.rs` and filed rather than
                // absorbed.
                //
                // `pdfcer-core` `6e2b69e` folded the descended-form set into the
                // digest the same night, so the invalidation is computed on the
                // inside, where this shell said it belonged. The counter is
                // gone; nothing replaces it.
                vector_edit_on_page(doc, "move-leaves-in-form", page, leaves.len(), |session| {
                    session
                        .move_objects_in_form(page, &leaves, dx, dy)
                        .map(|outcome| outcome.disclosures)
                });
            }
        }
        VectorAction::MoveSubpath {
            page,
            object,
            subpath,
            dx,
            dy,
        } => {
            vector_edit_on_page(doc, "move-subpath", page, 1, |session| {
                session.move_subpath(page, object, subpath, dx, dy)
            });
        }
        VectorAction::MoveNode {
            page,
            object,
            node,
            to,
        } => {
            vector_edit_on_page(doc, "move-node", page, 1, |session| {
                session.move_node(page, object, node, to)
            });
        }
        // ★★ A resize, and it is `move_nodes` because there is no scale
        // verb — see `VectorAction::MoveNodes` and `crate::canvas::resizing`.
        //
        // ONE call with every node in it, deliberately: the slice is what
        // makes a whole resize one command and one undo entry, and a loop
        // over `move_node` would be neither.
        //
        // The count passed to `vector_edit` is the number of NODES, which
        // is what its trace line reports as the operand size. That is the
        // honest figure for this edit — one object, many points — and it is
        // the number a reader comparing a trace against a drag would want.
        // ★ A handle drag, and the only vector edit in this crate whose
        // RETURN VALUE is a disclosure rather than a count.
        //
        // `move_handle` answers with a list of sentences that is empty
        // unless a `v`/`y` segment had to be re-spelled as `c` — see the
        // variant's own docs. The curve draws identically either way, so
        // this is an inference the operator cannot see, which is exactly
        // the case rule 4 says still owes an off-canvas report.
        VectorAction::MoveHandle {
            page,
            object,
            node,
            handle,
            to,
        } => {
            // `said` is filled by the closure and read after it, because
            // `vector_edit` owns the borrow of the session and the note has
            // to be recorded against the epoch the edit produced — which
            // does not exist until `vector_edit` has returned.
            let mut said = Vec::new();
            vector_edit_on_page(doc, "move-handle", page, 1, |session| {
                let out = session.move_handle(page, object, node, handle, to);
                if let Ok(disclosures) = &out {
                    said.clone_from(disclosures);
                }
                out
            });
            for sentence in said {
                crate::app::actions::record_note(doc.edit_epoch, sentence);
            }
        }
        VectorAction::MoveNodes {
            page,
            object,
            moves,
        } => {
            let count = moves.len();
            vector_edit_on_page(doc, "move-nodes", page, count, |session| {
                session.move_nodes(page, object, &moves)
            });
        }
        // ★★★ Move / resize / rotate anything. One call, one command, one
        // undo entry, whatever the selection is made of.
        //
        // ★ `TransformOptions::default()` and no override, and both halves
        // of that are decisions the operator made himself before the
        // question reached him: *"make things work both ways as options.
        // default it to your best guess as to what would be normally
        // expected."* The engine shipped both, so the defaults are
        //
        //   * a **mixed selection transforms whole** rather than refusing —
        //     an operator who drags a grip round a picture and a box means
        //     both, and `RefuseHeterogeneous` exists for a caller that
        //     needs the other answer;
        //   * a **singular matrix refuses by name** rather than clamping —
        //     a clamp silently substitutes a shape nobody drew, and
        //     `SingularPolicy::Clamp` exists for a caller that wants it.
        //
        // A commit-on-release gesture makes exactly-zero scale nearly
        // unreachable anyway, and `resizing::is_usable` refuses before we
        // ever get here.
        //
        // ★★ `objects_transformed`, not `objects.len()`, is what the trace
        // reports — see the variant's docs for what the engine collapses and
        // why. A count taken from our own slice would be a number this
        // shell wished were true.
        // ★★★ PASTE. The operator's oldest open request, and the arm is short
        // because the engine's clip owns everything it needs.
        //
        // ★ The deserialisation is INSIDE the closure, so a payload that is not
        // a clip refuses through `vector_edit`'s own channel with the engine's
        // sentence — `ClipError::NotAClip`, checked before any length prefix is
        // read. A shell that unwrapped here would have to invent a sentence for
        // a case the engine already words.
        //
        // ★★ `resources_added` is on the trace at the engine's suggestion:
        // *"every paste adds fresh /Resources entries, so a shell that pastes
        // the same clip forty times and wonders why the file grew has the
        // answer in hand."* Not on the status row — it is a fact about the
        // file rather than about the page, and rule 4 asks for a disclosure in
        // terms of what the operator can see.
        VectorAction::PasteObjects { page, clip, at } => {
            let mut added = 0_u64;
            let mut pasted = 0_u64;
            vector_edit_on_page(doc, "paste-objects", page, clip.len(), |session| {
                let clip = pdfcer_core::vector::ObjectClip::from_bytes(&clip)?;
                session.paste_objects(page, &clip, at).map(|outcome| {
                    added = outcome.resources_added;
                    pasted = outcome.objects_pasted;
                    outcome.disclosures
                })
            });
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "paste-objects-applied page={page} pasted={pasted} \
                     resources_added={added} at=[{:.4} {:.4} {:.4} {:.4} {:.2} {:.2}]",
                    at.a, at.b, at.c, at.d, at.e, at.f,
                )
            });
        }
        VectorAction::TransformObjects {
            page,
            objects,
            matrix,
        } => {
            let mut transformed = 0_u64;
            vector_edit_on_page(doc, "transform-objects", page, objects.len(), |session| {
                session
                    .transform_objects(
                        page,
                        &objects,
                        matrix,
                        pdfcer_core::vector::TransformOptions::default(),
                    )
                    .map(|outcome| {
                        transformed = outcome.objects_transformed;
                        outcome.disclosures
                    })
            });
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                //
                // ★ It carries the MATRIX, and it has to. A line saying only
                // "a transform committed" would be identical for a build
                // that translated when it meant to scale, scaled about the
                // wrong pivot, or applied the transform in the object's
                // local space instead of the page's — which is the one error
                // here that lands the object at a plausible wrong distance
                // with nothing erroring. `resize-commit`'s own note makes
                // the same argument: a trace line must carry the number a
                // wrong build would get wrong.
                format!(
                    "transform-objects-applied page={page} asked={} \
                     transformed={transformed} m=[{:.4} {:.4} {:.4} {:.4} {:.2} {:.2}]",
                    objects.len(),
                    matrix.a,
                    matrix.b,
                    matrix.c,
                    matrix.d,
                    matrix.e,
                    matrix.f,
                )
            });
        }
    }
}
