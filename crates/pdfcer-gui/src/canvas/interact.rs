//! # canvas::interact — what the operator just did, and what happens as a result
//!
//! The **interaction** half of the canvas. [`super`] is the composition half:
//! it settles *where everything goes* — the scroll area, the strip of page
//! rectangles, the fit against this frame's viewport, the one screen⟷page map
//! — and hands the settled facts here in a single [`Frame`]. This file answers
//! the other question, and only that one: **what did the operator just do, and
//! what happens as a result?**
//!
//! Everything a canvas gesture means passes through one function, [`interact`]:
//! reading the frame's pointer, deciding what a press would land on, advancing
//! the gesture machine, building a decomposition *only* if something needs one,
//! applying the outcome, the right-click, the keys, the re-resolve, the draw
//! and the cursor. Its own header carries the order of those steps and the
//! argument for that order — including the two orderings that are load-bearing
//! rather than incidental (Escape read at step 1 and honoured at step 6; the
//! right-click applied before the resolve). The sections below carry what is
//! true of the file as a whole.
//!
//! ## Why this is a separate file
//!
//! Rule R2 (no `.rs` file over 1,500 lines) forced the split at 1,526, and the
//! seam was already drawn by the two subjects: **composition needs a live `Ui`
//! and changes when the layout changes** — a new page-display mode, a ruler
//! reservation, a strip; **interaction needs this frame's input and changes
//! when a gesture changes** — a new tool, a new outcome, a new key. They change
//! for different reasons, so they are two files. [`super`]'s header carries the
//! table of every other `canvas::*` module.
//!
//! ## Actions, not mutations — this is the file that has to hold the line
//!
//! Nothing here runs from a widget to a document. Every verb an operator
//! reaches through the canvas leaves as an [`Action`] pushed onto `actions` and
//! applied after the frame, in one place. Delete is the sharp case and it is
//! visible in [`keys::canvas_keys`]: it removes nothing, it raises
//! [`VectorAction::DeleteSelection.into()`] carrying the operand list. The zoom a released
//! region marquee asks for is an [`Action`] too, and so is every move a drag
//! commits.
//!
//! The one document field this file writes back is the **selection**, which is
//! the third of the three bookkeeping fields [`super::show`] is permitted to
//! write — see [`super`]'s "Actions, not mutations" section for the whole
//! argument, and [`interact`]'s own docs for why the selection is taken out of
//! the document by value and put back at the bottom.
//!
//! ## ★ Selection survives navigation — the invariant this stage is accountable for
//!
//! `GUI_ROADMAP.md` Phase 1 states it and names three ways it is lost.
//! [`selection`](super::selection)'s header carries the full table; the
//! wiring's share of it is visible right here in [`interact`]:
//!
//! - the selection is **read, resolved and stored** on every frame, and the
//!   resolve does no work unless `(page, edit epoch)` moved — so a zoom, a
//!   pan, a fit change, a view rotation, a page-display change and a ribbon-tab
//!   change all cost one comparison and change nothing;
//! - the only thing that can clear it is [`gesture::GestureOutcome::Click`],
//!   which is raised on a **completed click with no drag**;
//! - a decomposition is built **only when a gesture needs one or the epoch
//!   moved** — never per frame, and never merely because the view changed.
//!
//! **A move does not threaten the invariant, and that is measured rather than
//! hoped.** `move_*` rewrites operator operands in place, adds and removes no
//! operator, and therefore renumbers nothing — `pdfcer-core`'s
//! `object_identity_across_edits.rs` decomposes, edits and decomposes again to
//! prove it. So a committed move leaves every `Selection` untouched and only
//! the *outlines* have to catch up, which the epoch bump already handles. The
//! `delete_*` family is the one that renumbers. See [`moving`]'s header for the
//! full table.
//!
//! ## ★ The two seams that were wiring, and how they were closed
//!
//! Both were recorded here as *"one-line changes once the field they want
//! exists"*. The field exists; both are closed. They are written up rather
//! than deleted because each closed a live hazard, and the next person to
//! consider reopening one should have to read what it cost.
//!
//! 1. **The selection was held in `egui`'s `Memory`.** It is document-scoped
//!    state, and `Memory` outlives documents — so the canvas had to *detect*
//!    a document change with a `DocumentToken` built from the
//!    `Arc<EditSession>`'s **allocation address** mixed with the page count,
//!    compared once per frame. That is the same address-as-identity key
//!    `panels::DocKey` was deleted for earlier in this stage, with the same
//!    ABA hazard.
//!
//!    It now lives on [`crate::app::state::OpenDoc::selection`], and
//!    `DocumentToken` and `SelectionState::sync_document` are **deleted**.
//!    `OpenDoc::new`'s own argument does the work: constructing fresh state
//!    per document is what makes *"a cached value can never refer to a
//!    previous file"* true by construction, on every frame, with nothing to
//!    compare. The canvas still owns the selection in the sense that matters —
//!    it is the only writer — and the *edit* a Delete leads to is still an
//!    [`Action`] applied after the frame.
//!
//! 2. **The decomposition was built transiently, per gesture.** The Objects
//!    panel's cache moved onto `OpenDoc` earlier in this stage; until this
//!    change the canvas could not reach it, so `build_targets` called
//!    `ObjectModelProvider::build` for itself on every click and every marquee
//!    release — a **second** decomposition of a page the panel had already
//!    decomposed, which is the *"two decompositions quietly diverge"* failure
//!    decision 011 names.
//!
//!    [`interact`] now reads [`crate::app::state::OpenDoc::page_objects`], and
//!    `build_targets` is deleted. One decomposition per `(page, epoch)`, shared
//!    by the panel, the Properties row, the `objects n=` trace and the hit
//!    test — the same value, not two values that agree. [`show`](super::show)
//!    needs no provider argument, because the document it already takes
//!    carries it.
//!
//!    The gating is kept and still matters: the cache is asked for **only**
//!    when a gesture needs a hit test or when `(page, epoch)` has moved, so a
//!    zoom or a pan with the Objects panel closed still decomposes nothing.
//!    Drawing needs no provider at all — the outlines are cached in canvas
//!    space, which is zoom-independent.

use crate::app::actions::forms::FieldAction;
use egui::{Key, PointerButton, Rect};
use egui_shell::HandlerToken;

use crate::app::actions::Action;
use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
use crate::canvas::clicking;
use crate::canvas::gesture::{GestureOutcome, MarqueeIntent, Phase, PointerFrame};
use crate::canvas::input::{load_gesture, store_gesture};
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::PickFilter;
use crate::canvas::target::CanvasTargetProvider;
use crate::canvas::tool::CanvasTool;
use crate::shell::menus::MenuHost;

// The sibling modules this file wires together, imported as modules rather than
// as items so that every call below reads exactly as it did when it lived in
// `mod.rs` — `overlay::grip_box`, `zoom::arm_anchor`, `keys::canvas_keys` — and
// so the doc links above and below resolve to the same places they always did.
use super::{CANVAS_MARGIN, dimdrag, keys, markup, measure, strip, textsel, tool, zoom};

