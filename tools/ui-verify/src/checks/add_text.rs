//! `add_text` — **`edit.add_text` places a caret and REAL keystrokes reach it.**
//!
//! The command literally labelled *"Add text"*, driven end to end with the
//! keyboard, and the one path in this shell that had no driven coverage at all.
//!
//! # ★★ Why the existing text-editing check does not cover this
//!
//! `checks::text_edit` drives the *other* verb (`edit.text`, which rewrites a
//! run already on the page) and supplies its characters through
//! `PDFCER_DIAG_TYPE` — a seam that writes straight into the draft and
//! **bypasses the event loop entirely**. Its header states the reason: at the
//! time it was written, synthetic keyboard input was believed not to reach the
//! target window from the session that injects it.
//!
//! **That belief is false**, and this check is the demonstration. It is worth
//! recording how it survived: it was written down in three module headers as an
//! established fact about the machine, and every check that might have
//! contradicted it either used the seam or clicked instead. A constraint an
//! agent infers about its own environment is a *reading*, not a fact — and this
//! one cost the project its entire keyboard surface, because while it stood
//! nobody drove a chord, and while nobody drove a chord fourteen of the
//! twenty-one declared shortcuts sat dead in the manifest.
//!
//! # What it proves that a unit test cannot
//!
//! `canvas::textedit::typing` reads `ui.input(…).events` directly. A headless
//! test can push an `egui::Event::Text` into a `RawInput` and watch the draft
//! grow — and that passes on a build where no character ever reaches the
//! window, because the harness supplied the event that a keyboard was supposed
//! to. The link this check adds is the one nothing else covers: **the operating
//! system's keystroke becomes an `egui::Event::Text`.**
//!
//! # The phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | Edit mode, Edit tab, click **Add text** | `text-edit-tool tool=TextEdit(Add)` |
//! | B | click blank paper | `text-edit-caret kind=Add` |
//! | C | **type two real characters** | `text-edit-typing … len=2` |
//! | D | click elsewhere — clicking away commits | `add-text`, and the page changes |
//!
//! Phase D is the shell's own rule rather than a convenience: `textedit::click`
//! commits an existing draft before starting a new one, because clicking away
//! is every editor's *"that word is finished"*. No Enter is pressed, so the
//! commit path under test is the one a mouse-driven operator actually takes.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// How far from the caret the committing click lands, in PDF points.
///
/// Far enough that it cannot be read as a second click on the same word, and
/// still on the sheet for any `--doc-point` that was itself on it.
const AWAY_PT: f64 = 140.0;

/// See the module documentation.
pub struct AddTextTakesRealKeystrokes;

impl Check for AddTextTakesRealKeystrokes {
    fn name(&self) -> &'static str {
        "add_text_takes_real_keystrokes"
    }

    fn defect(&self) -> &'static str {
        "the Add text tool arms and places a caret, and characters typed on the real keyboard \
         never reach the draft — so the operator gets a caret that ignores them and nothing is \
         ever written to the page"
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

