//! `panel_tabs_can_be_rearranged` — dragging a panel's tab along its dock
//! strip moves it, marks where it will land while the pointer is down, and
//! does **not** change which panel is on screen.
//!
//! # Why this needs driving
//!
//! `egui_shell::dock::DockLayout::reorder_tab`'s arithmetic has unit tests,
//! including both directions of the boundary-to-index conversion and the
//! active-tab case. **Every one of them passes on a build where a panel tab
//! cannot be dragged at all**, because the arithmetic is a pure function of a
//! layout and two integers, and the gesture is three frame-level facts none of
//! them can reach:
//!
//! 1. the tab's `Button` senses a **drag** and not only a click — `egui`'s
//!    default is clicks alone, which is the state a strip of plain buttons
//!    ships in and looks entirely correct until somebody tries;
//! 2. a drag begun on a tab survives to a release read from **raw pointer
//!    input**, because a drag begun on a tab ends anywhere;
//! 3. the boundary the caret marked and the boundary the release used are the
//!    same decision, resolved once.
//!
//! The crate's own driven tests cover the same three in a headless
//! `egui::Context`. This one covers what they cannot: the real binary, its
//! real arrangement, and a pointer driven through the OS.
//!
//! # ★ Which strip, and why it is not the left one
//!
//! **The left side draws no tab strip in this application.** The operator
//! asked for *"no tabs in the left side bar when the left rail is visible"*,
//! and `egui_shell::dock::Dock::with_rail_reach` delivers it — so the panels
//! reachable from the rail have no tabs to drag. The strip this check drives
//! is therefore whichever one the application actually drew, found by reading
//! the tab bars out of the trace rather than named here. A check that hard-
//! coded a compartment would start reporting a broken feature the day the
//! default arrangement moved a panel.
//!
//! # ★★ The two assertions that are the point
//!
//! **The caret was drawn.** A reorder that commits correctly and marks nothing
//! while the pointer is down has answered the wrong half of the feature — and
//! it is the half no capture taken afterwards can see, because the caret
//! exists only during the gesture. It is observable because the dock publishes
//! it as a region and the application's `ui-rect` channel is a change log, so
//! a completed drag leaves the declaration behind.
//!
//! **The panel on screen did not change.** Rearranging tabs is tidying, not
//! navigation. The failure — a stack's active tab tracked as an *index* across
//! the move — leaves the strip in the right order with a different panel
//! showing, and nothing errors.
//!
//! # What a passing run does NOT prove
//!
//! * That the caret was drawn in the operator's accent colour, or that it is
//!   visible against the strip behind it. This reads its rectangle. A pixel
//!   oracle is the instrument for the other question and this is not it.
//! * That a tab can be dragged to a *different* compartment. It cannot yet,
//!   deliberately: there is no drop grammar behind such a gesture, so the dock
//!   proposes nothing over a strip the drag did not start in.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, declared_since, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The prefix of the per-panel tab regions.
const TAB: &str = "dock.tab.";
/// The prefix of the per-panel body regions — what is actually on screen.
const BODY: &str = "dock.body.";
/// The suffix a stack's tab bar region carries.
const TABBAR: &str = ".tabbar";
/// The suffix the insertion caret's region carries.
const CARET: &str = ".caret";
/// The trace line a committed panel reorder produces.
const REORDER: &str = "panel-reorder";
/// The mode whose arrangement has the most tabs in one stack to work with.
const MODE: &str = "edit";

/// See the module documentation.
pub struct PanelTabsCanBeRearranged;

impl Check for PanelTabsCanBeRearranged {
    fn name(&self) -> &'static str {
        "panel_tabs_can_be_rearranged"
    }

    fn defect(&self) -> &'static str {
        "a panel's dock tab cannot be dragged to a new position — the stack's order is whatever \
         the mode's default said and stays that way — or it can, and doing it shows no marker \
         of where the tab will land, or switches the operator to a different panel"
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

/// One tab bar and the tabs drawn inside it, left to right.
struct Strip {
    /// The region name of the bar itself.
    bar: String,
    /// The bar's own rectangle.
    rect: LRect,
    /// `(region name, rect)` per tab, in drawn order.
    tabs: Vec<(String, LRect)>,
}

impl Strip {
    /// The caret region this strip would publish — the bar's name with its
    /// suffix swapped, because both are built from the same compartment
    /// address and deriving one from the other is what keeps them agreeing.
    fn caret(&self) -> String {
        format!("{}{CARET}", self.bar.trim_end_matches(TABBAR))
    }

    /// The panel a tab region names.
    fn panel(region: &str) -> &str {
        region.trim_start_matches(TAB)
    }
}

