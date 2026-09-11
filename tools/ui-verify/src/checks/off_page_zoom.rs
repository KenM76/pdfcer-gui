//! `an_object_off_the_page_survives_being_zoomed_in_on` — **O23's "edit"
//! half.**
//!
//! # The report
//!
//! Ken, 2026-09-11, `OPERATOR_REQUESTS.md` **O23**:
//!
//! > *"how do I view and edit objects that are off of the page? we added this
//! > feature but I didn't see how to enable it."*
//!
//! Three verbs, and each one broke on its own:
//!
//! | verb | check | what it proves |
//! |---|---|---|
//! | **reach** | `off_page_press` | a press in the grey becomes a gesture |
//! | **see** | `off_page_visible` | the object is painted at all |
//! | **edit** | **this file** | it is still there once you zoom in on it |
//!
//! # ★★★ Why "edit" is a separate property, and why the first two were green
//!
//! **Editing an object means zooming in on it.** Nobody nudges a `/Rect` by a
//! point at 100 %. And until 2026-09-11 zooming in on an off-page object *took
//! it away* — measured, not inferred.
//!
//! The pasteboard O23 shipped was `viewport × 1.0`: a fixed count of **screen
//! pixels**. The slice of the **drawing** that slack covers is therefore
//! `viewport / zoom`, and it shrinks with every notch. An object `k` points off
//! the sheet can be brought to the viewport's **edge** only while
//! `viewport ≥ k × zoom`, and to its **centre** only while
//! `viewport ≥ k × zoom + viewport / 2`. Past that, the offset solved by the
//! zoom anchor is thrown away by `canvas::geometry::strip_offset`'s clamp —
//! which clamps to `content_extent − viewport`, and `content_extent` is built
//! from that same fixed pasteboard.
//!
//! ⇒ **The operator zooms toward the object and it walks off the screen.** Both
//! sibling checks stay green throughout: they run at 100 %, where the fixed
//! pasteboard is enormous relative to the 100 pt of overhang. That is the shape
//! of this whole request — each verb green, the operator still stuck.
//!
//! The fix folds the *content's* overhang into the pasteboard.
//! `render::halo::overhang` measures it in page points, `canvas::present`
//! multiplies it by the zoom and publishes it once per frame onto
//! `OpenDoc::pasteboard_overhang`, and `canvas::geometry::pasteboard` takes
//! `max(viewport × FRACTION, overhang + viewport / 2)`. The `+ viewport / 2` is
//! the difference between *reaching* a point and *looking at* it.
//!
//! # ★★ The climb is calibrated against THIS window, not against a constant
//!
//! The old ceiling is a function of the viewport, so a check that zoomed to a
//! hard-coded 1000 % would have been a real test on a laptop and a vacuous one
//! on a wide monitor — the same silently-inert control this suite keeps
//! catching itself building. So the check **measures** the viewport, computes
//! the zoom at which the old pasteboard stopped reaching
//! ([`OFF_PTS`] × zoom > viewport), and climbs [`PAST_THE_OLD_CEILING`] beyond
//! it before it asserts anything.
//!
//! ★ It reports that number. A reader of a *passing* run can see how far past
//! the old limit it actually got, which is the difference between "green" and
//! "measured".
//!
//! # The oracle: ink above the edge, paper below it
//!
//! The anchor is the **midpoint of the off-page square's bottom edge** —
//! `(−100, 100)` on the fixture. After the climb the trace is re-read and that
//! doc point is converted afresh, so this check does **not** assert that the
//! zoom anchor held it under the pointer: `zoom_keeps_place` owns that
//! property, and one check covering both would go red for one reason while the
//! other was still broken.
//!
//! Two patches, at a fixed **logical-point** offset either side of where the
//! application says that edge now is:
//!
//! * **above** it on screen — inside the square, because page y grows up while
//!   screen y grows down — must be ink;
//! * **below** it on screen — off the sheet, inside the widened raster — must
//!   be paper.
//!
//! A fixed *screen* offset rather than a *page-point* one is what makes the
//! pair work at any magnification: the clearance from the edge stays constant
//! in the units antialiasing happens in, and shrinks to nothing in page points
//! exactly as fast as the zoom makes that irrelevant.
//! [`the_patches_straddle_the_edge_at_every_zoom_this_check_reaches`] pins the
//! arithmetic against the fixture's own content stream.
//!
//! ★★★ The **pair** is the point. If the halo is not painted, both patches read
//! canvas grey: the ink patch fails and the control passes, which is the
//! feature failing. If the probe is aimed at a panel or at the desktop, both
//! read dark: the control fires first and the check reports a **harness**
//! finding rather than an application one. A single-patch check cannot tell
//! those apart, and this suite has been wrong that way before.
//!
//! # ⚠ One deliberate non-skip: the conversion refusing
//!
//! `CanvasMapping::doc_to_window_off_page` returns an error when the point has
//! left the viewport. Everywhere else in this suite that is a SKIP. **Here it
//! is the defect itself** — "the object walked off the screen while I zoomed
//! toward it" is the literal complaint — so it is caught and reported as a
//! FAILURE carrying the arithmetic that predicts it.
//!
//! ★ The same call is made *before* the climb as well, and there it **is** a
//! SKIP, because a fixture that is already out of view at 100 % says nothing
//! about zooming. Same error, opposite verdict, decided by which side of the
//! climb it happened on.
//!
//! # Every way this reports SKIP
//!
//! No binary, no diagnostic channel, input disabled, no `canvas-viewport`
//! region, the anchor already off screen at 100 %, the canvas never settling,
//! or the zoom saturating before it reached the calibrated target. **Not** "the
//! object was not painted" and **not** "the point left the viewport after the
//! climb" — those are failures, and they are the two this check exists for.

