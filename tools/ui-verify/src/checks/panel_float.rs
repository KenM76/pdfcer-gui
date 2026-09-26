//! `panels_float_close_and_dock` — **a panel tears out into a real OS
//! window, draws its body in it, comes back where it came from, and can be
//! closed.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/panel_float.md`.

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
///
/// ⚠ On every repository fixture this panel's whole body is **one sentence**,
/// because none of them carries an `/OCProperties`. That is fine for what is
/// asserted here — see the module header's last section — but it is the
/// reason the oracle had to become *"the body filled a rectangle"* rather
/// than *"the body published one of its own controls"*.
const PANEL: &str = "view.panel_layers";

/// The command every section fires first, so each launch starts from the
/// mode's default dock rather than from what the previous launch saved.
///
/// See the module header, defect 1. This is the whole of the fix for the
/// `moved=false` / `docked=0` family.
const RESET: &str = "view.reset_layout";

/// **The mode this check must be in before it resets anything**, and the
/// reason it is now stated instead of inherited.
///
/// # ★★★ This check was passing because a DIFFERENT check leaked its mode
///
/// `view.panel_layers` is not in every mode's default dock. Read mounts five
/// panels, Review seven, Edit thirteen — measured off `mode-changed … panels=`
/// — and the Layers panel is an Edit-mode surface. So a float of it can only
/// succeed in a mode whose default dock carries it.
///
///
/// ⇒ **A check that does not state its preconditions is not passing — it is
/// agreeing with whatever ran before it.** The isolation fix did not break
/// this check; it revealed that the check had never been testing what it
/// claimed on its own. That is the price of isolation and it is worth paying
/// once per check that had been coasting.
///
/// ⚠ It is `mode.edit` rather than a click on the mode segment because
/// `PDFCER_DIAG_INVOKE` is this check's whole input channel — the harness has
/// no pointer here, which is the same reason the float command carries an
/// operand at all.
const MODE: &str = "mode.edit";

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

/// One scripted run: launch, fire `view.reset_layout` and then `invoke`,
/// settle, hand back the trace.
///
/// The reset is prepended **here** rather than at each call site so that no
/// future section can be added without it. See the module header, defect 1.
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
    // ★ MODE first, then the reset, then the work. One command per frame, and
    // the order is the precondition chain: the reset must be asked of the mode
    // this check needs, or it resets a dock that was never going to mount the
    // panel. See [`MODE`] for why this is stated rather than inherited.
    spec.env
        .push(((*INVOKE_ENV).to_owned(), format!("{MODE},{RESET},{invoke}")));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    // Generous: a child viewport is created DURING a frame and the window
    // manager settles it over several more. `dialog_windows` uses 40 for the
    // same reason and records that a settle tuned to the fastest surface
    // reports "no window" for one that was a frame from having it.
    //
    // ★ One frame more than before, because the reset now takes the first
    // frame — `PDFCER_DIAG_INVOKE` fires exactly one command per frame, and
    // the ordering is what makes the reset a precondition rather than a race.
    //
    session.settle(80);
    session.trace()
}

/// **The precondition every section shares**: the layout was reset, so what
/// follows is being asked of the mode's default dock and not of whatever the
/// previous launch happened to save.
///
/// Returns the failure sentence when it did not happen. Written as its own
/// function because a section that skipped this and went on to report
/// `moved=false` would be reporting the application broken for the check's
/// own reason — which is exactly what happened for days.
fn reset_landed(trace: &crate::trace::Trace, section: &str) -> Option<String> {
    let line = trace.events("layout-reset").last().map(|l| l.raw.clone());
    match line {
        Some(raw) => {
            if raw.contains("changed=") {
                None
            } else {
                Some(format!(
                    "section {section}: `{RESET}` traced {raw:?}, which does not carry a \
                     `changed=` field. Everything after this point is being asked of an unknown \
                     layout"
                ))
            }
        }
        None => Some(format!(
            "section {section}: `{RESET}` never fired, so this launch inherited whatever the \
             PREVIOUS launch saved to `userdata/layout.ron`. Every verdict downstream of this is \
             about a state the harness put the application into, which is exactly the defect this \
             check was rewritten to stop reporting"
        )),
    }
}

