//! # `dock::floatgrab` — the gesture that carries a float window back
//!
//! [`super::floatdrag`] owns what a carried float *means* to the dock: the
//! compass, the preview and the drop. This owns the gesture that produces
//! it — a drag on the float window's own header strip — and the arithmetic
//! that turns a pointer reported in a child window into a point in the
//! application window's space.
//!
//! ## Why the header strip and not the OS title bar
//!
//! A drag begun on the window's OS title bar is the platform's gesture: it
//! runs its own modal move loop, and `egui` in either context sees no
//! pointer at all until the button comes up. Only an OS cursor read — a
//! platform crate this crate does not link — could follow it.
//!
//! A drag begun on the strip **inside** the window is the child window's,
//! and the child keeps reporting the pointer for the whole gesture,
//! including far outside its own bounds. That is measured, not assumed:
//! `D:/dev/rag/egui/a_pointer_from_a_child_viewport_converts_through_inner_rects_the_os_cursor_through_outer.md`.
//!
//! ## The two halves, and why they take different inputs
//!
//! | Half | What it computes | In which space |
//! |---|---|---|
//! | **Report the drop** | [`window_point`] of the live pointer | the application window's, because that is where the dock measured the rectangles a drop resolves against |
//! | **Move the window** | [`carry_to`] of the live pointer and the press | the child window's own, because the point the operator grabbed is a point *on the window* and never moves relative to it |
//!
//! ⚠ The move is **not** expressed by re-asserting the window's position in
//! its [`egui::ViewportBuilder`]. That value is read back out of the window
//! one frame late, so asserting it every frame drags the window toward where
//! the program thinks it is — [`super::floatwin`]'s header carries the rule.
//!
//! ⚠ Nor is it an accumulated delta. Per-frame deltas oscillate here: move
//! the window by however far the pointer moved and the platform re-reports
//! the cursor at a *local* position that much further back, so the next
//! frame's delta is the negative of the last and the window shakes. What
//! [`carry_to`] computes is an **absolute target** — put the window where the
//! grabbed point lands under the cursor — which is self-cancelling instead:
//! once the window is where it should be, the offset it is driven by is zero.
//!
//! ⚠ That is necessary and not sufficient. A pass that runs immediately after
//! a commanded move reads a pointer measured across that move, and it reports
//! the residual as the exact negative of the one just acted on — so an
//! otherwise correct absolute target still sends the window back where it came
//! from, every step, for the whole gesture. [`Settling`] is what declines that
//! one pass. Measured on Windows 11 by driving the binary; see
//! `D:/dev/rag/egui/a_window_moved_under_the_cursor_reports_one_pass_of_unreconciled_pointer.md`.
//!
//! Design and rationale: `docs/modules/egui-shell/dock/floatgrab.md`.

use egui::{Pos2, ViewportClass, ViewportId};

use super::floatdrag::FloatDrag;
use super::model::PanelId;

/// **Convert a point reported inside a child viewport into the application
/// window's own space.**
#[must_use]
pub fn window_point(child_origin: Pos2, app_origin: Pos2, local: Pos2) -> Pos2 {
    local + (child_origin - app_origin)
}

/// **Where a carried window's outer corner has to be for the grabbed point to
/// stay under the cursor.**
#[must_use]
pub fn carry_to(outer: Pos2, grabbed: Pos2, local: Pos2) -> Pos2 {
    outer + (local - grabbed)
}

/// **Sense a carry on a float window's header, move the window with it, and
/// report the drop.**
pub(super) fn carry(
    ui: &egui::Ui,
    class: ViewportClass,
    header: &egui::Response,
    panel: &PanelId,
) -> Option<FloatDrag> {
    if class != ViewportClass::Immediate {
        return None;
    }
    let dragging = header.dragged();
    let released = header.drag_stopped();
    if !dragging && !released {
        return None;
    }
    let ctx = ui.ctx();
    let local = header.interact_pointer_pos()?;

    // The window follows first, so the point reported below is reported from a
    // window that has already been asked to be where the pointer is.
    //
    // `press_origin` is the grabbed point: recorded once, in this window's
    // coordinates, and never re-based — which is what makes it the fixed end
    // of the arithmetic while `local` is re-reported against a moving origin.
    if dragging {
        let mut settling = Settling::of(ctx, panel);
        if header.drag_started() {
            settling.reset();
        }
        let grabbed = ctx.input(|i| i.pointer.press_origin());
        let outer = ctx.input(|i| i.viewport().outer_rect);
        if let (Some(grabbed), Some(outer)) = (grabbed, outer)
            && local != grabbed
            && settling.may_move()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(carry_to(
                outer.min, grabbed, local,
            )));
            settling.moved();
        }
    }

    let child_origin = ctx.input(|i| i.viewport().inner_rect)?.min;
    // The ROOT viewport's state rather than this one's. Both live in the same
    // `Context`, and the root's input was filled at the start of this same
    // frame — the child's pass runs from inside it — so this is the
    // application window's current content origin, not a stale one.
    //
    // Sequenced after the read above rather than nested inside it: both take
    // the context's lock.
    let app_origin = ctx
        .viewport_for(ViewportId::ROOT, |v| v.input.viewport().inner_rect)?
        .min;

    // A carry is a continuous gesture, and the dock draws its offer from a
    // report it consumes on the *next* application frame. Without this, a
    // drag over a main window with nothing else going on offers nothing at
    // all, because nothing asked it to repaint.
    ctx.request_repaint_of(ViewportId::ROOT);

    if released {
        Settling::of(ctx, panel).reset();
    }

    Some(FloatDrag {
        panel: panel.clone(),
        pointer: window_point(child_origin, app_origin, local),
        released,
    })
}

