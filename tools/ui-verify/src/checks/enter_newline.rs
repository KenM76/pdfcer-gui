//! # `enter_newline` — Enter makes a second line, and Ctrl+Enter finishes it
//!
//! `OPERATOR_REQUESTS.md` **O127**, defect 2:
//!
//! > *"also can the enter key create new lines when we are editing or creating
//! > text?"*
//!
//! ## ⚠ WRITTEN 2026-09-04 AND **NOT EXECUTED**
//!
//! The session that wrote it was instructed not to launch the GUI — the
//! operator was at his keyboard and a second run would have fought his pointer.
//! It is registered anyway, deliberately and on the precedent this file's
//! neighbours set: `left_rail`, `properties_tool` and `protect` all carry the
//! same note. **A check that is not in the list is a check nobody will ever
//! run**, and an unregistered file is a promise rather than an instrument.
//!
//! ⇒ Whoever runs the suite next is the first thing that executes this. If it
//! is red on its first run, read §"what each failure means" before assuming the
//! shell is broken: an unrun check's first failure is as likely to be in the
//! check as in the subject, and that is not a reason to weaken it.
//!
//! ## ★★★ What this check is for, and why the unit tests are not enough
//!
//! `canvas::textedit::keys::enter_means` is a pure function with four unit
//! tests, and they prove **the rule**. They cannot prove any of these:
//!
//! | link | provable without driving? |
//! |---|---|
//! | the Enter keystroke **reaches** the draft at all | **no** |
//! | it is not eaten by a guard, the ribbon keymap or a focused widget | **no** |
//! | the caret ends up on the second line rather than at the end of the first | **no** |
//! | the newline **survives the commit** into `add_text` | **no** |
//! | `Ctrl+Enter` is not intercepted before the draft sees it | **no** |
//!
//! Every one of those has failed in this project at least once. `caret::newline`
//! exists **because** `insert` silently ate the exact keystroke this check
//! presses, for a whole driven run, while every unit test stayed green — the
//! trace said the key arrived and the length did not move. That is the shape
//! this file is against.
//!
//! ## The oracle, and why it is a COUNT
//!
//! `add-text page=… n=…` — the funnel's operand count, which
//! `app::actions::addtext` sets to the number of **hard newlines** in what was
//! committed. So:
//!
//! * a build where Enter did nothing commits `n=1`;
//! * a build where Enter committed instead of inserting commits `n=1` **and
//!   commits early**, so the second half of the typing never arrives;
//! * a build where the newline was dropped between the draft and the engine
//!   commits `n=1`;
//! * only a build where the whole chain works commits `n=2`.
//!
//! ★ A count rather than a screenshot, for the reason `text_edit`'s check gives
//! about the same choice: two lines of 11 pt text on an A1 sheet are a few
//! pixels, and an oracle that cannot tell one line from two is not an oracle.
//!
//! ## ★★ Why the text is SEEDED and the Enter is REAL
//!
//! `PDFCER_DIAG_TYPE` puts characters in the draft, because this machine's
//! harness cannot inject arbitrary characters — `sys::vk` is a deliberately
//! closed list of non-character virtual keys and its own comment refuses to
//! grow into `pub const A..Z`.
//!
//! **Enter is not seeded.** It is `sys::vk::ENTER`, pressed for real, because
//! the keystroke is the entire subject: a seam that inserted the newline would
//! be verifying that this check can write a `\n` into a `String`.
//!
//! ⇒ So the shape is: seed a word, press Enter, seed nothing more, press
//! Ctrl+Enter, and read the count. The seed runs once per draft (`Draft::seeded`
//! is consumed on the first frame), so the second line is empty — which is
//! fine and is deliberate: `n` counts hard newlines, and one Enter is one
//! newline whatever is on either side of it.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// Edit mode, then arm **add** text — both through the harness seam.
///
/// ★ `edit.add_text` rather than `edit.text`, because the subject is a caret on
/// **bare page** (`Anchor::Origin`) — the draft that could not take a line break
/// until O127. A box draft always could, so a check that dragged one would pass
/// on the build the operator reported.
const INVOKE: &str = "mode.edit,edit.add_text";
/// The characters the draft is seeded with, before Enter is pressed.
const SEED: &str = "FIRST";
/// `text-edit-caret kind=… page=… origin=… len=…` — a click opened a draft.
const CARET_EVENT: &str = "text-edit-caret";
/// `text-edit-declined reason=…` — a click did not.
const DECLINED_EVENT: &str = "text-edit-declined";
/// `text-edit-enter means=…` — the keystroke arrived, and which branch it took.
///
/// ★★ The one instrument that separates *"the key never got here"* from *"the
/// key got here and the rule chose wrong"*. Those are two different repairs and
/// they leave identical evidence everywhere else: the draft's length does not
/// move either way.
const ENTER_EVENT: &str = "text-edit-enter";
/// `add-text page=… n=… epoch=… disclosures=…` — the funnel's own line.
const ADD_EVENT: &str = "add-text";
/// The page's own region, so a failure can say whether a sheet was drawn.
const PAGE_REGION: &str = "page";
/// Where to click, in PDF user space — bare paper, well clear of the border.
///
/// ★ On `a1-titleblock.pdf`, which is 2384 × 1684 pt. The point is chosen in
/// the middle of the sheet where the fixture draws nothing, so the click cannot
/// land on a run and turn the `Add` draft into an `Edit` one — which
/// `textedit::click` does deliberately, and which would make this check
/// silently test the wrong anchor.
const CLICK_AT: (f64, f64) = (1100.0, 900.0);