use crate::checks::driving::{SHELL_DIAG_ENV, declared};
use crate::checks::zoom_keeps_place::{CANVAS_REGION, VK_CONTROL, settled};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Single-page display, then **100 %** — a property of the document rather than
/// of the window, so the climb starts from the same place on every machine.
///
/// ★★★ `mode.edit` is named FIRST, and it is not decoration. Since
/// 2026-09-11 the display of off-sheet content is a per-mode preference and
/// **Read ships with it OFF** — the operator's request: *"by default, read
/// doesn't show off page items, review and edit do show off page items."*
/// This check's whole subject is off the sheet, so without an explicit mode it
/// would run in whatever mode the shell opens in, find nothing, and report a
/// defect that is a correctly-implemented setting.
///
/// Edit rather than Review because that is the mode this check's gestures
/// belong in anyway, and because a mode named explicitly cannot drift when a
/// later session changes which mode the shell opens in.
const INVOKE: &str = "mode.edit,view.page_single,view.zoom_actual";

/// The fixture, relative to the workspace root. Shared with all three siblings,
/// because three fixtures for one property is three chances for one of them to
/// quietly stop having it.
const FIXTURE: &str = "fixtures/off-page-object.pdf";

/// Its page.
const FIXTURE_PAGE: PageGeometry = PageGeometry {
    width_pt: 200.0,
    height_pt: 200.0,
};

/// **The midpoint of the off-page square's bottom edge**, in page points.
///
/// The fixture draws `-160 100 120 40 re f`, so the square spans x −160…−40 and
/// y 100…140, and this is `(−100, 100)`. An *edge* rather than a centre, so
/// that ink and paper are a few pixels apart at any magnification — see the
/// module header.
const EDGE_AT: (f64, f64) = (-100.0, 100.0);

/// How far the anchor is off the left edge of the sheet, in page points.
///
/// This is the `k` in the ceiling arithmetic, and it is `|EDGE_AT.0|`;
/// [`the_off_page_distance_matches_the_anchor`] pins the two together so the
/// numbers this check prints cannot drift from the point it actually drives.
const OFF_PTS: f64 = 100.0;

/// How far past the old ceiling the climb must get before anything is asserted.
///
/// The old pasteboard stopped **reaching** `OFF_PTS` at `viewport / OFF_PTS`
/// and stopped **centring** it at half that. Climbing to 1.5× the looser of the
/// two clears both with margin, and the margin is what keeps a passing run from
/// depending on which rung of the zoom ladder the application happens to land
/// on.
const PAST_THE_OLD_CEILING: f64 = 1.5;

