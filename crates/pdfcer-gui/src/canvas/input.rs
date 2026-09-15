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
/// # The engine's precision is not uniform across object kinds
///
/// `pdfcer_core`'s `object_hit` scores each kind differently, and the shell
/// inherits the asymmetry whole:
///
/// - **Path** — tested against its ink: bounding-box reject, then fill interior
///   under the object's own winding rule, then stroke/outline proximity.
/// - **Text** — tested against each run's bounds.
/// - **Image** (inline image or image XObject) — tested as its page bounding
///   box inflated by the tolerance. No alpha, no `/SMask`, no clip, no
///   geometry.
///
/// So a mostly transparent picture takes every click inside its rectangle, and
/// it does so at both depths, because page objects and form leaves are scored
/// by the same function. Paint order is back-to-front, so such an image masks
/// everything beneath it.
///
/// A **form XObject** is never a candidate: the deep pick skips
/// `ImageSource::Form` because a form's `/BBox` is an extent declaration rather
/// than ink, and only its leaves compete.
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
    // ★★★ **A DIRECT HIT BEATS A NEAR MISS** — the last step, and the one that
    // makes text reachable on a CAD sheet. The whole argument is on
    // [`direct_hits_first`]; what matters here is that it runs last, on the
    // finished candidate list, so it reorders what the rest of this function
    // decided rather than deciding anything itself.
    //
    // Gated on there being something to reorder. One candidate cannot be
    // reordered and zero candidates is blank paper, so the second engine query
    // is paid for exactly in the ambiguous case and nowhere else — which
    // matters, because this function is also what the status strip calls to say
    // *"3 objects here"*.
    if out.len() > 1 {
        out = direct_hits_first(targets, page_index, point, scope, out);
    }
    out
}

/// The slot [`direct_hits_first`] traces under. One subject, one literal, and
/// it is spelled here rather than inline so it can be grepped from a captured
/// trace back to the code that wrote it.
const DIRECT_SLOT: &str = "canvas-pick-direct";