/// **Find a tab bar with at least two tabs drawn in it.**
///
/// By containment rather than by name: a tab region is named for its panel and
/// carries no compartment, which is deliberate — a panel keeps its name when
/// the operator moves it — so the only thing that says which bar a tab is in
/// is where it was drawn.
fn find_strip(trace: &crate::trace::Trace, ui_rect: &str) -> Option<Strip> {
    let tabs: Vec<(String, LRect)> = declared_names(trace, ui_rect, TAB)
        .into_iter()
        .filter_map(|name| {
            let rect = declared(trace, ui_rect, &name)?;
            rect.is_substantial().then_some((name, rect))
        })
        .collect();
    for bar in declared_names(trace, ui_rect, "dock.") {
        if !bar.ends_with(TABBAR) {
            continue;
        }
        let Some(rect) = declared(trace, ui_rect, &bar) else {
            continue;
        };
        let mut mine: Vec<(String, LRect)> = tabs
            .iter()
            .filter(|(_, t)| rect.contains_rect(*t))
            .cloned()
            .collect();
        if mine.len() < 2 {
            continue;
        }
        mine.sort_by(|a, b| a.1.min.x.total_cmp(&b.1.min.x));
        return Some(Strip {
            bar,
            rect,
            tabs: mine,
        });
    }
    None
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
            "input is disabled (--no-input). This check drags a panel tab along its strip. \
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("panel_tab_reorder.trace.txt"));
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

    // --- 1: a mode with a tabbed stack in it -------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(strip) = find_strip(&trace, ui_rect) else {
        return Err(Error::new(format!(
            "no tab bar with two drawn tabs in it, so there is nothing to rearrange and this \
             check has learned nothing about dragging. Bars declared: {}. Tabs declared: {}. \
             The left side draws none by design — see this module's header.",
            list(
                &declared_names(&trace, ui_rect, "dock.")
                    .into_iter()
                    .filter(|n| n.ends_with(TABBAR))
                    .collect::<Vec<_>>()
            ),
            list(&declared_names(&trace, ui_rect, TAB)),
        )));
    };
    report.note(format!(
        "{} holds {} tab(s): {}",
        strip.bar,
        strip.tabs.len(),
        list(
            &strip
                .tabs
                .iter()
                .map(|(n, _)| n.clone())
                .collect::<Vec<_>>()
        ),
    ));

    // ★ The panel on screen, before anything is dragged. Read from the BODY
    // regions rather than assumed to be the first tab: the arrangement is the
    // application's and a saved layout may have raised any tab in the stack.
    let bodies_before = declared_names(&trace, ui_rect, BODY);

    // --- 2: drag the SECOND tab left, past the middle of the first ---------
    //
    // Leftwards and across the first tab, because the first tab is the stack's
    // active one in every default arrangement — so this is the move that makes
    // the active tab change index without changing panel, which is the failure
    // the third assertion is about. Dragging the active tab itself would leave
    // a build that tracks an index looking correct.
    let (moving, from_rect) = strip.tabs[1].clone();
    let (first, onto_rect) = strip.tabs[0].clone();
    let panel = Strip::panel(&moving).to_owned();
    let frame = session.frame()?;
    let from = frame.declared_center(from_rect);
    // A quarter across the first tab: past its centre, which is where the
    // boundary flips, and not so close to the centre that the harness's
    // reading of an `f32` rectangle could land on the wrong side of it.
    let onto = frame.declared_at(onto_rect, 0.25, 0.5);
    report.note(format!(
        "dragging `{moving}` from ({}, {}) past the middle of `{first}` at ({}, {})",
        from.x(),
        from.y(),
        onto.x(),
        onto.y()
    ));
    let mark = trace.mark();
    driver.drag(from, onto)?;
    session.settle(40);

    // --- 3: it moved -------------------------------------------------------
    let trace = session.trace()?;
    let moved = trace
        .events(REORDER)
        .filter(|l| l.lineno > mark)
        .last()
        .cloned();
    let Some(moved) = moved else {
        return Ok(Some(format!(
            "`{moving}` was dragged past the middle of `{first}` and no `{REORDER}` line was \
             traced. The tab does not sense a drag, or the release never reached the dock — \
             which is the state a strip of plain `egui::Button`s ships in, because a `Button` \
             senses clicks and nothing else."
        )));
    };
    report.note(format!("the strip took the drop: `{}`", moved.raw));
    if moved.get("panel") != Some(panel.as_str()) {
        return Ok(Some(format!(
            "`{panel}` was dragged and the dock reported `{}` moving instead. Line: `{}`.",
            moved.get("panel").unwrap_or("?"),
            moved.raw
        )));
    }
    if moved.get("tab") != Some("0") {
        return Ok(Some(format!(
            "`{panel}` was dropped past the middle of the FIRST tab, which is the boundary \
             before it, so it must land at index 0 — and the dock reports index {}. Line: \
             `{}`.",
            moved.get("tab").unwrap_or("?"),
            moved.raw
        )));
    }

    // --- 4: ★ and it said where it was going while it went -----------------
    let caret = strip.caret();
    let Some(drawn) = declared_since(&trace, ui_rect, &caret, mark) else {
        return Ok(Some(format!(
            "`{panel}` moved and no `{caret}` region was declared during the drag, so the \
             operator was given no marker of where the tab would land. The reorder is the half \
             of this feature that can be checked afterwards; the marker is the half they can \
             see, and it is the half that was asked for."
        )));
    };
    if !drawn.is_substantial() {
        return Ok(Some(format!(
            "`{caret}` was declared at {drawn:?}, which has no usable area — a marker nobody \
             could see. A caret is published at the stroke's own width, so a degenerate one \
             means it was positioned but never given a height, i.e. drawn against an empty \
             strip rectangle."
        )));
    }
    if !strip.rect.contains_rect(drawn) {
        return Ok(Some(format!(
            "`{caret}` was drawn at {drawn:?}, outside its own tab bar at {:?}. A marker past \
             the end of the strip names a boundary the operator cannot relate to a tab.",
            strip.rect
        )));
    }
    report.note(format!("the caret marked the boundary at {drawn:?}"));

    // --- 5: ★★ and the operator is still looking at the same panel ---------
    let bodies_after = declared_names(&trace, ui_rect, BODY);
    if bodies_after != bodies_before {
        return Ok(Some(format!(
            "before the drag the dock was drawing {}; after it, {}. Rearranging tabs switched \
             the operator to a different panel, which means a stack's active tab is tracked as \
             an INDEX and the move slid a different panel under it.",
            list(&bodies_before),
            list(&bodies_after),
        )));
    }
    report.note("the same panels are still on screen");
    Ok(None)
}
