//! `VectorAction` — one change to the marks on a page.

use pdfcer_core::vector::{Handle, Matrix, Point};

/// One change to the marks on a page.
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
    /// `pdfcer_gui::canvas::selection::SelectionState::object_indices_on`,
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
    /// Raised by `pdfcer_gui::canvas::moving::drag` when a move drag that began
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
    /// # Why this does NOT invalidate the selection, and Delete does
    ///
    /// Because `move_*` **does not renumber**, and that is measured rather
    /// than assumed — `crates/pdfcer-core/tests/object_identity_across_edits.rs`
    /// decomposes, edits, and decomposes again. A move rewrites operands
    /// *inside* existing operators, so no operator is added or removed and the
    /// second decomposition yields the same objects at the same indices. The
    /// `delete_*` family excises byte **spans** and therefore does renumber,
    /// which is why `pdfcer_core::vector::remap_index_after_delete` exists and
    /// why nothing like it is needed here. See
    /// `pdfcer_gui::canvas::moving`'s header for the full table.
    ///
    /// # Units
    ///
    /// `dx`/`dy` are **PDF user-space** points, Y-**up** — produced by
    /// `pdfcer_gui::canvas::moving::page_delta`, which is the one place a
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
    /// Move several page objects, **each by its own delta**, as one undo
    /// step — what Align and Distribute commit.
    ///
    /// Units as [`Self::MoveSelection`]: PDF user-space points, Y up. Indices
    /// are paint-order positions on `page`, unique.
    MoveEach {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// `(index, dx, dy)` per object; zero moves are dropped before this.
        moves: Vec<(usize, f64, f64)>,
        /// The gesture, for the trace: `align`, `distribute`, `rearrange`.
        gesture: &'static str,
    },
    /// Transform several page objects, **each by its own PAGE-space
    /// matrix**, as one undo step, all-or-nothing — what Circular arrange
    /// with *Rotate objects* commits.
    TransformEach {
        /// The 0-based page the indices are positions on.
        page: usize,
        /// `(index, matrix)` per object, indices unique.
        transforms: Vec<(usize, Matrix)>,
        /// The gesture, for the trace.
        gesture: &'static str,
    },
    /// Delete every selected object **inside a form XObject** —
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
    /// **Remove ONE subpath of one path object** — `EditSession::delete_
    /// subpath`, Pass 25.2, and the Part rung's delete verb.
    ///
    /// # What it closes
    ///
    ///
    /// The engine's own reason for the verb is the operator's file: *"one
    /// stroked path with 1194 subpaths covering a whole isometric view"*, on
    /// which `delete_object` can only remove the entire view. *"Delete this
    /// line"* is what he means, and this is that operation.
    ///
    /// # Deleting the only subpath deletes the object, and that is the verb's
    /// rule rather than this shell's
    ///
    /// A painting operator with no path left is not a smaller object; it is
    /// meaningless. So a one-subpath path object vanishes, which is both
    /// correct and exactly what the operator asked for — they entered the
    /// object, found it had one line in it, and deleted that line.
    ///
    /// # Disclosures
    ///
    /// **Always empty**, measured against the locked engine rather than assumed:
    /// both arms of `plan_delete_subpath` return `disclosures: Vec::new()`.
    /// The arm below still routes them, because the funnel does that for every
    /// verb and a hand-written exception here would be the thing that stops
    /// being true when the planner grows a re-spelling path.
    DeleteSubpath {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The subpath, in decomposition order — the order
        /// `hit_test_subpaths` returns, so a picked line goes straight here.
        subpath: usize,
    },
    /// **Remove ONE label off a sheet that holds all of them in one text
    /// object** — `EditSession::delete_text_run`, `Pass 32.0`.
    ///
    /// # The defect this closes, in the engine's own measurement
    ///
    /// > *"on the operator's drawing **one text object holds all 237 dimension
    /// > labels**, so deleting 'a label' deleted every one of them."*
    ///
    ///
    /// Those 237 are **pdf dimensions** (R8b Rule 15): page content pdfcer
    /// reads and must not silently alter. A **ce dimension** is one pdfcer
    /// authors, lives in `super::dimensions`, and has nothing to do with this
    /// variant.
    ///
    /// # The refusal that is asked BEFORE the press, and where
    ///
    /// §9.4.2: a following run with no positioning operator of its own starts
    /// wherever this one ends, so excising this one **slides it**. The engine
    /// refuses with `DeleteWouldMoveNextRun`, and
    /// `crate::canvas::deleting` asks the identical question ahead of the
    /// press through `ObjectModelProvider::text_line_delete_would_move_next` —
    /// R83 — so the operator gets the remedy (*delete the later label first*)
    /// instead of a cause-less decline. This variant is therefore never raised
    /// for a run the guard would refuse; if one arrives anyway the engine
    /// still refuses it and the funnel still declines in words.
    ///
    /// # Disclosures
    ///
    /// **Always empty** — `plan_delete_text_run` returns `Vec::new()` on both
    /// of its arms. Routed anyway, for [`Self::DeleteSubpath`]'s reason.
    DeleteTextLine {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by paint-order index.
        object: usize,
        /// The visual line, numbered as
        /// `ObjectModelProvider::text_line_count` counts — **not** a
        /// show-operator index. The apply arm translates it into the line's
        /// run range and calls `delete_text_run` once per run, **descending**,
        /// because `plan_delete_text_run` excises `TextRun::bytes` and every
        /// later run in the same object shifts when an earlier one goes.
        line: usize,
    },
    /// **Remove ONE anchor of one path object** — `EditSession::delete_
    /// node`, Pass 36.1, and the Node rung's delete verb.
    ///
    /// Its twins [`Self::MoveNode`] and [`Self::MoveNodes`] have been wired
    /// since Pass 28.0, so on a CAD export the operator could nudge one point
    /// of a polyline and could not remove it.
    ///
    /// ⚠ **This is not the markup-annotation vertex verb.** That family edits
    /// an annotation's `/Vertices` through `reshape_annotation` and its
    /// helpers in `super::annots` are confusingly called `move_node` and
    /// `remove_node`. They share nothing with this but a name — different
    /// address space, different engine verb, different undo command — and the
    /// collision deferred this gap by an evening once already.
    ///
    /// # THE DISCLOSURE THIS ONE OWES, and it is the whole reason the arm
    /// is not one line
    ///
    /// `delete_node` returns a disclosure list that is **non-empty when
    /// deleting the point discarded a curve**:
    ///
    /// > *"The curve that ran into this point was removed along with it, so the
    /// > shape now goes straight from the point before to the point after."*
    ///
    /// That is a shape change the operator **cannot reverse by re-adding a
    /// point** — the two control points are gone — and the engine's own doc
    /// says rule 4 forbids letting them find it out from a diff: *"the caller
    /// must surface these."*
    ///
    /// The surfacing is `super::funnel::vector_edit_on_page`'s, not this
    /// arm's, and that is the point: the funnel records **every** verb's
    /// disclosure list to the status bar's row, stamped with the epoch the edit
    /// produced. So returning the list from the closure *is* surfacing it, and
    /// a hand-written `record_note` beside it would be a second mechanism for
    /// the same sentence — the one that later forgets to retire itself.
    ///
    /// The one refusal worth knowing about, because it is the commonest:
    /// `NodeDeleteWouldEmptySubpath`, when the line has only two points left.
    /// Left to the engine, where it is judged against the bytes.
    DeleteNode {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by paint-order index.
        object: usize,
        /// The anchor, **object-scoped** — the numbering
        /// [`Self::MoveNode`] takes, `vector::anchor_count` reports and
        /// `pdfcer node-move --node N` addresses. A second numbering would make
        /// the number pdfcer shows disagree with the number the operator can
        /// act on.
        node: usize,
    },
    /// [`Self::DeleteSubpath`] for a path painted **inside a form XObject** —
    /// `EditSession::delete_subpath_in_form`. Deleting inside a shared form
    /// removes the subpath from every page the form is drawn on.
    DeleteSubpathInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by leaf index.
        leaf: usize,
        /// The subpath, in decomposition order.
        subpath: usize,
    },
    /// [`Self::DeleteTextLine`] for a text object painted **inside a form
    /// XObject** — `EditSession::delete_text_run_in_form`, once per show
    /// operator of the line, descending, folded into one undo entry.
    DeleteTextLineInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by leaf index.
        leaf: usize,
        /// The visual line, numbered as
        /// `ObjectModelProvider::text_line_count_of` counts.
        line: usize,
    },
    /// [`Self::DeleteNode`] for a path painted **inside a form XObject** —
    /// `EditSession::delete_node_in_form`. Its curve-discard disclosure is
    /// surfaced by the funnel, as the page twin's is.
    DeleteNodeInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by leaf index.
        leaf: usize,
        /// The anchor, object-scoped.
        node: usize,
    },
    /// Move one **Bézier control point of an object inside a form
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
    /// Displace one **subpath of an object inside a form XObject** —
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
    /// Move one **anchor of an object inside a form XObject** to an
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
    /// Move **several anchors** of an object inside a form XObject, as one
    /// command — `EditSession::move_nodes_in_form`.
    MoveNodesInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// Each anchor and where it lands, object-scoped indices.
        moves: Vec<(usize, pdfcer_core::vector::Point)>,
    },
    /// Displace every selected object **inside a form XObject** by a
    /// page-space delta — `EditSession::move_objects_in_form`.
    ///
    ///
    /// The coordinates are **page space**, exactly as the page-level verbs
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
    /// **Displace ONE LINE of a text object** —
    /// `EditSession::move_text_run`, the Part rung's move verb for text, and
    /// `OPERATOR_REQUESTS.md` O188's move half.
    ///
    /// # What the operator asked for, and why it took a month
    ///
    ///
    /// Those labels are **pdf dimensions** (R8b Rule 15) — page content
    /// a CAD exporter wrote. This moves them; it does not re-measure them, and
    /// they have nothing to do with the **ce dimensions** pdfcer authors.
    ///
    /// # The refusal that is asked BEFORE the press, and where
    ///
    /// 9.4.2 again, and the mirror image of [`Self::DeleteTextLine`]'s: a run
    /// with no positioning operator of its own starts wherever the previous one
    /// ended, so there is no operand to rewrite; and a run whose SUCCESSOR is
    /// in that state cannot move without dragging the successor along.
    /// `crate::canvas::moving::eligible` asks
    /// `ObjectModelProvider::text_line_move_refusal_of` before a ghost is drawn,
    /// and **that is the engine's own guard rather than a copy of it** —
    /// `pdfcer_core::vector::edit::text_run_move_refusal`, the function
    /// `plan_move_text_run` runs first. Compare the note on
    /// [`Self::DeleteTextLine`], whose pre-check IS a hand-rolled copy because
    /// the delete side has no exported twin yet.
    ///
    /// # Disclosures — and this one is NOT always empty
    ///
    /// The planner rewrites `Tm` or `Td` operands where it can. Where the run
    /// was placed by `TD`, `T*`, `'`, `"` or by nothing at all it **inserts a
    /// `Td`**, and discloses that it did: the page looks identical, the move is
    /// exact, and dragging back by the same amount will not restore the
    /// original bytes. Rule 4 in its purest form — an inference the operator
    /// cannot see, so it is reported off-canvas and nothing is drawn
    /// differently. The funnel records them; this variant needs no code for it.
    MoveTextLine {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by paint-order index.
        object: usize,
        /// The visual line, in the numbering
        /// `ObjectModelProvider::text_line_count` produces and
        /// [`Self::DeleteTextLine`] already uses. **Nothing renumbers**: the
        /// move family rewrites operands in place, so the selection survives
        /// the drag naming the same line. The apply arm hands the line's runs
        /// to `EditSession::move_text_runs` as one set.
        line: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// **Displace one line of a text object INSIDE a form XObject** —
    /// `EditSession::move_text_runs_in_form`.
    ///
    /// **This is the variant O188 is actually about.** On the operator's
    /// SolidWorks sets the title block *is* a form XObject, drawn once per
    /// sheet; the labels he wants to nudge live inside it. A page-scoped verb
    /// alone would have answered his request everywhere except where he asked
    /// it, which the engine said in as many words when it shipped the pair
    /// together.
    ///
    /// ⚠ **One call changes every sheet the form is drawn on**, because the
    /// form's stream is shared. `FormSurgeryOutcome::invocations` and `::pages`
    /// are the measured pair that says how many, and the engine folds the reach
    /// sentence into `disclosures` when the count is above one — so routing
    /// the disclosures through the funnel, as every `*_in_form` arm here does,
    /// IS how the operator is told. There is no second mechanism and there must
    /// not be one.
    MoveTextLineInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by **leaf** index — a different address
        /// space from [`Self::MoveTextLine`]'s `object`, which is why it is a
        /// separate variant rather than a flag.
        leaf: usize,
        /// The visual line, numbered as
        /// `ObjectModelProvider::text_line_count_of` counts for that leaf.
        line: usize,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// **Displace SEVERAL chunks of one text object by one drag** —
    /// `EditSession::move_text_runs` over every run of every named line: one
    /// command, one press of Undo.
    ///
    /// A set is planned whole, so a run positioned relative to its
    /// predecessor moves with it when both are in the set.
    ///
    /// # The refusal is asked of the whole set, before the ghost
    ///
    /// `crate::canvas::moving::run_move` asks the engine's set guard,
    /// `text_run_move_refusal_of_set`, and refuses the drag whole if it
    /// declines, so this arm cannot be reached holding a set the planner will
    /// decline.
    /// Moving the movable ones and leaving the rest would read as a rendering
    /// fault rather than as a refusal.
    MoveTextLines {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by paint-order index.
        object: usize,
        /// The visual lines, ascending and unique, never fewer than two.
        lines: Vec<usize>,
        /// Horizontal displacement, PDF user-space points.
        dx: f64,
        /// Vertical displacement, PDF user-space points (Y is up).
        dy: f64,
    },
    /// [`Self::MoveTextLines`] for a text object **inside a form XObject**.
    ///
    /// ⚠ One call changes every sheet the form is drawn on, for
    /// [`Self::MoveTextLineInForm`]'s reason: the form's stream is shared, and
    /// the engine's own reach sentence is what tells the operator so.
    MoveTextLinesInForm {
        /// The 0-based page.
        page: usize,
        /// The enclosing text object, by **leaf** index.
        leaf: usize,
        /// The visual lines, ascending and unique, never fewer than two.
        lines: Vec<usize>,
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
    /// `pdfcer_gui::canvas::moving::Refusal::NodeNotFound`.
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
    /// **Move many of one object's nodes at once** — what a RESIZE is, in
    /// the absence of a scale verb.
    ///
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
    /// # Why this is a separate verb and not "move a node that happens to be
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
    /// # The disclosure it owes, and it is not the obvious one
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
    /// **Move, resize or rotate any objects at all** — `Pass 113.0`,
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
    /// # The one thing that must not be got wrong: the matrix is PAGE space
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
    /// # What the engine collapses, and why the count is not ours
    ///
    ///
    /// > *"can you get cut copy and paste working for objects I select on the
    /// > canvas?"* — asked in the first week and repeatedly since.
    ///
    /// # Why the clip travels as BYTES
    ///
    /// Because that is what the shell is holding: `canvas::clipboard` parks an
    /// `ObjectClip::to_bytes` payload in `egui::Memory` so that the same
    /// representation serves the in-process clipboard and the OS one. See
    /// `Clipped::Selection` for the three reasons, and for why the third decides
    /// it.
    ///
    /// The deserialisation therefore happens here, in the apply arm, and its
    /// refusals are the engine's own — `ClipError::NotAClip` is checked **before
    /// any length prefix is read**, so an unrelated payload the OS clipboard
    /// hands back is refused with a sentence rather than with whatever a length
    /// prefix read out of the wrong bytes.
    ///
    /// # `at` is a PAGE-SPACE matrix, exactly as [`Self::TransformObjects`]
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
    /// Join consecutive text runs of one page text object into one run
    /// (`EditSession::merge_text_runs`, default `MergeOptions`). One undo entry,
    /// `CommandKind::MergeTextRuns`; later runs renumber down by `runs.len() - 1`.
    MergeTextRuns {
        /// The 0-based page.
        page: usize,
        /// The page object index of the text object.
        object: usize,
        /// Ascending, consecutive run indices; at least two.
        runs: Vec<usize>,
    },
    /// Cut one page text object into one object per inferred line
    /// (`EditSession::text_object_split_plan` with `SplitGranularity::Line`,
    /// then `EditSession::split_text_object`). One undo entry,
    /// `CommandKind::SplitTextObject`; later objects renumber up by the cut count.
    SplitTextLines {
        /// The 0-based page.
        page: usize,
        /// The page object index of the text object.
        object: usize,
    },
    /// Draw page `source_page` of the PDF at `file` into `page`'s content,
    /// filling `rect` (`EditSession::place_page_content`). One undo entry,
    /// `CommandKind::PlacePageContent`; the placed form is appended last in
    /// paint order and selected.
    PlacePageContent {
        /// The 0-based page to draw on.
        page: usize,
        /// Where the source's crop box lands, in PDF user space.
        rect: pdfcer_core::page_tree::Rect,
        /// The source PDF, read when the action applies.
        file: std::path::PathBuf,
        /// The 0-based page of `file` to draw.
        source_page: usize,
    },
    /// Move page objects in paint order (`EditSession::restack_objects`).
    /// One undo entry, `CommandKind::RestackObjects`; the engine answers each
    /// object's new index and the selection follows them.
    Restack {
        /// The 0-based page.
        page: usize,
        /// Page object indices, ascending and unique.
        objects: Vec<usize>,
        /// Which end of the stack.
        to: super::ArrangeTo,
    },
    /// **Format ▸ Nodes on the selected nodes of one path** — one undo entry;
    /// a node the engine refuses is skipped and reported.
    NodeShape {
        /// The 0-based page.
        page: usize,
        /// The path.
        host: super::NodeHost,
        /// Object-scoped node indices, ascending and unique.
        nodes: Vec<usize>,
        /// What each node gets.
        shape: super::NodeShape,
    },
    /// **Format ▸ Replace image** — draw `image` in place of the image
    /// object at `object` (`EditSession::replace_image`, `ImageFit::Contain`).
    /// One undo entry, `CommandKind::ReplaceImage`; the object keeps its
    /// index, so the selection stands.
    ReplaceImage {
        /// The 0-based page.
        page: usize,
        /// The page object index of the image.
        object: usize,
        /// The imported raster, shared rather than copied through the queue.
        image: std::sync::Arc<pdfcer_core::image_import::ImportedImage>,
    },
}
