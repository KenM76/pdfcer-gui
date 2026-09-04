//! `markup_freehand_and_vertex_kinds` — the four kinds that are not
//! drag-shaped, and the **one control in this application whose availability is
//! decided by a gesture in progress**.
//!
//! # What this is about
//!
//! `FEATURES.md` carried Ink, PolyLine and Polygon as *"engine-ready, but not
//! drag-shaped; each needs its own gesture"* for the whole project. They shipped
//! on 2026-08-14 with two new gestures — a freehand trail
//! (`canvas::markup::ink`) and a run of clicks with two endings
//! (`canvas::markup::vertex`) — and neither of those gestures can be observed by
//! any unit test in the workspace, because both are **joins**:
//!
//! | # | Link | Its own test |
//! |---|---|---|
//! | 1 | the ribbon click reports the command | `egui-shell`'s `band::render_command` — yes |
//! | 2 | dispatch routes the id to a `MarkupKind` | `shell::commands::mapping` — yes |
//! | 3 | the kind arms the canvas tool | `canvas::tool::arm_markup` — yes |
//! | 4 | `press_kind` gives a vertex kind a **click** and no drag | `canvas::gesture::meaning` — yes |
//! | 5 | `canvas::interact` routes that click to `vertex::click` rather than to the selection | **nothing** |
//! | 6 | `app::conditions` publishes `markup.finishable` from the run | its own test — yes |
//! | 7 | the ribbon reads that condition and enables the control | **nothing** |
//!
//! Links 5 and 7 are call sites, and a call site's effect is observable only in
//! a running window — which is `HANDOFF.md` defect 2's structure, and the reason
//! [`crate::checks::markup_rectangle`] exists for the four-link version of the
//! same chain.
//!
//! # The six phases, and why B measures rather than asserts
//!
//! | Phase | State | Action | Expected |
//! |---|---|---|---|
//! | A | Review, Markup tab, nothing clicked | click **Finish shape** | **no** `ribbon-command-invoked` — the control is greyed |
//! | B | Freehand armed | drag across the page | `markup-tool tool=Markup(Ink)`, then `markup-commit kind=Ink raw=N kept=M` with **M < N**, then `add-markup` |
//! | C | Polyline armed | three canvas clicks, then Finish | three `markup-vertex n=1,2,3`, then `markup-finish via=command`, `markup-commit kind=PolyLine vertices=3`, `add-markup` |
//! | D | Polygon armed | **two** canvas clicks, then Finish | **no** invoke — two corners are a line drawn there and back |
//! | E | …then a third click, Finish, and Finish again | one `markup-commit kind=Polygon vertices=3`, and the **second** press authors nothing |
//! | F | **Revision cloud** armed | three clicks, then Finish | one `markup-commit kind=`**`Cloud`**` vertices=3` — and NOT `kind=Polygon` |
//!
//! ★ **Phase F asserts one field**, and one field is the whole of it. A revision
//! cloud is a `/Polygon` with `/BE` on it, so a control that armed `Polygon`
//! instead of `Cloud` would place three vertices, finish, author a legal
//! annotation, render it and add an undo entry — every observable in phases C
//! through E unchanged. The operator's only symptom is a revision cloud with no
//! scallop, which reads as an engine rendering bug. `kind=` on `markup-commit`
//! is the one place the distinction leaves the process.
//!
//! **Phase B is the only place in this project where the ink simplification can
//! be measured against a real pointer.** `canvas::markup::ink` §3.3 quotes a
//! synthetic table from a unit test, and a synthetic curve can be argued with;
//! `raw=` and `kept=` on the trace of an OS-injected drag cannot. The assertion
//! is `kept < raw`, which is what a build whose simplification did nothing fails
//! — and it fails *identically* to a working one on every other oracle, because
//! both author a perfectly valid `/Ink`.
//!
//! The drag this harness can deliver is a **straight line**
//! ([`crate::input::Driver::drag`] walks the pointer in eight increments), so
//! the reduction it measures is the easy case and the numbers are reported
//! rather than bounded. What the phase establishes is that the code path runs in
//! the real binary at all; the *quality* of the simplification is the unit
//! test's subject, and that division is stated rather than blurred.
//!
//! # ★ Phase D is the falsifier, and here is the build it catches
//!
//! Everything in A, B, C and E would pass against a build whose
//! `markup.finishable` was published **unconditionally** — or gated on
//! `doc.pages`, which is the predicate every other `markup.*` command uses and
//! therefore the one a copy-paste registration would inherit. The control would
//! be live, the press would commit, and the operator would have a Finish that
//! does nothing on almost every press: exactly what `RIBBON_IA.md` P3 forbids,
//! and exactly what `measure.finish` set the precedent for refusing.
//!
//! Phase A alone does not catch it either, and the difference is the point:
//! phase A finds the control dead when there is **no run at all**, which a
//! `doc.pages` gate would also produce if the document had no pages — and this
//! check opens one that does. Phase D finds it dead with a run **in progress**,
//! two clicks deep, at the exact moment the identical two clicks made a
//! *polyline* finishable one phase earlier. Only a predicate that really asks
//! `markup::action` — three vertices for a `/Polygon`, two for a `/PolyLine` —
//! can produce that pair of answers.
//!
//! Phase E is a second, smaller falsifier: a build whose commit did not **empty**
//! the run would author the same polygon twice from one gesture, and the operator
//! would get two identical annotations stacked exactly on top of each other —
//! indistinguishable on screen and two undo steps to remove.
//!
//! # Mouse only, and what is therefore NOT covered
//!
//! Every gesture here is a real `SetCursorPos` + `mouse_event`, because nothing
//! in this check needs a key.
//!
//! ★ **CORRECTED 2026-08-18.** These headers used to say synthetic keyboard
//! input does not reach the target window on this machine. It DOES — see
//! [`crate::checks::add_text`], which types real characters into a caret
//! draft and asserts they landed. The belief came from `Ctrl+E` producing no
//! trace, which was the dead-keymap defect (fourteen of twenty-one declared
//! chords were dispatched by nothing) misread as a property of the machine —
//! and while it stood nobody drove a chord, so nothing could contradict it.