/// The float window's viewport id, as this run's own trace spells it.
///
/// Read rather than hard-coded: the id is a hash of the panel id, so a
/// constant would keep matching after a rename while naming a window that no
/// longer exists.
fn float_viewport(trace: &crate::trace::Trace) -> Option<String> {
    trace
        .events(VIEWPORT_INNER_EVENT)
        .last()
        .and_then(|l| l.get("id").map(str::to_owned))
}

/// Whether the trace retires the region key `viewport-inner:"<id>"`, which is
/// how `diag`'s per-frame census reports a child window that stopped being
/// drawn.
fn window_retired(trace: &crate::trace::Trace, viewport: &str) -> bool {
    let key = format!("viewport-inner:{viewport:?}");
    trace
        .events("ui-rect-gone")
        .any(|l| l.get("name").is_some_and(|n| n == key))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let mut failures: Vec<String> = Vec::new();

    // -----------------------------------------------------------------
    // A. Float it — a real OS window, with the panel's body drawn in it.
    // -----------------------------------------------------------------
    let trace = run_once(ctx, "float", &format!("view.panel_float@{PANEL}"))?;
    report.artifact(ctx.out("panel-float-float.trace.txt"));
    failures.extend(reset_landed(&trace, "A"));

    let moved = trace
        .events("panel-float")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if !moved.contains("moved=true") {
        failures.push(format!(
            "the float command did not move the panel (trace said {moved:?}). Either the \
             operand did not reach `dock_menu_panel` or `DockLayout::float` declined — and \
             since section A resets the layout first, `declined` now means the mode's DEFAULT \
             dock does not mount `{PANEL}`, not that a previous launch left it somewhere"
        ));
    } else {
        report.note(format!("★ the panel floated: {moved}"));
    }

    // The window itself. `viewport-inner` is emitted by every child viewport,
    // including a dialog's, so this run must not open one — it does not: the
    // only commands fired are the reset and the float.
    let viewport = float_viewport(&trace);
    match &viewport {
        Some(id) => {
            report.note(format!("★ a child OS window exists, viewport {id}"));
        }
        None => failures.push(format!(
            "no `{VIEWPORT_INNER_EVENT}` after floating. The panel left the dock and no window \
             opened for it, which is the panel VANISHING — the exact state \
             `DockFrameReport::floats_undrawn` exists to make detectable and the exact class of \
             defect that shipped three unreachable panels on 2026-08-10"
        )),
    }

    // ★★★ And the body DREW IN IT. Two regions, both required, and both
    // required to carry THIS window's viewport tag — see the module header on
    // why the previous "any viewport-tagged rect" oracle could never pass.
    if let Some(id) = &viewport {
        let body_name = format!("float.body.{PANEL}");
        let content_name = format!("float.content.{PANEL}");
        let tagged = |name: &str| {
            trace
                .events("ui-rect")
                .filter(|l| l.get("name").is_some_and(|n| n == name))
                .find(|l| l.get("viewport").is_some_and(|v| v == id.as_str()))
                .cloned()
        };

        match tagged(&body_name) {
            Some(l) => {
                report.note(format!(
                    "★ the shell gave the panel a compartment: {}",
                    l.raw
                ));
            }
            None => failures.push(format!(
                "no `ui-rect name={body_name}` tagged `viewport={id:?}`. The window opened and \
                 the dock never handed the panel a body rectangle inside it — or \
                 `crate::app::surfaces::floating_panels` stopped publishing the region, in which \
                 case this check has gone blind rather than the application having broken"
            )),
        }

        match tagged(&content_name) {
            Some(l) => {
                let area = l
                    .get_rect("rect")
                    .map(|r| (r.width(), r.height()))
                    .unwrap_or((0.0, 0.0));
                if area.0 > 0.0 && area.1 > 0.0 {
                    report.note(format!(
                        "★ and the panel FILLED it — {:.1} x {:.1} pt of content: {}",
                        area.0, area.1, l.raw
                    ));
                } else {
                    failures.push(format!(
                        "the float window opened and its panel allocated NOTHING — \
                         `{content_name}` is {area:?}. That is the defect this check is named \
                         for: an OS window with a title bar and a blank interior, which every \
                         other line in this trace reports as a success. R9 permits a panel with \
                         nothing to say to draw one sentence; it does not permit a window to \
                         draw nothing at all"
                    ));
                }
            }
            None => failures.push(format!(
                "no `ui-rect name={content_name}` tagged `viewport={id:?}`, so how much of the \
                 window the panel filled is unknown. This region is published unconditionally \
                 after the body draws, so its absence means the body callback was never reached"
            )),
        }
    }

    // The dock's own count of the same fact, from the other side of the seam.
    let windows = trace
        .events("float-windows")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if windows.contains("empty=0") {
        report.note(format!("★ and the dock agrees nothing is empty: {windows}"));
    } else {
        failures.push(format!(
            "`float-windows` did not report `empty=0` (trace said {windows:?}). \
             `FloatFrameReport::empty_bodies` is the dock's own measurement of the body `Ui` it \
             handed over, and it is an independent witness to the `float.content` region above — \
             the two disagreeing is itself worth reading before believing either"
        ));
    }

    // -----------------------------------------------------------------
    // B. Dock it back — the panel returns and the WINDOW GOES.
    // -----------------------------------------------------------------
    let trace = run_once(
        ctx,
        "dock",
        &format!("view.panel_float@{PANEL},view.panel_dock@{PANEL}"),
    )?;
    report.artifact(ctx.out("panel-float-dock.trace.txt"));
    failures.extend(reset_landed(&trace, "B"));
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
    // ★★ The window has to GO, and this is the half a state-only assertion
    // cannot reach: a dock-back that rebuilt the stack and left the window
    // open would draw the same panel twice, from two `Ui`s with the same
    // widget ids, and `panel-dock moved=true` would still be true.
    match float_viewport(&trace) {
        Some(id) => {
            if window_retired(&trace, &id) {
                report.note(format!(
                    "★ and the window went with it (viewport {id} retired)"
                ));
            } else {
                failures.push(format!(
                    "the panel docked back and its window ({id}) was never retired — no \
                     `ui-rect-gone name=viewport-inner:{id:?}`. The panel is being drawn in two \
                     places at once"
                ));
            }
        }
        None => failures.push(
            "section B saw no float window at all, so whether docking back closes one is \
             unanswered here"
                .to_owned(),
        ),
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
    failures.extend(reset_landed(&trace, "C"));
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
    if let Some(id) = float_viewport(&trace) {
        if window_retired(&trace, &id) {
            report.note("★ and the closed panel's window is gone too".to_owned());
        } else {
            failures.push(format!(
                "the panel was closed and its window ({id}) is still being drawn. A window for a \
                 panel that is no longer in the layout is unreachable from every menu, because \
                 every one of them asks whether the panel is open"
            ));
        }
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
    failures.extend(reset_landed(&trace, "D"));
    // ★★★ The precondition, asserted rather than assumed. `docked=0` is a
    // TRUE answer when nothing was floating, and for days this section was
    // reporting the recovery verb broken on the strength of a state its own
    // earlier launch had left behind. Assert the float first, and `docked=0`
    // becomes evidence again.
    let floated = trace
        .events("panel-float")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if !floated.contains("moved=true") {
        failures.push(format!(
            "section D could not float the panel (trace said {floated:?}), so what \
             `view.dock_all_panels` answers below is a statement about an empty float list and \
             says nothing about the recovery verb"
        ));
    }
    let all = trace
        .events("panels-dock-all")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if all.contains("docked=1") {
        report.note(format!("★ and Dock all panels recovers it: {all}"));
    } else {
        failures.push(format!(
            "`view.dock_all_panels` recovered nothing (trace said {all:?}) after a float this \
             run had just proved happened. It is the only remedy that does not require the \
             operator to act ON the window, so it is the only one that works when the window is \
             on a monitor that no longer exists"
        ));
    }

    if failures.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!(
        "★ {} panel-window propert{} failed:\n  · {}",
        failures.len(),
        if failures.len() == 1 { "y" } else { "ies" },
        failures.join("\n  · ")
    )))
}

