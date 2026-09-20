//! `the_travelling_copy_is_on_the_glass` — **O215 ask 5, measured in pixels
//! instead of in the painter's own account of itself.**
//!
//! # What this adds to the row beside it
//!
//! [`crate::checks::chunk_ghost`] asserts that `overlay::draw_raster_ghost`
//! decided to blit, and how many pieces it blitted. That is the painter's claim
//! about the painter. A blit at alpha 0, a blit behind an opaque panel and a
//! blit sampling the wrong corner of the page texture all write the same
//! `drawn=1 clipped=0 reason=none`, so the trace cannot separate a working
//! preview from three broken ones.
//!
//! Layout and clipping defects have exactly one oracle, a rendered screenshot.
//! This is that oracle for the one class of behaviour the harness was blind
//! to: an affordance that exists only between the press and the release.
//! [`crate::input`]'s `drag_observed` holds the frame open so it can be
//! photographed.
//!
//! # The five questions, in the order they must be asked
//!
//! | # | Question | Instrument |
//! |---|---|---|
//! | 1 | is the destination blank BEFORE the press? | [`pixels::region_not_uniform`] over the drop rect |
//! | 2 | is anything drawn there DURING the hold? | the same, inverted |
//! | 3 | did the document stay unmarked? | [`pixels::mean_luminance`], source before against source during |
//! | 4 | is it a COPY rather than the content? | the same, destination against source |
//! | 5 | did it come from the right part of the texture? | [`pixels::ink_run_into`], coverage compared |
//!
//! ★★ **The order of 3 and 4 is load-bearing, and it was got wrong first.**
//! Question 4 measures the copy against the source, so it is only meaningful
//! while the source is what it was. A build that washed the un-moved line
//! while a move was in flight was caught — but by question 4, which named it
//! *"the copy is not distinguishable from the content"* when the actual defect
//! was the opposite: the content had been altered to look like a copy. A
//! reference has to be established before it can be compared against.
//!
//! ⚠ **The inset is what makes question 2 an assertion rather than
//! theatre.** The outline ghost strokes the boundary of the very rectangle
//! being measured, so ink anywhere in the un-inset region is satisfied by the
//! affordance that already shipped — an assertion both outcomes satisfy. Only
//! the strict interior separates lettering from a box. Every region here is
//! inset by [`INSET_PX`] on all four edges for that reason, and the source
//! region is inset by the same amount so the two are comparable.
//!
//! ★ **Question 3 is the R8b assertion and it is the reason this check
//! measures the source twice.** *Fuzzy, never sneaky* forbids marking applied
//! content: a document being dragged must render exactly as the saved document
//! will render, with no tint, dashed outline or provisional layer over the
//! content that has not moved yet. A build that faded the source line while a
//! move was in flight would pass every other question here and would be
//! marking the canvas. Nothing else in the suite asks it.
//!
//! ⚠ **Question 5 is weak and is labelled weak.** It is a similarity test, not
//! an identity test: a build that sampled a *different line of the same
//! paragraph* would produce a comparable ink fraction and pass. It discriminates
//! blank paper from lettering and nothing finer. Tightening it means comparing
//! the per-column ink profile — the word shapes — which is worth building the
//! day a defect of that shape appears and not before.
//!
//! # Calibration, and why it comes first
//!
//! Question 1 is not a precondition dressed up as an assertion; it is the
//! second direction of the oracle. An after-capture alone says *"there is ink
//! at the destination"* without establishing that the ink was not already
//! there, and a destination chosen over a line of the paragraph would satisfy
//! every question below on a build that drew no ghost at all. The precedent is
//! `tools/ui-verify/tests/pixel_oracle_against_real_evidence.rs`, which pins
//! the contrast oracle by asserting both the unreadable heading and the
//! readable prose eighty pixels from it.
//!
//! A destination that fails the before-check is a **SKIP**, not a FAIL: it says
//! the fixture was not laid out where this check believes it is, which is a
//! statement about the harness rather than about the build.
//!
//! # Fixture — pinned, and `--pdf` is ignored
//!
//! `fixtures/paragraph.pdf`, six lines of one text object. The geometry table
//! is in [`crate::checks::chunk_band`]'s header; [`LINE_BAND`] is its line 0
//! verbatim, chosen because it is the widest of the six and therefore the one
//! carrying the most pixels to measure.
//!
//! [`DROP_DY_PT`] puts the copy 300 pt below that line, which on this document
//! is blank paper: the text occupies 620.0 to 708.4 and the drop band is 400.0
//! to 408.4, clear of line 5's floor by 212 pt.
//!
//! # ⚠ HOW TO FALSIFY THIS CHECK — do this before believing a PASS
//!
//! **Copy each file aside first** and restore from the byte copy; never revert
//! with git, because this project runs parallel tracks and a chained revert
//! discards another track's uncommitted work. Rebuild **both** `-p pdfcer-gui`
//! and `-p ui-verify`, and confirm the exe is newer than the source: a stale
//! binary is the commonest cause of a falsification that did not reproduce.
//!
//! 1. **Delete the call site.** Remove the `overlay::draw_raster_ghost(…)`
//!    statement in `canvas::painting::draw`. Question 2 goes red: the
//!    destination interior stays as blank as the before-capture found it.
//!    `chunk_ghost` beside it goes red too — the call site is where its trace
//!    line is written, so this plant does not separate the two rows.
//! 2. **Remove the tint.** `overlay::raster::RASTER_GHOST_ALPHA` to `255`.
//!    **This is the plant that justifies the row existing.** `chunk_ghost`
//!    stays GREEN — the painter still decides to blit and still says so, and
//!    every trace field is unchanged — while question 4 here goes red, because
//!    a copy that renders at full strength is indistinguishable from the
//!    content itself. Measured, both ways, on the same build.
//! 3. **Take the UV against the page rect** rather than against
//!    `strip::PageView`'s `paint_rect`. ⚠ **Measured: a NO-OP at the zoom this
//!    check runs at, and it stayed green.** `paint_rect` equals the page rect
//!    at every zoom below the pixmap ceiling, and [`WANT_ZOOM`] is far below
//!    it, so the plant changes nothing to catch. Question 5 is therefore
//!    **unfalsified**, not merely weak: nothing here is evidence about where
//!    the UV came from. Exercising it needs this gesture driven at the region
//!    tier, which is a different check and is not this one.
//! 4. **Mark the canvas.** Paint the un-moved line over itself at
//!    `Color32::from_white_alpha(110)` after the blit in `draw_raster_ghost`.
//!    Question 3 goes red and nothing else does.

