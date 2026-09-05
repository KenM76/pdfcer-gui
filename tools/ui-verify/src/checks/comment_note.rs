//! `a_note_can_be_written_onto_a_shape_that_exists` — **the Comments panel
//! stopped being a viewer, and this is what says it stayed that way.**
//!
//! # The gap this closes, in the engine's own words
//!
//! `pdfcer-core`'s reply to this shell's blocker (`Pass 154.0`) lists what a
//! read-only comment list costs a reviewer, and none of the four is an edge
//! case:
//!
//! > comment a shape you just drew, comment a highlight you just swept, fix a
//! > typo in your own comment, answer someone else's.
//!
//! All four were impossible here until 2026-08-28, for a reason that is
//! structural rather than lazy: `MarkupOptions` is an **author-time** type, and
//! a cloud, a rectangle and an arrow are authored on mouse-release from
//! geometry alone. There is no text-entry moment in that gesture and there must
//! not be one — a dialog on every shape a reviewer draws is the interaction
//! nobody ships. So the conventional model needs a verb acting on an annotation
//! that **already exists**, and until `set_markup_note` there was none.
//!
//! # ★★★ Why this cannot be a unit test, in the specific
//!
//! The chain is five links and each has its own passing test:
//!
//! | # | Link | Its own test |
//! |---|---|---|
//! | 1 | the row decides the annotation is note-editable | yes (`note_controls` is a `match` over `CommentRow`) |
//! | 2 | *Add note* opens the draft | yes (`NoteDraft`'s suite) |
//! | 3 | Save raises `AnnotAction::SetNote` | no — that is a widget, and a widget's effect is observable only in a window |
//! | 4 | the apply arm resolves the author and calls the engine | partially |
//! | 5 | the engine writes `/Contents` and the panel reads it back | yes, on both sides separately |
//!
//! Link 3 is the one that has burned this project repeatedly, most recently
//! **on 2026-08-28 itself**: the O51 scale switches were written into an arm
//! that never runs, compiled, read correctly, and drew nothing, with every unit
//! test green. *"Nothing tested that the control is on screen."* This check is
//! that test for this control.
//!
//! # What it does
//!
//! `PDFCER_DIAG_INVOKE` supplies the two commands at launch rather than clicking
//! for them — Review mode, the Comments panel, and the rectangle tool — because
//! the subject here is the note, not the ribbon, and three extra clicks are
//! three extra ways for the check to fail at something it is not testing.
//! `markup_rectangle` is the check that proves those controls are reachable by
//! mouse.
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | drag a rectangle on the page | `add-markup` — there is now one annotation |
//! | B | read the panel's census | `comments-panel listed=1 with_note=0` |
//! | C | click *Add note* | `comments.note_box` and `comments.note_save` appear |
//! | D | type four letters into the box | — |
//! | E | click *Save note* | `set-markup-note-applied … keys=…Contents…` |
//! | F | read the census again | `with_note=1` |
//! | G | select the shape, Ctrl+C, Ctrl+V | `paste-markup … note=true` |
//!
//! # ★★★ Phase G exists because THIS CHECK CREATED THE DEFECT IT GUARDS
//!
//! The object clipboard copies a markup by reading it into a `MarkupSpec` and
//! authoring a new one. That is lossless only for what a spec can express — and
//! the note this check writes in phase E **is not expressible in a spec**. So
//! on the day the note editor shipped, copying a commented cloud and pasting it
//! produced an anonymous one, and **nothing on the page would show it**: the
//! words live in a pop-up this shell does not draw.
//!
//! ⇒ The general form, worth carrying: **a copy implemented as a re-author
//! loses ground every time the authoring side gains a key**, silently, in a
//! direction no screenshot can see. This phase is the tripwire on that, and it
//! is here rather than in `object_clipboard` because this is the check that can
//! produce an annotation with a note to copy in the first place.
//!
//! # ★★ Phase F is the assertion that matters, and B is what makes it mean
//! anything
//!
//! `set-markup-note-applied` says the engine accepted the call. `with_note=1`
//! says **the panel read the words back out of the document**, which is the
//! only evidence that a reviewer would see anything. Both are needed and
//! neither is sufficient: a build that wrote `/Contents` into a session the
//! panel does not read from would pass the first and fail the second, and that
//! is not a hypothetical — the Comments panel reads `doc.session.view()`
//! precisely because reading the file on disk showed nothing until a save.
//!
//! Phase B pins `with_note=0` first, so F cannot be satisfied by a fixture that
//! arrived with a commented annotation already on it.
//!
//! # ★ The word typed is TAIL
//!
//! Four letters, all of them already in the closed `vk` list (`T`, `A`, `I`,
//! `L`), and a word a drafter would recognise in a failure message. The list is
//! deliberately closed and grown one key at a time with a reason; spelling a
//! word out of what is already there is cheaper than adding two constants to
//! type something prettier.

