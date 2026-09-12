//! # `canvas::present` — **drawing the canvas**: the scroll area, the pages in
//! it, and the geometry the frame hands back
//!
//! [`show`] and its body [`show_in`], plus the constants and helpers that only
//! they use. Everything else about the canvas — what a click means, what a drag
//! does, what is selected — lives in the sibling modules `canvas` indexes.
//!
//! ## Why this is its own file
//!
//! R2, and it is the second time of asking. `canvas/mod.rs` reached 1,501 lines
//! **twice on 2026-08-29** — once when the form-field clipboard added a module
//! to the index and again when the cut gate added another. Both times the
//! cheap fix was to shorten the new module's doc comment, and the first time
//! that is what happened, with a note saying the real seam was `show_in` and
//! that trimming was kicking the can.
//!
//! ⇒ It was. R2's own instruction is *"when a file approaches the limit, that is
//! the signal to find the seam, not to raise the limit"* — and a module index
//! that cannot accept a new entry without something else being deleted is a
//! file that has stopped being an index.
//!
//! ## The seam, stated
//!
//! `canvas/mod.rs` is now **only** a module index and the canvas header: 53
//! `pub mod` lines, each with the paragraph that says why that module exists.
//! Adding a 54th costs nothing and takes nothing away.
//!
//! This file is the one thing that file also happened to contain — a thousand
//! lines of *drawing*, which is a different subject from *what the canvas is
//! made of*. They change for different reasons, which is the test this project
//! applies to every split it makes.
//!
//! ## ★ Nothing moved except its address
//!
//! The move is textual: the same items, in the same order, with the same
//! documentation. `show` and [`Sampled`] are re-exported from `canvas`, so
//! every call site still says `canvas::show(...)` and no caller learned that
//! this file exists.

// ★★ **A glob, deliberately, and it is the honest shape for a textual move.**
//
// Every item below was written inside `canvas/mod.rs`, where all 53 sibling
// modules and every `use` at the top of that file were in scope by being in
// the same module. Re-listing them here would be a hand-maintained copy of a
// list the compiler already has — and the first attempt at this move did
// exactly that, missed four (`paging`, `deep`, `fit`, `offset`), and had to be
// told about each one by a separate compiler error.
//
// ⇒ The glob says what is true: *this file is the rest of its parent.* If it
// ever stops being that, the glob is the thing that will look wrong.
use super::*;

use egui::{Pos2, Rect, Sense, Vec2, scroll_area::ScrollSource, vec2};
use egui_shell::HandlerToken;

use crate::app::actions::Action;
use crate::app::modes::Capabilities;
use crate::app::state::OpenDoc;
// The interaction half, next door. `Frame` is this frame's settled facts on the
// way in; `interact` is everything that follows from them. Imported by name
// rather than called as `interact::interact(…)` so the one call site below
// reads exactly as it did before the split.
use crate::canvas::interact::{Frame, interact};
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::PickFilter;
use crate::canvas::rulers::CanvasGeometry;
use crate::shell::menus::MenuHost;
use crate::viewer;

/// Padding, in points, left around the page inside the canvas so the page
/// does not sit flush against the panel edges under a fit mode.
///
/// Subtracted from the viewport *before* the fit scale is derived rather
/// than added as a layout margin afterwards, so "fit page" really does fit
/// with the gap visible instead of fitting exactly and then being clipped
/// by the gap.
///
/// `pub` because [`zoom::zoom_to_rect`] fits a *region* by the identical rule
/// and must leave the identical gap — a framing command that pressed the
/// region flush against the panel edges while "Fit page" left 16 points would
/// read as two different ideas of what fitting means.
pub const CANVAS_MARGIN: f32 = 16.0;

/// ★★ **The three values the canvas samples once per frame**, bundled.
///
/// They were three separate parameters until 2026-08-21, when adding the
/// fourth-from-last put [`show`] over clippy's seven-argument ceiling. The
/// lint was right and the bundle is not a workaround for it: these three
/// already had a paragraph in [`interact::Frame`] arguing that they belong
/// together, and it reads as a definition of this type —
///
/// > `tool` is *what* the operator armed, `caps` is *whether* the mode
/// > permits it, and `pen` is *what it will look like*. All three are
/// > sampled once per frame and for the same reason — a gesture means what
/// > it meant when it started.
///
/// `tool` is not here because the canvas reads it from `egui::Memory`
/// itself; these three are the ones the application owns and must hand over.
///
/// # Why sampling matters more than tidiness
///
/// Every field is `Copy` and every one is read at the top of the frame, so
/// the canvas sees a **consistent** snapshot for the whole frame. A canvas
/// that re-read any of them mid-frame could start a drag under one mode and
/// finish it under another, which is the class of defect
/// `app::gating::on_mode_capabilities_changed` exists to prevent from the
/// other end.
#[derive(Debug, Clone, Copy)]
pub struct Sampled {
    /// **What the active mode lets this frame do to the document.**
    pub caps: Capabilities,
    /// **What a click on the page may land on** — the operator's selection
    /// filter (`OPERATOR_REQUESTS.md` O17).
    ///
    /// Composes with [`Self::caps`] as an `AND` in one direction only:
    /// switching a class on here can never grant a capability the mode
    /// withholds. See [`pick`](mod@pick).
    pub pick: PickFilter,
    /// ★ The operator's configured maximum zoom, as a percentage (O24).
    pub max_zoom_percent: f32,
    /// The colour and width the next markup will be authored with.
    pub pen: crate::canvas::markup::pen::Pen,
}

/// Draw the page, read the canvas gestures, and attach the canvas context
/// menus.
///
/// Operator intent leaves by two routes, and the split is not arbitrary:
///
/// * **`actions`** carries what the canvas itself decides — a zoom step, a
///   fit, a Delete raised by the Delete key. These are already `Action`s
///   because the canvas knows what they mean.
/// * **the return value** carries `egui_shell::HandlerToken`s: the commands
///   the operator chose from a context menu. The canvas must *not* translate
///   those, because translating them is what `PdfcerApp::dispatch_token`
///   does for the ribbon, and a second translation is how the two surfaces
///   start disagreeing about what `format.delete` means. Handing the token
///   on unchanged is what makes `RIBBON_IA.md` §5.8's *"carries the same
///   commands again"* literally true.
///
/// `host` is `None` when the application has no validated shell — see
/// [`MenuHost`] — in which case no menu is attached and a right-click does
/// nothing, which is the correct behaviour for a build with no menu
/// document rather than a disabled feature.
///
/// Beyond those two, everything this function decides lands in the three
/// documented bookkeeping fields (see the module docs). The document itself
/// is never touched.
///
/// # ★ The rulers wrap this, and the wrapping is three statements
///
/// [`rulers::reserve`] takes a **constant** bite out of `ui` before anything
/// measures the viewport (rule R128 — see that module's header §3), the whole
/// of the canvas is then drawn into a child `Ui` covering what is left, and
/// the gutters are painted afterwards from the geometry [`show_in`] hands
/// back. Painting them *after* is what puts a guide preview over the page
/// rather than under it, and what lets the ruler mark the page's own edges —
/// neither of which is knowable until the scroll area has settled.
#[must_use]
pub fn show(
    ui: &mut egui::Ui,
    doc: &mut OpenDoc,
    host: Option<&MenuHost<'_>>,
    find: &crate::find::FindState,
    sampled: Sampled,
    actions: &mut Vec<Action>,
) -> Vec<HandlerToken> {
    let gutters = rulers::reserve(ui, doc.view.rulers && !doc.pages.is_empty());
    let mut content = gutters.content_ui(ui);
    let (tokens, geometry) = show_in(&mut content, doc, host, find, sampled, actions);
    rulers::draw(ui, doc, gutters, geometry.as_ref());
    // Starting a guide drag needs a ruler to drag out of; *finishing* one does
    // not, because it may have started on the canvas. So the two halves are
    // separate calls and only the first is conditional — see `guides::settle`.
    guides::ruler_drag(ui, doc, gutters);
    guides::settle(ui, doc, geometry.as_ref(), actions);
    tokens
}

