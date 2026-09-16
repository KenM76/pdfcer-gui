//! # `dialogs::print::position` — where the page sits on the paper
//!
//! Operator request O208: *"can we add a control to our print preview screen
//! so that when we are printing at a scale that will lose content we have the
//! option to drag the drawing to a new position on the print page? That way we
//! can choose what gets cropped."*
//!
//! ## What this module owns
//!
//! One quantity: a **displacement from the placement pdfcer chose**, per
//! document page. Everything else here is either arithmetic over that quantity
//! or the controls that set it. The copy is next door in
//! [`crate::text::print`]; the preview that draws the result is
//! [`super::preview`].
//!
//! ## ★ Contract: a delta, keyed on the DOCUMENT page, sparse
//!
//! - **A delta, not a position.** Zero means *where pdfcer put it*, which is
//!   what makes Reset a meaningful command distinct from Centre — see
//!   [`Positions::centre`] for why those two are not synonyms.
//! - **Keyed on `PagePlan::index`**, never on a position in the plan list. The
//!   job may be reversed, subset-filtered, or print a page more than once, so
//!   the two coincide only for a whole-document forward job.
//! - **Sparse.** An empty map means every placement is byte-identical to the
//!   one [`super::spooler::plan`] returned, which is what stops this feature
//!   from being able to change a job nobody has touched (`R6`).
//!
//! ## Where it is applied, and why not in the spooler
//!
//! [`Positions::displace`] is called from `PrintDialog::show` on the value
//! `plan` returns, **before any reader**. Not inside [`super::spooler`],
//! because that module's header states that nothing in it computes a
//! placement, a sequence or a scale; the operator's displacement is a shell
//! decision and it belongs on the shell side of that line.
//!
//! Applying it there rather than at each reader is what makes the preview, the
//! per-edge readout, the clip count, the commit and the trace agree by
//! construction — including [`super::verdicts`], whose cache key is the
//! [`Placement`] itself, so a displaced page correctly invalidates the ink
//! verdict measured at its old position.

use std::collections::BTreeMap;

use egui::Ui;

use crate::dialogs::print::PrintDialog;
use crate::dialogs::print::spooler::{Job, Placement};
use crate::text::print as t;
use crate::units;

/// Below this, a displacement is not one.
///
/// Drag arithmetic in `f32` screen points divided by a scale does not return to
/// exactly zero, and a page holding a delta of 1e-14 pt would read as *moved*
/// forever: Reset would stay live, the moved-page count would say "1 page", and
/// the operator would have no way to make either go away. Canonicalising in
/// [`Positions::set`] is what keeps "unmoved" reachable.
///
/// A hundredth of a point is four thousandths of a millimetre — two orders of
/// magnitude below this dialog's own unit, so nothing an operator can see is
/// rounded away.
const SETTLED_PT: f64 = 0.01;

/// The tolerance for *does it still fit*, taken from the engine rather than
/// chosen here.
///
/// `pdfcer_print::place_page` compares against `const EPS: f64 = 0.5` when it
/// sets [`Placement::clipped`]. [`clips`] recomputes that flag after a
/// displacement, so it must use the engine's number: a tighter one here would
/// report a clip on a page the engine considers fitting, and the two verdicts
/// would differ by a hair on exactly the sheets that sit on the boundary.
const EPS_PT: f64 = 0.5;

/// One arrow press, in millimetres.
///
/// A millimetre rather than a point because the readouts beside the preview are
/// in whole millimetres. A point is about a third of one, so three presses of
/// an arrow key would leave every number on screen unchanged — which reads as a
/// control that is not listening, not as a fine adjustment.
const NUDGE_MM: f64 = 1.0;

/// One arrow press with Shift held, in millimetres.
const NUDGE_COARSE_MM: f64 = 10.0;

/// A displacement from the placement pdfcer chose, in paper points.
///
/// Positive is right and down: the sense of the device context the offsets are
/// eventually handed to, and the sense of the preview on screen. It is the
/// opposite of a PDF page's own Y axis, which is why [`t::position_frame`]
/// states it rather than leaving it to be inferred.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Offset {
    /// Rightward displacement in paper points.
    pub(crate) dx_pt: f64,
    /// Downward displacement in paper points.
    pub(crate) dy_pt: f64,
}

/// Which axes a centring command acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axes {
    /// Both, from the Centre button.
    Both,
    /// Left to right only.
    Horizontally,
    /// Top to bottom only.
    Vertically,
}

