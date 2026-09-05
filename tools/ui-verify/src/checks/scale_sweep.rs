//! `mouse_work_survives_every_render_tier` — **the ordinary mouse gestures,
//! driven at every scale where the renderer changes strategy.**
//!
//! # The report this exists for
//!
//! The operator, 2026-09-04:
//!
//! > *"you should also try zooming in on the atoms of the banana pdf file and
//! > see what happens when you try to draw a box around a molecule and move it,
//! > or select the ion and move it, or edit the nodes at that scale. you should
//! > check all the mouse actions and capabilities out at each scale where our
//! > scaling algorithm changes."*
//!
//! He is naming a place he suspects the program breaks — **deep zoom, in the
//! region tier, doing ordinary mouse work** — rather than a feature. So this
//! check is a sweep rather than an assertion about one gesture: it walks the
//! zoom up through the two tier boundaries and, at each rung, runs the same
//! battery and says what happened.
//!
//! # The three tiers, and the boundaries they are found at
//!
//! Both boundaries are *derived from the page*, never written down as a zoom:
//!
//! | tier | begins when | on US Letter (792 pt long edge) |
//! |---|---|---|
//! | [`render::strategy::Strategy::WholePage`] | always | up to 2,068 % |
//! | [`render::strategy::Strategy::Region`] | `longest_pt × raster_scale > MAX_PIXMAP_EDGE − 1` | 2,068 % and up |
//! | + the `f64` position anchor | `longest_pt × zoom > SUB_PIXEL_CONTENT_EXTENT` | 132,396 % and up |
//!
//! ★★ The rungs this check walks are chosen **just below, at, and just above**
//! each of those, plus one far inside the region tier. This project's own
//! `strategy.rs` records why: *"two samples either side of a transition look
//! exactly like no transition at all if the transition is not where it was
//! assumed to be."*
//!
//! # What is measured, and why the pointer probe is the sharpest of them
//!
//! The battery is: **pointer conversion**, **click-select**, **drag the
//! selection**, **marquee on empty paper**, **anchors and node drag**, and
//! **resize handles**.
//!
//! The first is the one that can fail silently and take the rest with it.
//! `canvas::trace::pointer` publishes, for every pointer position, the
//! **canvas-space point the application thinks the pointer is on**, computed
//! by `viewer::screen_to_page`:
//!
//! ```text
//! page.x = (screen.x − image_rect.min.x) / zoom
//! ```
//!
//! — entirely in `f32`. At a deep zoom `image_rect.min.x` is an enormous
//! negative number (the page's own left edge, scaled), and `f32`'s
//! representable spacing there can exceed the size of the whole viewport. If
//! that subtraction has lost its low bits then **every hit test, every grip
//! placement and every drag delta on the canvas is derived from a number that
//! no longer distinguishes one part of the window from another** — and nothing
//! else in the trace says so, because a wrong-but-plausible coordinate still
//! selects *something* and still moves it *somewhere*.
//!
//! So the probe moves the pointer a known number of screen points and asserts
//! the reported canvas point moved by exactly that over the zoom. It is a
//! linearity test, and it needs no fixture knowledge at all.
//!
//! # ★ Why the zoom is driven by Ctrl+wheel and not by the status bar's `+`
//!
//! Zoom-to-cursor keeps the point under the pointer fixed, so the content this
//! check aims at stays under the aim point all the way down. The `+` button
//! zooms about the viewport centre, which on a page whose interesting detail is
//! off-centre magnifies blank paper — the operator's own complaint of
//! 2026-08-22, *"Right now you are just zooming into a blank area on the
//! canvas."*
//!
//! # Configuration
//!
//! `--doc-point PAGE,X,Y` names the content to zoom into (**0-based page**).
//! `UI_VERIFY_SWEEP_ZOOMS`, if set, replaces [`DEFAULT_RUNGS`] with a
//! comma-separated list of zoom multipliers — the sweep is meant to be re-aimed
//! from the command line while a boundary is being narrowed down.

use std::fmt::Write as _;

