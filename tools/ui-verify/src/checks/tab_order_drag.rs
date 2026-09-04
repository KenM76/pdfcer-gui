//! `tab_order_drag_moves_a_field_and_shows_where` — dragging a row in the
//! Tab-order list draws an insertion caret, and releasing commits the new
//! `/Annots` order.
//!
//! # The gap this closes
//!
//! The operator asked for this by name and named the reference himself:
//!
//! > *"the tab order list is supposed to be able to be reordered by dragging
//! > and dropping rows around like we can with pages in the page preview, and
//! > have **clear markers** of where the field is going to move to."*
//!
//! The list shipped read-only on 2026-08-30, because `pdfcer-core` had no verb
//! that reordered a page's `/Annots`. The verb shipped 2026-09-02; this is the
//! check that the gesture reached it.
//!
//! # ★★ Why this cannot be a unit test, and the shape of the failure it catches
//!
//! [`crate`]'s standing lesson, in its sharpest form. The permutation
//! arithmetic — `panels::forms::tab_order::drag::reordered` — has seven unit
//! tests, one of them exhaustive over every `from`/`gap` pair on a list with
//! interleaved non-widget entries. **Not one of them can fail on a build where
//! the row does not sense a drag**, because the arithmetic is reached only by a
//! gesture, and the gesture is three frame-level edges the harness alone can
//! produce:
//!
//! 1. `Response::drag_started()` on a row — which requires the row to have been
//!    built with `Sense::drag()` rather than as a plain `ui.label`, and a plain
//!    label is exactly what it was for the whole of its previous life;
//! 2. a resolved drop target, which exists only inside the layout pass because
//!    a *gap* has no position until the rows are placed;
//! 3. a release read from raw pointer input, because a drag begun on a row ends
//!    anywhere.
//!
//! This is the same shape as the two founding defects: a green suite over a
//! feature that does nothing when a human touches it.
//!
//! # ★ The assertion that is the point of the feature
//!
//! **The caret was drawn.** He did not ask to be able to drag rows; he asked
//! for *"clear markers of where the field is going to move to"*. A drag that
//! reorders correctly and shows nothing while it is in flight has answered the
//! wrong half of the request — and it is the half that is invisible to every
//! other kind of test, because the caret exists only while the pointer is down.
//!
//! It is observable after the fact because `crate::diag::ui_rect_visible` is a
//! **change log**: the region is emitted the first frame the caret is drawn and
//! a matching `ui-rect-gone` when it stops. So a completed drag leaves both
//! lines behind, and their presence is proof the marker existed during a
//! gesture no screenshot could have caught mid-flight. Hence
//! [`declared_since`](crate::checks::driving::declared_since) and never
//! `declared`, which asks the present tense of a thing whose nature is to be
//! gone.
//!
//! # What a passing run does NOT prove
//!
//! * That the caret was in the right *place*. This reads its rectangle and
//!   asserts a non-zero **width** and an overlap with the row it was dropped
//!   on, which rules out the two failures that publish a region and draw
//!   nothing an operator could see. Width, not height: this caret is a
//!   horizontal line, where the page rail's is vertical.
//! * That the file on disk is correct. The check stops at the engine's own
//!   applied line and its `moved=` count. Whether the bytes are right is
//!   `pdfcer-core`'s twenty-odd tests for the verb, not this harness's job.
//! * That `/Tabs` was left alone. Nothing here observes the page dictionary,
//!   and it deliberately does not try: the sourced reason `/Tabs` must not be
//!   written by a drag lives in `panels::forms::tab_order::drag`'s header, and
//!   the place to hold the engine to it is `pdfcer-core`'s own dirty-set test,
//!   which asserts the reorder touches exactly one object.
//!
//! What it DOES prove about correctness, beyond the gesture, is one thing, and
//! it is step 6's last assertion: **`non_widgets=0`**. `/Annots` order is paint
//! order, so a permutation that moved a `/Link` would change what is drawn over
//! what — a visible change to the page, produced by a gesture whose whole
//! subject was tab sequence. That the count is zero is the one correctness
//! claim this harness is in a position to make.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, declared_or_in_overflow, declared_since, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode the Forms panel is reachable from.
const MODE: &str = "edit";
/// The ribbon item that opens the Forms panel, and the tab it lives on.
const PANEL_ITEM: &str = "ribbon.item.view.panel_forms";
/// The collapsing header for the Tab-order section, which ships **closed**.
///
/// Deliberately closed — the section is a diagnostic rather than the panel's
/// main job — so a check that assumed it open would report the whole feature
/// missing on a correct build. That exact mistake has been made in this harness
/// before, on the OCR check, where a collapsed ribbon group made a working
/// command look absent.
const HEADER: &str = "forms.tab_order.header";
/// The Forms panel's dock body — an on-screen-by-construction scroll target.
const PANEL_BODY: &str = "dock.body.view.panel_forms";
/// The prefix of the per-row regions: `page.row`, both 0-based.
const ROW: &str = "forms.tab_order.row.";
/// The insertion caret's region.
const CARET: &str = "forms.tab_order.drop-caret";
/// The trace line the gesture emits when a row starts being dragged.
const DRAG_BEGIN: &str = "tab-order-drag-begin";
/// The trace line the gesture emits when the pointer is released.
const DRAG_RELEASE: &str = "tab-order-drag-release";
/// The label the edit funnel traces when `reorder_annotations` succeeded.
const APPLIED: &str = "reorder-annotations-applied";
/// The fewest rows this check can say anything with.
///
/// Two. With one, every gap is the row's own boundary and the correct answer to
/// a drag is *do nothing* — so a run on a one-widget page could not tell a
/// working gesture from a dead one, which is the failure mode this check
/// exists to catch. It would report PASS on a build with no drag at all.
const MIN_ROWS: usize = 2;
/// How far down the landing row the pointer is released, as a fraction of its
/// height.
///
/// Nine-tenths, not the bottom edge. The row's own midpoint decides *before* or
/// *after*, so anything past halfway means the same gap — and a point exactly
/// ON the boundary is the one place a rounding difference between the
/// application's `f32` rectangle and this harness's reading of it could flip
/// the answer. The page rail's check makes the same argument for its
/// three-quarters across.
const LAND_DOWN: f32 = 0.9;

