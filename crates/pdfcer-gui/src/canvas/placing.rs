//! # `canvas::placing` — **point at the page instead of typing coordinates**
//!
//!
//! > *"anything we are inserting like this should have an option in its
//! > dialogue box to place it with the mouse instead of by positional
//! > co-ordinates."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/placing.md`.

use crate::app::modes::Capabilities;
use crate::canvas::tool::CanvasTool;

/// Where the pending placement lives in [`egui::Memory`].
const PLACING_MEMORY_KEY: &str = "pdfcer-canvas-placing"; // ui-text-exempt: internal memory id, never displayed

/// Where a completed placement waits for `app::frame` to collect it.
const RESULT_MEMORY_KEY: &str = "pdfcer-canvas-placing-result"; // ui-text-exempt: internal memory id, never displayed

/// Where a cancellation waits to be collected.
const CANCELLED_MEMORY_KEY: &str = "pdfcer-canvas-placing-cancelled"; // ui-text-exempt: internal memory id, never displayed

/// **Which window is waiting for a point.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceKind {
    /// [`crate::dialogs::insert_image`] — the only numeric-position dialog in
    /// the crate, and the one the operator was looking at.
    Image,
}

impl PlaceKind {
    /// **Whether the active mode may finish what this placement starts.**
    #[must_use]
    pub const fn capability(self, caps: Capabilities) -> bool {
        match self {
            Self::Image => caps.edit_content,
        }
    }
}

/// A placement waiting for the operator to point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pending {
    /// Which window is waiting.
    pub kind: PlaceKind,
    /// The page the dialog was opened against, for the trace.
    pub page: usize,
}

/// **Arm a placement**: record who is waiting, and put the canvas in the tool
/// that collects it.
pub fn arm(ctx: &egui::Context, kind: PlaceKind, page: usize) {
    ctx.data_mut(|d| d.insert_temp(id(PLACING_MEMORY_KEY), Pending { kind, page }));
    crate::canvas::tool::select(ctx, CanvasTool::Place(kind));
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("place-armed kind={kind:?} page={page}")
    });
}

/// Who is waiting, if anyone.
///
/// The one read [`crate::dialogs::placing::PlaceHandoff::hidden`] derives from.
#[must_use]
pub fn pending(ctx: &egui::Context) -> Option<Pending> {
    ctx.data(|d| d.get_temp::<Pending>(id(PLACING_MEMORY_KEY)))
}

/// **Abandon a pending placement**, reporting whether there was one.
pub fn cancel(ctx: &egui::Context) -> bool {
    let Some(pending) = pending(ctx) else {
        return false;
    };
    ctx.data_mut(|d| {
        d.remove::<Pending>(id(PLACING_MEMORY_KEY));
        d.insert_temp(id(CANCELLED_MEMORY_KEY), pending.kind);
    });
    crate::canvas::tool::select(ctx, CanvasTool::Select);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("place-cancelled kind={:?}", pending.kind)
    });
    true
}

/// Take a completed placement, if one landed this frame.
pub fn take_result(ctx: &egui::Context) -> Option<(PlaceKind, pdfcer_core::page_tree::Rect)> {
    // `get_temp` then `remove`, rather than `remove_temp` — the latter needs
    // `Default` on the stored type, and neither a `PlaceKind` nor a rectangle
    // has an honest default. Two calls inside one `data_mut` is the same
    // atomicity for the same cost.
    ctx.data_mut(|d| {
        let taken = d.get_temp::<(PlaceKind, PlacedRect)>(id(RESULT_MEMORY_KEY));
        if taken.is_some() {
            d.remove::<(PlaceKind, PlacedRect)>(id(RESULT_MEMORY_KEY));
        }
        taken
    })
    .map(|(kind, r)| (kind, r.0))
}

/// Take a cancellation, if one landed this frame. See [`cancel`].
pub fn take_cancelled(ctx: &egui::Context) -> Option<PlaceKind> {
    ctx.data_mut(|d| {
        let taken = d.get_temp::<PlaceKind>(id(CANCELLED_MEMORY_KEY));
        if taken.is_some() {
            d.remove::<PlaceKind>(id(CANCELLED_MEMORY_KEY));
        }
        taken
    })
}

