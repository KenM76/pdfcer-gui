//! `text_edit_on_a_real_drawing` — **the operator's own report, driven**: arm
//! Edit text on a dense CAD sheet, click on text the engine says is there, and
//! find out what actually happens.
//!
//! # Why this exists when two text checks already pass
//!
//! `text_edit_pins_an_aligned_tail` and `add_text_takes_real_keystrokes` both
//! pass, and the operator's report on 2026-08-19 was *"text editing on canvas
//! still doesn't work"* — twice, weeks apart.
//!
//! **Both of those checks drive a fixture this repository generated.**
//! `tail-alignment.pdf` is 924 bytes with three lines of text placed by a
//! script, and `add_text` writes onto blank paper. Neither has ever touched the
//! documents the operator actually opens: SolidWorks-exported drawing sheets,
//! 1584 × 1224 pt, dense with vector geometry, where the text is small and
//! sparse and lives inside title blocks and tables.
//!
//! > **A feature verified only against the fixture that was written to verify
//! > it is verified against the author's model of the problem, not against the
//! > problem.**
//!
//! # ★ What this check is really asking
//!
//! Not *"does the caret work"* — the other two settle that. This asks the two
//! questions that separate a working feature from an operator's *"it doesn't
//! work"*:
//!
//! 1. **Does a click on real drawing text resolve a run at all?** The hit test
//!    runs against extracted page text on a page with tens of thousands of
//!    objects. `pdfcer find-text` gives the ground truth: aim at a rectangle
//!    the engine itself reports, and a miss is the application's, not the aim's.
//! 2. **When it declines, does the operator get told?** Every decline path here
//!    ends in `crate::text::textedit::refusal`, whose sentences are good, are
//!    tested, and were aimed at a status row that `R128` forbids growing.
//!    `Refusal::SpansRuns` is 47 words. **A decline nobody can read is
//!    indistinguishable from a feature that does nothing** — which is the
//!    operator's sentence, exactly.
//!
//! # ★★ The aim comes from the ENGINE, not from a guess
//!
//! `--doc-point` is required and there is deliberately no default, for the
//! reason `CheckContext::target` already records: *a click on empty page is
//! symptom-identical to a broken hit test*, and this project has already filed
//! and retracted one defect over that confusion.
//!
//! So the point is supplied by whoever runs it, and the honest way to obtain
//! one is to ask the engine where text is:
//!
//! ```text
//! pdfcer find-text --needle PART D:/Dev/temp/pdfcer/SW41177.pdf
//!   match page=1 text="PART" rect=1187.45,1178.37,1215.82,1191.21
//!
//! ui-verify … --doc-point 0,1201,1185 --check text_edit_on_a_real_drawing
//! ```
//!
//! A failure at a point sourced that way is the application's. A failure at a
//! guessed point is nobody's.

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
/// The tab the two text commands live on, as (region, id).
///
/// A pair rather than two constants, because `click_tab` takes one: a region
/// name and the id the shell reports for it are two spellings of one thing and
/// a check that let them drift would click one tab and assert about another.
const EDIT_TAB: (&str, &str) = ("ribbon.tab.edit", "edit");
/// The tab id the shell reports for [`EDIT_TAB`].
const EDIT_TAB_ID: &str = EDIT_TAB.1;
/// The command that arms the caret in Edit mode, as (region, id).
const EDIT_TEXT_ITEM: (&str, &str) = ("ribbon.item.edit.text", "edit.text");
/// The command id alone, for the messages.
const EDIT_TEXT: &str = EDIT_TEXT_ITEM.1;
/// `text-edit-tool tool=…` — the canvas reporting what armed.
const TOOL_EVENT: &str = "text-edit-tool";
/// The `Debug` spelling the canvas reports for the Edit variant.
const TOOL_EDIT: &str = "TextEdit(Edit)";
/// `text-edit-caret page=… run=… len=…` — a click resolved a run.
const CARET_EVENT: &str = "text-edit-caret";
/// `text-edit-declined reason=…` — a click did not.
/// `edit-text-target page=… run=… form=… invocations=… pages=…` — which content
/// stream the commit aimed at, and how many places paint it.
///
/// Raised by the `edit_text` apply arm from the engine's own `EditReport`. It is
/// the observable half of `Pass 119.0`: the prose disclosure goes to the status
/// row for the operator, and this goes to the channel for a check.
const TARGET_EVENT: &str = "edit-text-target";

