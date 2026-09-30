//! # `editactions` — the verbs whose subject is text style, a ce dimension, a redaction or an attachment

use crate::subactions::AttachmentRef;
use pdfcer_core::dimension::{
    DimStandard, DimensionId, DimensionKind, GroupId, GroupStyle, NumberFormat, ScaleState,
    StyleOverrides, Unit,
};
use pdfcer_core::edit::GroupDeletion;
use pdfcer_core::text_edit::{FormatRequest, NewFill, StyleSynthesis};

/// One property of a text run, and the value the operator chose for it.
#[derive(Debug, Clone, PartialEq)]
pub enum StyleChange {
    /// A new size in points, changing the `Tf` operand.
    Size(f64),
    /// A new fill colour, stored in the space the operator chose.
    Fill(NewFill),
    /// A new face, named by `/Resources /Font` key or by `/BaseFont`.
    ///
    /// A `String` rather than a `FontSelector` because `pdfcer_gui::app::actions::Action`
    /// derives `PartialEq` and `FontSelector` is `#[non_exhaustive]`. The
    /// conversion is one line at the point of use.
    Face(String),
    /// A face taken from a font file and embedded as a subset of the
    /// characters the restyled runs carry (`FormatRequest::embedded_font`).
    /// `None` asks the operator which file; the dispatcher asks, and restyles
    /// with `Some` only when a file was picked.
    FaceFile(Option<std::path::PathBuf>),
    /// Weight and slant, as two independent flags.
    ///
    /// Deliberately **not** named `Synthetic`, because whether it ends up
    /// synthetic is the engine's decision and not the operator's: the variant
    /// maps to `set_style` and the engine walks its four-rung ladder. The
    /// operator asked for bold; how bold is achieved on this page is a fact
    /// they are told afterwards.
    Weight {
        /// Bold wanted.
        bold: bool,
        /// Italic wanted.
        italic: bool,
    },
    /// A text render mode (`Tr`, 0..=7), as the engine's `TextRenderMode`
    /// byte. `3` is invisible (the OCR layer's mode).
    RenderMode(u8),
    /// Fit one run to a page-point width (`EditSession::set_text_run_width`).
    ///
    /// Addressed by paint-order object and run, not by the extraction runs the
    /// other variants use, so `apply` routes it before `restyle` and ignores
    /// `runs`.
    RunWidth {
        /// Paint-order index of the text object on the page.
        object: usize,
        /// Index into that object's `runs`.
        run: usize,
        /// The width wanted, in PDF points.
        width: f64,
    },
}
impl StyleChange {
    /// Stamp this change onto a request that is already pinned.
    pub fn stamp(&self, req: FormatRequest) -> FormatRequest {
        match self {
            Self::Size(points) => req.size(*points),
            Self::Fill(fill) => req.fill(fill.clone()),
            Self::Face(selector) => req.font(pdfcer_core::text_edit::FontSelector::new(selector)),
            // `style`, **not** `synthetic`, and the one word is the whole
            // feature. `set_synthetic` means *"thicken the strokes"* and is
            // gated; `set_style` means *"make this bold"* and walks the
            // ladder. What the operator sees is a real `Helvetica-Bold`
            // instead of a stroked `Helvetica` on every page that carries no
            // bold resource, which is most CAD title blocks. Module header.
            //
            // ⚠ The two are **mutually exclusive per axis**: combining
            // `set_style` with `set_font`, or with an overlapping
            // `set_synthetic`, is `FormatError::Unsupported`. Nothing else in
            // this table sets either, and `Face` is its own variant, so one
            // press is one verb.
            Self::Weight { bold, italic } => req.style(StyleSynthesis::new(*bold, *italic)),
            Self::RenderMode(mode) => req.render_mode(*mode),
            // Never stamped: `apply` routes it to `runwidth` first, and a
            // font file's plan is built once per gesture by `textstyle`,
            // which attaches it after this call.
            Self::RunWidth { .. } | Self::FaceFile(_) => req,
        }
    }

    /// The trace word for this change, for `PDFCER_DIAG`.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Size(_) => "size",
            Self::Fill(_) => "fill",
            Self::Face(_) => "face",
            Self::FaceFile(_) => "face-file",
            Self::Weight { .. } => "weight",
            Self::RenderMode(_) => "render-mode",
            Self::RunWidth { .. } => "run-width",
        }
    }
}
/// Everything the ce-dimension feature can ask the document for.
///
/// # Why this is a sub-enum rather than that many more variants on `pdfcer_gui::app::actions::Action`
///
/// `pdfcer_gui::app::actions::Action`: super::Action
///
/// 1. **They share a routing rule that the flat enum could not express.** Some
///    are group-scoped and must invalidate the whole strip; the rest are
///    annotation-scoped and must not. As flat variants that rule lives in one
///    arm per verb and is re-derived in each; as a family it lives once, in
///    [`Self::regenerates_the_whole_group`], where a new verb has to pick a
///    side to compile.
/// 2. **R2.** `super`'s enum is already near the file-size ceiling and these
///    variants carry the documentation this project asks for. The alternative
///    to a seam is thinner prose, which the file-size gate's own header names
///    as the incentive it refuses to create.
/// 3. **It matches what the engine did.** `pdfcer-core` groups these verbs
///    behind one module (`pdfcer_core::dimension`) and one sidecar, for the
///    same reason.
///
/// What it is **not** is a general policy of grouping actions by feature.
/// `CommitMarkup` and `DeleteAnnotation` stay flat because they share no
/// routing rule — one authors a gesture's product, the other removes an object
/// — and grouping them would be filing rather than structure.
#[derive(Debug, Clone, PartialEq)]
pub enum DimensionAction {
    /// **Author a ce dimension on the page** — the release of a completed
    /// measure pick.
    ///
    /// Raised by `pdfcer_gui::canvas::measure` when a pick machine returns a
    /// `DimensionKind`, which for the linear tool is the **third** click (what,
    /// to what, and where it sits) and for the others is the pick that first
    /// makes the geometry knowable.
    ///
    /// # Why the geometry arrives whole rather than as points
    ///
    /// `DimensionKind` is `pdfcer-core`'s own type and it is carried across
    /// unchanged, which is the property the salvage's equivalence tests exist
    /// to protect: the value built here is **byte-for-byte the one `pdfcer
    /// dimension-add` builds** from the same picks, so a ce dimension authored
    /// on the canvas and one authored from the command line are the same bytes
    /// in the file. Decomposing it into coordinates here and rebuilding it in
    /// the apply arm would put a second constructor in the path and quietly end
    /// that guarantee.
    ///
    /// This is also why the variant carries no colour, width or standard: those
    /// live on the **group**, which is why `group` is the other field.
    Commit {
        /// Page index the ce dimension is placed on.
        page: usize,
        /// The authoring group it joins, which is what carries its scale,
        /// number format, drafting standard and style tier.
        group: GroupId,
        /// The immutable geometry, straight from the pick machine.
        kind: DimensionKind,
        /// **What the gesture inferred, in the operator's words** — carried
        /// with the edit rather than recorded when the gesture ended.
        ///
        /// Empty for the linear and circular tools, whose output is what the
        /// operator pointed at. Non-empty for the **two-line** tool, which
        /// classifies: it may read two lines as parallel *because the operator
        /// asked*, overriding a real measured angle, and it may find an apex
        /// that exists only if the lines are extended. `pdfcer-core` requires
        /// both inferences to be said aloud (`03-capabilities.md` §1.5
        /// obligation 4).
        ///
        /// # Why it travels HERE and not through `record_note`
        ///
        /// Because the apply phase runs **after** the frame that raised this,
        /// and `vector_edit` writes its own disclosure list to the same slot on
        /// success. A note recorded at gesture time would be wiped by the
        /// commit it is about — silently, and only on the successful path,
        /// which is the path it exists for.
        ///
        /// The funnel already has the mechanism: `vector_edit`'s closure
        /// returns the disclosure list. This field is what lets a gesture put
        /// something in it.
        ///
        /// A refusal is the opposite case and correctly uses `record_note`:
        /// nothing is committed, so no apply phase will overwrite it.
        disclosures: Vec<String>,
    },

