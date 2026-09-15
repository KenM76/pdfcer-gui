//! `preview_width_ignores_zoom` — the driven proof of `OPERATOR_REQUESTS.md`
//! **O184**: the blue outline that follows your hand while you drag an object
//! is **the cursor**, and a cursor does not grow when the document is
//! magnified.
//!
//! # The report
//!
//!
//! > *"The live preview blue outlines that appear when we drag an object scale
//! > with zooming in and out of the page instead of being independent of zoom —
//! > at high zoom levels they end up being the width of the canvas. I think
//! > they keep the same size as the line widths they are moving and that is ok
//! > — but if we set the line width view to the one pixel width option the
//! > preview lines should also be affected by this setting."*
//!
//! Two rulings, and they are not the same ruling:
//!
//! | # | the ruling | what this check asserts |
//! |---|---|---|
//! | 1 | the preview may take its width from the object's own line width, but **zoom must not enter** | phases **A** and **B**: the same drag at two zooms produces the **same** preview width |
//! | 2 | the one-pixel line-weight view governs the preview too | phase **C**: with `view.line_weights` off, the preview is **1.00 px** |
//!
//! # ★★★ Why this cannot be a screenshot, and why it needs the trace
//!
//! Zoom-invariance is **a claim about two frames at two zooms.** No single
//! capture can carry it: a 12-pixel outline at 100 % and a 12-pixel outline at
//! 900 % are individually unremarkable, and it is only the *pair* that says
//! anything. So `canvas::shapes::draw` publishes the widest preview stroke it
//! painted, in device pixels, on every frame it draws one:
//!
//! ```text
//! canvas-shape-drawn shapes=3 segments=88 erased=3 zoom=8.412 real_widths=true widest_px=2.00
//! ```
//!
//! and this check reads that number at two zooms and asserts it did not move.
//!
//! ⚠ **The number is computed by the same function that sizes the stroke**
//! ([`StrokeRule::preview_px`]), which makes it a report of the decision rather
//! than an independent measurement of the pixels. That is a deliberate and
//! stated limitation: the alternative — counting blue pixels across a stroke in
//! a capture — cannot separate the preview from the erase band beneath it,
//! which is *supposed* to scale. What this check owns is the decision; what a
//! human owns is that the decision is drawn. ★ The `zoom=` field on the same
//! line is the guard against the degenerate reading: if `zoom` did not move
//! either, the check SKIPs rather than passing, because two readings at one
//! zoom assert nothing at all.
//!
//! # ★★ Why phase C can SKIP, and why that is honest rather than weak
//!
//! [`StrokeRule::preview_px`] floors the width at one device pixel — a hairline
//! (`0 w`, PDF 32000-1 §8.4.3.2) is one device pixel and a zero-width egui
//! stroke vanishes under antialiasing. So on an object whose stroke is already
//! at or below 1 pt, **the hairline view cannot be distinguished from the
//! normal view**, both answer 1.00, and an assertion that they differ would be
//! asserting something the correct build does not do.
//!
//! ⇒ Phase C therefore asks phase A what it measured first. If phase A already
//! read 1.00, phase C reports that it cannot measure this ruling **on this
//! object** and says which fixture would. This project has written down twice
//! that a check which cannot fail is not evidence; saying so out loud is the
//! only version of that which anybody ever reads.
//!
//! # The gesture, and why it is the top rung and not a node drag
//!
//! `shape_preview` descends two rungs and drags an **anchor**, because O63 was
//! about node editing. O184 is about *"when we drag an object"* — the plain
//! gesture, one click and a pull — so this check drives that and nothing else.
//! The two checks exercise different `MoveSubject` arms into the same painter,
//! which is worth having: `for_move_subject`'s object arm and its node arm
//! reach [`super::super`]'s stroke rule by different routes.
//!
//! # Fixture requirements
//!
//! `--pdf` and `--doc-point PAGE,X,Y` (★ **0-based page**) naming a point on a
//! **stroked vector object**. The sweep's default aim,
//! `fixtures/a1-titleblock.pdf` at `0,2000,320`, is the title block's linework
//! and is what this was measured on. A point on an image or on text selects
//! something with no stroked geometry, `for_move_subject` answers with an erase
//! and no shapes, and the check SKIPs naming that.

