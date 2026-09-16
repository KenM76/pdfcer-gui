//! # `dialogs::print::preview::geometry` — the preview's arithmetic, with
//! no dialog in it
//!
//! Three rectangles and one verdict: where the sheet and the printable area
//! land on screen, where one page lands inside them, and which parts of that
//! page fall outside the printable area while carrying ink.
//!
//! ## Why it is a module of its own
//!
//! Everything here is a pure function of a job, a rectangle and a scale. That
//! is what makes [`super::tests`] able to assert the hatch geometry
//! with no `egui` context, no printer and no rendered page — and the
//! properties it asserts (bands pairwise disjoint, union exactly the overhang)
//! are the kind that a body sharing a file with a painter invites nobody to
//! check.
//!
//! The split was forced by R2 when operator request O208 widened the hatch from
//! two bands to four, and the seam was already there: the tests next door drive
//! exactly this and nothing else in `preview`.
//!
//! ## What it must not learn
//!
//! No `PrintDialog`. The zoom and the pan reach these functions as a `scale`
//! and a `Vec2`, which is what lets the drag classifier in [`super::column`]
//! and the painter in [`super::paint`] hit-test and draw the same rectangle
//! without either one owning the arithmetic.

use egui::{Color32, Rect, Stroke, Vec2};

use crate::dialogs::print::ink;
use crate::dialogs::print::spooler::Job;

use super::Overhang;

