//! `a_placed_square_can_be_made_cloudy` — **the Properties panel's Cloudy
//! border checkbox turns a straight square cloudy on the page**, and the panel
//! then reads the new intensity back from the file.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/cloud_border.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, arm_select_from_ribbon, declared, declared_names, list,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::pixels::InkReport;
use crate::report::CheckReport;

/// Review mode, then the rectangle tool.
const INVOKE: &str = "mode.review,markup.rectangle";
const PROPERTIES_TAB: &str = "dock.tab.file.properties";
const CLOUD_REGION: &str = "properties.markup.cloud";
const INTENSITY_REGION: &str = "properties.markup.cloud.intensity";
const APPLY_EVENT: &str = "add-markup";
const SELECT_EVENT: &str = "annot-select";
const RESTYLE_EVENT: &str = "set-markup-style";
/// `markup-cloud-row id=… intensity=none|<i>` — what the panel is displaying.
const ROW_EVENT: &str = "markup-cloud-row";

/// Where the square is drawn, as fractions of the page.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.35, 0.35), (0.55, 0.50));
/// The strip measured beside the edge at the shape's larger y, in points
/// outward from the edge: past a straight 2 pt line's half-width, inside a
/// cloud's scallops.
const STRIP_PT: (f64, f64) = (2.0, 9.0);
/// How much of the edge's length the strip covers, centred.
const STRIP_SPAN: f64 = 0.6;
/// Where the pointer rests for a capture: above the shape, clear of the strip,
/// and on screen at [`ZOOM_FACTOR`].
const PARK: (f64, f64) = (0.40, 0.62);
/// How much the check zooms in on the square before measuring: at fit-page on
/// an A1 sheet a scallop is a pixel deep.
const ZOOM_FACTOR: f32 = 3.0;
const ZOOM_SPINS: usize = 30;
/// Most ink the strip may hold beside a straight edge.
const STRAIGHT_AT_MOST: usize = 4;
/// Least ink the strip must hold beside a cloudy edge.
const CLOUDY_AT_LEAST: usize = 12;

/// See the module documentation.
pub struct APlacedSquareCanBeMadeCloudy;

impl Check for APlacedSquareCanBeMadeCloudy {
    fn name(&self) -> &'static str {
        "a_placed_square_can_be_made_cloudy"
    }

    fn defect(&self) -> &'static str {
        "a rectangle already on the page cannot be given a cloudy border from the Properties \
         panel, or the checkbox writes and the page still shows a straight edge"
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

fn strip_ink(
    session: &Session,
    ctx: &CheckContext,
    page: PageGeometry,
    out: &str,
    report: &mut CheckReport,
) -> Result<InkReport> {
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?;
    let path = ctx.out(out);
    let image = crate::capture::window_to_png(session, &path)?;
    report.artifact(path);
    let (x0, x1) = (SHAPE.0.0 * page.width_pt, SHAPE.1.0 * page.width_pt);
    let half = (x1 - x0) * STRIP_SPAN / 2.0;
    let mid = f64::midpoint(x0, x1);
    let edge = SHAPE.1.1 * page.height_pt;
    let a = mapping.doc_to_window(DocPoint::new(0, mid - half, edge + STRIP_PT.0))?;
    let b = mapping.doc_to_window(DocPoint::new(0, mid + half, edge + STRIP_PT.1))?;
    let rect = LRect::new(
        Pt::new(a.x().min(b.x()), a.y().min(b.y())),
        Pt::new(a.x().max(b.x()), a.y().max(b.y())),
    );
    Ok(crate::pixels::ink_run_into(
        &image,
        session.frame()?.logical_to_capture_pixels(rect),
    ))
}