/// [`show`]'s body, drawn into the canvas region the rulers left.
///
/// Returns the context-menu tokens *and* what the frame learned about where
/// its pages ended up — see [`CanvasGeometry`] on why that has to travel
/// outwards rather than be read again.
#[must_use]
fn show_in(
    ui: &mut egui::Ui,
    doc: &mut OpenDoc,
    host: Option<&MenuHost<'_>>,
    find: &crate::find::FindState,
    sampled: Sampled,
    actions: &mut Vec<Action>,
) -> (Vec<HandlerToken>, Option<CanvasGeometry>) {
    let Sampled {
        caps,
        pick,
        max_zoom_percent,
        pen,
    } = sampled;
    if doc.pages.is_empty() {
        let placeholder = ui.centered_and_justified(|ui| ui.label(crate::text::canvas_no_pages()));
        // Say so on the trace rather than staying silent. A consumer that
        // finds no `canvas` line otherwise has to guess between "this build
        // does not trace its layout" and "there was no layout to trace", and
        // those need opposite responses. See `trace_layout`.
        crate::diag::trace_changed(trace::LAYOUT_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-unavailable reason=no-pages".to_owned()
        });
        crate::diag::ui_rect(trace::REGION_PAGE_MESSAGE, placeholder.inner.rect);
        return (Vec::new(), None);
    }

    // The viewport the fit modes are measured against, minus the margin the
    // page sits inside. Clamped to at least one point because a window
    // dragged to nothing would otherwise produce a zero or negative
    // viewport, and `fit_scale` would fall back to actual size on a window
    // the operator is still resizing — a visible jump at the end of a drag.
    //
    // ★★★ **MEASURED INSIDE THE SCROLL BARS** — 2026-09-08. The canvas's bars
    // are solid and take real width (see [`scroll_style`]), so the room the
    // page actually has is the outer size **minus** `allocated_width()` on
    // each axis. Every viewport this function reads is derived from this one
    // number — the fit here, `vp` below, and the `viewport_size` the scroll
    // area reports back — so the three cannot disagree. They disagreed for one
    // build: the fit was sized against the outer, the frame was recorded
    // against the inner, the centre rule placed against the outer, and the
    // page crept **7 px per frame** — half the bar's 14 pt — until it left the
    // screen. Measured on the driven `resize_scales_a_shape` run, not reasoned.
    //
    // The style is set HERE, before anything measures, because
    // `allocated_width()` reads it; setting it beside the `ScrollArea` builder
    // forty lines down would leave this measurement reading egui's floating
    // default (allocation 0) and the mismatch would be back.
    ui.style_mut().spacing.scroll = scroll_style();
    let inner_avail = ui.available_size() - Vec2::splat(ui.spacing().scroll.allocated_width());
    let viewport = (
        (inner_avail.x - CANVAS_MARGIN).max(1.0),
        (inner_avail.y - CANVAS_MARGIN).max(1.0),
    );

    // Resolve a fit mode against THIS frame's viewport. Under
    // `FitMode::None` this is a no-op, so it is safe to call always — and
    // calling it always is what makes "Fit page" a mode rather than a
    // one-shot: resize the window and the page re-fits.
    //
    // ★ Against the current **row**, not the current page, and the ceiling
    // against the row's tightest page. Under Single and Continuous a row is
    // one page and both reduce to exactly what they were; under a facing mode
    // a row is the spread, and fitting one half of a spread would leave the
    // other half off screen from a control called "Fit page". See
    // [`viewer::strip::row_metrics`] for why the row is measured without
    // laying the strip out — the strip's geometry depends on the zoom this
    // produces, so it cannot be the source of it.
    let pixels_per_point = ui.ctx().pixels_per_point();
    //
    // ★ `fit_metrics`, NOT `row_metrics`, and the difference is a closed
    // feedback loop. Under a continuous mode `page_index` is derived from the
    // scroll, so fitting the current row makes the zoom depend on the scroll
    // and the scroll depend on the zoom — measured oscillating between
    // `page=0 zoom=1.4773` and `page=1 zoom=0.9559` on a mixed-size document
    // for as long as the wheel was moving. `fit_metrics` fits the tightest row
    // under a continuous mode and the current row otherwise; on a document of
    // one page size the two are identical.
    let row = viewer::strip::fit_metrics(
        &doc.pages,
        doc.view.display,
        doc.view.page_index,
        pixels_per_point,
    );
    doc.view.apply_fit(row.extent, viewport, row.max_zoom);

    // ★ The whole-canvas render failure is the **single-page** answer, and it
    // stays exactly that.
    //
    // With one page on screen, "this page would not draw" is the only thing
    // there is to say and a sentence in the middle of the canvas is the right
    // way to say it. With several, replacing the entire strip with one
    // sentence would hide thirty-nine sheets that drew perfectly because one
    // did not — so every other mode draws the refusal **in the failing page's
    // own rectangle** instead. See [`crate::render::strip::draw_page_state`].
    if doc.view.display == viewer::PageDisplay::Single
        && let Some(message) = &doc.render_error
    {
        let text = crate::text::canvas_render_failed(message);
        let placeholder =
            ui.centered_and_justified(|ui| ui.colored_label(ui.visuals().error_fg_color, text));
        // Same argument as the no-pages arm: there is genuinely no page rect
        // this frame, and saying that is more useful than silence. `reason=`
        // is a fixed token, not the operator-facing message — the message can
        // be reworded and a consumer keying on it would break.
        crate::diag::trace_changed(trace::LAYOUT_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-unavailable reason=render-failed".to_owned()
        });
        crate::diag::ui_rect(trace::REGION_PAGE_MESSAGE, placeholder.inner.rect);
        return (Vec::new(), None);
    }

    // ★ **Where every page this view shows sits.** One page under `Single`,
    // whose rect is `(0,0)..display_size` — so everything below is the
    // arithmetic it already was. See [`viewer::strip`].
    let layout = doc.strip();
    let display_size = layout.size();
    let current = doc.view.page_index;
    // The current page's placement, which is the frame of reference every
    // single-page solve in `canvas::zoom` and `find::reveal` is handed. Falls
    // back to the whole strip for the degenerate case where the current page is
    // not laid out at all, which the page-index clamp normally prevents.
    let current_rect = layout
        .rect_of(current)
        .unwrap_or_else(|| Rect::from_min_size(Pos2::ZERO, display_size));
    let current_display = (current_rect.width(), current_rect.height());
    // The scale every page on screen is rasterized at. Derived once, here, and
    // used to look each visible page's raster up: deriving it a second time
    // inside the draw loop is how a page could be *drawn* against one key while
    // being *requested* against another, which would show as a page that never
    // stops saying it is not drawn yet.
    let raster_scale =
        viewer::raster_scale(doc.view.zoom, pixels_per_point, doc.prefs.render_quality);

    // `ScrollSource::ALL` = scroll bars + plain mouse wheel + drag-to-pan.
    // Ctrl+wheel never reaches this, because egui routes a modified wheel
    // event to `zoom_delta` instead of to the scroll delta.
    //
    // `drag` is switched OFF: egui's drag-to-scroll is button-agnostic, and
    // the primary button belongs to the S4 selection marquee. Panning is
    // the middle button, implemented below against the offset directly.
    let mut scroll_source = ScrollSource::ALL;
    scroll_source.drag = egui::scroll_area::DragScroll::Never;
    // ★★★ AND THE WHEEL ITSELF, when it is a page turn — O30.
    //
    // The scroll area consumes a plain wheel, so a page-turning wheel has to
    // be taken away from it BEFORE it is built; reading the delta afterwards
    // and paging on it as well would scroll and page off one gesture. The
    // decision and the spending are two places, one frame apart, and they ask
    // `paging::flips_pages` — one predicate, so they cannot disagree about
    // which frames are which.
    //
    // ★ Only the wheel. The scroll BARS keep working, which matters: with the
    // wheel turning pages, dragging the bar is how the operator moves within
    // a sheet that is larger than the window, and a mode that took both away
    // would have made a zoomed-in page unreachable.
    if paging::flips_pages(doc) {
        scroll_source.mouse_wheel = false;
    }

    // ★★★ **THE SCROLL BARS ARE SOLID AND BESIDE THE PAGE, NEVER OVER IT** —
    // 2026-09-08, the cause of the `resize_scales_a_shape` red after five
    // wrong explanations. See [`scroll_style`] for the measurement. The style
    // itself is applied at the top of this function, where the viewport is
    // measured, because the measurement depends on it.
    let mut scroll_area = egui::ScrollArea::both()
        .id_salt("page-canvas") // ui-text-exempt: internal widget id, never displayed
        .scroll_source(scroll_source)
        // ★ Always, not when needed. The content is the strip plus O23's
        // pasteboard — a whole viewport each side — so it exceeds the viewport
        // on every frame below the deep tier and the bars would show anyway.
        // Saying so outright makes the space they take a CONSTANT of the
        // layout rather than a value that appears one frame after the content
        // does, which is the shape of R128 (`D:/dev/rag/egui/`, fit-zoom
        // feedback): a viewport that changes size because of what was drawn
        // in it is a fit zoom that recomputes, which moves the page under a
        // click the operator has already aimed.
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);

    // Which tool the primary button is in this frame — the select tool, or the
    // hand (chosen, or borrowed for as long as the space bar is down). Read
    // once, here, and passed down: two readings could disagree within a frame
    // and the disagreement would be a drag that panned AND marquee'd.
    let active_tool = tool::active(ui.ctx());

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
    // ★ The INNER size — `inner_avail`, measured above with the bars taken
    // off. `ui.available_size()` here is the outer, and every margin term in
    // `geometry` and every centre in `fit::placement` is derived against this
    // number, so it must be the room the content will actually get.
    let vp = inner_avail;

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
    // ★★★ The MEASUREMENT moved to `tier::overhang` on 2026-09-11, with
    // `View ▸ Off-Page Content`. It is the same decision `tier::decide` makes
    // — *does this page reach past its sheet, and is the operator asking to be
    // shown it?* — asked against the layout instead of against the raster, and
    // the two halves of one switch must not live in two files. That function
    // carries the peek's 469 ms argument, the zoom multiplication and the
    // gate; what stays HERE is the single write, because the field must be
    // written exactly once per frame and this is that frame.
    doc.pasteboard_overhang = super::tier::overhang(doc, current);
    let overhang = doc.pasteboard_overhang;
    super::trace::pasteboard(overhang, doc.view.off_page);

    // The page the pending zoom anchor was armed against, and that page's
    // drawn size — which is what `zoom::consume_anchor` must compare its
    // recorded size against, for the same reason.
    let anchor_page = doc.zoom_anchor.map_or(current, |a| a.page);
    let anchor_display = layout
        .rect_of(anchor_page)
        .map_or(current_display, |r| (r.width(), r.height()));
    // ★★★ TIER 3 — the `f64` anchor takes over the POSITION.
    //
    // `OPERATOR_REQUESTS.md` O24. Below this the scroll offset says where the
    // view is, and it is an `f32` over a content space of `page × zoom` where
    // one unit is one screen pixel — so past about 2^24 content points it can
    // only address every second pixel, then every sixteenth, and the view
    // judders and sticks. Measured: at a trillion percent it moves in
    // 2,048-pixel jumps.
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
        &layout,
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
        current_rect,
        current_display,
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
    if let Some(offset) = offset::decide(
        ui,
        doc,
        &layout,
        active_tool,
        offset::Frame {
            deep,
            deep_handover,
            fit_placement,
            anchor_page,
            anchor_display,
            current,
            current_display,
            display_size,
            vp,
        },
    ) {
        scroll_area = scroll_area.scroll_offset(offset);
    }

    // How many canvas frames this document has had. Saturating, and only ever
    // read against a small constant — see the seeding arm above.
    doc.canvas_frames = doc.canvas_frames.saturating_add(1);

    let scroll_output = scroll_area.show(ui, |ui| {
        // Centre the STRIP manually rather than with
        // `ui.centered_and_justified`, because that helper returns the
        // JUSTIFIED CONTAINER rect — the whole available area — while
        // drawing the image centred inside it. Taking that rect as
        // `image_rect` makes every page↔screen mapping wrong by the centring
        // margin whenever the content is smaller than the viewport.
        //
        // The symptom in the old GUI was severe and specific: at "Fit page"
        // on a page narrower/shorter than the canvas, selection outlines
        // drew offset from the object they outlined (~105 px on one measured
        // case — exactly the vertical margin), and clicking directly ON a
        // visible object MISSED it. At high zoom, where the page exceeds the
        // viewport and the margin is zero, the same click landed perfectly.
        // That is the giveaway: the error scaled with the margin, not with
        // the zoom — and it was worst at exactly the zoom an operator uses
        // to see a whole page.
        //
        // So: reserve `max(strip, viewport)` so the ScrollArea still scrolls
        // when the content is larger AND there is a margin to centre within
        // when it is smaller, then place each page at an explicit rect
        // derived from the strip's own. `Ui::put` and `allocate_rect` return
        // a Response whose `.rect` IS that rect, so every page's screen rect
        // is its true drawn rect by construction rather than by coincidence.
        let avail = ui.available_size();
        // ★★ O23's pasteboard. Measured against `vp`, the viewport taken
        // BEFORE the scroll area is built — never `avail`, which is measured
        // inside it and therefore depends on whether scrollbars are showing,
        // which the pasteboard is what causes. That feedback is R128.
        // (Since 2026-09-08 the bars are always visible and `vp` is measured
        // with their allocation already taken off, so the two agree — but the
        // rule stands: `vp` is the one measurement, `avail` is egui's echo.)
        // ★★ TIER 3 takes the content down to the viewport. There is then
        // nothing for egui to scroll and nothing for it to round: the
        // position is the anchor's, and the strip is placed from it below.
        let outer = if deep {
            avail
        } else {
            vec2(
                geometry::content_extent(display_size.x, vp.x, overhang.x).max(avail.x),
                geometry::content_extent(display_size.y, vp.y, overhang.y).max(avail.y),
            )
        };
        // ★★ The response is KEPT, and Ctrl+wheel is gated on it —
        // `OPERATOR_REQUESTS.md` O26. It covers the whole scroll content:
        // every page, the gaps between them, and O23's pasteboard. See the
        // wheel block near the end of `show` for why the current page's own
        // response was the wrong gate.
        //
        // ★★★ **AND IT SENSES CLICKS — this `Sense` was O23's one-way door.**
        // It read `Sense::hover()` from the day the pasteboard shipped until
        // 2026-09-10, and that single word was the whole of why the operator
        // could not find the off-page feature: there was a viewport of slack
        // to scroll into on every side, and a press out there was not a
        // gesture, so an object dragged past the sheet edge became invisible,
        // unclickable, and still in the file. Everything underneath already
        // coped — `mapping::PageMapping::to_page` does not clamp, the engine's
        // decomposer does not cull to the page box, and `hit_test_point_all`
        // asks only that the query point be finite.
        //
        // ★★ **Widening this does NOT steal clicks from the pages.** This
        // rectangle is allocated BEFORE any page, and egui resolves an overlap
        // in favour of the widget registered later, so every page still wins
        // on its own sheet and this one only ever sees a pointer no page
        // wanted. That ordering is load-bearing: reverse it and every click on
        // paper would land out here instead. It is also why the fix is not the
        // obvious one of widening each page's own interaction rect — in a
        // continuous strip those overlap each OTHER, and the last page
        // allocated would steal clicks aimed at its neighbours.
        //
        // Which of the two responses a frame's gesture belongs to is decided
        // by [`super::pasteboard::surface`], at the `interact` call site near
        // the end of `show`.
        let (outer_rect, content_response) = ui.allocate_exact_size(outer, Sense::click_and_drag());
        // The strip's own rect on screen. Every page's rect is this origin
        // plus its strip-space placement, which is what makes the strip the
        // single owner of "where is page N".
        // ★★★ WHERE THE STRIP SITS, and at tier 3 the anchor decides.
        //
        // Below: centred in the content, as it has always been, with the
        // scroll offset moving the viewport over it.
        //
        // Above: the anchor says *this page point is under that screen
        // pixel*, so the current page's origin lands at
        // `anchor.screen - anchor.page × zoom` and the strip follows from it.
        // Every large magnitude is subtracted inside `f64` before anything
        // narrows — which is the whole technique, and the same one the
        // engine's own deep-zoom commit describes as "one subtraction moved
        // into f64".
        let strip_rect = if deep {
            deep::strip_placement(doc, &layout, current, outer_rect.min, display_size)
        } else {
            // ★★★ PLACED FROM THE CONTENT'S ORIGIN, NOT FROM ITS CENTRE —
            // `OPERATOR_REQUESTS.md` O26g.
            //
            // `Rect::from_center_size(outer_rect.center(), display_size)` is
            // the same rectangle and was a catastrophic cancellation: it forms
            // `centre − strip/2`, and in a continuous mode the strip is
            // `pages × page_height × zoom`. On a 36-page drawing set at a
            // million percent that is 4.6 × 10⁸ logical points, where an
            // `f32`'s step is **32 points** — so the strip's origin, every
            // page rect derived from it, the zoom anchor's `frac`, the raster
            // region and the pointer mapping were all quantised to 32 points.
            // Measured: an anchored zoom notch slid the view 16 points there,
            // and 10 points at 292,415 %, both of which the step size predicts.
            //
            // `strip_origin_offset` evaluates the same quantity symbolically —
            // a centring margin that is exactly zero once the strip exceeds
            // the viewport, plus one viewport of pasteboard — so no large
            // intermediate is formed and the origin is exact at every zoom.
            Rect::from_min_size(
                Pos2::new(
                    outer_rect.min.x
                        + geometry::strip_origin_offset(display_size.x, vp.x, avail.x, overhang.x),
                    outer_rect.min.y
                        + geometry::strip_origin_offset(display_size.y, vp.y, avail.y, overhang.y),
                ),
                display_size,
            )
        };
        let strip_origin = strip_rect.min.to_vec2();

        // The viewport, expressed in strip space — what decides which pages
        // are drawn at all. `last_scroll_offset` is the previous frame's
        // settled offset, which is the best estimate available *before* this
        // frame's is known; a page that appears one frame late during a fast
        // fling is the cost, and it is bounded by one frame.
        // ★★★ CONVERTED OUT OF CONTENT SPACE FIRST. O23.
        //
        // `last_scroll_offset` is where the SCROLL AREA is, measured from the
        // content's origin; this rect is intersected with the STRIP's layout.
        // Before the pasteboard those were the same space. With one they
        // differ by exactly the margin, and feeding the raw offset in puts
        // this rect a whole pasteboard past the end of the strip — so
        // `layout.visible()` returns nothing and the canvas draws nothing at
        // all.
        // ★ At tier 3 the scroll offset is not the position, so asking it
        // which pages are visible would answer about the wrong place. The
        // strip's own placement on screen is the truth there: whatever of it
        // overlaps the viewport is what can be seen.
        let visible_rect = if deep {
            deep::visible_in_strip(outer_rect.min, strip_rect.min, avail)
        } else {
            Rect::from_min_size(
                Pos2::new(
                    geometry::scroll_to_strip(
                        doc.last_scroll_offset.x,
                        display_size.x,
                        vp.x,
                        overhang.x,
                    ),
                    geometry::scroll_to_strip(
                        doc.last_scroll_offset.y,
                        display_size.y,
                        vp.y,
                        overhang.y,
                    ),
                ),
                avail,
            )
        };

        // ★★★ **A PAGE IS VISIBLE WHEN ITS CONTENT IS, NOT WHEN ITS SHEET IS.**
        //
        // `OPERATOR_REQUESTS.md` **O23**, third verb, measured on 2026-09-11 by
        // `ui-verify`'s `an_object_off_the_page_survives_being_zoomed_in_on`.
        //
        // `Strip::visible` culls on the PAGE's rectangle, and that was the only
        // truth there was until this shell learned to draw outside it. With the
        // halo tier a page's *content* can reach far past its own sheet — the
        // whole point of the feature — so the sheet can slide entirely off the
        // viewport while the object the operator is editing is sitting in the
        // middle of the screen. The canvas then laid out **nothing**, traced
        // `canvas-unavailable reason=nothing-visible`, dropped its `page` and
        // `canvas-viewport` rects, and went grey.
        //
        // ★★ Measured, not reasoned: at 978 % on a 200 pt sheet with an object
        // 160 pt off its left edge, the page rect stood at x 1562…3486 against
        // a viewport of 288…1578 — sixteen points of overlap, then none — while
        // that object spanned x −4…1170, squarely inside it. The operator was
        // looking straight at his object and the shell had decided there was
        // nothing to draw.
        //
        // ★ Expanding the VIEW by the overhang is exactly equivalent to
        // expanding each page's rect by it, and is one operation instead of
        // one per page. It is also the conservative direction: the worst a too
        // generous bound can do is rasterize a page a fraction of a second
        // early at a row boundary, where the worst a too tight one does is
        // what the paragraph above describes.
        //
        // `overhang` is already this frame's content reach multiplied by the
        // zoom — the same number the pasteboard is built from, published once
        // by `OpenDoc::pasteboard_overhang` — so it is in strip points here,
        // which is the space `visible_rect` is in.
        let visible_content = visible_rect.expand2(overhang);

        // `click_and_drag`, not `hover`, on EVERY page — not only the current
        // one. A press on a page the operator is not currently "on" is how
        // they move to it under a continuous mode, and a page that did not
        // sense the press would swallow the click entirely: the operator would
        // have to click twice, once to arrive and once to act, with nothing on
        // screen to say why. Both branches must also agree with each other —
        // a first frame that reserved the space with a different sense would
        // swallow the click that opened the document, experienced as "the
        // first click never works".
        let sense = Sense::click_and_drag();
        let mut drawn: Vec<strip::DrawnPage> = Vec::new();

        // ★★★ **Which picture this page needs** — the region / halo / whole
        // decision, in `canvas::tier`. Decided here, from the canvas, because
        // only the canvas knows where the operator is looking; kept in its own
        // module because it is the only block in this file that decides what to
        // ASK the renderer for rather than what to draw with what arrived.
        //
        // Its whole output is `doc.raster_region`, which `OpenDoc::region_for`
        // feeds into both the cache key and the engine request — so the draw
        // loop below needs to be told nothing.
        tier::decide(
            doc,
            &layout,
            current,
            raster_scale,
            deep,
            // ★ The TRUE viewport, deliberately, and not `visible_content`.
            // The region tier already intersects this with
            // `halo::reach(place, ..)` — the page's box WIDENED to its
            // content — so it reaches off-page ink without help. Handing it
            // the inflated window instead would grow the region raster with
            // `overhang × zoom` and break the one property that makes that
            // tier safe at any magnification: its device size is a multiple
            // of the WINDOW, never of the document.
            visible_rect,
            avail,
        );

        for placement in layout.visible(visible_content) {
            let rect = placement.rect.translate(strip_origin);
            let key = doc.render_key_for(placement.page, raster_scale);
            // The current page's raster lives in its own slot; every other
            // page's lives in the strip cache. See `render::strip`'s header
            // for why the split exists and why the rule is enforced rather
            // than remembered.
            // ★★★ THE TEXTURE AND THE REGION IT IS A PICTURE OF, TOGETHER.
            //
            // `OPERATOR_REQUESTS.md` O24c. The current page's slot is served
            // WITHOUT a staleness check, on purpose — that is what shows the
            // last good picture during a pan instead of blank paper, and it is
            // the operator's explicit requirement that detail never has to be
            // waited for after moving. The consequence is that these pixels
            // may be a picture of a DIFFERENT region from the one the shell
            // now wants, and placing them at the wanted region's rect is what
            // made the page lurch backwards whenever a pan crossed
            // `render::strategy::region_for`'s half-viewport grid line.
            //
            // So the region travels with the pixels. See
            // `render::worker::RenderKey::region` for the full report and why
            // rejecting the stale texture would have been the wrong fix.
            let held = if placement.page == current {
                doc.page_texture
                    .as_ref()
                    // ★ And only if it is a picture of THIS page. The slot is
                    // normally kept in step with the current page by
                    // `render::settle`, but a raster that lands in the same
                    // frame as a page change would otherwise be placed by a
                    // region computed for its neighbour — a rectangle that is
                    // perfectly valid and completely wrong, which is the
                    // failure mode `OpenDoc::region_for`'s own page check
                    // exists to prevent one level up.
                    .filter(|t| t.key.page() == placement.page)
                    .map(|t| (t.texture.clone(), t.key.region()))
            } else {
                doc.strip_page_texture(placement.page, key)
                    .map(|t| (t.texture.clone(), t.key.region()))
            };
            let has_raster = held.is_some();
            // ★★ Where the texture goes. A whole-page raster fills the page's
            // rect; a REGION raster covers only part of the page and must be
            // drawn at that part's rect, or the operator sees the right pixels
            // in the wrong place — which reads as the page having jumped.
            //
            // ★★★ The region read here is the HELD texture's, never
            // `doc.region_for(..)`. Those differ exactly while a new region's
            // raster is in flight, which under a pan is most of the time — and
            // using the wanted one is O24c, the backwards lurch at every grid
            // crossing. The page's own screen rect and the anchor below are
            // still the CURRENT ones, which is right: they say where page
            // space is now, and the held region says which part of page space
            // these pixels are. Together they slide the stale picture along
            // with the page instead of pinning it.
            //
            // The rect is deliberately NOT clamped to the page: the region
            // carries overscan, so it legitimately reaches outside. Clamping
            // the destination without cropping the source would stretch the
            // image, which is a subtler wrong than a misplaced one.
            let paint_rect = match held.as_ref().and_then(|(_, region)| *region) {
                // ★★★ At tier 3 the placement comes from the ANCHOR, not from
                // this page's screen rect.
                //
                // `rect` has a magnitude around 10^12 px at four billion
                // percent, where an `f32`'s spacing is 131,072 px — coarser
                // than the entire window — and the thing being drawn is about
                // 1,400 px across. Deriving the small result from the huge
                // intermediate inherits an error that never had to exist.
                //
                // ★ Which is why the answer to "32-bit strip or 64-bit strip?"
                // is neither: not forming the number beats carrying it more
                // precisely, costs nothing, and leaves one code path.
                Some(region) if deep => {
                    let anchor = doc
                        .deep_anchor
                        .unwrap_or_else(viewer::deep::DeepAnchor::origin);
                    crate::render::region::region_on_screen_deep(
                        region,
                        crate::render::region::PageFrame::of(&doc.pages[placement.page]),
                        anchor,
                        f64::from(doc.view.zoom),
                        outer_rect.min,
                    )
                }
                Some(region) => crate::render::region::region_on_screen(
                    region,
                    viewer::page_extent_pts(&doc.pages[placement.page]),
                    crate::render::region::PageFrame::of(&doc.pages[placement.page]),
                    rect,
                ),
                None => rect,
            };
            // ★ The backdrop, and the coverage number that makes its absence
            // falsifiable. Both live in `canvas::backdrop`; see that module for
            // why a screenshot is the wrong oracle for this one thing.
            let backdrop = backdrop::paint(ui, doc, placement.page, current, rect);
            backdrop::publish_coverage(
                ui,
                doc,
                rect,
                paint_rect,
                backdrop,
                held.is_some(),
                placement.page == current,
            );
            let response = match held {
                Some((texture, _)) => {
                    // ★ The IMAGE goes at `paint_rect`; the page's INTERACTION
                    // stays at `rect`. Allocating the image's rect would give
                    // the page a hit area that reaches off the sheet and
                    // overlaps its neighbours in a continuous strip.
                    // `Image::paint_at` rather than `painter().image(..)`: the
                    // latter needs an explicit tint, and the identity tint is a
                    // raw white that `check-theme-colors` rejects — correctly,
                    // since a colour with no role in the palette is one a
                    // restyle cannot reach. This form has no colour at all.
                    egui::Image::from_texture(&texture).paint_at(ui, paint_rect);
                    // ★★★ WHERE THE SHEET ENDS, when the picture is bigger
                    // than it.
                    //
                    // A halo raster covers the ground outside the page, and
                    // the engine flattens every raster over white (§11.4.7
                    // page group), so that ground arrives as WHITE PAPER. Left
                    // alone, the operator would see one white rectangle with
                    // no way to tell which part of it is his page — and the
                    // answer decides what saves, what prints and what a page
                    // resize would crop.
                    //
                    // So the boundary is drawn as chrome: the same
                    // `render::strip::boundary_stroke` an undrawn page uses,
                    // whose own comment states the rule this inherits — *"the
                    // boundary is a real fact about the document, so it is
                    // drawn at full strength rather than as a hint."*
                    //
                    // ★ R8b rule 4 is satisfied and it is worth saying why,
                    // because a line drawn near content usually is not: this
                    // marks the SHEET, not the object. Nothing about the
                    // off-page content is styled, tinted or flagged — it
                    // renders exactly as it will render when saved and
                    // reopened, and the only difference is that the operator
                    // can now see where the paper stops.
                    //
                    // ★ The condition also picks up the region tier's
                    // overscan, which has always bled a little white past the
                    // sheet edge at deep zoom with nothing to say so.
                    if !rect.contains_rect(paint_rect) {
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            crate::render::strip::boundary_stroke(ui.visuals()),
                            egui::StrokeKind::Inside,
                        );
                    }
                    ui.allocate_rect(rect, sense)
                }
                None => {
                    // No raster. Reserve the same rect with the same sense so
                    // nothing jumps when one arrives, then SAY what is
                    // happening rather than leaving white paper — see
                    // `render::strip::draw_page_state` on why a blank
                    // rectangle would be a placeholder and this is not.
                    let response = ui.allocate_rect(rect, sense);
                    let state = if placement.page == current {
                        strip::current_page_state(doc)
                    } else {
                        doc.strip_page_state(placement.page, key)
                    };
                    if let Some(state) = state {
                        crate::render::strip::draw_page_state(
                            ui.painter(),
                            ui.visuals(),
                            rect,
                            // The scroll viewport in SCREEN terms — which
                            // inside a `ScrollArea` is exactly the `Ui`'s clip
                            // rect. It is passed so the state sentence is
                            // centred in the part of the page the operator can
                            // actually see: a page whose top edge is showing
                            // and whose middle is a metre below the window
                            // would otherwise draw as a silent empty
                            // rectangle. See `draw_page_state`.
                            ui.clip_rect(),
                            placement.page + 1,
                            &state,
                        );
                    }
                    response
                }
            };
            drawn.push(strip::DrawnPage {
                page: placement.page,
                rect,
                response,
                has_raster,
                paint_rect,
            });
        }

        // `avail` rides out with the pages because it is the viewport the
        // zoom-to-cursor solve needs, and it is only knowable in here — the
        // same `avail` that decided `outer` above, so the margin the solve
        // reconstructs is the margin this frame actually drew.
        //
        // ★ The strip's own rect used to ride out with it, so that
        // `remember_frame` could subtract it from the page's to recover the
        // page's place *within the strip*. O26e replaced that reconstruction
        // with `geometry::offset_from_drawn`, which measures against the
        // viewport instead and therefore needs nothing from in here that the
        // pages do not already carry.
        // ★★ The whole `Response` now rides out, not just its `hovered()`.
        // O23's off-page half needs it: it is the OTHER of the two surfaces a
        // gesture can belong to, and `interact` — which runs after the closure
        // has closed — has to be able to be handed it. `egui::Response` is a
        // plain cloneable struct with no borrow of the `Ui`, so carrying it out
        // costs nothing and borrows nothing. `content_hovered` is still
        // computed in here, unchanged, because the Ctrl+wheel gate below reads
        // only that and reading it at two different moments would be a second
        // way to be wrong.
        (drawn, avail, content_response.hovered(), content_response)
    });

    let (drawn, viewport_size, content_hovered, content_response) = scroll_output.inner;
    // The offset the area settled on THIS frame: the `offset_before` of any
    // zoom step the operator starts now, and the base the next frame's
    // middle-drag pan moves from.
    doc.last_scroll_offset = scroll_output.state.offset;
    let scroll_offset = scroll_output.state.offset;

    // ★ **Which page this frame's input is about.** Decided by the strip
    // module, which owns the question and the two answers to it — the scroll
    // and a press. See `strip::track_current_page`.
    strip::track_current_page(
        doc,
        &layout,
        &drawn,
        (scroll_offset.x, scroll_offset.y),
        display_size,
        viewport_size,
        deep,
    );

    // ★ **A page drag from somewhere else, landing here.**
    //
    // Here rather than inside the scroll-area closure, because that is the
    // first point at which every visible page's *screen* rectangle is known —
    // and the caret has to be drawn over the pages rather than under them,
    // which in an immediate-mode painter is a matter of call order.
    //
    // It reads no `Response`, which is the point: the press that started this
    // drag happened in a panel — possibly on another document's page list —
    // so no widget on this canvas has ever seen it. See `canvas::pagedrop`.
    //
    // Costs one `egui::Memory` lookup on a frame with no drag in flight, which
    // is every frame but the handful the operator is carrying something.
    pagedrop::offer(ui, doc, &drawn, scroll_output.inner_rect, actions);

    // The page being acted on: whatever the pointer acted on, else the current
    // page. Its response and its rect are what everything below reads, which is
    // what keeps `interact` a single-page function — the selection, the hit
    // test and the decomposition all describe one page, and that page is this
    // one.
    let Some(active) = drawn
        .iter()
        .find(|d| d.page == doc.view.page_index)
        .or_else(|| drawn.first())
    else {
        // Nothing was laid out at all: a strip whose visible window fell
        // outside every page, which happens for one frame after a mode change
        // before the scroll area has settled. Say so, and let the next frame
        // sort it out rather than inventing a rect for a page nobody drew.
        crate::diag::trace_changed(trace::LAYOUT_SLOT, || {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-unavailable reason=nothing-visible".to_owned()
        });
        return (Vec::new(), None);
    };
    // ★★★ THE ACTING PAGE IS THE ONE WHOSE RECT WE ARE ABOUT TO USE —
    // `OPERATOR_REQUESTS.md` O26c.
    //
    // This used to be `doc.view.page_index`, decided *before* the fallback
    // above and then never revisited. When the current page was not among the
    // drawn ones the fallback took `drawn.first()` — a **different page** —
    // and the next two lines then paired **that page's rect** with **the
    // current page's extent**.
    //
    // ★★ On a document whose sheets are all the same size that mismatch is
    // invisible. `SW41177.pdf` mixes 1584 × 1224 sheets with 1224 × 792 ones,
    // and the trace caught it exactly:
    //
    // ```text
    // canvas rect=[[-5634238.0 681671.0] - [5515170.0 7895993.0]] zoom=9108.99
    // canvas-pos … ext=1584.000,1224.000
    // ```
    //
    // 11,149,400 × 7,214,300 is 1224 × 792 at that zoom, while `ext` says the
    // page is 1584 × 1224. Every consequence follows from those two lines
    // disagreeing: [`PageMapping`] is built from both, so the pointer maps to
    // a page point that is not where the pointer is (the same frame reported
    // `page=(618.59, −74.79)` for a pointer sitting well inside the sheet),
    // the zoom anchor's `frac` is taken from that, the next frame's solve asks
    // for an offset far outside the scrollable range, `strip_offset` clamps it
    // to zero — and **zero is the content's top-left corner**. The page lands
    // in a corner of the screen with everything else off it, which is exactly
    // what the operator described and exactly why it was intermittent: it
    // needs `drawn.first()` to be a *differently sized* page.
    //
    // ★ Taking the page from `active` also makes the frame's whole downstream
    // truthful rather than merely consistent: `interact`, the hit test, the
    // selection and `region_for` all describe the page whose rectangle is on
    // screen, instead of one the operator cannot see. Acting on an invisible
    // page was never the intent — it was the fallback's silence.
    let acting = active.page;
    let image_response = active.response.clone();
    let image_rect = active.rect;
    let extent = viewer::page_extent_pts(&doc.pages[acting]);

    // ★ **Publish what the renderer should work on**, nearest the viewport
    // centre first — the order `render::settle` fills the strip in, and the
    // whole of why a scroll feels like it is keeping up rather than starting
    // from the top every time. Only knowable here, once the scroll area has
    // settled, which is the same reason `last_scroll_offset` is stored.
    let centres: Vec<(usize, f32)> = drawn.iter().map(|d| (d.page, d.rect.center().y)).collect();
    doc.strip_visible = strip::nearest_first(&centres, scroll_output.inner_rect.center().y);

    // The frame's screen⟷page map for the page being acted on. Built here,
    // immediately after the scroll area has settled and the page's true drawn
    // rect is known, and handed to everything below — nothing past this line
    // divides by the zoom for itself. See `mapping`'s header for why that
    // matters twice over.
    let map = PageMapping::new(image_rect, extent, doc.view.zoom);
    // …and one map per drawn page, for the Find wash, which has to land on
    // whichever page its hits are on rather than on the one being acted upon.
    // See `interact`'s step 8.
    let page_views: Vec<strip::PageView> = drawn
        .iter()
        .map(|d| strip::PageView {
            page: d.page,
            map: PageMapping::new(
                d.rect,
                viewer::page_extent_pts(&doc.pages[d.page]),
                doc.view.zoom,
            ),
        })
        .collect();

    // ★ The guides' catch bands, registered AFTER every page widget and before
    // the gesture layer runs. The order is the whole mechanism: a later widget
    // in the same layer is the topmost one under the pointer, so a press on a
    // guide never reaches the page's `Response` and therefore never reaches
    // the gesture machine — a guide drag cannot also rubber-band a selection.
    // See `guides`' header §3. Registers nothing when the toggle is off or the
    // document has no guides.
    guides::canvas_drag(ui, doc, &page_views, actions);

    // ★ Form filling on the page, registered in that same layer and for the
    // same reason: the focused field's editor must be topmost, and nothing is
    // registered for an unfocused one. See `forms`' header §4.
    forms::overlay(
        ui,
        doc,
        &page_views,
        &drawn,
        active_tool,
        caps.edit_content,
        actions,
    );

    // ★★ **The pointing hand over a followable link**, registered after the
    // form widgets' own cursor pass and before the gesture layer runs — so
    // `canvas::tool::cursor_for` still has the last word, which is right: an
    // opinion it holds is about a gesture already under way and outranks a
    // hover. See `canvas::links`' header on why this is the WHOLE of the
    // pre-click affordance and why nothing is drawn into the page.
    //
    // ★ After forms rather than before, deliberately. A `/Widget` and a `/Link`
    // overlapping is a form control inside a table of contents; the control is
    // the more specific thing and its own cursor should survive.
    links::cursor(ui.ctx(), doc, &page_views, caps.edit_content);

    // ★ The frame's geometry, recorded for the commands that arrive with none.
    // A zoom raised from a keyboard chord, the ribbon or the status bar has no
    // `Ui` and no page rect, and it must describe its anchor against the view
    // as it stands BEFORE the zoom is applied — which is exactly this. See
    // [`zoom::CanvasFrame`].
    //
    // ★ The offset recorded is the **page-local** one, not the strip's. Every
    // consumer of this record — the anchor rule, both framing verbs — is
    // written for a scroll area holding one page at the origin, and converting
    // here is what lets all of them keep working unchanged over a strip. Under
    // `Single` the conversion is the identity. See `geometry`'s header.
    //
    // ★★★ MEASURED FROM THE DRAWN RECT, NOT RECONSTRUCTED FROM THE SCROLL
    // OFFSET — `OPERATOR_REQUESTS.md` O26e.
    //
    // This used to call `geometry::page_local_offset(scroll_offset, …)`, and
    // on the shallow tier the two are algebraically the same number — a unit
    // test asserts exactly that, so this is not a behaviour change below the
    // threshold. Above it they are not the same at all: the deep branch
    // **forces the scroll offset to zero** (the content is the viewport there,
    // so zero is the only valid offset) and holds the position in `f64`
    // instead. Reconstructing from that forced zero recorded "the page is
    // centred in the pasteboard" on every deep frame.
    //
    // Nothing consumed the lie while the tier held. The first zoom that
    // crossed back **did**: `offset_before` is this field, so
    // `zoom_anchor_offset` solved the descent against a position the operator
    // had never been in and put the page's own origin under the pointer.
    // Driven, 2026-08-24, descending through 1,185,799 %: the page point under
    // the viewport centre went from (791.93, 1152.34) to (−0.02, −0.03).
    //
    // `image_rect` and `inner_rect` are this frame's real screen rects, at
    // every tier, produced by whichever branch placed the strip. An offset
    // derived from them cannot disagree with the pixels, and — the point —
    // needs no knowledge of which mechanism owns the position.
    zoom::remember_frame(
        ui.ctx(),
        zoom::CanvasFrame {
            map,
            extent,
            display: (image_rect.width(), image_rect.height()),
            viewport: (viewport_size.x, viewport_size.y),
            // ★ The OUTER size, measured before the scroll area — the one
            // `canvas::fit` places against on the next frame. See
            // `CanvasFrame::outer` for why measuring "before" against the inner
            // size and placing "after" against the outer lands the centre half
            // a scroll bar off. O78.
            outer: (vp.x, vp.y),
            viewport_rect: scroll_output.inner_rect,
            // ★ The ACTING page, not `view.page_index`. They are the same on
            // almost every frame and differ on exactly the frames that broke:
            // see `acting`'s own note, and `ZoomAnchor::page`.
            page: acting,
            offset: geometry::offset_from_drawn(
                (image_rect.min.x, image_rect.min.y),
                (
                    scroll_output.inner_rect.min.x,
                    scroll_output.inner_rect.min.y,
                ),
                (image_rect.width(), image_rect.height()),
                (viewport_size.x, viewport_size.y),
            ),
        },
    );

    // ★★★ **Which surface this frame's gesture belongs to — O23's off-page
    // half.** The rule, and the argument for each of its four rows, is in
    // [`pasteboard`]; it is a pure function here so that the rows a running
    // window is awkward to put into a given state can still be held by a unit
    // test. The two `Response`s differ in exactly one way that matters: the
    // page's covers its sheet, the content's covers the sheet AND the gaps AND
    // the slack `geometry::content_extent` adds on every side, which is where
    // an object dragged past the page edge actually is.
    let page_gesture = image_response.dragged()
        || image_response.drag_stopped()
        || image_response.clicked()
        || image_response.secondary_clicked();
    let paste_gesture = content_response.dragged()
        || content_response.drag_stopped()
        || content_response.clicked()
        || content_response.secondary_clicked();
    let on_page = image_response.contains_pointer();
    // ★★★ The page's own context menu, asked for by id rather than inferred.
    //
    // `contains_pointer` is layer-aware, so the open menu COVERS the page and
    // `on_page` goes false the moment the cursor reaches the menu. That flips
    // `acting_response`, which changes the popup's id, which makes egui drop
    // the popup as abandoned — silently, with no close call and no trace.
    // See `pasteboard`'s module header: this is the defect `bfc8dea`
    // introduced on 2026-09-10 and the first full sweep found on 2026-09-12.
    let page_owns_open_popup =
        egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&image_response));
    let surface = pasteboard::surface(
        on_page,
        page_gesture,
        paste_gesture,
        content_hovered,
        page_owns_open_popup,
    );
    trace::surface(
        surface,
        on_page,
        page_gesture,
        paste_gesture,
        page_owns_open_popup,
    );
    let acting_response = match surface {
        pasteboard::Surface::Page => &image_response,
        pasteboard::Surface::Pasteboard => &content_response,
    };

    // Selection, before the layout trace: the trace reports `sel=`, and a
    // count taken before the frame's click was applied would describe the
    // previous frame.
    let (selected, tokens) = interact(
        ui,
        doc,
        acting_response,
        &Frame {
            pen,
            map,
            pages: &page_views,
            clip: scroll_output.inner_rect,
            tool: active_tool,
            caps,
            pick,
            max_zoom_percent,
        },
        host,
        find,
        actions,
    );

    trace::layout(
        doc,
        image_rect,
        scroll_offset,
        selected,
        drawn.len(),
        drawn.iter().filter(|d| d.has_raster).count(),
    );
    // ★★ The pan position, in `f64`, from whichever tier owns it.
    //
    // `OPERATOR_REQUESTS.md` O24: the operator reported that a small pan at a
    // high zoom either did nothing or snapped back. Neither symptom can be
    // measured from the line above, because `rect=` is an `f32` whose spacing
    // at that depth is larger than the pan — so this is emitted beside it and
    // says the same thing in double precision. See `trace::position`.
    let pan_at = if deep {
        let anchor = doc
            .deep_anchor
            .unwrap_or_else(viewer::deep::DeepAnchor::origin);
        let z = f64::from(doc.view.zoom);
        (
            anchor.page.0 * z - f64::from(anchor.screen.0),
            anchor.page.1 * z - f64::from(anchor.screen.1),
        )
    } else {
        (
            f64::from(scroll_output.inner_rect.min.x - image_rect.min.x),
            f64::from(scroll_output.inner_rect.min.y - image_rect.min.y),
        )
    };
    // Where the acting page's raster was actually painted. Equal to
    // `image_rect` below the pixmap ceiling; the region's rect above it, and
    // the only field that can witness O24c's lurch.
    let painted = drawn
        .iter()
        .find(|d| d.page == acting)
        .map_or(image_rect, |d| d.paint_rect);
    // ★ The region of the raster that was actually PAINTED, taken from the
    // held texture's own key — the same source the placement used, so the
    // harness's independent recomputation is checking the placement rather
    // than agreeing with itself. `None` for a whole-page raster.
    let painted_region = if acting == current {
        doc.page_texture
            .as_ref()
            .filter(|t| t.key.page() == acting)
            .and_then(|t| t.key.region())
    } else {
        doc.strip_page_texture(acting, doc.render_key_for(acting, raster_scale))
            .and_then(|t| t.key.region())
    };
    trace::position(
        pan_at,
        if deep { "deep" } else { "scroll" },
        (painted.min.x, painted.min.y),
        painted_region,
        // …and the region the shell WANTS, which moves the instant the view
        // does. The gap between the two is O25.
        doc.region_for(acting),
        extent,
    );
    crate::diag::ui_rect(trace::REGION_PAGE, image_rect);
    crate::diag::ui_rect(trace::REGION_CANVAS_VIEWPORT, scroll_output.inner_rect);
    trace::pointer(ui, doc, image_rect, extent);

    // ★ The plain wheel as a page turn — O30. Before the Ctrl+wheel block
    // rather than after it, so the two are read in the order egui produced
    // them; they cannot both fire on one gesture, because a modified wheel
    // event populates `zoom_delta` and contributes nothing to the scroll
    // delta this reads.
    paging::flip(
        ui,
        doc,
        content_hovered || image_response.hovered(),
        actions,
    );

    // Ctrl+wheel over the canvas: multiply the zoom. Gated on hover so a
    // Ctrl+wheel aimed at some other surface does not zoom the page out from
    // under the operator.
    //
    // ★★★ THE GATE IS THE CANVAS, NOT THE CURRENT PAGE — `OPERATOR_REQUESTS.md`
    // O26.
    //
    // It used to read `image_response.hovered()`, which is the response of the
    // **acting page only**. Three ordinary positions were therefore inert:
    //
    // * the pointer over a *different* visible page, which under a continuous
    //   mode is most of the screen whenever more than one page fits;
    // * the pointer in the gap between two pages;
    // * the pointer over O23's **pasteboard** — a whole viewport of it on
    //   every side, added deliberately so any page corner can be brought to
    //   any point of the screen, and therefore a position the operator is now
    //   *expected* to be in.
    //
    // ★★ It is also what turned O26's page catapult from a lurch into a
    // freeze. Once the current-page tracker had thrown `page_index` seven
    // pages down the strip, the acting page was off screen, nothing under the
    // pointer was it, and **every subsequent Ctrl+wheel did nothing at all**:
    // driven, 2026-08-24, five further notches produced a byte-identical
    // trace. A view that jumps is a bug; a view that jumps and then will not
    // zoom back is the operator's report.
    //
    // `content_response` is the scroll area's whole content — pages, gaps and
    // pasteboard — so this asks *"is the pointer over the canvas?"*, which is
    // the question the comment above always claimed it was asking. It is a
    // real egui `Response`, so it still respects layer order and a floating
    // window over the canvas still swallows the wheel; a `rect.contains`
    // test would not have.
    if content_hovered || image_response.hovered() {
        let factor = ui.ctx().input(|i| i.zoom_delta());
        if (factor - 1.0).abs() > f32::EPSILON {
            // Zoom to cursor, half one: remember WHERE on the page the
            // pointer is before the zoom lands. Anchoring on the viewport
            // centre instead (which is what happens when nothing records
            // this) drags the detail being inspected out from under the
            // operator, worse the further off-centre they point — reported
            // as "jarring" on 2026-08-04.
            //
            // ★ Through [`zoom::arm_anchor`], the same call the discrete
            // commands make — which is what "the rule is decided once for all
            // four" means in code. The wheel used to build its own `ZoomAnchor`
            // inline from the pointer position; that inline version WAS the
            // rule, in a place no command could reach, and duplicating it at
            // three more call sites is how the four would have drifted apart.
            //
            // The pointer guard that used to live here has moved with it: a
            // pointer off-window (a trackpad pinch can produce exactly that)
            // falls back to the viewport centre rather than to nothing, and a
            // zero drawn size can no longer produce a NaN because
            // `zoom::frac_of` divides by the page EXTENT, which is finite and
            // positive for any page that drew at all.
            zoom::arm_anchor(ui.ctx(), doc);
            actions.push(Action::ZoomBy(factor));
        }
    }

    // ★ What the frame learned, handed outwards so the rulers can be drawn
    // against it. Only knowable here — after the scroll area has settled — for
    // the same reason `last_scroll_offset` is stored and `strip_visible` is
    // published during layout. See [`CanvasGeometry`].
    (
        tokens,
        Some(CanvasGeometry {
            pages: page_views,
            current: acting,
            viewport: scroll_output.inner_rect,
        }),
    )
}

