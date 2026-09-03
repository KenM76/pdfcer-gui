//! # `canvas::input` — reading one frame's pointer: what it landed on, what it is panning, and where the gesture is kept
//!
//! ## Why this is a module rather than four functions at the bottom of [`super`]
//!
//! Rule R2's 1,500-line ceiling forced a split when the rulers landed, and
//! this is the seam it forced — the same way it produced [`super::trace`] when
//! Phase 4 added the strip, and [`super::strip`] alongside it. Both of those
//! headers record that the forced seam turned out to be a real one, and so
//! does this.
//!
//! Everything here answers **"what is the pointer doing this frame?"**, and
//! every one of them is a question with a single, local answer:
//!
//! | function | question |
//! |---|---|
//! | [`probe`] | what a click landed on, at every rung of the selection ladder at once |
//! | [`pan_delta`] | whether *either* of the two panning gestures is in flight, and how far it moved |
//! | [`load_gesture`] / [`store_gesture`] | where the in-flight press lives between frames |
//!
//! What is left behind in [`super`] answers a different question — *how is the
//! frame composed?* — and it is a question about layout, the scroll area, the
//! strip and the order the overlay is painted in. Nothing here needs any of
//! that: [`probe`] needs a provider and a mapping, [`pan_delta`] needs an
//! input state and a rect, and the two `Memory` accessors need a `Context`.
//!
//! ## The one thing that is still in `egui::Memory`, and why
//!
//! [`GESTURE_MEMORY_KEY`]. The selection moved off `Memory` and onto
//! `OpenDoc` at stage S4 because it is **document-scoped** state and `Memory`
//! outlives documents; the argument, and the address-as-identity hazard that
//! came with the workaround, are in [`crate::app::state::OpenDoc::selection`].
//!
//! A gesture is the opposite case and it is worth being explicit about why.
//! The drag that is happening *right now* is genuinely frame-local UI state.
//! It has no meaning across a document, and a gesture that survived one would
//! be a drag continuing over a file it did not start on. Keying it in `Memory`
//! means it cannot: `Memory` is per-`Context`, and every document change
//! starts the next frame with no press in flight — by construction, with
//! nothing to compare and nothing to forget.

use egui::{Pos2, Vec2};

use crate::canvas::gesture::GestureState;
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::{PickClass, PickFilter};
use crate::canvas::selection::{ClickHit, SelectionState};
use crate::canvas::target::{CanvasTargetProvider, TargetId};
use crate::canvas::tool::CanvasTool;

/// `egui::Memory` key for the in-flight pointer gesture.
///
/// ★ **The one thing that stayed in `Memory` when the selection left**, and
/// the distinction is the point rather than an omission — see this module's
/// header.
const GESTURE_MEMORY_KEY: &str = "pdfcer-canvas-gesture"; // ui-text-exempt: internal memory id, never displayed

