//! `document_tabs_can_be_rearranged` — dragging a tab along the strip moves
//! it, and does **not** change which document is on screen.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/tab_reorder.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The prefix of the per-tab regions.
const TAB: &str = "doc-tab.";
/// The trace line a completed reorder produces.
const REORDER: &str = "document-reorder";
/// The strip's once-per-change summary, for the active slot afterwards.
const SUMMARY: &str = "doc-tabs";
/// The picker-answering environment variable. See [`super::document_tabs`].
const OPEN_PATH_ENV: &str = "PDFCER_DIAG_OPEN_PATH";
/// The chord that opens a document.
const CTRL_O: u16 = 0x4F;

/// See the module documentation.
pub struct DocumentTabsCanBeRearranged;

impl Check for DocumentTabsCanBeRearranged {
    fn name(&self) -> &'static str {
        "document_tabs_can_be_rearranged"
    }

    fn defect(&self) -> &'static str {
        "a document tab cannot be dragged to a new position — the strip's order is whatever \
         order the files were opened in and stays that way — or it can, and doing it switches \
         the operator to a different document"
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
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    let second = ctx.second_pdf.clone().ok_or_else(|| {
        Error::new(
            "no second document. Pass --second-pdf <path> — a file DIFFERENT from --pdf. \
             There is nothing to rearrange with one tab.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check drags a tab along the strip. Reported \
             as SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("tab_reorder.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((OPEN_PATH_ENV.to_owned(), second.display().to_string()));
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
    let driver = Driver::new(session.window());

    // --- 1: two tabs -------------------------------------------------------
    driver.press_chord(&[crate::input::Key::Ctrl.vk()], CTRL_O)?;
    session.settle(60);
    let trace = session.trace()?;
    let tabs = declared_names(&trace, ui_rect, TAB);
    if tabs.len() < 2 {
        return Err(Error::new(format!(
            "only {} document tab(s) after Ctrl+O. `two_documents_get_two_tabs` is the check \
             that diagnoses why. Tabs: {}.",
            tabs.len(),
            list(&tabs)
        )));
    }

    // ★ The SECOND document is the active one — a newly opened document always
    // is — and that is the operand this check needs, because the interesting
    // failure is the active document following an index. Dragging tab 0 across
    // tab 1 moves the tab the operator is NOT looking at, so a build that
    // tracks indices switches them to it.
    let Some(before) = trace.last(SUMMARY) else {
        return Err(Error::new(format!(
            "no `{SUMMARY}` line, so the strip did not report what it drew."
        )));
    };
    let active_before = before.get("active").unwrap_or("?").to_owned();
    report.note(format!(
        "two documents, slot {active_before} on screen: `{}`",
        before.raw
    ));

    let Some(tab0) = declared(&trace, ui_rect, &format!("{TAB}0")) else {
        return Err(Error::new(format!("no `{TAB}0` region.")));
    };
    let Some(tab1) = declared(&trace, ui_rect, &format!("{TAB}1")) else {
        return Err(Error::new(format!("no `{TAB}1` region.")));
    };

    // --- 2: drag the FIRST tab past the middle of the second ---------------
    //
    // Three-quarters across, not the far edge: the boundary is resolved from
    // the neighbour's centre, so anything past the midpoint names the same gap
    // — and a point exactly on the centre is where a rounding difference
    // between the application's `f32` rectangle and this harness's reading of
    // it could flip the answer.
    let frame = session.frame()?;
    let from = frame.declared_center(tab0);
    let onto = frame.declared_at(tab1, 0.75, 0.5);
    report.note(format!(
        "dragging tab 0 from ({}, {}) past the middle of tab 1 at ({}, {})",
        from.x(),
        from.y(),
        onto.x(),
        onto.y()
    ));
    driver.drag(from, onto)?;
    session.settle(40);

    // --- 3: it moved -------------------------------------------------------
    let trace = session.trace()?;
    let Some(reorder) = trace.last(REORDER) else {
        return Ok(Some(format!(
            "tab 0 was dragged past the middle of tab 1 and no `{REORDER}` line was traced. \
             The tab does not sense a drag — which is the state a strip built from plain \
             `egui::Button`s ships in, because a `Button` senses clicks and nothing else."
        )));
    };
    report.note(format!("the strip took the drop: `{}`", reorder.raw));

    if reorder.get("to") != Some("1") {
        return Ok(Some(format!(
            "tab 0 was dropped past the middle of tab 1 and landed at index {}. Past the \
             middle of the second tab is the boundary AFTER it, which for a tab moving \
             rightward is index 1 — the tab is removed before it is re-inserted, so the \
             boundary and the destination differ by one. Line: `{}`.",
            reorder.get("to").unwrap_or("?"),
            reorder.raw
        )));
    }

    // --- 4: ★★ and the operator is still looking at the same document ------
    let Some(after) = trace.last(SUMMARY) else {
        return Ok(Some(format!(
            "the reorder happened and the strip stopped reporting `{SUMMARY}`."
        )));
    };
    let active_after = after.get("active").unwrap_or("?");
    // Slot 1 held the active document before; tab 0 moved to index 1, so the
    // document that was at 1 is now at 0. The ACTIVE SLOT must have followed
    // it down, which is the whole assertion.
    if active_after != "0" {
        return Ok(Some(format!(
            "before the drag the operator was looking at slot {active_before}; tab 0 moved to \
             index 1, so the document they were reading is now at slot 0 — and the strip \
             reports slot {active_after} active. The active document followed an INDEX rather \
             than its own tab, so rearranging the strip switched the operator to a different \
             drawing. Line: `{}`.",
            after.raw
        )));
    }
    report.note("the same document is still on screen, at its new slot");
    Ok(None)
}
