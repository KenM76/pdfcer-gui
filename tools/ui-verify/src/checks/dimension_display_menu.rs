//! `a_radius_switches_to_a_diameter_from_its_right_click_menu` — a circular ce
//! dimension's right-click menu offers the measure it does not show, and
//! picking it redraws the dimension; the menu then offers the way back.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_display_menu.md`.

use crate::checks::dimension_label_drag::{ink, quad_pixels};
use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode the dimension is drawn in.
const MODE: &str = "review";
/// The Measure tab.
const TAB: &str = "ribbon.tab.measure";
/// The ribbon item that arms the radius/diameter tool.
const ITEM: &str = "ribbon.item.measure.radius_diameter";
/// The ribbon item that ends the circular gesture.
const FINISH: &str = "ribbon.item.measure.finish";
/// `add-dimension …` — the engine accepted the fitted circle.
const COMMIT_EVENT: &str = "add-dimension";
/// `dimension-display id=… diameter=0|1` — the switch was raised.
const DISPLAY_EVENT: &str = "dimension-display";
/// The two menu rows.
const TO_DIAMETER: &str = "menu.item.canvas.dimension.format.dimension_diameter";
const TO_RADIUS: &str = "menu.item.canvas.dimension.format.dimension_radius";
/// The canvas viewport region, the clip for every pixel rectangle.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
/// The circle's centre as fractions of the page box (y up), and its radius as
/// a fraction of the page width: the blank right half of `blank-overhang.pdf`.
const CENTRE: (f64, f64) = (0.70, 0.35);
const RADIUS: f64 = 0.10;
/// Where the three rim picks sit, degrees. None at 0° or 180°, where the
/// leader is drawn.
const PICK_DEGREES: [f64; 3] = [90.0, 210.0, 330.0];
/// Where the leader is clicked, in radii from the centre along 0°.
const PRESS_AT: f64 = 0.85;
/// A thin box across the 180° half of the leader, in radii about the centre.
/// A radius leader runs from the centre to 0° and leaves it blank; a diameter
/// leader crosses it.
const FAR_HALF: [(f64, f64); 4] = [(-0.75, -0.04), (-0.45, -0.04), (-0.45, 0.04), (-0.75, 0.04)];
/// The fraction of the box that must be ink for the leader to count as drawn.
const INKED: f64 = 0.01;

/// See the module documentation.
pub struct ARadiusSwitchesToADiameterFromItsRightClickMenu;

impl Check for ARadiusSwitchesToADiameterFromItsRightClickMenu {
    fn name(&self) -> &'static str {
        "a_radius_switches_to_a_diameter_from_its_right_click_menu"
    }

    fn defect(&self) -> &'static str {
        "a circular ce dimension can be switched between radius and diameter only by finding \
         the Properties panel. His ask, verbatim: \"did we make it so we can change between \
         radius and diameter\""
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

/// A point `(u, v)` radii from the centre, in page points.
fn about_centre(page: PageGeometry, (u, v): (f64, f64)) -> (f64, f64) {
    let r = RADIUS * page.width_pt;
    (
        CENTRE.0 * page.width_pt + u * r,
        CENTRE.1 * page.height_pt + v * r,
    )
}

