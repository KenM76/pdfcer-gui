//! # `app::actions::annot` — the verbs whose subject is a whole annotation
//!
//! Move it, resize it, remove it, write the note on it. Split out of
//! [`super::action`] under **R2** on 2026-08-28, when `ResizeAnnotation` grew
//! the operator's Tool-row scale switches and took that file past 1,500 lines
//! for the fifth time. It grew the two note verbs the same evening, when
//! `pdfcer-core` `Pass 154.0` answered the blocker that had kept the Comments
//! panel read-only.
//!
//! ## ★★ Why THIS family, when the file's header still names markup
//!
//! [`super::action`]'s header pre-measured **markup** as the next sub-enum and
//! it is still the largest. The rule it states is *"the next family of variants
//! to **grow**"*, and today that was this one — the same reading that took the
//! text family out this morning. The markup measurement stands and is still the
//! answer the day markup grows.
//!
//! ## ★★★ What these all share, and it is not "they are annotations"
//!
//! **None of them takes a page index.** Every other authoring verb in this
//! crate does. The reason is a property of the engine's annotation verbs:
//! `move_annotation`, `resize_annotation`, `delete_annotation`,
//! `set_markup_note` and `clear_markup_note` all find their operand by
//! **stable object id**, so a page number would be a second
//! way of naming a thing that is already named — and one that goes wrong the
//! moment a page is reordered between the gesture and the queue draining.
//!
//! ⇒ That is why `Delete` carries a page and the others do not: its page is
//! for the **trace and the disclosure**, not for finding the annotation. The
//! asymmetry is real and is documented on the variant rather than smoothed
//! away, because smoothing it would mean adding a page to two verbs that must
//! not use one.
//!
//! ## ★ `CommitMarkup` and `PasteMarkup` are deliberately NOT here
//!
//! They **author** an annotation, which needs a page, a spec and a pen. These
//! three act on one that exists. Authoring and editing are different subjects
//! however much they share a noun, and a sub-enum drawn around the noun rather
//! than around the subject would be the larger of the two families and the less
//! useful one.

