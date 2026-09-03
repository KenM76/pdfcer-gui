//! `save_copy_round_trip` — **the check for the one thing this application
//! could not do**: put an operator's work on disk and get it back.
//!
//! # What was wrong, and why nothing noticed
//!
//! `file.save_copy` has been registered since the ribbon landed. It is drawn on
//! File ▸ Save, it is on the quick-access toolbar, it is bound to `Ctrl+S`, and
//! its tooltip prints "(Ctrl+S)". It had **no dispatch arm**, so every press
//! traced `command-unimplemented` and did nothing — which meant that every
//! feature this project has shipped (dimensions, markup, text marks, form
//! fills, page operations, a document made by `file.new`) was **unwritable**.
//!
//! The whole suite was green throughout, and it had to be: the engine's write
//! verbs are tested in `pdfcer-core`, the command's registration is tested in
//! `shell::commands`, the picker's seam is tested in `app::files`. What has no
//! test anywhere in the workspace is the **join** — that pressing the control
//! reaches a dispatch arm, that the arm reaches a picker, that the picker's
//! answer reaches a write, and that what is written is the document the
//! operator was looking at rather than the one they opened.
//!
//! # The five links, and where each is otherwise covered
//!
//! | # | Link | Its own test |
//! |---|---|---|
//! | 1 | the ribbon click reports the command | `egui-shell`'s `band::render_command` — yes |
//! | 2 | dispatch raises `Action::SaveCopy` | `app::files::tests::the_save_copy_command_raises_the_save_action` — yes |
//! | 3 | the apply reaches the picker and the picker's answer reaches the write | **nothing** |
//! | 4 | the bytes are an **incremental update**, not a full rewrite | `app::save`'s unit tests — yes, on a synthetic edit |
//! | 5 | the operator's **canvas-authored** annotation is in the file that comes out | **nothing** |
//!
//! Link 3 cannot be tested off the binary at all: applying `Action::SaveCopy`
//! in a unit test opens a **real modal dialog** and hangs `cargo test` behind an
//! invisible window (`app::files`' rule 3). Link 5 cannot either, because the
//! only thing that authors an annotation from a *drag* is the running canvas.
//!
//! # The six phases
//!
//! | Phase | Does | Expected |
//! |---|---|---|
//! | A | Review, Markup tab, Comments panel | a `comments-panel listed=N` baseline |
//! | B | arm Rectangle, drag on the page | `markup-tool`, `markup-commit kind=Rectangle`, `add-markup`, and `listed=N+1` |
//! | C | File tab, **Save a copy** | `save-copy … appended=>0`, and a file at the named path |
//! | D | re-read the **source** | byte-identical to before the run |
//! | E | compare the copy's prefix with the source | the source's bytes are the copy's prefix, verbatim |
//! | F | **launch a second process on the copy** | `comments-panel listed=N+1` |
//!
//! # ★ Three falsifying phases, and the build each one catches
//!
//! Phases D, E and F each fail against a *different* plausible wrong
//! implementation, and **no two of them catch the same one**. That is what makes
//! them worth the run time; a check whose phases all fail together is one
//! assertion wearing six hats.
//!
//! ## D catches: *the save wrote over the file that was opened*
//!
//! `dialogs::ocr`'s check already makes this assertion for the OCR write, and it
//! matters more here, not less: this command's own tooltip promises *"The
//! original is never overwritten unless you pick it"*, and the operator does not
//! pick it — `PDFCER_DIAG_SAVE_PATH` names somewhere else. A build whose write
//! ignored the picker's answer and used `doc.path` would pass A, B, C, E and F
//! (E vacuously, F because the annotation really would be in the file) and would
//! have destroyed the operator's drawing. The digest is what survives a reviewer
//! who did not look.
//!
//! ## E catches: *the save was a full rewrite*
//!
//! `EditSession::to_full_bytes` produces a perfectly good PDF that carries the
//! annotation. It passes A, B, C, D and **F**. What it does is destroy every
//! digital signature in the file (§12.8.1) and discard the previous revision
//! that `file.save_copy`'s shipped tooltip promises *"stays intact inside the
//! file"* — neither of which any oracle in this harness can see except this one.
//!
//! A §7.5.6 incremental update cannot rewrite the original bytes by
//! construction: it leaves the base revision alone and appends. So
//! `copy[..source.len()] == source` is a **property of the save mode**, and it
//! is the cheapest possible test that tells the two modes apart.
//!
//! ## F catches: *a file was written and the edit is not in it*
//!
//! The one that gives the check its name. A build that wrote the **base
//! document's** bytes — `Document::bytes()`, the file as opened — instead of the
//! session's update produces a file that exists, opens, has the right page count
//! and is byte-identical to the source. It passes A, B, C, D **and E**, and E
//! passes *trivially*, because the copy simply is the source.
//!
//! So the copy is re-opened **in a second process**, through the same
//! `Document::load` an operator's own File ▸ Open would use, and the Comments
//! panel is asked how many annotations it can see. *A save that writes a file is
//! not the same claim as a save that writes the edit*, and this is the second
//! one.
//!
//! # ★ Both falsifiers were RUN, on 2026-08-14, and both fired
//!
//! [`crate::checks`]' rule for a new check is that *"it must fail against a
//! build where the wiring is absent"*, and that every check here "has been run
//! against such a build and seen to fail". Two builds were planted, one per
//! falsifier, against `fixtures/a1-titleblock.pdf` (12 annotations, page 1
//! 2384 × 1684 pt):
//!
//! | Plant | One line changed | Result |
//! |---|---|---|
//! | `to_full_bytes` in place of `to_incremental_bytes` | phase E: *"they agree for the first 152 bytes and then diverge"*, copy 39,932 B against a source of 39,509 B | **FAIL at E**, having passed A–D |
//! | write `std::fs::read(&doc.path)` — the file as opened — instead of the session's bytes | phase F: the second process listed **12**, not 13 | **FAIL at F**, having passed A–E, *including E* |
//!
//! The second plant is the more instructive of the two, and it is the reason
//! phase F exists rather than being trusted to the trace: **the planted build's
//! own `save-copy` line still read `appended=1002`**, because it computed a
//! perfectly correct `SaveReport` and then wrote different bytes. A trace line is
//! written by the code under test, about itself. Only the two phases that read
//! the *file* — and, for F, read it back through a second process — could tell
//! the difference.
//!
//! The unedited build passes both. That pair is what makes a green result here
//! evidence rather than an absence of evidence.
//!
//! # Why the Comments panel is the oracle, and not a pixel
//!
//! Because a 2 pt rectangle over a drawing is a few hundred antialiased pixels
//! among a page already full of thin dark lines, and no threshold this crate has
//! separates the two — the same reason [`crate::checks::markup_shapes`] saves its
//! capture as evidence rather than using it as an oracle.
//!
//! `panels::comments` traces `comments-panel … listed=N` once per frame, and `N`
//! is derived by walking the **session's own annotation list** through
//! `pdfcer-core`. In the second process that session was built by loading the
//! saved file from disk, so `listed=` there is a statement about the *file*,
//! made by the engine, in a process that never saw the first one.
//!
//! In Review the panel is the first tab of the right stack and is therefore
//! active on the first frame — but a persisted layout beside the binary can say
//! otherwise, so both sessions click `markup.comments` rather than assuming.
//! `app/panels.rs` makes that idempotent: it *shows*, it does not toggle.
//!
//! # The picker is answered, not driven
//!
//! `PDFCER_DIAG_SAVE_PATH` supplies the save dialog's result and the dialog is
//! never opened — `app::files`' established pattern for a native picker, and the
//! RAG note it quotes: *"Don't try to script the dialog."* That is what makes
//! phase C an assertion about **a file on disk** rather than about a button
//! having been pressed.
//!
//! ★ **Say plainly which path was driven, because it is not the whole of the
//! operator's one.** Everything from the ribbon click to the write is the real
//! code path: the click, the dispatch arm, `Action::SaveCopy`, the apply phase,
//! `app::save::save_copy`, `pick_save_path`, `to_incremental_bytes`,
//! `std::fs::write`. The **one** substituted call is the `rfd` dialog itself,
//! and it is substituted because the alternative is not "test it a harder way" —
//! it is a modal top-level window owned by the OS shell that no synthetic input
//! can reach. What is therefore NOT covered here: that the dialog opens with the
//! suggested name pre-filled and in the right folder. That is
//! `app::save::suggested_path`'s unit tests plus two `rfd` calls, and it is
//! stated rather than implied by a green result.
//!
//! # Mouse only
//!
//! Every gesture is a real `SetCursorPos` + `mouse_event`. **`Ctrl+S` is not
//! driven here**, and the keymap binding is covered only by
//!
//! ★ **CORRECTED 2026-08-18.** These headers used to say synthetic keyboard
//! input does not reach the target window on this machine. It DOES — see
//! [`crate::checks::add_text`], which types real characters into a caret
//! draft and asserts they landed. The belief came from `Ctrl+E` producing no
//! trace, which was the dead-keymap defect (fourteen of twenty-one declared
//! chords were dispatched by nothing) misread as a property of the machine —
//! and while it stood nobody drove a chord, so nothing could contradict it.
//!
//! ★★ `Ctrl+S` was one of the fourteen dead chords. It is now dispatchable and
//! this check SHOULD drive it — that is unwritten work, not a limitation.
//! Continuing the original note:
//! `shell::manifest`'s keymap test and by the one dispatcher every route shares.
//!
//! # Every way this reports SKIP, and why none of them is a pass
//!
//! * no binary, no `--pdf`, `--no-input` — the harness never began;
//! * the diagnostic switches did not reach the process;
//! * the page size could not be read and no `--page-size` was given;
//! * a mode segment, a tab or a control was never declared, or took no click;
//! * the canvas is not showing page 1;
//! * **the fixture already carries so many annotations that the panel excludes
//!   some** — the count would then not move by exactly one and the check would
//!   be measuring the panel's editorial rules rather than the save.

