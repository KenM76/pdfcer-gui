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

use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::PickFilter;
use crate::canvas::selection::{Selection, SelectionState};
use crate::canvas::tool::CanvasTool;

/// `egui::Memory` key for *did the press that began this gesture change what is
/// selected?*
///
/// Frame-local gesture state, kept where [`crate::canvas::input`]'s header says
/// gesture state belongs and for its reason: it has no meaning across a
/// document, and a value that survived one would be a claim about a file it was
/// not measured on.
const CHANGED_KEY: &str = "pdfcer-canvas-press-changed"; // ui-text-exempt: internal memory id, never displayed

/// Did the press that began the gesture in flight change the selection?
///
/// ★★★ **The one fact that tells a first click from a second.** A click is a
/// press and a release, and [`at_press`] runs on the press while
/// [`crate::canvas::clicking`] runs on the release — by which time the state
/// they would both read is identical: one object selected, at the Object rung.
/// So *"the operator clicked a block that was already selected"* and *"the
/// operator clicked a block and this press is what selected it"* are the same
/// state, and without this they cannot be told apart.
///
/// They must be, because the chunk rung is entered on the **second** click.
/// Descending on the first would advance two rungs in one gesture: the block
/// would never be selectable, and the boxes O215 ask 3 draws around its chunks
/// would never be on screen when the operator went to aim at one.
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
    // ★★★ **1b. A press on an unselected object selects it — before the
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
    // ★ And `Shift` declines, because a Shift-press is the *extend* gesture and
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
///
/// Three outcomes, in precedence order, and the first two are what O215 ask 1
/// is about.
///
/// # ★★★ 1. A press that is already inside this object stays inside it
///
/// The operator: *"sometimes it moves the chunk and sometimes it takes the
/// whole block."* [`covers`] declined a moment ago, which at the Part rung
/// means the press landed **outside the selected chunk's box** — on a
/// neighbouring chunk, or in the white between two of them. Selecting the whole
/// object there is what promotes the operand back to the block mid-gesture, and
/// the drag that follows moves everything.
///
/// So a press on the object the operator is already inside **re-picks within
/// it** rather than leaving it. The rung is the operator's, and only a press on
/// something else takes them out of it.
///
/// ⚠ Gated on [`crate::canvas::chunks::boxed`] — text, more than one chunk, and
/// the switch on — so it is offered exactly where a box is drawn to aim at. A
/// path's subpaths have no such affordance, and a press on a selected shape at
/// the Part rung goes on behaving as it always has.
///
/// # 2. A press on what is already the whole selection changes nothing
///
/// Re-selecting an object that is already selected alone, at the Object rung,
/// is a write with no effect — but it is not a *report* with no effect, because
/// [`changed_selection`] is what the click path reads to decide whether this is
/// the operator's first click on this block or their second. Saying "changed"
/// here would make the chunk rung unreachable through the gap between two
/// chunks, which is where [`covers`] declines most often on a CAD note.
///
/// # 3. Anything else selects, exactly as it always did
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
        // ★ The test is *membership of the whole set*, never
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
///
/// Part-rung only: at the Node rung `subpath` names the container an anchor
/// lives in, not a chunk, so a node selection would answer this question about
/// a different index space.
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
///
/// **Five** conditions, each with its own reason in
/// [`interact`]'s step 1b: the press **landed on this canvas**, the primary
/// button went down **this frame**, the plain Select tool is armed, the mode
/// may edit content, and no region zoom is waiting to be spent.
///
/// `Shift` declines here rather than at the call site so that the whole
/// predicate reads in one place — a reader asking *"when does a press select?"*
/// gets one answer rather than a function plus a condition beside it.
///
/// # ★★★ The first condition: the press must have landed on this canvas
///
/// `OPERATOR_REQUESTS.md` row **O75**:
///
/// > *"When I am working in the right side panel objects are getting selected
/// > through the side panel when I am trying to edit fields in the Properties
/// > section."*
///
/// Asking the **`Context`** — i.e. the whole window — *"did the primary button
/// go down this frame?"* and then mapping `press_origin()` through the page's
/// affine transform is what produces that. [`crate::viewer::screen_to_page`] is
/// unbounded and unclamped, so **any** screen point converts to a valid page
/// coordinate: a press on a `TextEdit` in the right dock resolves to real page
/// content and replaces the selection the operator was editing the properties
/// of.
///
/// It hides at fit zoom, because there the dock maps off the sheet and the hit
/// test misses. Zoom past fit on a CAD sheet — which is every working session
/// on an A1 drawing — and the whole window maps inside the page.
///
/// ★★ **This is `DEFECTS.md` D1's class, arrived at through the pointer
/// instead of the keyboard**: a guard asking exactly the right question of
/// exactly the wrong object. Every signal in [`interact`]'s `PointerFrame` must
/// come from the page's own [`egui::Response`], never from the `Context`.
///
/// # Why [`egui::Response::is_pointer_button_down_on`] and nothing else
///
/// egui resolves it from `Memory::interaction()`'s `potential_click_id` /
/// `potential_drag_id`, which are assigned with **full layer and z-order
/// awareness**. So one term rejects, at once and with no rectangle
/// arithmetic: a press on either dock, on the ribbon, on the document tab
/// strip, on the status bar, on the find bar's floating `Area`, on a
/// context-menu popup, and on a modal `Window`. A `clip.contains(origin)`
/// test would cover **none** of the last three, because all three sit
/// geometrically *inside* the canvas viewport.
///
/// It is also the term that keeps a legitimate gesture alive: it stays true
/// for the widget that **owns** the press for the whole gesture, so a marquee
/// dragged off the page and out over a dock — which [`interact`]'s step 1
/// explicitly protects — is unaffected.
///
/// ★ **Do not substitute [`egui::Context::egui_wants_pointer_input`].** It is
/// true whenever *any* egui widget wants the pointer, and this canvas's page
/// **is** an egui widget, so it would be true during a legitimate canvas press
/// and would suppress selection entirely. It is the pointer twin of the D1
/// trap and swapping one for the other would trade a wrong selection for no
/// selection. (`Context::is_pointer_over_area` does not exist in egui 0.35;
/// it was looked for three ways before this was written.)
///
/// # Why BOTH terms, and not just the new one
///
/// `is_pointer_button_down_on` is true for **every frame the button is held**,
/// not only the frame it went down. `button_pressed` is the this-frame edge.
/// Collapsing them into one would re-run the pick on every frame of a drag —
/// a different defect, and a worse one, because it would re-select mid-move.
///
/// # The one narrowing this accepts, recorded rather than left to be found
///
/// `response` is the **acting** page's response — the page
/// [`crate::canvas::present`] chose as `active`, which is also the page
/// `map` describes and the page `page_index` names. In a continuous strip a
/// press on a *neighbouring* page therefore no longer selects. That is not a
/// regression: such a press was already being mapped through the acting
/// page's transform, so it was already resolving to a point on the wrong
/// sheet. It is now refused instead of answered wrongly. If per-page pressing
/// is wanted in the strip, the fix is to make the pressed page the acting
/// page, not to widen this guard.
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