/// The verbs whose subject is a whole annotation that already exists.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnotAction {
    /// ★★★ **Move a markup annotation by a page-space delta**, as one undoable
    /// command.
    ///
    /// Raised by `crate::canvas::annotdrag` on the release of a drag, and by
    /// nothing else. **That module's header is the argument** for why this
    /// carries a delta rather than a new rectangle, and it is not repeated
    /// here: a `/Rect` names only the half of a move a renderer can see, and
    /// the absolute-coordinate geometry keys — which any *other* tool rebuilds
    /// an appearance from — are the half that would be silently left behind.
    ///
    /// ★ No page, for [`Self::DeleteAnnotation`]'s reason inverted: that one
    /// carries a page purely for its trace and its disclosure, and this one
    /// needs neither — `move_annotation` finds the annotation by id, and the
    /// disclosure it owes is about a pop-up rather than a sheet.
    Move {
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
        /// Horizontal displacement, PDF points.
        dx: f64,
        /// Vertical displacement, PDF points. **Positive is up** -- y increases
        /// upward in PDF user space (§8.3.2.3).
        dy: f64,
    },
    /// ★★★ **Scale a markup annotation about an anchor**, as one undoable
    /// command. `OPERATOR_REQUESTS.md` **O51**.
    ///
    /// Raised by `crate::canvas::resizing` on the release of a grip drag, and
    /// by nothing else. **Anchor plus FACTORS, not a target rectangle** — the
    /// shape this shell asked the engine for so it would match
    /// `transform_objects`, and the argument is at that call site.
    ///
    /// ★★ [`Self::uniform`] travels because the engine **asked for it by
    /// name**, and it is not a number the engine could derive equally well: it
    /// reports what the operator did with their hand — a Shift-constrained
    /// corner drag versus a free edge drag. Neither PDF nor SVG has a per-axis
    /// stroke width, so a non-uniform scale of a foreign appearance produces an
    /// anisotropic border by arithmetic, and that case is refused rather than
    /// silently distorted.
    Resize {
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
        /// The point that stays still, in PDF page space — the corner
        /// **opposite** the grip that was grabbed.
        anchor: (f64, f64),
        /// Horizontal scale factor.
        sx: f64,
        /// Vertical scale factor.
        sy: f64,
        /// Whether the two factors are equal. See the variant docs.
        uniform: bool,
        /// ★★ **The operator's Tool-row switches, CARRIED rather than read at
        /// apply time** — `OPERATOR_REQUESTS.md` O51.
        ///
        /// The same rule `CommitMarkup` follows and for its stated reason: a
        /// resize is raised by a gesture that completed frames before the queue
        /// drains, so a value read at apply time is a value that may have moved
        /// under it. `CommitTextAnnot` reads its pen live instead, and its own
        /// comment says why that is safe there — it is raised by a dialog the
        /// operator is sitting in, on the frame they press Accept.
        ///
        /// ★ Nobody can tick a checkbox during a drag, so the two would agree
        /// today. Carrying it is what keeps that an observation rather than a
        /// dependency.
        modifiers: crate::canvas::scaling::Modifiers,
    },
    /// ★★★ **Turn a markup annotation about a pivot**, as one undoable command.
    /// `Pass 155.0`.
    ///
    /// Raised by `crate::canvas::rotating` on the release of a rotate-handle
    /// drag, and by nothing else.
    ///
    /// # ★★★ Why there is no options type, unlike [`Self::Resize`]
    ///
    /// **Because a rotation is an isometry.** Every length is preserved,
    /// including the drawn stroke width — so the whole question
    /// [`Self::Resize::modifiers`] exists to answer (*does a line weight scale
    /// with the shape?*) has no counterpart here. There is nothing to ask, so
    /// nothing is asked, and no switch is offered on the Tool row for one.
    ///
    /// `pdfcer-core` put it in one sentence and it is the sentence that decides
    /// this variant's shape: *"if your grip UI offers rotate and resize
    /// together, **rotate needs no confirmation step and no distortion
    /// warning.** Resize does."*
    ///
    /// # ★★ A foreign appearance turns correctly, where it cannot be scaled
    ///
    /// [`Self::Resize`] has to refuse artwork pdfcer did not draw: §12.5.5's
    /// placement matrix scales it *after* stroking and no scalar `/BS /W`
    /// describes an anisotropic stroke. **Rotation has no such problem**, and
    /// the reason is in the standard rather than in an implementation choice —
    /// step (a) transforms the appearance `BBox` through its **own** `/Matrix`,
    /// so pdfcer composes the rotation into the matrix a producer already wrote.
    /// Nothing is redrawn and nobody's artwork is replaced. It works on a stamp
    /// Acrobat made.
    ///
    /// ⇒ Which is why the operator gets no confirmation, no warning and no
    /// second thought on this gesture, and gets all three on the resize.
    ///
    /// # ★ `pivot` in page space, and `degrees` anticlockwise
    ///
    /// The **same anchor-plus-scalar shape** [`Self::Move`] and [`Self::Resize`]
    /// already take, which the engine chose deliberately so this shell's grip
    /// code needs no third convention. `canvas::rotating` performs the single
    /// screen→page negation that gets it here; see that module's header for why
    /// it happens exactly once.
    ///
    /// ★ No page, for this module's stated reason: `rotate_annotation` finds
    /// its operand by stable object id.
    Rotate {
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
        /// The point that stays still, in PDF page space — the **centre** of
        /// the selection's box, which is what `Grip::Rotate::pivot` answers and
        /// what the ghost turned about.
        pivot: (f64, f64),
        /// Degrees **anticlockwise** in PDF user space. Any real angle; the
        /// engine does not quantise, and Shift's 15° snap is this shell's
        /// affordance rather than a limit of the verb.
        degrees: f64,
    },
    /// ★★★ **Turn a ce dimension about a pivot**, as one undoable command.
    /// `Pass 159.0`.
    ///
    /// Raised by `crate::canvas::rotating` on the release of a rotate-handle
    /// drag over a selected ce dimension, and by nothing else.
    ///
    /// # ★★★ A separate variant, because [`Self::Rotate`] REFUSES a dimension
    /// by name
    ///
    /// `rotate_annotation` returns `AnnotationMoveWrongVerb` for a ce dimension
    /// and points here, with its reason attached: *"a ce dimension's
    /// orientation is part of its measurement, so turning it must re-measure
    /// rather than spin a rectangle."* A dimension is a `/Line` with `/IT
    /// /LineDimension`; rotating it as an annotation would turn the `/Rect` and
    /// the baked `/AP` and leave the sidecar geometry — the thing the number is
    /// derived from — where it was.
    ///
    /// ⇒ Two variants rather than one with a kind flag, for exactly the reason
    /// `canvas::selection::annot::AnnotKind` is an enum: **a bool is a fact a
    /// caller may forget to read; a variant is one the compiler makes them
    /// handle.** The routing decision is made once, in `canvas::rotating`, over
    /// a `match` that cannot fall through.
    ///
    /// # ★★ The measured value cannot change, and the UI is built around that
    ///
    /// A rotation preserves every distance, so the number is identical either
    /// side of it **by construction** rather than because pdfcer holds it. The
    /// engine says so in as many words: *"if you show a live readout while
    /// dragging a rotate handle, it will simply not move — that is correct, not
    /// a stale binding."* There is therefore no before/after value on the
    /// outcome and none carried here.
    ///
    /// # ★★★ What DOES change, and is disclosed
    ///
    /// A `Linear` dimension locked to horizontal or vertical **cannot stay
    /// locked through a rotation**, and the engine relaxes it to *aligned*
    /// rather than refusing the rotation or keeping a constraint that
    /// contradicts the drawn line. `crate::app::actions::annots::rotate_dimension`
    /// words that on the status row, because — the engine's argument, adopted —
    /// *"an operator whose dimension silently stopped being axis-locked will
    /// find out later and blame something else."*
    ///
    /// # ★ It carries a `DimensionId`, not an `ObjId`
    ///
    /// The one place this sub-enum departs from its own header's rule, and
    /// deliberately: `rotate_dimension` takes the **sidecar record's** id, and
    /// the annotation's object id maps to it only by a scan (`record.annot` is
    /// stored one way round). Resolving it in the gesture rather than at apply
    /// time is what lets the drag **decline in words** when the sidecar carries
    /// no record for the selection, instead of raising an action that the
    /// engine would then refuse into silence.
    RotateDimension {
        /// The dimension's sidecar record id, resolved by
        /// `canvas::dimdrag::selected` at the moment of the gesture.
        dimension: pdfcer_core::dimension::DimensionId,
        /// The annotation's object id — **for the trace only**, so a failed run
        /// can be tied back to the thing the operator had selected. Not used to
        /// find the dimension; see the variant docs.
        annot: pdfcer_core::object::ObjId,
        /// The point that stays still, in PDF page space.
        pivot: (f64, f64),
        /// Degrees anticlockwise in PDF user space.
        degrees: f64,
    },
    Delete {
        /// The page it is on — for the trace and the disclosure, not for the
        /// verb, which finds the annotation by id wherever it lives. A reply
        /// may sit on a different page from the comment it replies to, so a
        /// page-scoped delete would miss it.
        page: usize,
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
    },
    /// ★★★ **Write the note on an annotation that already exists** —
    /// `/Contents`, and conditionally `/T` and `/M` — as one undoable command.
    ///
    /// Raised by the Comments panel's editor and by nothing else. It is the
    /// second half of the interaction every reviewer UI converges on, which
    /// `pdfcer-core` states in its own words on `set_markup_note`:
    ///
    /// > draw the shape → **it is selected** → type the comment in the panel
    /// > beside the page.
    ///
    /// The first half has worked since Phase 6. The second half had **no verb
    /// behind it at all** until `Pass 154.0`, which is why this shell's
    /// Comments panel was read-only for the life of the project: `MarkupOptions`
    /// is an author-time structure, and a cloud, a highlight and an arrow are
    /// authored on mouse-release from geometry alone, with no text-entry moment
    /// to hang a note off.
    ///
    /// ★ **No page.** Like [`Self::Move`] and [`Self::Resize`], the engine finds
    /// its operand by stable object id — see this module's header for why a page
    /// index would be a second, weaker name for a thing already named.
    SetNote {
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
        /// The words, exactly as the operator typed them. Empty is permitted
        /// and is **not** the same as [`Self::ClearNote`]: an empty comment is
        /// a comment, and `pdfcer-core` models the two as separate verbs for
        /// that reason.
        text: String,
        /// ★★★ **Whether the annotation already carries a `/T`, and its byline
        /// must therefore be left alone.**
        ///
        /// This is the flag that decides whether correcting somebody else's
        /// typo re-attributes their comment to the operator. `pdfcer-core`
        /// leaves an omitted key untouched — deliberately, and its reply to
        /// this shell called it *"the easiest way to get this wrong"* — so
        /// `true` means *send no author at all* rather than *send the existing
        /// one back*.
        ///
        /// ★ It travels on the action rather than being read at apply time
        /// because the panel drew the row and knows what the byline said; an
        /// apply-time re-read would be a second walk of the annotation for a
        /// fact the raising surface already had, and two reads of one fact is
        /// how they come to disagree.
        keep_author: bool,
    },
    /// **Remove an annotation's note entirely** — `/Contents`, `/T` and `/M` —
    /// as one undoable command, leaving the markup itself on the page.
    ///
    /// A separate variant rather than [`Self::SetNote`] with an empty string,
    /// because `pdfcer-core` models them as separate verbs and its reason is the
    /// operator's: *"an empty comment is a comment, and a reviewer deleting
    /// their remark is not the same as leaving a blank one."*
    ///
    /// ★ It is **not** a delete. The shape stays, its geometry is untouched, and
    /// `Ctrl+Z` restores the words. The wording of the control and of its
    /// disclosure both say so, because a canvas cannot: a shape with a note and
    /// the same shape without one are the same picture.
    ClearNote {
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
    },
    // =======================================================================
    // The NODES of a markup shape — the operator's report of 2026-09-05
    // =======================================================================
    //
    // > *"I also can't edit or delete nodes of a markup shape once it is
    // > drawn."*
    //
    // ★★★ **Three variants and not one with a `VertexEdit` inside it**, and the
    // reason is `canvas::dimdrag::VertexIntent`'s: the three reach **three
    // different engine wrappers**, and the one thing that must never happen on
    // this canvas is a gesture aimed at the wrong verb. A single variant
    // carrying `pdfcer_core::edit::VertexEdit` would compile, would be shorter,
    // and would put the engine's own enum on this shell's action bus — where a
    // future `VertexEdit` variant would arrive as a silent `..` match rather
    // than as a compile error.
    //
    // ★ All three take **no page**, for [`Self::Move`]'s reason: the engine
    // finds its operand by stable object id, and a page index would be a
    // second, weaker name for a thing already named.
    /// **Move one node of a markup shape** by a page-space delta, as one
    /// undoable command. `EditSession::move_annotation_vertex`.
    ///
    /// Raised by `crate::canvas::annotnodes` on the release of a node drag, and
    /// by nothing else.
    ///
    /// ★★ A **delta**, not a destination, and it is the same choice
    /// `annotdrag::Move` made for the same reason one level up: the engine's
    /// verb takes `(index, dx, dy)`, and a shell that sent an absolute point
    /// would have to subtract the old one — which means reading the geometry a
    /// second time, at apply time, and getting a different answer if anything
    /// moved in between.
    MoveNode {
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
        /// Which node. For a `/Polygon` or `/PolyLine` this indexes
        /// `/Vertices`; for a `/Line` it is 0 (start) or 1 (end).
        index: usize,
        /// Horizontal displacement, PDF points.
        dx: f64,
        /// Vertical displacement, PDF points. **Positive is up** — y increases
        /// upward in PDF user space (§8.3.2.3).
        dy: f64,
    },
    /// **Add a node immediately after `after`**, at `at`.
    /// `EditSession::insert_annotation_vertex`.
    ///
    /// The new node's index is `after + 1`. There is deliberately no
    /// "insert before the first" spelling — the engine refuses `after >= count`
    /// and says to rotate the polygon's start instead, which is what every
    /// other tool does as well.
    ///
    /// ★ `at` is **already snapped**. `annotnodes` resolves the destination
    /// through the same `measure::snap_point` the preview drew a marker at, so
    /// the point committed and the point shown are one value rather than two
    /// derivations of one intention.
    InsertNode {
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
        /// The node the new one goes after.
        after: usize,
        /// Where it goes, in page space (PDF user space, y-up).
        at: pdfcer_core::vector::Point,
    },
    /// **Take a node away.** `EditSession::remove_annotation_vertex`.
    ///
    /// ★ No destination, because a removal has none. The drop point of the
    /// gesture that raised this is ignored on purpose, and `annotnodes` draws
    /// no snap marker for it — a marker would point at a node that is about to
    /// stop existing.
    ///
    /// The engine refuses below the shape's floor (`/Polygon` keeps three,
    /// `/PolyLine` keeps two) and this shell asks it **before** the gesture
    /// previews anything, so a release that reaches this variant is one the
    /// preflight already allowed.
    RemoveNode {
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
        /// Which node to remove.
        index: usize,
    },
    /// **A node edit did not happen, and the operator is owed the sentence.**
    ///
    /// ★★★ Raised on the release frame of a gesture whose preflight refused,
    /// and by `annotnodes::explain_unreshapable` when the Points tool is armed
    /// over a shape that shows no anchors. Carries no id and no index: the
    /// operator is looking at the shape, and what they need is the reason.
    ///
    /// # Why a gesture with a preflight still needs this
    ///
    /// Because the preflight is what makes it the **only** report that exists.
    /// `annotnodes` asks `reshape_annotation_preview` before it draws anything,
    /// so a refused edit is never previewed and never raised as an action — the
    /// engine is never asked to refuse, no funnel is entered, and no
    /// `EditRefused` is recorded. Without this the operator drags a corner of a
    /// triangle out of the shape, releases, and the triangle is still a
    /// triangle with nothing anywhere saying why. **That silence is precisely
    /// the shape of the report this whole feature answers.**
    ///
    /// ★ Handed inward as an action rather than recorded at the gesture,
    /// because the decline store is `pub(super)` inside `crate::app` and the
    /// canvas is outside that boundary. `DimensionAction::DeclineVertexEdit`
    /// carries the same argument for the ce-dimension twin.
    DeclineNodeEdit {
        /// Which sentence. The mapping from `EditError` lives at
        /// `canvas::annotnodes::refusal_for`, so the engine's vocabulary stays
        /// out of the string catalog.
        why: crate::text::markup::NodeEditRefusal,
    },
}