/// See the module documentation.
pub struct TabOrderDragMovesAFieldAndShowsWhere;

impl Check for TabOrderDragMovesAFieldAndShowsWhere {
    fn name(&self) -> &'static str {
        "tab_order_drag_moves_a_field_and_shows_where"
    }

    fn defect(&self) -> &'static str {
        "the tab-order list cannot be reordered by dragging a row, or it can and nothing shows \
         where the field will land — so a form's tab sequence is whatever order the file \
         happens to list its annotations in, and the only way to change it is to rebuild the \
         fields"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check drags the pointer across a panel. \
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

    // ★ Its OWN fixture, not `--pdf`, and this is not a convenience.
    //
    // The harness's usual fixture is a CAD drawing with no `/AcroForm` at all,
    // on which the Tab-order list is legitimately empty — so a run against it
    // would SKIP, every time, for the whole life of the feature. A skip is not
    // red, so the check would quietly stop being evidence and nothing would
    // ever go amber. `demo-form.pdf` carries exactly two widgets on one page,
    // which is [`MIN_ROWS`] and the smallest fixture on which a drag can be
    // told from a dead gesture.
    let fixture = form_fixture().ok_or_else(|| {
        Error::new(format!(
            "the engine fixture `{FIXTURE}` is not on disk, so there is no document with form \
             fields to reorder. This check does NOT fall back to `--pdf`: the harness's usual \
             fixture is a drawing with no `/AcroForm`, on which an empty tab-order list is \
             correct — falling back would turn a real failure into a pass."
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("tab_order_drag.trace.txt"));
    spec.pdf = Some(fixture);
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
    // ★ MAXIMISED, before anything is aimed at. The Forms panel is a docked
    // pane, and at the default window width its sections collapse — which is
    // how the OCR check spent a week SKIPping over a working command. A
    // published region that does not exist because its group folded is
    // indistinguishable from a feature that was never built.
    session.maximize();
    session.settle(20);
    let driver = Driver::new(session.window());

    crate::checks::driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- 1: the Forms panel, then the Tab-order section --------------------
    // ★★★ ONLY IF IT IS NOT ALREADY THERE — a panel toggle that is already on
    // CLOSES the thing this check needs.
    //
    // `crate::checks::pages_drag` carries the same guard and the same sentence,
    // and this check shipped without it. The symptom was not "the panel is
    // missing": the Forms panel is in the default layout, the ribbon press shut
    // it, a later press or a re-dock reopened it **somewhere else**, and the
    // section header was read at x = 3126 and clicked at x = 786. The click
    // landed on nothing, the Tab-order section stayed collapsed, and the check
    // reported zero rows — which reads as "the feature is not there".
    if declared(&session.trace()?, ui_rect, PANEL_BODY).is_none() {
        open_from_tab(&session, &driver, ui_rect, "view", PANEL_ITEM)?;
        session.settle(24);
    }
    // ★★★ GIVE THE PANE ROOM BEFORE OPENING THE SECTION.
    //
    // Measured 2026-09-02: the Forms panel is the BOTTOM of three stacked panes
    // in the right dock — 396 points tall. The tab-order section sits below the
    // whole-form controls, and once its explainer, count and separator are laid
    // out there are roughly 42 points left for a list of rows about 30 points
    // each. `ui_rect_visible` publishes nothing for a row that is mostly
    // clipped, correctly, so the check saw an empty list and no amount of
    // scrolling helped: the content is not scrolled away, it has nowhere to go.
    //
    // ★★ Scrolling was tried first and failed twice, each time for a different
    // and instructive reason — the panel's centre lands on the fill list's OWN
    // nested scroll area, so the wheel moved that instead; and the section
    // header publishes through plain `ui_rect`, so a stale rectangle sent the
    // wheel outside the window. Both are recorded in `scroll_rows_into_view`,
    // which is kept because a taller pane still needs it on a long form.
    //
    // Dragging the splitter is what an operator does when a docked list is too
    // short, it uses regions the dock already publishes, and it leaves the
    // window in a state where the feature under test is actually usable —
    // which is the state worth measuring.
    enlarge_forms_pane(&session, &driver, ui_rect)?;
    open_tab_order(&session, &driver, ui_rect)?;

    // ★★★ SCROLL THE PANEL UNTIL THE ROWS ARE ON SCREEN, and this is not
    // housekeeping — it is the precondition the whole check rests on.
    //
    // The Tab-order section sits below the fill list and the groups in a docked
    // panel with its own scroll area. Measured 2026-09-02 on a two-field form:
    // its rows lay at y = 1406 in a client area 1369 points tall — off the
    // bottom of the window on a 3440 x 1440 display, and further off on
    // anything smaller.
    //
    // ★★ Until the rows published through `ui_rect_visible` this was INVISIBLE.
    // They published a rectangle regardless of the clip, the harness converted
    // it to a screen point below the window, pressed there, and reported *"the
    // row does not sense a drag"* — a confident, specific, entirely wrong defect
    // report about working code. The fix on the application side makes the
    // absence honest; this is the fix on the harness side that makes the check
    // able to proceed.
    //
    // Scrolled at the panel's own centre, a few notches at a time, stopping as
    // soon as the rows appear. Bounded rather than looping: a panel that never
    // yields a row is a finding, not a reason to spin.
    scroll_rows_into_view(&session, &driver, ui_rect)?;

    let trace = session.trace()?;
    let rows = declared_names(&trace, ui_rect, ROW);
    if rows.len() < MIN_ROWS {
        return Err(Error::new(format!(
            "the Tab-order section declares {} row region(s) and this check needs at least \
             {MIN_ROWS}. Rows declared: {}. On a one-row page every gap is the row's own \
             boundary, where doing nothing is the CORRECT answer — so a run here would report \
             PASS on a build with no drag sensing at all.",
            rows.len(),
            list(&rows)
        )));
    }
    report.note(format!("{} tab-order row(s) on screen", rows.len()));

    // Page 0's first two rows. Named by page and row index rather than by tab
    // POSITION, because position counts widgets and shifts when an unclaimed
    // one is registered — a check that aimed by it would hit a different row
    // after an unrelated edit.
    let first = declared(&trace, ui_rect, &format!("{ROW}0.0")).ok_or_else(|| {
        Error::new(format!(
            "no `{ROW}0.0` region — the first page's first row is not on screen. Declared: {}.",
            list(&rows)
        ))
    })?;
    let second = declared(&trace, ui_rect, &format!("{ROW}0.1")).ok_or_else(|| {
        Error::new(format!(
            "no `{ROW}0.1` region — the first page's second row is not on screen. Declared: \
             {}.",
            list(&rows)
        ))
    })?;

    // --- 2: drag row 0 below row 1 -----------------------------------------
    //
    // The LOWER part of row 1, because the row's midpoint resolves before/after
    // and the lower half means the boundary AFTER row 1, which is gap 2. From
    // row 0 that is a real move — the two fields swap — and it is outside the
    // dragged row's own two boundaries, so it is not the no-op the panel
    // correctly refuses.
    let frame = session.frame()?;
    let start = frame.declared_center(first);
    let landing = frame.declared_at(second, 0.5, LAND_DOWN);
    report.note(format!(
        "dragging row 0 from ({}, {}) to the lower half of row 1 at ({}, {})",
        start.x(),
        start.y(),
        landing.x(),
        landing.y()
    ));
    driver.drag(start, landing)?;
    session.settle(24);

    // --- 3: the row sensed the drag at all ---------------------------------
    let trace = session.trace()?;
    let Some(began) = trace.last(DRAG_BEGIN) else {
        return Ok(Some(format!(
            "the pointer was pressed on the first tab-order row and dragged down past the \
             second, and no `{DRAG_BEGIN}` line was traced. The row does not sense a drag — \
             which is the state this list shipped in: a plain `ui.label`, deliberately, while \
             the engine had no verb to commit to."
        )));
    };
    report.note(format!("gesture began: `{}`", began.raw));

    // --- 4: ★ the marker was drawn -----------------------------------------
    let Some(caret) = declared_since(&trace, ui_rect, CARET, began.lineno) else {
        return Ok(Some(format!(
            "the drag was sensed (`{DRAG_BEGIN}` was traced) and NO `{CARET}` region was ever \
             published, so nothing showed the operator where the field would land. The \
             reorder may still be correct; the marker — which is what he actually asked for, \
             in those words — is absent."
        )));
    };
    if caret.width() <= 0.0 {
        return Ok(Some(format!(
            "the `{CARET}` region was published with zero WIDTH ({caret:?}), so the caret was \
             'drawn' as a point. This caret is a horizontal line — the list flows downward and \
             an insertion mark runs across the flow — so width is its extent, where the page \
             rail's vertical caret is checked for height."
        )));
    }
    if !overlaps(caret, second) {
        return Ok(Some(format!(
            "the `{CARET}` region ({caret:?}) does not overlap the row it was dropped on \
             ({second:?}), so the caret was drawn somewhere the operator is not looking. The \
             likeliest cause is a coordinate space: the caret is painted inside the scroll \
             area, and a rectangle built in the parent's space would land at an offset."
        )));
    }
    report.note(format!(
        "the insertion caret was drawn across the landing row, {:.0} pt wide",
        caret.width()
    ));

    // --- 5: the release committed ------------------------------------------
    let Some(release) = trace.last(DRAG_RELEASE) else {
        return Ok(Some(format!(
            "the drag started and drew its caret and no `{DRAG_RELEASE}` line was traced, so \
             the gesture never ended. A drag left in flight leaves a caret nobody can get rid \
             of — which is why the release is read from raw pointer input rather than from the \
             row's own response."
        )));
    };
    report.note(format!("gesture ended: `{}`", release.raw));

    if release.get("reordered") != Some("1") {
        return Ok(Some(format!(
            "the drag was released over gap {} and reported `reordered=0`, so no reorder was \
             raised. Either the drop target was not resolved (the pointer was over no row at \
             release) or the landing was judged a no-op. Dragging the first row below the \
             second is neither. Line: `{}`.",
            release.get("gap").unwrap_or("?"),
            release.raw
        )));
    }
    // ★ Cross-check: the caret the operator was looking at and the boundary the
    // field actually went to must be the SAME decision. They are computed once,
    // in the layout pass, and carried together — which is exactly the kind of
    // claim that is true when written and false after a refactor.
    if release.get("gap") != Some("2") {
        return Ok(Some(format!(
            "the drop landed at gap {} and the pointer was released over the LOWER half of \
             the second row, which is gap 2. The marker and the move are not the same \
             decision. Line: `{}`.",
            release.get("gap").unwrap_or("?"),
            release.raw
        )));
    }

    // --- 6: the engine applied it ------------------------------------------
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "the drag was released and committed (`reordered=1`) and the edit funnel traced no \
             `{APPLIED}` line, so `EditSession::reorder_annotations` either was not called or \
             returned an error. The gesture reaches the action and the action does not reach \
             the document."
        )));
    };
    report.note(format!("engine: `{}`", applied.raw));

    if applied.get("moved") == Some("0") {
        return Ok(Some(format!(
            "the engine accepted the order and reported `moved=0`, so the permutation it was \
             handed was the order the page already had. The gesture ran, the verb ran, and \
             nothing changed — which is what a drag whose slot arithmetic collapses to the \
             identity looks like from the outside. Line: `{}`.",
            applied.raw
        )));
    }

    // ★★ THE ASSERTION THAT IS ABOUT CORRECTNESS RATHER THAN ABOUT THE GESTURE.
    //
    // `non_widgets` counts annotations that are NOT form widgets whose index
    // changed. This route must always report zero, by construction: the widgets
    // move among the slots widgets already occupied and everything else keeps
    // its index. If it is ever non-zero, a drag in a list of *form fields* has
    // silently changed **paint order** — `/Annots` order is z-order — and the
    // operator has a page where a link or a stamp is now drawn over something
    // it was under, from a gesture whose entire subject was tab sequence.
    if applied.get("non_widgets").is_some_and(|v| v != "0") {
        return Ok(Some(format!(
            "the reorder moved {} annotation(s) that are not form widgets. `/Annots` order is \
             PAINT order, so this drag has changed what is drawn on top of what — from a \
             gesture whose whole subject was the order fields are tabbed through. The \
             permutation is meant to move widgets among widget slots and leave every other \
             entry at its index. Line: `{}`.",
            applied.get("non_widgets").unwrap_or("?"),
            applied.raw
        )));
    }
    report.note("no non-widget annotation changed index, so paint order is untouched");

    Ok(None)
}

