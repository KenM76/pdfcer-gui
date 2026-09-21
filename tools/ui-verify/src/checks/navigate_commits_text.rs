//! `navigate_commits_text` — **arming another tool WRITES the draft**, so the
//! typing is on the page before the operator looks for it.
//!
//! # The request
//!
//! > *"when I add or edit text then click to another navigation tool my
//! > changes don't show. If I click elsewhere on the page it works and shows me
//! > my changes."*
//!
//! `OPERATOR_REQUESTS.md` O222.
//!
//! # ★★★ What the operator is actually reporting
//!
//! Not a rendering bug. The text was never committed. A canvas draft lives in
//! `egui::Memory` and is drawn by the caret layer, not by the page; the only
//! thing that turned it into document content was the *next click on the
//! canvas*, because that is where the commit was written. An operator who
//! typed and then reached for the hand tool had a draft, a caret that was no
//! longer armed, and nothing anywhere that would write it — so the sheet showed
//! the text the document had, which is the correct rendering of a document that
//! does not have it yet.
//!
//! The reading matters because it names what the fix had to be. "Redraw after a
//! tool change" would have been a second rendering path for provisional
//! content, which is the thing `R8b` forbids. The draft has to **land**.
//!
//! # ★★ The oracle, and why the tool line is not enough on its own
//!
//! Two lines, and each is worthless without the other.
//!
//! `canvas-tool armed=Hand` says the rail click armed something — which is a
//! precondition, not the subject. An armed canvas and an un-armed one are the
//! same screenshot, so without this line a failure below cannot be told from a
//! click that missed the rail entirely, and the check would report the defect
//! it exists to find while measuring a mis-aimed pointer.
//!
//! `add-text` is the engine's own line for characters reaching the page, and it
//! is the assertion. The behaviour being replaced produces the tool change and
//! **no commit at all** — that is precisely the operator's sentence — so a
//! check that watched only the tool would pass on the broken build.
//!
//! Counted before the gesture rather than asserted absolutely, for the reason
//! [`crate::checks::escape_commits_text`] states: a phase inserted above would
//! silently convert an absolute test into one that passes on a build where
//! arming a tool commits nothing.
//!
//! # The phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | Edit mode, Edit tab, click **Add text** | `text-edit-tool tool=TextEdit(Add)` |
//! | B | click blank paper | `text-edit-caret kind=Add` |
//! | C | type two real characters | `text-edit-typing … len>0` |
//! | D | **click the rail's Hand tool** | `canvas-tool armed=Hand` *and* `add-text` |
//! | E | `Ctrl+Z` | `undo-applied` — the typing is recoverable, not merely gone |
//!
//! Phase E is not a bonus arm. Committing on a tool change is the eager
//! reading of the operator's gesture, and eager is only defensible while it is
//! undoable; a build that wrote the draft and could not take it back would have
//! satisfied phase D completely and still be a build that costs him work.
//!
//! # Why the rail and not the ribbon
//!
//! Because the rail is the surface his sentence describes — it is on screen in
//! every mode, needs no tab change, and is where a hand tool is reached from
//! while editing. Driving the ribbon instead would insert a tab switch between
//! the typing and the tool change, and a tab switch is a second event that
//! could plausibly be what committed the draft.
//!
//! # What it cannot see
//!
//! * **Whether the committed text is what was typed.** Phase D asserts the
//!   commit reached the engine, not its content — the same scope
//!   [`crate::checks::escape_commits_text`] and [`crate::checks::add_text`]
//!   take.
//! * **The other three navigation tools.** Select, Points and Text reach the
//!   canvas through the same single statement in the frame order, and the
//!   commit is not conditioned on which tool was armed; this drives Hand.
//! * **A draft orphaned by a document-tab switch.** A canvas draft is context
//!   global, so switching documents mid-draft remains unaddressed and
//!   unasserted.
//! * **The Edit-mode caret only.** A form field's editor carries text too and
//!   settles by its own path; `checks::form_field` drives that one.

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

/// The rail row this check clicks — `crate::app::rail::region("navigate",
/// "view.tool_hand")` as the running program publishes it.
const HAND_REGION: &str = "rail.navigate.view.tool_hand"; // ui-text-exempt: a declared region name, never displayed
/// The line the sole writer of the armed tool emits on a change.
const TOOL_EVENT: &str = "canvas-tool"; // ui-text-exempt: a trace event name, never displayed
/// The tool the rail row must arm, as `CanvasTool` debug-prints it.
const HAND: &str = "Hand"; // ui-text-exempt: a trace field value, never displayed
/// The engine's own line for text arriving on the page.
const COMMIT_EVENT: &str = "add-text"; // ui-text-exempt: a trace event name, never displayed
/// The line that says an undo actually changed the document.
const UNDO_APPLIED_EVENT: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct ArmingANavigationToolCommitsTheDraft;

