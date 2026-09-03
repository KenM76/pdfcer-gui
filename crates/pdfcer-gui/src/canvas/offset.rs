//! # `canvas::offset` — who decides where the view is, this frame
//!
//! Split out of [`super`] under **R2** on 2026-08-24. It is the one subject in
//! `canvas::show` that is genuinely a *decision procedure* rather than a
//! drawing step, and it had grown to six ranked sources.
//!
//! ## ★★★ The ranking, and why it is the whole of the subject
//!
//! Every source below can be right on the same frame, and the order is the
//! only thing standing between them. Highest first:
//!
//! | # | source | why it outranks the one below |
//! |---|---|---|
//! | 1 | **the deep tier's forced zero** | above the threshold the content IS the viewport, so zero is the only valid offset. Not a preference — anything else is a frame that renders a different part of the page |
//! | 2 | **the hand-over out of the deep tier** | this frame is the one that left it, and the `f64` anchor holds the position the `f32` machinery is about to inherit |
//! | 3 | **a fit command's placement** | the operator's most recent explicit instruction about the view. It also SPENDS a pending zoom anchor: a wheel anchor armed a frame earlier says "hold this page point", and a fit has just decided the page goes somewhere else |
//! | 4 | **an anchored zoom** | the whole point of the anchor is that one page point does not move as the zoom does |
//! | 5 | **a find reveal** | the operator asked to be taken somewhere, and a one-shot navigation outranks nothing else in flight |
//! | 6 | **a middle-drag pan** | a live gesture — and it is LAST for the reason it wins anyway: it re-arms itself on the next frame, while every one-shot above it is spent once |
//!
//! A **seventh** arrives with Phase 4 — a page *command* under a continuous
//! mode, which has to scroll the strip to the page it named — and it sits
//! between 5 and 6, by the same reasoning: a one-shot the operator asked for,
//! above a gesture that re-arms itself.
//!
//! ## Why it returns an offset instead of configuring the area
//!
//! So the `ScrollArea` is built in exactly one place. A procedure that both
//! decided and applied would make "which branch won?" answerable only by
//! reading the whole chain, and the frame where two branches both applied
//! would look identical to the frame where the right one did.

use egui::{Vec2, vec2};

use crate::app::state::OpenDoc;
use crate::canvas::geometry;
use crate::canvas::input::pan_delta;
use crate::canvas::tool::CanvasTool;
use crate::canvas::zoom;
use crate::viewer;

/// This frame's geometry and its already-solved one-shots, gathered so the
/// decision below reads as a ranked list rather than as an argument list.
///
/// ★ A struct rather than nine parameters because the ranking is what a reader
/// comes here for, and nine positional arguments at the call site would be the
/// thing they had to read first. Every field is `Copy` and small.
pub(super) struct Frame {
    /// Whether the `f64` position tier owns the view this frame.
    pub deep: bool,
    /// The offset handed back by the `f64` anchor on the frame that leaves the
    /// deep tier, and `None` on every other frame.
    pub deep_handover: Option<Vec2>,
    /// The page-local offset a fit command asked for, if one is pending.
    pub fit_placement: Option<Vec2>,
    /// The page a pending zoom anchor was armed against, and that page's drawn
    /// size — see `viewer::ZoomAnchor::page` for why the anchor names a page.
    pub anchor_page: usize,
    /// See [`Self::anchor_page`].
    pub anchor_display: (f32, f32),
    /// The acting page.
    pub current: usize,
    /// The acting page's drawn size.
    pub current_display: (f32, f32),
    /// The whole strip's drawn size.
    pub display_size: Vec2,
    /// The viewport, measured before the scroll area was built.
    pub vp: Vec2,
}

