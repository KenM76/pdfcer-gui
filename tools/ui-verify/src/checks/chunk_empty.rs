//! `emptying_a_chunk_commits_the_emptying` — **O216 ask 1, driven**: select
//! every character inside a text chunk, delete them, commit, and find out
//! whether anything was ever asked of the engine.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/chunk_empty.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::save_copy::{click_command, click_tab};
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The mode whose tab list carries Edit.
const MODE: &str = "edit";
/// The tab the text commands live on, as (region, id).
const EDIT_TAB: (&str, &str) = ("ribbon.tab.edit", "edit");
/// The command that arms the caret, as (region, id).
const EDIT_TEXT_ITEM: (&str, &str) = ("ribbon.item.edit.text", "edit.text");
/// The command id alone, for the messages.
const EDIT_TEXT: &str = EDIT_TEXT_ITEM.1;
/// `text-edit-tool tool=…` — the canvas reporting what armed.
const TOOL_EVENT: &str = "text-edit-tool";
/// The `Debug` spelling the canvas reports for the Edit variant.
const TOOL_EDIT: &str = "TextEdit(Edit)";
/// `text-edit-caret kind=… page=… run=… len=…` — a click resolved a run.
const CARET_EVENT: &str = "text-edit-caret";
/// `text-edit-declined reason=…` — a click did not.
const DECLINED_EVENT: &str = "text-edit-declined";
/// `text-edit-became-add reason=no-run-under-the-click` — the aim was on paper.
const BECAME_ADD_EVENT: &str = "text-edit-became-add";

/// The draft's own account of its selection, as `canvas::textedit::keys`
/// publishes it: `from=… to=… n=…`, or `none caret=…`.
///
/// Emitted by `trace_on_change`, so it appears once per change rather than
/// once per frame — which is what makes counting the lines meaningful and what
/// makes the *last* one the current state.
const SELECT_EVENT: &str = "text-select";

/// `text-edit-plan page=… run=… disposition=… reason=… pinned=…` — **the line
/// whose absence was the whole defect.**
///
/// Raised by `app::actions::textcommit::commit_text_edit` before the engine is
/// called, so its presence says an action was raised and a plan was built, and
/// says nothing about whether the engine agreed. Those are separate questions
/// and this check asks them in that order: a build that reinstates the guard
/// fails here, a build whose engine refuses fails one step further down with
/// the engine's own sentence quoted.
const PLAN_EVENT: &str = "text-edit-plan";

/// `edit-text page=… n=… epoch=… disclosures=…` — the funnel's success line.
const EDIT_EVENT: &str = "edit-text";
/// `edit-text-refused page=… n=… detail=…` — the funnel's refusal line.
const REFUSED_EVENT: &str = "edit-text-refused";

/// See the module documentation.
pub struct EmptyingAChunkCommitsTheEmptying;

impl Check for EmptyingAChunkCommitsTheEmptying {
    fn name(&self) -> &'static str {
        "emptying_a_chunk_commits_the_emptying"
    }

    fn defect(&self) -> &'static str {
        "deleting every character inside a text chunk and committing asks the engine for \
         nothing at all — no plan, no edit, no refusal and no sentence — so an edit the \
         operator made is dropped in silence and reads as a save that failed"
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

