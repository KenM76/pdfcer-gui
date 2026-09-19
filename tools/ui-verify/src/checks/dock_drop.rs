//! The two affordances a panel drag shows before it commits — **the drop
//! compass over a compartment, and the outline of the window a tear would
//! open** — driven through the OS and asserted against the running binary.
//!
//! # Why these needed a channel before they could have a check
//!
//! Both affordances are *only* a picture. The compass is a wash of the accent
//! colour over five quadrants with one of them stronger; the tear is a thin
//! outline at the pointer. Neither is a thing a trace could carry, and a
//! screenshot taken after the gesture cannot see either, because both exist
//! only while the button is down.
//!
//! What a wrong build gets wrong, though, is not the colour. It is *which
//! compartment the pointer resolved to*, *which zone of it*, *what a release
//! would then do*, and *whether the thing the operator was shown is the thing
//! that happened*. All four are decisions, and the application now publishes
//! them on the `dock-drop` and `dock-tear` trace slots. These checks read
//! those.
//!
//! # ★ Two channels, published by different mechanisms, cross-checked
//!
//! The decision arrives on the trace slot. The *painting* arrives as a
//! published region — `dock.<addr>.zone.<zone>` for the armed quadrant and
//! `dock.tear.outline` for the outline — put there by the dock's own rect
//! reporter. A build that resolved a compartment correctly and painted
//! nothing satisfies one and not the other, and so does a build that painted
//! the compass over the compartment next door. Asserting both is what makes
//! the pair mean *"the operator was shown the thing that then happened"*
//! rather than *"a decision was taken somewhere"*.
//!
//! # ★★ Nothing here is hard-coded to a compartment
//!
//! The arrangement is discovered from the regions the run itself published:
//! which compartments the dock drew, which tabs are in which, and which of
//! them has a body to aim at. `panel_tab_reorder`'s lesson, and it is not a
//! stylistic one — the left dock in this application draws **no tab strip at
//! all** when its rail is showing, so a check that named a strip would be
//! asserting about a surface the operator cannot see. Discovery also means
//! these keep asserting the same property the day the default arrangement
//! moves a panel, instead of turning red about the move.
//!
//! # What a passing run does NOT prove
//!
//! * **That the compass is legible.** These read rectangles and decisions. A
//!   compass painted in the background colour passes every assertion here;
//!   that is a pixel question and a pixel oracle is the instrument for it.
//! * **That every zone resolves correctly.** One drop into one compartment.
//!   The five-zone grammar has `dock::overlay`'s unit tests; what those
//!   cannot reach is whether a real pointer, driven through the OS across a
//!   real window, arrives at the grammar at all.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_since, list, live_names,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::{LRect, Pt};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// The variable that fires commands at start-up, one per frame.
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";

/// The mode whose default arrangement fills both docks.
const MODE: &str = "mode.edit";

/// Fired before the gesture so the arrangement is the mode's default and not
/// whatever the previous run saved to `userdata/layout.ron`.
const RESET: &str = "view.reset_layout";

/// The prefix of the per-panel tab regions.
const TAB: &str = "dock.tab.";
/// The prefix of the per-panel body regions.
const BODY: &str = "dock.body.";
/// The suffix a stack's tab bar region carries.
const TABBAR: &str = ".tabbar";
/// The region the outline of a torn-out window is published under.
const TEAR_OUTLINE: &str = "dock.tear.outline";
/// The trace slot the drop compass publishes its decision on.
const DROP: &str = "dock-drop";
/// The trace slot the tear outline publishes its decision on.
const TEAR: &str = "dock-tear";
/// What either slot prints when there is no offer — the control that tells a
/// silent slot apart from a slot nothing writes to.
const NONE: &str = "none";

/// How long the pointer rests on the destination before the button comes up.
///
/// The offer is resolved on the frame the pointer arrives and the release is
/// read on a later one; a gesture that let go on arrival would be releasing
/// against an offer that did not exist yet. `Driver::drag_via` spends this as
/// one-pixel nudges rather than one sleep, because a stationary pointer
/// generates no input and a build is not obliged to repaint without it.
const DWELL: std::time::Duration = std::time::Duration::from_millis(600);

/// How far the window that opens may sit from the origin the outline promised.
///
/// The measured residual is **0.0 pt on both axes** — `dock-tear at=[1267.0
/// 502.0]` and `viewport-outer rect=[[1267.0 502.0] - ...]` from the same run.
/// The allowance is not slack for a wrong conversion: it covers only the round
/// trip through the window manager, which is asked for a position in logical
/// points and reports one back after snapping to whole physical pixels. At a
/// `ppp` above 1 that rounding is a fraction of a point, so one point is the
/// ceiling on a correct build at any scale this application runs at.
///
/// ★ Sized deliberately far below the failure it is guarding against. A
/// conversion that adds the wrong window origin, or none, is wrong by the
/// application window's own position — hundreds of points. Anything between one
/// point and that is a defect nobody has met yet and should be read, not
/// tolerated, so widening this constant is the wrong response to it failing.
const ORIGIN_TOLERANCE_PTS: f32 = 1.0;

