//! `the_left_rail_is_reachable_and_constant_width` — the strip O123 part 7 asked
//! for, proved on a running build.
//!
//! # What this is for — `OPERATOR_REQUESTS.md` **O123** part 7 and **O126**
//!
//! > *"the navigate selectors and some other related selection controls (lasso
//! > tool when we implement one, etc) and these will fold up into a drop down
//! > arrow if space becomes scarce."*
//!
//! > *"also add rotate pages to that area, and those should be available in
//! > every mode including read."*
//!
//! # ★★★ Why this check has to exist, and why it is THIS check
//!
//! **The rail is the feature that shipped the 2026-08-10 defect.** Bookmarks,
//! Layers and Signatures went out **unreachable**, each with a rail entry, each
//! publishing a perfectly healthy rectangle, and **every gate green** — because
//! the dock's rect channel published *layout* and every check that read it was
//! treating the answer as *visibility*. `SHELL_LAYOUT_PROPOSAL.md` §5 made
//! converting that channel a **precondition** for scheduling the rail at all,
//! on exactly the ground that no driven check could otherwise tell a working
//! rail from that defect. The channel was converted on 2026-09-04.
//!
//! ⇒ So every region this check reads comes through
//! `crate::diag::ui_rect_visible`, and a `declared` here is a claim about
//! **reachability**, not about layout. That is what makes the check worth
//! running rather than a re-statement of the unit tests.
//!
//! # The five assertions, and why none is redundant
//!
//! | # | assertion | the build it fails on |
//! |---|---|---|
//! | 1 | `dock.left.toolrail` is on screen in **every** mode | a rail reserved but not drawn; a rail that only exists in Edit |
//! | 2 | all five panel-tab rows are reachable in **every** mode | the 2026-08-10 defect, restored — and the one thing the fold ladder may never do |
//! | 3 | the rail's **x-extent is the same in all three modes**, and is `WIDTH_PTS` | a rail sized from its widest word: the R128 fit-zoom loop, which a unit test can only assert about a number the renderer might not use |
//! | 4 | `pages.rotate_*` are reachable **in Read** | a build that quietly mode-gated them back, which is a silent reversal of an operator decision |
//! | 5 | `view.tool_node` is **absent in Read** and present in Edit | an authoring control sitting on a mode whose own dispatch refuses it — shipped once already, fixed 2026-09-04 |
//!
//! ★ Assertion 3 is the one that cannot be had any other way. The unit test
//! `the_width_is_constant_at_every_rung_and_every_budget` asserts that the
//! **planner** reports a constant; it cannot assert that the **renderer** used
//! it. Only a rect from a running build can, and only by comparing across
//! modes whose contents differ — which is why this check switches modes rather
//! than measuring once.
//!
//! # ⚠ NOT RUN
//!
//! **This check was written and has not been executed.** The operator is at his
//! keyboard and a watchdog kills GUI processes on sight, so `ui-verify` was not
//! launched. His standing instruction is that missing driven verification must
//! not stop the work or the release; it must be *named*. It is named here and
//! in the report.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment, declared, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// A fixture with pages, because every rail command is `enabled_when("doc.pages")`
/// and a rail of disabled rows would make assertion 4 unable to fail.
const FIXTURE: &str = "fixtures/a1-titleblock.pdf";

/// The strip the dock reserves. Published by `egui_shell::dock::rail::draw`.
///
/// ★ `toolrail`, **not** `dock.left.rail` — that name belongs to the sliver a
/// *collapsed* side leaves behind, which is a different surface. Reading the
/// wrong one is the failure
/// `two_trace_lines_sharing_an_event_name_make_a_check_read_the_wrong_one`
/// records.
const STRIP: &str = "dock.left.toolrail";

/// The five panel tabs, as `crate::app::rail::region("tabs", id)` names them.
const TABS: [&str; 5] = [
    "rail.tabs.view.panel_pages",
    "rail.tabs.view.panel_bookmarks",
    "rail.tabs.view.panel_layers",
    "rail.tabs.view.panel_signatures",
    "rail.tabs.file.fonts",
];

/// Rotate — O126, and reachable in Read is the whole point of the assertion.
const ROTATE: [&str; 2] = [
    "rail.rotate.pages.rotate_left",
    "rail.rotate.pages.rotate_right",
];

