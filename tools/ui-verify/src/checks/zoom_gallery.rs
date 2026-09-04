//! `the_page_still_renders_at_every_decade_of_zoom` — pixels on the screen,
//! not numbers in a trace.
//!
//! # The request
//!
//! The operator, 2026-08-22:
//!
//! > *"Can you confirm that rendering on screen is actually happening at
//! > maximum zoom? zoom in on one of the michocondria structures and post
//! > screenshots here to confirm. start with the full page first to confirm it
//! > renders."*
//!
//! # ★★★ Why the existing checks do not answer this
//!
//! `zooming_does_not_throw_away_where_the_operator_panned` proves the view
//! stays where it was put to a trillion percent, and
//! `zooming_past_the_pixmap_ceiling_still_renders` proves no raster is refused.
//! **Neither looks at the screen.** A canvas could satisfy both while drawing a
//! blank sheet: the position arithmetic would be perfect and the rasters would
//! complete, and the operator would see nothing.
//!
//! That is not a hypothetical gap. `D:/dev/rag/egui/` records panels that
//! shipped unreachable in real builds with every gate green, and the rule it
//! draws from them is that layout and clipping defects have exactly one oracle:
//! a rendered screenshot. This check is that oracle for deep zoom.
//!
//! # What it does
//!
//! Opens the document, captures the window, then climbs by Ctrl+wheel with the
//! pointer parked on a target that is given in **document coordinates**, so the
//! same run works whatever the window size. It captures again at each decade.
//!
//! At every step it asserts three things, and each rules out a different way of
//! being wrong:
//!
//! | assertion | rules out |
//! |---|---|
//! | the capture is not near-uniform | a blank or white canvas |
//! | the canvas traced `drawn ≥ 1` | a page reserving space with no raster in it |
//! | no `outcome=failed` render | a refused rasterization the shell swallowed |
//!
//! ★ All three, because any two can hold while the third fails. A page that
//! draws its *state message* is not near-uniform; a page whose raster completed
//! can still be drawn off-screen; a shell that stopped asking cannot report a
//! failure.
//!
//! # ★ The captures are evidence, kept on pass as well as fail
//!
//! Written to the output directory and named by zoom, because the question
//! being answered is *"show me"* and the answer is a file the operator can
//! open. On a later failure they are also the only thing to compare against.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// `VK_CONTROL`, held while the wheel rolls to make it a zoom.
const VK_CONTROL: u16 = 0x11;

/// The canvas viewport, whose declared rect aims the pointer.
const CANVAS_REGION: &str = "canvas-viewport";

/// The canvas's own state line.
const CANVAS_EVENT: &str = "canvas";

/// The worker's completion line.
const RENDER_EVENT: &str = "render-async-done";

/// The `f64` position line — the only one that can aim a pointer at the top of
/// the range. See [`on_screen_now`].
const POS_EVENT: &str = "canvas-pos";

/// The zooms to photograph, as multipliers.
///
/// # ★★ Chosen against what the fixture actually contains
///
/// `banana.pdf`'s own generator prints the scale chain, and these are its
/// tiers rather than round numbers picked for the look of them:
///
/// | zoom | what becomes visible |
/// |---|---|
/// | 1 × | the banana, at life size |
/// | 20 × | the two cell outlines |
/// | 120 × | cell labels, starch grains |
/// | 450 × | organelle labels |
/// | 4,000 × | chloroplast grana, plasmodesmata |
/// | 26,000 × | mitochondrial cristae |
/// | 350,000 × | ATP synthase heads — the 10 nm features |
/// | 10,000,000,000 × | the configured ceiling |
///
/// ★ The last rung is not a feature tier. It is there because the operator
/// asked whether rendering *still happens* at the maximum, and a gallery that
/// stopped where the detail stops would not have answered him.
const TIERS: &[f32] = &[
    1.0,
    20.0,
    120.0,
    450.0,
    4_000.0,
    26_000.0,
    350_000.0,
    // ★ The decade between the fixture tiers and the ceiling. Added when the
    // gallery was pointed at a molecule drawn at true scale: a benzene ring is
    // 1.4e-6 pt across, so it frames a window at about 1.4e8x, and jumping
    // 350,000x -> 1e10x skipped straight past the zoom at which the subject is
    // actually the size of the screen. The intermediate rungs cost seconds and
    // are where the interesting pictures are.
    3_500_000.0,
    35_000_000.0,
    350_000_000.0,
    3_500_000_000.0,
    10_000_000_000.0,
];

