//! `panning_at_deep_zoom_stays_where_it_was_put` — the operator's "it jumps
//! back" report, made falsifiable.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/deep_pan.md`.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// `VK_CONTROL`, held while the wheel rolls to make it a zoom.
const VK_CONTROL: u16 = 0x11;

/// **Where on the page to zoom INTO**, in PDF user-space points.
const CELLS_PT: (f64, f64) = (540.0, 560.0);

/// The zoom group on the status bar — its right end is `+`.
const ZOOM_REGION: &str = "status-group:zoom";

/// The canvas viewport, which the drag happens inside.
const CANVAS_REGION: &str = "canvas-viewport";

/// The canvas's own state line, read for the zoom it reports.
const CANVAS_EVENT: &str = "canvas";

/// The canvas's `f64` pan position — the only field that can measure this.
const POS_EVENT: &str = "canvas-pos";

/// How far to zoom in before the FIRST probe.
const PRESSES: usize = 20;

/// Refuse to PASS unless a region raster was actually placed.
const REGION_TIER_REQUIRED: bool = true;

/// How many MORE presses before the second probe.
const MORE_PRESSES: usize = 110;

/// How many wheel notches to roll.
const NOTCHES: i32 = -3;

/// How many separate wheel rolls to make, sampling the placement after each.
const ROLLS: usize = 8;

/// The page's own rect, on the `ui-rect` channel.
const PAGE_REGION: &str = "page";

/// How far a raster may sit from where its own region says it belongs, in
/// window logical points, before this is a defect.
const PLACEMENT_TOLERANCE_PT: f64 = 2.0;

/// See the module documentation.
pub struct PanningAtDeepZoomStaysWhereItWasPut;

impl Check for PanningAtDeepZoomStaysWhereItWasPut {
    fn name(&self) -> &'static str {
        "panning_at_deep_zoom_stays_where_it_was_put"
    }

    fn defect(&self) -> &'static str {
        "panning a little at high zoom does nothing, or moves and then jumps back to where it \
         started — so a point just off screen cannot be reached at all"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// One reading of the canvas's `f64` position line.
#[derive(Debug, Clone)]
struct Pos {
    /// How far the view has been panned from the acting page's corner, in
    /// screen pixels.
    at: (f64, f64),
    /// `scroll` or `deep` — which mechanism owns the position.
    tier: String,
    /// Where the raster was actually painted, window logical points.
    paint: (f64, f64),
    /// The page-space rect that raster is a picture of, or `None` for a
    /// whole-page raster.
    region: Option<(f64, f64, f64, f64)>,
    /// The page's extent, same units as `region`.
    ext: (f64, f64),
}

/// The canvas's latest `f64` position line, parsed, **with the page rect from
/// the same reading**.
fn position(session: &Session, ui_rect: &str) -> Result<Option<(Pos, crate::geom::LRect)>> {
    let trace = session.trace()?;
    let Some(page_rect) = driving::declared(&trace, ui_rect, PAGE_REGION) else {
        return Ok(None);
    };
    Ok(trace.events(POS_EVENT).last().and_then(|l| {
        let pair = |k: &str| -> Option<(f64, f64)> {
            let (x, y) = l.get(k)?.split_once(',')?;
            Some((x.parse().ok()?, y.parse().ok()?))
        };
        let region = match l.get("region") {
            Some("none") | None => None,
            Some(v) => {
                let n: Vec<f64> = v.split(',').filter_map(|p| p.parse().ok()).collect();
                if n.len() == 4 {
                    Some((n[0], n[1], n[2], n[3]))
                } else {
                    return None;
                }
            }
        };
        Some((
            Pos {
                at: pair("at")?,
                tier: l.get("tier").unwrap_or("?").to_owned(),
                paint: pair("paint")?,
                region,
                ext: pair("ext")?,
            },
            page_rect,
        ))
    }))
}

