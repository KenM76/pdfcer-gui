//! **The page-previews row** — the operator's instruction about thumbnails,
//! the time limit that bounds one, and the sentence explaining a tile that
//! has no picture.
//!
//! # Why this is its own file
//!
//! R2, immediately: the O151 rewrite pushed `panels/pages/mod.rs` past 1,500
//! lines. But the seam was already there, which is the part worth stating —
//! everything here reads and writes exactly one object,
//! [`crate::panels::pages::thumbnails::ThumbnailCache`], and nothing else in
//! the pages panel reads what it writes. A subject that touches one type and
//! is touched by nothing is a file.
//!
//! # What it draws, and the one rule that governs all of it
//!
//! ```text
//! [x] Draw page previews   [≤ 2.0 s]
//! Page 3 needed more than 2.0 s to draw and was skipped. Raise the …
//! ```
//!
//! ★★★ **The tick is the operator's and nothing else may write it** —
//! `OPERATOR_REQUESTS.md` O151, 2026-09-08:
//!
//! > *"the drawing page previews checkbox should never automatically turn
//! > off. You can add a box next to the checkbox to enter a timeout value."*
//!
//! Before that, an expensive page cleared the checkbox on the operator's
//! behalf; `thumbnails.rs`'s "the skipping rule" section carries the full
//! argument for why that was wrong and what replaced it. What binds *this*
//! file is the consequence: [`row`] writes
//! [`ThumbnailCache::force_on`](crate::panels::pages::thumbnails::ThumbnailCache::force_on)
//! from a `changed()` checkbox and from nowhere else, and the cache's own
//! source-scan tripwire holds the other half.
//!
//! # Rule 4 — where the disclosure goes
//!
//! Off-canvas, above the grid, and it names **one page**. A skipped page has
//! no picture for a specific reason, and rule 4's *report separately* means
//! that reason must be stated somewhere the operator will meet it before
//! they draw their own conclusion — which, for a grid, is above it rather
//! than under it. It is not drawn on any tile: the tile carries a word
//! ([`crate::text::pages::thumbnail_abandoned`]) and nothing else.

use crate::text::pages as t;

use super::{PagesUi, thumbnails};

/// Trace region for the previews checkbox, so a driven check can assert that
/// it is where the operator can reach it — and, since O151, that **it is
/// still ticked** after a document whose pages exceed the budget.
const PREVIEWS_REGION: &str = "panel-pages-previews"; // ui-text-exempt: trace region name, never displayed

/// Trace region for the per-page time limit beside it.
const BUDGET_REGION: &str = "panel-pages-budget"; // ui-text-exempt: trace region name, never displayed