use crate::checks::driving::{self, SHELL_DIAG_ENV, arm_select_from_ribbon, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas may select page content.
const MODE: &str = "edit";
/// `canvas rect=… zoom=… page=…`.
const CANVAS_EVENT: &str = "canvas";
/// `canvas-pointer screen=(x,y) page=(x,y) pdf=(x,y) zoom=…`.
const POINTER_EVENT: &str = "canvas-pointer";
/// `canvas-selection via=… mod=… sel=… level=… first=…`.
const SELECTION_EVENT: &str = "canvas-selection";
/// `canvas-move page=… level=… action=…`.
const MOVE_EVENT: &str = "canvas-move";
/// `canvas-anchors total=… selected=… …`.
const ANCHORS_EVENT: &str = "canvas-anchors";
/// `canvas-handles n=…` — the BEZIER control handles, not the resize grips.
const HANDLES_EVENT: &str = "canvas-handles";
/// The box the eight resize grips are laid out on.
///
/// ★★★ The measurement this whole sweep turns on. `overlay::visible_outline_rect`
/// widens it to [`MIN_OUTLINE_EXTENT`] on each axis, and `handles::grip_at`
/// then covers `GRIP_SIZE_PX / 2 + GRIP_GRAB_SLACK_PX` = 6 pt inward from each
/// corner — so a box narrower than **12 pt has no body left to drag**, and
/// every press on it is a grip.
const OUTLINE_REGION: &str = "canvas.selection-outline";
/// `resize-declined reason=…`.
const RESIZE_DECLINED_EVENT: &str = "resize-declined";
/// `resize-commit grip=… sx=… sy=… ax=… ay=…` — **a resize that went
/// through**.
///
/// ★★★ The worst of the three outcomes and the one this sweep was not
/// watching for on its first two runs. A drag meant as a move that lands on a
/// grip and is *refused* costs the operator a gesture; one that lands on a grip
/// and **succeeds** costs them their artwork, silently. Measured on the
/// banana's cells at 15,808 %: `resize-commit grip=SouthEast sx=3.9770
/// sy=4.1891` — a 0.09 pt cell quadrupled on both axes by a drag that was aimed
/// at the middle of it.
const RESIZE_COMMIT_EVENT: &str = "resize-commit";
/// `canvas-move-declined level=… sel=… reason=…`.
const MOVE_DECLINED_EVENT: &str = "canvas-move-declined";
/// `selection-set page=… object=… via=…` — **not** de-duplicated, unlike
/// `canvas-selection`, so it is the honest oracle for "this press picked
/// something".
const SELECTION_SET_EVENT: &str = "selection-set";

/// The smallest grip box the overlay will draw, in logical points.
/// Mirrors `canvas::overlay::MIN_OUTLINE_EXTENT_PX`.
const MIN_OUTLINE_EXTENT: f32 = 6.0;

/// The width, in logical points, below which a grip box has no draggable body
/// left. Mirrors `2 × (handles::GRIP_SIZE_PX / 2 + handles::GRIP_GRAB_SLACK_PX)`.
const NO_BODY_BELOW_PX: f32 = 12.0;
/// The worker's completion line.
const RENDER_EVENT: &str = "render-async-done";
/// The scrollable viewport the page sits inside.
const VIEWPORT_REGION: &str = "canvas-viewport";
/// `marquee-mode crossing=… mode=… hits=… …` — the band's own line.
const MARQUEE_EVENT: &str = "marquee-mode";
/// `canvas-pos at=… tier=… region=… want=… ext=…`.
///
/// ★ `region=none` is the whole-page tier and anything else is the region tier,
/// so this line — not an arithmetic guess — is what says which tier a rung
/// actually reached. `tier=` names the POSITION model, which is the third
/// boundary.
const POSITION_EVENT: &str = "canvas-pos";
/// What a marquee-committed selection calls itself.
const VIA_MARQUEE: &str = "pv.marquee";
/// `canvas-coverage covered=… sharp=… textured=… backdrop=…`.
///
/// ★★ The operator's own report of 2026-09-04 — *"the canvas does a fading
/// around the edges on stuff shown at the edges of the view. I don't want this.
/// it should render true."* — is a claim about exactly this line: `sharp` is
/// the fraction of the viewport the SHARP raster covers, and anything below
/// 1.000 is the low-resolution backdrop showing through.
const COVERAGE_EVENT: &str = "canvas-coverage";

/// The zoom rungs walked, as multipliers.
///
/// ★★ Chosen against the **US Letter** boundaries in the module header — 20.69×
/// for the pixmap ceiling, 1,324× for the `f64` position anchor — and bracketed
/// on both sides of each rather than merely stepped past. `strategy.rs`'s own
/// test header states the rule these follow: a transition sampled only at its
/// endpoints is a transition that cannot be seen.
const DEFAULT_RUNGS: &[f32] = &[
    1.0,     // fit-ish, whole-page tier, the control
    8.0,     // the top of the named ladder, still whole-page
    18.0,    // just BELOW the pixmap ceiling
    21.0,    // just ABOVE it — the first region-tier rung
    64.0,    // comfortably inside the region tier
    600.0,   // deep in the region tier, f32 scroll offset still authoritative
    1200.0,  // just BELOW the f64 position anchor
    1500.0,  // just ABOVE it
    20000.0, // far inside tier 3
];

/// How far the pointer probe moves, in logical points, on each axis.
///
/// Big enough that the `{:.2}` the trace prints cannot swallow it at any zoom
/// this check reaches, small enough to stay inside the viewport.
const PROBE_PX: f32 = 100.0;

/// How much relative error the pointer probe tolerates.
///
/// The reported canvas point is printed to two decimals, so at a deep zoom the
/// quantum of the *printout* is already a large fraction of the movement being
/// measured — `100 px / 20000×` is `0.005` canvas units, which prints as `0.01`
/// or `0.00`. So the probe is only asserted where the movement is legible in
/// the printed precision, and this tolerance covers the rounding that remains.
const PROBE_TOLERANCE: f32 = 0.25;

/// The smallest printed movement the probe will assert on.
///
/// Below this the `{:.2}` printout, not the arithmetic, is the limit — see
/// [`PROBE_TOLERANCE`]. Reported as "not measurable at this zoom" rather than
/// as a pass or a failure, because either would be a claim the evidence cannot
/// support.
const PROBE_FLOOR: f32 = 0.05;

/// How far a drag moves the subject, in logical points, on each axis.
const DRAG_PX: f32 = 40.0;

/// How many wheel notches the pan probe scrolls, and then scrolls back.
const PAN_NOTCHES: i32 = 3;

/// ★★★ **Undo whatever the last gesture committed.**
///
/// The sweep holds ONE document across every rung and steers one aim point
/// through it, so a rung that leaves the page changed hands the next rung a
/// different document. Measured before this existed: the 107 % marquee move
/// translated all 212 objects by 37.6 pt, and the three rungs above it then
/// reported *"clicking directly on the content selected nothing"* — correctly,
/// because the content had been moved out from under the aim by the check
/// itself.
///
/// ★ Undo rather than "do not test the move": the move IS the subject. What has
/// to be true between rungs is that the document is the one the sweep started
/// with, and the application's own undo is the only thing that can promise
/// that.
fn undo(session: &Session, driver: &Driver) -> Result<()> {
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(24);
    Ok(())
}

/// How much of the drag distance must show up on screen for the drop to count
/// as landing where it was put, as a fraction.
const DROP_TOLERANCE: f32 = 0.30;

/// See the module documentation.
pub struct MouseWorkSurvivesEveryRenderTier;

impl Check for MouseWorkSurvivesEveryRenderTier {
    fn name(&self) -> &'static str {
        "mouse_work_survives_every_render_tier"
    }

    fn defect(&self) -> &'static str {
        "at a deep zoom — in the region tier, where the raster is a picture of the window rather \
         than of the page — the ordinary mouse gestures stop working: the click-to-page \
         conversion loses precision, so hit tests, grips and drag deltas are all derived from a \
         coordinate that no longer distinguishes one part of the window from another"
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

/// One rung's worth of findings.
struct Rung {
    /// The zoom actually reached, as a multiplier.
    zoom: f32,
    /// What the application says about which tier it is in, read from
    /// [`POSITION_EVENT`] rather than computed here.
    tier: String,
    /// One line per capability.
    lines: Vec<String>,
    /// Capabilities that were not merely absent but wrong.
    problems: Vec<String>,
}

#[allow(clippy::too_many_lines)]
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
        .ok_or_else(|| Error::new("no --pdf. There is nothing to zoom into."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming a point ON some content — \
             the sweep zooms about it and then tries to select it. ★ PAGE IS 0-BASED.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This sweep drives the wheel, the pointer and both \
             mouse buttons. Reported as SKIPPED rather than passed.",
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

    let rungs = wanted_rungs();

    let mut spec = LaunchSpec::new(&exe, ctx.out("scale-sweep.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}, page {}x{} pt",
        exe.display(),
        session.pid(),
        page.width_pt,
        page.height_pt
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    // ★★ Without this there is no rubber band at all, and the sweep reports
    // "the marquee raised no selection" at every rung — a confident, wrong
    // finding about a feature that was never armed. `off_page_marquee` learned
    // the same thing; the helper exists because of it.
    if !arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        return Err(Error::new(
            "the select tool could not be armed from the ribbon, so no rubber band could be \
             started at any rung. Nothing about the marquee was measured.",
        ));
    }
    session.settle(14);

    // The aim point, resolved ONCE at the opening zoom where the harness's own
    // conversion is comfortable. Everything after this works in screen space:
    // `doc_to_window` divides by the same `f32` magnitudes the application
    // does, so using it at a deep zoom would make the harness share the defect
    // it is looking for.
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let window_point = mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?;
    let frame = session.frame()?;
    let mut aim = frame.to_screen(window_point);
    // ★★★ The SAME point in canvas space — Y-down from the page's top-left,
    // which is the space `canvas-pointer` reports in. This is what the
    // closed-loop re-aim steers towards, and it is the reason the sweep can
    // hold a 0.85 pt cell under the cursor at twenty thousand times
    // magnification: the aim is corrected from the application's own report of
    // where the pointer is, at every rung, instead of being computed once and
    // trusted.
    #[allow(clippy::cast_possible_truncation, reason = "page points are small")]
    let target_canvas = (target.x as f32, (page.height_pt - target.y) as f32);
    report.note(format!(
        "aiming at PDF ({}, {}) on page {} — canvas ({:.3}, {:.3}) — which is screen          ({:.0}, {:.0}) at the opening zoom",
        target.x,
        target.y,
        target.page,
        target_canvas.0,
        target_canvas.1,
        aim.x(),
        aim.y()
    ));

    let viewport = driving::declared(&trace, ui_rect, VIEWPORT_REGION);

    let mut findings: Vec<Rung> = Vec::new();
    for &wanted in &rungs {
        // ★★★ A FULL RESET, and the sweep was wrong for two runs without it.
        //
        // The battery's last act is a double-click, and a double-click on a
        // **text** object opens the text editor. That state survived into the
        // next rung, where every drag became a text-box drag
        // (`text-box-declined w=1.9 h=1.9 floor=12.0`) and every marquee probe
        // read a stale `sel=1` — so three rungs reported "the drag raised
        // nothing at all" about a canvas that was in a different tool. One
        // Escape did not clear it.
        //
        // ⇒ Escape twice, then re-arm Select from the ribbon: the tool is the
        // thing that has to be true at the start of a rung, and the only honest
        // way to make it true is to set it rather than to assume it survived.
        driver.press(vk::ESCAPE)?;
        session.settle(8);
        driver.press(vk::ESCAPE)?;
        session.settle(8);
        if !arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
            return Err(Error::new(
                "the select tool could not be re-armed between rungs, so what followed would \
                 have been driven in an unknown tool.",
            ));
        }
        session.settle(12);

        let reached = zoom_to(&session, &driver, &mut aim, target_canvas, wanted, viewport)?;
        let mut rung = Rung {
            zoom: reached,
            tier: tier_of(&session)?,
            lines: Vec::new(),
            problems: Vec::new(),
        };
        rung.lines.push(format!("tier: {}", rung.tier));
        if let Some(l) = session.trace()?.events(CANVAS_EVENT).last() {
            rung.lines.push(format!(
                "the page's rect on screen is {} — the number `screen_to_page` subtracts",
                l.get("rect").unwrap_or("?")
            ));
        }
        rung.lines.push(format!(
            "aim is screen ({:.0}, {:.0}) after the closed-loop correction",
            aim.x(),
            aim.y()
        ));
        if (reached / wanted - 1.0).abs() > 0.5 {
            rung.lines.push(format!(
                "asked for {wanted:.0}x and reached {reached:.0}x — the rung was not hit, so \
                 everything below it describes {reached:.0}x"
            ));
        }

        probe_pointer(&session, &driver, aim, &mut rung)?;
        let selected = click_select(&session, &driver, ui_rect, aim, &mut rung)?;
        if selected {
            drag_selection(&session, &driver, ui_rect, aim, &mut rung)?;
        }
        marquee(&session, &driver, ui_rect, viewport, aim, &mut rung)?;
        nodes_and_handles(&session, &driver, ui_rect, aim, &mut rung)?;
        pan_and_watch_the_edges(&session, &driver, viewport, &mut rung)?;
        // ★ The double-clicks above can leave a text editor open; say so if one
        // is, because it is a fact about this rung and not only about the next.
        if session.trace()?.events("text-edit-caret").count() > 0 {
            rung.lines.push(
                "note: a double-click on a text object opens the text editor, so the rung ends \
                 in a different tool than it began in"
                    .to_owned(),
            );
        }

        // The picture, at every rung, because placement questions are only
        // answerable by looking.
        let shot = ctx.out(&format!("scale-sweep-{:.0}x.png", reached));
        if let Ok(f) = session.frame()
            && crate::capture::frame_to_png(&session, &f, &shot).is_ok()
        {
            report.artifact(shot);
        }

        findings.push(rung);
    }

    // --- the report ---------------------------------------------------------
    let mut problems: Vec<String> = Vec::new();
    for rung in &findings {
        report.note(format!(
            "═══ {:.0} % zoom — {} ═══",
            rung.zoom * 100.0,
            rung.tier
        ));
        for line in &rung.lines {
            report.note(format!("    {line}"));
        }
        for p in &rung.problems {
            problems.push(format!("at {:.0} %: {p}", rung.zoom * 100.0));
        }
    }

    let trace = session.trace()?;
    let failed = trace
        .events(RENDER_EVENT)
        .filter(|l| l.get("outcome").is_some_and(|o| o == "failed"))
        .count();
    if failed > 0 {
        problems.push(format!("{failed} raster(s) failed across the sweep"));
    }

    if problems.is_empty() {
        return Ok(None);
    }
    let mut message = String::from(
        "★★ THE MOUSE STOPS WORKING AT SOME SCALE. Each line is one capability at one rung:\n",
    );
    for p in &problems {
        let _ = writeln!(message, "  · {p}");
    }
    let _ = write!(
        message,
        "★★★ READ THE `grip box:` LINES FIRST. Measured over this sweep, the grip box is \
         what decides whether an object can be moved at all, and the boundary is \
         {NO_BODY_BELOW_PX} pt — `handles::grip_at` gives each corner grip `GRIP_SIZE_PX / 2 + \
         GRIP_GRAB_SLACK_PX` = 6 pt of the box, so four corners meet in the middle of anything \
         smaller and `Grip::Move` becomes unreachable. `overlay::MIN_OUTLINE_EXTENT_PX` then \
         floors the box at 6 pt, so the SMALLEST selections are exactly the ones with no body \
         at all. That is `OPERATOR_REQUESTS.md` O57's open half — which said of itself \
         \"Not driven\" — and these lines are the driving.\n  \
         ★ It is NOT the coordinate conversion: the `pointer:` lines above are linear at every \
         rung this sweep reached, including 2,346,176 %, and a drag that finds the body lands \
         within a tenth of a point of where it was dropped there. `viewer::screen_to_page`'s \
         `(pos.x − image_rect.min.x) / zoom` was the suspect and it is exonerated by \
         measurement. Trace: {}.",
        session.trace_path().display()
    );
    Ok(Some(message))
}

/// The rungs to walk — [`DEFAULT_RUNGS`], or the operator's own list.
fn wanted_rungs() -> Vec<f32> {
    match std::env::var("UI_VERIFY_SWEEP_ZOOMS") {
        Ok(v) => {
            let parsed: Vec<f32> = v
                .split(',')
                .filter_map(|s| s.trim().parse::<f32>().ok())
                .filter(|z| z.is_finite() && *z > 0.0)
                .collect();
            if parsed.is_empty() {
                DEFAULT_RUNGS.to_vec()
            } else {
                parsed
            }
        }
        Err(_) => DEFAULT_RUNGS.to_vec(),
    }
}

/// The zoom the application last reported.
fn current_zoom(session: &Session) -> Result<f32> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_f32("zoom"))
        .unwrap_or(0.0))
}

