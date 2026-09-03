//! # `canvas::placing` — **point at the page instead of typing coordinates**
//!
//! **Operator request, `OPERATOR_REQUESTS.md` O66, 2026-08-31:**
//!
//! > *"anything we are inserting like this should have an option in its
//! > dialogue box to place it with the mouse instead of by positional
//! > co-ordinates."*
//!
//! Note *"anything we are inserting"*. That is a rule about a **class** of
//! dialog, not a feature for one window, and it is why this is a shared arm
//! rather than a drag added to `dialogs::insert_image`.
//!
//! ## What was actually missing, which is not "a drag"
//!
//! Exactly one dialog in this crate asks for a page-space position
//! numerically — [`crate::dialogs::insert_image`], with four millimetre
//! spinners. Its own module header names this gap, gives three reasons for the
//! numeric route, and ends: *"A drag-to-place gesture is a second ROUTE to the
//! same action, not a replacement for this window, and it is the natural next
//! slice."*
//!
//! Meanwhile the **canvas → dialog** direction already exists three times over
//! (`Action::BeginTextAnnot`, `FieldAction::Begin`, `open_scale_calibrated`)
//! and the **dialog → canvas** direction exactly once, hard-wired in
//! `app::frame` for the Set-scale window. So the missing thing was never the
//! gesture; it was a *general* way for a window to step aside, let the operator
//! point, and come back.
//!
//! ## ★★★ The one design decision, and it is the whole file
//!
//! **A dialog is hidden for exactly as long as a placement is pending for it,
//! and "hidden" is DERIVED from the pending record rather than stored.**
//!
//! That is `canvas::tool`'s own space-bar idiom — *"no stored override and
//! nothing to restore"* — and it is chosen here for a reason with teeth: the
//! precedent this arm generalises **is already broken in exactly the way a
//! stored flag breaks.**
//!
//! Press Escape during a Set-scale calibration today and the key lands on
//! `disarm_measure`. Nothing reopens the window, and `close_scale` has already
//! destroyed the half-typed ratio. The operator is stranded with no route back,
//! and no line of code is wrong — the cancel path simply was never one of the
//! places anybody remembered to reopen from.
//!
//! ⇒ With a stored `hidden: bool` this arm would inherit that, five times over:
//! a mode change through `tool::arm::retire_forbidden`, the Tool panel putting
//! the pen down, a ribbon control arming a different tool, the document
//! closing, Escape. Every one is a route somebody has to remember to clear a
//! flag on.
//!
//! With `hidden` derived, **stranding is unrepresentable**. Whatever clears the
//! pending record — anything at all, including a route added next year by
//! somebody who has never read this file — the window comes back on the next
//! frame, because its absence was never a fact of its own. A lost cancel costs
//! one frame of missing window instead of an operator with nowhere to go.
//!
//! ## Rule 4
//!
//! The preview drawn while a placement is armed is a **pre-commit affordance**
//! — the cursor — and is explicitly allowed. Nothing here marks applied
//! content, and the moment the placement commits it is an ordinary insert that
//! renders exactly as a saved one will.

use crate::app::modes::Capabilities;
use crate::canvas::tool::CanvasTool;

/// Where the pending placement lives in [`egui::Memory`].
///
/// Beside `canvas::tool`'s own `TOOL_MEMORY_KEY`, and for the same reason: it
/// is per-window state with no meaning outside the frame loop, and putting it
/// on `OpenDoc` would make a *gesture* a property of the *document*.
const PLACING_MEMORY_KEY: &str = "pdfcer-canvas-placing"; // ui-text-exempt: internal memory id, never displayed

/// Where a completed placement waits for `app::frame` to collect it.
const RESULT_MEMORY_KEY: &str = "pdfcer-canvas-placing-result"; // ui-text-exempt: internal memory id, never displayed

/// Where a cancellation waits to be collected.
///
/// ★ Separate from the result rather than an `Option` inside it. *"The
/// operator placed nothing"* and *"the operator has not finished yet"* are
/// different states, and a single slot would make them the same absence — the
/// distinction `crate::app::files::Picked` exists to preserve, applied here.
const CANCELLED_MEMORY_KEY: &str = "pdfcer-canvas-placing-cancelled"; // ui-text-exempt: internal memory id, never displayed

/// **Which window is waiting for a point.**
///
/// One variant today. It is an enum rather than a bare marker because the
/// operator's sentence is about a class — *"anything we are inserting"* — and
/// the second member is the whole reason this file is shared rather than
/// inlined. See [`crate::dialogs::placing`] for what a dialog has to do to
/// join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceKind {
    /// [`crate::dialogs::insert_image`] — the only numeric-position dialog in
    /// the crate, and the one the operator was looking at.
    Image,
}

impl PlaceKind {
    /// **Whether the active mode may finish what this placement starts.**
    ///
    /// Asked at the press rather than only at the arm, on
    /// `canvas::gesture::meaning`'s standing rule: a mode can change while a
    /// tool is armed, and a gesture that could not commit must not begin.
    ///
    /// Placing an image is page-content authoring, so it takes `edit_content` —
    /// the same gate `edit.insert_image` itself carries.
    #[must_use]
    pub const fn capability(self, caps: Capabilities) -> bool {
        match self {
            Self::Image => caps.edit_content,
        }
    }
}

/// A placement waiting for the operator to point.
///
/// The **page** travels with the kind because a placement is a position *on a
/// page*, and the operator can turn pages while pointing. Capturing it at the
/// arm would be wrong — they may well page to the sheet they want *because*
/// the window stepped aside — so this records which page the request came
/// from and the result carries whichever page they actually clicked on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pending {
    /// Which window is waiting.
    pub kind: PlaceKind,
    /// The page the dialog was opened against, for the trace.
    pub page: usize,
}

