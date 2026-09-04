//! `print_paper_changes_the_plan` — choosing a sheet must re-plan the job,
//! not merely re-label it.
//!
//! # What this is about
//!
//! On 2026-08-18 `pdfcer-print` answered this project's third filing about the
//! print path and shipped paper selection: `printer_forms`, `PaperSelection`,
//! `printer_caps_for` and a properties dialog. The shell grew a paper combo
//! and a **Properties…** button beside the printer selector — a surface the
//! operator had asked for three times.
//!
//! The interesting failure is not "the combo does not appear". It is this:
//!
//! > The combo appears, the operator picks A3, the label reads A3, the job is
//! > **planned against the device's default sheet**, and the pages come out
//! > scaled for Letter on A3 paper with no clip reported and nothing to
//! > explain it.
//!
//! That is the same defect `DeviceGeometry::from_caps` exists to prevent — a
//! plan computed against one sheet and printed onto another — arriving through
//! a second dimension. `printer_caps` reads the device's *default* `DEVMODE`;
//! `printer_caps_for` reads the geometry of the sheet the job actually asked
//! for. A shell that adopted paper selection without switching to the second
//! function would look entirely correct from inside: the label is right, the
//! request is sent, the paper is right, and only the *scale* is wrong.
//!
//! **No unit test in this workspace can see it.** The conversion test pins
//! that `PaperChoice::Form(8)` becomes `PaperSelection::Form(8)`, and it would
//! keep passing. The geometry comes from a Windows device context, so the only
//! evidence that the plan followed the request is what a running process says
//! about a real driver.
//!
//! # What it measures
//!
//! One trace line, emitted every frame the dialog is open:
//!
//! ```text
//! print-plan printer="…" … orientation=Auto duplex=Simplex paper=DeviceDefault
//!            sheet=Some((612.0, 792.0)) config=false scale=Some(0.97) tab=PagesLayout
//! ```
//!
//! `paper=` is what was **asked for**; `sheet=` is the physical sheet the
//! geometry came **back** with. The check reads the line before and after
//! choosing an entry from the list, and requires both to move:
//!
//! | `paper=` | `sheet=` | verdict |
//! |---|---|---|
//! | unchanged | unchanged | the click did not reach the entry — harness failure, reported as a skip with the rect it aimed at |
//! | changed | unchanged | ★ **the defect** — the request was recorded and the plan ignored it |
//! | changed | changed | pass |
//!
//! ## ★ Why the second row needs more than one click before it is believed
//!
//! A driver is entitled to enumerate a form whose size **equals the sheet the
//! device already defaults to** — `dmPaperSize` naming Letter on a
//! Letter-default printer is not a bug, it is the list being complete. Choosing
//! that one moves `paper=` and correctly leaves `sheet=` where it was, which is
//! indistinguishable, from one click, from the defect above.
//!
//! The harness cannot read an entry's label — it publishes rects, not text — so
//! it cannot pick a form it knows to be different. It therefore tries several,
//! and reaches the verdict only when **every** form tried leaves the sheet
//! unmoved. One that moves it proves the plumbing and stops the loop.
//!
//! The number tried is capped and the cap is **reported**, not silent: a
//! driver's list can run to forty entries, five clicks is enough to settle the
//! question, and a report that said "tried the list" while trying an eighth of
//! it would read as coverage it did not have.
//!
//! # ★ What it deliberately does NOT do
//!
//! **It never presses Properties….** That button opens the *driver's own*
//! modal dialog — a nested Win32 message loop owned by a vendor's print
//! driver, whose layout, controls and dismissal keys pdfcer does not know and
//! cannot publish rects for. A harness that opened one would be driving
//! somebody else's UI blind, and a driver dialog left up blocks the
//! application's event loop, so a failed dismissal does not fail the check —
//! it hangs the run and every check after it.
//!
//! The button is still asserted, from outside: its rect is published as
//! `print.properties`, and this check requires it to be declared and
//! substantial. That is the whole of what a harness can honestly claim about
//! it — *it is drawn, where it is drawn, at a usable size* — and the rest is
//! a human pressing it once. The same reasoning, and the same limit, as
//! `print_dialog`'s refusal to press commit.
//!
//! **It never presses commit.** Same rule, same reason, and it is stated in
//! both files rather than by reference: a harness that can start a print job
//! will eventually start one by accident.

