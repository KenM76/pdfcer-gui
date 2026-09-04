//! `text_tool_selects_and_marks_in_edit` — the regression test for **a tool
//! whose entire visible effect is the mouse pointer**, and for the `RIBBON_IA.md`
//! P3 tension it was built to close.
//!
//! # What this is about
//!
//! Until 2026-08-14, `canvas::textsel::takes_the_press` gave a press its text
//! meaning only *"when the select tool is active and the mode cannot select
//! content"* — Read ✓, Review ✓, **Edit ✗**. Two things followed, and the second
//! is the one that made a fix urgent:
//!
//! 1. a reviewer could sweep text and an editor could not, which is an
//!    inversion;
//! 2. the three text-markup controls (`markup.underline`, `markup.strikeout`,
//!    `markup.squiggly`) are **drawn** on the Markup tab in Edit and could
//!    **never enable** there, because `selection.text` was never true. P3
//!    reserves greying for *temporarily* unavailable and says an absent
//!    capability should render nothing; a control that is greyed for the whole
//!    life of a build is neither, and it could not be fixed by hiding, because a
//!    command lives on exactly one tab and the Markup tab is in both Review and
//!    Edit.
//!
//! Both close with `CanvasTool::Text`, armed by **`view.tool_text`** in
//! View ▸ Navigate. This check is the evidence that they actually did, **in one
//! mode, in one run, with the same control observed dead and then live.**
//!
//! # ★ Why the trace is the only possible oracle here
//!
//! Two independent reasons, and either alone would be enough:
//!
//! * **An armed text tool changes the cursor and nothing else on the canvas.** A
//!   captured window does not carry the pointer, so a screenshot of an armed
//!   canvas and an un-armed one are not merely similar — they are *the same
//!   bytes*. `canvas::tool::toggle_text` emits `text-tool tool=…` for exactly
//!   this reason, as `markup-tool` and `measure-tool` already do.
//! * **A text selection's whole feedback is a translucent wash** at
//!   `overlay::TEXT_SELECTION_ALPHA`, deliberately low so the operator can read
//!   through it. [`crate::checks::text_selection`]'s header carries that
//!   argument in full: on a drawing sheet, selected and unselected are the same
//!   picture to any threshold.
//!
//! The one thing a pixel oracle *could* have answered — is the control enabled?
//! — is answered better by the invoke, and for the reason
//! [`crate::checks::text_markup`] gives: a greyed `egui` control never reports
//! itself invoked, so the absence of `ribbon-command-invoked` is positive
//! evidence of disablement from outside the process, where the visible
//! difference is a few dozen antialiased pixels of *text* colour inside a fill
//! that does not change.
//!
//! # The five phases, and why phase A is a negative and phase E is a falsifier
//!
//! | Phase | State | Action | Expected | If it does not hold |
//! |---|---|---|---|---|
//! | A | Edit, nothing armed, nothing selected | click Underline | **no** `ribbon-command-invoked` | FAIL — the control is live with no operand, which is the P3 violation in the *other* direction |
//! | B | Edit | click `view.tool_text` | `text-tool tool=Text` | FAIL — the ribbon control does not arm the canvas tool |
//! | C | Edit, tool armed | sweep a band | `canvas-text-selection` with `chars` > 0 **and** `quads` > 0 | SKIP if no band has text; FAIL if a band selects characters and draws no boxes |
//! | D | Edit, selection live | click Underline | invoke **and** `text-markup-commit` **and** `add-text-markup` | FAIL, with the missing line naming the link |
//! | E | Edit, tool retired | sweep the **same** band | `text-tool tool=Select`, and **no** new `canvas-text-selection` | FAIL — the sweep works in Edit without the tool, so the mode gate has been deleted rather than the tool added |
//!
//! **Phase A is what makes phase D mean anything.** Without it, a build whose
//! three text-markup controls were simply always live would pass D perfectly —
//! and the whole point of the fix is that the controls become *reachable*, not
//! that they become unconditional.
//!
//! **Phase E is what makes phase C mean anything**, and it is the rule this
//! crate states as *"never treat an absence as evidence unless you have shown the
//! thing that would have produced it was working"* run backwards: here the
//! *presence* in phase C is the claim, and E establishes that the presence is
//! caused by the tool rather than by the mode gate having been removed. A build
//! that had deleted `takes_the_press`'s second half — the `!caps.edit_content`
//! clause — would sweep text in Edit with nothing armed, would pass A, B, C and
//! D, and would have silently replaced the marquee, which is the only
//! content-selection gesture the product has. E is the only phase that catches
//! it.
//!
//! `text_selection`'s own phase C asserts the same property from the other side
//! (Read sweeps, Edit does not) and is deliberately **not** removed by this
//! check's arrival: that one runs with no tool ever armed in the process, so it
//! is evidence about the shipped default, while this one is evidence about the
//! default surviving beside the new tool.
//!
//! # Mouse only
//!
//! Every gesture here is a real `SetCursorPos` + `mouse_event`. Nothing in this
//! check needs a key, which is a consequence of the interaction model rather than a
//! coincidence: the tool is armed from the ribbon and the mark is a ribbon press,
//! so the whole feature is clicks and one drag.
//!
//! What is therefore **not** covered here, and is on the record rather than
//! implied by a green result: Escape's behaviour with the tool armed (rung 5
//! clears the text selection and the tool is deliberately not an Escape
//! claimant — `canvas::keys`' header), and Ctrl+A / Ctrl+C in Edit with the tool
//! armed. All three are covered by unit test alone.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the diagnostic switches did not reach the process;
//! * the page size could not be read and no `--page-size` was given;
//! * the Edit segment, the View or Markup tab, or one of the two controls was
//!   never declared;
//! * the canvas is not showing page 1, so the harness's one known page size does
//!   not describe the page it would be sweeping;
//! * **no band had text under it** — phase C never succeeded, so phase A's
//!   silence proves nothing and phase D has no operand.