/// Draw the previews checkbox, the per-page time limit, and the skip note.
///
/// Takes [`PagesUi`] rather than the cache alone so the call site reads the
/// same as every other section of the panel — `previews::row(ui, pages)` — and
/// so a future addition here (a second control, a second disclosure) does not
/// change the signature and therefore the call site.
pub fn row(ui: &mut egui::Ui, pages: &mut PagesUi) {
    // ★★★ THE PREVIEWS ROW — a checkbox and the time limit beside it (O151).
    //
    // Both controls read from the cache and write straight back, so "is the
    // box ticked" and "will anything be drawn" are one expression rather than
    // two that can disagree — see `ThumbnailCache::previews_on`.
    //
    // `horizontal_wrapped`, not `horizontal`, and that is R128 rather than
    // taste. The dock splitter can put this panel at any width the operator
    // likes; a plain `horizontal` would let the `DragValue` run off the right
    // edge and become unreachable, which is the failure the egui RAG records
    // as panels that shipped unreachable in real builds with every gate green.
    // Wrapping puts the box on a second line instead, where it can still be
    // hit.
    ui.horizontal_wrapped(|ui| {
        let mut previews_on = pages.cache.previews_on();
        let checkbox = ui
            .checkbox(&mut previews_on, t::previews_label())
            .on_hover_text(t::previews_tooltip());
        if checkbox.changed() {
            pages.cache.force_on(previews_on);
        }
        let _ = crate::diag::ui_rect_visible(PREVIEWS_REGION, checkbox.rect, ui.clip_rect());

        // ★ The per-page time limit. A `DragValue` rather than a slider, for
        // the same reason `canvas::markup::swatch` gives for the pen width: an
        // operator setting a time limit has a specific number in mind — one
        // second, five — rather than a value they want to explore, and a drag
        // value takes a typed number where a slider cannot.
        //
        // ⚠ It is NOT disabled while previews are off. A greyed control is
        // reserved for *temporarily* unavailable (R9), and this one is
        // perfectly meaningful with the tick clear: the operator's most likely
        // reason for being here at all is that previews were too slow, so the
        // sequence "set a limit, then turn them on" must work. Greying it
        // would make the limit reachable only from the state it is meant to
        // fix.
        //
        // ★★★ THE DRAFT, AND WHY THIS CONTROL IS NOT WRITTEN THE OBVIOUS WAY.
        //
        // The obvious way — seed a local from `cache.budget()` every frame,
        // commit on `changed()` — was the first draft of this file and it had
        // two independent defects, neither of which any unit test in this
        // crate could have seen:
        //
        //   1. **A drag would have committed nothing, ever.** `DragValue`
        //      accumulates the pointer's motion into the borrowed value
        //      *within a frame*; re-seeding from the cache next frame throws
        //      that away. `app::spinnerdraft`'s header carries the whole
        //      argument, and the egui RAG carries the receipt — this project
        //      shipped two undraggable controls in the markup band, over ~3,800
        //      green tests, and found them only by driving the real pointer.
        //   2. **It would have committed once per PIXEL of the drag.**
        //      `set_budget` re-queues every abandoned page, so dragging from
        //      2 s to 8 s would have re-rendered the expensive page dozens of
        //      times on the UI thread — while the operator was still choosing
        //      a number, and each render bounded by the number they had not
        //      finished choosing. The second defect is the more expensive one
        //      and it is invisible until the document is slow, which is the
        //      only kind of document anyone touches this control on.
        //
        // ⇒ Both go away with a draft that outlives the frame but not the
        // interaction, and a commit on `ended` rather than on `changed`.
        let id = ui.id().with("pages-previews-budget.draft");
        let was = pages.cache.budget().as_secs_f32();
        let mut seconds = crate::app::spinnerdraft::drafted(ui, id, was);
        let budget = ui
            .add(
                egui::DragValue::new(&mut seconds)
                    .speed(0.1)
                    .range(
                        thumbnails::MIN_PAGE_BUDGET.as_secs_f32()
                            ..=thumbnails::MAX_PAGE_BUDGET.as_secs_f32(),
                    )
                    .max_decimals(1)
                    .prefix(t::previews_budget_prefix())
                    .suffix(t::previews_budget_suffix()),
            )
            .on_hover_text(t::previews_budget_tooltip());
        let ended = crate::app::spinnerdraft::keep_draft(ui, id, &budget, seconds);
        if ended && (seconds - was).abs() > f32::EPSILON {
            // `set_budget` clamps again — a control narrower than what the
            // value may legally hold silently rewrites it, so the clamp lives
            // on the value as well as on the widget.
            pages
                .cache
                .set_budget(std::time::Duration::from_secs_f32(seconds));
        }
        let _ = crate::diag::ui_rect_visible(BUDGET_REGION, budget.rect, ui.clip_rect());
    });
    // The disclosure sits ABOVE the grid, not below it — the same rule the
    // Bookmarks, Signatures and Fonts panels state: an operator who looks at
    // a grid of undrawn tiles and stops has already drawn a conclusion by the
    // time a footnote would reach them.
    //
    // ⚠ And it is a report about ONE PAGE, not about the feature. Before O151
    // this sentence explained why the whole grid had stopped; it now explains
    // why one tile reads "Not finished" while its neighbours have pictures.
    if let Some(skipped) = pages.cache.skipped() {
        ui.label(
            egui::RichText::new(t::previews_skipped_note(skipped.page_index, skipped.millis))
                .small()
                .weak(),
        );
    }
}