//!
//! Two things follow, and both are on the record rather than implied by a green
//! result:
//!
//! * **The double-click ending is not driven here.** It is the ending most
//!   operators will use, and a synthetic double-click depends on two injected
//!   clicks landing inside `egui`'s double-click window with the harness's own
//!   settles in between — a timing race that would make this check flaky, and a
//!   flaky check is worse than an absent one. It is covered by
//!   `canvas::markup::vertex`'s
//!   `the_double_click_and_the_command_author_the_same_annotation`, which runs
//!   **both** endings over identical runs and compares the actions they raise —
//!   so a driven proof of one ending is a proof about the other by that test's
//!   equality.
//! * **Escape's two rungs are not driven** — abandoning a vertex run and then
//!   retiring the pen. Covered by `canvas::keys`'
//!   `escape_abandons_a_vertex_run_before_it_puts_the_markup_tool_down` alone.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the diagnostic switches did not reach the process;
//! * the page size could not be read and no `--page-size` was given;
//! * the Review segment, the Markup tab, or one of the four controls was never
//!   declared;
//! * the canvas is not showing page 1, so the harness's one known page size does
//!   not describe the page it would be clicking on;
//! * **the freehand drag produced fewer than three trail points** — the harness
//!   could not deliver enough frames of pointer movement, so `kept < raw` would
//!   be measuring the harness rather than the simplification.

use crate::checks::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, UNIMPLEMENTED_EVENT, declared,
    declared_names, list, list_str, shell_trace,
};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// **Review**, whose tab list contains Markup and whose `edit_content` is
/// absent.
///
/// The weaker claim, and [`crate::checks::markup_rectangle`]'s reason for the
/// same choice: a markup tool that works in Review works in Edit. It is also the
/// mode in which the vertex click has the least competition — Review's primary
/// button selects no content — so a build whose vertex click fell through to the
/// selection would still show *nothing happening*, which is what phase C's
/// `markup-vertex` count is for.
const MODE: &str = "review";

/// The tab carrying all four controls.
const TAB: &str = "ribbon.tab.markup";

/// The tab id the shell reports for [`TAB`].
const TAB_ID: &str = "markup";

/// The three tools and the ending, as region names and command ids.
///
/// A table rather than eight constants, because every one of them is used in the
/// same two ways — locate the rect, count the invokes — and a name/id pair that
/// drifted apart would make this check aim at one control and assert about
/// another.
const INK: (&str, &str) = ("ribbon.item.markup.ink", "markup.ink");
/// PolyLine — the open run.
const POLYLINE: (&str, &str) = ("ribbon.item.markup.polyline", "markup.polyline");
/// Polygon — the closed run, and the kind that needs one more corner.
const POLYGON: (&str, &str) = ("ribbon.item.markup.polygon", "markup.polygon");
/// Revision cloud — the closed run again, with `/BE` on the border.
///
/// ★ **The kind whose failure mode is silent**, which is why it is driven at
/// all rather than left to the unit test that already asserts its subtype. A
/// cloud IS a `/Polygon` in the file — `MarkupSpec::Cloud` writes `/Subtype
/// /Polygon` and differs only by `/BE << /S /C /I n >>` — so a build whose
/// ribbon control armed `Polygon` instead of `Cloud` would place vertices,
/// finish, author a legal annotation and render it. The operator would get a
/// polygon from the revision-cloud button, with no error, no refusal and no
/// trace line to notice. Phase F reads the **kind** out of `markup-commit`,
/// which is the one place that distinction is externally visible.
const CLOUD: (&str, &str) = ("ribbon.item.markup.cloud", "markup.cloud");
/// The ending, and the only control in this application gated on a gesture.
const FINISH: (&str, &str) = ("ribbon.item.markup.finish", "markup.finish");

/// `markup-tool tool=…` — `canvas::tool::arm_markup`'s report that the ribbon
/// press reached the canvas.
const ARM_EVENT: &str = "markup-tool";

/// `markup-vertex kind=… page=… n=… x=… y=…` — one line per placed vertex.
///
/// The only external evidence a click became a vertex: an armed vertex tool with
/// two corners and one with three are the same screenshot at any threshold, since
/// the rubber-banded segments are hairlines in the pen colour over a drawing that
/// is already full of them.
const VERTEX_EVENT: &str = "markup-vertex";

/// `markup-commit kind=… page=… …` — the shell decided to author one.
const COMMIT_EVENT: &str = "markup-commit";

/// `markup-finish via=… kind=… page=…` — **which** of the two endings asked.
const FINISH_EVENT: &str = "markup-finish";

/// `markup-declined kind=… page=… reason=…`, read only to improve a failure
/// message: `TooFewVertices` and `NoExtent` send a reader to two different places.
const DECLINE_EVENT: &str = "markup-declined";

/// `add-markup page=… n=… epoch=… disclosures=…` — the **engine** authored it.
const APPLY_EVENT: &str = "add-markup";

/// The raw trail length, and the length after simplification.
const RAW_FIELD: &str = "raw";
/// See [`RAW_FIELD`].
const KEPT_FIELD: &str = "kept";
/// How many vertices a committed run carried.
const VERTICES_FIELD: &str = "vertices";

/// The freehand drag, in page fractions: `((x0, y0), (x1, y1))`, PDF user space
/// with y measured from the bottom.
///
/// Well inside the page on every side, because a drag that left the page would
/// be clamped by the canvas and the trail would be shorter than the gesture. A
/// diagonal rather than an axis-aligned line so that both coordinates move,
/// which is what makes the intermediate points distinct rather than duplicates
/// the capture filter would drop.
const INK_DRAG: ((f64, f64), (f64, f64)) = ((0.20, 0.25), (0.62, 0.34));