    /// **Calibrate a dimension group** — say what its numbers mean.
    ///
    /// Raised by `crate::dialogs::scale` and by nothing else.
    ///
    /// # Why this is an `Action` and not a call
    ///
    /// `EditSession::set_group_scale` **re-propagates every member's baked
    /// appearance stream**. A ce dimension's label is drawn into its `/AP`, so
    /// changing the scale rewrites every member of the group — which may be
    /// dozens of annotations across several pages.
    ///
    /// That makes it a document edit with an undo step, and the funnel's whole
    /// purpose is that such an edit is ordered against every other and appears
    /// **once** in the command log. One `Ctrl+Z` undoes a recalibration,
    /// whatever it touched. That is the group model's own promise — *a group
    /// exists so its members agree* — and a dialog issuing one call per member
    /// would break it in the most annoying way available: an undo stack the
    /// operator has to press forty times.
    ///
    /// # Why it carries no page
    ///
    /// Every annotation-scoped variant here names one. A group is
    /// **document-scoped by construction**: its members may be on any page, and
    /// the sidecar that records it is not a page property. Adding a page here
    /// would be a field `apply` had to ignore, which is how a reader comes to
    /// believe a recalibration is page-local.
    SetGroupScale {
        /// The group to recalibrate.
        group: GroupId,
        /// The tri-state scale to store — always `Calibrated` from the scale
        /// dialog, because a back-calculated scale is by definition neither
        /// "1:1" nor "never set".
        scale: ScaleState,
        /// The number format: the display unit, and how its fractional part is
        /// written.
        format: NumberFormat,
    },

    /// **Create a dimension group** — a second answer to *"what scale is
    /// this drawn at?"* in one document.
    ///
    /// Raised by `crate::dialogs::dimension_groups` and by nothing else.
    ///
    /// # Why a document needs more than one, and why this is not a preference
    ///
    /// A group is the carrier of every property its members share: the scale,
    /// the display unit, the number format, the drafting standard, the layer
    /// they can be hidden on, and the middle tier of the style cascade. A
    /// drawing with a plan at 1:50 and a detail at 1:5 on the same sheet needs
    /// two of them, and there is no arrangement of one group that expresses it
    /// — [`Self::SetGroupScale`] recalibrates *every* member, which is exactly
    /// the promise a group makes.
    ///
    /// So this is a document edit, undoable like any other, and not a setting.
    /// It writes a record into the `/PieceInfo` sidecar; a document saved after
    /// it carries the group whether or not anything has joined it yet.
    ///
    /// # Why the name travels as an owned `String`
    ///
    /// Because the action outlives the frame that raised it — the funnel's
    /// standing property — and the text field it came from is redrawn, and may
    /// be closed, before the queue drains. Borrowing the dialog's buffer would
    /// tie an `Action` to a widget's lifetime, which is the coupling the funnel
    /// exists to remove.
    AddGroup {
        /// The operator's name for it.
        ///
        /// Trimmed and non-empty by the time it gets here;
        /// `crate::dialogs::dimension_groups` declines to raise the action
        /// otherwise rather than letting the engine store a blank one, because
        /// a blank row in a group picker is indistinguishable from a broken
        /// one.
        name: String,
        /// The display unit the group starts in.
        ///
        /// Carried because `EditSession::add_dimension_group` takes it, and
        /// because `Unit::default_format` derives the whole starting
        /// [`NumberFormat`] from it — a millimetre group starts in decimals and
        /// an inch group in eighths, which is what a drafter expects without
        /// being asked twice.
        unit: Unit,
    },

    /// **Rename a dimension group.**
    ///
    /// Raised by `crate::dialogs::dimension_groups`.
    ///
    /// # Why this is an edit at all, when nothing is redrawn
    ///
    /// Because the name is **in the document** — a field of the group record in
    /// the `/PieceInfo` sidecar — and it travels with the file. It is not a
    /// label this shell keeps for its own convenience.
    ///
    /// It is the one group verb that regenerates **nothing**: no member's
    /// appearance depends on what its group is called, so
    /// [`Self::regenerates_the_whole_group`] answers `false` and no raster is
    /// invalidated. That is a decision rather than an omission — it is the only
    /// group verb of which it is true, and a reader checking why the list has a
    /// hole in it should find the reason here.
    ///
    /// # Why the verb has to exist
    ///
    /// `Group::name` is a `pub String` on a snapshot, so without a session verb
    /// a mistyped group name would be permanent for the life of the document.
    /// It is the only field of `Group` reached solely through this verb: the
    /// display unit rides on `set_group_scale`, which takes a whole
    /// `NumberFormat`, and the rest have verbs of their own.
    RenameGroup {
        /// The group to rename.
        group: GroupId,
        /// The new name. Trimmed and non-empty by the time it gets here, for
        /// [`Self::AddGroup`]'s reason: a blank row in a group picker is
        /// indistinguishable from a broken one.
        name: String,
    },