/// The Points tool, which Read may not have.
const POINTS: &str = "rail.navigate.view.tool_node";

/// The rail's declared width, from `egui_shell::dock::rail::WIDTH_PTS`.
const WIDTH_PTS: f32 = 52.0;

/// How far the measured strip may be from [`WIDTH_PTS`] before it is a failure.
///
/// One point. Not a "roughly" — the width is a constant and the panel is drawn
/// with `exact_size`, so anything outside rounding is a build that computed it.
const WIDTH_TOLERANCE_PTS: f32 = 1.0;

/// See the module documentation.
pub struct TheLeftRailIsReachableAndConstantWidth;

impl Check for TheLeftRailIsReachableAndConstantWidth {
    fn name(&self) -> &'static str {
        "the_left_rail_is_reachable_and_constant_width"
    }

    fn defect(&self) -> &'static str {
        "the left rail is laid out but not reachable, folds the panel tabs it exists to keep \
         one click away, changes width between modes, or gates rotate out of Read — none of \
         which a unit test can see, because the arrangement a planner intends and the \
         arrangement a dock draws are two different facts"
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

/// One mode's worth of evidence: the strip's rect and every rail region that
/// was reachable while that mode was showing.
struct Sweep {
    mode: &'static str,
    strip: crate::geom::LRect,
    regions: Vec<String>,
}