/// The three corners a run is clicked out of, in page fractions.
///
/// A triangle rather than three collinear points: `markup::action` refuses a run
/// with no extent, and three points on a line have extent but are the least
/// interesting shape available. Spread wide enough that the harness's rounding to
/// whole screen pixels cannot merge two of them.
const CORNERS: [(f64, f64); 3] = [(0.30, 0.45), (0.58, 0.47), (0.44, 0.66)];

/// See the module documentation.
pub struct MarkupFreehandAndVertexKinds;

impl Check for MarkupFreehandAndVertexKinds {
    fn name(&self) -> &'static str {
        "markup_freehand_and_vertex_kinds"
    }

    fn defect(&self) -> &'static str {
        "Markup ▸ Freehand draws nothing or writes the raw pointer trail unsimplified, a click \
         with Polyline or Polygon armed places no vertex, or Finish shape is live when there is \
         nothing to finish — the three kinds `FEATURES.md` carried as engine-ready-but-not-drag- \
         shaped, and the two call sites in their chain that no unit test in the workspace can \
         observe"
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

/// How many times the shell has reported `id` invoked.
///
/// A **count**, never a presence: this check clicks four different controls and
/// presses Finish four times in one run, so "has it ever been invoked?" would be
/// answered `true` by a click made ten seconds earlier. Every assertion below is
/// a comparison of two counts taken around one click.
fn invokes(session: &Session, id: &str) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count())
}

/// Every `markup-commit` line for one kind.
fn commits<'a>(trace: &'a Trace, kind: &str) -> Vec<&'a crate::trace::TraceLine> {
    trace
        .events(COMMIT_EVENT)
        .filter(|l| l.get("kind") == Some(kind))
        .collect()
}

/// Click a ribbon tab and confirm the shell reported it.
fn click_tab(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{TAB}` region in `{MODE}`. Either this build does not \
             show that tab in this mode, or the tab strip is too narrow and it has moved into \
             the overflow menu — which this check cannot open, because a menu's contents are not \
             published as regions. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(12);
    if !shell_trace(session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(TAB_ID))
    {
        return Err(Error::new(format!(
            "the click on `{TAB}` produced no `{TAB_EVENT} tab={TAB_ID}` line. The mode click DID \
             land, so pointer input works and this is not the input channel; the likely cause is \
             that the tab moved between the frame that declared its rect and the frame that \
             received the click."
        )));
    }
    Ok(())
}

/// Locate a declared control and refuse a degenerate rectangle.
fn control(trace: &Trace, ui_rect: &str, name: &str) -> Result<LRect> {
    let rect = declared(trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the Markup tab is active and its controls publish their rects, but none of them is \
             `{name}`. This build does not have that command on that tab — check \
             `shell::manifest::markup` and `shell::commands`. Controls declared: {}.",
            list(&declared_names(trace, ui_rect, ITEM_PREFIX))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{name}` was declared at {rect:?}, which has no usable area. A click aimed at a \
             degenerate rectangle proves nothing."
        )));
    }
    Ok(rect)
}