/// What the primary button took hold of when a preview drag began.
///
/// ★ **Latched at `drag_started_by`, never re-derived mid-gesture.** The page
/// rectangle moves under the pointer while the page is being dragged, so a
/// per-frame hit test would classify the same gesture differently from one
/// frame to the next: drag the page far enough and the pointer leaves it, the
/// classification flips to `Paper`, and the rest of the stroke pans the view
/// instead. Deciding once is the only stable reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Grab {
    /// No primary drag is in progress.
    #[default]
    Nothing,
    /// The press landed on the placed page: the drag moves it on the sheet.
    Page,
    /// The press landed anywhere else: the drag pans the view, as it always
    /// did.
    Paper,
}

/// How far a placed page extends past the printable area, edge by edge, in
/// paper points. Never negative.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Cropped {
    /// Past the left edge.
    pub(crate) left_pt: f64,
    /// Past the right edge.
    pub(crate) right_pt: f64,
    /// Past the top edge.
    pub(crate) top_pt: f64,
    /// Past the bottom edge.
    pub(crate) bottom_pt: f64,
}

impl Cropped {
    /// The four amounts as whole millimetres, this dialog's own unit.
    fn whole_mm(self) -> (i64, i64, i64, i64) {
        (
            units::whole_mm_from_points(self.left_pt),
            units::whole_mm_from_points(self.right_pt),
            units::whole_mm_from_points(self.top_pt),
            units::whole_mm_from_points(self.bottom_pt),
        )
    }

    /// The off-canvas sentence, in whole millimetres.
    ///
    /// # ★ Reported at the dialog's resolution, deliberately
    ///
    /// An overhang that rounds to zero millimetres on all four edges is
    /// reported as fitting. That is not a rounding error being hidden: whole
    /// millimetres are the unit every length in this dialog is stated in, and
    /// both alternatives are worse — a sentence reading *"extends past the
    /// printable area — right 0 mm"* names a quantity the operator cannot act
    /// on, and a decimal here would be the only decimal on the surface.
    fn line(self) -> String {
        let (left, right, top, bottom) = self.whole_mm();
        if left <= 0 && right <= 0 && top <= 0 && bottom <= 0 {
            t::position_fits_entirely().to_owned()
        } else {
            t::position_extends_past(left, right, top, bottom)
        }
    }
}

/// **Every displacement the operator has chosen, for this dialog's lifetime.**
///
/// Sparse by design; see the module header for the contract.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Positions {
    /// Document page index to displacement. A page absent from this map is at
    /// the placement pdfcer chose.
    moved: BTreeMap<usize, Offset>,
}

impl Positions {
    /// This page's displacement, or zero.
    pub(crate) fn of(&self, page: usize) -> Offset {
        self.moved.get(&page).copied().unwrap_or_default()
    }

    /// Whether the operator has moved this page.
    pub(crate) fn is_moved(&self, page: usize) -> bool {
        self.moved.contains_key(&page)
    }

    /// How many pages carry a displacement.
    pub(crate) fn moved_count(&self) -> usize {
        self.moved.len()
    }

    /// Whether any page carries one.
    pub(crate) fn any(&self) -> bool {
        !self.moved.is_empty()
    }

    /// Set this page's displacement, canonicalising a settled one to absent.
    ///
    /// The single writer. Every other mutator routes through it so that the
    /// "a delta under [`SETTLED_PT`] is not a delta" rule cannot be bypassed by
    /// a new command forgetting it.
    fn set(&mut self, page: usize, offset: Offset) {
        if offset.dx_pt.abs() < SETTLED_PT && offset.dy_pt.abs() < SETTLED_PT {
            self.moved.remove(&page);
        } else {
            self.moved.insert(page, offset);
        }
    }

    /// Move this page by a further displacement, in paper points.
    ///
    /// The drag and the arrow keys are both this, which is why neither has any
    /// arithmetic of its own.
    pub(crate) fn nudge(&mut self, page: usize, dx_pt: f64, dy_pt: f64) {
        let was = self.of(page);
        self.set(
            page,
            Offset {
                dx_pt: was.dx_pt + dx_pt,
                dy_pt: was.dy_pt + dy_pt,
            },
        );
    }

