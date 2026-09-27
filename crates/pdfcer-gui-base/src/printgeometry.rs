//! # `printgeometry` — the preview's arithmetic, with
//! no dialog in it
//!
//! Three rectangles and one verdict: where the sheet and the printable area
//! land on screen, where one page lands inside them, and which parts of that
//! page fall outside the printable area while carrying ink.
//!
//! ## Why it is a module of its own
//!
//! Everything here is a pure function of a job, a rectangle and a scale, so
//! the preview tests in `pdfcer_gui::dialogs::print::preview` assert the hatch
//! geometry with no `egui` context, no printer and no rendered page.
//!
//! ## What it must not learn
//!
//! No `PrintDialog`. The zoom and the pan reach these functions as a `scale`
//! and a `Vec2`, which is what lets the preview's drag classifier
//! and its painter hit-test and draw the same rectangle
//! without either one owning the arithmetic.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/printgeometry.md`.

use egui::{Color32, Rect, Stroke, Vec2};

use crate::printink as ink;
use crate::printspooler::Job;

use crate::printpreviewkey::Overhang;

/// **The sheet and the printable area, in screen points.**
pub fn frames(job: &Job, rect: Rect, pan: Vec2, scale: f32) -> (Rect, Rect) {
    let sheet = job.device.physical_pt;
    let sheet_px = egui::vec2(sheet.0 as f32 * scale, sheet.1 as f32 * scale);
    let origin = rect.center() - sheet_px / 2.0 + pan;
    (
        Rect::from_min_size(origin, sheet_px),
        Rect::from_min_size(
            origin
                + egui::vec2(
                    job.device.offset_pt.0 as f32 * scale,
                    job.device.offset_pt.1 as f32 * scale,
                ),
            egui::vec2(
                job.device.printable_pt.0 as f32 * scale,
                job.device.printable_pt.1 as f32 * scale,
            ),
        ),
    )
}

/// **Where one page lands on screen**, given the printable rectangle
/// [`frames`] returned.
pub fn placed_rect(
    printable: Rect,
    placement: crate::printspooler::Placement,
    size: (f64, f64),
    scale: f32,
) -> Rect {
    Rect::from_min_size(
        printable.min
            + egui::vec2(
                placement.offset_x_pt as f32 * scale,
                placement.offset_y_pt as f32 * scale,
            ),
        egui::vec2(
            (size.0 * placement.scale) as f32 * scale,
            (size.1 * placement.scale) as f32 * scale,
        ),
    )
}

/// Hatch **only the parts of `placed` that fall outside `printable` AND carry
/// ink**.
pub fn hatch_lost_content(
    painter: &egui::Painter,
    placed: Rect,
    printable: Rect,
    mask: Option<&ink::InkMask>,
    colour: Color32,
) -> Overhang {
    let (lost, overhang) = lost_regions(placed, printable, mask);
    for region in lost {
        hatch(painter, region, colour);
    }
    overhang
}

/// **The four disjoint overhangs of a placed page**, in SCREEN points, in the
/// order left, right, top, bottom.
pub fn edge_bands(placed: Rect, printable: Rect) -> [Rect; 4] {
    [
        placed.intersect(Rect::everything_left_of(printable.min.x)),
        placed.intersect(Rect::everything_right_of(printable.max.x)),
        placed
            .intersect(Rect::everything_above(printable.min.y))
            .intersect(Rect::everything_right_of(printable.min.x))
            .intersect(Rect::everything_left_of(printable.max.x)),
        placed
            .intersect(Rect::everything_below(printable.max.y))
            .intersect(Rect::everything_right_of(printable.min.x))
            .intersect(Rect::everything_left_of(printable.max.x)),
    ]
}

/// The letter each of [`edge_bands`]' four bands is named by in the trace, in
/// its order.
pub const EDGE_LETTERS: [char; 4] = ['l', 'r', 't', 'b'];

/// **Which** of the page's four edges hang past the printable area, in
/// [`edge_bands`]' order.
pub fn overhang_edges(placed: Rect, printable: Rect) -> [bool; 4] {
    edge_bands(placed, printable).map(|band| band.is_positive())
}

/// **What is actually lost, and what to call it** — the whole of operator
/// request O113's decision, with no painter in it.
pub fn lost_regions(
    placed: Rect,
    printable: Rect,
    mask: Option<&ink::InkMask>,
) -> (Vec<Rect>, Overhang) {
    let bands = edge_bands(placed, printable);
    let Some(mask) = mask else {
        // No raster to ask. Disclose the whole of every band rather than
        // nothing: "unknown" must not present as "nothing is lost".
        let whole: Vec<Rect> = bands.into_iter().filter(|b| b.is_positive()).collect();
        // A clipped placement with no positive band is a geometric
        // contradiction the arithmetic can produce at a degenerate scale.
        // Report it as `Fits` rather than as an unexplained warning with no
        // picture under it.
        let verdict = if whole.is_empty() {
            Overhang::Fits
        } else {
            Overhang::Unknown
        };
        return (whole, verdict);
    };

    let mut lost = Vec::new();
    for band in bands {
        if !band.is_positive() {
            continue;
        }
        // The band, expressed as a fraction of the page, handed to the mask;
        // the ink extent inside it, brought back to screen points. `placed` is
        // the page's own rectangle on screen, so it is exactly the frame that
        // converts between the two — and it already carries the zoom, the pan
        // and the placement scale, which is why nothing here has to know about
        // any of them.
        //
        // THE REQUEST, in one `else`: no ink in the band means the band is
        // empty paper, so nothing is lost and nothing is drawn.
        let Some(extent) = mask.ink_extent(normalised_in(band, placed)) else {
            continue;
        };
        let region = denormalised_in(extent, placed);
        if region.is_positive() {
            lost.push(region);
        }
    }
    let verdict = if lost.is_empty() {
        Overhang::BlankBand
    } else {
        Overhang::Losing
    };
    (lost, verdict)
}

/// Express `part` as a fraction of `whole`: 0..1 page space, the coordinate
/// system [`ink::InkMask`] speaks.
pub fn normalised_in(part: Rect, whole: Rect) -> Rect {
    if whole.width() <= 0.0 || whole.height() <= 0.0 {
        return Rect::NOTHING;
    }
    Rect::from_min_max(
        egui::pos2(
            (part.min.x - whole.min.x) / whole.width(),
            (part.min.y - whole.min.y) / whole.height(),
        ),
        egui::pos2(
            (part.max.x - whole.min.x) / whole.width(),
            (part.max.y - whole.min.y) / whole.height(),
        ),
    )
}

/// The inverse of [`normalised_in`]: 0..1 page space back to screen points.
pub fn denormalised_in(fraction: Rect, whole: Rect) -> Rect {
    Rect::from_min_max(
        egui::pos2(
            whole.min.x + fraction.min.x * whole.width(),
            whole.min.y + fraction.min.y * whole.height(),
        ),
        egui::pos2(
            whole.min.x + fraction.max.x * whole.width(),
            whole.min.y + fraction.max.y * whole.height(),
        ),
    )
}

/// Draw diagonal hatching across `area`.
pub fn hatch(painter: &egui::Painter, area: Rect, colour: Color32) {
    let step = 6.0;
    let mut x = area.min.x;
    while x < area.max.x + area.height() {
        painter.line_segment(
            [
                egui::pos2(x.min(area.max.x), area.min.y),
                egui::pos2((x - area.height()).max(area.min.x), area.max.y),
            ],
            Stroke::new(1.0, colour),
        );
        x += step;
    }
}
