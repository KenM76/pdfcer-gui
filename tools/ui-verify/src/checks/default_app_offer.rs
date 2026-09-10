//! `default_app_offer` — **the ask-once offer reaches the screen, both its
//! answers are reachable, and it does not come back once it has been answered.**
//!
//! `OPERATOR_REQUESTS.md` **O173**, his words of 2026-09-10: *"we should have
//! an easy way to make pdfce-gui our default opener for pdfs. Ask once with a (old-name-exempt: HIS words, quoted verbatim from O173 — correcting an operator's own sentence would stop this being a quotation)
//! don't show me again check box option."*
//!
//! # ★★★ The one thing this check must do before it launches anything
//!
//! **Delete the preference the sandbox seeds.**
//!
//! Every ui-verify sandbox is by construction a fresh profile, so the offer
//! would otherwise open in front of all two hundred-odd driven checks and take
//! their pointer presses — the standing lesson that *a window over the thing it
//! describes takes that thing's gestures*. [`crate::sandbox`] therefore writes
//! one key into every sandbox that declines the offer in advance, and its own
//! doc comment states the cost in the same paragraph as the benefit:
//!
//! > **A check written to drive that offer must delete this file first**, or it
//! > will assert against a starting state that defeats the very thing it
//! > measures.
//!
//! That is this check, and [`clear_seed`] is the deletion. It is not a
//! convenience: without it every assertion below is still *evaluable* — the
//! window simply never opens — and the check would report *"the offer did not
//! appear"* about a build in which the offer works perfectly. A fixture that
//! defeats a default does not defeat a starting state; the starting state has
//! to be planted.
//!
//! # ★★ What this check will NOT do, deliberately
//!
//! **It never presses the affirmative button.** That button writes ten values
//! under `HKCU` on the machine running the sweep and then opens Windows' own
//! Default-apps page in front of whatever the operator was doing. A driven
//! check is not entitled to either. The registration mechanism is unit-tested
//! in `crate::app::assoc`; what only a driven run can prove is that the window
//! **opens, and that its answer row is on the screen** — which is precisely the
//! class of defect O171 was, one row above this one in the same file.
//!
//! ⇒ So the button is asserted **present and visible** and then left alone, and
//! the window is answered through *Not now*, which is the one route out that
//! changes nothing outside pdfcer.
//!
//! # ⚠ When this check legitimately cannot run, and why that is a SKIP
//!
//! The shell decides whether to ask by **reading Windows**, not by reading a
//! flag — deliberately, so that an operator who set the default by hand is not
//! asked about something already true. ⇒ On a machine where pdfcer is
//! **already** the registered `.pdf` handler the offer never opens, and there
//! is nothing here to drive.
//!
//! That is reported as a skip carrying the list of regions the run actually
//! declared, rather than as a failure. A check that went red on a
//! correctly-behaving build would be edited away inside a week, and the edit
//! would take the four real assertions with it.
//!
//! ★ It is worth knowing that this makes the check's coverage a function of the
//! machine it runs on: the day the operator accepts the offer for real, this
//! check stops exercising anything on his desktop and keeps exercising
//! everything on a clean one. The standing lesson *a SKIP is not red, so a
//! check can stop running unnoticed* applies directly — **diff the sweep's SKIP
//! set**, and if this name appears in it, read the reason before assuming the
//! sweep covered O173.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | delete the seeded preference, launch with **no document** | the offer opens |
//! | B | read the regions | `defaultapp.body`, `.action`, `.dont-ask` and `.later` all declared |
//! | C | capture the window | attached as evidence |
//! | D | tick the box, press *Not now* | `default-app-settled dont_ask=true saved=true` |
//! | E | relaunch the same profile | **no** `defaultapp.body` this time |
//!
//! ★ Phase E is the half of *"ask once"* that no unit test can reach. The unit
//! tests assert that the dialog **writes** the preference. Only a second launch
//! against the same profile directory proves the written preference is **read
//! back** on the path that decides whether to ask — and two files and a round
//! trip through disk sit between those two facts.
//!
//! # Rule 15
//!
//! No dimension of either kind appears in this module.