use crate::checks::chunk_ghost::{MOVE_DECLINED_EVENT, MOVED_ONE_EVENT, descend};
use crate::checks::driving::{
    self, SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list,
};
use crate::checks::text_chunks::{MODE, PAGE_REGION, Verdict, press_the_toggle, verdict};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint, WindowFrame};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::pixels;
use crate::report::CheckReport;
use crate::sys::vk;

/// The scrollable region the sheet sits inside.
///
/// ★ Every measured rectangle is clipped to it, and the check reads blank
/// panel furniture as content without that. A line of this fixture is 267 pt
/// wide and the canvas at [`WANT_ZOOM`] is narrower than that, so the band a
/// document rectangle converts to runs off the right-hand edge of the canvas
/// and on into whatever dock is beside it. `logical_to_capture_pixels` clamps
/// to the WINDOW, which is the wrong boundary here and cannot know it.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name

/// Which of the fixture's six lines is dragged.
///
/// Line 0 is the widest — 266.8 pt against line 5's 38.0 — so it puts the most
/// glyph pixels inside the measured region. Every question here is a statistic
/// over that region, and a statistic over forty pixels is a coin toss.
const LINE: usize = 0;

/// Line 0's glyph band, verbatim from [`crate::checks::chunk_band`]'s geometry
/// table: `(x_left, y_bottom, x_right, y_top)` in PDF user space, where y
/// increases **upwards**.
const LINE_BAND: (f64, f64, f64, f64) = (72.0, 700.0, 338.8, 708.4);

