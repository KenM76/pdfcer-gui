//! `the_first_frame_names_the_armed_tool` — **zero clicks**, and the operator
//! is told what they are holding.
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
//!    not the new **Points** tool.
//!
//! # ★★★ What changed on 2026-09-04, and why this check was rewritten rather
//! than deleted
//!
//! `OPERATOR_REQUESTS.md` **O123** dissolved the Tool panel: *"The Tool panel
//! becomes a one-line tool status (name, one sentence, 'Put this tool down');
//! its buttons duplicate the ribbon and go."* So the four `tool.row.*` regions
//! this check used to demand **no longer exist**, and the list it defended is
//! gone by instruction.
//!
//! ⚠ **Deleting the check along with them would have been the wrong move, and
//! the reason is the one this whole file is about.** What was under test was
//! never *"is there a list"* — it was *"does the first frame, with no clicks,
//! tell the operator something true about what they can do."* That question
//! survives its answer changing, and the surface that answers it now is
//! `crate::app::toolstatus`: a strip the right dock reserves above its columns,
//! permanent, uncloseable, present at frame one.
//!
//! ★ If this check had been deleted, the *"nearly shipped stale"* failure of
//! step 4 would have lost its only driven guard, and the new strip would have
//! shipped with none — which is the SKIP-shaped hole `SHELL_LAYOUT_PROPOSAL.md`
//! §3.5 warns about in as many words: *"any check written against the existing
//! Tool panel must not be deleted along with it."*
//!
//! # ★★ The two regions, and why BOTH are asserted
//!
//! | region | published by | the claim |
//! |---|---|---|
//! | `dock.right.banner` | `egui_shell::dock::banner`, through the application's `ui_rect_visible` sink | **the strip's compartment is on screen** |
//! | `toolstatus` | `crate::app::toolstatus::banner`, through `ui_rect_visible` | **the application drew something into it** |
//!
//! A build whose handler returned early — no document, an unregistered command,
//! a `None` from the menu host — keeps the first and loses the second. A build
//! whose dock stopped reserving the strip loses both. Asserting only the dock's
//! would pass on an empty strip; asserting only the application's would pass on
//! a strip drawn outside the side it belongs to.
//!
//! # ★★★ And a pixel, because a rectangle cannot say that anything was painted
//!
//! `SHELL_LAYOUT_PROPOSAL.md` §3.5, on this exact check: *"It is vacuous if it
//! asserts only the region's presence, because the banner has a constant height
//! and will publish a rect whether or not it painted anything."* So the strip's
//! rectangle is sampled with [`crate::pixels::region_not_uniform`]. A strip that
//! laid out correctly and painted nothing is a flat block of panel colour, and
//! that is precisely what `is_uniform` reports.
//!
//! This is the two-channel discipline `read_mode_chrome.rs` states: *"the rect
//! is exact and cheap and would be satisfied by a build that moved the canvas
//! without repainting anything; the pixels cannot be faked by an arithmetic
//! error."*
//!
//! # ★ Why "inside the client area" is still a real assertion
//!
//! Because the failure this project has already shipped once is a control drawn
//! **below the fold**: the dimension-groups window was tall enough to push its
//! own title bar off the desktop, which is the operator's complaint #3. A strip
//! that draws at y = 1400 in a 900-pixel window is drawn, publishes a rect, and
//! is invisible.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode O123 gave the strip, and the one whose dock it changes.
const MODE: &str = "edit";
/// The dock's own name for the reserved strip — `egui_shell::dock::report::banner`.
const BANNER_REGION: &str = "dock.right.banner";
/// The application's name for what it drew into that strip —
/// `crate::app::toolstatus::REGION`.
const STATUS_REGION: &str = "toolstatus";

/// See the module documentation.
pub struct TheFirstFrameNamesTheArmedTool;

