//! `the_armed_tools_settings_are_in_properties` — every control the Tool panel
//! held is on screen in its new home.
//!
//! # What this is for — `OPERATOR_REQUESTS.md` **O123**, part 2
//!
//! > *"I never understood why there is a tool dock when everything can be in
//! > object and properties."*
//!
//! The Tool panel was dissolved. Its live controls — the text pen's face, size
//! and colour, the circular measure's pick list, and the three resize switches
//! — moved to `crate::panels::properties::tool`. **That move is the one thing in
//! O123 that could silently cost a capability**, and
//! `SHELL_LAYOUT_PROPOSAL.md` §3 was right to say so before the operator
//! overruled the remedy rather than the diagnosis.
//!
//! # ★★★ Why a unit test cannot close this, and this file can
//!
//! `panels::properties::tool::block_for` is a pure function and it IS unit
//! tested: `each_moved_control_has_a_tool_that_reaches_it` asserts the shipped
//! mapping from an armed tool to its block. That is worth having and it is not
//! enough, and the reason is written into the module those controls came from:
//!
//! > *"an option row added there is dead code that compiles, reads correctly,
//! > and draws nothing … Every unit test in the chain passed. Nothing tested
//! > that the control is on screen."*
//!
//! That was `the_line_weight_switch_reaches_the_resize` catching `scale_switches`
//! written into a branch `CanvasTool::Select` cannot reach. The branch is
//! different now; the failure mode is identical, and a **move between modules
//! is exactly when it recurs**.
//!
//! # ★★ Three launches, not three arming clicks
//!
//! Each block belongs to a different armed tool, so one arming cannot exercise
//! them all:
//!
//! | armed | block | regions asserted |
//! |---|---|---|
//! | the resting tool (`Select`) | the three resize switches | `properties.tool.scale.stroke`, `.insets`, `.distort` |
//! | *Add text* (`edit.add_text`) | the text pen | `properties.tool.text_pen` |
//! | radius/diameter (`measure.radius_diameter`) | the pick list | `properties.tool.measure_points` |
//!
//! ★ Each is a **separate launch** with its own `PDFCER_DIAG_INVOKE`, rather
//! than one session that arms three tools in turn. Two reasons, and the second
//! is the load-bearing one:
//!
//! 1. The commands live on three different ribbon tabs, so arming them in one
//!    session means driving the tab strip — which is a different check's
//!    subject and a way for this one to fail for a reason it is not about.
//! 2. **The dock layout persists.** A session that raised the Properties tab
//!    leaves it raised, so a later case in the same process would be testing a
//!    state the earlier case created. `scale_switch.rs` records what that costs:
//!    an order-dependence that only became reliably wrong once `LayoutStore`
//!    started flushing on exit. One launch per case has no shared state to
//!    normalise.
//!
//! Every region asserted is published through `crate::diag::ui_rect_visible`
//! against the panel's clip rectangle — which matters here more than anywhere,
//! because Properties is one `ScrollArea` and a control below its fold has a
//! perfectly healthy rectangle. `geometry_fields` is the recorded incident:
//! *"`Apply` was not [on screen]. The typed-geometry feature was complete,
//! wired, tested and unusable."*
//!
//! # ★ What it deliberately does NOT assert
//!
//! That the controls **work**. `the_line_weight_switch_reaches_the_resize`
//! already drives the stroke switch through to a resized annotation, and
//! `measure_circular_points` already presses a pick row and watches the point
//! leave the set. Both were re-pointed at these regions on 2026-09-04. This
//! check answers the one question neither of them asks about all three at once:
//! *are they there at all, after the move?*

use std::path::Path;

use crate::checks::driving::{self, SHELL_DIAG_ENV, declared, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The Properties panel's body compartment, as the dock reports it.
const PROPERTIES_BODY: &str = "dock.body.file.properties";
/// The Properties panel's dock tab header, for raising it from behind a sibling.
const PROPERTIES_TAB: &str = "dock.tab.file.properties";

/// One case: what to invoke at launch, and what must then be on screen.
struct Case {
    /// A human name for the block, for the report's notes.
    what: &'static str,
    /// The comma-separated `PDFCER_DIAG_INVOKE` list.
    invoke: &'static str,
    /// Every region the block must publish.
    regions: &'static [&'static str],
}

