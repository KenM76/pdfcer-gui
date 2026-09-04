//! `bookmark_add` — **a bookmark can be written, and the panel gets it back.**
//!
//! # ★ Why the fixture having NO bookmarks is the point, not a limitation
//!
//! `SW41177.pdf` is a CAD export with no outline. That is the state the
//! Bookmarks panel spent its whole life unable to leave: it drew *"no
//! bookmarks"* and returned early, so the one document that most needs a first
//! bookmark was the one document where nothing could be added.
//!
//! The early return is gone, and this check is what proves it stayed gone. A
//! unit test cannot: the panel body takes an `OpenDoc` and an `egui::Ui`, and
//! the guard that would come back is a two-line `if outline.items.is_empty() {
//! return }` that every unit test of `read_outline` would still pass over.
//!
//! # ★★ What is asserted, and the one number that must NOT be used
//!
//! The engine's reply on `add_outline_item` spends its longest section on this
//! and calls it *"not a footnote — the entire difficulty of the feature"*:
//!
//! > `/Count` on the outline root counts **visible** items. Adding a bookmark
//! > under a **collapsed** ancestor does not change it. A UI that says "added
//! > N" by diffing that number reports **zero for a correct save**.
//!
//! So this check diffs `bookmarks-panel items=`, which is
//! `read_outline`'s **walked-tree** count and not `/Count` — every item at
//! every depth, open or closed. The distinction is invisible on this fixture
//! (a first top-level bookmark moves both) and would become visible the moment
//! anybody adds a nested case, which is exactly when a harness quietly
//! measuring the wrong quantity does its damage.
//!
//! # ★ The greyed Add button is asserted in BOTH states
//!
//! R9 reserves greying for *temporarily* unavailable, always explained. An
//! empty title is the textbook case — one keystroke away from live — and
//! `panels::bookmarks::add` greys it with a hover explanation rather than
//! hiding the row, because the row is the whole of the feature and an operator
//! would go looking for where bookmarks are added.
//!
//! Asserting only the live state would let the control ship permanently
//! enabled, which turns the empty-title press into an engine refusal an
//! operator never sees. Asserting only the disabled state would let it ship
//! permanently disabled, which is D5's family: a visible control that does
//! nothing.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | open the Bookmarks panel | `bookmarks.new_title` and `bookmarks.add` declared, on a document with no outline |
//! | B | read the census | `bookmarks-panel items=N` |
//! | C | click Add with the title empty | **no** `bookmark-add` trace — the control is greyed |
//! | D | type a title, click Add | `bookmark-add chars=5`, then `add-bookmark page=… epoch=…` |
//! | E | read the census again | `items = N + 1`, and no `add-bookmark-refused` |

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode the Bookmarks panel is **authored** in.
///
/// ★★★ `review`, not `read`, since 2026-09-04 — and the change is a change of
/// SUBJECT rather than of route.
///
/// This said `read` for the whole of its life, with no argument beside it:
/// Read is the mode the application starts in, so it was the cheapest way to
/// reach a panel that is shown in all three. That made it an accident, not a
/// decision — and it quietly asserted that **Read offers bookmark authoring**.
///
/// It did, and an outside reviewer flagged it (`REVIEW_TRIAGE.md` A5): the mode
/// whose whole promise is that it changes nothing was offering Add, Rename,
/// Remove, Copy, Cut and a drag hint. `MODES_AND_PANELS.md`'s own panel table
/// already draws that line by name, giving Read *"Comments (read)"* against
/// Review's *"Comments (authoring)"*; Bookmarks was granted to Read with no
/// authoring qualifier and nobody split the panel.
///
/// The authoring half is now gated on `Capabilities::authors_anything`, so
/// **Review is the lowest mode that has it** — which is also the right mode to
/// test it in, because it proves the gate admits more than Edit. A bookmark is
/// document *structure*, not page content, so Review must keep it.
///
/// ⇒ The check now fails if authoring is missing from Review, and
/// `read_mode_offers_no_bookmark_authoring` is the other half of the pair.
const MODE: &str = "review";
/// The command that shows the panel.
const PANEL_ITEM: &str = "ribbon.item.view.panel_bookmarks";
/// The panel's own dock tab, declared by the dock whenever it is showing.
///
/// ★ The evidence that the panel is OPEN, independent of anything its body
/// draws — which is what lets an absence test tell "nothing is offered"
/// from "nothing opened".
const PANEL_TAB: &str = "dock.tab.view.panel_bookmarks";
/// The title box the panel publishes.
const TITLE_BOX: &str = "bookmarks.new_title";
/// The Add button the panel publishes, in BOTH its greyed and its live state.
const ADD_BUTTON: &str = "bookmarks.add";
/// The panel's per-frame census.
const CENSUS: &str = "bookmarks-panel";
/// The press the panel traces before raising its action.
const PRESSED: &str = "bookmark-add";
/// The funnel's success line for the verb.
const APPLIED: &str = "add-bookmark";
/// `TITLE`, five keystrokes.
const TITLE_KEYS: [u16; 5] = [vk::T, vk::I, vk::T, vk::L, vk::E];