/// The canvas's scroll-bar style: **solid, visible, and outside the page**.
///
/// # ★★★ The defect this closes, measured on the real binary
///
/// egui's default `ScrollStyle` is [`egui::style::ScrollStyle::floating`]:
/// the bars allocate no space, are drawn **over** the content, and are fully
/// transparent until hovered (`dormant_handle_opacity: 0.0`). Their press
/// target is nonetheless real on every frame the bar is shown — egui 0.35's
/// `scroll_area.rs:1316` interacts with `outer_rect.with_min_x(max_cross -
/// bar_width)`, the rightmost **10 logical points** of the viewport, with
/// `Sense::CLICK | Sense::DRAG` — and a press anywhere on it centres the handle
/// on the pointer (`scroll_area.rs:1405`), i.e. **jumps the scroll**.
///
/// At a fit zoom the page's right edge sits a few points inside the viewport's
/// right edge — measured: page `max.x = 726.0` against viewport `max.x = 732.0`
/// — which is **inside that invisible band**. So the south-east grip of any
/// object that reaches the page's edge (the sheet border, a full-bleed image,
/// a title block) was drawn on top of a control the operator could not see and
/// that took the press first. The trace read, on the press frame:
///
/// ```text
/// canvas-press   grip=SE … drag=Resize            ← the press was understood
/// canvas-gesture started=0 dragging=0 … origin=1  ← and never delivered
/// canvas rect=… off=[442.3 1045.9]                ← the view jumped 453 pt
/// canvas-unavailable reason=nothing-visible       ← and the page left the screen
/// ```
///
/// No resize event, no decline, and the drag became a scroll — which is what
/// `resize_scales_a_shape` had been reporting as *"committed nothing and
/// declined nothing"*, and what five successive explanations reached by
/// reading source all failed to find. **A control drawn under an invisible
/// control is unreachable in silence** — the same finding as the pop-up that
/// slid back over its own anchor (`feedback_a_window_over_the_thing_it_describes…`).
///
/// # Why solid rather than a reserved margin under floating bars
///
/// A margin would fix the *fit* case and leave every other zoom able to put
/// the page edge under the band after a pan. Solid bars take their own strip
/// **outside** `inner_rect` (`bar_inner_margin + bar_width` = 14 pt, see
/// [`egui::style::ScrollStyle::allocated_width`]), so the page can never be
/// under them at any zoom — and the geometry already expects this: O78 gave
/// [`crate::canvas::zoom::CanvasFrame`] separate `viewport` (inner) and `outer`
/// sizes precisely so a bar taking real width would not land the fit centre
/// half a bar off. This is also the convention of the product class: Acrobat,
/// Word and every Windows document viewer draw their bars beside the page,
/// never over it, and never transparent.
///
/// # The two halves the dock already learned
///
/// Mirrors `egui-shell`'s panel-body style, and for the two reasons its RAG
/// entry records (`D:/dev/rag/egui/scrollstyle_solid_draws_the_handle_in_bg_fill…`):
/// `solid()` alone draws the handle in `widgets.inactive.bg_fill`, which on a
/// light theme is near-white on near-white, so `foreground_color` makes it
/// inherit the theme's text contrast — no raw colour, so the theme gate stays
/// green — and `bar_width: 10.0` matches the dock so the two look like one
/// application.
#[must_use]
fn scroll_style() -> egui::style::ScrollStyle {
    let mut scroll = egui::style::ScrollStyle::solid();
    scroll.foreground_color = true;
    scroll.bar_width = 10.0;
    scroll
}
