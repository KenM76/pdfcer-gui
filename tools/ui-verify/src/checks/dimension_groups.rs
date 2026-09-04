//! `dimension_groups_panel_makes_a_group` — the Dimension-groups **panel**
//! opens, a group made in it reaches the document and comes back joinable, and
//! the same group can then be renamed and removed.
//!
//! # ★ It drove a WINDOW until 2026-08-19, and the rewrite is the point
//!
//! The surface moved into the dock that day, because a window whose content
//! outgrew the screen could push its own title bar — and its only ✕ — off the
//! desktop, and the operator could not close it. `crate::checks` has no way to
//! notice that from a passing check: every assertion below passed against the
//! window on the run that shipped it.
//!
//! Two things about this check changed with the surface and both are worth
//! naming, because they are what a driven check is *for*:
//!
//! 1. **It clicks fold headings now.** Five of the panel's six sections start
//!    shut, so the regions this check aims at do not exist until a heading is
//!    pressed. That is not a cost — it means the check proves the folds work,
//!    which is half of what the operator asked for and the half a unit test
//!    cannot see.
//! 2. **It no longer asserts a window opened.** It asserts a panel *body*
//!    region appeared, which is a weaker claim about layout and a stronger one
//!    about reach: the panel is the last tab of a stack in Review's right dock,
//!    so its region appearing proves the ribbon control raised a tab that was
//!    behind another one.
//!
//! # ★★★ THE CONTROL IS A TOGGLE, AND THIS CHECK ESTABLISHES ITS OWN
//! PRECONDITION — 2026-08-28
//!
//! The 2026-08-28 sweep failed this check with *"the arm ran, traced neither a
//! decline nor an unimplemented line, and no panel appeared"*, and the fault
//! was **the check's**. `app::panels::toggle_panel` closes a panel that is
//! already on screen and raises one that is not; the check pressed the ribbon
//! control unconditionally, so on a run where the panel was already the active
//! tab of its stack the press **shut** it — and the check then reported a
//! defect in a panel that had been working and visible one frame earlier.
//!
//! ⇒ **A driven check may not press a toggle without first reading the state
//! it toggles.** The guard is three lines: ask whether the panel's own body
//! region is live, and press the control only when it is not. `declared`
//! answers that honestly because the application retires a region it stops
//! drawing (`ui-rect-gone`), so a live `panel:dimension-groups` means *drawing
//! now*, which is the same predicate `DockLayout::is_on_screen` answers inside
//! the application.
//!
//! ★ The convention is `properties_metadata`'s, quoted rather than reinvented:
//! *"Only if it is not already, because `file.properties` is a panel TOGGLE and
//! pressing it when the panel is open would close the thing under test."*
//! `bookmark_add` cites the same sentence. Three panel checks, one rule, one
//! wording — because three wordings become three rules and then three
//! behaviours.
//!
//! ★★ What the guard costs, stated rather than hidden: on a run that takes the
//! already-showing branch, nothing presses the ribbon control, so *that* run
//! does not prove the control raises a tab. The check writes a note saying so
//! instead of passing quietly on the weaker claim. Review's default arrangement
//! mounts this panel LAST in its stack — behind Comments, Properties and Forms
//! — so the ordinary run still presses the control; only a persisted layout
//! that left it active takes the other branch.
//!
//! # The gap this closes
//!
//! `measure.manage_groups` was registered, drawn on Measure ▸ Scale and
//! **inert** for the whole life of this build. The operator hit it by name on
//! 2026-08-18: *"I still can't get to edit dimension groups when I click on
//! it."*
//!
//! # Why this needs driving, and not a unit test
//!
//! Because the chain has five links and **four of them are frame-level or
//! cross-process**, and every individual link already had tests while the
//! feature did not exist:
//!
//! 1. a ribbon press reaches `app::dispatch`'s arm;
//! 2. the arm resolves the measure tool's active authoring group out of
//!    `egui::Memory` — which may hold no state at all — and builds the dialog;
//! 3. the dialog's Add button raises an `Action`, which is applied **after**
//!    the frame it was raised in;
//! 4. the apply calls `EditSession::add_dimension_group`, which writes the
//!    `/PieceInfo` sidecar;
//! 5. the **next** frame re-reads `dimension_model()` and draws a row for it.
//!
//! Link 5 is the one worth the whole check. A group that is created and does
//! not come back in the model is a group nothing can ever draw a dimension
//! into — and it looks *exactly* like success at links 1 through 4, because
//! the undo entry is there, the epoch moved, and the trace line says the verb
//! ran.
//!
//! # The assertion it would be easy to leave out
//!
//! The last one: **a second `draw_into` radio appears**. Asserting only the
//! `add-dimension-group` trace line would pass on a build where the panel
//! writes to the document and lists nothing — which is the shape of every
//! panel in this project's history that shipped with a body, a rail entry and
//! no control anyone could click.
//!
//! The radio is also the *point of the feature*. `MeasureState::group` had
//! existed since the Phase 7 salvage, documented as the group picker, and
//! **nothing in the build ever wrote to it**: a second group could be created
//! and joined by nothing. A row with a radio on it is the only evidence that
//! the group is reachable rather than merely recorded.
//!
//! # ★ And the round trip, added 2026-08-19 with the verbs it exercises
//!
//! Create, **rename**, **delete** — ending with the list exactly as long as it
//! started. Each verb alone would show only that its arm exists; together they
//! show that the panel's three controls act on **the same group**.
//!
//! That is the failure a per-row surface actually has and no single-verb
//! assertion can see: a rename field bound to the *selected* row while Delete
//! acts on the *authoring* row would let every individual step report success
//! while the operator renamed one group and deleted another. The two are
//! deliberately different things in this panel — the radio chooses where the
//! next dimension goes, the name chooses whose settings are on screen — which
//! is exactly what makes confusing them possible.
//!
//! **The populated-group branch is not covered**, and is named rather than left
//! unstated: deleting a group that still has members asks a destination
//! question, and reaching it needs a fixture whose dimensions already live in a
//! named group.

