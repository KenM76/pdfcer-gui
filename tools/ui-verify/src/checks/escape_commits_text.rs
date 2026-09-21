//! `escape_commits_text` — **Escape on a text draft WRITES it, and `Ctrl+Z`
//! takes it back.**
//!
//! # The request
//!
//! > *"I think for adding and editing text when using any tool that has text
//! > escape should also save changes to the text. The user can always undo if
//! > they want, but it is easy to accidentally press escape and lose a lot of
//! > text that has been entered."*
//!
//! # ★★★ Why this is a correctness rule and not a preference
//!
//! The operator's sentence carries its own argument, and it is about
//! **asymmetric cost**. A draft written by mistake is one `Ctrl+Z`. A draft
//! discarded by mistake is minutes of typing with **no recovery anywhere**,
//! because a draft lives in `egui::Memory` and never reaches the undo stack —
//! there is nothing for undo to restore. The two mistakes are not comparable,
//! so the exit that happens by accident must be the recoverable one.
//!
//! That is why phase E is part of this check rather than a separate one. The
//! ruling is not *"Escape commits"*; it is *"Escape commits **and** the commit
//! is recoverable"*, and a build that did the first without the second would
//! have replaced an unrecoverable loss with an unrecoverable gain.
//!
//! # ★★ The oracle the wrong build cannot produce
//!
//! `canvas-escape outcome=SettledTextDraft` says the Escape ladder reached the
//! text-draft rung. It does **not** say the text landed: the rung could fire
//! and the commit still fail. `add-text` is the line the engine writes when
//! the characters actually reach the document, and it is the assertion this
//! check turns on, because the behaviour being replaced — Escape discarding —
//! produces the rung's teardown and **no `add-text` at all**.
//!
//! ⚠ `text-edit-abandon` is not usable as an oracle in either direction. Its
//! own contract says it is not evidence that an edit was lost: it is emitted
//! by `settle`'s teardown half whatever brought the draft to an end, so it
//! appears on the correct build and on the wrong one alike.
//!
//! # The phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | Edit mode, Edit tab, click **Add text** | `text-edit-tool tool=TextEdit(Add)` |
//! | B | click blank paper | `text-edit-caret kind=Add` |
//! | C | type two real characters | `text-edit-typing … len>0` |
//! | D | **press Escape** | `canvas-escape outcome=SettledTextDraft` *and* `add-text` |
//! | E | `Ctrl+Z` | `undo-applied` — the recovery the ruling rests on |
//!
//! # What it cannot see
//!
//! * **The other surface.** `checks::form_field` drives a form-field editor,
//!   which carries text too and settles the same way on Escape. This check
//!   drives the canvas draft only.
//! * **Whether the committed text is what was typed.** Phase D asserts the
//!   commit reached the engine, not its content — the same scope as the
//!   clicking-away commit in [`crate::checks::add_text`].
//! * **A draft orphaned by a document-tab switch.** A canvas draft is context
//!   global, so switching documents mid-draft is a separate and unaddressed
//!   question; nothing here exercises it.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, list, shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The line the Escape ladder writes, and the field naming which rung took it.
const ESCAPE_EVENT: &str = "canvas-escape"; // ui-text-exempt: a trace event name, never displayed
/// The rung this check is about: the text draft, settled rather than discarded.
const SETTLED: &str = "SettledTextDraft"; // ui-text-exempt: a trace field value, never displayed
/// The engine's own line for text arriving on the page.
const COMMIT_EVENT: &str = "add-text"; // ui-text-exempt: a trace event name, never displayed
/// The line that says an undo actually changed the document.
const UNDO_APPLIED_EVENT: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct EscapeCommitsATextDraft;

