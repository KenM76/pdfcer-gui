//! # `canvas::moving` — dragging a selection, and the four things that make it honest
//!
//! ## What this module is for
//!
//! A press inside the selection, a drag, a release: the object moves. That is
//! one sentence and four separate obligations, and this module exists because
//! each of them is a place the gesture can go quietly wrong.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/moving/mod.md`.
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

use crate::app::actions::Action;
use crate::canvas::gesture::Phase;
use crate::canvas::selection::{SelectionLevel, SelectionState};
use crate::panels::objects::provider::{ObjectModelProvider, PartKind, RunMoveBlock};
use crate::viewer;

/// A drag displacement in **PDF page space** — the frame every `move_*` verb
/// consumes.
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
    #[must_use]
    pub fn is_travel(self) -> bool {
        self.dx.is_finite() && self.dy.is_finite() && (self.dx != 0.0 || self.dy != 0.0)
    }
}

/// Which core verb a completed move drag on this selection would reach, with
/// its operands already resolved.
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
    /// Page objects and parts of placed drawings selected together, every
    /// one moved by `VectorAction::MoveMixed` as one undo step.
    Mixed {
        /// The 0-based page.
        page: usize,
        /// Paint-order indices, ascending and unique.
        objects: Vec<usize>,
        /// Leaf indices grouped by the drawing each is directly inside,
        /// none inside one of `objects` (moving the drawing moves them).
        leaves: Vec<Vec<usize>>,
    },
    /// **The Object rung for things painted INSIDE a form XObject**:
    /// `move_objects_in_form`.
    ///
    ///
    /// ## A separate variant, not `Objects` with leaf indices in it
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
    /// The Part rung **inside a form XObject**: `move_subpath_in_form`.
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
    /// The Node rung inside a form, one anchor: `move_node_in_form`.
    NodeInForm {
        /// The page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The anchor, object-scoped.
        node: usize,
    },
    /// The Node rung inside a form, several anchors: `move_nodes_in_form`.
    NodesInForm {
        /// The page.
        page: usize,
        /// The enclosing object, by **leaf** index.
        leaf: usize,
        /// The anchors, object-scoped, ascending and unique.
        nodes: Vec<usize>,
    },
    /// **The Part rung of a TEXT object: one visual line.**
    ///
    /// The engine's verb is `move_text_run`, which addresses one show
    /// operator; a producer may write one visual line as many of those. This
    /// variant names the line, and `VectorAction::MoveTextLine` translates it
    /// into the run range and calls the verb once per run.
    ///
    /// # Reaching this variant is CONDITIONAL, unlike every sibling here
    ///
    /// [`eligible`] only builds it when
    /// [`ObjectModelProvider::text_line_move_refusal_of`] answers `None` —
    /// which is the engine's own `text_run_move_refusal` asked of every run in
    /// the line. A line whose first piece takes its origin from the preceding
    /// run's advance (9.4.2) has no coordinate to rewrite; one whose *interior*
    /// piece does would be torn in half. Either refuses with
    /// [`Refusal::TextRunCannotMove`] *before* a ghost is drawn, so obligation
    /// 3 in the module header holds exactly.
    TextLine {
        /// The page.
        page: usize,
        /// The enclosing text object, by paint-order index.
        object: usize,
        /// The visual line, numbered as
        /// [`ObjectModelProvider::text_line_count_of`] counts — **not** a
        /// show-operator index.
        line: usize,
    },
    /// **The Part rung of a text object INSIDE a form XObject**:
    /// `move_text_run_in_form`, once per run in the line.
    ///
    /// This is the variant O188 is actually about: on the operator's
    /// SolidWorks sets the title block *is* a form XObject, drawn on every
    /// sheet, so a page-scoped verb alone would answer his request everywhere
    /// except where he asked it.
    ///
    /// ⚠ **One call changes every sheet the form is drawn on.** The form's
    /// stream is shared, so this is not a per-page edit wearing a per-page
    /// address — see `VectorAction::MoveTextLineInForm` for what the shell
    /// owes the operator about that.
    TextLineInForm {
        /// The page the leaf is on.
        page: usize,
        /// The enclosing text object, by **leaf** index.
        leaf: usize,
        /// The visual line, numbered as
        /// [`ObjectModelProvider::text_line_count_of`] counts.
        line: usize,
    },
    /// **Several chunks of one text object, moved by one drag.**
    ///
    /// The plural twin of [`Self::TextLine`], reached when the operator has
    /// Shift- or Ctrl-clicked more than one chunk inside a block. Every line
    /// travels by the same delta, and the whole set commits as **one undo
    /// entry** — `VectorAction::MoveTextLines` translates each line to its run
    /// range, calls `move_text_run` once per run, and coalesces the lot.
    ///
    /// # Why this is a second variant and not `TextLine` with a `Vec`
    ///
    /// [`Self::Nodes`]' argument, one rung up: the singular case is a member
    /// and the plural case is a set, and which of the two the shell is holding
    /// decides what it owes the operator before the drag starts. The engine's
    /// move guard is asked of **every** line here rather than of the entered
    /// one, so a set containing one unmovable chunk refuses whole — see
    /// [`MoveContext::run_move`]. A `Vec` on the singular variant would have
    /// made that distinction an `if` in the ghost-drawing path instead of a
    /// thing the type says.
    ///
    /// ⚠ **The whole set refuses, or the whole set moves.** Moving the chunks
    /// that qualify and leaving the rest behind would read as a rendering
    /// fault rather than as a refusal — `move_objects`' own contract, applied
    /// at this rung by hand because the engine has no plural run verb (`G030`).
    TextLines {
        /// The page.
        page: usize,
        /// The enclosing text object, by paint-order index.
        object: usize,
        /// The visual lines, numbered as
        /// [`ObjectModelProvider::text_line_count_of`] counts, ascending and
        /// unique.
        ///
        /// Never empty and never of length one — [`eligible`] produces
        /// [`Self::TextLine`] for the singular case, so a reader of this
        /// variant may assume two or more without checking.
        lines: Vec<usize>,
    },
    /// **Several chunks of one text object INSIDE a form XObject.**
    ///
    /// [`Self::TextLines`] for a leaf, and it inherits
    /// [`Self::TextLineInForm`]'s warning: the form's stream is shared, so one
    /// commit changes every sheet the form is drawn on.
    TextLinesInForm {
        /// The page the leaf is on.
        page: usize,
        /// The enclosing text object, by **leaf** index.
        leaf: usize,
        /// The visual lines, ascending and unique, never fewer than two.
        lines: Vec<usize>,
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
    /// # Why this is a second variant and not `Node` with a `Vec`
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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MoveContext {
    /// The paint-order index of the first selected object `move_objects`
    /// refuses on its kind — an image — asked of the engine's own guard,
    /// `object_move_refusal`. Paths and text objects move by operand rewrite.
    ///
    /// `move_objects` refuses the WHOLE call over a single such member rather
    /// than moving the rest and leaving an image behind — a partial application that would read as
    /// a rendering fault rather than as a refusal. The *index* is carried
    /// rather than a bare `bool` because the engine's own error carries it for
    /// exactly this purpose: a refusal that cannot say which object refused is
    /// a refusal the operator cannot act on.
    pub non_path: Option<usize>,
    /// What kind of part the entered object decomposes into, at the Part and
    /// Node rungs. `None` for an object with no Part rung at all (an image).
    pub part_kind: Option<PartKind>,
    /// **Whether the engine would refuse to move any SELECTED chunk**,
    /// and why — `None` when every one of them would move, or when the
    /// selection is not text at all.
    ///
    /// Asked of the whole set rather than of the entered chunk, so a
    /// multi-chunk drag refuses whole instead of drawing a ghost over four
    /// lines and committing three. See [`run_move`] for that argument.
    ///
    /// Carried here, alongside [`Self::part_kind`], for the reason the whole
    /// struct exists: [`eligible`] must stay a pure function of *what is
    /// selected* and *what the model says about it*, so that the tests which
    /// prove it never have to build a decomposition. The decomposition-shaped
    /// question is asked once, by [`context`], which owns the provider.
    ///
    /// It is the ENGINE's answer, not this crate's —
    /// [`ObjectModelProvider::text_line_move_refusal_of`] calls the same
    /// function `plan_move_text_run` runs first. That is what makes the ghost
    /// and the commit structurally unable to disagree; see [`RunMoveBlock`] for
    /// the argument in full, and for the one place this crate still keeps a
    /// hand-rolled copy of a rule like this.
    pub run_move: Option<RunMoveBlock>,
    /// Selected parts of placed drawings beside selected page objects, not
    /// inside any of them, grouped by drawing; empty at a deeper rung or
    /// when no page object is selected.
    pub beside: Vec<Vec<usize>>,
}

/// Convert a **canvas-space** drag delta into a **PDF page-space** one.
///
/// # Why this is the only zoom-safe way to do it, and why no zoom appears
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

/// Gather what the object model says about the selection a move would act on.
fn context(
    selection: &SelectionState,
    page: usize,
    provider: Option<&ObjectModelProvider>,
) -> Option<MoveContext> {
    let provider = provider?;
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
    //
    let entry = selection.entered_object();
    let entered = entry.map(|e| e.object);
    Some(MoveContext {
        non_path: selection.object_indices_on(page).into_iter().find(|&i| {
            provider
                .page_objects()
                .objects
                .get(i)
                .is_none_or(|o| pdfcer_core::vector::edit::object_move_refusal(o, i).is_some())
        }),
        part_kind: entered.and_then(|target| provider.part_kind_of(target)),
        run_move: run_move(selection, page, provider),
        beside: beside(selection, page, provider),
    })
}

/// [`MoveContext::beside`]: the selected parts not carried by a selected
/// drawing, one group per drawing because the engine moves one drawing's
/// parts per call.
fn beside(
    selection: &SelectionState,
    page: usize,
    provider: &ObjectModelProvider,
) -> Vec<Vec<usize>> {
    let objects = selection.object_indices_on(page);
    let leaves = selection.leaf_indices_on(page);
    if selection.level() != SelectionLevel::Object || objects.is_empty() || leaves.is_empty() {
        return Vec::new();
    }
    let model = provider.page_objects();
    let (kept, _) = crate::panels::align::leftout::uncarried(model, &objects, leaves);
    crate::panels::align::leftout::by_drawing(model, kept, |&leaf| leaf)
}

/// **The engine's set move guard, asked of every run of every selected
/// chunk at once** — the guard `EditSession::move_text_runs` runs — once per
/// frame of the drag.
fn run_move(
    selection: &SelectionState,
    page: usize,
    provider: &ObjectModelProvider,
) -> Option<RunMoveBlock> {
    if selection.level() != SelectionLevel::Part {
        return None;
    }
    let object = selection.entered_object()?.object;
    let lines = selection.selected_parts_on(page, object);
    if lines.is_empty() {
        return None;
    }
    let runs: Vec<usize> = lines
        .into_iter()
        .filter_map(|line| provider.text_line_runs_of(object, line))
        .flatten()
        .collect();
    provider.text_runs_move_refusal_of(object, &runs)
}

/// A leaf index as the provider addresses it.
const fn leaf_target(leaf: usize) -> crate::panels::objects::provider::TargetId {
    crate::panels::objects::provider::TargetId::Leaf(leaf as u64)
}

/// The entered anchor's current page-space position, or `None` if the object's
/// anchor list no longer holds that index.
fn node_point(provider: &ObjectModelProvider, object: usize, node: usize) -> Option<Point> {
    provider
        .object_node_points(object)
        .into_iter()
        .find(|(index, _)| *index == node)
        .map(|(_, point)| point)
}

/// **What one frame of a move drag gives the painter.**
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MovePreview {
    /// The canvas-space displacement, for the bounding ghost.
    pub ghost: Option<Vec2>,
    /// The selection's own geometry at its new position, in page space.
    pub shape: Option<crate::canvas::shapes::ShapePreview>,
    /// **The same geometry, on the frame the gesture RELEASED**, for the
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
        // THE LIVE SHAPE, `OPERATOR_REQUESTS.md` O63.
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
        // The bounding ghost is returned as well, not instead. It is the
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
        (MoveSubject::NodeInForm { leaf, node, .. }, Some(provider)) => provider
            .object_node_points_of(leaf_target(*leaf))
            .into_iter()
            .find_map(|(index, point)| (index == *node).then_some(point)),
        _ => None,
    };
    // The plural rung needs every anchor's position, and that is the allocation
    // the singular rung's comment above is at pains to avoid — 6,681 anchors on
    // one measured CAD export. Asked for once, on release, and only when the
    // selection actually holds more than one node: the cost is paid by the
    // gesture that needs it and by no other.
    let points = match (&subject, provider) {
        (MoveSubject::Nodes { object, .. }, Some(provider)) => provider.object_node_points(*object),
        (MoveSubject::NodesInForm { leaf, .. }, Some(provider)) => {
            provider.object_node_points_of(leaf_target(*leaf))
        }
        _ => Vec::new(),
    };

    // Cloned before `action` consumes it, so the hold below describes the
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
            // A refused release holds NOTHING. The document did not change,
            // so there is nothing for a preview to be true about, and holding
            // one would show the operator a move that was declined — the
            // "placeholder" failure R9 forbids, in its most misleading form.
            return MovePreview::default();
        }
    }
    // HOLD IT. `OPERATOR_REQUESTS.md` O63, third piece.
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
    // Built from the SAME `delta` and the SAME `subject` the Action carries,
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
fn decline(selection: &SelectionState, reason: Refusal, actions: &mut Vec<Action>) {
    // **Two refusals out of the eleven have something to say to the
    // operator, and they are the two you meet without having made a mistake.**
    //
    // Which two, and why, is [`Refusal::worded`]'s subject and the argument
    // lives there rather than here — beside the exhaustive match that makes
    // adding a twelfth refusal a compile error until somebody decides.
    //
    //
    // Recorded from the CANVAS, which no other decline in this application
    // does. It is sound for the reason `status::decline`'s header gives for the
    // store being a thread-local at all: `eframe`'s update loop is one thread,
    // the writer and the reader are the same thread, and this changes no
    // document. And the retirement rule still holds — `retire` runs at the top
    // of `dispatch_command`, so the operator's next command ends the sentence,
    // and `Declined::InsideForm::still_true` ends it the moment they select
    // something that is not in a form.
    // Raised as an **action**, not written to the decline store directly.
    //
    // The store is `pub(super)` inside `crate::app` on purpose — *"a decline is
    // written by the one dispatcher and read by the one bar"* — and widening it
    // so the canvas could reach in would trade a real boundary for four fewer
    // lines. It would also cut against this crate's grain everywhere else: a
    // panel that wants to change something **asks**, because it holds
    // `&OpenDoc` and not `&mut`, and `Action` is that channel. The canvas has
    // the same relationship to the application state and gets the same answer.
    if let Some(what) = reason.worded() {
        actions.push(Action::DeclineOnCanvas(what));
    }
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-move-declined level={:?} sel={} reason={} detail={reason:?}",
            selection.level(),
            selection.len(),
            reason.token(),
        )
    });
}

/// **The keyboard's way of asking for the same move** — the arrow keys.
pub(crate) mod nudge;
mod refusal;
mod subject;

pub use refusal::Refusal;
pub use subject::{action, eligible};

#[cfg(test)]
mod tests;