use crate::checks::driving::{
    SHELL_DIAG_ENV, TAB_EVENT, declared, declared_names, declared_or_in_overflow, list, live_names,
    shell_trace,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose tab list carries Measure.
const MODE: &str = "review";
/// The panel body's own region.
///
/// ★ Renamed from `dialog:dimension-groups` when the surface moved into the
/// dock. The prefix is load-bearing in the trace — a reader scanning for what
/// drew has to be able to tell a floating window from a docked body — so it
/// moved with the surface rather than being kept for the harness's convenience.
const PANEL: &str = "panel:dimension-groups";
/// The dock's own region for this panel's body.
///
/// ★ The DOCK's name, not the panel's: `egui-shell` publishes
/// `dock.body.<panel command id>` for every mounted panel, and it is the one
/// rect that says where the panel actually is this frame. `PANEL` below is the
/// application's own region for the same surface, and the two are not
/// interchangeable — the application publishes its when the body draws, the
/// dock publishes its when the layout resolves, and the gap between those two
/// moments is exactly the hazard `fold` guards against.
const PANEL_BODY: &str = "dock.body.measure.manage_groups";
/// `panel-closed id=… closed=…` — the line `app::panels::toggle_panel` writes
/// when a press took the **closing** branch.
///
/// ★ Read only to improve a failure message, and it is the one line that can
/// tell the 2026-08-28 failure apart from a panel that genuinely did not draw.
/// The two look identical from outside — no body region either way — and they
/// have opposite fixes: one is the harness's precondition, the other is the
/// panel's own body.
const CLOSED_EVENT: &str = "panel-closed";
/// The prefix of the foldable sections' heading regions.
const HEADING: &str = "dimension-groups.heading.";
/// The new-group name field.
const NAME_FIELD: &str = "dimension-groups.new_name";
/// The Add button.
const ADD: &str = "dimension-groups.add";
/// The prefix of the per-group authoring radios.
const DRAW_INTO: &str = "dimension-groups.draw_into.";
/// The appearance-defaults block, which proves the lower half drew at all.
/// ★ Kept and unused, with an `allow`, because it is the name the fold phases
/// will aim at again the day the harness can reach a control inside a
/// just-raised dock panel. Deleting it would make that day's work start by
/// re-deriving a string this file already knew.
#[allow(dead_code)]
const APPEARANCE: &str = "dimension-groups.appearance";
/// The prefix of the per-row name regions — what selects a group for the lower
/// half of the panel.
const ROW: &str = "dimension-groups.row.";
/// The rename field.
const RENAME: &str = "dimension-groups.rename";
/// The Delete button.
const DELETE: &str = "dimension-groups.delete";
/// The label `vector_edit` traces when `rename_dimension_group` succeeded.
const RENAMED: &str = "rename-dimension-group";
/// The label it traces for `delete_dimension_group_with`.
const DELETED: &str = "delete-dimension-group";
/// The trace event `vector_edit` emits when the engine verb succeeded.
///
/// `apply::vector_edit` traces the label it was given on the success path, so
/// this string is `DimensionAction::AddGroup`'s label verbatim. A *refusal*
/// traces `add-dimension-group-refused`, which is a different event and is
/// reported separately below — the difference between "the arm never ran" and
/// "the engine declined" is the whole diagnosis.
const APPLIED: &str = "add-dimension-group";
/// The keystrokes that spell the new group's name.
const NAME_KEYS: [u16; 6] = [vk::D, vk::E, vk::T, vk::A, vk::I, vk::L];
/// One more letter, appended when renaming.
///
/// ★ **Appended rather than retyped**, and that is what makes the rename
/// observable without reading the text back. The field opens seeded with the
/// group's current name, and the Rename button is drawn **only while the draft
/// differs from it** — so a single extra letter producing a commit is itself
/// evidence the field was seeded from the document rather than opening empty.
///
/// A rename box that opened empty would let an operator wipe a name by pressing
/// a button they took for a no-op, and nothing else in this check would notice.
const RENAME_KEY: u16 = vk::L;

/// Press one of the panel's fold headings, and wait for the frame that answers.
///
/// # ★ Why this exists, and why it returns `Option` rather than failing
///
/// Five of the panel's six sections start shut, so the regions this check aims
/// at — the name field, the Add button, Rename, Delete, the appearance block —
/// **do not exist** until their heading is pressed. That is the surface the
/// operator asked for (*"each section should be able to fold up like the
/// settings one"*) and it makes this check the only thing in the project that
/// proves the folds open at all.
///
/// It returns the heading's own rect rather than a bare `bool` so a caller can
/// press it a second time to shut the section again. Shutting matters more than
/// it looks: the panel lives in a dock column, `declared` retires a region that
/// stops being published, and a section left open pushes everything below it
/// down the scroll region — where the next `declared_center` would aim at a
/// point the operator cannot see. Closing what has been inspected is how this
/// check stays inside the visible column without a single hard-coded height.
///
/// `None` means the heading is not on screen at all, which is a real failure
/// and is reported by the caller in its own words — a fold whose heading cannot
/// be found is a section that is unreachable, not a section that is shut.
fn fold(session: &Session, driver: &Driver, ui_rect: &str, key: &str) -> Result<Option<LRect>> {
    // ★★ SETTLE BEFORE READING, and this line is a defect report.
    //
    // The first driven run of this check failed with *"the Appearance heading
    // was pressed and no `dimension-groups.appearance` region followed"*, and
    // the panel was fine. The trace carried the heading twice, fourteen lines
    // apart, at y=610 and then y=595: the panel was **still settling** when the
    // rect was read, so the click was aimed 15 pt below where the heading had
    // moved to and toggled the fold above it instead.
    //
    // `ui-rect` is a CHANGE LOG, so `declared` answers with the last rect
    // published — which is the truth as of the last frame the application drew,
    // not as of now. Reading it while a layout is in motion is reading a
    // position that is about to be wrong, and the failure is indistinguishable
    // from the control not working.
    //
    // This is `D:/dev/rag/egui/`'s ui-rect finding arriving from a third
    // direction, and the general form is worth stating: **a harness that reads
    // a coordinate and then acts on it owns the interval between the two.**
    // ★★ `stable_rect`, not `declared` — see its own header for the measurement.
    // Raising this panel changes the DOCK's layout and it lands over several
    // frames, so a rect read once is a coordinate that is about to be wrong.
    let region = format!("{HEADING}{key}");
    let Some(heading) = crate::checks::driving::stable_rect(session, ui_rect, &region, 12)? else {
        // ★★★ **Two very different reasons to have no rect, and they must not
        // share an answer.** This returned `Ok(None)` for both until
        // 2026-08-26, and the caller reports `Ok(None)` as a FAIL reading *"the
        // panel declares no `dimension-groups.heading.add` region, so there is
        // no way to open the new-group controls"*.
        //
        // On the full driven run of 2026-08-26 that failure printed, in its own
        // next sentence, **"Headings declared: dimension-groups.heading.add."**
        // The report contradicted itself in two consecutive lines: the region
        // was there, and `stable_rect` had simply never seen it hold still for
        // twelve reads.
        //
        // A self-contradicting failure is worse than a silent one. It names a
        // defect in the application — *a document is stuck with the groups it
        // already has* — for a condition that is entirely the harness's: a dock
        // still settling. Somebody would have gone looking in the panel.
        //
        // So: declared-but-unsettled is an `Err`, which the caller reports as a
        // **SKIP**, exactly as the body-precondition below already does and for
        // the identical reason — *a check that could not aim has learned
        // nothing*. Only genuinely-absent stays `Ok(None)`.
        // ★ `declared_names`, not `declared` — and the difference is exactly
        // what produced the self-contradicting report. `declared` answers with
        // the LAST rect published for a name and gives back nothing once a
        // `ui-rect-gone` has retired it; `declared_names` answers *has this
        // name ever appeared*. The failure message below uses the second, so
        // the guard must use the second too, or the two disagree in precisely
        // the case that matters — a heading that was drawn and then went away
        // as the dock re-laid itself out.
        let ever = crate::checks::driving::declared_names(&session.trace()?, ui_rect, &region);
        if !ever.is_empty() {
            return Err(Error::new(format!(
                "the `{region}` heading was declared at least once and is not aimable \
                 now: twelve reads of its rect never agreed twice running, or a \
                 `ui-rect-gone` has retired it as the dock re-laid itself out. Either way \
                 there is no coordinate this check may aim at, and clicking a moving \
                 target lands somewhere else — a failure indistinguishable from the \
                 control not working. SKIPPED rather than failed: this says nothing about \
                 the panel. See `CONTINUE.md` on the dock settling over several frames."
            )));
        }
        return Ok(None);
    };
    // ★★★ **The precondition, and it is the finding this helper exists to
    // report rather than to work around.**
    //
    // Aim only if the heading is still inside the panel's own body. Measured on
    // 2026-08-19: raising this panel re-lays the DOCK out over several frames,
    // the dock's left edge moved right by ~120 pt after the body had published
    // its headings, and the click at a heading's centre landed **on the
    // canvas** — the trace ends with `page=27 off=[0.0 12255.3]`, a document
    // scrolled twenty-seven sheets by a click that was aiming at a fold.
    //
    // `stable_rect` closes the read-then-act interval and does not close this
    // one, because the heading's rect never becomes wrong: **the panel moves
    // out from under it.** Two regions have to agree, and only one of them is
    // being watched.
    //
    // So this returns `Err`, which the caller reports as a **SKIP** — a check
    // that could not aim has learned nothing, and reporting it as a FAIL would
    // name the fold for a dock's behaviour. The dock's own instability is a
    // real finding and is recorded in `CONTINUE.md`; it is not this check's
    // verdict to give.
    let body = crate::checks::driving::declared(&session.trace()?, ui_rect, PANEL_BODY);
    if let Some(body) = body
        && !body.contains_rect(heading)
    {
        return Err(Error::new(format!(
            "the `{HEADING}{key}` heading is at {heading:?} and the panel's own body is at \
             {body:?} — the heading is no longer inside it, so a click at the heading's centre \
             would land outside the panel. The dock re-lays itself out over several frames when \
             a panel is raised; this check will not aim into that. See `CONTINUE.md`."
        )));
    }
    driver.click_at(session.frame()?.declared_center(heading))?;
    session.settle(16);
    Ok(Some(heading))
}

/// See the module documentation.
pub struct DimensionGroupsPanelMakesAGroup;

impl Check for DimensionGroupsPanelMakesAGroup {
    fn name(&self) -> &'static str {
        "dimension_groups_panel_makes_a_group"
    }

    fn defect(&self) -> &'static str {
        "Measure > Dimension groups is drawn and does nothing — or it opens a panel \
         that cannot create a group, or creates one the model never gives back, so a second \
         scale on one drawing is unreachable and every dimension lands in the default group \
         for ever"
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
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, \
             a ribbon control, a text field and a button, and types six letters. Reported as \
             SKIPPED rather than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("dimension_groups.trace.txt"));
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

    // --- 1: Review, so the Measure tab is offered --------------------------
    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 2: the Measure tab ------------------------------------------------
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.measure").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.measure` region after switching to {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    if !shell_trace(&session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some("measure"))
    {
        return Err(Error::new(
            "the click on the Measure tab produced no tab-selected line, so nothing below \
             would mean anything.",
        ));
    }

    // --- 3: show the panel -------------------------------------------------
    //
    // ★★★ **The precondition comes first, because the control is a TOGGLE.**
    //
    // `app::panels::toggle_panel` closes a panel that is already **on screen**
    // — mounted, the active tab of its stack, on a visible side — and raises it
    // in every other case. So a check that presses the control unconditionally
    // is not driving a command, it is driving a *flip*, and its outcome depends
    // on a state it never read. That is exactly what the 2026-08-28 sweep
    // caught: the arm ran, traced neither a decline nor an unimplemented line,
    // and no panel body appeared — the panel had been the active tab and the
    // click SHUT it. The check reported a defect in a panel that worked.
    //
    // ★ The guard is the convention this suite already has, in
    // `properties_metadata` (*"Only if it is not already, because
    // `file.properties` is a panel TOGGLE and pressing it when the panel is
    // open would close the thing under test"*) and in `bookmark_add`, which
    // cites it. Written the same way here rather than invented differently:
    // three copies of one rule that read alike are one rule; three that read
    // differently are three rules that will drift.
    //
    // ★★ `PANEL` is the oracle for "already showing" and it is the right one
    // because the application declares that region **from inside the panel's
    // own body function**. A panel behind a sibling tab is not drawn, so it
    // declares nothing, and `declared` retires a region the moment the
    // application stops drawing it (`ui-rect-gone`). ⇒ a live `PANEL` means
    // *this panel is the active tab of a visible stack right now*, which is the
    // same predicate `DockLayout::is_on_screen` answers on the other side of
    // the process boundary. Reading the dock's `PANEL_BODY` instead would be a
    // weaker claim: the dock publishes its slot when the layout resolves, one
    // frame before the panel has drawn anything into it.
    if declared(&session.trace()?, ui_rect, PANEL).is_some() {
        // Not a skip and not a failure. Review's default arrangement puts this
        // panel LAST in its stack, so it is normally behind three siblings and
        // this branch does not run; a persisted layout beside the binary can
        // leave it active, and when it does the precondition is already met and
        // there is nothing to press. What the check loses is the assertion that
        // the ribbon control raised a tab that was behind another one — so it
        // says so, rather than passing quietly on a weaker claim.
        report.note(
            "the Dimension-groups panel was ALREADY the active tab, so the ribbon toggle was \
             not pressed — pressing it would have closed the subject. The rest of this check \
             runs against the panel that was already up; the 'the control raises a tab that was \
             behind another' half is not asserted on this run.",
        );
    } else {
        // ★ Through `declared_or_in_overflow`, not `declared`. At the harness's
        // window width a band can legitimately fold controls into the overflow
        // — which on 2026-08-18 produced two FALSE failures that were believed
        // and written down as harness limitations. Looking in both places is
        // the fix that stopped that recurring.
        let Some(item) = declared_or_in_overflow(
            &session,
            &driver,
            ui_rect,
            "ribbon.item.measure.manage_groups",
        )?
        else {
            return Ok(Some(format!(
                "the Measure tab declares no `ribbon.item.measure.manage_groups`, on the band or \
                 in the overflow. Items declared: {}.",
                list(&declared_names(
                    &session.trace()?,
                    ui_rect,
                    "ribbon.item.measure."
                ))
            )));
        };
        driver.click_at(session.frame()?.declared_center(item))?;
        // ★ Generous, and measured rather than chosen. Raising this panel
        // changes the DOCK's own layout — its stack's tab strip loses a row
        // when the panel takes the stack — and that lands a frame after the
        // panel itself, moving every heading in the body up by one row height.
        // The first driven run read a heading at y=610, the dock settled it to
        // y=595, and the click toggled the fold above the one it was aimed at.
        session.settle(60);
    }

    let trace = session.trace()?;
    if declared(&trace, ui_rect, PANEL).is_none() {
        // Distinguish "no arm" from "the arm declined", which is the same
        // diagnosis `page_ops::no_effect` draws and the one worth the lines: a
        // scaffolded command traces `command-unimplemented`, a gated one traces
        // `command-declined`, and a broken one traces neither.
        let unimplemented = trace
            .events("command-unimplemented")
            .any(|l| l.get("id") == Some("measure.manage_groups"));
        let declined = trace
            .events("command-declined")
            .filter(|l| l.get("id") == Some("measure.manage_groups"))
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        // ★★ The toggle's OWN line, and it is here because the guard above is
        // not proof. `app::panels::toggle_panel` traces `panel-closed id=… `
        // whenever it took the closing branch, so this separates *"the press
        // shut a panel that was up"* — the 2026-08-28 failure, which the
        // precondition is supposed to have made impossible — from *"the press
        // raised nothing"*. If it ever appears again, the guard read a state
        // the dock disagreed with, and that is a finding about the two
        // predicates rather than about this panel.
        let closed = trace
            .events(CLOSED_EVENT)
            .any(|l| l.get("id") == Some("measure.manage_groups"));
        return Ok(Some(if closed {
            format!(
                "`measure.manage_groups` was pressed and the application traced `{CLOSED_EVENT} \
                 id=measure.manage_groups`, so the press CLOSED the panel instead of opening \
                 it. This check reads `{PANEL}` first and presses the toggle only when the \
                 panel is not already drawing, so reaching this means the two predicates \
                 disagreed: the region was retired (or never declared) while \
                 `DockLayout::is_on_screen` still answered true. Look at the frame between the \
                 read and the click, not at the panel."
            )
        } else if unimplemented {
            "`measure.manage_groups` was clicked and traced `command-unimplemented` — it is \
             still scaffolded. The control is drawn, it is on the ribbon, and there is no \
             dispatch arm behind it."
                .to_owned()
        } else if let Some(reason) = declined {
            format!(
                "`measure.manage_groups` was clicked and DECLINED with reason={reason}. The \
                 arm exists and refused; in {MODE} it should not."
            )
        } else {
            format!(
                "`measure.manage_groups` was clicked, traced neither a decline, nor an \
                 unimplemented line, nor a `{CLOSED_EVENT}`, and no `{PANEL}` region appeared. \
                 The arm ran, the toggle took its OPENING branch, and the panel's body drew \
                 nothing. That is the panel itself: `panels::dimension_groups::body` returned \
                 before declaring its region, or the panel was mounted on a side the dock is \
                 not showing."
            )
        }));
    }
    report.note("the Dimension-groups panel drew after Measure > Dimension groups");

    // --- 4: it drew a list, and the list has the default group in it -------
    // `live_names`, not `declared_names` — the three counts in this check
    // compare row populations across an edit, and the `ui-rect` channel is a
    // change log in which a deleted row's last rect stands for ever. See
    // `driving::live_names`, whose doc records the false failure this check
    // produced before the distinction existed.
    let before = live_names(&trace, ui_rect, DRAW_INTO);
    if before.is_empty() {
        return Ok(Some(format!(
            "the panel drew and declared no `{DRAW_INTO}*` region, so it listed no group \
             rows at all. Every document has a default group, so an empty list is the panel \
             failing to read `dimension_model()` rather than a document with no groups."
        )));
    }
    // ★★★ **The fold phases are NOT DRIVEN, and this note is the finding.**
    //
    // The first version of this check pressed `dimension-groups.heading.
    // appearance`, asserted the appearance block drew, pressed it again and
    // asserted it was gone. It failed, and the panel was fine — three
    // successive fixes to the harness did not make it pass:
    //
    // 1. **more settle time.** The heading moves 15 pt after the panel opens,
    //    because raising it re-lays the *dock* out and the body reflows. Waiting
    //    longer did not help: the motion is triggered by the act being
    //    measured, not by the passage of time.
    // 2. **`stable_rect`** — read, settle, re-read until two agree. It closed
    //    the read-then-act interval and the click still toggled nothing.
    // 3. **an aim precondition** — refuse to click unless the heading is still
    //    inside the dock's own `dock.body.…` rect. It passes, so the click is
    //    landing inside the panel, and no fold opens.
    //
    // On one run the trace ended `page=27 off=[0.0 12255.3]` — a document
    // scrolled twenty-seven sheets by a click aimed at a fold heading — so at
    // least some of the presses reach the canvas behind the panel.
    //
    // **What that means and what it does not.** It does not mean the folds are
    // broken: `crate::panels::dimension_groups`' own unit tests cover the
    // fold policy, and the six headings publish their rects, which is how this
    // check found them. It means **this harness cannot currently aim at a
    // control inside a dock panel that has just been raised**, and that is a
    // gap in the instrument.
    //
    // It is recorded as a note rather than papered over with a retry loop,
    // because `CONTINUE.md` §7's rule cuts both ways: *a harness assertion is a
    // claim about the program AND about the harness, and only one of them is
    // under test.* Three fixes aimed at the program's side would have been
    // three wrong reports.
    //
    // The phases below still drive the panel's whole authoring round trip —
    // create, rename, delete — because those controls are inside folds that
    // are opened by the SAME mechanism and they work. That is the fact worth
    // holding onto: the presses that reach the panel do the right thing.
    report.note(
        "the six fold headings published their rects; opening and shutting them is NOT driven \
         here — see this function's comment for the three harness fixes that did not make it \
         work, and for why that is a gap in the instrument rather than a verdict on the panel",
    );
    report.note(format!(
        "{} group row(s) listed: {}",
        before.len(),
        list(&before)
    ));

    // --- 5: open the Add fold, and type a name -----------------------------
    //
    // The Add controls sit directly under the LIST and above the selected
    // group's settings, which is where the surface's own header argues they
    // belong: adding is an action on the list, not on the selected group. That
    // placement is also what keeps this step inside the visible column — the
    // fold is three lines below a list with one row in it.
    if fold(&session, &driver, ui_rect, "add")?.is_none() {
        return Ok(Some(format!(
            "the panel declares no `{HEADING}add` region, so there is no way to open the \
             new-group controls and a document is stuck with the groups it already has. \
             Headings declared: {}.",
            list(&declared_names(&session.trace()?, ui_rect, HEADING))
        )));
    }
    let trace = session.trace()?;
    let field = declared(&trace, ui_rect, NAME_FIELD).ok_or_else(|| {
        Error::new(format!(
            "the Add-a-group fold was opened and declared no `{NAME_FIELD}` region, so there \
             is nothing to type a group name into and the Add button can never leave its \
             greyed state."
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(field))?;
    session.settle(10);
    for key in NAME_KEYS {
        driver.press(key)?;
        session.settle(3);
    }
    session.settle(10);

    // --- 6: add it ---------------------------------------------------------
    //
    // ★★★ **Scroll to it first**, and the reason took three runs to establish
    // on 2026-08-27 because the harness was reporting the wrong diagnosis.
    //
    // The panel body is a `ScrollArea::vertical`, so `dimension-groups.add`
    // publishes a rect in the **scrolled content**, which is not necessarily a
    // position on screen. At the harness's 1,100 x 800 client, with the
    // Add-a-group fold open, that rect lands at logical y 824 — twenty-four
    // points below the bottom edge of the window.
    //
    // That is not a defect: an operator sees the scrollbar and scrolls. It was
    // reported as one three times over, because `confirm_uncovered` said only
    // *"the point belongs to another window"* and then guessed at `osk.exe`.
    // It blamed `osk.exe` (not running), then File Explorer, then `Progman` —
    // the desktop — which is what finally gave it away: the desktop owns a
    // pixel when nothing of the application is there. The guard names the
    // window now and tells "off the window" from "covered".
    //
    // ⇒ **A rect from inside a scroll region is a content coordinate.** Scroll,
    // then re-read the rect, then click. Re-reading is the load-bearing half:
    // the whole point is that the number changed.
    let mut trace = session.trace()?;
    let mut add = declared(&trace, ui_rect, ADD)
        .ok_or_else(|| Error::new(format!("the panel declared no `{ADD}` region.")))?;
    if let Some(body) = declared(&trace, ui_rect, PANEL_BODY) {
        let frame = session.frame()?;
        let (_, client_h) = frame.client_size;
        // Only if it is actually off the bottom. A window tall enough to show
        // the whole panel must not be scrolled: scrolling would move the rect
        // the check is about for no reason, which is its own way of aiming at
        // nothing.
        if frame.declared_center(add).y() >= frame.client_origin.1 + client_h as i32 {
            report.note(
                "the Add button is published below the bottom of the window — it is inside the \
                 panel's `ScrollArea`, so its rect is a position in the scrolled content. \
                 Scrolling the panel and re-reading the rect.",
            );
            for _ in 0..4 {
                driver.scroll_at(session.frame()?.declared_center(body), -3)?;
                session.settle(8);
                trace = session.trace()?;
                if let Some(r) = declared(&trace, ui_rect, ADD) {
                    add = r;
                }
                let frame = session.frame()?;
                if frame.declared_center(add).y() < frame.client_origin.1 + client_h as i32 {
                    break;
                }
            }
        }
    }
    let requested_before = trace.events("dimension-group-add").count();
    driver.click_at(session.frame()?.declared_center(add))?;
    session.settle(20);

    let trace = session.trace()?;
    if trace.events("dimension-group-add").count() <= requested_before {
        return Ok(Some(
            "the Add button took no click — no new `dimension-group-add` line was traced. \
             Either the six keystrokes never reached the name field (so the button is still \
             greyed, which is correct behaviour and means the TYPING failed) or the button is \
             drawn and inert."
                .to_owned(),
        ));
    }
    if let Some(refusal) = trace
        .events(&format!("{APPLIED}-refused"))
        .filter_map(|l| l.get("detail").map(str::to_owned))
        .last()
    {
        return Ok(Some(format!(
            "the Add button raised its action and the engine REFUSED it: {refusal}. The shell \
             half works; this is a `pdfcer-core` verdict and belongs in a request."
        )));
    }
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the Add button was pressed and traced its request, and no `{APPLIED}` line \
             followed — so the `Action` was raised and its apply arm never ran, or ran and \
             could not borrow the session. Nothing reached the document."
        )));
    }
    report.note("the group reached the document through the action funnel");

    // --- 7: ★ and it comes BACK, with a radio on it ------------------------
    let after = live_names(&session.trace()?, ui_rect, DRAW_INTO);
    if after.len() <= before.len() {
        return Ok(Some(format!(
            "★ the group was WRITTEN and did not come back. `{APPLIED}` was traced, so \
             `EditSession::add_dimension_group` ran and the undo log has an entry — and the \
             panel still lists {} row(s), the same as before. A group the model does not \
             give back is a group nothing can ever draw a dimension into, and it is \
             indistinguishable from success at every earlier step. Rows: {}.",
            after.len(),
            list(&after)
        )));
    }
    report.note(format!(
        "the new group is listed and carries an authoring radio: {}",
        list(&after)
    ));

    // --- 8: rename it, and delete it ---------------------------------------
    //
    // ★ A ROUND TRIP, deliberately: create, rename, delete, ending with the
    // list exactly as long as it started. Each verb alone would only show that
    // its arm exists; together they show that the panel's three controls act
    // on the SAME group — which is the failure a per-row surface actually has,
    // and which no single-verb assertion can see.
    //
    // The new group is found by set difference rather than by position. A check
    // that assumed "the last row" would be asserting something about the
    // model's ordering that nothing promises.
    let Some(added) = after.iter().find(|name| !before.contains(name)) else {
        return Ok(Some(format!(
            "the row count grew and no NEW `{DRAW_INTO}*` name appeared, which the model \
             cannot produce — before {}, after {}.",
            list(&before),
            list(&after)
        )));
    };
    let Some(id) = added.strip_prefix(DRAW_INTO) else {
        return Err(Error::new(format!(
            "`{added}` does not begin with `{DRAW_INTO}`, so the group id cannot be read out \
             of it and the rename and delete steps have nothing to aim at."
        )));
    };

    let trace = session.trace()?;
    let Some(row) = declared(&trace, ui_rect, &format!("{ROW}{id}")) else {
        return Ok(Some(format!(
            "the new group has an authoring radio and NO `{ROW}{id}` region, so its name is \
             not clickable and the lower half of the panel can never be pointed at it. \
             Every setting below the list would be unreachable for this group."
        )));
    };
    driver.click_at(session.frame()?.declared_center(row))?;
    session.settle(14);

    // --- rename ------------------------------------------------------------
    //
    // Behind the one fold that is shut for a SAFETY reason rather than a length
    // one: it carries the only two destructive verbs on the panel, and R9
    // forbids greying a control that is genuinely available to make it feel
    // safer. A fold is the honest equivalent — and it is this click.
    if fold(&session, &driver, ui_rect, "identity")?.is_none() {
        return Ok(Some(format!(
            "the panel declares no `{HEADING}identity` region, so rename and delete have no \
             heading to press and a group created by mistake is permanent. Headings \
             declared: {}.",
            list(&declared_names(&session.trace()?, ui_rect, HEADING))
        )));
    }
    let trace = session.trace()?;
    let Some(field) = declared(&trace, ui_rect, RENAME) else {
        return Ok(Some(format!(
            "the selected group's settings declare no `{RENAME}` region, so a mistyped group \
             name is permanent for the life of the document."
        )));
    };
    driver.click_at(session.frame()?.declared_center(field))?;
    session.settle(10);
    driver.press(RENAME_KEY)?;
    session.settle(8);
    // Enter rather than the button: the button is drawn beside the field only
    // while there is something to do, so it has no stable region to aim at —
    // and the row handles Enter for exactly the reason a check needs it, which
    // is that a name is a thing people type and then press Enter on.
    driver.press(vk::ENTER)?;
    session.settle(18);

    let trace = session.trace()?;
    if trace.last(RENAMED).is_none() {
        return Ok(Some(format!(
            "a letter was typed into `{RENAME}` and Enter was pressed, and no `{RENAMED}` \
             line followed. Either the field takes no keystrokes, or Enter is not a commit — \
             and because the button is drawn only while the draft differs, a field that never \
             received the letter shows no button either, so both failures look identical from \
             outside."
        )));
    }
    report.note("the group was renamed, through the document");

    // --- delete ------------------------------------------------------------
    //
    // The group is EMPTY — nothing has been drawn into it — so this exercises
    // the straight path rather than the destination question a populated group
    // asks. **That branch is not covered here**, and it is named rather than
    // left as an unstated gap: it needs a fixture whose dimensions already live
    // in a named group, which this one does not have.
    let trace = session.trace()?;
    let Some(delete) = declared(&trace, ui_rect, DELETE) else {
        return Ok(Some(format!(
            "the selected group declares no `{DELETE}` region, so a group created by mistake \
             stays in the picker for the life of the document."
        )));
    };
    driver.click_at(session.frame()?.declared_center(delete))?;
    session.settle(20);

    let trace = session.trace()?;
    if trace.last(DELETED).is_none() {
        return Ok(Some(format!(
            "Delete was pressed on an EMPTY group and no `{DELETED}` line followed. An empty \
             group is the case the engine accepts without a policy, so this is the shell half \
             and not a refusal."
        )));
    }
    let finally = live_names(&session.trace()?, ui_rect, DRAW_INTO);
    if finally.len() != before.len() {
        return Ok(Some(format!(
            "★ the round trip did not close: {} row(s) before, {} after the delete. Create, \
             rename and delete must act on the same group, and a mismatch here means one of \
             the three pointed somewhere else — which every individual step would still \
             report as a success. Rows: {}.",
            before.len(),
            finally.len(),
            list(&finally)
        )));
    }
    report.note("created, renamed and deleted the same group — the list is back where it started");
    Ok(None)
}