/// The most wheel notches to roll before giving up.
///
/// ★ A cap, not a count: the loop climbs until the calibrated target is
/// reached. Hitting the cap is a SKIP, because a run that never got past the
/// old ceiling has not tested anything — it is neither a pass nor evidence of a
/// defect. One notch is about 1.223×, so this reaches roughly 4 × 10⁵ %.
const MAX_NOTCHES: usize = 60;

/// Vertical distance from the converted edge to each patch's centre, in logical
/// points.
const PATCH_OFFSET_PT: f32 = 40.0;

/// Half the width and half the height of each patch, in logical points.
const PATCH_HALF_PT: f32 = 10.0;

/// A pixel at or below this in every channel is ink. The fixture fills with
/// `0 0 0 rg`; the slack is for the capture's colour management.
const INK: u8 = 96;

/// At least this fraction of the ink patch must be dark.
///
/// Not 1.0 — the patch is rounded through the window frame's scale and its
/// outermost row can land a pixel outside the square on a fractional-DPI
/// display.
const INK_FRACTION: f64 = 0.90;

/// At most this fraction of the control patch may be dark. Above this and the
/// probe is not measuring the page at all.
const PAPER_INK_FRACTION: f64 = 0.05;

/// See the module documentation.
pub struct AnObjectOffThePageSurvivesBeingZoomedInOn;

impl Check for AnObjectOffThePageSurvivesBeingZoomedInOn {
    fn name(&self) -> &'static str {
        "an_object_off_the_page_survives_being_zoomed_in_on"
    }

    fn defect(&self) -> &'static str {
        "an object past the edge of the sheet walks off the screen as the operator zooms in on \
         it, because the pasteboard is a fixed count of SCREEN pixels and so covers less and \
         less of the drawing with every notch — making the one thing editing requires, \
         magnification, the one thing that takes the object away"
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

/// The workspace root, from this crate's manifest directory.
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// The fraction of `patch` that is ink, and the pixel count it was measured
/// over.
///
/// `None` when the patch clipped to no area at all — which is a finding about
/// the capture geometry, never "no ink".
fn ink_fraction(image: &Image, patch: crate::geom::PixRect) -> Option<(f64, u64)> {
    let total = u64::from(patch.w) * u64::from(patch.h);
    if total == 0 {
        return None;
    }
    let dark = image
        .pixels_in(patch)
        .filter(|p| p.r <= INK && p.g <= INK && p.b <= INK)
        .count() as u64;
    #[allow(
        clippy::cast_precision_loss,
        reason = "a patch is a few hundred pixels; f64 is exact far past that" // ui-text-exempt: clippy lint justification, never displayed
    )]
    Some((dark as f64 / total as f64, total))
}

#[allow(clippy::too_many_lines)]
/// The event the canvas writes when it laid out **no page at all**.
///
/// `canvas-unavailable reason=nothing-visible` is the one line that separates
/// *"the shell declined to zoom any further"* from *"the shell zoomed, and the
/// document surface went blank."*
const UNAVAILABLE_EVENT: &str = "canvas-unavailable";

/// The reason field that means the strip's cull kept nothing.
const NOTHING_VISIBLE: &str = "nothing-visible";

