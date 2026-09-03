//! # `canvas::pagedrop` — dropping pages onto the page view
//!
//! The second half of the operator's request of 2026-08-19:
//!
//! > *"…drag and drop pages from one thumbnail image sidebar to another **or
//! > onto the canvas** to add pages and insert them **in between the pages
//! > we've dragged to** on the canvas or the thumbnail preview area."*
//!
//! The Pages panel's grid already knew how to resolve a gap and draw a caret;
//! this is the same gesture with the same vocabulary, aimed at the page view.
//! [`crate::pagedrag`] holds the drag itself, which is what lets a gesture that
//! *began* in a panel — possibly in another document — end here.
//!
//! ---
//!
//! ## 1. Where a gap is, on a page view
//!
//! The strip lays pages out in **rows**, top to bottom, in every display mode
//! ([`crate::viewer::strip`]). So a boundary between two pages is a horizontal
//! line, and the question *"which boundary is the operator aiming at?"* is
//! answered by the **vertical half** of the page under the pointer: the top
//! half means *before this page*, the bottom half means *after it*.
//!
//! That is deliberately the same shape as the Pages panel's rule, rotated
//! ninety degrees to match the flow of the surface it is on — the grid flows
//! left to right within a row, so it splits on the horizontal half and draws a
//! vertical caret; the page view flows top to bottom, so it splits on the
//! vertical half and draws a horizontal one. An operator who has used one
//! knows the other without being told, which is the property worth having.
//!
//! ### ★ The facing modes get the same rule, and it is right there too
//!
//! Under `PageDisplay::Facing` a row holds two pages side by side, so the
//! left/right split would also carry meaning. It is **not** used, and the
//! reason is not laziness: *before the right-hand page of a spread* and *after
//! the left-hand page of a spread* are the same boundary, so a horizontal
//! split would offer the operator two ways to name one gap and no way to name
//! the gap between rows. The vertical rule names every boundary exactly once.
//!
//! ## 2. Single-page mode is not a special case
//!
//! It is a one-row strip ([`crate::viewer::strip`]'s header says why that is
//! load-bearing), so the same rule applies unchanged: the top half of the sheet
//! on screen means *before this sheet*, the bottom half means *after it*. An
//! operator who never turns continuous scroll on can still drag a sheet out of
//! another drawing and drop it in front of the one they are reading.
//!
//! ## 3. Rule 4 — the caret is the cursor, and nothing here marks content
//!
//! `panels::pages::paint_caret`'s argument, verbatim and for the same surface:
//! *"snap indicators, hover highlights, rubber-bands and selection handles are
//! the cursor and are welcome"*. This draws one line, over the gap, while a
//! button is held, and it is gone the instant it is released. It tints no page,
//! badges nothing, and adds no second rendering path — the one-line test is
//! that a screenshot of this canvas with a drag in flight differs from one of
//! the same document saved and reopened only by where the pointer is.
//!
//! The words half of the disclosure lives off-canvas, in the status row and in
//! the Pages panel's header, exactly as rule 4 requires.
//!
//! ## 4. What it does NOT do
//!
//! **It does not accept files dropped from Explorer.** That is
//! `crate::app::dropped`, which reads `egui`'s window-level `dropped_files` and
//! is a different mechanism with a different operand. Both can be in flight at
//! once and they do not interact; this one is only ever about pages already
//! open in pdfcer.

use eframe::egui::{self, Pos2, Rect};

use crate::app::actions::{Action, pages::PageAction};
use crate::app::state::OpenDoc;
use crate::canvas::strip::DrawnPage;
use crate::panels::pages::ops;

/// How thick the insertion caret is drawn.
///
/// `panels::pages`' `CARET_PTS`, restated rather than imported, because the two
/// are the same *number* and not the same *decision*: this one is measured
/// against a rendered page at the operator's zoom and that one against a
/// thumbnail tile. If a future zoom-aware caret makes one of them change, the
/// other must not follow by accident.
const CARET_PTS: f32 = 2.0;

