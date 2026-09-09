//! **The wobble probe** — which widget in a side's body ran past the window's
//! edge, published by egui id so two frames can be compared.
//!
//! # The mechanism this instrument exists to attribute (2026-09-09)
//!
//! `egui::Panel::show` (0.35, `containers/panel.rs`, the block after the
//! frame's `show`) takes the panel's rect back from the frame's response —
//! the **union of everything the body allocated** — and, if that union is
//! wider than `exact_size` along the panel's axis, `set_rect_size`s it back
//! to the exact width **by moving the inner edge**: for a right panel,
//! `rect.min.x = rect.max.x - size`. So a body whose union reaches
//! `1400.4` on a `1400`-wide window becomes `[1080.4, 1400.4]` — width
//! exactly 320, translated +0.4 — and the central panel beside it, cut at
//! `visible_outer_rect.min.x`, is 0.4 narrower on that frame. That is the
//! whole of the "central-panel width jitter" `RESUME.md` has chased for two
//! days: not egui rounding, not the window, but **one widget overflowing its
//! pane by a sub-pixel amount on isolated frames**, amplified into a panel
//! translation by egui's exact-size clamp.
//!
//! `dock.<side>.body_min` (published at the end of the side's closure)
//! measured exactly that union — `max.x = 1400.4`, left edge unchanged — on
//! the wobble frame, while every rect this crate names stayed put. So the
//! culprit is a widget the dock does not name: something inside a panel
//! body, drawn by the application or by egui itself.
//!
//! # What this module does
//!
//! Nothing, on a frame where the side's frame rect lies inside its parent.
//! On a frame where it crosses the parent's outer edge, every widget egui
//! registered **this pass** whose rect crosses that same edge is published
//! as `dock.<side>.overflow.<id>`, where `<id>` is the widget's egui id in
//! its short debug form. A hash rather than a name — but the **same** hash
//! for the same widget on every frame, which is what a bisect needs: read
//! two wobble frames, intersect the id sets, and the survivor is the widget
//! to go and find (`egui::Id::short_debug_format` of an id built from a
//! known salt can be reproduced in a unit test to confirm the match).
//!
//! It reads `this_pass.widgets` rather than the body ui's own children
//! because the union egui takes back is built from **allocations**
//! (`Ui::allocate_space` → `register_rect`), and every allocation reaches
//! `this_pass.widgets`; a probe that walked only what the dock draws would
//! miss precisely the widget it is looking for.
//!
//! # Why it also publishes `.frame`
//!
//! `dock/mod.rs` sits at R2's 1,500-line ceiling, so the `.frame` report —
//! the rect egui ALLOCATED for the side, the thing that visibly wobbles —
//! moved here beside the probe that explains it. They are read together.
//!
//! # What it found, on its first read — and why it stays
//!
//! Five widgets, one leaf: the Comments list's vertical scroll bar, at
//! `[1390.0, 292.7]–[1400.4, 866.0]`, decaying to `1400.1` on the next
//! frame and `1400.0` on the one after. Reproduced with nothing but egui
//! in `scroll_fade_repro.rs`: a solid-style `ScrollArea` fading its bar
//! in rounds its content rect to whole pixels, adopts that rounded width
//! as the inner width (`auto_shrink`), and adds the *fractional* animated
//! bar use back — overshooting its pane by the rounding residue on the
//! frames where the fade is mid-way. `D:/dev/rag/egui/` has the entry.
//!
//! The fix is in `Dock::draw_stack`: a body is drawn in a child ui whose
//! union is never merged into the side, so nothing a body does can move
//! the side's frame. This module stays as the **tripwire** for that
//! promise — it costs one comparison per side per frame and publishes
//! nothing until something gets past the guard, at which point it names
//! the widget rather than leaving the next reader a wobble to hunt.

use egui::Rect;

use super::ctx::Ctx;
use super::model::DockSide;
use super::report;

/// How far past the parent's edge counts as an overflow, in points.
///
/// The measured wobble is 0.3–0.4 pt, a 1/32-grid multiple; `0.05` is
/// well under it and well over f32 noise at coordinates near 1400.
const OVERFLOW_TOLERANCE_PT: f32 = 0.05;

/// Publish the side's allocated frame rect, and — on a frame where it
/// crosses the parent's outer edge — every registered widget that crosses
/// that edge too.
///
/// `parent` is the ui the side was shown in (its `max_rect` is the window's
/// content rect, so its outer edge is the edge a right dock must not pass);
/// `frame` is the response rect `Panel::show` returned.
pub(super) fn publish(parent: &egui::Ui, ctx: &mut Ctx<'_>, side: DockSide, frame: Rect) {
    ctx.reporter
        .report(parent, frame, || format!("{}.frame", report::side(side)));

    let outer = parent.max_rect();
    if !crosses(side, frame, outer) {
        return;
    }
    let offenders: Vec<(String, Rect)> = parent.ctx().viewport(|v| {
        v.this_pass
            .widgets
            .layers()
            .flat_map(|(_, ws)| ws.iter())
            .filter(|w| crosses(side, w.rect, outer))
            .map(|w| (w.id.short_debug_format(), w.rect))
            .collect()
    });
    for (id, rect) in offenders {
        ctx.reporter.report(parent, rect, || {
            format!("{}.overflow.{id}", report::side(side))
        });
    }
}

