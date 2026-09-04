//! `measure_calibrates_by_picking_two_points` — set the scale by measuring
//! something on the drawing, which is the workflow a drafter actually uses.
//!
//! # The gap this closes
//!
//! Reported by the operator on 2026-08-17, in their words:
//!
//! > *"Measure tool still missing the feature where we set the scale by
//! > selecting two lines or points and defining what that distance
//! > represents."*
//!
//! It was real and unusually well documented: `dialogs::scale`'s own header
//! said the real-length path *"is offered when a reference line has been drawn,
//! and drawing one is a canvas gesture (`ScalePick`) that is not yet armed by
//! any command"*. The **model** had been complete since the Phase 7 salvage —
//! `ScalePick`, `ScaleEntryFields::sync_real_length`, the back-calculation
//! through the engine's own `preview_group_scale` — all pure and unit-tested.
//! What was missing was arming, click routing, and a way back into the dialog.
//!
//! # Why this needs driving
//!
//! Because every part of it was already unit-tested while the feature did not
//! exist. `ScalePick::commit_point` has tests, `ScaleEntryFields::commit` has
//! tests, `ScaleDialog` has tests, and an operator still could not calibrate a
//! drawing — because **nothing in the workspace can observe those three being
//! connected**. The chain is five links and three are frame-level:
//!
//! 1. a ribbon press opens the Set-scale dialog;
//! 2. a button in it raises a request and closes the window;
//! 3. `app::frame` notices the request and arms `MeasureKind::Scale`;
//! 4. two canvas clicks advance `ScalePick` to a completed reference line;
//! 5. `app::frame` notices *that*, re-opens the dialog with the measured length
//!    in it, and disarms the tool.
//!
//! Steps 3 and 5 are edges read once per frame. Only a running window sees
//! them.
//!
//! # The assertion it would be easy to leave out
//!
//! The last one: **the measured length is not zero**, and is near the distance
//! actually clicked. Without it this passes on a build that re-opens the dialog
//! carrying `0.0` — which is what a broken snap, a mis-mapped coordinate or a
//! pick that recorded the same point twice all produce, and all three are
//! indistinguishable from success at every earlier step.
//!
//! A scale is a number every later dimension is multiplied by. A calibration
//! that silently measured nothing makes every dimension on the sheet wrong in
//! the same direction, which is worse than a tool that plainly fails.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, frame_of, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// How far apart the two picked points are, in PDF points along x.
///
/// 400 pt is roughly a quarter of the benchmark sheet's width — long enough
/// that a snap landing on nearby content cannot account for the whole
/// distance, short enough to stay on the page at any fit this check may meet.
const SPAN_PT: f64 = 400.0;

/// How far the measured length may sit from [`SPAN_PT`] and still pass.
///
/// Deliberately wide. The picks go through the **snap** machinery, which is
/// correct behaviour and is the entire point of calibrating against a drawing:
/// pdfcer moves the click to the endpoint the operator meant. So the measured
/// length is the distance between two *snapped* points and is not required to
/// equal the raw span.
///
/// What a wide bound still catches is the failure that matters — zero, or a
/// value an order of magnitude out from a coordinate-space mix-up. A tighter
/// bound would fail on a correct build whenever the fixture had geometry near
/// a pick, which on a real drawing is most of the time.
const SPAN_TOLERANCE_PT: f64 = 150.0;

/// See the module documentation.
pub struct MeasureCalibratesByPickingTwoPoints;