use crate::checks::driving::{
    self, SHELL_DIAG_ENV, arm_select_from_ribbon, click_mode_segment, declared,
    declared_or_in_overflow, list,
};
use crate::checks::scale_aim::zoom_to;
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::sys::vk;
use crate::trace::Trace;

/// The mode segment to press. A move is an edit.
const MODE: &str = "edit";

/// The painter's own report of the preview it drew.
const DRAWN: &str = "canvas-shape-drawn";

/// The canvas's per-frame line, which carries `sel=` and `zoom=`.
const CANVAS_EVENT: &str = "canvas";

/// The canvas viewport's published region, used to keep the zoom aim inside it.
const VIEWPORT_REGION: &str = "canvas-viewport";

/// The ribbon item that turns real line widths off.
const ITEM: &str = "ribbon.item.view.line_weights";

/// The ribbon tab that item lives on.
const VIEW_TAB: &str = "ribbon.tab.view";

/// How far to pull the object, in screen pixels.
///
/// ★ Far enough that the press is a drag and not a click, and short enough
/// that the object stays on the page at the deep rung — at 900 % a 40 px pull
/// is under 5 pt of document, which no fixture can fall off.
const DRAG_PX: f32 = 40.0;

/// The zoom phase B climbs to, as a multiplier.
///
/// ★★ Nine times, not the 20,000 % `scale_sweep` reaches, and the reason is
/// that this check wants a **large, reliable** zoom rather than an extreme one.
/// The defect multiplies the preview width by the zoom, so 9× turns a 2 px
/// outline into an 18 px one — an eight-sigma difference against a tolerance of
/// a hundredth of a pixel — while staying well inside the region tier where the
/// aim is stable and a drag still lands.
const DEEP_ZOOM: f32 = 9.0;

/// How much higher than the opening zoom phase B must actually get for the
/// comparison to mean anything.
///
/// ★★★ The guard against the degenerate pass. If the wheel does not reach the
/// canvas — which has happened, and is `zoom_gallery`'s report to make — both
/// phases measure the same zoom, the widths are trivially equal, and this check
/// would report a green it did not earn. Three times is far below
/// [`DEEP_ZOOM`]'s nine and far above any rounding.
const MIN_ZOOM_RATIO: f32 = 3.0;

/// How close two preview widths must be to count as the same number.
///
/// The trace prints two decimals, so anything under half a hundredth is below
/// the resolution of the report and 0.02 is two ticks of it. The defect being
/// guarded against is multiplicative by a factor of nine; there is no version
/// of it that hides inside a fiftieth of a pixel.
const WIDTH_TOLERANCE: f32 = 0.02;

/// See the module documentation.
pub struct ADragPreviewDoesNotThickenWithZoom;

impl Check for ADragPreviewDoesNotThickenWithZoom {
    fn name(&self) -> &'static str {
        "preview_width_ignores_zoom"
    }

    fn defect(&self) -> &'static str {
        "the live blue outline that follows the pointer while an object is dragged is sized in \
         DOCUMENT units, so it multiplies by the zoom — at high magnification a 6 pt outline is \
         two hundred device pixels across and hides the very geometry the operator zoomed in to \
         align against, which defeats the reason a preview is stroked and never filled. Or the \
         one-pixel line-weight view governs the page and not the preview, so the two disagree \
         about the same document"
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

/// What one drag produced: the widest preview stroke painted, and the zoom the
/// painter was working at when it painted it.
#[derive(Clone, Copy, Debug)]
struct Painted {
    /// `widest_px` — the widest preview stroke, in device pixels.
    widest: f32,
    /// `zoom` — device pixels per page point on that frame.
    zoom: f32,
    /// `real_widths` — whether the line-weight view was on.
    real_widths: bool,
    /// How many `canvas-shape-drawn` lines carried shapes.
    frames: usize,
}

/// The live zoom, as the canvas published it.
fn zoom_now(session: &Session) -> Result<f32> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0))
}