/// The three cases, one per block that moved.
const CASES: [Case; 3] = [
    Case {
        what: "the three resize switches",
        // ★ Nothing is armed. `CanvasTool::Select` is the default and the
        // application opens in it, which is the whole point of the branch these
        // live in — and the reason the first version of them was dead code was
        // that the old panel could not reach that branch at all.
        invoke: "mode.edit",
        regions: &[
            "properties.tool.scale.stroke",
            "properties.tool.scale.insets",
            "properties.tool.scale.distort",
        ],
    },
    Case {
        what: "the text pen",
        invoke: "mode.edit,edit.add_text",
        regions: &["properties.tool.text_pen"],
    },
    Case {
        what: "the radius tool's pick list",
        invoke: "mode.edit,measure.radius_diameter",
        regions: &["properties.tool.measure_points"],
    },
];

/// See the module documentation.
pub struct TheArmedToolsSettingsAreInProperties;

impl Check for TheArmedToolsSettingsAreInProperties {
    fn name(&self) -> &'static str {
        "the_armed_tools_settings_are_in_properties"
    }

    fn defect(&self) -> &'static str {
        "a control that was in the Tool panel is in no panel at all — the move to Properties \
         compiles, reads correctly and draws nothing, which is the exact shape that let three \
         resize switches ship unreachable with every unit test in the chain passing"
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
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new("no --pdf. Every one of these tools needs a page to be armed over.")
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check raises a dock tab when the panel is \
             behind a sibling. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    for (i, case) in CASES.iter().enumerate() {
        if let Some(failure) = probe(ctx, report, &exe, &pdf, ui_rect, case, i)? {
            return Ok(Some(failure));
        }
    }
    report.note("★★ every control the Tool panel held is reachable in Properties");
    Ok(None)
}

/// Drive one case in its own process.
fn probe(
    ctx: &CheckContext,
    report: &mut CheckReport,
    exe: &Path,
    pdf: &Path,
    ui_rect: &str,
    case: &Case,
    index: usize,
) -> Result<Option<String>> {
    let mut spec = LaunchSpec::new(exe, ctx.out(&format!("properties-tool-{index}.trace.txt")));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), case.invoke.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(45);
    let driver = Driver::new(session.window());
    raise_properties(&session, &driver, ui_rect, report)?;

    let trace = session.trace()?;
    let missing: Vec<&str> = case
        .regions
        .iter()
        .copied()
        .filter(|r| declared(&trace, ui_rect, r).is_none())
        .collect();
    if !missing.is_empty() {
        return Ok(Some(format!(
            "★★★ {} IS NOT ON SCREEN after `{}`: {missing:?} did not publish. The control was \
             in the Tool panel until O123 and the move is what this check exists to catch — a \
             block behind an arm nothing reaches compiles, reads correctly and draws nothing. \
             Two other explanations to rule out before touching `block_for`: the block may be \
             laid out below the Properties panel's fold, where `ui_rect_visible` publishes \
             nothing and the operator can reach nothing either; or the arming command may not \
             have run. Regions beginning `properties.tool`: {}. Trace: {}.",
            case.what,
            case.invoke,
            list(&driving::declared_names(&trace, ui_rect, "properties.tool")),
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ {} is on screen after `{}`",
        case.what, case.invoke
    ));
    Ok(None)
}

/// Put the Properties panel on screen, or SKIP.
///
/// ★ A dock tab header, never a ribbon toggle: a toggle would *unmount* a panel
/// that is already mounted, and this check would then report absent controls
/// about a panel it closed itself. Every mode's default arrangement mounts
/// Properties, so an absence here means the operator's persisted layout removed
/// it — a dock question, not this check's subject.
fn raise_properties(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<()> {
    let trace = session.trace()?;
    if declared(&trace, ui_rect, PROPERTIES_BODY).is_some() {
        return Ok(());
    }
    if let Some(tab) = declared(&trace, ui_rect, PROPERTIES_TAB) {
        driver.click_at(
            driving::frame_of(session, &trace, ui_rect, PROPERTIES_TAB)?.declared_center(tab),
        )?;
        session.settle(20);
        if declared(&session.trace()?, ui_rect, PROPERTIES_BODY).is_some() {
            report.note("raised the Properties panel by its dock header");
            return Ok(());
        }
    }
    let trace = session.trace()?;
    Err(Error::new(format!(
        "the Properties panel is not on screen and cannot be raised — neither \
         `{PROPERTIES_BODY}` nor `{PROPERTIES_TAB}` published. SKIPPED rather than failed: this \
         check's subject is the controls, not the dock. Regions beginning `dock.tab.`: {}.",
        list(&driving::declared_names(&trace, ui_rect, "dock.tab."))
    )))
}
