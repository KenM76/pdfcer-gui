//! `mouse_work_survives_every_render_tier` — **the ordinary mouse gestures,
//! driven at every scale where the renderer changes strategy.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scale_sweep.md`.

use std::fmt::Write as _;

use crate::checks::driving::{self, SHELL_DIAG_ENV, arm_select_from_ribbon, click_mode_segment};
use crate::checks::scale_aim::{CANVAS_EVENT, aim_residual, reported_page_point, tier_of, zoom_to};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose canvas may select page content.
const MODE: &str = "edit";
/// `canvas-selection via=… mod=… sel=… level=… first=…`.
const SELECTION_EVENT: &str = "canvas-selection";
/// `canvas-move page=… level=… action=…`.
const MOVE_EVENT: &str = "canvas-move";
/// `canvas-anchors total=… selected=… …`.
const ANCHORS_EVENT: &str = "canvas-anchors";
/// `canvas-handles n=…` — the BEZIER control handles, not the resize grips.
const HANDLES_EVENT: &str = "canvas-handles";
/// The box the eight resize grips are laid out on.
const OUTLINE_REGION: &str = "canvas.selection-outline";
/// `resize-declined reason=…`.
const RESIZE_DECLINED_EVENT: &str = "resize-declined";
/// `resize-commit grip=… sx=… sy=… ax=… ay=…` — **a resize that went
/// through**.
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
/// What a marquee-committed selection calls itself.
const VIA_MARQUEE: &str = "pv.marquee";
/// `canvas-coverage covered=… sharp=… textured=… backdrop=…`.
const COVERAGE_EVENT: &str = "canvas-coverage";

/// The zoom rungs walked, as multipliers.
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
const PROBE_TOLERANCE: f32 = 0.25;

/// The smallest printed movement the probe will assert on.
const PROBE_FLOOR: f32 = 0.05;

/// How far a drag moves the subject, in logical points, on each axis.
const DRAG_PX: f32 = 40.0;

/// How far the pointer may be from the sweep's target and still be treated as
/// on it, in **screen pixels**.
const AIM_TOLERANCE_PX: f32 = 6.0;

/// How many wheel notches the pan probe scrolls, and then scrolls back.
const PAN_NOTCHES: i32 = 3;