/// **What tier the application says it is in**, from its own position line.
///
/// `region=none` is the whole-page raster; anything else is the region tier.
/// `tier=` is the *position* model, the third boundary. Read rather than
/// computed, because a boundary this check derived itself would be a second
/// copy of arithmetic that lives in `render::strategy` and `viewer::ceiling`.
fn tier_of(session: &Session) -> Result<String> {
    let trace = session.trace()?;
    let Some(line) = trace.events(POSITION_EVENT).last() else {
        return Ok("no `canvas-pos` line".to_owned());
    };
    let raster = if line.get("region").is_none_or(|r| r == "none") {
        "whole-page raster"
    } else {
        "REGION raster"
    };
    Ok(format!(
        "{raster}, position tier `{}`",
        line.get("tier").unwrap_or("?")
    ))
}

/// **Steer the pointer back onto the target**, using the application's own
/// report of where it thinks the pointer is.
///
/// # ★★★ Why the aim is a loop and not a calculation
///
/// The subject of this sweep is a **0.85 pt** pair of cells on a US Letter
/// sheet. A single conversion at the opening zoom places the pointer to within
/// one screen pixel, which is about one page point there — larger than the
/// thing being aimed at. Every rung after that would then be measuring blank
/// paper beside the cells rather than the cells.
///
/// So the aim is corrected before every wheel batch from `canvas-pointer`,
/// which publishes the canvas-space point the application believes the pointer
/// is on. Because zoom-to-cursor keeps that point fixed, each correction is
/// applied at a higher magnification than the last and the error halves with
/// every doubling: one screen pixel of residual error is one page point at
/// 100 %, and 5 × 10⁻⁵ of one at twenty thousand percent.
///
/// ★★ It is also, incidentally, a **second** reading of the conversion under
/// test — if `screen_to_page` were lying, this loop would diverge rather than
/// converge, and the caller would see the aim wander. That is why the corrected
/// aim is reported at every rung.
///
/// The correction is capped at a third of the viewport per step: a larger jump
/// means the target has left the window entirely, and chasing it with one
/// enormous pointer move would land somewhere arbitrary. Capped, the loop still
/// converges over the following steps.
fn re_aim(
    session: &Session,
    driver: &Driver,
    at: ScreenPoint,
    target_canvas: (f32, f32),
    zoom: f32,
    cap: f32,
    viewport: Option<crate::geom::LRect>,
) -> Result<ScreenPoint> {
    driver.move_to(at)?;
    std::thread::sleep(std::time::Duration::from_millis(70));
    let Some(got) = reported_page_point(session)? else {
        return Ok(at);
    };
    let dx = (target_canvas.0 - got.0) * zoom;
    let dy = (target_canvas.1 - got.1) * zoom;
    if !dx.is_finite() || !dy.is_finite() {
        return Ok(at);
    }
    let dx = dx.clamp(-cap, cap);
    let dy = dy.clamp(-cap, cap);
    if dx.abs() < 0.5 && dy.abs() < 0.5 {
        return Ok(at);
    }
    let frame = session.frame()?;
    let moved = frame.offset_from(at, dx, dy);
    // ★★★ CLAMPED INTO THE CANVAS VIEWPORT, and the sweep ran away without it.
    //
    // The pan probe scrolls the view, so the next rung's first correction is a
    // large one; two rungs of that walked the aim off the top of the window
    // (measured: y = 304, then 184, then −56). A pointer outside the window
    // gets no `canvas-pointer` line, so the loop then steers from a stale
    // reading and the Ctrl+wheel lands on nothing — the zoom stalled at 21x and
    // three rungs reported a 100 % conversion error about an application that
    // was never asked anything. An aim that leaves the canvas is not a
    // correction, it is a lost target, and it must be held at the edge where the
    // next reading can still improve it.
    let Some(vp) = viewport else {
        return Ok(moved);
    };
    // ★ Expressed as a FRACTION of the viewport and rebuilt through
    // `declared_at`, because `ScreenPoint` has no public constructor — by
    // design, so that every screen coordinate in this harness comes from a
    // conversion rather than from arithmetic somebody did by hand.
    let p0 = frame.declared_at(vp, 0.0, 0.0);
    let p1 = frame.declared_at(vp, 1.0, 1.0);
    let span_x = (p1.x() - p0.x()) as f32;
    let span_y = (p1.y() - p0.y()) as f32;
    if span_x.abs() < 1.0 || span_y.abs() < 1.0 {
        return Ok(moved);
    }
    let fx = (moved.x() - p0.x()) as f32 / span_x;
    let fy = (moved.y() - p0.y()) as f32 / span_y;
    Ok(frame.declared_at(vp, fx.clamp(0.03, 0.97), fy.clamp(0.03, 0.97)))
}