/// How far down the page the line is dragged, in PDF points.
///
/// Negative because PDF y increases upwards and the drag goes down the sheet.
/// 150 pt puts the copy's band at 550.0 to 558.4 — blank paper on this
/// document, 62 pt clear of the lowest line of the paragraph, and near enough
/// to the source that both ends of the gesture are on screen at [`WANT_ZOOM`].
const DROP_DY_PT: f64 = -150.0;

/// The zoom this check measures at.
///
/// ★ Not a preference — an arithmetic requirement, and the check reports SKIP
/// rather than a verdict without it. The subject is one 8.4 pt line of text;
/// at the zoom a letter sheet arrives fitted to the window it is six device
/// pixels tall, which leaves nothing at all once [`INSET_PX`] has taken the
/// outline off both edges. Two page points to the logical point gives about
/// seventeen, and an interior of eleven rows to take a mean over.
///
/// The ceiling on it is the other end of the gesture: the copy is dropped
/// [`DROP_DY_PT`] away, and a zoom that put the destination outside the
/// viewport would make `doc_to_window` refuse the conversion.
const WANT_ZOOM: f32 = 2.0;

/// How many Ctrl+wheel notches are spent reaching for [`WANT_ZOOM`] before the
/// attempt is called stalled.
///
/// A notch's factor is egui's, not this harness's to know, so the loop reads
/// the application's own `zoom=` back after every spin rather than counting. The
/// cap is a runaway guard.
const ZOOM_SPINS: usize = 60;

/// How far each measured region is inset from the rectangle it describes, in
/// capture pixels.
///
/// Three, and the number is load-bearing in both directions. Smaller and the
/// outline ghost's own stroke — one logical point, up to two pixels at the
/// scale factors this runs at — is inside the region, which would let a box
/// satisfy an assertion about lettering. Larger and a 8.4 pt line at fit zoom
/// has no interior left to measure.
const INSET_PX: u32 = 3;

/// How much lighter the copy must read than the content it copies, in relative
/// luminance.
///
/// The raster ghost composites at 190/255, so a black glyph over white paper
/// lands at 65 and the gap over a region of ordinary text is two orders of
/// magnitude above this. The floor is set where it is to be clear of capture
/// quantisation rather than to be a tolerance on the alpha: a build whose tint
/// had drifted from 190 to 230 should still pass, because the ask is *a copy,
/// visibly not the content*, and it is not a claim about a constant.
const LIGHTER_BY: f64 = 0.01;

/// How far the source region may move between the before-capture and the
/// mid-gesture one, in relative luminance.
///
/// Meant to be zero: the same pixels, captured twice, by a deterministic
/// screenshot of a window that is not animating. It is not written as an exact
/// comparison because the page under the source line is being repainted every
/// frame while the drag is in flight, and a single antialiased pixel landing
/// differently is not the defect this question is about.
const UNMARKED_WITHIN: f64 = 0.002;

/// How far the copy's ink coverage may fall from the source's, as a fraction.
///
/// Wide on purpose. The copy is the same glyphs at 74.5 % over the same paper,
/// so its core pixels are ink by [`pixels::ink_run_into`]'s threshold and some
/// of its antialiased edge pixels are not — a real and expected loss that
/// tracks the alpha. What this band excludes is the case it was written for: a
/// UV that sampled blank paper, where the fraction is zero.
const COVERAGE_BAND: (f64, f64) = (0.4, 1.6);

/// See the module documentation.
pub struct TheTravellingCopyIsOnTheGlass;

impl Check for TheTravellingCopyIsOnTheGlass {
    fn name(&self) -> &'static str {
        "the_travelling_copy_is_on_the_glass"
    }

    fn defect(&self) -> &'static str {
        "The copy of a dragged line that is supposed to travel with the pointer is not actually \
         on the glass — blitted at an alpha that vanishes, clipped away, sampled out of the wrong \
         part of the page texture, or drawn so strongly that the operator cannot tell the \
         preview from the text that has not moved yet"
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

