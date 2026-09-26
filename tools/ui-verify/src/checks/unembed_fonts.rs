//! `removing_embedded_fonts_reaches_the_document` — **press Remove and the
//! font programs come out.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/unembed_fonts.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command, invoked through the harness seam.
const INVOKE: &str = "mode.edit,tools.unembed_fonts";
/// The window body's region.
const BODY: &str = "unembed.body";
/// The Remove button's region.
const BUTTON: &str = "unembed.commit";
/// The line the dialog writes when the button is pressed.
const REQUESTED: &str = "unembed-fonts-requested";
/// The line the apply arm writes when the engine has removed.
const APPLIED: &str = "unembed-fonts-applied";
/// The line the window writes when it opens, carrying its plan's counts.
const OPENED: &str = "unembed-fonts-opened";
/// The line the dispatcher writes when there is nothing to open.
const DECLINED: &str = "unembed-fonts-declined";

/// See the module documentation.
pub struct RemovingEmbeddedFontsReachesTheDocument;

impl Check for RemovingEmbeddedFontsReachesTheDocument {
    fn name(&self) -> &'static str {
        "removing_embedded_fonts_reaches_the_document"
    }

    fn defect(&self) -> &'static str {
        "Remove embedded fonts is on the Tools tab and removes nothing — the command was \
         registered, drawn and inert for the whole life of the project behind a blocker that was \
         real, and an operator who presses it gets a window listing what would happen and a \
         button that changes nothing"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check's subject is a click on the Remove \
             button. The window opening proves nothing on its own — a document with nothing \
             removable draws the same window with its button greyed, which is correct.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check needs a document carrying an embedded font that pdfcer judges \
             removable; a document with none declines by name and the decline is not the \
             behaviour under test.",
        )
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("unembed-fonts.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(60);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;

    // A decline is a statement about the `--pdf`, not about the program.
    if let Some(declined) = trace.events(DECLINED).last() {
        return Err(Error::new(format!(
            "the command declined: `{}`. SKIPPED rather than failed: it says the --pdf carries no \
             embedded font pdfcer judges removable, which is the harness's aim rather than the \
             program's behaviour. Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }

    // The fixture check, BEFORE the click, and it is a SKIP.
    //
    // A document whose embedded fonts are all identity-encoded has nothing
    // removable, draws this window to say so, and greys the button. That is
    // correct behaviour and it is indistinguishable from a broken button once
    // the click has landed on nothing - which is exactly what the first run of
    // this check reported, against `a1-titleblock.pdf`, whose three embedded
    // faces are all `unknown-symbolic-builtin`.
    //
    // => Blaming the feature for the fixture is a named failure in this
    // harness, and the fix is never a looser assertion. It is a line in the
    // trace that says which of the two states the program is in.
    if let Some(opened) = trace.events(OPENED).last()
        && opened.get("targets") == Some("0")
    {
        return Err(Error::new(format!(
            "the window opened with nothing to remove: `{}`. SKIPPED rather than failed: every \
             embedded font in this --pdf is one pdfcer judges unsafe to unembed - \
             identity-encoded or Type 3 - so the greyed button is correct. `pdfcer list-fonts \
             <file>` names the verdict per font; this check needs one reading `removable`. \
             Trace: {}.",
            opened.raw,
            session.trace_path().display()
        )));
    }

    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "★ REMOVE EMBEDDED FONTS WAS INVOKED AND NO WINDOW APPEARED: no `{BODY}` region and \
             no `{DECLINED}` line.\n\
             The command has no dispatch arm, or the window was built and never drawn, or the \
             plan construction returned early without tracing. Regions beginning `unembed`: {}. \
             Trace: {}.",
            list(&declared_names(&trace, ui_rect, "unembed")),
            session.trace_path().display()
        )));
    }

    // `stable_rect`, because the window lays out over several frames as the
    // scroll area measures its rows, and a coordinate read before it stops
    // moving is a number rather than an error.
    let Some(button) = stable_rect(&session, ui_rect, BUTTON, 8)? else {
        return Ok(Some(format!(
            "the Remove window drew its body and declared no `{BUTTON}` region, so the one \
             control that changes anything was never laid out. Regions beginning `unembed`: {}. \
             Trace: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "unembed")),
            session.trace_path().display()
        )));
    };

    // `frame_of`, never `session.frame()`. This window is a child viewport
    // and its coordinates are its own; asking the main window yields a point on
    // the ribbon, and the click lands somewhere plausible and wrong.
    let trace = session.trace()?;
    let frame = frame_of(&session, &trace, ui_rect, BUTTON)?;
    driver.click_at(frame.declared_center(button))?;
    session.settle(50);

    let trace = session.trace()?;
    let Some(requested) = trace.events(REQUESTED).last() else {
        return Ok(Some(format!(
            "the Remove button was clicked and the window raised nothing: no `{REQUESTED}` \
             line.\n\
             Either the click missed — the button is at {button:?} in the window's own frame — \
             or it landed on a **greyed** button, which is what the window draws when its plan \
             has no targets. A greyed button on a document whose fonts pdfcer reports as \
             removable means the plan and the report disagree. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the window raised the action: `{}`",
        requested.raw
    ));

    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "★ THE REMOVAL WAS REQUESTED AND NOTHING REACHED THE DOCUMENT: `{}` and no \
             `{APPLIED}` line.\n\
             The action was raised and its apply arm did not run, or the engine refused. A \
             refusal traces `unembed-fonts-refused …` through the edit funnel and carries the \
             engine's own reason. Trace: {}.",
            requested.raw,
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the engine removed: `{}`", applied.raw));

    // --- the oracle ---------------------------------------------------------
    let removed: usize = applied
        .get("removed")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let bytes: u64 = applied
        .get("bytes")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if removed == 0 {
        return Ok(Some(format!(
            "★ THE REMOVAL RAN AND REMOVED NOTHING: `{}`.\n\
             The chain works end to end and the plan was empty. `dialogs::unembed` greys its \
             button on exactly that condition, so an empty plan reaching the engine means the \
             plan the button was gated on is not the plan that was sent. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if bytes == 0 {
        return Ok(Some(format!(
            "★ {removed} FONT(S) WERE UNEMBEDDED AND NO BYTES WERE FREED: `{}`.\n\
             `bytes` is the engine's `bytes_reclaimable`, summed from each target's measured \
             span. Zero with a non-zero count means every program was SHARED with a font that \
             stayed embedded — legal, and disclosed on each row — or the spans measured zero, \
             which would be a measurement fault rather than a removal one. Both are worth \
             reading before this is called a pass. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ {removed} font(s) unembedded, {bytes} byte(s) of program freed, renamed={}",
        applied.get("renamed").unwrap_or("?")
    ));

    // Reported, never asserted. Whether subset tags were stripped depends on
    // whether the fixture's fonts carried any, which is a fact about the
    // producer that made it and not about this program.
    Ok(None)
}