    /// **Say something other than the measurement**, without changing it.
    ///
    /// `EditSession::set_dimension_label`. Raised by
    /// `panels::properties::dimension::label_row` and by nothing else.
    ///
    /// # It does not destroy the measurement
    ///
    /// The override is a **caption**. The measured value stays underneath, so
    /// `None` restores it with **no re-measurement** — the number that comes
    /// back is the one that was always there rather than a fresh calculation
    /// that might round differently.
    ///
    /// Which is why `label` is an `Option` and why the panel has no Clear
    /// button: clearing the box *is* `None`, and a second control that meant
    /// the same thing would be a second way to reach one state.
    SetLabel {
        /// Which ce dimension.
        dimension: pdfcer_core::dimension::DimensionId,
        /// The caption, or `None` to show the measurement again.
        label: Option<String>,
    },

    /// **Delete a dimension group**, answering the members question.
    ///
    /// Raised by `crate::dialogs::dimension_groups`.
    ///
    /// # The policy is the whole design, and it is the ORPHAN question again
    ///
    /// A group with members cannot simply be removed, and the engine's answer
    /// is the one it gave for `insert_pages`' orphaned widgets — **report and
    /// refuse to guess**:
    ///
    /// | policy | what happens |
    /// |---|---|
    /// | [`GroupDeletion::Refuse`] | `EditError::DimensionGroupNotEmpty { id, members }` if it is populated. The **default** |
    /// | `GroupDeletion::Reassign(dest)` | the members move to `dest` first, **re-measured** against its scale and format, then the group goes |
    ///
    /// The member count is in the error because *"this group is not empty"* and
    /// *"this group holds forty dimensions"* prompt different decisions from an
    /// operator — and only a surface can put that question in front of them.
    ///
    /// **There is deliberately no delete-the-members policy.** Deleting a ce
    /// dimension also removes its annotation from the page's `/Annots`, so
    /// doing it inside this verb would be a second implementation of
    /// `delete_dimension`'s removal; and calling `delete_dimension` in a loop
    /// would produce one undo entry **per member**, so undoing a group deletion
    /// would take one press per member and could stop halfway with the group
    /// already gone. Factoring `delete_dimension`'s core out belongs in the
    /// engine, as a request — not as a workaround written here.
    ///
    /// # A refusal validates before mutating, and the dialog relies on it
    ///
    /// A rejected deletion leaves the model byte-identical, so the verb may be
    /// called speculatively to populate a confirmation dialog. The dialog does
    /// not — it reads `member_count` from the model it is already holding,
    /// which costs nothing and needs no round trip — but the guarantee is what
    /// makes a Delete press safe to offer at all rather than gated behind a
    /// count the surface might have got wrong.
    DeleteGroup {
        /// The group to remove.
        group: GroupId,
        /// What to do about its members.
        policy: GroupDeletion,
    },

    /// **Move a placed ce dimension to another group.**
    ///
    /// Raised by `crate::panels::properties::dimension`.
    ///
    /// # This is NOT a field assignment, and the surface has to know that
    ///
    /// The single most important fact about this verb, and the engine spent a
    /// section of its reply on it. A ce dimension's label is **derived from its
    /// group** — the scale it is measured at, the precision and unit it is
    /// formatted with, the standard it is drawn to. So the verb re-measures and
    /// regenerates the baked `/AP`, `/Rect`, `/Contents`, `/Measure` and `/L`,
    /// and **the number on the page changes**:
    ///
    /// ```text
    /// before  "70.6 mm"   (a 1:1 millimetre group)
    /// after   "2.00 m"     (a 1 cm-per-point metre group)
    /// ```
    ///
    /// Same geometry. Different group. Different printed value, correctly.
    ///
    /// Two consequences for this shell, and both are honoured:
    ///
    /// 1. **It is disclosed before the operator commits**, because a dimension
    ///    that silently changes what it reads is rule 4's sneaky case with a
    ///    number attached. `crate::text::panels::dimension` carries the
    ///    sentence.
    /// 2. **Nothing reaches past this verb into `DimensionModel::dimension_mut`.**
    ///    The engine's own first test for it asserted only that `d.group` had
    ///    changed and undo put it back — and passed against an implementation
    ///    that writes the field and does nothing else, which is exactly the
    ///    wrong verb. That is the failure a shortcut here would ship.
    ///
    /// # Blast radius
    ///
    /// **One annotation**, so [`Self::regenerates_the_whole_group`] answers
    /// `false`: neither the source group's remaining members nor the
    /// destination's existing ones are touched. Only the mover is redrawn.
    SetDimensionGroup {
        /// The ce dimension to move.
        dimension: DimensionId,
        /// Where it goes. Carrying its scale, unit, number format, drafting
        /// standard, layer and style tier — which is why the label changes.
        group: GroupId,
    },

    /// **Move one vertex of a perimeter ce dimension**, re-measuring it.
    ///
    /// Raised by `crate::canvas::dimdrag::drag_vertex` on the release of a
    /// corner drag. The operator's ask: *"I want to be able to edit the
    /// endpoints of the lines to adjust the shape."*
    ///
    /// # This one CHANGES THE NUMBER
    ///
    /// [`Self::Place`] writes fields the value function does not read, so no
    /// label drag can alter what a dimension prints. `SetDimensionGroup`
    /// re-measures under a different scale. This moves a corner of the measured
    /// shape itself, which is what makes it the ce-dimension verb that
    /// deliberately changes what a ce dimension measures.
    ///
    /// So it owes a disclosure the others do not, and the engine hands over the
    /// material for it — `VertexOutcome` carries `previous_label` beside
    /// `label`, because **the old value cannot be reconstructed afterwards**:
    /// the geometry it came from is gone. A status line reading `12.40 m →
    /// 13.85 m` is a disclosure; one reading `13.85 m` is just the number
    /// already visible on the page.
    ///
    /// # Blast radius
    ///
    /// **One annotation**, redrawn with its new shape and its new label, so
    /// [`Self::regenerates_the_whole_group`] answers `false`.
    MoveVertex {
        /// The perimeter to reshape.
        dimension: DimensionId,
        /// Which vertex, by index into its points.
        index: usize,
        /// How far it moves, page space, points.
        dx: f64,
        /// See [`Self::MoveVertex::dx`].
        dy: f64,
    },