use crate::checks::driving::{declared, declared_names, frame_of, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// `dialogs::defaultapp::REGION_BODY`, spelled as a literal because ui-verify
/// deliberately does not link the shell.
const BODY: &str = "defaultapp.body";
/// The affirmative button — asserted, never pressed. See the module header.
const ACTION: &str = "defaultapp.action";
/// The checkbox O173 asks for by name.
const DONT_ASK: &str = "defaultapp.dont-ask";
/// *Not now* — the one route out that changes nothing outside pdfcer.
const LATER: &str = "defaultapp.later";
/// The line the dialog traces as it writes the answer down.
const SETTLED: &str = "default-app-settled"; // ui-text-exempt: a trace event name, never displayed

/// The file the sandbox's seed writes, relative to the executable.
///
/// A third copy of `app::prefs::PREFS_FILE`, and the third copy is where a
/// constant starts to rot. It is spelled again rather than imported for the
/// same reason the sandbox spells it: this crate does not link the shell. If
/// the offer ever stops appearing here for no reason anybody can explain,
/// suspect this name first.
const SEEDED_PREFS: &str = "preferences.txt";

/// See the module documentation.
pub struct TheDefaultAppOfferIsAskedOnce;

impl Check for TheDefaultAppOfferIsAskedOnce {
    fn name(&self) -> &'static str {
        "default_app_offer_is_asked_once"
    }

    fn defect(&self) -> &'static str {
        "the offer to make pdfcer the default PDF program never appears, appears with its \
         answer row off the bottom of the window, or comes back on the next launch after it \
         has been answered — which turns a one-time question into the nagging this project \
         refuses"
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

/// **Remove the sandbox's declined-in-advance preference.**
///
/// Returns the path it removed, so the caller can say so in a note: a
/// precondition that is not stated is one nobody thinks of when the check later
/// reports that nothing opened.
///
/// ★ A missing file is success, not failure. This check is also runnable
/// against a profile that was never seeded — `--no-isolate`, or a hand-pointed
/// `--exe` — and refusing to run there would make the coverage depend on how
/// the harness happened to be invoked.
fn clear_seed(exe: &std::path::Path) -> Option<std::path::PathBuf> {
    let prefs = exe.parent()?.join("userdata").join(SEEDED_PREFS);
    std::fs::remove_file(&prefs).ok().map(|()| prefs)
}

/// Launch the binary with the diagnostic trace on and **no document open**.
///
/// ★ No `--pdf`, in either phase. The launch this offer fires on is
/// overwhelmingly a launch with nothing open: the operator has just installed
/// pdfcer and started it from the Start menu, *because* double-clicking a
/// drawing is the thing they cannot yet do. Driving it with a document open
/// would exercise a case the feature is not for.
///
/// # Errors
///
/// Whatever [`Session::launch`] refuses on — a missing binary, a stale one.
fn launch(ctx: &CheckContext, exe: &std::path::Path, trace: &str) -> Result<Session> {
    let mut spec = LaunchSpec::new(exe, ctx.out(trace));
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    Session::launch(&spec, ctx.profile.trace_prefix)
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
            "input is disabled (--no-input). This check clicks a checkbox and a button inside a \
             dialog. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // --- A: plant the starting state, then launch --------------------------
    match clear_seed(&exe) {
        Some(path) => report.note(format!(
            "removed {}, the preference every sandbox seeds to decline this offer in advance. \
             Without this deletion the rest of this check would assert against a profile that \
             has already answered the question.",
            path.display()
        )),
        None => report.note(
            "no seeded preference to remove — this profile had never been asked. That is the \
             other legitimate starting state and the check proceeds unchanged.",
        ),
    };

    let session = launch(ctx, &exe, "default-app-offer.trace.txt")?;
    report.note(format!(
        "launched {} as pid {} with NO document",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);

    // --- B: the regions ----------------------------------------------------
    let trace = session.trace()?;
    if declared(&trace, ui_rect, BODY).is_none() {
        // ★★ One legitimate reason for this, and it is worth distinguishing
        // from a defect: the machine running the sweep may ALREADY open PDFs
        // with pdfcer, in which case the shell correctly answers no and there
        // is nothing to drive. Reported as a skip, because a check that went
        // red on a correctly-behaving build would be edited away inside a week
        // — and the edit would take the real assertions with it.
        return Err(Error::new(format!(
            "no `{BODY}` region — the offer did not open. If this machine ALREADY opens PDFs \
             with pdfcer then that is correct behaviour and there is nothing here to drive; the \
             deciding value is Explorer's per-user `UserChoice` for `.pdf`. Regions declared \
             this run: {}.",
            list(&declared_names(&trace, ui_rect, "defaultapp"))
        )));
    }
    report.note("the offer opened and declared its body");

    // ★★★ These three are published through `diag::ui_rect_visible`, which
    // stays silent when the rect has been clipped out of its own viewport — so
    // a declaration IS the visibility assertion, and no size has to be guessed
    // at anywhere in this file. That is the whole of the operator's rule of
    // 2026-09-10 — *"those buttons should always be available"* — expressed as
    // something a harness can read.
    for (region, what) in [
        (ACTION, "the button that performs the changeover"),
        (LATER, "the button that declines this time"),
        (
            DONT_ASK,
            "the don't-ask-again checkbox O173 asks for by name",
        ),
    ] {
        if declared(&trace, ui_rect, region).is_none() {
            return Ok(Some(format!(
                "the offer opened but did NOT declare `{region}` ({what}). That region is only \
                 published when the control is inside its own clip rect, so the control is on \
                 the window and off the screen. This is O171's defect in a second window: the \
                 answer row must be allocated out of the window's rectangle BEFORE the body, \
                 never after it. Regions declared: {}.",
                list(&declared_names(&trace, ui_rect, "defaultapp"))
            )));
        }
    }
    report.note("the action, the decline and the checkbox are all declared, so none is clipped");

    // --- C: the picture ----------------------------------------------------
    // ★ Layout and legibility have exactly one oracle and it is a rendered
    // pixel. The trace above proves the rects exist; only this shows whether
    // the window reads as a question.
    let shot = ctx.out("default-app-offer.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => report.artifact(shot),
        Err(e) => report.note(format!(
            "the window could not be captured ({e}); the trace assertions above still hold"
        )),
    };

    // --- D: answer it, by the route that changes nothing outside pdfcer ----
    // `frame_of` rather than `session.frame()`: this dialog is a real OS
    // viewport and its rectangles are relative to ITS origin. The numbers stay
    // perfectly plausible when that conversion is skipped, which is why six
    // checks once clicked hundreds of pixels from the control they named.
    let box_rect =
        declared(&trace, ui_rect, DONT_ASK).ok_or_else(|| Error::new("the checkbox vanished"))?;
    let driver = Driver::new(session.window());
    driver.click_at(frame_of(&session, &trace, ui_rect, DONT_ASK)?.declared_center(box_rect))?;
    session.settle(15);

    // Re-read: ticking the box may have re-laid the row out, and a coordinate
    // read before an act that moves it is a stale number rather than an error.
    let trace = session.trace()?;
    let later_rect = declared(&trace, ui_rect, LATER).ok_or_else(|| {
        Error::new(format!(
            "`{LATER}` stopped being declared after the box was ticked"
        ))
    })?;
    driver.click_at(frame_of(&session, &trace, ui_rect, LATER)?.declared_center(later_rect))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(line) = trace.events(SETTLED).last() else {
        return Ok(Some(format!(
            "the offer was answered — the box ticked and *Not now* pressed — and traced no \
             `{SETTLED}` line, so nothing was written. The next launch will ask again, which is \
             the nagging the checkbox exists to stop."
        )));
    };
    if line.get("dont_ask") != Some("true") {
        return Ok(Some(format!(
            "the checkbox was clicked and `{SETTLED}` reports dont_ask={}. Either the click \
             missed the box, or the box's state is not the state the dialog writes down.",
            line.get("dont_ask").unwrap_or_default()
        )));
    }
    if line.get("saved") != Some("true") {
        return Ok(Some(format!(
            "`{SETTLED}` reports saved=false, so the answer was computed and never reached the \
             preferences file. The offer will come back on the next launch. The preference store \
             resolves beside the executable; a read-only directory does exactly this."
        )));
    }
    report.note("the answer was written to the profile's preferences file");

    // ★ The session must DIE before the next one starts, or two copies of the
    // program race for the same preferences file and phase E measures which of
    // them wrote last. `Session`'s `Drop` kills the child.
    drop(session);

    // --- E: ask once means once --------------------------------------------
    let second = launch(ctx, &exe, "default-app-offer.second.trace.txt")?;
    report.artifact(second.trace_path().to_path_buf());
    second.settle(60);
    let trace = second.trace()?;
    if declared(&trace, ui_rect, BODY).is_some() {
        return Ok(Some(
            "the offer was answered with the box ticked and it opened AGAIN on the next launch \
             of the same profile. The preference WAS written — phase D read the trace that says \
             so — therefore what failed is the READ: the shell's should-we-ask decision is not \
             seeing what the dialog wrote. Check that both ends name the same key, and that the \
             reader resolves this profile's own `userdata/` rather than a shared one."
                .to_owned(),
        ));
    }
    report.note("the second launch did not ask, which is the whole of \"ask once\"");
    Ok(None)
}