#[cfg(test)]
mod tests {
    use super::{float_viewport, reset_landed, window_retired};
    use crate::trace::Trace;

    /// The prefix the application writes in front of every trace line.
    const PREFIX: &str = "pdfcer-diag";

    /// A trace built from the literal lines a real run produced, so the
    /// parsing these helpers depend on is asserted against the application's
    /// actual spelling rather than against a reconstruction of it.
    ///
    /// Every line here is copied from
    /// `D:\temp\uvdrive\A\panels_float_close_and_dock\panel-float-dock.trace.txt`.
    fn real_lines() -> Trace {
        Trace::parse(
            "pdfcer-diag layout-reset scope=All changed=true\n\
             pdfcer-diag panel-float panel=Some(\"view.panel_layers\") moved=true\n\
             pdfcer-diag viewport-inner id=\"F83E\" rect=[[788.0 71.0] - [1108.0 551.0]]\n\
             pdfcer-diag float-windows drawn=1 real=1 empty=0 closed=None docked=None\n\
             pdfcer-diag panel-dock panel=Some(\"view.panel_layers\") moved=true\n\
             pdfcer-diag ui-rect-gone name=viewport-inner:\"F83E\"\n",
            PREFIX,
        )
    }

    /// ★★★ **The retirement key survives the parser**, which is the one
    /// place this check could go silently blind.
    ///
    /// `diag` spells the census key `viewport-inner:"F83E"` — a value with a
    /// quote **inside** it rather than around it — and the trace parser
    /// strips surrounding quotes from a field. If it stripped these, or if
    /// it split the field at the quote, `window_retired` would answer `false`
    /// for every run and this check would report *"the panel is being drawn
    /// in two places at once"* about a build that closes its window
    /// perfectly. That failure would be articulate, permanent and about
    /// nothing, which is this harness's signature defect.
    #[test]
    fn the_windows_retirement_is_recognised_from_a_real_trace_line() {
        let trace = real_lines();
        let id = float_viewport(&trace).expect("the run named its float viewport");
        assert_eq!(id, "F83E", "the id must come back without its quotes");
        assert!(
            window_retired(&trace, &id),
            "`ui-rect-gone name=viewport-inner:\"F83E\"` must be recognised as this window going"
        );
    }