    /// **Add a corner to a perimeter ce dimension**, answering half of the
    /// operator's report:
    ///
    /// > *"I also can't edit or delete nodes of a markup shape once it is
    /// > drawn."*
    ///
    /// Raised by `crate::canvas::dimdrag::count_edit` on the release of a
    /// Ctrl-drag from a corner handle with the Points tool armed, and by
    /// nothing else.
    ///
    /// # `after` is a SEGMENT, not a position in a list
    ///
    /// `EditSession::insert_dimension_vertex(dimension, after, at)` puts the
    /// new corner **between** `after` and the one following it, which is why
    /// the engine's own doc comment describes the gesture as *"what a
    /// right-click on a segment offers"*: `after` names that segment by its
    /// first corner.
    ///
    /// `after == len - 1` is meaningful rather than out of range, and both
    /// readings are correct: on a **closed** perimeter it names the closing
    /// segment back to corner 0, which is drawn and measured like any other and
    /// must therefore be able to take a point; on an **open** path the same
    /// call extends the path past its end.
    ///
    /// # This RE-MEASURES, like [`Self::MoveVertex`] and unlike [`Self::Place`]
    ///
    /// A new corner changes the shape, so it changes the printed number. The
    /// same disclosure obligation applies and the same engine material
    /// discharges it — `VertexOutcome` carries `previous_label` beside `label`
    /// because the old value cannot be reconstructed once the geometry that
    /// produced it is gone — with the **corner count** added, because that is
    /// the fact this verb exists to change and the one a mis-aimed gesture
    /// would get wrong invisibly.
    ///
    /// # Blast radius
    ///
    /// **One annotation**, redrawn with its new shape and its new label. Its
    /// baked `/AP` and the sidecar catalog are rewritten together inside one
    /// engine `Command`, so this is one Ctrl+Z (`drag-moves` D4).
    /// [`Self::regenerates_the_whole_group`] answers `false`.
    InsertVertex {
        /// The perimeter to reshape.
        dimension: DimensionId,
        /// The first corner of the segment the new one is dropped onto.
        after: usize,
        /// Where the new corner goes, page space, points — the snapped
        /// position the preview showed, so what commits is what was on screen.
        at: pdfcer_core::vector::Point,
    },

    /// **Take a corner off a perimeter ce dimension** — the other half of *"or
    /// delete nodes"*.
    ///
    /// Raised by `crate::canvas::dimdrag::count_edit` on the release of a
    /// Ctrl+Shift-drag from a corner handle with the Points tool armed, and by
    /// nothing else.
    ///
    /// # The refusal is preflighted, so this variant never carries one
    ///
    /// `EditSession::remove_dimension_vertex` refuses rather than leaving a
    /// degenerate record — an open path keeps two corners, a closed one keeps
    /// three. The rule is to grey the affordance from
    /// `EditSession::vertex_edit_preview` rather than catch the error, because
    /// a verb with no preflight makes the UI find out by pressing. This shell
    /// has no menu item to grey, so it applies the same rule to the gesture:
    /// `dimdrag` asks `vertex_edit_preview` on every frame, the preview shows
    /// the shape unchanged when the answer is no, and **this action is never
    /// raised**.
    ///
    /// So an arm here that mapped `EditError` to a sentence would be dead code
    /// describing a state the caller guarantees cannot arrive. The refusal's
    /// sentence lives where the refusal is detected —
    /// `app::status::decline::record_vertex_edit_refused`.
    ///
    /// # Blast radius
    ///
    /// **One annotation**, exactly as [`Self::InsertVertex`]. One engine
    /// command, one undo entry.
    RemoveVertex {
        /// The perimeter to reshape.
        dimension: DimensionId,
        /// Which corner goes, by index into its points.
        index: usize,
    },

    /// **Say, in words, why a corner could not be added or taken away.**
    ///
    /// Raised by `crate::canvas::dimdrag::count_edit` on the release frame of a
    /// count-editing drag whose preflight refused. It edits nothing and is not
    /// an edit — no funnel, no epoch, no undo entry — and that is the whole
    /// point of it.
    ///
    /// # Why a refusal travels as an ACTION rather than being recorded on
    /// the spot
    ///
    /// Because of a module boundary that is deliberate. `app::status::decline`
    /// is `pub(super)` inside `crate::app`, with its reason written on the
    /// declaration: *"a decline is written by the one dispatcher and read by
    /// the one bar."* The canvas is outside that boundary and cannot reach the
    /// store, so a gesture that needs to decline has to hand the fact inward.
    ///
    /// `pdfcer_gui::app::actions::Action::DeclineOnCanvas` is the standing
    /// precedent for exactly this — an action whose entire body is one
    /// `decline::record_*` call. This one rides on `DimensionAction` rather
    /// than joining it at the top level for a reason that is unglamorous and
    /// real: `appaction.rs` sits against R2's file-size ceiling and
    /// this file does not. The subject is a ce dimension either way.
    ///
    /// It is deliberately NOT `record_note`. That channel draws
    /// *"⚑ About your last edit:"*, and `app::status::decline`'s own header
    /// rules it out for this case in as many words — *"an operator who reads
    /// 'About your last edit' after a gesture that did nothing has been told a
    /// small lie confidently"*. Nothing happened; the slot that says so wears
    /// `⊗`.
    DeclineVertexEdit {
        /// Which of the shell's sentences this refusal is. The mapping from
        /// `EditError` lives at `canvas::dimdrag::refusal_for`, so the engine's
        /// vocabulary stops at the canvas and never reaches the string catalog.
        why: crate::text::measure::VertexEditRefusal,
    },

