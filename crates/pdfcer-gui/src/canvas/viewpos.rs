//! # `canvas::viewpos` — **where the view sits this frame**, decided before
//! the strip is drawn
//!
//!
//! ## Why this is the seam
//!
//! [`super::present::show_in`] does two separable things in sequence. First it
//! settles **where the view is** — how much pasteboard slack this frame has,
//! whether the zoom is deep enough that the `f32` scroll offset can no longer
//! address a pixel, and which of six competing sources gets to say what the
//! scroll offset is. Only then does it draw: the strip, the pages, the
//! overlays, the selection, the gestures.
//!
//! The two halves communicate through exactly three values — `overhang`,
//! `deep`, and an optional forced offset — and nothing in the second half
//! writes anything the first half reads. That is a seam rather than a cut: a
//! reader here needs to know nothing about how a page is painted, and a reader
//! there needs to know nothing about the anchor handshake or the deep tier's
//! hand-over.
//!
//! ## What this module does NOT own
//!
//! Each of the decisions below belongs to a module of its own and is *called*
//! from here, not made here: [`super::tier`] measures the overhang,
//! [`super::deep`] owns the two hand-overs across the deep threshold,
//! [`super::fit`] spends a fit command's placement request, and
//! [`super::offset`] ranks the six sources. This module is the **order** they
//! happen in and the one frame's worth of geometry they all read — which is
//! itself load-bearing, and is why it is one function rather than four calls
//! scattered through a 1,300-line one.
//!
//! ## The one thing it deliberately does not do
//!
//! It does not apply the offset. [`Position::offset`] is returned, and
//! `show_in` is where the `ScrollArea` is configured — for the same reason
//! [`super::offset::decide`] returns rather than applies: **one place
//! configures the area**, so a second opinion cannot be added by accident.

use egui::Vec2;

use crate::app::state::OpenDoc;
use crate::canvas::tool::CanvasTool;
use crate::canvas::{deep, fit, offset, zoom};
use crate::viewer;

/// **This frame's geometry**, as the placement decision needs it.
///
/// Every field is already bound in [`super::present::show_in`] before this is
/// called; none is derived here. Grouped into a struct rather than passed as
/// seven positional arguments because five of them are sizes and three of
/// those are `(f32, f32)` — an argument order a caller can get wrong in
/// silence is a defect waiting for a resize.
pub(super) struct Geometry {
    /// The page the view is currently about.
    pub current: usize,
    /// That page's drawn size.
    pub current_display: (f32, f32),
    /// The rect of the **row** holding it — the page's own rect outside a
    /// facing mode, the spread's union inside one. O177.
    pub row_rect: egui::Rect,
    /// The whole strip's size.
    pub display_size: Vec2,
    /// The viewport, measured **inside** the scroll bars. Every margin term in
    /// `geometry` and every centre in [`fit::placement`] is derived against
    /// this number, so it must be the room the content will actually get.
    pub vp: Vec2,
    /// The tool in force, which two of the decisions below consult.
    pub active_tool: CanvasTool,
}

/// **What the drawing half of the frame needs to know** about where the view
/// ended up.
///
/// Three values, and the fact that it is only three is what makes the split
/// above a seam. See the module header.
pub(super) struct Position {
    /// This frame's pasteboard slack, in drawn points, each side. Also written
    /// to the document — see the write's own note for why it is not threaded
    /// as an argument to the eight `geometry` calls that want it.
    pub overhang: Vec2,
    /// Whether the zoom is past the point where an `f32` scroll offset can
    /// address a screen pixel, and the `f64` anchor owns the position instead.
    pub deep: bool,
    /// The scroll offset this frame is being forced to, if any source won.
    /// **Returned rather than applied** — see the module header.
    pub offset: Option<Vec2>,
}

