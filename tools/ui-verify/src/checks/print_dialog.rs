//! `print_dialog_reaches_the_spooler` — the regression test for the defect
//! the operator reported as *"the print dialogue didn't work"*.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/print_dialog.md`.

use crate::checks::driving::{
    INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, UNIMPLEMENTED_EVENT,
    VIEWPORT_INNER_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The command this check is about.
const SUBJECT_ID: &str = "file.print";

/// The region the ribbon publishes for its control.
const SUBJECT: &str = "ribbon.item.file.print";

/// The tab it lives on, and the region that activates it.
const TAB_ID: &str = "file";
const TAB: &str = "ribbon.tab.file";

/// The trace event `PrintDialog::open` emits, once, when the dialog is built.
const OPEN_EVENT: &str = "print-open";

pub struct PrintDialogReachesTheSpooler;

impl Check for PrintDialogReachesTheSpooler {
    fn name(&self) -> &'static str {
        "print_dialog_reaches_the_spooler"
    }

    fn defect(&self) -> &'static str {
        "File ▸ Print opens a dialog that says this build cannot print, on a machine with printers"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // A document is a precondition, not a convenience: `file.print` is
    // registered `enabled_when("doc.open")`, so with nothing open the control
    // is greyed and a click on it would be measuring the enable predicate
    // rather than the adapter behind it.
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. `file.print` is gated on `doc.open`, so with nothing open the control is \
             greyed and a click on it would be measuring the enable predicate rather than the \
             spooler adapter.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is two clicks on ribbon controls. \
             Reported as SKIPPED rather than passed — a check that did not run has learned \
             nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("print_dialog.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the \
             process and nothing below could be observed. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }

    // Nothing may have opened this dialog yet.
    //
    // A print dialog that appears without being asked for is the specified
    // default of `view.app_initiative` — **Never** — broken in the most
    // expensive way available, since the surface it floats over the canvas is
    // the one with an irreversible button on it.
    if trace.events(OPEN_EVENT).next().is_some() {
        return Ok(Some(format!(
            "`{OPEN_EVENT}` appears in the trace before anything was clicked. The print dialog \
             opened on its own, which pdfcer may not do — and this is the one dialog whose commit \
             button consumes paper."
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- A. the File tab ---------------------------------------------------
    //
    // Clicked rather than assumed. The application opens with a tab already
    // active and it is not guaranteed to be this one; clicking a tab that is
    // already active is a no-op the ribbon handles, so this costs nothing and
    // removes a precondition.
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. The File tab is in every mode's tab \
             list, so this is not a mode problem — either the tab strip is too narrow and it has \
             moved into the overflow menu, or the manifest has changed. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(12);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line, so no click \
             reached the ribbon and the control below was never on screen."
        )));
    }

    // --- B. the Print control ----------------------------------------------
    // Through the overflow when the ribbon has folded it there.
    //
    // At the harness's 1100 pt window the File tab correctly folds its
    // rightmost groups — Print among them — into the overflow menu. That is the
    // responsive layout working. A lookup that read only the tab surface
    // reported "none of its controls is `ribbon.item.file.print`", which is
    // true, reads as "Print is missing", and is false — and it stood as this
    // check's FAIL for days, written up as a harness gap and left there.
    // See [`crate::checks::driving::declared_or_in_overflow`].
    let Some(control) =
        crate::checks::driving::declared_or_in_overflow(&session, &driver, ui_rect, SUBJECT)?
    else {
        let trace = session.trace()?;
        return Ok(Some(format!(
            "the File tab is active and neither it nor its overflow declares `{SUBJECT}`. \
             Controls declared: {}.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    if !control.is_substantial() {
        return Ok(Some(format!(
            "`{SUBJECT}` was declared at {control:?}, which has no usable area — the control is \
             laid out and not on screen."
        )));
    }
    driver.click_at(session.frame()?.declared_center(control))?;
    // Longer than a tab click's settle, and for a reason specific to this
    // surface: enumerating printers touches the spooler, which BLOCKS on a
    // network printer. This machine's list includes network devices, and a
    // settle tuned to a local-only machine would report "the dialog never
    // opened" for a dialog that was three seconds from opening.
    session.settle(40);

    // --- C. the verdict ----------------------------------------------------
    let trace = session.trace()?;
    let Some(open) = trace.events(OPEN_EVENT).next() else {
        // Distinguish the three ways this can be empty before naming one.
        //
        // "No `print-open` line" is a symptom with three causes and wildly
        // different fixes, and a check that reports the wrong one costs the
        // reader the whole investigation. `egui-shell` publishes the command
        // it dispatched, so the trace can tell them apart:
        //
        // - no invoke at all   → the click never reached the ribbon
        // - invoke, then       → the command was dispatched and refused: a
        //   `command-unimplemented`  registered command with no arm
        // - invoke, no refusal → the arm ran and opened nothing
        let shell = shell_trace(&session)?;
        let invoked = shell
            .events(INVOKE_EVENT)
            .any(|l| l.get("command") == Some(SUBJECT_ID));
        let refused = shell
            .events(UNIMPLEMENTED_EVENT)
            .any(|l| l.get("command") == Some(SUBJECT_ID));
        let diagnosis = match (invoked, refused) {
            (false, _) => {
                "no `command-invoke` line names it either, so the click did not reach \
                           the ribbon at all — this is a harness failure, not an application one, \
                           and the rect it aimed at is in the trace"
            }
            (true, true) => {
                "the shell dispatched it and reported `command-unimplemented`, so \
                             `file.print` is a registered command with no arm in `app::dispatch`"
            }
            (true, false) => {
                "the shell dispatched it and nothing refused it, so the arm ran and \
                              opened no dialog"
            }
        };
        return Ok(Some(format!(
            "the click on `{SUBJECT}` produced no `{OPEN_EVENT}` line, and {diagnosis}. Trace: {}.",
            session.trace_path().display()
        )));
    };

    let unavailable = open.get("unavailable").unwrap_or("<absent>");
    let printers: usize = open
        .get("printers")
        .and_then(|n| n.parse().ok())
        .unwrap_or_default();

    // The defect, named exactly. `NotLinked` was the refusal the four holes
    // returned; the variant no longer exists, so seeing it means an old binary
    // is being driven, which is a far more useful thing to say than "printers
    // was zero".
    if unavailable.contains("NotLinked") {
        return Ok(Some(format!(
            "the print dialog opened and reported `unavailable={unavailable}`. That is the \
             refusal the four adapter holes in `dialogs::print::spooler` returned before they \
             were filled — and it is now unconstructible in source, so this binary predates the \
             fix. Rebuild, or pass --exe at the build under test."
        )));
    }

    if unavailable != "None" {
        // A real spooler failure. Not this check's defect, and not a pass
        // either: the adapter reached the engine (which is the property under
        // test) but the machine could not answer, so nothing was learned about
        // whether the dialog works.
        return Err(Error::new(format!(
            "the adapter reached the engine and the engine refused: `unavailable={unavailable}`. \
             That is a real spooler failure on this machine, not the defect this check is about — \
             the Print Spooler service may be stopped. Reported as SKIPPED because a refused \
             enumeration proves nothing either way about the dialog."
        )));
    }

    if printers == 0 {
        return Err(Error::new(
            "the spooler answered and reported no printers installed. `unavailable=None` alone \
             is satisfiable by a stub returning an empty list, so this check requires at least \
             one device before it will claim a pass — see the module header. Reported as SKIPPED: \
             on a machine with no printers this check has learned nothing.",
        ));
    }

    report.note(format!(
        "the spooler answered with {printers} printer(s); the adapter is reaching `pdfcer-print`"
    ));

    // --- D. AND IT OPENED IN ITS OWN OS WINDOW -------------------------
    //
    //
    // > *"Print dialogue box doesn't pop up in its own movable window. It is
    // > locked within the boundaries of the program's window. Like, I just
    // > assume you've been trained on a million lines of code and software that
    // > pops it up in its own window."*
    //
    // The oracle is `viewport-inner`, and there is no other. A screenshot of
    // the application window cannot show it — a dialog in its own window is
    // *absent* from that capture, and an in-viewport panel that regressed would
    // look like a perfectly good dialog in it. "Is this a separate OS window"
    // is a fact about the window manager, and the only thing in the process
    // that knows the answer is the viewport egui created.
    //
    // A build that reverted to `egui::Window` emits no such line, and every
    // other assertion in this check goes on passing — which is exactly the
    // shape of regression this harness exists to catch.
    let trace = session.trace()?;
    let Some(viewport) = trace.events(VIEWPORT_INNER_EVENT).last() else {
        return Ok(Some(format!(
            "★ THE PRINT DIALOG DID NOT OPEN IN ITS OWN OS WINDOW: no \
             `{VIEWPORT_INNER_EVENT}` line followed `{OPEN_EVENT}`.\n\
             That is the operator's report of 2026-08-20 — a dialog locked inside the \
             application's window, which cannot be moved off the document it is asking about, \
             cannot go to a second monitor and does not appear in the taskbar. Look at \
             `dialogs::print::PrintDialog::show`: it must go through `dialogs::host::Host`, not \
             `egui::Window`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ it is a real OS window: `{}`", viewport.raw));
    Ok(None)
}