/// Arm one of the three tools and confirm the canvas took it.
///
/// The two-oracle move [`crate::checks::markup_rectangle`] establishes, in one
/// function because this check makes it three times: the **shell** says the click
/// reached the control, and the **application** says the tool armed. A present
/// invoke with an absent `markup-tool` names the application's dispatch and
/// nothing else; an absent invoke means no click was delivered, which is a SKIP
/// rather than a failure.
fn arm(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
    debug_spelling: &str,
) -> Result<Option<String>> {
    let rect = control(&session.trace()?, ui_rect, region)?;
    let before = invokes(session, id)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(16);
    if invokes(session, id)? <= before {
        return Ok(Some(format!(
            "`{region}` DID NOT TAKE THE CLICK. It was declared at {rect:?} and the click \
             produced no `{INVOKE_EVENT} id={id}`. It is registered `enabled_when(\"doc.pages\")` \
             and a document with pages is open, so a greyed control is the wrong reading — check \
             that `shell::manifest::markup`'s Shapes group really lists it and that the id there \
             matches the one in `shell::commands`. Commands the shell reported invoked this run: \
             {}.",
            list_str(
                &shell_trace(session)?
                    .events(INVOKE_EVENT)
                    .filter_map(|l| l.get("id"))
                    .collect::<Vec<&str>>()
            )
        )));
    }
    let trace = session.trace()?;
    if !trace
        .events(ARM_EVENT)
        .any(|l| l.get("tool") == Some(debug_spelling))
    {
        return Ok(Some(format!(
            "`{id}` WAS INVOKED AND THE CANVAS TOOL DID NOT ARM. The shell traced \
             `{INVOKE_EVENT} id={id}`, so the token reached the application — and there is no \
             `{ARM_EVENT} tool={debug_spelling}` line, which `canvas::tool::arm_markup` emits \
             unconditionally the moment it is called. {} Look at `app/dispatch.rs`'s guard arm on \
             `shell::commands::markup_for_command(id).is_some()`, and at `markup_for_command` \
             itself, which is the single binding between an id and a `MarkupKind`. Tools reported \
             this run: {}.",
            if trace
                .events(UNIMPLEMENTED_EVENT)
                .any(|l| l.get("id") == Some(id))
            {
                format!(
                    "The application traced `{UNIMPLEMENTED_EVENT} id={id}`, which is \
                     `dispatch_command`'s fall-through: the command arrived and dispatch had no \
                     arm for it."
                )
            } else {
                format!(
                    "There is no `{UNIMPLEMENTED_EVENT}` either, so the command did not reach the \
                     fall-through — check `dispatch_token`'s token-to-id lookup."
                )
            },
            list_str(
                &trace
                    .events(ARM_EVENT)
                    .filter_map(|l| l.get("tool"))
                    .collect::<Vec<&str>>()
            )
        )));
    }
    Ok(None)
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. Every markup command is gated on `doc.pages`, so with nothing open all \
             four controls are correctly disabled and this check would be measuring the gate \
             rather than the feature.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is nine clicks and one drag on a real \
             canvas. Reported as SKIPPED rather than passed: a check that did not run has learned \
             nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its controls are and this check has nothing to aim at.",
            ctx.profile.name
        ))
    })?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "cannot read a page size from {}. The harness needs the page box to turn this \
                 check's fractions into points, and the page height to flip PDF y (up) into \
                 window y (down). Pass --page-size WxH. It refuses to guess: a wrong page height \
                 mirrors every click about the page centre, which lands on the page and places a \
                 plausible-looking vertex somewhere else entirely.",
                pdf.display()
            ))
        })?,
    };
    report.note(format!(
        "fixture {} — page 1 is {:.0}x{:.0} pt",
        pdf.display(),
        page.width_pt,
        page.height_pt
    ));

    // --- launch, with BOTH diagnostic channels armed -----------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("markup_shapes.trace.txt"));
    spec.pdf = Some(pdf.clone());
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
        "launched {} as pid {} with {}={} and {}={}",
        exe.display(),
        session.pid(),
        ctx.profile.diag_env.0,
        ctx.profile.diag_env.1,
        SHELL_DIAG_ENV.0,
        SHELL_DIAG_ENV.1
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the process \
             and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    let driver = Driver::new(session.window());

    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    click_tab(&session, &driver, ui_rect)?;
    report.note(format!(
        "the {MODE} segment and the Markup tab both reported their clicks"
    ));

    // ==================================================================
    // PHASE A — Finish is DEAD with nothing clicked out
    // ==================================================================
    //
    // The "before" half of the availability claim, on the SAME control phases C
    // and E will find live. Without it the run could not distinguish a control
    // that becomes reachable from one that was always live — and "always live
    // and does nothing" is what P3 forbids outright.
    let finish_rect = control(&session.trace()?, ui_rect, FINISH.0)?;
    let frame = session.frame()?;
    let before = invokes(&session, FINISH.1)?;
    driver.click_at(frame.declared_center(finish_rect))?;
    session.settle(12);
    if invokes(&session, FINISH.1)? > before {
        return Ok(Some(format!(
            "P3, IN THE OTHER DIRECTION: `{}` IS LIVE WITH NOTHING CLICKED OUT. No tool has been \
             armed this run and no vertex placed, so there is no run to finish — and the click \
             produced `{INVOKE_EVENT} id={}` anyway, which a disabled control cannot do. The \
             command is registered `enabled_when(\"markup.finishable\")` and `app::conditions` \
             publishes that name only from `canvas::markup::vertex::finishable`, so either the \
             registration lost its predicate or the condition is being published unconditionally.",
            FINISH.0, FINISH.1
        )));
    }
    report.note(format!(
        "with nothing clicked out, a press of `{}` produced no `{INVOKE_EVENT}` — the control is \
         greyed, which is the state phases C and E have to make escapable",
        FINISH.0
    ));

    // ==================================================================
    // PHASE B — Freehand: the trail, and the only real measurement of the
    //           simplification anywhere in this project
    // ==================================================================
    if let Some(failure) = arm(&session, &driver, ui_rect, INK, "Markup(Ink)")? {
        return Ok(Some(failure));
    }
    report.note("Freehand armed");

    let from = aim(
        ctx,
        &session,
        page,
        DocPoint::new(
            0,
            INK_DRAG.0.0 * page.width_pt,
            INK_DRAG.0.1 * page.height_pt,
        ),
    )?;
    let to = aim(
        ctx,
        &session,
        page,
        DocPoint::new(
            0,
            INK_DRAG.1.0 * page.width_pt,
            INK_DRAG.1.1 * page.height_pt,
        ),
    )?;
    let applies_before = session.trace()?.events(APPLY_EVENT).count();
    driver.drag(from, to)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(ink) = commits(&trace, "Ink").last().copied() else {
        return Ok(Some(format!(
            "THE FREEHAND DRAG AUTHORED NOTHING. The tool armed — `{ARM_EVENT} \
             tool=Markup(Ink)` — and a real drag across the page produced no `{COMMIT_EVENT} \
             kind=Ink`. {} `canvas::interact`'s `GestureOutcome::Markup` arm branches on \
             `kind.is_freehand()`; a build that routed this to `markup::band::drag` instead would \
             hit that function's family guard and silently draw and author nothing, which is \
             exactly this symptom. Trace: {}.",
            match trace.last(DECLINE_EVENT) {
                Some(line) => format!(
                    "The application DID trace `{}`, so the gesture ran and refused: that names \
                     the rule that rejected it rather than the routing.",
                    line.raw
                ),
                None => format!("There is no `{DECLINE_EVENT}` either."),
            },
            session.trace_path().display()
        )));
    };
    let raw = ink.get_usize(RAW_FIELD).unwrap_or(0);
    let kept = ink.get_usize(KEPT_FIELD).unwrap_or(0);
    report.note(format!("the freehand stroke committed: `{}`", ink.raw));
    if raw < 3 {
        return Err(Error::new(format!(
            "the drag produced a trail of only {raw} point(s), so `{KEPT_FIELD} < {RAW_FIELD}` \
             would be measuring the harness rather than the simplification: \
             `markup::ink::simplify` returns a run of fewer than three points unchanged, by \
             definition. `Driver::drag` walks the pointer in eight increments, so this means the \
             application saw at most two frames of movement — a window that is rendering too \
             slowly for this measurement, not a defect in Freehand. Reported as SKIPPED, because \
             a check that could not measure has learned nothing."
        )));
    }
    if kept >= raw {
        return Ok(Some(format!(
            "THE POINTER TRAIL IS WRITTEN INTO THE FILE UNSIMPLIFIED. The drag traced \
             `{RAW_FIELD}={raw} {KEPT_FIELD}={kept}` — every raw point survived. \
             `canvas::markup::ink` section 3 is the whole argument for why it must not: each \
             point costs two `Real`s in `/InkList` AND one `l` operator in the appearance stream, \
             so the cost is paid twice in the file and again on every render. The shipped \
             tolerance is `SIMPLIFY_TOLERANCE_PTS`, derived from the pen's half-width, and the \
             unit test `the_measured_retention_at_the_shipped_tolerance` measures a 87 % \
             reduction on a synthetic trail — so a build that reduced nothing here has lost the \
             call to `simplify`, not tuned it. THIS IS THE ASSERTION NO OTHER ORACLE CAN MAKE: \
             both builds author a perfectly valid /Ink and look identical on screen."
        )));
    }
    #[allow(clippy::cast_precision_loss)]
    let saved = 100.0 - (kept as f64 * 100.0 / raw as f64);
    report.note(format!(
        "★ MEASURED THROUGH THE RUNNING BINARY: the OS-injected drag produced {raw} raw trail \
         point(s) and `markup::ink::simplify` kept {kept} — {saved:.0} % discarded. The drag this \
         harness can deliver is a straight line, so this is the easy case and the figure is \
         reported rather than bounded; the quality measurement is the unit test's, on a curve \
         with a wander and a jitter on it (`canvas::markup::ink` section 3.3)"
    ));
    if session.trace()?.events(APPLY_EVENT).count() <= applies_before {
        return Ok(Some(format!(
            "THE ENGINE NEVER AUTHORED THE STROKE. The application decided to author one — \
             `{}` — and no `{APPLY_EVENT}` line followed, so `app::actions`' apply arm never ran \
             or `EditSession::add_markup` refused. A `vector_edit` that returned `Err` traces \
             `add-markup-refused` instead, so check for that line first.",
            ink.raw
        )));
    }
    report.note("the engine authored the /Ink annotation");

    // ==================================================================
    // PHASE C — Polyline: three clicks become three vertices, and Finish
    //           comes alive
    // ==================================================================
    if let Some(failure) = arm(&session, &driver, ui_rect, POLYLINE, "Markup(PolyLine)")? {
        return Ok(Some(failure));
    }
    let placed = click_out(ctx, &session, &driver, page, &CORNERS)?;
    if placed != CORNERS.len() {
        return Ok(Some(format!(
            "A CLICK WITH POLYLINE ARMED PLACED NO VERTEX. {} click(s) on the page produced \
             {placed} `{VERTEX_EVENT}` line(s). The chain is \
             `gesture::press_kind` giving a vertex kind a live click and NO drag -> \
             `canvas::interact`'s Click arm routing it to `markup::vertex::click` rather than to \
             the selection -> the vertex being converted through the page transform. The middle \
             link is the one with no test that observes it in a window: check the \
             `active_tool.markup_kind().filter(|k| k.is_vertex())` branch, which must sit ahead \
             of the selection fall-through.",
            CORNERS.len()
        )));
    }
    report.note(format!(
        "three canvas clicks placed three vertices: {}",
        vertex_lines(&session.trace()?)
    ));

    let (failure, finished) = press_finish(&session, &driver, ui_rect, report)?;
    if let Some(failure) = failure {
        return Ok(Some(failure));
    }
    if !finished {
        return Ok(Some(format!(
            "`{}` IS STILL DEAD WITH THREE CORNERS CLICKED OUT. Three `{VERTEX_EVENT}` lines were \
             traced, so a run exists on the page — and the press produced no `{INVOKE_EVENT} \
             id={}`, which means the control is still disabled. The chain is \
             `markup::vertex::click` storing the run -> `app::conditions` publishing \
             `markup.finishable` from `vertex::finishable` -> the command's `enabled_when`. The \
             middle link is the one with no test that observes it in a window.",
            FINISH.0, FINISH.1
        )));
    }
    let trace = session.trace()?;
    let Some(line) = commits(&trace, "PolyLine").last().copied() else {
        return Ok(Some(format!(
            "FINISH WAS PRESSED AND AUTHORED NOTHING. The shell traced `{INVOKE_EVENT} id={}` and \
             there is no `{COMMIT_EVENT} kind=PolyLine`. {} `canvas::markup::vertex::finish` and \
             the canvas's double-click ending share ONE commit path, so a failure here is in that \
             path or in the `\"markup.finish\"` arm of `app/dispatch.rs`.",
            FINISH.1,
            match trace.last(DECLINE_EVENT) {
                Some(l) => format!("The application DID trace `{}`.", l.raw),
                None => format!("There is no `{DECLINE_EVENT}` either."),
            }
        )));
    };
    if line.get_usize(VERTICES_FIELD) != Some(CORNERS.len()) {
        return Ok(Some(format!(
            "THE RUN AND THE ANNOTATION DISAGREE ABOUT HOW MANY CORNERS THERE WERE. {} clicks \
             were placed and the annotation was authored from `{}`. A `/PolyLine` joins \
             consecutive `/Vertices`, so a lost or duplicated entry is a different figure from \
             the one the preview drew — and only visible after saving.",
            CORNERS.len(),
            line.raw
        )));
    }
    if !trace
        .events(FINISH_EVENT)
        .any(|l| l.get("via") == Some("command"))
    {
        return Ok(Some(format!(
            "THE ANNOTATION WAS AUTHORED AND THE COMMAND ENDING DID NOT ASK FOR IT. `{}` was \
             committed and there is no `{FINISH_EVENT} via=command` line, which \
             `markup::vertex::finish` emits unconditionally on success. Something other than the \
             ribbon ending committed this run.",
            line.raw
        )));
    }
    report.note(format!(
        "★ the SAME control phase A found greyed authored a polyline: `{}`, by the command \
         ending. The double-click ending is not driven here (see the module header) and is \
         covered by the unit test that runs BOTH and compares the actions",
        line.raw
    ));

    // ==================================================================
    // PHASE D — ★ THE FALSIFIER: two corners are a polyline and are not a
    //           polygon
    // ==================================================================
    if let Some(failure) = arm(&session, &driver, ui_rect, POLYGON, "Markup(Polygon)")? {
        return Ok(Some(failure));
    }
    let placed = click_out(ctx, &session, &driver, page, &CORNERS[..2])?;
    if placed != 2 {
        return Ok(Some(format!(
            "A CLICK WITH POLYGON ARMED PLACED NO VERTEX: two clicks produced {placed} \
             `{VERTEX_EVENT}` line(s). Polyline placed three from three one phase earlier, so the \
             routing works for one vertex kind and not the other — look for a `matches!` that \
             names `PolyLine` where `MarkupKind::is_vertex` was meant."
        )));
    }
    let (failure, finished) = press_finish(&session, &driver, ui_rect, report)?;
    if let Some(failure) = failure {
        return Ok(Some(failure));
    }
    if finished {
        return Ok(Some(format!(
            "★ THE FALSIFIER FIRED: `{}` IS LIVE FOR A TWO-CORNER POLYGON. The identical two \
             clicks made a POLYLINE finishable one phase earlier, and a two-vertex `/Polygon` is \
             a line drawn there and back — `canvas::markup::action` refuses it as \
             `TooFewVertices`, and `markup::vertex::finishable` asks that same function so the \
             control cannot be live while pressing it would decline.\n\
             This is the assertion phases A, B, C and E cannot make. All four of them pass \
             against a build whose `markup.finishable` is published UNCONDITIONALLY — or gated \
             on `doc.pages`, which is the predicate every other `markup.*` command uses and \
             therefore the one a copy-paste registration inherits. Only a predicate that really \
             asks `markup::action` can answer `true` for two polyline corners and `false` for two \
             polygon corners, and that pair of answers is what this phase measures.",
            FINISH.0
        )));
    }
    report.note(format!(
        "★ FALSIFIER CLEAR: with a run two corners deep, `{}` is greyed for a Polygon where the \
         identical run left it live for a Polyline. The condition really asks \
         `markup::action`, rather than being published for any run or for none",
        FINISH.0
    ));

    // ==================================================================
    // PHASE E — the third corner finishes it, and the run is spent
    // ==================================================================
    let placed = click_out(ctx, &session, &driver, page, &CORNERS[2..])?;
    if placed != 1 {
        return Ok(Some(
            "THE THIRD CORNER PLACED NO VERTEX, so phase E cannot distinguish a run that was \
             emptied by its commit from one that was never long enough to commit."
                .to_owned(),
        ));
    }
    let commits_before = commits(&session.trace()?, "Polygon").len();
    let (failure, finished) = press_finish(&session, &driver, ui_rect, report)?;
    if let Some(failure) = failure {
        return Ok(Some(failure));
    }
    if !finished {
        return Ok(Some(format!(
            "`{}` IS STILL DEAD WITH THREE CORNERS OF A POLYGON CLICKED OUT, though the same \
             three made a polyline finishable in phase C. `markup::action` requires three \
             vertices for a `/Polygon` and it has three — so the refusal is somewhere between \
             `vertex::finishable` and the ribbon.",
            FINISH.0
        )));
    }
    let trace = session.trace()?;
    let polygons = commits(&trace, "Polygon");
    let Some(line) = polygons.last().copied() else {
        return Ok(Some(format!(
            "FINISH WAS PRESSED WITH THREE POLYGON CORNERS AND AUTHORED NOTHING. {}",
            match trace.last(DECLINE_EVENT) {
                Some(l) => format!("The application traced `{}`.", l.raw),
                None => format!("There is no `{DECLINE_EVENT}` either."),
            }
        )));
    };
    if polygons.len() != commits_before + 1 {
        return Ok(Some(format!(
            "ONE PRESS AUTHORED {} POLYGONS. Exactly one annotation per ending is the whole point \
             of there being one commit path.",
            polygons.len() - commits_before
        )));
    }
    report.note(format!("the polygon committed: `{}`", line.raw));

    // …and pressing Finish again authors nothing, because the run was emptied.
    let commits_before = polygons.len();
    let (failure, _) = press_finish(&session, &driver, ui_rect, report)?;
    if let Some(failure) = failure {
        return Ok(Some(failure));
    }
    if commits(&session.trace()?, "Polygon").len() > commits_before {
        return Ok(Some(format!(
            "★ THE SECOND FALSIFIER FIRED: A SECOND PRESS OF `{}` AUTHORED THE SAME POLYGON \
             AGAIN. `markup::vertex::commit` empties the run on success precisely so this cannot \
             happen. The failure is quiet and expensive: the operator presses Finish, sees the \
             shape land, presses it again out of habit or because they did not see the first, and \
             gets two annotations stacked exactly on top of each other — indistinguishable on \
             screen and two undo steps to remove.",
            FINISH.0
        )));
    }
    report.note(format!(
        "a second press of `{}` authored nothing: the run was emptied by its own commit",
        FINISH.0
    ));

    // ==================================================================
    // PHASE F — ★ THE REVISION CLOUD, and the one thing that distinguishes
    //           it from phase E
    // ==================================================================
    //
    // The same three corners, the same ending, one different `kind=` in the
    // commit line — and that is the whole assertion, because it is the whole
    // difference. Everything else about this tool is `Polygon`'s: the gesture,
    // the three-vertex floor, the closing segment in the preview,
    // `MarkupKind::is_vertex`.
    //
    // Which is exactly why it is driven. The defect this phase exists to catch
    // is a `markup.cloud` control that arms `MarkupKind::Polygon` — a one-token
    // slip in `shell::commands::mapping`, in a `match` whose two arms are
    // adjacent and nearly identical — and every other observable would be
    // unchanged: three vertices placed, Finish live at three and dead at two,
    // one annotation authored, one undo entry, a shape on the page. The
    // operator's only symptom is that the revision cloud has no scallop, which
    // reads as a rendering bug in the engine.
    //
    // The falsifier phase D established is **not** repeated. It is a property
    // of `markup::action`'s Polygon-or-Cloud arm, which this kind now shares —
    // the same function, the same guard, one match arm — so re-driving it would
    // be measuring the same line of code a second time and calling it coverage.
    if let Some(failure) = arm(&session, &driver, ui_rect, CLOUD, "Markup(Cloud)")? {
        return Ok(Some(failure));
    }
    let placed = click_out(ctx, &session, &driver, page, &CORNERS)?;
    if placed != 3 {
        return Ok(Some(format!(
            "A CLICK WITH REVISION CLOUD ARMED PLACED {placed} VERTEX/VERTICES OUT OF THREE. \
             Polygon placed three from the same three corners one phase earlier, so the routing \
             works for one closed-run kind and not the other — look for a `matches!` naming \
             `Polygon` where `MarkupKind::is_vertex` was meant."
        )));
    }
    let clouds_before = commits(&session.trace()?, "Cloud").len();
    let polygons_before = commits(&session.trace()?, "Polygon").len();
    let (failure, finished) = press_finish(&session, &driver, ui_rect, report)?;
    if let Some(failure) = failure {
        return Ok(Some(failure));
    }
    if !finished {
        return Ok(Some(format!(
            "`{}` IS DEAD WITH THREE REVISION-CLOUD CORNERS CLICKED OUT, though the same three \
             made a polygon finishable in phase E. `markup::action` takes Polygon and Cloud \
             through ONE arm with one three-vertex floor, so a difference here means the arm was \
             split.",
            FINISH.0
        )));
    }
    let trace = session.trace()?;
    let clouds = commits(&trace, "Cloud");
    if clouds.len() != clouds_before + 1 {
        // ★ The interesting failure, and the reason this phase exists. Read
        // the POLYGON count before blaming the commit path: a control that
        // armed the wrong kind authors an annotation perfectly well, it just
        // authors the wrong one, and every other signal in the trace looks
        // identical.
        let polygons_now = commits(&trace, "Polygon").len();
        return Ok(Some(if polygons_now > polygons_before {
            format!(
                "★ THE REVISION CLOUD AUTHORED A PLAIN POLYGON. Finish was pressed with three \
                 cloud corners clicked out and the `{COMMIT_EVENT}` line says `kind=Polygon`: \
                 the ribbon control is registered and reachable and it arms the WRONG \
                 `MarkupKind`. `shell::commands::mapping::markup_command`'s Cloud and Polygon \
                 arms are adjacent and nearly identical, and this is what a slip between them \
                 costs — a legal annotation, a rendered shape, an undo entry, and no cloudy \
                 border, which the operator will report as a rendering bug."
            )
        } else {
            format!(
                "FINISH WAS PRESSED WITH THREE REVISION-CLOUD CORNERS AND AUTHORED NO CLOUD. {}",
                match trace.last(DECLINE_EVENT) {
                    Some(l) => format!("The application traced `{}`.", l.raw),
                    None => format!("There is no `{DECLINE_EVENT}` either."),
                }
            )
        }));
    }
    report.note(format!(
        "★ the revision cloud committed as its OWN kind rather than as a polygon: `{}`",
        clouds.last().map_or("", |l| l.raw.as_str())
    ));

    // --- the picture, saved as evidence rather than asserted on -------------
    //
    // `crate::capture`'s standing rule: every check that could look at pixels
    // saves its evidence, pass or fail. It is not the oracle — a 2 pt red
    // polyline over a CAD sheet is a few hundred antialiased pixels among a
    // drawing already full of thin dark lines, and no threshold this crate has
    // separates the two — but a reader who wants to know whether the three marks
    // landed where they were aimed needs the picture, and on a failure it is the
    // first thing they will ask for.
    let shot = ctx.out("markup_shapes.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
            report.note(
                "the window carrying the freehand stroke, the polyline, the polygon and the \
                 revision cloud is saved beside the trace. Evidence, not the oracle — and note \
                 that the cloud's scallop is the one thing in it a reader COULD check by eye, \
                 because it is the only mark on the page whose border is not a straight line",
            );
        }
        Err(e) => {
            report.note(format!(
                "could not capture the window ({e}); every assertion above still stands, and they \
                 are what this check's verdict rests on"
            ));
        }
    }

    report.note(
        "not covered here: the double-click ending, and Escape's two rungs (abandon the run, then \
         retire the pen). This check does not drive them; keystrokes DO reach the window \
         and a synthetic double-click is a timing race that would make this check flaky, so both \
         are covered by unit test alone and the gap is on the record rather than implied by a \
         green result",
    );
    Ok(None)
}