use crate::checks::driving::{
    self, INVOKE_EVENT, ITEM_PREFIX, SHELL_DIAG_ENV, TAB_EVENT, UNIMPLEMENTED_EVENT, declared,
    declared_names, list, list_str, shell_trace,
};
use crate::checks::text_selection::{BANDS, aim};
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// ★ **Edit, and only Edit** — the one mode where this tool changes anything.
///
/// Read and Review already sweep text with the select tool, so arming the tool
/// there is a no-op an operator cannot see; the two gaps it closes are both in
/// Edit, whose primary button is the content marquee. A check aimed at Review
/// would pass against a build where `view.tool_text` did nothing at all.
const MODE: &str = "edit";

/// The tab carrying `view.tool_text`. View is in **every** mode's tab list,
/// which is why the tool lives there — see `shell::manifest::view`'s Navigate
/// group.
const TOOL_TAB: &str = "ribbon.tab.view";

/// The tab id the shell reports for [`TOOL_TAB`].
const TOOL_TAB_ID: &str = "view";

/// The control that arms the tool.
const TOOL_ITEM: &str = "ribbon.item.view.tool_text";

/// Its command id, as dispatch and the shell spell it.
const TOOL_ID: &str = "view.tool_text";

/// Its neighbour in View ▸ Navigate, asserted for **presence only**.
///
/// The two pointer-tool toggles are one group and one idea, and a build that
/// registered the new one while losing the old would otherwise pass this check
/// completely — the same half-done-registration guard
/// [`crate::checks::text_markup`] applies to Strikeout and Squiggly.
const TOOL_SIBLING: &str = "ribbon.item.view.tool_hand";

/// The tab carrying the text-markup controls.
const MARKUP_TAB: &str = "ribbon.tab.markup";

/// The tab id the shell reports for [`MARKUP_TAB`].
const MARKUP_TAB_ID: &str = "markup";

/// **The control whose reachability is the P3 claim.**
///
/// Underline rather than Strikeout or Squiggly for
/// [`crate::checks::text_markup`]'s reason: the three are one dispatch arm with
/// one `match` between them, `shell::commands::mapping` walks all three, and the
/// *join* under test here is per-command only in its id.
const MARK_ITEM: &str = "ribbon.item.markup.underline";

/// [`MARK_ITEM`]'s command id.
const MARK_ID: &str = "markup.underline";

/// `text-tool tool=…` — `canvas::tool::toggle_text`'s report that the ribbon
/// press reached the canvas.
///
/// The only evidence there is: an armed text tool and an un-armed one are the
/// same captured window, because a capture does not carry the cursor.
const TOOL_EVENT: &str = "text-tool";

/// The field on [`TOOL_EVENT`] carrying the `Debug` spelling of the tool now
/// chosen.
const TOOL_FIELD: &str = "tool";