/// Ask the provider what is under a click, at every rung at once.
///
/// # Why the part and node queries are scoped to the ENTERED object
///
/// Because that is what makes the deeper rungs predictable. A node query
/// against an object's whole flat anchor list is the hazard decision 028 found
/// already shipped: one measured CAD object holds **6,681 anchors**, so "the
/// nearest anchor to the press" can easily belong to a subpath the operator is
/// not pointing at, with nothing drawn beforehand to say which.
///
/// When nothing is entered yet, the subject is the object under the pointer —
/// which is what a double-click needs, since it descends into whatever it
/// landed on.
#[allow(
    clippy::too_many_arguments,
    reason = "eight independent facts about one click — the provider, the selection, the page, the point, the mapping, the filter, the cycling depth and the container scope. Grouping any subset would be grouping by arity rather than by meaning, and the resulting type would have no name that was true." // ui-text-exempt: a lint justification, never displayed
)]
pub(super) fn probe(
    targets: &dyn CanvasTargetProvider,
    selection: &SelectionState,
    page_index: usize,
    point: Pos2,
    map: &PageMapping,
    filter: PickFilter,
    depth: usize,
    scope: crate::canvas::smart::Scope,
) -> ClickHit {
    // ONE tolerance, converted once, in page units. Passing
    // `SELECT_SCREEN_TOLERANCE_PX` here would compile, run, and merely drift
    // with zoom — see `mapping`.
    let tolerance = map.tolerance();
    // ★★★ **A wider radius for an ANCHOR, and only for an anchor** —
    // `OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*
    //
    // Eight screen pixels rather than six, which is what a Bézier control
    // point already got — so an anchor stops being harder to hit than the
    // handle hanging off it — and what Inkscape's grab sensitivity defaults to.
    //
    // ★★ It is used for `nearest_node` alone. `nth_allowed` (object picking)
    // and `part_hits` keep the shared radius, so a press on a sheet this
    // project has measured at 129,758 objects still resolves to the same
    // object it did before. Widening the shared constant would have changed
    // the answer to *"what did I click?"* everywhere in order to make one
    // rung easier, which is the trade this refuses.
    let node_tolerance = map.node_tolerance();
    let object = nth_allowed(targets, page_index, point, tolerance, filter, depth, scope);

    // ★★★ **BOTH INDEX SPACES, as of 2026-09-01** — `OPERATOR_REQUESTS.md` O70.
    //
    // This read `.and_then(TargetId::page_object_index)`, with a comment saying
    // the deeper rungs *"are simply not offered"* for a target inside a form
    // XObject — *"the ladder stopping at the Object rung for a leaf, expressed
    // where the address space runs out"*. That was true and is the clearest
    // kind of limitation: structural, stated, and impossible to forget.
    //
    // The address space stopped running out. `part_hits_of` and
    // `nearest_node_of` take the `TargetId` itself, and `provider::geometry`
    // answers both from whichever list it names — so the subject is the target,
    // not a page index, and the ladder goes as deep inside a container as it
    // does outside one.
    let subject = selection.entered_object().map(|e| e.object).or(object);
    // ★ The two deeper rungs are gated by the SAME filter, and switching a
    // rung off is not the same act as switching an object class off — it
    // changes how deep a click may go rather than what it may reach. With
    // `Parts` off, a double-click stops descending and the sheet behaves like
    // a diagram of whole objects; with `Points` off, no anchor is ever picked
    // and none is offered as a drag target.
    //
    // Node is gated behind Part deliberately rather than independently: an
    // anchor is addressed as `(part, node)` and there is no way to name one
    // without its subpath. Allowing Points while forbidding Parts would be a
    // state the address space cannot express, so it resolves the only way it
    // can — no part, therefore no node.
    let (part, node) = match subject {
        Some(target) if filter.allows(PickClass::Part) => {
            let part = targets
                .part_hits_of(page_index, target, point, tolerance)
                .first()
                .copied();
            let node = part
                .filter(|_| filter.allows(PickClass::Node))
                // ★ The wider radius, here and nowhere else. See its binding above.
                .and_then(|p| {
                    targets.nearest_node_of(page_index, target, p, point, node_tolerance)
                });
            (part, node)
        }
        _ => (None, None),
    };
    ClickHit { object, part, node }
}