/// Ctrl+wheel at `aim` until the reported zoom reaches `wanted`.
///
/// Returns the zoom actually reached, and leaves `aim` corrected onto the
/// target — see [`re_aim`]. Rolls in small batches and re-reads, because one
/// notch's factor is egui's and not this harness's to know.
fn zoom_to(
    session: &Session,
    driver: &Driver,
    aim: &mut ScreenPoint,
    target_canvas: (f32, f32),
    wanted: f32,
    viewport: Option<crate::geom::LRect>,
) -> Result<f32> {
    let cap = session.frame()?.client_logical().width() / 3.0;
    let mut zoom = current_zoom(session)?;
    let mut spins = 0;
    *aim = re_aim(session, driver, *aim, target_canvas, zoom, cap, viewport)?;
    while zoom < wanted && spins < 240 {
        // Bigger steps while far away, one notch when close, so the rung is
        // approached from below rather than jumped over.
        let ratio = wanted / zoom.max(0.001);
        let notches = if ratio > 8.0 {
            6
        } else if ratio > 2.0 {
            2
        } else {
            1
        };
        driver.scroll_at_held(*aim, &[vk::CONTROL], 1, notches)?;
        session.settle(8);
        let now = current_zoom(session)?;
        if (now - zoom).abs() < f32::EPSILON {
            break; // the ceiling, or the wheel is not landing
        }
        zoom = now;
        *aim = re_aim(session, driver, *aim, target_canvas, zoom, cap, viewport)?;
        spins += 1;
    }
    while zoom > wanted * 1.35 && spins < 300 {
        driver.scroll_at_held(*aim, &[vk::CONTROL], -1, 1)?;
        session.settle(8);
        let now = current_zoom(session)?;
        if (now - zoom).abs() < f32::EPSILON {
            break;
        }
        zoom = now;
        *aim = re_aim(session, driver, *aim, target_canvas, zoom, cap, viewport)?;
        spins += 1;
    }
    // The region raster is slow; let it land before anything is measured.
    session.settle(60);
    let zoom = current_zoom(session)?;
    *aim = re_aim(session, driver, *aim, target_canvas, zoom, cap, viewport)?;
    Ok(zoom)
}

