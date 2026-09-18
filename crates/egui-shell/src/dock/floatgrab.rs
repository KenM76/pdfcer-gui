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

use egui::{Pos2, ViewportClass, ViewportId};

use super::floatdrag::FloatDrag;
use super::model::PanelId;

/// **Convert a point reported inside a child viewport into the application
/// window's own space.**
///
/// Both origins are the **content** (inner) origins of their windows, in
/// monitor space at ui-point scale — [`egui::ViewportInfo::inner_rect`]'s
/// `min`. A child viewport's widget coordinates are relative to its own
/// content origin, so the desktop point is `child_origin + local` and the
/// application-local point is that minus the application's content origin.
///
/// ⚠ **Inner on both sides.** [`egui::ViewportInfo::outer_rect`] is content
/// *plus* decoration, so substituting it injects a constant error the size
/// of a title bar — 31 points on Windows 11 — which is plausible enough to
/// survive review and large enough to land a drop in the wrong compartment.
///
/// ⚠ A test that reads both origins from a bare [`egui::Context`] harness
/// measures nothing: `show_viewport_immediate` falls back to an embedded
/// window there, so the two origins are the same and every conversion —
/// including omitting this one entirely — collapses to the identity. The
/// tests below supply both origins themselves, and the wiring is verified by
/// driving two real windows.
#[must_use]
pub fn window_point(child_origin: Pos2, app_origin: Pos2, local: Pos2) -> Pos2 {
    local + (child_origin - app_origin)
}

/// **Where a carried window's outer corner has to be for the grabbed point to
/// stay under the cursor.**
///
/// `grabbed` is where the button went down, `local` is where the pointer is
/// now, both in the **window's own** coordinates; `outer` is the window's
/// current outer origin. The window has to move by however far the pointer
/// has got from the point it grabbed.
///
/// ★ `grabbed` does not go stale when the window moves, and that is the whole
/// reason this is expressed as an absolute target. It is a point *on the
/// window* — on the header strip — so it keeps the same window coordinates
/// however far the window travels, while `local` is re-reported against the
/// new origin every frame. Their difference is therefore the residual error
/// and not a velocity: when the window has arrived, it is zero, and the
/// command stops being sent.
///
/// ⚠ A per-frame `drag_delta` in its place oscillates. The delta that moved
/// the window is re-reported next frame as an equal and opposite local
/// movement, so the window is commanded back where it came from.
#[must_use]
pub fn carry_to(outer: Pos2, grabbed: Pos2, local: Pos2) -> Pos2 {
    outer + (local - grabbed)
}

/// **Sense a carry on a float window's header, move the window with it, and
/// report the drop.**
///
/// Called from inside the float window's viewport callback, where `ui` is
/// the child's root and `header` is the strip's `Response`. Returns the
/// report to hand to [`super::state::DockState::set_float_drag`], or `None`
/// on any frame that is not part of a carry.
///
/// # What makes it decline
///
/// - The gesture is not a drag — an ordinary click on the strip opens the
///   menu and must not also offer a drop.
/// - `class` is not [`ViewportClass::Immediate`]. An embedded fallback has
///   no window of its own, so moving it would move the application's, and
///   its origin coincides with the application's so the conversion would be
///   the identity rather than an answer.
/// - Either window's content rectangle is unreported. Both are `Option` and
///   are `None` on Android and Wayland, where a window's position cannot be
///   obtained. There is no conversion without them, so the gesture degrades
///   to what a float window does anyway — it moves — and the panel still has
///   the menu's **Dock** row as a position-blind route home.
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
        let grabbed = ctx.input(|i| i.pointer.press_origin());
        let outer = ctx.input(|i| i.viewport().outer_rect);
        if let (Some(grabbed), Some(outer)) = (grabbed, outer)
            && local != grabbed
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(carry_to(
                outer.min, grabbed, local,
            )));
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

    Some(FloatDrag {
        panel: panel.clone(),
        pointer: window_point(child_origin, app_origin, local),
        released,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The measured calibration, as an assertion.**
    ///
    /// The numbers are one drag sample from driving two real windows on
    /// Windows 11 at `ppp = 1.0`: a child window 272 points narrower than the
    /// pointer had travelled, and the *application's own* pointer report for
    /// the same physical cursor — a witness the arithmetic never reads.
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
    ///
    /// Where the two windows share an origin the conversion is the identity,
    /// which is correct — and is also exactly what omitting the conversion
    /// looks like. Asserting it here records that the test above, with two
    /// different origins, is the one doing the work.
    #[test]
    fn coinciding_origins_are_the_identity_and_prove_nothing_else() {
        let at = Pos2::new(99.0, 42.0);
        let origin = Pos2::new(300.0, 300.0);
        assert_eq!(window_point(origin, origin, at), at);
    }

    /// **A point outside the application window converts to a coordinate
    /// outside it rather than to an edge.**
    ///
    /// Off-window points are the ordinary case during a carry: the operator
    /// is over the desktop, or over another application, for most of the
    /// gesture. Clamping here would put every one of them on an edge, and the
    /// dock would offer a drop the pointer is nowhere near. Whether a point
    /// is over a compartment is [`super::super::geometry`]'s question, and it
    /// needs the truth to answer it.
    /// ★★★ **Two frames of a carry settle, where a per-frame delta would
    /// oscillate.**
    ///
    /// The second frame is the whole point and is the one a delta-driven
    /// implementation fails: the pointer has not moved physically, but the
    /// window has, so the platform re-reports the cursor 20 points further
    /// back in the window's own coordinates. A delta reads that as "the
    /// pointer moved −20" and sends the window back where it came from,
    /// giving a window that shakes for as long as the operator holds it.
    /// Here the residual is zero and nothing is sent.
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
