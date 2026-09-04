//! `a_bookmark_can_be_dragged_and_a_branch_collapsed` — **the panel that could
//! create, rename and delete, and could not REORGANISE.**
//!
//! # What this proves
//!
//! `pdfcer-core` `Pass 161.0` shipped `move_outline_item` and
//! `set_outline_open`. Until then a bookmark could be written, retitled and
//! removed, and an outline in the wrong **order** could only be fixed by
//! deleting a branch and re-authoring it — which loses every destination,
//! colour and style on it. Two verbs, one gesture apiece, and both of them
//! reach the operator through the row list rather than through a button:
//!
//! * **drag a row onto the middle of another** and it is filed inside it;
//! * **press the triangle** on a row with children and the branch folds away.
//!
//! ⇒ One check, because the second gesture is the only honest oracle for the
//! first half of the third phase and because they cannot be exercised
//! separately without paying twice for a launch, a mode click, a panel open and
//! two authored bookmarks.
//!
//! # ★★★ The collapse oracle is the DISAGREEMENT between two numbers
//!
//! This is the assertion the whole check is built around, and it is the one no
//! unit test in the workspace can make:
//!
//! > after the triangle is pressed, the panel says the document holds exactly
//! > as many bookmarks as it did a moment ago, and draws **one row fewer**.
//!
//! Both halves are needed and neither alone is evidence. `bookmarks-panel
//! items=` counts what `read_outline` read — every item at every level,
//! collapsed branches included — so it proves the collapse **did not delete
//! anything**. The row count proves the panel **honours `/Count`'s sign**,
//! which it did not do before this feature: the walk used to recurse
//! unconditionally, so a triangle that wrote the sign into the file would have
//! changed nothing on screen and read as a control that does not work.
//!
//! A build that wrote the sign and kept drawing the children passes every unit
//! test about the tree, passes the funnel-line assertion below, and fails this.
//! A build that "collapsed" by deleting the subtree fails the item count. There
//! is no third build that passes both.
//!
//! ★ **Every count in this check is a DELTA**, measured against the outline the
//! fixture arrived with. The corpus is not uniform — `fixtures/four-pages.pdf`
//! ships with a six-item outline and the CAD exports have none — and a check
//! that hard-coded either number would SKIP on half of it while blaming the
//! fixture.
//!
//! # ★★ The move oracle is the LEVEL, not the order
//!
//! `bookmark-row level=` is `OutlineItem::level` — `0` for a top-level
//! bookmark, `1` for its child. The drag in phase B drops TAIL on the middle of
//! DETAIL, which is [`OutlinePlacement::LastChild`], so TAIL's level must go
//! **0 → 1**.
//!
//! Asserting the *order* instead would pass on a build that reordered where it
//! should have re-parented, which is precisely the defect a three-band drop
//! model can produce by mis-reading the pointer's y. The level cannot be
//! reached by a reorder at all.
//!
//! ★ And `bookmark-move-report reparented=` is asserted beside it, from the
//! engine's own report. Two independent witnesses to one fact: the shell's
//! read of the tree afterwards, and the engine's account of what it did. They
//! agree on a correct build and a build that lies has to lie twice.
//!
//! # ★★★ Two rectangles per row, and using the wrong one aims at nothing
//!
//! `bookmark-row` carries `rect=` (the **label**) and `row=` (the **full-width
//! strip**). They are different questions:
//!
//! | to | aim with | why |
//! |---|---|---|
//! | press or lift a row | `rect=` | only the label is a widget; the strip's centre is empty space |
//! | drop onto a landing band | `row=` | the band test is over the strip, and the pointer must be inside it |
//!
//! Getting that backwards produces a silent failure in each direction: a press
//! at the strip's centre lands on nothing and starts no drag, and a drop aimed
//! at the label's centre is over the row but tells you nothing about whether
//! the strip was the thing being tested.
//!
//! ★ Neither is a `ui_rect`. Both come from the panel's own per-row diagnostic
//! line, which is written for **every** row whether or not it is on screen —
//! see [`visible_rows`] and the incident it carries.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | author DETAIL, then TAIL, both at the top level | `items` up by 2, two more rows, both `level=0` |
//! | B | drag TAIL onto the middle band of DETAIL | `bookmark-drag-released placement=last-child`, `bookmark-move-report moved=1 reparented=1`, `move-bookmark`, and TAIL at `level=1` |
//! | C | press DETAIL's triangle | `bookmark-disclosure open=0`, `set-bookmark-open`, and **the item count unmoved with one row fewer drawn** |
//!
//! [`OutlinePlacement::LastChild`]: https://docs.rs/pdfcer-core

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;
use crate::trace::{Trace, TraceLine};