/// Click a list of page fractions, and report how many new vertices the
/// application says it placed.
///
/// The count is taken as a **difference** across the clicks rather than as a
/// total, because this check clicks out three runs in one session and a total
/// would be answered by a run finished a phase earlier.
fn click_out(
    ctx: &CheckContext,
    session: &Session,
    driver: &Driver,
    page: PageGeometry,
    corners: &[(f64, f64)],
) -> Result<usize> {
    let before = session.trace()?.events(VERTEX_EVENT).count();
    for &(fx, fy) in corners {
        let at = aim(
            ctx,
            session,
            page,
            DocPoint::new(0, fx * page.width_pt, fy * page.height_pt),
        )?;
        driver.click_at(at)?;
        session.settle(12);
    }
    Ok(session.trace()?.events(VERTEX_EVENT).count() - before)
}

/// Press **Finish shape** and report whether the control took the click.
///
/// Returns `(failure, invoked)`: a `Some` failure is a control that could not be
/// located at all, and `invoked` is the availability answer every phase reads —
/// `false` means the control was greyed, which is positive evidence rather than
/// an absence, because a disabled `egui` control never reports itself invoked.
fn press_finish(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
) -> Result<(Option<String>, bool)> {
    let rect = match control(&session.trace()?, ui_rect, FINISH.0) {
        Ok(rect) => rect,
        Err(e) => return Ok((Some(e.message().to_owned()), false)),
    };
    let before = invokes(session, FINISH.1)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(20);
    let after = invokes(session, FINISH.1)?;
    report.note(format!(
        "pressed `{}` — the shell reported {} invocation(s) of `{}` after it, {before} before",
        FINISH.0, after, FINISH.1
    ));
    Ok((None, after > before))
}

