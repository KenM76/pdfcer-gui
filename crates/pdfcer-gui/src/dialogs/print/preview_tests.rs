#![cfg(test)]
//! # `dialogs::print::preview_tests` — the preview's arithmetic, proved headlessly
//!
//! Split out of [`super::preview`] on 2026-09-03, when operator request O113
//! took that file past R2's 1500-line ceiling. The seam is the one
//! `egui-shell`'s `dock/width_tests.rs` already uses in this workspace: the
//! module keeps the code, the sibling keeps the proof.
//!
//! ## What is proved here, and why every one of these needs no GUI
//!
//! Everything in this file exercises a **pure function** — the zoom anchor, the
//! raster scale, the premultiplied-alpha read, the page-space mapping, and the
//! lost-region decision O113 added. That is not a coincidence of what happened
//! to be testable; it is the reason `preview.rs` is shaped the way it is. Each
//! of these can be *silently* wrong — a drifting anchor, a hatch over the wrong
//! strip, a caption contradicting the picture beside it — and none of them
//! would look broken enough in a screenshot to investigate.
//!
//! ★ The one thing a capture genuinely cannot distinguish is
//! [`super::Overhang::BlankBand`] from a mask that found no ink anywhere: both
//! draw nothing. That is what
//! [`a_blank_overhang_hatches_nothing_and_says_so`]'s second assertion, and the
//! `overhang=` field in `column`'s diagnostic trace, exist to separate.
//!
//! `#![cfg(test)]` is the FIRST line of the file, so nothing here reaches a
//! release build and the module costs the shipped binary nothing.

use super::*;

/// The screen position a sheet point lands at, for a given view.
///
/// Mirrors [`paint`]'s own `origin` computation so the anchor tests below
/// assert the property that matters — "this point did not move" — rather
/// than re-stating the formula they are meant to be checking.
fn on_screen(
    sheet_pt: Vec2,
    fit: f32,
    zoom: f32,
    pan: Vec2,
    centre: Pos2,
    point_in_sheet: Vec2,
) -> Pos2 {
    let s = fit * zoom;
    let origin = centre - (sheet_pt * s) / 2.0 + pan;
    origin + point_in_sheet * s
}

/// ★ The point under the pointer does not move when you zoom on it.
///
/// This is the whole reason the anchor term exists, and it is the one
/// property a reader can check without re-deriving the algebra. Asserted
/// on an OFF-CENTRE point, because every wrong version of this formula —
/// including simply omitting the term — is correct at the centre.
#[test]
fn ctrl_wheel_zoom_holds_the_point_under_the_pointer_still() {
    // US Letter, fitted into a 340 x 400 canvas at the same margin factor
    // the preview uses.
    let sheet = egui::vec2(612.0, 792.0);
    let fit = (340.0_f32 / sheet.x).min(400.0 / sheet.y) * FIT_MARGIN;
    let centre = egui::pos2(170.0, 200.0);
    // A point near the sheet's bottom-right, which is where an operator
    // checking a margin actually looks.
    let target_in_sheet = egui::vec2(560.0, 730.0);

    let (zoom0, pan0) = (1.0_f32, Vec2::ZERO);
    let at = on_screen(sheet, fit, zoom0, pan0, centre, target_in_sheet);

    let (zoom1, pan1) = zoomed_view(zoom0, pan0, 2.5, at, centre);
    let after = on_screen(sheet, fit, zoom1, pan1, centre, target_in_sheet);

    assert!(
        (after - at).length() < 0.001,
        "the anchored point moved from {at:?} to {after:?} — without the \
         (at - centre)(1 - k) term, zooming in on a corner walks the sheet \
         off the canvas"
    );
    assert!(
        (zoom1 - 2.5).abs() < 1e-6,
        "the zoom itself must still be applied; got {zoom1}"
    );
}