    /// **A different window's retirement is not this one's.**
    ///
    /// Without this, `window_retired` could be written as *"is there any
    /// `ui-rect-gone` naming a viewport"* and pass on a run where a dialog
    /// closed and the panel's window stayed open.
    #[test]
    fn another_windows_retirement_does_not_count() {
        let trace = Trace::parse(
            "pdfcer-diag viewport-inner id=\"F83E\" rect=[[1.0 2.0] - [3.0 4.0]]\n\
             pdfcer-diag ui-rect-gone name=viewport-inner:\"AAAA\"\n",
            PREFIX,
        );
        assert!(!window_retired(&trace, "F83E"));
    }

    /// ★★★ **A run whose layout was never reset is REPORTED, not believed.**
    ///
    /// This is the whole of the fix for the `moved=false` / `docked=0`
    /// family: without it a section inherits the previous launch's saved
    /// layout and every verdict downstream describes a state the harness
    /// created. See the module header, defect 1.
    #[test]
    fn a_section_that_never_reset_its_layout_says_so() {
        let trace = Trace::parse(
            "pdfcer-diag panel-float panel=Some(\"view.panel_layers\") moved=false\n",
            PREFIX,
        );
        let complaint = reset_landed(&trace, "D").expect("a missing reset must be a failure");
        assert!(
            complaint.contains("layout.ron"),
            "and it must name the mechanism that leaked, not merely say `reset`: {complaint}"
        );
    }

    /// **…and a run that did reset raises nothing**, so the guard above is
    /// not simply always on.
    #[test]
    fn a_section_that_reset_its_layout_raises_nothing() {
        assert!(reset_landed(&real_lines(), "A").is_none());
    }
}