impl Check for EscapeCommitsATextDraft {
    fn name(&self) -> &'static str {
        "escape_commits_a_text_draft"
    }

    fn defect(&self) -> &'static str {
        "Escape on a text draft throws away what was typed — minutes of work gone with no undo \
         to recover it, because a draft never reached the undo stack in the first place"
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
            "input is disabled (--no-input). This check clicks the ribbon, clicks the page, \
             types on the real keyboard and presses Escape. Reported as SKIPPED rather than \
             passed.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("escape_commits_text.trace.txt"));
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
        return Err(Error::new(
            "clicking Edit > Add text armed no Add-mode text tool, so there is no draft for \
             Escape to settle and nothing below would mean anything.",
        ));
    }

    // --- B: place the caret -------------------------------------------------
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
    let frame = session.frame()?;
    let at = frame.to_screen(mapping.doc_to_window(target)?);
    driver.click_at(at)?;
    session.settle(14);
    if !session
        .trace()?
        .events("text-edit-caret")
        .any(|l| l.get("kind") == Some("Add"))
    {
        return Err(Error::new(
            "the click on the page started no Add draft, so there is nothing for Escape to \
             settle. `checks::add_text` is the check that covers this step failing.",
        ));
    }

    // --- C: type, for real --------------------------------------------------
    //
    // Two keys already in `sys::vk`. WHAT is typed does not matter here — the
    // assertion is that a draft with characters in it survived Escape, not
    // what it says.
    for key in [vk::F, vk::DIGIT_2] {
        driver.press(key)?;
        session.settle(10);
    }
    let grew = session
        .trace()?
        .events("text-edit-typing")
        .filter_map(|l| l.get("len"))
        .filter_map(|v| v.parse::<usize>().ok())
        .any(|n| n > 0);
    if !grew {
        return Err(Error::new(
            "two real keystrokes reached no draft, so an Escape below would have nothing to \
             settle and a pass would measure nothing. `checks::add_text` owns this failure.",
        ));
    }
    report.note("a draft with characters in it is in flight");

    // Counted BEFORE Escape rather than asserted absolutely afterwards. Nothing
    // above commits today, so the two readings agree on this build — but a
    // later phase inserted above would silently turn the absolute test into one
    // that passes on a build where Escape does nothing, and that is the exact
    // failure this check exists to catch.
    let commits_before = session.trace()?.events(COMMIT_EVENT).count();
    let undos_before = session.trace()?.events(UNDO_APPLIED_EVENT).count();

    // --- D: ★★★ ESCAPE, WHICH MUST WRITE -----------------------------------
    driver.press(vk::ESCAPE)?;
    session.settle(24);
    let trace = session.trace()?;

    let rung = trace
        .events(ESCAPE_EVENT)
        .any(|l| l.get("outcome") == Some(SETTLED));
    let committed = trace.events(COMMIT_EVENT).count() > commits_before;

    // Both are read before either is judged, because WHICH of them is missing
    // names a different defect and sends the reader to a different file.
    if !committed {
        let outcomes: Vec<&str> = trace
            .events(ESCAPE_EVENT)
            .filter_map(|l| l.get("outcome"))
            .collect();
        return Ok(Some(format!(
            "★★★ ESCAPE THREW THE DRAFT AWAY. Characters were typed and no `{COMMIT_EVENT}` \
             ever reached the engine, so what the operator wrote is gone — and gone with no \
             recovery, because a draft lives in `egui::Memory` and never reaches the undo \
             stack. That is the exact loss this behaviour was changed to prevent: an Escape \
             pressed by accident must cost one `Ctrl+Z`, not the typing. Escape outcomes \
             traced this run: {}. Trace: {}.",
            list_str(&outcomes),
            session.trace_path().display()
        )));
    }
    if !rung {
        let outcomes: Vec<&str> = trace
            .events(ESCAPE_EVENT)
            .filter_map(|l| l.get("outcome"))
            .collect();
        return Ok(Some(format!(
            "the draft WAS committed, but no `{ESCAPE_EVENT} outcome={SETTLED}` line says the \
             Escape ladder is what committed it — so this check cannot tell Escape's rung from \
             some other exit settling the draft on the same frame, and the next session reading \
             a pass here would be reading one that measures the wrong thing. Either the rung \
             stopped publishing itself or a different exit is taking the key. Escape outcomes \
             traced: {}. Trace: {}.",
            list_str(&outcomes),
            session.trace_path().display()
        )));
    }
    report.note("★★★ Escape settled the draft and the text reached the engine");

    // --- E: the recovery the ruling rests on --------------------------------
    //
    // ★★ Not a bonus arm. The operator's argument is that a commit is
    // recoverable and a discard is not; a build that committed on Escape and
    // could not undo it would have swapped an unrecoverable loss for an
    // unrecoverable gain and satisfied phase D completely.
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(24);
    if session.trace()?.events(UNDO_APPLIED_EVENT).count() <= undos_before {
        return Ok(Some(format!(
            "★★ Escape committed the text and `Ctrl+Z` did not take it back: no \
             `{UNDO_APPLIED_EVENT}`. The whole case for committing rather than discarding is \
             that the mistake it can cause is the cheap one — an accidental commit costs one \
             undo, an accidental discard costs the typing. Without the undo this is not the \
             safer behaviour, it is a different unrecoverable one. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★ …and `Ctrl+Z` took it back, which is what makes committing the safe default");
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