/// A button press anchors on the canvas centre, which is the degenerate
/// case `pan1 = k * pan0` — the sheet grows about the middle rather than
/// about wherever the pointer happened to be resting.
#[test]
fn a_button_zoom_scales_the_existing_pan_about_the_centre() {
    let centre = egui::pos2(170.0, 200.0);
    let (zoom, pan) = zoomed_view(2.0, egui::vec2(30.0, -12.0), 1.25, centre, centre);
    assert!((zoom - 2.5).abs() < 1e-6);
    assert!((pan.x - 37.5).abs() < 1e-4, "pan.x was {}", pan.x);
    assert!((pan.y + 15.0).abs() < 1e-4, "pan.y was {}", pan.y);
}

/// ★ A zoom the clamp refuses must not pan either.
///
/// The bug this pins is subtle and would look like a hardware fault: at
/// maximum zoom the wheel stops magnifying but keeps sliding the sheet
/// sideways, so the preview appears to drift on its own. It comes from
/// using the REQUESTED step for the anchor term instead of the effective,
/// post-clamp ratio.
#[test]
fn a_refused_zoom_leaves_the_pan_exactly_where_it_was() {
    let pan = egui::vec2(21.0, -8.0);
    let (zoom, after) = zoomed_view(
        ZOOM_MAX,
        pan,
        4.0,
        egui::pos2(300.0, 40.0),
        egui::pos2(170.0, 200.0),
    );
    assert!((zoom - ZOOM_MAX).abs() < 1e-6, "clamped at the ceiling");
    assert!(
        (after - pan).length() < 1e-4,
        "a refused zoom moved the sheet from {pan:?} to {after:?}"
    );

    // The same at the floor.
    let (zoom, after) = zoomed_view(
        ZOOM_MIN,
        pan,
        0.1,
        egui::pos2(300.0, 40.0),
        egui::pos2(170.0, 200.0),
    );
    assert!((zoom - ZOOM_MIN).abs() < 1e-6);
    assert!((after - pan).length() < 1e-4);
}

/// A hostile or degenerate step is a no-op rather than a `NaN` that
/// poisons every later frame's pan arithmetic.
#[test]
fn a_non_finite_or_negative_step_changes_nothing() {
    let pan = egui::vec2(3.0, 4.0);
    let centre = egui::pos2(0.0, 0.0);
    for step in [f32::NAN, f32::INFINITY, 0.0, -1.5] {
        let (zoom, after) = zoomed_view(2.0, pan, step, egui::pos2(10.0, 10.0), centre);
        assert!(
            (zoom - 2.0).abs() < 1e-6 && (after - pan).length() < 1e-6,
            "step {step} must be ignored, got zoom {zoom} pan {after:?}"
        );
    }
}

/// An ordinary page renders at the target DPI — the pixel ceiling does not
/// bind, and must not quietly downgrade every normal preview.
#[test]
fn a_letter_page_previews_at_the_target_resolution() {
    let scale = raster_scale((612.0, 792.0));
    assert!(
        (scale - TARGET_DPI / 72.0).abs() < 1e-6,
        "a Letter page must not be capped; got {scale}"
    );
}

/// ★ A large-format sheet is capped by PIXELS, not by DPI.
///
/// The bound that matters. An ANSI E sheet at the target DPI would be
/// 5100 x 6600 px and about 134 MB of RGBA for a picture drawn 300 pt
/// wide — and CAD sheets are exactly the population this project's
/// operator prints, so this is the common case, not the exotic one.
#[test]
fn a_large_format_sheet_is_capped_by_pixels() {
    let sheet = (2448.0, 3168.0); // ANSI E, 34 x 44 inches.
    let scale = raster_scale(sheet);
    let longest = sheet.0.max(sheet.1) as f32 * scale;
    assert!(
        longest <= MAX_SIDE_PX + 0.5,
        "the long side rendered to {longest} px, over the {MAX_SIDE_PX} ceiling"
    );
    assert!(
        scale < TARGET_DPI / 72.0,
        "the cap must actually bind on this size; got {scale}"
    );
}