/// See the module documentation.
pub struct EnterMakesASecondLineAndControlEnterCommits;

impl Check for EnterMakesASecondLineAndControlEnterCommits {
    fn name(&self) -> &'static str {
        "enter_makes_a_second_line_and_control_enter_commits"
    }

    fn defect(&self) -> &'static str {
        "Enter committed a clicked text draft instead of breaking the line, so new page text \
         could only ever be one line and the operator's question was answered by an edit \
         finishing under him"
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
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks on bare page to open a draft and \
             then presses Enter and Ctrl+Enter for real. The keystrokes ARE the subject — a \
             seam that inserted the newline would verify nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // ★ This check owns its fixture for `reflow`'s reason: the click point is
    // quoted in one sheet's geometry, and aiming it at another sheet lands on
    // whatever happens to be there — which on a CAD drawing is a run, which
    // turns the draft into an `Edit` and makes the check test the anchor it is
    // NOT about, quietly and in the passing direction.
    let pdf = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/a1-titleblock.pdf");
    if !pdf.exists() {
        return Err(Error::new(format!(
            "the fixture is missing at {}.",
            pdf.display()
        )));
    }
    let page = PageGeometry {
        width_pt: 2384.0,
        height_pt: 1684.0,
    };
    let measured = crate::fixture::page_geometry(&pdf);
    if let Some(actual) = measured
        && (actual.width_pt - page.width_pt).abs() > 1.0
    {
        return Err(Error::new(format!(
            "fixtures/a1-titleblock.pdf measures {}x{} pt and this check's click point is \
             quoted in the A1 geometry. Update CLICK_AT — the old point on a new sheet may \
             land on a run, which turns the Add draft into an Edit draft and makes this check \
             pass while testing the wrong thing.",
            actual.width_pt, actual.height_pt
        )));
    }
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are.",
            ctx.profile.name
        ))
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("enter-newline.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    // The one thing this machine cannot type. See the module header.
    spec.env
        .push(("PDFCER_DIAG_TYPE".to_owned(), SEED.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} on fixtures/a1-titleblock.pdf as pid {} with PDFCER_DIAG_INVOKE={INVOKE} \
         and PDFCER_DIAG_TYPE={SEED}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nowhere to put a caret. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- 1: open a draft on bare page --------------------------------------
    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(0, CLICK_AT.0, CLICK_AT.1),
    )?;
    driver.click_at(at)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(caret) = trace.events(CARET_EVENT).last() else {
        let declined = trace
            .events(DECLINED_EVENT)
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Some(match declined {
            Some(reason) => format!(
                "THE CARET WAS REFUSED WITH reason={reason} on bare paper with the Add tool \
                 armed. Since 2026-08-19 a click that names no run opens an origin draft, so \
                 there is nothing here that should refuse. Trace: {}.",
                session.trace_path().display()
            ),
            None => format!(
                "THE CLICK PRODUCED NEITHER A CARET NOR A DECLINE: no `{CARET_EVENT}`, no \
                 `{DECLINED_EVENT}`. Either `edit.add_text` did not arm — look for \
                 `command-declined id=edit.add_text reason=mode-cannot-edit-content` — or the \
                 click missed the sheet. It aimed at ({}, {}) in page points. Trace: {}.",
                CLICK_AT.0,
                CLICK_AT.1,
                session.trace_path().display()
            ),
        }));
    };
    report.note(format!("★ a draft is open: `{}`", caret.raw));
    // ★★ The ANCHOR is asserted, not assumed. A click that landed on a run
    // would produce an `Edit` draft, where Enter is *supposed* to decline — so
    // without this line a check that missed the blank spot would report the
    // decline as the defect and send somebody to fix the thing that is right.
    if !caret.raw.contains("origin=") {
        return Ok(Some(format!(
            "★★ THE CLICK DID NOT OPEN AN ORIGIN DRAFT: `{}`. It landed on existing text, so \
             the anchor is a run and Enter correctly declines there — this check would then be \
             measuring the wrong branch. Move CLICK_AT to bare paper. Trace: {}.",
            caret.raw,
            session.trace_path().display()
        )));
    }

    // --- 2: press Enter for real -------------------------------------------
    driver.press(vk::ENTER)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(enter) = trace.events(ENTER_EVENT).last() else {
        return Ok(Some(format!(
            "★★★ THE ENTER KEY NEVER REACHED THE DRAFT: no `{ENTER_EVENT}` line at all, with a \
             draft open (`{}`).\n\
             The keystroke handler runs only while a text tool is armed \
             (`canvas::interact`), and it reads events only when its `owns_keyboard` \
             argument is true. A focused widget anywhere in the shell takes every key the \
             canvas would have had. This is `DEFECTS.md` D1's shape: a guard asking a wider \
             question than it meant. Trace: {}.",
            caret.raw,
            session.trace_path().display()
        )));
    };
    let means = enter.get("means").unwrap_or("");
    if means != "NewLine" {
        return Ok(Some(format!(
            "★★★ ENTER MEANT `{means}` IN A CLICKED DRAFT, and it must mean `NewLine`.\n\
             `canvas::textedit::keys::enter_means` decides this and is unit-tested, so a \
             disagreement here means the ARM is not asking it, or the anchor is not what the \
             caret line says. `Commit` in particular is the pre-O127 behaviour returning: the \
             operator's *\"can the enter key create new lines\"* answered by an edit finishing \
             under him. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ Enter arrived and broke the line: `{}`",
        enter.raw
    ));

    // --- 3: Ctrl+Enter finishes it -----------------------------------------
    //
    // ★ The keyboard route, which is half of what O127 asked for: *"commit must
    // not be reachable only by mouse."* Clicking away would also commit and
    // would not test the chord.
    driver.press_chord(&[vk::CONTROL], vk::ENTER)?;
    session.settle(40);

    let trace = session.trace()?;
    let Some(added) = trace.events(ADD_EVENT).last() else {
        return Ok(Some(format!(
            "★★★ CTRL+ENTER COMMITTED NOTHING: no `{ADD_EVENT}` line after the chord.\n\
             Either the chord never reached the draft — `enter_means` takes `modifiers.command` \
             first, so a build that reads the modifier wrongly would insert a SECOND newline \
             instead of committing, and the draft would still be open — or the commit reached \
             `add_text` and the engine refused it. A refused funnel traces `add-text-refused \
             detail=…`; look for it. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- the oracle: TWO hard lines reached the engine ----------------------
    let n = added.get("n").and_then(|v| v.parse::<u32>().ok());
    match n {
        Some(2) => {
            report.note(format!(
                "★★★ two hard lines reached the engine: `{}`",
                added.raw
            ));
            Ok(None)
        }
        Some(1) => Ok(Some(format!(
            "★★★ THE COMMIT CARRIED ONE LINE, NOT TWO: `{}`.\n\
             The Enter was seen (`{}`) and the newline did not survive to the request. Three \
             places can eat it and all three have eaten it before:\n\
             * `caret::newline` vs `caret::insert` — `insert` FILTERS control characters, \
               correctly, and ate this exact keystroke for a whole driven run in August;\n\
             * `commit_into`, which must carry `draft.text` verbatim;\n\
             * `app::actions::addtext::request`, which must promote a multi-line point draft to \
               a boxed request — a point `AddTextRequest` REFUSES a newline by name, so a build \
               that skipped the promotion would more likely refuse than report n=1, but a build \
               that joined the lines with a space would land exactly here.\n\
             Trace: {}.",
            added.raw,
            enter.raw,
            session.trace_path().display()
        ))),
        other => Ok(Some(format!(
            "THE COMMIT REPORTED n={other:?}, which this check cannot interpret: `{}`. It \
             expects the hard-newline count that `app::actions::addtext` computes. If the \
             funnel's operand count has changed meaning, this oracle has to change with it. \
             Trace: {}.",
            added.raw,
            session.trace_path().display()
        ))),
    }
}
