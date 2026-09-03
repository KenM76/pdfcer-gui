//! `two_documents_get_two_tabs` — opening a second PDF adds a tab, and the
//! tab switches to it.
//!
//! # The gap this closes
//!
//! Until 2026-08-20 pdfcer could hold exactly one document. Opening the next
//! drawing **replaced** the one on screen, which is why `Action::Open` needed
//! an unsaved-edits prompt in front of it. The operator asked for the
//! obvious thing:
//!
//! > *"make it so we can open multiple PDFs at once…"*
//!
//! # Why this needs driving, and could not be unit-tested
//!
//! `crate::app::documents` has nine unit tests over the tab arithmetic — the
//! strip order, the browser close rule, the wrap on Ctrl+Tab. **Every one of
//! them passes on a build where the strip is never drawn**, because the
//! arithmetic is a pure function of two fields and the drawing is four
//! frame-level facts none of them can reach:
//!
//! 1. the top panel is composed at all, and *before* the docks, or it starts
//!    at the dock's edge instead of spanning the window;
//! 2. `egui_shell::tabstrip` is handed a non-empty list;
//! 3. a tab's rectangle is somewhere an operator can click;
//! 4. a click on it reaches `activate_slot`.
//!
//! Exactly the shape of gap this project was founded on: `panels::pages` had
//! its whole capability, a passing test suite, and **no registration**, so an
//! operator never saw it (`panels::pages`' header §1, *"invisible rather than
//! broken, which is the honest failure and also the silent one"*).
//!
//! # What a passing run does NOT prove
//!
//! That the labels are legible, or that the ✕ closes anything. The first is
//! [`super::legibility`]'s kind of question and the second is a destructive
//! act this check deliberately does not perform — a close behind an
//! unsaved-edits prompt is its own gesture with its own failure modes.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The strip's own region.
const STRIP: &str = "doc-tabs";
/// The prefix of the per-tab regions; the 0-based slot is appended.
const TAB: &str = "doc-tab.";
/// The trace line a tab click produces.
const ACTIVATE: &str = "document-activate";
/// The environment variable that answers the Open dialog instead of opening
/// it, so a check can open a second document without a native modal.
///
/// `D:\dev\rag\egui\native_file_dialog_is_a_hard_wall_substitute_the_answer_via_env_var.md`
/// carries the finding; `crate::app::files::DIAG_OPEN_PATH` is the constant on
/// the application side.
const OPEN_PATH_ENV: &str = "PDFCER_DIAG_OPEN_PATH";
/// The chord that opens a document.
const CTRL_O: u16 = 0x4F;

/// See the module documentation.
pub struct TwoDocumentsGetTwoTabs;

impl Check for TwoDocumentsGetTwoTabs {
    fn name(&self) -> &'static str {
        "two_documents_get_two_tabs"
    }

    fn defect(&self) -> &'static str {
        "opening a second PDF replaces the first, or opens beside it with no tab to say so — \
         so the operator has no way to see what they have open and no way back to it"
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
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no fixture document. Pass --pdf."))?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check presses a chord and clicks a tab. \
             Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    // ★ The second document must be a DIFFERENT FILE, and the check skips
    // rather than falling back to `--pdf`.
    //
    // `crate::app::documents` §3 makes opening an already-open path **activate
    // its tab** rather than open a duplicate — deliberately, because two
    // `EditSession`s over one file would be two undo stacks and a save from
    // either would silently discard the other's work. So a build that got that
    // rule right would answer a same-path open with ONE tab, and this check
    // would fail for the correct reason, reported as the wrong one.
    let second = ctx.second_pdf.clone().ok_or_else(|| {
        Error::new(
            "no second document. Pass --second-pdf <path> — a file DIFFERENT from --pdf. \
                 The same path cannot be used twice: opening a path that is already open \
                 activates its tab by design, so this check would be asserting the opposite \
                 of what it is for.",
        )
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("document_tabs.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // The picker's answer, supplied before the process starts. See
    // `OPEN_PATH_ENV`.
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

    // --- 1: one document, one tab -----------------------------------------
    //
    // Asserted BEFORE the second is opened, because it is the precondition
    // that makes step 3 a statement about opening rather than about drawing.
    // A build that draws no strip at all fails here, with a message that says
    // so, instead of failing later with "the second tab is missing".
    let trace = session.trace()?;
    if declared(&trace, ui_rect, STRIP).is_none() {
        return Ok(Some(format!(
            "one document is open and no `{STRIP}` region was published, so the document tab \
             strip is not drawn. Regions beginning `doc-`: {}.",
            list(&declared_names(&trace, ui_rect, "doc-"))
        )));
    }
    let first = declared_names(&trace, ui_rect, TAB);
    if first.len() != 1 {
        return Ok(Some(format!(
            "one document is open and the strip drew {} tab(s): {}. It should draw exactly \
             one — a strip that is empty with a document open cannot be clicked, and one that \
             is not empty with none is furniture asserting there is something to switch \
             between.",
            first.len(),
            list(&first)
        )));
    }
    report.note("one document, one tab");

    // --- 2: open the second ------------------------------------------------
    driver.press_chord(&[crate::input::Key::Ctrl.vk()], CTRL_O)?;
    session.settle(60);

    let trace = session.trace()?;
    let tabs = declared_names(&trace, ui_rect, TAB);
    if tabs.len() < 2 {
        return Ok(Some(format!(
            "Ctrl+O was pressed with `{OPEN_PATH_ENV}` set to {}, and the strip still shows \
             {} tab(s): {}. Either the open did not happen — look for an `open ok` line — or \
             it REPLACED the first document, which is the behaviour this whole feature exists \
             to remove.",
            second.display(),
            tabs.len(),
            list(&tabs)
        )));
    }
    report.note(format!("two documents, {} tabs", tabs.len()));

    // The second document is the one on screen: a newly opened document is
    // always the active one, in every application in this class.
    let Some(opened) = trace.last("open") else {
        return Ok(Some(
            "two tabs are drawn and no `open` line was traced, so the strip grew without a \
             document arriving. That is a tab list that is not a list of documents."
                .to_owned(),
        ));
    };
    report.note(format!("second document arrived: `{}`", opened.raw));

    // --- 3: clicking the first tab goes back to it -------------------------
    let Some(tab0) = declared(&trace, ui_rect, &format!("{TAB}0")) else {
        return Ok(Some(format!(
            "no `{TAB}0` region, so the first document's tab is not on screen even though its \
             document is open. Tabs drawn: {}.",
            list(&tabs)
        )));
    };
    let frame = session.frame()?;
    let centre = frame.declared_center(tab0);
    report.note(format!(
        "clicking the first document's tab at ({}, {})",
        centre.x(),
        centre.y()
    ));
    driver.click_at(centre)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(activated) = trace.last(ACTIVATE) else {
        return Ok(Some(format!(
            "the first tab was clicked at its own published rectangle and no `{ACTIVATE}` line \
             followed. The tab is drawn and inert — which is `R9`'s failure exactly: a visible \
             control that does nothing."
        )));
    };
    if activated.get("slot") != Some("0") {
        return Ok(Some(format!(
            "clicking the tab at slot 0 activated slot {}. The strip's drawn order and the \
             application's slot numbering disagree, so every click lands on the wrong \
             document. Line: `{}`.",
            activated.get("slot").unwrap_or("?"),
            activated.raw
        )));
    }
    report.note("the click switched back to the first document");
    Ok(None)
}