/// ★ Where the pixel ceiling starts to bind, asserted from both sides.
///
/// The regression the ceiling's own doc comment records is a value chosen
/// too low: 1600 px silently downgraded Letter, Legal and A4 — the common
/// case — in order to bound the rare one. Asserting only that those three
/// are uncapped would let the constant drift *upward* unnoticed instead,
/// so the boundary is pinned from both directions.
///
/// **A3 is on the capped side, and that is correct rather than a
/// near-miss.** Its long edge is 1191 pt, which at the target DPI is
/// 2481 px — past the 2200 ceiling. A3 is a drafting sheet, not an office
/// page, so it belongs with the large-format population this bound exists
/// for; US Legal at 2100 px is the largest size that clears it. If either
/// constant moves, this test says which side of the line each size landed
/// on rather than merely that something changed.
#[test]
fn the_pixel_ceiling_binds_above_the_office_sizes() {
    for (name, size) in [
        ("A4", (595.0, 842.0)),
        ("Letter", (612.0, 792.0)),
        ("Legal", (612.0, 1008.0)),
    ] {
        let scale = raster_scale(size);
        assert!(
            (scale - TARGET_DPI / 72.0).abs() < 1e-6,
            "{name} was capped to {scale}; the ceiling is meant to leave every \
             office page size at the full target DPI"
        );
    }
    for (name, size) in [("A3", (842.0, 1191.0)), ("ANSI E", (2448.0, 3168.0))] {
        let scale = raster_scale(size);
        assert!(
            scale < TARGET_DPI / 72.0,
            "{name} was NOT capped ({scale}); the ceiling is meant to bind on \
             drafting and large-format sheets, which is where the memory goes"
        );
    }
}

/// A degenerate `/MediaBox` must not divide by zero. Real files carry
/// them — the renderer has its own guards, and this only has to hand it a
/// finite number.
#[test]
fn a_zero_sized_page_yields_a_finite_scale() {
    let scale = raster_scale((0.0, 0.0));
    assert!(scale.is_finite() && scale > 0.0, "got {scale}");
}

/// ★★★ **The overhang band maps into page space correctly under zoom and
/// pan** — the half of operator request O113 that lives in this file.
///
/// [`super::ink::InkMask`] speaks 0..1 page space and knows nothing about
/// the canvas. Everything the preview does — the fit, the operator's zoom,
/// their pan, and the placement's own scale — reaches the mask only through
/// [`normalised_in`], whose `whole` is the page's on-screen rectangle. If
/// that mapping is wrong the mask is asked about **the wrong part of the
/// page**, and the failure is quiet: a hatch appears somewhere plausible
/// and covers the wrong ink.
///
/// So the property asserted is the one that matters — the same band comes
/// back as the same fraction of the page no matter where or how big the
/// page is drawn — rather than a re-statement of the arithmetic.
#[test]
fn the_overhang_band_is_the_same_fraction_of_the_page_at_every_zoom_and_pan() {
    // A page 1000 pt wide whose right 200 pt overhang: a fifth of it.
    let placements = [
        // Fitted, unpanned.
        (
            Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1000.0, 800.0)),
            800.0,
        ),
        // Zoomed in three times and panned off to the left.
        (
            Rect::from_min_size(egui::pos2(-1700.0, -450.0), egui::vec2(3000.0, 2400.0)),
            -1700.0 + 2400.0,
        ),
        // Zoomed out below fit.
        (
            Rect::from_min_size(egui::pos2(120.0, 60.0), egui::vec2(250.0, 200.0)),
            320.0,
        ),
    ];
    for (placed, printable_right) in placements {
        let band = placed.intersect(Rect::everything_right_of(printable_right));
        let fraction = normalised_in(band, placed);
        assert!(
            (fraction.min.x - 0.8).abs() < 1e-4 && (fraction.max.x - 1.0).abs() < 1e-4,
            "the overhanging fifth of the page came back as {fraction:?} for a page \
             drawn at {placed:?} — the mask would then be asked about the wrong strip \
             and would hatch the wrong ink"
        );
        // And the round trip is exact, which is what lets the extent the
        // mask returns be drawn straight back onto the canvas.
        let back = denormalised_in(fraction, placed);
        assert!(
            (back.min.x - band.min.x).abs() < 1e-2 && (back.max.x - band.max.x).abs() < 1e-2,
            "round trip of {band:?} through page space returned {back:?}"
        );
    }
}