    /// **Place a ce dimension** — where its line stands off the geometry, and
    /// where its number sits along that line.
    ///
    /// Raised by `crate::canvas::dimdrag` on the release of a drag, and by
    /// nothing else. There is no panel control for these two numbers and that
    /// is deliberate: they are a position, and a position is set by putting it
    /// somewhere, not by typing two scalars whose frame the operator would have
    /// to hold in their head.
    ///
    /// # Why this and not `move_dimension`
    ///
    /// `place_dimension` writes two fields the value function does not read, so
    /// it is **value-preserving by construction**: no drag, however far, can
    /// change the number the dimension prints. `move_dimension` translates the
    /// measured points as well — the distance survives a rigid motion, but the
    /// dimension leaves the feature it was measuring, which is not what an
    /// operator dragging a dimension line means. The engine's own doc comment
    /// settles it: *"This, not `move_dimension`, is what dragging a dimension
    /// does."* See `canvas::dimdrag`'s header for the table.
    ///
    /// # Blast radius
    ///
    /// **One annotation**, redrawn where it now sits, so
    /// [`Self::regenerates_the_whole_group`] answers `false`. Nothing else in
    /// the group moves and no value is re-measured.
    Place {
        /// The ce dimension to place.
        dimension: DimensionId,
        /// Standoff perpendicular to the measured axis, in points, signed
        /// along the canonical normal `DimensionKind::axis_frame` returns.
        offset: f64,
        /// Position of the value text along the dimension line, in points,
        /// measured from its midpoint.
        text_along: f64,
    },

    /// **Set a dimension group's drafting standard** — ANSI or ISO.
    ///
    /// Raised by `crate::dialogs::dimension_groups`.
    ///
    /// # Why it is an `Action` and not a call
    ///
    /// The same reason [`Self::SetGroupScale`] is. `set_group_standard` returns
    /// a **count of members regenerated**, because the standard governs
    /// terminator form, whether the dimension line is broken for its text, text
    /// orientation, and whether the extension-line gap and overshoot are
    /// absolute or line-width-relative. Every member's baked `/AP` is redrawn.
    /// That is one document edit touching many annotations across many pages,
    /// and the funnel is what makes it one `Ctrl+Z`.
    ///
    /// # Why it is per group and not per ce dimension
    ///
    /// `pdfcer_core::dimension::Group::standard` carries the reasoning: per ce
    /// dimension is a foot-gun with no use case — nobody wants dimension #3 ISO
    /// and #4 ANSI — and the standards' decimal conventions are unit-dependent
    /// while the unit is per group. The style cascade does allow a per-ce-dimension
    /// override of it — see [`Self::SetStyle`] — for the operator who has a
    /// reason; this is the default that override departs from.
    SetGroupStandard {
        /// The group whose members are redrawn.
        group: GroupId,
        /// The standard to draw them to.
        standard: DimStandard,
    },

    /// **Set a dimension group's appearance defaults** — the middle tier of
    /// the style cascade.
    ///
    /// Raised by `crate::dialogs::dimension_groups`.
    ///
    /// # Why the whole [`GroupStyle`] travels, rather than one property
    ///
    /// Because a `GroupStyle` **is** the tier: every field is an `Option` that
    /// is the operator's override checkbox for one property, and `None` on any
    /// of them is a meaningful value — *"this group has not spoken; use the
    /// factory default"*. A per-property variant would have to carry
    /// `Option<Option<T>>` to distinguish *leave it alone* from *clear it*,
    /// which is a shape nobody reads correctly twice.
    ///
    /// `EditSession::set_group_style` takes the whole struct for the same
    /// reason: per-property setters would make *clear this override* a
    /// different call from *set it*, so a surface would have two code paths
    /// where the operator sees one checkbox. The dialog therefore
    /// performs the read-modify-write that the CLI convention describes —
    /// setting one property leaves the others alone — which is what keeps a
    /// panel click and a `pdfcer group-style` invocation the same edit.
    SetGroupStyle {
        /// The group whose defaults change.
        group: GroupId,
        /// The complete next tier, read-modify-written by the dialog.
        style: GroupStyle,
    },

    /// **Show or hide a dimension group's layer.**
    ///
    /// Raised by `crate::dialogs::dimension_groups`.
    ///
    /// # Why this is not `Action::SetLayerVisible`
    ///
    /// They look identical and are not, and the difference is the operator's
    /// rather than an implementation detail.
    ///
    /// `SetLayerVisible` is a **view stance**: it toggles an optional-content
    /// group in the render key, changes nothing a save would write, and does
    /// not bump `edit_epoch`. This one calls
    /// `EditSession::toggle_dimension_layer`, which writes the group's default
    /// visibility into the document's `/OCProperties /D` configuration — so it
    /// is what the *file* says the next reader should see, and it survives into
    /// any other viewer that honours optional content.
    ///
    /// Hiding a layer for the afternoon and publishing a drawing whose ce
    /// dimensions are off by default are different acts. Only the second
    /// belongs in the undo log, and only the second is this.
    ///
    /// # The default group cannot be hidden
    ///
    /// The engine refuses it — `docs/core-api/02-editing-and-saving.md` §1.19,
    /// *"the default group is un-hideable"* — and the dialog therefore
    /// **omits** the control for that group rather than drawing one the engine
    /// declines, which is R9's rule that an affordance which cannot be honoured
    /// is not drawn. The variant still exists for every other group, and
    /// `apply` surfaces the refusal by name if one ever reaches it from a
    /// customized keymap, because a keymap is not the dialog.
    ToggleLayer {
        /// The group whose layer default changes.
        group: GroupId,
        /// `true` ⇒ on by default; `false` ⇒ registered in `/D /OFF`.
        visible: bool,
    },

    /// **Set one ce dimension's own style overrides** — the bottom tier of
    /// the cascade.
    ///
    /// Raised by `crate::panels::dimension`, against the selected annotation.
    ///
    /// # Why the whole [`StyleOverrides`] travels
    ///
    /// Identical reasoning to [`Self::SetGroupStyle`]: every field is an
    /// `Option` standing for one override checkbox, and `None` is a value
    /// rather than an absence — it means *inherit*.
    ///
    /// `Some(Tolerance::None)` and `None` are deliberately different states, and
    /// the engine's own doc comment gives the case that makes the distinction
    /// necessary rather than pedantic: *a group that tolerances everything and
    /// one feature that must not be toleranced is a real drawing, and it cannot
    /// be expressed if the two collapse.* Only a whole-struct action can carry
    /// that difference without a nested `Option`.
    ///
    /// # Why it names a `DimensionId` and not the selected `ObjId`
    ///
    /// Because that is what the engine verb takes. The canvas selection
    /// addresses an annotation by `ObjId` — stable, and what every annotation
    /// verb wants — while a ce dimension additionally has a sidecar record with
    /// its own id. The panel resolves one to the other through
    /// `DimensionModel::dimensions()` at the moment the operator acts and
    /// carries the resolved id, because an action is a complete statement of
    /// intent and the selection may be gone by the time the queue drains.
    SetStyle {
        /// The sidecar record to override.
        dimension: DimensionId,
        /// The complete next tier for it, read-modify-written by the panel.
        style: StyleOverrides,
    },

