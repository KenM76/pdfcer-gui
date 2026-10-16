//! `a_dock_right_click_hides_and_shows_any_panel` — right-clicking a dock tab
//! lists every panel with a show/hide tick; unticking one closes it, and the
//! same list on the tab bar's empty space brings it back (O306). See
//! `docs/modules/ui-verify/checks/panel_toggles.md`.

use super::dimdrive::{Fixture, press, run_on, ui_rect_event};
use super::driving::{declared, declared_names, declared_since, list, live_names};
use crate::checks::{Check, CheckContext};
use crate::coords::{PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const FIXTURE: Fixture = Fixture {
    file: "a1-titleblock.pdf",
    method: "It is checked in under fixtures/.",
    page: PageGeometry {
        width_pt: 2383.94,
        height_pt: 1683.78,
    },
};

const TAB: &str = "dock.tab."; // ui-text-exempt: a trace region prefix, never displayed
const ROW: &str = "dock.menu.panel."; // ui-text-exempt: a trace region prefix, never displayed
const BLANK: &str = ".tabbar.blank"; // ui-text-exempt: a trace region suffix, never displayed

/// See the module documentation.
pub struct ADockRightClickHidesAndShowsAnyPanel;

impl Check for ADockRightClickHidesAndShowsAnyPanel {
    fn name(&self) -> &'static str {
        "a_dock_right_click_hides_and_shows_any_panel"
    }

    fn defect(&self) -> &'static str {
        "A panel the operator closed can be brought back only from the ribbon: right-clicking \
         a dock tab or the tab bar's empty space lists no panels, or the list does not hide \
         or show the one picked"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(
            self,
            ctx,
            &FIXTURE,
            "view.reset_layout",
            "panel-toggles",
            drive,
        )
    }
}

/// Right-click `at`, then the panel rows the menu drew after it.
fn menu_rows(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    at: &str,
) -> Result<Option<Vec<String>>> {
    let ui_rect = ui_rect_event(ctx)?;
    let Some(r) = declared(&session.trace()?, ui_rect, at) else {
        return Ok(None);
    };
    let mark = session.trace()?.mark();
    pointer.right_click(session, WindowPoint::centre_of(r))?;
    session.settle(30);
    let trace = session.trace()?;
    Ok(Some(
        declared_names(&trace, ui_rect, ROW)
            .into_iter()
            .filter(|n| declared_since(&trace, ui_rect, n, mark).is_some())
            .collect(),
    ))
}

/// Press `row` and wait for `event id=panel`.
fn toggle(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    panel: &str,
    event: &str,
) -> Result<bool> {
    let mark = session.trace()?.mark();
    press(ctx, session, pointer, &format!("{ROW}{panel}"))?;
    session.settle(30);
    Ok(session
        .trace()?
        .last_after(event, mark)
        .is_some_and(|l| l.get("id") == Some(panel)))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    session.settle(30);
    let ui_rect = ui_rect_event(ctx)?;
    let tabs = live_names(&session.trace()?, ui_rect, TAB);
    let Some(tab) = tabs.last().cloned() else {
        return Ok(Some(format!(
            "no dock tab is on screen after Reset layout; nothing to right-click. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let panel = tab[TAB.len()..].to_owned();
    report.note(format!("tabs on screen: {}", list(&tabs)));

    let Some(rows) = menu_rows(ctx, session, pointer, &tab)? else {
        return Ok(Some(format!(
            "`{tab}` vanished before it could be right-clicked."
        )));
    };
    report.note(format!("tab menu rows: {}", list(&rows)));
    if rows.len() < 2 || !rows.iter().any(|r| r[ROW.len()..] == panel) {
        return Ok(Some(format!(
            "★ right-clicking `{tab}` drew panel rows {}: expected every registered panel, \
             including `{ROW}{panel}`.",
            list(&rows)
        )));
    }
    if !toggle(ctx, session, pointer, &panel, "panel-closed")? {
        return Ok(Some(format!(
            "★ unticking `{ROW}{panel}` traced no `panel-closed id={panel}`."
        )));
    }

    let blanks: Vec<String> = live_names(&session.trace()?, ui_rect, "dock.")
        .into_iter()
        .filter(|n| n.ends_with(BLANK))
        .collect();
    let Some(blank) = blanks.first() else {
        return Ok(Some(format!(
            "★ no dock tab bar declared its empty space (`*{BLANK}`) once `{panel}` was hidden."
        )));
    };
    let Some(rows) = menu_rows(ctx, session, pointer, blank)? else {
        return Ok(Some(format!(
            "`{blank}` vanished before it could be right-clicked."
        )));
    };
    report.note(format!("blank menu rows: {}", list(&rows)));
    if !rows.iter().any(|r| r[ROW.len()..] == panel) {
        return Ok(Some(format!(
            "★★ right-clicking `{blank}` drew panel rows {}, without the hidden `{panel}`.",
            list(&rows)
        )));
    }
    if !toggle(ctx, session, pointer, &panel, "panel-shown")? {
        return Ok(Some(format!(
            "★★ ticking `{ROW}{panel}` on the blank-bar menu traced no `panel-shown id={panel}`."
        )));
    }
    Ok(None)
}
