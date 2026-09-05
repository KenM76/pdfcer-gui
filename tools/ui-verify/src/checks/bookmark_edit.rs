//! `a_bookmark_can_be_renamed_and_removed` — **the panel that could only ever
//! create.**
//!
//! # What this proves, and why the two verbs are one check
//!
//! `pdfcer-core` `Pass 156.0` shipped `set_outline_title` and
//! `delete_outline_item` together, with the engine's own note saying *"bookmarks
//! could be created and not changed — renaming is the commonest bookmark edit
//! there is"*. Both reach the operator through the **same block**: click a row,
//! and a *Selected bookmark* section appears above the list carrying a name
//! field and a Remove.
//!
//! ⇒ So one check drives one gesture (the row click) and then both verbs. Two
//! checks would each pay for a launch, a mode click, a panel open and a
//! bookmark authored — about four seconds apiece on this machine — to assert
//! two halves of one surface that cannot appear separately.
//!
//! # ★★★ The fixture has NO outline, and that is the point rather than a
//! limitation
//!
//! `SW41177.pdf` and every other CAD export in this project's fixture set are
//! exported without bookmarks. So phase A **authors one**, through the same
//! authoring row `bookmark_can_be_written` drives, and phases B and C then act
//! on it.
//!
//! That is a real dependency and it is stated rather than hidden: if
//! `bookmark_can_be_written` fails, this check SKIPS rather than reporting a
//! rename defect, because there is nothing to rename and *"could not set up"* is
//! a different fact from *"the feature is broken"*. A harness that cannot tell
//! those apart reports the wrong module, which is what seven of ten failures in
//! the 2026-08-28 sweep turned out to be.
//!
//! # ★★ The rename oracle is the PANEL's census, not the trace alone
//!
//! `rename-bookmark …` says the engine accepted the call.
//! `bookmarks-panel items=N` unchanged says the outline still holds one item
//! rather than two — which is the assertion that a rename did not silently
//! become an *add*, and that is not a hypothetical failure: both verbs take a
//! title, both go through `vector_edit`, and a dispatch arm routed to the wrong
//! one produces a document that looks right until somebody counts.
//!
//! The title itself is deliberately **not** asserted from the trace: the panel
//! traces the LENGTH of a bookmark name and not its text, because a bookmark's
//! name is the operator's own words about their drawing and the trace is a file
//! a harness keeps. `chars=` moving from 5 to 6 is the evidence available, and
//! it is enough to distinguish the two builds that matter.
//!
//! # ★★★ The delete oracle is the COUNT, and the reason is the engine's
//!
//! `delete_outline_item` removes the subtree. This fixture's outline is one
//! top-level item, so `descendants=0` and the count goes 1 → 0. A check that
//! asserted only *"shorter than before"* would pass on every defect the engine
//! itself injected on that Pass — its own words:
//!
//! > The delete test asserted the bookmark list was *"shorter than before"*, and
//! > every defect we injected leaves a shorter list. One leaves it **empty**,
//! > which is also shorter.
//!
//! So this asserts the count **exactly**, and the panel's promised subtree size
//! against the engine's reported one.
//!
//! # Phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | open the panel, type a title, press Add | `add-bookmark`, and `items=1` |
//! | B | click the row, retype the name, press **Enter** | `bookmark-rename chars=6`, `rename-bookmark`, and `items` **unchanged** |
//! | C | press Remove | `bookmark-delete descendants=0`, `delete-bookmark`, and `items=0` |
//!
//! ★★★ **Two of those three gestures were missing until 2026-08-29, and without
//! them this check could not pass on any build.** It went from the Add press
//! straight to reading `bookmarks.rename` — on a comment saying *"fall through
//! to the row click below"*, and there was no row click below — and then typed
//! six letters and read the trace without committing them. `BookmarksUi`'s
//! selection is set only by a row click, and `edit::rename_row` raises its
//! action only on the button or on Enter, so both halves reported a working
//! panel as broken.
//!
//! ★ Enter, not the Rename button: that button publishes no `ui_rect` region,
//! so there is no coordinate for a harness to aim at. `edit::rename_row` commits
//! on either, and its own header commits to the keystroke.
//!
//! ★ The row publishes no region either — it is a frameless `Button` in a
//! `ScrollArea` — so the aim comes from the `bookmark-row … rect=` line the
//! panel traces per row. See [`ROW`].

