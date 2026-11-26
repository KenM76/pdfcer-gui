//! `copier_scan` — **a 600 dpi colour copier scan of a legal page shows its
//! picture.** The page is one 5100 × 8400 JPEG, white with a solid blue block;
//! the block must be drawn blue. A page refused for its size is drawn white,
//! so a refusal cannot pass. Driven off the desktop; pixels come from the
//! window's own frame.

use super::dimdrive::{Fixture, launch_with, ui_rect_event};
use crate::checks::driving::declared;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::Image;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "copier-scan-600dpi.pdf",
    method: "Rebuild it with `python fixtures/copier-scan-600dpi.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 1008.0,
    },
};
const VIEWPORT: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
/// The fixture's blue block (150..462 × 300..708 pt), inset 10 pt so JPEG
/// ringing at its edge is not measured.
const BOX: (f64, f64, f64, f64) = (160.0, 310.0, 452.0, 698.0);
/// The share of the box that must be blue.
const BLUE_MIN: f64 = 0.90;
const COVERAGE: &str = "canvas-coverage"; // ui-text-exempt: a trace event name
/// Settles allowed for the page to finish drawing. A 42.84 Mpx JPEG decodes
/// on the processor in seconds, after the window is already up.
const WAITS: usize = 60;

/// See the module documentation.
pub struct CopierScanShowsItsPicture;

impl Check for CopierScanShowsItsPicture {
    fn name(&self) -> &'static str {
        "a_600_dpi_copier_scan_shows_its_picture"
    }

    fn defect(&self) -> &'static str {
        "a 600 dpi colour scan of a legal page opens blank, because its image is refused for \
         its size"
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

/// The fixture's blue (20, 60, 200), with room for JPEG and colour rounding.
const fn is_blue(r: u8, g: u8, b: u8) -> bool {
    r < 90 && g < 130 && b > 140
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let stem = "copier_scan";
    let (session, pointer) = launch_with(ctx, report, &FIXTURE, "mode.read", stem, &[])?;
    // The LAST coverage line: the first is published before the page renders.
    let mut covered = 0.0_f64;
    for _ in 0..WAITS {
        covered = session
            .trace()?
            .last(COVERAGE)
            .and_then(|l| l.get("covered")?.parse().ok())
            .unwrap_or(0.0);
        if covered >= 0.99 {
            break;
        }
        session.settle(10);
    }
    report.note(format!("the page reported itself {covered:.3} drawn"));
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
    let (mut all, mut blue) = (0_usize, 0_usize);
    for p in img.pixels_in(band) {
        all += 1;
        if is_blue(p.r, p.g, p.b) {
            blue += 1;
        }
    }
    if all == 0 {
        return Err(Error::new("the block's box holds no pixels. SKIPPED."));
    }
    #[allow(clippy::cast_precision_loss)]
    let share = blue as f64 / all as f64;
    report.note(format!(
        "{blue} of {all} pixels in the block are blue ({share:.3})"
    ));
    if share < BLUE_MIN {
        return Ok(Some(format!(
            "the scan's blue block is {share:.3} blue, under {BLUE_MIN}: the picture was not \
             drawn."
        )));
    }
    Ok(None)
}