/// Drag the dock splitter above the Forms pane upward, so the panel has room.
///
/// The splitter is resolved by **geometry** rather than by name: whichever
/// published `…split.row.N` region sits immediately above the Forms panel body
/// is the one that controls its top edge. Naming `dock.right.0.split.row.1`
/// would bake in which dock the panel happens to live in and which of three
/// stacked panes it is, both of which are layout that can legitimately change.
///
/// Silent when there is no splitter above it — a panel that is already the only
/// pane in its dock needs no help, and the caller's own assertions report the
/// case where the rows still do not appear.
fn enlarge_forms_pane(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    let Some(body) = declared(&trace, ui_rect, PANEL_BODY) else {
        return Ok(());
    };
    // The nearest splitter whose bottom edge is at or above the panel's top.
    let Some(split) = declared_names(&trace, ui_rect, "dock.")
        .into_iter()
        .filter(|n| n.contains(".split.row."))
        .filter_map(|n| declared(&trace, ui_rect, &n))
        .filter(|r| r.max.y <= body.min.y + 1.0)
        .max_by(|a, b| a.max.y.total_cmp(&b.max.y))
    else {
        return Ok(());
    };
    // The dock the panel lives in, so the target is expressed against the
    // layout rather than as a fixed pixel offset that a different display would
    // put somewhere else.
    let Some(dock) = declared_names(&trace, ui_rect, "dock.")
        .into_iter()
        .filter(|n| n.matches('.').count() == 1)
        .filter_map(|n| declared(&trace, ui_rect, &n))
        .find(|d| d.min.x <= body.min.x + 1.0 && d.max.x + 1.0 >= body.max.x)
    else {
        return Ok(());
    };
    let frame = session.frame()?;
    let from = frame.declared_center(split);
    // A quarter of the way down the dock: above every splitter, inside the
    // topmost pane, and far enough that the drag comfortably clears egui's
    // click/drag threshold.
    let to = frame.declared_at(dock, 0.5, 0.25);
    driver.drag(from, to)?;
    session.settle(20);
    Ok(())
}