/// See the module documentation.
pub struct BookmarkCanBeWritten;

impl Check for BookmarkCanBeWritten {
    fn name(&self) -> &'static str {
        "bookmark_can_be_written"
    }

    fn defect(&self) -> &'static str {
        "the Bookmarks panel is read-only — it lists an outline and offers no way to add to one, \
         or offers one that does not reach the document, so a drawing exported without bookmarks \
         can never be given its first"
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
/// **Read mode offers no way to change the outline** — the other half of the
/// pair, and the regression test for `REVIEW_TRIAGE.md` A5.
///
/// # The defect this detects
///
/// An outside reviewer, 2026-09-03: *"Read mode offers 'Add a bookmark'."* It
/// did — a title field, a parent line and an Add button, plus Rename, Remove,
/// Copy, Cut and a drag hint, in the mode whose entire promise is that it
/// cannot change the document.
///
/// # ★★ Why this exists as well as its sibling, and not instead of it
///
/// `BookmarkCanBeWritten` proves the authoring row is REACHABLE — in Review,
/// since 2026-09-04. This one proves it is ABSENT in Read. Either alone is
/// satisfiable by a build that is simply wrong in the other direction: a panel
/// that never draws the row passes an absence test perfectly, and a panel that
/// draws it everywhere passes a presence test perfectly. **The pair is the
/// assertion**; neither half is.
///
/// ★ It is also the shape this project keeps being caught by. The sibling said
/// `MODE = "read"` for its whole life with no argument beside it — Read was
/// merely the mode the application starts in — and that accident quietly
/// asserted the defect was correct behaviour. An absence test written down
/// beside it is what stops the next accident being read as a decision.
pub struct ReadModeOffersNoBookmarkAuthoring;