/// How many objects the canvas last reported as selected.
///
/// ★ Read from `canvas … sel=N` rather than by counting `selection-set` lines.
/// `scale_sweep`'s header carries why: the trace suppresses a line identical to
/// its predecessor, so a second click that picks the same object writes
/// nothing, and a check counting those events reads "the click did nothing"
/// about a click that worked.
fn selection_count(session: &Session) -> Result<usize> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_usize("sel"))
        .unwrap_or(0))
}

/// Read every `canvas-shape-drawn` line written **after** `mark`, and answer
/// with the widest preview stroke among the frames that actually carried
/// shapes.
///
/// ★★ `shapes=0` lines are skipped rather than counted as zero. A delete
/// preview publishes an erase and no shapes — `ShapePreview::is_empty` asks
/// about `shapes` precisely because of that — and folding a `widest_px=0.00`
/// from such a frame into the maximum would be harmless, but folding it into
/// the *count* would let a phase that drew nothing report that it drew.
fn painted_since(trace: &Trace, mark: usize) -> Option<Painted> {
    let mut out: Option<Painted> = None;
    for line in trace.events(DRAWN) {
        if line.lineno <= mark {
            continue;
        }
        if line.get_usize("shapes").unwrap_or(0) == 0 {
            continue;
        }
        let (Some(widest), Some(zoom)) = (line.get_f32("widest_px"), line.get_f32("zoom")) else {
            continue;
        };
        let real_widths = line.get("real_widths") == Some("true");
        out = Some(match out {
            None => Painted {
                widest,
                zoom,
                real_widths,
                frames: 1,
            },
            Some(prev) => Painted {
                widest: prev.widest.max(widest),
                zoom: prev.zoom.max(zoom),
                real_widths,
                frames: prev.frames + 1,
            },
        });
    }
    out
}

/// Press, pull, and report what the painter drew on the way.
///
/// ★ The press point is the aim — the same coordinate the click immediately
/// before selected the object from, so it is on the object by the same evidence
/// that produced the selection. `scale_sweep`'s `drag_selection` header carries
/// the run that established this: pressing at the *outline's centre* instead
/// made every rung report "the drag raised nothing", including the baseline,
/// and a uniform failure at every rung of a sweep is evidence about the probe.
fn drag_and_read(
    session: &Session,
    driver: &Driver,
    aim: crate::coords::ScreenPoint,
) -> Result<Option<Painted>> {
    let mark = session.trace()?.mark();
    let frame = session.frame()?;
    driver.drag(aim, frame.offset_from(aim, DRAG_PX, DRAG_PX))?;
    session.settle(30);
    Ok(painted_since(&session.trace()?, mark))
}

/// Ctrl+Z, and wait for it to land.
///
/// ★★ The document has to be the one this check started with between phases.
/// A drag that is not undone leaves the object [`DRAG_PX`] away from the aim,
/// and the next phase's click then selects whatever is now under that
/// coordinate — which reads as "clicking on the content selected nothing" and
/// is the check having moved its own target out from under itself.
fn undo(session: &Session, driver: &Driver) -> Result<()> {
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(24);
    Ok(())
}

