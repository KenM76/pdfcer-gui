//! # `canvas::presspick` — **the selection catches up with the pointer, at
//! press time**
//!
//! One step, run once per frame, immediately before [`crate::canvas::pressing`]
//! is asked what a press would mean.
//!
//! ## Why it is its own file, and not part of `pressing`
//!
//! `pressing`'s header opens with *"nothing here changes anything"*, and that
//! sentence is load-bearing — it is what lets that module be read as a pure
//! answer to *"what is under the pointer?"*. This step **mutates the
//! selection**. Putting it there would have made a stated contract false, which
//! is worse than having two small files.
//!
//! The canvas's seam is: look, then decide, then act. This is the one "act"
//! that has to happen before the "decide".
//!
//! ## What it is for
//!
//! Every graphics editor selects on **press**. egui's `clicked()` fires on
//! *release*, so a selection made there arrives too late for the gesture the
//! same press starts: `pressing::look` would find an empty selection, no grip
//! under the origin, and start a marquee across the object the operator was
//! trying to drag. Running this first is what makes press-and-drag on an
//! unselected object move it in one gesture. Read [`at_press`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/presspick.md`.

use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::PickFilter;
use crate::canvas::selection::{Selection, SelectionState};
use crate::canvas::tool::CanvasTool;

/// `egui::Memory` key for *did the press that began this gesture change what is
/// selected?*
const CHANGED_KEY: &str = "pdfcer-canvas-press-changed"; // ui-text-exempt: internal memory id, never displayed

/// Did the press that began the gesture in flight change the selection?
#[must_use]
pub(super) fn changed_selection(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::new(CHANGED_KEY)))
        .unwrap_or(false)
}

/// Record what this press did to the selection.
fn note_changed(ctx: &egui::Context, changed: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(CHANGED_KEY), changed));
}

/// Select whatever an unselected press landed on.
///
/// Called from `canvas::interact` step 1b, before `pressing::look`, so that the
/// grip test on the very next statement sees the selection this made.
///
#[allow(clippy::too_many_arguments)]
pub(super) fn at_press(
    ctx: &egui::Context,
    // The **acting page's own** response, and the answer to *"was this press
    // mine?"*. See `press_selects` for `OPERATOR_REQUESTS.md` O75 and why the
    // `Context` cannot answer it.
    response: &egui::Response,
    doc: &OpenDoc,
    selection: &mut SelectionState,
    map: &PageMapping,
    page_index: usize,
    active_tool: CanvasTool,
    caps: Capabilities,
    pick: PickFilter,
    shift: bool,
) {
    // **1b. A press on an unselected object selects it — before the
    // gesture machine is asked what the press means.**
    //
    // The operator: *"if I add an image I Expect to click on it to resize but
    // dragging doesn't resize […] Editing should work like 99% of the graphics
    // programs out there."*
    //
    // # Why it belongs HERE rather than in the gesture machine
    //
    // The gesture machine's rule — *"no grip under the origin, so marquee"* —
    // is right. What it needs is a selection that has already caught up with
    // the pointer. Selecting at press time makes `pressing::look` (called on
    // the very next statement, in this same frame) find `Grip::Move` and
    // produce `DragKind::Move` through the path that already exists and is
    // already tested.
    //
    // That is why this is nine statements rather than a new `DragKind`, a new
    // gesture phase, and an audit of every arm that reads one. Anything that
    // re-answers "what does this press mean" inside the machine is the more
    // invasive shape and buys nothing.
    //
    // # The four things it must not disturb, and how each is held off
    //
    // 1. **A press on empty paper still marquees.** No object under the origin
    //    means no selection is made and nothing below changes.
    // 2. **A press on the CURRENT selection still moves it without
    //    re-selecting.** `grip_box` already contains the origin in that case, so
    //    the guard declines and the existing move runs untouched. This matters
    //    for a *multiple* selection: re-selecting would silently drop it to one
    //    object mid-gesture.
    // 3. **An armed tool keeps its press.** Only the plain Select tool reaches
    //    here — the pen, the caret, the measure tools and the Node tool all own
    //    their press by this codebase's standing rule.
    // 4. **An armed region zoom outranks it**, on the same argument the text
    //    row makes: it is a one-shot the operator armed deliberately from the
    //    ribbon, and it is spent on the next press.
    //
    // And `Shift` declines, because a Shift-press is the *extend* gesture and
    // the click path owns it. Selecting on press would replace the selection the
    // operator was adding to — the same mid-gesture loss as case 2, arrived at
    // from the other direction.
    if press_selects(ctx, response, active_tool, caps, shift)
        && let Some(origin) = ctx.input(|i| i.pointer.press_origin())
    {
        if covers(ctx, doc, map, page_index, selection, origin) {
            note_changed(ctx, false);
            return;
        }
        let point = map.to_page(origin);
        let hit = doc.page_objects().and_then(|provider| {
            crate::canvas::input::topmost(
                &*provider,
                page_index,
                point,
                map,
                pick,
                crate::canvas::smart::scope(ctx, page_index),
            )
        });
        let Some(object) = hit else {
            note_changed(ctx, false);
            return;
        };
        note_changed(
            ctx,
            take(ctx, doc, selection, map, page_index, point, object),
        );
    }
}