/// How many following absolutely-placed `Tm`s one edit may reposition before
/// this check calls it a defect.
///
/// ★★ **This bound is a fact about the BUILD, not about the fixture.** Reflow
/// shifts *the rest of the line* by the advance delta; a line is a handful of
/// show operators in prose and often exactly one on a drawing. A number in the
/// hundreds means the scan did not find the end of the line and ran on into the
/// rest of the stream — which is true wherever it happens and on whatever
/// document.
///
/// The number that earned it, measured by the engine on this operator's own
/// benchmark drawing on 2026-08-20 (`Pass 121.1`):
///
/// | | `followers_repositioned` | changed pixels | bounding box |
/// |---|---|---|---|
/// | before the fix | **1,676** | 34,059 | x 62–858, y 34–795 — the whole page |
/// | after | small | **42** | x 542–561, y 378–384 — one label |
///
/// The cause: reflow walked forward shifting every absolute `Tm` until a
/// `Td`/`TD`/`T*` boundary, and **a CAD stream positions everything with `Tm`
/// and never emits `Td`** — so there was no boundary. One four-character edit
/// slid the rest of the drawing sideways.
///
/// 64 is deliberately generous: it is far above any real line and two orders of
/// magnitude below the failure. A bound tuned close to the observed-good value
/// would fail on the first document with a long justified line, and a check that
/// cries wolf gets disabled.
const MAX_FOLLOWERS: u64 = 64;
const DECLINED_EVENT: &str = "text-edit-declined";

/// See the module documentation.
pub struct TextEditOnARealDrawing;