/// Scroll the Forms panel until the tab-order rows publish a visible rectangle.
///
/// Returns as soon as at least [`MIN_ROWS`] rows are on screen. If the panel
/// runs out of scroll without producing them, the caller's own check reports
/// that — this function does not decide it is a failure, because "the section
/// is empty on this document" and "the section is below the fold" are different
/// findings and only the caller knows which it was looking for.
fn scroll_rows_into_view(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    const NOTCHES: i32 = -3;
    const TRIES: usize = 12;
    for _ in 0..TRIES {
        let trace = session.trace()?;
        if declared_names(&trace, ui_rect, ROW).len() >= MIN_ROWS {
            return Ok(());
        }
        // ★★★ THE WHEEL GOES OVER THE SECTION HEADER, NOT THE PANEL'S CENTRE —
        // because the panel contains NESTED scroll areas.
        //
        // The Forms panel's body is one `ScrollArea`; the fill list and the
        // tab-order list each have another inside it. A wheel event is consumed
        // by the innermost scrollable under the pointer, so aiming at the
        // panel's centre — which lands on the fill list — scrolled the fill
        // list and left the panel exactly where it was. Measured: twelve
        // notches, zero movement, zero rows.
        //
        // The section header is a plain widget in the OUTER area, so the wheel
        // over it moves the panel. It is validated against the panel body first
        // because the header publishes through plain `ui_rect` and its
        // rectangle survives going out of view — an unchecked stale rect would
        // send the wheel outside the window again.
        let Some(body) = declared(&trace, ui_rect, PANEL_BODY) else {
            return Ok(());
        };
        let target = declared(&trace, ui_rect, HEADER)
            .filter(|h| h.min.y >= body.min.y && h.max.y <= body.max.y)
            .unwrap_or(body);
        let at = session.frame()?.declared_center(target);
        driver.scroll_at(at, NOTCHES)?;
        session.settle(10);
    }
    Ok(())
}