impl Check for MeasureCalibratesByPickingTwoPoints {
    fn name(&self) -> &'static str {
        "measure_calibrates_by_picking_two_points"
    }

    fn defect(&self) -> &'static str {
        "the scale can only be set by typing a ratio — the two-point calibration the \
         operator asked for does not arm, does not route its clicks, or re-opens the dialog \
         with no measurement in it"
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
            "input is disabled (--no-input). This check clicks a ribbon tab, a ribbon control, \
             a dialog button and two points on the page. Reported as SKIPPED rather than \
             passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. This check needs somewhere on the page to measure FROM, and a \
             guessed one can land off the sheet — which is symptom-identical to a pick that \
             never registered.",
        )
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("measure_calibrate.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Review mode, so the Measure tab is offered ---------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, "review")?;

    // --- 2: the Measure tab ------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.measure").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.measure` region after switching to Review. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("measure"))
    {
        return Err(Error::new(
            "the click on the Measure tab produced no tab-selected line, so nothing below \
             would mean anything.",
        ));
    }

    // --- 3: open Set scale -------------------------------------------------
    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, "ribbon.item.measure.set_scale").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.item.measure.set_scale` region on the Measure tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.measure."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(16);
    let trace = session.trace()?;
    if declared(&trace, ui_rect, "dialog:set-scale").is_none() {
        return Ok(Some(
            "`measure.set_scale` was clicked and no `dialog:set-scale` region appeared, so the \
             Set-scale dialog did not open."
                .to_owned(),
        ));
    }
    report.note("the Set-scale dialog opened from Measure > Scale");

    // --- 4: ask to measure it on the drawing -------------------------------
    let Some(button) = declared(&trace, ui_rect, "scale.calibrate") else {
        return Ok(Some(
            "the Set-scale dialog declared no `scale.calibrate` region, so there is no route \
             from it into the two-point calibration. This is the operator's reported gap \
             exactly: the dialog offers the ratio path and nothing else."
                .to_owned(),
        ));
    };
    driver.click_at(
        frame_of(&session, &trace, ui_rect, "scale.calibrate")?.declared_center(button),
    )?;
    session.settle(16);
    let trace = session.trace()?;
    if !trace
        .events("scale-calibrate")
        .any(|l| l.get("armed") == Some("true"))
    {
        return Ok(Some(
            "the calibrate button was clicked and no `scale-calibrate armed=true` line was \
             traced, so the request never reached `app::frame` and the measure tool was never \
             armed. The button is drawn and does nothing."
                .to_owned(),
        ));
    }
    report.note("the calibrate button armed the two-point pick and closed the dialog");

    // --- 5: pick two points on the page ------------------------------------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    report.note(format!(
        "canvas at zoom {:.3}; picking two points {SPAN_PT:.0} pt apart",
        mapping.zoom
    ));
    for (label, doc) in [
        ("A", target),
        (
            "B",
            DocPoint {
                page: target.page,
                x: target.x + SPAN_PT,
                y: target.y,
            },
        ),
    ] {
        let window = mapping.doc_to_window(doc)?;
        let screen = session.frame()?.to_screen(window);
        report.note(format!(
            "pick {label}: document ({:.1}, {:.1}) -> screen ({}, {})",
            doc.x,
            doc.y,
            screen.x(),
            screen.y()
        ));
        driver.click_at(screen)?;
        session.settle(14);
    }

    // --- 6: the dialog comes back, carrying a measurement ------------------
    let trace = session.trace()?;
    let Some(measured) = trace
        .events("scale-calibrate")
        .filter_map(|l| l.get("measured_pt"))
        .filter_map(|v| v.parse::<f64>().ok())
        .last()
    else {
        return Ok(Some(
            "two points were clicked on the page and no `scale-calibrate measured_pt=` line \
             was traced. Either the clicks did not reach `ScalePick` — arming worked and \
             routing did not — or the pick never completed. The reference line is the only \
             thing between the gesture and the dialog."
                .to_owned(),
        ));
    };
    report.note(format!("the pick completed and measured {measured:.2} pt"));

    if declared(&trace, ui_rect, "dialog:set-scale").is_none() {
        return Ok(Some(format!(
            "the pick completed and measured {measured:.2} pt, and the Set-scale dialog did \
             not re-open — so the operator is left holding a measurement with nowhere to say \
             what it represents. The gesture is the easy half; coming back is the half that \
             makes it a feature."
        )));
    }
    if declared(&trace, ui_rect, "scale.real_length").is_none() {
        return Ok(Some(format!(
            "the dialog re-opened after measuring {measured:.2} pt and declared no \
             `scale.real_length` region, so it came back on the RATIO path. The operator did \
             the work of picking two points and is being asked for a ratio anyway — the \
             reported gap wearing a longer route."
        )));
    }
    report.note("the dialog re-opened on the real-length path, asking what the line represents");

    // --- 7: the measurement is real ----------------------------------------
    if measured <= 0.0 {
        return Ok(Some(format!(
            "the calibration measured {measured:.2} pt — a zero-length reference line. Every \
             earlier assertion held and the number the operator is about to calibrate against \
             is nothing. A scale derived from it multiplies every later dimension on the \
             sheet by a wrong constant, in silence."
        )));
    }
    if (measured - SPAN_PT).abs() > SPAN_TOLERANCE_PT {
        return Ok(Some(format!(
            "the calibration measured {measured:.2} pt for two points {SPAN_PT:.0} pt apart \
             (tolerance {SPAN_TOLERANCE_PT:.0}). Snapping legitimately moves a pick, which is \
             why the tolerance is wide — a difference this large is a coordinate-space \
             mix-up rather than a snap."
        )));
    }
    report.note(format!(
        "{measured:.2} pt is within {SPAN_TOLERANCE_PT:.0} pt of the {SPAN_PT:.0} pt clicked, \
         so the measurement is of the page and not of something else"
    ));
    Ok(None)
}
