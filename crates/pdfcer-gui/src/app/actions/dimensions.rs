//! # `app::actions::dimensions` — everything the ce-dimension feature asks the
//! document to do
//!
//! ## Rule 15 first, because this module is where it bites
//!
//! A **ce dimension** is one *pdfcer itself authors*: a `/Line` annotation
//! carrying `/IT /LineDimension`, a baked `/AP`, and a record in the
//! document's `/PieceInfo` sidecar that says which group it belongs to and
//! what it measures. A **pdf dimension** is CAD-exported page content that
//! pdfcer reads and must not silently alter. Nothing in this file touches the
//! second kind. Every verb here names the first, and the bare word
//! "dimension" is not used on its own anywhere in it.
//!
//! ## Why this is its own file
//!
//! **R2**, and the seam the sibling modules already draw: [`super::pages`] is
//! *what happens to a page*, [`super::annots`] is *what happens to an
//! annotation that already exists*, [`super::apply`] is *what happens to page
//! content*. This is **what happens to the dimensioning model** — the groups,
//! their scales and standards, the style cascade, and the per-ce-dimension
//! overrides.
//!
//! It is a real subject rather than a size-driven cut, and the evidence is
//! that its verbs share a property none of the others do: **most of them
//! regenerate appearance streams for annotations the operator is not looking
//! at.** A group's members are wherever they were placed, across any number of
//! pages, and a change to the group rewrites all of them. Every arm below has
//! to reason about that, and no arm anywhere else does.
//!
//! ## ★ The one thing every reader of this file needs to know first
//!
//! `EditSession`'s ce-dimension verbs come in two shapes, and confusing them
//! is the failure this module is arranged to prevent:
//!
//! | shape | verbs | blast radius |
//! |---|---|---|
//! | **group** | `set_group_scale`, `set_group_standard`, `set_group_style`, `toggle_dimension_layer` | every member of the group, on every page |
//! | **one ce dimension** | `set_dimension_style`, `set_dimension_display`, `place_dimension`, `delete_dimension` | exactly one annotation |
//!
//! The group verbs therefore clear **every** cached raster rather than the
//! current page's, and they pass page `0` to [`super::apply::vector_edit`]
//! with a note, because a group is document-scoped and has no page. The
//! per-ce-dimension verbs pass the real page and invalidate normally.
//!
//! ## ★ And the second: a returned count is not a count of anything visible
//!
//! `set_group_style` and `set_group_standard` both return `usize`, and
//! `docs/core-api/03-capabilities.md` §1.6 trap (a) warns in as many words
//! that it is the number of members **regenerated** — which is every wired
//! member, including the ones that override the property being changed,
//! because regenerating an overrider produces byte-identical output and is
//! free in the diff.
//!
//! The number an operator wants is how many will visibly **move**, which is a
//! strictly smaller set (the members whose `StyleProvenance` for the edited
//! property reports `follows_group() == true`) and which **must be computed
//! before the edit** if it is to be shown before the edit. That is the
//! surface's job, not this file's, and it is why no arm here discloses a
//! count. Disclosing the engine's number would be worse than disclosing
//! nothing: it is a real number, plausibly labelled, answering a different
//! question.

use pdfcer_core::dimension::{
    DimStandard, DimensionId, DimensionKind, GroupId, GroupStyle, NumberFormat, ScaleState,
    StyleOverrides, Unit,
};
use pdfcer_core::edit::GroupDeletion;

use crate::app::state::OpenDoc;

