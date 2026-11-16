//! `the_markup_tab_arms_the_text_tools` — in Edit mode, Markup ▸ Content's
//! Add text arms the caret in Add mode (`text-edit-tool tool=TextEdit(Add)`,
//! the line Edit ▸ Add text's arm emits) and draws pressed while it is armed; a
//! second press retires it. Its Paragraphs flips the one paragraph switch
//! (`open-paragraphs enabled=…`) both ways and draws pressed exactly when on.
//!
//! `the_markup_text_tools_are_absent_in_review` — in Review the Markup tab
//! draws its Notes group and no Content group: page text is not authorable
//! there, so its controls are absent rather than greyed (R9).
//!
//! Oracle: the app trace's `text-edit-tool` and `open-paragraphs` lines, the
//! shell trace's `ribbon-item-selected`, and the declared `ribbon.group.*`
//! regions.

use super::dimdrive::{press, press_on_tab, run, ui_rect_event};
use crate::checks::driving::{TAB_EVENT, declared_names, list, shell_trace};
use crate::checks::{Check, CheckContext};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const TAB: &str = "ribbon.tab.markup";
const ADD_TEXT: &str = "ribbon.item.markup.add_text";
const PARAGRAPHS: &str = "ribbon.item.markup.paragraphs";
// ui-text-exempt: trace event names and command ids, never displayed.
const TOOL: &str = "text-edit-tool";
const SWITCH: &str = "open-paragraphs";
const SELECTED: &str = "ribbon-item-selected";
const DECLINED: &str = "command-declined";

/// See the module documentation.
pub struct TheMarkupTabArmsTheTextTools;
/// See the module documentation.
pub struct TheMarkupTextToolsAreAbsentInReview;

impl Check for TheMarkupTabArmsTheTextTools {
    fn name(&self) -> &'static str {
        "the_markup_tab_arms_the_text_tools"
    }

    fn defect(&self) -> &'static str {
        "Markup > Add text or Markup > Paragraphs is missing, arms a different tool, or does not \
         show pressed while it is on"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, "mode.edit", "markup-text-tools", arms)
    }
}

impl Check for TheMarkupTextToolsAreAbsentInReview {
    fn name(&self) -> &'static str {
        "the_markup_text_tools_are_absent_in_review"
    }

    fn defect(&self) -> &'static str {
        "Review draws the page-text controls on the Markup tab, where page content cannot be \
         edited"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run(self, ctx, "mode.review", "markup-text-review", absent)
    }
}

/// The last `ribbon-item-selected` for `id` after `mark`, as drawn pressed.
fn pressed_after(session: &Session, id: &str, mark: usize) -> Result<Option<bool>> {
    Ok(shell_trace(session)?
        .events(SELECTED)
        .filter(|l| l.lineno > mark && l.get("id") == Some(id))
        .last()
        .map(|l| l.get("selected") == Some("1")))
}

fn arms(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let (app, shell) = (session.trace()?.mark(), shell_trace(session)?.mark());
    press_on_tab(ctx, session, pointer, ADD_TEXT, TAB)?;
    let trace = session.trace()?;
    let tool = trace.last_after(TOOL, app).and_then(|l| l.get("tool"));
    report.note(format!("Markup > Add text traced tool={tool:?}"));
    if tool != Some("TextEdit(Add)") {
        let declined = trace.last_after(DECLINED, app).map(|l| l.raw.clone());
        return Ok(Some(format!(
            "Markup > Add text traced `{TOOL} tool={tool:?}`, not `TextEdit(Add)`: the press \
             did not reach `arm_text_edit` with the Add kind (a missing `markup.add_text` arm \
             in `dispatch::text` traces nothing; a mode refusal traces {declined:?}). Trace: \
             {path}."
        )));
    }
    if pressed_after(session, "markup.add_text", shell)? != Some(true) {
        return Ok(Some(format!(
            "Add text armed and `markup.add_text` was not drawn pressed: `conditions::armed` \
             publishes no selected condition for it. Trace: {path}."
        )));
    }
    let (app, shell) = (session.trace()?.mark(), shell_trace(session)?.mark());
    press(ctx, session, pointer, ADD_TEXT)?;
    let tool = session
        .trace()?
        .last_after(TOOL, app)
        .map(|l| l.raw.clone());
    if !tool.as_deref().is_some_and(|t| t.contains("tool=Select")) {
        return Ok(Some(format!(
            "a second press on the armed Add text traced {tool:?}, not `tool=Select`: the \
             control does not retire the tool it armed. Trace: {path}."
        )));
    }
    if pressed_after(session, "markup.add_text", shell)? != Some(false) {
        return Ok(Some(format!(
            "Add text was retired and `markup.add_text` still draws pressed. Trace: {path}."
        )));
    }
    report.note("a second press retired the caret and released the control");
    paragraphs(ctx, report, session, pointer)
}

/// Press Paragraphs twice; each press must flip the switch and the drawn state.
fn paragraphs(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    let mut seen = Vec::new();
    for round in 1..=2 {
        let (app, shell) = (session.trace()?.mark(), shell_trace(session)?.mark());
        press(ctx, session, pointer, PARAGRAPHS)?;
        let Some(on) = session
            .trace()?
            .last_after(SWITCH, app)
            .and_then(|l| l.get("enabled").map(|v| v == "true"))
        else {
            return Ok(Some(format!(
                "press {round} on Markup > Paragraphs traced no `{SWITCH}` line: the press \
                 did not reach `dispatch::text::paragraphs`. Trace: {path}."
            )));
        };
        let drawn = pressed_after(session, "markup.paragraphs", shell)?;
        report.note(format!(
            "press {round}: enabled={on}, drawn pressed {drawn:?}"
        ));
        if drawn != Some(on) {
            return Ok(Some(format!(
                "press {round} set paragraphs enabled={on} and `markup.paragraphs` drew \
                 {drawn:?} after it: the control does not show the switch it flips. Trace: \
                 {path}."
            )));
        }
        seen.push(on);
    }
    Ok((seen[0] == seen[1]).then(|| {
        format!(
            "two presses on Markup > Paragraphs both set enabled={}: the control does not \
             toggle. Trace: {path}.",
            seen[0]
        )
    }))
}

fn absent(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    press(ctx, session, pointer, TAB)?;
    if !shell_trace(session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("markup"))
    {
        return Ok(Some(format!(
            "the press on the Markup tab activated no tab, so the absence below would mean \
             nothing. Trace: {path}."
        )));
    }
    let groups = declared_names(
        &session.trace()?,
        ui_rect_event(ctx)?,
        "ribbon.group.markup.",
    );
    report.note(format!(
        "Markup groups declared in Review: {}",
        list(&groups)
    ));
    if !groups
        .iter()
        .any(|g| g.starts_with("ribbon.group.markup.notes"))
    {
        return Ok(Some(format!(
            "Review's Markup tab declared no Notes group, the witness that the tab drew. \
             Groups: {}. Trace: {path}.",
            list(&groups)
        )));
    }
    Ok(groups
        .iter()
        .any(|g| g.starts_with("ribbon.group.markup.content"))
        .then(|| {
            format!(
                "Review drew Markup > Content although neither of its items may author \
                 there: their `shown_when(mode.edit_content)` is missing. Trace: {path}."
            )
        }))
}
