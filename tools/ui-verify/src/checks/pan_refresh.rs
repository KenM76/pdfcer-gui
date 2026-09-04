//! `panning_past_the_overscan_renders_the_new_area` — the operator's blank
//! strip, made falsifiable.
//!
//! # The report
//!
//! `OPERATOR_REQUESTS.md` O25, 2026-08-23:
//!
//! > *"if I pan to far to one side when I am beyond 800% zoom it doesn't always
//! > render the new exposed area, and the same thing happens usually when I
//! > zoom out."*
//!
//! # ★★★ What was actually wrong, and why every existing check was green
//!
//! Above the pixmap ceiling a raster covers the **visible region** rather than
//! the page, so two textures of the same page at the same scale can be pictures
//! of *different places*. `render::settle`'s staleness test asked two questions
//! — has a discrete input changed (page, annotations, layers), and has the
//! scale changed — and **the region was in the cache key without being in
//! either**.
//!
//! So a pan that changed nothing but which part of the page is on screen was
//! not stale by any measure, and no render was ever requested. The picture the
//! operator had kept being drawn correctly at its own region and simply slid
//! off, leaving the newly exposed area blank for as long as they cared to look
//! at it.
//!
//! Every check passed throughout. `panning_at_deep_zoom_stays_where_it_was_put`
//! asks whether the view *moves* and whether the pixels are *placed* correctly
//! — both were perfect. `the_page_still_renders_at_every_decade_of_zoom`
//! photographs after a **zoom**, which does change the scale and therefore does
//! request a render. Nothing in the suite panned far enough to leave the
//! overscan and then looked at the screen.
//!
//! # What this asserts
//!
//! Pan by more than a whole viewport, so the destination is certainly outside
//! `render::strategy::OVERSCAN`'s half-viewport margin, then require **both**:
//!
//! | | rules out |
//! |---|---|
//! | a render completes after the pan | the shell never asked, which is O25 |
//! | the canvas is not near-uniform afterwards | it asked, and what arrived is blank anyway |
//!
//! ★ Both, because either alone is satisfiable while the operator looks at
//! nothing: a render can complete for the region the view has already left, and
//! a canvas can be non-uniform because of the page's *edge* while its middle is
//! empty.

use crate::checks::driving;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// `VK_CONTROL`, held while the wheel rolls to make it a zoom.
const VK_CONTROL: u16 = 0x11;

/// The canvas viewport.
const CANVAS_REGION: &str = "canvas-viewport";

/// The canvas's own state line.
const CANVAS_EVENT: &str = "canvas";

/// The `f64` position line, which carries the region actually drawn.
const POS_EVENT: &str = "canvas-pos";

/// The worker's **asynchronous** completion line.
const RENDER_EVENT: &str = "render-async-done";

/// The worker's **inline** completion line — the other half of the same fact.
///
/// ★★★ Added 2026-08-28, after this check reported *"NO RENDER WAS REQUESTED"*
/// against a build that had spawned and completed **nineteen** of them.
///
/// `render::worker` has two completion paths and takes whichever is cheaper: a
/// raster that finishes fast enough is done **inline**, on the frame that asked
/// for it, and only a slow one goes to the thread and comes back as
/// `render-async-done`. A region raster above the pixmap ceiling covers the
/// viewport rather than the page, so it is *small* — 3 ms on the fixture this
/// check now uses — and it never takes the asynchronous path at all.
///
/// ⇒ **A check that counts one completion path fails on a build that took the
/// other one**, and it fails by naming the feature rather than the instrument.
/// This one printed `render::settle`'s staleness test as the suspect, in detail,
/// down to `RenderKey::same_region` — and that mechanism was working perfectly.
///
/// ★★ The general rule, which this project has now met three times in one day:
/// **ask what the check SAMPLED before asking what is broken.** A failing
/// measurement is a claim about an instrument as much as about a program.
const RENDER_INLINE_EVENT: &str = "render-inline";

/// How far to zoom before panning, in Ctrl+wheel notches.
///
/// ★ Enough to be **past the pixmap ceiling**, which is where a raster stops
/// covering the page and starts covering the window — the tier this check is
/// about. Below it a pan is free and this check would be measuring nothing.
/// Twenty notches lands around 4,000 % on a Letter sheet, comfortably above the
/// ~2,070 % crossover.
const ZOOM_NOTCHES: usize = 20;

/// How far to pan, in wheel notches, and it is deliberately a lot.
///
/// `render::strategy::OVERSCAN` gives the raster half a viewport of margin on
/// every side, and `region_for` snaps to a half-viewport grid — so a pan has to
/// exceed a whole viewport before the operator is certainly looking at
/// something the current raster does not contain. This is the "too far to one
/// side" in his report, made specific.
const PAN_NOTCHES: i32 = -40;