use std::path::Path;

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

/// **Review**, whose tab list contains Markup and which mounts the Comments
/// panel by default.
///
/// The weaker claim, and [`crate::checks::markup_rectangle`]'s reason for the
/// same choice: a markup tool that works in Review works in Edit. `file.save_copy`
/// itself is on the File tab, which **every** mode is shown, so nothing about
/// the save is mode-specific — which is worth knowing, because it means this
/// check's mode choice is entirely about reaching the *edit*, not the save.
const MODE: &str = "review";

/// The tab carrying Rectangle and the Comments toggle.
const MARKUP_TAB: (&str, &str) = ("ribbon.tab.markup", "markup");

/// The tab carrying Save a copy. Shown in every mode.
pub(crate) const FILE_TAB: (&str, &str) = ("ribbon.tab.file", "file");

/// The tool that authors the annotation this check follows onto disk.
///
/// Rectangle rather than one of the vertex kinds because it is the shortest
/// gesture that authors anything — one drag, one release, no ending to press —
/// and this check's subject is the save, not the tool.
const RECTANGLE: (&str, &str) = ("ribbon.item.markup.rectangle", "markup.rectangle");

/// The Comments panel's control. `app/panels.rs` makes this a **show**, not a
/// toggle, so pressing it when the panel is already up is a no-op.
const COMMENTS: (&str, &str) = ("ribbon.item.markup.comments", "markup.comments");

