//! `a_pan_keeps_the_fit_and_the_resize_keeps_the_position` — **fit, pan away,
//! resize: the view stays where the operator put it AND the fit is still
//! live.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/fit_left_by_a_pan.md`.

use super::fit_places_the_view::{invoke, page_rect};
use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Arms the hand tool, so a primary drag is a pan. See the module header.
const INVOKE: &str = "view.tool_hand";
/// The canvas viewport, which every measurement below is made against.
const CANVAS_REGION: &str = "canvas-viewport";
/// The Fit page control.
const FIT_ITEM: &str = "ribbon.item.view.zoom_fit_page";
/// How far the pan drags, as a fraction of the canvas on each axis.
const PAN_BY: f32 = 0.25;
/// How much smaller the window is made, in physical pixels.
const RESIZE_BY_PX: i32 = 160;
/// The window chrome, added back so the restore returns to the start size.
const BORDER_PX: i32 = 16;
/// The title bar and border, vertically.
const TITLEBAR_PX: i32 = 39;
/// How far a margin may differ and still count as "centred", in points.
const CENTRED_TOLERANCE: f32 = 4.0;

/// See the module documentation.
pub struct APanLeavesTheFit;

impl Check for APanLeavesTheFit {
    fn name(&self) -> &'static str {
        "a_pan_keeps_the_fit_and_the_resize_keeps_the_position"
    }

    fn defect(&self) -> &'static str {
        "a fit stays active after the operator pans away from it, so the next canvas resize \
         re-places the page and throws away the position they chose — the half of O55 that \
         only bites once a resize re-places at all"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control, drags the page \
             and resizes the window.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to fit."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("fit-pan.trace.txt"));
    spec.pdf = Some(pdf);
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
    session.settle(45);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    let canvas = declared(&trace, ui_rect, CANVAS_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{CANVAS_REGION}`; is a document open? Regions: {}.",
            list(&declared_names(&trace, ui_rect, "canvas"))
        ))
    })?;

    // --- A: fit the page ----------------------------------------------------
    invoke(&session, &driver, ui_rect, FIT_ITEM)?;
    session.settle(24);
    let Some(fitted) = page_rect(&session)? else {
        return Err(Error::new(
            "the canvas published no page rect after Fit page. SKIPPED.".to_owned(),
        ));
    };
    report.note(format!(
        "★ fitted: page {:.1},{:.1}..{:.1},{:.1}",
        fitted.min.x, fitted.min.y, fitted.max.x, fitted.max.y
    ));

    // --- B: pan away, and assert it moved -----------------------------------
    let frame = session.frame()?;
    // Both ends expressed as FRACTIONS of the canvas rather than as a point
    // plus a pixel offset, so the drag scales with whatever viewport the window
    // happens to have — the same reason `PAN_BY` below is a fraction and not a
    // distance. A fixed pixel travel is a distance in the screen's space, and
    // the canvas is not the screen.
    let from = frame.declared_at(canvas, 0.35, 0.35);
    let to = frame.declared_at(canvas, 0.35 + PAN_BY, 0.35 + PAN_BY);
    driver.drag(from, to)?;
    session.settle(24);

    let Some(panned) = page_rect(&session)? else {
        return Err(Error::new(
            "the canvas stopped publishing a page rect after the pan. SKIPPED.".to_owned(),
        ));
    };
    if (panned.min.x - fitted.min.x).abs() < CENTRED_TOLERANCE
        && (panned.min.y - fitted.min.y).abs() < CENTRED_TOLERANCE
    {
        return Err(Error::new(format!(
            "the pan did not move the page (still at {:.1},{:.1}), so this run cannot tell a fit \
             that was LEFT from one that was never disturbed. The hand tool may not have armed — \
             `view.tool_hand` is rung through PDFCER_DIAG_INVOKE at launch. SKIPPED rather than \
             passed.",
            panned.min.x, panned.min.y
        )));
    }
    report.note(format!(
        "★★ panned: page {:.1},{:.1}..{:.1},{:.1}",
        panned.min.x, panned.min.y, panned.max.x, panned.max.y
    ));

    // --- C: resize ----------------------------------------------------------
    let Some(handle) = session.window() else {
        return Err(Error::new(
            "no window handle to resize. SKIPPED.".to_owned(),
        ));
    };
    let start = session.frame()?;
    let (cw, ch) = start.client_size;
    let (w, h) = (
        i32::try_from(cw).unwrap_or(1100) + BORDER_PX,
        i32::try_from(ch).unwrap_or(800) + TITLEBAR_PX,
    );
    crate::sys::resize_window(handle, w - RESIZE_BY_PX, h - RESIZE_BY_PX);
    session.settle(30);

    let after_trace = session.trace()?;
    let canvas_after = declared(&after_trace, ui_rect, CANVAS_REGION);
    let after = page_rect(&session)?;
    crate::sys::resize_window(handle, w, h);
    session.settle(25);

    let (Some(page_after), Some(canvas_after)) = (after, canvas_after) else {
        return Err(Error::new(
            "the canvas stopped publishing its rect across the resize. SKIPPED.".to_owned(),
        ));
    };
    if (canvas_after.width() - canvas.width()).abs() < CENTRED_TOLERANCE {
        return Err(Error::new(format!(
            "the resize did not change the canvas (still {:.1} pt wide), so there is no resize \
             for the fit to have survived. SKIPPED rather than passed.",
            canvas_after.width()
        )));
    }

    // --- D: the page must NOT have been re-centred --------------------------
    let left = page_after.min.x - canvas_after.min.x;
    let right = canvas_after.max.x - page_after.max.x;
    let top = page_after.min.y - canvas_after.min.y;
    let bottom = canvas_after.max.y - page_after.max.y;
    report.note(format!(
        "★★★ after the resize: margins l={left:.1} r={right:.1} t={top:.1} b={bottom:.1}"
    ));
    if (left - right).abs() < CENTRED_TOLERANCE && (top - bottom).abs() < CENTRED_TOLERANCE {
        return Ok(Some(format!(
            "★★★ THE RESIZE RE-CENTRED A PAGE THE OPERATOR HAD PANNED AWAY FROM: margins \
             l={left:.1} r={right:.1} t={top:.1} b={bottom:.1} are equal on both axes.\n\
             `OPERATOR_REQUESTS.md` O78: *\"whatever area was centered in the current canvas \
             should stay centered\"*. Preserving an OFF-CENTRE point keeps it off centre, so \
             equal margins mean the position was discarded rather than preserved.\n\
             `canvas::fit::placement` must measure the centred page fraction from the previous \
             frame (`canvas::geometry::centred_frac`) and place it back with \
             `offset_holding_anchor_at`, rather than re-placing the page from the fit. Trace: \
             {}.",
            session.trace_path().display()
        )));
    }
    report.note("★ the pan survived the resize, so the position was preserved as he expected");

    // --- E: …AND THE FIT IS STILL LIVE, which is O78's reversal -----------
    //
    // The assertion that makes this check a FALSIFIER for the change
    // rather than a survivor of it.
    //
    // Step D alone passes on both builds: the old one dropped the fit on the
    // pan and therefore did not re-centre, and the new one keeps the fit and
    // preserves the centre, and both leave the page off centre. A check that
    // passes either way is measuring nothing, which this project has recorded
    // three times as its own commonest defect.
    //
    // So: the page's drawn WIDTH must have changed across the resize. A live
    // fit re-scales from the viewport every frame (`ViewState::apply_fit`), so
    // a narrower canvas means a narrower page. On the pre-O78 build the pan
    // dropped the fit, the zoom was frozen, and the width is unchanged — this
    // fails, by name.
    let width_before = panned.width();
    let width_after = page_after.width();
    report.note(format!(
        "★★ page width across the resize: {width_before:.1} pt → {width_after:.1} pt"
    ));
    if (width_after - width_before).abs() < CENTRED_TOLERANCE {
        return Ok(Some(format!(
            "★★★ THE FIT DID NOT SURVIVE THE PAN: the canvas narrowed by {RESIZE_BY_PX} px and \
             the page is still {width_after:.1} pt wide, so nothing re-scaled it.\n\
             `OPERATOR_REQUESTS.md` O78: *\"unless I have manually changed the zoom after \
             clicking one of the preset options, the pdf should maintain whichever option was \
             selected\"*. His earlier sentence (O55, 2026-08-28) also said \"or panned around\"; \
             this one does not, and it does not have to — a rule about POSITION now protects \
             the position, so a rule about ZOOM no longer has to be abandoned to do it.\n\
             `canvas::offset`'s pan arm must NOT call `doc.view.set_fit(FitMode::None)`. Only \
             `set_zoom` leaves a fit. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★ …and the fit re-scaled the page, so it survived the pan (O78)");
    Ok(None)
}