/// How many notches to roll before re-checking against the target.
///
/// ★ A small batch rather than a computed count. A Ctrl+wheel notch multiplies
/// by about 1.22, but the ladder's rungs are not a pure geometric series near
/// the bottom, so the number of notches to reach a given zoom is not something
/// a check should predict. It rolls, reads what the application says, and stops
/// when it is there — which is also what makes it survive a change to the
/// ladder.
const BATCH: usize = 2;

/// The most batches to roll toward any one tier before giving up.
///
/// A backstop against a build whose zoom does not climb; reaching it is a SKIP,
/// never a pass.
const MAX_BATCHES: usize = 400;

/// See the module documentation.
pub struct ThePageStillRendersAtEveryDecadeOfZoom;

impl Check for ThePageStillRendersAtEveryDecadeOfZoom {
    fn name(&self) -> &'static str {
        "the_page_still_renders_at_every_decade_of_zoom"
    }

    fn defect(&self) -> &'static str {
        "the canvas stops actually drawing the page at high zoom — the position arithmetic and \
         the rasterizer both report success while the operator is looking at blank paper"
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

/// The zoom the canvas last reported, as a multiplier.
fn zoom_now(session: &Session) -> Result<f32> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0))
}

/// Where a page point is on screen **right now**, from the `f64` position line.
///
/// # ★★★ Why the climb has to re-aim, and why `CanvasMapping` cannot do it
///
/// Zoom-to-cursor holds the point under the pointer, and it holds it to about
/// half a per-notch tolerance — which is excellent per notch and still
/// accumulates over the hundred-odd notches it takes to reach the ceiling. Left
/// alone, the climb wanders off a 3 µm mitochondrion long before it gets there
/// and photographs blank cytoplasm. That is not a rendering defect; it is what
/// zooming a hundred times without looking does, in any application.
///
/// So the pointer is re-aimed at the target between tiers, which is what a
/// person does — magnify, see it drifting, nudge.
///
/// ★★ `CanvasMapping` cannot be used for it. That converts through the `canvas`
/// line's `rect=`, an `f32` whose magnitude at a trillion percent is 5 × 10¹²
/// where the representable spacing is half a million pixels — so it would aim
/// the pointer anywhere within half a million points of the target. The
/// `canvas-pos` line carries the same geometry in `f64`:
///
/// ```text
/// screen.x = viewport.min.x + (page.x × zoom − at.x)
/// ```
///
/// with `at` the pan distance from the page's corner. Both terms are around
/// 5 × 10¹⁴ at the ceiling and their difference is a few hundred, which `f64`
/// resolves to a hundredth of a point. ★ The subtraction is done in `f64` and
/// only the small result is narrowed — the same technique, and the same reason,
/// as `render::region::region_on_screen_deep`.
fn on_screen_now(
    session: &Session,
    canvas: crate::geom::LRect,
    target: crate::coords::DocPoint,
) -> Result<Option<(f64, f64)>> {
    let trace = session.trace()?;
    let Some(zoom) = trace
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
    else {
        return Ok(None);
    };
    let Some(line) = trace.events(POS_EVENT).last() else {
        return Ok(None);
    };
    let pair = |k: &str| -> Option<(f64, f64)> {
        let (x, y) = line.get(k)?.split_once(',')?;
        Some((x.parse().ok()?, y.parse().ok()?))
    };
    let (Some(at), Some(ext)) = (pair("at"), pair("ext")) else {
        return Ok(None);
    };
    let z = f64::from(zoom);
    // Page space is y-up and the canvas is y-down, so the target's canvas y is
    // measured from the top of the sheet.
    let canvas_y = ext.1 - target.y;
    Ok(Some((
        f64::from(canvas.min.x) + (target.x * z - at.0),
        f64::from(canvas.min.y) + (canvas_y * z - at.1),
    )))
}