/// ★★★ **Put the candidates the pointer is genuinely ON in front of the ones it
/// is merely NEAR**, each group keeping its own front-to-back order.
///
/// # The defect this closes, measured on his own drawing
///
/// `OPERATOR_REQUESTS.md` O198, first sentence:
///
/// > *"I really need you to focus on finding ways to make all text editable on
/// > the sw drawing that is in pdftests folder. Find a way to make it happen."*
///
///
/// `crates/pdfcer-gui/tests/ken_sw41177_pick.rs` then asked the engine the same
/// question at four tolerances, and the answer is unambiguous:
///
/// | tolerance | frontmost candidate is text |
/// |---|---:|
/// | 0.0 pt | **9 of 9** |
/// | 8.0 pt | 4 of 9 |
///
/// ★★ The path was not on top of his text. It was winning on **slack**.
/// `pdfcer_core::vector` hits a path on *fill interior, or stroke proximity
/// within half the scaled line width **plus the tolerance***, and hits a text
/// object on *its bounding box inflated by the tolerance*. A title-block label
/// sits a few points from the rules of its own cell, and a tolerance derived
/// from a handful of screen pixels is several points of PDF user space on a
/// 1,584 pt sheet. So the press was inside the text and beside a line, and the
/// line, being painted later, won.
///
/// ★★★ **Every font control, Properties field and restyle verb in this program
/// is reached through a text selection.** A hit test that cannot produce one
/// makes all of them unreachable at once — which is precisely how a capability
/// that is present, registered and green on a fixture reaches the operator as
/// *"that entire area is always greyed out in the menu"*. Claims 1, 3 and 4 of
/// O198 are one defect seen from three surfaces.
///
/// # ★★ Why this is the conventional rule and not an invention
///
/// Every vector editor in the class behaves this way, and it is why none of
/// them needs a modifier to click a label on a busy drawing. Slack exists to
/// make thin things clickable; it is not a claim that a thin thing outranks the
/// thing the pointer is inside. Stated in full, the rule this function enacts:
///
/// > Slack is a tie-breaker of last resort. It may promote a candidate over
/// > **nothing**. It may never promote one over a candidate that needed no
/// > slack at all.
///
/// # ★★ What it deliberately does NOT do
///
/// * **It does not know what a text object is.** The partition is *exact versus
///   inexact*. Text comes out in front on his drawing as a consequence of where
///   he clicked, not because text is special — and a rule that named text
///   would fix his title block while leaving a click on a hatch beside a leader
///   line behaving exactly as it does today.
/// * **It never adds or removes a candidate.** Both groups are subsets of the
///   list that was already going to be returned, concatenated. The count the
///   status strip reports and the length an `Alt`-cycle wraps on are unchanged,
///   so nothing that was reachable before this function existed becomes
///   unreachable after it. That is the same property
///   [`crate::canvas::pick::PickFilter`] states subtractively one layer up.
/// * **It leaves a genuine overlap alone.** If the press is dead-on both the
///   line and the label, both are exact, the partition is a no-op and paint
///   order decides — which is right. You clicked the line.
///
/// # The cost
///
/// One extra `hit_test_all` at tolerance zero, and only when two or more things
/// survive the filter under the pointer. The engine bounds that at one linear
/// pass over the page's objects, which is the same pass the first query already
/// made.
fn direct_hits_first(
    targets: &dyn CanvasTargetProvider,
    page_index: usize,
    point: Pos2,
    scope: crate::canvas::smart::Scope,
    candidates: Vec<TargetId>,
) -> Vec<TargetId> {
    // ★ Resolved through the SAME Smart-Selector substitution the first query
    // used. A raw leaf compared against a resolved container never matches, and
    // the partition would then classify every candidate as inexact — a silent
    // no-op indistinguishable from the rule simply not applying.
    let exact: Vec<TargetId> = targets
        .hit_test_all(page_index, point, 0.0)
        .into_iter()
        .map(|target| scope.resolve(targets, page_index, target))
        .collect();
    if exact.is_empty() {
        // Nothing is dead-on, so every candidate is a near miss and paint order
        // is the only thing left to rank them by. This is a press in the white
        // space between two thin lines, and it must keep behaving as it did.
        return candidates;
    }
    let exact_count = exact.len();
    let was = candidates.first().copied();
    let (mut direct, near): (Vec<TargetId>, Vec<TargetId>) = candidates
        .into_iter()
        .partition(|target| exact.contains(target));
    if direct.is_empty() {
        // Every exact hit was removed by the pick filter. Promoting nothing is
        // the honest outcome: the operator has said he is not interested in
        // that class, and a candidate he has filtered out must not be allowed
        // to reorder the ones that survived.
        return near;
    }
    direct.extend(near);
    if direct.first().copied() != was {
        let head = direct.first().copied();
        // Traced on CHANGE rather than every call. This runs on hover as well
        // as on press — the status strip asks the same question every frame —
        // and an unconditional line here would bury every other trace in the
        // capture the moment the pointer moved.
        crate::diag::trace_changed(DIRECT_SLOT, || {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI.
                "canvas-pick-direct head={} was={} exact={exact_count}",
                name(head),
                name(was)
            )
        });
    }
    direct
}

/// `object:N`, `leaf:N`, or `none` — the spelling
/// [`crate::canvas::trace`] already uses for the same idea, so one capture can
/// be read with one vocabulary.
///
/// ★ Spelled out rather than `{:?}`-formatted. A `Debug` rendering of a target
/// has already made a driven check in this project report the opposite of the
/// truth while quoting the truth in its own message, and the two variants index
/// two different lists in the same document — so which list a number belongs
/// to has to be on the line, not inferred from it.
fn name(target: Option<TargetId>) -> String {
    target.map_or_else(
        // ui-text-exempt: diagnostic trace, never displayed in the UI.
        || "none".to_owned(),
        |t| {
            let list = if t.is_leaf() { "leaf" } else { "object" };
            format!("{list}:{}", t.raw())
        },
    )
}

