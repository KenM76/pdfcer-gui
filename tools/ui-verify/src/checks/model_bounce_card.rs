//! `checks::model_bounce_card` — **the 3D viewer builds the bounce light on
//! the graphics card, and it is the processor's bounce**
//!
//! Drives `checks::model_lighting`'s sequence twice on `floor-box.prc`: once
//! as shipped, once with `PDFCER_3D_BOUNCE_CPU=1`. Each ends with the lights
//! fixed to the model, whose bounce is the last one built.
//!
//! Oracles, from the `model-view-bounce built=` lines: the shipped run's last
//! build says `built=card` (a card that fell back says `built=processor
//! reason=…`, quoted in the failure); the forced run's says
//! `built=processor`; and their `irradiance=` means agree within
//! [`TOLERANCE`]. A card build that skipped a pass, read the wrong atlas or
//! bound a sampler to the wrong table is off by a percent or more.

use crate::checks::driving::repo_fixture;
use crate::checks::model_lighting;
use crate::checks::model_view_window::launch_with_model_env;
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::report::CheckReport;

const MODEL: &str = "floor-box.prc";
const METHOD: &str = "Rebuild it with `python fixtures/floor-box.PROVENANCE.py <pdfcer.exe>`.";
const BUILT: &str = "model-view-bounce"; // ui-text-exempt: a trace event name
/// The largest relative difference of the two means. On this fixture the two
/// builds agree to 0.004%; a card build missing its second bounce is 1% off.
const TOLERANCE: f64 = 0.005;

/// See the module documentation.
pub struct TheCardBuildsTheProcessorsBounce;

impl Check for TheCardBuildsTheProcessorsBounce {
    fn name(&self) -> &'static str {
        "the_card_builds_the_processors_bounce"
    }

    fn defect(&self) -> &'static str {
        "the 3D viewer builds its bounce light on the processor although the card can, or the \
         card's bounce is not the processor's"
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
    let card = match last_build(ctx, report, "bounce-card", &[])? {
        Err(failure) => return Ok(Some(failure)),
        Ok(line) => line,
    };
    report.note(format!("as shipped: `{}`", card.0));
    if card.1 != "card" {
        return Ok(Some(format!(
            "★★ the card can render the viewer and its last bounce was built on the processor: \
             `{}`.",
            card.0
        )));
    }
    let forced = [("PDFCER_3D_BOUNCE_CPU", "1".to_owned())];
    let cpu = match last_build(ctx, report, "bounce-processor", &forced)? {
        Err(failure) => return Ok(Some(failure)),
        Ok(line) => line,
    };
    report.note(format!("forced to the processor: `{}`", cpu.0));
    if cpu.1 != "processor" {
        return Ok(Some(format!(
            "`PDFCER_3D_BOUNCE_CPU=1` still built on the card: `{}`.",
            cpu.0
        )));
    }
    let off = (card.2 - cpu.2).abs() / cpu.2.abs().max(1e-12);
    Ok((cpu.2 <= 0.0 || off > TOLERANCE).then(|| {
        format!(
            "★★★ the card's bounce has mean irradiance {:.6} and the processor's {:.6}, {:.1}% \
             apart (at most {:.1}%).",
            card.2,
            cpu.2,
            off * 100.0,
            TOLERANCE * 100.0
        )
    }))
}

/// One driven run's last `model-view-bounce built=` line: raw, where, mean.
fn last_build(
    ctx: &CheckContext,
    report: &mut CheckReport,
    stem: &str,
    env: &[(&str, String)],
) -> Result<std::result::Result<(String, String, f64), String>> {
    let model = repo_fixture(MODEL, METHOD)?;
    let (session, pointer) =
        launch_with_model_env(ctx, report, stem, &model.to_string_lossy(), env)?;
    if let Some(failure) = model_lighting::drive(ctx, report, &session, &pointer)? {
        return Ok(Err(failure));
    }
    let trace = session.trace()?;
    let Some(line) = trace
        .events(BUILT)
        .filter(|l| l.get("built").is_some())
        .last()
    else {
        return Ok(Err(format!(
            "the bounce arrived and no `{BUILT} built=` line says where."
        )));
    };
    let mean = line.get("irradiance").and_then(|m| m.parse::<f64>().ok());
    Ok(match (line.get("built"), mean) {
        (Some(at), Some(mean)) => Ok((line.raw.clone(), at.to_owned(), mean)),
        _ => Err(format!("`{}` carries no `irradiance=`.", line.raw)),
    })
}
