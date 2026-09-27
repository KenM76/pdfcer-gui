//! # `printposition` — where the page sits on the paper
//!
//! Operator request O208: *"can we add a control to our print preview screen
//! so that when we are printing at a scale that will lose content we have the
//! option to drag the drawing to a new position on the print page? That way we
//! can choose what gets cropped."*
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/printposition.md`.

use std::collections::BTreeMap;

use egui::Ui;

use crate::printspooler::{Job, Placement};
use crate::text::print as t;
use crate::units;

/// Below this, a displacement is not one.
const SETTLED_PT: f64 = 0.01;

/// The tolerance for *does it still fit*, taken from the engine rather than
/// chosen here.
const EPS_PT: f64 = 0.5;

/// One arrow press, in millimetres.
pub const NUDGE_MM: f64 = 1.0;

/// One arrow press with Shift held, in millimetres.
const NUDGE_COARSE_MM: f64 = 10.0;

/// A displacement from the placement pdfcer chose, in paper points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Offset {
    /// Rightward displacement in paper points.
    pub dx_pt: f64,
    /// Downward displacement in paper points.
    pub dy_pt: f64,
}

/// Which axes a centring command acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axes {
    /// Both, from the Centre button.
    Both,
    /// Left to right only.
    Horizontally,
    /// Top to bottom only.
    Vertically,
}

/// What the primary button took hold of when a preview drag began.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Grab {
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
pub struct Cropped {
    /// Past the left edge.
    pub left_pt: f64,
    /// Past the right edge.
    pub right_pt: f64,
    /// Past the top edge.
    pub top_pt: f64,
    /// Past the bottom edge.
    pub bottom_pt: f64,
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
    pub fn line(self) -> String {
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
pub struct Positions {
    /// Document page index to displacement. A page absent from this map is at
    /// the placement pdfcer chose.
    moved: BTreeMap<usize, Offset>,
}

impl Positions {
    /// This page's displacement, or zero.
    pub fn of(&self, page: usize) -> Offset {
        self.moved.get(&page).copied().unwrap_or_default()
    }

    /// Whether the operator has moved this page.
    pub fn is_moved(&self, page: usize) -> bool {
        self.moved.contains_key(&page)
    }

    /// How many pages carry a displacement.
    pub fn moved_count(&self) -> usize {
        self.moved.len()
    }

    /// Whether any page carries one.
    pub fn any(&self) -> bool {
        !self.moved.is_empty()
    }

    /// Set this page's displacement, canonicalising a settled one to absent.
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
    pub fn nudge(&mut self, page: usize, dx_pt: f64, dy_pt: f64) {
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
    pub fn set_axis(&mut self, page: usize, axis: Axes, value_pt: f64) {
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
    pub fn reset(&mut self, page: usize) {
        self.moved.remove(&page);
    }

    /// Put every page back.
    pub fn reset_all(&mut self) {
        self.moved.clear();
    }

    /// **Centre the page on the printable area** — and this is not
    /// [`Self::reset`].
    pub fn centre(
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
    pub fn displace(&self, mut job: Job, page_sizes: &[(f64, f64)]) -> Job {
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
pub fn clips(placement: Placement, page_pt: (f64, f64), printable_pt: (f64, f64)) -> bool {
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
pub fn cropped(placement: Placement, page_pt: (f64, f64), printable_pt: (f64, f64)) -> Cropped {
    Cropped {
        left_pt: (-placement.offset_x_pt).max(0.0),
        right_pt: (placement.offset_x_pt + page_pt.0 * placement.scale - printable_pt.0).max(0.0),
        top_pt: (-placement.offset_y_pt).max(0.0),
        bottom_pt: (placement.offset_y_pt + page_pt.1 * placement.scale - printable_pt.1).max(0.0),
    }
}

/// An arrow-key nudge, in paper points, or `None` if no arrow was pressed.
pub fn arrow_nudge(ui: &Ui) -> Option<(f64, f64)> {
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
        use crate::printspooler::{DeviceGeometry, JobResolution, PagePlan};
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
                tile: None,
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

    /// **A tripwire on the engine, not on this module.**
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
