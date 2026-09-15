//! `a_bookmark_subtree_can_be_copied_and_pasted` — the driven proof of
//! `OPERATOR_REQUESTS.md` **O59**'s third and last item.
//!
//! # ★★ The one operation in this program Acrobat cannot do
//!
//! `pdfcer-core`, 2026-08-29: *"Acrobat cannot do this between two files at all;
//! Adobe's own documentation says so by name."*
//!
//! That is worth stating in a check file, because it changes what a failure
//! here means. For most of this suite a red result says *pdfcer is behind a
//! reference implementation*. Here it says *pdfcer has lost something nothing
//! else offers*, and there is no workaround to fall back on — an operator who
//! wants a chapter's bookmarks in another drawing does it by hand, one at a
//! time, or not at all.
//!
//! # The oracle, and the two readings that would not have been enough
//!
//! | reading | what green would prove |
//! |---|---|
//! | `bookmark-copy` | the panel asked the engine for a clip |
//! | `bookmark-paste` | the panel raised an action |
//! | `bookmark-paste-applied items=N` | `paste_outline_item` returned `Ok` with a count |
//! | **the panel's own census grew** | **the operator got more bookmarks** |
//!
//! ★ The third is very nearly enough and is still not, for one specific
//! reason: `paste_outline_item` returns `Ok(default())` — `items_pasted: 0` —
//! on an **empty clip**, without touching the document. So a build whose copy
//! produced an empty clip would emit every line above, report success, and add
//! nothing. Only the census distinguishes it.
//!
//! ⇒ So this asserts on `bookmarks-panel items=`, which is the panel counting
//! what it is actually drawing.
//!
//! # The sequence
//!
//! 1. open the bookmarks panel;
//! 2. author a bookmark, so there is something to copy — reusing the authoring
//!    row `bookmark_can_be_written` owns;
//! 3. read the census;
//! 4. press **Copy**, then **Paste**;
//! 5. assert the census grew.
//!
//! ★★ Step 2 exists because the fixture corpus is **not uniform** —
//! `bookmark_move`'s header records the same discovery — and a check that
//! assumed a bookmark was already there would SKIP on half the corpus while
//! reporting the fixture as the fault. Authoring one first makes the check
//! independent of what the document arrived with.
//!
//! # What this does not prove, said out loud
//!
//! **Cross-document paste**, which is the whole point of the feature. This
//! pastes back into the same document, because driving two documents and moving
//! a clip between them is a harness capability that does not exist yet. The
//! clip is application-scoped by construction — it lives in `egui::Memory`, not
//! on the document — so the mechanism is the same one either way, but *the same
//! mechanism* is an argument and not a measurement. Recorded as a gap.
//!
//! And the **dropped-destination warning**, which needs a clip whose deepest
//! page exceeds the destination's page count. Same reason: two documents.