/// ★★★ **Whether the current selection already claims this point** — its body,
/// its eight resize grips, or its rotate handle.
///
/// # The guard that keeps a press on an existing selection from re-selecting
///
/// It asks `handles::grip_at` against the box
/// [`crate::canvas::pressing::grabbable`] answers with — the **same** two
/// functions [`crate::canvas::pressing::look`] asks a moment later — rather
/// than testing the selection's entries. A second opinion computed differently
/// here would disagree with the gesture machine at the margins, and every
/// disagreement is a press that selects when it should have transformed.
///
/// # ★★ Why the grips, and not just the box
///
/// **The rotate handle sits OUTSIDE the box** — `handles::rotate_rect` puts it
/// above the top edge — so a press on it is not "covered" by the body. Testing
/// `grip_box.contains(point)` alone leaves any object underneath the handle to
/// be selected by the body below, and the rotate becomes a select-and-move.
///
/// The eight resize grips are inside the box and are never at risk from that,
/// but they go through the same call anyway rather than through an argument
/// that they are safe: an argument is a thing that stops being true.
///
/// A working gesture aimed at the wrong verb is this canvas's recurring failure
/// mode, and it is worth naming because it never *looks* broken from a chair —
/// something moves.
///
/// # ★★★ The box must come from `pressing::grabbable`, not `overlay::grip_box`
///
/// `overlay::grip_box` derives its answer from the selection's cached
/// **content** outlines, which `select_annot` clears — an annotation is not
/// content and has nothing decomposed to cache. Ask it about a markup or a ce
/// dimension and it answers `None`, `covers` is **false**, and the press falls
/// into the select-on-press body below — which picks the topmost *content*
/// object at that point and **replaces the annotation selection with it**,
/// twenty points away from the shape the operator was aiming at. Then
/// `pressing::look`, on the very next statement, finds a content selection and
/// the release rotates a page object: a perfect gesture, on something the
/// operator never selected.
///
/// `pressing::grabbable` is the one function that knows all four kinds of
/// grabbable box — page content, a markup, a ce dimension and a form field's
/// widget — which is why it is the one called here.
///
/// ★ **The rule is about phrasing, not about this call.** "The same two
/// functions `pressing::look` asks" is a claim about a call site somewhere
/// else, held together by nothing. A guard that must agree with another module
/// has to **call that module**, not resemble it.
///
/// # ★★★ The companion rule: order against the fork
///
/// **A guard written in one destination's vocabulary must stand AFTER the
/// branch that picks the destination, never before it.** Three destinations
/// share the rotate gesture and four share a press; a content-shaped test such
/// as `selection.object_indices_on(page).is_empty()` placed in front of the
/// fork answers about a subject the gesture may already have routed away from,
/// and returns before the routing decision is reached at all.
/// `canvas::rotating`'s header carries the same rule for its own fork.
///
/// # ★ `GripSet::all()` here, where `pressing::look` narrows it
///
/// `look` passes `grabbable`'s own `offer`, which is narrower for three of the
/// four kinds. This passes `all()` unconditionally, and the difference errs
/// toward **declining**: this answers `Some(grip)` for points `look` will call
/// `None`, so the press falls through to the gesture machine unchanged instead
/// of re-selecting under a node the operator is in the middle of editing.
/// Erring the other way would be a press that silently leaves node editing.
///
/// ★★ It matters in the new direction too. A selected **ce dimension** is
/// offered `GripSet::rotate_only()`, so `look` will not call a corner press a
/// resize — but `all()` here still claims that corner for the *existing
/// selection*, which is right: whatever the press turns out to mean, it is
/// about the dimension the operator already has, not about the linework
/// underneath it.
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
    // ★★★ **ONLY `Grip::Move` is second-guessed.**
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
    // ★ The eight grips and the handle are DRAWN. The operator can see them,
    // and a press on one is unambiguous. `Grip::Move` is the only member with
    // no visible affordance of its own — it is "anywhere inside" — which is
    // exactly why it is the one that can be claimed by mistake, and the only
    // one worth asking a second question about.
    if !matches!(grip, crate::canvas::handles::Grip::Move) {
        return true;
    }
    // ★★★ **…and for page CONTENT, inside the box is not the same as on the
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
    // ★★ It calls `pressing::body_under` rather than asking its own version.
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
            // ★ As in `pressing::look`: the same scope the click resolves in.
            crate::canvas::smart::scope(ctx, page_index),
        )
}