#[allow(clippy::too_many_lines, reason = "three driven phases, each explained")]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check selects an object, drags it, zooms in \
             and drags it again. Reported as SKIPPED rather than passed.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // ---- the fixture is PINNED, and --pdf / --doc-point are ignored -------
    //
    // ★★★ Both rulings need a stroke WIDER THAN ONE POINT, and neither the
    // sweep's shared aim nor any aim a caller might pass can be relied on to
    // provide one. Driven on the sweep's own `a1-titleblock.pdf 0,2000,320`
    // this check SKIPPED - that coordinate is over text
    // (`marquee-mode hits=5 paths=0 text=5`), and a text run has no stroked
    // geometry to preview. Driven at `0,300,500` it PASSED at 1.00 px at both
    // zooms, which is ruling 1 measured and ruling 2 not measured at all:
    // `preview_px` floors at one device pixel, so a number already at the
    // floor cannot be lowered to it.
    //
    // ⇒ `fixture::heavy_stroke_target` holds the document, the point, and
    // the whole of that reasoning. A check whose subject cannot exist under an
    // arbitrary aim must not be steerable into a place where its subject does
    // not exist - it does not then report *my input is wrong*, it reports
    // something specific and believable about the program.
    let (pdf, target) = crate::fixture::heavy_stroke_target();
    if !pdf.is_file() {
        return Err(Error::new(format!(
            "the heavy-stroke fixture is not at {}. This check cannot measure a stroke rule \
             on a hairline; `fixture::heavy_stroke_target` says why, and what the two \
             thinner aims measured before it was pinned.",
            pdf.display()
        )));
    }
    report.note(format!(
        "--pdf and --doc-point are IGNORED: this check pins {} at page 0, 150, 260 — one \
         open path drawn `3.0 w`, which is the only pen in this fixture set heavy enough \
         for the one-pixel view to have something to cap",
        pdf.display()
    ));
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;
    // ★ `--page-size` is ignored for the same reason `--pdf` is: it describes
    // the document the caller passed, and this check does not open that
    // document. Honouring it would map every aim point through the geometry
    // of a file that is not on screen, and a mis-mapped aim does not fail
    // loudly - it selects whatever happens to be under the wrong coordinate
    // and then reports something specific and wrong about the preview.
    let page: PageGeometry = crate::fixture::page_geometry(&pdf).ok_or_else(|| {
        Error::new(format!(
            "cannot read a page size from the pinned fixture {}.",
            pdf.display()
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("preview-width.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
    session.settle(40);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    // ★★ Without this there is no select tool armed and the press becomes a
    // rubber band, which draws no shape preview at all — the check would then
    // report "nothing was painted" about a feature it never reached.
    if !arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        return Err(Error::new(
            "the select tool could not be armed from the ribbon, so a press on the object would \
             have started a marquee rather than a move. Nothing about the preview was measured.",
        ));
    }
    session.settle(14);

    // The aim, resolved once at the opening zoom where the harness's own
    // conversion is comfortable.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let mut aim = session.frame()?.to_screen(window_point);
    // The same point in canvas space — Y-down from the page's top-left, which
    // is the space `canvas-pointer` reports in and what the closed-loop re-aim
    // in `scale_aim` steers towards.
    #[allow(clippy::cast_possible_truncation, reason = "page points are small")]
    let target_canvas = (target.x as f32, (page.height_pt - target.y) as f32);
    let viewport = driving::declared(&trace, ui_rect, VIEWPORT_REGION);

    // ---- phase A: the opening zoom ----------------------------------------
    driver.click_at(aim)?;
    session.settle(20);
    if selection_count(&session)? == 0 {
        return Err(Error::new(format!(
            "clicking at --doc-point {},{},{} selected nothing, so there was no object to drag \
             and no preview to measure. That is a fact about the aim and the fixture, not about \
             the preview's width. Trace: {}.",
            target.page,
            target.x,
            target.y,
            session.trace_path().display()
        )));
    }
    let zoom_a_reported = zoom_now(&session)?;
    let Some(shallow) = drag_and_read(&session, &driver, aim)? else {
        return Err(Error::new(format!(
            "the object was selected and dragged and the painter never wrote a `{DRAWN}` line \
             carrying `shapes>0`. Either the aim is on an object with no stroked geometry (an \
             image, a text run — `for_move_subject` answers those with an erase and no shapes), \
             or the drag was routed somewhere other than a move. `shape_preview` is the check \
             that owns \"the preview was not built\"; this one SKIPs, because a width cannot be \
             measured on a stroke nobody drew. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ at the opening zoom the painter drew the preview across {} frame(s): widest stroke \
         {:.2} px, canvas zoom {:.3} (the canvas itself reported {:.3}), real widths {}",
        shallow.frames, shallow.widest, shallow.zoom, zoom_a_reported, shallow.real_widths
    ));
    undo(&session, &driver)?;

    // ---- phase B: the same drag, nine times magnified ----------------------
    let reached = zoom_to(
        &session,
        &driver,
        &mut aim,
        target_canvas,
        DEEP_ZOOM,
        viewport,
    )?;
    report.note(format!("climbed to {:.0} %", reached * 100.0));
    driver.click_at(aim)?;
    session.settle(20);
    if selection_count(&session)? == 0 {
        return Err(Error::new(format!(
            "after climbing to {:.0} % a click at the corrected aim selected nothing, so the \
             deep half of the comparison could not be driven. The closed-loop re-aim in \
             `scale_aim` is the thing to read — `aim_residual` is the number that says whether \
             the pointer is still on the target. SKIPPED rather than failed: this says nothing \
             about the preview's width. Trace: {}.",
            reached * 100.0,
            session.trace_path().display()
        )));
    }
    let Some(deep) = drag_and_read(&session, &driver, aim)? else {
        return Err(Error::new(format!(
            "the deep drag produced no `{DRAWN}` line with shapes, although the shallow one \
             did. The object is selected and the press is on it, so the likeliest cause is that \
             the drag was read as a resize — at {:.0} % the aim may be sitting on a corner grip \
             of an object whose whole box is now larger than the window. SKIPPED: no width was \
             measured at this zoom. Trace: {}.",
            reached * 100.0,
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ at {:.0} % the painter drew the preview across {} frame(s): widest stroke {:.2} px, \
         canvas zoom {:.3}",
        reached * 100.0,
        deep.frames,
        deep.widest,
        deep.zoom
    ));
    undo(&session, &driver)?;

    // ---- the guard against a pass nobody earned ---------------------------
    //
    // ★★★ Asked BEFORE the widths are compared. Two readings at one zoom make
    // the equality trivially true, and a check that cannot fail is not
    // evidence — this project has written that down three times.
    let ratio = if shallow.zoom > f32::EPSILON {
        deep.zoom / shallow.zoom
    } else {
        0.0
    };
    if ratio < MIN_ZOOM_RATIO {
        return Err(Error::new(format!(
            "the two drags were painted at zooms {:.3} and {:.3}, a ratio of {ratio:.2}, below \
             the {MIN_ZOOM_RATIO:.0}x this comparison needs. The wheel is not reaching the \
             canvas, or the page hit its raster ceiling before {:.0} %. SKIPPED, not passed: \
             the widths being equal across two readings at the SAME zoom asserts nothing. \
             `zoom_gallery` owns \"the wheel does not zoom\". Trace: {}.",
            shallow.zoom,
            deep.zoom,
            DEEP_ZOOM * 100.0,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the two drags really were {ratio:.1}x apart in zoom, so the comparison below is a \
         measurement rather than a tautology"
    ));

    // ---- ruling 1: zoom must not enter ------------------------------------
    let drift = (deep.widest - shallow.widest).abs();
    if drift > WIDTH_TOLERANCE {
        return Ok(Some(format!(
            "★★★ THE DRAG PREVIEW THICKENS WITH ZOOM — O184. The same object, dragged the same \
             way, was previewed with a {:.2} px outline at zoom {:.3} and a {:.2} px outline at \
             zoom {:.3} — {:.1}x wider for {ratio:.1}x the magnification, which is the operator's \
             own report: \"at high zoom levels they end up being the width of the canvas\". \
             \n\n\
             ★ This is not only what he asked for, it is what this module already argues for \
             itself: `canvas::shapes` says a preview is stroked and never filled because \"a \
             filled shape following the pointer would hide what is under it, and what is under \
             it is the page the operator is aligning against\". A stroke two hundred pixels wide \
             IS a fill — so a zoom-scaled preview defeats the reason the preview exists, at \
             exactly the zoom where alignment is the whole task. \
             \n\n\
             The fix is `StrokeRule::preview_px`, which must not multiply by `self.zoom`. Note \
             that `StrokeRule::erase_px` deliberately DOES: it covers ink a renderer actually \
             put on the texture, and that ink scaled. Trace: {}.",
            shallow.widest,
            shallow.zoom,
            deep.widest,
            deep.zoom,
            if shallow.widest > f32::EPSILON {
                deep.widest / shallow.widest
            } else {
                0.0
            },
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ the preview was {:.2} px wide at both zooms — drift {drift:.3} px across a \
         {ratio:.1}x magnification",
        shallow.widest
    ));

    // ---- ruling 2: the hairline view governs the preview too --------------
    //
    // ★★ Asked of phase A's number first. `preview_px` floors at one device
    // pixel, so on an object already at or under 1 pt the hairline view and the
    // normal view give the same answer and the correct build cannot make them
    // differ. Reporting that is the honest outcome; asserting a difference
    // would be asserting something untrue.
    if (shallow.widest - 1.0).abs() <= WIDTH_TOLERANCE {
        report.note(
            "⚠ the hairline ruling could NOT be measured on this object: its preview is already \
             1.00 px, because `preview_px` floors at one device pixel and this object's stroke \
             is at or under 1 pt. Turning line weights off cannot change a number that is \
             already at the floor, so phase C would be an assertion the correct build does not \
             satisfy. Aim --doc-point at a heavier stroke — a markup line or a title-block rule \
             — to measure it.",
        );
        return Ok(None);
    }

    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, VIEW_TAB).ok_or_else(|| {
        Error::new(format!(
            "no `{VIEW_TAB}` region, so the line-weights item could not be reached. Tabs \
             declared: {}.",
            list(&driving::declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let item = declared_or_in_overflow(&session, &driver, ui_rect, ITEM)?.ok_or_else(|| {
        Error::new(format!(
            "no `{ITEM}` on the View tab or in its overflow, so the second half of O184 could \
             not be driven. `line_weights` is the check that owns that control's reachability \
             and its failure should be read first. Items declared: {}.",
            list(&driving::declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.view."
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(40);

    // Re-select: clicking the ribbon did not clear the selection, but the
    // canvas has had two undos and a tab change since, and asserting on a
    // selection nobody re-established is how a check ends up measuring the
    // frame before the one it meant to.
    driver.click_at(aim)?;
    session.settle(20);
    if selection_count(&session)? == 0 {
        return Err(Error::new(format!(
            "after turning line weights off, a click at the aim selected nothing — so the \
             hairline half could not be driven. SKIPPED: nothing measured. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some(hairline) = drag_and_read(&session, &driver, aim)? else {
        return Err(Error::new(format!(
            "with line weights off the drag produced no `{DRAWN}` line with shapes, although \
             the same drag produced one twice with them on. SKIPPED: no width measured. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    undo(&session, &driver)?;

    if hairline.real_widths {
        return Err(Error::new(format!(
            "the `{ITEM}` item was pressed and the painter still reports `real_widths=true`, so \
             the toggle did not reach `doc.view.line_weights` before the drag — or the press \
             landed twice. SKIPPED rather than failed: this is a statement about the click, not \
             about the width. Trace: {}.",
            session.trace_path().display()
        )));
    }

    if (hairline.widest - 1.0).abs() > WIDTH_TOLERANCE {
        return Ok(Some(format!(
            "★★★ THE ONE-PIXEL LINE-WIDTH VIEW DOES NOT GOVERN THE PREVIEW — O184's second \
             ruling, in the operator's own words: \"if we set the line width view to the one \
             pixel width option the preview lines should also be affected by this setting\". \
             \n\n\
             With `view.line_weights` off the painter reports `real_widths=false` and still \
             draws the preview {:.2} px wide, where the page underneath it is being drawn at one \
             device pixel per stroke. The preview and the page are describing the same document \
             and disagreeing about it — and the preview is the one claiming to show what you \
             will get. \
             \n\n\
             `StrokeRule::preview_px` must return 1.0 when `real_widths` is false. ★ Note the \
             erase band has the same requirement for the opposite reason: the renderer put ONE \
             device pixel on the texture whatever the file said, so sizing the erase from the \
             file's line width under a hairline view paints a white band far wider than the ink \
             it is hiding. Trace: {}.",
            hairline.widest,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ and with the one-pixel view on, the same object previewed at {:.2} px instead of \
         {:.2} — the preview answers to the same setting the page does",
        hairline.widest, shallow.widest
    ));

    Ok(None)
}
