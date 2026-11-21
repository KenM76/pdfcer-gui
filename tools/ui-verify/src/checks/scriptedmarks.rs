//! `checks::scriptedmarks` — the scripted pointer's marking, picking and
//! Comments-census helpers, re-exported through `checks::scripteddrive`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/scripteddrive.md`.

use crate::checks::comments_census::Census;
use crate::checks::driving::{declared_in, declared_names, list};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;

/// The canvas object menu's redact row, its siblings, and the menu's own line.
const REDACT_ROW: &str = "menu.item.canvas.object.edit.redact_selection"; // ui-text-exempt: a trace region name, never displayed

const REDACT_ROW_PREFIX: &str = "menu.item.canvas.object."; // ui-text-exempt: a trace region name, never displayed

const CANVAS_MENU_EVENT: &str = "canvas-menu"; // ui-text-exempt: a trace event name, never displayed

const OBJECT_CONTEXT: &str = "canvas.object"; // ui-text-exempt: a trace token, never displayed

/// Right-click `at`, press the object menu's redact row, and read what it
/// requested: the scripted twin of `redact_menu::mark_through_the_menu`.
/// `Ok(Err(_))` is a failure sentence naming `what` was right-clicked.
pub(super) fn mark_through_the_menu(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    at: WindowPoint,
    what: &str,
) -> Result<std::result::Result<crate::checks::redact_menu::Marked, String>> {
    pointer.right_click(session, at)?;
    session.settle(35);
    let trace = session.trace()?;
    let Some(menu) = trace.events(CANVAS_MENU_EVENT).last() else {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} RESOLVED NO MENU AT ALL: no `{CANVAS_MENU_EVENT}` line \
             after a secondary click on the page. Trace: {}",
            session.trace_path().display()
        )));
    };
    let context = menu.get("context").unwrap_or_default();
    if context != OBJECT_CONTEXT {
        return Ok(Err(format!(
            "THE RIGHT-CLICK ON {what} RESOLVED `{context}`, NOT `{OBJECT_CONTEXT}`: `{}`. The \
             secondary hit test lost the selection the left click made.",
            menu.raw
        )));
    }
    let Some((row, viewport)) = declared_in(&trace, ui_rect, REDACT_ROW) else {
        return Ok(Err(format!(
            "★★★ THE REDACT ROW IS NOT IN THE CANVAS OBJECT MENU: no `{REDACT_ROW}` region after \
             the menu opened on {what}. Rows it DID publish: {}.",
            list(&declared_names(&trace, ui_rect, REDACT_ROW_PREFIX))
        )));
    };
    if !row.is_substantial() {
        return Ok(Err(format!(
            "`{REDACT_ROW}` was published at {row:?}, which has no usable area."
        )));
    }
    let mark = trace.mark();
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(row))?;
    session.settle(30);
    Ok(read_mark(&session.trace()?, mark, what))
}

/// The `quads=` and `bbox=` of the last redact request after `mark`.
fn read_mark(
    trace: &crate::trace::Trace,
    mark: usize,
    what: &str,
) -> std::result::Result<crate::checks::redact_menu::Marked, String> {
    use crate::checks::redact_menu::{Bounds, Marked, REQUESTED};
    let line = trace.last_after(REQUESTED, mark).ok_or_else(|| {
        format!(
            "★★★ THE ROW WAS PRESSED AND THE VERB DID NOT RUN: no `{REQUESTED}` line after \
             pressing the redact row on {what}. A build whose page filter kept no outline writes \
             `redact-mark-selection-declined … reason=no-bounds` instead."
        )
    })?;
    let bbox = line.get("bbox").and_then(Bounds::parse).ok_or_else(|| {
        format!(
            "★★★ THE VERB RAN AND DID NOT SAY WHAT IT MARKED AS A RECTANGLE: `{}`.",
            line.raw
        )
    })?;
    let quads = line.get_usize("quads").ok_or_else(|| {
        format!(
            "★★★ THE VERB RAN AND DID NOT SAY HOW MANY REGIONS IT BUILT: `{}` carries no usable \
             `quads=` field.",
            line.raw
        )
    })?;
    Ok(Marked { quads, bbox })
}