/// **The placement invariant, recomputed independently.**
///
/// The pixels on screen must be a picture of the page area they cover. The
/// application places a region raster with `render::region::region_on_screen`,
/// which is
///
/// ```text
/// screen.x = page_rect.min.x + region.llx / extent.x * page_rect.width()
/// screen.y = page_rect.min.y + (extent.y - region.ury) / extent.y * page_rect.height()
/// ```
///
/// — the y term flipped because page space is y-up and the canvas is y-down.
/// This recomputes it here, from the traced region and the page's own rect,
/// and compares against the traced paint rect.
///
/// It is an **independent** check rather than a restatement because the region
/// it reads is the one the *held texture* is a picture of. `OPERATOR_REQUESTS`
/// O24c was the placement being computed from the region the shell wanted
/// *next* instead, which differs exactly while a new raster is in flight —
/// most of the time during a pan. Under that defect the traced region still
/// describes the pixels, this formula still says where they belong, and the
/// two disagree by a whole grid step.
///
/// Returns the disagreement in window logical points.
fn placement_error(pos: &Pos, page_rect: crate::geom::LRect) -> Option<(f64, f64)> {
    let (llx, _lly, _urx, ury) = pos.region?;
    if pos.ext.0 <= 0.0 || pos.ext.1 <= 0.0 {
        return None;
    }
    let w = f64::from(page_rect.max.x - page_rect.min.x);
    let h = f64::from(page_rect.max.y - page_rect.min.y);
    let want_x = f64::from(page_rect.min.x) + llx / pos.ext.0 * w;
    let want_y = f64::from(page_rect.min.y) + (pos.ext.1 - ury) / pos.ext.1 * h;
    Some((pos.paint.0 - want_x, pos.paint.1 - want_y))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. There is nothing to pan."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check zooms in and drags the canvas. Reported \
             as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("deep-pan.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    // The fixture's page box, for the document→window conversion that aims the
    // zoom at the cells. `--page-size` overrides it for a fixture whose
    // `/MediaBox` this harness cannot read.
    let page: crate::coords::PageGeometry = match ctx.page_size {
        Some((w, h)) => crate::coords::PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}, so the cells cannot be aimed at. Pass                  --page-size WxH.",
                pdf.display()
            ))
        })?,
    };
    // Asserted present but not clicked. The zoom is driven by Ctrl+wheel at
    // the cells (see `CELLS_PT`); this is the check that a document is open at
    // all, and its absence is a far clearer failure than a mapping built from a
    // canvas that was never laid out.
    driving::declared(&trace, ui_rect, ZOOM_REGION)
        .ok_or_else(|| Error::new(format!("no `{ZOOM_REGION}`; is a document open?")))?;
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;

    // ZOOM AT THE CELLS, not at the middle of the sheet.
    //
    // Ctrl+wheel is zoom-about-the-pointer, so the point under the cursor stays
    // put: aim once and every subsequent notch magnifies the same content. The
    // `+` button zooms about the viewport centre, which on this page is blank
    // paper. See `CELLS_PT`.
    let mapping = crate::coords::CanvasMapping::from_trace(&trace, vocab, page, 0)?;
    let at_cells = frame
        .to_screen(mapping.doc_to_window(crate::coords::DocPoint::new(0, CELLS_PT.0, CELLS_PT.1))?);
    report.note(format!(
        "zooming at the cells — page ({}, {})",
        CELLS_PT.0, CELLS_PT.1
    ));
    driver.scroll_at_held(at_cells, &[VK_CONTROL], 1, PRESSES)?;
    session.settle(60);

    let zoom = session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0);
    report.note(format!("zoomed to {:.0}%", zoom * 100.0));

    // Whether any reading, at any tier, described a REGION raster. See
    // `REGION_TIER_REQUIRED` — a run in which this stays false has not
    // exercised the thing this check is named after and must not report PASS.
    let mut saw_region = false;

    if let Some(bad) = probe(
        &session,
        &driver,
        &frame,
        canvas,
        ui_rect,
        &mut saw_region,
        report,
    )? {
        return Ok(Some(bad));
    }

    if REGION_TIER_REQUIRED && !saw_region {
        return Err(Error::new(
            "no reading described a REGION raster — every one said `region=none`, so the whole \
             page was being rasterized throughout, the placement cross-check had nothing to \
             compare, and the run never reached the tier this check is named after. \
             The region tier engages above MAX_PIXMAP_EDGE / page_height, which is about \
             2,070 % on a US Letter sheet. Raise PRESSES until the first probe lands above it, \
             or drive a larger page, whose threshold is lower. \
             ★ Reported as SKIPPED rather than passed, and this is not pedantry: with PRESSES \
             at 16 the first probe landed at 1,867 % and this check reported PASS twice against \
             a binary that had O24c deliberately put back in.",
        ));
    }

    // …and again on the other side of the deep threshold, where a different
    // mechanism owns the position entirely. See `MORE_PRESSES`.
    driver.scroll_at_held(at_cells, &[VK_CONTROL], 1, MORE_PRESSES)?;
    session.settle(120);
    if let Some(bad) = probe(
        &session,
        &driver,
        &frame,
        canvas,
        ui_rect,
        &mut saw_region,
        report,
    )? {
        return Ok(Some(bad));
    }
    Ok(None)
}