    /// Set one axis absolutely, in paper points, leaving the other alone.
    ///
    /// What the two typed entries write.
    pub(crate) fn set_axis(&mut self, page: usize, axis: Axes, value_pt: f64) {
        let mut next = self.of(page);
        match axis {
            Axes::Horizontally => next.dx_pt = value_pt,
            Axes::Vertically => next.dy_pt = value_pt,
            Axes::Both => {
                next.dx_pt = value_pt;
                next.dy_pt = value_pt;
            }
        }
        self.set(page, next);
    }

    /// Put this page back where pdfcer placed it.
    pub(crate) fn reset(&mut self, page: usize) {
        self.moved.remove(&page);
    }

    /// Put every page back.
    pub(crate) fn reset_all(&mut self) {
        self.moved.clear();
    }

    /// **Centre the page on the printable area** — and this is not
    /// [`Self::reset`].
    ///
    /// # ★★ Why the two commands differ, which is the whole feature
    ///
    /// `pdfcer_print::place_page` centres a page that fits and then clamps:
    /// `offset_x_pt: ((aw - w) / 2.0).max(0.0)`, *"clamped at zero so an
    /// oversized page starts at the edge of the printable area rather than at a
    /// negative offset"*. So for the sheets O208 is about — the ones losing
    /// content — pdfcer's placement is flush to the top-left corner, and the
    /// whole loss falls off the right and bottom.
    ///
    /// Reset returns to that corner. Centre moves to the middle, which crops
    /// the drawing evenly on all four edges. Both are wanted, and an operator
    /// choosing what to lose off a big drawing wants the second far more often.
    ///
    /// # The one primitive, stated once
    ///
    /// > new delta = old delta + (target − current)
    ///
    /// `current` is the placement as the preview is drawing it — already
    /// displaced — so the undisplaced engine offset is never needed and
    /// therefore can never be double-counted. The drag, the arrow keys and all
    /// three centring commands are this one line, which is why there is no
    /// second arithmetic path to keep level with the first.
    ///
    /// A page whose engine placement is already centred lands on a delta of
    /// zero, which [`Self::set`] canonicalises to *unmoved* — so Centre on a
    /// page that fits correctly leaves Reset greyed rather than claiming a move
    /// that did nothing.
    pub(crate) fn centre(
        &mut self,
        page: usize,
        axes: Axes,
        current: Placement,
        page_pt: (f64, f64),
        printable_pt: (f64, f64),
    ) {
        let old = self.of(page);
        let mut next = old;
        if matches!(axes, Axes::Both | Axes::Horizontally) {
            let target = (printable_pt.0 - page_pt.0 * current.scale) / 2.0;
            next.dx_pt = old.dx_pt + (target - current.offset_x_pt);
        }
        if matches!(axes, Axes::Both | Axes::Vertically) {
            let target = (printable_pt.1 - page_pt.1 * current.scale) / 2.0;
            next.dy_pt = old.dy_pt + (target - current.offset_y_pt);
        }
        self.set(page, next);
    }

    /// **Apply every displacement to a planned job.**
    ///
    /// Called once per frame on the value [`super::spooler::plan`] returned,
    /// before any reader. Takes the job by value and hands it back so there is
    /// no window in which a caller could hold the undisplaced one.
    ///
    /// # ★ `clipped` is recomputed, but only for a page that moved
    ///
    /// The flag is a geometric verdict and a displacement changes the geometry,
    /// so leaving it alone would leave the hatch, the caption and the commit
    /// button's count describing the position the page used to be at.
    ///
    /// The guard is not an optimisation. `place_page` returns
    /// `clipped: true, scale: 1.0, offsets: 0` for degenerate input — a
    /// zero-size page or sheet — which no purely geometric formula can
    /// reproduce, so recomputing unconditionally would *clear* a flag the
    /// engine set deliberately. An unmoved page keeps the engine's answer; only
    /// a page the operator displaced gets ours.
    pub(crate) fn displace(&self, mut job: Job, page_sizes: &[(f64, f64)]) -> Job {
        if self.moved.is_empty() {
            return job;
        }
        let printable = job.device.printable_pt;
        for plan in &mut job.plans {
            let (Some(offset), Some(&size)) =
                (self.moved.get(&plan.index), page_sizes.get(plan.index))
            else {
                continue;
            };
            plan.placement.offset_x_pt += offset.dx_pt;
            plan.placement.offset_y_pt += offset.dy_pt;
            plan.placement.clipped = clips(plan.placement, size, printable);
        }
        job
    }
}