/// Everything the ce-dimension feature can ask the document for.
///
/// ## Why this is a sub-enum rather than eight more variants on [`Action`]
///
/// [`Action`]: super::Action
///
/// Three reasons, in the order they decided it:
///
/// 1. **They share a routing rule that the flat enum could not express.**
///    Four of the eight are group-scoped and must invalidate the whole strip;
///    four are annotation-scoped and must not. As flat variants that rule
///    lives in eight arms and is re-derived in each; as a family it lives once,
///    in [`apply`], where a new verb has to pick a side to compile.
/// 2. **R2.** `super`'s enum was 1,456 lines before this family existed and
///    these variants carry the documentation this project asks for. The
///    alternative to a seam is thinner prose, which the file-size gate's own
///    header names as the incentive it refuses to create.
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
    /// Raised by [`crate::canvas::measure`] when a pick machine returns a
    /// `DimensionKind`, which for the linear tool is the **third** click (what,
    /// to what, and where it sits) and for the others is the pick that first
    /// makes the geometry knowable.
    ///
    /// # ★ Why the geometry arrives whole rather than as points
    ///
    /// `DimensionKind` is `pdfcer-core`'s own type and it is carried across
    /// unchanged, which is the property the salvage's two equivalence tests
    /// exist to protect: the value built here is **byte-for-byte the one
    /// `pdfcer dimension-add` builds** from the same picks, so a ce
    /// dimension authored on the canvas and one authored from the command line
    /// are the same bytes in the file. Decomposing it into coordinates here and
    /// rebuilding it in the apply arm would put a second constructor in the
    /// path and quietly end that guarantee.
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
        /// ★ **What the gesture inferred, in the operator's words** — carried
        /// with the edit rather than recorded when the gesture ended.
        ///
        /// Empty for the linear and circular tools, whose output is what the
        /// operator pointed at. Non-empty for the **two-line** tool, which
        /// classifies: it may read two lines as parallel *because the operator
        /// asked*, overriding a real measured angle, and it may find an apex
        /// that exists only if the lines are extended. `pdfcer-core` requires
        /// both to be said (`03-capabilities.md` §1.5 obligation 4), and this
        /// build said neither until 2026-08-19.
        ///
        /// # ★ Why it travels HERE and not through `record_note`
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

    /// ★ **Calibrate a dimension group** — say what its numbers mean.
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
    /// would be a field [`apply`] had to ignore, which is how a reader comes to
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

    /// ★ **Create a dimension group** — a second answer to *"what scale is
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

    /// ★ **Rename a dimension group.**
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
    /// # Why it took a request
    ///
    /// `Group::name` is a `pub String` on a snapshot, and `EditSession` had no
    /// verb for it, so a mistyped group name was permanent for the life of the
    /// document. Filed 2026-08-18, shipped 2026-08-19. The engine's reply
    /// corrected the request's other half: of `Group`'s eight fields, **`name`
    /// alone** had no session route — the unit is reachable through
    /// `set_group_scale`, which takes a whole `NumberFormat`.
    RenameGroup {
        /// The group to rename.
        group: GroupId,
        /// The new name. Trimmed and non-empty by the time it gets here, for
        /// [`Self::AddGroup`]'s reason: a blank row in a group picker is
        /// indistinguishable from a broken one.
        name: String,
    },

    /// ★ **Delete a dimension group**, answering the members question.
    ///
    /// Raised by `crate::dialogs::dimension_groups`.
    ///
    /// # ★ The policy is the whole design, and it is the ORPHAN question again
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
    /// The count is in the error because *"this group is not empty"* and
    /// *"this group holds forty dimensions"* prompt different decisions from an
    /// operator — and only a surface can put that question in front of them.
    ///
    /// **There is deliberately no delete-the-members policy.** Deleting a ce
    /// dimension also removes its annotation from the page's `/Annots`, so
    /// doing it inside this verb would be a second implementation of
    /// `delete_dimension`'s removal; and calling `delete_dimension` in a loop
    /// would produce one undo entry **per member**, so undoing a group deletion
    /// would take forty presses and could stop halfway with the group already
    /// gone. If this shell ever needs it, the engine has offered to factor
    /// `delete_dimension`'s core out properly — which is a request, not a
    /// workaround to write here.
    ///
    /// # ★ A refusal validates before mutating, and the dialog relies on it
    ///
    /// The engine's reply states it: *"a rejected deletion leaves the model
    /// byte-identical. You can call it speculatively to populate a confirmation
    /// dialog."* The dialog does not — it reads `member_count` from the model
    /// it is already holding, which costs nothing and needs no round trip — but
    /// the guarantee is what makes a Delete press safe to offer at all rather
    /// than gated behind a count the surface might have got wrong.
    /// ★★★ **Say something other than the measurement**, without changing it.
    ///
    /// `EditSession::set_dimension_label`, shipped 2026-08-30. Raised by
    /// `panels::properties::dimension::label_row` and by nothing else.
    ///
    /// # The engine's own headline: it does NOT destroy the measurement
    ///
    /// The override is a **caption**. The measured value stays underneath, so
    /// `None` restores it with **no re-measurement** — the number that comes
    /// back is the one that was always there rather than a fresh calculation
    /// that might round differently.
    ///
    /// ⇒ Which is why `label` is an `Option` and why the panel has no Clear
    /// button: clearing the box *is* `None`, and a second control that meant
    /// the same thing would be a second way to reach one state.
    SetLabel {
        /// Which ce dimension.
        dimension: pdfcer_core::dimension::DimensionId,
        /// The caption, or `None` to show the measurement again.
        label: Option<String>,
    },
    DeleteGroup {
        /// The group to remove.
        group: GroupId,
        /// What to do about its members.
        policy: GroupDeletion,
    },

    /// ★ **Move a placed ce dimension to another group.**
    ///
    /// Raised by `crate::panels::properties::dimension`.
    ///
    /// # ★ This is NOT a field assignment, and the surface has to know that
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

    /// ★ **Place a ce dimension** - where its line stands off the geometry, and
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
    /// measured points as well - the distance survives a rigid motion, but the
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
    /// ★★ **Move one vertex of a perimeter ce dimension**, re-measuring it.
    ///
    /// Raised by `crate::canvas::dimdrag::drag_vertex` on the release of a
    /// corner drag. The operator's ask of 2026-08-20: *"I want to be able to
    /// edit the endpoints of the lines to adjust the shape."*
    ///
    /// # ★ This one CHANGES THE NUMBER, and it is the first that does
    ///
    /// [`Self::Place`] writes fields the value function does not read, so no
    /// label drag can alter what a dimension prints. `SetDimensionGroup`
    /// re-measures under a different scale. This moves a corner of the measured
    /// shape itself, and the engine names it: *"the first ce-dimension verb that
    /// deliberately changes what a ce dimension measures."*
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

    /// ★ **Set a dimension group's drafting standard** — ANSI or ISO.
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
    /// `pdfcer-core`'s own reasoning, quoted because it is the answer to the
    /// obvious question (`dimension/group.rs:71-79`): per ce dimension *"would
    /// be a foot-gun with no use case (nobody wants dimension #3 ISO and #4
    /// ANSI)"*, and the standards' decimal conventions are unit-dependent while
    /// the unit is per group. The style cascade does allow a per-ce-dimension
    /// override of it — see [`Self::SetStyle`] — for the operator who has a
    /// reason; this is the default that override departs from.
    SetGroupStandard {
        /// The group whose members are redrawn.
        group: GroupId,
        /// The standard to draw them to.
        standard: DimStandard,
    },

    /// ★ **Set a dimension group's appearance defaults** — the middle tier of
    /// the style cascade.
    ///
    /// Raised by `crate::dialogs::dimension_groups`.
    ///
    /// # Why the whole [`GroupStyle`] travels, rather than one property
    ///
    /// Because a `GroupStyle` **is** the tier: seven `Option`s, each of which
    /// is the operator's override checkbox for one property, and `None` on any
    /// of them is a meaningful value — *"this group has not spoken; use the
    /// factory default"*. A per-property variant would have to carry
    /// `Option<Option<T>>` to distinguish *leave it alone* from *clear it*,
    /// which is a shape nobody reads correctly twice.
    ///
    /// The engine's own verb takes the whole struct and says why
    /// (`edit.rs:18263-18271`): per-property setters *"would make 'clear this
    /// override' a different call from 'set it', so a surface would have two
    /// code paths where the operator sees one checkbox."* The dialog therefore
    /// performs the read-modify-write that the CLI convention describes —
    /// setting one property leaves the others alone — which is what keeps a
    /// panel click and a `pdfcer group-style` invocation the same edit.
    SetGroupStyle {
        /// The group whose defaults change.
        group: GroupId,
        /// The complete next tier, read-modify-written by the dialog.
        style: GroupStyle,
    },

    /// ★ **Show or hide a dimension group's layer.**
    ///
    /// Raised by `crate::dialogs::dimension_groups`.
    ///
    /// # ★ Why this is not `Action::SetLayerVisible`
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
    /// [`apply`] surfaces the refusal by name if one ever reaches it from a
    /// customized keymap, because a keymap is not the dialog.
    ToggleLayer {
        /// The group whose layer default changes.
        group: GroupId,
        /// `true` ⇒ on by default; `false` ⇒ registered in `/D /OFF`.
        visible: bool,
    },

    /// ★ **Set one ce dimension's own style overrides** — the bottom tier of
    /// the cascade.
    ///
    /// Raised by `crate::panels::dimension`, against the selected annotation.
    ///
    /// # Why the whole [`StyleOverrides`] travels
    ///
    /// Identical reasoning to [`Self::SetGroupStyle`]: eleven `Option`s, each
    /// one an override checkbox, and `None` is a value rather than an absence —
    /// it means *inherit*.
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

    /// ★ **Switch a placed circular ce dimension between radius and diameter.**
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
    /// # ★ It commits even when nothing changes
    ///
    /// `set_dimension_display` is documented as committing unconditionally
    /// (`docs/core-api/02-editing-and-saving.md` §1.19 flags it as *"the
    /// opposite of `set_info_field`"*), so asking for the value it already has
    /// writes an undo entry for a no-op.
    ///
    /// The panel therefore raises this **only on an actual change** of the
    /// control's value. That guard is in the surface rather than in [`apply`]
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
}

