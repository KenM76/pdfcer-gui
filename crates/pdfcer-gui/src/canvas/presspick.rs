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
//! R2 forced the question on 2026-08-27 (`interact.rs` reached 1,570 lines) and
//! the seam was already drawn: look, then decide, then act. This is the "act"
//! that has to happen before the "decide".
//!
//! ## What it is for
//!
//! Read [`at_press`]. The short version: selection used to happen on the
//! **click**, which in egui means on release, so a press-and-drag on an
//! unselected object could not move it — the operator got a marquee across the
//! thing they were dragging. Every graphics editor selects on press, and the
//! operator said so in those words.

use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::PickFilter;
use crate::canvas::selection::SelectionState;
use crate::canvas::tool::CanvasTool;

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
    // The operator, 2026-08-26: *"if I add an image I Expect to click on it to
    // resize but dragging doesn't resize […] Editing should work like 99% of
    // the graphics programs out there."*
    //
    // # What was actually wrong
    //
    // Selection happened on the **click**, which in egui means on *release*.
    // So a press-and-drag on an object that was not already selected never
    // selected anything: `pressing::look` saw an empty selection, found no grip
    // under the origin, and `press_kind` fell to
    // `(None, None) => Marquee(Select)` — the operator got a rubber band across
    // the thing they were trying to drag, and on release it selected. Two
    // gestures to do what every other editor does in one.
    //
    // # Why it is fixed HERE rather than in the gesture machine
    //
    // Because the gesture machine's answer was never wrong. *"No grip under the
    // origin, so marquee"* is correct — the fault was that the selection had not
    // caught up with the pointer yet. Selecting at press time makes
    // `pressing::look` (called on the very next statement, in this same frame)
    // find `Grip::Move` and produce `DragKind::Move` through the path that
    // already existed and is already tested.
    //
    // That is why this is nine statements rather than a new `DragKind`, a new
    // gesture phase, and an audit of every arm that reads one.
    // `INTERACTION_GAP.md` priced this item as the most invasive of the
    // unblocked set on the assumption it had to be done in the machine.
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
        && !covers(ctx, doc, map, page_index, selection, origin)
    {
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
        if let Some(object) = hit {
            selection.select_only(page_index, object, "press");
        }
    }
}