/// How far the window's client area may differ from the size the outline drew.
///
/// Same measurement, same reasoning: the outline is `DEFAULT_SIZE_PTS` and the
/// window is built `with_inner_size` from the same value, so the two agree
/// exactly (320x480 measured) and the allowance covers only pixel rounding.
const SIZE_TOLERANCE_PTS: f32 = 1.0;

/// **A drag over another compartment offers that compartment, and the release
/// lands where the offer said.**
pub struct ADragOverTheDockOffersTheCompartmentUnderThePointer;

impl Check for ADragOverTheDockOffersTheCompartmentUnderThePointer {
    fn name(&self) -> &'static str {
        "a_drag_over_the_dock_offers_the_compartment_under_the_pointer"
    }

    fn defect(&self) -> &'static str {
        "a panel tab dragged onto a different compartment proposes nothing — the operator drags \
         into an unmarked dock and finds out where it went by letting go — or it proposes a \
         compartment and the release puts the panel somewhere else, which is worse than \
         proposing nothing because it is a promise that was not kept"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_drop(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// **A drag carried off the dock offers a window, and the release opens one.**
pub struct ADragCarriedOffTheDockOffersAWindow;

impl Check for ADragCarriedOffTheDockOffersAWindow {
    fn name(&self) -> &'static str {
        "a_drag_carried_off_the_dock_offers_a_window"
    }

    fn defect(&self) -> &'static str {
        "a panel tab dragged out of the dock shows nothing on the way — so the operator cannot \
         tell a tear from a drag that is about to do nothing — or it shows an outline and the \
         release opens no window, or opens one while the compass was also offering a \
         compartment, which means two affordances answered one drag"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_tear(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

// --- the arrangement, discovered ------------------------------------------

/// **One compartment the dock drew**, by its structural address.
struct Compartment {
    /// The address as the dock spells it — `left.0.0`, `right.0.1`. This is
    /// the same spelling the `dock-drop` slot prints under `over=`, which is
    /// what lets the two be compared without a translation table.
    addr: String,
    /// Tab bar and body together.
    rect: LRect,
}

impl Compartment {
    /// The region name of this compartment's tab bar.
    ///
    /// Derived rather than looked up, so the two cannot drift: a compartment
    /// and its strip are built from one address by the dock, and are rebuilt
    /// from one address here.
    fn bar(&self) -> String {
        format!("dock.{}{TABBAR}", self.addr)
    }
}

/// Every compartment the run declared, in address order.
///
/// A compartment's region name is `dock.<side>.<column>.<stack>` and nothing
/// else is: four dot-separated parts, of which the last two are numbers. The
/// alternative — matching on the side words — would also catch
/// `dock.right.0.1.tabbar` and every other suffix the dock hangs off the same
/// stem.
fn compartments(trace: &Trace, ui_rect: &str) -> Vec<Compartment> {
    let mut found: Vec<Compartment> = declared_names(trace, ui_rect, "dock.")
        .into_iter()
        .filter_map(|name| {
            let addr = name.strip_prefix("dock.")?;
            let parts: Vec<&str> = addr.split('.').collect();
            let [_side, column, stack] = parts.as_slice() else {
                return None;
            };
            if !column.chars().all(|c| c.is_ascii_digit()) || !stack.chars().all(char::is_numeric) {
                return None;
            }
            Some(Compartment {
                addr: addr.to_owned(),
                rect: declared(trace, ui_rect, &name)?,
            })
        })
        .collect();
    found.sort_by(|a, b| a.addr.cmp(&b.addr));
    found
}

/// Which compartment a point is in, if any.
fn holding(compartments: &[Compartment], p: Pt) -> Option<&Compartment> {
    let dot = LRect::new(p, p);
    compartments.iter().find(|c| c.rect.contains_rect(dot))
}

/// The centre of a rectangle, in the coordinates it was declared in.
fn centre(r: LRect) -> Pt {
    Pt::new(
        f32::midpoint(r.min.x, r.max.x),
        f32::midpoint(r.min.y, r.max.y),
    )
}

/// **A tab that can be picked up, and the compartment it is in.**
///
/// The first in region-name order whose rectangle lies inside a compartment
/// that has a tab bar — which is the condition for the tab being *visible*,
/// and this application has a dock side that draws none.
fn a_draggable_tab<'a>(
    trace: &Trace,
    ui_rect: &str,
    compartments: &'a [Compartment],
) -> Option<(String, LRect, &'a Compartment)> {
    let bars = declared_names(trace, ui_rect, "dock.");
    declared_names(trace, ui_rect, TAB)
        .into_iter()
        .find_map(|name| {
            let rect = declared(trace, ui_rect, &name)?;
            let home = holding(compartments, centre(rect))?;
            bars.iter()
                .any(|b| *b == home.bar())
                .then_some((name, rect, home))
        })
}

/// Collect the failures into the sentence the report carries.
fn verdict(failures: &[String]) -> Option<String> {
    if failures.is_empty() {
        return None;
    }
    Some(format!(
        "★ {} propert{} failed:\n  · {}",
        failures.len(),
        if failures.len() == 1 { "y" } else { "ies" },
        failures.join("\n  · ")
    ))
}

/// The launch every check in this module makes: Edit, default arrangement, the
/// fixture open, both diagnostic channels on.
fn launch(ctx: &CheckContext, artifact: &str) -> Result<Session> {
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
    let mut spec = LaunchSpec::new(&exe, ctx.out(artifact));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // The mode first, because the reset restores *that* mode's default dock.
    spec.env
        .push(((*INVOKE_ENV).to_owned(), format!("{MODE},{RESET}")));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    Session::launch(&spec, ctx.profile.trace_prefix)
}

/// The preconditions both gestures share, asserted rather than assumed.
///
/// Returns the sentence to fail with, or the ui-rect event name to read
/// regions under.
fn preconditions(ctx: &CheckContext) -> Result<std::result::Result<&'static str, String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check drags a panel tab across the application \
             with the button held. Reported as SKIPPED rather than passed: a check that did not \
             run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;
    Ok(Ok(ui_rect))
}