/// The front-most target at `point` whose CLASS the operator has left
/// switched on.
///
/// # ★ Why this is not `hit_test` with a predicate bolted on
///
/// [`CanvasTargetProvider::hit_test`] is defined as the head of
/// [`CanvasTargetProvider::hit_test_all`], and that definition is load-bearing:
/// it is what makes *"what does a plain click select?"* and *"what does
/// cycling step through?"* structurally the same answer rather than a
/// convention two implementations have to keep in step.
///
/// A filter must not break that. So it walks the same depth-ordered list and
/// takes the first **allowed** entry, which keeps the two answers derived from
/// one query — and, as a side effect, is exactly the traversal a future
/// "select the object underneath" needs.
///
/// # A provider that cannot classify lets everything through
///
/// [`CanvasTargetProvider::object_class`] returns `None` for a target it does
/// not know, and the default implementation returns `None` for every target.
/// `None` means *"I cannot say"* and is treated as ALLOWED.
///
/// Getting that default backwards would be quiet and severe: every test double
/// in the crate uses the default, so treating `None` as forbidden would make
/// every object unselectable in every harness — and the failure would look
/// like a broken hit test rather than a filter, because nothing would name the
/// filter in the output.
fn allowed_candidates(
    targets: &dyn CanvasTargetProvider,
    page_index: usize,
    point: Pos2,
    tolerance: f64,
    filter: PickFilter,
    scope: crate::canvas::smart::Scope,
) -> Vec<TargetId> {
    let mut out: Vec<TargetId> = Vec::new();
    for target in targets.hit_test_all(page_index, point, tolerance) {
        // ★★★ **The Smart-Selector substitution happens HERE**, before the
        // filter and before anything downstream sees a candidate —
        // `OPERATOR_REQUESTS.md` O70.
        //
        // Here rather than at each call site because this is the one function
        // every picking question funnels through, and the press and the click
        // that follows it MUST agree about what is under the pointer: a drag
        // that began on a container and a click that selected a leaf would be
        // one gesture acting on two different objects.
        let target = scope.resolve(targets, page_index, target);
        let allowed = match targets.object_class(page_index, target) {
            Some(class) => filter.allows(class),
            None => true,
        };
        // ★★ **Deduplicated, and that is not tidiness.** Ten leaves of one
        // title block under one point all resolve to the same container, so
        // without this an `Alt`-cycle through the stack would offer the same
        // object ten times and read as a control that has stopped responding.
        // The class is asked of the RESOLVED target, so switching form
        // XObjects off in the pick filter switches off the containers this
        // substitution produces rather than the leaves it produced them from.
        if allowed && !out.contains(&target) {
            out.push(target);
        }
    }
    out
}

/// ★★★ **Which of the candidates under the pointer this click means**, given
/// how many times the operator has asked to go deeper at this same point.
///
/// # The defect this closes
///
/// The operator, 2026-08-26: *"when I click on one of the objects all I get is
/// the page selected."*
///
/// The engine computes the **whole** front-to-back list of what is under a
/// point — `hit_test_all` — and this module called `.find()` on it and threw
/// the tail away. So the front-most candidate was the only reachable one, at
/// every point, for ever. On a page carrying anything page-sized, that one
/// candidate is the answer to every click anywhere.
///
/// ★ The root cause of his complaint is one level below this — the engine does
/// not enter form XObjects, so the objects he is pointing at are not in the
/// list at all, and that is filed as an engine request. **This is the other
/// half**, and it is the half that is ours: even for the objects that ARE in
/// the list, anything underneath anything was unreachable.
///
/// # `depth` and its wrap
///
/// `0` is a plain click and is exactly what `.find()` used to return, so the
/// ordinary gesture is unchanged by construction. Each `Alt`+click at the same
/// point adds one, and the index **wraps** — a fifth `Alt`+click on a stack of
/// four returns to the top rather than sticking at the bottom.
///
/// Wrapping rather than clamping because a cycle the operator can walk out the
/// far side of is a cycle they can get lost in: with no visible list, a control
/// that stops responding is indistinguishable from one that has broken. Coming
/// back round says *"that was all of them"* without a word of copy.
fn nth_allowed(
    targets: &dyn CanvasTargetProvider,
    page_index: usize,
    point: Pos2,
    tolerance: f64,
    filter: PickFilter,
    depth: usize,
    scope: crate::canvas::smart::Scope,
) -> Option<TargetId> {
    let candidates = allowed_candidates(targets, page_index, point, tolerance, filter, scope);
    if candidates.is_empty() {
        return None;
    }
    candidates.get(depth % candidates.len()).copied()
}

/// ★ **The frontmost object under a point**, after the pick filter — the plain
/// answer, with no selection and no cycling depth involved.
///
/// [`probe`]'s narrow sibling, for the one caller that has neither: the
/// press-time selection in [`crate::canvas::interact`] runs *before* anything
/// has decided what the gesture is, so there is no depth to honour and no
/// entered object to keep. It wants the top of the stack and nothing else.
///
/// Depth zero deliberately. `Alt`-cycling is a property of repeated **clicks**
/// at one point (`canvas::clicking`'s `CycleCursor`), and a drag is not a click
/// — an operator pressing to move something means the thing they can see.
pub(super) fn topmost(
    targets: &dyn CanvasTargetProvider,
    page_index: usize,
    point: Pos2,
    map: &PageMapping,
    filter: PickFilter,
    scope: crate::canvas::smart::Scope,
) -> Option<TargetId> {
    nth_allowed(
        targets,
        page_index,
        point,
        map.tolerance(),
        filter,
        0,
        scope,
    )
}

