//! `card_pages` — **Settings ▸ Display ▸ Draw pages on the graphics card draws
//! the page the processor draws.** The A1 title block is opened twice, the
//! switch off and on. On, the trace must say the card drew the page; off, it
//! must not mention the card. The page's pixels on screen must agree between
//! the two launches to within anti-aliasing. Driven off the desktop; pixels
//! come from the window's own frame.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/card_pages.md`.

use super::dimdrive::{Fixture, launch_with, ui_rect_event};
use crate::checks::driving::declared;
use crate::checks::scripteddrive::userdata;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::image::{Image, Rgb};
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "a1-titleblock.pdf",
    method: "It is checked in under fixtures/.",
    page: PageGeometry {
        width_pt: 2383.94,
        height_pt: 1683.78,
    },
};
const VIEWPORT: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
const KEY: &str = "draw_pages_on_graphics_card"; // ui-text-exempt: a preferences-file key
const EVENT: &str = "render-card"; // ui-text-exempt: a trace event name
/// A pixel differs when any channel moves by more than this.
const CHANNEL: u8 = 48;
/// At most this share of the page's pixels may differ: edge anti-aliasing,
/// never a missing or moved line.
const DIFFER_MAX: f64 = 0.005;

/// See the module documentation.
pub struct CardPages;

impl Check for CardPages {
    fn name(&self) -> &'static str {
        "the_graphics_card_draws_the_page_the_processor_draws"
    }

    fn defect(&self) -> &'static str {
        "with Draw pages on the graphics card on, the page is still drawn on the processor, or \
         the card's page differs from the processor's"
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

/// One launch: the page's pixels and the trace's card lines.
struct Shot {
    pixels: Vec<Rgb>,
    card: Vec<String>,
}

fn shoot(ctx: &CheckContext, report: &mut CheckReport, tag: &str, prefs: &str) -> Result<Shot> {
    crate::sandbox::write_prefs(&userdata(ctx)?, prefs)
        .map_err(|e| Error::new(format!("could not plant the preferences: {e}")))?;
    let stem = format!("card_pages.{tag}");
    let (session, pointer) = launch_with(ctx, report, &FIXTURE, "mode.read", &stem, &[])?;
    session.settle(60);
    let trace = session.trace()?;
    let frame = session.frame()?;
    if declared(&trace, ui_rect_event(ctx)?, VIEWPORT).is_none() {
        return Err(Error::new(format!("no `{VIEWPORT}` region. SKIPPED.")));
    }
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE.page, 0)?;
    // Inset: the page's own corner sits on the canvas edge at fit.
    let (w, h) = (FIXTURE.page.width_pt - 4.0, FIXTURE.page.height_pt - 4.0);
    let tl = mapping.doc_to_window(DocPoint::new(0, 4.0, h))?;
    let br = mapping.doc_to_window(DocPoint::new(0, w, 4.0))?;
    let band = frame
        .logical_to_capture_pixels(LRect::new(Pt::new(tl.x(), tl.y()), Pt::new(br.x(), br.y())));
    let path = ctx.out(&format!("{stem}.png"));
    pointer.screenshot(&session, &path)?;
    let img = Image::load_png(&path)?;
    report.artifact(path);
    let _ = pointer.gone(&session);
    let card: Vec<String> = trace.events(EVENT).map(|l| l.raw.clone()).collect();
    for line in card.iter().take(3) {
        report.note(format!("{tag}: {line}"));
    }
    Ok(Shot {
        pixels: img.pixels_in(band).collect(),
        card,
    })
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let off = shoot(ctx, report, "off", "")?;
    let on = shoot(ctx, report, "on", &format!("{KEY} = true"))?;
    if !off.card.is_empty() {
        return Ok(Some(format!(
            "with the switch off the trace says `{}`: the card was asked anyway.",
            off.card[0]
        )));
    }
    if !on.card.iter().any(|l| l.contains("drew=1")) {
        let said = on.card.first().map_or("nothing", String::as_str);
        return Ok(Some(format!(
            "with {KEY} = true the card drew no page; the trace said {said}."
        )));
    }
    if off.pixels.is_empty() || off.pixels.len() != on.pixels.len() {
        return Err(Error::new(format!(
            "the page's box held {} pixels off and {} on: the two launches did not frame the \
             page alike. SKIPPED.",
            off.pixels.len(),
            on.pixels.len()
        )));
    }
    let far = |a: u8, b: u8| a.abs_diff(b) > CHANNEL;
    let differ = off
        .pixels
        .iter()
        .zip(&on.pixels)
        .filter(|(a, b)| far(a.r, b.r) || far(a.g, b.g) || far(a.b, b.b))
        .count();
    #[allow(clippy::cast_precision_loss)]
    let share = differ as f64 / off.pixels.len() as f64;
    report.note(format!(
        "{differ} of {} page pixels differ between the processor and the card ({share:.5})",
        off.pixels.len()
    ));
    if share > DIFFER_MAX {
        return Ok(Some(format!(
            "the card's page differs from the processor's at {share:.5} of its pixels, over \
             {DIFFER_MAX}: the card does not draw the same page."
        )));
    }
    Ok(None)
}