use crate::checks::driving::{
    ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, declared, declared_in, declared_names, frame_for, list,
    shell_trace,
};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};

/// The ribbon control that opens the dialog, and the tab it lives on.
const SUBJECT: &str = "ribbon.item.file.print";
const TAB_ID: &str = "file";
const TAB: &str = "ribbon.tab.file";

/// The trace event the dialog emits once, when it is built.
const OPEN_EVENT: &str = "print-open";

/// The per-frame line carrying `paper=` and `sheet=`.
const PLAN_EVENT: &str = "print-plan";

/// The line reporting what the selected device said about itself.
const FEATURES_EVENT: &str = "print-features";

/// The paper combo, closed, and the prefix its open entries publish under.
const PAPER: &str = "print.paper";
const PAPER_ITEM_PREFIX: &str = "print.paper.item.";

/// The Properties… button. Read, never clicked — see the module header.
const PROPERTIES: &str = "print.properties";

pub struct PrintPaperChangesThePlan;

impl Check for PrintPaperChangesThePlan {
    fn name(&self) -> &'static str {
        "print_paper_changes_the_plan"
    }

    fn defect(&self) -> &'static str {
        "choosing a paper size relabels the control and the job is still planned for the device's default sheet"
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

#[allow(clippy::too_many_lines)]
fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. `file.print` is gated on `doc.open`, so with nothing open the control is \
             greyed and there is no dialog to reach.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check is four clicks. Reported as SKIPPED \
             rather than passed — a check that did not run has learned nothing.",
        ));
    }

    let mut spec = LaunchSpec::new(&exe, ctx.out("print_paper.trace.txt"));
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
            "the trace has no `{}` line, so {}={} did not reach the process. Captured stderr is \
             at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }

    let driver = Driver::new(session.window());
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // --- A. open the dialog -------------------------------------------------
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region. Tabs declared: {}.",
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
             reached the ribbon."
        )));
    }

    // ★ Through the overflow when the ribbon has folded it there.
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
        return Err(Error::new(format!(
            "the File tab is active and neither it nor its overflow declares `{SUBJECT}`. \
             Controls declared: {}. That is `print_dialog`'s defect, not this one — it is \
             reported there.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    };
    driver.click_at(session.frame()?.declared_center(control))?;
    // Enumerating printers touches the spooler, which BLOCKS on a network
    // printer, and this dialog now also enumerates the selected device's paper
    // forms. Same settle as `print_dialog`'s, for the same reason.
    session.settle(40);

    let trace = session.trace()?;
    let Some(open) = trace.events(OPEN_EVENT).next() else {
        return Err(Error::new(format!(
            "the click on `{SUBJECT}` produced no `{OPEN_EVENT}` line, so the dialog never \
             opened. That is `print_dialog`'s subject and it reports the three causes apart; \
             nothing about paper can be learned here."
        )));
    };
    if open.get("unavailable").unwrap_or("<absent>") != "None" {
        return Err(Error::new(
            "the spooler refused on this machine, so there is no device to enumerate paper for. \
             Reported as SKIPPED: a refused enumeration proves nothing either way.",
        ));
    }

    // --- B. what the device said about itself -------------------------------
    //
    // Read before touching anything. `forms=0` is a legal answer from a driver
    // and is NOT a defect — it means the shell renders a sentence instead of a
    // combo — but it does mean this check has nothing to choose from.
    let features = trace.events(FEATURES_EVENT).last().ok_or_else(|| {
        Error::new(format!(
            "the dialog opened and emitted no `{FEATURES_EVENT}` line, so the per-device read \
             that fills the paper list did not run."
        ))
    })?;
    let forms: usize = features
        .get("forms")
        .and_then(|n| n.parse().ok())
        .unwrap_or_default();
    report.note(format!(
        "the selected device enumerated {forms} paper form(s); form_source={}",
        features.get("form_source").unwrap_or("<absent>")
    ));
    if forms < 2 {
        return Err(Error::new(format!(
            "the selected device enumerated {forms} paper form(s). This check needs at least two \
             to have a sheet to switch TO. Reported as SKIPPED — a driver that lists no forms is \
             answering legally, and the shell's response to it (a sentence, not an empty combo) \
             is asserted by the string gate rather than here."
        )));
    }

    // --- C. the Properties… button is drawn ---------------------------------
    //
    // Read, never clicked. See the module header for why a harness may not
    // open a vendor driver's modal dialog.
    let trace = session.trace()?;
    let Some(properties) = declared(&trace, ui_rect, PROPERTIES) else {
        return Ok(Some(format!(
            "the print dialog is open and published no `{PROPERTIES}` region, so the \
             Properties… button beside the printer selector is not being drawn. That control is \
             what the operator asked for by name, three times."
        )));
    };
    if !properties.is_substantial() {
        return Ok(Some(format!(
            "`{PROPERTIES}` was declared at {properties:?}, which has no usable area — the \
             button is laid out and not on screen."
        )));
    }

    // --- D. the plan, before ------------------------------------------------
    let before = last_plan(&session)?;
    let paper_before = before.0;
    let sheet_before = before.1;
    report.note(format!("before: paper={paper_before} sheet={sheet_before}"));

    // --- E. open the list and choose a DIFFERENT sheet -----------------------
    // ★★ FROM HERE ON THE REGIONS ARE IN THE DIALOG'S OWN OS WINDOW.
    //
    // As of 2026-08-20 the print dialog is a real OS window (`dialogs::host`,
    // and `ui-conventions/dialogs.md` G1 — the operator's report). Its
    // `ui-rect` rectangles are relative to **its** client area, not the
    // application's, so `session.frame()` is the wrong origin for every one of
    // them: it produces coordinates that look entirely reasonable and land
    // several hundred points away.
    //
    // `declared_in` carries the viewport tag and `frame_for` turns it into the
    // right origin. It is re-resolved per click rather than hoisted, because
    // **the operator can move the window** — and more to the point, so can this
    // harness's own clicks nudge it. A frame captured once and reused is the
    // stale-coordinate defect `stable_rect` exists for, one space along.
    let (paper, paper_vp) = declared_in(&trace, ui_rect, PAPER).ok_or_else(|| {
        Error::new(format!(
            "the dialog published no `{PAPER}` region. The Pages & Layout tab is the dialog's \
             default tab, so this is not a tab problem — either the combo is not being drawn or \
             the device enumerated no forms, and the line above says it enumerated {forms}."
        ))
    })?;
    driver.click_at(frame_for(&session, &trace, paper_vp.as_deref())?.declared_center(paper))?;
    session.settle(12);

    let trace = session.trace()?;
    let entries = declared_names(&trace, ui_rect, PAPER_ITEM_PREFIX);
    if entries.len() < 2 {
        return Err(Error::new(format!(
            "the click on `{PAPER}` published {} entry region(s). The popup did not open, or \
             opened and closed within the settle. Reported as SKIPPED: this is a harness timing \
             question, not an application claim.",
            entries.len()
        )));
    }

    // ★ Entry 1 upward, never entry 0. Entry 0 is "from the printer's own
    // settings" — the state the dialog is already in — so clicking it would
    // leave `paper=` unchanged and the check would report a harness failure it
    // had caused itself. Entry 1 is the driver's first enumerated form.
    //
    // ★★ And it TRIES SEVERAL, which is the difference between a check and a
    // false accusation. A driver is entitled to list a form whose size equals
    // the sheet the device already defaults to — `dmPaperSize` naming Letter on
    // a Letter-default printer is not a bug — and choosing that one produces an
    // honest, correct, UNCHANGED `sheet=`. A check that took one entry and
    // concluded "the plan ignored the request" would be reporting the driver's
    // paper list as a pdfcer defect.
    //
    // So: the verdict is only reached when EVERY form tried leaves the sheet
    // where it was. One that moves it proves the plumbing, and the loop stops
    // there.
    let mut attempts: Vec<String> = Vec::new();
    let mut last_paper = paper_before.clone();
    let mut moved: Option<(String, String)> = None;

    // A cap, stated rather than silent: five clicks is seconds of wall time
    // and a driver's list can run to forty entries. If none of the first five
    // moves the sheet, a sixth is not going to change the verdict — and the
    // report says how many were tried, so nobody reads "tried the list" for
    // "tried all of it".
    let ceiling = entries.len().min(6);
    for index in 1..ceiling {
        let target = format!("{PAPER_ITEM_PREFIX}{index}");
        let trace = session.trace()?;
        let Some((entry, entry_vp)) = declared_in(&trace, ui_rect, &target) else {
            // The popup closed after the previous click, which is what a
            // combo does. Reopen it and look again.
            driver.click_at(
                frame_for(&session, &trace, paper_vp.as_deref())?.declared_center(paper),
            )?;
            session.settle(12);
            let trace = session.trace()?;
            let Some((entry, entry_vp)) = declared_in(&trace, ui_rect, &target) else {
                continue;
            };
            attempts.push(target.clone());
            driver.click_at(
                frame_for(&session, &trace, entry_vp.as_deref())?.declared_center(entry),
            )?;
            // Longer than a widget settle: the choice re-reads the device
            // geometry through `printer_caps_for`, which opens an information
            // device context.
            session.settle(25);
            let (paper_after, sheet_after) = last_plan(&session)?;
            report.note(format!("{target}: paper={paper_after} sheet={sheet_after}"));
            last_paper.clone_from(&paper_after);
            if paper_after != paper_before && sheet_after != sheet_before {
                moved = Some((paper_after, sheet_after));
                break;
            }
            continue;
        };
        attempts.push(target.clone());
        driver
            .click_at(frame_for(&session, &trace, entry_vp.as_deref())?.declared_center(entry))?;
        session.settle(25);
        let (paper_after, sheet_after) = last_plan(&session)?;
        report.note(format!("{target}: paper={paper_after} sheet={sheet_after}"));
        last_paper.clone_from(&paper_after);
        if paper_after != paper_before && sheet_after != sheet_before {
            moved = Some((paper_after, sheet_after));
            break;
        }
    }

    // --- F. the verdict -----------------------------------------------------
    if let Some((paper_after, sheet_after)) = moved {
        report.note(format!(
            "choosing `{paper_after}` re-planned the job from {sheet_before} to {sheet_after} \
             (after {} click(s) of a {}-entry list)",
            attempts.len(),
            entries.len()
        ));
        return Ok(None);
    }

    if last_paper == paper_before {
        return Err(Error::new(format!(
            "{} click(s) on {} left `paper={last_paper}` unchanged, so none of them landed on an \
             entry. Reported as SKIPPED rather than failed: nothing was learned about whether the \
             plan follows a choice, because no choice was made. Trace: {}.",
            attempts.len(),
            list(&attempts),
            session.trace_path().display()
        )));
    }

    Ok(Some(format!(
        "★ the paper request moved to `{last_paper}` and the planned sheet did NOT: it is still \
         `{sheet_before}` after trying {} of this driver's {} form(s) ({}). The job is being laid \
         out against the device's default sheet and printed onto the requested one — the pages \
         will be scaled for the wrong paper, no clip will be reported, and nothing on screen will \
         say so. `dialogs::print::spooler::plan` must read `printer_caps_for(printer, config, \
         paper)`, not `printer_caps(printer)`.",
        attempts.len(),
        entries.len().saturating_sub(1),
        list(&attempts),
    )))
}

/// The `paper=` and `sheet=` fields of the most recent `print-plan` line.
///
/// The **last** line rather than the first: the dialog emits one per frame, so
/// the first describes the state it opened in and only the last describes the
/// state after a click. Reading the first is a mistake that would make every
/// assertion below trivially true and is worth naming.
fn last_plan(session: &Session) -> Result<(String, String)> {
    let trace = session.trace()?;
    let line = trace.events(PLAN_EVENT).last().ok_or_else(|| {
        Error::new(format!(
            "the dialog is open and the trace has no `{PLAN_EVENT}` line, so nothing reports what \
             the job was planned against."
        ))
    })?;
    let paper = line.get("paper").unwrap_or("<absent>").to_owned();
    let sheet = line.get("sheet").unwrap_or("<absent>").to_owned();
    Ok((paper, sheet))
}