/// Click `at` until the application resolves it into one measure pick,
/// confirming a promoted derived candidate with a second click: the scripted
/// twin of `picking::resolve_pick`, with the same bound and the same verdicts.
pub fn resolve_pick(
    session: &Session,
    pointer: &ScriptedPointer,
    report: &mut crate::report::CheckReport,
    label: &str,
    at: WindowPoint,
    settle: u32,
) -> Result<std::result::Result<crate::checks::picking::ResolvedPick, String>> {
    use crate::checks::picking::{MAX_CLICKS_PER_PICK, PICK_EVENT, PICK_PROMOTED, ResolvedPick};
    let mut clicks = 0usize;
    loop {
        let before = session.trace()?.events(PICK_EVENT).count();
        clicks += 1;
        pointer.click(session, at)?;
        session.settle(settle);
        let trace = session.trace()?;
        let new: Vec<&crate::trace::TraceLine> = trace.events(PICK_EVENT).skip(before).collect();
        let [line] = new.as_slice() else {
            return Ok(Err(format!(
                "GESTURE: click {clicks} of pick {label} produced {} `{PICK_EVENT}` line(s), not \
                 1. `canvas::measure::click` traces once per click it is handed, so zero means \
                 the click never became a pick and more than one means it was delivered twice. \
                 New lines: {}.",
                new.len(),
                crate::checks::driving::list_str(
                    &new.iter().map(|l| l.raw.as_str()).collect::<Vec<_>>()
                )
            )));
        };
        if line.get("outcome") != Some(PICK_PROMOTED) {
            return Ok(Ok(ResolvedPick {
                line: (*line).clone(),
                clicks,
            }));
        }
        report.note(format!(
            "pick {label}: the application promoted a derived snap candidate rather than \
             committing it (`{}`); clicking the same point again to confirm",
            line.raw
        ));
        if clicks >= MAX_CLICKS_PER_PICK {
            return Ok(Err(format!(
                "GESTURE: the two-click confirm did not converge. Pick {label} was promoted on \
                 click 1 and again on click {clicks} at the same point, so the point the snap \
                 query resolved to moved between two clicks of a stationary pointer. Line: `{}`.",
                line.raw
            )));
        }
    }
}

/// `comments_census::refresh` through any [`crate::input::Click`]: read a
/// census published after `after`, bringing the Comments panel to the front
/// first if it has gone quiet (its dock tab, then Markup > Comments).
pub fn census_refresh(
    session: &Session,
    pointer: &impl crate::input::Click,
    ui_rect: &str,
    after: usize,
    report: &mut crate::report::CheckReport,
) -> Result<Option<Census>> {
    use crate::checks::driving::{ITEM_PREFIX, TAB_EVENT, declared, shell_trace};
    const DOCK_TAB: &str = "dock.tab.markup.comments"; // ui-text-exempt: a trace region name, never displayed
    const COMMAND: &str = "ribbon.item.markup.comments"; // ui-text-exempt: a trace region name, never displayed
    const MARKUP_TAB: &str = "ribbon.tab.markup"; // ui-text-exempt: a trace region name, never displayed
    if let Some(census) = Census::since(&session.trace()?, after)? {
        return Ok(Some(census));
    }
    report.note(
        "the Comments panel published no census, so it is not the front tab of its stack; \
         bringing it forward before reading",
    );
    if let Some(rect) = declared(&session.trace()?, ui_rect, DOCK_TAB) {
        pointer.click_rect(session, rect)?;
        session.settle(14);
        if let Some(census) = Census::since(&session.trace()?, after)? {
            report.note(format!("the panel came forward on a click of `{DOCK_TAB}`"));
            return Ok(Some(census));
        }
    }
    let trace = session.trace()?;
    let Some(tab) = declared(&trace, ui_rect, MARKUP_TAB) else {
        report.note(format!(
            "no `{MARKUP_TAB}` region either. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ));
        return Ok(None);
    };
    pointer.click_rect(session, tab)?;
    session.settle(14);
    if shell_trace(session)?
        .events(TAB_EVENT)
        .all(|l| l.get("tab") != Some("markup"))
    {
        report.note(format!(
            "the click on `{MARKUP_TAB}` produced no `{TAB_EVENT} tab=markup`."
        ));
        return Ok(None);
    }
    let trace = session.trace()?;
    let Some(rect) = declared(&trace, ui_rect, COMMAND) else {
        report.note(format!(
            "the Markup tab is active and declares no `{COMMAND}`. Controls declared: {}.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        ));
        return Ok(None);
    };
    pointer.click_rect(session, rect)?;
    session.settle(18);
    let census = Census::since(&session.trace()?, after)?;
    if census.is_some() {
        report.note("the panel came forward on a press of `markup.comments`");
    }
    Ok(census)
}

/// `comments_census::baseline` through any [`crate::input::Click`]: enter
/// `mode`, bring the Comments panel forward, and require a clean census.
pub fn census_baseline(
    session: &Session,
    pointer: &impl crate::input::Click,
    ui_rect: &str,
    (mode, what, subject): (&str, &str, &str),
    report: &mut crate::report::CheckReport,
) -> Result<Census> {
    let before_click = session.trace()?.mark();
    crate::checks::driving::click_mode_segment(session, pointer, ui_rect, mode)?;
    session.settle(16);
    let anchor = session
        .trace()?
        .events("mode-changed")
        .filter(|l| l.get("to") == Some(mode))
        .last()
        .map_or(before_click, |l| l.lineno);
    let census = census_refresh(session, pointer, ui_rect, anchor, report)?.ok_or_else(|| {
        Error::new(format!(
            "the Comments panel traced no `{}` line for {what} after the switch to `{mode}`, and \
             neither its dock tab nor its ribbon control could raise it, so there is no oracle. \
             A persisted layout can name a different active tab. Trace: {}.",
            crate::checks::comments_census::EVENT,
            session.trace_path().display()
        ))
    })?;
    census.require_a_clean_fixture(what, subject)?;
    report.note(format!(
        "{what}: the Comments panel lists {} annotation(s) — `{}`",
        census.listed, census.raw
    ));
    Ok(census)
}