/// **Settle where the view is, before anything is drawn.**
///
/// Called once per canvas frame, from [`super::present::show_in`], immediately
/// before the `ScrollArea` is shown. Mutates `doc` in three places — the
/// published overhang, the deep tier's anchor bookkeeping inside
/// [`deep::track`], and the frame counter — and every one of those is a fact
/// about *this* frame that a later frame reads.
pub(super) fn position(
    ui: &egui::Ui,
    doc: &mut OpenDoc,
    layout: &viewer::strip::Strip,
    g: Geometry,
) -> Position {
    let Geometry {
        current,
        current_display,
        row_rect,
        display_size,
        vp,
        active_tool,
    } = g;

    // Zoom to the anchor, half two: a zoom step was armed on an earlier frame
    // and the new zoom is now known (post-clamp), so solve for the offset that
    // keeps the anchored page point where the rule says it belongs, and force
    // it onto the area before it lays out. `consume_anchor` owns the gate that
    // decides whether the zoom has actually landed yet — see [`zoom`]'s header
    // on the two-frame handshake, and on why an unconditional `take()` here
    // made every *command*-driven zoom silently unanchored.
    //
    // ★ Three sources of a forced scroll offset, and the order between them is
    // a precedence rather than a coincidence:
    //
    // 1. **a zoom anchor**, because a zoom has just landed and the whole point
    //    of the anchor is that one page point does not move as it does;
    // 2. **a find reveal**, because the operator asked to be taken somewhere
    //    and a one-shot navigation outranks nothing else in flight;
    // 3. **a middle-drag pan**, which is a live gesture — and a live gesture is
    //    LAST here for the reason it wins anyway: it re-arms itself on the next
    //    frame, while both of the others are spent once.
    //
    // ★ A **fourth** source arrives with Phase 4 — a page *command* under a
    // continuous mode, which has to scroll the strip to the page it named —
    // and it sits third, below the two one-shots and above the live gesture,
    // by the same reasoning: it is a one-shot the operator asked for, and a
    // live gesture re-arms itself while the one-shots are spent once.
    //
    // ★ Two of the three offsets below are solved by code this work does not
    // own — `canvas::zoom`'s anchor handshake and `find::reveal`'s two-frame
    // reveal — and both are written for a scroll area whose content is **one
    // page at the origin**. Rather than teach either about a strip, the canvas
    // converts: `geometry::page_local_offset` presents the world the way those
    // solves expect, and `geometry::strip_offset` converts their answer back.
    // The conversion is exact, and under `Single` it is the identity. See
    // `geometry`'s header for the whole argument.

    // ★★★ **THE PASTEBOARD'S OVERHANG, PUBLISHED ONCE FOR THE WHOLE FRAME.**
    //
    // O23's third and last part. Parts A and B gave the operator slack to
    // scroll into and made a press out there a gesture; this is what makes the
    // off-page object survive being **zoomed in on**, which is what "edit"
    // means and is the half his follow-up report was about:
    //
    // > *"how do I view and edit objects that are off of the page? we added
    // > this feature but I didn't see how to enable it."*
    //
    // `geometry::pasteboard` carries the arithmetic and the measurement. In
    // one line: the slack was a fixed count of **screen pixels**, so the slice
    // of the **drawing** it covered shrank in exact proportion to the zoom,
    // and above a few hundred per cent an object placed off the sheet could no
    // longer be brought to the middle of the screen at all.
    //
    // ★ Written to the document rather than threaded as an argument because
    // eight call sites below hand it to `geometry`, and two spellings of the
    // pasteboard is the exact defect O23 spent three attempts on — see the
    // field's own documentation.
    //
    doc.pasteboard_overhang = super::tier::overhang(doc, current);
    let overhang = doc.pasteboard_overhang;
    super::trace::pasteboard(overhang, doc.view.off_page);

    // The page the pending zoom anchor was armed against, and that page's
    // drawn size — which is what `zoom::consume_anchor` must compare its
    // recorded size against, for the same reason.
    let anchor_page = doc.frame.zoom_anchor.map_or(current, |a| a.page);
    let anchor_display = layout
        .rect_of(anchor_page)
        .map_or(current_display, |r| (r.width(), r.height()));
    // ★★★ TIER 3 — the `f64` anchor takes over the POSITION.
    //
    // `OPERATOR_REQUESTS.md` O24. Below this the scroll offset says where the
    // view is, and it is an `f32` over a content space of `page × zoom` where
    // one unit is one screen pixel — so past 2^24 content points it can only
    // address every second pixel, then every sixteenth, and the view judders
    // and sticks. The step is a property of the CONTENT EXTENT rather than of
    // the zoom: measured at an extent of 2.05e10 it was 2,048 px, and the
    // hand-over is set far below that, at
    // `viewer::ceiling::SUB_PIXEL_CONTENT_EXTENT`, because holding the point
    // under the cursor gives out long before addressing a pixel does.
    //
    // Above it the position becomes `DeepAnchor` — a page point in `f64` and
    // the screen pixel it sits under — which does not decay with zoom, and
    // the scroll area stops being asked to hold it. Its content becomes the
    // viewport, so egui has nothing to scroll and nothing to round.
    //
    // ★ Everything below the threshold is untouched, deliberately. This
    // canvas has twice been broken by a change that meant to affect only deep
    // zoom, so the tier is a hard branch rather than a re-parameterisation.
    let deep =
        viewer::deep_position_needed(viewer::page_extent_pts(&doc.pages[current]), doc.view.zoom);
    let deep_handover = deep::track(
        ui,
        doc,
        layout,
        current,
        current_display,
        display_size,
        vp,
        active_tool,
        deep,
    );

    // ★ Where a fit command puts the view — `OPERATOR_REQUESTS.md` O28, and
    // the whole of it is in `canvas::fit` because it is a rule about fitting
    // rather than about this frame's geometry.
    //
    // Taken unconditionally, even at the deep tier and even on a frame where
    // something else wins the offset: a request left pending would fire on
    // whatever frame the chain next reached it, which is a view that jumps for
    // a button pressed some seconds ago.
    // ★ `zoom::last_frame` is read HERE and handed in, rather than read inside
    // `fit::placement`, so the "before" state is fetched once per frame at the
    // point that already owns the frame's geometry — and so the function stays
    // a pure decision over its arguments, which is what makes its arithmetic
    // unit-testable without a window. O78.
    let previous = zoom::last_frame(ui.ctx());
    let fit_placement = fit::placement(
        doc,
        current_display,
        row_rect,
        display_size,
        vp,
        previous,
        // The page this frame is about. `acting` is not bound yet at this
        // point in the frame — it is resolved after the strip lays out — and
        // `current` is what every other pre-layout decision here uses. The two
        // differ only on the frames `acting`'s own note describes, and a
        // disagreement here can only DECLINE a centre-preservation, never
        // misplace one.
        current,
    );
    // ★★★ WHO DECIDES WHERE THE VIEW IS THIS FRAME, in one ranked list.
    //
    // Six sources, and the ranking is the whole of the subject — see
    // `canvas::offset`'s header for each one's argument. It returns an offset
    // rather than applying it, so the `ScrollArea` is configured in exactly
    // one place.
    let decision = offset::decide(
        ui,
        doc,
        layout,
        active_tool,
        offset::Frame {
            deep,
            deep_handover,
            fit_placement,
            anchor_page,
            anchor_display,
            current,
            current_display,
            row_rect,
            display_size,
            vp,
        },
    );

    // ★ Published BEFORE the frame counter moves, so `frames=` on the line is
    // the value `offset::decide` actually branched on rather than the value the
    // next frame will see. An off-by-one here would make the open-seed arm look
    // as though it fired on the wrong frame, which is the single question this
    // line was added to answer.
    super::trace::placed(
        &decision,
        doc.canvas_frames,
        display_size,
        row_rect,
        vp,
        overhang,
    );
    let offset = decision.offset;

    // How many canvas frames this document has had. Saturating, and only ever
    // read against a small constant — by `offset::decide`'s open-seed arm,
    // which is the only reader and is one call up this same function.
    doc.canvas_frames = doc.canvas_frames.saturating_add(1);

    Position {
        overhang,
        deep,
        offset,
    }
}