/// How many of the visible pages had a raster on the last reported frame.
fn drawn_now(session: &Session) -> Result<u32> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get("drawn"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0))
}

/// Capture the window and assert it shows a drawn page.
///
/// Returns `Ok(Some(verdict))` when it does not — the three ways of being wrong
/// in this module's header.
fn photograph(
    ctx: &CheckContext,
    session: &Session,
    frame: &crate::coords::WindowFrame,
    canvas: crate::geom::LRect,
    report: &mut CheckReport,
    label: &str,
) -> Result<Option<String>> {
    let zoom = zoom_now(session)?;
    let path = ctx.out(&format!("zoom-gallery-{label}.png"));

    // ★ `capture::window_to_png` refuses a near-uniform grab itself, with the
    // three causes it has actually seen. Its refusal is an Error and therefore
    // a SKIP — right for "the display is asleep", wrong for "the canvas is
    // blank", so the uniformity verdict is re-stated as a FAILURE below rather
    // than left to it.
    let image = match crate::capture::window_to_png(session, &path) {
        Ok(image) => image,
        Err(e) => {
            return Ok(Some(format!(
                "at {:.0}% the window capture was rejected: {e}\n\nIf the display is awake and \
                 the window was raised, this is the answer to the question — the canvas is not \
                 drawing anything at this zoom.",
                zoom * 100.0
            )));
        }
    };
    report.artifact(path.clone());

    // ★★★ UNIFORMITY IS ASKED OF THE CANVAS, NOT OF THE WINDOW.
    //
    // `capture::window_to_png` refuses a near-uniform *window*, and a window
    // always contains a ribbon, a status bar and two panels — so it is never
    // uniform, and its guard can never fire for the reason this check cares
    // about. The first run of this gallery passed a screenshot of blank white
    // paper on exactly that technicality.
    //
    // Same shape as `deep_pan`'s `REGION_TIER_REQUIRED` and one hour older: an
    // assertion aimed at the wrong surface reads as green and has measured
    // nothing. The region is the declared canvas viewport, converted to capture
    // pixels, so what is sampled is the page and only the page.
    let region = frame.logical_to_capture_pixels(canvas);
    let uniformity = crate::pixels::region_not_uniform(&image, region);
    if uniformity.is_uniform() {
        return Ok(Some(format!(
            "at {:.0}% the CANVAS is near-uniform ({}) — the page is not being drawn, \
             whatever the rest of the window shows. The capture is at {}.",
            zoom * 100.0,
            uniformity.summary(),
            path.display()
        )));
    }

    let drawn = drawn_now(session)?;
    if drawn == 0 {
        return Ok(Some(format!(
            "at {:.0}% the canvas reported `drawn=0` — space was reserved for the page and no \
             raster was painted into it. The capture is at {}, and it will show the page's state \
             message rather than the page.",
            zoom * 100.0,
            path.display()
        )));
    }

    let failures = session
        .trace()?
        .events(RENDER_EVENT)
        .filter(|l| l.get("outcome").is_some_and(|o| o == "failed"))
        .count();
    if failures > 0 {
        return Ok(Some(format!(
            "at {:.0}% there have been {failures} failed render(s). A raster the renderer refused \
             is a part of the page the operator cannot see, whatever else is on screen.",
            zoom * 100.0
        )));
    }

    report.note(format!(
        "{label}: {:.0}% — drawn={drawn}, canvas shows {} distinct tone(s), capture at {}",
        zoom * 100.0,
        uniformity.distinct,
        path.display()
    ));
    Ok(None)
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
        .ok_or_else(|| Error::new("no --pdf. There is nothing to photograph."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs the page coordinate to magnify toward — on \
             `banana.pdf` a mitochondrion in the pulp cell is at 0,539.9717,560.3515. Without it \
             the climb would centre on whatever happens to be in the middle of the sheet, which \
             on that fixture is blank paper.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check zooms the canvas. Reported as SKIPPED \
             rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("zoom-gallery.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    // The canvas's own rect — not used to AIM (the pointer goes to a document
    // coordinate below) but to say which pixels of the capture are the page,
    // which is what makes the uniformity assertion mean anything.
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;

    // ★★ The pointer is aimed ONCE, in document coordinates, and then left
    // there. Ctrl+wheel is zoom-about-the-pointer, so the target stays under it
    // for the whole climb — which is both what makes the gallery show the same
    // structure at every magnification and what removes any need to re-aim as
    // the mapping changes underneath.
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let at =
        frame.to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    driver.move_to(at)?;
    report.note(format!(
        "aiming at page {} ({}, {})",
        target.page, target.x, target.y
    ));

    let mut at = at;
    for &tier in TIERS {
        // Roll until the application says it is at or past this tier. Asking it
        // rather than counting notches is what makes this survive a change to
        // the zoom ladder — and what lets the last rung mean "the ceiling,
        // wherever the operator has set it" rather than a number written here.
        let mut batches = 0;
        loop {
            let now = zoom_now(&session)?;
            if now >= tier || batches >= MAX_BATCHES {
                break;
            }
            let before = now;
            driver.scroll_at_held(at, &[VK_CONTROL], 1, BATCH)?;
            session.settle(4);
            batches += 1;
            // Saturated below the tier: the operator's maximum-zoom setting is
            // lower than this rung. Not a defect — stop climbing and report.
            if batches > 2 && zoom_now(&session)? <= before {
                report.note(format!(
                    "stopped short of {:.0}%: the application saturated at {:.0}%, which is its \
                     configured maximum",
                    tier * 100.0,
                    before * 100.0
                ));
                break;
            }
        }
        if batches >= MAX_BATCHES {
            return Err(Error::new(format!(
                "after {MAX_BATCHES} batches the zoom was still below {:.0}% (it is {:.0}%). \
                 Either Ctrl+wheel lost its Ctrl and panned instead, or the ladder is refusing to \
                 step. SKIPPED rather than passed.",
                tier * 100.0,
                zoom_now(&session)? * 100.0
            )));
        }

        // ★★ RE-AIM. Zoom-to-cursor holds the point to about half a per-notch
        // tolerance, which accumulates over the hundred-odd notches this climb
        // takes; without this the run wanders off a 3 µm mitochondrion and
        // photographs blank cytoplasm. See `on_screen_now`.
        if let Some((sx, sy)) = on_screen_now(&session, canvas, target)?
            && sx.is_finite()
            && sy.is_finite()
        {
            let (wx, wy) = (sx as f32, sy as f32);
            // ★ Only if the target is still ON the canvas. Off it, the pointer
            // would leave the page — and Ctrl+wheel outside the canvas is not a
            // zoom at all, so the climb would stop dead rather than merely
            // drift. A margin, so it is never on the very edge.
            let m = 8.0;
            if wx > canvas.min.x + m
                && wx < canvas.max.x - m
                && wy > canvas.min.y + m
                && wy < canvas.max.y - m
            {
                // ★ Expressed as fractions of the canvas rather than built as
                // a `WindowPoint`. That type's fields are private on purpose —
                // `coords`' whole enforcement mechanism is that a screen point
                // can only come from a trace-derived mapping — and
                // `declared_at` is the sanctioned way in.
                let fx = (wx - canvas.min.x) / (canvas.max.x - canvas.min.x);
                let fy = (wy - canvas.min.y) / (canvas.max.y - canvas.min.y);
                at = frame.declared_at(canvas, fx, fy);
                driver.move_to(at)?;
            }
        }

        // Let the raster for this depth actually arrive. ★ Generous, and the
        // reason is in `deep_zoom`'s own note: a settle that is too short does
        // not fail the check, it makes its evidence ambiguous — `drawn=0` from
        // a render still in flight is indistinguishable from a page that cannot
        // be drawn at all, and this check's whole product is the evidence.
        session.settle(200);

        let now = zoom_now(&session)?;
        let label = format!("{:013.0}", now * 100.0);
        if let Some(bad) = photograph(ctx, &session, &frame, canvas, report, &label)? {
            return Ok(Some(bad));
        }
        if now < tier {
            // Saturated; every remaining tier is above the ceiling.
            break;
        }
    }

    Ok(None)
}