impl Check for TheFirstFrameNamesTheArmedTool {
    fn name(&self) -> &'static str {
        "the_first_frame_names_the_armed_tool"
    }

    fn defect(&self) -> &'static str {
        "the surface whose job is to say what the operator is holding is not on screen at frame \
         one — it can be laid out, publish a healthy rectangle and paint nothing, because every \
         other test of it asks whether a function ran and none asks what an operator sees"
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
        .ok_or_else(|| Error::new("no --pdf. The tool status needs a document to describe."))?;
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
    // operator chooses, and Edit is the one O123 rearranged; asking what Read
    // shows would be asking a different and easier question. Nothing after this
    // is clicked.
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(30);

    let trace = session.trace()?;
    let frame = session.frame()?;
    let client = frame.client_logical();

    let Some(banner) = driving::declared(&trace, ui_rect, BANNER_REGION) else {
        return Ok(Some(format!(
            "★★★ THE DOCK RESERVED NO STRIP. `{BANNER_REGION}` was never published, so either \
             the application stopped calling `Dock::with_side_banner`, or \
             `egui_shell::dock::banner::resolve_height` clamped the request to zero — which it \
             does deliberately when a side is too short to afford both the strip and a panel. \
             Check the window height before checking the wiring. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the dock reserved the strip at {banner:?}"));

    let Some(status) = driving::declared(&trace, ui_rect, STATUS_REGION) else {
        return Ok(Some(format!(
            "★★★ THE STRIP IS RESERVED AND EMPTY. `{BANNER_REGION}` published at {banner:?} and \
             `{STATUS_REGION}` did not, which is the exact pair this check exists to tell \
             apart: the dock gave the application a strip and the application drew nothing into \
             it. `crate::app::toolstatus::banner` returns early on three conditions — no \
             document, no command naming the armed tool, and a menu host that could not label \
             it — and each of those is a real defect wearing this symptom. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the status line drew at {status:?}"));

    // ★ Inside the strip it was given. A line drawn at the right coordinates in
    // the wrong compartment is a line painted over the first stack's tab bar,
    // and it looks like a rendering fault rather than a wiring one.
    if !banner.contains_rect(status) {
        return Ok(Some(format!(
            "★★ THE STATUS LINE IS OUTSIDE THE STRIP IT WAS GIVEN. The strip is {banner:?} and \
             the line is at {status:?}. `egui_shell::dock::banner::draw` sets the child `Ui`'s \
             clip to the strip, so a line escaping it means the rectangle being published is \
             not the one that was drawn."
        )));
    }
    if status.max.y > client.max.y || status.min.y < client.min.y {
        return Ok(Some(format!(
            "★★ THE STATUS LINE IS DRAWN AND NOT VISIBLE: {status:?} against a client area of \
             {client:?}. A strip below the fold publishes a rect, passes every containment test \
             written against the DOCK, and cannot be read — which is complaint #3 (a window \
             whose content pushed its own title bar off the desktop) arriving in dock chrome."
        )));
    }

    // ★★★ The pixel channel. Everything above is arithmetic the application did
    // about itself; this is the only assertion in the check that a build with a
    // correct layout and an empty painter cannot satisfy.
    let path = ctx.out("first_frame.png");
    let image = crate::capture::window_to_png(&session, &path)?;
    report.artifact(path.clone());
    let region = frame.logical_to_capture_pixels(status);
    let uniformity = crate::pixels::region_not_uniform(&image, region);
    if uniformity.is_uniform() {
        return Ok(Some(format!(
            "★★★ THE STATUS LINE'S RECTANGLE IS A FLAT BLOCK OF COLOUR ({}). The region is \
             published, it is inside the strip, it is inside the window, and nothing was \
             painted into it. That is the `absent` case a rect stream cannot report and is \
             exactly why this check samples pixels as well: a constant-height banner publishes \
             a rectangle whether or not its handler drew a glyph. The capture is at {}.",
            uniformity.summary(),
            path.display()
        )));
    }

    report.note(
        "★★ the armed tool is named in the right dock's strip, on screen, painted, with no click",
    );
    Ok(None)
}