/// The three facts about *this frame's canvas* that [`interact`] needs, and
/// that every part of it must agree on.
///
/// A struct rather than three parameters, and the reason is the same one that
/// made [`PageMapping`] a struct: these are facts about one frame, they are
/// settled together once the scroll area has laid out, and passing them
/// separately invites a call site to compute one of them for itself. `tool` is
/// the newest member and the most dangerous to re-derive — two readings of
/// "is the hand active?" within one frame that disagreed would be a drag that
/// panned **and** marquee'd, which is exactly what Phase 3.2 must not ship.
#[derive(Debug, Clone, Copy)]
pub(super) struct Frame<'a> {
    /// The frame's ONE screen ⟷ canvas map, **for the page being acted on**.
    ///
    /// Every gesture, every hit test and every selection outline goes through
    /// this one. Phase 4 did not add a second: a strip shows several pages, but
    /// input is about exactly one of them, and which one is settled before this
    /// struct is built (see [`show`](super::show)).
    pub(super) map: PageMapping,
    /// One map per page this frame drew, for the **Find wash** — the single
    /// thing that is legitimately about pages other than the one being acted
    /// on, because a search describes the whole document and its hits have to
    /// land on the pages they are on.
    ///
    /// Exactly one entry under
    /// [`viewer::PageDisplay::Single`](crate::viewer::PageDisplay::Single), and
    /// it is the same page `map` describes.
    pub(super) pages: &'a [strip::PageView],
    /// The scroll viewport, which is both the painter's clip rect and the
    /// region "is the pointer over the canvas?" is asked against.
    pub(super) clip: Rect,
    /// What the primary button means this frame.
    pub(super) tool: CanvasTool,
    /// **What the active mode lets this frame do to the document.**
    ///
    /// Beside `tool` rather than anywhere else because the two answer the same
    /// question from opposite ends: `tool` is what the operator armed, `caps`
    /// is what the mode they are in permits, and `gesture::press_kind` is the
    /// one function that reads both and decides what a press means. Sampled
    /// once per frame by [`show`](super::show), for the same reason `tool` is —
    /// a gesture means what it meant when it started, and a mode change
    /// mid-drag is handled by cancelling the drag, not by re-reading it here.
    pub(super) caps: Capabilities,
    /// ★ **What the operator is allowing clicks to land on** — the
    /// selection filter (`OPERATOR_REQUESTS.md` O17).
    ///
    /// The fourth member of the sampled-once-per-frame set, and it belongs
    /// with `tool`, `caps` and `pen` for the same reason they belong with
    /// each other: `tool` is what the operator armed, `caps` is what the
    /// mode permits, `pen` is what it will look like, and this is what is
    /// worth pointing at. All four are read at the top of the frame, so a
    /// gesture means what it meant when it started.
    ///
    /// It composes with `caps` as an `AND` in one direction only: switching
    /// a class on here can never grant a capability the mode withholds.
    /// [`crate::canvas::pick`]'s header carries the argument for why those
    /// are two questions with two owners rather than one flag.
    pub(super) pick: PickFilter,
    /// ★ **The operator's configured maximum zoom**, as a percentage — O24.
    ///
    /// The fifth member of the sampled-once-per-frame set, here for the same
    /// reason as the other four: a marquee-zoom's ceiling must be the one that
    /// was in force when the gesture started, not one re-read mid-drag.
    pub(super) max_zoom_percent: f32,
    /// ★ The colour and width the next markup will be authored with.
    ///
    /// Beside `caps` and `tool` because it belongs to the same triple: `tool`
    /// is *what* the operator armed, `caps` is *whether* the mode permits it,
    /// and `pen` is *what it will look like*. All three are sampled once per
    /// frame and for the same reason — a gesture means what it meant when it
    /// started, so an operator who changes the pen mid-drag gets the pen they
    /// had when they pressed, not a stroke whose colour changed under them.
    ///
    /// It reaches the **preview** as well as the commit, which is the property
    /// that makes it worth threading rather than reading at commit time: the
    /// band drawn while dragging is the pen's real width at this magnification
    /// and the pen's real colour, so what the operator sees is what lands.
    pub(super) pen: crate::canvas::markup::pen::Pen,
}

