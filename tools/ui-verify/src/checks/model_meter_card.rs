//! `checks::model_meter_card` — **the light meter's bounce is built on the
//! graphics card, and the reading is the processor's**
//!
//! Opens `fixtures/glow-room.prc` in the viewer and reads the light at the
//! first point a lamp shines on straight (`model_light_meter`'s sweep), once
//! as shipped and once with `PDFCER_3D_BOUNCE_CPU=1`. The sweep clicks the
//! same spots in the same order, so both runs read the same point.
//!
//! Oracles: the shipped run traced `model-light-meter-bounce built=card`
//! before its reading, the forced run never did; the two readings' `direct`
//! agree to the trace's rounding (the card builds only the bounce); and
//! their `bounced` agree within [`TOLERANCE`]. A card build missing a pass,
//! reading the wrong atlas or finishing the meter from no atlas leaves
//! `bounced` at zero or far off.

use crate::checks::driving::repo_fixture;
use crate::checks::model_inferences::open_viewer;
use crate::checks::model_light_meter::{first_lit_reading, number};
use crate::checks::model_pick::Drive;
use crate::checks::model_view_window::launch_with_model_env;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;
use crate::trace::TraceLine;

const MODEL: &str = "glow-room.prc";
const METHOD: &str = "python fixtures/glow-room.PROVENANCE.py <pdfcer.exe>";
const BUILT: &str = "model-light-meter-bounce"; // ui-text-exempt: a trace event name
/// The largest relative difference of the two `bounced` figures.
const TOLERANCE: f64 = 0.02;
/// The trace's rounding of `direct`, twice.
const DIRECT_SLACK: f64 = 2e-6;

/// See the module documentation.
pub struct TheMeterBuildsItsBounceOnTheCard;

impl Check for TheMeterBuildsItsBounceOnTheCard {
    fn name(&self) -> &'static str {
        "the_light_meter_builds_its_bounce_on_the_card"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer's light meter builds its bounce on the processor although the card draws \
         the picture, or the card's reading is not the processor's"
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
    let (card, card_built) = match reading(ctx, report, "meter-card", &[])? {
        Err(failure) => return Ok(Some(failure)),
        Ok(found) => found,
    };
    report.note(format!("as shipped: `{}`", card.raw));
    let forced = [("PDFCER_3D_BOUNCE_CPU", "1".to_owned())];
    let (cpu, cpu_built) = match reading(ctx, report, "meter-processor", &forced)? {
        Err(failure) => return Ok(Some(failure)),
        Ok(found) => found,
    };
    report.note(format!("forced to the processor: `{}`", cpu.raw));
    if !card_built {
        return Ok(Some(format!(
            "★★ the card draws the picture and the meter's reading came with no `{BUILT} \
             built=card`."
        )));
    }
    if cpu_built {
        return Ok(Some(format!(
            "`PDFCER_3D_BOUNCE_CPU=1` still traced `{BUILT} built=card`."
        )));
    }
    let direct = (number(&card, "direct"), number(&cpu, "direct"));
    let one_point = (direct.0 - direct.1).abs() <= DIRECT_SLACK;
    if !one_point {
        return Ok(Some(format!(
            "★★★ the two readings' direct light differ ({} and {}): they are not of one point.",
            direct.0, direct.1
        )));
    }
    let (c, p) = (number(&card, "bounced"), number(&cpu, "bounced"));
    let off = (c - p).abs() / p.abs().max(1e-12);
    // A figure the line lacks is NaN, which fails every comparison.
    Ok((!(p > 0.0 && off <= TOLERANCE)).then(|| {
        format!(
            "★★★ the card's reading has bounced light {c:.6} and the processor's {p:.6}, {:.2}% \
             apart (at most {:.0}%).",
            off * 100.0,
            TOLERANCE * 100.0
        )
    }))
}

/// One run's first lit reading, and whether a card build preceded it.
fn reading(
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
    let drive = Drive {
        ctx,
        session: &session,
        pointer: &pointer,
    };
    let line = match first_lit_reading(&drive)? {
        Ok(line) => line,
        Err(failure) => return Ok(Err(failure)),
    };
    let built = session
        .trace()?
        .events(BUILT)
        .any(|l| l.get("built") == Some("card"));
    Ok(Ok((line, built)))
}
