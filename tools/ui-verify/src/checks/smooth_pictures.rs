//! `smooth_pictures` — **Settings ▸ Display ▸ Smooth enlarged pictures decides
//! how a picture enlarged past its own pixels is drawn.** On (the default), a
//! 4 × 4 checkerboard shown 300 pt wide is smoothed into a spread of greys;
//! off, it is black and white squares. Driven off the desktop; pixels come from
//! the window's own frame.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/smooth_pictures.md`.

use super::dimdrive::{Fixture, launch_with, ui_rect_event};
use crate::checks::driving::declared;
use crate::checks::scripteddrive::userdata;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "checker-image.pdf",
    method: "Rebuild it with `python fixtures/checker-image.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 400.0,
        height_pt: 400.0,
    },
};
const VIEWPORT: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
/// The checkerboard's box, inset 10 pt from its 50..350 edges so the
/// anti-aliased outline is not measured.
const BOX: (f64, f64, f64, f64) = (60.0, 60.0, 340.0, 340.0);
const KEY: &str = "smooth_enlarged_pictures"; // ui-text-exempt: a preferences-file key
/// A grey this far from both black and white is a blend between squares.
const MID: std::ops::RangeInclusive<u8> = 40..=215;
/// Smoothed: at least this share of the box is mid-grey. A 4-pixel image
/// bilinearly enlarged ramps across most of every square.
const SMOOTH_MIN: f64 = 0.20;
/// Point-sampled: at most this share — square edges only.
const BLOCKY_MAX: f64 = 0.05;

/// See the module documentation.
pub struct SmoothPicturesSetting;

impl Check for SmoothPicturesSetting {
    fn name(&self) -> &'static str {
        "smooth_enlarged_pictures_follows_the_setting"
    }

    fn defect(&self) -> &'static str {
        "a scanned page zoomed in is drawn the same whether Smooth enlarged pictures is on or off"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = drive(ctx, &mut report);
        if let Ok(dir) = userdata(ctx) {
            let _ = crate::sandbox::reset_prefs(&dir);
        }
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Launch under `prefs` and return the share of the box that is mid-grey.
fn mid_share(ctx: &CheckContext, report: &mut CheckReport, tag: &str, prefs: &str) -> Result<f64> {
    crate::sandbox::write_prefs(&userdata(ctx)?, prefs)
        .map_err(|e| Error::new(format!("could not plant the preferences: {e}")))?;
    let stem = format!("smooth_pictures.{tag}");
    let (session, pointer) = launch_with(ctx, report, &FIXTURE, "mode.read", &stem, &[])?;
    let trace = session.trace()?;
    let frame = session.frame()?;
    if declared(&trace, ui_rect_event(ctx)?, VIEWPORT).is_none() {
        return Err(Error::new(format!("no `{VIEWPORT}` region. SKIPPED.")));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE.page, 0)?;
    let (x0, y0, x1, y1) = BOX;
    let tl = mapping.doc_to_window(DocPoint::new(0, x0, y1))?;
    let br = mapping.doc_to_window(DocPoint::new(0, x1, y0))?;
    let band = frame
        .logical_to_capture_pixels(LRect::new(Pt::new(tl.x(), tl.y()), Pt::new(br.x(), br.y())));
    let path = ctx.out(&format!("{stem}.png"));
    pointer.screenshot(&session, &path)?;
    let img = Image::load_png(&path)?;
    report.artifact(path);
    let _ = pointer.gone(&session);
    let (mut all, mut mid) = (0_usize, 0_usize);
    for p in img.pixels_in(band) {
        all += 1;
        if MID.contains(&p.r) && MID.contains(&p.g) && MID.contains(&p.b) {
            mid += 1;
        }
    }
    if all == 0 {
        return Err(Error::new(
            "the checkerboard's box holds no pixels. SKIPPED.",
        ));
    }
    #[allow(clippy::cast_precision_loss)]
    let share = mid as f64 / all as f64;
    report.note(format!(
        "{tag}: {mid} of {all} pixels in the box are mid-grey ({share:.3})"
    ));
    Ok(share)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let smooth = mid_share(ctx, report, "default", "")?;
    let blocky = mid_share(ctx, report, "off", &format!("{KEY} = false"))?;
    if smooth < SMOOTH_MIN {
        return Ok(Some(format!(
            "with the shipped default the enlarged checkerboard is {smooth:.3} mid-grey, under \
             {SMOOTH_MIN}: it is not smoothed."
        )));
    }
    if blocky > BLOCKY_MAX {
        return Ok(Some(format!(
            "with {KEY} = false the enlarged checkerboard is {blocky:.3} mid-grey, over \
             {BLOCKY_MAX}: it is still smoothed."
        )));
    }
    report.note("smoothed by default, square blocks with the setting off");
    Ok(None)
}