/// How far outside the page edge the caret sits, in points.
///
/// Enough to read as *"in the gap"* rather than as a border the sheet has
/// grown, which is the same reason the grid's caret sits half the inter-tile
/// spacing beyond the tile.
const CARET_INSET_PTS: f32 = 4.0;

/// How much of the caret's colour survives when the drop would change nothing.
///
/// `panels::pages::CARET_DIMMED`, and the argument travels with it: **dimmed,
/// not hidden**, because drawing nothing over a boundary that would not land
/// cannot be told apart from the canvas having stopped tracking the pointer —
/// and the no-op boundary is where every same-document drag begins.
const CARET_DIMMED: f32 = 0.35;

/// Named region: the caret, when one is drawn.
const REGION_CARET: &str = "canvas-drop-caret"; // ui-text-exempt: trace region name, never displayed

/// Where a page drop on the canvas would land.
#[derive(Debug, Clone, Copy, PartialEq)]
struct CanvasDrop {
    /// The gap index, in [`ops`]' sense: `0` before the first page,
    /// `page_count` after the last.
    gap: usize,
    /// The line to draw, in screen space. Horizontal — see §1.
    caret: Rect,
    /// Whether releasing here would change anything.
    lands: bool,
}

/// **Offer the page view as a drop target for a page drag, and settle a
/// release on it.**
///
/// Called once per frame from [`crate::canvas::show_in`], after the scroll area
/// has closed and therefore after every visible page's screen rectangle is
/// known. Does nothing at all — not one branch past the first — when no page
/// drag is in flight, which is every frame but the handful the operator is
/// carrying something.
///
/// `drawn` is the visible pages with the rectangles they were actually drawn
/// in, which is what makes the gap resolution exact rather than reconstructed.
/// `D:\dev\rag\egui` records the rule this obeys: **do not compute a coordinate
/// the application could publish** — a harness, or a second piece of the
/// application, that derives a widget position by arithmetic can be wrong in
/// the same direction as the code under test.
pub(super) fn offer(
    ui: &egui::Ui,
    doc: &OpenDoc,
    drawn: &[DrawnPage],
    viewport: Rect,
    actions: &mut Vec<Action>,
) {
    let Some(drag) = crate::pagedrag::current(ui.ctx()) else {
        return;
    };
    // Which document this canvas is showing. `unwrap_or_default` is slot 0
    // with no label and is only reachable from a unit test that draws a canvas
    // without an application around it — the frame publishes this before any
    // surface draws.
    let here = crate::pagedrag::active(ui.ctx()).unwrap_or_default();
    let page_count = doc.pages.len();

    let target = resolve(ui, &drag, &here, drawn, viewport, page_count);

    // The cursor says the canvas is carrying something, on every frame the
    // pointer is over it — `egui` resolves the cursor per frame from whatever
    // asked most recently, so a single request at the start of the gesture
    // would be overwritten by the next widget the pointer crossed.
    if viewport.contains(ui.ctx().pointer_latest_pos().unwrap_or(Pos2::ZERO)) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    }

    if let Some(target) = &target {
        crate::pagedrag::set_landing(
            ui.ctx(),
            crate::pagedrag::DropLanding {
                target_slot: here.slot,
                gap: target.gap,
                page_count,
                lands: target.lands,
            },
        );
        paint(ui, target);
    }

    // ★ The release is read from RAW POINTER INPUT, not from a page's own
    // `Response`.
    //
    // `panels::pages::settle_drag`'s discipline and its reason, unchanged: the
    // press that started this drag happened in another widget — usually in
    // another *panel*, sometimes for another *document* — so no response on
    // this canvas has ever seen it, and none of them will report its release.
    if !ui
        .ctx()
        .input(|i| i.pointer.button_released(egui::PointerButton::Primary))
    {
        return;
    }
    // Only a release the canvas can legitimately claim. A release anywhere
    // else is the panel's to settle, or nobody's — and taking it here would
    // end a drag the operator was still making somewhere the canvas cannot
    // see.
    let Some(target) = target else {
        return;
    };
    crate::pagedrag::end(ui.ctx());

    if drag.source_slot == here.slot {
        // Same document: a reorder, exactly as a drop on the Pages grid is.
        match ops::drop_order(&drag.pages, page_count, target.gap) {
            Ok(order) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "canvas-drop gap={} moving={} reordered=1",
                        target.gap,
                        drag.pages.len()
                    )
                });
                actions.push(Action::Page(PageAction::ReorderPages { order }));
            }
            Err(refusal) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "canvas-drop gap={} moving={} reordered=0 refusal={refusal:?}",
                        target.gap,
                        drag.pages.len()
                    )
                });
            }
        }
        return;
    }

    // Sampled at the release, for `panels::pages::settle_drag`'s reason.
    let take = crate::pagedrag::wants_move(ui.ctx());
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "canvas-drop from-slot={} gap={} moving={} copied=1 take={}",
            drag.source_slot,
            target.gap,
            drag.pages.len(),
            u8::from(take),
        )
    });
    actions.push(Action::InsertPagesFromOpenDocument {
        source_slot: drag.source_slot,
        pages: drag.pages,
        position: crate::pagedrag::insert_position(target.gap, page_count),
        take,
    });
}