/// Did the reset land? A run that inherited a saved layout is a run whose
/// arrangement the harness created rather than measured.
fn reset_landed(trace: &Trace) -> std::result::Result<(), String> {
    let reset = trace
        .events("layout-reset")
        .last()
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    if reset.contains("changed=") {
        return Ok(());
    }
    Err(format!(
        "`{RESET}` never landed (trace said {reset:?}), so this run inherited whatever the \
         previous one saved and the arrangement below is unknown rather than measured"
    ))
}

/// **What a slot was offering when the button came up, and whether the release
/// cleared it.**
///
/// # ★ Why this is the last *two* lines and not the last one
///
/// The affordance is gone the instant the button is — that is what makes it a
/// pre-commit affordance rather than a mark on the document — so the release
/// itself writes the no-offer control, and the terminal line of *every*
/// completed gesture is `none`. A check that read only the last line would
/// report a perfectly correct build as having offered nothing, which is the
/// assertion inverted.
///
/// So the standing offer is the line **immediately before** the terminal
/// control, and it counts only when the control is genuinely terminal.
/// Requiring adjacency is what distinguishes *the pointer was over a
/// compartment when it was released* from *the pointer was over one earlier in
/// the gesture and had left by the end* — two histories whose last offer line
/// is the same line.
///
/// Returns the standing offer, and whether the slot's last line is the
/// control. The second is reported separately because an offer still standing
/// after the release is its own defect: the affordance has outlived the
/// gesture and is now marking the page.
fn standing_at_release<'a>(
    trace: &'a Trace,
    slot: &'a str,
) -> (Option<&'a crate::trace::TraceLine>, bool) {
    let lines: Vec<&crate::trace::TraceLine> = trace.events(slot).collect();
    let cleared = lines.last().is_some_and(|l| l.get("panel").is_none());
    if !cleared {
        return (None, false);
    }
    let standing = lines
        .len()
        .checked_sub(2)
        .and_then(|i| lines.get(i))
        .filter(|l| l.get("panel").is_some())
        .copied();
    (standing, true)
}

/// The trace line the drag begins after, for [`declared_since`].
///
/// # ★ Why a pre-commit affordance cannot be read with `declared`
///
/// [`declared`] answers *is this region on screen now*, and honours the
/// `ui-rect-gone` line that retires one. Every affordance this module measures
/// is retired by the time the trace is read — the wash, the caret and the
/// outline all vanish at the release, which is what makes them affordances
/// rather than marks on the document — so `declared` reports each of them
/// absent on a build that painted all three correctly, and the failure
/// sentence it produces names the region in its own *"regions drawn"* list.
///
/// [`declared_since`] asks the right question, *was it published during this
/// gesture*, and requires an anchor so that it cannot degrade into reading a
/// fossil left by an earlier drag in the same run. This is that anchor: the
/// last region line the application wrote before the pointer moved.
fn gesture_start(before: &Trace, ui_rect: &str) -> usize {
    before.events(ui_rect).last().map_or(0, |l| l.lineno)
}

