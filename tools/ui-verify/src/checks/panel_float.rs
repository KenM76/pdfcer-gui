//! `panels_float_close_and_dock` — **a panel tears out into a real OS
//! window, comes back where it came from, and can be closed.**
//!
//! # What this is for
//!
//! `OPERATOR_REQUESTS.md` O126, the operator, 2026-09-04:
//!
//! > *"you understand that there are options to float, close, and dock those
//! > panels … No shortcuts or lazy half-implementation."*
//!
//! The unit tests in `egui_shell::dock::float` prove the **state machine**:
//! float records the home, docking rebuilds it, a stale home still lands, a
//! close removes the entry. Every one of them is headless and none of them
//! opens a window. This check answers the question they structurally cannot:
//! **did the platform actually give us a window, and is the panel's body in
//! it?**
//!
//! # ★★★ The oracle, and why a `ui-rect` is not enough
//!
//! Two lines, together:
//!
//! | line | answers |
//! |---|---|
//! | `viewport-inner` | *a child OS window exists* — the fact only egui and the window manager know. A screenshot of the application window cannot see it, because a real window is **absent** from that capture and so is a panel that failed to draw. |
//! | `ui-rect … viewport=<id>` | *the panel's body drew INSIDE that window*. The application enters `diag::ViewportScope` from the float body, so every rect the panel publishes carries the viewport it was drawn in. |
//!
//! ★★ **The pairing is the point, and it is the lesson this project already
//! paid for.** `crate::checks::dialog_windows` asserts `viewport-inner` alone
//! and its own header records why that was enough there: it was asking
//! whether a dialog was a window at all. Here the failure that matters is
//! different — a window that opens and is **empty**, because
//! `Dock::show_floating` was called and the body callback was not wired, or
//! because the viewport's root `Ui` was never painted. A window with no
//! content publishes `viewport-inner` perfectly happily.
//!
//! ⇒ So this check requires a rect **tagged with the float's own viewport**.
//! `D:/dev/rag/egui/a_child_viewports_ui_rects_are_relative_to_ITS_origin_so_a_harness_aims_hundreds_of_points_away.md`
//! is why the tag exists at all, and it is what makes the assertion
//! expressible.
//!
//! # ★★ Why no pointer is needed
//!
//! The three verbs act on *the panel the operator right-clicked*, and a
//! harness has no pointer. `PDFCER_DIAG_INVOKE` therefore carries an
//! **operand**: `view.panel_float@view.panel_layers`. That widening is
//! documented at the seam in `crate::app::frame`, and it is the same finding
//! as the comma list before it — *a seam that can express less than the
//! interface can leaves part of the interface unverifiable*.
//!
//! It means this check takes no cursor and is safe to run on a machine
//! somebody is using, exactly like `dialog_windows`.
//!
//! # What is deliberately NOT covered, so a green run does not imply it
//!
//! * **The tab's context menu.** The rows are `shown_when` a per-tab
//!   condition, and reaching them needs a right-click on a tab plus a click
//!   on a row — two pointer gestures. `menu_rows` is the check that would
//!   grow to cover it; until then the *menu* is verified by
//!   `shell::menus::tests::each_menu_holds_exactly_the_documented_items` and
//!   the *act* is verified here.
//! * **Dragging or resizing the window.** The position and size round trip is
//!   `dock::float::a_float_survives_serialization`, and moving a real window
//!   needs the window manager.
//! * **A monitor being unplugged.** Not drivable. The recovery is
//!   `view.dock_all_panels` and a layout reset, and section D drives the
//!   first of those.

use crate::checks::driving::{SHELL_DIAG_ENV, VIEWPORT_INNER_EVENT};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The environment variable that fires commands at start-up, one per frame.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";

/// The panel this check tears out.
///
/// Layers, and the choice is not arbitrary: it is mounted by default in every
/// mode, it has a body that draws something with no gesture first, and it is
/// the panel the other two features in O126 are about — so a failure here and
/// a failure in the search check point at the same surface.
const PANEL: &str = "view.panel_layers";

/// See the module documentation.
pub struct PanelsFloatCloseAndDock;

impl Check for PanelsFloatCloseAndDock {
    fn name(&self) -> &'static str {
        "panels_float_close_and_dock"
    }

    fn defect(&self) -> &'static str {
        "a panel floated out of the dock does not open in its own OS window, or opens in one \
         that is empty. The operator asked for float, close and dock on every panel; a window \
         with no panel in it is the failure that every headless test and every rect trace \
         would still call a success"
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

