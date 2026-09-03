//! `the_title_bar_carries_the_build_time` — the window title ends with the date
//! **and the time** this binary was compiled.
//!
//! # The operator's ask, `OPERATOR_REQUESTS.md` O101
//!
//! > *"also in the next release add the local compilation time to the top bar at
//! > the end of the date you added."*
//!
//! ★★ He is closing a loop his own reports opened. The date went into the title
//! on 2026-09-01 after he spent a morning reporting a defect that had already
//! been fixed, against a build he did not know was old. **Two backlog rows have
//! been closed by "you were running an old build"** — O85 and O87 — and on a day
//! with several publishes the date alone cannot tell two builds apart. A date
//! answers *is this today's*; a date and a time answer *is this the one I just
//! installed*, which is the question that was actually being got wrong.
//!
//! ⇒ So the failure this guards against is not cosmetic. A title that silently
//! lost its time would put the project back to spending mornings on defects that
//! do not exist in the build on disk.
//!
//! # ★★★ The one check in this suite that needs no input at all
//!
//! The title is published as `window-title "..."` whenever it changes, and the
//! window is placed with `PDFCER_DIAG_VIEWPORT`, which lays out a real window
//! **without taking focus**. So this reads a trace line from a launched process
//! and asserts on it — no pointer, no keyboard, nothing that competes with
//! whoever is using the machine.
//!
//! That is worth naming rather than just doing: most of this suite is gated on
//! `--allow-input` and therefore on the operator being away from the desk. A
//! check that can run at any time is a check that can run *often*.
//!
//! # ★★ What is asserted, and why the zone rule is the interesting part
//!
//! `PDFCER_BUILD_TIME` has two producers and they disagree about zone:
//!
//! | producer | stamp | zone |
//! |---|---|---|
//! | `package-portable.py` | `2026-09-02 06:25 +0100` | **local** |
//! | `build.rs` fallback | `2026-09-02 06:25 UTC` | UTC, and labelled |
//!
//! A packaged build's time is already local, so its offset is noise to somebody
//! standing in that zone and is dropped. A dev build's is UTC, and showing
//! `06:25` bare would invite reading an hour that is not the wall clock — so
//! `UTC` is kept. `build.rs`'s own sentence is the rule: *a stamp that says the
//! wrong hour is worse than one that says a true hour in a named zone.*
//!
//! ⇒ Hence the third assertion: **a raw offset must never survive into the
//! title.** That is what the obvious simpler implementation — truncate to
//! sixteen characters and stop — would get wrong in one direction, and what
//! "just print the whole stamp" would get wrong in the other.
//!
//! # What a passing run does NOT prove
//!
//! That the time is *correct* — that the stamp matches when the binary was
//! actually compiled. Nothing observable from outside can establish that, and
//! `build.rs` owns it. This asserts the **shape** reaches the operator, which is
//! the part that has silently regressed before.

use crate::checks::driving::SHELL_DIAG_ENV;
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The trace event the frame emits when the window title changes.
const TITLE: &str = "window-title";
/// Where and how large the window is placed, as `PDFCER_DIAG_VIEWPORT` takes it.
///
/// Modest, because nothing here depends on width — and it switches `with_active`
/// off, which is the property that lets this run beside somebody working.
const VIEWPORT: &str = "0,0,1400,900";

/// See the module documentation.
pub struct TheTitleBarCarriesTheBuildTime;

