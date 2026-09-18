//! `panel_carried_home_lands_where_it_was_aimed` — **a floating panel dragged
//! by its header onto a dock compartment joins THAT compartment.**
//!
//! # What this is for, and what it is not
//!
//! `panel_float` proves a panel tears out into a real window, draws in it, and
//! can be put back — by a **command**. Putting it back by command always
//! returns it to the compartment it came from, because that is the only
//! address a command has. This check is about the other route: the operator
//! picks the window up and drops it somewhere *else*.
//!
//! The two routes are worth separating because they fail apart. A build can
//! have a perfect `view.panel_dock` and no carry at all — and the symptom is a
//! window whose only way home is a menu row.
//!
//! # The oracle is *which* compartment, not *whether it docked*
//!
//! A carry that landed nowhere falls back to leaving the window where it was
//! let go; a carry that landed *home* is indistinguishable from the command
//! route. So the aim is deliberately a compartment the panel is **not** from:
//! Layers is a left-dock panel in Edit's default arrangement, and this drops
//! it onto the right dock's Objects stack. The assertion is that the two tabs
//! end up in the **same tab bar**, found by containment — which is false for a
//! drop that went home, false for a drop that was ignored, and false for a
//! drop that resolved against the wrong compartment's rectangle.
//!
//! ★ Containment rather than a compartment address, for `panel_tab_reorder`'s
//! reason: a tab region is named for its panel and carries no compartment, so
//! the only thing that says which bar a tab is in is where it was drawn.
//!
//! # Why the gesture needs its own verb
//!
//! `Driver::drag` raises the application's main window first, which on this
//! gesture would bury the float window the press is aimed at. `Driver::carry`
//! raises the window the gesture *begins* in, and rests on the destination
//! before releasing — see its documentation for why a release on the arrival
//! frame lands nothing.
//!
//! # Where the aim points come from
//!
//! Both are regions the application published on the frame it drew them, which
//! is region source 1 and the only one that survives a layout change.
//!
//! | end | region |
//! |---|---|
//! | from | `float.header.<panel>`, tagged with the float window's viewport |
//! | to | `dock.body.<target>`, in the application window |
//!
//! The `from` region exists **because of this check**. The header strip is the
//! shell's own geometry, derived from `floatwin::BODY_MARGIN_PTS` and
//! `floatwin::split_header`, and the shell has no diagnostic channel and must
//! not grow one (R7). A harness that re-derived the strip from those two
//! constants would be holding a copy of the shell's layout, and would go on
//! aiming confidently at the old place the day either constant moved. The
//! application publishes it instead, from the tab-menu handler it already
//! supplies for that strip.
//!
//! # What this deliberately does NOT assert
//!
//! * **That the compass was visible while the window was over it.** It is
//!   drawn in the application window and the carried window is on top of it,
//!   which is a disclosure shortfall recorded in `GUI_ROADMAP.md` rather than
//!   a correctness one — the drop still resolves and still lands. Asserting it
//!   needs a screenshot taken mid-gesture and a rule about what "visible
//!   enough" means.
//! * **Every zone.** One drop into one compartment. The five-zone grammar and
//!   its previews are `dock::overlay`'s unit tests; what those cannot reach is
//!   whether a real pointer crossing a real window boundary arrives at the
//!   grammar at all, and that is this file's single question.

