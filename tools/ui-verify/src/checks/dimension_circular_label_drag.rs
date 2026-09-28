//! `a_radius_label_drag_swings_the_leader_to_the_drop` — dragging a radius
//! or diameter ce dimension's label previews it at the drop with the button
//! down, and the release stores a leader that faces the drop.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/dimension_circular_label_drag.md`.

use crate::checks::dimension_label_drag::{ink, label_quad, quad_pixels};
use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_or_in_overflow, list};
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
/// `measure-tool tool=…` — the canvas reporting what armed.
const ARM_EVENT: &str = "measure-tool";
/// The `Debug` spelling of `CanvasTool::Measure(MeasureKind::Circular)`.
const ARM_VALUE: &str = "Measure(Circular)";
/// `add-dimension …` — the engine accepted the fitted circle.
const COMMIT_EVENT: &str = "add-dimension";
/// `dim-preview baked=1 … label=x0,y0,…,x3,y3` / `baked=0 refused=…`.
const PREVIEW_EVENT: &str = "dim-preview";
/// `dimension-place id=… offset=… text_along=…`. For a circular ce
/// dimension `offset` is the text distance past the rim and `text_along` the
/// leader angle, degrees counter-clockwise from page +x.
const PLACE_EVENT: &str = "dimension-place";
/// The canvas viewport region, the clip for every pixel rectangle.
const VIEWPORT_REGION: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
/// The circle's centre as fractions of the page box (y up): the blank right
/// half of `blank-overhang.pdf`, clear of its frame and title block.
const CENTRE: (f64, f64) = (0.70, 0.35);
/// The circle's radius as a fraction of the page width.
const RADIUS: f64 = 0.10;
/// Where the three rim picks sit, degrees. None at 0°, where the new
/// dimension's leader is drawn.
const PICK_DEGREES: [f64; 3] = [90.0, 210.0, 330.0];
/// Where the press lands on the leader, as a fraction of the radius from the
/// centre along 0°: past the label, short of the rim.
const PRESS_AT: f64 = 0.85;
/// The drag, in radii: the label anchor moves from (½, 0) to (2, 2) about the
/// centre, so the leader should end at 45° with the text outside the rim and
/// clear of the selection outline, which hugs the circle.
const DRAG: (f64, f64) = (1.5, 2.0);
/// The leader angle the drop implies, and how far the stored one may be off.
const EXPECTED_ANGLE: f64 = 45.0;
const ANGLE_TOLERANCE: f64 = 5.0;
/// Where the label sat before the drag, as a box about the centre in radii.
/// A new circular ce dimension puts its text half way along a 0° leader.
const OLD_LABEL: [(f64, f64); 4] = [(0.3, -0.07), (0.7, -0.07), (0.7, 0.07), (0.3, 0.07)];
/// The fraction of a box that must be ink for text to count as drawn there.
const INKED: f64 = 0.01;

/// See the module documentation.
pub struct ARadiusLabelDragSwingsTheLeaderToTheDrop;