/// How many objects the pointer is over, after the pick filter.
///
/// Read by the status bar so the operator can be told *"3 objects here"* rather
/// than having to discover a stack by cycling into it. Deliberately a count and
/// not the list: a caller that wanted the list would be re-deriving the
/// selection, which is [`probe`]'s job.
pub(super) fn candidate_count(
    targets: &dyn CanvasTargetProvider,
    page_index: usize,
    point: Pos2,
    tolerance: f64,
    filter: PickFilter,
    scope: crate::canvas::smart::Scope,
) -> usize {
    allowed_candidates(targets, page_index, point, tolerance, filter, scope).len()
}

/// Read the in-flight pointer gesture.
pub(super) fn load_gesture(ctx: &egui::Context) -> GestureState {
    let id = egui::Id::new(GESTURE_MEMORY_KEY);
    ctx.data_mut(|d| d.get_temp::<GestureState>(id).unwrap_or_default())
}

/// Write the in-flight pointer gesture back.
pub(super) fn store_gesture(ctx: &egui::Context, gestures: GestureState) {
    let id = egui::Id::new(GESTURE_MEMORY_KEY);
    ctx.data_mut(|d| d.insert_temp(id, gestures));
}

/// **Abandon a gesture in flight without committing it**, reporting whether
/// there was one.
///
/// The programmatic equivalent of the operator pressing Escape mid-drag, and it
/// has exactly one caller: `PdfcerApp::on_mode_capabilities_changed`, honouring
/// `MODES_AND_PANELS.md` rule 1 — *"If a mode change would hide a pending,
/// uncommitted gesture … that gesture is committed or cancelled first."*
///
/// **Cancelled rather than committed**, which is the half of that sentence this
/// function chooses. The operator asked for a mode; they did not ask for the
/// half-drawn rectangle their pointer happens to be holding, and committing one
/// on their behalf would author an annotation nobody typed. Discarding the
/// state is all that is needed for that to be true — a markup is written only
/// by `Action::CommitMarkup`, which none of this raises, and a move ghost is a
/// preview that has changed nothing.
///
/// Written by *replacing* the stored state rather than by driving `update` with
/// a `cancel` frame: there is no frame here to drive it with, and
/// `GestureOutcome::Cancelled` exists to tell the key handler that Escape was
/// spent — a fact with no meaning outside the frame that produced it.
pub(crate) fn abandon_gesture(ctx: &egui::Context) -> bool {
    let had_one = load_gesture(ctx).active().is_some();
    if had_one {
        store_gesture(ctx, GestureState::default());
    }
    had_one
}

/// The pointer movement of an in-progress pan over this canvas, or `None` when
/// no pan is happening.
///
/// **Two buttons, one path.** The middle button always pans — the CAD /
/// Inkscape / Illustrator / browser convention, requested on 2026-08-04 — and
/// the primary button pans as well while the hand tool is active, whether the
/// operator chose it or is borrowing it with the space bar. They share this
/// function and therefore share [`super::geometry::pan_offset`], its clamp and
/// its cursor: `GUI_ROADMAP` 3.2 asks for a hand tool, not for a second
/// panning implementation that rounds differently at the edges of the scroll
/// range.
///
/// Gated on the pointer being over the canvas so a drag that began on some
/// other surface does not yank the page sideways.
///
/// ★ **`ui` is the canvas's own child `Ui`**, whose `max_rect` is the region
/// *inside* the ruler gutters — see [`super::rulers::Gutters::content_ui`].
/// That is what stops a drag begun on a ruler from also panning the page: the
/// gutter is outside this rect, so `over` is false there.
pub(super) fn pan_delta(ui: &egui::Ui, tool: CanvasTool) -> Option<Vec2> {
    let rect = ui.max_rect();
    ui.input(|i| {
        let over = i.pointer.latest_pos().is_some_and(|p| rect.contains(p));
        let panning =
            i.pointer.middle_down() || (tool.pans_with_primary() && i.pointer.primary_down());
        if panning && over {
            let delta = i.pointer.delta();
            (delta != Vec2::ZERO).then_some(delta)
        } else {
            None
        }
    })
}