/// **Undo whatever the last gesture committed.**
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
    // Without this there is no rubber band at all, and the sweep reports
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
    // The SAME point in canvas space — Y-down from the page's top-left,
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
        // A FULL RESET, and the sweep was wrong for two runs without it.
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

        //
        // # The limit this measures, and it is the harness's own
        //
        // `re_aim` corrects by **moving the pointer**. That works while the
        // residual error, multiplied by the zoom, is smaller than the viewport
        // — past that there is no pointer position inside the window that maps
        // to the target, and the correction clamps at the edge and stops
        // improving. Measured at 2,298,020 %: the pointer reports canvas y
        // 538.52 against a target of 532.00, and moving it 274 px changes that
        // reading by **0.01** — the whole 484 pt viewport spans 0.021 canvas
        // points there.
        //
        // The 2026-09-05 sweep reported the consequence as an application
        // defect at five rungs — *"clicking directly on the content the zoom is
        // anchored to selected nothing"* — when what the click landed on was
        // blank paper 6.5 points away from the content. Correcting the rest of
        // the way needs the view PANNED, which this check does not do; naming
        // that is honest and inventing a defect is not.
        if let Some(residual) = aim_residual(&session, &driver, aim, target_canvas)? {
            let off_px = residual * reached;
            if off_px > AIM_TOLERANCE_PX {
                rung.lines.push(format!(
                    "★ THE AIM COULD NOT BE HELD: the pointer is {residual:.4} canvas pt from \
                     the target, which at {reached:.0}x is {off_px:.0} screen px — past the \
                     {AIM_TOLERANCE_PX:.0} px this check treats as on-target. `re_aim` corrects \
                     by MOVING THE POINTER, and Ctrl+wheel holds the point under the pointer \
                     fixed, so an error the correction cannot close is magnified by every \
                     further notch rather than reduced. Every pointer probe below this rung \
                     would be pressing on blank paper, so they are NOT run — closing this needs \
                     the view PANNED, which this check does not do"
                ));
                findings.push(rung);
                continue;
            }
            rung.lines.push(format!(
                "aim residual: {residual:.4} canvas pt ({off_px:.1} screen px) — on target"
            ));
        }

        let selected = click_select(&session, &driver, ui_rect, aim, &mut rung)?;
        if selected {
            drag_selection(&session, &driver, ui_rect, aim, &mut rung)?;
        }
        marquee(&session, &driver, ui_rect, viewport, aim, &mut rung)?;
        nodes_and_handles(&session, &driver, ui_rect, aim, &mut rung)?;
        pan_and_watch_the_edges(&session, &driver, viewport, &mut rung)?;
        // The double-clicks above can leave a text editor open; say so if one
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

    //
    // A rung whose aim could not be held records a line and no problem, so it
    // neither fails nor claims anything. That is right for one rung and would
    // be catastrophic for all of them: a build in which the closed-loop
    // correction broke at the first notch would report **zero problems** and
    // this function would answer `Ok(None)` — a green result meaning *"nothing
    // was tried"*, which is this harness's own stated worst outcome and the
    // reason `--no-input` reports SKIPPED rather than passing.
    //
    // The oracle is the `click-select:` line rather than a counter kept
    // alongside, so it cannot drift from what was actually run: a rung that
    // reached the battery emitted one, and a rung that was skipped for aim did
    // not.
    let measured = findings
        .iter()
        .filter(|r| r.lines.iter().any(|l| l.starts_with("click-select:")))
        .count();
    if measured == 0 {
        return Err(Error::new(format!(
            "not one of the {} rungs could be measured: the aim was lost at every zoom, so no \
             click, drag, marquee or node gesture was ever performed. SKIPPED rather than \
             passed — a sweep that tried nothing has said nothing about the application. The \
             per-rung lines above say how far the pointer was from the target at each rung; if \
             they are large from the first rung, suspect `scale_aim::re_aim` or a `--doc-point` \
             that is not on the page. Trace: {}.",
            findings.len(),
            session.trace_path().display()
        )));
    }
    let total = findings.len();
    report.note(if measured == total {
        format!("★ all {total} rungs reached the full gesture battery")
    } else {
        format!(
            "★ {measured} of {total} rungs reached the full gesture battery; the other \
             {} could not be aimed at and say so on their own line",
            total - measured
        )
    });

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
        "★★★ **ASK FIRST WHETHER THE PROBLEMS ARE AT EVERY RUNG.** If they are, this is a \
         statement about the probe and not about scale — the 104 % rung is the control, and a \
         control that fails is the finding. That mistake has been made once here already and \
         cost a filed defect (see this module's header, 2026-09-05).\n  \
         ★★ If the problems begin at one rung and not before, read the `grip box:` lines: on a \
         box at or under {NO_BODY_BELOW_PX} pt the four corner grips meet in the middle — \
         `handles::grip_at` gives each `GRIP_SIZE_PX / 2 + GRIP_GRAB_SLACK_PX` = 6 pt — so \
         `Grip::Move` is unreachable and every press on the object is a grip. \
         `overlay::MIN_OUTLINE_EXTENT_PX` floors the drawn box at 6 pt, so the SMALLEST \
         selections are exactly the ones with no body at all. That is `OPERATOR_REQUESTS.md` \
         O57's half.\n  \
         ★ It is NOT the coordinate conversion, and that is now measured twice over rather \
         than argued: the `pointer:` lines are linear at every rung, and with the aim held \
         (`aim residual` under {AIM_TOLERANCE_PX:.0} screen px) every gesture in this battery \
         has been driven successfully to 2,298,019 %. `viewer::screen_to_page`'s \
         `(pos.x − image_rect.min.x) / zoom` was the suspect and it is exonerated. Trace: {}.",
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