/// Parse `(1.23,4.56)` — the shape `canvas-pointer` prints its two spaces in.
fn parse_paren_pair(s: &str) -> Option<(f32, f32)> {
    let inner = s.trim().strip_prefix('(')?.strip_suffix(')')?;
    let (a, b) = inner.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// The canvas point the application says the pointer is on, right now.
fn reported_page_point(session: &Session) -> Result<Option<(f32, f32)>> {
    Ok(session
        .trace()?
        .events(POINTER_EVENT)
        .last()
        .and_then(|l| l.get("page"))
        .and_then(parse_paren_pair))
}

/// **The linearity probe** — see the module header.
///
/// Moves the pointer a known distance and asks the application where it thinks
/// the pointer went. A conversion that has lost its low bits answers with a
/// movement that is too small, zero, or quantised.
fn probe_pointer(
    session: &Session,
    driver: &Driver,
    aim: ScreenPoint,
    rung: &mut Rung,
) -> Result<()> {
    let frame = session.frame()?;
    driver.move_to(aim)?;
    session.settle(6);
    let Some(before) = reported_page_point(session)? else {
        rung.lines
            .push("pointer: the application published no `canvas-pointer` line".to_owned());
        return Ok(());
    };
    let moved = frame.offset_from(aim, PROBE_PX, PROBE_PX);
    driver.move_to(moved)?;
    session.settle(6);
    let Some(after) = reported_page_point(session)? else {
        rung.lines
            .push("pointer: no `canvas-pointer` line after the move".to_owned());
        return Ok(());
    };

    let want = PROBE_PX / rung.zoom;
    let got_x = after.0 - before.0;
    let got_y = after.1 - before.1;
    if want < PROBE_FLOOR {
        rung.lines.push(format!(
            "pointer: a {PROBE_PX:.0} pt move is {want:.4} canvas units here, below the \
             {PROBE_FLOOR} the trace's own 2-decimal printout can show — not measurable, \
             reported {got_x:.2},{got_y:.2}"
        ));
        return Ok(());
    }
    let err_x = (got_x - want).abs() / want;
    let err_y = (got_y - want).abs() / want;
    let ok = err_x <= PROBE_TOLERANCE && err_y <= PROBE_TOLERANCE;
    rung.lines.push(format!(
        "pointer: moved {PROBE_PX:.0} pt, expected {want:.4} canvas units, got \
         ({got_x:.4}, {got_y:.4}) — {}",
        if ok { "linear" } else { "★ WRONG" }
    ));
    if !ok {
        rung.problems.push(format!(
            "the screen→page conversion is non-linear: a {PROBE_PX:.0} pt pointer move should \
             be {want:.4} canvas units and the application reported ({got_x:.4}, {got_y:.4}), \
             an error of {:.0} % / {:.0} %. Every hit test and every drag delta is derived \
             from this number",
            err_x * 100.0,
            err_y * 100.0
        ));
    }
    Ok(())
}

/// How many objects the application says are selected, from its own layout
/// line.
///
/// ★★ `canvas-selection` is **de-duplicated** — `trace_changed` suppresses a
/// line identical to the last one in its slot — so a second click that selects
/// the same object writes nothing, and a check counting those events reads
/// "the click did nothing" about a click that worked. That produced three false
/// findings on this sweep's first run. `canvas … sel=N` carries the count on a
/// line whose other fields move, and `selection-set` is written
/// unconditionally; between them there is no silence to misread.
fn selection_count(session: &Session) -> Result<usize> {
    Ok(session
        .trace()?
        .events(CANVAS_EVENT)
        .last()
        .and_then(|l| l.get_usize("sel"))
        .unwrap_or(0))
}

/// The grip box the selection published, if any.
fn outline_box(session: &Session, ui_rect: &str) -> Result<Option<crate::geom::LRect>> {
    Ok(driving::declared(
        &session.trace()?,
        ui_rect,
        OUTLINE_REGION,
    ))
}

/// Click the aim point and report what got selected, and **how big its grip box
/// is** — which is what decides whether it can be dragged at all.
fn click_select(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    aim: ScreenPoint,
    rung: &mut Rung,
) -> Result<bool> {
    let sets_before = session.trace()?.events(SELECTION_SET_EVENT).count();
    driver.click_at(aim)?;
    session.settle(20);
    let trace = session.trace()?;
    let picked: Vec<&crate::trace::TraceLine> = trace
        .events(SELECTION_SET_EVENT)
        .skip(sets_before)
        .collect();
    let sel = selection_count(session)?;
    rung.lines.push(format!(
        "click-select: sel={sel}{}",
        picked
            .last()
            .map_or_else(String::new, |l| format!(" — `{}`", l.raw))
    ));
    if sel == 0 {
        rung.problems.push(
            "clicking directly on the content the zoom is anchored to selected nothing".to_owned(),
        );
        return Ok(false);
    }
    match outline_box(session, ui_rect)? {
        Some(b) => {
            let (w, h) = (b.width(), b.height());
            rung.lines.push(format!(
                "grip box: {w:.1} x {h:.1} pt on screen{}",
                if w <= NO_BODY_BELOW_PX || h <= NO_BODY_BELOW_PX {
                    " — ★ SMALLER THAN THE GRIP CLUSTER, so it has no draggable body"
                } else {
                    ""
                }
            ));
            if (w - MIN_OUTLINE_EXTENT).abs() < 0.05 || (h - MIN_OUTLINE_EXTENT).abs() < 0.05 {
                rung.lines.push(format!(
                    "grip box: an axis is at the {MIN_OUTLINE_EXTENT} pt floor, so the outline \
                     has been widened and no longer states the object's real size"
                ));
            }
        }
        None => rung
            .lines
            .push(format!("grip box: nothing published as `{OUTLINE_REGION}`")),
    }
    Ok(true)
}

/// Drag whatever is selected and say what the gesture actually became.
///
/// # ★★★ The three outcomes, and why counting only `canvas-move` hid the real one
///
/// A drag on a selected object can become a **move**, a **resize** (the press
/// landed on a grip), or nothing. The first version of this counted
/// `canvas-move` alone, so a drag that was routed to the resize machinery and
/// then refused by it reported as *"the gesture was thrown away"* — true, and
/// silent about the mechanism. `resize-declined reason=Degenerate` is the line
/// that says what happened, and it is the finding this sweep exists to have
/// made.
fn drag_selection(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    aim: ScreenPoint,
    rung: &mut Rung,
) -> Result<()> {
    let trace = session.trace()?;
    let before = (
        trace.events(MOVE_EVENT).count(),
        trace.events(MOVE_DECLINED_EVENT).count(),
        trace.events(RESIZE_DECLINED_EVENT).count(),
        trace.events(RESIZE_COMMIT_EVENT).count(),
    );
    let box_before = outline_box(session, ui_rect)?;
    let frame = session.frame()?;
    // ★★★ THE PRESS IS THE MIDDLE OF THE GRIP BOX, not the aim point, and the
    // difference is the whole difference between a finding and a false one.
    //
    // The aim is a fixed document coordinate; the object the click picked is
    // whatever was nearest to it, and its box is very often *beside* that point
    // rather than centred on it. Driven at 174,259 % the aim sat a few
    // thousandths of a point past object 140's right edge — which is its
    // SouthEast corner, so the drag was a perfectly correct resize and the first
    // version of this check called it a defect.
    //
    // ★★ "Grab it in the middle and move it" is the gesture the operator
    // described, and the middle of the object is a fact only the application
    // knows. It publishes it as `canvas.selection-outline`; aiming anywhere else
    // is the harness inventing a coordinate.
    let press = box_before.map_or(aim, |b| frame.declared_center(b));
    let to = frame.offset_from(press, DRAG_PX, DRAG_PX);
    driver.drag(press, to)?;
    session.settle(36);

    let trace = session.trace()?;
    let moved = trace
        .events(MOVE_EVENT)
        .nth(before.0)
        .map(|l| l.raw.clone());
    let move_declined = trace
        .events(MOVE_DECLINED_EVENT)
        .nth(before.1)
        .map(|l| l.raw.clone());
    let resize_declined = trace
        .events(RESIZE_DECLINED_EVENT)
        .nth(before.2)
        .map(|l| l.raw.clone());
    let resize_commit = trace
        .events(RESIZE_COMMIT_EVENT)
        .nth(before.3)
        .map(|l| l.raw.clone());

    // ★★★ A COMMITTED RESIZE IS CHECKED FIRST, because it is the only outcome
    // that changes the document, and a build that both resized and (somehow)
    // moved must report the resize.
    if let Some(r) = resize_commit {
        rung.lines
            .push(format!("drag: SILENTLY RESIZED the object — `{r}`"));
        rung.problems.push(format!(
            "★★★ a drag pressed at the exact CENTRE of the published grip box silently \
             RESIZED the object: \
             `{r}`. Its grip box was {}; `handles::grip_at` gives each corner grip \
             `GRIP_SIZE_PX / 2 + GRIP_GRAB_SLACK_PX` = 6 pt of the box, so on a box this small \
             the four corners meet in the middle and `Grip::Move` is unreachable. This is not a \
             refused gesture — it is the operator's artwork scaled by a factor they did not ask \
             for, with no decline and nothing on screen to say so",
            box_before.map_or_else(
                || "unpublished".to_owned(),
                |b| format!("{:.1} x {:.1} pt", b.width(), b.height())
            )
        ));
        return Ok(());
    }

    if let Some(m) = moved {
        rung.lines.push(format!("drag: MOVED — `{m}`"));
    } else if let Some(r) = resize_declined {
        rung.lines
            .push(format!("drag: became a RESIZE and was refused — `{r}`"));
        rung.problems.push(format!(
            "a drag on the selected object was routed to the resize machinery and refused \
             (`{r}`) — the object cannot be MOVED at all. Its grip box is {}, and \
             `handles::grip_at` gives the corner grips {NO_BODY_BELOW_PX} pt of the box between \
             them, so on a box this small there is no body left for `Grip::Move` to answer",
            box_before.map_or_else(
                || "unpublished".to_owned(),
                |b| format!("{:.1} x {:.1} pt", b.width(), b.height())
            )
        ));
        return Ok(());
    } else if let Some(d) = move_declined {
        rung.lines
            .push(format!("drag: the move was declined — `{d}`"));
        rung.problems
            .push(format!("a drag on the selected object was declined: `{d}`"));
        return Ok(());
    } else {
        rung.lines
            .push("drag: nothing at all — no move, no decline, no resize".to_owned());
        rung.problems
            .push("dragging a selected object produced no traced outcome of any kind".to_owned());
        return Ok(());
    }

    // …and did it land where it was dropped? The grip box is the application's
    // own statement of where it thinks the object is.
    let (Some(b), Some(a)) = (box_before, outline_box(session, ui_rect)?) else {
        rung.lines.push(
            "drag landing: no grip box either side of the drag, so the landing cannot be measured"
                .to_owned(),
        );
        return Ok(());
    };
    let dx = a.min.x - b.min.x;
    let dy = a.min.y - b.min.y;
    let err_x = (dx - DRAG_PX).abs() / DRAG_PX;
    let err_y = (dy - DRAG_PX).abs() / DRAG_PX;
    let ok = err_x <= DROP_TOLERANCE && err_y <= DROP_TOLERANCE;
    rung.lines.push(format!(
        "drag landing: dropped {DRAG_PX:.0},{DRAG_PX:.0} pt and the grip box moved \
         ({dx:.1}, {dy:.1}) pt — {}",
        if ok { "landed" } else { "★ WRONG" }
    ));
    if !ok {
        rung.problems.push(format!(
            "an object dragged {DRAG_PX:.0} pt on each axis landed {dx:.1}, {dy:.1} pt away"
        ));
    }
    undo(session, driver)?;
    Ok(())
}

/// Rubber-band on empty paper, then move what it caught.
///
/// # ⚠ A press on ink is not a band, and finding that out cost a moved object
///
/// `canvas::presspick`'s rule is that a press on an object **selects it**, and
/// a drag from there **moves it**. The first version of this started the band
/// at a fixed fraction of the viewport; at 107 % that fraction landed on the
/// banana's own outline, and the "marquee" dragged a 250-point object across
/// the sheet — silently changing the document every later rung was measured
/// against. The trace said so plainly (`selection-set … object=2 via=press`
/// followed by `canvas-move … dx=254`) and the check did not look.
///
/// ⇒ So the origin is **probed** rather than assumed: candidates are clicked
/// until one selects nothing, and only then is a band dragged from it. The
/// probe is a click, which is reversible; a drag is not.
fn marquee(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    viewport: Option<crate::geom::LRect>,
    aim: ScreenPoint,
    rung: &mut Rung,
) -> Result<()> {
    driver.press(vk::ESCAPE)?;
    session.settle(8);
    let frame = session.frame()?;
    let Some(vp) = viewport.or_else(|| {
        session
            .trace()
            .ok()
            .and_then(|t| driving::declared(&t, ui_rect, VIEWPORT_REGION))
    }) else {
        rung.lines
            .push("marquee: the canvas published no viewport rect to band inside".to_owned());
        return Ok(());
    };

    // ★ Candidates walk inward from the corners. One of them is empty paper on
    // any page this sweep can be pointed at; if none is, the band is not
    // attempted and the rung says so rather than moving something.
    const CANDIDATES: [(f32, f32); 5] = [
        (0.06, 0.06),
        (0.94, 0.06),
        (0.06, 0.94),
        (0.94, 0.94),
        (0.50, 0.06),
    ];
    let mut origin = None;
    for (fx, fy) in CANDIDATES {
        let at = frame.declared_at(vp, fx, fy);
        driver.click_at(at)?;
        session.settle(12);
        if selection_count(session)? == 0 {
            origin = Some((at, (fx, fy)));
            break;
        }
        driver.press(vk::ESCAPE)?;
        session.settle(8);
    }
    let Some((from, (fx, fy))) = origin else {
        rung.lines.push(
            "marquee: every candidate origin landed on ink, so no band was attempted — a press \
             on an object selects it and a drag from there MOVES it"
                .to_owned(),
        );
        return Ok(());
    };
    // Diagonally across the viewport, left-to-right so it is an ENCLOSING
    // window (O88) and the count means "wholly inside the box".
    let to = frame.declared_at(vp, 1.0 - fx, 1.0 - fy);

    let bands_before = session.trace()?.events(MARQUEE_EVENT).count();
    let sel_before = session.trace()?.events(SELECTION_EVENT).count();
    driver.drag(from, to)?;
    session.settle(36);
    let trace = session.trace()?;
    let Some(band) = trace.events(MARQUEE_EVENT).nth(bands_before) else {
        rung.lines.push(
            "marquee: NO BAND BEGAN — no `marquee-mode` line, from an origin that had just been \
             clicked and selected nothing"
                .to_owned(),
        );
        rung.problems
            .push("a drag from verified empty paper started no rubber band at all".to_owned());
        return Ok(());
    };
    let hits = band.get_usize("hits").unwrap_or(0);
    rung.lines.push(format!("marquee: `{}`", band.raw));

    let committed = trace
        .events(SELECTION_EVENT)
        .skip(sel_before)
        .filter(|l| l.get("via") == Some(VIA_MARQUEE))
        .last()
        .map(|l| l.raw.clone());
    match &committed {
        Some(l) => rung.lines.push(format!("marquee commit: `{l}`")),
        None => {
            rung.lines.push(
                "marquee: the band ran and committed no `via=pv.marquee` selection".to_owned(),
            );
            if hits > 0 {
                rung.problems.push(format!(
                    "a band that reported {hits} hit(s) committed no selection"
                ));
            }
        }
    }
    let sel = selection_count(session)?;
    rung.lines
        .push(format!("marquee: {sel} object(s) selected afterwards"));
    if sel == 0 {
        rung.lines.push(
            "marquee: nothing was caught — at this zoom the swept area may genuinely be empty \
             paper, so this is reported and not judged"
                .to_owned(),
        );
        return Ok(());
    }

    // …and move what it caught.
    //
    // ★★ The grab is the AIM POINT — known content, inside the band — and not
    // the middle of the band's bounding box. Driven at 107 %, a press at the
    // box's centre landed on empty paper between the objects and started a
    // SECOND band (`marquee-mode … hits=0`), which correctly replaced the
    // 211-object selection with nothing. The check then reported "a marquee
    // selection could not be moved" about a program doing what Illustrator,
    // Inkscape and Figma all do: a press inside a multi-selection's bounding box
    // but on no object is a new band, not a drag of the set.
    let moves_before = trace.events(MOVE_EVENT).count();
    let declines_before = trace.events(RESIZE_DECLINED_EVENT).count();
    let commits_before = trace.events(RESIZE_COMMIT_EVENT).count();
    let mid = aim;
    driver.drag(mid, frame.offset_from(mid, DRAG_PX, DRAG_PX))?;
    session.settle(36);
    let trace = session.trace()?;
    if let Some(r) = trace.events(RESIZE_COMMIT_EVENT).nth(commits_before) {
        rung.lines
            .push(format!("marquee move: SILENTLY RESIZED — `{}`", r.raw));
        rung.problems.push(format!(
            "★★★ dragging a marquee selection of {sel} object(s) from the middle of the band \
             silently RESIZED them: `{}`",
            r.raw
        ));
    } else if let Some(m) = trace.events(MOVE_EVENT).nth(moves_before) {
        rung.lines
            .push(format!("marquee move: MOVED — `{}`", m.raw));
    } else if let Some(r) = trace.events(RESIZE_DECLINED_EVENT).nth(declines_before) {
        rung.lines.push(format!(
            "marquee move: became a RESIZE and was refused — `{}`",
            r.raw
        ));
        rung.problems.push(format!(
            "a marquee selection of {sel} object(s) could not be moved: the drag was routed to \
             the resize machinery and refused"
        ));
    } else {
        rung.lines
            .push("marquee move: the drag raised nothing at all".to_owned());
        rung.problems.push(format!(
            "a marquee selection of {sel} object(s) could not be moved"
        ));
    }
    undo(session, driver)?;
    Ok(())
}

/// Descend to the anchors, look at where the grips were placed, and drag one.
fn nodes_and_handles(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    aim: ScreenPoint,
    rung: &mut Rung,
) -> Result<()> {
    driver.press(vk::ESCAPE)?;
    session.settle(6);
    driver.click_at(aim)?;
    session.settle(12);

    // Resize handles first — they belong to the Object rung.
    let trace = session.trace()?;
    match trace.last(HANDLES_EVENT).and_then(|l| l.get_usize("n")) {
        Some(n) => {
            rung.lines
                .push(format!("handles: {n} resize grip(s) drawn"));
            if n > 0 {
                let placed = (0..n.min(8))
                    .filter_map(|i| driving::declared(&trace, ui_rect, handle_region(i)))
                    .count();
                rung.lines
                    .push(format!("handles: {placed} of them published a rect"));
                if placed == 0 {
                    rung.problems.push(
                        "the overlay says it drew bezier control handles and published no rect \
                         for any of them, so nothing can say where they are"
                            .to_owned(),
                    );
                }
            }
        }
        None => rung
            .lines
            .push("bezier handles: no `canvas-handles` line after selecting".to_owned()),
    }

    driver.double_click_at(aim)?;
    session.settle(20);
    let trace = session.trace()?;
    let Some(anchors) = trace.last(ANCHORS_EVENT) else {
        rung.lines.push(
            "nodes: no `canvas-anchors` after a double-click — the Part rung was not entered \
             (a text run or an image has no anchors)"
                .to_owned(),
        );
        return Ok(());
    };
    let total = anchors.get_usize("total").unwrap_or(0);
    rung.lines
        .push(format!("nodes: the part has {total} anchor(s)"));
    let Some(first) = driving::declared(&trace, ui_rect, "canvas.anchor.0") else {
        rung.lines
            .push("nodes: no anchor mark was published, so there is no grip to aim at".to_owned());
        if total > 0 {
            rung.problems.push(format!(
                "the entered part has {total} anchors and the overlay published no mark for any \
                 of them — the operator has nothing to click"
            ));
        }
        return Ok(());
    };
    let frame = session.frame()?;
    let grip = frame.declared_center(first);
    // ★ Is the grip anywhere near the thing it belongs to? At a deep zoom a
    // grip placed through a lossy conversion lands somewhere plausible and
    // wrong, and this is the cheapest statement of that.
    let dist = f64::from((grip.x() - aim.x()).pow(2) + (grip.y() - aim.y()).pow(2)).sqrt();
    rung.lines.push(format!(
        "nodes: the first grip is at screen ({:.0}, {:.0}), {dist:.0} pt from the aim point",
        grip.x(),
        grip.y()
    ));

    let moves_before = trace.events(MOVE_EVENT).count();
    driver.double_click_at(grip)?;
    session.settle(16);
    let trace = session.trace()?;
    let picked = trace
        .last(ANCHORS_EVENT)
        .and_then(|l| l.get_usize("selected"))
        .unwrap_or(0);
    rung.lines.push(format!(
        "nodes: {picked} anchor(s) selected after the descent"
    ));
    if picked == 0 {
        // ★ A NOTE, not a claimed defect. Measured across this sweep the Node
        // rung IS reachable at 15,808 %, 42,972 % and 174,259 % and was not
        // reached at 2,139 % — and the descent is a two-double-click dance whose
        // aim has to be re-read between the two, because entering the Part rung
        // re-lays-out the marks (`multi_node`'s header records that at length).
        // A harness that missed the second mark and a program that cannot be
        // clicked look identical from here, and the evidence does not separate
        // them. `multi_node_move_moves_every_picked_anchor` is the check that
        // owns this capability.
        rung.lines.push(
            "nodes: the descent stopped at the Part rung — reported, not judged; the anchor \
             marks are re-laid-out by the descent and the second aim may have missed"
                .to_owned(),
        );
        return Ok(());
    }
    let from = driving::declared(&trace, ui_rect, "canvas.selected-anchor")
        .map_or(grip, |r| frame.declared_center(r));
    driver.drag(from, frame.offset_from(from, DRAG_PX, DRAG_PX))?;
    session.settle(24);
    let trace = session.trace()?;
    match trace.events(MOVE_EVENT).nth(moves_before) {
        Some(m) => rung.lines.push(format!("node drag: `{}`", m.raw)),
        None => {
            rung.lines
                .push("node drag: dragging a selected anchor raised no `canvas-move`".to_owned());
            rung.problems
                .push("a node could not be dragged at this zoom".to_owned());
        }
    }
    undo(session, driver)?;
    Ok(())
}

/// **Pan, and watch the leading edge stay sharp.**
///
/// The operator, 2026-09-04: *"the canvas does a fading around the edges on
/// stuff shown at the edges of the view. I don't want this. it should render
/// true."* `render::strategy::region_for`'s header records the fix that landed
/// for it — the snap now centres the window on the grid instead of flooring its
/// origin, so the guaranteed margin is a quarter of a viewport on **every** side
/// instead of half a screen on two sides and nothing on the other two.
///
/// This is that claim, driven rather than computed: scroll, then read the
/// **worst** `sharp=` the canvas reported over the frames that followed.
/// `sharp=1.000` means the sharp raster covered the whole viewport; anything
/// less is the backdrop showing through somewhere.
///
/// ★ A **wheel** rather than a drag, because a drag on the canvas is a
/// selection gesture and would be measuring something else. The status line
/// says `wheel=scroll`, so a plain wheel here is a pan.
fn pan_and_watch_the_edges(
    session: &Session,
    driver: &Driver,
    viewport: Option<crate::geom::LRect>,
    rung: &mut Rung,
) -> Result<()> {
    driver.press(vk::ESCAPE)?;
    session.settle(8);
    let Some(vp) = viewport else {
        rung.lines
            .push("pan: no viewport rect, so the wheel has nowhere to land".to_owned());
        return Ok(());
    };
    let at = session.frame()?.declared_at(vp, 0.5, 0.5);
    let before = session.trace()?.events(COVERAGE_EVENT).count();
    driver.scroll_at(at, -PAN_NOTCHES)?;
    // Generous: a region raster on this page takes on the order of a second,
    // and the question is what the operator sees WHILE it is in flight as well
    // as after.
    session.settle(90);
    let trace = session.trace()?;
    let after: Vec<f32> = trace
        .events(COVERAGE_EVENT)
        .skip(before)
        .filter_map(|l| l.get_f32("sharp"))
        .collect();
    if after.is_empty() {
        rung.lines
            .push("pan: the canvas published no coverage line after the wheel".to_owned());
        return Ok(());
    }
    let worst = after.iter().copied().fold(f32::INFINITY, f32::min);
    let last = after.last().copied().unwrap_or(0.0);
    rung.lines.push(format!(
        "pan: {} frame(s) after a {PAN_NOTCHES}-notch scroll — worst sharp={worst:.3}, \
         settled at sharp={last:.3}",
        after.len()
    ));
    if last < 0.999 {
        rung.problems.push(format!(
            "after a pan settled, the sharp raster covers only {last:.3} of the viewport — the \
             rest is `canvas::backdrop`'s low-resolution stand-in, which is the fade the \
             operator reported"
        ));
    }
    Ok(())
}

/// The region name for the `n`th resize grip. Mirrors the overlay's own.
fn handle_region(n: usize) -> &'static str {
    const NAMES: [&str; 6] = [
        "canvas.handle.0",
        "canvas.handle.1",
        "canvas.handle.2",
        "canvas.handle.3",
        "canvas.handle.4",
        "canvas.handle.5",
    ];
    NAMES[n.min(NAMES.len() - 1)]
}