/// **The command under test.**
pub(crate) const SAVE: (&str, &str) = ("ribbon.item.file.save_copy", "file.save_copy");

/// `markup-tool tool=…` — the ribbon press reached the canvas.
const ARM_EVENT: &str = "markup-tool";

/// The `Debug` spelling of `CanvasTool::Markup(MarkupKind::Rectangle)`.
const ARM_VALUE: &str = "Markup(Rectangle)";

/// `markup-commit kind=… page=… …` — the shell decided to author one.
const COMMIT_EVENT: &str = "markup-commit";

/// `add-markup page=… n=… epoch=… …` — the **engine** authored it.
const APPLY_EVENT: &str = "add-markup";

/// `comments-panel pages=… listed=N …` — the oracle, in both processes.
const COMMENTS_EVENT: &str = "comments-panel";

/// The field on [`COMMENTS_EVENT`] that counts the rows the panel drew.
const LISTED_FIELD: &str = "listed";

/// The field on [`COMMENTS_EVENT`] that counts annotations the panel left out.
///
/// Read to turn a confusing failure into a SKIP: widgets, popups and trap nets
/// are excluded by editorial rule, so on a fixture full of form fields the
/// listed count would not move by one for a reason that has nothing to do with
/// saving.
const EXCLUDED_FIELD: &str = "excluded_total";

/// `save-copy path=… bytes=… appended=… …` — the write happened.
const SAVED_EVENT: &str = "save-copy";

/// `save-copy-failed path=… detail=…` — a destination was named and no file
/// appeared. Read to make a failure message name the engine's own reason.
const FAILED_EVENT: &str = "save-copy-failed";

/// `save-copy-declined reason=no-document` — the apply arm found nothing open.
const DECLINED_EVENT: &str = "save-copy-declined";

/// `save-picked source=env answer=…` — the seam answered instead of the dialog.
///
/// Read only to improve a SKIP: its absence with `PDFCER_DIAG_SAVE_PATH` set
/// means the picker was reached and did **not** consult the seam, which would
/// have opened a real modal and hung this harness rather than failing it.
const PICKED_EVENT: &str = "save-picked";

/// Bytes the copy must exceed by, at minimum, for `appended` to mean anything.
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH";

/// The rectangle drag, in page fractions: `((x0, y0), (x1, y1))`, PDF user
/// space with y measured from the bottom.
///
/// Well inside the page on every side, so the band cannot be clamped, and big
/// enough that `markup::action`'s no-extent refusal is nowhere near.
const DRAG: ((f64, f64), (f64, f64)) = ((0.24, 0.28), (0.58, 0.46));

/// See the module documentation.
pub struct SaveCopyRoundTrip;

impl Check for SaveCopyRoundTrip {
    fn name(&self) -> &'static str {
        "save_copy_round_trip"
    }

    fn defect(&self) -> &'static str {
        "Save a copy writes nothing, writes over the document that was opened, writes a full \
         rewrite where the command's own tooltip promises an appended update, or writes a file \
         the operator's edit is not in — the five-link chain from a ribbon click to a file on \
         disk, three links of which no test in the workspace can observe"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(skip) => report.skip(skip.to_string()),
        }
    }
}

/// A cheap content digest — length plus FNV-1a over the bytes.
///
/// The same function, for the same reason, as [`crate::checks::ocr`]'s: the
/// question is *"did this file change"*, the adversary is a bug rather than a
/// forger, and carrying a SHA-2 implementation into this crate to answer it
/// would be a dependency for nothing. The **length is part of the digest** so a
/// truncation cannot be hidden by a hash collision.
fn digest(bytes: &[u8]) -> (usize, u64) {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (bytes.len(), hash)
}

/// How many times the shell has reported `id` invoked.
///
/// A **count**, never a presence: this check clicks three different controls in
/// one run, and "has it ever been invoked?" would be answered `true` by a click
/// made ten seconds earlier.
fn invokes(session: &Session, id: &str) -> Result<usize> {
    Ok(shell_trace(session)?
        .events(INVOKE_EVENT)
        .filter(|l| l.get("id") == Some(id))
        .count())
}

/// The most recent `comments-panel` line's `listed=` count.
///
/// `None` when the panel has not drawn at all, which the caller reports as a
/// SKIP rather than as a failure: a panel that is not on screen is a layout
/// fact, not a save one.
fn listed(trace: &Trace) -> Option<usize> {
    trace.last(COMMENTS_EVENT)?.get_usize(LISTED_FIELD)
}