/// **The sheet and the printable area, in screen points.**
///
/// Its own function because two callers need the same answer and neither may
/// compute it: [`super::paint`] draws these rectangles, and [`super::column`] hit-tests the
/// page inside them to decide whether a primary drag moves the page or pans the
/// view (operator request O208). A second copy of this arithmetic would let the
/// gesture be classified against a rectangle other than the one on screen,
/// which is a defect with no symptom except the occasional drag doing the wrong
/// thing near an edge.
///
/// `pan` is passed rather than read from the dialog so the caller decides which
/// frame's pan it wants. [`super::column`] hit-tests against the pan the page was
/// last DRAWN at, which is the position the operator pressed on.
pub(crate) fn frames(job: &Job, rect: Rect, pan: Vec2, scale: f32) -> (Rect, Rect) {
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
///
/// The offsets are *within the printable area*, not within the sheet, which is
/// why this takes `printable` rather than the sheet origin — see
/// [`crate::dialogs::print::spooler::Placement`].
///
/// ★ The `Placement` here is the spooler's, spelled out in full because
/// this module has a private enum of its own by the same name (the dialog-or-
/// popout one). Importing it would make the two indistinguishable at a glance
/// in a file that uses both.
pub(crate) fn placed_rect(
    printable: Rect,
    placement: crate::dialogs::print::spooler::Placement,
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
///
/// # Why ink and not geometry — operator request O113
///
/// > *"can you make it so the red pattern you put over the page if it is going
/// > to print beyond the printable borders is only over the areas that extend
/// > beyond the printable page? Our drawing get drawn 1:1 and the area that
/// > isn't printed is just empty border."*
///
/// `Placement::clipped` is a *geometric* verdict — the page box exceeds the
/// printable rectangle — and on a CAD sheet printed 1:1 the part that exceeds
/// it is empty paper. Hatching on that flag alone shouts about losing something
/// on every drawing while nothing is being lost, which is a disclosure that is
/// technically true and practically false. An operator who sees the same red
/// band on every 1:1 drawing learns to ignore it, and then does not see it on
/// the one sheet where the border really does have a title block in it.
///
/// So this asks [`ink::InkMask`] what is actually in the band, and hatches the
/// **ink extent within it**. No ink in the band ⇒ **no hatch at all**.
///
/// # All four edges, because the operator can now move the page
///
/// Operator request O208, his second clause: *"the hash lines we use to show
/// what won't be printed should have a line for each edge of the page."*
///
/// Before the page could be dragged, only the far edges could overhang: a
/// placement offsets the page *into* the printable area from its top-left
/// corner and `place_page` clamps that offset at zero, so the near edges were
/// unreachable and a near band would have hatched paper that prints. Both
/// halves of that changed together. The operator's displacement is applied
/// after the clamp and is allowed to go negative, so a page dragged left or up
/// loses content off the near edges, and a hatch that marked only the far ones
/// would be silent about exactly the crop the drag was performed to choose.
///
/// The four bands are computed unconditionally rather than under a test for
/// which edges overhang, because an empty band is dropped by `is_positive()`
/// below — so a page nobody has moved produces the same two bands it
/// always did, and the widening costs nothing to a job with no displacement.
///
/// # The four bands are DISJOINT, and `Rect::union` cannot make them so
///
/// `Rect::union` is a **bounding box**, not a set union: the union of a tall
/// strip on the right and a wide strip along the bottom is a rectangle that
/// also covers the region which is neither right of nor below the printable
/// area — paper that prints perfectly. That is the same over-hatch O113 reports,
/// one size smaller and hiding inside it.
///
/// So the bottom band is cut at `printable.max.x` and the two meet without
/// overlapping. Disjoint also means the shared bottom-right corner is hatched
/// once rather than twice, so its lines are the same weight as everywhere else
/// instead of reading as a darker patch.
///
/// # What happens when there is no mask
///
/// `mask` is `None` when the page did not render — the same degraded state
/// [`super::texture_for`] documents, in which the preview shows a flat fill instead of
/// the page. In that state the honest answer to *"is anything in the band?"* is
/// **"unknown"**, so the disclosure falls back to hatching the whole band.
/// Silence is the wrong failure direction here: a missing render must not be
/// able to turn a warning off.
/// # Returns
///
/// What the band turned out to hold, so the caption can be written from the
/// same computation the hatch was — see [`Overhang`].
pub(crate) fn hatch_lost_content(
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
///
/// Left and right take the page's full height; top and bottom are then cut to
/// the printable area's own horizontal span. That asymmetry is what makes the
/// four disjoint: each corner of the overhang belongs to the vertical band that
/// covers it and to no horizontal one, so their union is exactly `placed` minus
/// `printable` with nothing counted twice.
///
/// A band that does not exist comes back non-positive rather than absent, so
/// the array is always four long and the order is a contract shared by the
/// hatch and by [`overhang_edges`]. An unmoved page can only overhang right and
/// bottom, which is why widening this from two bands to four is a no-op until
/// somebody moves one.
pub(crate) fn edge_bands(placed: Rect, printable: Rect) -> [Rect; 4] {
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
pub(crate) const EDGE_LETTERS: [char; 4] = ['l', 'r', 't', 'b'];

/// **Which** of the page's four edges hang past the printable area, in
/// [`edge_bands`]' order.
///
/// The headless evidence for the four-edge hatch, and it has to say *which*
/// rather than *how many*: a capture cannot tell three hatched bands from two,
/// and a count cannot tell a page hanging off the left from one hanging off the
/// right, which is the whole of what changed. The hatch is then filtered by the
/// ink test, so the geometry has to be reported separately from what was
/// actually drawn.
///
/// Derived from [`edge_bands`] rather than from the offsets, so the word in the
/// trace and the rectangles on screen cannot disagree.
pub(crate) fn overhang_edges(placed: Rect, printable: Rect) -> [bool; 4] {
    edge_bands(placed, printable).map(|band| band.is_positive())
}

/// **What is actually lost, and what to call it** — the whole of operator
/// request O113's decision, with no painter in it.
///
/// # Pure on purpose, because this is the pair that must not disagree
///
/// It returns the rectangles to hatch *and* the [`Overhang`] the caption is
/// written from, from **one** computation. That is the only structural
/// guarantee that the picture and the sentence agree: they are not two readings
/// of the same data, they are two halves of one answer. Splitting it out from
/// [`hatch_lost_content`] also makes that agreement **testable with no GUI at
/// all** — see [`super::tests::a_blank_overhang_hatches_nothing_and_says_so`], which
/// asserts the empty list and the `BlankBand` verdict together, and
/// [`super::tests::an_inked_overhang_hatches_only_the_ink_and_says_so`], which asserts
/// the hatched rectangle really is a small part of the band.
///
/// The returned rectangles are in screen points and are ready to draw; the
/// caller does no further arithmetic on them.
pub(crate) fn lost_regions(
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
///
/// A degenerate `whole` — a page placed at zero scale, which a nonsense
/// `/MediaBox` can produce — yields a rectangle the mask rejects rather than a
/// `NaN` that would propagate into the hatch geometry.
pub(crate) fn normalised_in(part: Rect, whole: Rect) -> Rect {
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
pub(crate) fn denormalised_in(fraction: Rect, whole: Rect) -> Rect {
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
///
/// Separate from [`hatch_lost_content`] so the geometry question (*what is
/// lost?*) and the drawing question (*what does a hatch look like?*) do not
/// share a body. The lines run at 45° and are clamped to the rectangle at both
/// ends, which is what lets a caller hatch several small regions without any of
/// them bleeding into the paper between.
pub(crate) fn hatch(painter: &egui::Painter, area: Rect, colour: Color32) {
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
