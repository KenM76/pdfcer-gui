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
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/input.md`.

use egui::{Pos2, Vec2};

use crate::canvas::gesture::GestureState;
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::{PickClass, PickFilter};
use crate::canvas::selection::{ClickHit, SelectionState};
use crate::canvas::target::{CanvasTargetProvider, TargetId};
use crate::canvas::tool::CanvasTool;

/// `egui::Memory` key for the in-flight pointer gesture.
const GESTURE_MEMORY_KEY: &str = "pdfcer-canvas-gesture"; // ui-text-exempt: internal memory id, never displayed

/// Ask the provider what is under a click, at every rung at once.
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
    // **A wider radius for an ANCHOR, and only for an anchor** —
    // `OPERATOR_REQUESTS.md` O69: *"the nodes are hard to see and click on."*
    //
    // Eight screen pixels rather than six, which is what a Bézier control
    // point already got — so an anchor stops being harder to hit than the
    // handle hanging off it — and what Inkscape's grab sensitivity defaults to.
    //
    // It is used for `nearest_node` alone. `nth_allowed` (object picking)
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
    // The two deeper rungs are gated by the SAME filter, and switching a
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
                // The wider radius, here and nowhere else. See its binding above.
                .and_then(|p| {
                    targets.nearest_node_of(page_index, target, p, point, node_tolerance)
                });
            (part, node)
        }
        _ => (None, None),
    };
    // `chunk` is NOT answered here. It asks whether the part is a text line
    // the operator can see a box around, which needs the document's line count
    // and the `View ▸ Text chunks` preference — a `CanvasTargetProvider` has
    // neither, and this function has no `Context`. `canvas::clicking` fills it
    // through `chunks::boxed` on the very next statement after this returns;
    // the field's own doc carries the argument.
    ClickHit {
        object,
        part,
        node,
        chunk: false,
    }
}

/// The front-most target at `point` whose CLASS the operator has left
/// switched on.
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
        // **The Smart-Selector substitution happens HERE**, before the
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
        // **Deduplicated, and that is not tidiness.** Ten leaves of one
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
    // **A DIRECT HIT BEATS A NEAR MISS** — the last step, and the one that
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

/// **Put the candidates the pointer is genuinely ON in front of the ones it
/// is merely NEAR**, each group keeping its own front-to-back order.
fn direct_hits_first(
    targets: &dyn CanvasTargetProvider,
    page_index: usize,
    point: Pos2,
    scope: crate::canvas::smart::Scope,
    candidates: Vec<TargetId>,
) -> Vec<TargetId> {
    // Resolved through the SAME Smart-Selector substitution the first query
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

/// **Which of the candidates under the pointer this click means**, given
/// how many times the operator has asked to go deeper at this same point.
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

/// **The frontmost object under a point**, after the pick filter — the plain
/// answer, with no selection and no cycling depth involved.
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
pub(crate) fn abandon_gesture(ctx: &egui::Context) -> bool {
    let had_one = load_gesture(ctx).active().is_some();
    if had_one {
        store_gesture(ctx, GestureState::default());
    }
    had_one
}

/// The pointer movement of an in-progress pan over this canvas, or `None` when
/// no pan is happening.
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

/// **The direct-hits-first rule, in the three states it can be in.**
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
    use crate::canvas::{target::CanvasTargetProvider, targetstub::StubTargets};
    use crate::panels::objects::provider::TargetId;

    /// A rect from its top-left corner and size, in canvas units.
    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h))
    }

    /// **His title block, reduced to two rectangles.**
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

    /// **The re-rank moves candidates; it never loses one.**
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