use crate::checks::comments_census;
use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The commands supplied at launch: Review mode, the Comments panel, the
/// rectangle tool.
///
/// ★ The panel opens **before** the shape is drawn, deliberately. A dock
/// appearing between the frame a check takes its coordinate mapping from and
/// the frame it clicks in changes the canvas width and puts the click somewhere
/// else — a fault this project has already recorded twice, and one that reads
/// as a broken feature rather than as a stale coordinate.
const INVOKE: &str = "mode.review,markup.comments,markup.rectangle";
/// The panel's per-frame census.
///
/// ★★★ **Every comparison read of it below is ANCHORED — 2026-09-05.** A docked
/// pane that is not the front tab of its stack draws nothing and so traces
/// nothing, which means `Trace::last(CENSUS)` goes on answering with the line
/// the panel published before it went quiet. On the sweep of that morning
/// `save_copy_round_trip` and `undo_redo_round_trip` each read such a fossil
/// and reported a working panel as broken, in identical words — which read as
/// corroboration, because each carried its own copy of the same helper.
///
/// This file carried a third copy of the pattern and passed that sweep only
/// because the layout happened to be favourable. Its phase F would otherwise
/// have printed *"THE WORDS WENT INTO THE DOCUMENT AND THE PANEL CANNOT SEE
/// THEM"* about a panel that had merely gone behind another tab. The two reads
/// that compare now go through [`crate::checks::comments_census`], which
/// requires the line to postdate a named cause and brings the panel forward
/// when it has stopped speaking.
const CENSUS: &str = "comments-panel";
/// The line the apply arm writes once the engine has written the note.
///
/// `-applied`, per the convention this project adopted after making the
/// same-name mistake twice: `vector_edit` writes its own bare `set-markup-note`
/// line for the identical edit, and `.last()` on the bare name reads that one
/// and finds no keys.
const APPLIED: &str = "set-markup-note-applied";
/// The apply arm's line for the shape drawn in phase A.
const MARKUP_APPLIED: &str = "add-markup";
/// The region the first row's *Add note* control publishes.
const EDIT_REGION: &str = "comments.note_edit";
/// The region the open editor's text box publishes.
const BOX_REGION: &str = "comments.note_box";
/// The region the open editor's Save publishes.
const SAVE_REGION: &str = "comments.note_save";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";
/// Where the rectangle is drawn, as fractions of the page — well inside the
/// sheet, away from a title block, and away from the edges.
const SHAPE: ((f64, f64), (f64, f64)) = ((0.35, 0.35), (0.55, 0.50));
/// `TAIL`, four keystrokes. See the module header.
const WORD: [u16; 4] = [vk::T, vk::A, vk::I, vk::L];
/// The apply arm's line for a paste, carrying whether the note survived.
const PASTED: &str = "paste-markup-requested";
/// `chord-not-offered id=… mode=…` — the shell's line for a chord that arrived
/// and was refused by the mode gate. See the phase-G branch in [`drive`].
const CHORD_NOT_OFFERED: &str = "chord-not-offered";
/// The command `Ctrl+V` resolves to.
const PASTE_COMMAND: &str = "edit.paste";
/// The line the canvas writes when a click selects an annotation.
const SELECTED: &str = "annot-select";