/// The fixture, relative to the engine repository's synthetic corpus.
///
/// Two widgets on one page — `FullName` and `Subscribe` — which is exactly
/// [`MIN_ROWS`] and the smallest document a drag can be distinguished from a
/// dead gesture on.
const FIXTURE: &str = "forms/demo-form.pdf";

/// Resolve [`FIXTURE`] under the engine repository.
///
/// Read-only, as everything under `D:\Dev\pdfcer` is until fold-in day. The
/// harness opens it, drags in the shell's own window, and never saves.
fn form_fixture() -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(FIXTURE);
    path.is_file().then_some(path)
}

/// Click a ribbon tab, then the item on it, following it into the overflow if
/// the group has collapsed.
fn open_from_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    tab: &str,
    item: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let tab_region = declared(&trace, ui_rect, &format!("ribbon.tab.{tab}")).ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.{tab}` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab_region))?;
    session.settle(14);
    let found = declared_or_in_overflow(session, driver, ui_rect, item)?.ok_or_else(|| {
        Error::new(format!(
            "no `{item}` region on the {tab} tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                &format!("ribbon.item.{tab}.")
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(found))?;
    session.settle(20);
    Ok(())
}

/// Expand the Tab-order collapsing header, which ships closed.
fn open_tab_order(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    if let Some(header) = declared(&trace, ui_rect, HEADER) {
        driver.click_at(session.frame()?.declared_center(header))?;
        session.settle(20);
    }
    Ok(())
}

/// Whether two logical rectangles share any area.
///
/// A caret is a zero-height line, so this is deliberately an *overlap* test
/// rather than a containment one: the caret sits ON the row's boundary, and a
/// containment test would reject the correct answer half the time depending on
/// which side of the edge floating point put it.
fn overlaps(a: LRect, b: LRect) -> bool {
    a.min.x < b.max.x && b.min.x < a.max.x && a.min.y <= b.max.y && b.min.y <= a.max.y
}