/// **Arm a placement**: record who is waiting, and put the canvas in the tool
/// that collects it.
///
/// ★ One call rather than two, so the record and the tool cannot disagree.
/// A `Pending` with no `CanvasTool::Place` armed is a hidden dialog nothing
/// will ever return to, and the only way to make that unreachable is to make
/// the two impossible to set separately.
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
///
/// Called by Escape, by `tool::arm::retire_forbidden` when a mode change takes
/// the capability away, and by `app::frame`'s invariant sweep when the dialog
/// that asked has gone. Every one of those is a route back to a visible window,
/// because clearing the record is all "un-hide" means.
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
///
/// Read-and-clear, on `canvas::measure`'s stated rule for every edge held in
/// `egui::Memory`: a value left behind fires again on some later frame, which
/// the operator experiences as a window moving something they placed minutes
/// ago.
pub fn take_result(ctx: &egui::Context) -> Option<(PlaceKind, pdfcer_core::page_tree::Rect)> {
    // ★ `get_temp` then `remove`, rather than `remove_temp` — the latter needs
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
///
/// Lower-left rather than centre, and it is `canvas::clicking`'s own rule for
/// the form-field arm: *"it matches what the drag does"*, so the two gestures
/// agree about what the pointer means and an operator who switches between them
/// is not surprised.
/// ★★★ **`point` is in CANVAS space and is converted here.** The two spaces
/// differ by a y flip, and the flip is invisible: a mirrored placement is a
/// plausible number on a plausible page, so nothing refuses it and nothing
/// looks wrong until an operator clicks near the top of a sheet and the picture
/// lands near the bottom.
///
/// That is exactly what this function shipped doing. Every sibling arm in
/// `canvas::clicking` — the form field, the sticky note — converts through
/// [`crate::canvas::markup::band::endpoints`] before building a
/// `page_tree::Rect`, and this one passed the raw canvas point straight into
/// one. It agreed with itself, it agreed with its unit tests, and it disagreed
/// with [`band_released`] by a mirror.
///
/// ⇒ Found on the first run of the driven check
/// `the_insert_window_steps_aside_so_you_can_point`, which is the whole
/// argument for R1: the click landed at PDF y 759 — the application's own
/// `canvas-pointer` line says so — and the placement was recorded at 465.
/// `1224 − 759 = 465`.
pub fn click(ctx: &egui::Context, page: &pdfcer_core::page_tree::Page, point: egui::Pos2) {
    let Some(pending) = pending(ctx) else {
        return;
    };
    // ★ Through `band::endpoints` rather than `viewer::canvas_to_pdf_space`
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
///
/// Normalised here rather than at the call site so a drag up-and-left produces
/// the same rect as one down-and-right, which is what every other band in this
/// canvas does.
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
///
/// A plain rectangle, because that is what a placement is: two corners and the
/// area between them. It borrows `markup::band`'s preview type rather than
/// growing one, so the band a placement drags and the band a markup drags are
/// drawn by the same code and cannot look different.
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
///
/// The whole body of `canvas::interact`'s `GestureOutcome::Place` arm, lifted
/// here so that file keeps its R2 headroom and so the page-space conversion
/// sits beside the rest of this module's arithmetic rather than in the middle
/// of a gesture pipeline.
///
/// ★ Declines silently when the page cannot be resolved, which is the same
/// answer every other band in `interact` gives: a release over no page is not
/// a placement, and inventing one would put an image at a coordinate nobody
/// pointed at.
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
///
/// ★ A named function rather than two inline casts, and not only for tidiness:
/// the narrowing happens at exactly one boundary — where `markup::band`'s
/// `f64` endpoints meet `egui`'s `f32` geometry — so a reader asking *"where
/// does the precision go?"* gets one answer instead of two identical ones with
/// an attribute apiece.
///
/// The loss is not material here. These are page coordinates on a sheet
/// measured in points, where `f32` carries about seven significant figures
/// against a largest sensible magnitude in the low hundreds of thousands.
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
    // ★ The tool goes down HERE rather than in `app::frame`, so the crosshair
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
///
/// ★ A newtype rather than four `f64`s in the slot: the four numbers have an
/// order and a meaning, and a tuple of them is a thing three call sites could
/// each get subtly wrong.
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
    ///
    /// The property `Picked` exists to preserve, asserted here because the
    /// tempting simplification — one `Option<Rect>` slot where `None` means
    /// cancelled — makes "they pointed nowhere" and "they have not pointed
    /// yet" the same observation, and the second is true on almost every frame.
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
    ///
    /// Load-bearing: Escape consults this on every press, and a `cancel` that
    /// reported success unconditionally would swallow the key from the three
    /// claimants below it in `canvas::keys`.
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
            contents_flattened: 0,
        }
    }

    /// ★★★ **A click is recorded in PDF space, not canvas space.**
    ///
    /// The regression test for the defect the driven check found on its first
    /// run: `click` took the canvas point it was handed and wrote it into a
    /// `page_tree::Rect` unconverted, so a placement near the TOP of the sheet
    /// was recorded near the BOTTOM. Nothing refused it — a mirrored
    /// coordinate is a perfectly ordinary number on a perfectly ordinary page.
    ///
    /// ★★ Note what the previous version of this test asserted: that the rect
    /// carried the numbers passed in. That is true of the broken build and of
    /// the fixed one, because it was a test of the *plumbing* on a function
    /// whose defect was the *space*. This one asserts the flip by magnitude —
    /// canvas y 200 on an 800 pt page is PDF y 600 — which no unconverted
    /// build can satisfy.
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

        // ★ Backwards on both axes, because a band dragged up-and-left must
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

    /// ★★ Placing with nothing armed does nothing at all.
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