/// The mode the Bookmarks panel is reached in.
const MODE: &str = "read";
/// Supplied at launch so the panel is open before anything is aimed at it.
const INVOKE: &str = "view.panel_bookmarks";
/// The title box on the authoring row.
const TITLE_BOX: &str = "bookmarks.new_title";
/// The Add button.
const ADD_BUTTON: &str = "bookmarks.add";
/// The dock body the rows are drawn inside — the bound a clickable row must sit
/// within. See [`visible_rows`].
const PANEL_BODY: &str = "dock.body.view.panel_bookmarks";
/// The panel's per-frame census. Traced at the TOP of the body, before any row,
/// which is what makes it the frame separator [`visible_rows`] uses.
const CENSUS: &str = "bookmarks-panel";
/// One line per outline row drawn.
const ROW: &str = "bookmark-row";
/// The panel's line for a released drag.
const RELEASED: &str = "bookmark-drag-released";
/// The apply arm's line, carrying the engine's whole report.
const MOVE_REPORT: &str = "bookmark-move-report";
/// The funnel's line for the move verb.
const MOVED: &str = "move-bookmark";
/// The panel's line for a pressed disclosure triangle.
const DISCLOSURE: &str = "bookmark-disclosure";
/// The funnel's line for the expand/collapse verb.
const OPEN_SET: &str = "set-bookmark-open";
/// The prefix of the per-row triangle regions; the object number is appended.
const DISCLOSE_REGION: &str = "bookmarks.disclose.";

/// `DETAIL` — the first bookmark, and the one that ends up the parent.
///
/// ★ Spelled from the letters `crate::sys::vk` actually publishes. That module
/// adds virtual-key constants **one at a time, with a reason**, deliberately —
/// its own note says so — so a check invents a word from the alphabet that is
/// there rather than widening a shared file for a fixture name. `DETAIL` is
/// also the word `bookmark_edit` renames to, so a reader comparing the two
/// traces sees a name they recognise.
const PARENT_KEYS: [u16; 6] = [vk::D, vk::E, vk::T, vk::A, vk::I, vk::L];
/// `TAIL` — the second, and the one that is dragged.
///
/// ★ A **different length** from the first, deliberately, so a trace that
/// reported only a character count could still tell them apart. Nothing below
/// needs that today; it costs nothing, and it is the property `bookmark_edit`
/// had to go back and add after the fact.
const CHILD_KEYS: [u16; 4] = [vk::T, vk::A, vk::I, vk::L];

/// See the module documentation.
pub struct ABookmarkCanBeDraggedAndABranchCollapsed;

impl Check for ABookmarkCanBeDraggedAndABranchCollapsed {
    fn name(&self) -> &'static str {
        "a_bookmark_can_be_dragged_and_a_branch_collapsed"
    }

    fn defect(&self) -> &'static str {
        "the Bookmarks panel can create, rename and remove a bookmark and cannot MOVE one — an \
         outline in the wrong order can only be fixed by deleting a branch and re-authoring it, \
         which loses every destination on it — and a branch cannot be folded away, so a long \
         outline is a wall of rows"
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

/// The `bookmarks-panel` line's item count, or `None` if the panel did not draw.
fn census(session: &Session) -> Result<Option<usize>> {
    Ok(session
        .trace()?
        .last(CENSUS)
        .and_then(|line| line.get_usize("items")))
}

/// The rows of the **most recent frame** that are inside the panel body.
///
/// # ★★★ Why both filters, and why each was paid for
///
/// **The frame filter.** The trace holds every frame the application drew, and
/// this check needs to count rows — *"one row is drawn"* is its central
/// assertion. Counting `bookmark-row` lines across the whole trace counts
/// hundreds. The panel traces its census line at the **top** of its body,
/// before any row, so the lines after the last census are exactly the last
/// frame's rows.
///
/// **The visibility filter.** `bookmark-row` is a diagnostic line, not a
/// `ui_rect`: the panel writes one per row per frame **whether or not the row
/// is inside the visible scroll area**, because the listing has to be provable
/// from a trace without anybody clicking. `bookmark_edit` learned that the
/// expensive way on 2026-08-29 — its aim landed three thousand points below the
/// panel, off the window entirely, and it then reported that the
/// Selected-bookmark block never appeared. The panel was right; the aim was
/// three metres low.
///
/// ⇒ The general form, and this suite has now met it three times: **a trace
/// line written for every item is not a list of the items you can click.** The
/// `ui-rect` census answers *what is on screen*; a per-item diagnostic answers
/// *what was computed*.
fn visible_rows(trace: &Trace, body: Option<crate::geom::LRect>) -> Vec<&TraceLine> {
    let last_census = trace
        .lines
        .iter()
        .rposition(|line| line.event == CENSUS)
        .unwrap_or(0);
    trace.lines[last_census..]
        .iter()
        .filter(|line| line.event == ROW)
        .filter(|line| {
            let Some(rect) = line.get_rect("row") else {
                return false;
            };
            body.is_none_or(|body| rect.min.y >= body.min.y && rect.max.y <= body.max.y)
        })
        .collect()
}