/// **A click placed it**: the pointer is the lower-left corner and the size is
/// left to the dialog.
pub fn click(ctx: &egui::Context, page: &pdfcer_core::page_tree::Page, point: egui::Pos2) {
    let Some(pending) = pending(ctx) else {
        return;
    };
    // Through `band::endpoints` rather than `viewer::canvas_to_pdf_space`
    // directly, so the click and the drag share ONE conversion. Two call sites
    // of the same helper can drift; one helper called twice cannot.
    let Some((at, _)) = crate::canvas::markup::band::endpoints(point, point, page) else {
        return;
    };
    // A degenerate rect, deliberately. The dialog reads the corner and keeps
    // whatever size it already had — see `dialogs::insert_image::place`.
    let rect = pdfcer_core::page_tree::Rect {
        llx: at.0,
        lly: at.1,
        urx: at.0,
        ury: at.1,
    };
    finish(ctx, pending.kind, rect);
}

/// **A drag placed it**: the two corners are the box.
pub fn completed(ctx: &egui::Context, from: egui::Pos2, to: egui::Pos2) {
    let Some(pending) = pending(ctx) else {
        return;
    };
    let rect = pdfcer_core::page_tree::Rect {
        llx: f64::from(from.x.min(to.x)),
        lly: f64::from(from.y.min(to.y)),
        urx: f64::from(from.x.max(to.x)),
        ury: f64::from(from.y.max(to.y)),
    };
    finish(ctx, pending.kind, rect);
}

/// **The rubber band a placement drags out**, for `canvas::interact` to paint.
#[must_use]
pub fn band(from: egui::Pos2, to: egui::Pos2) -> crate::canvas::markup::band::Preview {
    crate::canvas::markup::band::Preview {
        kind: crate::canvas::markup::MarkupKind::Rectangle,
        from,
        to,
    }
}

/// **Collect the answer when the band is released**, and do nothing on every
/// other frame of the drag.
pub fn band_released(
    ctx: &egui::Context,
    doc: &crate::app::state::OpenDoc,
    from: egui::Pos2,
    to: egui::Pos2,
    phase: crate::canvas::gesture::Phase,
) {
    if phase != crate::canvas::gesture::Phase::Complete {
        return;
    }
    let Some(page) = doc.current_page() else {
        return;
    };
    let Some((start, end)) = crate::canvas::markup::band::endpoints(from, to, page) else {
        return;
    };
    completed(ctx, page_pos(start), page_pos(end));
}

/// One page-space `(f64, f64)` as the `Pos2` this module's arithmetic uses.
#[allow(clippy::cast_possible_truncation)] // ui-text-exempt: a clippy lint name, never displayed
fn page_pos(p: (f64, f64)) -> egui::Pos2 {
    egui::pos2(p.0 as f32, p.1 as f32)
}

/// Record the answer, put the tool down, and say so.
fn finish(ctx: &egui::Context, kind: PlaceKind, rect: pdfcer_core::page_tree::Rect) {
    ctx.data_mut(|d| {
        d.remove::<Pending>(id(PLACING_MEMORY_KEY));
        d.insert_temp(id(RESULT_MEMORY_KEY), (kind, PlacedRect(rect)));
    });
    // The tool goes down HERE rather than in `app::frame`, so the crosshair
    // is gone on the same frame the window comes back. Leaving it armed for one
    // more frame would put a placement cursor over a dialog that is asking for
    // a different kind of input.
    crate::canvas::tool::select(ctx, CanvasTool::Select);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "place-result kind={kind:?} llx={:.1} lly={:.1} urx={:.1} ury={:.1}",
            rect.llx, rect.lly, rect.urx, rect.ury
        )
    });
}

/// `pdfcer_core::page_tree::Rect` is not `Clone` in the way `egui::Memory`
/// wants, so it travels wrapped.
#[derive(Debug, Clone, Copy)]
struct PlacedRect(pdfcer_core::page_tree::Rect);

