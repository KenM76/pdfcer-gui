//! # `canvas::offset` — who decides where the view is, this frame
//!
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
//! | 6 | **a point destination's scroll** | the same argument as the reveal, and one more: it must outrank the page-change scroll below, which would otherwise satisfy the destination's page turn without visiting the point. O200 |
//! | 7 | **a Tab ring's minimal reveal** | below the destination because a link the operator clicked outranks a focus move they tabbed to, and above the page-change scroll for the destination's own reason: a cross-page Tab raises a page command, and letting that park the view at the top of the sheet would leave the field unvisited. O204 |
//! | 8 | **a page command's scroll** | under a continuous mode a page command has to scroll the strip to the page it named, and that is a one-shot the operator asked for |
//! | 9 | **a middle-drag pan** | a live gesture — and it is LAST for the reason it wins anyway: it re-arms itself on the next frame, while every one-shot above it is spent once |
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
    /// Where a fit command, or a page-display recentre, asked the view to go —
    /// and **which layout unit that offset is measured against**. See
    /// [`crate::canvas::fit::Placed`]; O177.
    pub fit_placement: Option<crate::canvas::fit::Placed>,
    /// The page a pending zoom anchor was armed against, and that page's drawn
    /// size — see `viewer::ZoomAnchor::page` for why the anchor names a page.
    pub anchor_page: usize,
    /// See [`Self::anchor_page`].
    pub anchor_display: (f32, f32),
    /// The acting page.
    pub current: usize,
    /// The acting page's drawn size.
    pub current_display: (f32, f32),
    /// The rect of the **row** holding the acting page — the page's own rect
    /// outside a facing mode, the spread's union inside one. The frame of
    /// reference a [`crate::canvas::fit::Placed::Row`] offset is converted
    /// through. O177.
    pub row_rect: egui::Rect,
    /// The whole strip's drawn size.
    pub display_size: Vec2,
    /// The viewport, measured before the scroll area was built.
    pub vp: Vec2,
}

/// **The canvas frame the open-seed arm places a freshly opened view on.**
///
/// `OpenDoc::canvas_frames` counts the canvas frames this document has had and
/// is incremented at the end of [`super::viewpos::position`], *after* this
/// chain has run — so during the chain it reads as the zero-based index of the
/// frame being decided. The seed fires on index `1`, the **second** frame; see
/// the open-seed arm for the four bisecting runs that argued against index `0`.
///
pub(super) const SEED_FRAME: u8 = 1;

/// **Which arm of the ranked chain won this frame, and what it produced.**
///
/// # Why the winner is returned and not merely the number
///
///
/// ★★★ That cost a full session of diagnosis. A regression placed a freshly
/// opened multi-page document off the bottom-right corner, the published
/// offset was `[0.0 0.0]`, and **three different arms of this chain can
/// produce exactly `(0.0, 0.0)`** — the deep-tier arm returns it as a literal,
/// [`geometry::strip_offset`]'s lower clamp produces it from any sufficiently
/// negative page-local solve, and a strip-space page scroll to the very top of
/// the content produces it honestly. With only the number in hand, those are
/// indistinguishable, and so is a fourth case: no arm firing at all. Naming
/// the winner collapses four hypotheses into one measurement.
///
/// The name is a short stable token, never a sentence, because its readers are
/// a `grep` in a trace file and a `ui-verify` assertion rather than a person
/// reading prose.
pub(super) struct Decision {
    /// The offset the winning arm produced, or `None` when no arm claimed the
    /// frame and egui's own scrolling is left alone.
    ///
    /// **`None` is the ordinary case.** A canvas that forced an offset every
    /// frame would be a canvas the wheel could not move; see the module
    /// header.
    pub offset: Option<Vec2>,
    /// The winning arm's name, or [`Decision::NONE`]'s `"none"`.
    ///
    /// One token per arm, in the same order the chain tests them: `deep`,
    /// `handover`, `fit`, `zoom-anchor`, `reveal`, `dest-scroll`, `min-reveal`,
    /// `page-scroll`, `pan`, `open-seed`. Diagnostic only — it reaches the
    /// operator through nothing but a `PDFCER_DIAG` line.
    pub source: &'static str,
}

