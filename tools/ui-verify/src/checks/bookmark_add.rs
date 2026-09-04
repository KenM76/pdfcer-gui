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

/// The mode the Bookmarks panel is reached in.
const MODE: &str = "read";
/// The command that shows the panel.
const PANEL_ITEM: &str = "ribbon.item.view.panel_bookmarks";
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