/// One region's readings, so the before and during captures can be compared
/// field by field rather than by three parallel variables.
#[derive(Clone, Copy)]
struct Reading {
    /// Mean relative luminance — how light the region is.
    light: f64,
    /// Ink pixels as a fraction of the region.
    coverage: f64,
    /// How many distinct quantised colours the region holds.
    distinct: usize,
}

impl Reading {
    /// Read a region, or say why it could not be read.
    ///
    /// A region with no pixels is an `Err` rather than a zero: it means the
    /// rectangle fell outside the capture, which is a finding about the layout
    /// and must never be averaged into a verdict.
    fn of(img: &Image, region: PixRect, what: &str) -> Result<Self> {
        let light = pixels::mean_luminance(img, region).ok_or_else(|| {
            Error::new(format!(
                "the {what} region {region:?} holds no pixels of the capture. SKIPPED: the \
                 rectangle this check computed is off the window, so the fixture is not laid out \
                 where the geometry table says it is."
            ))
        })?;
        let ink = pixels::ink_run_into(img, region);
        let uniform = pixels::region_not_uniform(img, region);
        #[allow(clippy::cast_precision_loss)]
        Ok(Self {
            light,
            coverage: ink.ink as f64 / ink.sampled.max(1) as f64,
            distinct: uniform.distinct,
        })
    }

    /// A one-line account for a note or a failure sentence.
    fn summary(&self) -> String {
        format!(
            "light {:.4}, ink {:.3} of the region, {} distinct colours",
            self.light, self.coverage, self.distinct
        )
    }
}

/// Inset a rectangle on all four edges, or `None` if nothing is left.
///
/// `None` rather than a degenerate rectangle: a zero-area region reads as
/// "nothing was sampled" in every oracle in [`pixels`], which is
/// indistinguishable from "nothing was drawn" and is the exact confusion this
/// check exists to avoid.
fn inset(r: PixRect, by: u32) -> Option<PixRect> {
    let (w, h) = (r.w.checked_sub(by * 2)?, r.h.checked_sub(by * 2)?);
    (w > 0 && h > 0).then(|| PixRect::new(r.x + by, r.y + by, w, h))
}

/// The part of `r` that is also inside `to`, or `None` if they do not overlap.
///
/// `None` rather than an empty rectangle, for [`inset`]'s reason: every oracle
/// in [`pixels`] reads a zero-area region as "nothing was sampled", which is
/// indistinguishable from "nothing was drawn".
fn clip(r: PixRect, to: PixRect) -> Option<PixRect> {
    let x0 = r.x.max(to.x);
    let y0 = r.y.max(to.y);
    let x1 = (r.x + r.w).min(to.x + to.w);
    let y1 = (r.y + r.h).min(to.y + to.h);
    (x1 > x0 && y1 > y0).then(|| PixRect::new(x0, y0, x1 - x0, y1 - y0))
}

/// The pixel rectangle a PDF-user-space band occupies in the window capture.
///
/// Written from the geometry table rather than read from anything the
/// application published, deliberately: an oracle that takes its aim point from
/// the build it is measuring agrees with whatever that build does.
fn band_to_pixels(
    mapping: &CanvasMapping,
    frame: &WindowFrame,
    band: (f64, f64, f64, f64),
    dy_pt: f64,
) -> Result<PixRect> {
    let (x0, y_bottom, x1, y_top) = band;
    // PDF y increases upwards and window y increases downwards, so the band's
    // TOP edge is its larger user-space y.
    let top_left = mapping.doc_to_window(DocPoint::new(0, x0, y_top + dy_pt))?;
    let bottom_right = mapping.doc_to_window(DocPoint::new(0, x1, y_bottom + dy_pt))?;
    Ok(frame.logical_to_capture_pixels(LRect::new(
        Pt::new(top_left.x(), top_left.y()),
        Pt::new(bottom_right.x(), bottom_right.y()),
    )))
}

