//! `the_standards_presets_group_is_reachable` — the conformance presets'
//! heading is on screen in the Settings window, not merely laid out in it.
//!
//! # What this is for, `OPERATOR_REQUESTS.md` O100
//!
//! > *"the engine I think has a couple of new options for colour rendering that
//! > we might need to surface and set for our standards presets."*
//!
//! The *surfacing* half of that ask is already held by two unit-level
//! contracts, and they are strong ones:
//!
//! * `every_setting_the_store_carries_has_a_control_in_this_window` enumerates
//!   the **engine's own** `Settings::write_to_string` at runtime and demands a
//!   control for each key. It has caught a newly-added setting four times.
//! * `every_key_a_standard_leaves_alone_has_an_operator_facing_title` does the
//!   same for `PresetKey`.
//!
//! Neither of them, and nothing else, answers the question this check exists
//! for: **can the operator actually get to it?**
//!
//! # ★★★ It asserts the HEADING, not the row, and the first draft got that wrong
//!
//! The first version of this check demanded the `settings.presets` region — the
//! row itself — and **failed on a correct build**. The presets live in a
//! `CollapsingHeader` that ships **CLOSED, deliberately, since 2026-08-26**:
//! expanded, ten radios plus a detail block is about 730 pt in a scroll area
//! roughly 620 pt tall, and it pushed *every other setting and every other group
//! heading below the fold*. The check's own failure text would have sent
//! somebody to undo that decision.
//!
//! ★★ That is the second time in one session a check I wrote was wrong rather
//! than the code, and both had the same shape: **a measurement aimed at the
//! wrong surface looks exactly like a broken feature.** The rule that catches
//! it is to ask what a failing assertion actually *sampled* before asking what
//! is broken.
//!
//! ⇒ So the claim is: the **heading** is visible, which for a collapsed group is
//! the whole of "can the operator get to it". The row behind it is one click
//! away by design, and opening it needs the pointer — there is no command that
//! focuses this group the way `tools.font_folders` focuses Fonts, so a no-input
//! route to the row does not exist. Named rather than left as a silence.
//!
//! # ★★★ Why "reachable" needs its own check, and it is a named hazard here
//!
//! `D:/dev/rag/egui/` records this project shipping **panels that were
//! unreachable in real builds with every gate green**. A control inside a
//! `ScrollArea` is laid out whether or not anyone can see it, and `egui` will
//! happily report a rectangle for a row a hundred points below the fold. So a
//! test that finds the widget in the tree, and a check that finds a rectangle in
//! the trace, can both be satisfied by a row nobody can reach.
//!
//! Both the heading and the row publish through `crate::diag::ui_rect_visible`
//! — intersected with the clip rectangle, so an off-screen row publishes
//! nothing at all. This check asserts the region exists, which is therefore a
//! claim about **visibility** and not merely about layout.
//!
//! ★★ The same reasoning already burned this suite once in the other direction:
//! `settings_headings_legible` measured three headings that were laid out below
//! the fold and sampled the Pages panel and the drawing behind the dialog,
//! reporting three illegible headings in a dialog whose visible headings
//! measured 13.91:1. The fix there was `ui_rect_visible`; this check is what
//! makes its absence detectable rather than silent.
//!
//! # No input
//!
//! `PDFCER_DIAG_INVOKE` raises the command at startup, so the Settings window
//! opens without a pointer. Like `title_build_stamp` and `field_shading`, this
//! can run beside somebody using the machine.
//!
//! Settings is also one of the few windows that must work with **nothing open**
//! — the presets are a preference, not a property of a document — so this drives
//! it on an empty shell, which is what proves that.
//!
//! # What a passing run does NOT prove
//!
//! That the *claim sentences* are shown. Those are labels with no region of
//! their own, and they appear only under the selected standard — which means
//! selecting one, which means the pointer. Their content is held by
//! `a_pdf_x_preset_says_a_composite_viewer_will_differ_and_pdf_a_does_not` and
//! their delivery would need a second, input-gated check. Named here so the gap
//! is a decision rather than an oversight.

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
            Err(skip) => report.skip(skip.to_string()),
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

    // ★ The two-way diagnosis. If no heading published either, the window never
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

    // ★ The row's state line is NOT asserted, and the absence is deliberate.
    //
    // `preset::row` runs only while the group is expanded, and the group ships
    // closed. Demanding `settings-preset` here would be demanding the group be
    // open — which is the mistake the first draft of this check made in the
    // other direction. It is read opportunistically instead: if a future build
    // opens the group by default, the line appears and is reported, and this
    // check does not have to change to notice.
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
