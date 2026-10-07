//! `changing_a_groups_unit_keeps_its_real_lengths` — a ce dimension group
//! calibrated so a member reads 1000 mm is switched to feet from the
//! dimension-groups panel; the member must read about 3.28 ft, not 1000 ft.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/group_unit_converts.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared_in, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1200,1000";
const INVOKE: &str = "mode.review,measure.manage_groups";
const FIXTURE: &str = "dimension-scaled.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/dimension-scaled.PROVENANCE.py`.";
/// The default group's row. Mirrors `panels::dimension_groups::REGION_ROW_PREFIX`.
const ROW: &str = "dimension-groups.row.0";
/// Mirrors `panels::dimension_groups::REGION_UNIT_COMBO`.
const COMBO: &str = "dimension-groups.unit.combo";
/// Decimal feet. Mirrors `REGION_UNIT_OPTION_PREFIX` plus `Unit::token`.
const FEET: &str = "dimension-groups.unit.option.ft";
const SHOWN: &str = "dimension-member-shown"; // ui-text-exempt: a trace event name, never displayed
/// The fixture's member: 200 pt at 5 mm/pt.
const REAL_MM: f64 = 1000.0;
const MM_PER_FOOT: f64 = 304.8;

/// See the module documentation.
pub struct ChangingAGroupsUnitKeepsItsRealLengths;

impl Check for ChangingAGroupsUnitKeepsItsRealLengths {
    fn name(&self) -> &'static str {
        "changing_a_groups_unit_keeps_its_real_lengths"
    }

    fn defect(&self) -> &'static str {
        "a ce dimension group measuring 1000 mm, switched to feet, reads 1000 ft: the unit \
         changed and the scale did not"
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let doc = ctx.out("group-unit-converts.pdf");
    std::fs::copy(repo_fixture(FIXTURE, METHOD)?, &doc)
        .map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("group_unit_converts.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("group_unit_converts.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(60);
    Ok((session, pointer))
}

/// Click `region` where it was last declared; an error naming it when it is not.
fn press(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    region: &str,
) -> Result<()> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let (rect, vp) = declared_in(&session.trace()?, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "`{region}` was never declared. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    pointer.click_in(session, vp.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(30);
    Ok(())
}

/// The leading number of a shown measurement, `"3.28 ft"` → 3.28.
fn leading_number(text: &str) -> Option<f64> {
    text.split_whitespace().next()?.parse().ok()
}

fn steps(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    press(ctx, session, pointer, ROW)?;
    press(ctx, session, pointer, COMBO)?;
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, FEET)?;
    let trace = session.trace()?;
    let Some(shown) = trace.last_after(SHOWN, mark) else {
        return Ok(Some(format!(
            "feet was chosen and no `{SHOWN}` line followed: the group's unit was never set. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("after feet: `{}`", shown.raw));
    let text = shown.get("text").unwrap_or_default();
    let want = REAL_MM / MM_PER_FOOT;
    let right =
        text.ends_with(" ft") && leading_number(text).is_some_and(|v| (v - want).abs() < 0.01);
    Ok((!right).then(|| {
        format!(
            "the member reads `{text}` in feet; 1000 mm is {want:.2} ft. A unit change must \
             re-express the scale, not keep it. Trace: {}.",
            session.trace_path().display()
        )
    }))
}