impl Sweep {
    fn has(&self, name: &str) -> bool {
        self.regions.iter().any(|r| r == name)
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks the mode segments and then \
             looks; without input it can see one mode, and the width assertion needs three. \
             Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let pdf = ctx.source_root.clone().unwrap_or_default().join(FIXTURE);
    let pdf = if pdf.exists() {
        pdf
    } else {
        std::path::PathBuf::from(FIXTURE)
    };
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture {FIXTURE} is not on disk. This check cannot run on no document: \
             every command in the rail is `enabled_when(\"doc.pages\")`, so an empty build \
             would draw a strip of disabled rows and the reachability assertions would be \
             asking a question with no content behind it."
        )));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("left-rail.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(40);

    let driver = Driver::new(session.window());

    // ★★ The modes are visited in capability order and the trace is read after
    // EACH, not once at the end. `declared` is retirement-aware — a region that
    // stopped being drawn is not declared — but the channel is a change log, so
    // a single read at the end could not say *which mode* a region was
    // reachable in, and assertions 4 and 5 are entirely about that.
    let mut sweeps: Vec<Sweep> = Vec::new();
    for mode in ["read", "review", "edit"] {
        click_mode_segment(&session, &driver, ui_rect, mode)?;
        session.settle(30);
        let trace = session.trace()?;
        let Some(strip) = declared(&trace, ui_rect, STRIP) else {
            return Ok(Some(format!(
                "★★★ THE LEFT RAIL IS NOT ON SCREEN IN `{mode}`. `{STRIP}` was never \
                 published while that mode was showing, so either the dock reserved no strip \
                 or the application drew nothing into it. O123 part 7 makes this PERMANENT \
                 chrome — it is not a panel and it cannot be closed. Dock regions that did \
                 publish: {}.",
                list(&driving::declared_names(&trace, ui_rect, "dock.left"))
            )));
        };
        sweeps.push(Sweep {
            mode,
            strip,
            regions: driving::declared_names(&trace, ui_rect, "rail."),
        });
    }

    // --- 1 & 2: the tabs are reachable in every mode -------------------------
    //
    // ⚠ This is the 2026-08-10 assertion, and it is the reason the whole
    // feature waited for the visibility channel. A `rail.tabs.*` rect that is
    // published but 90 % outside its clip does NOT appear here.
    for sweep in &sweeps {
        for tab in TABS {
            if !sweep.has(tab) {
                return Ok(Some(format!(
                    "★★★ `{tab}` IS NOT REACHABLE IN `{}`. The five panel tabs are the one \
                     group the rail may never fold — \"all five panels one click away\" is \
                     its entire argument for existing, and a rail that folds them is strictly \
                     worse than the tab stack it replaced. This is also the exact shape of the \
                     2026-08-10 defect: a rail entry that is laid out and cannot be reached. \
                     Rail regions reachable in `{}`: {}.",
                    sweep.mode,
                    sweep.mode,
                    list(&sweep.regions)
                )));
            }
        }
        report.note(format!(
            "`{}`: strip at {:?}, {} rail regions reachable",
            sweep.mode,
            sweep.strip,
            sweep.regions.len()
        ));
    }

    // --- 3: the width is the same constant in every mode ---------------------
    //
    // ★ The assertion the unit tests cannot make. They pin what the planner
    // REPORTS; this pins what the dock DREW, across three modes whose rail
    // contents differ — which is precisely the input a content-derived width
    // would react to.
    for sweep in &sweeps {
        let width = sweep.strip.max.x - sweep.strip.min.x;
        if (width - WIDTH_PTS).abs() > WIDTH_TOLERANCE_PTS {
            return Ok(Some(format!(
                "★★★ THE RAIL IS {width:.1} pt WIDE IN `{}`, NOT {WIDTH_PTS:.0}. The width is \
                 a CONSTANT at every rung and in every mode, and it is the whole R128 safety \
                 argument for this surface: a rail sized from its widest word moves the \
                 canvas, which re-fits the zoom, which moves the rail — the loop this project \
                 measured at 230 % → 224 % → 215 % over three frames.",
                sweep.mode
            )));
        }
    }
    let widths: Vec<f32> = sweeps
        .iter()
        .map(|s| s.strip.max.x - s.strip.min.x)
        .collect();
    let spread = widths.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        - widths.iter().copied().fold(f32::INFINITY, f32::min);
    if spread > WIDTH_TOLERANCE_PTS {
        return Ok(Some(format!(
            "★★★ THE RAIL CHANGES WIDTH BETWEEN MODES: {widths:?}. Read drops the Points tool \
             and Review drops nothing, so a rail whose width tracks its contents would move by \
             exactly this much — and moving is the defect, not the amount."
        )));
    }
    report.note(format!("constant width across all three modes: {widths:?}"));

    // --- 4: rotate is reachable in READ — O126, and it is his call ------------
    let read = sweeps
        .iter()
        .find(|s| s.mode == "read")
        .ok_or_else(|| Error::new("the read sweep is missing"))?;
    for id in ROTATE {
        if !read.has(id) {
            return Ok(Some(format!(
                "★★★ `{id}` IS NOT REACHABLE IN READ. O126, verbatim: *\"also add rotate pages \
                 to that area, and those should be available in every mode including read.\"* \
                 ⚠ This is the assertion that guards an operator decision against being \
                 quietly reversed by a later reader who notices that rotate writes `/Rotate` \
                 and \"fixes\" Read to author nothing. It is his call and it is recorded in \
                 `crate::shell::manifest::rail`'s header. Rail regions reachable in read: {}.",
                list(&read.regions)
            )));
        }
    }
    report.note("rotate is reachable in Read — O126, and deliberately so");

    // --- 5: Points is absent in Read and present in Edit ---------------------
    if read.has(POINTS) {
        return Ok(Some(format!(
            "★★ `{POINTS}` IS DRAWN IN READ. The Points tool edits the nodes of a path and \
             Read cannot; R9 says an unavailable capability renders NOTHING rather than \
             greying, and on permanent chrome a control that is wrong for the mode is wrong on \
             screen for the whole session rather than for one click. This exact defect shipped \
             once and was fixed on 2026-09-04."
        )));
    }
    let edit = sweeps
        .iter()
        .find(|s| s.mode == "edit")
        .ok_or_else(|| Error::new("the edit sweep is missing"))?;
    if !edit.has(POINTS) {
        return Ok(Some(format!(
            "★★ `{POINTS}` IS ABSENT IN EDIT TOO. The mode gate has stopped being a gate and \
             become a deletion — which would make assertion 5 above pass for the wrong reason \
             for ever. Rail regions reachable in edit: {}.",
            list(&edit.regions)
        )));
    }
    report.note("Points is drawn in Edit and absent in Read");

    report.note(
        "★★ the rail is reachable in every mode, keeps all five panel tabs, holds one constant \
         width, and gates Points without gating rotate",
    );
    Ok(None)
}