/// ★★★ **Which of the candidates under the pointer this click means**, given
/// how many times the operator has asked to go deeper at this same point.
///
/// # The defect this closes
///
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

/// ★★★ **The direct-hits-first rule, in the three states it can be in.**
///
/// These exercise [`allowed_candidates`] rather than [`probe`] because the
/// rule is about the ORDER of a list, and `probe` returns only its head. A
/// test that could see the head alone could not tell *"the near miss was
/// demoted"* from *"the near miss vanished"*, and the second would be a
/// selection defect this function is explicitly promising not to introduce.
///
/// # ★★ What is NOT exercised here, stated rather than implied
///
/// The `direct.is_empty()` branch — every exact hit removed by the operator's
/// pick filter, so nothing may be promoted. [`crate::canvas::target::StubTargets`]
/// does not implement `object_class`, so it returns `None` for every target
/// and the filter lets everything through; there is no way to reach that arm
/// through this double. Naming the gap is the point: a reader counting four
/// tests against four branches would otherwise conclude the arm is covered.
/// The live cover for it is the driven check on his own drawing, where the
/// filter is real.
///
///
/// The call in [`allowed_candidates`] was gated behind `false`, the four tests
/// re-run, and the source restored from a file copy. Exactly one went red:
/// `a_direct_hit_outranks_a_near_miss`. The other three stayed green — which is
/// what they are FOR. They assert that the rule changed nothing else, so a
/// plant that turned them red as well would mean they were measuring the
/// re-rank rather than its blast radius, and a suite in which every test fails
/// together cannot tell a regression from a rewrite.
#[cfg(test)]
mod tests {
    use egui::{Pos2, Rect, Vec2};

    use super::allowed_candidates;
    use crate::canvas::pick::PickFilter;
    use crate::canvas::smart::Scope;
    // `CanvasTargetProvider` is in scope for the PREMISE assertions only: each
    // test asks the stub the raw question first, so a failure below cannot be
    // read as the fixture failing to reproduce the ambiguity it was built to
    // reproduce.
    use crate::canvas::target::{CanvasTargetProvider, StubTargets};
    use crate::panels::objects::provider::TargetId;