use crate::checks::driving::{
    self, SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode the Bookmarks panel is **authored** in.
///
/// ★★★ **Was `read`, and that made this check permanently unrunnable —
/// corrected 2026-09-05 on the first sweep that ever executed it.**
///
/// Read's dock does carry Bookmarks (see `app::modes::defaults` — *Read: Pages,
/// Bookmarks*), so the panel is on screen and `dock.body.view.panel_bookmarks`
/// is published. What Read does **not** carry is the *authoring row*: the
/// application deliberately withholds `bookmarks.new_title` and `bookmarks.add`
/// there, and `bookmark_add`'s second half —
/// `read_mode_offers_no_bookmark_authoring` — asserts that absence and passes.
///
/// So this check's phase A, which types a title into that box to give itself a
/// bookmark to rename, could never begin. Its SKIP reason was accurate and
/// unhelpful: *"no `bookmarks.new_title` region … Regions beginning
/// `bookmarks`: none."* Nothing was broken; the check was asking Read for a
/// control Read is specified not to offer.
///
/// ⇒ **Review**, which is the mode `bookmark_add` authors in and the one whose
/// whole posture is marking up somebody else's drawing. Its default dock also
/// carries Bookmarks, so no extra toggle is needed — which is why [`INVOKE`]
/// stopped toggling the panel at the same time.
///
/// ★★ The general shape, and it is this project's commonest: **a check that
/// SKIPs is not red, so a check aimed at a surface the application has since
/// been specified not to have can sit there for ever looking like an ordinary
/// wrong-fixture skip.** The tell here was two checks disagreeing — one
/// asserting the row is absent in Read and passing, three others requiring it
/// in Read and skipping.
const MODE: &str = "review";
/// Supplied at launch. **Nothing**, deliberately.
///
/// ★★ It used to be `view.panel_bookmarks`, which is a **toggle**, over a mode
/// whose default layout already mounts the panel — so it was as likely to close
/// the panel as to open it, and which it did depended on `userdata/layout.ron`
/// written by whichever run went before. [`MODE`]'s default dock mounts
/// Bookmarks, so the correct number of toggles is zero.
const INVOKE: &str = "";
/// The title box on the authoring row.
const TITLE_BOX: &str = "bookmarks.new_title";
/// The Add button.
const ADD_BUTTON: &str = "bookmarks.add";
/// The rename field on the Selected bookmark block.
const RENAME_BOX: &str = "bookmarks.rename";
/// The Remove button on the Selected bookmark block.
const DELETE_BUTTON: &str = "bookmarks.delete";
/// The panel's per-frame census.
const CENSUS: &str = "bookmarks-panel";
/// The dock body the rows are drawn inside — the bound a clickable row must sit
/// within. See the row-picking comment in [`drive`].
const PANEL_BODY: &str = "dock.body.view.panel_bookmarks";
/// ★★★ **One line per outline row drawn, carrying that row's own rectangle.**
///
/// This is how the row is aimed at, and it is not a `ui_rect` region because a
/// row is not one: `panels::bookmarks::rows` draws a frameless `Button` per
/// item inside a `ScrollArea` and traces
/// `bookmark-row level=… title=… page=… enabled=… rect=…` from the same
/// `Response`. `rect` is `egui::Rect`'s `Debug`, so `TraceLine::get_rect` reads
/// it and `WindowFrame::declared_center` converts it exactly as it converts a
/// declared region — same space, same origin, same frame.
///
/// ★ `.last()` is the most recently drawn row of the most recently drawn
/// frame. This check authors exactly one bookmark before it aims, so there is
/// one row and the choice does not arise; a check that authored several would
/// have to filter on `title=` instead.
const ROW: &str = "bookmark-row";
/// The panel's line for a rename press.
const RENAME_PRESSED: &str = "bookmark-rename";
/// The panel's line for a delete press, carrying the subtree size it promised.
const DELETE_PRESSED: &str = "bookmark-delete";
/// The funnel's line for the rename verb.
const RENAMED: &str = "rename-bookmark";
/// The funnel's line for the delete verb.
const DELETED: &str = "delete-bookmark";
/// `TITLE`, the name the bookmark is authored with — five keystrokes, matching
/// `bookmark_can_be_written` so a reader comparing the two traces sees the same
/// word.
const TITLE_KEYS: [u16; 5] = [vk::T, vk::I, vk::T, vk::L, vk::E];
/// `DETAIL`, the name it is renamed to — a DIFFERENT LENGTH, deliberately.
///
/// ★ Six letters against five is the whole oracle: the panel traces the length
/// of a bookmark name and not its text, so a rename to a same-length word would
/// be indistinguishable from no rename at all in the only evidence available.
const RENAME_KEYS: [u16; 6] = [vk::D, vk::E, vk::T, vk::A, vk::I, vk::L];

/// See the module documentation.
pub struct ABookmarkCanBeRenamedAndRemoved;

impl Check for ABookmarkCanBeRenamedAndRemoved {
    fn name(&self) -> &'static str {
        "a_bookmark_can_be_renamed_and_removed"
    }

    fn defect(&self) -> &'static str {
        "the Bookmarks panel can create a bookmark and cannot change one — no rename, no delete — \
         so a title typed wrongly is permanent and a bookmark added to the wrong parent can only \
         be lived with"
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

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment and four panel \
             controls and types eleven letters. Reported as SKIPPED rather than passed: a check \
             that did not run has learned nothing.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("bookmark-edit.trace.txt"));
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
    // ★★ The dock draws only the ACTIVE tab's body, and in this mode's default
    // layout Bookmarks shares a stack with Pages. See
    // [`crate::checks::driving::raise_dock_tab`].
    driving::raise_dock_tab(&session, &driver, ui_rect, "view.panel_bookmarks")?;

    // --- A: author the bookmark this check then edits -----------------------
    let trace = session.trace()?;
    let Some(title_box) = declared(&trace, ui_rect, TITLE_BOX) else {
        return Err(Error::new(format!(
            "no `{TITLE_BOX}` region, so the authoring row is not on screen and there is nothing \
             to set up with. `bookmark_can_be_written` owns that surface; SKIPPED rather than \
             failed, because this check is about renaming. Regions beginning `bookmarks`: {}.",
            list(&declared_names(&trace, ui_rect, "bookmarks"))
        )));
    };
    driver.click_at(session.frame()?.declared_center(title_box))?;
    session.settle(8);
    for key in TITLE_KEYS {
        driver.press(key)?;
    }
    session.settle(8);
    let add = declared(&session.trace()?, ui_rect, ADD_BUTTON)
        .ok_or_else(|| Error::new(format!("no `{ADD_BUTTON}` region to press.")))?;
    driver.click_at(session.frame()?.declared_center(add))?;
    session.settle(20);

    let Some(after_add) = census(&session)? else {
        return Err(Error::new(format!(
            "the panel traced no `{CENSUS}` line, so it is not reading the outline and nothing \
             below can be measured."
        )));
    };
    if after_add == 0 {
        return Err(Error::new(format!(
            "the setup did not author a bookmark — `{CENSUS} items=0` after pressing Add. That is \
             `bookmark_can_be_written`'s subject, not this one. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("set-up: the outline holds {after_add} item(s)"));

    // --- B: click the row, then rename it -----------------------------------
    //
    // ★ The rename block appears only once a row is SELECTED, and the click
    // that selects it is the same click that navigates — the panel's own
    // decision, because a bookmark click means "take me there" first and always.
    // So there is no separate select gesture to drive: the row is the control.
    //
    // ★★★ AND THE ROW HAS TO BE CLICKED. Until 2026-08-29 this check went
    // straight from the Add press to reading `bookmarks.rename`, on a comment
    // saying "fall through to the row click below" — and there was no row click
    // below. `BookmarksUi::selected` is set in exactly one place,
    // `panels::bookmarks::show`'s `if let Some(id) = picked`, and `picked` comes
    // only from a row's `Response::clicked`. Authoring a bookmark does NOT
    // select it: the add row leaves the selection alone deliberately, because
    // what it means there is *the parent for the next add*, and `Move to top
    // level` clears it. So `bookmarks.rename` was never declared, the branch
    // below always taken, and this check could not pass on any build — it was
    // reporting THE SELECTED-BOOKMARK BLOCK NEVER APPEARED about a panel
    // behaving exactly as its own module header says it does.
    let trace = session.trace()?;
    // ★★★ A row that is ON SCREEN, not merely the last one traced.
    //
    // `bookmark-row` is a diagnostic line, not a `ui_rect`: the panel writes one
    // per row per frame **whether or not the row is inside the visible scroll
    // area**, because the listing has to be provable from a trace without
    // anybody clicking. So `.last()` answers with the BOTTOM row of the last
    // frame — and on the operator's own drawing that is row 124 of 124, at
    // y=3767 in a panel whose body ends at y=766.
    //
    // Measured, not reasoned: the failing run clicked **three thousand points
    // below the panel**, off the window entirely, and then reported that the
    // Selected-bookmark block never appeared. The panel was right; the aim was
    // three metres low.
    //
    // ⇒ The general form, and this suite has now met it twice: **a trace line
    // written for every item is not a list of the items you can click.** The
    // `ui-rect` census answers *what is on screen*; a per-item diagnostic
    // answers *what was computed*. Aiming with the second is aiming at
    // something that may not be there.
    let body = declared(&trace, ui_rect, PANEL_BODY);
    let visible_row = |line: &crate::trace::TraceLine| {
        let Some(rect) = line.get_rect("rect") else {
            return false;
        };
        body.is_none_or(|body| rect.min.y >= body.min.y && rect.max.y <= body.max.y)
    };
    // The LAST visible one: the trace holds every frame, so later lines describe
    // the arrangement the click is about to land in.
    let row = trace.events(ROW).filter(|line| visible_row(line)).last();
    let Some(row) = row else {
        return Err(Error::new(format!(
            "the panel counted {after_add} item(s) and traced no `{ROW}` line, so the outline \
             was read and no row was drawn. There is nothing to click and therefore nothing to \
             select, which is `bookmark_can_be_written`'s surface rather than this one's. \
             SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    };
    // ★ A row whose destination pdfcer could not resolve is drawn DISABLED, and
    // a disabled `Button` never reports a click — so it can never be selected
    // and the rename block can never appear for it. That is correct behaviour
    // and it would read here as the defect, so it is a SKIP with the reason.
    if row.get("enabled") != Some("true") {
        return Err(Error::new(format!(
            "the only bookmark row is DISABLED: `{}`. A row is enabled only when its \
             destination resolves to a page, and a disabled `egui::Button` reports no click — \
             so no click can select it and the Selected-bookmark block cannot appear. The \
             bookmark this check authors carries the current page, so reaching this means the \
             add row wrote no destination. SKIPPED: that is `bookmark_can_be_written`'s \
             subject.",
            row.raw
        )));
    }
    let row_rect = row.get_rect("rect").ok_or_else(|| {
        Error::new(format!(
            "the `{ROW}` line carries no parsable `rect=`: `{}`. Without it there is no \
             coordinate for the row and the selection cannot be made.",
            row.raw
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(row_rect))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(rename_box) = declared(&trace, ui_rect, RENAME_BOX) else {
        let names = declared_names(&trace, ui_rect, "bookmarks");
        report.note(format!(
            "no `{RENAME_BOX}` after the row was clicked. Regions: {}",
            list(&names)
        ));
        return Ok(Some(format!(
            "THE SELECTED-BOOKMARK BLOCK NEVER APPEARED. A bookmark exists (`{CENSUS} \
             items={after_add}`) and the panel published no `{RENAME_BOX}` region. Look first at \
             whether clicking a row records the selection: the block is keyed on \
             `BookmarksUi`'s selected id, and the click that sets it is the same click that \
             navigates. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // Select the whole field and type over it: the draft is seeded with the
    // existing name, so typing alone would append.
    driver.click_at(session.frame()?.declared_center(rename_box))?;
    session.settle(8);
    driver.press_chord(&[vk::CONTROL], vk::A)?;
    for key in RENAME_KEYS {
        driver.press(key)?;
    }
    session.settle(10);
    // ★★★ AND THEN COMMIT IT. Typing into the field only moves the DRAFT:
    // `panels::bookmarks::edit::rename_row` raises the action on
    // `button.clicked() || (response.lost_focus() && Enter)`, so a check that
    // typed and then read the trace would find nothing however well the feature
    // worked. This check did exactly that until 2026-08-29.
    //
    // ★ Enter rather than the button, and not by preference: the Rename button
    // publishes **no `ui_rect` region** — only the field does (`REGION_RENAME`)
    // and only the Remove does (`REGION_DELETE`) — so there is no coordinate to
    // aim at, and Enter is the one route a harness has. That module's own
    // header commits to the keystroke in as many words ("Enter commits …
    // checking only the button would make that keystroke do nothing"), which is
    // what makes it safe to pin a check to.
    driver.press(vk::ENTER)?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(pressed) = trace.events(RENAME_PRESSED).last() else {
        // The Rename button appears only when the typed name DIFFERS from the
        // current one, so its absence is two different faults and the check
        // says which it can rule out.
        return Ok(Some(format!(
            "typing a new name raised no `{RENAME_PRESSED}` line. The Rename control is drawn only \
             when the typed name differs from the current one and is non-empty, so suspect, in \
             order: the keystrokes not reaching the field (a chord with a dock panel open is not a \
             reliable primitive — see `scale_switch`), the draft not being written back, or Enter \
             not committing. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the panel raised the rename: `{}`", pressed.raw));
    // ★★ The oracle this module's header names and the code did not have. The
    // panel traces the LENGTH of the name and never its text, so the one thing
    // a trace can say about *which* name was committed is that it is not the
    // length of the one already there. A rename that raised the existing title
    // back at the document is a no-op wearing a success line, and every
    // assertion below — the funnel line, the unchanged count — holds for it.
    //
    // ★ Asserted as "different from five", not as "exactly six": Ctrl+A is a
    // chord, this panel is a dock, and `scale_switch` measured a chord over a
    // dock arriving zero times in six. A failed select-all leaves
    // `TITLEDETAIL` — eleven characters, a name genuinely committed from
    // genuinely delivered keystrokes — and failing on that would be reporting
    // the harness as a defect in the panel.
    match pressed.get_usize("chars") {
        Some(n) if n != TITLE_KEYS.len() => {
            report.note(format!(
                "★ the committed name is {n} character(s) long, against the {} it was authored \
                 with — so a different name reached the verb",
                TITLE_KEYS.len()
            ));
        }
        Some(n) => {
            return Ok(Some(format!(
                "THE RENAME COMMITTED A NAME OF THE ORIGINAL LENGTH: `{}` reports chars={n}, and \
                 the bookmark was authored with {} character(s). The panel traces the length and \
                 never the text — a bookmark's name is the operator's own words about their \
                 drawing — so this is as close as the trace gets to 'the same name went back in', \
                 and that is a no-op wearing a success line: the funnel writes its line, the count \
                 does not move, and every assertion below this one passes.",
                pressed.raw,
                TITLE_KEYS.len()
            )));
        }
        None => {
            return Ok(Some(format!(
                "the rename was raised and its line carries no readable `chars=`: `{}`. That \
                 field is the only evidence available for WHICH name was committed, so without \
                 it this check cannot tell a rename from a re-commit of the same title.",
                pressed.raw
            )));
        }
    }
    if trace.events(RENAMED).count() == 0 {
        return Ok(Some(format!(
            "the panel raised `{}` and no `{RENAMED}` line followed, so the action reached no \
             apply arm or `set_outline_title` refused. A refused `vector_edit` traces \
             `{RENAMED}-refused`; look for that first.",
            pressed.raw
        )));
    }
    let after_rename = census(&session)?.unwrap_or(0);
    if after_rename != after_add {
        return Ok(Some(format!(
            "THE RENAME ADDED A BOOKMARK INSTEAD OF CHANGING ONE: the outline went from \
             {after_add} to {after_rename}. Both verbs take a title and both go through \
             `vector_edit`, so a dispatch arm routed to the wrong one produces exactly this — a \
             document that looks right until somebody counts."
        )));
    }
    report.note(format!(
        "★ the outline still holds {after_rename} item(s) — a rename, not an add"
    ));

    // --- C: remove it -------------------------------------------------------
    let trace = session.trace()?;
    let delete = declared(&trace, ui_rect, DELETE_BUTTON).ok_or_else(|| {
        Error::new(format!(
            "no `{DELETE_BUTTON}` region on the Selected bookmark block. Regions beginning \
             `bookmarks`: {}.",
            list(&declared_names(&trace, ui_rect, "bookmarks"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(delete))?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(pressed) = trace.events(DELETE_PRESSED).last() else {
        return Ok(Some(format!(
            "clicking Remove raised no `{DELETE_PRESSED}` line. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let promised = pressed.get_usize("descendants");
    if trace.events(DELETED).count() == 0 {
        return Ok(Some(format!(
            "the panel raised `{}` and no `{DELETED}` line followed, so the verb was not reached \
             or the engine refused it.",
            pressed.raw
        )));
    }
    let after_delete = census(&session)?.unwrap_or(usize::MAX);
    // ★★★ EXACTLY, not "fewer". The engine's own account of this Pass records
    // three injected defects surviving a "shorter than before" assertion,
    // including one that emptied the list — which is also shorter.
    if after_delete != after_add - 1 {
        return Ok(Some(format!(
            "the outline went from {after_add} to {after_delete}, and removing one top-level \
             bookmark with no children must leave exactly {}. The panel promised \
             descendants={promised:?}. ★ A count that fell FURTHER than promised is the delete \
             taking a subtree it should not have; a count that did not move is the verb reaching \
             the document and changing nothing.",
            after_add - 1
        )));
    }
    report.note(format!(
        "★★ the outline went {after_add} → {after_delete}, with the panel promising \
         descendants={promised:?}"
    ));
    Ok(None)
}
