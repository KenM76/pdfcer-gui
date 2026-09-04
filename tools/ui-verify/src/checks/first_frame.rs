//! `the_first_frame_names_the_tools` — **zero clicks**, and the tools are on
//! screen.
//!
//! # What this is for
//!
//! Every other check in this suite drives a gesture. This one drives **nothing**
//! — it opens a document, enters Edit, and asks what an operator sees before
//! they have touched anything. That is the only question the last three weeks of
//! this project actually turned on.
//!
//! The sequence it exists to break:
//!
//! 1. `edit.text` and `edit.add_text` were registered, drawn, chord-bound and
//!    covered by passing driven checks. The operator reported them missing.
//! 2. Diagnosed as *"a discoverability defect"*. The Tool panel shipped, naming
//!    both, with their chords and their ribbon tab.
//! 3. **He came back the same day and still could not type.** Naming a
//!    four-step route is not removing it.
//! 4. The route was removed (one text tool, one click) — and the Tool panel's
//!    own list was **nearly shipped stale**, still naming the six old tools and
//!    not the new **Points** tool, which is the answer to *"how do I get to see
//!    the end points of an object"*.
//!
//! Step 4 is why this check exists. The panel whose entire job is to teach the
//! tools had no test that it taught the *current* ones, because every test asked
//! whether a named command was in the list — and a list can be complete about
//! yesterday's tools and silent about today's.
//!
//! # ★★ What it asserts, and why it is the region and not the strings
//!
//! That the four pointer tools' **rows** are drawn, in the Tool panel, inside
//! the visible client area, without a click.
//!
//! It deliberately does **not** read the sentences. Those live in
//! `crate::text::tool`, are gated by `check-ui-strings`, and are unit-tested;
//! asserting them here would be a second copy of the catalogue in a second
//! language, which is the drift `text::commands`' own rule warns about. What no
//! unit test can answer is *is it on screen* — and that is exactly the question
//! the three weeks were lost to.
//!
//! # ★ Why "inside the client area" is a real assertion
//!
//! Because the failure this project has already shipped once is a control drawn
//! **below the fold**: the dimension-groups window was tall enough to push its
//! own title bar off the desktop, which is the operator's complaint #3. A row
//! that draws at y = 1400 in a 900-pixel window is drawn, publishes a rect, and
//! is invisible. The rect alone is not enough; it has to be *in* the frame.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose tool list is the full one.
const MODE: &str = "edit";
/// The Tool panel's own region.
const TOOLS_REGION: &str = "tool.tools";
/// The four pointer tools, in the order the row draws them.
///
/// ★ The ORDER is asserted as well as the presence, and it is the palette order
/// every program in the class uses — arrow, white arrow, type, hand. An
/// operator's eye knows where to go before they read a label, and that only
/// works if this list and the ribbon band agree. They are two lists in two
/// files; this is what keeps them one order.
const TOOL_ROWS: [&str; 4] = [
    "tool.row.view.tool_select",
    "tool.row.view.tool_node",
    "tool.row.view.tool_text",
    "tool.row.view.tool_hand",
];

/// See the module documentation.
pub struct TheFirstFrameNamesTheTools;

impl Check for TheFirstFrameNamesTheTools {
    fn name(&self) -> &'static str {
        "the_first_frame_names_the_tools"
    }

    fn defect(&self) -> &'static str {
        "the panel whose job is to teach the tools does not name the current ones — it can be \
         complete about yesterday's tools and silent about today's, because every test of it \
         asks whether a NAMED command is listed and none asks what an operator sees"
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
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. The tool list needs a document to describe."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks ONE thing — the mode segment — \
             and then looks. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("first_frame.trace.txt"));
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

    // ★ The ONE click, and it is not a concession. A mode is a stance the
    // operator chooses, and Edit is the one whose tool list is complete; asking
    // what Read shows would be asking a different and easier question. Nothing
    // after this is clicked.
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(30);

    let trace = session.trace()?;
    let frame = session.frame()?;
    let client = frame.client_logical();

    let Some(tools) = driving::declared(&trace, ui_rect, TOOLS_REGION) else {
        return Ok(Some(format!(
            "★★ THE TOOL PANEL'S LIST DID NOT DRAW AT ALL. Either the panel is not in this \
             profile's `{MODE}` arrangement — which would be a defect in its own right, since it \
             is the surface an operator meets without asking — or `panels::tool::idle::tools` \
             returned before publishing `{TOOLS_REGION}`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the tool list drew at {tools:?}"));

    let mut missing = Vec::new();
    let mut offscreen = Vec::new();
    let mut order = Vec::new();
    for region in TOOL_ROWS {
        match driving::declared(&trace, ui_rect, region) {
            None => missing.push(region),
            Some(r) => {
                order.push((region, r.min.y));
                // ★ Drawn is not seen. See the module header: this project has
                // already shipped a window tall enough to push its own title
                // bar off the desktop.
                if r.max.y > client.max.y || r.min.y < client.min.y {
                    offscreen.push(region);
                }
            }
        }
    }

    if !missing.is_empty() {
        return Ok(Some(format!(
            "★★★ THE TOOL PANEL DOES NOT NAME: {missing:?}.\n\
             This is the defect this check exists for, and it is the one that cost three weeks \
             in a different costume: a surface whose entire job is to teach the tools, silent \
             about the tools that exist. `panels::tool::idle::rows` is the list. Note that \
             every OTHER test of that function asks whether a named command is present — which \
             is a question that stays green while the answer goes stale, because a list can be \
             complete about yesterday's tools and say nothing about today's."
        )));
    }
    if !offscreen.is_empty() {
        return Ok(Some(format!(
            "★★ THESE ROWS ARE DRAWN AND NOT VISIBLE: {offscreen:?}. The client area is \
             {client:?}. A row below the fold publishes a rect, passes every containment test \
             written against the PANEL, and cannot be read — which is complaint #3 (a window \
             whose content pushed its own title bar off the desktop) arriving in a dock column."
        )));
    }

    // ★ The order, which is the half a presence check cannot see. Palette order
    // — arrow, white arrow, type, hand — is what lets an operator's eye find a
    // tool before reading a label, and it only works if this list and the
    // ribbon band agree.
    let mut sorted = order.clone();
    sorted.sort_by(|a, b| a.1.total_cmp(&b.1));
    if sorted.iter().map(|(r, _)| *r).ne(TOOL_ROWS) {
        return Ok(Some(format!(
            "★ THE TOOLS ARE LISTED OUT OF PALETTE ORDER. Top to bottom: {:?}, expected {:?}.\n\
             Arrow, white arrow, type, hand is the order every program in this class uses, and \
             it is the order the View ▸ Navigate band draws them in. The two lists agreeing is \
             not decoration — this panel exists to teach the ribbon, and one that taught a \
             different order would be teaching something false.",
            sorted.iter().map(|(r, _)| *r).collect::<Vec<_>>(),
            TOOL_ROWS
        )));
    }

    report.note("★★ all four pointer tools are named, on screen, in palette order, with no click");
    Ok(None)
}