/// Click a ribbon tab and confirm the shell reported it.
/// `pub(crate)` for [`crate::checks::text_edit`], which has to reach the Edit
/// tab to arm the caret tool and the File tab to save. A second copy of these
/// twelve lines would be a second place the "a click that produced no
/// `ribbon-tab-activated` is a SKIP, not a failure" rule could drift — and that
/// rule is the one thing standing between this harness and reporting a machine
/// that will not take a click as a broken application.
pub(crate) fn click_tab(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
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
    let before = shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count();
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(14);
    if shell_trace(session)?
        .events(TAB_EVENT)
        .filter(|l| l.get("tab") == Some(id))
        .count()
        <= before
    {
        return Err(Error::new(format!(
            "the click on `{region}` produced no new `{TAB_EVENT} tab={id}` line. The mode click \
             DID land, so pointer input works and this is not the input channel; the likely cause \
             is that the tab moved between the frame that declared its rect and the frame that \
             received the click."
        )));
    }
    Ok(())
}

/// Locate a declared control and refuse a degenerate rectangle.
fn control(trace: &Trace, ui_rect: &str, name: &str) -> Result<LRect> {
    let rect = declared(trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the active tab publishes its controls' rects and none of them is `{name}`. This \
             build does not have that command on that tab — check `shell::manifest` and \
             `shell::commands`. Controls declared: {}.",
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

/// Click a band control and confirm the **shell** reported the invoke.
///
/// A SKIP rather than a failure when nothing was reported, on
/// [`crate::checks::markup_rectangle`]'s rule: a check that could not deliver a
/// click has learned nothing about the application, and naming a feature as the
/// culprit when nothing was ever clicked at it is worse than no check at all.
/// `pub(crate)` for [`crate::checks::text_edit`] — see [`click_tab`].
pub(crate) fn click_command(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    (region, id): (&str, &str),
    settle: u32,
) -> Result<()> {
    let rect = control(&session.trace()?, ui_rect, region)?;
    let before = invokes(session, id)?;
    driver.click_at(session.frame()?.declared_center(rect))?;
    session.settle(settle);
    if invokes(session, id)? <= before {
        return Err(Error::new(format!(
            "`{region}` DID NOT TAKE THE CLICK: it was declared at {rect:?} and the click produced \
             no `{INVOKE_EVENT} id={id}`. A document with pages is open, so a greyed control is \
             the wrong reading for a `doc.open`/`doc.pages` gate. Commands the shell reported \
             invoked this run: {}.",
            list_str(
                &shell_trace(session)?
                    .events(INVOKE_EVENT)
                    .filter_map(|l| l.get("id"))
                    .collect::<Vec<&str>>()
            )
        )));
    }
    Ok(())
}

/// Put a launched session into Review with the Comments panel showing, and
/// report how many annotations the panel can see.
///
/// Used **twice** — once on the fixture and once on the saved copy in a second
/// process — which is the whole reason it is a function: the two counts have to
/// be produced by the identical sequence, or the comparison at the end is
/// between two different measurements.
///
/// # ★ The panel is reached by the MODE, not by its ribbon control
///
/// `app::modes::defaults`' `review` arrangement mounts Comments as the first tab
/// of the right stack, so it is active and tracing from the first Review frame.
/// The obvious belt-and-braces — click `markup.comments` as well, since
/// `app/panels.rs` makes that a *show* rather than a toggle — was tried and
/// **removed**, because it does not work on this build and the reason is a
/// finding rather than a harness problem: on a 2384 pt-wide sheet the Markup
/// tab's bands overflow and the Comments group is not drawn at all, so
/// `ribbon.item.markup.comments` is never declared and there is nothing to aim
/// at. (The groups that do publish rects are Shapes, Text and Notes.)
///
/// So the control is a **fallback**, attempted only when the mode alone did not
/// produce the panel — which is the state a persisted layout beside the binary
/// could produce — and its absence is reported as part of one SKIP rather than
/// as a failure of the save.
fn comments_count(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
    what: &str,
) -> Result<usize> {
    driving::click_mode_segment(session, driver, ui_rect, MODE)?;
    session.settle(16);

    if listed(&session.trace()?).is_none() {
        // The mode's default arrangement did not bring the panel up. Try its
        // control; if that is not on screen either, say both things at once.
        report.note(format!(
            "the `{MODE}` arrangement did not mount the Comments panel for {what}; trying its \
             ribbon control"
        ));
        click_tab(session, driver, ui_rect, MARKUP_TAB)?;
        click_command(session, driver, ui_rect, COMMENTS, 18)?;
    }

    let trace = session.trace()?;
    let count = listed(&trace).ok_or_else(|| {
        Error::new(format!(
            "the Comments panel never traced `{COMMENTS_EVENT}` for {what}, so this check has no \
             oracle. Either the panel is not mounted (check `app::modes::defaults`' `{MODE}` \
             arrangement, which is supposed to put it first in the right stack) or it is drawing \
             without tracing. Trace: {}.",
            session.trace_path().display()
        ))
    })?;
    let excluded = trace
        .last(COMMENTS_EVENT)
        .and_then(|l| l.get_usize(EXCLUDED_FIELD))
        .unwrap_or(0);
    if excluded > 0 {
        return Err(Error::new(format!(
            "the Comments panel excluded {excluded} annotation(s) on {what} — widgets, popups or \
             trap nets, which it leaves out by editorial rule. This check's verdict is that \
             `{LISTED_FIELD}` moves by exactly one when one rectangle is authored, and on a \
             document with excluded annotations that arithmetic is measuring the panel's rules \
             rather than the save. Point --pdf at a drawing without form fields."
        )));
    }
    report.note(format!(
        "{what}: the Comments panel lists {count} annotation(s) — `{}`",
        trace
            .last(COMMENTS_EVENT)
            .map_or_else(String::new, |l| l.raw.clone())
    ));
    Ok(count)
}

/// Launch one process with both diagnostic channels armed and the save seam set.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    pdf: &Path,
    target: &Path,
    trace_name: &str,
) -> Result<Session> {
    let mut spec = LaunchSpec::new(
        ctx.resolve_exe().ok_or_else(|| {
            Error::new(format!(
                "no binary to drive. Pass --exe, or build the profile's default at {}.",
                ctx.profile.default_exe
            ))
        })?,
        ctx.out(trace_name),
    );
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} on {} as pid {} with {SAVE_PATH_ENV}={}",
        spec.exe.display(),
        pdf.display(),
        session.pid(),
        target.display()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    if !session.trace()?.started(ctx.profile.vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch {}={} did not reach the process \
             and this check has no oracle. Captured stderr is at {}.",
            ctx.profile.vocab.start_event,
            ctx.profile.diag_env.0,
            ctx.profile.diag_env.1,
            session.trace_path().display()
        )));
    }
    Ok(session)
}