    /// **Switch a placed circular ce dimension between radius and diameter.**
    ///
    /// Raised by `crate::panels::dimension`.
    ///
    /// # Why this exists at all when the tool already asked
    ///
    /// The ui-spec names it as a *"real, named usability gap"* (§C.11.1): the
    /// toggle existed only in the draw-time tool options, so an operator who
    /// placed a radius and later wanted a diameter had to delete the ce
    /// dimension and re-draw it — losing its placement, its overrides and its
    /// object identity, in order to change which of two numbers derived from
    /// *the same fitted circle* gets printed.
    ///
    /// # It commits even when nothing changes
    ///
    /// `set_dimension_display` is documented as committing unconditionally
    /// (`docs/core-api/02-editing-and-saving.md` §1.19 flags it as *"the
    /// opposite of `set_info_field`"*), so asking for the value it already has
    /// writes an undo entry for a no-op.
    ///
    /// The panel therefore raises this **only on an actual change** of the
    /// control's value. That guard is in the surface rather than in `apply`
    /// deliberately: the arm cannot see what the operator pressed, only what
    /// they asked for, and re-reading the model here to compare would be a
    /// second source of truth for a value the widget already had.
    SetDisplay {
        /// The circular ce dimension.
        ///
        /// The engine refuses a non-circular one by name
        /// (`EditError::NotACircularDimension`) and refuses **before**
        /// mutating; the panel does not offer the control for a linear kind, so
        /// the refusal is a backstop rather than a path.
        dimension: DimensionId,
        /// `true` ⇒ print the diameter; `false` ⇒ print the radius.
        show_diameter: bool,
    },