/// A page drawn 1000 x 800 on screen whose right 200 pt and bottom 100 pt
/// hang past the printable area — the shape of a 1:1 CAD sheet on a
/// smaller device.
fn overhanging_sheet() -> (Rect, Rect) {
    let placed = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1000.0, 800.0));
    let printable = Rect::from_min_size(egui::pos2(-50.0, -50.0), egui::vec2(850.0, 750.0));
    (placed, printable)
}

/// A mask over a `size x size` page with the given pixel rectangles inked.
fn mask_with(size: u32, ink: &[(u32, u32, u32, u32)]) -> super::ink::InkMask {
    let mut data = vec![255u8; (size * size * 4) as usize];
    for &(x0, y0, x1, y1) in ink {
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y * size + x) * 4) as usize;
                data[i] = 0;
                data[i + 1] = 0;
                data[i + 2] = 0;
            }
        }
    }
    super::ink::InkMask::from_rgba_premultiplied(size, size, &data)
}

/// ★★★ **The operator's case, end to end: nothing hatched AND the caption
/// says so.** Operator request O113.
///
/// The page overhangs on both axes — the placement reports a clip and the
/// job-wide count will still name this sheet — but the overhanging bands
/// are empty paper. Nothing may be drawn, and the verdict must be
/// `BlankBand` so [`column`] can print the sentence that stops the count
/// and the picture contradicting each other.
///
/// The two halves are asserted **together**, in one call, because the whole
/// point of [`lost_regions`] returning both is that they cannot disagree.
#[test]
fn a_blank_overhang_hatches_nothing_and_says_so() {
    let (placed, printable) = overhanging_sheet();
    // Ink confined to the top-left 60% of the page: well inside the
    // printable area on both axes.
    let mask = mask_with(400, &[(10, 10, 240, 240)]);
    let (lost, verdict) = lost_regions(placed, printable, Some(&mask));
    assert!(
        lost.is_empty(),
        "the overhang is empty paper, so NOTHING may be hatched; got {lost:?}"
    );
    assert_eq!(
        verdict,
        Overhang::BlankBand,
        "with nothing hatched the caption must say the overhang is blank, or the \
         job-wide clip count sits above a picture that contradicts it"
    );
}

/// ★ **A mark out in the border is hatched, and the hatch is a small part
/// of the band rather than the whole of it.**
///
/// The distinction the old code could not make. Before O113 this case and
/// the one above drew the identical full-height red band; the assertion on
/// the hatched area is what separates them.
#[test]
fn an_inked_overhang_hatches_only_the_ink_and_says_so() {
    let (placed, printable) = overhanging_sheet();
    // The page is 400 px in the raster and 1000 pt on screen. The printable
    // area's right edge is at 800 pt = 0.8 of the page = pixel 320. A small
    // mark at pixels 340..350 x 40..50 is out in the right-hand border.
    let mask = mask_with(400, &[(10, 10, 240, 240), (340, 40, 350, 50)]);
    let (lost, verdict) = lost_regions(placed, printable, Some(&mask));
    assert_eq!(verdict, Overhang::Losing);
    assert_eq!(
        lost.len(),
        1,
        "only the right-hand band carries ink: {lost:?}"
    );

    let band_area = 200.0 * 800.0; // the whole right-hand overhang
    let hatched = lost[0].width() * lost[0].height();
    assert!(
        hatched < band_area * 0.05,
        "the hatch covers {hatched} pt² of a {band_area} pt² band. Before O113 it \
         covered all of it; hatching most of it again would be the defect back"
    );
    // And it is over the mark. The two axes have DIFFERENT scales — the
    // page is 400 px square in the raster but 1000 x 800 pt on screen — so
    // the expected spans are computed per axis rather than shared, which is
    // the mistake this test made on its first run.
    //   x: pixels 340..350 of 400 -> 0.850..0.875 of the page -> 850..875 pt
    //   y: pixels  40..50  of 400 -> 0.100..0.125 of the page ->  80..100 pt
    assert!(
        lost[0].min.x <= 850.0 && lost[0].max.x >= 875.0,
        "the hatch {:?} does not cover the mark's x span 850..875",
        lost[0]
    );
    assert!(
        lost[0].min.y <= 80.0 && lost[0].max.y >= 100.0,
        "the hatch {:?} does not cover the mark's y span 80..100",
        lost[0]
    );
}