impl Check for ArmingANavigationToolCommitsTheDraft {
    fn name(&self) -> &'static str {
        "arming_a_navigation_tool_commits_the_draft"
    }

    fn defect(&self) -> &'static str {
        "text typed on the canvas stays invisible when the operator reaches for another tool — \
         it was never committed, so the sheet is correctly showing a document that does not \
         have it, and the only thing that writes it is a further click on the page"
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
             types on the real keyboard and clicks the rail. Reported as SKIPPED rather than \
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("navigate_commits_text.trace.txt"));
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
            "clicking Edit > Add text armed no Add-mode text tool, so there is no draft to \
             carry across a tool change and nothing below would mean anything.",
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
            "the click on the page started no Add draft, so there is nothing for a tool change \
             to commit. `checks::add_text` is the check that covers this step failing.",
        ));
    }

    // --- C: type, for real --------------------------------------------------
    //
    // WHAT is typed does not matter: the assertion is that a draft with
    // characters in it reached the document across a tool change, not what it
    // says.
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
            "two real keystrokes reached no draft, so the tool change below would have nothing \
             to commit and a pass would measure nothing. `checks::add_text` owns this failure.",
        ));
    }
    report.note("a draft with characters in it is in flight");

    // The rail row is located BEFORE the counts are taken and before the
    // gesture, so a rail that is not on screen is reported as the harness
    // problem it is rather than as a build that failed to commit.
    let trace = session.trace()?;
    let hand = declared(&trace, ui_rect, HAND_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{HAND_REGION}` region on screen in Edit. This check reaches the hand tool \
             through the rail because that is the surface the operator's sentence describes. \
             Navigate rows declared: {}.",
            list(&declared_names(&trace, ui_rect, "rail.navigate."))
        ))
    })?;
    let commits_before = trace.events(COMMIT_EVENT).count();
    let undos_before = trace.events(UNDO_APPLIED_EVENT).count();

    // --- D: ★★★ REACH FOR ANOTHER TOOL, WHICH MUST WRITE --------------------
    driver.click_at(session.frame()?.declared_center(hand))?;
    session.settle(24);
    let trace = session.trace()?;

    let armed = trace
        .events(TOOL_EVENT)
        .any(|l| l.get("armed") == Some(HAND));
    let committed = trace.events(COMMIT_EVENT).count() > commits_before;

    // The precondition is judged first, because a click that armed nothing
    // makes the commit question unanswerable rather than answered "no".
    if !armed {
        let armings: Vec<&str> = trace
            .events(TOOL_EVENT)
            .filter_map(|l| l.get("armed"))
            .collect();
        return Err(Error::new(format!(
            "the click on the rail's hand row armed no hand tool — no `{TOOL_EVENT} \
             armed={HAND}` line — so this run cannot say anything about what a tool change \
             does to a draft. Tools armed this run: {}. Trace: {}.",
            list_str(&armings),
            session.trace_path().display()
        )));
    }
    if !committed {
        return Ok(Some(format!(
            "★★★ THE TYPING NEVER LANDED. The hand tool armed and no `{COMMIT_EVENT}` reached \
             the engine, so the draft is still sitting in memory with its caret un-armed and \
             the page is correctly showing a document that does not contain it. That is his \
             report exactly: the changes do not show until he clicks the page again, because \
             that click is the only thing that writes them. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★★ arming the hand tool wrote the draft to the document");

    // --- E: and it is recoverable -------------------------------------------
    driver.press_chord(&[vk::CONTROL], vk::Z)?;
    session.settle(24);
    if session.trace()?.events(UNDO_APPLIED_EVENT).count() <= undos_before {
        return Ok(Some(format!(
            "★★ the tool change committed the text and `Ctrl+Z` did not take it back: no \
             `{UNDO_APPLIED_EVENT}`. Writing a draft the operator has navigated away from is \
             the eager reading of their gesture, and eager is defensible only while it is \
             undoable. Without the undo this is a build that commits text on a gesture that \
             was not asking for a commit and offers no way out of it. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★ …and `Ctrl+Z` took it back, which is what makes writing it the safe reading");
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
