//! `the_standards_presets_group_is_reachable` — the conformance presets'
//! heading is on screen in the Settings window, not merely laid out in it.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/preset_group_reachable.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command raised at startup to open the window.
const INVOKE: &str = "file.settings";
/// The presets group's heading, published through `ui_rect_visible` — so it is
/// absent when the heading is below the fold, which is the 2026-08-26 defect.
const HEADING: &str = "settings.heading.presets";
/// The row's state trace: which standard is chosen, and whether Save is live.
const STATE: &str = "settings-preset";
/// A group heading, so a failure can tell "the window never opened" from "the
/// window opened and this row is not in it".
const ANY_HEADING: &str = "settings.heading.";

/// See the module documentation.
pub struct TheStandardsPresetsGroupIsReachable;

impl Check for TheStandardsPresetsGroupIsReachable {
    fn name(&self) -> &'static str {
        "the_standards_presets_group_is_reachable"
    }

    fn defect(&self) -> &'static str {
        "the conformance-standard presets are built and cannot be reached — laid out inside the \
         Settings window's scroll area but never on screen, which every unit test and every \
         gate would report as present"
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
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // No `--pdf`: Settings must work with nothing open, and driving it on an
    // empty shell is what proves that.
    let mut spec = LaunchSpec::new(&exe, ctx.out("preset_group_reachable.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}, no document open, no input",
        exe.display(),
        session.pid()
    ));
    // The Settings window is its own OS viewport and settles over several
    // frames.
    session.settle(50);

    let trace = session.trace()?;

    // The two-way diagnosis. If no heading published either, the window never
    // opened and the presets row is not the subject of the failure — reporting
    // "the presets row is missing" when the whole window is absent is a
    // confident, specific, wrong defect report, which this suite has produced
    // before and now guards against by naming the other cause first.
    let headings = declared_names(&trace, ui_rect, ANY_HEADING);
    if headings.is_empty() {
        return Ok(Some(format!(
            "`{INVOKE}` was raised at startup and NO settings group heading was published, so \
             the Settings window never opened. This is not a finding about the presets row — \
             the command has no claimant, or the window failed before drawing. Regions \
             beginning `settings`: {}.",
            list(&declared_names(&trace, ui_rect, "settings"))
        )));
    }
    report.note(format!(
        "the Settings window is open with {} group heading(s)",
        headings.len()
    ));

    if declared(&trace, ui_rect, HEADING).is_none() {
        return Ok(Some(format!(
            "★ THE PRESETS HEADING IS NOT ON SCREEN, though the Settings \
             window is open: no `{HEADING}` region. It is published through \
             `ui_rect_visible`, which intersects with the clip rectangle — so \
             an absence means the heading is laid out somewhere nobody can \
             reach it, NOT that it was never built. Every unit test and every \
             gate would still be green. This is the 2026-08-26 defect exactly, \
             when the presets were a bare row of ten radios and pushed every \
             group heading below the fold — so the first thing to check is \
             whether the group has been un-collapsed. Headings that DID \
             publish: {}.",
            list(&headings)
        )));
    }
    report.note("the presets group heading is on screen");

    // The row's state line is NOT asserted, and the absence is deliberate.
    //
    if let Some(state) = trace.last(STATE) {
        report.note(format!(
            "the group is expanded and the row reported its state: `{}`",
            state.raw
        ));
    } else {
        report.note(
            "the group is collapsed, as it ships — the row behind it is one \
             click away and reaching it needs the pointer",
        );
    }

    Ok(None)
}