/// **The offset this frame's `ScrollArea` should be forced to**, or `None` to
/// leave it wherever the operator left it.
///
/// See the module header for the ranking. The body below is that table in
/// code, in the same order, and the comments on each arm are the ones that
/// were written when each source was added.
pub(super) fn decide(
    ui: &egui::Ui,
    doc: &mut OpenDoc,
    layout: &viewer::strip::Strip,
    active_tool: CanvasTool,
    frame: Frame,
) -> Option<Vec2> {
    let Frame {
        deep,
        deep_handover,
        fit_placement,
        anchor_page,
        anchor_display,
        current,
        current_display,
        display_size,
        vp,
    } = frame;
    // The strip conversion every page-local answer below is handed back
    // through. Spelled here rather than passed in as a closure so this module
    // can be read on its own; it is the same one `canvas::show` uses, and
    // `canvas::geometry`'s header carries the argument for why it exists.
    let strip_offset_for = |page: usize, local: (f32, f32)| {
        let rect = layout
            .rect_of(page)
            .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, display_size));
        let (x, y) = geometry::strip_offset(
            local,
            (rect.min.x, rect.min.y),
            (display_size.x, display_size.y),
            (rect.width(), rect.height()),
            (vp.x, vp.y),
        );
        vec2(x, y)
    };
    let to_strip = |local: (f32, f32)| strip_offset_for(current, local);

    if deep {
        // ★★★ FORCE THE SCROLL OFFSET TO ZERO — `OPERATOR_REQUESTS.md` O24f.
        //
        // At this tier the content IS the viewport, so zero is the only valid
        // offset and egui will clamp to it. **One frame later**, which is the
        // whole problem: on the frame the tier flips, the area is still
        // carrying the offset it settled on while the position was still
        // its to hold — measured at 6,264,562 px — and `outer_rect.min` is
        // inside that scrolled content. The anchor then places the strip
        // relative to an origin that is itself displaced by the old offset,
        // so the page lands at roughly TWICE the intended distance and the
        // view is gone.
        //
        // Measured at the hand-over, 2,047,244 % → 2,181,987 %: the position
        // line said the page origin should be 6,676,376 px left of the
        // viewport and the page was drawn 12,940,650 px left of it. The
        // difference is 6,264,274 — the stale scroll offset, to four
        // significant figures.
        //
        // ★ Assigned rather than left to the clamp because a one-frame
        // discrepancy is not cosmetic here: the raster region is computed
        // from the same placement, so the frame is not merely misplaced, it
        // renders a different part of the page.
        return Some(vec2(0.0, 0.0));
    } else if let Some(offset) = deep_handover {
        // ★ FIRST, above the ordinary anchor: this frame is the one that left
        // the `f64` tier, and the offset solved above is the position the
        // anchor was actually holding. See the branch that produced it.
        return Some(to_strip((offset.x, offset.y)));
    } else if let Some(offset) = fit_placement {
        // ★★ ABOVE THE ZOOM ANCHOR, and it spends one if it finds it — O28.
        //
        // A fit is the operator's most recent explicit instruction about the
        // view. A wheel anchor armed a frame earlier says "hold this page
        // point where it was", and a fit has just decided that the page goes
        // somewhere else; letting the anchor win would make Fit page do
        // nothing whenever the operator had touched the wheel immediately
        // before pressing it, which is exactly when they would.
        //
        // Spent through `consume_anchor` rather than by clearing the field, so
        // the `waited` bookkeeping inside it stays consistent. Its answer is
        // discarded.
        let _ = zoom::consume_anchor(ui.ctx(), doc, anchor_display);
        return Some(strip_offset_for(current, (offset.x, offset.y)));
    } else if let Some(offset) = zoom::consume_anchor(ui.ctx(), doc, anchor_display) {
        return Some(strip_offset_for(anchor_page, (offset.x, offset.y)));
    } else if let Some(offset) = crate::find::take_reveal_offset(doc, current_display, (vp.x, vp.y))
    {
        // The other half of `Action::Find`'s navigation: the page change was
        // applied after the frame that asked for it, and this is the first
        // frame that is actually showing that page — so it is the first frame
        // on which the page's real drawn size is known and the offset can be
        // solved. `crate::find` owns both the gate and the solve; nothing
        // about a search is decided here.
        //
        // The reveal's gate is `reveal.page == view.page_index`, so the page it
        // solves against is always the current one — which is exactly the page
        // `to_strip` converts for. A reveal therefore lands on the right page
        // of a continuous strip without `find::reveal` knowing a strip exists.
        // ★ The side effect runs BEFORE the return, which it did not need to
        // when this chain assigned to a `ScrollArea` builder in place. The
        // reveal has navigated, so the page it landed on is the one being
        // tracked.
        doc.tracked_page = doc.view.page_index;
        return Some(to_strip((offset.x, offset.y)));
    } else if let Some(offset) = crate::canvas::strip::page_scroll_offset(doc, layout, (vp.x, vp.y))
    {
        return Some(offset);
    } else if let Some(pan) = pan_delta(ui, active_tool) {
        // Panning subtracts the pointer delta: the content follows the hand,
        // so the page moves WITH the pointer rather than under it.
        let (x, y) = geometry::pan_offset(
            (doc.last_scroll_offset.x, doc.last_scroll_offset.y),
            (pan.x, pan.y),
            (display_size.x, display_size.y),
            (vp.x, vp.y),
        );
        // ★★★ **A PAN NO LONGER LEAVES THE FIT**, as of 2026-08-31, and the
        // reversal is recorded here rather than in a commit message because
        // the line it removes was written to the operator's own words.
        //
        // `OPERATOR_REQUESTS.md` **O55**, 2026-08-28:
        //
        // > *"if the canvas window is resized the pdf should resize to match
        // > unless the person has changed the zoom **or panned around**."*
        //
        // `OPERATOR_REQUESTS.md` **O78**, 2026-08-31:
        //
        // > *"unless I have manually changed the zoom after clicking one of
        // > the preset options, the pdf should maintain whichever option was
        // > selected."*
        //
        // **The pan clause is gone from the condition**, and the same message
        // says why it could be: *"whatever area was centered in the current
        // canvas should stay centered."*
        //
        // ## ⇒ Why the clause was only ever load-bearing by accident
        //
        // A fit is a rule about **zoom**; where the operator is looking is a
        // rule about **position**. Until today nothing owned the second, so a
        // resize under a live fit *re-placed* the view — and the only defence
        // available for an operator who had panned somewhere deliberately was
        // to stop them being in a fit at all. Leaving the mode was a proxy for
        // defending the position.
        //
        // `canvas::fit::placement` now preserves the centred page point across
        // **any** viewport change, in or out of a fit, so the position defends
        // itself. The proxy is unnecessary, and keeping it would cost the
        // operator the thing he asked for twice: a page that stops re-fitting
        // the moment he drags it an inch.
        //
        // ★★ The two of his sentences are then both true at once, which is the
        // test a reading of a changed request has to pass. Preserving the
        // centre also SUBSUMES the fit's old re-placement — on a pinned axis
        // the two solve to the same number, which `canvas::geometry`'s
        // `centring_agrees_with_the_pinned_fit_answer` pins — so nothing that
        // worked before this stopped working.
        //
        // ## ★ What still leaves a fit, and it is the only thing
        //
        // `ViewState::set_zoom`. Changing the zoom by hand is the operator
        // saying the view should stop tracking the viewport, and it is exactly
        // the clause that survives in both of his sentences.
        //
        // The wheel never left the fit and still does not, on the argument
        // this comment has always carried: scrolling a fit-width document is
        // how every reader in the class is read.
        // The gesture has to look like what it is. Without a cursor change a
        // pan that hits the end of the scroll range is indistinguishable from
        // a pan that is not working. ★ Before the return, for the reason the
        // reveal arm above states.
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        return Some(vec2(x, y));
    } else if doc.canvas_frames == 1 {
        // ★★★ SEED ON THE SECOND FRAME, NOT THE FIRST.
        //
        // O23. `ScrollArea` starts its offset at zero, which used to mean the
        // strip's top-left and now means the CONTENT's — one pasteboard above
        // and left of the page — so the view has to be placed once.
        //
        // ★ Doing that on the FIRST frame is what broke the two previous
        // attempts, and it took four bisecting runs to see. Forcing an offset
        // before egui has laid the content out once costs the canvas its
        // pointer input entirely: the page is drawn, centred and correctly
        // published, and no `canvas-pointer` event is ever emitted again.
        // Pre-writing `scroll_area::State` fails the same way from the other
        // side — it is silently clamped against a content size that is not
        // known yet.
        //
        // ★★ It is NOT the magnitude. `scrolling_far_keeps_the_canvas_its_
        // pointer_input` drives the wheel to 1,600 pt and the canvas keeps
        // its input, so a large offset is fine once the content is real.
        //
        // So: frame 0 lays out with egui's own zero, frame 1 places the view.
        // One frame of pasteboard is visible at open, which is the cost of
        // this shape and is named rather than hidden.
        // ★★★ **…and it is placed at the page's CENTRE, not its corner** —
        // `OPERATOR_REQUESTS.md` O78: *"when starting the view should be
        // centered on the canvas when a pdf is first opened."*
        //
        // `(0.0, 0.0)` page-local means "the page's own origin", which is
        // *centred if the page fits and flush to its top-left corner if it does
        // not*. Under the shipped default — Fit page — the page always fits, so
        // this expression evaluates to exactly `(0.0, 0.0)` and **nothing
        // changes**. What it changes is the case the operator is actually
        // looking at: an opening preference of Actual size on a sheet larger
        // than the window, where the old seed showed him the top-left corner of
        // an A1 drawing.
        //
        // ★ Written as the general solve rather than as a special case, so the
        // seed and the resize path answer the same question with the same
        // function. A second spelling of "centre the page" is how the two would
        // come to disagree — which is the defect `canvas::fit`'s header
        // describes for the fit's own placement, arrived at from the other end.
        return Some(to_strip(crate::canvas::geometry::offset_holding_anchor_at(
            (0.5, 0.5),
            (vp.x / 2.0, vp.y / 2.0),
            current_display,
            (vp.x, vp.y),
        )));
    }
    None
}