/// Whether a slot ever printed its no-offer control.
///
/// Asserted because a change-only slot that simply stops emitting looks
/// exactly like a slot nothing writes to. The control line is what says the
/// channel is alive.
fn control_seen(trace: &Trace, slot: &str) -> bool {
    trace.events(slot).any(|l| l.raw.contains(NONE))
}

// --- the compass -----------------------------------------------------------

fn drive_drop(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = match preconditions(ctx)? {
        Ok(name) => name,
        Err(sentence) => return Ok(Some(sentence)),
    };
    let session = launch(ctx, "dock_drop_offer.trace.txt")?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let trace = session.trace()?;
    if let Err(sentence) = reset_landed(&trace) {
        return Ok(Some(sentence));
    }

    let rooms = compartments(&trace, ui_rect);
    let Some((tab_name, tab_rect, home)) = a_draggable_tab(&trace, ui_rect, &rooms) else {
        return Ok(Some(format!(
            "no panel tab is drawn inside a compartment that has a tab bar, so there is nothing \
             to pick up. Compartments: {}; tabs: {}",
            list(&rooms.iter().map(|c| c.addr.clone()).collect::<Vec<_>>()),
            list(&declared_names(&trace, ui_rect, TAB))
        )));
    };
    let panel = tab_name.trim_start_matches(TAB).to_owned();

    // The aim: another compartment's body. A body region rather than the
    // compartment's own centre, because a compartment includes its tab bar and
    // a point in the bar is a *reorder*, answered by the caret and not by the
    // compass — a different affordance and a different check.
    let home_addr = home.addr.clone();
    let Some((target_body, target_rect, target_room)) = declared_names(&trace, ui_rect, BODY)
        .into_iter()
        .find_map(|name| {
            let rect = declared(&trace, ui_rect, &name)?;
            let room = holding(&rooms, centre(rect))?;
            (room.addr != home_addr).then_some((name, rect, room))
        })
    else {
        return Ok(Some(format!(
            "every panel body the dock drew is in `{home_addr}`, the same compartment the tab \
             being dragged is in, so this run has no second compartment to aim at and the \
             property cannot be stated. Bodies: {}",
            list(&declared_names(&trace, ui_rect, BODY))
        )));
    };
    let aimed_at = target_room.addr.clone();

    let frame = session.frame()?;
    let from = frame.declared_center(tab_rect);
    let to = frame.declared_center(target_rect);
    report.note(format!(
        "dragging `{panel}`'s tab out of `{home_addr}` onto `{target_body}` in `{aimed_at}`"
    ));

    let driver = Driver::new(session.window());
    let gesture = gesture_start(&trace, ui_rect);
    driver.drag_via(from, to, DWELL, to, None)?;
    session.settle(40);
    let trace = session.trace()?;

    let mut failures: Vec<String> = Vec::new();

    if !control_seen(&trace, DROP) {
        failures.push(format!(
            "the `{DROP}` slot never printed `{NONE}`, so nothing says the channel is alive. A \
             change-only slot that has stopped emitting and a slot nothing writes to produce the \
             same silence, and the control line is the only thing that tells them apart"
        ));
    }

    // --- 1: an offer was made, naming the panel under the pointer ----------
    let offers: Vec<&crate::trace::TraceLine> = trace
        .events(DROP)
        .filter(|l| l.get("panel").is_some())
        .collect();
    if offers.is_empty() {
        failures.push(format!(
            "the drag proposed nothing anywhere along its path. The pointer left `{home_addr}`, \
             crossed the application and rested {}ms over `{aimed_at}`, and the dock offered no \
             compartment at any point — so the operator dragging into this dock is told nothing \
             about where the panel will go until they let go of it",
            DWELL.as_millis()
        ));
        return Ok(verdict(&failures));
    }
    if !offers
        .iter()
        .any(|l| l.get("panel") == Some(panel.as_str()))
    {
        failures.push(format!(
            "{} offer(s) were made and none of them names `{panel}`, the panel whose tab the \
             press landed on. The drag is carrying something other than what was grabbed",
            offers.len()
        ));
    }

    // --- 2: the last offer names the compartment the pointer was in --------
    //
    // The compartment is resolved twice, independently: the application
    // resolved it through `DockLayout::resolve_drop` from its own geometry,
    // and the harness resolves it here by containment from the rectangles the
    // dock published. A coordinate-space defect produces two different
    // answers; nothing else does.
    let (standing, cleared) = standing_at_release(&trace, DROP);
    if !cleared {
        failures.push(format!(
            "the `{DROP}` slot's last line still names a panel after the button came up: {:?}. \
             The release is what withdraws the offer, so an offer outliving it is an affordance \
             the operator can no longer act on, left painted over the arrangement",
            trace.events(DROP).last().map(|l| l.raw.clone())
        ));
        return Ok(verdict(&failures));
    }
    let Some(last) = standing else {
        failures.push(format!(
            "the button came up with no offer standing — the `{DROP}` slot's control line has no \
             offer in front of it. The pointer was resting inside `{aimed_at}` for {}ms when it \
             was released, so the compass had stood down over a compartment it should have been \
             offering, and a release against no offer lands nothing at all",
            DWELL.as_millis()
        ));
        return Ok(verdict(&failures));
    };
    let Some(over) = last.get("over").map(str::to_owned) else {
        failures.push(format!(
            "the offer standing at the release names a panel but no compartment ({:?}), so the \
             line is malformed and nothing can be said about where the drop would have gone",
            last.raw
        ));
        return Ok(verdict(&failures));
    };
    if over != aimed_at {
        failures.push(format!(
            "the offer standing at the release named `{over}` and the pointer was inside \
             `{aimed_at}`. Both addresses came from this same frame — one from the dock's own \
             drop resolution, one from the rectangles it published — so they disagree about where \
             the pointer is, which is a coordinate-space defect wearing plausible numbers"
        ));
    }
    let lands = last.get("lands") == Some("true");
    let zone = last.get("zone").unwrap_or("").to_owned();

    // --- 3: and the compass was PAINTED there ------------------------------
    //
    // The decision and the picture arrive on two channels. This is the one
    // that fails when a build resolves a compartment perfectly and paints the
    // wash over the compartment next door.
    if zone == "strip" {
        report.note(
            "the offer resolved to a tab strip rather than a body zone, so the affordance is the \
             insertion caret and `panel_tab_reorder` is the check that owns it"
                .to_owned(),
        );
    } else {
        let armed = format!("dock.{over}.zone.{zone}");
        match declared_since(&trace, ui_rect, &armed, gesture) {
            Some(rect) if rect.is_substantial() => {
                report.note(format!(
                    "★ the compass was painted on `{over}`, armed on its `{zone}` zone"
                ));
            }
            Some(rect) => failures.push(format!(
                "`{armed}` was published with no area ({rect:?}), so the armed quadrant is a \
                 rectangle and not a thing on screen"
            )),
            None => failures.push(format!(
                "the offer named `{over}` zone `{zone}` and no `{armed}` region was ever \
                 published, so the decision was taken and nothing was drawn for it. The operator \
                 sees an unmarked dock while the dock has already chosen. Zones drawn: {}",
                list(
                    &declared_names(&trace, ui_rect, "dock.")
                        .into_iter()
                        .filter(|n| n.contains(".zone."))
                        .collect::<Vec<_>>()
                )
            )),
        }
    }

    // --- 4: ★★★ and the release did what the offer promised ----------------
    if !lands {
        report.note(
            "the offer was knocked back (`lands=false`), so the release was legal and changes \
             nothing; the landing assertion below is not made",
        );
        return Ok(verdict(&failures));
    }
    //
    // ★ The subject is the panel's BODY, not its tab. Whether a landed panel
    // has a tab is a property of the compartment it landed in — this
    // application's left dock draws no tab strip at all while the rail is
    // visible — so a tab-shaped assertion reports a correct landing there as
    // the panel having left the dock. Every docked panel has a body.
    let landed_name = format!("{BODY}{panel}");
    let Some(landed_rect) = declared(&trace, ui_rect, &landed_name) else {
        failures.push(format!(
            "`{landed_name}` is not drawn anywhere after the release, so the panel is in no \
             stack. The offer promised `{over}`; what happened is that the panel left the dock. \
             Bodies drawn: {}",
            list(&live_names(&trace, ui_rect, BODY))
        ));
        return Ok(verdict(&failures));
    };
    let after = compartments(&trace, ui_rect);
    match holding(&after, centre(landed_rect)) {
        Some(room) if room.addr == aimed_at => {
            report.note(format!(
                "★★★ and the release kept the promise: `{panel}`'s body is now inside `{aimed_at}`"
            ));
        }
        Some(room) => failures.push(format!(
            "the compass offered `{aimed_at}` and the release put `{panel}` in `{}`. The \
             affordance and the commit are two readings of one decision and they disagreed, which \
             makes the compass a lie rather than a preview",
            room.addr
        )),
        None => failures.push(format!(
            "`{panel}`'s body is drawn inside no compartment after the release \
             ({landed_rect:?}). Every drawn body is drawn inside one, so this is the compartment \
             regions having stopped being published rather than a statement about the drop"
        )),
    }

    Ok(verdict(&failures))
}