/// ★★★ **A stalled climb is a FAIL when the canvas went blank, and a SKIP only
/// when it did not.**
///
/// # Why this function exists at all
///
/// The first draft of this check treated *any* stall as a precondition failure
/// and skipped, with the helpful-sounding suffix *"Raise `max_zoom_percent`, or
/// run in a narrower window."* Both halves of that sentence were wrong:
///
/// * `max_zoom_percent` defaults to `1e12` and its floor is `10.0`, so the
///   operator's zoom cap was never what stopped the climb. **That suffix was an
///   excuse the check had not measured**, and an unevidenced excuse is worse
///   than silence — it reads as an answered question, so nobody investigates.
/// * The stall on 2026-09-11 was **the defect this check exists to find**. The
///   strip culled pages on the sheet's rectangle rather than on the rectangle
///   its content actually reaches, so once the magnification carried the sheet
///   off the viewport the canvas laid out nothing, dropped its `canvas-viewport`
///   and `page` rects, and stopped publishing a zoom to climb with. Reported as
///   SKIP, that read as *"the harness could not run"*, which is the exact
///   failure mode this project has written down three times: **a SKIP is not
///   red, so a check can stop running unnoticed.**
///
/// # What it measures
///
/// A fresh trace, read at the moment of the stall, and only the
/// `canvas-unavailable` lines written **after** `mark` — the mark being taken
/// immediately before the first notch, so a line written while the document was
/// still opening cannot be mistaken for one the climb provoked.
///
/// * A `reason=nothing-visible` after the mark ⇒ **FAIL**, naming the cull.
/// * Anything else ⇒ **SKIP**, stating what was and was not measured and
///   offering no cause it did not observe.
///
/// A trace that cannot be re-read is itself a SKIP: the stall is real but the
/// evidence is not available, and guessing between the two verdicts is how a
/// harness invents defects that do not exist.
fn stall_verdict(session: &Session, mark: usize, what_happened: String) -> Error {
    let Ok(trace) = session.trace() else {
        return Error::new(format!(
            "{what_happened}, and the trace could not be re-read to find out why. SKIPPED: the \
             stall is real but this run has no evidence of its cause."
        ));
    };
    let blanked = trace
        .last_after(UNAVAILABLE_EVENT, mark)
        .filter(|line| line.get("reason") == Some(NOTHING_VISIBLE));
    match blanked {
        Some(line) => Error::new(format!(
            "★★★ THE CANVAS WENT BLANK WHILE ZOOMING TOWARD THE OFF-PAGE OBJECT — \
             {what_happened}, because the shell stopped laying out any page at all:\n\n    \
             {}\n\n\
             ★ That is not a zoom cap and not a harness limit. `nothing-visible` means the \
             strip's cull kept NO page, and the climb then had no published zoom left to \
             follow. The cull tests each page's own rectangle, so it is wrong the moment the \
             renderer can paint outside that rectangle — which is exactly what the halo tier \
             does for off-page content. At a high enough magnification the SHEET leaves the \
             viewport while the object being edited is mid-screen, and the operator is left \
             looking at grey.\n\n\
             ★ Suspect `viewer::strip::Strip::visible`'s caller in `canvas::present`: the rect \
             it culls with must be the viewport widened by `OpenDoc::pasteboard_overhang`, not \
             the bare viewport. Do NOT widen the rect handed to `canvas::tier::decide` as well \
             — that one is the true viewport on purpose.",
            line.raw
        ))
        .fatal(),
        None => Error::new(format!(
            "{what_happened}. SKIPPED. The canvas did NOT report `{UNAVAILABLE_EVENT} \
             reason={NOTHING_VISIBLE}` during the climb, so the defect this check exists for is \
             not what stopped it, and this run is no evidence either way. What stopped it was \
             not measured — read `off-page-zoom.trace.txt` rather than assuming a zoom cap, \
             which defaults to 1e12 and is almost never it."
        )),
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check rolls the wheel to zoom, so it has \
             nothing to measure without it. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so this check has neither a bound \
             to convert an off-page point against nor a viewport to calibrate the climb from.",
            ctx.profile.name
        ))
    })?;
    let pdf = workspace_root().join(FIXTURE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the off-page fixture is not at {}. It is 485 bytes of hand-written PDF syntax, \
             shared with `off_page_marquee`, `off_page_press` and `off_page_visible`.",
            pdf.display()
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("off-page-zoom.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} on {}",
        exe.display(),
        session.pid(),
        pdf.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // ★ Maximising makes this check HARDER, not easier: a wider viewport raises
    // the old pasteboard's ceiling, and the calibration below follows it up. It
    // also gives the climb room to keep the anchor on screen.
    session.maximize();
    // The halo cannot appear on the first frame — page, then decomposition,
    // then the widened raster. `off_page_visible`'s header has the sequence.
    session.settle(60);

    let driver = Driver::new(session.window());
    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process. \
             Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            session.trace_path().display()
        )));
    }
    let canvas = declared(&trace, ui_rect, CANVAS_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{CANVAS_REGION}` region. This check cannot fall back \
             to the page's own rect: the point it drives is outside that by construction."
        ))
    })?;

    // --- calibrate the climb against THIS window -----------------------------
    //
    // The old pasteboard was `viewport × 1.0`, so it stopped reaching a point
    // `OFF_PTS` past the sheet at `viewport / OFF_PTS`. That number is a
    // property of the window, so the check computes it rather than assuming it.
    let viewport_w = f64::from(canvas.max.x - canvas.min.x);
    if viewport_w <= 0.0 {
        return Err(Error::new(format!(
            "the `{CANVAS_REGION}` region is {viewport_w} points wide, so there is no viewport \
             to calibrate against. SKIPPED."
        )));
    }
    let old_ceiling = viewport_w / OFF_PTS;
    let target = old_ceiling * PAST_THE_OLD_CEILING;
    report.note(format!(
        "viewport {viewport_w:.0} pt wide, so the fixed pasteboard stopped REACHING {OFF_PTS:.0} \
         pt off the sheet at {:.0}% and stopped CENTRING it at {:.0}%. Climbing to {:.0}%.",
        old_ceiling * 100.0,
        old_ceiling * 50.0,
        target * 100.0
    ));

    // Where the square's bottom edge is now, so the wheel can be rolled over it.
    let frame = session.frame()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE_PAGE, 0)?;
    let start = mapping
        .doc_to_window_off_page(DocPoint::new(0, EDGE_AT.0, EDGE_AT.1), canvas)
        .map_err(|why| {
            Error::new(format!(
                "the off-page anchor is not on screen at 100 %, BEFORE any zooming: {why}\n\n\
                 ★ That is a finding about the STARTING state, not about the climb, so it is \
                 SKIPPED rather than reported as the defect this check exists for. If \
                 `off_page_visible` is green in the same run, suspect the window being too \
                 narrow for {OFF_PTS:.0} pt of overhang at 100 %."
            ))
        })?;
    let anchor = frame.to_screen(start);

    // ★★ Where the trace stood BEFORE the first notch, so that if the climb
    // stalls, `stall_verdict` looks only at what the climb itself provoked and
    // not at a `canvas-unavailable` written while the document was opening.
    let climb_mark = trace.mark();

    // --- climb, one notch at a time, over the anchor -------------------------
    let Some(mut now) = settled(&session, canvas)? else {
        return Err(Error::new(
            "the canvas never published a rect and a zoom, so there is no magnification to \
             follow. SKIPPED.",
        ));
    };
    let start_zoom = now.zoom;
    let mut notches = 0usize;
    while now.zoom < target && notches < MAX_NOTCHES {
        driver.scroll_at_held(anchor, &[VK_CONTROL], 1, 1)?;
        // ★ Wait for the notch to LAND: egui smooths a Ctrl+wheel notch over
        // about a dozen frames, and a fixed short wait reads a half-applied
        // zoom on every notch. `zoom_keeps_place::settled` is the shared reader
        // and the place that lesson is written down.
        let Some(after) = settled(&session, canvas)? else {
            return Err(Error::new(
                "the canvas stopped publishing a rect and a zoom mid-climb. SKIPPED.",
            ));
        };
        if after.zoom <= now.zoom {
            return Err(stall_verdict(
                &session,
                climb_mark,
                format!(
                    "the zoom stopped rising at {:.0}%, short of the calibrated {:.0}%",
                    after.zoom * 100.0,
                    target * 100.0
                ),
            ));
        }
        now = after;
        notches += 1;
    }
    if now.zoom < target {
        return Err(stall_verdict(
            &session,
            climb_mark,
            format!(
                "{MAX_NOTCHES} notches took the zoom from {:.0}% only to {:.0}%, short of the \
                 calibrated {:.0}%",
                start_zoom * 100.0,
                now.zoom * 100.0,
                target * 100.0
            ),
        ));
    }
    report.note(format!(
        "climbed {notches} notches from {:.0}% to {:.0}% — {:.1}× past the magnification at \
         which the fixed pasteboard stopped reaching this object",
        start_zoom * 100.0,
        now.zoom * 100.0,
        now.zoom / old_ceiling
    ));

    // --- where does the application say the edge is now? ---------------------
    //
    // ★ A FRESH trace, frame and mapping. Deliberately not the anchor-held
    // position: `zoom_keeps_place` owns that property, and asserting it here
    // too would make this check go red for its neighbour's reason.
    let trace = session.trace()?;
    let canvas = declared(&trace, ui_rect, CANVAS_REGION).ok_or_else(|| {
        Error::new(format!(
            "the `{CANVAS_REGION}` region disappeared during the climb. SKIPPED."
        ))
    })?;
    let frame = session.frame()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE_PAGE, 0)?;
    let edge = match mapping.doc_to_window_off_page(DocPoint::new(0, EDGE_AT.0, EDGE_AT.1), canvas)
    {
        Ok(p) => p,
        Err(why) => {
            return Ok(Some(format!(
                "★★★ THE OBJECT WALKED OFF THE SCREEN WHILE ZOOMING TOWARD IT, which is the \
                 operator's complaint in one sentence: {why}\n\n\
                 The anchor is ({:.0}, {:.0}) pt, {OFF_PTS:.0} pt past the left edge of a 200 pt \
                 sheet. At {:.0}% that is {:.0} screen points of overhang, against a viewport \
                 {viewport_w:.0} points wide.\n\n\
                 ★★ The arithmetic that predicts this belongs to the DEFECT, not to the fix: a \
                 pasteboard of `viewport × 1.0` is a fixed count of SCREEN pixels, so the slice \
                 of the DRAWING it covers is `viewport / zoom` and shrinks with every notch. \
                 `canvas::geometry::strip_offset` clamps to `content_extent − viewport`, and \
                 `content_extent` is built from that pasteboard — so the offset the zoom anchor \
                 solved was not WRONG, it was thrown away by the clamp.\n\n\
                 ⇒ Look at the overhang term in `canvas::geometry::pasteboard`, at \
                 `render::halo::overhang` which measures it, and at `canvas::present` which \
                 multiplies it by the zoom and publishes it onto `OpenDoc::pasteboard_overhang`. \
                 All eight geometry call sites must be handed the SAME number; one still passing \
                 `0.0` produces exactly this. Trace: {}.",
                EDGE_AT.0,
                EDGE_AT.1,
                now.zoom * 100.0,
                OFF_PTS * now.zoom,
                session.trace_path().display()
            )));
        }
    };

    // --- the pixels ----------------------------------------------------------
    //
    // Screen y grows DOWN and page y grows UP, so the patch ABOVE the edge on
    // screen is the one INSIDE the square.
    let patch_of = |centre: WindowPoint, dy: f32| -> crate::geom::PixRect {
        frame.logical_to_capture_pixels(LRect::new(
            Pt::new(centre.x() - PATCH_HALF_PT, centre.y() + dy - PATCH_HALF_PT),
            Pt::new(centre.x() + PATCH_HALF_PT, centre.y() + dy + PATCH_HALF_PT),
        ))
    };
    let ink_patch = patch_of(edge, -PATCH_OFFSET_PT);
    let paper_patch = patch_of(edge, PATCH_OFFSET_PT);

    let shot = ctx.out("off-page-zoom.png");
    let image = crate::capture::window_to_png(&session, &shot)?;
    report.artifact(shot.clone());

    let Some((ink, ink_px)) = ink_fraction(&image, ink_patch) else {
        return Err(Error::new(format!(
            "the ink patch clipped to zero area in the capture ({ink_patch:?} of a {} x {} \
             image), so nothing was measured. The conversion accepted the point, so this is a \
             capture-geometry disagreement rather than an application defect — suspect the \
             window frame's scale. Screenshot: {}.",
            image.width(),
            image.height(),
            shot.display()
        )));
    };
    let Some((paper, paper_px)) = ink_fraction(&image, paper_patch) else {
        return Err(Error::new(format!(
            "the PAPER control patch clipped to zero area ({paper_patch:?}), so the measurement \
             has no control and this check refuses to report on the ink patch alone. \
             Screenshot: {}.",
            shot.display()
        )));
    };
    report.note(format!(
        "at {:.0}%: ink patch {ink:.3} over {ink_px} px, {PATCH_OFFSET_PT:.0} pt above the \
         square's bottom edge; paper control {paper:.3} over {paper_px} px, the same distance \
         below it",
        now.zoom * 100.0
    ));

    // ★★★ THE CONTROL FIRST. If the patch below the edge is dark, the probe is
    // not looking at the page and nothing said about the ink patch would mean
    // anything — including a pass.
    if paper > PAPER_INK_FRACTION {
        return Ok(Some(format!(
            "★★★ THE PROBE IS NOT MEASURING THE PAGE: the CONTROL patch is {paper:.3} dark, \
             where at most {PAPER_INK_FRACTION:.2} is allowed. It sits {PATCH_OFFSET_PT:.0} \
             logical points BELOW the off-page square's bottom edge on screen — at {:.0}% that \
             is {:.4} page points below y = {:.0}, and the fixture has no mark down there.\n\n\
             ⇒ Read this as a finding about the HARNESS before reading it as one about the \
             feature: a panel, a dropped shadow, a context menu, or the window not being where \
             `WindowFrame` thinks it is. The ink patch measured {ink:.3}, and whatever that \
             number is, it is not evidence. Screenshot: {}.",
            now.zoom * 100.0,
            f64::from(PATCH_OFFSET_PT) / now.zoom,
            EDGE_AT.1,
            shot.display()
        )));
    }

    if ink < INK_FRACTION {
        return Ok(Some(format!(
            "★★★ THE OFF-PAGE OBJECT IS NOT PAINTED AT {:.0}%: only {ink:.3} of the patch inside \
             it is ink, where at least {INK_FRACTION:.2} is required. The control below the edge \
             is clean at {paper:.3}, so the probe IS looking at the right address — the object \
             is simply not there.\n\n\
             ★★ This run climbed {:.1}× past the magnification at which the OLD fixed pasteboard \
             stopped reaching this object, so the first suspect is the pasteboard's overhang \
             term — `canvas::geometry::pasteboard`, fed from `render::halo::overhang` through \
             `OpenDoc::pasteboard_overhang`.\n\n\
             ★ Second suspect, and it is a different module: the widened raster itself. \
             `off_page_visible` asserts the halo is painted at 100 %; if THAT check is red in \
             the same run, this is not the pasteboard at all and the finding belongs to \
             `render::halo::region` or to its pixmap ceiling — a halo box is `page + overhang` \
             points wide, so at high zoom it reaches the ceiling sooner than the page alone \
             does, and `region` declines rather than clipping.\n\n\
             Screenshot: {}.",
            now.zoom * 100.0,
            now.zoom / old_ceiling,
            shot.display()
        )));
    }

    report.note(format!(
        "★★★ the object that lies ENTIRELY off the left edge of the sheet is still on screen at \
         {:.0}% — {:.1}× past the magnification at which the fixed pasteboard stopped reaching \
         it. {ink:.3} of a patch inside it is ink, against {paper:.3} at the control just below \
         its edge. That is O23's third verb: the operator can EDIT what he put past the page \
         edge, because he can get close enough to see what he is doing",
        now.zoom * 100.0,
        now.zoom / old_ceiling
    ));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{
        EDGE_AT, INK_FRACTION, OFF_PTS, PAPER_INK_FRACTION, PAST_THE_OLD_CEILING, PATCH_HALF_PT,
        PATCH_OFFSET_PT,
    };

    /// The fixture's off-page square `(left, bottom, right, top)`, transcribed
    /// from its own content stream — `-160 100 120 40 re f`.
    const SQUARE_B: (f64, f64, f64, f64) = (-160.0, 100.0, -40.0, 140.0);

    /// The fixture's media box, which is also its crop box: it declares none.
    const MEDIA: (f64, f64, f64, f64) = (0.0, 0.0, 200.0, 200.0);

    /// ★★ The anchor is the midpoint of the off-page square's bottom edge, and
    /// that square really is off the page. Both halves are the premise, and
    /// both are transcribed from the fixture rather than assumed.
    #[test]
    fn the_anchor_is_the_bottom_edge_of_a_square_that_is_off_the_page() {
        assert!(
            (EDGE_AT.0 - (SQUARE_B.0 + SQUARE_B.2) / 2.0).abs() < 1e-9,
            "the anchor's x must be the square's horizontal midpoint"
        );
        assert!(
            (EDGE_AT.1 - SQUARE_B.1).abs() < 1e-9,
            "the anchor's y must be the square's bottom edge exactly — the whole oracle is ink \
             on one side of it and paper on the other"
        );
        assert!(
            SQUARE_B.2 < MEDIA.0,
            "the square's right edge {} must be left of the sheet's left edge {}",
            SQUARE_B.2,
            MEDIA.0
        );
    }

    /// ★ The ceiling arithmetic in the module header, in the calibration and in
    /// every failure message is written in terms of `OFF_PTS`, while the anchor
    /// is what is actually driven. If they drift, every number this check
    /// prints is wrong while the check itself stays green — the exact failure
    /// mode this suite has been corrected for seven times.
    #[test]
    fn the_off_page_distance_matches_the_anchor() {
        assert!((OFF_PTS - EDGE_AT.0.abs()).abs() < 1e-9);
    }

    /// ★★★ **The patches straddle the edge at every zoom this check can
    /// reach** — the theorem that makes a fixed SCREEN offset legal.
    ///
    /// In page points the patches reach `(OFFSET + HALF) / zoom` from the edge,
    /// so the ink patch stays inside the square while that is under the
    /// square's height, and the paper patch stays inside the widened raster
    /// while it is under the distance down to the raster's bottom. Both get
    /// *easier* as the zoom rises, so the binding case is the LOWEST zoom the
    /// check ever asserts at — which is not a constant, it is
    /// `PAST_THE_OLD_CEILING × viewport / OFF_PTS`. Evaluated here at a
    /// viewport narrower than any dock layout this shell produces.
    #[test]
    fn the_patches_straddle_the_edge_at_every_zoom_this_check_reaches() {
        const NARROWEST_VIEWPORT: f64 = 200.0;
        let lowest_zoom = PAST_THE_OLD_CEILING * NARROWEST_VIEWPORT / OFF_PTS;
        let reach_pt = f64::from(PATCH_OFFSET_PT + PATCH_HALF_PT) / lowest_zoom;

        let into_the_square = SQUARE_B.3 - SQUARE_B.1;
        assert!(
            reach_pt < into_the_square,
            "at {lowest_zoom}x the ink patch reaches {reach_pt} pt above the edge, past the \
             square's {into_the_square} pt height"
        );

        // The halo box is the content union, whose bottom is the lower of the
        // sheet's and the square's. Below that is canvas grey, not paper — a
        // different colour and a different claim.
        let halo_bottom = MEDIA.1.min(SQUARE_B.1);
        assert!(
            reach_pt < EDGE_AT.1 - halo_bottom,
            "at {lowest_zoom}x the paper patch reaches {reach_pt} pt below the edge, past the \
             widened raster's bottom at {halo_bottom}"
        );

        // …and sideways it must stay inside the square's width, or the ink
        // patch samples paper at one of the square's ends.
        let half_pt = f64::from(PATCH_HALF_PT) / lowest_zoom;
        assert!(
            EDGE_AT.0 - half_pt > SQUARE_B.0 && EDGE_AT.0 + half_pt < SQUARE_B.2,
            "at {lowest_zoom}x a half-patch of {half_pt} pt leaves the square sideways"
        );
    }

    /// The two thresholds cannot both be satisfied by one uniform image, which
    /// is what makes the pair an oracle rather than two opinions.
    #[test]
    fn the_thresholds_are_mutually_exclusive() {
        const {
            assert!(
                PAPER_INK_FRACTION < INK_FRACTION,
                "a uniform capture must fail one of the two"
            );
        }
    }
}
