//! `checks::model_light_map_card` — **Colour by light level is measured on
//! the graphics card, and its map is the processor's**
//!
//! Opens `fixtures/glow-room.prc` in the viewer and ticks *Colour by light
//! level* (`model_light_map`'s gesture), once as shipped and once with
//! `PDFCER_3D_BOUNCE_CPU=1`.
//!
//! Oracles: the shipped run's `model-light-map-ready` says `card=1` after a
//! `model-light-map-card built=card` line, the forced run's says `card=0`
//! and traced no card build; both maps cover the same `triangles`; and their
//! `lo` and `hi` agree within [`TOLERANCE`] of the processor's `hi`. A pass
//! missing a run of directions, reading the wrong input or finishing from
//! an unsummed read-back leaves the lit room's range far off.

use crate::checks::driving::repo_fixture;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_light_map::{FAILED, READY, WAITS, steady, toggle};
use crate::checks::model_view_window::{launch_with_model_env, ui_rect};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const MODEL: &str = "glow-room.prc";
const METHOD: &str = "python fixtures/glow-room.PROVENANCE.py <pdfcer.exe>";
const BUILT: &str = "model-light-map-card"; // ui-text-exempt: a trace event name
/// The largest difference of either end of the range, as a share of the
/// processor's `hi`.
const TOLERANCE: f64 = 0.02;

/// See the module documentation.
pub struct TheLightMapIsMeasuredOnTheCard;

impl Check for TheLightMapIsMeasuredOnTheCard {
    fn name(&self) -> &'static str {
        "the_light_map_is_measured_on_the_card"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's Colour by light level is measured on the processor although the card \
         draws the picture, or the card's map is not the processor's"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (card, card_built) = match map(ctx, report, "map-card", &[])? {
        Err(failure) => return Ok(Some(failure)),
        Ok(found) => found,
    };
    report.note(format!("as shipped: `{}`", card.raw));
    let forced = [("PDFCER_3D_BOUNCE_CPU", "1".to_owned())];
    let (cpu, cpu_built) = match map(ctx, report, "map-processor", &forced)? {
        Err(failure) => return Ok(Some(failure)),
        Ok(found) => found,
    };
    report.note(format!("forced to the processor: `{}`", cpu.raw));
    if !(card_built && card.get("card") == Some("1")) {
        return Ok(Some(format!(
            "★★ the card draws the picture and the map came with card={} and {} `{BUILT} \
             built=card`.",
            card.get("card").unwrap_or("absent"),
            if card_built { "a" } else { "no" }
        )));
    }
    if cpu_built || cpu.get("card") != Some("0") {
        return Ok(Some(format!(
            "`PDFCER_3D_BOUNCE_CPU=1` still measured on the card: card={}.",
            cpu.get("card").unwrap_or("absent")
        )));
    }
    let n = |l: &TraceLine, k: &str| l.get(k).and_then(|v| v.parse::<f64>().ok());
    if n(&card, "triangles") != n(&cpu, "triangles") {
        return Ok(Some(format!(
            "★★★ the two maps cover {:?} and {:?} triangles: not one lattice.",
            card.get("triangles"),
            cpu.get("triangles")
        )));
    }
    let (Some(hi), Some(lo)) = (n(&cpu, "hi"), n(&cpu, "lo")) else {
        return Ok(Some("the processor's map has no lo or hi.".to_owned()));
    };
    let off = |k: &str, p: f64| n(&card, k).map_or(f64::INFINITY, |c| (c - p).abs() / hi);
    let (off_lo, off_hi) = (off("lo", lo), off("hi", hi));
    // A figure the line lacks is infinitely off.
    Ok(
        (!(hi > 0.0 && off_lo <= TOLERANCE && off_hi <= TOLERANCE)).then(|| {
            format!(
                "★★★ the card's map spans {:?}..{:?} and the processor's {lo}..{hi}: {:.2}% and \
             {:.2}% of its top apart (at most {:.0}%).",
                card.get("lo"),
                card.get("hi"),
                off_lo * 100.0,
                off_hi * 100.0,
                TOLERANCE * 100.0
            )
        }),
    )
}

/// One run's map line, and whether a card build preceded it.
fn map(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
    env: &[(&str, String)],
) -> Result<std::result::Result<(TraceLine, bool), String>> {
    let model = repo_fixture(MODEL, METHOD)?;
    let (session, pointer) =
        launch_with_model_env(ctx, report, stem, &model.to_string_lossy(), env)?;
    if let Some(failure) = open_viewer(ctx, &session, &pointer)? {
        return Ok(Err(failure));
    }
    let ui_rect = ui_rect(ctx)?;
    if steady(&session)?.is_none() {
        return Ok(Err("the viewer drew no picture line.".to_owned()));
    }
    if !toggle(&session, &pointer, ui_rect)? {
        return Ok(Err(
            "no *Colour by light level* switch in the options menu.".to_owned(),
        ));
    }
    for _ in 0..WAITS {
        let trace = session.trace()?;
        if let Some(line) = trace.last(READY) {
            let built = trace.events(BUILT).any(|l| l.get("built") == Some("card"));
            return Ok(Ok((line.clone(), built)));
        }
        if let Some(line) = trace.last(FAILED) {
            return Ok(Err(format!("★★★ the light map failed: {}", line.raw)));
        }
        session.settle(15);
    }
    Ok(Err(
        "★★★ the switch is on but no light map arrived.".to_owned()
    ))
}
