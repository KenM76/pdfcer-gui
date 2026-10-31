//! `the_page_stays_still_while_the_3d_viewer_is_steered` — a drag and the
//! wheel over the 3D viewer's picture turn and zoom the model and leave the
//! page under the viewer where it was. Driven twice: with the viewer in its
//! own OS window, and drawn inside the application (`PDFCER_DIAG_EMBED`), the
//! host where the viewer is a window over the canvas.
//!
//! The page's position is the `canvas-strip` region, the screen rect the
//! page rasters were painted into; a scroll or a pan moves it. The page is
//! zoomed in first, by Ctrl+= in the main window, so it is larger than the
//! window and a pan has somewhere to go: a page that fits cannot move, and
//! the check would pass on the defect.

use crate::checks::driving::declared_in;
use crate::checks::model_axes::{launch_with, opened_from_page};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::launch::Session;
use crate::report::CheckReport;

const IMAGE: &str = "model3d.image";
const STRIP: &str = "canvas-strip";
/// How far the strip may sit from where it was, in points: rounding only.
const SLACK: f64 = 0.5;

/// See the module documentation; `embed` draws the viewer inside the
/// application.
pub struct ThePageStaysStill {
    pub embed: bool,
}

impl Check for ThePageStaysStill {
    fn name(&self) -> &'static str {
        if self.embed {
            "the_page_stays_still_under_the_embedded_3d_viewer"
        } else {
            "the_page_stays_still_while_the_3d_viewer_is_steered"
        }
    }

    fn defect(&self) -> &'static str {
        "Steering the 3D viewer — a drag or the wheel over its picture — also scrolls or pans \
         the page underneath it"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report, self.embed) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport, embed: bool) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let env: &[(&str, &str)] = if embed {
        &[("PDFCER_DIAG_EMBED", "1")]
    } else {
        &[]
    };
    let stem = if embed {
        "model-still-embed"
    } else {
        "model-still"
    };
    let (session, pointer) = launch_with(ctx, stem, env)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome =
        opened_from_page(ctx, report, &session, &pointer, ui_rect).and_then(|f| match f {
            Some(failure) => Ok(Some(failure)),
            None => steer(report, &session, &pointer, ui_rect),
        });
    pointer.gone(&session)?;
    outcome
}

fn steer(
    report: &mut CheckReport,
    session: &Session,
    pointer: &crate::input::scripted::ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
    session.settle(30);
    let strip = |s: &Session| -> Result<Option<LRect>> {
        Ok(declared_in(&s.trace()?, ui_rect, STRIP).map(|(r, _)| r))
    };
    let Some(fitted) = strip(session)? else {
        return Ok(Some(format!(
            "no `{STRIP}` region: the page was never drawn."
        )));
    };
    for _ in 0..3 {
        pointer.key(session, None, "Equals", Some("ctrl"))?;
        session.settle(10);
    }
    session.settle(30);
    let Some(before) = strip(session)? else {
        return Ok(Some(format!(
            "the `{STRIP}` region vanished after zooming in."
        )));
    };
    if before.width() < fitted.width() * 1.5 {
        return Ok(Some(format!(
            "Ctrl+= in the main window did not zoom the page in: strip {fitted:?} then {before:?}; a page that fits cannot show a pan."
        )));
    }
    let Some((image, vp)) = declared_in(&session.trace()?, ui_rect, IMAGE) else {
        return Ok(Some(format!(
            "no `{IMAGE}` region: the viewer drew no picture."
        )));
    };
    report.note(format!(
        "viewer in {vp:?}; page strip {fitted:?}, zoomed {before:?}"
    ));
    let centre = WindowPoint::centre_of(image);
    let mut down = image;
    down.min.y += image.height() * 0.4;
    down.min.x += image.width() * 0.3;
    let to = WindowPoint::centre_of(down);
    for btn in ["l", "m"] {
        pointer.drag_in(session, vp.as_deref(), centre, to, 8, btn)?;
        session.settle(20);
    }
    pointer.wheel_in(session, vp.as_deref(), centre, -3.0)?;
    session.settle(20);
    pointer.wheel_in(session, vp.as_deref(), centre, 3.0)?;
    session.settle(30);
    let Some(after) = strip(session)? else {
        return Ok(Some(format!(
            "the `{STRIP}` region vanished after steering."
        )));
    };
    report.note(format!("page strip after steering {after:?}"));
    let moved = [
        after.min.x - before.min.x,
        after.min.y - before.min.y,
        after.max.x - before.max.x,
        after.max.y - before.max.y,
    ]
    .iter()
    .any(|d| f64::from(*d).abs() > SLACK);
    if moved {
        return Ok(Some(format!(
            "the page moved while the viewer was steered: strip {before:?} became {after:?}."
        )));
    }
    Ok(None)
}