/// The last `text-select` line's fields, as the draft reported them.
///
/// `None` where the event has not been seen at all, which is a different
/// finding from a selection of zero and must not be flattened into one.
fn last_selection(session: &Session) -> Result<Option<(Option<usize>, Option<usize>)>> {
    let trace = session.trace()?;
    Ok(trace
        .last(SELECT_EVENT)
        .map(|l| (l.get_usize("n"), l.get_usize("caret"))))
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
        .ok_or_else(|| Error::new("no --pdf. This check needs a document with text in it."))?;
    let Some(target) = ctx.target else {
        return Err(Error::new(
            "no --doc-point. There is deliberately no default: an Edit click on blank paper \
             becomes an ADD draft, and every assertion here is about emptying an EXISTING run. \
             Get a point from the engine — `pdfcer find-text --needle WORD <file>` prints a \
             rectangle per hit — and pass its centre as `--doc-point page,x,y`.",
        ));
    };
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, a \
             ribbon control and the page, and types three chords. Reported as SKIPPED rather \
             than passed: a check that did not run has learned nothing.",
        ));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
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
                "cannot read a page size from {}. Pass --page-size WxH.",
                pdf.display()
            ))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("chunk_empty.trace.txt"));
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

    // --- 1: Edit mode, then the Edit tab, then the caret -------------------
    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
    session.settle(14);
    click_command(&session, &driver, ui_rect, EDIT_TEXT_ITEM, 18)?;
    if !session
        .trace()?
        .events(TOOL_EVENT)
        .any(|l| l.get("tool") == Some(TOOL_EDIT))
    {
        return Ok(Some(format!(
            "`{EDIT_TEXT}` was pressed and no `{TOOL_EVENT} tool={TOOL_EDIT}` followed, so \
             nothing below would mean anything."
        )));
    }

    // --- 2: click where the ENGINE says there is text ----------------------
    let at = crate::checks::text_selection::aim(
        ctx,
        &session,
        page,
        DocPoint::new(target.page, target.x, target.y),
    )?;
    driver.click_at(at)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(caret) = trace.last(CARET_EVENT) else {
        let why = trace
            .events(DECLINED_EVENT)
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Some(match why {
            Some(reason) => format!(
                "the click placed no caret and the canvas declined it with reason={reason}. \
                 There is no chunk to empty, so this check's subject was never reached."
            ),
            None => "the click placed no caret and the canvas declined nothing either, which \
                     is the shape of a click that never arrived."
                .to_owned(),
        }));
    };

    // --- 2b: THE AIM GUARD. A caret with no `run=` opened on bare paper -----
    //
    // `Anchor::Run` prints `run=N`; `Anchor::Origin` prints `origin=x,y`. SKIPPED
    // rather than failed, because it says where the harness aimed and not what
    // the program did — the rule `text_edit_on_a_real_drawing` learned by
    // reporting the opposite once.
    if caret.get("run").is_none() {
        let became = session
            .trace()
            .ok()
            .and_then(|t| t.last(BECAME_ADD_EVENT).map(|l| l.raw.clone()));
        return Err(Error::new(format!(
            "the --doc-point (page {}, {:.1}, {:.1}) found NO RUN under the click, so the caret \
             opened on bare page: `{}`{}. Emptying an Add draft is not this check's subject and \
             every oracle below would be describing a draft that is not a run. SKIPPED rather \
             than failed. ★ Aim with `pdfcer find-text --needle WORD FILE.pdf`. Trace: {}.",
            target.page,
            target.x,
            target.y,
            caret.raw,
            became.map_or_else(String::new, |raw| format!(", and it said so: `{raw}`")),
            session.trace_path().display()
        )));
    }

    // --- 2c: IS THERE ANYTHING TO EMPTY? The calibration, not a formality.
    //
    // Emptying a run of zero characters is a no-op the shell is RIGHT to drop:
    // `draft.text != *original` is false, no action is raised, and this check
    // would then fail naming a defect that is not there. The caret line carries
    // the length, so the question costs one field.
    let Some(len) = caret.get_usize("len").filter(|n| *n > 0) else {
        return Err(Error::new(format!(
            "the caret landed on a run the shell reports as empty or unlengthed: `{}`. There is \
             nothing to delete, so an absent commit below would be correct behaviour rather \
             than the defect. SKIPPED: aim at a run with text in it.",
            caret.raw
        )));
    };
    report.note(format!(
        "★ the click resolved a run of {len} characters: `{}`",
        caret.raw
    ));

    // --- 3: select every character, and CHECK THAT IT HAPPENED --------------
    //
    // `Ctrl+A` in a live draft means *this text*, not *every object on the
    // page* — `canvas::textedit::keys` takes the chord first and consumes it.
    // That is the operator's own gesture and it is also the one place this
    // sequence could silently do nothing.
    driver.press_chord(&[vk::CONTROL], vk::A)?;
    session.settle(14);
    match last_selection(&session)? {
        Some((Some(n), _)) if n == len => {
            report.note(format!("Ctrl+A selected all {n} characters of the run"));
        }
        Some((Some(n), _)) => {
            return Err(Error::new(format!(
                "Ctrl+A selected {n} of the run's {len} characters. The chord arrived and did \
                 something else, so a missing commit below would be about the SELECTION and not \
                 about the commit. SKIPPED."
            )));
        }
        other => {
            return Err(Error::new(format!(
                "Ctrl+A produced no selection the draft would report ({other:?}). Either the \
                 chord never reached the draft or the ribbon took it — and on this build the \
                 ribbon's Select-all acts on OBJECTS, which would explain an empty trace \
                 perfectly. SKIPPED rather than failed: this is the harness's keystroke, not \
                 the program's commit."
            )));
        }
    }

    // --- 4: delete it, and check THAT ---------------------------------------
    //
    // After taking a selection the draft clears its mark and puts the caret at
    // the start of what it removed, so `none caret=0` on a draft whose whole
    // contents were selected is the emptying, observed from outside.
    driver.press(vk::DELETE)?;
    session.settle(14);
    match last_selection(&session)? {
        Some((None, Some(0))) => {
            report.note("Delete took the whole selection — the chunk is empty");
        }
        other => {
            return Err(Error::new(format!(
                "Delete left the draft reporting {other:?} rather than `none caret=0`, so the \
                 run is not empty and what follows would be testing an ordinary edit. SKIPPED."
            )));
        }
    }

    // --- 5: commit -----------------------------------------------------------
    //
    // `Ctrl+Enter`, not Enter: in a text draft a bare Enter means NewLine, and
    // on a run that cannot hold one it is declined — which
    // `text_edit_on_a_real_drawing` once reported as "the commit never reached
    // the engine" when it had never been requested.
    driver.press_chord(&[vk::CONTROL], vk::ENTER)?;
    session.settle(30);
    let trace = session.trace()?;

    // --- 6: THE ASSERTION, and its order is the whole point -------------
    //
    // Was an action raised at all? That is O216's question and nothing else
    // answers it: the plan line is written before the engine is called, so its
    // presence separates *the shell declined in silence* from *the engine
    // refused*, which are different modules and different fixes.
    let Some(plan) = trace.last(PLAN_EVENT) else {
        return Ok(Some(format!(
            "★ EMPTYING THE CHUNK ASKED THE ENGINE FOR NOTHING. The run held {len} characters, \
             Ctrl+A selected all of them, Delete took them and Ctrl+Enter committed — and no \
             `{PLAN_EVENT}` line followed, so no action was raised, no plan was built, no \
             `{EDIT_EVENT}` was attempted and no `{REFUSED_EVENT}` was classified. The operator \
             is told nothing and the document is unchanged, which is his report exactly. The \
             guard that did this is `canvas::textedit::commit_into`'s `Anchor::Run` arm; it \
             belongs on `Origin` and `Box` alone. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the commit built a plan: `{}`", plan.raw));

    // Did the engine take it? A refusal is asked about BEFORE an absence, and
    // quoted verbatim: the engine's sentence is the whole of the diagnosis and
    // any paraphrase here would be a second account of it that could drift.
    if let Some(refused) = trace.last(REFUSED_EVENT) {
        return Ok(Some(format!(
            "the emptying reached the engine and was REFUSED: `{}`. The shell's half of O216 is \
             working — a plan was built and sent — and the defect is now the engine's answer or \
             the shape of the request. Trace: {}.",
            refused.raw,
            session.trace_path().display()
        )));
    }
    let Some(edit) = trace.last(EDIT_EVENT) else {
        return Ok(Some(format!(
            "a plan was built and neither `{EDIT_EVENT}` nor `{REFUSED_EVENT}` followed it, so \
             the commit stopped between the plan and the engine — which is a third place, and \
             not the guard this check was written for. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the engine committed the emptying: `{}`",
        edit.raw
    ));
    Ok(None)
}