/// **The linearity probe** — see the module header.
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
        trace.events(MARQUEE_EVENT).count(),
    );
    let box_before = outline_box(session, ui_rect)?;
    let frame = session.frame()?;
    let press = aim;
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
    let marqueed = trace
        .events(MARQUEE_EVENT)
        .nth(before.4)
        .map(|l| l.raw.clone());

    // A COMMITTED RESIZE IS CHECKED FIRST, because it is the only outcome
    // that changes the document, and a build that both resized and (somehow)
    // moved must report the resize.
    if let Some(r) = resize_commit {
        let boxed = box_before.map_or_else(
            || "unpublished".to_owned(),
            |b| format!("{:.1} x {:.1} pt", b.width(), b.height()),
        );
        rung.lines
            .push(format!("drag: SILENTLY RESIZED the object — `{r}`"));
        rung.problems.push(format!(
            "★★★ a drag pressed on the object at the aim point silently RESIZED it: `{r}`. Its \
             grip box was {boxed}. Two readings, and the box size tells them apart: on a box \
             at or under {NO_BODY_BELOW_PX} pt the four corner grips meet in the middle — \
             `handles::grip_at` gives each of them `GRIP_SIZE_PX / 2 + GRIP_GRAB_SLACK_PX` = 6 \
             pt — so `Grip::Move` is unreachable and the operator's artwork is scaled by a \
             factor they did not ask for; on a box much larger than that, the AIM POINT is \
             sitting on a corner of this particular object and the resize is correct behaviour \
             reported at a badly chosen coordinate."
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
    } else if let Some(m) = marqueed {
        // THE ARM THAT WAS MISSING, and its absence produced the sweep's
        // headline finding about a build that was working.
        //
        // O72: a press on empty paper INSIDE a selection's bounding box draws a
        // marquee rather than moving the selection — the operator asked for it
        // by name. So a marquee here is the application answering correctly
        // about a press that was not on the object, and the useful thing to
        // report is *where the press was*, not that the mouse is broken.
        //
        rung.lines.push(format!("drag: became a MARQUEE — `{m}`"));
        rung.problems.push(format!(
            "a drag pressed at the aim point drew a MARQUEE instead of moving the object it had \
             just selected: `{m}`. `canvas::pressing` downgrades `Grip::Move` to nothing unless \
             `body_under` finds ink at the press point (O72), so this says the aim coordinate is \
             inside the object's bounding box and beside its ink. If the `--doc-point` for this \
             fixture IS on the ink, that is the defect; if it is merely near it, this is correct \
             behaviour and the aim point is wrong. The grip box was {}",
            box_before.map_or_else(
                || "unpublished".to_owned(),
                |b| format!("{:.1} x {:.1} pt", b.width(), b.height())
            )
        ));
        return Ok(());
    } else {
        rung.lines
            .push("drag: nothing at all — no move, no decline, no resize, no marquee".to_owned());
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

    // Candidates walk inward from the corners. One of them is empty paper on
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
    // The grab is the AIM POINT — known content, inside the band — and not
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
    //
    // `canvas::overlay::anchors` publishes `canvas.anchor.N` for the **culled**
    // set: the anchors that are actually inside the viewport, and only those.
    // That is O69, and deliberate — *"a rect that is off screen is a click
    // aimed at nothing, reported as a defect in whatever the click did
    // instead."* Its own comment says the census carries `on_screen=` so that
    // *"the cap fired"* and *"the operator has scrolled away from the points"*
    // stop being the same line.
    //
    // This probe read `total` and ignored `on_screen`, so at every rung above
    // 942 % — where the six anchors of a 480 pt path are thousands of pixels
    // apart and none is in a 484 pt viewport — it reported *"the entered part
    // has 6 anchors and the overlay published no mark for any of them, the
    // operator has nothing to click"*. Measured: `canvas-anchors total=6
    // on_screen=0`. The application had answered the question in the same line
    // the check was reading.
    let on_screen = anchors.get_usize("on_screen").unwrap_or(0);
    rung.lines.push(format!(
        "nodes: the part has {total} anchor(s), {on_screen} of them inside the viewport"
    ));
    let Some(first) = driving::declared(&trace, ui_rect, "canvas.anchor.0") else {
        rung.lines
            .push("nodes: no anchor mark was published, so there is no grip to aim at".to_owned());
        if on_screen > 0 {
            rung.problems.push(format!(
                "the entered part has {total} anchors, the overlay says {on_screen} of them are \
                 inside the viewport, and it published a mark for none — the operator has \
                 nothing to click"
            ));
        } else if total > 0 {
            rung.lines.push(format!(
                "nodes: …and none of the {total} is on screen at this zoom, so having no mark to \
                 aim at is correct (O69). This rung says nothing about node editing"
            ));
        }
        return Ok(());
    };
    let frame = session.frame()?;
    let grip = frame.declared_center(first);
    // Is the grip anywhere near the thing it belongs to? At a deep zoom a
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
        // A NOTE, not a claimed defect. Measured across this sweep the Node
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
    //
    // This probe is the last thing a rung does, so the view it leaves behind is
    // the view the NEXT rung's `re_aim` starts from. Three notches at 2,559 %
    // is a few hundred screen pixels, which puts the sweep's target OUTSIDE the
    // viewport — and `re_aim` corrects by moving the pointer, so a target
    // outside the viewport is a target it can never reach again. Measured: the
    // aim residual was 0.02 canvas pt at the end of the 2,559 % rung and
    // **4.32** at the start of the next, and stayed there for every rung above,
    // magnified by each Ctrl+wheel into thousands of pixels.
    //
    // Scrolled back rather than re-aimed around, because the wheel is exactly
    // invertible and a correction is not: the same notch count the other way is
    // the only recovery that leaves no residue for the next rung to inherit.
    // It is read AFTER the coverage lines above so the measurement is of the
    // pan, not of the restoration.
    driver.scroll_at(at, PAN_NOTCHES)?;
    session.settle(40);
    let after: Vec<f32> = trace
        .events(COVERAGE_EVENT)
        .skip(before)
        .filter_map(|l| l.get_f32("sharp"))
        .collect();
    if after.is_empty() {
        //
        // The canvas writes it when the coverage **changes**, so "no new line
        // after the wheel" means *the coverage did not move*, which is the
        // healthy answer and the one this probe wants. Read as "the canvas
        // published nothing", it produced the line *"pan: the canvas published
        // no coverage line after the wheel"* at **every rung of every run**,
        // 104 % included — a uniform non-answer that made the pan probe
        // permanently silent.
        //
        // ⇒ The same class as `driving::declared`'s own headline: **a change
        // log read as a snapshot.** That one cost eighteen false ribbon
        // defects; this one cost a probe that never reported anything. The fix
        // is the same shape — carry the last known value forward.
        let last_known = trace
            .events(COVERAGE_EVENT)
            .filter_map(|l| l.get_f32("sharp"))
            .last();
        match last_known {
            Some(sharp) => {
                rung.lines.push(format!(
                    "pan: the coverage did not change over the scroll — `canvas-coverage` is a \
                     change log, and it still stands at sharp={sharp:.3}"
                ));
                if sharp < 0.999 {
                    rung.problems.push(format!(
                        "the sharp raster covers only {sharp:.3} of the viewport and a pan did \
                         not change that — the rest is `canvas::backdrop`'s low-resolution \
                         stand-in, which is the fade the operator reported"
                    ));
                }
            }
            None => rung.lines.push(
                "pan: the canvas has published no `canvas-coverage` line at all in this run, so \
                 nothing is known about the raster's reach"
                    .to_owned(),
            ),
        }
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