/// Whether a placed page falls outside the printable area.
///
/// The verdict `pdfcer_print::place_page` computes, restated over an arbitrary
/// offset rather than only over the centred-and-clamped one it produces. See
/// [`EPS_PT`] for why the tolerance is the engine's and not one chosen here,
/// and [`Positions::displace`] for the one case where the engine's own answer
/// must be preferred to this.
pub(crate) fn clips(placement: Placement, page_pt: (f64, f64), printable_pt: (f64, f64)) -> bool {
    let over = Cropped {
        left_pt: -placement.offset_x_pt,
        right_pt: placement.offset_x_pt + page_pt.0 * placement.scale - printable_pt.0,
        top_pt: -placement.offset_y_pt,
        bottom_pt: placement.offset_y_pt + page_pt.1 * placement.scale - printable_pt.1,
    };
    over.left_pt > EPS_PT
        || over.right_pt > EPS_PT
        || over.top_pt > EPS_PT
        || over.bottom_pt > EPS_PT
}

/// How far the placed page extends past the printable area, edge by edge.
///
/// The per-edge half of O208's second clause. The hatch answers *where* on the
/// picture; this answers *how much* as a number, which is the quantity the
/// operator is steering. Each edge is clamped at zero, so a page inside the
/// area on an edge reports nothing for it rather than a negative slack the
/// sentence would have to explain.
pub(crate) fn cropped(
    placement: Placement,
    page_pt: (f64, f64),
    printable_pt: (f64, f64),
) -> Cropped {
    Cropped {
        left_pt: (-placement.offset_x_pt).max(0.0),
        right_pt: (placement.offset_x_pt + page_pt.0 * placement.scale - printable_pt.0).max(0.0),
        top_pt: (-placement.offset_y_pt).max(0.0),
        bottom_pt: (placement.offset_y_pt + page_pt.1 * placement.scale - printable_pt.1).max(0.0),
    }
}

/// An arrow-key nudge, in paper points, or `None` if no arrow was pressed.
///
/// The step lives here rather than at the call site because it is a fact about
/// the unit the readouts are in, not about the preview — see [`NUDGE_MM`].
/// Shift multiplies it, which is the convention every drawing program on this
/// machine uses for the same gesture.
///
/// Keys are **consumed**, so an arrow that moved the page cannot also reach a
/// sibling control on the same frame.
pub(super) fn arrow_nudge(ui: &Ui) -> Option<(f64, f64)> {
    let step = units::points_from_mm(if ui.input(|i| i.modifiers.shift) {
        NUDGE_COARSE_MM
    } else {
        NUDGE_MM
    });
    let (mut dx, mut dy) = (0.0_f64, 0.0_f64);
    ui.input_mut(|i| {
        // Both modifier spellings per arrow, because the unmodified and the
        // Shift-held gesture are the SAME command at two step sizes. A single
        // `Modifiers::NONE` consume would leave Shift+Arrow unconsumed and
        // reachable by a sibling, which is how a coarse nudge would silently
        // become somebody else's keystroke.
        for (key, delta) in [
            (egui::Key::ArrowLeft, (-step, 0.0)),
            (egui::Key::ArrowRight, (step, 0.0)),
            (egui::Key::ArrowUp, (0.0, -step)),
            (egui::Key::ArrowDown, (0.0, step)),
        ] {
            if i.consume_key(egui::Modifiers::NONE, key)
                || i.consume_key(egui::Modifiers::SHIFT, key)
            {
                dx += delta.0;
                dy += delta.1;
            }
        }
    });
    if dx == 0.0 && dy == 0.0 {
        None
    } else {
        Some((dx, dy))
    }
}

/// Publish one Position button's rectangle for `ui-verify`.
///
/// `ui_rect_visible` and not `ui_rect`: this group is at the FOOT of a
/// scrolling options column, so on a short dialog it is genuinely off screen,
/// and a driver handed a rectangle for an unreachable button would click
/// whatever is drawn over it and then report the wrong thing about the result.
///
/// Published even while a button is greyed. Whether **Reset** is enabled is
/// part of what a driven check asserts — an unmoved page must refuse it — and
/// a region that vanished when the control greyed would make "refused" and
/// "absent" the same reading.
fn publish(ui: &Ui, name: &str, rect: egui::Rect) {
    crate::diag::ui_rect_visible(name, rect, ui.clip_rect());
}