impl Check for ARadiusLabelDragSwingsTheLeaderToTheDrop {
    fn name(&self) -> &'static str {
        "a_radius_label_drag_swings_the_leader_to_the_drop"
    }

    fn defect(&self) -> &'static str {
        "a radius or diameter ce dimension's label could not be dragged, or dragged without a \
         live preview. His ask, verbatim: \"did we make it so we can change between radius and \
         diameter, and make these easy to move and extend again with live preview\""
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
            "input is disabled (--no-input). This check measures a circle and drags its label. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("dimension_circular_label_drag.trace.txt"));
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

    // --- 1: Review, Measure, radius/diameter --------------------------------
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
        return Ok(Some(format!(
            "the Measure tab declares no `{ITEM}`. Items declared: {}.",
            list(&crate::checks::driving::declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.measure."
            ))
        )));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    let trace = session.trace()?;
    if !trace
        .events(ARM_EVENT)
        .any(|l| l.get("tool") == Some(ARM_VALUE))
    {
        return Ok(Some(format!(
            "the radius/diameter item was clicked and no `{ARM_EVENT} tool={ARM_VALUE}` followed."
        )));
    }

    // --- 2: three rim picks, then Finish ------------------------------------
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
            "Finish committed no circular ce dimension, so there is no label to drag. That is \
             `three_clicks_round_a_hole_measure_the_hole`'s subject — run it first. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 3: select it by its leader -----------------------------------------
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
                "the application declares no `{VIEWPORT_REGION}` region, so there is no clip for \
                 the label box. SKIPPED."
            ))
        })?,
    );
    let old_quad = OLD_LABEL.map(|p| about_centre(page, p));
    let Some(old_px) = quad_pixels(&mapping, &frame, old_quad, canvas_px)? else {
        return Err(Error::new(
            "the label's starting box is off the canvas. SKIPPED — the window is too small for \
             this check's geometry.",
        ));
    };
    let before_shot = ctx.out("dimension_circular_label_drag_before.png");
    let before = crate::capture::window_to_png(&session, &before_shot)?;
    report.artifact(before_shot);
    let (old_was, old_was_account) = ink(&before, old_px);
    if old_was < INKED {
        return Ok(Some(format!(
            "the new dimension's label is not where a 0° leader puts it: {old_px:?} carries \
             {old_was_account}. Either the engine's default placement changed, or the circle \
             was not fitted where it was picked."
        )));
    }

    // --- 4: drag the label up and out, photographed with the button down ----
    let drop_at = at(about_centre(page, (PRESS_AT + DRAG.0, DRAG.1)))?;
    let during_shot = ctx.out("dimension_circular_label_drag_during.png");
    let during = driver.drag_observed(grab, drop_at, || {
        crate::capture::window_to_png(&session, &during_shot)
    })?;
    report.artifact(during_shot);
    let trace = session.trace()?;
    if let Some(refused) = trace
        .events(PREVIEW_EVENT)
        .find(|l| l.get("baked") == Some("0"))
    {
        return Ok(Some(format!(
            "★ THE ENGINE REFUSED TO BAKE THE MOVED CIRCULAR DIMENSION: `{}`.",
            refused.raw
        )));
    }
    let Some(baked) = trace
        .events(PREVIEW_EVENT)
        .filter(|l| l.get("baked") == Some("1"))
        .last()
    else {
        return Ok(Some(format!(
            "★ NO LIVE PREVIEW WHILE THE LABEL WAS DRAGGED: no `{PREVIEW_EVENT} baked=1` line. \
             The press did not reach `canvas::dimdrag::drag`, or `dimpreview::paint` declined. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let Some(quad) = baked.get("label").and_then(label_quad) else {
        return Err(Error::new(format!(
            "the `{PREVIEW_EVENT}` line carries no readable `label=` field: `{}`",
            baked.raw
        )));
    };
    let Some(label_px) = quad_pixels(&mapping, &frame, quad, canvas_px)? else {
        return Err(Error::new(format!(
            "the previewed label box {quad:?} is off the canvas. SKIPPED — the window is too \
             small for this check's geometry."
        )));
    };
    let (was, was_account) = ink(&before, label_px);
    let (now, now_account) = ink(&during, label_px);
    if was >= INKED {
        return Err(Error::new(format!(
            "the label's destination {label_px:?} already carried ink before the drag \
             ({was_account}), so ink there afterwards would prove nothing. SKIPPED."
        )));
    }
    if now < INKED {
        return Ok(Some(format!(
            "★ THE PREVIEW IS NOT ON THE GLASS. The trace says `{}`, and the label box it names, \
             {label_px:?}, carries {now_account} with the button down.",
            baked.raw
        )));
    }
    report.note(format!(
        "the label is drawn at the drop with the button down: `{}`; {label_px:?} went from \
         {was_account} to {now_account}",
        baked.raw
    ));

    // --- 5: release — the leader faces the drop -----------------------------
    session.settle(30);
    let trace = session.trace()?;
    let Some(place) = trace.last(PLACE_EVENT) else {
        return Ok(Some(format!(
            "the preview was drawn and the release placed nothing: no `{PLACE_EVENT}` line. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let angle = place.get("text_along").and_then(|v| v.parse::<f64>().ok());
    let distance = place.get("offset").and_then(|v| v.parse::<f64>().ok());
    let (Some(angle), Some(distance)) = (angle, distance) else {
        return Err(Error::new(format!(
            "`{}` carries no readable `offset=` and `text_along=`.",
            place.raw
        )));
    };
    let off = (angle - EXPECTED_ANGLE + 180.0).rem_euclid(360.0) - 180.0;
    if off.abs() > ANGLE_TOLERANCE || distance <= 0.0 {
        return Ok(Some(format!(
            "★ THE LEADER DOES NOT FACE THE DROP. The label was dropped at {EXPECTED_ANGLE}° \
             outside the rim; `{}` stores a leader at {angle:.1}° with the text {distance:.1} pt \
             past the rim.",
            place.raw
        )));
    }
    driver.press(vk::ESCAPE)?;
    session.settle(18);
    let after_shot = ctx.out("dimension_circular_label_drag_after.png");
    let after = crate::capture::window_to_png(&session, &after_shot)?;
    report.artifact(after_shot);
    let (then, then_account) = ink(&after, label_px);
    let (old_now, old_now_account) = ink(&after, old_px);
    if then < INKED {
        return Ok(Some(format!(
            "★ THE RELEASE PUT THE LABEL SOMEWHERE ELSE. The preview drew it in {label_px:?}; \
             after `{}` that box carries {then_account}.",
            place.raw
        )));
    }
    if old_now >= INKED {
        return Ok(Some(format!(
            "★ THE LABEL WAS COPIED, NOT MOVED. After `{}` its starting box {old_px:?} still \
             carries {old_now_account} (it was {old_was_account}).",
            place.raw
        )));
    }
    report.note(format!(
        "`{}` moved the label to the drop ({then_account}) and cleared where it was \
         ({old_was_account} → {old_now_account})",
        place.raw
    ));
    Ok(None)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn the_drop_implies_the_expected_angle() {
        let (u, v) = (PRESS_AT + DRAG.0 - (PRESS_AT - 0.5), DRAG.1);
        assert!((v.atan2(u).to_degrees() - EXPECTED_ANGLE).abs() < 1e-9);
        assert!(u.hypot(v) > 1.0, "the drop is outside the rim");
    }
}