/// What [`TOOL_FIELD`] reads once the tool is armed.
const TOOL_ARMED: &str = "Text";

/// …and once it is retired, which is the toggle's other half.
const TOOL_RETIRED: &str = "Select";

/// `canvas-text-selection via=… page=… chars=… quads=…`.
const TEXT_EVENT: &str = "canvas-text-selection";

/// `text-markup-commit kind=… page=… quads=…`.
const COMMIT_EVENT: &str = "text-markup-commit";

/// `text-markup-declined kind=… reason=…`, read only to improve a failure
/// message: `reason=Stale` and `reason=NoSelection` send a reader to two
/// different places.
const DECLINE_EVENT: &str = "text-markup-declined";

/// `add-text-markup page=… n=… epoch=…` — the **engine** authored it.
const APPLY_EVENT: &str = "add-text-markup";

/// The byte length of the selected string.
const CHARS_FIELD: &str = "chars";

/// How many line boxes. Compared across [`TEXT_EVENT`] and [`COMMIT_EVENT`].
const QUADS_FIELD: &str = "quads";

/// See the module documentation.
pub struct TextToolSelectsAndMarksInEdit;

impl Check for TextToolSelectsAndMarksInEdit {
    fn name(&self) -> &'static str {
        "text_tool_selects_and_marks_in_edit"
    }

    fn defect(&self) -> &'static str {
        "View ▸ Navigate ▸ Text does not arm the canvas tool, a sweep in Edit with it armed \
         selects nothing, or Markup ▸ Underline stays greyed in Edit once text IS selected — the \
         P3 tension of three controls drawn on a tab where they could never enable, and the join \
         between a ribbon toggle, a canvas gesture and a condition that no unit test observes in \
         one window"
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

/// Every `canvas-text-selection` line reporting a **non-empty** selection.
///
/// Filtered on `chars > 0` for the reason both sibling checks record: a *clear*
/// is traced too, with `chars=0`, so counting the event would be satisfied by
/// the gesture that ends a selection.
fn selections(trace: &Trace) -> Vec<&crate::trace::TraceLine> {
    trace
        .events(TEXT_EVENT)
        .filter(|l| l.get_usize(CHARS_FIELD).unwrap_or(0) > 0)
        .collect()
}

/// How many times the shell has reported `id` invoked.
///
/// A **count**, never a presence: this check clicks the same control twice and
/// two different controls in one run, so "has it ever been invoked?" would be
/// answered `true` by a click made ten seconds earlier.
fn invokes(session: &Session, id: &str) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count())
}