/// Make `object` the subject of the press, and answer **whether that changed
/// the selection**.
fn take(
    ctx: &egui::Context,
    doc: &OpenDoc,
    selection: &mut SelectionState,
    map: &PageMapping,
    page_index: usize,
    point: egui::Pos2,
    object: crate::canvas::target::TargetId,
) -> bool {
    if let Some(entered) = selection.entered_object()
        && entered.object == object
        && entered.page == page_index
        && crate::canvas::chunks::boxed(ctx, doc, object)
    {
        // The chunk under the press, or — for a press in the white between two
        // of them — the chunk the operator already had. Staying put is the
        // conservative answer and it is the one that keeps a drag begun just
        // outside a glyph on the chunk it was aimed at.
        //
        // The test is *membership of the whole set*, never
        // `entered.subpath`: [`SelectionState::entered_object`] answers with the
        // FIRST entry and [`SelectionState::select_part`] replaces the entry
        // list outright, so a Shift-built set of four chunks would re-pick — and
        // collapse to one — on a press that landed on any of the other three.
        //
        // `covers` normally claims such a press before this runs, because
        // [`crate::canvas::pressing::body_under`] asks membership of the chunk
        // under the point at this rung. This is what makes the collapse
        // impossible on the paths where it does NOT — a withheld grip box, for
        // one — rather than a second opinion about the same geometry.
        if let Some(part) =
            crate::canvas::chunks::under(doc, page_index, object, point, map.tolerance())
            && !holds_part(selection, page_index, object, part)
        {
            selection.select_part(page_index, object, part, "press");
        }
        return false;
    }
    if selection.level() == crate::canvas::selection::SelectionLevel::Object
        && selection.entries() == [Selection::object(page_index, object)].as_slice()
    {
        return false;
    }
    selection.select_only(page_index, object, "press");
    true
}

/// Whether `part` is already one of the chunks selected on this object.
fn holds_part(
    selection: &SelectionState,
    page: usize,
    object: crate::canvas::target::TargetId,
    part: usize,
) -> bool {
    selection.level() == crate::canvas::selection::SelectionLevel::Part
        && selection.selected_parts_on(page, object).contains(&part)
}

/// Whether a press this frame may select what is under it.
fn press_selects(
    ctx: &egui::Context,
    response: &egui::Response,
    tool: CanvasTool,
    caps: Capabilities,
    shift: bool,
) -> bool {
    response.is_pointer_button_down_on()
        && matches!(tool, CanvasTool::Select)
        && caps.edit_content
        && !shift
        && !crate::canvas::zoom::region_zoom_armed(ctx)
        && ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
}

/// **Whether the current selection already claims this point** — its body,
/// its eight resize grips, or its rotate handle.
fn covers(
    ctx: &egui::Context,
    doc: &OpenDoc,
    map: &PageMapping,
    page_index: usize,
    selection: &SelectionState,
    point: egui::Pos2,
) -> bool {
    let grabbable = crate::canvas::pressing::grabbable(ctx, doc, map, selection);
    let grip = grabbable.bounds.and_then(|f| {
        crate::canvas::handles::grip_at_in(f, point, crate::canvas::handles::GripSet::all())
    });
    let Some(grip) = grip else {
        return false;
    };
    // **ONLY `Grip::Move` is second-guessed.**
    //
    // The O72 downgrade below asks whether the press really landed on the
    // selected object. A press on a RESIZE GRIP or the ROTATE HANDLE does not:
    // those sit on the box's edges and corners, outside the object's own
    // geometry, so `body_under` answers false for every one of them. Applying
    // the downgrade to every grip therefore refuses to cover a press on a grip,
    // `at_press` re-selects whatever is under it, and the resize becomes a
    // select-and-move — two `selection-set … via=press` lines on the trace
    // where there should be one.
    //
    // That is the disagreement `covers`' own header warns about, reached by
    // making this function and `pressing::look` agree on the PREDICATE and not
    // on which grip it applies to.
    //
    // The eight grips and the handle are DRAWN. The operator can see them,
    // and a press on one is unambiguous. `Grip::Move` is the only member with
    // no visible affordance of its own — it is "anywhere inside" — which is
    // exactly why it is the one that can be claimed by mistake, and the only
    // one worth asking a second question about.
    if !matches!(grip, crate::canvas::handles::Grip::Move) {
        return true;
    }
    // **…and for page CONTENT, inside the box is not the same as on the
    // object** — `OPERATOR_REQUESTS.md` O72.
    //
    // For a ce dimension, a markup annotation and a form field the box IS the
    // `/Rect`, so `in_box` is the whole answer and nothing more is asked. For
    // page content the box is `selection.outline_union()` — a rectangle around
    // scattered geometry, mostly empty paper. Without this second question a
    // selected title-block border (a hollow rectangle spanning a CAD sheet)
    // makes every subsequent press on the drawing "already covered", so nothing
    // can be selected and no marquee can be started.
    //
    // It calls `pressing::body_under` rather than asking its own version.
    // This function's header is explicit that a second opinion computed
    // differently here would disagree with the gesture machine at the margins,
    // and every disagreement is a press that selects when it should have
    // transformed. `pressing::look` applies the identical downgrade to
    // `Grip::Move` on the very next statement after this one runs.
    !grabbable.content
        || crate::canvas::pressing::body_under(
            ctx,
            doc,
            selection,
            map,
            page_index,
            point,
            crate::canvas::pick::PickFilter::all(),
            // As in `pressing::look`: the same scope the click resolves in.
            crate::canvas::smart::scope(ctx, page_index),
        )
}