/// One scripted run: launch, fire `invoke`, settle, hand back the trace.
fn run_once(ctx: &CheckContext, tag: &str, invoke: &str) -> Result<crate::trace::Trace> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. The Layers panel draws nothing without a document, so a run with no file \
             would produce an empty window that looks exactly like the defect — an absence \
             proving nothing.",
        )
    })?;
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("panel-float-{tag}.trace.txt")));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env.push(((*INVOKE_ENV).to_owned(), invoke.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    // Generous: a child viewport is created DURING a frame and the window
    // manager settles it over several more. `dialog_windows` uses 40 for the
    // same reason and records that a settle tuned to the fastest surface
    // reports "no window" for one that was a frame from having it.
    session.settle(60);
    session.trace()
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let mut failures: Vec<String> = Vec::new();

    // -----------------------------------------------------------------
    // A. Float it — a real OS window, with the panel's body inside it.
    // -----------------------------------------------------------------
    let trace = run_once(ctx, "float", &format!("view.panel_float@{PANEL}"))?;
    report.artifact(ctx.out("panel-float-float.trace.txt"));

    let moved = trace
        .events("panel-float")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if !moved.contains("moved=true") {
        failures.push(format!(
            "the float command did not move the panel (trace said {moved:?}). Either the \
             operand did not reach `dock_menu_panel` or `DockLayout::float` declined"
        ));
    } else {
        report.note(format!("★ the panel floated: {moved}"));
    }

    // The window itself. `viewport-inner` is emitted by every child viewport,
    // including a dialog's, so this run must not open one — it does not: the
    // only command fired is the float.
    let window = trace.events(VIEWPORT_INNER_EVENT).last().cloned();
    match &window {
        Some(l) => {
            report.note(format!("★ a child OS window exists: {}", l.raw));
        }
        None => failures.push(format!(
            "no `{VIEWPORT_INNER_EVENT}` after floating. The panel left the dock and no window \
             opened for it, which is the panel VANISHING — the exact state \
             `DockFrameReport::floats_undrawn` exists to make detectable and the exact class of \
             defect that shipped three unreachable panels on 2026-08-10"
        )),
    }

    // ★★ And the body drew INSIDE it. A rect tagged with a viewport is the
    // only evidence that separates "a window opened" from "a window opened
    // with the panel in it".
    let inside: Vec<String> = trace
        .events("ui-rect")
        .filter(|l| l.raw.contains("viewport="))
        .map(|l| l.raw.clone())
        .collect();
    if inside.is_empty() {
        failures.push(
            "the float window published no viewport-tagged `ui-rect`, so nothing is known to \
             have been drawn in it. A window with no content publishes `viewport-inner` \
             perfectly happily, which is why this check requires both lines and not one"
                .to_owned(),
        );
    } else {
        report.note(format!(
            "★ {} region(s) drew inside the float window, e.g. {}",
            inside.len(),
            inside[0]
        ));
    }

    // -----------------------------------------------------------------
    // B. Dock it back — and the panel is in a stack again.
    // -----------------------------------------------------------------
    let trace = run_once(
        ctx,
        "dock",
        &format!("view.panel_float@{PANEL},view.panel_dock@{PANEL}"),
    )?;
    report.artifact(ctx.out("panel-float-dock.trace.txt"));
    let docked = trace
        .events("panel-dock")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if docked.contains("moved=true") {
        report.note(format!("★ and it docked back: {docked}"));
    } else {
        failures.push(format!(
            "the dock-back command did not move the panel (trace said {docked:?}). The window \
             is still open and the operator has no way to put the panel back"
        ));
    }

    // -----------------------------------------------------------------
    // C. Close it — from the floating state, which is the harder of the two.
    // -----------------------------------------------------------------
    let trace = run_once(
        ctx,
        "close",
        &format!("view.panel_float@{PANEL},view.panel_close@{PANEL}"),
    )?;
    report.artifact(ctx.out("panel-float-close.trace.txt"));
    let closed = trace
        .events("panel-close")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if closed.contains("closed=true") {
        report.note(format!("★ and a floating panel closes: {closed}"));
    } else {
        failures.push(format!(
            "closing a FLOATING panel did not act (trace said {closed:?}). A leaked float entry \
             draws a window every frame for a panel every \"is it open\" query answers no about, \
             so nothing would ever offer to close it again"
        ));
    }

    // -----------------------------------------------------------------
    // D. The recovery command — the answer to a window nobody can reach.
    // -----------------------------------------------------------------
    let trace = run_once(
        ctx,
        "dock-all",
        &format!("view.panel_float@{PANEL},view.dock_all_panels"),
    )?;
    report.artifact(ctx.out("panel-float-dock-all.trace.txt"));
    let all = trace
        .events("panels-dock-all")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if all.contains("docked=1") {
        report.note(format!("★ and Dock all panels recovers it: {all}"));
    } else {
        failures.push(format!(
            "`view.dock_all_panels` recovered nothing (trace said {all:?}). It is the only \
             remedy that does not require the operator to act ON the window, so it is the only \
             one that works when the window is on a monitor that no longer exists"
        ));
    }

    if failures.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "★ {} of the four panel-window properties failed:\n  · {}",
        failures.len(),
        failures.join("\n  · ")
    )))
}
