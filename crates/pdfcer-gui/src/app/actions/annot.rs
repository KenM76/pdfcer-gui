//! # `app::actions::annot` — the verbs whose subject is a whole annotation
//!
//! Move it, resize it, remove it, write the note on it, **answer it, and
//! record whether its window opens**. Split out of
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
    /// ★★★ **Change where a markup annotation sits in its page's paint
    /// order** — Bring to front, Bring forward, Send backward, Send to back.
    ///
    /// Raised by `crate::app::dispatch::markup` from the ribbon's Arrange group
    /// and from nothing else.
    ///
    /// # ★★ It carries an INTENT, not an order — and that is the whole design
    ///
    /// `EditSession::reorder_annotations` takes the page's whole `/Annots`
    /// permutation, and the obvious shape for this variant is therefore
    /// `order: Vec<ObjId>`, computed by the dispatcher which has the document in
    /// front of it. **It would be wrong**, for the reason this action bus exists:
    /// an action is raised on one frame and drained on another, behind every
    /// action queued ahead of it. A permutation computed at the press describes
    /// the array as it was *before* whatever ran in between, and the engine
    /// refuses a stale one by name (`AnnotsNotAPermutation`) rather than applying
    /// it approximately.
    ///
    /// ⇒ So the array is read at apply time, in
    /// [`crate::app::actions::reorder::arrange`]. This is the same rule
    /// [`Self::Move`] follows by carrying a **delta** rather than a rectangle,
    /// and its reason restated: a value resolved at the press is a value that may
    /// have moved under you.
    ///
    /// # ★ It takes a page, unlike its neighbours, and the header's rule holds
    ///
    /// This module's header says *"none of them takes a page index"*, because
    /// every annotation verb finds its operand by stable object id. That is still
    /// true of the operand — the mark is named by [`Self::Arrange::id`] — and the
    /// page here is not an address for it. It is `reorder_annotations`' **own**
    /// operand: the array being permuted belongs to a page, and the engine takes
    /// `page_index` because that is what it is reordering. Exactly like
    /// [`Self::Delete`]'s page, the asymmetry is documented rather than smoothed
    /// away.
    Arrange {
        /// The page whose `/Annots` is permuted.
        page: usize,
        /// The mark to move within it, by stable object id.
        id: pdfcer_core::object::ObjId,
        /// Which end, or which single step.
        to: crate::app::actions::reorder::ArrangeTo,
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
    /// ★★★ **Answer a comment** — a new `/Text` annotation carrying `/IRT`
    /// and `/RT /R`, as one undoable command. `EditSession::add_reply`,
    /// `pdfcer-core` `Pass 253.0` (§12.5.6.2, Table 170).
    ///
    /// Raised by the Comments panel's editor when its draft is aimed at a
    /// reply rather than at the row's own `/Contents`, and by nothing else.
    ///
    /// # ★★ Why this is a separate variant and not [`Self::SetNote`] with a
    /// flag
    ///
    /// Because it reaches a **different engine verb with a different
    /// outcome**. `set_markup_note` edits a dictionary that already exists;
    /// `add_reply` *creates an annotation*, places it, gives it a `/Popup` and
    /// returns a `ReplyAdded` naming an object number that did not exist
    /// before the call. Folding the two behind one variant with a `reply: bool`
    /// would put the shell one boolean away from writing a reviewer's answer
    /// over the top of the comment they were answering — which is the single
    /// worst thing this surface can do — and would arrive as a silently
    /// mishandled field rather than as a compile error.
    ///
    /// ★ **No page**, like every neighbour in this module: `add_reply` finds
    /// the parent by stable object id and places the reply on *the parent's
    /// own page*, resolved by its own page-tree walk
    /// (`pdfcer-core/src/edit.rs:26981-27002`). A page carried from the panel
    /// would be a second, weaker name for a sheet the engine has already
    /// found, and one that goes stale if pages are reordered between the press
    /// and the queue draining.
    ///
    /// ★★ **No `keep_author`**, unlike [`Self::SetNote`], and the asymmetry is
    /// the whole point of the pair. `SetNote` may be editing *somebody else's*
    /// comment, so it must be able to say *"write no `/T` at all"*. A reply is
    /// a **new annotation this operator is authoring**, so its byline is
    /// always theirs — there is no prior `/T` to preserve, and a flag that
    /// could suppress it would only ever produce an unsigned reply nobody
    /// asked for.
    Reply {
        /// The comment being answered, by stable object id — the reply's
        /// `/IRT`.
        ///
        /// It may itself be a reply. §12.5.6.2 permits a reply to a reply and
        /// `add_reply` refuses only a reply to *itself*, which is not a
        /// thread; see `crate::panels::comments`' threading-depth paragraph
        /// for what the panel does with the depth it thereby allows.
        parent: pdfcer_core::object::ObjId,
        /// The words, exactly as the operator typed them — **never blank**.
        ///
        /// # ★★★ The guard is THIS SHELL's, not the engine's, and the
        /// difference was measured
        ///
        /// `add_reply`'s own doc comment lists *"[`EditError::MarkupNoteEmpty`]
        /// and the note's own validation"* among its errors. **That variant
        /// does not exist.** Measured 2026-09-06 against the pinned engine at
        /// `d2ea5de`: `MarkupNoteEmpty` occurs exactly once in the whole crate
        /// and the occurrence is that doc line; `MarkupNote::validate`
        /// (`edit.rs:4731`) checks only the `/M` date's §7.9.4 shape and
        /// returns `Ok(())` for an empty `/Contents`. So an empty reply is
        /// **authored**, not refused.
        ///
        /// ⇒ Which makes the blank case the shell's to decide, and it is
        /// decided against. A `/Text` sticky with no `/Contents` is an
        /// ordinary and useful thing — it is what an operator has just placed
        /// and is about to type into. A **reply** with none is an annotation
        /// permanently added to somebody's thread that says nothing, and there
        /// is no later gesture that gives it words from this panel. So the
        /// control is not drawn until the box holds something —
        /// `panels::comments::editor::reply_is_postable`, R83.
        ///
        /// ★ Recorded here rather than only in the guard because a future
        /// reader who checks the engine's error list will find the variant
        /// named there and conclude the shell-side guard is redundant. It is
        /// not: it is the only thing standing between a stray press and a
        /// wordless comment in the file.
        text: String,
    },
    /// ★★★ **Open or close a comment's pop-up window IN THE FILE** — `/Open`
    /// on the annotation and on its `/Popup` companion, as one undoable
    /// command. `EditSession::set_annotation_open`, `pdfcer-core`
    /// `Pass 253.3` (§12.5.6.4 Table 172, §12.5.6.14 Table 183).
    ///
    /// Raised by the canvas pop-up's *Open by default* control, and by nothing
    /// else. **It is emphatically not raised by opening or closing a pop-up on
    /// screen** — see below, because that distinction is the entire design.
    ///
    /// # ★★★ Why a reading gesture must not reach this variant
    ///
    /// Clicking a sticky note to read it, and pressing the window's ✕ when
    /// done, are how an operator *reads a marked-up drawing*. They happen
    /// dozens of times in a review. Every one of them is a
    /// `CommandKind::SetAnnotationOpen` if it is wired to this — which means a
    /// reviewer who opened six comments and closed them again has six undo
    /// entries standing between `Ctrl+Z` and the last thing they actually
    /// changed, and a document that reports itself modified after a session in
    /// which they altered nothing.
    ///
    /// ⇒ So `crate::canvas::notepopup::open` keeps on owning what is *showing*
    /// — per-document interface state, no undo, no dirty flag — and this
    /// variant exists for the separate, deliberate act of saying **"and record
    /// that in the file, for whoever opens it next."** One press, one undo
    /// entry, and the entry is honest because the operator asked for it.
    ///
    /// `crate::canvas::markup::swatch`'s header makes the same argument from
    /// the other side and reaches the opposite conclusion for its own case:
    /// setting a pen colour *"has no undo, it raises no `Action`"* because it
    /// touches no document. The rule both obey is that **the undo log records
    /// changes to the document, and only what the operator meant as one.**
    ///
    /// ★ **No page**, for this module header's reason: `set_annotation_open`
    /// finds its operand by stable object id.
    SetOpen {
        /// The annotation whose window state is being written — the note, not
        /// its `/Popup`. The engine writes both halves of the pair itself and
        /// reports which it reached; see `AnnotationOpenChange`.
        id: pdfcer_core::object::ObjId,
        /// The state to record. `true` is §12.5.6.4's *"shall initially be
        /// displayed open"*.
        open: bool,
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
    /// ★★★ **Restyle a text-BEARING annotation** — a sticky note's icon and
    /// colour, a stamp's colour. `EditSession::set_text_annot_style`
    /// (`pdfcer-core` `edit.rs:27124`), raised by
    /// `crate::panels::properties::markup::textannot` and by nothing else.
    ///
    /// # ★★★ Why this is a SECOND style verb and not a field on
    /// `Action::SetMarkupStyle`
    ///
    /// Because they are two verbs over two spec families, and the engine says
    /// so in the type's own doc (`edit.rs:15969`): `MarkupStyle` reaches its
    /// annotation through `annot_author::spec_from_dict`, *"whose arms are the
    /// geometric family and the four text markups. **There is no `/Text`
    /// arm**"* — and that function's own `UnsupportedSubtype` names `Text`
    /// explicitly. `TextAnnotStyle` reaches it through
    /// `annot_author::text_spec_from_dict` instead.
    ///
    /// ⇒ So the two are not a split anyone chose for tidiness, and merging
    /// them into one action would put the routing decision in an apply arm
    /// where a wrong turn is a runtime refusal. Keeping them apart makes the
    /// panel's guard a `match` the compiler checks —
    /// `panels::properties::markup::textannot::Reach` — which is the whole
    /// mechanism by which a `/Stamp` cannot be sent to `set_markup_style`
    /// again.
    ///
    /// # ★ Here rather than beside `Action::SetMarkupStyle`, and that is this
    /// # enum's own rule
    ///
    /// This module's header: *"none of them takes a page to locate one"*.
    /// `set_text_annot_style(annot_id, style)` takes an `ObjId` and nothing
    /// else, which is the property that defines this family.
    /// `Action::SetMarkupStyle` carries a page, and does so for the funnel's
    /// undo label rather than to find the mark — an asymmetry documented
    /// there and not copied here.
    ///
    /// # ⚠ Two constraints the caller cannot express, both by design
    ///
    /// * **`icon` is `/Text` only.** Any other subtype is refused by name with
    ///   `EditError::StylePropertyNotApplicable` rather than silently ignored
    ///   — a `/Stamp`'s face comes from its own `/Name` vocabulary and a
    ///   `/FreeText` has no icon at all.
    /// * **`color` cannot CLEAR**, unlike `MarkupStyle::stroke`.
    ///   `TextAnnotSpec`'s three variants each carry a **required** `Color`, so
    ///   *"no colour"* is not a state the authoring type can express, and a
    ///   Clear would have to invent a fallback. The panel therefore draws no
    ///   Clear button beside the swatch — absent, not greyed (R9).
    SetTextAnnotStyle {
        /// The annotation, by stable object id.
        id: pdfcer_core::object::ObjId,
        /// What to change. Every field `None` but the one control that moved,
        /// which is `MarkupStyle`'s contract restated for this family — the
        /// engine's own words are *"an override set, not a replacement"*.
        style: pdfcer_core::edit::TextAnnotStyle,
    },
}