/// ★★ **A failed render hatches the WHOLE band and says `Unknown`.**
///
/// The degraded state [`texture_for`] documents. "We could not look" must
/// not present as "nothing is lost" — a missing raster is not allowed to
/// switch a warning off, so the fallback is the pre-O113 behaviour.
#[test]
fn no_raster_falls_back_to_the_whole_band_rather_than_to_silence() {
    let (placed, printable) = overhanging_sheet();
    let (lost, verdict) = lost_regions(placed, printable, None);
    assert_eq!(verdict, Overhang::Unknown);
    assert_eq!(lost.len(), 2, "both overhangs, whole: {lost:?}");
    let total: f32 = lost.iter().map(|r| r.width() * r.height()).sum();
    assert!(
        total > 200.0 * 800.0,
        "the fallback must cover at least the full right-hand band; got {total} pt²"
    );
}

/// ★★ **The two bands are disjoint**, which fixes a second over-hatch that
/// was hiding inside the first.
///
/// The old code took `right.union(bottom)`, and `Rect::union` is a
/// **bounding box**, not a set union: the union of a tall strip on the
/// right and a wide strip along the bottom also covers the region that is
/// neither right of nor below the printable area — paper that prints
/// perfectly. This pins that the two bands now meet without overlapping,
/// and that neither reaches back into the printable rectangle.
#[test]
fn the_two_overhang_bands_do_not_overlap_or_reach_into_the_printable_area() {
    let (placed, printable) = overhanging_sheet();
    let (lost, _) = lost_regions(placed, printable, None);
    let [right, bottom] = [lost[0], lost[1]];
    assert!(
        !right.intersect(bottom).is_positive(),
        "the right band {right:?} and the bottom band {bottom:?} overlap, so the \
         shared corner would be hatched twice and read as a darker patch"
    );
    for band in lost {
        assert!(
            band.min.x >= printable.max.x - 1e-3 || band.min.y >= printable.max.y - 1e-3,
            "band {band:?} starts inside the printable area — a warning drawn over \
             paper that will print"
        );
    }
}

/// A degenerate placed rectangle — a page at zero scale, which a nonsense
/// `/MediaBox` can produce — yields a rectangle the mask rejects rather
/// than a `NaN` that would propagate into the hatch geometry and paint a
/// line across the whole canvas.
#[test]
fn a_zero_sized_page_maps_to_nothing_rather_than_to_nan() {
    let placed = Rect::from_min_size(egui::pos2(50.0, 50.0), egui::vec2(0.0, 0.0));
    let fraction = normalised_in(
        Rect::from_min_size(egui::pos2(50.0, 50.0), egui::vec2(10.0, 10.0)),
        placed,
    );
    assert_eq!(fraction, Rect::NOTHING);
    assert_eq!(
        super::ink::InkMask::from_rgba_premultiplied(4, 4, &[0u8; 64]).ink_extent(fraction),
        None,
        "a degenerate page must produce no hatch at all"
    );
}

/// ★ The preview reads pixels as premultiplied, exactly as the canvas does.
///
/// The same fixture `render::raster`'s own test uses — a half-transparent
/// red pixel stored the way `tiny-skia` stores it (`R·A, G·A, B·A, A`).
/// Read as *unmultiplied*, epaint would take the red channel at face value
/// and re-multiply it, yielding `r = 64`; read as premultiplied it
/// round-trips.
///
/// This test exists because [`upload`] is a **second** call site for a
/// convention that module says must have exactly one. Until that is fixed
/// there, this is what stops the two drifting silently — and the failure
/// mode being defended against is not a crash but every antialiased glyph
/// edge in the preview quietly darkening.
#[test]
fn the_preview_upload_reads_pixels_as_premultiplied() {
    let image = premultiplied_image(1, 1, &[128, 0, 0, 128]);
    assert_eq!(image.size, [1, 1]);
    let px = image.pixels[0];
    assert_eq!((px.r(), px.g(), px.b(), px.a()), (128, 0, 0, 128));
}