/// Click a ribbon tab and confirm the shell reported it.
///
/// Three tab clicks in one run is what this check costs — View to arm, Markup to
/// mark, and Markup again after the sweep — so the move is written once rather
/// than three times. It is not in [`driving`] because the two existing tab
/// clicks in the suite are inline and folding them in would rewrite checks that
/// are already known to detect their defects, which is the argument that
/// module's own header makes about `markup_rectangle`.
fn click_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    region: &str,
    tab_id: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let rect = declared(&trace, ui_rect, region).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{region}` region in `{MODE}`. Either this build does \
             not show that tab in this mode, or the tab strip is too narrow and it has moved into \
             the overflow menu — which this check cannot open, because a menu's contents are not \
             published as regions. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(12);
    if !shell_trace(session)?
        .events(TAB_EVENT)
        .any(|l| l.get("tab") == Some(tab_id))
    {
        return Err(Error::new(format!(
            "the click on `{region}` produced no `{TAB_EVENT} tab={tab_id}` line. The mode click \
             DID land, so pointer input works and this is not the input channel; the likely cause \
             is that the tab moved between the frame that declared its rect and the frame that \
             received the click."
        )));
    }
    Ok(())
}

/// Locate a declared control and refuse a degenerate rectangle.
fn control(trace: &Trace, ui_rect: &str, name: &str, what: &str) -> Result<crate::geom::LRect> {
    let rect = declared(trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the tab is active and its controls publish their rects, but none of them is \
             `{name}`. This build has no {what} control. Controls declared: {}.",
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
            "no --pdf. This check arms a tool, sweeps text with it and then marks what it swept, \
             so it needs a document with readable text on its first page. With nothing open every \
             control involved is correctly greyed and the check would be measuring `doc.pages` \
             rather than the feature.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is five clicks and two drags on a real \
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
                 mirrors every sweep about the page centre, which lands on the page and selects \
                 something plausible.",
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
    let mut spec = LaunchSpec::new(&exe, ctx.out("text_tool.trace.txt"));
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

    // --- step 1: Edit, the one mode this tool changes anything in ----------
    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    report.note(format!(
        "the {MODE} segment reported the click — the mode whose primary button is the content \
         marquee, and therefore the only one where the text tool is not redundant"
    ));

    // ==================================================================
    // PHASE A — Underline must be DEAD in Edit before anything is swept
    // ==================================================================
    //
    // This is the "before" half of the P3 claim, and it is taken in the SAME
    // mode and on the SAME control that phase D will find live. Without it the
    // run could not distinguish a control that became reachable from one that
    // was always live — and "always live and does nothing" is what P3 forbids
    // outright, so failing here is a violation in the opposite direction rather
    // than a missing feature.
    click_tab(&session, &driver, ui_rect, MARKUP_TAB, MARKUP_TAB_ID)?;
    let frame = session.frame()?;
    let mark = control(&session.trace()?, ui_rect, MARK_ITEM, "Underline")?;
    report.note(format!(
        "{MARK_ITEM} at {mark:?} on the Markup tab in {MODE}"
    ));

    let before = invokes(&session, MARK_ID)?;
    driver.click_at(frame.declared_center(mark))?;
    session.settle(12);
    if invokes(&session, MARK_ID)? > before {
        return Ok(Some(format!(
            "P3, IN THE OTHER DIRECTION: `{MARK_ITEM}` IS LIVE IN {MODE} WITH NOTHING SELECTED. \
             Nothing has been swept this run, so there is no text selection — and the click \
             produced `{INVOKE_EVENT} id={MARK_ID}` anyway, which a disabled control cannot do. \
             The command is registered `enabled_when(\"selection.text\")` and `app::conditions` \
             publishes that name only for a **live** text selection on the open document, so \
             either the registration lost its predicate or the condition is being published \
             unconditionally. {}",
            match session.trace()?.last(DECLINE_EVENT) {
                Some(line) => format!(
                    "The application then declined it — `{}` — which is the belt working and the \
                     braces missing: nothing was authored, and the operator still pressed a live \
                     button that did nothing.",
                    line.raw
                ),
                None => format!(
                    "No `{DECLINE_EVENT}` either, so the press may have authored something: check \
                     the trace for `{APPLY_EVENT}`."
                ),
            }
        )));
    }
    report.note(format!(
        "with nothing selected, a click on {MARK_ITEM} produced no `{INVOKE_EVENT}` — the control \
         is greyed, which is the state this feature had to make ESCAPABLE rather than remove"
    ));

    // ==================================================================
    // PHASE B — the ribbon control arms the canvas tool
    // ==================================================================
    click_tab(&session, &driver, ui_rect, TOOL_TAB, TOOL_TAB_ID)?;
    let frame = session.frame()?;
    let trace = session.trace()?;
    let tool = control(&trace, ui_rect, TOOL_ITEM, "Text tool")?;
    if declared(&trace, ui_rect, TOOL_SIBLING).is_none() {
        return Err(Error::new(format!(
            "`{TOOL_ITEM}` is declared and `{TOOL_SIBLING}` is not. The two pointer-tool toggles \
             are one group and one idea, and a build carrying the new one while losing the old is \
             a registration that was half done — which this check would otherwise pass without \
             noticing. Controls declared: {}.",
            list(&declared_names(&trace, ui_rect, ITEM_PREFIX))
        )));
    }
    report.note(format!(
        "{TOOL_ITEM} at {tool:?}, with {TOOL_SIBLING} beside it in View ▸ Navigate"
    ));

    let before = invokes(&session, TOOL_ID)?;
    driver.click_at(frame.declared_center(tool))?;
    session.settle(16);
    if invokes(&session, TOOL_ID)? <= before {
        return Ok(Some(format!(
            "THE TEXT TOOL CONTROL DID NOT TAKE THE CLICK. `{TOOL_ITEM}` was declared at {tool:?} \
             and the click produced no `{INVOKE_EVENT} id={TOOL_ID}`. It is registered \
             `enabled_when(\"doc.pages\")` and a document with pages is open, so a greyed control \
             is the wrong reading — check that the manifest's View ▸ Navigate group really lists \
             it, and that the id in `shell::commands` matches the one in `shell::manifest::view`. \
             Commands the shell reported invoked this run: {}.",
            list_str(
                &shell_trace(&session)?
                    .events(INVOKE_EVENT)
                    .filter_map(|l| l.get("id"))
                    .collect::<Vec<&str>>()
            )
        )));
    }
    let armed = session
        .trace()?
        .events(TOOL_EVENT)
        .filter(|l| l.get(TOOL_FIELD) == Some(TOOL_ARMED))
        .count();
    if armed == 0 {
        return Ok(Some(format!(
            "THE COMMAND WAS INVOKED AND THE CANVAS TOOL DID NOT ARM. The shell traced \
             `{INVOKE_EVENT} id={TOOL_ID}`, so the token reached the application — and there is no \
             `{TOOL_EVENT} {TOOL_FIELD}={TOOL_ARMED}` line, which `canvas::tool::toggle_text` \
             emits unconditionally. {} Look at `app/dispatch.rs`'s `\"{TOOL_ID}\"` arm: it is one \
             call, and the only ways to reach here are that the arm is missing (in which case the \
             fall-through fires) or that the id it matches differs from the one registered.",
            if session
                .trace()?
                .events(UNIMPLEMENTED_EVENT)
                .any(|l| l.get("id") == Some(TOOL_ID))
            {
                format!(
                    "The application traced `{UNIMPLEMENTED_EVENT} id={TOOL_ID}`, which is \
                     `dispatch_command`'s fall-through: the command arrived and dispatch had no \
                     arm for it."
                )
            } else {
                format!(
                    "There is no `{UNIMPLEMENTED_EVENT}` either, so the command did not reach the \
                     fall-through — check `dispatch_token`'s token-to-id lookup."
                )
            }
        )));
    }
    report.note(format!(
        "the control armed the canvas: `{}`. This line is the ONLY evidence available — an armed \
         text tool changes the cursor and nothing else, and a captured window does not carry the \
         cursor",
        session
            .trace()?
            .last(TOOL_EVENT)
            .map_or(String::new(), |l| l.raw.clone())
    ));

    // ==================================================================
    // PHASE C — a sweep in Edit now selects text
    // ==================================================================
    let mut unreachable: Vec<String> = Vec::new();
    let mut found: Option<(usize, usize, String)> = None;

    for (n, (start, end)) in BANDS.iter().enumerate() {
        let from = DocPoint::new(0, start.0 * page.width_pt, start.1 * page.height_pt);
        let to = DocPoint::new(0, end.0 * page.width_pt, end.1 * page.height_pt);
        let (from, to) = match (aim(ctx, &session, page, from), aim(ctx, &session, page, to)) {
            (Ok(a), Ok(b)) => (a, b),
            (Err(e), _) | (_, Err(e)) => {
                unreachable.push(format!("band {}: {}", n + 1, e.message()));
                continue;
            }
        };
        let before = selections(&session.trace()?).len();
        driver.drag(from, to)?;
        session.settle(16);
        let after = session.trace()?;
        let lines = selections(&after);
        // The **last** new line: a sweep traces every distinct state it passes
        // through, and the settled one is the selection the operator is left
        // holding — which is the one phase D will mark.
        if let Some(line) = lines.last().filter(|_| lines.len() > before) {
            let quads = line.get_usize(QUADS_FIELD).unwrap_or(0);
            if quads == 0 {
                return Ok(Some(format!(
                    "THE SELECTION HIGHLIGHTS NOTHING. Band {} traced `{}` in {MODE} — characters \
                     were selected and no line boxes were produced. Those two are one derivation \
                     from one pass (`canvas::textsel` header section 5), so `quads=0` with \
                     `chars>0` means the box half fell out — and there is nothing for a \
                     `/QuadPoints` to be authored from either, which is the half phase D consumes.",
                    n + 1,
                    line.raw
                )));
            }
            report.note(format!(
                "band {}: the sweep traced `{}` — a drag in {MODE} selected TEXT, which is the \
                 inversion this tool closes",
                n + 1,
                line.raw
            ));
            found = Some((n + 1, quads, line.raw.clone()));
            break;
        }
        report.note(format!(
            "band {}: no text under the sweep; trying the next",
            n + 1
        ));
    }

    let Some((band, selection_quads, selection_line)) = found else {
        return Err(Error::new(format!(
            "no band had text under it: {} sweeps were performed in `{MODE}` with the text tool \
             armed and none selected a character. This check declines to call that a pass — with \
             no selection established, phase A's silence proves nothing (a control greyed for want \
             of a selection and a control greyed because the feature is missing look identical), \
             and phase D has no operand. {}Trace: {}.",
            BANDS.len(),
            if unreachable.is_empty() {
                String::new()
            } else {
                format!(
                    "Bands that could not be aimed at all, which is a different problem and may be \
                     the whole of this one: {}. ",
                    driving::list(&unreachable)
                )
            },
            session.trace_path().display()
        )));
    };

    // --- the picture, saved as evidence rather than asserted on -------------
    //
    // `crate::capture`'s standing rule: every check that could look at pixels
    // saves its evidence, pass or fail. It is emphatically not the oracle here —
    // see the module header on why an armed canvas and an un-armed one are the
    // same bytes — but a reader who wants to know whether the band landed on the
    // words they expected needs the picture, and on a failure it is the first
    // thing they will ask for.
    let shot = ctx.out("text_tool.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
            report.note(
                "the window with the Edit-mode text selection on it is saved beside the trace. \
                 Evidence, not the oracle: the wash is deliberately low-alpha and the cursor is \
                 not captured at all",
            );
        }
        Err(e) => {
            report.note(format!(
                "could not capture the window ({e}); every assertion above still stands, and they \
                 are what this check's verdict rests on"
            ));
        }
    }

    // ==================================================================
    // PHASE D — and Underline comes alive, in Edit
    // ==================================================================
    //
    // ★ The P3 tension closing, observed rather than argued: the same control,
    // in the same mode, in the same run, that phase A found dead.
    click_tab(&session, &driver, ui_rect, MARKUP_TAB, MARKUP_TAB_ID)?;
    let frame = session.frame()?;
    let mark = control(&session.trace()?, ui_rect, MARK_ITEM, "Underline")?;
    let invokes_before = invokes(&session, MARK_ID)?;
    let applies_before = session.trace()?.events(APPLY_EVENT).count();
    driver.click_at(frame.declared_center(mark))?;
    session.settle(24);

    if invokes(&session, MARK_ID)? <= invokes_before {
        return Ok(Some(format!(
            "THE P3 TENSION IS NOT CLOSED: `{MARK_ITEM}` IS STILL DEAD IN {MODE}. Band {band} was \
             swept with the text tool armed and the application traced `{selection_line}`, so a \
             live text selection exists on the document — and clicking the control produced no \
             `{INVOKE_EVENT} id={MARK_ID}`, which means it is still disabled. The chain is \
             `canvas::interact` storing the selection -> `app::conditions` publishing \
             `selection.text` from a **live** one -> the command's `enabled_when`. The middle link \
             is the one with no test that observes it in a window: check that the condition asks \
             `live(doc.edit_epoch)` against the same epoch the selection was stamped with, and \
             that nothing in the mode gate clears the selection on the way through. Commands the \
             shell reported invoked this run: {}.",
            list_str(
                &shell_trace(&session)?
                    .events(INVOKE_EVENT)
                    .filter_map(|l| l.get("id"))
                    .collect::<Vec<&str>>()
            )
        )));
    }
    report.note(format!(
        "★ the SAME control that phase A found greyed reported `{INVOKE_EVENT} id={MARK_ID}` — in \
         {MODE}, with a text selection made by the text tool. That is the P3 tension closed, \
         observed from outside the process"
    ));

    let trace = session.trace()?;
    let Some(commit) = trace.events(COMMIT_EVENT).last() else {
        return Ok(Some(format!(
            "THE CLICK REACHED THE CONTROL AND AUTHORED NOTHING. The shell traced \
             `{INVOKE_EVENT} id={MARK_ID}` and there is no `{COMMIT_EVENT}`. {} Look at \
             `app/dispatch.rs`'s guard arm matching \
             `shell::commands::text_mark_for_command(id).is_some()`, and at whether the mode's \
             `author_markup` is being read — Edit has it, so a decline on that ground would mean \
             the capability derivation and the tab list disagree.",
            match trace.last(DECLINE_EVENT) {
                Some(line) => format!(
                    "The application DID trace `{}`, so the arm ran and refused: that names the \
                     rule that rejected the selection rather than the routing.",
                    line.raw
                ),
                None => format!("There is no `{DECLINE_EVENT}` either."),
            }
        )));
    };
    let commit_quads = commit.get_usize(QUADS_FIELD).unwrap_or(0);
    if commit_quads != selection_quads {
        return Ok(Some(format!(
            "THE MARK AND THE WASH DESCRIBE DIFFERENT BOXES. The selection was traced with \
             `{QUADS_FIELD}={selection_quads}` — `{selection_line}` — and the annotation was \
             authored from `{QUADS_FIELD}={commit_quads}`: `{}`. `canvas::textsel` section 5.1 \
             claims both lists are the same accumulation from one walk over the same glyphs. A \
             mismatch means something re-derived the authoring quads, and the operator would be \
             marking glyphs they never saw highlighted — only discoverable after saving.",
            commit.raw
        )));
    }
    if trace.events(APPLY_EVENT).count() <= applies_before {
        return Ok(Some(format!(
            "THE ENGINE NEVER AUTHORED IT. The application decided to author an annotation — \
             `{}` — and no `{APPLY_EVENT}` line followed, so `app::actions`' apply arm never ran \
             or `EditSession::add_markup` refused. A `vector_edit` that returned `Err` traces \
             `add-text-markup-refused` instead, so check for that line first.",
            commit.raw
        )));
    }
    report.note(format!(
        "the engine authored it: `{}` — and the wash and the mark agree on {selection_quads} \
         box(es), which is the one-derivation promise asserted across two modules and two trace \
         lines",
        trace
            .events(APPLY_EVENT)
            .last()
            .map_or(String::new(), |l| l.raw.clone())
    ));

    // ==================================================================
    // PHASE E — retire the tool, and Edit is a content marquee again
    // ==================================================================
    //
    // ★ The falsifier. Everything above would pass against a build that had
    // simply deleted `takes_the_press`'s `!caps.edit_content` clause — text
    // would sweep in Edit unconditionally, the tool would arm and change
    // nothing, and the marquee (the ONLY content-selection gesture the product
    // has) would be silently gone. This is the only phase that can tell the two
    // apart, and it costs one click and one drag.
    click_tab(&session, &driver, ui_rect, TOOL_TAB, TOOL_TAB_ID)?;
    let frame = session.frame()?;
    let tool = control(&session.trace()?, ui_rect, TOOL_ITEM, "Text tool")?;
    driver.click_at(frame.declared_center(tool))?;
    session.settle(16);
    let retired = session
        .trace()?
        .events(TOOL_EVENT)
        .filter(|l| l.get(TOOL_FIELD) == Some(TOOL_RETIRED))
        .count();
    if retired == 0 {
        return Ok(Some(format!(
            "THE TOOL CANNOT BE PUT DOWN. A second press of `{TOOL_ITEM}` produced no \
             `{TOOL_EVENT} {TOOL_FIELD}={TOOL_RETIRED}` line, so `canvas::tool::toggle_text` did \
             not retire it. The button renders pressed while armed, so pressing it is how an \
             operator un-presses it — a control that only ever arms leaves someone who armed it by \
             mistake with no way back to the select tool from this group."
        )));
    }
    let (start, end) = BANDS[band - 1];
    let from = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, start.0 * page.width_pt, start.1 * page.height_pt),
    )?;
    let to = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, end.0 * page.width_pt, end.1 * page.height_pt),
    )?;
    let before = selections(&session.trace()?).len();
    driver.drag(from, to)?;
    session.settle(16);
    let after = session.trace()?;
    let lines = selections(&after);
    if lines.len() > before {
        return Ok(Some(format!(
            "THE TEXT GESTURE DOES NOT NEED THE TOOL, SO THE MODE GATE IS GONE. Band {band} was \
             swept again in {MODE} with the text tool RETIRED — the application traced \
             `{TOOL_EVENT} {TOOL_FIELD}={TOOL_RETIRED}` — and it still produced `{}`. In Edit with \
             nothing armed there is nothing that may write that line: \
             `canvas::textsel::takes_the_press`'s second disjunct requires `!caps.edit_content`, \
             so `press_kind` must yield `Marquee(Select)`. This build has silently replaced the \
             only content-selection gesture the product has, and every phase above would pass \
             anyway — which is what this phase exists for.",
            lines.last().map_or("", |l| l.raw.as_str())
        )));
    }
    report.note(format!(
        "with the tool retired the SAME band traced no new `{TEXT_EVENT}` — so the sweep in phase \
         C was caused by the tool, not by a missing mode gate. Without this phase every assertion \
         above would hold against a build that had deleted the gate outright"
    ));

    report.note(format!(
        "verdict established on band {band}: Underline greyed in {MODE}, the ribbon control armed \
         the canvas tool, the same drag then selected text, Underline came alive and authored one \
         annotation over the boxes that were highlighted, and retiring the tool put the content \
         marquee back"
    ));
    report.note(
        "not covered here: Escape with the tool armed (rung 5 clears the text selection, and the \
         tool is deliberately NOT an Escape claimant — see `canvas::keys`' header), and Ctrl+A / \
         Ctrl+C in Edit. This check does not drive them; keystrokes DO reach the window \
         (see find_bar), so all three are covered by unit test alone and the gap is on the record \
         rather than implied by a green result",
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names this check greps for are the ones `egui-shell` builds, and the
    /// ids are the ones the application registers.
    ///
    /// Pinned for the reason both sibling checks pin theirs: the two crates are
    /// joined by a **string** and nothing else, so a rename would leave both
    /// sides compiling while every assertion here quietly stopped matching — and
    /// a check that matches nothing passes vacuously.
    #[test]
    fn the_selectors_match_the_shells_own_spelling() {
        assert_eq!(TOOL_ITEM, format!("ribbon.item.{TOOL_ID}"));
        assert_eq!(MARK_ITEM, format!("ribbon.item.{MARK_ID}"));
        assert_eq!(TOOL_TAB, format!("ribbon.tab.{TOOL_TAB_ID}"));
        assert_eq!(MARKUP_TAB, format!("ribbon.tab.{MARKUP_TAB_ID}"));
        for name in [TOOL_ITEM, TOOL_SIBLING, MARK_ITEM] {
            assert!(name.starts_with(ITEM_PREFIX), "{name}");
        }
        assert_ne!(TOOL_ITEM, TOOL_SIBLING);
        // ★ Edit, and the constant is where that finding is enforced. Aimed at
        // Read the Markup tab would not exist; aimed at Review the select tool
        // already sweeps text, so phase C would pass against a build where
        // `view.tool_text` did nothing at all.
        assert_eq!(MODE, "edit");
        // The two tabs are different tabs, which is the whole reason this check
        // clicks three times: the tool lives on View (shown in every mode) and
        // the mark lives on Markup, and a command lives on exactly one tab.
        assert_ne!(TOOL_TAB, MARKUP_TAB);
    }

    /// ★ **The two halves of the toggle are read from the same field and are not
    /// the same value** — the arithmetic phase B and phase E rest on.
    ///
    /// `canvas::tool::toggle_text` traces the tool it moved *to*, so arming and
    /// retiring differ only in that one word. A check that grepped for the event
    /// name alone would be satisfied by either, and phase E — the falsifier —
    /// would then pass on the frame the tool was armed.
    #[test]
    fn arming_and_retiring_are_told_apart_by_the_tool_field() {
        let trace = Trace::parse(
            "pdfcer-diag text-tool tool=Text\n\
             pdfcer-diag canvas-text-selection via=drag page=0 chars=27 quads=2\n\
             pdfcer-diag text-tool tool=Select",
            "pdfcer-diag",
        );
        assert_ne!(TOOL_ARMED, TOOL_RETIRED);
        assert_eq!(
            trace
                .events(TOOL_EVENT)
                .filter(|l| l.get(TOOL_FIELD) == Some(TOOL_ARMED))
                .count(),
            1
        );
        assert_eq!(
            trace
                .events(TOOL_EVENT)
                .filter(|l| l.get(TOOL_FIELD) == Some(TOOL_RETIRED))
                .count(),
            1
        );
        assert_eq!(
            trace.events(TOOL_EVENT).count(),
            2,
            "the event name alone cannot separate the two, which is why the field is read"
        );
    }

    /// A cleared selection is not a selection — the filter phases C and E both
    /// depend on, and the same one both sibling checks document.
    #[test]
    fn only_a_non_empty_selection_counts() {
        let trace = Trace::parse(
            "pdfcer-diag canvas-text-selection via=clear page=0 chars=0 quads=0\n\
             pdfcer-diag canvas-text-selection via=drag page=0 chars=27 quads=2",
            "pdfcer-diag",
        );
        let found = selections(&trace);
        assert_eq!(found.len(), 1, "the clear must not be counted");
        assert_eq!(found[0].get_usize(QUADS_FIELD), Some(2));
    }
}