/// Right-click `at`, require `offered` and the absence of `withheld`, click
/// `offered`, and return the `dimension-display` line it raised.
fn switch(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    at: ScreenPoint,
    offered: &str,
    withheld: &str,
) -> Result<std::result::Result<String, String>> {
    driver.right_click_at(at)?;
    session.settle(20);
    let trace = session.trace()?;
    let rows = || {
        list(&declared_names(
            &trace,
            ui_rect,
            "menu.item.canvas.dimension.",
        ))
    };
    if declared(&trace, ui_rect, withheld).is_some() {
        return Ok(Err(format!(
            "the menu offers `{withheld}`, the measure the dimension already shows. Rows: {}.",
            rows()
        )));
    }
    let Some(row) = declared(&trace, ui_rect, offered) else {
        return Ok(Err(format!(
            "the right-click menu offers no `{offered}`. Rows: {}.",
            rows()
        )));
    };
    let mark = trace.mark();
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(30);
    let trace = session.trace()?;
    Ok(trace.last_after(DISPLAY_EVENT, mark).map_or_else(
        || {
            Err(format!(
                "`{offered}` was clicked and no `{DISPLAY_EVENT}` line followed."
            ))
        },
        |l| Ok(l.raw.clone()),
    ))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check measures a circle and right-clicks it. \
             Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("dimension_display_menu.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: measure a circle ------------------------------------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, TAB) else {
        return Ok(Some(format!(
            "the `{MODE}` mode declares no `{TAB}` region."
        )));
    };
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let Some(item) = declared_or_in_overflow(&session, &driver, ui_rect, ITEM)? else {
        return Ok(Some(format!("the Measure tab declares no `{ITEM}`.")));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    let trace = session.trace()?;
    if trace
        .last("canvas")
        .and_then(|l| l.get("page"))
        .and_then(|v| v.parse::<usize>().ok())
        != Some(0)
    {
        return Err(Error::new(
            "the canvas is not showing page 1, so the page geometry does not describe what is \
             on screen.",
        ));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let frame = session.frame()?;
    let at = |(x, y): (f64, f64)| -> Result<ScreenPoint> {
        Ok(frame.to_screen(mapping.doc_to_window(DocPoint::new(0, x, y))?))
    };
    for deg in PICK_DEGREES {
        let (s, c) = deg.to_radians().sin_cos();
        driver.click_at(at(about_centre(page, (c, s)))?)?;
        session.settle(12);
    }
    let Some(finish) = declared_or_in_overflow(&session, &driver, ui_rect, FINISH)? else {
        return Ok(Some(format!(
            "three rim picks were made and the Measure tab declares no `{FINISH}`."
        )));
    };
    driver.click_at(session.frame()?.declared_center(finish))?;
    session.settle(30);
    if session.trace()?.events(COMMIT_EVENT).count() == 0 {
        return Ok(Some(format!(
            "Finish committed no circular ce dimension. That is \
             `three_clicks_round_a_hole_measure_the_hole`'s subject — run it first. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 2: select it; the far half of the leader is blank ------------------
    if !crate::checks::driving::arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        driver.press(vk::V)?;
        session.settle(12);
    }
    let grab = at(about_centre(page, (PRESS_AT, 0.0)))?;
    driver.click_at(grab)?;
    session.settle(18);
    let canvas_px = frame.logical_to_capture_pixels(
        declared(&session.trace()?, ui_rect, VIEWPORT_REGION).ok_or_else(|| {
            Error::new(format!(
                "the application declares no `{VIEWPORT_REGION}` region. SKIPPED."
            ))
        })?,
    );
    let far = FAR_HALF.map(|p| about_centre(page, p));
    let Some(far_px) = quad_pixels(&mapping, &frame, far, canvas_px)? else {
        return Err(Error::new(
            "the far half of the leader is off the canvas. SKIPPED — the window is too small \
             for this check's geometry.",
        ));
    };
    let radius_shot = ctx.out("dimension_display_menu_radius.png");
    let radius = crate::capture::window_to_png(&session, &radius_shot)?;
    report.artifact(radius_shot);
    let (was, was_account) = ink(&radius, far_px);
    if was >= INKED {
        return Err(Error::new(format!(
            "the far half of a radius leader, {far_px:?}, already carries {was_account}, so ink \
             there after the switch would prove nothing. SKIPPED."
        )));
    }

    // --- 3: right-click ▸ Show as diameter ----------------------------------
    let line = match switch(&session, &driver, ui_rect, grab, TO_DIAMETER, TO_RADIUS)? {
        Ok(line) => line,
        Err(why) => return Ok(Some(why)),
    };
    if !line.contains("diameter=1") {
        return Ok(Some(format!(
            "Show as diameter raised `{line}`, which does not ask for the diameter."
        )));
    }
    session.settle(20);
    let diameter_shot = ctx.out("dimension_display_menu_diameter.png");
    let diameter = crate::capture::window_to_png(&session, &diameter_shot)?;
    report.artifact(diameter_shot);
    let (now, now_account) = ink(&diameter, far_px);
    if now < INKED {
        return Ok(Some(format!(
            "★ THE DIMENSION DID NOT REDRAW AS A DIAMETER. `{line}` was raised, and the far half \
             of the leader, {far_px:?}, carries {now_account}."
        )));
    }
    report.note(format!(
        "Show as diameter: `{line}`; the leader now crosses the circle ({was_account} → \
         {now_account})"
    ));

    // --- 4: the menu now offers the way back --------------------------------
    let line = match switch(&session, &driver, ui_rect, grab, TO_RADIUS, TO_DIAMETER)? {
        Ok(line) => line,
        Err(why) => return Ok(Some(format!("after the switch to diameter: {why}"))),
    };
    if !line.contains("diameter=0") {
        return Ok(Some(format!(
            "Show as radius raised `{line}`, which does not ask for the radius."
        )));
    }
    session.settle(20);
    let back_shot = ctx.out("dimension_display_menu_back.png");
    let back = crate::capture::window_to_png(&session, &back_shot)?;
    report.artifact(back_shot);
    let (then, then_account) = ink(&back, far_px);
    if then >= INKED {
        return Ok(Some(format!(
            "★ SHOW AS RADIUS LEFT THE DIAMETER DRAWN. `{line}` was raised, and {far_px:?} still \
             carries {then_account}."
        )));
    }
    report.note(format!(
        "Show as radius: `{line}`; the far half is blank again ({then_account})"
    ));
    Ok(None)
}