/// Run the sequence. `Err` is SKIP, `Ok(Some(_))` is FAIL, `Ok(None)` is a pass.
#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    // --- preconditions -----------------------------------------------------
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check authors an annotation on a real page and then follows it onto \
             disk, so it needs a document — and `file.save_copy` is gated on `doc.open`, which \
             means with nothing open the control is correctly greyed and this check would be \
             measuring the gate rather than the feature.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is seven clicks and a drag across two \
             processes. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
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
                 window y (down). Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    // The source's digest, taken BEFORE anything runs. Phase D's whole verdict
    // rests on this pair.
    let source = std::fs::read(&pdf)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", pdf.display())))?;
    let before = digest(&source);
    report.note(format!(
        "fixture {} — page 1 is {:.0}x{:.0} pt, {} bytes, digest {:016x}",
        pdf.display(),
        page.width_pt,
        page.height_pt,
        before.0,
        before.1
    ));

    // Where the copy goes. Beside the harness's own output, never beside the
    // fixture: a stray file in a fixtures directory is one somebody eventually
    // commits.
    let target = ctx.out("save-copy-round-trip.pdf");
    let _ = std::fs::remove_file(&target);
    if target == pdf {
        return Ok(Some(
            "the copy's path IS the fixture's path, so phases D and E would compare a file with \
             itself and prove nothing. This is a harness defect rather than an application one, \
             and it is reported as a failure so it cannot be mistaken for a pass."
                .to_owned(),
        ));
    }

    // =======================================================================
    // The authoring process. Scoped, so it is killed before the second one
    // launches — two pdfcer windows competing for the foreground would make
    // every click after the first one a race.
    // =======================================================================
    let listed_before;
    {
        let session = launch(ctx, report, &pdf, &target, "save_copy.trace.txt")?;
        let driver = Driver::new(session.window());

        // --- PHASE A: the baseline the whole round trip is measured against --
        listed_before = comments_count(&session, &driver, ui_rect, report, "the fixture")?;

        // --- PHASE B: author a rectangle on the page ------------------------
        //
        // The tab click is idempotent when phase A's fallback already made it,
        // and the shell reports every click on a tab including one that is
        // already active, so `click_tab`'s own assertion holds either way.
        click_tab(&session, &driver, ui_rect, MARKUP_TAB)?;
        click_command(&session, &driver, ui_rect, RECTANGLE, 16)?;
        let trace = session.trace()?;
        if !trace
            .events(ARM_EVENT)
            .any(|l| l.get("tool") == Some(ARM_VALUE))
        {
            return Ok(Some(format!(
                "RECTANGLE WAS INVOKED AND THE CANVAS TOOL DID NOT ARM, so this check cannot get \
                 as far as the save. The shell traced `{INVOKE_EVENT} id={}` and there is no \
                 `{ARM_EVENT} tool={ARM_VALUE}`. That is `markup_rectangle`'s subject rather than \
                 this one's — read its verdict first.",
                RECTANGLE.1
            )));
        }

        let from = aim(
            ctx,
            &session,
            page,
            DocPoint::new(0, DRAG.0.0 * page.width_pt, DRAG.0.1 * page.height_pt),
        )?;
        let to = aim(
            ctx,
            &session,
            page,
            DocPoint::new(0, DRAG.1.0 * page.width_pt, DRAG.1.1 * page.height_pt),
        )?;
        let applies_before = session.trace()?.events(APPLY_EVENT).count();
        driver.drag(from, to)?;
        session.settle(24);

        let trace = session.trace()?;
        let Some(commit) = trace
            .events(COMMIT_EVENT)
            .filter(|l| l.get("kind") == Some("Rectangle"))
            .last()
        else {
            return Ok(Some(format!(
                "THE DRAG AUTHORED NOTHING, so there is no edit for the save to carry. The tool \
                 armed and a real drag across the page produced no `{COMMIT_EVENT} kind=Rectangle`. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        };
        if trace.events(APPLY_EVENT).count() <= applies_before {
            return Ok(Some(format!(
                "THE ENGINE NEVER AUTHORED THE RECTANGLE. The application decided to author one — \
                 `{}` — and no `{APPLY_EVENT}` line followed, so `app::actions`' apply arm never \
                 ran or `EditSession::add_markup` refused.",
                commit.raw
            )));
        }
        report.note(format!("the rectangle was authored: `{}`", commit.raw));

        // …and the panel sees it. This is the *in-memory* half of the round
        // trip, and it is what makes phase F's number meaningful: without it, a
        // build whose panel could not see a freshly authored annotation would
        // fail phase F for a reason that has nothing to do with saving.
        session.settle(10);
        let listed_after = listed(&session.trace()?).unwrap_or(listed_before);
        if listed_after != listed_before + 1 {
            return Ok(Some(format!(
                "THE COMMENTS PANEL DOES NOT SEE THE ANNOTATION THAT WAS JUST AUTHORED: it listed \
                 {listed_before} before the drag and {listed_after} after it. The engine traced \
                 `{APPLY_EVENT}`, so the annotation IS on the session — which makes this the \
                 panel's finding rather than the save's, and it also means phase F's oracle would \
                 be measuring the panel. Reported here rather than at the end, where it would read \
                 as a save that lost the edit."
            )));
        }
        report.note(format!(
            "the Comments panel now lists {listed_after}, one more than before the drag — the \
             annotation is on the session in memory"
        ));

        // --- PHASE C: save a copy -------------------------------------------
        click_tab(&session, &driver, ui_rect, FILE_TAB)?;
        click_command(&session, &driver, ui_rect, SAVE, 40)?;

        let trace = session.trace()?;
        let Some(saved) = trace.last(SAVED_EVENT) else {
            return Ok(Some(format!(
                "NOTHING WAS WRITTEN. `{}` was invoked — the shell traced it — and no \
                 `{SAVED_EVENT}` line followed. {}\n{}\n{}\nTrace: {}.",
                SAVE.1,
                if trace
                    .events(UNIMPLEMENTED_EVENT)
                    .any(|l| l.get("id") == Some(SAVE.1))
                {
                    format!(
                        "★ The application traced `{UNIMPLEMENTED_EVENT} id={}`, which is \
                         `dispatch_command`'s fall-through: the command arrived and dispatch had \
                         NO ARM for it. That is the exact state this check was written against — \
                         look at `app/dispatch.rs`'s `\"file.save_copy\"` arm.",
                        SAVE.1
                    )
                } else {
                    format!(
                        "There is no `{UNIMPLEMENTED_EVENT}` for it, so the command did reach an \
                         arm and the failure is further down: `Action::SaveCopy` may not be \
                         raised, or `PdfcerApp::apply` may not be routing it."
                    )
                },
                match trace.last(FAILED_EVENT) {
                    Some(l) => format!("The write was ATTEMPTED and refused: `{}`.", l.raw),
                    None => format!("There is no `{FAILED_EVENT}` either, so no write was tried."),
                },
                match trace.last(DECLINED_EVENT) {
                    Some(l) => format!("The apply arm declined: `{}`.", l.raw),
                    None => match trace.last(PICKED_EVENT) {
                        Some(l) => format!("The picker WAS reached and answered: `{}`.", l.raw),
                        None => format!(
                            "There is no `{PICKED_EVENT}` either, so `pick_save_path` was never \
                             called — the arm did not get that far."
                        ),
                    },
                },
                session.trace_path().display()
            )));
        };
        report.note(format!("the copy was written: `{}`", saved.raw));

        let appended = saved.get_usize("appended").unwrap_or(0);
        if appended == 0 {
            return Ok(Some(format!(
                "THE SAVE APPENDED NOTHING. `{}` reports `appended=0`, which \
                 `save_incremental`'s contract says happens only for an **empty dirty set** — and \
                 an annotation had just been authored. So either the write ran against a session \
                 that does not carry the edit, or it wrote the base document's bytes. Phase F \
                 would catch it too; it is caught here because the trace names the moment.",
                saved.raw
            )));
        }
    }

    // =======================================================================
    // PHASES D and E — what is on disk, asserted by the harness rather than by
    // the process that wrote it
    // =======================================================================
    if !target.is_file() {
        return Ok(Some(format!(
            "THE SAVED FILE IS NOT WHERE IT WAS ASKED FOR. The application traced `{SAVED_EVENT}` \
             and nothing exists at {}. {SAVE_PATH_ENV} named that path, so the picker's answer and \
             the write's destination have come apart — which is the one thing standing between \
             this command and the file the operator opened.",
            target.display()
        )));
    }
    let copy = std::fs::read(&target)
        .map_err(|e| Error::new(format!("cannot read {}: {e}", target.display())))?;
    report.artifact(target.clone());

    // --- ★ PHASE D: the document that was opened is untouched ---------------
    let after_bytes = std::fs::read(&pdf)
        .map_err(|e| Error::new(format!("cannot re-read {}: {e}", pdf.display())))?;
    let after = digest(&after_bytes);
    if after != before {
        return Ok(Some(format!(
            "★ THE DOCUMENT THAT WAS OPENED HAS BEEN MODIFIED. {} was {} bytes (digest {:016x}) \
             before the run and is {} bytes (digest {:016x}) after it.\n\n\
             `file.save_copy`'s own tooltip promises *\"The original is never overwritten unless \
             you pick it\"*, and the operator did not pick it — {SAVE_PATH_ENV} named somewhere \
             else. Look at `app::save::save_copy`, and at whether \
             `app::files::pick_save_path`'s answer is the path actually passed to the write.",
            pdf.display(),
            before.0,
            before.1,
            after.0,
            after.1
        )));
    }
    report.note(format!(
        "★ the opened document is byte-identical after the run — {} bytes, digest {:016x}",
        after.0, after.1
    ));

    // --- ★ PHASE E: the copy is an APPENDED revision, not a rewrite ---------
    if !copy.starts_with(&source) {
        let shared = copy
            .iter()
            .zip(source.iter())
            .take_while(|(a, b)| a == b)
            .count();
        return Ok(Some(format!(
            "★ THE COPY IS A FULL REWRITE, NOT AN APPENDED UPDATE. The source is {} bytes and the \
             copy is {} bytes; they agree for the first {shared} and then diverge.\n\n\
             A §7.5.6 incremental update cannot rewrite the base revision — it leaves the original \
             bytes alone and appends after them — so this is `EditSession::to_full_bytes`, which \
             **destroys every digital signature in the file** (§12.8.1) and discards the previous \
             revision that `file.save_copy`'s shipped tooltip promises *\"stays intact inside the \
             file\"*.\n\
             THIS IS THE ASSERTION NO OTHER PHASE CAN MAKE: a full rewrite passes A, B, C, D and \
             F, because the annotation really is in the file it produces. See `app::save` \
             section 1, which records that the save mode is not that module's to re-open.",
            source.len(),
            copy.len()
        )));
    }
    report.note(format!(
        "★ the copy's first {} bytes are the source's, verbatim, and it is {} bytes long — an \
         appended revision, which is what the command's tooltip promises and what preserves any \
         signature the file carries",
        source.len(),
        copy.len()
    ));

    // =======================================================================
    // ★ PHASE F — THE ROUND TRIP: re-open the file that came out
    // =======================================================================
    let listed_reopened = {
        // A second `PDFCER_DIAG_SAVE_PATH` is set and never used; harmless, and
        // cheaper than a second launch helper that differs in one field.
        let session = launch(
            ctx,
            report,
            &target,
            &ctx.out("save-copy-unused.pdf"),
            "save_copy.reopen.trace.txt",
        )?;
        let driver = Driver::new(session.window());
        comments_count(&session, &driver, ui_rect, report, "the SAVED COPY")?
    };

    if listed_reopened != listed_before + 1 {
        return Ok(Some(format!(
            "★ THE EDIT IS NOT IN THE SAVED FILE. The fixture listed {listed_before} \
             annotation(s); a rectangle was authored and the panel listed {} in the process that \
             authored it; and a SECOND process, opening the saved copy from disk through the same \
             `Document::load` an operator's File ▸ Open uses, lists {listed_reopened}.\n\n\
             A save that writes a file is not the same claim as a save that writes the EDIT, and \
             this is the second one. The plausible build this catches writes the **base \
             document's** bytes rather than the session's incremental update: it produces a valid \
             PDF with the right page count, it leaves the original alone (phase D passes) and it \
             trivially begins with the original's bytes (phase E passes, because it IS them). \
             Look at `app::save::write_copy` and at what it hands \
             `EditSession::to_incremental_bytes`.",
            listed_before + 1
        )));
    }
    report.note(format!(
        "★★ ROUND TRIP PROVEN: a second process opened the saved copy from disk and its Comments \
         panel lists {listed_reopened} annotation(s), one more than the fixture's \
         {listed_before}. The rectangle an OS-injected drag authored in the first process is in \
         the file the second one read"
    ));

    report.note(
        "NOT covered here: the save dialog itself. `PDFCER_DIAG_SAVE_PATH` answers it and it is \
         never opened, because it is a modal top-level window owned by the OS shell that no \
         synthetic input can reach (`app::files`' rule 1). Everything else — the ribbon click, the \
         dispatch arm, `Action::SaveCopy`, the apply phase, `to_incremental_bytes` and the write — \
         is the real code path. That the dialog opens pre-filled with the suggested name is \
         covered by `app::save::suggested_path`'s unit tests and by nothing that runs a window",
    );
    report.note(
        "NOT covered here: `Ctrl+S`, which is unwritten work — keystrokes DO reach the window, \
         this session (see find_bar), so the chord is covered by the manifest's keymap test and by \
         the single dispatcher every route shares, and the gap is on the record rather than \
         implied by a green result",
    );

    let _ = std::fs::remove_file(ctx.out("save-copy-unused.pdf"));
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names this check greps for are the ones `egui-shell` builds, and the
    /// ids are the ones the application registers.
    ///
    /// Pinned for the reason every sibling check pins its own: the two crates
    /// are joined by a **string** and nothing else, so a rename would leave both
    /// sides compiling while every assertion here quietly stopped matching — and
    /// a check that matches nothing passes vacuously.
    #[test]
    fn the_selectors_match_the_shells_own_spelling() {
        for (region, id) in [RECTANGLE, COMMENTS, SAVE] {
            assert_eq!(region, format!("ribbon.item.{id}"));
            assert!(region.starts_with(ITEM_PREFIX), "{region}");
        }
        assert_eq!(MARKUP_TAB.0, format!("ribbon.tab.{}", MARKUP_TAB.1));
        assert_eq!(FILE_TAB.0, format!("ribbon.tab.{}", FILE_TAB.1));
        // The save lives on File and the authoring lives on Markup: two tabs,
        // and the check has to cross between them. If these ever became the
        // same tab the second `click_tab` would be a no-op the shell does not
        // report and phase C would SKIP for a confusing reason.
        assert_ne!(MARKUP_TAB.1, FILE_TAB.1);
        assert!(
            SAVE.1.starts_with(FILE_TAB.1),
            "a command id names its owning tab, so `{}` on `{}` must share the prefix",
            SAVE.1,
            FILE_TAB.1
        );
        assert_eq!(MODE, "review");
    }

    /// ★ **The digest notices a single changed byte and a truncation.**
    ///
    /// Phase D's whole verdict rests on this function, so a digest that answered
    /// "unchanged" for a modified file would turn the check's assertion about
    /// the operator's own drawing into a formality that always passes.
    #[test]
    fn the_digest_notices_a_single_changed_byte_and_a_truncation() {
        let a = b"%PDF-1.7 a drawing";
        let mut b = a.to_vec();
        b[9] ^= 0x01;
        assert_ne!(digest(a), digest(&b), "one flipped bit must change it");
        assert_ne!(
            digest(a),
            digest(&a[..a.len() - 1]),
            "a truncation must change it, which is what the length is in the tuple for"
        );
        assert_eq!(digest(a), digest(a), "and it must be stable");
    }

    /// ★ **Phase E's comparison really distinguishes an appended update from a
    /// rewrite.**
    ///
    /// The predicate is one `starts_with`, which is exactly the kind of line
    /// that gets "simplified" into something that always passes. Both
    /// directions are pinned, and the *rewrite* fixture is deliberately one that
    /// shares a long prefix with the source — a full rewrite of a PDF really
    /// does begin `%PDF-1.x`, so a check that only compared the first few bytes
    /// would pass against the build this phase exists to catch.
    #[test]
    fn an_appended_revision_is_told_apart_from_a_rewrite() {
        let source = b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\ntrailer\n%%EOF\n";
        let appended = {
            let mut v = source.to_vec();
            v.extend_from_slice(b"2 0 obj\n<<>>\nendobj\ntrailer\n%%EOF\n");
            v
        };
        let rewritten = b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n2 0 obj\n<<>>\nendobj\ntrailer\n%%EOF\n";

        assert!(appended.starts_with(source), "an update keeps the original");
        assert!(
            !rewritten.starts_with(source),
            "a rewrite does not, even though it shares the header — which is why the comparison \
             is over the WHOLE source rather than over a prefix of it"
        );
        // …and the two share more than a trivial prefix, so this fixture is
        // exercising the bound rather than flattering it.
        let shared = rewritten
            .iter()
            .zip(source.iter())
            .take_while(|(a, b)| a == b)
            .count();
        assert!(
            shared > 20,
            "the rewrite fixture must agree with the source for a long way, or it is not testing \
             what a real full rewrite looks like (shared {shared})"
        );
    }

    /// The drag is a real rectangle, well inside the page.
    ///
    /// A degenerate one would be refused by `markup::action`'s no-extent rule
    /// and phase B would report "the drag authored nothing" about a fixture
    /// defect. A drag near the edge would be clamped by the canvas.
    #[test]
    fn the_drag_is_a_real_rectangle_inside_the_page() {
        assert!((DRAG.0.0 - DRAG.1.0).abs() > 0.1);
        assert!((DRAG.0.1 - DRAG.1.1).abs() > 0.1);
        for (x, y) in [DRAG.0, DRAG.1] {
            assert!(
                (0.05..=0.95).contains(&x) && (0.05..=0.95).contains(&y),
                "({x}, {y}) is too close to the page edge to survive a margin"
            );
        }
    }

    /// The two streams are parsed out of one file without contaminating each
    /// other, and every field this check reads is read from the line the
    /// application really writes.
    #[test]
    fn the_application_and_shell_streams_do_not_contaminate_each_other() {
        let text = "pdfcer-diag start argv1=None\n\
                    egui-shell-diag ribbon-command-invoked id=file.save_copy handler=110\n\
                    pdfcer-diag comments-panel pages=1 listed=3 with_note=0 excluded_total=0\n\
                    pdfcer-diag save-copy path=\"D:\\\\Program Files\\\\a copy.pdf\" bytes=9001 \
                    appended=412 objects=3 verbatim=2 reserialized=1 promoted=0 deleted=0 \
                    identical=false delinearized=false epoch=1 origin=Opened";
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
                .any(|l| l.get("id") == Some(SAVE.1))
        );
        assert_eq!(listed(&app), Some(3));
        assert_eq!(
            app.last(COMMENTS_EVENT)
                .and_then(|l| l.get_usize(EXCLUDED_FIELD)),
            Some(0)
        );

        // ★ The save line's fields survive a path with SPACES in it. The
        // application Debug-quotes the path for exactly this reason, and a
        // build that stopped would leave every field after `path=` unreadable —
        // so `appended` would parse as `None`, default to 0, and this check
        // would report "the save appended nothing" about a working save.
        let saved = app.last(SAVED_EVENT).expect("the save line parses");
        assert_eq!(saved.get_usize("appended"), Some(412));
        assert_eq!(saved.get_usize("bytes"), Some(9001));
        assert_eq!(saved.get("identical"), Some("false"));
        assert_eq!(saved.get("epoch"), Some("1"));
    }
}