/// **Whether the window may be moved on this pass, or has just been moved and
/// is not yet reconciled with the pointer.**
struct Settling {
    id: egui::Id,
    ctx: egui::Context,
    just_moved: bool,
}

impl Settling {
    fn of(ctx: &egui::Context, panel: &PanelId) -> Self {
        let id = egui::Id::new(("egui-shell::floatgrab::settling", panel));
        let just_moved = ctx.data(|d| d.get_temp::<bool>(id).unwrap_or(false));
        Self {
            id,
            ctx: ctx.clone(),
            just_moved,
        }
    }

    /// **Consumes the flag.** A declined pass clears it, so the pass after it
    /// is free whether or not the platform honoured the command — which is
    /// what keeps a refused move a stutter rather than a freeze.
    fn may_move(&mut self) -> bool {
        let was = self.just_moved;
        if was {
            self.set(false);
        }
        !was
    }

    fn moved(&mut self) {
        self.set(true);
    }

    fn reset(&mut self) {
        self.set(false);
    }

    fn set(&mut self, value: bool) {
        self.just_moved = value;
        let id = self.id;
        self.ctx.data_mut(|d| d.insert_temp(id, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The measured calibration, as an assertion.**
    #[test]
    fn the_conversion_reproduces_the_application_windows_own_report() {
        let child_origin = Pos2::new(242.0, 265.0);
        let app_origin = Pos2::new(190.0, 213.0);
        let local = Pos2::new(558.0, 335.0);
        assert_eq!(
            window_point(child_origin, app_origin, local),
            Pos2::new(610.0, 387.0)
        );
    }

    /// **Coinciding origins are the case a harness produces, and they must
    /// not be the case that carries the evidence.**
    #[test]
    fn coinciding_origins_are_the_identity_and_prove_nothing_else() {
        let at = Pos2::new(99.0, 42.0);
        let origin = Pos2::new(300.0, 300.0);
        assert_eq!(window_point(origin, origin, at), at);
    }

    /// **A point outside the application window converts to a coordinate
    /// outside it rather than to an edge.**
    #[test]
    fn a_carry_settles_where_a_delta_would_oscillate() {
        let grabbed = Pos2::new(50.0, 10.0);
        let outer = Pos2::new(100.0, 100.0);

        // Frame 1 — the pointer has moved 20 points right of the grab.
        let local = Pos2::new(70.0, 10.0);
        let moved_to = carry_to(outer, grabbed, local);
        assert_eq!(moved_to, Pos2::new(120.0, 100.0));

        // Frame 2 — the cursor has not moved, so its position in the window
        // that moved under it is the grabbed point again.
        let local = local - egui::vec2(20.0, 0.0);
        assert_eq!(local, grabbed, "the residual is what settles the loop");
        assert_eq!(
            carry_to(moved_to, grabbed, local),
            moved_to,
            "an arrived window must be commanded nowhere"
        );
    }

    /// **The measured three-pass cycle, replayed, with the gate in place.**
    #[test]
    fn the_pass_after_a_move_is_declined_so_the_window_never_reverses() {
        let grabbed = Pos2::new(160.0, 21.0);
        #[rustfmt::skip]
        let passes = [
            (Pos2::new(270.0, 270.0), Pos2::new(174.0, 35.0)),
            (Pos2::new(284.0, 284.0), Pos2::new(146.0,  7.0)),
            (Pos2::new(284.0, 284.0), Pos2::new(160.0, 21.0)),
            (Pos2::new(284.0, 284.0), Pos2::new(174.0, 35.0)),
            (Pos2::new(298.0, 298.0), Pos2::new(146.0,  7.0)),
            (Pos2::new(298.0, 298.0), Pos2::new(160.0, 21.0)),
        ];

        let ctx = egui::Context::default();
        let panel = PanelId::new("carried");
        let mut settling = Settling::of(&ctx, &panel);
        settling.reset();

        let mut sent = Vec::new();
        let mut at = passes[0].0;
        for (outer, local) in passes {
            // The window is wherever the last honoured command put it; the
            // replayed `outer` is what the pass reported having read.
            let _ = outer;
            if local != grabbed && Settling::of(&ctx, &panel).may_move() {
                let target = carry_to(at, grabbed, local);
                sent.push(target);
                at = target;
                Settling::of(&ctx, &panel).moved();
            }
        }

        assert_eq!(sent.len(), 2, "one move per pointer step, not one per pass");
        for pair in sent.windows(2) {
            assert!(
                pair[1].x > pair[0].x && pair[1].y > pair[0].y,
                "the window reversed: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    /// **A command the platform declined is re-sent, not waited on forever.**
    #[test]
    fn a_refused_move_is_retried_on_the_pass_after_the_declined_one() {
        let ctx = egui::Context::default();
        let panel = PanelId::new("clamped");
        let mut allowed = 0;
        for _ in 0..6 {
            if Settling::of(&ctx, &panel).may_move() {
                allowed += 1;
                Settling::of(&ctx, &panel).moved();
            }
        }
        assert_eq!(allowed, 3, "every other pass, indefinitely");
    }

    #[test]
    fn a_point_outside_the_application_window_stays_outside() {
        let converted = window_point(
            Pos2::new(10.0, 10.0),
            Pos2::new(500.0, 400.0),
            Pos2::new(20.0, 30.0),
        );
        assert_eq!(converted, Pos2::new(-470.0, -360.0));
    }
}