fn row_reading(session: &Session) -> Result<Option<String>> {
    Ok(session
        .trace()?
        .last(ROW_EVENT)
        .and_then(|l| l.get("intensity").map(str::to_owned)))
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input); reported as SKIPPED rather than passed.",
        ));
    }
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs blank paper to draw on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf)
            .ok_or_else(|| Error::new("cannot read a page size. Pass --page-size WxH."))?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("cloud_border.trace.txt"));
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
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());
    let at = |f: (f64, f64)| {
        aim(
            ctx,
            &session,
            page,
            DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt),
        )
    };

    let blank = strip_ink(&session, ctx, page, "cloud_blank.png", report)?;
    if blank.ink > STRAIGHT_AT_MOST {
        return Err(Error::new(format!(
            "the fixture has its own ink where the check measures ({}); SKIPPED, that is a fact \
             about {} and not the program.",
            blank.summary(),
            pdf.display()
        )));
    }

    driver.drag(at(SHAPE.0)?, at(SHAPE.1)?)?;
    session.settle(40);
    if session.trace()?.events(APPLY_EVENT).count() == 0 {
        return Err(Error::new(format!(
            "the rectangle tool authored nothing (`{APPLY_EVENT}` absent), so there is no \
             square to restyle; `the_format_tab_restyles_a_selected_mark` owns that step. \
             SKIPPED."
        )));
    }

    let centre = (
        f64::midpoint(SHAPE.0.0, SHAPE.1.0),
        f64::midpoint(SHAPE.0.1, SHAPE.1.1),
    );
    let start = crate::checks::scale_aim::current_zoom(&session)?;
    for _ in 0..ZOOM_SPINS {
        if crate::checks::scale_aim::current_zoom(&session)? >= start * ZOOM_FACTOR {
            break;
        }
        driver.scroll_at_held(at(centre)?, &[crate::sys::vk::CONTROL], 1, 1)?;
        session.settle(10);
    }
    session.settle(40);
    report.note(format!(
        "zoomed from {start:.3} to {:.3}",
        crate::checks::scale_aim::current_zoom(&session)?
    ));
    driver.move_to(at(PARK)?)?;
    session.settle(20);
    let straight = strip_ink(&session, ctx, page, "cloud_straight.png", report)?;
    report.note(format!("beside the straight edge: {}", straight.summary()));

    if !arm_select_from_ribbon(&session, &driver, ui_rect, report)? {
        driver.press(crate::sys::vk::V)?;
        session.settle(12);
    }
    driver.click_at(at(centre)?)?;
    session.settle(24);
    if session.trace()?.last(SELECT_EVENT).is_none() {
        return Err(Error::new(format!(
            "a click at the square's centre produced no `{SELECT_EVENT}`; selecting is the step \
             before this one. SKIPPED."
        )));
    }
    let trace = session.trace()?;
    if let Some(tab) = declared(&trace, ui_rect, PROPERTIES_TAB) {
        driver.click_at(session.frame()?.declared_center(tab))?;
        session.settle(20);
    }

    let trace = session.trace()?;
    let Some(toggle) = declared(&trace, ui_rect, CLOUD_REGION) else {
        return Ok(Some(format!(
            "no `{CLOUD_REGION}` for a selected square: the Properties panel offers no cloudy \
             border. Regions under `properties.markup`: {}.",
            list(&declared_names(&trace, ui_rect, "properties.markup"))
        )));
    };
    let before = row_reading(&session)?;
    if before.as_deref() != Some("none") {
        return Ok(Some(format!(
            "a freshly drawn square reads as cloudy ({before:?}) in the panel before the \
             checkbox is touched."
        )));
    }
    if declared(&trace, ui_rect, INTENSITY_REGION).is_some() {
        return Ok(Some(format!(
            "`{INTENSITY_REGION}` is drawn for a straight square, which has no intensity."
        )));
    }

    let restyles = trace.events(RESTYLE_EVENT).count();
    driver.click_at(session.frame()?.declared_center(toggle))?;
    session.settle(30);
    let trace = session.trace()?;
    let written: Vec<_> = trace.events(RESTYLE_EVENT).skip(restyles).collect();
    if written.is_empty() {
        return Ok(Some(format!(
            "the Cloudy border checkbox was clicked and no `{RESTYLE_EVENT}` followed: nothing \
             was written. Look for `{RESTYLE_EVENT}-refused`."
        )));
    }
    report.note(format!("written: `{}`", written[written.len() - 1].raw));
    let after = row_reading(&session)?;
    report.note(format!("the panel reads intensity {after:?}"));
    if after.as_deref().is_none_or(|v| v == "none") {
        return Ok(Some(format!(
            "the restyle was written and the panel still reads {after:?}: the checkbox is not \
             reading the file back."
        )));
    }
    if declared(&trace, ui_rect, INTENSITY_REGION).is_none() {
        return Ok(Some(format!(
            "the square is cloudy and `{INTENSITY_REGION}` is not drawn, so its intensity \
             cannot be changed."
        )));
    }

    // Deselected, so selection handles are out of the strip: they would move
    // outward with a cloud's larger `/Rect` and read as scallops.
    driver.click_at(at(PARK)?)?;
    session.settle(30);
    let cloudy = strip_ink(&session, ctx, page, "cloud_cloudy.png", report)?;
    report.note(format!("beside the cloudy edge: {}", cloudy.summary()));
    if straight.ink > STRAIGHT_AT_MOST {
        return Err(Error::new(format!(
            "the straight edge already put {} in the strip, over {STRAIGHT_AT_MOST}: the strip \
             is not beside the edge at this zoom, so it cannot tell a cloud from a line.",
            straight.summary()
        )));
    }
    if cloudy.ink < CLOUDY_AT_LEAST {
        return Ok(Some(format!(
            "the file says the square is cloudy and the page does not show it: {} beside the \
             edge, under {CLOUDY_AT_LEAST}. Compare cloud_straight.png with cloud_cloudy.png.",
            cloudy.summary()
        )));
    }
    Ok(None)
}