/// Roll the wheel once over the canvas and report what the view did.
fn probe(
    session: &Session,
    driver: &Driver,
    frame: &crate::coords::WindowFrame,
    canvas: crate::geom::LRect,
    ui_rect: &str,
    saw_region: &mut bool,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    let zoom = session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0);

    let Some((first, rect_first)) = position(session, ui_rect)? else {
        return Err(Error::new(
            "the canvas never reported a `canvas-pos` line, so there is nothing to compare. \
             Either the build predates the f64 position trace or no page was drawn. SKIPPED.",
        ));
    };

    // The placement is checked BEFORE the wheel as well as after, and this
    // ordering matters. When the defect was re-introduced deliberately to
    // falsify this check, the movement assertion below happened to trip first
    // and the run reported a stuck view — a true failure for the wrong reason,
    // which would have sent the next reader to the scroll offset instead of to
    // the paint rectangle. A check that can only fail one way at a time should
    // fail in the order that names the cause.
    if let Some(bad) = check_placement("before", &first, rect_first, zoom) {
        return Ok(Some(bad));
    }

    let (before, tier) = (first.at, first.tier.clone());

    // ROLL REPEATEDLY, SAMPLING THE PLACEMENT AFTER EACH.
    //
    // One roll cannot cross `render::strategy::region_for`'s half-viewport
    // grid, so it cannot reproduce O24c. See `ROLLS`.
    for roll in 0..ROLLS {
        driver.scroll_at(frame.declared_at(canvas, 0.5, 0.5), NOTCHES)?;
        // TWO FRAMES, and the shortness is the point: this reading must land
        // INSIDE the raster transient. O24c only exists while a new cell's
        // raster is in flight; once it lands, the held region and the wanted
        // region agree again and the placement is correct on a broken build as
        // much as on a fixed one. A twenty-frame wait here walked straight past
        // a defect that was deliberately present in the binary being driven.
        session.settle(2);
        let Some((flight, rect_flight)) = position(session, ui_rect)? else {
            return Err(Error::new(
                "the canvas stopped reporting a position. SKIPPED.",
            ));
        };
        *saw_region |= flight.region.is_some();
        if let Some(bad) = check_placement(&format!("mid-roll {roll}"), &flight, rect_flight, zoom)
        {
            return Ok(Some(bad));
        }
    }

    // A SECOND reading, far enough out for the SCROLL to have been applied.
    //
    // The two answer different questions and cannot share a moment. `flight`
    // above must land inside the raster transient or the placement defect has
    // already healed; this one must land after the view has actually moved, or
    // the movement assertion reports a stuck view on a build that panned
    // perfectly — which is what two frames produced on two of four runs when
    // they were asked to do both jobs. A check that passes on two runs in four
    // has not measured anything.
    session.settle(20);
    let Some((mid, rect_mid)) = position(session, ui_rect)? else {
        return Err(Error::new(
            "the canvas stopped reporting a position. SKIPPED.",
        ));
    };

    // And again several frames later. A view that moves and is then put back
    // is a different defect from one that never moved, and only a second
    // reading can tell them apart — the operator described both in one
    // sentence, so the check must be able to say which he saw.
    session.settle(90);
    let Some((last, rect_last)) = position(session, ui_rect)? else {
        return Err(Error::new(
            "the canvas stopped reporting a position. SKIPPED.",
        ));
    };

    let (after, settled) = (mid.at, last.at);
    report.note(format!(
        "at {:.0}% on tier `{tier}`: {before:?} → {after:?} → {settled:?}",
        zoom * 100.0
    ));

    let moved = (after.0 - before.0).abs() + (after.1 - before.1).abs();
    // One notch is tens of pixels on every platform this runs on; a build that
    // moved by less than a single pixel did not move.
    if moved < 1.0 {
        return Ok(Some(format!(
            "★★ THE VIEW DID NOT MOVE. {NOTCHES} wheel notches at {:.0}% zoom left the position \
             at {before:?} (tier `{tier}`, moved {moved:.3} px). This is the operator's \"I \
             couldn't pan to the next point\". On the `scroll` tier it means the `f32` offset's \
             representable spacing exceeds the movement, so `last - pan` rounds back to `last`; \
             on the `deep` tier it means the wheel is not reaching `DeepAnchor::panned`.",
            zoom * 100.0
        )));
    }

    let reverted = (settled.0 - before.0).abs() + (settled.1 - before.1).abs();
    if reverted < moved / 2.0 {
        return Ok(Some(format!(
            "★★ THE VIEW REVERTED. The wheel moved the position from {before:?} to {after:?} \
             at {:.0}% on tier `{tier}`, and ninety frames later it is back at {settled:?}. This \
             is the operator's \"it would jump back to its original location\" — something is \
             re-setting the position after the gesture. Read what forces one every frame: \
             `zoom::consume_anchor`, `find::take_reveal_offset` and `strip::page_scroll_offset`, \
             and the last of those fires whenever the current-page reading disagrees with the \
             tracked one.",
            zoom * 100.0
        )));
    }

    for (when, pos, page_rect) in [("after", &mid, rect_mid), ("settled", &last, rect_last)] {
        *saw_region |= pos.region.is_some();
        if let Some(bad) = check_placement(when, pos, page_rect, zoom) {
            return Ok(Some(bad));
        }
    }

    Ok(None)
}

/// O24c — THE PIXELS MUST BE A PICTURE OF WHERE THEY ARE DRAWN.
fn check_placement(
    when: &str,
    pos: &Pos,
    page_rect: crate::geom::LRect,
    zoom: f32,
) -> Option<String> {
    if pos.tier != "scroll" {
        return None;
    }
    let (ex, ey) = placement_error(pos, page_rect)?;
    if ex.abs() <= PLACEMENT_TOLERANCE_PT && ey.abs() <= PLACEMENT_TOLERANCE_PT {
        return None;
    }
    Some(format!(
        "★★ THE PIXELS ARE IN THE WRONG PLACE ({when} the wheel, at {:.0}%). The raster on \
         screen is a picture of page region {:?}, which belongs at the rect this harness \
         recomputed from the page's own rect — but it was painted {ex:.1},{ey:.1} points away \
         from there. This is the operator's \"if I pan a little too far it jumps back in the \
         opposite direction\": `canvas::show` is placing the held texture by the region the \
         shell wants NEXT rather than by the one those pixels are OF, and the two differ by \
         `render::strategy::region_for`'s half-viewport grid step whenever a raster is in \
         flight. See `render::worker::RenderKey::region`.",
        zoom * 100.0,
        pos.region
    ))
}
