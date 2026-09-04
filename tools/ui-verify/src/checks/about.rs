//! `about` — **About opens, reports the build, and shows its attributions.**
//!
//! The first driven check of the About window, which until now had no declared
//! region and so could not be found by anything.
//!
//! # ★ Why About is worth driving at all
//!
//! It is the one surface in this program with a **legal** obligation behind it.
//! `dialogs::about`'s own header sets it out: the third-party attributions are
//! not decoration, they are the discharge of the notice requirements the
//! bundled licences impose. A build where that list silently stopped rendering
//! would look completely normal — nobody opens About — and would be a licence
//! breach shipping unnoticed. Unit tests assert the *strings*; only a driven
//! run asserts they reached a window.
//!
//! # What it asserts, and why the build block is a trace rather than pixels
//!
//! The operator asked on 2026-08-18 for About to carry *"the date and time of
//! the build … and the date and time of the builds of the used pdfcer and
//! iccce"*. Those values come from `build.rs` through `env!`, so the thing that
//! can go wrong is not the wording — that is unit-tested — but a value arriving
//! **empty**, which renders as `built  from abc1234` and reads as a layout
//! glitch rather than as a missing stamp.
//!
//! Reading four fields out of a PNG is not something this harness can do, so
//! the application traces them alongside drawing them and this check asserts on
//! the trace. The window is captured too, and the capture is attached, because
//! the layout question — does the block fit, is it legible — has exactly one
//! oracle and it is a rendered pixel.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | click **File ▸ About** | `dialog:about` declared |
//! | B | read the provenance trace | `about-build stamp=… rev=… engine=…`, none of them empty |
//! | C | capture the window | attached as evidence |

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The command that opens the window, and the region it must declare.
const ITEM: &str = "ribbon.item.file.about";
/// The body region `dialogs::about` publishes.
const BODY: &str = "dialog:about";
/// The provenance line the dialog traces as it draws the block.
const BUILD_EVENT: &str = "about-build";

/// See the module documentation.
pub struct AboutReportsTheBuild;

impl Check for AboutReportsTheBuild {
    fn name(&self) -> &'static str {
        "about_reports_the_build"
    }

    fn defect(&self) -> &'static str {
        "About does not open, or opens without saying when this program was built and which \
         engine revision is inside it — so an operator looking at two builds that behave \
         differently has no way to tell them apart"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a ribbon control. Reported as \
             SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    // ★ No `--pdf`. About is one of the few commands that must work with
    // nothing open — `dialogs::open_about` takes no `Status` precisely so that
    // cannot regress — and driving it on an empty shell is what proves it.
    let mut spec = LaunchSpec::new(&exe, ctx.out("about.trace.txt"));
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} with NO document, which is a state About must work in",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    // ★★★ MAXIMISE, or the File tab's last two groups are not on the band at
    // all. Measured 2026-09-03: at the harness's default 1,100 pt window the
    // File tab publishes fourteen items and stops at `file.print` — the whole
    // "Document" and "pdfcer" groups (properties, fonts, settings, shortcuts,
    // about) are folded away, so THREE checks skipped reporting a lost
    // command. The commands are not lost; the band is narrower than they are.
    //
    // `declared_or_in_overflow` already knows about collapsed groups and the
    // overflow popup, and is still the right first resort — but it cannot
    // conjure width, and `Session::maximize`'s own doc comment says this is
    // what it is for: "a maximised window is the state an operator running a
    // drawing tool on a desktop is overwhelmingly in, so it is also the state
    // most worth verifying."
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());

    // --- A: open it --------------------------------------------------------
    // ★ Through the overflow when the ribbon has folded it there. At the
    // harness's 1100 pt window the File tab's rightmost groups are correctly in
    // the overflow menu, and a check that looked only at the tab would report
    // "About is missing" about a ribbon behaving exactly as designed. See
    // [`crate::checks::driving::declared_or_in_overflow`].
    let item = crate::checks::driving::declared_or_in_overflow(&session, &driver, ui_rect, ITEM)?
        .ok_or_else(|| {
        Error::new(format!(
            "no `{ITEM}` region on the File tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.file."
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);

    let trace = session.trace()?;
    if declared(&trace, ui_rect, BODY).is_none() {
        return Ok(Some(format!(
            "clicking About declared no `{BODY}` region, so the window did not open — or opened \
             and drew nothing. Regions declared this run: {}.",
            list(&declared_names(&trace, ui_rect, "dialog:"))
        )));
    }
    report.note("About opened and declared its body");

    // --- B: the provenance, which is the operator's request ----------------
    let Some(line) = trace.events(BUILD_EVENT).last() else {
        return Ok(Some(format!(
            "About opened and traced no `{BUILD_EVENT}` line, so the build block was not drawn. \
             That block is what tells an operator which engine is inside the executable they \
             are looking at."
        )));
    };
    let empty: Vec<&str> = ["stamp", "rev", "engine", "engine_rev"]
        .into_iter()
        .filter(|k| line.get(k).is_none_or(str::is_empty))
        .collect();
    if !empty.is_empty() {
        return Ok(Some(format!(
            "About drew its build block with EMPTY field(s): {}. An empty stamp renders as \
             `built  from abc1234`, which reads as a layout glitch rather than as a missing \
             value — look at `crates/pdfcer-gui/build.rs`, which sets each of these through \
             `cargo:rustc-env`. A `Cargo.lock` it could not read is the usual cause.",
            empty.join(", ")
        )));
    }
    report.note(format!(
        "built {} from {}; engine {} at {}",
        line.get("stamp").unwrap_or_default(),
        line.get("rev").unwrap_or_default(),
        line.get("engine").unwrap_or_default(),
        line.get("engine_rev").unwrap_or_default()
    ));
    // ★ Reported, never asserted. `iccce` is legitimately absent today, and a
    // check that failed on that would have to be edited on the day it lands —
    // which is the wrong direction for a gate to point.
    let icc = line.get("iccce").unwrap_or_default();
    report.note(if icc.is_empty() {
        "iccce is NOT in this build, and About says so".to_owned()
    } else {
        format!("iccce {icc} is in this build")
    });

    // --- C: the picture ----------------------------------------------------
    let shot = ctx.out("about.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
        }
        Err(e) => {
            report.note(format!(
                "the window could not be captured ({e}); the trace assertions above still hold"
            ));
        }
    }
    Ok(None)
}