/// Read the frame's canvas gestures and keys, update the selection, draw it,
/// and return how many entries are selected (for the trace line) together
/// with any context-menu commands the operator invoked.
///
/// # The order of the steps, and why it is this order
///
/// 1. **Read the pointer**, converting once through `map` — the boundary.
///    Escape rides in on the same frame, so a drag in flight can abort.
/// 2. **Decide what a press would land on**, from the *previous* frame's
///    selection. A grip is a target because it is already on screen.
/// 3. **Advance the gesture machine.** A press produces nothing; only a
///    completed click, a released marquee, a move drag or an Escape-abort
///    produces an outcome.
/// 4. **Build a decomposition only if something needs one.** A click, a
///    released marquee, **a right-click**, **a move drag**, or a
///    `(page, epoch)` that moved. Never merely because the view changed — that
///    is the invariant, and it is this `if`.
/// 5. **Apply the outcome**, then **the right-click**, then **re-resolve**,
///    then **draw** (outlines, then the marquee, then the move ghost).
/// 6. **Keys**, guarded by `text_edit_focused()` — `DEFECTS.md` D1 — and by
///    whether step 3 already spent the Escape on a drag.
///
/// Step 4 sitting *after* step 3 is what makes the whole thing affordable:
/// the expensive work is behind the gesture, not in front of it.
///
/// # ★ Why Escape is read in step 1 and honoured in step 6
///
/// Two things want the key and exactly one may have it per press: a drag in
/// flight wants to abandon itself, and the selection ladder wants to ascend a
/// rung. The gesture machine gets first refusal because it is the only thing
/// that knows whether there *is* a drag under the press — it takes the key only
/// when there is, and reports [`gesture::GestureOutcome::Cancelled`] when it
/// does. Step 6 passes that on to [`canvas_keys`], so a cancelled drag does not
/// also cost the operator the rung they were working in. With no drag in
/// flight, nothing is consumed and Escape ascends exactly as it always did.
///
/// # ★ Where the right-click sits, and why it is not step 5's business
///
/// The secondary button never reaches [`gesture`]: that machine reads
/// `PointerButton::Primary` throughout, deliberately, because the middle
/// button pans and the primary button owns press/drag/release. A right-click
/// is not a gesture with a beginning and an end — it is a single event that
/// asks a question — so it is read straight off the `Response` in step 5b,
/// *after* the primary gesture has been applied and *before* the resolve.
///
/// Before the resolve matters: a right-click over an unselected object
/// **selects it** ([`menus::select_under_right_click`]), and the outlines
/// drawn in step 8 have to be the new selection's or the highlight would lag
/// the menu by a frame — the operator would see a menu about an object that
/// is not yet outlined, which is the "which of these is it about?" ambiguity
/// the select-first rule exists to remove.
///
/// # ★ Why the selection is moved out of the document and back again
///
/// The selection now lives on [`OpenDoc`], and the decomposition it resolves
/// against is a [`std::cell::Ref`] **borrowed from the same `OpenDoc`**. Those
/// two cannot be held at once through one `&mut` — a `Ref` keeps a shared
/// borrow of the whole document alive, and mutating the selection in place
/// would need a mutable one.
///
/// So the selection is taken by value at the top and put back at the bottom.
/// That is not a workaround for the borrow checker so much as an honest
/// statement of what this function does: it computes *the next* selection from
/// the previous one and the frame's input, and stores it. Two further
/// properties fall out, both of which the `egui::Memory` version relied on and
/// documented:
///
/// * **nothing is cloned.** A marquee over a dense sheet can select thousands
///   of entries, and cloning that per frame at 60 Hz would be a real cost for
///   no reason;
/// * **a frame that panicked between the take and the put leaves an empty
///   selection**, not a half-updated one — a state the operator can see and
///   recover from with one click.
///
/// # ★ The hand tool suppresses the whole of step 1, and that is the fix
///
/// When [`tool::active`] reports `Hand` — chosen, or borrowed by a held space
/// bar — the primary button pans, and a `PointerFrame` describing that drag
/// would make it marquee **as well**. So the frame is built **blank** apart
/// from Escape: no press, no drag, no click, no position. Not "the marquee arm
/// checks the tool", not "the selection is restored afterwards" — the gesture
/// simply is not offered, which is the only version of this that cannot leave
/// a half-applied selection behind if a future arm forgets to ask.
///
/// A drag already in flight when the space bar goes down is therefore
/// *interrupted*, and [`gesture::GestureState::update`]'s last branch already
/// knows what to do with that: abandon it, commit nothing. An operator who
/// starts a marquee and then reaches for space has changed their mind, and the
/// worst outcome available is that nothing happened.
pub(super) fn interact(
    ui: &mut egui::Ui,
    doc: &mut OpenDoc,
    response: &egui::Response,
    frame_ctx: &Frame,
    host: Option<&MenuHost<'_>>,
    find: &crate::find::FindState,
    actions: &mut Vec<Action>,
) -> (usize, Vec<HandlerToken>) {
    // Destructured through the reference, so `map` stays a borrow — it is
    // handed on to the overlay and the probe, both of which take one.
    let Frame {
        map,
        pages,
        clip,
        tool: active_tool,
        caps,
        pick,
        max_zoom_percent,
        pen,
    } = frame_ctx;
    let (clip, active_tool, caps, pen) = (*clip, *active_tool, *caps, *pen);
    let ctx = ui.ctx().clone();
    let page_index = doc.view.page_index;

    // Out of the document for the duration of the frame's gesture — see this
    // function's docs. The `&mut doc` borrow ends on this line, which is what
    // lets the decomposition below be borrowed out of `doc` at all.
    //
    // ★ Notice what is NOT here: no document token is compared, because none
    // exists. A selection belongs to the `OpenDoc` it is a field of, so a
    // different document is a different `OpenDoc` carrying its own empty one.
    // A page change is not a document change and neither is an edit — both
    // re-resolve at step 7. See `selection`'s invariant 3.
    let mut selection = std::mem::take(&mut doc.selection);
    // ★ …and the **text** selection, out for the same duration and for exactly
    // the same borrow reason: the page's extraction is a `Ref` borrowed out of
    // this same `OpenDoc` (`OpenDoc::page_text`), so the value cannot be
    // mutated in place while the thing it is resolved against is being read.
    // Taken by value, put back at the bottom, and — like the object selection —
    // a frame that panicked between the two leaves an empty one rather than a
    // half-updated one.
    let mut text_selection = std::mem::take(&mut doc.text_selection);
    let mut gestures = load_gesture(&ctx);

    // ---- 1. the pointer, converted once ------------------------------
    //
    // `interact_pointer_pos` first: during a drag it keeps reporting even
    // after the pointer has left the widget, which is exactly the case a
    // rubber-band dragged off the page depends on. `pointer_latest_pos` is
    // the hover fallback, which is what the grip cursor needs.
    let screen_pos = response
        .interact_pointer_pos()
        .or_else(|| ctx.pointer_latest_pos());
    let shift = ctx.input(|i| i.modifiers.shift);
    // ★ Ctrl, read beside Shift — `OPERATOR_REQUESTS.md` O104. It means "take
    // this OUT of the selection", for a click and for a band; the argument for
    // that spelling lives with the mechanism, in `canvas::marquee::Combine`.
    //
    // `modifiers.command`, not `.ctrl`: on Windows they are the same key and on
    // macOS `command` carries this meaning, which is what the rest of the crate
    // already reads.
    let ctrl = ctx.input(|i| i.modifiers.command);
    // Escape is read whatever the tool is: a hand-tool frame still has to be
    // able to abandon a drag the previous tool left in flight, and it is still
    // the ladder's key when there is no drag. See this function's header.
    // typing-guard-exempt: this is `does a WIDGET hold Escape`, not `is anybody
    // composing`. A canvas draft must still let Escape reach the gesture ladder,
    // which is what abandons the draft — see `canvas::keys`' entry guard.
    let cancel = !ctx.text_edit_focused() && ctx.input(|i| i.key_pressed(Key::Escape));
    let frame = if active_tool.pans_with_primary() {
        PointerFrame {
            cancel,
            ..PointerFrame::default()
        }
    } else {
        PointerFrame {
            // `..._by(PointerButton::Primary)` throughout: the bare forms are
            // button-agnostic, and the middle button is the pan. See `gesture`.
            drag_started: response.drag_started_by(PointerButton::Primary),
            dragging: response.dragged_by(PointerButton::Primary),
            drag_stopped: response.drag_stopped_by(PointerButton::Primary),
            clicked: response.clicked_by(PointerButton::Primary),
            double_clicked: response.double_clicked_by(PointerButton::Primary),
            // The third click of a triple — a text selection's "take the whole
            // line" gesture. egui counts clicks and reports `is_double()` and
            // `is_triple()` as `count == 2` and `count == 3`, so this and the
            // line above are mutually exclusive on any one release and nothing
            // downstream has to order them.
            triple_clicked: response.triple_clicked_by(PointerButton::Primary),
            pos: screen_pos.map(|p| map.to_page(p)),
            // ★ Where the button actually went down, through the frame's ONE
            // map. Without it every drag begins wherever the pointer had got to
            // by the frame egui called it a drag — measured at 94 page points
            // on an A1 sheet. See `gesture::PointerFrame::press_origin`.
            press_origin: ctx
                .input(|i| i.pointer.press_origin())
                .map(|p| map.to_page(p)),
            shift,
            // Escape reaches the gesture machine BEFORE it reaches the ladder,
            // so a drag in flight can abandon itself. The machine consumes the
            // key only if there was a drag to abandon, and says so by returning
            // `Cancelled` — which is what step 6 passes on to `canvas_keys`.
            cancel,
        }
    };

    // ---- 2. what a press would land on -------------------------------
    //
    // Moved to `canvas::pressing` under R2 on 2026-08-19, when the Node tool
    // and the Bézier-handle hit test pushed this file past 1,500 lines. It is a
    // real seam: everything there answers *if the button went down here, right
    // now, what would happen?* and changes nothing, where every remaining
    // section of this function advances, routes or paints.
    //
    // ★ Its header carries the **precedence** — handle, then anchor, then grip,
    // then the selection body — and the three separate defects that taught it.
    // That rule is the single most bug-prone thing on this canvas and it now
    // lives in one place with its own reasoning beside it.
    // ★★★ **1b. A press on an unselected object selects it.**
    //
    // Moved to `canvas::presspick` under R2 on 2026-08-27, and it is a real
    // seam rather than a convenient cut: `canvas::pressing`'s header opens with
    // *"changes nothing"*, and this step changes the selection. Putting a
    // mutation inside the module whose contract is that it only looks would
    // have made that contract false the day someone relied on it.
    //
    // Its header carries the operator's report, the four gestures it must not
    // disturb, and why this is nine statements rather than a new `DragKind`.
    crate::canvas::presspick::at_press(
        &ctx,
        // ★ The acting page's response, added 2026-08-31 for O75: this step
        // was the one place on the canvas that asked the *window* whether a
        // press had happened rather than asking *itself*, so a press in a
        // dock selected page content through the panel.
        response,
        doc,
        &mut selection,
        map,
        page_index,
        active_tool,
        caps,
        *pick,
        shift,
    );

    let press = crate::canvas::pressing::look(
        &ctx,
        doc,
        &selection,
        map,
        page_index,
        screen_pos,
        active_tool,
        caps,
    );
    let hovered_grip = press.grip;
    let press_kind = press.meaning;

    // ---- 3. advance the gesture --------------------------------------
    //
    // ★ 3a. The freehand trail's whole lifetime, in one line — and it is read
    // **before** the machine advances, which is the load-bearing half.
    //
    // `canvas::markup::ink` keeps the pointer trail of an in-flight `/Ink` drag
    // in `egui::Memory`, and it deliberately does **not** clear it on an event.
    // A drag ends four ways — released, Escaped, interrupted by focus loss,
    // interrupted by the space bar borrowing the hand — and only the first two
    // are events anything could hook; the other two would leave a stale trail
    // that the operator's next stroke would begin by joining onto.
    //
    // So the trail is DERIVED from the gesture machine's own answer, exactly as
    // `canvas::tool` derives the space-bar hand rather than storing it: it exists
    // while `active()` says a freehand markup drag is in flight, and there is no
    // restore step to miss.
    //
    // ★ **Before `update`, not after, and the difference is the whole stroke.**
    // `update` clears its own `drag` on the frame it reports `Complete`, so an
    // `active()` read *afterwards* answers `None` on exactly the frame the
    // release arrives — the trail would be discarded a few lines before the arm
    // that commits it, and every stroke would author the two points egui happened
    // to report on that last frame. **Measured on the real binary before the
    // ordering was fixed: `markup-commit kind=Ink raw=2 kept=2` for a drag the
    // harness walked in eight increments.** Read first, the answer is the state
    // the *previous* frame left, which is what "is a stroke in progress?" means.
    //
    // It costs one `Id` lookup on frames where no trail exists, which is every
    // frame nobody is drawing on. See `markup::ink` §2.
    markup::ink::sync(&ctx, gestures.active());

    let outcome = gestures.update(frame, press_kind);

    // ---- 4. the decomposition, only if a HIT TEST needs one -------------
    //
    // ★ `doc.page_objects()` — THE decomposition, the one the Objects panel
    // lists and the `objects n=` trace counts. The canvas used to build its
    // own here (module docs, seam 2); that second `decompose_page` over the
    // same page is gone.
    //
    // The gate is kept even though the value is now cached, and it still buys
    // something real: `page_objects()` builds on first use, so asking for it
    // on a frame that has no hit test to do would decompose the page the first
    // time the operator merely zoomed — with the Objects panel closed, that is
    // a whole content-stream walk bought by a mouse wheel. Drawing needs no
    // provider at all (the outlines are cached in canvas space, which is
    // zoom-independent), so on the overwhelming majority of frames nothing is
    // asked for and nothing is built.
    //
    // Not "if a resolve needs one" — see step 6 for why the resolve's build
    // has to happen after the keys rather than before them.
    //
    // A **right-click** is in this set for the same reason a click is: it has
    // to know what is under the pointer in order to select it, and a menu
    // about the wrong object is worse than no menu. It is a rare event, so
    // the gate still buys what it always bought — a zoom or a pan with the
    // Objects panel closed decomposes nothing.
    // A **move drag** is in the set too, at either phase, and it is the one
    // member that is not a hit test — which is why the flag is named for what
    // it gates rather than for what most of its members do. It needs the model
    // to answer two questions the selection alone cannot: *is every selected
    // object a path* (a non-path refuses the whole move, and a ghost drawn over
    // one would promise a move that gets refused), and, at the Node rung,
    // *where is the anchor now* (`move_node` takes a destination, not a delta).
    //
    // Asking on every frame of an in-flight drag is affordable because the
    // answer is already built: the selection cannot have outlines to drag
    // without a decomposition, so `page_objects()` is a cache hit for the
    // whole gesture. The gate still buys what it always bought — a zoom or a
    // pan with nothing being dragged decomposes nothing.
    //
    // ★ A **zoom** marquee is deliberately NOT in the set. It selects nothing,
    // so it hit-tests nothing, so it decomposes nothing — a region zoom over a
    // 129,758-object drawing costs one scroll offset. That falls out of the
    // intent being carried on the outcome rather than being asked for at the
    // release, and it is the concrete payoff for sampling it at the press.
    // ★ A mode that cannot edit content hit-tests **nothing**, and this is the
    // one line that says so for the whole step.
    //
    // The gesture gate above already means no `Click`, `Move` or select-marquee
    // outcome can arrive here, so the `matches!` below is dead in Read either
    // way. The right-click is *not* covered by it — a secondary click is not a
    // gesture — so without this conjunction Read would decompose the page on
    // every right-click in order to find an object it may not offer a verb for.
    // On the benchmark drawing that is 129,758 objects built to be discarded.
    //
    // It also decides the menu: with no object under the pointer, `attach`
    // resolves to `canvas.empty`, whose three items are named zoom levels. That
    // is the correct menu for a reader — it is about the view, because there is
    // nothing here that is theirs to act on — and it is reached by the menu
    // system's own rule rather than by a second mode branch inside it.
    // ★★★ **Every mode hears a right-click**; which menu opens is
    // `canvas::menus::attach`'s decision — O71, 2026-09-01. Its `reading` arm
    // carries why the gate moved off this line.
    let secondary_clicked = response.secondary_clicked();
    // ★ An armed measure tool needs the decomposition **on every frame**, not
    // only on the frame of a click.
    //
    // Two distinct consumers, and the second is the one that makes this a
    // per-frame term rather than a per-click one:
    //
    // * the **two-line pick** reads the page's geometry through
    //   `pick_line_in_page` — picking a line is *reading* the page, not
    //   selecting it, which is why this is its own term rather than a
    //   relaxation of `caps.edit_content`;
    // * the **snap query** runs while the operator is still deciding where to
    //   click, because an indicator that appeared only on the click it is
    //   meant to guide would be useless. That applies to every measure kind,
    //   which is why this is not narrowed to `TwoLine`.
    //
    // The cost is bounded by the same cache every other consumer shares:
    // `page_objects()` is one decomposition per `(page, epoch)`, so an armed
    // tool pays for the first frame and then hits the cache. An **un-armed**
    // canvas is untouched — the whole term is false — so panning a 129,758-
    // object drawing still decomposes nothing.
    let measure_needs_model = active_tool.measure_kind().is_some();
    let needs_targets = secondary_clicked
        || measure_needs_model
        || matches!(
            outcome,
            GestureOutcome::Click { .. }
                | GestureOutcome::Move { .. }
                // ★★ Resize joined this list on 2026-08-19, and its absence
                // was the second defect the first driven resize found.
                //
                // The decomposition is what `canvas::resizing` reads every node
                // position out of, so without it the commit declined with
                // `NoObjectModel` — a refusal that is correct for "the model
                // could not be read" and was here reporting "nobody asked for
                // it". The list was written when a resize committed nothing, so
                // there was genuinely nothing for it to need.
                | GestureOutcome::Resize { .. }
                // ★ Same reason as `Resize`, and it was learned there: the
                // commit needs the object model to refuse a stale index, and a
                // gesture that ran on a canvas which never asked for a provider
                // gets `None` and declines. The resize spent a whole driving
                // session on exactly this.
                | GestureOutcome::Handle { .. }
                // ★★ …and `DimensionVertex`, added 2026-08-20 when the vertex
                // drag learned to snap. THIRD time this list has been the
                // defect: `Resize` and `Handle` both shipped needing the model
                // and not asking for it, and both spent a driving session
                // presenting as "the gesture does nothing".
                //
                // The failure here would have been quieter than either, because
                // the drag works perfectly without the model — it just never
                // snaps, and a snap that never fires is indistinguishable from
                // a snap that found nothing nearby. That is precisely the class
                // of defect that survives a green suite.
                | GestureOutcome::DimensionVertex { .. }
                // ★ …and `Rotate`, for the same reason as `Resize` one line
                // above: the commit resolves paint-order indices, and a gesture
                // on a canvas that never asked for a provider would address
                // indices nothing has verified.
                | GestureOutcome::Rotate { .. }
                | GestureOutcome::Marquee {
                    phase: Phase::Complete,
                    intent: MarqueeIntent::Select,
                    ..
                }
        );
    // ★ Drop a held preview that has stopped being live — the raster caught up,
    // the edit was refused, or the backstop fired.
    //
    // ★★ Placed ABOVE the provider borrow deliberately, and it is not a style
    // choice: `page_objects()` returns a `Ref` that lives until `drop(targets)`
    // a few hundred lines below, and nothing may take `&mut doc` while it is
    // held. Every mutation of the document in this function is either above
    // this borrow or below that drop, and the two O63 calls follow the same
    // rule — `hold_preview` waits until after it.
    doc.retire_held_preview();
    let mut targets = if needs_targets {
        doc.page_objects()
    } else {
        None
    };

    // ---- 5. apply the gesture -----------------------------------------
    // ★★★ **The fourteen pre-commit slots one frame might fill**, extracted
    // to `canvas::previews` on 2026-08-30 when O63's shape preview pushed
    // this file past R2's ceiling.
    //
    // The seventy lines of argument that used to sit here — why each slot is
    // its own value and not a variant of another — moved with them, stated
    // once in that module's header instead of nine times in this function.
    // `interact` answers *what does this frame's pointer mean*; those answer
    // *what is drawn while the answer is still provisional*.
    let mut pv = crate::canvas::previews::Slots::default();
    // ★ The one preview value that does NOT live in `pv`, because it does not
    // belong to this frame: it is handed to the document to keep on screen
    // until the raster catches up. Parked here and applied after
    // `drop(targets)`, since taking `&mut doc` inside the match is impossible
    // while the provider `Ref` is alive.
    let mut hold_after_drop: Option<crate::canvas::shapes::ShapePreview> = None;
    match outcome {
        // ★ A click is EITHER a measure pick or a selection, never both.
        //
        // The branch is on the armed tool rather than on a capability, and the
        // two are mutually exclusive by construction: there is one armed tool
        // per frame, so there is no ordering here for a future reader to get
        // wrong and no state in which a click both places a dimension and
        // replaces the selection.
        GestureOutcome::Click {
            point,
            shift,
            double,
            triple,
        } => clicking::click(
            clicking::Frame {
                ctx: &ctx,
                doc,
                page_index,
                map,
                targets: targets.as_deref(),
                active_tool,
                caps,
                pick: *pick,
                pen,
                point,
                // ★ Ctrl toggles too, not only Shift. Reaching for Ctrl used to
                // give a plain click, which REPLACES the selection with the one
                // object he was trying to remove — see O104.
                shift: shift || ctrl,
                double,
                triple,
            },
            &mut selection,
            &mut text_selection,
            actions,
        ),
        GestureOutcome::Marquee {
            rect,
            crossing,
            shift,
            intent: MarqueeIntent::Select,
            phase: Phase::Complete,
        } => crate::canvas::marquee::on_release(
            targets.as_deref().map(|t| t as &dyn CanvasTargetProvider),
            page_index,
            rect,
            crossing,
            crate::canvas::marquee::mode(shift, ctrl),
            &mut selection,
        ),
        // ★ The same rubber band, released with the other intent. **The
        // selection is not touched** — not cleared, not replaced, not even
        // read: a navigation gesture that rearranged the selection would break
        // the invariant this whole stage is accountable for, and the way to not
        // break it is to have no line of code here that could. `zoom_to_rect`
        // takes the band in canvas space, which is the space it already
        // arrived in.
        //
        // Disarmed on release, not on press: the one-shot has to survive the
        // whole drag, or an Escape-cancelled zoom marquee would leave the
        // operator with a disarmed tool and nothing to show for it.
        GestureOutcome::Marquee {
            rect,
            intent: MarqueeIntent::Zoom,
            phase: Phase::Complete,
            ..
        } => {
            zoom::disarm_region_zoom(&ctx);
            // Deferred to step 7b for one structural reason: `targets` is a
            // `Ref` borrowed out of `doc`, and arming an anchor needs `&mut
            // doc`. The same constraint the `drop(targets)` below exists for,
            // and the same answer the marquee and the ghost already use —
            // decide here, act once the borrow has ended.
            pv.zoom_region = Some(rect);
        }
        GestureOutcome::Marquee {
            rect,
            phase: Phase::InFlight,
            ..
        } => pv.marquee = Some(rect),
        // ★ The move. `moving::drag` owns every rule — which verb the rung
        // reaches, whether the operands qualify, the canvas→page delta — and
        // returns the ghost's canvas-space offset when, and only when, the
        // release would commit. Nothing about the move is decided here, on
        // purpose: this arm is wiring, and the rules are unit-tested without a
        // window in `moving`.
        // ★★★ The move, routed. Three verbs share this one gesture and the
        // selection decides which — `canvas::dragroute` is that decision and
        // carries the whole argument, including what the ABSENCE of its third
        // branch cost: an annotation drag that was consumed and discarded for
        // ten days, because the fork's two branches could both answer "not
        // mine" and the gesture had nowhere to go.
        //
        // Moved out of this file under R2 on 2026-08-28. Nothing about a move
        // is decided here, which is this arm's whole shape: it is wiring.
        GestureOutcome::Move { delta, phase } => {
            let previews = crate::canvas::dragroute::moved(
                &crate::canvas::dragroute::Frame {
                    ctx: &ctx,
                    doc,
                    selection: &selection,
                    page_index,
                    provider: targets.as_deref(),
                    shift,
                },
                delta,
                phase,
                actions,
            );
            pv.ghost = previews.ghost;
            pv.shape = previews.shape;
            // ★★★ O63's third piece. On the release frame `moving::drag` hands
            // back the geometry it just committed; the document keeps it on
            // screen until the raster carries the edit, so the object does not
            // appear to snap back to where it started.
            //
            // Stored on the DOCUMENT rather than in a frame value, because it
            // has to survive frames — that is the entire point of it.
            hold_after_drop = previews.hold;
            pv.annot_ghost = previews.annot.or(previews.widget);
            pv.dimension = previews.dimension;
        }
        // ★ The markup band. `markup::drag` owns every rule — the canvas→page
        // conversion, the degenerate-drag refusal, which endpoints stay raw —
        // and hands back a band only when the release would commit, which is
        // the same honesty contract the move ghost is held to. Nothing about a
        // markup is decided here: this arm is wiring, and the rules are
        // unit-tested without a window in `markup`. Note what it does NOT need:
        // a decomposition. A markup hit-tests nothing, which is why this
        // outcome is absent from `needs_targets` above.
        // ★ The text sweep. `textsel::drag` owns every rule — the canvas→PDF
        // hop, the range between the two ends, the line-grouped boxes and the
        // string, all from one pass — and hands back a whole
        // `TextSelection` or `None`. Nothing about a text selection is decided
        // here: this arm is wiring, and the rules are unit-tested against a real
        // extraction in `canvas::textsel`.
        //
        // **Both phases are applied**, where the markup band above applies only
        // its release. That is the difference between authoring and selecting: a
        // markup's in-flight band is a *preview* of something not yet written,
        // while a text selection's in-flight state IS the selection — it has to
        // grow under the pointer, because watching it grow is how the operator
        // knows where to let go. Nothing is committed at either phase; see
        // `canvas::textsel`'s header §6.
        //
        // Note what it does NOT need: a decomposition. A text sweep hit-tests
        // glyphs, not objects, which is why this outcome is absent from
        // `needs_targets` above — and why a sweep over the 129,758-object
        // benchmark sheet decomposes nothing.
        // ★ The sweep, routed. Every rule it once applied inline — the
        // extraction options, the degenerate-drag refusal, what counts as a
        // changed range — already lived in `textsel`, so the body moved there
        // on 2026-09-02 under R2 and this arm became what it always described
        // itself as: wiring.
        GestureOutcome::TextSelect { from, to, phase } => {
            text_selection = textsel::sweep(doc, page_index, from, to, phase);
        }
        // ★ Two gesture modules behind one outcome, split on the family rather
        // than on a list of kinds.
        //
        // `markup::band::drag` owns the two-point rule — the canvas→page
        // conversion, the degenerate-drag refusal, which endpoints stay raw —
        // and `markup::ink::drag` owns the trail, its simplification and a
        // preview that draws the simplified polyline rather than the raw input.
        // Both hand back a preview only when the release would commit, which is
        // the honesty contract the move ghost is held to.
        //
        // The branch reads `kind.is_freehand()` rather than
        // `kind == MarkupKind::Ink` for the reason `MarkupKind`'s family
        // predicates exist: each module guards its own entry point with the same
        // question, so a routing change and a guard cannot drift apart. The two
        // guards are asserted from both ends —
        // `band::a_non_band_kind_is_refused_by_the_band_gesture` and
        // `ink::a_non_freehand_kind_is_refused_by_the_freehand_gesture`.
        //
        // Nothing about a markup is decided here: this arm is wiring, and the
        // rules are unit-tested without a window. Note what neither needs: a
        // decomposition. A markup hit-tests nothing, which is why this outcome is
        // absent from `needs_targets` above.
        // ★ The markup drag, routed. Three gesture modules behind one outcome
        // — a band, a freehand trail, and the line-grouped quads of a highlight
        // that found text under it — and `markup::route` is the decision. Moved
        // out of this arm on 2026-08-28 when the third destination landed, for
        // the reason every arm here states: this is wiring.
        GestureOutcome::Markup {
            kind,
            from,
            to,
            phase,
        } => {
            let previews = markup::route::drag(
                markup::route::Drag {
                    ctx: &ctx,
                    kind,
                    pen,
                    doc,
                    page_index,
                    from,
                    to,
                    phase,
                },
                actions,
            );
            pv.band = previews.band;
            pv.ink_trail = previews.trail;
            pv.text_marks = previews.text_marks;
        }
        // ★ A text-annotation band. It draws exactly as a markup band does and
        // COMMITS NOTHING — on `Phase::Complete` it raises the request that
        // opens the dialog, and the words decide whether anything is authored.
        //
        // The band reuses `markup::band::preview_rect` rather than growing its
        // own painter, so a text box's rubber band and a rectangle's are the
        // same pixels from the same code. What differs is only what release
        // means, which is the whole reason the two have separate variants.
        GestureOutcome::TextAnnot {
            kind,
            from,
            to,
            phase,
        } => {
            // A plain RECTANGLE band, whatever the kind. `Preview` carries a
            // `MarkupKind` because that is what decides the shape drawn, and
            // both dragged text kinds occupy a rectangle — so this is not a
            // borrowed constant, it is the correct answer.
            pv.band = Some(markup::band::Preview {
                kind: markup::MarkupKind::Rectangle,
                from,
                to,
            });
            if phase == crate::canvas::gesture::Phase::Complete {
                // `endpoints` — the SAME canvas-to-page conversion the markup
                // band uses, already public. A second conversion here is how a
                // preview and an authored box come to disagree about where the
                // operator dragged.
                if let Some(page) = doc.current_page()
                    && let Some((start, end)) = markup::band::endpoints(from, to, page)
                {
                    actions.push(Action::BeginTextAnnot {
                        page: page_index,
                        kind,
                        rect: pdfcer_core::page_tree::Rect {
                            llx: start.0.min(end.0),
                            lly: start.1.min(end.1),
                            urx: start.0.max(end.0),
                            ury: start.1.max(end.1),
                        },
                    });
                }
            }
        }
        // ★★ A FORM CONTROL's rectangle, and it behaves exactly as the
        // text-annotation band above: the same band pixels, and a release that
        // AUTHORS NOTHING.
        //
        // That last property is the feature rather than an implementation
        // detail. Nothing exists in the document until the operator presses OK
        // in the dialog, which is what makes Escape free and what stops a
        // mis-drag leaving a stray field behind — and a stray form field is
        // worse than a stray rectangle, because it is invisible on a printed
        // page and swallows every keystroke aimed near it.
        //
        // The rect is normalised HERE, once, at the point it becomes a page
        // rect — the same rule the text-annotation arm states — so the band the
        // operator watched and the control that is authored cannot disagree
        // about which corner was the anchor.
        // O66 — the band, and the answer on release. Both bodies live in
        // `canvas::placing`, which is where the reasoning is; R2 leaves this
        // file six lines and they are spent on routing.
        GestureOutcome::Place {
            from, to, phase, ..
        } => {
            pv.band = Some(crate::canvas::placing::band(from, to));
            crate::canvas::placing::band_released(&ctx, doc, from, to, phase);
        }
        GestureOutcome::FormField {
            kind,
            from,
            to,
            phase,
        } => {
            pv.band = Some(markup::band::Preview {
                kind: kind.drag_shape(),
                from,
                to,
            });
            if phase == crate::canvas::gesture::Phase::Complete
                && let Some(page) = doc.current_page()
                && let Some((start, end)) = markup::band::endpoints(from, to, page)
            {
                actions.push(
                    FieldAction::Begin {
                        page: page_index,
                        kind,
                        rect: pdfcer_core::page_tree::Rect {
                            llx: start.0.min(end.0),
                            lly: start.1.min(end.1),
                            urx: start.0.max(end.0),
                            ury: start.1.max(end.1),
                        },
                    }
                    .into(),
                );
            }
        }
        // ★★ A resize drag COMMITS, as of 2026-08-19.
        //
        // The comment that stood here read *"a resize drag is CONSUMED and
        // commits nothing … `pdfcer-core` has no scale verb, so there is nothing
        // to commit and nothing to preview either"*. The first clause is still
        // true — re-derived against the engine's source rather than taken from
        // this note — and the conclusion no longer follows: **scaling a path is
        // moving every one of its nodes**, and `move_nodes` takes a slice, so a
        // whole resize is one command and one undo entry. See
        // `canvas::resizing`.
        //
        // Consuming the drag is still load-bearing and is now a consequence
        // rather than the point: without it the drag would fall through to a
        // marquee, so aiming at a grip would replace the selection the operator
        // was trying to act on.
        // ★★ THE NINTH GRIP. `ui-conventions/handles.md` H2, and the third word
        // of the operator's *"reposition, resize, or rotate"*.
        //
        // Everything about the gesture is `canvas::rotating`'s: the bearing
        // between two rays from the selection's centre, the 15° snap under
        // Shift, the wrap that stops a drag past 180° spinning a whole turn, and
        // the single negation at the page crossing. This arm is wiring.
        //
        // ★ It needs the decomposition for the same reason `Resize` and
        // `Handle` do — the commit addresses paint-order indices and this shell
        // will not send unverified ones to a verb that rewrites bytes — so
        // `DimensionVertex`'s note in `needs_targets` applies to it too, and it
        // is in that list.
        GestureOutcome::Rotate { from, at, phase } => {
            pv.rotate_ghost = crate::canvas::rotating::drag(
                &ctx,
                crate::canvas::rotating::Frame {
                    from,
                    at,
                    phase,
                    // ★★★ `grabbable`, NOT `overlay::grip_box` — the one line
                    // the annotation rotation hangs on; `canvas::rotating`'s
                    // header is the argument.
                    bounds: crate::canvas::pressing::grabbable(&ctx, doc, map, &selection).bounds,
                    page_index,
                    constrain: shift,
                    map: Some(map),
                    page: doc.current_page(),
                    // ★ Resolved HERE so a dimension with no sidecar record
                    // declines in words rather than in silence.
                    dimension: dimdrag::selected(doc, &selection).map(|(id, _)| id),
                },
                &selection,
                actions,
            );
        }
        // ★★★ A TEXT BOX being dragged out. `ui-conventions` has no row for
        // this because it is not a convention question — it is the file
        // format's: a PDF has no paragraph, so multi-line text needs a width to
        // wrap against, and a width is a rectangle the operator draws.
        //
        // The band is `markup::band::preview_rect`, reused rather than given its
        // own painter, for the reason the text-annotation band gives about the
        // same reuse: a rubber band is a rubber band, and two sets of pixels for
        // one gesture would be two things to keep level.
        //
        // On `Complete` it **opens a draft and authors nothing** — which is the
        // whole difference from a markup band, and why `canvas::textedit` gets
        // the box rather than `apply` getting an action.
        GestureOutcome::TextBox { from, to, phase } => {
            pv.band = Some(markup::band::Preview {
                kind: markup::MarkupKind::Rectangle,
                from,
                to,
            });
            if phase == crate::canvas::gesture::Phase::Complete
                && let Some(page) = doc.current_page()
            {
                crate::canvas::textedit::begin_box(&ctx, doc, page_index, from, to, page);
            }
        }
        // ★ The resize, routed. Three destinations share these eight grips —
        // page content, a markup annotation and a form field's box — and
        // `canvas::resizing` is the decision. Moved out of this arm's body on
        // 2026-08-28 for the reason every other arm here states: this is
        // wiring, and the rules are unit-tested without a window.
        GestureOutcome::Resize { grip, delta, phase } => {
            pv.resize_ghost = crate::canvas::resizing::drag(
                crate::canvas::resizing::Frame {
                    // ★ Read live from `egui::Memory`, at the frame the commit
                    // happens. See `canvas::scaling` for why they are a
                    // per-drag modifier rather than a preference.
                    modifiers: crate::canvas::scaling::read(&ctx),
                    grip,
                    delta,
                    phase,
                    bounds: crate::canvas::pressing::grabbable(&ctx, doc, map, &selection).bounds,
                    page_index,
                    constrain: crate::canvas::constrain::resize(&ctx, shift),
                    map: Some(map),
                    page: doc.current_page(),
                    selected_field: doc.selected_field.as_ref(),
                },
                &selection,
                targets.as_deref(),
                actions,
            )
            .map(|f| (grip, f));
        }
        // ★ A perimeter's corner. Routed beside the Bézier handle it is
        // modelled on, and to a different verb - `move_dimension_vertex` rather
        // than `move_handle` - which is the whole reason they are two drag
        // kinds rather than one with a flag in it.
        //
        // It reuses `dimension_preview` because the picture is the same shape:
        // page-space segments of a polyline redrawn from the geometry the
        // release will commit. A ghosted outline would be the wrong picture
        // here for the same reason it is wrong for a label drag - moving one
        // corner stretches two segments rather than translating a box.
        GestureOutcome::DimensionVertex {
            index,
            from,
            at,
            phase,
        } => {
            // ★ SHIFT LOCKS A CORNER TO ONE AXIS, measured from the PRESS —
            // so the grab point survives (`drag-moves` D8).
            let at = crate::canvas::constrain::reposition(&ctx, shift, from, at);
            // ★ ALT SUSPENDS THE SNAP, read live and asked of the same
            // `snap_query_enabled` a measure pick asks. It is what makes a
            // generous catch radius affordable: the offer is refusable, so it
            // can afford to be eager.
            let alt_held = ctx.input(|i| i.modifiers.alt);
            let dragged = dimdrag::drag_vertex(
                dimdrag::VertexFrame {
                    ctx: &ctx,
                    index,
                    from,
                    at,
                    phase,
                    doc,
                    selection: &selection,
                    targets: targets.as_deref().map(|t| t as &dyn CanvasTargetProvider),
                    map,
                    alt_held,
                },
                actions,
            );
            pv.dimension = dragged.segments;
            // ★ The candidate travels to the painter rather than being
            // re-queried there, which is `measure::Resolved`'s whole reason for
            // existing: a marker resolved a second time is a second derivation,
            // and this project has already shipped one that sat away from the
            // point it described for four days because a raw screen position
            // and a converted canvas one are the same type.
            pv.vertex_snap = dragged.snap;
        }
        GestureOutcome::Handle {
            node,
            handle,
            at,
            phase,
        } => {
            // ★★ SHIFT LOCKS A CONTROL POINT TO ITS ANCHOR'S AXIS — the
            // reference point is the ANCHOR, not the press, and that is the one
            // place the four constrained drags differ. A handle's meaning is
            // the tangent it defines, so the line that matters runs through the
            // on-curve point; Illustrator and Inkscape both lock it there.
            //
            // A `None` anchor leaves the drag UNCONSTRAINED rather than
            // refusing it, and announces nothing — the operator asked to move a
            // handle, and declining the whole gesture because a modifier could
            // not be honoured is a worse answer than honouring the gesture
            // without it. See `handledrag::anchor` for why it is looked up only
            // when Shift is down.
            let origin = shift
                .then(|| doc.pages.get(page_index).zip(targets.as_deref()))
                .flatten()
                .and_then(|(page, provider)| {
                    crate::canvas::handledrag::anchor(&selection, provider, page, node)
                });
            let at = match origin {
                Some(origin) => crate::canvas::constrain::reposition(&ctx, true, origin, at),
                None => at,
            };
            pv.handle = crate::canvas::handledrag::drag(
                crate::canvas::handledrag::Frame {
                    node,
                    handle,
                    at,
                    phase,
                    page_index,
                    map: Some(map),
                    page: doc.pages.get(page_index),
                },
                &selection,
                targets.as_deref(),
                actions,
            )
            .map(|p| (node, handle, p));
        }
        // `Cancelled` draws nothing, commits nothing, and its only remaining
        // effect is to keep Escape away from the ladder at step 6.
        GestureOutcome::Cancelled | GestureOutcome::Idle => {}
    }

    // ---- 5b. the right-click ---------------------------------------------
    //
    // ★ Split to [`super::rightclick`] under **R2** on 2026-08-28, when the
    // fourth canvas menu took this file past 1,500 lines. One subject —
    // *which menu does a secondary click open* — with three hit tests and a
    // frame-ordering hazard, all of which belong together and none of which
    // belongs in the middle of a gesture pipeline.
    let tokens = super::rightclick::attach(super::rightclick::Click {
        response,
        ctx: &ctx,
        doc,
        caps: &caps,
        selection: &mut selection,
        targets: targets.as_deref(),
        screen_pos,
        map,
        page_index,
        secondary_clicked,
        host,
    });

    // ---- 6. keys, BEFORE the resolve -----------------------------------
    //
    // Escape ascends a rung, which changes which box each entry outlines —
    // the part's, or the object's. Running the keys after the resolve would
    // leave the outline one frame behind the rung, visible as the part's box
    // lingering for a frame after the operator stepped out of it.
    //
    // `Cancelled` at step 3 means Escape was spent abandoning a drag, and one
    // press may have one effect: the ladder must not also ascend, or an
    // operator who cancels a move drag finds they have left the rung they were
    // working in as well.
    // ---- 6a. the text selection's own two keys ---------------------------
    //
    // Ctrl+A and Ctrl+C, before `canvas_keys` and separate from it. Every rule
    // about them — why they are not in `canvas_keys`, why they are gated on the
    // press predicate, and why the chord is read before the extraction is
    // fetched — is in `textsel::keys`, with the 392 ms measurement that earned
    // the last of them.
    textsel::keys(
        &ctx,
        doc,
        page_index,
        active_tool,
        caps,
        &mut text_selection,
    );

    keys::canvas_keys(
        keys::Keys {
            ctx: &ctx,
            page_index,
            caps,
            // ★ The third surface Delete has to know about, and the one it
            // could not see. See the field's own doc on `keys::Keys`.
            selected_field: doc.selected_field.as_ref(),
            annot_delete_refused: crate::panels::properties::annotdelete::refuses(doc, &selection),
            field_delete_refused: crate::panels::properties::formfield::refuses_delete(doc),
            escape_consumed: matches!(outcome, GestureOutcome::Cancelled),
        },
        &mut selection,
        &mut text_selection,
        actions,
    );

    // ---- 7. re-resolve -------------------------------------------------
    //
    // ★ A decomposition is attempted whenever a resolve is due, and `resolve`
    // records the key either way. Both halves matter and the pairing is easy
    // to get wrong:
    //
    // * passing `None` when no decomposition was ATTEMPTED would clear the
    //   outlines and then record the key, so they would never come back —
    //   a selection that is still selected and no longer drawn;
    // * not recording the key on a failed attempt would re-decompose an
    //   undecodable page on every single frame.
    // `!needs_targets` rather than `targets.is_none()`: a build that was
    // already attempted and failed must not be attempted a second time in the
    // same frame.
    if !needs_targets && selection.needs_resolve(page_index, doc.edit_epoch) {
        targets = doc.page_objects();
    }
    selection.resolve(
        targets.as_ref().map(|t| &**t as &dyn CanvasTargetProvider),
        page_index,
        doc.edit_epoch,
    );
    // ★ Both measure affordances, resolved in one call while the
    // decomposition is still borrowed.
    //
    // They were forty lines of this function until R2 asked for four back.
    // Moving them together rather than one at a time is deliberate: they share
    // the constraint that made them fiddly — the `Ref` is dropped before
    // anything paints, so both queries must happen HERE — and a reader who
    // finds one should find the other beside it. See `measure::resolve::frame`.
    let measure_hover = measure::resolve::frame(
        &ctx,
        doc,
        page_index,
        active_tool.measure_kind(),
        screen_pos.map(|p| map.to_page(p)),
        targets.as_deref().map(|t| t as &dyn CanvasTargetProvider),
        map,
    );

    // The decomposition is a `Ref` into `doc`, and `Ref` implements `Drop` —
    // so its borrow does NOT end at its last use, and the store at the bottom
    // of this function needs `&mut doc`. Dropped explicitly rather than by
    // shuffling the code into a block, because the ordering is a real
    // constraint worth naming: nothing below this line may read the
    // decomposition.
    drop(targets);
    // ★★★ O63's third piece, applied at the first point in this function where
    // `&mut doc` is available again. See `hold_after_drop`'s declaration.
    if let Some(hold) = hold_after_drop {
        doc.hold_preview(hold);
    }

    // ---- 7b. the released zoom marquee ----------------------------------
    //
    // Here rather than in its arm because arming an anchor writes to `doc` and
    // the decomposition's `Ref` has only just been released. The outcome is
    // reported by `zoom` on the diagnostic channel; there is nothing for this
    // frame to do with it, because a region zoom that hit the raster ceiling
    // still zooms — to the ceiling, centred on the region — and the status
    // bar's readout states the scale that was actually pinned. See
    // [`zoom::ZoomOutcome::ceiling_changed_the_answer`].
    if let Some(rect) = pv.zoom_region {
        let _ = zoom::zoom_to_rect(&ctx, doc, rect, CANVAS_MARGIN, *max_zoom_percent, actions);
    }

    // ---- 7c. a bookmark's destination ------------------------------------
    //
    // One call, because the whole subject lives in `canvas::destination` —
    // including why it drains through the same framing the zoom marquee uses
    // and why it is a one-shot.
    crate::canvas::destination::arrive(&ctx, doc, *max_zoom_percent, actions);

    // ---- 8. draw --------------------------------------------------------
    //
    // ★ Lifted to `canvas::painting` on 2026-08-19 — see that module's header
    // for why this is the seam. Everything above decides; that decides nothing.
    crate::canvas::painting::draw(
        ui,
        &ctx,
        doc,
        pages,
        &crate::canvas::painting::Frame {
            page_index,
            clip,
            map,
            selection: &selection,
            marquee: pv.marquee,
            ghost: pv.ghost,
            shape_preview: pv.shape.as_ref(),
            annot_ghost: pv.annot_ghost,
            resize_ghost: pv.resize_ghost,
            handle_drag: pv.handle,
            dimension_preview: pv.dimension.as_deref(),
            vertex_snap: pv.vertex_snap,
            rotate_ghost: pv.rotate_ghost,
            band: pv.band,
            text_marks: pv.text_marks.clone(),
            ink_trail: pv.ink_trail,
            active_tool,
            pen,
            screen_pos,
            find,
            text_selection: text_selection.as_ref(),
            measure_hover,
        },
    );

    if active_tool.text_edit_kind().is_some() {
        // typing-guard-exempt: SELF-REFERENTIAL. This asks whether a widget is
        // stealing the keyboard FROM the canvas draft, and the draft is the
        // thing asking — `composing()` here would include the draft itself and
        // be permanently false, so the caret would never take a keystroke.
        let owns_keyboard = !ctx.text_edit_focused();
        let _ = crate::canvas::textedit::keys::typing(ui, &ctx, doc, owns_keyboard, actions);
        // ★ Evidence for *"it doesn't type anything in the box when I type and
        // nothing gets added"* — the operator, 2026-08-18. Four facts, each
        // killing a different hypothesis: `draft=false` (the click stored
        // none), `owns_keyboard=false` (a `TextEdit` has focus, so `typing`
        // reads no events), `text_events=0` (egui delivered none — the keys
        // are not reaching this window), `len` not rising (read and stored,
        // insert not landing).
        //
        // ★ Why a trace rather than a test: the driven check for text editing
        // seeds the draft through `PDFCER_DIAG_TYPE`, the one path that BYPASSES
        // the event loop, so it passes on a build where real typing is dead.
        // Until a check types for real, this line is what tells them apart.
        crate::diag::trace_on_change("text-edit-typing", || {
            let draft = crate::canvas::textedit::read(&ctx);
            let text_events = ctx.input(|i| {
                i.events
                    .iter()
                    .filter(|e| matches!(e, egui::Event::Text(t) if !t.is_empty()))
                    .count()
            });
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                // ★ The ANCHOR, added 2026-08-21. `kind` is the armed TOOL and
                // the anchor is what the caret is actually on, and the two can
                // disagree: an `Add` tool whose click landed on a run edits it,
                // and a dragged box is an `Add` tool with a rectangle. The
                // multi-line work needed to tell those apart from a trace and
                // could not, which cost an investigation.
                "kind={:?} anchor={} draft={} owns_keyboard={owns_keyboard} text_events={text_events} len={}",
                active_tool.text_edit_kind(),
                draft.as_ref().map_or("none", |d| match d.anchor {
                    crate::canvas::textedit::Anchor::Run { .. } => "run",
                    crate::canvas::textedit::Anchor::Origin { .. } => "origin",
                    crate::canvas::textedit::Anchor::Box { .. } => "box",
                }),
                draft.is_some(),
                draft.map_or(0, |d| d.text.chars().count()),
            )
        });
    }

    // ★ The cursor — the whole precedence in one pure function, in `tool`,
    // where the first rung of it already lived. See [`tool::cursor_for`]; this
    // is only the gathering of the four facts that are knowable nowhere but
    // here. `clip` is the scroll VIEWPORT rather than the page's rect, because
    // the hand pans the grey surround as readily as the paper.
    let pointer_down = ctx.input(|i| i.pointer.primary_down() || i.pointer.middle_down());
    let over_canvas = ctx.pointer_latest_pos().is_some_and(|p| clip.contains(p));
    let icon = tool::cursor_for(
        active_tool,
        gestures.active(),
        hovered_grip.filter(|_| response.hovered()),
        pointer_down,
        over_canvas,
    );
    if let Some(icon) = icon {
        ctx.set_cursor_icon(icon);
    }

    // ★ …and where that answer is a CROSSHAIR, supply our own bitmap.
    //
    // The operator, 2026-08-18: *"The crosshairs when over the canvas are white
    // making it hard to see them."* Nothing in this crate drew them — the
    // platform's stock crosshair is monochrome and its colour belongs to the
    // operator's pointer scheme, which no application can read. So pdfcer stops
    // asking for it and hands the OS a two-tone bitmap instead; the full
    // argument, including why it is not inverted and why the two tones are not
    // theme colours, is in `canvas::cursor`'s header.
    //
    // The icon is still set above and is the **fallback**: `egui-winit` drops
    // to `cursor_icon` on any integration or platform that cannot take a
    // bitmap, so the stock crosshair remains the answer where ours cannot be.
    //
    // Only set here, never cleared here. `app::frame` clears it once per frame
    // before anything draws — see the note there for why a canvas-local clear
    // would miss the frames that matter. `cursor::apply` also emits the one
    // trace line this feature has, because a cursor is the one thing in this
    // application a screenshot cannot contain: Windows composites the pointer
    // separately, so `ui-verify`'s window capture returns an image with no
    // cursor in it at any price.
    // ★★ …and where that answer is an I-BEAM, turn it to match the text under
    // the pointer.
    //
    // The operator, 2026-08-26: *"In Adobe when I hover over it the I cursor
    // re-orients itself to match the text orientation […] as it is now the I
    // cursor doesn't reorient."* Acrobat is right — the I-beam's meaning is
    // *"text flows this way"*, and over a 90° title-block stamp an upright one
    // says it about the wrong axis. `canvas::cursor::Tilt` carries the argument
    // and the reason this shell can do it at all where most applications
    // cannot.
    //
    // ★ **The cost question answers itself here, and that is why the tilt is
    // applied at this point and not in `tool::cursor_for`.** Turning the beam
    // needs the page's EXTRACTION — 355 ms on the benchmark sheet the first
    // time, cached on `(page, edit epoch)` thereafter — and `arm.rs` records at
    // length why this canvas refuses to hit-test the extraction on hover in
    // general: it would be paid on every reading canvas, which is most of them.
    //
    // That objection does not reach this line, because `icon` is only ever
    // `Text` in the two situations where the extraction is wanted anyway:
    //
    //   * the **Text or Caret tool is armed**, which is the operator saying
    //     outright that the next press is about text, and whose very next press
    //     builds the extraction regardless; or
    //   * a **text sweep is already in flight**, in which case it is built.
    //
    // A reader in Read mode who never touches text never reaches here, gets no
    // extraction, and pays nothing — which is exactly the property `arm.rs`
    // protects.
    let shape = icon
        .and_then(crate::canvas::cursor::Shape::of)
        .map(|shape| {
            let crate::canvas::cursor::Shape::Ibeam(_) = shape else {
                return shape;
            };
            let tilt = screen_pos
                .filter(|p| clip.contains(*p))
                .and_then(|p| {
                    let page = doc.pages.get(page_index)?;
                    let page_text = doc.page_text()?;
                    let text_ctx = textsel::PageContext {
                        text: &page_text,
                        page,
                        index: page_index,
                        epoch: doc.edit_epoch,
                    };
                    textsel::tilt_at(&text_ctx, map.to_page(p))
                })
                .map_or(crate::canvas::cursor::Tilt::UPRIGHT, |degrees| {
                    crate::canvas::cursor::Tilt::nearest(degrees)
                });
            crate::canvas::cursor::Shape::Ibeam(tilt)
        });
    crate::canvas::cursor::apply(&ctx, shape);

    let count = selection.len();
    // Back onto the document, by value. Moved rather than cloned: a marquee
    // over a dense sheet can select thousands of entries, and cloning that per
    // frame at 60 Hz would be a real cost for no reason.
    doc.selection = selection;
    // …and the text selection, on the same argument twice over: it carries the
    // copied string, which for a select-all over a dense sheet is the whole
    // page's text.
    doc.text_selection = text_selection;
    store_gesture(&ctx, gestures);
    (count, tokens)
}