impl DimensionAction {
    /// Whether this verb's blast radius is **the whole document** rather than
    /// one page.
    ///
    /// The module header's first table, expressed once as code so a ninth
    /// variant cannot be added without picking a side. [`apply`] uses it to
    /// decide whether to clear every cached raster, and the honest answer is
    /// derived from *what the engine verb touches*, not from what the operator
    /// was looking at when they asked.
    ///
    /// [`Self::Commit`] is `false` even though it is the one that *creates* a
    /// member: authoring places a single annotation on a single page, and the
    /// group's other members are not redrawn by it.
    ///
    /// ★ Three of the 2026-08-19 verbs are `false` and each for its own reason,
    /// which is why they are worth stating rather than leaving to the
    /// `matches!`:
    ///
    /// - [`Self::RenameGroup`] regenerates **nothing at all** — no member's
    ///   appearance depends on what its group is called.
    /// - [`Self::SetDimensionGroup`] regenerates **exactly one** annotation.
    ///   Its label changes, which is startling and is still one annotation.
    /// - [`Self::DeleteGroup`] regenerates **as many as its policy moves**,
    ///   which is zero under `Refuse` and every member under `Reassign`. That
    ///   is a property of the *policy* rather than of the verb, and a predicate
    ///   taking `&self` cannot see inside the variant honestly — so [`apply`]
    ///   decides it there, at the one place the policy is in hand.
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

/// Apply one ce-dimension verb to the open document.
///
/// ## The two-step every arm shares
///
/// 1. **Invalidate as widely as the verb reaches.** A group verb clears
///    `doc.strip_rasters` wholesale, because a group's members are wherever the
///    operator put them and a strip entry drawn before the edit would keep
///    showing the old number with nothing to say so. This is the same
///    wholesale-invalidation argument `app::pages` makes for a page
///    permutation, arriving from a different direction.
/// 2. **Mutate through [`super::apply::vector_edit`]**, so the
///    cancel-mutate-bump-invalidate protocol, the undo entry, the trace line
///    and the disclosure store are the ones every other edit in this
///    application uses, rather than a second implementation of them here.
///
/// ## Why the group arms pass page `0`
///
/// `vector_edit` takes a page for its trace line and its per-page raster drop.
/// A group is document-scoped and has no page, so `0` is passed with this note
/// rather than the signature gaining an `Option<usize>` that every other caller
/// would have to spell. The wholesale clear in step 1 is what actually
/// discharges the invalidation; the page reaches the funnel only as a label.
/// **Set or clear a ce dimension's caption.**
///
/// # ★★ What is reported, and why the restore says something different
///
/// `DimensionLabelChange` carries `measured` and `printed` separately. When an
/// override goes on they differ, and the receipt names **both** — the operator
/// has just hidden a number and the one place that number must still be
/// available is the sentence about hiding it.
///
/// When the override comes **off**, `printed` becomes `measured` again and the
/// receipt says so plainly. That is not a formality: the whole reassurance this
/// feature rests on is that clearing the caption restores the *original*
/// measurement rather than re-measuring, and a receipt naming the number is
/// what lets an operator confirm it did.
///
/// ★ `changed: false` produces no disclosure at all. The engine returns `Ok`
/// for a no-op — setting a caption to what it already says — and a sentence
/// there would evict a real disclosure to report that nothing happened.
fn set_label(
    doc: &mut OpenDoc,
    dimension: pdfcer_core::dimension::DimensionId,
    label: Option<&str>,
) {
    let page = doc.view.page_index;
    super::apply::vector_edit(doc, "dimension-label", page, 1, |session| {
        session.set_dimension_label(dimension, label).map(|report| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "dimension-label-applied id={} changed={} measured={:?} printed={:?}",
                    dimension.0, report.changed, report.measured, report.printed
                )
            });
            if !report.changed {
                return Vec::new();
            }
            vec![match report.applied {
                Some(_) => crate::text::panels::dimension::label_set(&report.measured),
                None => crate::text::panels::dimension::label_restored(&report.printed),
            }]
        })
    });
}