/// Which boundary the pointer is aiming at, and where to draw it.
///
/// `None` when the pointer is outside the page view, or over no page — the
/// margin around a page that is smaller than the viewport is not a gap, and
/// treating it as one would let a drop land somewhere the operator was not
/// pointing.
fn resolve(
    ui: &egui::Ui,
    drag: &crate::pagedrag::PageDrag,
    here: &crate::pagedrag::ActiveDocument,
    drawn: &[DrawnPage],
    viewport: Rect,
    page_count: usize,
) -> Option<CanvasDrop> {
    if page_count == 0 {
        // A document with no pages has no gaps to aim at, and `drawn` is empty
        // anyway — but asking first is what stops the whole question being
        // decided by an empty iterator, which would be true by accident rather
        // than by rule.
        return None;
    }
    let pointer = ui.ctx().pointer_latest_pos()?;
    if !viewport.contains(pointer) {
        return None;
    }
    let page = drawn.iter().find(|d| d.rect.contains(pointer))?;

    // §1 — the vertical half decides, because the strip flows downward.
    let after = pointer.y > page.rect.center().y;
    let gap = if after { page.page + 1 } else { page.page };
    let y = if after {
        page.rect.bottom() + CARET_INSET_PTS
    } else {
        page.rect.top() - CARET_INSET_PTS
    };

    Some(CanvasDrop {
        gap,
        caret: Rect::from_min_max(
            Pos2::new(page.rect.left(), y),
            Pos2::new(page.rect.right(), y),
        ),
        // The two questions the Pages grid asks, for the same two reasons: a
        // same-document drag is a reorder and can be a no-op, a cross-document
        // drag is a copy and every gap is a legal place to put a sheet that is
        // not there yet.
        lands: drag.source_slot != here.slot
            || !ops::drag_is_a_no_op(&drag.pages.iter().copied().collect(), gap),
    })
}

/// Draw the caret. §3 — this is the cursor.
fn paint(ui: &egui::Ui, target: &CanvasDrop) {
    // The theme's accent, never a literal, and the same source the Pages
    // grid's caret, the current-page ring and the guide preview all take — so
    // a preset that changes the accent changes every one of them together.
    let base = ui.visuals().selection.stroke.color;
    let colour = if target.lands {
        base
    } else {
        base.gamma_multiply(CARET_DIMMED)
    };
    ui.painter().line_segment(
        [target.caret.left_top(), target.caret.right_top()],
        egui::Stroke::new(CARET_PTS, colour),
    );
    crate::diag::ui_rect_visible(REGION_CARET, target.caret.expand(CARET_PTS), ui.clip_rect());
}