// --- the tear --------------------------------------------------------------

fn drive_tear(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let ui_rect = match preconditions(ctx)? {
        Ok(name) => name,
        Err(sentence) => return Ok(Some(sentence)),
    };
    let session = launch(ctx, "dock_tear_offer.trace.txt")?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let trace = session.trace()?;
    if let Err(sentence) = reset_landed(&trace) {
        return Ok(Some(sentence));
    }

    let rooms = compartments(&trace, ui_rect);
    let Some((tab_name, tab_rect, home)) = a_draggable_tab(&trace, ui_rect, &rooms) else {
        return Ok(Some(format!(
            "no panel tab is drawn inside a compartment that has a tab bar, so there is nothing \
             to pick up. Compartments: {}",
            list(&rooms.iter().map(|c| c.addr.clone()).collect::<Vec<_>>())
        )));
    };
    let panel = tab_name.trim_start_matches(TAB).to_owned();
    let home_addr = home.addr.clone();

    // The aim: the canvas. Found as the widest gap between the dock sides
    // rather than named, because what a tear needs is a point **outside every
    // compartment the dock drew**, and that is a property of the arrangement
    // this run produced, not of a region name.
    let canvas = trace
        .last(ctx.profile.vocab.canvas_event)
        .and_then(|l| l.get_rect(ctx.profile.vocab.canvas_rect_field));
    let Some(canvas) = canvas else {
        return Ok(Some(format!(
            "the trace carries no `{}` event with a parsable `{}=`, so where the page is drawn is \
             unknown and this check has nowhere outside the dock to aim at",
            ctx.profile.vocab.canvas_event, ctx.profile.vocab.canvas_rect_field
        )));
    };
    let aim = centre(canvas);
    if let Some(room) = holding(&rooms, aim) {
        return Ok(Some(format!(
            "the centre of the canvas is inside `{}`, which means the dock is drawn over the \
             page and a release there would be a drop rather than a tear. The arrangement, not \
             the gesture, is what this run found",
            room.addr
        )));
    }

    let frame = session.frame()?;
    let from = frame.declared_center(tab_rect);
    let to = frame.declared_center(LRect::new(aim, aim));
    report.note(format!(
        "dragging `{panel}`'s tab out of `{home_addr}` onto the canvas"
    ));

    let driver = Driver::new(session.window());
    let gesture = gesture_start(&trace, ui_rect);
    driver.drag_via(from, to, DWELL, to, None)?;
    session.settle(80);
    let trace = session.trace()?;

    let mut failures: Vec<String> = Vec::new();

    if !control_seen(&trace, TEAR) {
        failures.push(format!(
            "the `{TEAR}` slot never printed `{NONE}`, so nothing says the channel is alive"
        ));
    }

    // --- 1: the outline was offered, naming the panel ----------------------
    let (standing, cleared) = standing_at_release(&trace, TEAR);
    if !cleared {
        failures.push(format!(
            "the `{TEAR}` slot's last line is not the control ({:?}). Either the drag rested \
             {}ms on the canvas and the dock proposed no window at all — the operator pulling a \
             panel out of the dock shown nothing on the way — or an outline is still standing \
             after the button came up, which is an affordance that has outlived its gesture",
            trace.events(TEAR).last().map(|l| l.raw.clone()),
            DWELL.as_millis()
        ));
        return Ok(verdict(&failures));
    }
    let Some(last) = standing else {
        failures.push(format!(
            "the button came up with no tear standing, though the pointer was resting on the \
             canvas for {}ms. Either the outline stood down early or it was never offered; the \
             earlier lines on this slot say which",
            DWELL.as_millis()
        ));
        return Ok(verdict(&failures));
    };
    let Some(named) = last.get("panel").map(str::to_owned) else {
        failures.push(format!(
            "the tear standing at the release names no panel ({:?}), so the line is malformed",
            last.raw
        ));
        return Ok(verdict(&failures));
    };
    if named != panel {
        failures.push(format!(
            "the tear standing at the release names `{named}` and the tab that was pressed is \
             `{panel}`. The drag is carrying something other than what was grabbed"
        ));
    }
    // Kept for step 4: the offer is the promise, and a promise is only worth
    // tracing if something afterwards measures the window against it.
    let promised = last.get_rect("rect").filter(|r| r.is_substantial());
    let promised_at = last.get_vec2("at");
    match (promised, promised_at) {
        (Some(r), Some(at)) => {
            report.note(format!(
                "★ the outline offered a window {:.0}×{:.0} pts at ({:.0}, {:.0}) on the desktop",
                r.width(),
                r.height(),
                at.x,
                at.y,
            ));
        }
        (None, _) => failures.push(format!(
            "the tear offered no usable outline ({:?}). The rectangle is the whole of what \
             the affordance says, so an empty one is the affordance having nothing to show",
            last.get_rect("rect")
        )),
        (Some(_), None) => failures.push(format!(
            "the tear offered an outline and no `at` ({:?}), so it says how big a window would \
             be and never where. `at` is the value handed to the window as its position, so \
             without it on the line nothing can check that the window opened where the \
             operator was shown it would",
            last.raw
        )),
    }

    // --- 2: and it was PAINTED ---------------------------------------------
    match declared_since(&trace, ui_rect, TEAR_OUTLINE, gesture) {
        Some(rect) if rect.is_substantial() => {
            report.note("★ the outline was drawn at the pointer".to_owned());
        }
        Some(rect) => failures.push(format!(
            "`{TEAR_OUTLINE}` was published with no area ({rect:?})"
        )),
        None => failures.push(format!(
            "the tear was decided and no `{TEAR_OUTLINE}` region was ever published, so nothing \
             was drawn for it. The panel leaves the dock on release with no warning that it was \
             going to"
        )),
    }

    // --- 3: ★★ and the compass stood down ----------------------------------
    //
    // The two affordances are mutually exclusive by construction — the compass
    // needs a compartment under the pointer and the tear needs there to be
    // none — and the crate asserts it in a `debug_assert` a release build does
    // not run. This is that invariant measured from outside, on the build the
    // operator has.
    //
    // A healthy run leaves the drop slot either empty or holding a single
    // `none`, so the two ways it can hold an offer at the release are read
    // together: one standing in front of the release's own control, and one
    // that the release never cleared at all.
    let (compass, compass_cleared) = standing_at_release(&trace, DROP);
    let compass = compass.or_else(|| {
        (!compass_cleared)
            .then(|| trace.events(DROP).last())
            .flatten()
    });
    match compass {
        Some(l) => failures.push(format!(
            "a compass offer was still standing when the tear was offered (drop: {:?}). Two \
             affordances answered one drag, and which of them the release obeys is then decided \
             by the order the fields happen to be read in",
            l.raw
        )),
        None => {
            report.note("★★ the compass stood down over the canvas".to_owned());
        }
    }

    // --- 4: ★★★ and the release opened the window --------------------------
    //
    // The body, not the tab, for the reason `drive_drop`'s landing assertion
    // records: a tab is a property of the compartment a panel is in, a body is
    // a property of its being docked at all.
    let docked = format!("{BODY}{panel}");
    if let Some(rect) = declared(&trace, ui_rect, &docked) {
        failures.push(format!(
            "`{panel}` is still drawn in the dock after the release ({rect:?}), so the outline \
             promised a window and the panel stayed where it was. A pre-commit affordance for \
             something that does not then happen is the one failure mode worse than showing \
             nothing"
        ));
    }
    let Some(id) = trace
        .events(crate::checks::driving::VIEWPORT_INNER_EVENT)
        .last()
        .and_then(|l| l.get("id").map(str::to_owned))
    else {
        failures.push(
            "no float window was created by the release, so the panel left the dock and went \
             nowhere the operator can see"
                .to_owned(),
        );
        return Ok(verdict(&failures));
    };
    report.note(format!("★★★ and a window opened for it (viewport {id})"));

    // --- 5: ★★★ and the window is the one the outline drew --------------
    //
    // Everything above is satisfied by a window of any size opening anywhere on
    // the desktop. The outline is a promise with two halves, and both are
    // checkable against what the window reports about itself.
    //
    // ★ SIZE against the INNER rectangle and ORIGIN against the OUTER one, and
    // the pairing is the whole point: `floatwin` builds the window with
    // `with_inner_size` and `with_position`, and those two egui setters speak
    // different rectangles. Checking the origin against `viewport-inner`
    // instead would fail by the host border and title bar — 8 pt across and
    // 31 pt down on this desktop — and the only way to make it pass would be to
    // put a chrome allowance in the harness, where it would be wrong on the
    // next platform and would swallow a real 30-point placement error for ever.
    let by_id = |event: &str| {
        trace
            .events(event)
            .filter(|l| l.get("id") == Some(id.as_str()))
            .filter_map(|l| l.get_rect("rect"))
            .last()
    };
    let (Some(promised), Some(at)) = (promised, promised_at) else {
        // Already reported at step 1; the window is real and the promise is not
        // readable, so there is nothing further to compare it against.
        return Ok(verdict(&failures));
    };

    match by_id(crate::checks::driving::VIEWPORT_INNER_EVENT) {
        Some(inner)
            if (inner.width() - promised.width()).abs() <= SIZE_TOLERANCE_PTS
                && (inner.height() - promised.height()).abs() <= SIZE_TOLERANCE_PTS => {}
        Some(inner) => failures.push(format!(
            "the outline offered a window {:.0}×{:.0} pts and the window that opened has a \
             {:.0}×{:.0} pt client area. The operator sized nothing — the outline is the \
             only statement of how big the panel was about to be, so a window of a \
             different size is the affordance having described something else",
            promised.width(),
            promised.height(),
            inner.width(),
            inner.height()
        )),
        None => failures.push(format!(
            "viewport `{id}` opened and published no client rectangle, so nothing says how \
             big the window the operator was promised actually is"
        )),
    }

    // ★★ THE EXPECTED DESKTOP ORIGIN IS MEASURED HERE, NOT TAKEN FROM THE
    // TRACE.
    //
    // Comparing the published `at` against the window that `at` positioned is
    // a tautology: both carry whatever the conversion produced, so a build
    // that adds the wrong application-window origin — or none — agrees with
    // itself and passes. That was measured: deleting the `+ origin` term in
    // the shell moved a real window 788 pt up and 71 pt left of the outline
    // the operator was shown, and a check written that way still said PASS.
    //
    // So the harness computes the same quantity from its own side — the
    // application window's client corner, read from the OS — and the outline
    // rectangle as the operator saw it, which is in application-window points.
    // The two inputs are then independent of each other, which is the only
    // arrangement in which agreement means anything.
    //
    // `client_origin` is desktop PIXELS and `viewport-outer` is desktop
    // POINTS, so the scale divides. At `ppp` 1 they are the same number and
    // the division is invisible; on a scaled display it is the whole
    // conversion.
    let expected = (
        promised.min.x + frame.client_origin.0 as f32 / frame.scale,
        promised.min.y + frame.client_origin.1 as f32 / frame.scale,
    );
    let off_by = ((at.x - expected.0).abs(), (at.y - expected.1).abs());
    if off_by.0 > ORIGIN_TOLERANCE_PTS || off_by.1 > ORIGIN_TOLERANCE_PTS {
        failures.push(format!(
            "the outline was drawn at ({:.0}, {:.0}) inside the application window, whose \
             client corner is at ({:.0}, {:.0}) on the desktop, so the window it promised \
             belongs at ({:.0}, {:.0}). The position handed to the window is ({:.0}, {:.0}) \
             — out by {:.0} across and {:.0} down. The outline and the window position are \
             the same rectangle in two coordinate spaces, and the conversion between them \
             is the only step that can put a real window a plausible distance from the \
             affordance that promised it",
            promised.min.x,
            promised.min.y,
            frame.client_origin.0 as f32 / frame.scale,
            frame.client_origin.1 as f32 / frame.scale,
            expected.0,
            expected.1,
            at.x,
            at.y,
            at.x - expected.0,
            at.y - expected.1
        ));
    }

    match by_id(crate::checks::driving::VIEWPORT_OUTER_EVENT) {
        Some(outer)
            if (outer.min.x - expected.0).abs() <= ORIGIN_TOLERANCE_PTS
                && (outer.min.y - expected.1).abs() <= ORIGIN_TOLERANCE_PTS =>
        {
            report.note(format!(
                "★★★ and it opened where the outline was drawn, at ({:.0}, {:.0})",
                outer.min.x, outer.min.y
            ));
        }
        Some(outer) => failures.push(format!(
            "the outline promised a window at ({:.0}, {:.0}) on the desktop and one opened \
             at ({:.0}, {:.0}) — {:.0} pt across and {:.0} pt down from where the operator \
             was shown it would be. Both are OUTER corners in desktop points, so this is \
             not the host border and title bar: the window was given a position and did \
             not take it",
            expected.0,
            expected.1,
            outer.min.x,
            outer.min.y,
            outer.min.x - expected.0,
            outer.min.y - expected.1
        )),
        None => failures.push(format!(
            "viewport `{id}` opened and published no `{}` line, so nothing says where on the \
             desktop the window went. The client rectangle cannot answer it: it is inset by \
             the host chrome, and reconciling that in this check would encode one platform \
             window frame in the instrument",
            crate::checks::driving::VIEWPORT_OUTER_EVENT
        )),
    }

    Ok(verdict(&failures))
}