pub(super) fn apply(doc: &mut OpenDoc, action: DimensionAction) {
    if action.regenerates_the_whole_group() {
        doc.strip_rasters.clear();
    }
    match action {
        // One `add_dimension`, one undo entry — the same contract
        // `CommitMarkup` holds, through the same funnel, so the protocol is
        // not written a second time.
        DimensionAction::Commit {
            page,
            group,
            kind,
            disclosures,
        } => {
            super::apply::vector_edit(doc, "add-dimension", page, 1, |session| {
                // The gesture's disclosures are returned as the edit's, which
                // is what puts them on the status bar stamped with the epoch
                // this commit produced. `map` rather than a discarded result:
                // the id the engine returns is not needed and the list is.
                session
                    .add_dimension(page, group, kind)
                    .map(|_| disclosures)
            });
        }
        DimensionAction::SetGroupScale {
            group,
            scale,
            format,
        } => {
            super::apply::vector_edit(doc, "set-group-scale", 0, 1, |session| {
                session
                    .set_group_scale(group, scale, format)
                    .map(|_| Vec::new())
            });
        }
        // ★ Creating a group regenerates nothing — it has no members yet — so
        // it is not in `regenerates_the_whole_group` and clears no rasters.
        // It is still a document edit: the sidecar gains a record, and a save
        // taken afterwards carries the group.
        DimensionAction::AddGroup { name, unit } => {
            super::apply::vector_edit(doc, "add-dimension-group", 0, 1, |session| {
                session.add_dimension_group(&name, unit).map(|_| Vec::new())
            });
        }
        // ★ The one group verb that regenerates NOTHING. No member's
        // appearance depends on what its group is called, so no raster is
        // dropped and `regenerates_the_whole_group` says so.
        DimensionAction::RenameGroup { group, name } => {
            super::apply::vector_edit(doc, "rename-dimension-group", 0, 1, |session| {
                session
                    .rename_dimension_group(group, &name)
                    .map(|()| Vec::new())
            });
        }
        // ★ Routed through `delete_dimension_group_with` for BOTH policies,
        // including `Refuse` — which is exactly what the no-argument
        // `delete_dimension_group` does.
        //
        // One call site rather than two, because the difference between them is
        // a value this variant already carries, and a `match` here would be a
        // second place for the default policy to be decided. The engine's own
        // pair exists for callers who have no policy to express; this one
        // always does.
        //
        // The returned count is the number REASSIGNED, and it is deliberately
        // dropped: the dialog computed the member count before pressing, from
        // the model it was already holding, and that is the number it showed.
        // Reporting a second count afterwards would be two answers to one
        // question — the shape `set_group_style`'s return value already taught
        // this module to refuse.
        // ★ One line: `set_dimension_label` owns the whole contract — the
        // whitespace-only refusal, keeping the measurement, and regenerating
        // the appearance at the new caption.
        DimensionAction::SetLabel { dimension, label } => {
            set_label(doc, dimension, label.as_deref());
        }
        DimensionAction::DeleteGroup { group, policy } => {
            // A reassignment moves members between groups, which re-measures
            // and redraws each of them wherever it is. A refusal moves nothing.
            // Deciding here rather than in `regenerates_the_whole_group` because
            // it is a property of the POLICY, not of the verb — the predicate
            // takes the variant and cannot see inside it.
            if matches!(policy, GroupDeletion::Reassign(_)) {
                doc.strip_rasters.clear();
            }
            super::apply::vector_edit(doc, "delete-dimension-group", 0, 1, |session| {
                session
                    .delete_dimension_group_with(group, policy)
                    .map(|_| Vec::new())
            });
        }
        // ★ One annotation redrawn, and its printed NUMBER changes — see the
        // variant. The page it is on is not known here (a `DimensionId` names a
        // sidecar record, not a page), so page `0` is passed with the note every
        // document-scoped verb in this file passes it with, and the strip is
        // deliberately NOT cleared: exactly one annotation moved.
        DimensionAction::SetDimensionGroup { dimension, group } => {
            super::apply::vector_edit(doc, "set-dimension-group", 0, 1, |session| {
                session
                    .set_dimension_group(dimension, group)
                    .map(|()| Vec::new())
            });
        }
        // ★ One annotation redrawn, in place. No value is re-measured -
        // `place_dimension` writes two fields the value function does not read
        // - so unlike `SetDimensionGroup` above there is not even a number to
        // disclose. The page is not known here (a `DimensionId` names a sidecar
        // record, not a page), so page `0` is passed with the note every
        // document-scoped verb in this file passes it with.
        // ★★ The one dimension verb that RE-MEASURES, so it is the one that
        // owes a disclosure. `VertexOutcome` carries the label before and
        // after, because the old value cannot be reconstructed once the
        // geometry it came from is gone - and "12.40 m -> 13.85 m" is a
        // disclosure where "13.85 m" is just the number already on the page.
        DimensionAction::MoveVertex {
            dimension,
            index,
            dx,
            dy,
        } => {
            super::apply::vector_edit(doc, "move-dimension-vertex", 0, 1, |session| {
                session
                    .move_dimension_vertex(dimension, index, dx, dy)
                    .map(|outcome| {
                        // Nothing to say when the number did not move - a
                        // corner dragged along its own segment changes the
                        // shape and not the length, and reporting "13.85 m ->
                        // 13.85 m" would train the operator to ignore the line
                        // that matters.
                        if outcome.label == outcome.previous_label {
                            Vec::new()
                        } else {
                            vec![crate::text::measure::vertex_remeasured(
                                &outcome.previous_label,
                                &outcome.label,
                            )]
                        }
                    })
            });
        }
        DimensionAction::Place {
            dimension,
            offset,
            text_along,
        } => {
            super::apply::vector_edit(doc, "place-dimension", 0, 1, |session| {
                session
                    .place_dimension(dimension, offset, text_along)
                    .map(|()| Vec::new())
            });
        }
        DimensionAction::SetGroupStandard { group, standard } => {
            super::apply::vector_edit(doc, "set-group-standard", 0, 1, |session| {
                session
                    .set_group_standard(group, standard)
                    .map(|_| Vec::new())
            });
        }
        DimensionAction::SetGroupStyle { group, style } => {
            super::apply::vector_edit(doc, "set-group-style", 0, 1, |session| {
                session.set_group_style(group, style).map(|_| Vec::new())
            });
        }
        // ★ The returned `bool` is the RESULTING visibility, and it is
        // deliberately dropped. The dialog re-reads the model next frame, so
        // carrying the answer back would give the surface two sources for one
        // fact — and the one carried here would be the older of the two by the
        // time anything drew it.
        DimensionAction::ToggleLayer { group, visible } => {
            super::apply::vector_edit(doc, "toggle-dimension-layer", 0, 1, |session| {
                session
                    .toggle_dimension_layer(group, visible)
                    .map(|_| Vec::new())
            });
        }
        DimensionAction::SetStyle { dimension, style } => {
            super::apply::vector_edit(doc, "set-dimension-style", 0, 1, |session| {
                session
                    .set_dimension_style(dimension, style)
                    .map(|_| Vec::new())
            });
        }
        DimensionAction::SetDisplay {
            dimension,
            show_diameter,
        } => {
            super::apply::vector_edit(doc, "set-dimension-display", 0, 1, |session| {
                session
                    .set_dimension_display(dimension, show_diameter)
                    .map(|_| Vec::new())
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfcer_core::dimension::DEFAULT_GROUP_ID;

    /// The blast-radius predicate agrees with the module header's table.
    ///
    /// Written as an exhaustive listing rather than as `matches!` twice, so
    /// that a ninth variant fails to compile here and forces the author to
    /// state which side it is on — which is the whole reason the predicate is a
    /// method rather than a condition inlined in [`apply`].
    #[test]
    fn every_group_verb_is_document_wide_and_every_other_is_not() {
        let g = DEFAULT_GROUP_ID;
        let d = DimensionId(0);

        // Group-scoped: every member, on every page.
        for a in [
            DimensionAction::SetGroupScale {
                group: g,
                scale: ScaleState::NeverSet,
                format: Unit::Millimeter.default_format(),
            },
            DimensionAction::SetGroupStandard {
                group: g,
                standard: DimStandard::default(),
            },
            DimensionAction::SetGroupStyle {
                group: g,
                style: GroupStyle::default(),
            },
            DimensionAction::ToggleLayer {
                group: g,
                visible: false,
            },
        ] {
            assert!(
                a.regenerates_the_whole_group(),
                "{a:?} touches every member and must clear the whole strip"
            );
        }

        // Annotation-scoped, plus the two that create rather than change.
        for a in [
            DimensionAction::AddGroup {
                name: "Detail".to_owned(),
                unit: Unit::Millimeter,
            },
            DimensionAction::SetStyle {
                dimension: d,
                style: StyleOverrides::default(),
            },
            DimensionAction::SetDisplay {
                dimension: d,
                show_diameter: true,
            },
        ] {
            assert!(
                !a.regenerates_the_whole_group(),
                "{a:?} touches one record and must not clear the whole strip"
            );
        }
    }
}