    /// **Set the gap between a linear ce dimension's measured point and its
    /// extension line**, from a drag of the grip at the line's near end.
    /// `EditSession::set_dimension_extension_gap`.
    ///
    /// Changes where an extension line starts and never what the dimension
    /// measures, so it owes no re-measure disclosure. One annotation redrawn.
    SetExtensionGap {
        /// The linear ce dimension.
        dimension: DimensionId,
        /// Which of its two extension lines.
        end: pdfcer_core::dimension::DimensionEnd,
        /// The gap in points, page space; `None` returns the end to the
        /// style's standard gap.
        gap: Option<f64>,
    },
}
impl DimensionAction {
    /// Whether this verb's blast radius is **the whole document** rather than
    /// one page.
    #[must_use]
    pub const fn regenerates_the_whole_group(&self) -> bool {
        matches!(
            self,
            Self::SetGroupScale { .. }
                | Self::SetGroupStandard { .. }
                | Self::SetGroupStyle { .. }
                | Self::ToggleLayer { .. }
        )
    }
}
/// **Everything whose subject is a redaction**, as one family.
#[derive(Debug, Clone, PartialEq)]
pub enum RedactAction {
    /// **Mark every occurrence of some text for redaction.**
    ///
    /// Raised by `pdfcer_gui::panels::redact`'s Find & mark control. Applied
    /// through `vector_edit`, so it is one undoable command however many marks
    /// it creates — which is the right granularity: the operator asked one
    /// question, and taking back "mark every occurrence of this name" one
    /// annotation at a time would be unusable.
    ///
    /// # The query is carried, not a hit list
    ///
    /// The panel could resolve the matches itself and push the quads, the way
    /// `pdfcer_gui::app::actions::Action::CommitTextMarkup` carries the selection's boxes. It must not,
    /// for a reason specific to this verb: `pdfcer-core`'s own
    /// `mark_redactions_by_search_with` documents the trap — a front end whose
    /// search and whose marking disagree about *which hits exist* produces
    /// "three highlights and eleven redaction marks", and *"on the one
    /// operation whose whole purpose is removing content irreversibly, 'the
    /// mark set is a superset of the highlight set' is not a cosmetic
    /// difference."* Handing the engine the query lets the engine answer both
    /// halves with one scan.
    BySearch {
        /// The text, already trimmed by the panel.
        query: String,
        /// Whether to read the query as a pattern (`#` any digit, `?` any
        /// character) rather than as literal text.
        ///
        /// A `bool` here rather than an enum, unlike
        /// `crate::redact::ResidualAcknowledgement` — because this one is
        /// *named at its field* and reads as a sentence at the one call site
        /// that builds it, while that one is a positional argument at a call
        /// site where a transposition would write a file.
        pattern: bool,
        /// How the marks this creates will look once applied.
        ///
        /// Carried on the action, not read at apply time, and it is the same
        /// rule the pen follows for markup: the operator's choice is the one
        /// they had **when they pressed the control**. Reading it in the
        /// dispatcher would let a frame in which they also changed the fill
        /// swatch author marks they did not choose — and on this verb the
        /// difference is not cosmetic, because the appearance is baked into
        /// each `/Redact` annotation at creation and there is no verb that
        /// modifies one afterwards.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// **Mark the whole of one page for redaction.**
    ///
    /// Raised by `pdfcer_gui::panels::redact`'s Mark whole page control. The page
    /// is carried rather than read from `doc.view` at apply time, on
    /// `pdfcer_gui::app::actions::Action::CommitTextMarkup`'s rule: the operator marked the sheet they
    /// were looking at, and an action applied after a frame in which they also
    /// paged away must mark the sheet they meant.
    ///
    /// The rectangle is not carried, because it is not the operator's choice —
    /// it is the page's crop box, and `crate::panels::redact::whole_page_spec`
    /// is the one place that decision is made and tested.
    WholePage {
        /// The 0-based page to cover.
        page: usize,
        /// How the mark will look once applied. See
        /// [`Self::BySearch`]'s field of the same name.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// **Mark what is SELECTED on the page for redaction** — the third
    /// marking route, and the first that does not go through text.
    ///
    /// **Ken:** *"am I able to select objects on the canvas and redact them
    /// that way yet? … it just told me it couldn't."* Without this route it
    /// cannot: [`Self::BySearch`] reaches text pdfcer can read as text and
    /// [`Self::WholePage`] reaches everything, and on a CAD drawing most of
    /// what wants redacting is in between.
    ///
    /// `super::redactsel`'s header carries the argument in full, including why
    /// neither a page nor a rectangle is carried here.
    Selection {
        /// How the mark will look once applied. See
        /// [`Self::BySearch`]'s field of the same name.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// **Mark everything drawn outside the page boundary, on every sheet
    /// that has any** — the fourth marking route.
    ///
    /// Raised by `pdfcer_gui::dialogs::offpage`, and it is the only one of the
    /// four raised by a **window**. That is not an accident of where the button
    /// ended up: the other three mark something the operator is already looking
    /// at (a search hit, the current sheet, a selection), and this one marks
    /// content that **is not on screen and cannot be** — it does not render, it
    /// does not print, and the whole reason the window exists is that nothing in
    /// an ordinary reading of the document discloses it.
    ///
    /// # Why the BANDS travel and not the objects
    ///
    /// The census this comes from lists objects, and it would be natural to
    /// carry their boxes. It would also be wrong twice over:
    ///
    /// - **A box per object is a mark per object.** A CAD sheet with a
    ///   superseded revision block off its left edge decomposes into hundreds
    ///   of stroked paths, and marking each would author hundreds of `/Redact`
    ///   annotations for one operator gesture. `pdfcer_core::offpage::offpage_bands`
    ///   answers with at most four **non-overlapping** rectangles per page that
    ///   cover the same area, which is what §12.5.6.23's `/QuadPoints` list is
    ///   for.
    /// - **An object's box is not the area to remove.** Removing "that path"
    ///   leaves whatever else happens to sit beside it, and the operator's
    ///   request is *"take off what is outside the sheet"* — an area, not an
    ///   inventory.
    ///
    /// ⇒ So the window runs the census, asks the engine for the bands, and
    /// sends the bands. The objects stay in the window, where they are the
    /// **disclosure** — which is exactly rule 4's split: what is removed is a
    /// geometry, what is reported is the words that were found in it.
    ///
    /// # Why the page indices travel with them
    ///
    /// One press covers many sheets, so there is no "current page" to resolve
    /// against — and resolving against one would silently mark one sheet of a
    /// thirty-six-sheet set. The pairing is carried whole for
    /// [`Self::WholePage`]'s reason taken further: the operator marked the
    /// sheets the **census** named, and a frame in which they also paged away
    /// must not change which.
    OffPage {
        /// One entry per sheet with content outside its own boundary: the
        /// 0-based page index, and the bands the engine computed for it.
        ///
        /// A sheet with an empty band list is not expected here — the window
        /// filters clean pages out — but an empty list is skipped rather than
        /// treated as an error, because an action is plain data a test can
        /// build and four zero-area marks would be four annotations that do
        /// nothing.
        bands: Vec<(usize, Vec<pdfcer_core::page_tree::Rect>)>,
        /// How many sheets the census could not read at all.
        ///
        /// Carried so the status line can say so, and it is the sentence
        /// that keeps this window from issuing a clean bill it has not earned:
        /// a page whose content streams will not decode was **not checked**,
        /// and "marked everything outside the sheet" said over such a document
        /// is a claim about pages nobody looked at. `pdfcer_core::offpage`
        /// returns the two separately for precisely this reason.
        unreadable: usize,
        /// How the marks will look once applied. See [`Self::BySearch`]'s field
        /// of the same name — and note that this route reads the **panel's**
        /// chosen appearance like the other three, so a window on the Edit tab
        /// cannot produce a differently-coloured mark from the panel beside it.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// **Take one redaction mark off.**
    ///
    /// Raised by a row's Remove control. The engine's
    /// `EditSession::delete_redaction_mark` rather than its general annotation
    /// delete, deliberately and on core's own instruction: the two record
    /// different `CommandKind`s so that an undo tooltip can say *"remove a
    /// redaction mark"* rather than *"delete annotation"*, and — as that
    /// method's docs put it — *"I decided not to redact that"* is a different
    /// claim from *"delete annotation"*.
    ///
    /// The **annotation id**, not a row index: a list position is a position in
    /// a census rebuilt every frame, and by the time the apply phase runs the
    /// same index may name a different mark. `crate::app` §10's rule —
    /// *selection is an identity, not a position* — applied to a list.
    RemoveMark {
        /// The `/Redact` annotation to delete.
        annot_id: pdfcer_core::object::ObjId,
    },
    /// **Arm the removal of every redaction mark at the next save, or
    /// disarm it** — O125. **The whole argument — undo-preserving, why Cancel
    /// exists — is on the apply arm** in `app::actions::redact`, on this
    /// file's own R2 rule.
    Pending(crate::redact::Staging),
    /// **Apply the removal into the open document, now.**
    ///
    /// # Why this carries COUNTS and not the document
    ///
    /// `PreparedRedaction::bytes` is private *"deliberately and
    /// load-bearingly"*, because a public accessor would restore the surface
    /// a caller would need to write an **unverified** file from. So
    /// the whole value travels, and the only thing this arm may do with it is
    /// call [`crate::redact::PreparedRedaction::into_verified_document`], which
    /// re-proves the removal before handing back anything.
    ///
    /// The acknowledgement travels with it for the same reason it is an
    /// argument to `write_to` rather than a field: consent belongs to the
    /// press, not to the preparation. A `PreparedRedaction` sitting in a queue
    /// carries no permission of its own.
    ///
    /// Boxed. `PreparedRedaction` holds a whole redacted document, and an
    /// unboxed variant would make every `Action` in the program that size.
    ApplyNow {
        /// How many marked regions the removal covered, for the trace and the
        /// operator's receipt.
        marks: usize,
        /// How many pages it touched.
        pages: usize,
    },
}
/// The three verbs whose subject is a whole file inside the document.
///
/// See the module header for what makes them a family, and [`AttachmentRef`]
/// for why the operand is neither an index nor an object id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentAction {
    /// **Embed a file in this document** (ISO 32000-1 §7.11.4.1, inclusion
    /// route 2).
    ///
    /// Raised by `crate::panels::attachments::attach` and by nothing else.
    ///
    /// # It carries no path, and that is the whole design of this variant
    ///
    /// The picker opens inside the **apply** phase, exactly as
    /// `super::write::WriteAction::FormData`'s does and for its stated reason.
    /// The alternative — pick in the widget, carry the path — puts a modal OS
    /// window inside a layout pass, which is the defect `super::write`'s header
    /// exists to name.
    ///
    /// It also buys the harness seam: `crate::app::files::DIAG_ATTACH_PATH`
    /// answers the dialog without a human, and a driven check of this feature
    /// is otherwise unwritable because no synthetic input reaches a native
    /// dialog.
    ///
    /// # The description travels, because it can only be set now
    ///
    /// `attach_file` takes `description: Option<&str>` and writes it to the
    /// file specification's `/Desc` (Table 44, whose own row says `/Desc`
    /// *"shall be used for files in the `EmbeddedFiles` name tree"* — exactly
    /// this route). `pdfcer-core` exposes **no verb that edits one afterwards**,
    /// so this is the operator's only opportunity, and the value has to be
    /// captured at the press rather than read at apply time: the panel's draft
    /// is cleared on the frame the action is raised, and the queue drains after
    /// it.
    ///
    /// `None` is the ordinary answer and is not the same as `Some("")`: an
    /// empty description would write a `/Desc` key holding nothing, which is a
    /// key a later reader has to interpret. Omitting it says the document
    /// carries no description, which is the truth.
    Attach {
        /// What the operator typed, trimmed and non-empty by the time it gets
        /// here, or `None` for no `/Desc` at all.
        description: Option<String>,
    },
    /// **Attach the clipboard's file to this document.**
    ///
    /// # `replacing` is carried, and it is not a convenience
    ///
    ///
    /// It is computed **before** the write, because afterwards the answer has
    /// changed: the document now has exactly one file of that name either way,
    /// so asking then cannot distinguish the two outcomes. That is the same
    /// reasoning `FormEdit::Recompute` records for carrying its plan.
    Paste {
        /// The clip, carried whole. See `panels::attachments::clip` for why it
        /// travels with the action rather than being re-read at apply time.
        clip: Box<pdfcer_core::attachments::AttachmentClip>,
        /// Whether a file of this name is already listed in the destination.
        replacing: bool,
    },
    /// **Remove one document-level attachment** — the index entry, the file
    /// specification and the bytes, as ONE undo entry.
    ///
    /// Raised by `crate::panels::attachments` and by nothing else.
    ///
    /// # What "removed" does NOT mean, and why this variant carries a name
    ///
    /// `detach_file`'s own doc comment states the obligation this shell is
    /// under, and it is unusually direct:
    ///
    /// > *"This is NOT a redaction verb and must not be described as one … the
    /// > attachment's bytes remain recoverable from the earlier revision. Only
    /// > a full rewrite drops superseded revisions … Shells are expected to say
    /// > so rather than let 'delete' imply erasure."*
    ///
    /// The `name` field exists for that sentence and for nothing else. The
    /// engine returns `()`, the row is gone from the panel by the time the
    /// disclosure is read, and *"An attachment was removed"* is a sentence that
    /// leaves an operator who removed the wrong one unable to tell.
    ///
    /// It is the **displayed** name rather than the key, deliberately: the key
    /// is bytes with no declared encoding (§7.9.6) and producers mangle it with
    /// numeric suffixes and portfolio folder prefixes, so it is the right thing
    /// to address the document with and the wrong thing to show a person.
    ///
    /// # Why there is no confirmation dialog
    ///
    /// A destructive verb must be *confirmed or clearly undoable*, and this is
    /// the second: one press is one `EditSession` command — `detach_file`
    /// removes the tree entry, the file specification and the stream *"all
    /// three, as ONE undo entry"* — so one `Ctrl+Z` puts the file back whole.
    ///
    /// The consequence the operator actually needs is not
    /// *"are you sure?"* but *"this does not erase the bytes"*, and a
    /// confirmation dialog is a bad place to put that because it arrives
    /// **after** the decision. It is on the panel, beside the button, before
    /// the press.
    Detach {
        /// The `/EmbeddedFiles` name-tree key, verbatim. See [`AttachmentRef`].
        key: Vec<u8>,
        /// The name the panel showed, for the disclosure. Never used to find
        /// anything.
        name: String,
    },
    /// **Write one attachment's bytes out to a file the operator picks.**
    ///
    /// Raised by `crate::panels::attachments` and by nothing else.
    ///
    /// # It changes nothing about the document
    ///
    /// No `vector_edit`, no undo entry, no epoch bump, no invalidation — the
    /// property `super::export`'s header names as what makes an export a
    /// subject of its own. It is filed here rather than there because its
    /// *operand* is an attachment and its refusals are attachment refusals; the
    /// seam `super::export` draws is *"what class of thing does this verb act
    /// on?"*, and by that seam this belongs beside its two siblings.
    ///
    /// # Why the bytes are not carried
    ///
    /// `super::write::WriteAction::Compacted` carries a whole serialised
    /// document, and its doc says why: the confirmation window quoted a
    /// measurement of those exact bytes, so those exact bytes are the operand.
    /// Nothing quoted anything here. Carrying the payload would mean decoding a
    /// possibly-enormous stream during the **frame**, to hand the apply phase a
    /// value it can decode itself — and a *stale* one, because an undo raised
    /// earlier in the same frame would leave the copy describing a revision
    /// that is no longer open.
    ///
    /// It would also break an explicit engine contract.
    /// `extract_attachment`'s docs warn that *"the view must be the one the
    /// `Attachment` was listed from"* — an `Attachment` carries object ids, and
    /// an id only means something relative to a document — so the listing and
    /// the extraction have to happen in one breath. `pdfcer_gui::app::actions::attachments::save_copy` does exactly
    /// that, against the session as it stands when the save runs, which is the
    /// only reading that can be defended.
    SaveCopy {
        /// Which attachment, addressed the only way its kind allows.
        at: AttachmentRef,
        /// The name the panel showed, for the trace and for the sentence that
        /// says whether pdfcer had to use a different one on disk.
        name: String,
    },
    /// **Write one embedded 3D model's data to a file the operator picks**,
    /// as stored (U3D, PRC or STEP), with only its compression undone.
    ///
    /// Carries the listing's own row; the apply arm re-lists and acts only on
    /// an identical row, so a model that moved or vanished is refused.
    SaveModel {
        /// The row as the panel listed it.
        artwork: pdfcer_core::threed::ThreeDArtwork,
    },
    /// **Pick a U3D, PRC or STEP file and place it on `page`** as a 3D
    /// annotation, centred, one undo entry. The picker runs in the apply arm,
    /// as it does for [`Self::Attach`].
    InsertModel {
        /// The 0-based page, frozen when the command ran.
        page: usize,
    },
}