use crate::checks::driving::{
    SHELL_DIAG_ENV, VIEWPORT_INNER_EVENT, declared, declared_in, declared_names, frame_for, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The variable that fires commands at start-up, one per frame.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";

/// **The panel that is carried.**
///
/// Layers, and the choice carries the oracle: it is a **left**-dock panel in
/// Edit's default arrangement, so a drop that landed in the right dock cannot
/// be a drop that went home.
const PANEL: &str = "view.panel_layers";

/// **The compartment it is aimed at** — Objects, the right dock's upper stack
/// in Edit.
///
/// Its body region is the aim point and its tab region is half the oracle.
const TARGET: &str = "view.panel_objects";

/// The prefix of the per-panel tab regions.
const TAB: &str = "dock.tab.";
/// The prefix of the per-panel body regions.
const BODY: &str = "dock.body.";
/// The suffix a stack's tab bar region carries.
const TABBAR: &str = ".tabbar";

/// The mode whose default arrangement puts [`PANEL`] and [`TARGET`] on
/// opposite sides, which is what makes the landing assertion mean something.
const MODE: &str = "mode.edit";

/// Fired before the float so the arrangement is the mode's default and not
/// whatever the previous run saved to `userdata/layout.ron`.
const RESET: &str = "view.reset_layout";

/// See the module documentation.
pub struct PanelCarriedHomeLandsWhereItWasAimed;

impl Check for PanelCarriedHomeLandsWhereItWasAimed {
    fn name(&self) -> &'static str {
        "panel_carried_home_lands_where_it_was_aimed"
    }

    fn defect(&self) -> &'static str {
        "a floating panel's window cannot be picked up by its header and dropped back into the \
         dock, or it can and the panel lands in the compartment it was torn from rather than the \
         one the operator aimed at — which makes the gesture a slow way of pressing Dock"
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

/// **The tab bar that contains a given tab**, by containment.
///
/// `None` when no declared bar contains it, which is itself a finding: every
/// drawn tab is drawn inside one.
fn bar_holding(trace: &crate::trace::Trace, ui_rect: &str, tab: LRect) -> Option<String> {
    declared_names(trace, ui_rect, "dock.")
        .into_iter()
        .filter(|n| n.ends_with(TABBAR))
        .find(|n| declared(trace, ui_rect, n).is_some_and(|r| r.contains_rect(tab)))
}

/// Collect the failures into the sentence the report carries.
fn verdict(failures: &[String]) -> Option<String> {
    if failures.is_empty() {
        return None;
    }
    Some(format!(
        "★ {} carry propert{} failed:\n  · {}",
        failures.len(),
        if failures.len() == 1 { "y" } else { "ies" },
        failures.join("\n  · ")
    ))
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
            "input is disabled (--no-input). This check drags a real OS window across the \
             desktop with the button held. Reported as SKIPPED rather than passed: a check that \
             did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("panel_carry.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // One command per frame, in precondition order: the mode first, because
    // the reset restores *that* mode's default dock, then the float.
    spec.env.push((
        (*INVOKE_ENV).to_owned(),
        format!("{MODE},{RESET},view.panel_float@{PANEL}"),
    ));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    // Generous, for `panel_float`'s reason: a child viewport is created DURING
    // a frame and the window manager settles it over several more.
    session.settle(80);
    let trace = session.trace()?;

    // --- the preconditions, asserted rather than assumed ------------------
    let reset = trace
        .events("layout-reset")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if !reset.contains("changed=") {
        return Ok(Some(format!(
            "`{RESET}` never landed (trace said {reset:?}), so this run inherited whatever the \
             previous one saved. Which compartment the panel starts in is then unknown, and the \
             landing assertion below would be about a state the harness created"
        )));
    }
    let floated = trace
        .events("panel-float")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if !floated.contains("moved=true") {
        return Ok(Some(format!(
            "the panel never floated (trace said {floated:?}), so there is no window to carry. \
             `panel_float` is the check that says which half of that is broken"
        )));
    }

    // --- the two aim points, both published by the application ------------
    let header_name = format!("float.header.{PANEL}");
    let Some((header_rect, header_vp)) = declared_in(&trace, ui_rect, &header_name) else {
        return Ok(Some(format!(
            "no `{header_name}` region, so where the window's grab handle is drawn is unknown \
             and this check has nothing to aim at. It is published by \
             `crate::app::surfaces::floating_panels`' header handler; its absence means the \
             handler stopped running or the region was removed, in which case the harness has \
             gone blind rather than the gesture having broken"
        )));
    };
    if header_vp.is_none() {
        return Ok(Some(format!(
            "`{header_name}` carries no `viewport=` tag, so its rectangle would be converted \
             against the APPLICATION window's origin and the press would land hundreds of points \
             away, inside whatever is there. The harness refuses to aim"
        )));
    }
    let float_frame = frame_for(&session, &trace, header_vp.as_deref())?;
    let from = float_frame.declared_center(header_rect);

    let target_body = format!("{BODY}{TARGET}");
    let Some((target_rect, target_vp)) = declared_in(&trace, ui_rect, &target_body) else {
        return Ok(Some(format!(
            "no `{target_body}` region, so the compartment this check aims at is not on screen. \
             Edit's default arrangement puts `{TARGET}` in the right dock; if that changed, this \
             check's whole premise — that the aim is somewhere the panel is not from — needs \
             restating rather than repairing"
        )));
    };
    let target_frame = frame_for(&session, &trace, target_vp.as_deref())?;
    let to = target_frame.declared_center(target_rect);

    report.note(format!(
        "carrying `{PANEL}` from its header at ({}, {}) onto `{TARGET}`'s body at ({}, {})",
        from.x(),
        from.y(),
        to.x(),
        to.y()
    ));

    // --- the gesture ------------------------------------------------------
    let driver = Driver::new(session.window());
    driver.carry(from, to)?;
    session.settle(60);
    let trace = session.trace()?;

    let mut failures: Vec<String> = Vec::new();

    // --- 1: the window went -----------------------------------------------
    match trace
        .events(VIEWPORT_INNER_EVENT)
        .last()
        .and_then(|l| l.get("id").map(str::to_owned))
    {
        Some(id) => {
            let key = format!("viewport-inner:{id:?}");
            if trace
                .events("ui-rect-gone")
                .any(|l| l.get("name").is_some_and(|n| n == key))
            {
                report.note(format!(
                    "★ the float window closed on the drop (viewport {id})"
                ));
            } else {
                failures.push(format!(
                    "the window ({id}) is still being drawn after the drop. If the panel also \
                     landed in the dock it is now drawn twice, from two `Ui`s with the same \
                     widget ids"
                ));
            }
        }
        None => failures.push(
            "no float window in this run's trace at all, so what the carry did is unanswered"
                .to_owned(),
        ),
    }

    // --- 2: the panel is in the dock --------------------------------------
    let tab_name = format!("{TAB}{PANEL}");
    let Some(tab_rect) = declared(&trace, ui_rect, &tab_name) else {
        failures.push(format!(
            "`{tab_name}` is not declared after the drop, so the panel is in no stack. Either \
             the release resolved no compartment — which leaves the window where it was let go, \
             and the assertion above should then also have failed — or the drop was applied to a \
             layout that dropped the panel entirely. Tabs drawn: {}",
            list(&declared_names(&trace, ui_rect, TAB))
        ));
        return Ok(verdict(&failures));
    };

    // --- 3: ★★★ and it is in the stack it was AIMED at ---------------------
    //
    // The assertion the two above cannot make between them: a panel that
    // docked back to its own left-dock home satisfies both of them.
    let target_tab = format!("{TAB}{TARGET}");
    match declared(&trace, ui_rect, &target_tab) {
        Some(anchor) => match (
            bar_holding(&trace, ui_rect, anchor),
            bar_holding(&trace, ui_rect, tab_rect),
        ) {
            (Some(aimed), Some(landed)) => {
                if aimed == landed {
                    report.note(format!(
                        "★★★ and it landed in the bar it was aimed at: `{landed}` now holds both \
                         `{PANEL}` and `{TARGET}`"
                    ));
                } else {
                    failures.push(format!(
                        "the panel docked into `{landed}` and the drop was aimed at `{aimed}`. \
                         The gesture reached the dock and the compartment it resolved is not the \
                         one under the pointer — a coordinate-space defect with plausible \
                         numbers, arriving through the conversion written to prevent it. The two \
                         inputs to that conversion are `{header_name}` and `{VIEWPORT_INNER_EVENT}`"
                    ));
                }
            }
            (aimed, landed) => failures.push(format!(
                "a tab was drawn inside no declared tab bar (aimed: {aimed:?}, landed: \
                 {landed:?}). Every drawn tab is drawn inside one, so this is the bar regions \
                 having stopped being published rather than a statement about the drop"
            )),
        },
        None => failures.push(format!(
            "`{target_tab}` is not declared after the drop, so the stack this check aimed at is \
             no longer on screen and there is nothing to compare the landing against"
        )),
    }

    Ok(verdict(&failures))
}