    /// A rect from its top-left corner and size, in canvas units.
    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h))
    }

    /// ★★ **His title block, reduced to two rectangles.**
    ///
    /// Object 0 is the text run — a small box the pointer lands INSIDE.
    /// Object 1 is the cell rule beside it: a thin strip the pointer lands
    /// four units short of, painted afterwards, which is what makes it
    /// front-most in [`StubTargets::hit_test_all`]'s reversed scan.
    ///
    /// That is the measured shape of the defect, not an invented one: on page
    /// 1 of `SW41177.pdf` the winner was object 5,899 of 5,903 — very nearly
    /// the last thing painted — at eight of nine aims.
    fn label_beside_a_rule() -> StubTargets {
        StubTargets::new(
            0,
            [
                // 0: the text, 20 x 8, spanning x 10..30 and y 10..18.
                rect(10.0, 10.0, 20.0, 8.0),
                // 1: the rule, hairline, at x 32 — two units clear of the
                // text's right edge, so a tolerance of four catches it from
                // INSIDE the text and a tolerance of zero does not. That gap
                // is the whole fixture: make it wider than the tolerance and
                // there is no ambiguity to re-rank, make it zero and the two
                // objects genuinely overlap, which is a different test below.
                rect(32.0, 0.0, 0.5, 40.0),
            ],
        )
    }

    /// The pointer inside the text and merely near the rule: the text wins.
    ///
    /// This is O198's first sentence as an assertion. Without the re-rank the
    /// head here is `Object(1)`, which is what the operator was selecting
    /// every time he clicked a label on that drawing.
    #[test]
    fn a_direct_hit_outranks_a_near_miss() {
        let targets = label_beside_a_rule();
        let point = Pos2::new(29.0, 14.0);

        // The premise first, so a failure below cannot be read as the fixture
        // simply not reproducing the ambiguity: at this tolerance BOTH are
        // candidates, and the rule is the front-most one.
        let both = targets.hit_test_all(0, point, 4.0);
        assert_eq!(
            both,
            vec![TargetId::Object(1), TargetId::Object(0)],
            "the fixture must present the near miss in front, or there is nothing to re-rank"
        );

        let got = allowed_candidates(&targets, 0, point, 4.0, PickFilter::all(), Scope::off());
        assert_eq!(
            got,
            vec![TargetId::Object(0), TargetId::Object(1)],
            "a press inside the text and beside the rule selects the text"
        );
    }

    /// Nothing dead-on: paint order is all there is, and it is left alone.
    ///
    /// A press in the gap between the two — outside the text, outside the
    /// rule, within four of each. Every candidate is a near miss, so promoting
    /// one over another would be inventing a ranking rather than applying one.
    #[test]
    fn nothing_dead_on_leaves_paint_order_alone() {
        let targets = label_beside_a_rule();
        let point = Pos2::new(33.0, 14.0);

        assert!(
            targets.hit_test_all(0, point, 0.0).is_empty(),
            "the premise: at zero tolerance this point is on blank paper"
        );

        let got = allowed_candidates(&targets, 0, point, 4.0, PickFilter::all(), Scope::off());
        assert_eq!(
            got,
            vec![TargetId::Object(1), TargetId::Object(0)],
            "with no exact hit the front-most painted candidate still leads"
        );
    }

    /// A genuine overlap is left to paint order, and that is right.
    ///
    /// The pointer inside a second text box that genuinely crosses the rule.
    /// Both are exact, the partition is a no-op, and the later-painted one
    /// wins — because you clicked the line.
    #[test]
    fn a_genuine_overlap_is_still_decided_by_paint_order() {
        let targets =
            StubTargets::new(0, [rect(10.0, 10.0, 40.0, 8.0), rect(36.0, 0.0, 4.0, 40.0)]);
        let point = Pos2::new(37.0, 14.0);

        assert_eq!(
            targets.hit_test_all(0, point, 0.0).len(),
            2,
            "the premise: at zero tolerance the pointer is on BOTH"
        );

        let got = allowed_candidates(&targets, 0, point, 4.0, PickFilter::all(), Scope::off());
        assert_eq!(
            got,
            vec![TargetId::Object(1), TargetId::Object(0)],
            "two exact hits are ranked by paint order, unchanged"
        );
    }

    /// ★★ **The re-rank moves candidates; it never loses one.**
    ///
    /// Asserted as a multiset over every point on a grid crossing both
    /// objects and the space around them, rather than at one chosen point.
    /// The status strip's *"3 objects here"* count and the wrap length of an
    /// `Alt`-cycle are both this list's length, so a rule that dropped a
    /// candidate would present as a control that had stopped responding
    /// rather than as a hit-test defect.
    #[test]
    fn the_rerank_is_a_permutation_at_every_point_on_a_grid() {
        let targets = label_beside_a_rule();
        let mut ambiguous = 0;
        let mut x = 0.0_f32;
        while x < 48.0 {
            let mut y = 0.0_f32;
            while y < 44.0 {
                let point = Pos2::new(x, y);
                let mut before = targets.hit_test_all(0, point, 4.0);
                let mut after =
                    allowed_candidates(&targets, 0, point, 4.0, PickFilter::all(), Scope::off());
                if before.len() > 1 {
                    ambiguous += 1;
                }
                before.sort_unstable();
                after.sort_unstable();
                assert_eq!(
                    before, after,
                    "the candidate SET must be identical at ({x}, {y}); only its order may change"
                );
                y += 2.0;
            }
            x += 2.0;
        }
        // The grid has to actually visit the interesting case, or this test
        // passes by walking blank paper and asserting that two empty lists
        // match. Measured on the fixture above; update it if the fixture moves.
        assert!(
            ambiguous >= 20,
            "the grid must cross the ambiguous region; it found {ambiguous} points with 2+ candidates"
        );
    }
}