/// How far to zoom back out afterwards, in Ctrl+wheel notches.
///
/// Enough to change the region substantially while staying **above the
/// crossover** — dropping below it in one step would put the raster back on the
/// whole-page path, where this defect cannot occur and the check would be
/// measuring the wrong tier.
const ZOOM_OUT_NOTCHES: usize = 6;

/// See the module documentation.
pub struct PanningPastTheOverscanRendersTheNewArea;

impl Check for PanningPastTheOverscanRendersTheNewArea {
    fn name(&self) -> &'static str {
        "panning_past_the_overscan_renders_the_new_area"
    }

    fn defect(&self) -> &'static str {
        "panning far to one side above the raster crossover leaves the newly exposed area blank — \
         the region changed, no re-render was ever requested, and the old picture simply slid off"
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

/// How many renders have completed so far, **by either path**.
///
/// ★ The asynchronous line carries an `outcome`, because a thread can come back
/// with a cancellation or a failure; the inline one cannot fail asynchronously
/// and carries none, so it is counted unconditionally. Two shapes for one fact,
/// and the asymmetry is the worker's rather than this function's.
fn renders_done(session: &Session) -> Result<usize> {
    let trace = session.trace()?;
    let asynchronous = trace
        .events(RENDER_EVENT)
        .filter(|l| l.get("outcome").is_some_and(|o| o == "done"))
        .count();
    let inline = trace.events(RENDER_INLINE_EVENT).count();
    Ok(asynchronous + inline)
}

/// A field of the canvas's `canvas-pos` line, as text.
///
/// Compared as text rather than parsed: this only needs to know *whether it
/// changed*, and the trace prints both regions from the same bits the cache
/// keys on, in the same format.
fn field(session: &Session, key: &str) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .events(POS_EVENT)
        .last()
        .and_then(|l| l.get(key))
        .map(str::to_owned))
}

