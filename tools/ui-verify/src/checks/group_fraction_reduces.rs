//! `a_groups_fractions_reduce_to_lowest_terms` — a ce dimension group switched
//! to feet and inches in sixteenths reads 6/16 with lowest terms off and 3/8
//! with it on, set from the Dimension groups tab's Numbers editor.
//!
//! Drives `dimension-scaled.pdf` (one 1000 mm member) off the desktop with the
//! scripted pointer: the default group's row, the unit combo, feet and inches,
//! the denominator combo, 16, then the lowest-terms box. 1000 mm is 39.37 in,
//! which rounds to 6/16. Oracle: the two readings either side of the box are
//! one `6/16` and one `3/8`, in whichever order the engine's default puts them.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::group_unit_converts::press;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1200,1000";
const INVOKE: &str = "mode.review,measure.manage_groups";
const FIXTURE: &str = "dimension-scaled.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/dimension-scaled.PROVENANCE.py`.";
const ROW: &str = "dimension-groups.row.0";
const COMBO: &str = "dimension-groups.unit.combo";
const FEET_INCHES: &str = "dimension-groups.unit.option.ft-in";
/// Mirrors `dimensionoverrides::REGION_DENOMINATOR`.
const DENOMINATOR: &str = "precision.denominator";
const SIXTEENTHS: &str = "precision.denominator.16";
/// Mirrors `dimensionoverrides::REGION_REDUCE`.
const REDUCE: &str = "precision.reduce";
const SHOWN: &str = "dimension-member-shown"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AGroupsFractionsReduceToLowestTerms;

impl Check for AGroupsFractionsReduceToLowestTerms {
    fn name(&self) -> &'static str {
        "a_groups_fractions_reduce_to_lowest_terms"
    }

    fn defect(&self) -> &'static str {
        "a ce dimension group's lowest-terms switch does not change 6/16 to 3/8"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let driven = launch(ctx, &mut report).and_then(|(session, pointer)| {
            let outcome = steps(ctx, &mut report, &session, &pointer);
            let parked = pointer.gone(&session);
            match outcome? {
                Some(failure) => Ok(Some(failure)),
                None => parked.map(|_| None),
            }
        });
        match driven {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(ctx: &CheckContext, report: &mut CheckReport) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe, or build the default."))?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out("group-fraction-reduces.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("group_fraction_reduces.trace.txt"));
    spec.pdf = Some(doc);
    for (k, v) in [
        (ctx.profile.diag_env.0, ctx.profile.diag_env.1),
        SHELL_DIAG_ENV,
        (viewport_env, OFFSCREEN),
        ("PDFCER_DIAG_INVOKE", INVOKE),
    ] {
        spec.env.push((k.to_owned(), v.to_owned()));
    }
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer =
        ScriptedPointer::attach(&mut spec, ctx.out("group_fraction_reduces.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(60);
    Ok((session, pointer))
}

/// Press `region` and return the member text the next `dimension-member-shown` carries.
fn reading_after(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<Option<String>> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, region)?;
    Ok(session
        .trace()?
        .last_after(SHOWN, mark)
        .and_then(|l| l.get("text").map(str::to_owned)))
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    press(ctx, session, pointer, ROW)?;
    press(ctx, session, pointer, COMBO)?;
    press(ctx, session, pointer, FEET_INCHES)?;
    press(ctx, session, pointer, DENOMINATOR)?;
    let Some(sixteenths) = reading_after(ctx, session, pointer, SIXTEENTHS)? else {
        return Ok(Some(format!(
            "sixteenths was chosen and no `{SHOWN}` line followed: the Numbers editor did \
             not reach the group. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("in sixteenths: `{sixteenths}`"));
    let Some(toggled) = reading_after(ctx, session, pointer, REDUCE)? else {
        return Ok(Some(format!(
            "lowest terms was toggled and no `{SHOWN}` line followed. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("lowest terms toggled: `{toggled}`"));
    let pair = [sixteenths.as_str(), toggled.as_str()];
    let unreduced = pair.iter().filter(|t| t.contains("6/16")).count();
    let reduced = pair.iter().filter(|t| t.contains("3/8")).count();
    Ok((unreduced != 1 || reduced != 1).then(|| {
        format!(
            "1000 mm in sixteenths must read 6/16 one way and 3/8 the other; it read \
             `{sixteenths}` then `{toggled}`. Trace: {}.",
            session.trace_path().display()
        )
    }))
}