/// Type one bookmark into the authoring row and press Add.
///
/// Factored because it happens twice and the second time must be identical to
/// the first — a set-up that differed between the two bookmarks would leave the
/// check unable to say which difference mattered.
fn author(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    keys: &[u16],
    label: &str,
) -> Result<()> {
    let trace = session.trace()?;
    let title_box = declared(&trace, ui_rect, TITLE_BOX).ok_or_else(|| {
        Error::new(format!(
            "no `{TITLE_BOX}` region while authoring {label}, so the authoring row is not on \
             screen and there is nothing to set up with. `bookmark_can_be_written` owns that \
             surface; SKIPPED rather than failed. Regions beginning `bookmarks`: {}.",
            list(&declared_names(&trace, ui_rect, "bookmarks"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(title_box))?;
    session.settle(8);
    for key in keys {
        driver.press(*key)?;
    }
    session.settle(8);
    let add = declared(&session.trace()?, ui_rect, ADD_BUTTON).ok_or_else(|| {
        Error::new(format!(
            "no `{ADD_BUTTON}` region to press while authoring {label}."
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(add))?;
    session.settle(20);
    Ok(())
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, types nine \
             letters, performs a drag and presses a disclosure triangle. Reported as SKIPPED \
             rather than passed: a check that did not run has learned nothing.",
        ));
    }
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
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("bookmark-move.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- A: two top-level bookmarks this check then reorganises -------------
    //
    // ★ Both go to the TOP LEVEL, and that is a property of the panel rather
    // than of this check: the authoring row files a new bookmark under
    // whichever row was last **clicked**, and nothing here clicks a row before
    // pressing Add. Authoring does not select what it authored — the add row
    // leaves the selection alone deliberately, because what a selection means
    // there is *the parent for the next add*.
    report.note("phase A: authoring DETAIL and TAIL, both at the top level".to_owned());
    // ★★ Measured BEFORE, never assumed to be zero. The fixture set is not
    // uniform — `four-pages.pdf` ships with a six-item outline and
    // `SW41177.pdf` has none — and a check that hard-coded either would SKIP on
    // half the corpus while reporting the fixture as the fault. Every count
    // below is a **delta** against this, which is what makes the check portable
    // across documents that already have bookmarks.
    let Some(before) = census(&session)? else {
        return Err(Error::new(format!(
            "the panel traced no `{CENSUS}` line before anything was authored, so it is not \
             reading the outline and nothing below can be measured."
        )));
    };
    author(&session, &driver, ui_rect, &PARENT_KEYS, "DETAIL")?;
    author(&session, &driver, ui_rect, &CHILD_KEYS, "TAIL")?;

    let authored = census(&session)?.unwrap_or(usize::MAX);
    if authored != before + 2 {
        return Err(Error::new(format!(
            "the set-up wanted two new bookmarks and the outline went {before} → {authored}. \
             That is `bookmark_can_be_written`'s subject, not this one — a move needs something \
             to move and somewhere to move it. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    }

    let trace = session.trace()?;
    let body = declared(&trace, ui_rect, PANEL_BODY);
    let rows = visible_rows(&trace, body);
    let rows_before_move = rows.len();
    // ★ The LAST TWO rows, not the first two. `add_outline_item` files a new
    // bookmark as the last child of the parent it is given, and this check
    // gives it the top level — so on a document that already had an outline the
    // two just authored are at the very end of the walk, after every branch
    // that was there before.
    let Some(&child) = rows.last() else {
        return Err(Error::new(format!(
            "the panel counted {authored} item(s) and drew no visible row in its last frame. \
             SKIPPED — a row that is scrolled out of view is a harness problem, not a defect in \
             the move. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let Some(&parent) = rows.len().checked_sub(2).and_then(|i| rows.get(i)) else {
        return Err(Error::new(format!(
            "only {rows_before_move} row(s) are visible, and this check needs the two it just \
             authored to be on screen together — one to lift and one to drop onto. SKIPPED: that \
             is the panel being scrolled, not the move being broken."
        )));
    };
    for (row, name) in [(parent, "DETAIL"), (child, "TAIL")] {
        if row.get_usize("level") != Some(0) {
            return Err(Error::new(format!(
                "{name} was authored at level {:?} rather than at the top level, so this check's \
                 0 → 1 oracle would prove nothing. SKIPPED: `{}`.",
                row.get("level"),
                row.raw
            )));
        }
    }
    let parent_id = parent.get_usize("id").ok_or_else(|| {
        Error::new(format!(
            "the `{ROW}` line carries no readable `id=`: `{}`. The triangle's region is named \
             after it, so phase C cannot aim without it.",
            parent.raw
        ))
    })?;
    // ★★ TAIL is identified by its OBJECT ID from here on, never by its
    // position. The move is about to change where it is in the walk — that is
    // the whole point of the gesture — so a check holding an index would be
    // holding the number the very edit it performs invalidates. That is the
    // hazard the panel itself is built to avoid, and a harness is not exempt
    // from it.
    let child_id = child.get_usize("id").ok_or_else(|| {
        Error::new(format!(
            "TAIL's row line carries no readable `id=`: `{}`. Without it the check cannot find \
             the bookmark again after the move, which is the one thing the move guarantees to \
             change.",
            child.raw
        ))
    })?;
    // ★ `rect=` to LIFT, because only the label is a widget. See the module
    // header's table; the strip's centre is empty space and a press there
    // starts no drag at all.
    let lift = child.get_rect("rect").ok_or_else(|| {
        Error::new(format!(
            "TAIL's row line carries no parsable `rect=`: `{}`. Without it there is no \
             coordinate for the label and no drag can begin.",
            child.raw
        ))
    })?;
    // ★ `row=` to DROP, because the band test is over the full-width strip.
    let onto = parent.get_rect("row").ok_or_else(|| {
        Error::new(format!(
            "DETAIL's row line carries no parsable `row=`: `{}`. That is the strip the landing \
             bands are measured against; the label's rectangle would test a different question.",
            parent.raw
        ))
    })?;
    report.note(format!(
        "set-up: the outline went {before} → {authored} items and draws {rows_before_move} rows; \
         DETAIL is object {parent_id} and TAIL object {child_id}, both at level 0"
    ));

    // --- B: drag TAIL onto the MIDDLE of DETAIL ------------------------------
    //
    // ★★ The middle of the strip, vertically and horizontally, which is the
    // `Into` band. A drop a quarter of the way up or down would be `Before` or
    // `After` — a reorder, not a re-parent — and the level would not move,
    // which is exactly the failure this aim is chosen to avoid.
    report.note("phase B: dragging TAIL onto the middle band of DETAIL".to_owned());
    let frame = session.frame()?;
    driver.drag(
        frame.declared_center(lift),
        frame.declared_at(onto, 0.5, 0.5),
    )?;
    session.settle(25);

    let trace = session.trace()?;
    let Some(released) = trace.events(RELEASED).last() else {
        return Ok(Some(format!(
            "the drag raised no `{RELEASED}` line, so the panel never saw a drag at all. Suspect, \
             in order: the row's `Sense` (a `Button` senses clicks only unless it is given \
             `click_and_drag`), the press landing outside the label — this check aims with \
             `rect=` for exactly that reason — or the release being read from a `Response` rather \
             than from raw pointer input. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the panel released the drag: `{}`", released.raw));
    match released.get("placement") {
        Some("last-child") => {}
        other => {
            return Ok(Some(format!(
                "THE DROP RESOLVED TO THE WRONG LANDING: `{}` reports placement={other:?}, and a \
                 drop on the vertical middle of a row is the nesting band. `before` or `after` \
                 means the three-band split is mis-measured — the edge bands are a quarter of the \
                 row each and the middle is the remaining half — and the operator would get a \
                 reorder every time they aimed at a re-parent.",
                released.raw
            )));
        }
    }
    if released.get("landing") != Some("Lands") {
        return Ok(Some(format!(
            "THE PANEL FORECAST THAT THE DROP WOULD DO NOTHING: `{}`. `Lands` is the only landing \
             that raises the move; `NoChange` means the shell's sibling arithmetic thinks TAIL is \
             already DETAIL's last child, and `OwnSubtree` means it thinks DETAIL is inside TAIL. \
             Both are `reorder::landing_for`, and both would leave a live gesture doing nothing.",
            released.raw
        )));
    }

    let Some(engine) = trace.events(MOVE_REPORT).last() else {
        return Ok(Some(format!(
            "the panel released a landing drop and no `{MOVE_REPORT}` line followed, so the \
             action reached no apply arm. A refused `vector_edit` traces `{MOVED}-refused`; look \
             for that first.",
        )));
    };
    report.note(format!("the engine reported: `{}`", engine.raw));
    if engine.get("moved") != Some("1") {
        return Ok(Some(format!(
            "THE ENGINE WROTE NOTHING: `{}` reports moved=0, which `move_outline_item` returns \
             for a placement the bookmark already occupies — no objects written, no undo entry. \
             The panel had forecast `Lands`, so the shell's arithmetic and the engine's disagree, \
             and the operator gets a drag that does nothing with a live caret under it.",
            engine.raw
        )));
    }
    // ★★ The engine's own word for what kind of move it was, asserted beside
    // the level below. `OutlineMove::reparented` is carried separately from
    // comparing the two parent ids precisely because it is *"the fact a
    // disclosure sentence turns on"*, so a build that got it wrong would word
    // a re-parent as a reorder in the status bar.
    if engine.get("reparented") != Some("1") {
        return Ok(Some(format!(
            "THE MOVE WAS A REORDER, NOT A RE-PARENT: `{}` reports reparented=0. The drop was on \
             the nesting band, so TAIL should have changed owner. A reorder that lands where a \
             re-parent was asked for is the defect a three-band drop model produces when the \
             pointer's y is mis-read, and on a long outline it looks like nothing happened.",
            engine.raw
        )));
    }
    if trace.events(MOVED).count() == 0 {
        return Ok(Some(format!(
            "the apply arm reported `{}` and no `{MOVED}` funnel line followed, so the edit never \
             went through `vector_edit` — which means no epoch bump, no undo entry and no \
             re-raster. The document changed and the application does not know it.",
            engine.raw
        )));
    }

    let trace = session.trace()?;
    let after_move = census(&session)?.unwrap_or(usize::MAX);
    if after_move != authored {
        return Ok(Some(format!(
            "THE MOVE CHANGED HOW MANY BOOKMARKS EXIST: the outline went from {authored} to \
             {after_move}. A move relinks; it neither adds nor removes. A count that FELL is the \
             move having dropped the subtree it was supposed to carry; one that ROSE is a copy \
             wearing a move's name."
        )));
    }
    let rows = visible_rows(&trace, declared(&trace, ui_rect, PANEL_BODY));
    let rows_after_move = rows.len();
    // By OBJECT ID, never by position — see where `child_id` is taken.
    let nested = rows
        .iter()
        .find(|row| row.get_usize("id") == Some(child_id));
    let Some(nested) = nested else {
        return Ok(Some(format!(
            "after the move the panel drew no row for TAIL (object {child_id}), although the \
             outline still holds {after_move} items. It has gone out of view without anybody \
             collapsing anything, and `move_outline_item` states that a destination parent which \
             was a LEAF is left open — so a bookmark dropped on one must still be visible. Look \
             at whether the walk recurses into an open parent's children. Rows: {}.",
            rows.iter()
                .map(|r| r.raw.as_str())
                .collect::<Vec<_>>()
                .join(" | ")
        )));
    };
    if rows_after_move != rows_before_move {
        return Ok(Some(format!(
            "THE MOVE CHANGED HOW MANY ROWS ARE DRAWN: {rows_before_move} → {rows_after_move}, \
             and a re-parent onto a leaf hides nothing. `move_outline_item` leaves a destination \
             parent that had no prior children OPEN, on its own stated reasoning — *the \
             alternative is dropping the bookmark into a collapsed parent where it is invisible \
             the instant the operator put it there* — so a row going missing here means the \
             engine's promise or the panel's walk is not holding."
        )));
    }
    // ★★★ The oracle. A level cannot be reached by a reorder.
    match nested.get_usize("level") {
        Some(1) => {
            report.note(format!("★★ TAIL is now nested: `{}`", nested.raw));
        }
        other => {
            return Ok(Some(format!(
                "TAIL IS AT LEVEL {other:?} AND SHOULD BE AT 1: `{}`. It was authored at the top \
                 level and dropped on DETAIL's nesting band, so `OutlineItem::level` must have \
                 moved 0 → 1. Level 0 means the drop reordered instead of re-parenting; a deeper \
                 level means it landed inside something else again.",
                nested.raw
            )));
        }
    }

    // --- C: collapse the branch ---------------------------------------------
    //
    // ★ The triangle exists only because DETAIL now HAS children, which is
    // itself a consequence of phase B. That ordering is deliberate: a check
    // that collapsed a branch it had not built would be testing the fixture.
    report.note("phase C: pressing DETAIL's disclosure triangle".to_owned());
    let region = format!("{DISCLOSE_REGION}{parent_id}");
    let trace = session.trace()?;
    let Some(triangle) = declared(&trace, ui_rect, &region) else {
        return Ok(Some(format!(
            "NO DISCLOSURE TRIANGLE ON A ROW THAT NOW HAS A CHILD. DETAIL is object {parent_id} and \
             published no `{region}` region, although TAIL is filed under it. The triangle is \
             drawn for a row whose `children` is non-empty and withheld from a leaf — §12.3.3 \
             Table 153 makes `/Count` required only for an item with descendants, so a leaf has \
             no open-or-closed state and gets no control. A parent that grew one by a move must \
             grow the triangle with it. Regions beginning `bookmarks`: {}.",
            list(&declared_names(&trace, ui_rect, "bookmarks"))
        )));
    };
    driver.click_at(session.frame()?.declared_center(triangle))?;
    session.settle(25);

    let trace = session.trace()?;
    let Some(pressed) = trace.events(DISCLOSURE).last() else {
        return Ok(Some(format!(
            "pressing the triangle raised no `{DISCLOSURE}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    if pressed.get("open") != Some("0") {
        return Ok(Some(format!(
            "the triangle asked to OPEN a branch that was already open: `{}`. `open=` is the \
             state being requested, not the state the row is in, so a build that traced the \
             current state here would report the opposite of what it did.",
            pressed.raw
        )));
    }
    if trace.events(OPEN_SET).count() == 0 {
        return Ok(Some(format!(
            "the panel raised `{}` and no `{OPEN_SET}` funnel line followed, so \
             `set_outline_open` was never reached or refused it. Expand and collapse is a \
             DOCUMENT edit here — §12.3.3 carries open-or-closed as the sign on `/Count` and \
             defines no `/Open` key — so it must go through the edit funnel like any other.",
            pressed.raw
        )));
    }

    // ★★★ The two numbers that must now disagree. See the module header.
    let after_collapse = census(&session)?.unwrap_or(usize::MAX);
    let rows = visible_rows(&trace, declared(&trace, ui_rect, PANEL_BODY));
    if after_collapse != authored {
        return Ok(Some(format!(
            "COLLAPSING REMOVED BOOKMARKS FROM THE DOCUMENT: the count went from {authored} to \
             {after_collapse}. `bookmarks-panel items=` is `read_outline`'s census of every item \
             at every level, collapsed branches included, so it must not move when a branch is \
             folded. A build that 'collapsed' by deleting the subtree lands exactly here."
        )));
    }
    // ★★★ EXACTLY one fewer, not "fewer". DETAIL holds exactly one child, so
    // folding it hides exactly one row. A "shorter than before" assertion would
    // pass on a build that emptied the list, which is also shorter — the trap
    // `pdfcer-core` reported from its own delete Pass, where *"every defect we
    // injected leaves a shorter list. One leaves it empty, which is also
    // shorter."*
    let expected = rows_before_move - 1;
    if rows.len() != expected {
        return Ok(Some(format!(
            "THE PANEL DRAWS {} ROWS AFTER THE BRANCH WAS COLLAPSED, and it should draw \
             {expected}. The document holds {after_collapse} bookmarks and must show one fewer \
             row; that disagreement IS the feature. `read_outline` populates `children` for a \
             closed item exactly as for an open one, so a walk that recurses unconditionally \
             writes `/Count`'s sign into the file and changes nothing on screen — a control that \
             appears not to work, which is what this panel did before `Pass 161.0`. A count that \
             fell FURTHER is the collapse hiding rows it does not own. Rows: {}.",
            rows.len(),
            rows.iter()
                .map(|r| r.raw.as_str())
                .collect::<Vec<_>>()
                .join(" | ")
        )));
    }
    report.note(format!(
        "★★★ the document still holds {after_collapse} bookmarks and the panel now draws {} rows \
         instead of {rows_before_move} — the collapse hid the branch without touching it",
        rows.len()
    ));
    Ok(None)
}