/// Whether `rect` runs past `outer`'s edge on the side's OUTER side — the
/// window edge for a right dock, which is the edge `Panel::show` holds fixed
/// while it moves the inner one.
fn crosses(side: DockSide, rect: Rect, outer: Rect) -> bool {
    match side {
        DockSide::Left => rect.min.x < outer.min.x - OVERFLOW_TOLERANCE_PT,
        DockSide::Right => rect.max.x > outer.max.x + OVERFLOW_TOLERANCE_PT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    /// The measured wobble frame — a body union 0.4 pt past a 1400-wide
    /// window — crosses; the same rect at rest does not; and 0.01 pt of
    /// float noise is below the tolerance.
    #[test]
    fn a_right_dock_crosses_only_past_the_window_edge() {
        let outer = Rect::from_min_max(pos2(0.0, 0.0), pos2(1400.0, 900.0));
        let at_rest = Rect::from_min_max(pos2(1080.0, 40.0), pos2(1400.0, 900.0));
        let wobble = Rect::from_min_max(pos2(1080.0, 40.0), pos2(1400.4, 900.0));
        let noise = Rect::from_min_max(pos2(1080.0, 40.0), pos2(1400.01, 900.0));
        assert!(!crosses(DockSide::Right, at_rest, outer));
        assert!(crosses(DockSide::Right, wobble, outer));
        assert!(!crosses(DockSide::Right, noise, outer));
    }

    /// The left dock's outer edge is the window's LEFT edge; a rect past the
    /// right edge is not its overflow.
    #[test]
    fn a_left_dock_crosses_only_past_the_left_edge() {
        let outer = Rect::from_min_max(pos2(0.0, 0.0), pos2(1400.0, 900.0));
        let left = Rect::from_min_max(pos2(-0.4, 40.0), pos2(320.0, 900.0));
        let right = Rect::from_min_max(pos2(0.0, 40.0), pos2(1400.4, 900.0));
        assert!(crosses(DockSide::Left, left, outer));
        assert!(!crosses(DockSide::Left, right, outer));
    }

    /// ★★★ **The regression guard for the wobble.** A body that allocates
    /// 0.4 pt past its compartment — what egui's own solid scroll bar does on
    /// a fade-in frame — must not move the side's frame by one point of a
    /// point. Written against the unfixed `draw_stack` first: it reported
    /// `dock.right.frame` at `[1080.4 .. 1400.4]` (falsified 2026-09-09),
    /// exactly the driven trace's line.
    ///
    /// Two assertions, and the second is not implied by the first: the frame
    /// staying put says the guard held; no `overflow.*` region says the
    /// tripwire agrees nothing got past it.
    #[test]
    fn a_body_that_overflows_its_pane_does_not_move_the_sides_frame() {
        use crate::dock::{Dock, DockLayout, DockState, SideLayout};
        let mut state = DockState::new(DockLayout::new(
            SideLayout::default(),
            SideLayout::single("comments"),
        ));
        state.layout_mut().right.width_pts = 320.0;
        let mut rects: Vec<(String, Rect)> = Vec::new();
        let mut sink = |r: &crate::dock::report::RectReport<'_>| {
            rects.push((r.name.to_string(), r.rect));
        };
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                pos2(0.0, 0.0),
                egui::vec2(1400.0, 900.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            Dock::new()
                .reporting_rects_to(&mut sink)
                .show(ui, &mut state, |_, ui| {
                    // 0.4 pt wider than the compartment, like a scroll bar mid-fade.
                    let w = ui.available_width() + 0.4;
                    ui.allocate_space(egui::vec2(w, 20.0));
                });
        });
        let frame = rects
            .iter()
            .rev()
            .find(|(n, _)| n == "dock.right.frame")
            .map(|(_, r)| *r)
            .expect("the side publishes its frame");
        assert!(
            (frame.min.x - 1080.0).abs() < 0.01 && (frame.max.x - 1400.0).abs() < 0.01,
            "the side's frame moved with its body's overflow: {frame:?}"
        );
        let tripped: Vec<&String> = rects
            .iter()
            .map(|(n, _)| n)
            .filter(|n| n.starts_with("dock.right.overflow."))
            .collect();
        assert!(tripped.is_empty(), "the tripwire fired: {tripped:?}");
    }
}