/// Every `markup-vertex` line, joined, for a report note.
fn vertex_lines(trace: &Trace) -> String {
    let lines: Vec<&str> = trace
        .events(VERTEX_EVENT)
        .map(|l| l.raw.as_str())
        .collect::<Vec<&str>>();
    list_str(&lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names this check greps for are the ones `egui-shell` builds, and the
    /// ids are the ones the application registers.
    ///
    /// Pinned for the reason every sibling check pins its own: the two crates are
    /// joined by a **string** and nothing else, so a rename would leave both
    /// sides compiling while every assertion here quietly stopped matching — and
    /// a check that matches nothing passes vacuously.
    #[test]
    fn the_selectors_match_the_shells_own_spelling() {
        for (region, id) in [INK, POLYLINE, POLYGON, CLOUD, FINISH] {
            assert_eq!(region, format!("ribbon.item.{id}"));
            assert!(region.starts_with(ITEM_PREFIX), "{region}");
        }
        assert_eq!(TAB, format!("ribbon.tab.{TAB_ID}"));
        // Review, not Edit: the weaker claim, and the mode whose primary button
        // selects no content — so a vertex click that fell through to the
        // selection would visibly do nothing rather than doing something else.
        assert_eq!(MODE, "review");
        // Finish must not be one of the three tools, or phase A would be
        // measuring a tool's availability rather than the gesture condition.
        for (region, _) in [INK, POLYLINE, POLYGON, CLOUD] {
            assert_ne!(region, FINISH.0);
        }
    }

    /// ★ **The fixture geometry is what the phases claim it is** — three corners
    /// that are not collinear, and two of them that are a legal polyline.
    ///
    /// Phase D rests entirely on `CORNERS[..2]` being a run a **polyline** would
    /// accept and a **polygon** would not. A fixture whose first two corners were
    /// coincident would make phase D pass for the wrong reason: the control would
    /// be greyed because the run had no extent, not because a polygon needs three
    /// corners, and the falsifier would be measuring nothing.
    #[test]
    fn the_corners_are_a_real_triangle_and_the_first_two_are_a_real_line() {
        let [a, b, c] = CORNERS;
        assert!(
            (a.0 - b.0).abs() > 0.05 || (a.1 - b.1).abs() > 0.05,
            "the first two corners must be far enough apart to be a line with extent"
        );
        // Twice the signed area: non-zero means not collinear.
        let area = (b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1);
        assert!(area.abs() > 0.01, "the three corners are nearly collinear");
        for (x, y) in CORNERS.into_iter().chain([INK_DRAG.0, INK_DRAG.1]) {
            assert!(
                (0.05..=0.95).contains(&x) && (0.05..=0.95).contains(&y),
                "({x}, {y}) is too close to the page edge to survive a margin"
            );
        }
    }

    /// The drag is diagonal, so both coordinates move.
    ///
    /// `markup::ink` drops a sample identical to the one before it, so a drag
    /// along one axis whose other coordinate never changes would still produce
    /// distinct points — but a drag that moved in neither would produce exactly
    /// one, and the `raw < 3` SKIP would fire on every run. Asserted so the
    /// fixture cannot quietly become degenerate.
    #[test]
    fn the_freehand_drag_moves_on_both_axes() {
        assert!((INK_DRAG.0.0 - INK_DRAG.1.0).abs() > 0.1);
        assert!((INK_DRAG.0.1 - INK_DRAG.1.1).abs() > 0.05);
    }

    /// The two channels are parsed out of one file without contaminating each
    /// other, and the two fields phase B measures are read from the line the
    /// application really writes.
    #[test]
    fn the_application_and_shell_streams_do_not_contaminate_each_other() {
        let text = "pdfcer-diag start argv1=None\n\
                    egui-shell-diag ribbon-command-invoked id=markup.ink handler=505\n\
                    pdfcer-diag markup-tool tool=Markup(Ink)\n\
                    pdfcer-diag markup-commit kind=Ink page=0 raw=14 kept=2\n\
                    pdfcer-diag markup-vertex kind=PolyLine page=0 n=1 x=10.00 y=20.00\n\
                    pdfcer-diag markup-commit kind=PolyLine page=0 vertices=3 x0=1.00 y0=2.00 \
                    xn=3.00 yn=4.00";
        let app = Trace::parse(text, "pdfcer-diag");
        let shell = Trace::parse(text, driving::SHELL_TRACE_PREFIX);

        assert!(app.started("start"));
        assert!(
            app.events(INVOKE_EVENT).next().is_none(),
            "the shell's line must not be read as the application's"
        );
        assert!(
            shell
                .events(INVOKE_EVENT)
                .any(|l| l.get("id") == Some(INK.1))
        );
        assert!(shell.events(ARM_EVENT).next().is_none());

        let ink = commits(&app, "Ink");
        assert_eq!(ink.len(), 1);
        assert_eq!(ink[0].get_usize(RAW_FIELD), Some(14));
        assert_eq!(ink[0].get_usize(KEPT_FIELD), Some(2));
        // …and the kind filter really filters: a polyline commit must not be
        // read as an ink one, or phase B would measure `vertices=` as `raw=`
        // and find it missing.
        assert_eq!(commits(&app, "PolyLine").len(), 1);
        assert_eq!(commits(&app, "Polygon").len(), 0);
        assert_eq!(
            commits(&app, "PolyLine")[0].get_usize(VERTICES_FIELD),
            Some(3)
        );
        assert_eq!(app.events(VERTEX_EVENT).count(), 1);
    }
}
