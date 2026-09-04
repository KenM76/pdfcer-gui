//! `dialogs_open_in_their_own_window` — **every dialog is an OS window**, not
//! only the one the operator complained about.
//!
//! # What this is for
//!
//! The operator, 2026-08-20:
//!
//! > *"Print dialogue box doesn't pop up in its own movable window. It is
//! > locked within the boundaries of the program's window. Like, I just assume
//! > you've been trained on a million lines of code and software that pops it
//! > up in its own window."*
//!
//! Print was fixed that evening and `print_dialog` asserts it REACHES THE
//! SPOOLER — which is not a claim about its window, and the difference let four
//! window defects ship in it. It is the first entry in `DIALOGS` since
//! 2026-09-03; see the comment there. **The other thirteen dialogs were not**, and the report was never about printing — it
//! was about a shell whose dialogs are drawn inside its own canvas. This check
//! is the general form of `print_dialog`'s section D.
//!
//! # ★★ Why one process per dialog
//!
//! `PDFCER_DIAG_INVOKE` fires **once** per process, by design: an environment
//! variable is not an event, and the latch that turns it into one is consumed
//! the first time it fires. So a check that wants eight dialogs launches eight
//! processes. That is slower and it is the honest shape — a dialog opened
//! *after* another dialog is a different state from a dialog opened first, and
//! this check is about the plain case.
//!
//! # ★★★ Why it needs no pointer, and why that matters
//!
//! Every dialog here is reachable by a command id, so the whole check runs
//! through the diagnostic invoke seam: no clicks, no keystrokes, nothing that
//! takes the operator's cursor. It is therefore one of the few checks that is
//! **safe to run on a machine somebody is using**, and it does not skip under
//! `--no-input`.
//!
//! ★ The price is stated rather than hidden: five dialogs in this directory are
//! reachable only by a gesture — Insert image (needs a chosen file), Insert
//! pages, Set scale, the text-annotation editor, and the unsaved-changes
//! question. They are **not covered here**, and a regression in any of them
//! would not fail this check. `OPERATOR_REQUESTS.md` records them as
//! NOT VERIFIED rather than letting a green run imply otherwise.
//!
//! # The oracle
//!
//! `viewport-inner`, and there is no other. A screenshot cannot answer this: a
//! dialog in its own window is **absent** from a capture of the application
//! window, and an in-viewport panel that regressed would look like a perfectly
//! good dialog in that same capture. *"Is this a separate OS window"* is a fact
//! about the window manager, and the only thing in the process that knows it is
//! the viewport egui created.

use crate::checks::driving::{SHELL_DIAG_ENV, VIEWPORT_INNER_EVENT};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The environment variable that fires one command at start-up.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";

/// Every dialog reachable by a command id, as `(command, what the operator
/// calls it)`.
///
/// ★ The second element is for the failure message and is deliberately the
/// operator's word rather than the module name: a report that says *"Export to
/// DXF opened inside the application window"* is actionable to whoever reads
/// it, and `export_dxf.rs` is not.
const DIALOGS: &[(&str, &str)] = &[
    // ★★★ PRINT IS FIRST, AND ITS ABSENCE FROM THIS LIST WAS A DEFECT — added
    // 2026-09-03.
    //
    // This module's header used to say Print "was fixed that evening and
    // `print_dialog` asserts it", and left it out. Both halves of that were
    // wrong in a way worth keeping:
    //
    //  · `print_dialog` asserts the dialog reaches the SPOOLER. It says nothing
    //    about the window, its margins, its scrollbars or its buttons.
    //  · So the one dialog whose report started this entire piece of work —
    //    *"Print dialogue box doesn't pop up in its own movable window"* — was
    //    the only command-reachable dialog with no headless check at all.
    //
    // On 2026-09-03 the operator reported four fresh defects in it: two
    // scrollbars that could not be dismissed, a window that would not close
    // after printing, a commit button flush against the window corner, and that
    // button rendered so pale it read as disabled. **A hand-written list inside
    // a completeness sweep is the gap**, and this is the third time this
    // project has been caught by that exact shape.
    ("file.print", "Print"),
    ("file.about", "About"),
    ("tools.render_diagnostics", "Render diagnostics"),
    ("file.export_dxf", "Export to DXF"),
    ("file.ocr", "Recognise text"),
    ("file.shortcuts", "Keyboard shortcuts"),
    ("file.settings", "Settings"),
    ("file.new_from_template", "New document"),
    ("edit.redact_apply", "Apply redactions"),
];

/// See the module documentation.
pub struct DialogsOpenInTheirOwnWindow;

impl Check for DialogsOpenInTheirOwnWindow {
    fn name(&self) -> &'static str {
        "dialogs_open_in_their_own_window"
    }

    fn defect(&self) -> &'static str {
        "a dialog is drawn inside the application's own window: it cannot be moved off the \
         document it is asking about, cannot be put on a second monitor, and does not appear in \
         the taskbar. Reported by the operator about Print; it was true of all fourteen"
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
        Error::new(
            "no --pdf. Several of these dialogs refuse to open without a document, and a dialog \
             that refused would look exactly like one that opened in the wrong kind of window — \
             an absence proving nothing, which rule 4 forbids treating as evidence.",
        )
    })?;

    let mut failures: Vec<String> = Vec::new();
    for (command, name) in DIALOGS {
        let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("dialog-{command}.trace.txt")));
        spec.pdf = Some(pdf.clone());
        spec.env.push((
            ctx.profile.diag_env.0.to_owned(),
            ctx.profile.diag_env.1.to_owned(),
        ));
        spec.env
            .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
        spec.env
            .push(((*INVOKE_ENV).to_owned(), (*command).to_owned()));
        spec.allow_stale = ctx.allow_stale;
        spec.source_root = ctx.source_root.clone();

        let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
        report.artifact(session.trace_path().to_path_buf());
        // Long enough for the dialog's own first frames. Several of these do
        // real work on open — Recognise text queries the engine, Apply
        // redactions runs the whole scan — and a settle tuned to About would
        // report "no window" for a dialog that was a second from having one.
        session.settle(40);
        let trace = session.trace()?;
        match trace.events(VIEWPORT_INNER_EVENT).last() {
            Some(l) => {
                let size = l
                    .get("rect")
                    .map_or_else(|| l.raw.clone(), std::borrow::ToOwned::to_owned);
                report.note(format!("★ {name} is a real OS window: {size}"));
            }
            None => failures.push((*name).to_owned()),
        }
        // ★ Also reported: a fit that ran. `Host::fit` grows a window whose
        // body overflowed it, and a dialog that needs one on every open is a
        // dialog whose declared size is wrong — worth knowing even though it
        // is not a failure, because the operator sees the window jump.
        if let Some(fit) = trace.events("dialog-fit").last() {
            report.note(format!("· {name} had to grow to fit its body: {}", fit.raw));
        }
    }

    if failures.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "★ {} of {} dialogs did NOT open in their own OS window: {}.\n\
         No `{VIEWPORT_INNER_EVENT}` line followed the command that opens them. That is the \
         operator's report of 2026-08-20 generalised — a dialog locked inside the application \
         window cannot be moved off the document it is asking about, cannot go to a second \
         monitor and does not appear in the taskbar. Each must go through `dialogs::host::Host` \
         rather than `egui::Window`; the traces are beside this report.",
        failures.len(),
        DIALOGS.len(),
        failures.join(", ")
    )))
}