/// Whether a press this frame may select what is under it.
///
/// **Five** conditions since 2026-08-31, each with its own reason in
/// [`interact`]'s step 1b: the press **landed on this canvas**, the primary
/// button went down **this frame**, the plain Select tool is armed, the mode
/// may edit content, and no region zoom is waiting to be spent.
///
/// `Shift` declines here rather than at the call site so that the whole
/// predicate reads in one place — a reader asking *"when does a press select?"*
/// gets one answer rather than a function plus a condition beside it.
///
/// # ★★★ The fifth condition, and why it was missing for so long
///
/// `OPERATOR_REQUESTS.md` row **O75**:
///
/// > *"When I am working in the right side panel objects are getting selected
/// > through the side panel when I am trying to edit fields in the Properties
/// > section."*
///
/// He is describing this function. Until today it asked the **`Context`** —
/// i.e. the whole window — *"did the primary button go down this frame?"*, and
/// then took `press_origin()` from the same place and mapped it straight
/// through the page's affine transform. [`crate::viewer::screen_to_page`] is
/// unbounded and unclamped, so **any** screen point converts to a valid page
/// coordinate: a press on a `TextEdit` in the right dock resolves to real page
/// content and replaces the selection the operator was editing the properties
/// of.
///
/// It hid at fit zoom, because there the dock maps off the sheet and the hit
/// test misses. Zoom past fit on a CAD sheet — which is every working session
/// on an A1 drawing — and the whole window maps inside the page.
///
/// ★★ **This is `DEFECTS.md` D1's class, arrived at through the pointer
/// instead of the keyboard**: a guard asking exactly the right question of
/// exactly the wrong object. D1 was `egui_wants_keyboard_input()` (= *any*
/// widget focused) where `text_edit_focused()` was meant, and it killed the
/// Delete key. This is the same substitution, and the whole rest of this
/// canvas already gets it right — every other signal in
/// [`interact`]'s `PointerFrame` comes from the page's own
/// [`egui::Response`]. Step 1b was the one that reached past it.
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
/// # ★★ Why the grips, and not just the box — found by driving, within the hour
///
/// The first version tested `grip_box.contains(point)` alone, and
/// `rotate_handle_turns_a_selection` failed on the next driven run. **The
/// rotate handle sits OUTSIDE the box** — `handles::rotate_rect` puts it above
/// the top edge — so a press on it is not "covered" by the body, and with any
/// object underneath this function selected that object and the rotate became a
/// select-and-move.
///
/// A working gesture aimed at the wrong verb, which is the failure mode this
/// canvas has now produced **five** separate times. It is worth naming every
/// time because it never *looks* broken from a chair — something moves.
///
/// # ★★★ THE FIFTH INSTANCE WAS IN THIS FUNCTION, AGAIN — 2026-08-28
///
/// The paragraph above records the fourth: `covers` asked the wrong *question*
/// (the box, not the grips). This one is subtler and had the identical symptom:
/// it asked the right question of **the wrong box**.
///
/// `overlay::grip_box` derives its answer from the selection's cached
/// **content** outlines, which `select_annot` clears — an annotation is not
/// content and has nothing decomposed to cache. So the moment a markup or a ce
/// dimension gained a rotate handle (`Pass 155.0` / `Pass 159.0`), this
/// function answered `None` for every press on one, `covers` was **false**, and
/// the press fell into the select-on-press body below — which picks the topmost
/// *content* object at that point and **replaces the annotation selection with
/// it**, twenty points above the shape the operator was aiming at.
///
/// ⇒ Then `pressing::look`, on the very next statement, would find a content
/// selection and the release would rotate a page object. A perfect gesture, on
/// something the operator never selected.
///
/// The fix is `pressing::grabbable`, which is the one function that knows about
/// all four kinds of grabbable box — page content, a markup, a ce dimension and
/// a form field's widget. That is what this doc comment always *claimed* was
/// being asked; it stopped being true when the second kind arrived, and nothing
/// said so.
///
/// ★ **The lesson is in the phrasing, not the diff.** *"The same two functions
/// `pressing::look` asks"* was a claim about a call site somewhere else, held
/// together by nothing. A guard that must agree with another module has to
/// **call that module**, not resemble it.
///
/// # ★★★ THE SIXTH INSTANCE WAS NOT HERE, AND IT WIDENS THE RULE — 2026-08-29
///
/// Recorded here because this is where the rule above lives and where the next
/// person will look for it. On the first ever driven run of
/// `rotating_a_markup_turns_it` the rotate handle was painted, was pressed at
/// the rect the application itself declared, and **committed nothing with
/// nothing said anywhere** — the same symptom as the fifth, produced by a line
/// in a different file.
///
/// The cause was a guard in `canvas::rotating::drag` that neither called
/// `grip_box` nor resembled it: `selection.object_indices_on(page).is_empty()`,
/// standing *in front of* the annotation branch. It counts page **content**,
/// which `select_annot` clears, so it returned before the routing decision was
/// ever reached, on every markup and every ce dimension.
///
/// ⇒ So the rule above is necessary and was not sufficient. Its companion:
/// **a guard written in one destination's vocabulary must stand AFTER the
/// branch that picks the destination, never before it.** Three destinations
/// share the rotate gesture and four share a press; a content-shaped test in
/// front of the fork answers about a subject the gesture may already have
/// routed away from. `canvas::rotating`'s header carries the full account.
///
/// The eight resize grips are inside the box and were never at risk. They are
/// covered by the same call anyway, rather than by an argument that they are
/// safe: an argument is a thing that stops being true.
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
    let grip = grabbable.bounds.and_then(|b| {
        crate::canvas::handles::grip_at(b, point, crate::canvas::handles::GripSet::all())
    });
    let Some(grip) = grip else {
        return false;
    };
    // ★★★ **ONLY `Grip::Move` is second-guessed** — and getting this wrong
    // shipped for one afternoon on 2026-08-31, caught by the driven suite.
    //
    // The O72 downgrade below asks whether the press really landed on the
    // selected object. A press on a RESIZE GRIP or the ROTATE HANDLE does not:
    // those sit on the box's edges and corners, outside the object's own
    // geometry, so `body_under` answers false for every one of them. The first
    // version asked `grip_at(..).is_some()` and therefore refused to cover a
    // press on a grip — `at_press` then RE-SELECTED whatever was under it, and
    // the resize became a select-and-move.
    //
    // `resize_scales_a_shape` and `shift_constrains_a_resize` both failed with
    // *"the grip drag committed nothing and declined nothing"*, and the trace
    // showed two `selection-set … via=press` lines where there should have
    // been one. Exactly the failure this file's header warns about — *"every
    // disagreement is a press that selects when it should have transformed"* —
    // arrived at by making the two functions agree on the PREDICATE and not on
    // which grip it applies to.
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
    // object** — `OPERATOR_REQUESTS.md` O72, 2026-08-31.
    //
    // For a ce dimension, a markup annotation and a form field the box IS the
    // `/Rect`, so `in_box` is the whole answer and nothing more is asked. For
    // page content the box is `selection.outline_union()` — a rectangle around
    // scattered geometry, mostly empty paper. Without this second question,
    // selecting a title-block border (a hollow rectangle spanning a CAD sheet)
    // made every subsequent press on the drawing "already covered", so nothing
    // could be selected and no marquee could be started.
    //
    // ★★ It calls `pressing::body_under` rather than asking its own version.
    // This function's header is explicit that a second opinion computed
    // differently here would disagree with the gesture machine at the margins,
    // and every disagreement is a press that selects when it should have
    // transformed. `pressing::look` applies the identical downgrade to
    // `Grip::Move` on the very next statement after this one runs.
    !grabbable.content
        || crate::canvas::pressing::body_under(
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