impl Check for ReadModeOffersNoBookmarkAuthoring {
    fn name(&self) -> &'static str {
        "read_mode_offers_no_bookmark_authoring"
    }

    fn defect(&self) -> &'static str {
        "Read mode's Bookmarks panel offers to add, rename, remove, copy or cut a bookmark — \
         authoring controls in the one mode whose whole promise is that it changes nothing"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive_read(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Open the Bookmarks panel in **Read** and assert every authoring region is
/// absent while the panel itself is present.
fn drive_read(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
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
            "input is disabled (--no-input). This check clicks a mode segment and a ribbon \
             control. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("bookmark_read.trace.txt"));
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
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());

    // Read is where the application starts, but say so anyway: a check that
    // relied on the default would stop testing Read the day the default moved.
    click_mode_segment(&session, &driver, ui_rect, "read")?;
    // ★ Only if it is not already showing. The ribbon item is a TOGGLE, and
    // the sibling check records what pressing it over an open panel does:
    // it closes the thing under test. That produced a SKIP on the first run
    // of this check and a FAIL on the second, from the same build — the
    // panel had been left open by whichever check ran before it.
    if declared(&session.trace()?, ui_rect, PANEL_TAB).is_none() {
        open_bookmarks(&session, &driver, ui_rect)?;
    }
    // ★★ …and then SELECT it. A dock tab is declared whether or not it is the
    // selected tab in its stack, and only the selected one draws its body. In
    // Read's default layout Bookmarks shares a stack with Pages and is not the
    // one in front, so the ribbon toggle alone leaves the panel present and
    // never executed — which made the first version of this check pass with the
    // defect planted back in.
    if let Some(tab) = declared(&session.trace()?, ui_rect, PANEL_TAB) {
        driver.click_at(session.frame()?.declared_center(tab))?;
        session.settle(20);
    }
    let trace = session.trace()?;

    // ★★★ THE PANEL MUST BE THERE. Without this the check passes on a build
    // where the Bookmarks panel is missing altogether, or where the ribbon
    // control never opened it — an absence proving nothing, which is the
    // failure this project has recorded more than any other. Read must still be
    // able to NAVIGATE by bookmark; only authoring goes.
    // ★★ THE PANEL'S OWN DOCK TAB, not a `bookmarks.*` region — corrected the
    // same hour it was written.
    //
    // The first version asked for any region beginning `bookmarks`, and SKIPPED
    // on every run: on a document with no outline the panel's only published
    // regions ARE the authoring ones, so once they are correctly gated away in
    // Read the panel publishes nothing — and "correctly empty" and "never
    // opened" become the same evidence.
    //
    // The dock declares the tab whenever the panel is showing, whatever its
    // body draws. That separates the two, which is the only property this
    // check needs from it.
    // ★★★ THE PANEL'S BODY MUST HAVE RUN, and the dock tab does NOT prove it.
    //
    // Second correction, and the falsification is what forced it. With the old
    // behaviour planted back — `authoring = true` — this check still PASSED,
    // because the tab was declared while the panel was not the SELECTED tab in
    // its stack, so its body never executed and drew no authoring row to find.
    // A check asserting an absence over a surface that never ran is the purest
    // form of "an absence proving nothing".
    //
    // `bookmarks-panel` is the census line the body emits when it reads the
    // outline. It exists only if the body ran. That is the evidence, and the
    // tab's rect is not.
    let names = declared_names(&trace, ui_rect, "bookmarks");
    if census(&session, CENSUS)?.is_none() {
        return Err(Error::new(format!(
            "the Bookmarks panel is not showing in Read — no `{PANEL_TAB}` region — so this \
             run cannot tell 'authoring is correctly absent' from 'the panel never opened'. \
             SKIPPED, not passed. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "dock.tab"))
        )));
    }
    report.note(format!(
        "the panel is open in Read and declares {} region(s)",
        names.len()
    ));

    // Every authoring surface, by name. The list is the panel's, not a guess:
    // each of these is a `crate::diag::ui_rect` the panel publishes when it
    // draws the control.
    const AUTHORING: &[&str] = &[
        TITLE_BOX,
        ADD_BUTTON,
        "bookmarks.rename",
        "bookmarks.delete",
    ];
    // `String`, not `&str`, because `driving::list` takes `&[String]` — it is
    // the same formatter every other check's failure text uses, and matching it
    // is what keeps two reports of one kind of fact readable side by side.
    let offered: Vec<String> = AUTHORING
        .iter()
        .filter(|r| declared(&trace, ui_rect, r).is_some())
        .map(|r| (*r).to_owned())
        .collect();

    if !offered.is_empty() {
        return Ok(Some(format!(
            "★★★ READ MODE OFFERS BOOKMARK AUTHORING: {}.\n  \
             Read's whole promise is that it cannot change the document, and \
             `MODES_AND_PANELS.md`'s panel table already draws the distinction by name — Read \
             gets \"Comments (read)\" against Review's \"Comments (authoring)\". The panel is \
             right to be reachable in Read; navigating by bookmark IS reading. Its authoring \
             half is gated on `Capabilities::authors_anything`, which Read does not have.\n  \
             Regions declared: {}.",
            list(&offered),
            list(&names)
        )));
    }
    report.note("no authoring region is drawn in Read");
    Ok(None)
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
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon control \
             and two panel controls, and types five letters. Reported as SKIPPED rather than \
             passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("bookmark_add.trace.txt"));
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

    click_mode_segment(&session, &driver, ui_rect, MODE)?;

    // --- A: the panel, and its authoring row -------------------------------
    //
    // Toggled only if it is not already showing, for the reason
    // `properties_metadata` gives: the ribbon item is a TOGGLE, and pressing it
    // over an open panel closes the thing under test.
    if declared(&session.trace()?, ui_rect, TITLE_BOX).is_none() {
        open_bookmarks(&session, &driver, ui_rect)?;
    }
    let trace = session.trace()?;
    if declared(&trace, ui_rect, TITLE_BOX).is_none() {
        return Ok(Some(format!(
            "no `{TITLE_BOX}` region after opening the Bookmarks panel. On a document with no \
             outline — which this fixture is — that is the early return coming back: the panel \
             says \"no bookmarks\" and offers no way to make one, which is the exact state the \
             document most needing a first bookmark is in. Regions beginning `bookmarks`: {}.",
            list(&declared_names(&trace, ui_rect, "bookmarks"))
        )));
    }
    report.note("the authoring row drew on a document with no outline");

    // --- B: what the panel says is there now -------------------------------
    let Some(before) = census(&session, CENSUS)? else {
        return Ok(Some(format!(
            "the panel drew and traced no `{CENSUS}` line, so it did not read the outline."
        )));
    };
    report.note(format!("the outline holds {before} item(s) to begin with"));

    // --- C: the button is greyed while the title is empty ------------------
    //
    // Pressed deliberately. A control that is drawn and does nothing is this
    // project's founding defect, and the ONLY way to tell "correctly greyed"
    // from "live but broken" is to press it and observe that nothing happened.
    let add = declared(&trace, ui_rect, ADD_BUTTON).ok_or_else(|| {
        Error::new(format!(
            "no `{ADD_BUTTON}` region — the Add control is published in both its greyed and its \
             live state, so its absence means neither drew."
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(add))?;
    session.settle(12);
    let pressed_while_empty = session.trace()?.events(PRESSED).count();
    if pressed_while_empty > 0 {
        return Ok(Some(format!(
            "clicking Add with the title box EMPTY raised the action ({pressed_while_empty} \
             `{PRESSED}` line(s)). The control must be greyed until there is a title — the \
             engine would refuse a blank one, and the refusal would reach the trace and not the \
             operator, which is a button that silently does nothing."
        )));
    }
    report.note("Add is inert while the title is empty, as R9 requires of a greyed control");

    // --- D: type a title and press it --------------------------------------
    let title_box = declared(&session.trace()?, ui_rect, TITLE_BOX).ok_or_else(|| {
        Error::new(format!(
            "the `{TITLE_BOX}` region went away between phases."
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(title_box))?;
    session.settle(10);
    for key in TITLE_KEYS {
        driver.press(key)?;
        session.settle(3);
    }
    session.settle(8);

    // Re-read the button's rect: the row is a `horizontal`, and a title box
    // that grew would have moved the button. `ui_rect` is a change log, so a
    // control that has not moved simply is not re-emitted — which is why this
    // reads the latest declaration rather than requiring a fresh one.
    let add = declared(&session.trace()?, ui_rect, ADD_BUTTON)
        .ok_or_else(|| Error::new(format!("the `{ADD_BUTTON}` region went away after typing.")))?;
    driver.click_at(session.frame()?.declared_center(add))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(press) = trace.events(PRESSED).last() else {
        return Ok(Some(format!(
            "with `TITLE` typed, clicking Add traced no `{PRESSED}` line — so the button is \
             still greyed with a non-empty title, or its click arm did not run. The gate is \
             `title.trim().is_empty()` in `panels::bookmarks::add`."
        )));
    };
    let chars = press.get("chars").unwrap_or_default();
    if chars != "5" {
        return Ok(Some(format!(
            "Add was pressed with `chars={chars}` where five keys were typed. The keystrokes \
             are not all reaching the title box — which is `add_text_takes_real_keystrokes`' \
             defect arriving in a second surface."
        )));
    }

    if let Some(refusal) = trace.events(&format!("{APPLIED}-refused")).last() {
        return Ok(Some(format!(
            "the panel raised its action and the engine REFUSED it: {}. The shell half works and \
             the document was not changed.",
            refusal.get("detail").unwrap_or_default()
        )));
    }
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "the panel traced `{PRESSED}` and the funnel never traced `{APPLIED}`, so the action \
             was queued and never applied — the arm in `actions::apply` did not run, or the \
             queue was not drained this frame."
        )));
    };
    report.note(format!(
        "the bookmark was written at epoch {}",
        applied.get("epoch").unwrap_or_default()
    ));

    // --- E: and the panel gets it back -------------------------------------
    //
    // The half that makes this a round trip rather than a press. `items` is
    // `read_outline`'s walked-tree count, NOT the root `/Count` — see this
    // module's header for why that distinction is load-bearing even though the
    // two agree on this fixture.
    let Some(after) = census(&session, CENSUS)? else {
        return Ok(Some(format!(
            "the panel stopped tracing `{CENSUS}` after the edit."
        )));
    };
    if after != before + 1 {
        return Ok(Some(format!(
            "the outline held {before} item(s) before the add and {after} after it, where \
             {} was expected. The engine reported success, so the item is in the document and \
             the panel is not reading it back — which is the shape an operator meets as \
             \"I added a bookmark and it is not there\".",
            before + 1
        )));
    }
    report.note(format!("the panel reads {after} item(s) back"));

    let shot = ctx.out("bookmark_add.png");
    match crate::capture::window_to_png(&session, &shot) {
        Ok(_) => {
            report.artifact(shot);
        }
        Err(e) => {
            report.note(format!(
                "the window could not be captured ({e}); the trace assertions above still hold"
            ));
        }
    }
    Ok(None)
}

/// The latest `items=` the panel reported.
///
/// `last()` rather than `first()`: the panel traces once per frame it draws, so
/// the newest line is the only one describing the document as it is now.
fn census(session: &Session, event: &str) -> Result<Option<usize>> {
    Ok(session
        .trace()?
        .events(event)
        .last()
        .and_then(|line| line.get("items").and_then(|v| v.parse().ok())))
}

/// Show the Bookmarks panel from the View tab.
fn open_bookmarks(session: &Session, driver: &Driver, ui_rect: &str) -> Result<()> {
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.view").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.view` region. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let item = declared_or_in_overflow(session, driver, ui_rect, PANEL_ITEM)?.ok_or_else(|| {
        Error::new(format!(
            "no `{PANEL_ITEM}` region on the View tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace().unwrap_or_default(),
                ui_rect,
                "ribbon.item.view."
            ))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);
    Ok(())
}