/// ★★★ The region the shell **wants**, which is the one that moves with the
/// view.
///
/// `region=` is what the pixels on screen are a picture of, and on a build with
/// O25 present it never changes — no render is requested, so no new texture
/// arrives, so the field that describes the texture stands still. A check
/// watching it reads *"the view did not move"* and skips, which is what the
/// first version of this check did against a binary with the defect
/// deliberately restored.
///
/// `want=` moves the instant the view does. **The gap between the two is the
/// defect**, and it takes both fields to measure a gap.
fn wanted_region(session: &Session) -> Result<Option<String>> {
    field(session, "want")
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
            "input is disabled (--no-input). This check zooms and pans the canvas. Reported as \
             SKIPPED rather than passed.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("pan-refresh.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(50);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    let canvas = driving::declared(&trace, ui_rect, CANVAS_REGION)
        .ok_or_else(|| Error::new(format!("no `{CANVAS_REGION}`; is a document open?")))?;
    let frame = session.frame()?;
    let centre = frame.declared_at(canvas, 0.5, 0.5);

    // --- 1: get above the raster crossover ----------------------------------
    driver.scroll_at_held(centre, &[VK_CONTROL], 1, ZOOM_NOTCHES)?;
    session.settle(200);

    let zoom = session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0);
    let Some(before_region) = wanted_region(&session)? else {
        return Err(Error::new(
            "the canvas never reported a `canvas-pos` line, so there is no region to watch. \
             SKIPPED.",
        ));
    };
    if before_region == "none" {
        return Err(Error::new(format!(
            "at {:.0}% the raster is still WHOLE-PAGE (`region=none`), so this check has not \
             reached the tier it is about — above the crossover a raster covers the window and a \
             pan can expose ground it does not contain; below it the texture is the whole page \
             and a pan is free. Raise ZOOM_NOTCHES. SKIPPED rather than passed.",
            zoom * 100.0
        )));
    }
    report.note(format!("at {:.0}%, region {before_region}", zoom * 100.0));

    // --- 2: pan well past the overscan --------------------------------------
    let renders_before = renders_done(&session)?;
    driver.scroll_at(centre, PAN_NOTCHES)?;
    // Generous: a region render at this depth re-interprets the content stream,
    // and the debounce has to expire first. ★ A settle that is too short does
    // not fail this check, it makes its evidence ambiguous — "no render yet" and
    // "no render ever" look identical, and the second is the defect.
    session.settle(260);

    let after_region = wanted_region(&session)?.unwrap_or_default();
    let renders_after = renders_done(&session)?;
    report.note(format!(
        "after the pan: region {after_region}, {} render(s) completed",
        renders_after - renders_before
    ));

    // --- 3: the assertions ---------------------------------------------------
    if after_region == before_region {
        return Err(Error::new(format!(
            "the pan did not change which region the canvas wants — it is still {before_region}. \
             Either the wheel did not move the view or {PAN_NOTCHES} notches is inside the \
             overscan at this zoom. SKIPPED rather than passed: nothing was exposed, so nothing \
             was owed."
        )));
    }

    if renders_after == renders_before {
        return Ok(Some(format!(
            "★★ NO RENDER WAS REQUESTED. The view moved from region {before_region} to \
             {after_region} at {:.0}% and not one raster completed in the {} frames after it. \
             This is the operator's \"it doesn't always render the new exposed area\": \
             `render::settle`'s staleness test compares the page, the annotations, the layers and \
             the scale — and the REGION is in the cache key without being in any of them, so a \
             pan is not stale by any measure it applies. See `RenderKey::same_region`.",
            zoom * 100.0,
            260
        )));
    }

    let path = ctx.out("pan-refresh.png");
    let image = match crate::capture::window_to_png(&session, &path) {
        Ok(image) => image,
        Err(e) => return Err(Error::new(format!("the window capture failed: {e}"))),
    };
    report.artifact(path.clone());
    let uniformity =
        crate::pixels::region_not_uniform(&image, frame.logical_to_capture_pixels(canvas));
    if uniformity.is_uniform() {
        return Ok(Some(format!(
            "a render completed after the pan and the CANVAS is still near-uniform ({}) — so the \
             shell asked, and what arrived is not a picture of where the view now is. The capture \
             is at {}.",
            uniformity.summary(),
            path.display()
        )));
    }
    report.note(format!(
        "the canvas shows {} distinct tone(s) after the pan",
        uniformity.distinct
    ));

    // --- 4: and the other half of his sentence ------------------------------
    //
    // ★★ *"…and the same thing happens usually when I zoom out."* Same root
    // cause arriving by a different route: a zoom DOES change the scale, so a
    // render is requested — but it is built from whatever region was current
    // when it spawned, and by the time it lands the gesture has moved on. Once
    // the scale settles, nothing notices the region it arrived with is the
    // wrong one.
    //
    // ★ Asserted separately rather than assumed fixed by the pan case. They
    // share a cause and they do not share a code path, and "it is probably the
    // same bug" is how the second half of a two-part report gets shipped
    // broken.
    let before_zoom_out = renders_done(&session)?;
    let region_before_out = wanted_region(&session)?.unwrap_or_default();
    driver.scroll_at_held(centre, &[VK_CONTROL], -1, ZOOM_OUT_NOTCHES)?;
    session.settle(260);

    let region_after_out = wanted_region(&session)?.unwrap_or_default();
    let after_zoom_out = renders_done(&session)?;
    report.note(format!(
        "after zooming out: {} render(s) completed",
        after_zoom_out - before_zoom_out
    ));

    if region_after_out == region_before_out {
        return Err(Error::new(
            "zooming out did not change which region the canvas wants, so nothing was exposed \
             and nothing was owed. Either the Ctrl was lost and the wheel panned instead, or \
             the zoom dropped below the crossover in one step. SKIPPED rather than passed.",
        ));
    }
    if after_zoom_out == before_zoom_out {
        return Ok(Some(format!(
            "★★ ZOOMING OUT ASKED FOR NOTHING. The wanted region moved from \
             {region_before_out} to {region_after_out} and no raster completed after it. This \
             is the second half of the operator's report. A zoom changes the scale, so the \
             FIRST render is requested — what is missing is the one that notices the region \
             the arriving raster was built for is no longer the region on screen. See \
             `RenderKey::same_region`."
        )));
    }

    let out_path = ctx.out("pan-refresh-zoomed-out.png");
    let out_image = match crate::capture::window_to_png(&session, &out_path) {
        Ok(image) => image,
        Err(e) => return Err(Error::new(format!("the window capture failed: {e}"))),
    };
    report.artifact(out_path.clone());
    let out_uniformity =
        crate::pixels::region_not_uniform(&out_image, frame.logical_to_capture_pixels(canvas));
    if out_uniformity.is_uniform() {
        return Ok(Some(format!(
            "after zooming out a render completed and the CANVAS is still near-uniform ({}) \
             — the shell asked, and what arrived is not a picture of where the view now is. \
             The capture is at {}.",
            out_uniformity.summary(),
            out_path.display()
        )));
    }
    report.note(format!(
        "the canvas shows {} distinct tone(s) after zooming out",
        out_uniformity.distinct
    ));

    Ok(None)
}