/// See the module documentation.
pub struct ANoteCanBeWrittenOntoAShape;

impl Check for ANoteCanBeWrittenOntoAShape {
    fn name(&self) -> &'static str {
        "a_note_can_be_written_onto_a_shape_that_exists"
    }

    fn defect(&self) -> &'static str {
        "the Comments panel lists annotations and cannot write one word onto any of them, so a \
         reviewer can draw a cloud round a mistake and has nowhere to say what is wrong with it \
         — a reviewer's main surface reduced to a viewer"
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

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check draws a shape with a drag, clicks two \
             panel controls and types four letters. Reported as SKIPPED rather than passed: a \
             check that did not run has learned nothing.",
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a page to draw a shape on."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!(
                "could not read a page size from {}, and this check places its shape in page \
                 fractions. Pass --page-size.",
                pdf.display()
            ))
        })?,
    };
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("comment-note.trace.txt"));
    spec.pdf = Some(pdf.clone());
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

    let trace = session.trace()?;
    if declared(&trace, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nothing to draw on. Regions beginning `page`: {}.",
            list(&declared_names(&trace, ui_rect, "page"))
        )));
    }
    if trace.last(CENSUS).is_none() {
        return Err(Error::new(format!(
            "the Comments panel drew no `{CENSUS}` line after `{INVOKE}`, so the panel is not \
             open and every control this check aims at is absent for that reason rather than \
             for the one under test. SKIPPED, not failed."
        )));
    }

    // --- A: draw a rectangle ------------------------------------------------
    let corner = |f: (f64, f64)| DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt);
    let from = aim(ctx, &session, page, corner(SHAPE.0))?;
    let to = aim(ctx, &session, page, corner(SHAPE.1))?;
    driver.drag(from, to)?;
    session.settle(30);

    let trace = session.trace()?;
    if trace.events(MARKUP_APPLIED).count() == 0 {
        return Err(Error::new(format!(
            "the drag authored no annotation — no `{MARKUP_APPLIED}` line — so there is nothing \
             on the page to comment on. That is the step BEFORE the one under test; \
             `markup_rectangle_arms_from_the_ribbon` and `dragging_a_markup_moves_it` are the \
             checks that own it. SKIPPED. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- B: the census before, which is what makes F mean anything ----------
    // ★ Anchored on the engine's own line, and recovered if the panel has gone
    // behind another tab — see [`CENSUS`]. A census from before the shape
    // existed says nothing about a panel that can see it.
    let drawn_at = trace.events(MARKUP_APPLIED).last().map_or(0, |l| l.lineno);
    let Some(before) = comments_census::refresh(&session, &driver, ui_rect, drawn_at, report)?
    else {
        return Err(Error::new(format!(
            "the panel has published no `{CENSUS}` since the shape was authored, and neither \
             its dock tab nor its ribbon control could bring it forward — so it is off screen \
             rather than no longer reading the document, and a dock draws only its active tab. \
             SKIPPED, not failed: that is a layout fact. Clear the `userdata/` directory beside \
             the binary and run again."
        )));
    };
    let listed = before.listed;
    let with_note_before = before.with_note;
    if listed == 0 {
        return Ok(Some(format!(
            "the engine authored the shape and the panel lists NOTHING: `{}`. The panel reads \
             `doc.session.view()` — the base revision with every unsaved edit applied — so a \
             zero here means it is reading the file on disk instead, and a reviewer would see \
             their own markup only after saving.",
            before.raw
        )));
    }
    if with_note_before != 0 {
        return Err(Error::new(format!(
            "the document already carries {with_note_before} commented annotation(s) before \
             this check writes one, so `with_note` cannot be used as the oracle. Use a fixture \
             with no commented markup, or the pass would be satisfied by the fixture. Line: \
             `{}`.",
            before.raw
        )));
    }
    report.note(format!(
        "the panel lists {listed} row(s), none carrying a note: `{}`",
        before.raw
    ));

    // --- C: open the editor -------------------------------------------------
    let edit = declared(&trace, ui_rect, EDIT_REGION).ok_or_else(|| {
        Error::new(format!(
            "no `{EDIT_REGION}` region. The panel publishes it for the FIRST row that offers a \
             note editor, so its absence means either no row offered one — a ce dimension and a \
             direct-dictionary annotation both decline, with a caption — or the control is not \
             drawn at all. Regions beginning `comments`: {}.",
            list(&declared_names(&trace, ui_rect, "comments"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(edit))?;
    session.settle(16);

    let trace = session.trace()?;
    let Some(text_box) = declared(&trace, ui_rect, BOX_REGION) else {
        return Ok(Some(format!(
            "clicking *Add note* opened no editor: no `{BOX_REGION}` region on the next frame. \
             The button is drawn — its rect is what was clicked — so the failure is between the \
             press and the draft: `NoteDraft::begin`, or the `draft.editing(id, epoch)` test \
             that decides whether the editor draws. ★ Suspect the EPOCH first. The draft is \
             stamped `(annotation, edit epoch)` and the rectangle drawn in phase A bumped the \
             epoch; if anything re-seeds or re-syncs after the press, the editor closes on the \
             frame it opens. Regions beginning `comments`: {}.",
            list(&declared_names(&trace, ui_rect, "comments"))
        )));
    };

    // --- D: type into it ----------------------------------------------------
    //
    // Clicked first: a `TextEdit` takes keystrokes only when it has focus, and
    // egui gives focus on a click rather than on being drawn.
    driver.click_at(session.frame()?.declared_center(text_box))?;
    session.settle(8);
    for key in WORD {
        driver.press(key)?;
    }
    session.settle(10);

    // --- E: save ------------------------------------------------------------
    let trace = session.trace()?;
    let save = declared(&trace, ui_rect, SAVE_REGION).ok_or_else(|| {
        Error::new(format!(
            "the editor is open and publishes no `{SAVE_REGION}`, so there is no way to commit \
             what was typed. Regions beginning `comments`: {}.",
            list(&declared_names(&trace, ui_rect, "comments"))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(save))?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "SAVE WROTE NOTHING: no `{APPLIED}` line after clicking Save. Look, in order, at \
             (1) whether the click reached the button — a `set-markup-note-refused` line means \
             it did and the engine declined; (2) the panel's own `verb` slot, which is drained \
             after the scroll area closes and pushes `Action::Annot(SetNote)`; (3) the apply \
             arm. ★ A refusal by name is the likely one: `set_markup_note` refuses a WIDGET and \
             a ce dimension, and this check aims at whatever row the panel published first. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let keys = applied.get("keys").unwrap_or("");
    if !keys.contains("Contents") {
        return Ok(Some(format!(
            "the engine accepted the call and did NOT write the words: `{}`. `keys_written` \
             names what actually moved, and `/Contents` is not among them — which is the one \
             outcome that looks like success from every other angle.",
            applied.raw
        )));
    }
    report.note(format!("★ the engine wrote the note: `{}`", applied.raw));

    // --- F: the panel reads it back ----------------------------------------
    let Some(after) = comments_census::refresh(&session, &driver, ui_rect, applied.lineno, report)?
    else {
        return Err(Error::new(format!(
            "the panel has published no `{CENSUS}` since the engine wrote the note, and it could \
             not be brought forward. That is the panel being OFF SCREEN, not the panel failing \
             to read the words back — the distinction this check was unable to make until \
             2026-09-05. SKIPPED, not failed."
        )));
    };
    let with_note_after = after.with_note;
    if with_note_after == 0 {
        return Ok(Some(format!(
            "THE WORDS WENT INTO THE DOCUMENT AND THE PANEL CANNOT SEE THEM. The engine \
             reported `{}` and the panel still says `{}`. The panel reads \
             `doc.session.view()`, so this is the case where the write went somewhere the read \
             does not look — or the epoch did not bump and the row is a cached listing.",
            applied.raw, after.raw
        )));
    }
    report.note(format!(
        "★★ the panel read the note back out of the session: `{}`",
        after.raw
    ));

    // --- G: the note survives a copy and a paste ---------------------------
    //
    // ★ The shape's centre, computed from the same fractions phase A drew it
    // at, so the click lands on the annotation rather than on whatever page
    // content is nearby. A markup tool is still armed from the launch invoke,
    // so the pointer is put down first — a click with the rectangle tool armed
    // is the start of a new shape, not a selection.
    let centre = ((SHAPE.0.0 + SHAPE.1.0) / 2.0, (SHAPE.0.1 + SHAPE.1.1) / 2.0);
    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, centre.0 * page.width_pt, centre.1 * page.height_pt),
    )?;
    driver.press(vk::V)?;
    session.settle(8);
    driver.click_at(at)?;
    session.settle(12);
    // ★★★ THE PRECONDITION IS READ FIRST, and the order is the whole point.
    //
    // Until 2026-08-29 the `selected=1` assertion below stood AHEAD of this
    // one, so a run in which the `V` never arrived — the case this SKIP exists
    // to name, and one `scale_switch` has measured at zero arrivals in six —
    // drew a second rectangle instead of selecting the first, reported
    // `selected=0`, and went red saying *"the canvas selected the shape and the
    // Comments panel did not find it"* about a canvas that had selected
    // nothing. A guard placed after the assertion it guards is not a guard.
    if session.trace()?.events(SELECTED).count() == 0 {
        return Err(Error::new(format!(
            "the click on the shape produced no `{SELECTED}` line, so nothing is selected and a \
             copy would have nothing to act on. The Select tool is armed with `V`, which is a \
             CHORD — and a chord with a dock panel open is not a reliable harness primitive \
             (`scale_switch` measured a bare key arriving zero times in six). SKIPPED rather \
             than failed: this is the step before the one under test."
        )));
    }
    // ★★★ The panel FOUND the annotation the canvas selected.
    //
    // This is the second half of the interaction `pdfcer-core` describes — *draw
    // the shape → it is selected → type the comment in the panel* — and it is
    // the half that is invisible from every other angle: the mark on the row is
    // a word inside a heading string, so a trace cannot see it and a screenshot
    // can only confirm it if the reader already knows which row to look at.
    // `selected=` in the census is the only oracle there is.
    //
    // ★ Anchored on the selection itself, for [`CENSUS`]' reason: the census
    // that answers *"did the panel find what the canvas selected?"* must be one
    // the panel drew AFTER the canvas selected it. The unanchored form fails
    // both ways — a fossil from before the click carries `selected=0` and would
    // report a working panel as blind, and a fossil from an earlier selection
    // carries `selected=1` and would hide a real one.
    let trace = session.trace()?;
    let selected_at = trace.events(SELECTED).last().map_or(0, |l| l.lineno);
    let census_now = trace.last_after(CENSUS, selected_at).map(|l| l.raw.clone());
    if let Some(line) = &census_now
        && !line.contains("selected=1")
    {
        return Ok(Some(format!(
            "the canvas selected the shape and the Comments panel did not find it: `{line}`. The panel reads `doc.selection.annot()` and matches it against the rows it drew, so `selected=0` here means either the click selected something else — the trace's `annot-select` line says which — or the row's `id` and the selection's disagree, which on a listing built from the same session should be impossible."
        )));
    }
    driver.press_chord(&[vk::CONTROL], vk::C)?;
    session.settle(10);
    driver.press_chord(&[vk::CONTROL], vk::V)?;
    session.settle(20);

    let trace = session.trace()?;
    let Some(pasted) = trace.events(PASTED).last() else {
        // ★★★ **READ THE MODE GATE BEFORE BLAMING THE CHORD — 2026-09-05.**
        //
        // The message below blamed "chords with a dock panel raised", which is a
        // real and measured harness weakness and was not what happened. On the
        // first run this check ever had, the trace read:
        //
        // ```text
        // chord-command chord="Ctrl+C" id=edit.copy via=clipboard-event
        // clipboard-copy kind=markup carrier=spec page=0 annots=1
        // chord-command chord="Ctrl+V" id=edit.paste via=clipboard-event
        // chord-not-offered id=edit.paste mode=review
        // ```
        //
        // Both chords arrived. `edit.copy` ran. `edit.paste` is **not offered in
        // Review**, so the paste this phase needs cannot happen in the mode this
        // check drives — and `copying_a_sticky_note_carries_the_whole_comment`
        // met the identical gate on the identical frame of the identical build,
        // by a different clipboard route. Two witnesses, one gate.
        //
        // ⇒ Still a SKIP, because the question *"does the paste carry the
        // note?"* genuinely cannot be answered while the paste is refused. But
        // the reason must name the gate: a reason may only assert what the
        // check actually looked at.
        //
        // ⇒ ⚠ **THE GATE WAS OPENED LATER THE SAME DAY** — `edit.cut`,
        // `edit.paste` and `edit.paste_duplicate` joined `edit.copy` in
        // `app::modes::capability::GATED_BY_THEIR_DISPATCHER`, so this branch
        // should no longer be reachable in Review. It is kept, unchanged, for
        // two reasons: it is one of only two witnesses that would notice the
        // escape list being narrowed again, and this check has NOT been re-run
        // against the fixed build — the session that made the fix worked
        // headlessly. If it still skips here, the fix regressed; if it skips
        // for the other reason below, that is the original, unrelated harness
        // weakness.
        let refused = trace
            .events(CHORD_NOT_OFFERED)
            .any(|l| l.get("id") == Some(PASTE_COMMAND));
        if refused {
            return Err(Error::new(format!(
                "★★★ `{PASTE_COMMAND}` IS NOT OFFERED IN THIS MODE. Ctrl+C ran and Ctrl+V \
                 reached the shell and was refused by the mode gate — \
                 `{CHORD_NOT_OFFERED} id={PASTE_COMMAND}` is in the trace. So this phase's \
                 question cannot be asked here, and that is an APPLICATION finding rather than \
                 a harness one: Review can copy a comment and cannot paste one. \
                 `copying_a_sticky_note_carries_the_whole_comment` meets the same gate and \
                 FAILS on it; this check SKIPs because its own subject is whether the paste is \
                 faithful, which is unanswerable while there is no paste. Trace: {}.",
                session.trace_path().display()
            )));
        }
        return Err(Error::new(format!(
            "Ctrl+C then Ctrl+V produced no `{PASTED}` line, and no `{CHORD_NOT_OFFERED}` line \
             for `{PASTE_COMMAND}` either. Both are chords and a chord with a dock panel raised \
             is the harness primitive this suite has measured as unreliable, so this is \
             reported as a SKIP: it says nothing about whether the paste is faithful. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    if pasted.get("note") != Some("true") {
        return Ok(Some(format!(
            "THE PASTE LOST THE COMMENT: `{}`. The annotation copied carries a note — this check \
             wrote it four steps ago and the panel read it back — and the pasted copy does not. \
             That is the clipboard round-tripping through `MarkupSpec`, which cannot express \
             `/Contents`, `/T` or `/M`. ★ It is invisible on the page: the words live in a pop-up \
             this shell does not draw, so the copy looks correct and is not. \
             `canvas::clipboard::carried_options` is what should have carried them.",
            pasted.raw
        )));
    }
    report.note(format!(
        "★★★ the note survived a copy and a paste: `{}`",
        pasted.raw
    ));
    Ok(None)
}