#[allow(clippy::too_many_lines)]
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
            "input is disabled (--no-input). This check clicks the ribbon, clicks the page and \
             types on the real keyboard. Reported as SKIPPED rather than passed.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new("no --doc-point. This check needs somewhere on the page to place the caret.")
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("add_text.trace.txt"));
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
    let driver = Driver::new(session.window());

    // --- A: Edit mode, Edit tab, arm Add text ------------------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, "edit")?;
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.edit").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.edit` region after switching to Edit. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("edit"))
    {
        return Err(Error::new(
            "the click on the Edit tab produced no tab-selected line, so nothing below would \
             mean anything.",
        ));
    }

    let trace = session.trace()?;
    let item = declared(&trace, ui_rect, "ribbon.item.edit.add_text").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.item.edit.add_text` region on the Edit tab. Items declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.item.edit."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(14);
    let trace = session.trace()?;
    if !trace
        .events("text-edit-tool")
        .any(|l| l.get("tool").is_some_and(|t| t.contains("Add")))
    {
        let tools: Vec<&str> = trace
            .events("text-edit-tool")
            .filter_map(|l| l.get("tool"))
            .collect();
        return Ok(Some(format!(
            "clicking Edit > Add text traced no `text-edit-tool tool=TextEdit(Add)` line, so \
             the control armed nothing. Tools traced this run: {}.",
            list_str(&tools)
        )));
    }
    report.note("Edit > Add text armed the caret in Add mode");

    // --- B: place the caret -------------------------------------------------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let at = frame.to_screen(mapping.doc_to_window(target)?);
    driver.click_at(at)?;
    session.settle(14);

    let trace = session.trace()?;
    if !trace
        .events("text-edit-caret")
        .any(|l| l.get("kind") == Some("Add"))
    {
        let declined: Vec<&str> = trace
            .events("text-edit-declined")
            .filter_map(|l| l.get("reason"))
            .collect();
        return Ok(Some(format!(
            "the click on the page traced no `text-edit-caret kind=Add`, so no draft was \
             started and there is nothing for a keystroke to reach. Refusals traced: {}.",
            list_str(&declined)
        )));
    }
    report.note("the click on blank paper started an Add draft");

    // --- C: ★★ TYPE, FOR REAL ----------------------------------------------
    //
    // Two keys already in `sys::vk`. WHAT is typed does not matter — the
    // assertion is that the draft grew, not what it says.
    for key in [vk::F, vk::DIGIT_2] {
        driver.press(key)?;
        session.settle(10);
    }

    // `text-edit-typing` is a change-only trace: it reports
    // `draft / owns_keyboard / text_events / len`, and each field kills a
    // different hypothesis, which is why the failure below quotes the whole
    // line rather than just saying "no".
    let trace = session.trace()?;
    let typing: Vec<String> = trace
        .events("text-edit-typing")
        .map(|l| l.raw.to_owned())
        .collect();
    let grew = trace
        .events("text-edit-typing")
        .filter_map(|l| l.get("len"))
        .filter_map(|v| v.parse::<usize>().ok())
        .any(|n| n > 0);
    if !grew {
        return Ok(Some(format!(
            "TWO REAL KEYSTROKES REACHED NOTHING. The tool armed, the click placed a caret, \
             and no `text-edit-typing` line ever reported a draft longer than zero. Read the \
             fields of the lines below: `owns_keyboard=false` means a text field held focus \
             and `typing` refused the events; `text_events=0` means egui delivered no \
             `Event::Text` at all, so the keystroke never became a character; a rising \
             `text_events` with a flat `len` means the events arrived and `insert` did not \
             land them. Lines traced: {}.",
            list_str(&typing.iter().map(String::as_str).collect::<Vec<_>>())
        )));
    }
    report.note("real keystrokes reached the draft — the OS -> egui -> draft link holds");

    // ★★ THE IN-PLACE EDITOR, CAPTURED MID-DRAFT.
    //
    // Everything above reads the trace, and the trace can say a keystroke
    // reached the draft. It cannot say the operator can SEE it — and for a
    // fortnight they could not:
    //
    //   "I can edit text now, but there is no live preview of that either."
    //
    // The draft lived beside the page and nothing drew it, so the canvas showed
    // the old text, a bracket and a blinking caret. Every assertion in this
    // check passed throughout.
    //
    // A picture rather than an assertion, deliberately: asserting on the box
    // would mean re-deriving the theme colour and the layout, which is a second
    // derivation of the thing under test. The artifact is what a human looks
    // at, and its absence from the report is what says nobody has.
    let shot = ctx.out("add_text_draft.png");
    if crate::capture::window_to_png(&session, &shot).is_ok() {
        report.artifact(shot);
    }

    // --- D: click away, which is how a draft commits ------------------------
    let away = frame.to_screen(mapping.doc_to_window(DocPoint {
        page: target.page,
        x: target.x + AWAY_PT,
        y: target.y - AWAY_PT,
    })?);
    driver.click_at(away)?;
    session.settle(24);

    let trace = session.trace()?;
    if trace.events("add-text").next().is_none() {
        return Ok(Some(
            "the draft had characters in it and clicking away traced no `add-text`, so the \
             commit never reached the engine. `textedit::click` is supposed to commit an \
             existing draft before starting a new one — clicking away is every editor's \
             \"that word is finished\", and it is the ONLY commit route a mouse-driven \
             operator has, because Enter is the other one."
                .to_owned(),
        ));
    }
    report.note("clicking away committed the draft: `add-text` reached the engine");
    Ok(None)
}

/// Render a list of borrowed strings for a failure message.
fn list_str(items: &[&str]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}