/// **The Position group, at the bottom of the Pages & Layout tab.**
///
/// Draws nothing at all when there is no job, or when the stepper is on a sheet
/// the job does not contain (`R9`: an unavailable capability renders nothing,
/// and a position control with no page to act on is unavailable rather than
/// temporarily disabled).
///
/// # Why here and not in the preview strip
///
/// The strip under the preview already lays seven controls into a
/// `horizontal_wrapped` row that measures wider than the column holding it.
/// Four more would wrap it into a second row, and the strip's height is fixed
/// for the feedback-loop reason `preview::STRIP_HEIGHT_PTS` documents — so they
/// would be clipped, not merely cramped. The options column is also where every
/// other *what will be printed* answer already is.
pub(super) fn group(
    ui: &mut Ui,
    dialog: &mut PrintDialog,
    job: Option<&Job>,
    page_sizes: &[(f64, f64)],
) {
    let Some(job) = job else { return };
    let shown = dialog.preview_page.min(job.plans.len().saturating_sub(1));
    let (Some(&plan), Some(&size)) = (
        job.plans.get(shown),
        job.plans.get(shown).and_then(|p| page_sizes.get(p.index)),
    ) else {
        return;
    };
    let page = plan.index;
    let printable = job.device.printable_pt;

    ui.add_space(8.0);
    ui.separator();
    ui.label(t::position_heading());
    ui.label(
        egui::RichText::new(t::position_page_label(page + 1, size))
            .small()
            .weak(),
    );

    // The two typed entries. Displayed through millimetres every frame so the
    // number on screen is in the unit the readouts beside it are; converted
    // back only on `changed()`, so an unedited entry is never rewritten by its
    // own display rounding.
    let offset = dialog.page_positions.of(page);
    ui.horizontal(|ui| {
        ui.label(t::position_across());
        let mut across_mm = units::mm_from_points(offset.dx_pt);
        if ui
            .add(
                egui::DragValue::new(&mut across_mm)
                    .speed(NUDGE_MM)
                    .fixed_decimals(1)
                    .suffix(t::position_mm_suffix()),
            )
            .changed()
        {
            dialog.page_positions.set_axis(
                page,
                Axes::Horizontally,
                units::points_from_mm(across_mm),
            );
        }
        ui.label(t::position_down());
        let mut down_mm = units::mm_from_points(offset.dy_pt);
        if ui
            .add(
                egui::DragValue::new(&mut down_mm)
                    .speed(NUDGE_MM)
                    .fixed_decimals(1)
                    .suffix(t::position_mm_suffix()),
            )
            .changed()
        {
            dialog
                .page_positions
                .set_axis(page, Axes::Vertically, units::points_from_mm(down_mm));
        }
    });
    ui.label(egui::RichText::new(t::position_frame()).small().weak());

    // Reset first, then the three centrings. Reset is what the operator reaches
    // for to undo an experiment, so it is where the eye lands.
    ui.horizontal_wrapped(|ui| {
        let moved = dialog.page_positions.is_moved(page);
        let reset = ui
            .add_enabled(moved, egui::Button::new(t::position_reset()))
            .on_disabled_hover_text(t::position_reset_unmoved());
        publish(ui, super::REGION_POSITION_RESET, reset.rect);
        if reset.clicked() {
            dialog.page_positions.reset(page);
        }
        let centre = ui
            .button(t::position_centre())
            .on_hover_text(t::position_centre_tooltip());
        publish(ui, super::REGION_POSITION_CENTRE, centre.rect);
        if centre.clicked() {
            dialog
                .page_positions
                .centre(page, Axes::Both, plan.placement, size, printable);
        }
        let across = ui.button(t::position_centre_horizontally());
        publish(ui, super::REGION_POSITION_CENTRE_H, across.rect);
        if across.clicked() {
            dialog
                .page_positions
                .centre(page, Axes::Horizontally, plan.placement, size, printable);
        }
        let down = ui.button(t::position_centre_vertically());
        publish(ui, super::REGION_POSITION_CENTRE_V, down.rect);
        if down.clicked() {
            dialog
                .page_positions
                .centre(page, Axes::Vertically, plan.placement, size, printable);
        }
    });

    // The job-wide reset, with its scope visible before it is pressed.
    ui.horizontal_wrapped(|ui| {
        let any = dialog.page_positions.any();
        let all = ui
            .add_enabled(any, egui::Button::new(t::position_reset_all()))
            .on_disabled_hover_text(t::position_reset_all_none());
        publish(ui, super::REGION_POSITION_RESET_ALL, all.rect);
        if all.clicked() {
            dialog.page_positions.reset_all();
        }
        let scope = if any {
            t::position_moved_count(dialog.page_positions.moved_count())
        } else {
            t::position_reset_all_none().to_owned()
        };
        ui.label(egui::RichText::new(scope).small().weak());
    });

    // ★ The disclosure, OFF-CANVAS and never in the warning colour — rule 4.
    // It states geometry (*the page extends past the printable area*) and never
    // loss, because on a 1:1 CAD drawing the overhang is usually empty paper
    // and the ink verdict beside the preview is the surface entitled to make a
    // claim about content. Two surfaces making overlapping claims about one
    // risk is how a dialog comes to contradict itself.
    ui.label(
        egui::RichText::new(cropped(plan.placement, size, printable).line())
            .small()
            .weak(),
    );
    ui.label(egui::RichText::new(t::position_drag_hint()).small().weak());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An A0-ish page on an A3-ish printable area, so it overhangs both far
    /// edges exactly as `place_page` leaves it: flush to the top-left corner.
    const PAGE: (f64, f64) = (1000.0, 800.0);
    const PRINTABLE: (f64, f64) = (850.0, 750.0);

    /// The placement `place_page` returns for [`PAGE`] on [`PRINTABLE`] at
    /// actual size — asserted against the real engine by
    /// [`the_engine_still_starts_an_oversized_page_flush_at_the_corner`].
    fn flush() -> Placement {
        Placement {
            scale: 1.0,
            offset_x_pt: 0.0,
            offset_y_pt: 0.0,
            clipped: true,
        }
    }

    /// The property the whole feature rests on: an empty map is a no-op, so a
    /// job nobody has touched plans exactly as it did before O208 existed.
    #[test]
    fn an_untouched_job_is_left_alone() {
        let positions = Positions::default();
        assert!(!positions.any());
        assert_eq!(positions.of(7), Offset::default());
        assert!(!positions.is_moved(7));
    }

    /// Reset is the corner; Centre is the middle. If these two ever agreed the
    /// operator would have four buttons and three behaviours.
    #[test]
    fn centre_is_not_reset_for_an_oversized_page() {
        let mut positions = Positions::default();
        positions.centre(0, Axes::Both, flush(), PAGE, PRINTABLE);
        let offset = positions.of(0);
        assert!(
            (offset.dx_pt - (PRINTABLE.0 - PAGE.0) / 2.0).abs() < 1e-9,
            "half the horizontal overhang, negative: {offset:?}"
        );
        assert!(
            (offset.dy_pt - (PRINTABLE.1 - PAGE.1) / 2.0).abs() < 1e-9,
            "half the vertical overhang, negative: {offset:?}"
        );
        positions.reset(0);
        assert_eq!(positions.of(0), Offset::default(), "back to the corner");
    }

    /// Centring twice must land in the same place. This is the test that fails
    /// if the primitive ever stops subtracting the *current* offset and starts
    /// accumulating the target.
    #[test]
    fn centring_is_idempotent() {
        let mut positions = Positions::default();
        positions.centre(0, Axes::Both, flush(), PAGE, PRINTABLE);
        let once = positions.of(0);
        // The second press sees the DISPLACED placement, exactly as the UI does.
        let mut moved = flush();
        moved.offset_x_pt += once.dx_pt;
        moved.offset_y_pt += once.dy_pt;
        positions.centre(0, Axes::Both, moved, PAGE, PRINTABLE);
        let twice = positions.of(0);
        assert!(
            (twice.dx_pt - once.dx_pt).abs() < 1e-9 && (twice.dy_pt - once.dy_pt).abs() < 1e-9,
            "{once:?} then {twice:?}"
        );
    }

    /// One axis at a time leaves the other exactly alone, including a value the
    /// operator typed rather than centred.
    #[test]
    fn centring_one_axis_does_not_disturb_the_other() {
        let mut positions = Positions::default();
        positions.set_axis(0, Axes::Vertically, 33.0);
        positions.centre(0, Axes::Horizontally, flush(), PAGE, PRINTABLE);
        let offset = positions.of(0);
        assert!((offset.dy_pt - 33.0).abs() < 1e-9, "vertical untouched");
        assert!(offset.dx_pt < 0.0, "horizontal centred leftward");
    }

    /// A page whose engine placement is already centred — one that fits — must
    /// come back *unmoved*, so Reset stays greyed rather than claiming a move
    /// that changed nothing.
    #[test]
    fn centring_a_page_that_already_fits_leaves_it_unmoved() {
        let fits = Placement {
            scale: 1.0,
            offset_x_pt: (PRINTABLE.0 - 400.0) / 2.0,
            offset_y_pt: (PRINTABLE.1 - 300.0) / 2.0,
            clipped: false,
        };
        let mut positions = Positions::default();
        positions.centre(0, Axes::Both, fits, (400.0, 300.0), PRINTABLE);
        assert!(!positions.is_moved(0), "a zero delta is not a displacement");
    }

    /// Drag arithmetic that returns almost to zero must return to *unmoved*, or
    /// Reset is live forever with nothing to reset.
    #[test]
    fn a_settled_displacement_is_not_a_displacement() {
        let mut positions = Positions::default();
        positions.nudge(0, 10.0, 10.0);
        assert!(positions.is_moved(0));
        positions.nudge(0, -10.0 + SETTLED_PT / 2.0, -10.0);
        assert!(!positions.is_moved(0), "canonicalised back to absent");
    }

    /// The negative direction is the whole point: `place_page` clamps at zero
    /// and the operator's delta is applied after that clamp, so it must be
    /// allowed past it.
    #[test]
    fn a_displacement_may_be_negative() {
        let mut positions = Positions::default();
        positions.nudge(4, -120.0, -60.0);
        assert_eq!(
            positions.of(4),
            Offset {
                dx_pt: -120.0,
                dy_pt: -60.0
            }
        );
    }

    /// [`clips`] must agree with the engine at zero delta, because
    /// [`Positions::displace`] swaps one for the other the moment a page moves.
    #[test]
    fn clips_agrees_with_the_engine_at_zero_delta() {
        assert!(clips(flush(), PAGE, PRINTABLE), "overhangs both far edges");
        let fits = Placement {
            scale: 1.0,
            offset_x_pt: (PRINTABLE.0 - 400.0) / 2.0,
            offset_y_pt: (PRINTABLE.1 - 300.0) / 2.0,
            clipped: false,
        };
        assert!(!clips(fits, (400.0, 300.0), PRINTABLE));
    }

    /// Dragged off the NEAR edges, a page that fitted now clips — a state no
    /// version of this program could reach before O208, and the reason the
    /// hatch had to learn about four edges rather than two.
    #[test]
    fn dragging_a_fitting_page_off_the_near_edge_clips_it() {
        let dragged = Placement {
            scale: 1.0,
            offset_x_pt: -20.0,
            offset_y_pt: 5.0,
            clipped: false,
        };
        assert!(clips(dragged, (400.0, 300.0), PRINTABLE));
        let crop = cropped(dragged, (400.0, 300.0), PRINTABLE);
        assert!((crop.left_pt - 20.0).abs() < 1e-9, "20 pt off the left");
        assert_eq!(crop.right_pt, 0.0);
        assert_eq!(crop.top_pt, 0.0);
        assert_eq!(crop.bottom_pt, 0.0);
    }

    /// A tolerance-width overhang is not one, and the tolerance is the
    /// engine's. If this fails because `EPS_PT` was tuned, the engine moved and
    /// [`clips`] must follow it rather than the other way round.
    #[test]
    fn an_overhang_within_the_engines_tolerance_does_not_clip() {
        let hair = Placement {
            scale: 1.0,
            offset_x_pt: 0.0,
            offset_y_pt: 0.0,
            clipped: false,
        };
        assert!(!clips(
            hair,
            (PRINTABLE.0 + EPS_PT / 2.0, PRINTABLE.1),
            PRINTABLE
        ));
        assert!(clips(
            hair,
            (PRINTABLE.0 + EPS_PT * 2.0, PRINTABLE.1),
            PRINTABLE
        ));
    }

    /// Centring an oversized page crops it evenly, which is the operator's own
    /// words for what Centre is for.
    #[test]
    fn centring_an_oversized_page_crops_it_evenly() {
        let mut positions = Positions::default();
        positions.centre(0, Axes::Both, flush(), PAGE, PRINTABLE);
        let offset = positions.of(0);
        let mut moved = flush();
        moved.offset_x_pt += offset.dx_pt;
        moved.offset_y_pt += offset.dy_pt;
        let crop = cropped(moved, PAGE, PRINTABLE);
        assert!(
            (crop.left_pt - crop.right_pt).abs() < 1e-9,
            "even left/right: {crop:?}"
        );
        assert!(
            (crop.top_pt - crop.bottom_pt).abs() < 1e-9,
            "even top/bottom: {crop:?}"
        );
    }

    /// The per-edge sentence names only the edges that overhang, and reports a
    /// sub-millimetre overhang as fitting — see [`Cropped::line`].
    #[test]
    fn the_per_edge_sentence_names_only_the_edges_that_overhang() {
        let crop = Cropped {
            left_pt: 0.0,
            right_pt: units::points_from_mm(12.0),
            top_pt: 0.0,
            bottom_pt: units::points_from_mm(3.0),
        };
        let line = crop.line();
        assert!(line.contains("right 12 mm"), "{line}");
        assert!(line.contains("bottom 3 mm"), "{line}");
        assert!(!line.contains("left"), "{line}");
        assert!(!line.contains("top"), "{line}");

        let hair = Cropped {
            left_pt: 0.1,
            ..Cropped::default()
        };
        assert_eq!(hair.line(), t::position_fits_entirely());
    }

    /// `reset_all` is the job-wide command, and `reset` is not a loop over it.
    #[test]
    fn reset_all_clears_every_page_and_reset_clears_one() {
        let mut positions = Positions::default();
        positions.nudge(0, 5.0, 5.0);
        positions.nudge(9, 5.0, 5.0);
        assert_eq!(positions.moved_count(), 2);
        positions.reset(0);
        assert_eq!(positions.moved_count(), 1);
        assert!(positions.is_moved(9));
        positions.reset_all();
        assert!(!positions.any());
    }

    /// A page absent from `page_sizes` is skipped rather than displaced against
    /// a guessed size — the one route by which a page-range edit mid-frame
    /// could otherwise recompute `clipped` from nothing.
    #[test]
    fn displacing_a_page_with_no_known_size_leaves_its_plan_alone() {
        use crate::dialogs::print::spooler::{DeviceGeometry, JobResolution, PagePlan};
        let job = Job {
            device: DeviceGeometry {
                dpi: (600, 600),
                printable_pt: PRINTABLE,
                physical_pt: (900.0, 800.0),
                offset_pt: (25.0, 25.0),
            },
            resolution: JobResolution {
                dpi: 600,
                device_dpi: 600,
                capped: false,
                uncapped_page_mb: 0,
            },
            plans: vec![PagePlan {
                index: 3,
                placement: flush(),
                render_scale: 1.0,
            }],
        };
        let mut positions = Positions::default();
        positions.nudge(3, -50.0, -50.0);
        // One page size, for document page 0 — page 3 is not in the slice.
        let displaced = positions.displace(job.clone(), &[PAGE]);
        assert_eq!(displaced, job, "no size, no displacement");
        let displaced = positions.displace(job, &[PAGE, PAGE, PAGE, PAGE]);
        assert!(
            (displaced.plans[0].placement.offset_x_pt + 50.0).abs() < 1e-9,
            "and with the size present it moves"
        );
    }

    /// ★★ **A tripwire on the engine, not on this module.**
    ///
    /// Every sentence in [`Positions::centre`] about why Reset and Centre
    /// differ rests on one measured fact: `place_page` clamps an oversized
    /// page's offset at zero, so pdfcer's own placement is the top-left corner.
    /// If the engine ever centres an oversized page instead, Reset and Centre
    /// become synonyms, three of this group's buttons become indistinguishable,
    /// and the doc comments above become wrong — and nothing else in this
    /// program would notice.
    ///
    /// So it is asserted against the engine directly, through the real
    /// `place_page`, rather than restated here as a belief about it.
    #[test]
    fn the_engine_still_starts_an_oversized_page_flush_at_the_corner() {
        let placement =
            pdfcer_print::place_page(PAGE, PRINTABLE, pdfcer_print::ScaleMode::ActualSize);
        assert_eq!(
            (placement.offset_x_pt, placement.offset_y_pt),
            (0.0, 0.0),
            "place_page clamps at zero; Positions::centre documents this"
        );
        assert!(placement.clipped, "and reports the clip");
    }
}