/// The whole gesture sequence. See the module documentation.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is two clicks and one drag held open \
             across a window capture, and it needs the pointer and the foreground. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let (pdf, grab_point) = crate::fixture::text_chunk_point(LINE);
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the pinned fixture {} is missing. That is a broken checkout rather than an absent \
             precondition — it is committed to this repository.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} and photographs a drag of its \
         line {LINE} while the button is still down",
        pdf.display()
    ));

    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new("could not read a page size from the fixture. Pass --page-size.")
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("chunk-ghost-pixels.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(45);
    let driver = Driver::new(session.window());
    let frame = session.frame()?;

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }
    // The chunk rung is offered only where a box is drawn, so a run that began
    // with the switch off would measure the switch. The preference is persisted
    // beside the exe, which makes its state a fact about the machine.
    if verdict(&session.trace()?, 0) == Some(Verdict::Declined("switched-off".to_owned())) {
        report.note(
            "the chunk boxes were OFF at launch. Turning them on: the chunk rung is offered only \
             where a box is drawn, so this check has no subject without them.",
        );
        if let Some(failure) = press_the_toggle(&session, &driver, ui_rect, true, report)? {
            return Ok(Some(failure));
        }
    }

    // --- zoom in far enough that one line of text HAS an interior ----------
    //
    // Anchored on the line itself: Ctrl+wheel in this application zooms about
    // the pointer, so the subject stays under the aim while the sheet grows
    // around it, and no separate re-centring gesture is needed.
    let at_line = aim_screen(
        &CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?,
        &frame,
        grab_point,
    )?;
    let reached = zoom_in_at(&session, &driver, at_line)?;
    if reached < WANT_ZOOM * 0.9 {
        return Err(Error::new(format!(
            "the zoom stalled at {reached:.2} where this check needs about {WANT_ZOOM:.2}. \
             SKIPPED: at any less than that one line of this fixture is a handful of device pixels \
             tall, and a mean over a handful of pixels is a coin toss rather than an oracle. \
             Either the Ctrl+wheel is not landing on the canvas or the application capped the \
             zoom."
        )));
    }
    report.note(format!("zoomed to {reached:.2} on line {LINE}"));

    // --- the geometry, converted once, after the zoom and before any gesture -
    //
    // ★ One mapping for every rectangle and both endpoints. A conversion taken
    // between gestures would re-read a canvas rect that the gesture itself had
    // already changed, and the displacement the copy is measured against would
    // then not be the displacement the pointer travelled.
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let canvas_px = frame.logical_to_capture_pixels(
        driving::declared(&session.trace()?, ui_rect, VIEWPORT_REGION).ok_or_else(|| {
            Error::new(format!(
                "the application declares no `{VIEWPORT_REGION}` region, so this check has no \
                 boundary to clip its rectangles to and would read the dock beside the canvas as \
                 page content. SKIPPED."
            ))
        })?,
    );
    let measure = |dy: f64, what: &str| -> Result<PixRect> {
        clip(band_to_pixels(&mapping, &frame, LINE_BAND, dy)?, canvas_px)
            .and_then(|r| inset(r, INSET_PX))
            .ok_or_else(|| {
                Error::new(format!(
                    "the {what} band has no interior inside the canvas at {reached:.2}: clipped to \
                     {canvas_px:?} and inset by {INSET_PX} px it is empty. SKIPPED — the window is \
                     too small, or the sheet is scrolled so that the subject is not wholly on \
                     screen."
                ))
            })
    };
    let source_px = measure(0.0, "source")?;
    let drop_px = measure(DROP_DY_PT, "drop")?;
    let grab = aim_screen(&mapping, &frame, grab_point)?;
    let drop_at = aim_screen(
        &mapping,
        &frame,
        DocPoint::new(0, grab_point.x, grab_point.y + DROP_DY_PT),
    )?;
    report.note(format!(
        "line {LINE}'s interior is {source_px:?} and the drop interior is {drop_px:?}, both \
         computed from the geometry table and inset by {INSET_PX} px"
    ));

    // --- 1: the destination is blank BEFORE the press -----------------------
    let before_shot = ctx.out("chunk-ghost-pixels.before.png");
    report.artifact(before_shot.clone());
    let before = crate::capture::window_to_png(&session, &before_shot)?;
    let drop_before = Reading::of(&before, drop_px, "drop")?;
    let source_before = Reading::of(&before, source_px, "source")?;
    if drop_before.distinct > 1 {
        return Err(Error::new(format!(
            "the drop region is not blank paper before the drag: {}. SKIPPED, not failed — every \
             question below compares against this region being empty, so a destination with \
             something already in it makes the whole oracle one-sided. The fixture is not laid \
             out where this check's geometry table says it is.",
            drop_before.summary()
        )));
    }
    report.note(format!(
        "calibrated: the drop region is blank before the press — {}",
        drop_before.summary()
    ));

    // --- select ONE line ----------------------------------------------------
    let part = match descend(&session, &driver, grab)? {
        Ok(part) => part,
        Err(why) => return Ok(Some(why)),
    };
    if part != LINE {
        return Ok(Some(format!(
            "the descent reached chunk {part} where the aim was chunk {LINE}. Every rectangle \
             below is stated against line {LINE}'s baseline, so a drag of another line would \
             measure blank paper and report it as a missing preview."
        )));
    }

    // --- 2 to 5: photograph the gesture with the button still down ----------
    let mark = session.trace()?.mark();
    let shot = ctx.out("chunk-ghost-pixels.during.png");
    report.artifact(shot.clone());
    let during = driver.drag_observed(grab, drop_at, || {
        crate::capture::window_to_png(&session, &shot)
    })?;
    session.settle(30);
    let drop_during = Reading::of(&during, drop_px, "drop")?;
    let source_during = Reading::of(&during, source_px, "source")?;

    if drop_during.distinct <= 1 {
        return Ok(Some(format!(
            "★★★ THE COPY IS NOT ON THE GLASS. The drop region was blank before the press and is \
             still blank with the button down at the destination — {}. The trace may well say \
             `canvas-raster-ghost drawn=1 reason=none`; that is the painter's account of its own \
             decision, and this is the frame. Between the two lie every way a blit can decide to \
             happen and not appear: an alpha that vanishes, a clip that removes it, a rectangle \
             off the canvas. The operator drags a line of a note and watches an empty box travel.",
            drop_during.summary()
        )));
    }
    report.note(format!(
        "the copy is on the glass — the drop region carries {} with the button still down",
        drop_during.summary()
    ));

    if (source_during.light - source_before.light).abs() > UNMARKED_WITHIN {
        return Ok(Some(format!(
            "★★★ THE DOCUMENT WAS MARKED WHILE THE MOVE WAS IN FLIGHT. The source line read \
             {:.4} before the press and {:.4} with the button down — it changed. Applied content \
             renders exactly as saved content will render: a line that dims, tints or outlines \
             itself because a gesture is in progress is pdfcer drawing its own uncertainty onto \
             the page, and a screenshot of the editing canvas then differs from a screenshot of \
             the same document saved and reopened. Before: {}. During: {}.",
            source_before.light,
            source_during.light,
            source_before.summary(),
            source_during.summary()
        )));
    }
    report.note("the source line was not marked while the move was in flight");

    if drop_during.light <= source_during.light + LIGHTER_BY {
        return Ok(Some(format!(
            "★★ THE COPY IS NOT DISTINGUISHABLE FROM THE CONTENT. The drop region reads {:.4} \
             against the source line's {:.4}, and a copy must read LIGHTER than the text it \
             copies by at least {LIGHTER_BY}. Drop: {}. Source: {}. The tint is what tells the \
             operator which of the two lines on his screen is the one that has not moved yet; \
             without it he is looking at two identical lines and only one of them is real.",
            drop_during.light,
            source_during.light,
            drop_during.summary(),
            source_during.summary()
        )));
    }
    report.note(format!(
        "the copy reads lighter than the line it copies — {:.4} against {:.4}",
        drop_during.light, source_during.light
    ));

    // ⚠ WEAK, and labelled weak in the header: a similarity test, not an
    // identity test. It separates blank paper from lettering and nothing finer.
    let ratio = drop_during.coverage / source_during.coverage.max(f64::EPSILON);
    if ratio < COVERAGE_BAND.0 || ratio > COVERAGE_BAND.1 {
        return Ok(Some(format!(
            "★ THE COPY DOES NOT LOOK LIKE THE LINE IT COPIES. Its ink coverage is {ratio:.2} of \
             the source's, outside {COVERAGE_BAND:?}. Drop: {}. Source: {}. The copy is a blit of \
             a sub-rectangle of the page texture, and the rectangle is taken relative to the \
             texture's paint rect rather than to the page rect — the two coincide at some zooms \
             and not at others, so a UV against the wrong one samples a neighbouring line or \
             blank paper at exactly the tier where the operator is working closest.",
            drop_during.summary(),
            source_during.summary()
        )));
    }
    report.note(format!(
        "the copy's ink coverage is {ratio:.2} of the line's, so it was sampled from the \
         lettering rather than from beside it"
    ));

    // --- and the gesture the copy previewed actually happened ---------------
    //
    // True on the SAME drag, which is why the release is at the destination
    // rather than back at the start. A preview asserted by one gesture and an
    // outcome asserted by another are two claims about two drags with nothing
    // joining them. And a preview of a refusal is worse than no preview: the
    // operator watches his line move and then finds it back where it started.
    let trace = session.trace()?;
    if trace.last_after(MOVED_ONE_EVENT, mark).is_none() {
        let declined = trace
            .last_after(MOVE_DECLINED_EVENT, mark)
            .map(|l| l.raw.clone())
            .unwrap_or_else(|| "and nothing was written at all".to_owned());
        return Ok(Some(format!(
            "★★ THE COPY PREVIEWED A MOVE THAT NEVER HAPPENED: no `{MOVED_ONE_EVENT}` \
             after the release — {declined}. Everything above held, so the \
             affordance is honest about where the line is going and dishonest about \
             whether it goes. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("the release committed the move the copy previewed");

    Ok(None)
}

/// Ctrl+wheel at `at` until the application reports [`WANT_ZOOM`], and say what
/// it reached.
///
/// The application's own `zoom=` is read back after every spin rather than the
/// factor of a notch being assumed: that factor is egui's, and a harness that
/// counted notches would be asserting against its own arithmetic. Stops early
/// when a spin moves the number not at all, which is the ceiling or a wheel
/// landing somewhere other than the canvas — both cases the caller has to hear
/// about rather than spin through.
fn zoom_in_at(session: &Session, driver: &Driver, at: ScreenPoint) -> Result<f32> {
    let mut zoom = crate::checks::scale_aim::current_zoom(session)?;
    for _ in 0..ZOOM_SPINS {
        if zoom >= WANT_ZOOM {
            break;
        }
        driver.scroll_at_held(at, &[vk::CONTROL], 1, 1)?;
        session.settle(10);
        let now = crate::checks::scale_aim::current_zoom(session)?;
        if (now - zoom).abs() < f32::EPSILON {
            break;
        }
        zoom = now;
    }
    // The region raster is slower than the wheel; let it land before anything
    // is photographed.
    session.settle(60);
    crate::checks::scale_aim::current_zoom(session)
}

/// A document point as a screen point, through the mapping this check already
/// built.
///
/// [`crate::checks::text_selection::aim`] re-reads the trace on every call,
/// which is right for a check whose gestures change the layout between aims and
/// wrong here: every rectangle and both endpoints of the one gesture must come
/// from the SAME mapping, or the displacement the ghost is measured against is
/// not the displacement the pointer travelled.
fn aim_screen(mapping: &CanvasMapping, frame: &WindowFrame, at: DocPoint) -> Result<ScreenPoint> {
    Ok(frame.to_screen(mapping.doc_to_window(at)?))
}
