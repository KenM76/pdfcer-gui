//! **Drawing one stack** — its tab bar, then its **active** panel's body,
//! in a child ui whose union never reaches the side's frame.
//!
//! Split out of `dock/mod.rs` on 2026-09-09, when the wobble fix's own
//! explanation pushed that file past R2's 1,500-line limit. The seam is the
//! one `plan.rs` and `tabs.rs` already draw: `mod.rs` decides the side's
//! geometry and walks its columns; this file draws what one compartment
//! holds. Nothing here reads the layout or the plan — it receives a
//! resolved `rect` and a `Stack`, and returns through `DockFrameReport`.
//!
//! The load-bearing decision in this file is the one `draw_stack`'s inline
//! note carries: a body is drawn in `ui.new_child`, **not** `scope_builder`,
//! so a body that allocates past its compartment cannot grow the side.
//! `overflow_probe.rs` has the measurement behind it.

use egui::{Align, Layout, Rect, UiBuilder, Vec2};

use super::ctx::Ctx;
use super::model::{DockSide, Stack};
use super::{Dock, DockFrameReport, PanelId, plan, report, tabs};

impl<'a> Dock<'a> {
    /// Draw one stack: its tab bar, then its **active** panel's body.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_stack(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut Ctx<'_>,
        side: DockSide,
        column: usize,
        index: usize,
        stack: &Stack,
        rect: Rect,
        report: &mut DockFrameReport,
        body: &mut impl FnMut(&PanelId, &mut egui::Ui),
    ) {
        // ★★★ The tab strip is suppressed when the rail is the switch — the
        // operator's fourth ask of 2026-09-05. See [`Self::with_rail_reach`]
        // for the three conditions and for the reachability argument, which is
        // the load-bearing half.
        let suppressed = self.tabs_suppressed(ctx, side, stack);
        let bar_height = if suppressed {
            0.0
        } else {
            plan::TAB_BAR_HEIGHT.min(rect.height())
        };
        let bar_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), bar_height));
        let body_rect =
            Rect::from_min_max(egui::pos2(rect.left(), rect.top() + bar_height), rect.max);

        if suppressed {
            report.tab_strips_suppressed += 1;
        } else {
            let outcome = tabs::tab_bar(ui, ctx, side, column, index, stack, bar_rect);
            report.panels_overflowed += outcome.hidden;
            if outcome.overflow_drawn {
                report.overflow_menus += 1;
            }
        }

        // ★ ONE body, the active tab's. Failure mode #3's design rule —
        // *size a container to its active child* — is honoured by there
        // being nothing else to size it to: an inactive tab's body is
        // never constructed, so it can neither impose a width nor consume
        // a frame's work.
        //
        // The RAG entry
        // `only_the_active_tab_is_emitted_so_scripted_harnesses_cannot_reach_other_tabs.md`
        // names the consequence honestly, and it is a consequence worth
        // paying for: a harness CANNOT observe a backgrounded panel, and
        // must select its tab first. The alternative — emit everything
        // and hide it — *"converts a keyboard-navigation improvement into
        // a keyboard-navigation regression with no visual symptom at
        // all"*, because every hidden control re-enters the focus chain.
        // The harness verb is [`DockState::activate`], and
        // [`DockFrameReport::panels_drawn`] is what tells a harness which
        // panels it can currently see.
        let Some(panel) = stack.active_panel().cloned() else {
            return;
        };
        if body_rect.height() <= 0.0 || body_rect.width() <= 0.0 {
            return;
        }

        // ★★★ `new_child`, NOT `scope_builder` — 2026-09-09. A body that
        // draws more than fits is truncated, never accommodated: accommodating
        // it is what makes a panel content-driven, and a content-driven panel
        // next to a fit-to-viewport zoom is the R128 feedback loop. The clip
        // below has always delivered the PAINT half of that promise. The
        // LAYOUT half it did not: `scope_builder` ends by allocating the
        // child's `min_rect()` into this ui, so a body whose union ran 0.4 pt
        // past the compartment grew the side's frame by 0.4 pt — and
        // `egui::Panel::show` answers a frame wider than `exact_size` by
        // sliding the whole panel INWARD by the excess, which narrowed the
        // central panel by 0.4 pt on that frame. That was the two-day
        // "central-panel width jitter": egui's own solid scroll bar, fading
        // in, overshoots its pane by a rounding residue on two or three
        // frames (`overflow_probe`, `scroll_fade_repro`). A child ui whose
        // union is never merged, plus the compartment's own rect allocated
        // in its place, is what makes the side's width content-independent
        // in fact and not only by `exact_size`'s name.
        let mut child = ui.new_child(
            UiBuilder::new()
                .id_salt(ctx.id("body", side, column, index))
                .max_rect(body_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        child.set_clip_rect(body_rect.intersect(child.clip_rect()));
        body(&panel, &mut child);
        ui.expand_to_include_rect(body_rect);

        ctx.reporter.report(ui, body_rect, || report::body(&panel));
        report.panels_drawn.push(panel);
    }
}