impl Check for TextEditOnARealDrawing {
    fn name(&self) -> &'static str {
        "text_edit_on_a_real_drawing"
    }

    fn defect(&self) -> &'static str {
        "Edit text arms on a real CAD sheet and a click on text the engine reports as present \
         resolves no run — or it declines and the operator is never told why, which is \
         indistinguishable from a feature that does nothing and is the operator's own report"
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
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. The whole point of this check is that it drives the operator's OWN \
             documents rather than a fixture this repository generated to verify itself.",
        )
    })?;
    let Some(target) = ctx.target else {
        return Err(Error::new(
            "no --doc-point. There is deliberately no default: a click on empty page is \
             symptom-identical to a broken hit test, and this project has already filed and \
             retracted one defect over that confusion. Get a point from the engine — \
             `pdfcer find-text --needle PART <file>` prints a rectangle per hit — and pass \
             its centre as `--doc-point page,x,y` in PDF user space.",
        ));
    };
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a mode segment, a ribbon tab, a \
             ribbon control and the page. Reported as SKIPPED rather than passed: a check that \
             did not run has learned nothing.",
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
    report.note(format!(
        "fixture {} — page 1 is {} x {} pt, aiming at ({:.1}, {:.1}) on page {}",
        pdf.display(),
        page.width_pt,
        page.height_pt,
        target.x,
        target.y,
        target.page + 1
    ));

    let mut spec = LaunchSpec::new(&exe, ctx.out("text_edit_real.trace.txt"));
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
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit mode, so the Edit tab exists at all -----------------------
    //
    // ★ Worth stating because it is a candidate explanation for the operator's
    // report all by itself: in Review the Edit tab is not shown and `Ctrl+E` is
    // refused by the chord gate, so an operator marking up a drawing — which is
    // the mode marking up puts them in — genuinely cannot reach either text
    // tool without changing mode first. That is by design and it is also
    // exactly the kind of design an operator experiences as "it doesn't work".
    driving::click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: the Edit tab ---------------------------------------------------
    click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
    session.settle(14);
    if !driving::shell_trace(&session)?
        .events(driving::TAB_EVENT)
        .any(|l| l.get("tab") == Some(EDIT_TAB_ID))
    {
        return Ok(Some(
            "the click on the Edit tab produced no tab-selected line, so nothing below would \
             mean anything."
                .to_owned(),
        ));
    }

    // --- 3: arm the caret --------------------------------------------------
    click_command(&session, &driver, ui_rect, EDIT_TEXT_ITEM, 18)?;
    let trace = session.trace()?;
    if !trace
        .events(TOOL_EVENT)
        .any(|l| l.get("tool") == Some(TOOL_EDIT))
    {
        let declined = trace
            .events("command-declined")
            .filter(|l| l.get("id") == Some(EDIT_TEXT))
            .filter_map(|l| l.get("reason").map(str::to_owned))
            .last();
        return Ok(Some(match declined {
            Some(reason) => format!(
                "`{EDIT_TEXT}` was pressed and DECLINED with reason={reason}, in {MODE}. The \
                 mode gate refused a command the mode's own tab is showing."
            ),
            None => format!(
                "`{EDIT_TEXT}` was pressed and no `{TOOL_EVENT} tool={TOOL_EDIT}` followed. The \
                 ribbon control is drawn and reachable and the canvas tool did not arm."
            ),
        }));
    }
    report.note("Edit ▸ Edit text armed the caret on the real drawing");

    // --- 4: click where the ENGINE says there is text ----------------------
    let at = crate::checks::text_selection::aim(
        ctx,
        &session,
        page,
        DocPoint::new(target.page, target.x, target.y),
    )?;
    driver.click_at(at)?;
    session.settle(24);

    let trace = session.trace()?;
    let caret = trace.last(CARET_EVENT);
    let declined = trace
        .events(DECLINED_EVENT)
        .filter_map(|l| l.get("reason").map(str::to_owned))
        .last();

    // ★★ THE ASSERTION, and it is deliberately two-sided.
    //
    // A caret is a pass. A decline is a FAIL **with the reason quoted**, and
    // that is the shape this check has to have: the operator's report is not
    // "it crashes", it is "it doesn't work", and a decline at a point the
    // engine says carries text is exactly what "it doesn't work" looks like
    // from a chair.
    //
    // The reason matters more than the verdict. `NoRun` at a
    // `find-text` rectangle means the hit test and the extractor disagree about
    // where text is — an application defect. `SpansRuns` means the feature is
    // working and refusing, and the question moves to whether the operator can
    // READ the refusal. `NoText` on a page `find-text` just matched on would be
    // the strangest of the three.
    // ★★★ `InsideForm` WAS A SKIP FOR ONE DAY. IT IS NOW A FAILURE.
    //
    // On 2026-08-20 this branch reported SKIPPED with a long argument about why
    // an absent capability is neither a pass nor a defect —
    // `SHELL_FRAMEWORK.md` §5b's `CapabilityAbsent` applied to a driven check.
    // It ended:
    //
    // > *"The day the engine gains form editing, this branch stops being
    // > reached and the check goes green on its own — there is nothing to
    // > remember to delete."*
    //
    // That day was **the same day**. `Pass 119.0` landed form-XObject text
    // editing that evening, and the shell's refusal was deleted.
    //
    // ★ So the branch is kept and INVERTED, which is worth more than deleting
    // it: a build that reports this reason again has reinstated a guard that
    // refuses a caret on **99 % of the text on a CAD drawing** — the operator's
    // own estimate, and the reason that Pass jumped the queue. That is a
    // regression nobody would notice from a screenshot, because a refused caret
    // and an un-armed tool look identical, and it is precisely what a
    // reinstated `#[deprecated]` match arm would produce.
    if declined.as_deref() == Some("InsideForm") {
        return Ok(Some(format!(
            "★ THE CARET WAS REFUSED WITH `InsideForm`, WHICH CANNOT HAPPEN IN A CORRECT BUILD. \
             `Pass 119.0` (2026-08-20) made form-XObject text editable and \
             `Editability::InsideForm` is `#[deprecated]` and never returned by the engine. A \
             build reporting it has reinstated the shell-side guard that was deleted the same \
             day — see `canvas::textedit::Refusal`, whose `InsideForm` variant's replacement \
             carries the whole episode. On this document that guard refuses roughly 99 % of the \
             text the operator wants to edit: 1,696 show operators inside the form against 3,007 \
             metadata glyphs in the page stream. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let Some(line) = caret else {
        return Ok(Some(decline_message(&session, declined.as_deref())));
    };
    report.note(format!(
        "★ the click on real drawing text placed a caret: `{}`",
        line.raw
    ));

    // --- 5: ★★ the multi-run DISCLOSURE, which is the half the refusal was
    //           right about --------------------------------------------------
    //
    // A CAD table row is one show operator per cell, so the pieces beside the
    // one being edited are separate absolutely-positioned runs and they will
    // NOT move. The operator is looking at what appears to be one line. Rule 4:
    // an inference the operator cannot see owes an off-canvas report, and this
    // one is owed BEFORE they type rather than after.
    //
    // Not an unconditional assertion, because whether this particular point is
    // on a multi-run line is a fact about the fixture rather than about the
    // application — so the check reports which shape it met and asserts the
    // disclosure only in the shape that owes one. A check that demanded the
    // multi-run case would be asserting something about `SW41177.pdf`.
    let trace = session.trace()?;
    let shares = trace.last("text-edit-shares-line").is_some();
    if shares {
        report.note(
            "★★ the line is drawn as several separate pieces — the shape that refused every click until 2026-08-19 — and the caret landed anyway",
        );
        // The disclosure rides the same channel every edit note does, so its
        // presence is observable from outside the process only through the
        // trace line above; asserting the sentence's PIXELS would be the
        // stronger check and needs the Tool panel's region, which is the
        // follow-up named in the module header.
    } else {
        report.note(
            "this point is on a single-run line, so no multi-piece disclosure was owed. The refusal this check exists for is not exercised at this coordinate — aim at a table cell to reach it",
        );
    }

    // --- 6: type, and commit through the ONE path a commit takes ------------
    //
    // ★ Real keystrokes. `add_text_takes_real_keystrokes` established that the
    // OS → egui → draft link holds; what is unproven here is that it holds for
    // the EDIT variant on a run that already has text in it, where the draft
    // starts seeded rather than empty.
    for key in [vk::A, vk::D] {
        driver.press(key)?;
        session.settle(4);
    }
    session.settle(10);
    driver.press(vk::ENTER)?;
    session.settle(30);

    let trace = session.trace()?;

    // ★★ A REFUSAL IS ASKED ABOUT FIRST, AND THE REASON IS WHY.
    //
    // Until 2026-08-20 this check tested only for the ABSENCE of a commit line
    // and, on finding none, reported *"THE COMMIT NEVER REACHED THE ENGINE."*
    // On the operator's own drawing that sentence was **false**: the commit
    // reached the engine perfectly and the engine REFUSED it —
    //
    //   edit-text-refused page=0 n=1
    //     detail=text to edit ("p") was not found in an editable run on the page
    //
    // — and `edit-text-refused` is not `edit-text`, so the old condition
    // matched and produced a confident, specific, wrong diagnosis about the
    // shell. That cost an investigation into a call path that was working.
    //
    // The rule this encodes: **a check that asserts on the absence of a line
    // must first ask whether a DIFFERENT line explains the absence.** "I did
    // not see what I expected" and "I saw the opposite" are different findings,
    // and reporting the second as the first sends the reader to the wrong
    // half of the system. The refusal detail is quoted verbatim because the
    // engine's sentence is the whole of the diagnosis and any paraphrase here
    // would be a second account of it that could drift.
    // ★★★ A REFUSAL ON A SPLIT RUN IS THE PROGRAM WORKING, AND THIS CHECK CALLED
    // IT A DEFECT.
    //
    // `pdfcer-core` `Pass 152.0` lets an empty `find` beside a pin mean *"this
    // whole show operator"*, which is what a caret in a run actually means — and
    // the shell uses it **only** when `pin::spans_one_operator` says so. On a run
    // split across two operators the whole-operator form would replace one
    // fragment's text with the whole replacement and leave the other painting its
    // old glyphs: **visible corruption reported as success.** The engine measures
    // 13 % of runs as split. There, the shell deliberately sends the
    // reconstructed `find`, which on this operator's CAD drawings cannot match —
    // `text_extract` synthesises inter-glyph spacing, and a title-block cell came
    // back with twenty-one spaces in it — so the engine refuses **cleanly**,
    // which is the designed outcome and the safe one.
    //
    // ⇒ On the sweep of 2026-08-29 this check met exactly that case and reported
    // *"This is a `pdfcer-core` verdict and belongs in a request"*. It does not.
    // Nothing in the trace said which path had been taken, so neither the check
    // nor its reader could tell the two apart — `edit-text-pin` was added for
    // this, and it answered `one_operator=false find_len=30` on the very first
    // run.
    //
    // ★ Note the distinction the old note blurred: it said *"this point is on a
    // single-run line"*, and a **run** is not an **operator**. One run, two
    // operators, is exactly this case.
    if let Some(pin) = trace.last("edit-text-pin")
        && pin.get("one_operator") == Some("false")
        && trace.last("edit-text-refused").is_some()
    {
        return Err(Error::new(format!(
            "the caret landed on a run that spans MORE THAN ONE show operator (`{}`), so the \
             shell deliberately sent the reconstructed `find` rather than the whole-operator \
             form — the split case, where whole-operator would corrupt the page and \
             find-based fails cleanly instead. The refusal that followed is the DESIGNED \
             outcome, not a defect. ★ To exercise the commit, aim at a single-operator run: \
             `edit-text-pin … one_operator=true` is the tell, and the engine measures ~87 % of \
             runs as single-operator. SKIPPED rather than failed, because at this coordinate \
             this check has learned nothing about the half it exists to test.",
            pin.raw
        )));
    }
    if let Some(refused) = trace.last("edit-text-refused") {
        return Ok(Some(format!(
            "the caret took keystrokes, the commit REACHED the engine, and the engine refused \
             it: `{}`. The shell half of this works — a caret was placed on run {}, characters \
             were typed and the plan was built. ★ The split-run case is handled above and is \
             NOT this, so `edit-text-pin` said `one_operator=true` and the whole-operator form \
             was used: the engine refused a request that named a pin and an empty find, which \
             is a `pdfcer-core` verdict and belongs in a request. Trace: {}.",
            refused.raw,
            line.get("run").unwrap_or("?"),
            session.trace_path().display()
        )));
    }
    if trace.last("edit-text").is_none() && trace.last("text-edit-commit").is_none() {
        return Ok(Some(format!(
            "the caret took keystrokes and NO commit was raised at all. A caret was placed on \
             run {}, characters were typed and Enter was pressed, and no `edit-text`, \
             `text-edit-commit` or `edit-text-refused` line followed — so the shell built no \
             plan. Trace: {}.",
            line.get("run").unwrap_or("?"),
            session.trace_path().display()
        )));
    }
    report.note("★★ the edit reached the engine on the operator's own drawing");

    // --- 7: ★★★ AND IT EDITED THE BUFFER THE CARET MEASURED --------------
    //
    // The assertion that makes step 6 mean something on THIS document. On a CAD
    // sheet the text the operator clicks lives in a form XObject, and the shell
    // pins a byte span into that form's decoded bytes. `EditTarget::Auto` — the
    // engine's default — would offer that pin to the page's own stream first,
    // and this page's stream holds 3,007 single-character show operators, so a
    // stray match there is a dense field of near-misses rather than a
    // theoretical collision. The result would be an edit that succeeded on the
    // wrong glyph, silently.
    //
    // `canvas::textedit::plan` therefore names the target from the same
    // provenance record it takes the pin from — the two fields are one fact —
    // and `edit-text-target` is that decision, observable.
    //
    // ★ What is asserted is that the line EXISTS and carries a form, not a
    // particular object number. Which object holds the title block is a fact
    // about the fixture; that the edit went into a form at all, on a document
    // whose editable text is inside one, is a fact about the build. A
    // `form=none` here would mean the target collapsed to the page stream — the
    // exact regression the explicit target exists to prevent.
    let Some(target) = trace.last(TARGET_EVENT) else {
        return Ok(Some(format!(
            "the edit committed and no `{TARGET_EVENT}` line followed, so the shell did not \
             report which content stream it aimed at. That line is raised from the `edit_text` \
             arm's report; its absence means either the arm reverted or the engine's report no \
             longer carries `form_object`. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★ the edit named its buffer: `{}`", target.raw));

    // --- 8: ★★★ AND IT DID NOT MOVE THE REST OF THE DRAWING ---------------
    //
    // The assertion that would have caught `Pass 121.1` before the operator
    // did — and the engine's own request when it shipped the fix:
    //
    // > *"Surface `followers_repositioned`. It is the cheapest tell that a
    // > reflow over-reached: on absolutely-placed content it should be `0`, and
    // > a large number means the edited 'line' ran further than the line. If
    // > you show one number from an edit report beyond the disclosures, make it
    // > that one."*
    //
    // ★ Note what this is NOT: it is not a pixel oracle, and the standing rule
    // says a trace cannot tell you the screen changed. It does not need to. The
    // claim being checked is about the SCOPE of a rewrite — how many operators
    // in the content stream were touched — and that is a number the engine
    // measured and this shell forwards. A screenshot would show the damage
    // without naming its size, and the size is what distinguishes "reflow
    // worked" from "reflow ran to the end of the stream".
    //
    // A missing field parses to `MAX_FOLLOWERS + 1` rather than 0, so a build
    // that stopped reporting fails here instead of passing silently. The safe
    // direction: a check that could not read the number must not report that
    // the number was fine.
    let followers: u64 = target
        .get("followers")
        .and_then(|v| v.parse().ok())
        .unwrap_or(MAX_FOLLOWERS + 1);
    if followers > MAX_FOLLOWERS {
        let shot = ctx.out("text-edit-followers.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ ONE EDIT REPOSITIONED {followers} FOLLOWING OPERATORS. A line is a handful of \
             show operators in prose and often exactly one on a drawing; a number in the \
             hundreds means the reflow scan never found the end of the line and ran on into the \
             rest of the stream.\n\
             That is `Pass 121.1` (engine, 2026-08-20): a CAD stream positions everything with \
             `Tm` and never emits the `Td`/`TD`/`T*` the scan was looking for as a boundary. One \
             four-character edit moved 1,676 labels and changed 34,059 pixels across the whole \
             sheet. If this fires, either the engine has regressed or this build links a \
             revision older than `bab0a23`.\n\
             A capture is attached: compare it against the page before the edit. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★ the edit stayed inside its own line: {followers} following operator(s) \
         repositioned, against 1,676 on the same drawing before the engine's `Pass 121.1`"
    ));
    // ★★ …and the shared-content fan-out, REPORTED rather than asserted.
    //
    // A form XObject may legally be painted from several pages, so an edit
    // inside one can change every sheet it appears on — and the engine puts a
    // "SHARED CONTENT: …" sentence in `report.disclosures` for exactly that,
    // which this shell already carries to the status row.
    //
    // Whether THIS fixture's title block is shared is a fact about the fixture.
    // Asserting a number here would make the check pass or fail on which
    // drawing it was pointed at, which is the distinction this harness exists
    // to keep. So it is recorded for a human to read, and the count is what
    // makes the record worth having.
    if let Some(n) = target.get("invocations") {
        report.note(format!(
            "the edited stream is painted in {n} place(s) in the document — if that is more than \
             one, the operator changed every one of them, and the engine's SHARED CONTENT \
             disclosure is on the status row saying so"
        ));
    }
    Ok(None)
}

/// The failure message for a click that placed no caret.
///
/// Split out so the happy path reads as a sequence rather than as a `match`
/// whose arms are 20 lines apart. Its content is the diagnosis, and the three
/// reasons mean genuinely different things — see the call site's own comment.
fn decline_message(session: &Session, declined: Option<&str>) -> String {
    let trace = session.trace_path().display().to_string();
    match declined {
        Some(reason) => {
            let mut out = format!(
                "★ THE CLICK WAS DECLINED: reason={reason}, at a point the engine reports as \
                 carrying text.\n"
            );
            out.push_str(
                "· `NoRun` means the caret HIT TEST and the TEXT EXTRACTOR disagree about where \
                 text is on this page. That is an application defect, and it is the operator's \
                 report: he clicks on words he can see and nothing happens.\n",
            );
            out.push_str(
                "· `NoText` on a page the engine just matched text on would mean the shell's own \
                 `page_text()` returns nothing where `find-text` returns hits.\n",
            );
            out.push_str(
                "· ★★ `SpansRuns` cannot appear here any more. It was REMOVED on 2026-08-19 \
                 because it refused nearly every click on a CAD sheet. If it is back, the \
                 multi-run refusal has been reinstated and this operator's documents are \
                 uneditable again.\n",
            );
            out.push_str(&format!("Trace: {trace}."));
            out
        }
        None => format!(
            "THE CLICK PRODUCED NEITHER A CARET NOR A DECLINE. The tool armed and the press \
             reached no `canvas::textedit::click` at all, so the routing dropped it before the \
             hit test. Look at `gesture::press_kind`'s caret rung and `canvas::interact`'s click \
             routing. ★ This is the worst of the three outcomes: it is silent on both channels, \
             which is precisely what the operator described. Trace: {trace}."
        ),
    }
}