/// One spelling of every memory id in this module.
fn id(key: &str) -> egui::Id {
    egui::Id::new(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cancellation and a result are different answers and must not share a
    /// slot.
    #[test]
    fn a_cancellation_is_not_an_absent_result() {
        let ctx = egui::Context::default();
        arm(&ctx, PlaceKind::Image, 3);
        assert_eq!(pending(&ctx).map(|p| p.kind), Some(PlaceKind::Image));
        assert!(take_result(&ctx).is_none(), "nothing has been placed yet");

        assert!(cancel(&ctx), "there was a placement to cancel");
        assert!(pending(&ctx).is_none(), "…and it is no longer pending");
        assert_eq!(take_cancelled(&ctx), Some(PlaceKind::Image));
        assert!(
            take_cancelled(&ctx).is_none(),
            "the cancellation is read-and-clear, or it fires again next frame"
        );
        assert!(take_result(&ctx).is_none(), "a cancel produces no result");
    }

    /// Cancelling nothing is not an event.
    #[test]
    fn cancelling_nothing_reports_nothing() {
        let ctx = egui::Context::default();
        assert!(!cancel(&ctx));
        assert!(take_cancelled(&ctx).is_none());
    }

    /// A page fixture, the same shape `markup::band`'s tests use — this
    /// module's conversion reads exactly what theirs does, `crop_box` and
    /// `rotate`.
    fn test_page(w: f64, h: f64) -> pdfcer_core::page_tree::Page {
        pdfcer_core::page_tree::Page {
            id: pdfcer_core::object::ObjId::new(1, 0),
            resources: pdfcer_core::object::Dict::new(),
            media_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, w, h),
            crop_box: pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, w, h),
            rotate: 0,
            contents: Vec::new(),
            contents_unresolved: 0,
            resources_defaulted: false,
            contents_flattened: 0,
        }
    }

    /// **A click is recorded in PDF space, not canvas space.**
    #[test]
    fn a_click_places_a_corner_and_a_drag_places_a_box() {
        let ctx = egui::Context::default();
        let page = test_page(600.0, 800.0);

        arm(&ctx, PlaceKind::Image, 0);
        click(&ctx, &page, egui::pos2(100.0, 200.0));
        let (kind, r) = take_result(&ctx).expect("a click places");
        assert_eq!(kind, PlaceKind::Image);
        assert!(
            (r.llx - 100.0).abs() < 1e-3,
            "x agrees between the two spaces: {r:?}"
        );
        assert!(
            (r.lly - 600.0).abs() < 1e-3,
            "★ canvas y counts DOWN and PDF y counts UP: 200 on an 800 pt page is 600. A rect carrying 200 is the mirrored placement: {r:?}"
        );
        assert!(
            (r.urx - r.llx).abs() < 1e-9,
            "a click carries no size — the dialog keeps the one it had"
        );
        assert!(pending(&ctx).is_none(), "the placement is spent");

        // Backwards on both axes, because a band dragged up-and-left must
        // produce the same rect as one dragged down-and-right.
        arm(&ctx, PlaceKind::Image, 0);
        completed(&ctx, egui::pos2(300.0, 400.0), egui::pos2(120.0, 250.0));
        let (_, r) = take_result(&ctx).expect("a drag places");
        assert!(
            (r.llx - 120.0).abs() < 1e-9 && (r.lly - 250.0).abs() < 1e-9,
            "{r:?}"
        );
        assert!(
            (r.urx - 300.0).abs() < 1e-9 && (r.ury - 400.0).abs() < 1e-9,
            "{r:?}"
        );
    }

    /// Placing with nothing armed does nothing at all.
    ///
    /// The guard that keeps a stray click on the canvas from delivering a
    /// placement to a window that never asked for one.
    #[test]
    fn a_click_with_nothing_pending_is_not_a_placement() {
        let ctx = egui::Context::default();
        click(&ctx, &test_page(600.0, 800.0), egui::pos2(10.0, 10.0));
        assert!(take_result(&ctx).is_none());
        completed(&ctx, egui::pos2(0.0, 0.0), egui::pos2(5.0, 5.0));
        assert!(take_result(&ctx).is_none());
    }

    /// Arming records the tool as well as the request.
    ///
    /// The pair that cannot be set separately — see [`arm`].
    #[test]
    fn arming_a_placement_arms_the_tool_that_collects_it() {
        let ctx = egui::Context::default();
        arm(&ctx, PlaceKind::Image, 0);
        assert_eq!(
            crate::canvas::tool::active(&ctx),
            CanvasTool::Place(PlaceKind::Image)
        );
        cancel(&ctx);
        assert_eq!(
            crate::canvas::tool::active(&ctx),
            CanvasTool::Select,
            "cancelling puts the pointer back, or the crosshair outlives its reason"
        );
    }
}