impl Check for TheTitleBarCarriesTheBuildTime {
    fn name(&self) -> &'static str {
        "the_title_bar_carries_the_build_time"
    }

    fn defect(&self) -> &'static str {
        "the window title does not say when this binary was built, so on a day with several \
         publishes there is no way to tell which build is running — which has already cost two \
         mornings spent reporting defects that were fixed in a newer build on disk"
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("title_build_stamp.trace.txt"));
    // ★★★ NO DOCUMENT, and deliberately not `ctx.pdf`.
    //
    // The stamp is a property of the BINARY, not of what is open, so a document
    // adds nothing to the assertion — and taking one adds a way to fail that
    // has nothing to do with the subject. Measured: passing `--pdf` at a path
    // that does not exist made this SKIP, and a SKIP is not red, so the check
    // would have quietly stopped being evidence the first time a fixture moved.
    //
    // Which is exactly what had happened. The whole project's documentation
    // named the benchmark drawing at `D:\\Dev\\temp\\pdfcer\\`, and it now lives
    // in `D:\\Dev\\pdfTests\\`. Three documents were corrected alongside this
    // line. A check whose subject does not need a document should not acquire
    // a dependency on one.
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    if let Some(name) = ctx.profile.viewport_env {
        spec.env.push((name.to_owned(), VIEWPORT.to_owned()));
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {} — no input is sent",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    let Some(line) = trace.last(TITLE) else {
        return Ok(Some(format!(
            "the application ran and never traced a `{TITLE}` line, so it never set a window \
             title. The title is the only surface that reaches an operator who is not looking \
             at the application — Alt-Tab, the taskbar and the accessibility window list all \
             read it, and none of them can see the tab strip."
        )));
    };
    let title = line.raw.trim();
    report.note(format!("title: {title}"));

    // The stamp is at the END of the title, after the last em dash. Parsed from
    // the right rather than by a whole-title pattern, because the left-hand part
    // is a file name and may contain anything a path can.
    let tail = title
        .rsplit('—')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"');
    if tail.is_empty() {
        return Ok(Some(format!(
            "the title carries no trailing stamp at all: `{title}`. Either the build stamp was \
             dropped from `text::doctabs::window_title`, or the separator changed and this \
             check is reading the wrong end of the string — check the separator before \
             concluding the feature is gone."
        )));
    }

    // --- 1: a date -------------------------------------------------------
    let date_ok = tail.len() >= 10
        && tail.as_bytes()[..10].iter().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                *b == b'-'
            } else {
                b.is_ascii_digit()
            }
        });
    if !date_ok {
        return Ok(Some(format!(
            "the title's trailing stamp is `{tail}`, which does not begin with a `YYYY-MM-DD` \
             date. The date shipped on 2026-09-01 and the time was added to it; losing the \
             date would undo both."
        )));
    }

    // --- 2: ★ AND A TIME, which is the whole of O101 ---------------------
    let rest = tail[10..].trim();
    let time_ok = rest.len() >= 5
        && rest.as_bytes()[..5].iter().enumerate().all(|(i, b)| {
            if i == 2 {
                *b == b':'
            } else {
                b.is_ascii_digit()
            }
        });
    if !time_ok {
        return Ok(Some(format!(
            "the title says `{tail}` — a date and NO TIME. This is the pre-O101 state. On a day \
             with several publishes the date alone cannot tell two builds apart, which is \
             exactly how two backlog rows came to be closed by 'you were running an old build'."
        )));
    }
    report.note(format!("the stamp carries a date and a time: `{tail}`"));

    // --- 3: ★★ and never a raw offset ------------------------------------
    let zone = rest[5..].trim();
    if zone.starts_with('+') || zone.starts_with('-') {
        return Ok(Some(format!(
            "the stamp is `{tail}` — the packager's raw UTC offset survived into the title. A \
             packaged build's time is ALREADY local, so its offset is noise to somebody \
             standing in that zone and is meant to be dropped. Seeing `+0100` beside a local \
             time invites reading it as a conversion that has not happened."
        )));
    }
    if !zone.is_empty() && zone != "UTC" {
        return Ok(Some(format!(
            "the stamp ends with `{zone}`, which is neither empty (a packaged build, local \
             time) nor `UTC` (a dev build from `build.rs`). A third form means a producer this \
             check has not been told about, and the zone rule — a stamp that says the wrong \
             hour is worse than one that says a true hour in a named zone — cannot be \
             evaluated for it."
        )));
    }
    report.note(if zone.is_empty() {
        "no zone shown, so this is a packaged build stamped in local time".to_owned()
    } else {
        "labelled UTC, so this is a dev build from build.rs's fallback".to_owned()
    });

    Ok(None)
}