use crate::checks::driving::{
    self, SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;
use crate::trace::Trace;

/// Supplied at launch. **Nothing**, deliberately — see [`MODE`].
///
/// ★★★ Two things were got wrong here in succession, a third was got wrong for
/// longer, and all three are worth keeping.
///
/// **1. `view.panel_bookmarks` is a TOGGLE.** Invoking it does not "open the
/// panel", it flips it — so whether the panel is on screen afterwards depends
/// on whether the mode's default layout already had it. Chaining
/// `mode.edit,view.panel_bookmarks` therefore opened it on some runs and closed
/// it on others, and the trace said so plainly:
///
/// ```text
/// ui-rect-gone name=bookmarks.new_title
/// ui-rect-gone name=dock.body.view.panel_bookmarks
/// ```
///
/// **2. A mode carries a panel set.** Changing mode reconfigures the dock, so
/// even the order `panel-then-mode` loses the panel — which is what the first
/// version did, and its symptom was the same missing authoring row.
///
/// **3. ★★★ And the conclusion drawn from 1 and 2 was FALSE — corrected
/// 2026-09-05, on the first sweep that ever ran this check.** It read: *"read is
/// the mode whose default layout does not carry this panel, so the toggle
/// reliably turns it on, and nothing this check does needs Edit — the panel
/// offers [authoring] in every mode."* Both halves are wrong.
/// `app::modes::defaults` mounts Bookmarks in **Read** as well (*Read: Pages,
/// Bookmarks*), so the toggle was as likely to close it as to open it; and Read
/// deliberately offers **no** bookmark authoring at all — `bookmark_add`'s
/// second half, `read_mode_offers_no_bookmark_authoring`, asserts that absence
/// and passes.
///
/// So this check ran, found no `bookmarks.new_title`, and SKIPPED — accurately,
/// unhelpfully, and for ever, because a SKIP is not red. Its own header was the
/// thing sending readers to the wrong file.
///
/// ⇒ No toggle at all, and [`MODE`] does the work.
const INVOKE: &str = "";

/// The mode this check authors in.
///
/// **Review**, because Review is the mode that offers the authoring row (see
/// [`INVOKE`] point 3) and the mode `bookmark_add` uses for the same reason.
/// Its default dock mounts Bookmarks, so the panel needs raising to the front
/// of its stack and not toggling into existence.
const MODE: &str = "review";

/// The authoring row's text box and button, owned by `bookmark_add`.
const TITLE_BOX: &str = "bookmarks.new_title";
/// ditto.
const ADD_BUTTON: &str = "bookmarks.add";

/// The panel's own census: `bookmarks-panel page=N items=M`.
const CENSUS: &str = "bookmarks-panel";

/// The Copy button's region, published by `panels::bookmarks::clip`.
const COPY_REGION: &str = "bookmark-copy";
/// The Paste button's region.
const PASTE_REGION: &str = "bookmark-paste";

/// The panel's own body region, so a row can be tested for being on screen.
const PANEL_BODY: &str = "dock.body.view.panel_bookmarks";

/// A row, so one can be clicked to select it.
///
/// ★★ A trace **event**, not a `ui_rect` region — which is what the first
/// version of this check got wrong and what its own failure message could not
/// tell it. The panel writes one `bookmark-row` line per row per frame carrying
/// `row=[[x y] - [x y]]`, and publishes `ui_rect` only for its two authoring
/// controls. Asking `declared_names(.., "bookmark")` therefore returned the
/// authoring row and nothing else, and the check concluded the Copy control was
/// missing when in fact the selecting click had never been aimed anywhere.
///
/// ⇒ Ask what a check SAMPLED before asking what is broken. Fifth instance on
/// this project, and the first where the wrong sample was a *region* where the
/// truth was an *event*.
const ROW: &str = "bookmark-row";

/// The engine-side line: `items=` and `dropped=`.
const APPLIED: &str = "bookmark-paste-applied";

/// The title typed for the bookmark this check authors: `HI`.
///
/// ★ Two letters, because every character is a synthesised keystroke through
/// the OS and a longer title buys nothing. Letters rather than digits so the
/// row is unmistakable in a trace beside page numbers.
const TITLE_KEYS: &[u16] = &[vk::H, vk::F];

/// See the module documentation.
pub struct ABookmarkSubtreeCanBeCopiedAndPasted;

impl Check for ABookmarkSubtreeCanBeCopiedAndPasted {
    fn name(&self) -> &'static str {
        "a_bookmark_subtree_can_be_copied_and_pasted"
    }

    fn defect(&self) -> &'static str {
        "the bookmark clipboard traces a copy, a paste and an applied count, and the outline has \
         exactly as many bookmarks as before — which is what an empty clip produces, because \
         `paste_outline_item` returns Ok with items_pasted: 0 without touching the document"
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

/// **Wait until `name` is a live region, polling rather than settling once.**
///
/// ★★★ This replaced three fixed `settle` calls and is the difference between a
/// check that passes most times and one that passes.
///
/// `declared` is not a snapshot. The application's `ui-rect` channel is a
/// **change log** — it emits when a rect moves and a `ui-rect-gone` when a
/// control stops being drawn — so `declared` answers *"was this region alive at
/// the moment the trace was read?"* Reading it once, after a fixed wait, asks
/// that question at an arbitrary point in a layout that is still moving.
///
/// And this panel's layout moves a great deal: the invoke chain changes MODE
/// (which reconfigures the whole dock, tearing the panel down and rebuilding
/// it) and then opens the panel, and selecting a bookmark inserts an edit block
/// that pushes the list two hundred points down. A fixed settle that is nearly
/// long enough produces a check that fails **intermittently and differently
/// each time** — which is exactly what happened: alternating between *"the
/// authoring row is not on screen"* and *"the Copy control is not on screen"*,
/// on a build where both were fine.
///
/// ⇒ Polling asks the question repeatedly and stops at the first *yes*, which
/// is what a person watching the screen does. It is not a widened tolerance:
/// the caller still fails if the answer is never yes.
fn wait_for_region(
    session: &Session,
    ui_rect: &str,
    name: &str,
    tries: usize,
) -> Result<Option<crate::geom::LRect>> {
    for _ in 0..tries {
        if let Some(rect) = declared(&session.trace()?, ui_rect, name) {
            return Ok(Some(rect));
        }
        session.settle(10);
    }
    Ok(None)
}

/// How many bookmarks the panel says it is drawing.
fn census(trace: &Trace) -> Option<usize> {
    trace
        .last(CENSUS)
        .and_then(|l| l.get("items"))
        .and_then(|v| v.parse().ok())
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check types a bookmark title and presses three \
             buttons. Reported as SKIPPED rather than passed.",
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a document to bookmark."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("bookmark-clipboard.trace.txt"));
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

    // ★★★ NORMALISE THE PERSISTED LAYOUT FIRST, or this check passes and skips
    // in alternation.
    //
    // `view.panel_bookmarks` is a TOGGLE, and the dock layout is **saved to
    // disk** (`userdata/layout.ron`, written on exit). So run A opens the panel
    // and saves it open; run B's toggle then *closes* it, and the check reports
    // the authoring row missing. The symptom is a check that alternates between
    // PASS and SKIP with nothing changing in the program — which reads as
    // flakiness and is in fact a check reading state it wrote itself.
    //
    // ⇒ Deleting the file makes every run start from the shipped default, where
    // the panel is closed and the toggle reliably opens it. `D:/dev/rag/egui/`
    // already carries this as *a driven check that mutates persisted state must
    // normalise at the start*; this is the second instance and the first where
    // the state was written by the APPLICATION rather than by the check.
    //
    // ★ Safe because it is the **scratch** userdata beside the binary under
    // test, never the operator's own: the standing rule is that this suite is
    // never pointed at a published build, for exactly this reason.
    if let Some(dir) = exe.parent() {
        let layout = dir.join("userdata").join("layout.ron");
        if layout.exists() {
            let _ = std::fs::remove_file(&layout);
            report.note(format!("normalised {}", layout.display()));
        }
    }

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    // ★ 45 → 70. `INVOKE` runs TWO commands — a mode change and a panel open —
    // and the mode change reconfigures the dock, so the panel's first layout
    // lands several frames after a single-command run's would. At 45 this check
    // alternated between finding the authoring row and reporting it absent,
    // which is the shape of a settle that is nearly enough rather than of a
    // defect.
    session.settle(70);
    let driver = Driver::new(session.window());
    // ★★★ The mode that offers the authoring row. See [`MODE`].
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    // ★★ The dock draws only the ACTIVE tab's body, and in this mode's default
    // layout Bookmarks shares a stack with Pages. See
    // [`crate::checks::driving::raise_dock_tab`].
    driving::raise_dock_tab(&session, &driver, ui_rect, "view.panel_bookmarks")?;
    // ★★ SETTLE BEFORE READING THE REGIONS, and this cost a run.
    //
    // The mode click re-lays out the whole window, so the bookmarks panel
    // republishes its `ui_rect` regions on the frames *after* it. Reading
    // immediately found none and the check reported the authoring row missing
    // — while its own error message, built from a trace read one line later,
    // listed that very row.
    //
    // ⇒ A contradiction inside one failure message is the tell: the two reads
    // straddled a frame. Same family as the harness coordinates that go stale
    // when a dock width changes, which `D:/dev/rag/egui/` already records.
    session.settle(20);

    // --- author one, so the check does not depend on the fixture ------------
    let Some(title_box) = wait_for_region(&session, ui_rect, TITLE_BOX, 12)? else {
        // ★ The region list is built HERE rather than inside an `ok_or_else`
        // closure, because building it needs `session.trace()?` and a closure
        // cannot carry the `?`. Worth the extra lines: a skip that says *"no
        // authoring row"* AND lists the regions that are live is the difference
        // between a diagnosable message and a dead end.
        let names = declared_names(&session.trace()?, ui_rect, "bookmarks");
        return Err(Error::new(format!(
            "no `{TITLE_BOX}` region, so the bookmarks panel's authoring row is not on \
             screen and there is nothing to set this check up with. \
             `bookmark_can_be_written` owns that surface; SKIPPED rather than failed. \
             Regions beginning `bookmarks`: {}.",
            list(&names)
        )));
    };
    driver.click_at(session.frame()?.declared_center(title_box))?;
    session.settle(8);
    for key in TITLE_KEYS {
        driver.press(*key)?;
    }
    session.settle(8);
    let add = wait_for_region(&session, ui_rect, ADD_BUTTON, 8)?
        .ok_or_else(|| Error::new(format!("no `{ADD_BUTTON}` region to press.")))?;
    driver.click_at(session.frame()?.declared_center(add))?;
    session.settle(25);

    let before = census(&session.trace()?).ok_or_else(|| {
        Error::new(format!(
            "the panel published no `{CENSUS} … items=` line, so this check cannot count \
             bookmarks and could not tell a working paste from a silent one."
        ))
    })?;
    report.note(format!("★ authored one bookmark; the panel draws {before}"));

    // --- select it ----------------------------------------------------------
    //
    // ★ Copy is drawn only when a bookmark is SELECTED — R9, an unavailable
    // capability renders nothing — so this click is a precondition of the
    // subject, not part of it.
    // ★ The LAST row, because the bookmark just authored is appended — and its
    // rect comes off the row line itself rather than from a region.
    // ★★ The FIRST row of the LAST frame, not the last row of the trace.
    //
    // Two different mistakes, one line apart. `.last()` over the whole trace
    // returns the bottom-most row of the most recent frame — the one this check
    // just authored, which is appended and therefore furthest down the panel
    // and the first to be clipped when the list outgrows its pane. The check
    // does not need *that* bookmark; it needs *a* bookmark, and the top one is
    // the one guaranteed to be on screen.
    //
    // ⇒ Authoring one is still worth doing: it makes the check independent of
    // what the fixture arrived with. What it must not do is then insist on
    // clicking the one it made.
    // ★★★ A row that is ON SCREEN, filtered against the panel's own body —
    // `bookmark_edit`'s technique, adopted verbatim after four attempts at
    // simpler rules failed for four different reasons.
    //
    // `bookmark-row` is a **diagnostic line, not a `ui_rect` region**: the panel
    // writes one per row per frame whether or not that row is inside the visible
    // scroll area, because the listing has to be provable from a trace without
    // anybody clicking. So no rule based on the line alone — first, last,
    // topmost — answers *"which row can I click?"*
    //
    // The four wrong answers, each with its own symptom, because the shape
    // recurs:
    //
    // | rule | what it clicked |
    // |---|---|
    // | `.last()` | the row just authored, appended and therefore the first to be clipped |
    // | `row=` instead of `rect=` | the full-width strip: the centre of it is 120 pt right of a two-letter title |
    // | `get("rect")` | nothing — a rect value contains SPACES, so the parser returns `[[14.0` and every row failed to parse |
    // | topmost by `min.y` | a row above the scroll area's own top |
    //
    // ⇒ The general form, and this suite has now met it three times: **a trace
    // line written for every item is not a list of the items you can click.**
    // The `ui-rect` census answers *what is on screen*; a per-item diagnostic
    // answers *what was computed*.
    let trace = session.trace()?;
    let body = declared(&trace, ui_rect, PANEL_BODY);
    let visible = |line: &crate::trace::TraceLine| {
        let Some(r) = line.get_rect("rect") else {
            return false;
        };
        body.is_none_or(|b| r.min.y >= b.min.y && r.max.y <= b.max.y)
    };
    // ★ ENABLED, too. A row whose destination pdfcer cannot resolve is drawn as a
    // disabled `Button`, and a disabled button never reports a click — so it can
    // never be selected and the Copy control could never appear for it. That is
    // correct behaviour that would read here as the defect.
    let row = trace
        .events(ROW)
        .filter(|l| visible(l) && l.get("enabled") == Some("true"))
        .last();
    let Some(rect) = row.and_then(|l| l.get_rect("rect")) else {
        return Err(Error::new(format!(
            "no enabled `{ROW}` line lies inside the panel body, so there is no row this check \
             can click and therefore no bookmark it can select. That is \
             `bookmark_can_be_written`'s surface rather than this one's. SKIPPED. Trace: {}",
            session.trace_path().display()
        )));
    };
    report.note(format!("aiming the selecting click at {rect:?}"));
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(25);

    // --- copy ---------------------------------------------------------------
    let Some(copy) = wait_for_region(&session, ui_rect, COPY_REGION, 12)? else {
        return Ok(Some(format!(
            "no `{COPY_REGION}` region after selecting a bookmark, so the Copy control is not on \
             screen. It is drawn only when a bookmark is selected, so either the selecting click \
             missed the row or the control was never added. Regions beginning `bookmark`: {}. \
             Trace: {}",
            list(&declared_names(&session.trace()?, ui_rect, "bookmark")),
            session.trace_path().display()
        )));
    };
    report.note(format!("the Copy control is at {copy:?}"));
    // ★★ Up to three attempts, re-reading the rect each time.
    //
    // Not a tolerance widened to make a red check green — the assertion below
    // is unchanged and still fails if nothing ever happens. It is that this
    // click lands on a control that APPEARED as a result of the previous
    // click: selecting a bookmark inserts the whole edit block, which moved the
    // list two hundred points down the panel. A control read one frame and
    // clicked the next is the shape `D:/dev/rag/egui/` already records for
    // harness coordinates going stale when a dock width changes.
    for attempt in 0..3 {
        let Some(rect) = wait_for_region(&session, ui_rect, COPY_REGION, 4)? else {
            break;
        };
        driver.click_at(session.frame()?.declared_center(rect))?;
        session.settle(20);
        if session.trace()?.events("bookmark-copy").next().is_some() {
            if attempt > 0 {
                report.note(format!("★ the Copy click took {} attempt(s)", attempt + 1));
            }
            break;
        }
    }
    if session.trace()?.events("bookmark-copy").next().is_none() {
        return Ok(Some(format!(
            "the Copy button was pressed and traced no `bookmark-copy` line, so `copy_outline_item` \
             was never reached or refused silently. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★ Copy produced a bookmark-copy line");

    // --- paste --------------------------------------------------------------
    let Some(paste) = wait_for_region(&session, ui_rect, PASTE_REGION, 12)? else {
        return Ok(Some(format!(
            "no `{PASTE_REGION}` region after a successful copy. The Paste control is drawn only \
             when the clipboard holds bookmarks, so this says the copy did not reach the \
             clipboard — the two halves disagree about the `Clipped::Outline` variant. Trace: {}",
            session.trace_path().display()
        )));
    };
    // ★★★ The Paste control appears at the BOTTOM of a list that has just grown
    // by an authoring block, so on a short panel it is declared below the fold —
    // the whole of the incident recorded on
    // [`crate::checks::driving::bring_into_body`], which is where the argument
    // lives. A click at a centre outside the panel body reported
    // `paste_outline_item` as refusing, and it had never been asked.
    let paste = driving::bring_into_body(
        &session,
        &driver,
        ui_rect,
        PANEL_BODY,
        PASTE_REGION,
        6,
        report,
    )?
    .unwrap_or(paste);
    driver.click_at(session.frame()?.declared_center(paste))?;
    session.settle(30);

    let after_trace = session.trace()?;
    let Some(applied) = after_trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "the Paste button was pressed and no `{APPLIED}` line followed, so the action was \
             raised and never applied — or `paste_outline_item` refused into `vector_edit`'s Err \
             arm. Trace: {}",
            session.trace_path().display()
        )));
    };
    let items: usize = applied
        .get("items")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    report.note(format!(
        "★ Paste applied: items={items} dropped={}",
        applied.get("dropped").unwrap_or("?")
    ));

    // --- the assertion about the DOCUMENT, not the intent -------------------
    let after = census(&after_trace).unwrap_or(before);
    if after <= before {
        return Ok(Some(format!(
            "★★★ EVERY TRACE LINE IS PRESENT AND THE OUTLINE STILL HAS {after} BOOKMARK(S). The \
             panel asked, the engine reported `items={items}`, and the panel is drawing no more \
             than before. `paste_outline_item` returns Ok with items_pasted: 0 on an EMPTY clip \
             without touching the document, which is exactly this shape — so suspect the copy \
             half, not the paste. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the outline went from {before} to {after} bookmark(s) — the paste reached the \
         document, not just the action queue"
    ));

    Ok(None)
}