impl Decision {
    /// The frame nobody claimed.
    ///
    /// A named constant rather than a literal at the one site that returns it,
    /// so that the `"none"` token has exactly one definition and a check
    /// grepping for it cannot be broken by a reworded arm.
    // ui-text-exempt: diagnostic token, never displayed in the UI
    pub(super) const NONE: Self = Self {
        offset: None,
        source: "none",
    };

    /// An arm claimed the frame with `offset`.
    ///
    /// Every `return` in [`decide`] goes through this, which is what keeps the
    /// name and the number impossible to separate: there is no way to add an
    /// arm that produces an offset without also naming it.
    #[must_use]
    fn won(source: &'static str, offset: Vec2) -> Self {
        Self {
            offset: Some(offset),
            source,
        }
    }
}

/// **The offset this frame's `ScrollArea` should be forced to**, or
/// [`Decision::NONE`] to leave it wherever the operator left it.
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
) -> Decision {
    let Frame {
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
    } = frame;
    // The strip conversion every page-local answer below is handed back
    // through. Spelled here rather than passed in as a closure so this module
    // can be read on its own; it is the same one `canvas::show` uses, and
    // `canvas::geometry`'s header carries the argument for why it exists.
    // Read once, before the closures and before `&mut doc` is captured
    // anywhere: the frame's pasteboard overhang. Every conversion below must
    // use the SAME number the scroll content was built from, or an offset
    // solved here lands in a content rectangle that does not exist. See
    // `OpenDoc::pasteboard_overhang`.
    let overhang = (doc.pasteboard_overhang.x, doc.pasteboard_overhang.y);
    // Where the view is sitting right now, in content space. Read here with
    // the overhang, before `doc` is borrowed mutably below, and used by the
    // dest-scroll arm as the offset an axis that must not move is held at.
    let doc_offset = doc.frame.last_scroll_offset;
    // ★ Takes the RECT rather than a page index, as of O177. Every offset
    // solved above arrives measured against *something* — a page for the zoom
    // anchor and the reveal, a whole facing row for a fit and for the
    // page-display recentre — and the conversion is the same arithmetic either
    // way. Naming the rect rather than a page is what lets the row-based
    // callers exist at all without a second copy of it.
    let strip_offset_in = |rect: egui::Rect, local: (f32, f32)| {
        let (x, y) = geometry::strip_offset(
            local,
            (rect.min.x, rect.min.y),
            (display_size.x, display_size.y),
            (rect.width(), rect.height()),
            (vp.x, vp.y),
            overhang,
        );
        vec2(x, y)
    };
    // The page-indexed form the anchor and the reveal want, with the fallback
    // that has always been here for a page this strip does not lay out.
    let strip_offset_for = |page: usize, local: (f32, f32)| {
        let rect = layout
            .rect_of(page)
            .unwrap_or_else(|| egui::Rect::from_min_size(egui::Pos2::ZERO, display_size));
        strip_offset_in(rect, local)
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
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won("deep", vec2(0.0, 0.0));
    } else if let Some(offset) = deep_handover {
        // ★ FIRST, above the ordinary anchor: this frame is the one that left
        // the `f64` tier, and the offset solved above is the position the
        // anchor was actually holding. See the branch that produced it.
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won("handover", to_strip((offset.x, offset.y)));
    } else if let Some(placed) = fit_placement {
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
        // ★ Converted through whichever rect the offset was solved against —
        // O177. A `Row` offset put through the page's rect is exactly the
        // defect this arm used to have: the spread was scaled to fit two pages
        // and then placed as though it were one, so half of it sat off the
        // canvas. See [`crate::canvas::fit::Placed`].
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won(
            "fit",
            match placed {
                crate::canvas::fit::Placed::Page(offset) => {
                    strip_offset_for(current, (offset.x, offset.y))
                }
                crate::canvas::fit::Placed::Row(offset) => {
                    strip_offset_in(row_rect, (offset.x, offset.y))
                }
            },
        );
    } else if let Some(offset) = zoom::consume_anchor(ui.ctx(), doc, anchor_display) {
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won(
            "zoom-anchor",
            strip_offset_for(anchor_page, (offset.x, offset.y)),
        );
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
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won("reveal", to_strip((offset.x, offset.y)));
    } else if let Some(offset) = crate::canvas::destscroll::take_dest_scroll_offset(
        doc,
        current_display,
        (vp.x, vp.y),
        doc_offset,
        &to_strip,
    ) {
        // A bookmark or link whose destination named a POINT rather than a
        // rectangle — `OPERATOR_REQUESTS.md` O200. Ranked here for the reveal's
        // reason and one more of its own:
        //
        // * BELOW the fit and the zoom anchor, because a `/FitH` or a `/XYZ`
        //   that carried an explicit magnification raises one of those itself,
        //   one frame earlier, and this scroll is meant to be solved against
        //   the size that produced.
        // * ABOVE the page-change scroll, which would otherwise satisfy the
        //   destination's page turn by parking the view at the top of the sheet
        //   and leave the point unvisited.
        //
        // Already converted to strip space: `destscroll` is handed `to_strip`
        // rather than reimplementing it, because the visibility test the
        // request turns on has to be made in the same space the answer is.
        //
        // The side effect runs before the return, as the reveal's does: a
        // destination has navigated, so the page it landed on is the tracked
        // one.
        doc.tracked_page = doc.view.page_index;
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won("dest-scroll", offset);
    } else if let Some(offset) = crate::canvas::minreveal::take_reveal_offset(
        doc,
        current_display,
        (vp.x, vp.y),
        doc_offset,
        &to_strip,
    ) {
        // A Tab press whose next stop was below the fold —
        // `OPERATOR_REQUESTS.md` O204. Ranked here for the two reasons the
        // module header's row states, and it is the only arm in this chain
        // that can decline to move at all: `minreveal` returns `None` when
        // both axes already show the rectangle, which is the common case and
        // is what keeps a Tab from nudging a view the operator had settled.
        //
        // Already converted to strip space, as the destination's is, and for
        // the same reason: the visibility test has to be made in the space
        // the answer is in.
        //
        // The side effect runs before the return, as the two arms above do:
        // a reveal that crossed pages has navigated, so the page it landed
        // on is the tracked one.
        doc.tracked_page = doc.view.page_index;
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won("min-reveal", offset);
    } else if let Some(offset) = crate::canvas::strip::page_scroll_offset(doc, layout, (vp.x, vp.y))
    {
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won("page-scroll", offset);
    } else if let Some(pan) = pan_delta(ui, active_tool) {
        // Panning subtracts the pointer delta: the content follows the hand,
        // so the page moves WITH the pointer rather than under it.
        let (x, y) = geometry::pan_offset(
            (
                doc.frame.last_scroll_offset.x,
                doc.frame.last_scroll_offset.y,
            ),
            (pan.x, pan.y),
            (display_size.x, display_size.y),
            (vp.x, vp.y),
            overhang,
        );
        //
        //
        // > *"if the canvas window is resized the pdf should resize to match
        // > unless the person has changed the zoom **or panned around**."*
        //
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
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won("pan", vec2(x, y));
    } else if doc.canvas_frames == SEED_FRAME {
        // ★★★ SEED ON THE SECOND FRAME, NOT THE FIRST.
        //
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
        // ★ Against the ROW, as of O177. A document whose remembered
        // arrangement is a facing spread opens straight into one, so the seed
        // is the first thing the operator sees and it must obey the same rule
        // the fit does: the thing being centred is the spread.
        // ui-text-exempt: diagnostic token, never displayed in the UI
        return Decision::won(
            "open-seed",
            strip_offset_in(
                row_rect,
                crate::canvas::geometry::offset_holding_anchor_at(
                    (0.5, 0.5),
                    (vp.x / 2.0, vp.y / 2.0),
                    (row_rect.width(), row_rect.height()),
                    (vp.x, vp.y),
                ),
            ),
        );
    }
    Decision::NONE
}
