//! # `typo_refusal` — **his spelling mistake, corrected**, driven on the file he
//! reported it on
//!
//! `OPERATOR_REQUESTS.md` **O140** and **O142**. The operator, 2026-09-05:
//!
//! > *"on page 2 there is a spelling mistake — clien instead of client. if I try
//! > to edit the edit is not accepted. **the lines I added below `price)` are
//! > editable, but everything else that existed when I got the pdf is not.**"*
//!
//! Two sentences, and the second is a complete diagnosis he made himself. This
//! check drives both halves of it in **one launch**.
//!
//! ## ★★★ THIS CHECK CHANGED SUBJECT ON 2026-09-06, AND THAT IS THE FIRST
//! THING TO KNOW ABOUT IT
//!
//! It used to be `a_refused_typo_fix_says_why_it_was_refused`, and it asserted
//! that the commit was **refused** and that the refusal was disclosed and
//! correctly categorised. That was the honest thing to assert while it was true.
//! It is not true any more, and a check left asserting it would be describing a
//! program that no longer exists — worse, it would go on **passing** on a build
//! where the fix had been reverted.
//!
//! ### What was actually wrong, which is not what it looked like
//!
//! His producer writes **one glyph per show operator** — a thirty-six character
//! line is thirty-six `Tj`s on one row, stepped by x-only `Td`s. `Pass 256.0`
//! taught `edit_text` to match a `find` across exactly that shape and the engine
//! measured `"clien"->"client"` on his own file, `operators_spanned=5`.
//! **That capability was in this shell's pin and his typo still failed.**
//!
//! The standing diagnosis was that the shell sends the run's whole text as
//! `find` and that extraction synthesises the spaces inside it, so the string
//! named characters no operator wrote. ⇒ **Measured on his file, that is false.**
//! One `EditSession` per shape, page 2, the run he reported:
//!
//! | request | result |
//! |---|---|
//! | whole-run `find` **+ pin** — what this shell sent | `NotFound` |
//! | whole-run `find`, **no pin** | **OK**, `operators_spanned=36` |
//! | `"clien"` **+ pin** | `NotFound` |
//! | `"clien"`, **no pin** | **OK**, `operators_spanned=5` |
//!
//! Thirty-six characters, thirty-six operators: **the spaces are in the
//! operators**, and the whole-run `find` matches perfectly once the pin is off.
//! The pin was the defect. `Pass 256.0`'s contract says *"a pinned request never
//! spans"*, so a `find` sent beside a pin is confined to the one operator the
//! pin names — which on his line holds a single character. The engine was
//! answering the question it was asked, correctly, every time.
//!
//! ## ★★★ What is asserted, and why it is TWO gestures and not one
//!
//! | gesture | what must happen |
//! |---|---|
//! | correct the typo in text the document arrived with | the commit **lands**, `occurrences=1 pinned=false` on the plan's own line, and **no `⊗` slot draws** |
//! | commit text pdfcer itself wrote | the edit lands **and no `⊗` slot draws after it** |
//!
//! The second row is the whole reason this file is long. The oracle for the
//! decline half is *"a region was published"*, and a probe whose baseline has no
//! dynamic range **cannot produce a verdict**. It is the contrast the operator
//! noticed — text pdfcer authored commits, text that arrived does not — so the
//! check's two rows and his two sentences are the same two facts. ★ Since the
//! inversion **both** rows now succeed, which is the point: his complaint was
//! that they differed.
//!
//! ## ★★★ THE ASSERTION THAT CARRIES THE VERDICT IS NOT "THE EDIT LANDED"
//!
//! It is `edit-text-pin … occurrences=1 pinned=false`, and the distinction is
//! the difference between a working program and a dangerous one.
//!
//! The pin is the **only** disambiguator `EditRequest` carries — there is no
//! occurrence index on it — so dropping it hands the choice of *which*
//! occurrence to edit to the engine's left-to-right scan. On a page holding the
//! same words twice that silently corrects whichever it reaches first. **The
//! document this was reported against is a signed quotation**: a wrong edit
//! there is not a defect he reports, it is one he finds later in a file he has
//! already sent.
//!
//! ⇒ So a build that dropped the pin **unconditionally** would land this edit,
//! satisfy a naive assertion for ever, and be exactly the build that must never
//! ship. The plan's own line is what tells the two apart, and this check reads
//! it. `canvas::textedit::Plan::occurrences` carries the reasoning;
//! `canvas::textedit::glyphwall` holds it as unit tests over two authored
//! fixtures — one where the run is unique and the edit must land, one where it
//! appears twice and the edit must be refused **by name**.
//!
//! ## The oracle, and its one honest weakness
//!
//! `status-group:decline` is a `ui-rect` region published on the frame it
//! draws. The harness cannot read rendered text — there is no accessibility
//! reader and no OCR — so this check asserts that the slot **did not draw**, not
//! what it would have said. The wording is held by unit tests in
//! `app::status::decline` and `text::textedit`, and by `check-ui-strings.sh`.
//!
//! ★ Ordering is load-bearing and is asserted by `lineno`. A whole-capture
//! `last(...)` is a fossil finder; every region read here is anchored to a cause
//! that must precede it — the successful commit for both arms.
//!
//! ## Aim
//!
//! `--doc-point PAGE,X,Y` in PDF user space, on a run the document arrived
//! with. For the operator's own file:
//!
//! ```text
//! --pdf "…/apartment work - signed.pdf" --doc-point 1,200.4,537.1
//! ```
//!
//! — the centre of *"Final quality walkthrough with clien"*, whose box
//! `extract-text --pages 2 --json` reports as `[33.47, 526.22, 367.26, 547.90]`.
//! ★ `PAGE` is **0-based**; his page 2 is `1`.
//!
//! ⚠ **Copy his file to scratch and drive the copy.** The edit under test writes
//! to the document; never point this at OneDrive.
//!
//! ## ★★ Why the caret is expected to be OFFERED, not withheld
//!
//! R9 argues that a control which fails on press is worse than one that is not
//! there, and the obvious reading of this defect is *"do not offer a caret on
//! text pdfcer cannot edit"*. **That reading was tested and is false**, and the
//! falsification is worth carrying here because it is the reason this check
//! asserts a caret rather than the absence of one.
//!
//! The tempting forecast is *"the run's font is `Identity-H`, so refuse"*.
//! `pdfcer-core`'s own fixture `fixtures/synthetic/text/composite-editable.pdf`
//! is `Type0/CIDFontType2`, `Identity-H`, `verdict=blocked-identity` — the
//! **same** `list-fonts` verdict as every uneditable face in the operator's
//! document — and it edits end to end, asserted by
//! `composite_refusal_reachable.rs::an_invertible_composite_run_is_editable_end_to_end`.
//! `Identity-H` is *necessary* for this refusal and nowhere near *sufficient*:
//! `Pass 29.0` made composite runs editable whenever their `/ToUnicode`
//! inverts.
//!
//! ⇒ A shell that withheld the caret on `Identity-H` would refuse editing on
//! text pdfcer can edit, silently, on every document — the exact failure
//! `Editability` was made an enum rather than a `bool` to prevent, and one this
//! project has already committed once (`Refusal::InsideForm`, whose whole
//! episode is written at `canvas::textedit::Refusal`). **The caret stays. The
//! silence goes.**
//!
//! The negative control needs no aim of its own: it arms **Add text** and
//! clicks the very same coordinate, which `place::click`'s `Add` arm turns into
//! an origin draft without consulting the hit test at all. `add_text` then
//! writes the engine's bundled Helvetica — which is exactly what the operator's
//! own editable lines are made of, so the control is his contrast rather than a
//! stand-in for it.
//!
//! ## ⚠ A defect this check found while looking for somewhere to put that
//! control
//!
//! `pdfcer-core`'s `EditableTextModel::hit_test` ends with *"fall back to the
//! nearest line by baseline distance"* and applies **no distance bound**. So on
//! a page carrying any text at all, every point resolves to a run:
//! `canvas::textedit::Refusal::NoRun` is unreachable, and with it
//! `place::click`'s *"a click that names no run starts a new one"* — the
//! 2026-08-19 answer to the operator's *"How do I make new text when I click on
//! the canvas and expect to edit there?"* Driven here: clicks 215 pt to the
//! right of a run's box and 99 pt below it both resolved that same run. Filed
//! rather than worked around.

use crate::checks::driving::{INVOKE_EVENT, SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// Reset the dock, take Edit mode, arm the **edit** caret — one per frame.
///
/// ★ `view.reset_layout` first, and it is not decoration. The application
/// persists its dock layout across runs and the harness does not clear it, so a
/// launch inherits whatever the previous launch left — including a previous
/// *driven* launch. A check that reads a docked region without resetting is
/// reading the last run's furniture; this project has filed one such report.
/// The reset's arrival is asserted below rather than assumed.
const INVOKE: &str = "view.reset_layout,mode.edit,edit.text";

/// The characters seeded into the draft. One letter, because the operator's
/// correction was one letter: `clien` → `clien`**`t`**.
///
/// ★ Seeded rather than typed, for `enter_newline`'s reason: `sys::vk` is a
/// deliberately closed list of non-character virtual keys and this machine
/// cannot inject an arbitrary character. The keystroke is not the subject here
/// — the refusal after the commit is — so a seam that puts the letter in the
/// draft costs this check nothing it was measuring.
const SEED: &str = "t";

/// `layout-reset scope=… changed=…` — the application's own report that the
/// dock went back to its default arrangement.
///
/// ★ Note `changed=false` is a perfectly good answer: it means the layout was
/// already default. What matters is that the reset **ran**, not that it moved
/// anything, so this is keyed on the line's presence and not on its field.
const RESET_EVENT: &str = "layout-reset";
/// `text-edit-caret kind=… page=… run=… len=…` — a click opened a draft.
const CARET_EVENT: &str = "text-edit-caret";
/// `text-edit-declined reason=…` — a click did not.
const DECLINED_EVENT: &str = "text-edit-declined";
/// `edit-text-refused page=… n=… detail=… kind=…` — the funnel's error arm.
const REFUSED_EVENT: &str = "edit-text-refused";
/// `edit-text page=… n=…` — the funnel's SUCCESS arm, and since 2026-09-06 the
/// line this check's positive assertion rests on.
///
/// ★ The bare verb name, not `edit-text-applied`: `vector_edit` names its
/// success line after the verb it was given, and this check is aimed at
/// `"edit-text"`. Spelling it `edit-text-applied` here would look right, find
/// nothing, and report a correct build as one whose edit never reached the
/// engine — the failure mode `RESUME.md` records as *"ask what the check
/// SAMPLED before asking what is broken"*.
const APPLIED_EVENT: &str = "edit-text";
/// `add-text page=… n=…` — the funnel's success arm for new page text.
const ADD_EVENT: &str = "add-text";
/// The `⊗` slot in the status bar. `app::status::decline` draws into it, and it
/// is published as a `ui-rect` on the frame it draws.
const DECLINE_REGION: &str = "status-group:decline";
/// The page's own region, so a failure can say whether a sheet was on screen.
const PAGE_REGION: &str = "page";
/// The canvas viewport, over which the wheel is rolled to reach the aim page.
const CANVAS_REGION: &str = "canvas-viewport";
/// The Edit ribbon tab, whose id the tab-click helper takes bare.
const EDIT_TAB: &str = "edit";
/// `edit.add_text` — the second door into the text tool, and the one the
/// negative control uses. It bypasses `resolve_run` entirely.
const ADD_TEXT: &str = "edit.add_text";
/// Wheel notches per scroll step while hunting for the aim page.
const NOTCHES: i32 = 3;
/// How many scroll steps before giving up on reaching the aim page.
///
/// Generous: a three-page letter document needs two or three steps, and a
/// check that gave up early would report "the page could not be reached" about
/// a document it simply had not finished scrolling.
const MAX_SCROLL_STEPS: usize = 40;

/// See the module documentation.
pub struct HisTypoCanBeCorrectedOnHisOwnFile;

impl Check for HisTypoCanBeCorrectedOnHisOwnFile {
    fn name(&self) -> &'static str {
        "his_typo_can_be_corrected_on_his_own_file"
    }

    fn defect(&self) -> &'static str {
        "a spelling mistake in text the document arrived with cannot be corrected — the commit \
         is refused — while text pdfcer itself added edits fine, so from his chair the program \
         silently ignores corrections to exactly the words he wants to correct"
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
            "input is disabled (--no-input). This check clicks page text, commits an edit and \
             then clicks bare paper for the negative control. Reported as SKIPPED rather than \
             passed: a check that did not run has learned nothing.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx.pdf.clone().ok_or_else(|| {
        Error::new(
            "no --pdf. This check exists because a refusal on the operator's OWN document was \
             silent, and a fixture this repository generated to verify itself cannot carry that \
             — the whole subject is a font that arrived in a document somebody else produced.",
        )
    })?;
    let Some(target) = ctx.target else {
        return Err(Error::new(
            "no --doc-point. There is deliberately no default: a click on empty page is \
             symptom-identical to a refused caret. Get a rectangle from the engine — \
             `pdfcer extract-text --pages N --json <file>` prints one per run — and pass its \
             centre as `--doc-point PAGE,X,Y` in PDF user space. ★ PAGE is 0-BASED.",
        ));
    };
    let ui_rect = ctx.profile.vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot say \
             where its controls are and there is no decline slot to observe.",
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
        "fixture {} — page 1 is {} x {} pt; aiming at ({:.1}, {:.1}) on page {} (1-based)",
        pdf.display(),
        page.width_pt,
        page.height_pt,
        target.x,
        target.y,
        target.page + 1
    ));

    let mut spec = LaunchSpec::new(&exe, ctx.out("typo-refusal.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_TYPE".to_owned(), SEED.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {} with PDFCER_DIAG_INVOKE={INVOKE}",
        exe.display(),
        session.pid()
    ));
    session.settle(50);
    let driver = Driver::new(session.window());

    // --- 0: the dock reset LANDED ------------------------------------------
    //
    // Asserted, not assumed. `PDFCER_DIAG_INVOKE` rings one command per frame
    // and a command the registry refuses is traced as `command-declined`, not
    // as an error — so a reset that never happened is indistinguishable from a
    // reset that did, unless somebody looks.
    let trace = session.trace()?;
    let Some(reset) = trace.events(RESET_EVENT).last() else {
        return Err(Error::new(format!(
            "`view.reset_layout` was requested through `PDFCER_DIAG_INVOKE` and no \
             `{RESET_EVENT}` line followed. The dock therefore holds whatever the previous \
             launch persisted, which for a previous DRIVEN launch is not a default at all. \
             Refusing to continue rather than reading last run's furniture.\n\
             ★ `{INVOKE_EVENT}` is the SHELL's line for a ribbon click and is deliberately not \
             what is asserted here: `PDFCER_DIAG_INVOKE` dispatches straight into \
             `dispatch_command` and never touches the ribbon, so a check keyed on it would fail \
             on a build whose reset works perfectly. What is asserted is the application's own \
             report that the layout was reset. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the dock layout was reset, and the application said so: `{}`",
        reset.raw
    ));

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "the application declared no `{PAGE_REGION}` region, so no sheet is on screen and \
             there is nowhere to put a caret. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- 1: bring the aim page onto the canvas ------------------------------
    //
    // ★ `text_selection::aim` REFUSES to convert a point whose page is not the
    // page on screen, and it is right to: mapping page 2's coordinates through
    // page 1's rect yields a click that is plausible, precise and in the wrong
    // place. The operator's typo is on his page 2, so the check has to get
    // there before it can aim, and the wheel is the only door that needs no
    // panel.
    scroll_to_page(ctx, &session, &driver, ui_rect, target.page, report)?;

    // --- 2: the positive case — edit text the document arrived with ---------
    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(target.page, target.x, target.y),
    )?;
    driver.click_at(at)?;
    session.settle(26);

    let trace = session.trace()?;
    let caret = trace.events(CARET_EVENT).last().map(|l| l.raw.clone());
    let declined_at_click = trace
        .events(DECLINED_EVENT)
        .filter_map(|l| l.get("reason").map(str::to_owned))
        .last();

    // ★★★ A CARET IS EXPECTED. See the module header's "Why the caret is
    // OFFERED" section: the tempting `Identity-H` forecast is falsified by the
    // engine's own fixture, so a build that withholds the caret here has
    // reinstated a guard that refuses editing on text pdfcer can edit.
    let Some(caret) = caret else {
        return Ok(Some(match declined_at_click {
            Some(reason) => format!(
                "★ THE CARET WAS WITHHELD: `{DECLINED_EVENT} reason={reason}`, at a point the \
                 engine's own extraction reports as carrying text.\n\
                 If `reason` names a font or an encoding, a forecast has been added that this \
                 check's header falsifies: `Identity-H` does NOT imply uneditable — \
                 `pdfcer-core`'s `composite-editable.pdf` carries the identical \
                 `verdict=blocked-identity` and edits end to end. A caret withheld on that \
                 predicate refuses editing document-wide on text pdfcer can edit, silently.\n\
                 If it names `NoRun`, the caret hit test and the text extractor disagree about \
                 where text is on this page. Trace: {}.",
                session.trace_path().display()
            ),
            None => format!(
                "THE CLICK PRODUCED NEITHER A CARET NOR A DECLINE — no `{CARET_EVENT}`, no \
                 `{DECLINED_EVENT}`. Either `edit.text` never armed (look for \
                 `command-declined id=edit.text`) or the press was dropped before the hit test. \
                 ★ This is the worst of the outcomes: silent on both channels, which is exactly \
                 what the operator described. Trace: {}.",
                session.trace_path().display()
            ),
        }));
    };
    report.note(format!("★ the click placed a caret: `{caret}`"));

    driver.press_chord(&[vk::CONTROL], vk::ENTER)?;
    session.settle(34);

    let trace = session.trace()?;

    // ═══════════════════════════════════════════════════════════════════════
    // ★★★ 2: THE CORRECTION MUST LAND. This arm was INVERTED on 2026-09-06,
    //        and the inversion is the whole subject of this file now.
    // ═══════════════════════════════════════════════════════════════════════
    //
    // Until today this check required the commit to be **refused**, and then
    // asserted that the refusal was disclosed and correctly categorised. That
    // was the honest thing to assert while it was true: the shell sent the run's
    // whole text as `find` **beside a provenance pin**, and `Pass 256.0`'s
    // contract says *"a pinned request never spans"* — so the request was
    // confined to the one show operator the pin named, which on his line holds a
    // single character, and a thirty-six character `find` could not match inside
    // it. He reported it as *"if I try to edit the edit is not accepted."*
    //
    // The shell now drops the pin when the run spans operators **and** the text
    // occurs exactly once on the page (`canvas::textedit::Plan::occurrences`),
    // which lets the engine's cross-operator matcher reach it. So the refusal is
    // gone and this arm asserts the correction instead.
    //
    // ⚠ **A check that still accepted the refusal would be describing a program
    // that no longer exists**, and worse: it would go on passing on a build
    // where the fix had been reverted, which is the single regression this file
    // is now the only driven instrument for.
    if let Some(refused) = trace.events(REFUSED_EVENT).last() {
        let refusal_raw = refused.raw.clone();
        let shot = ctx.out("typo-refusal-still-refused.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        let pin = trace.events("edit-text-pin").last().map_or_else(
            || "— no `edit-text-pin` line at all".to_owned(),
            |l| format!("`{}`", l.raw),
        );
        return Ok(Some(format!(
            "★★★ HIS TYPO STILL CANNOT BE CORRECTED. The caret was placed on the run he \
             reported, `Ctrl+Enter` committed, and the engine refused: `{refusal_raw}`.\n\
             ★ READ THE PLAN'S OWN LINE FIRST — it says which half is wrong: {pin}.\n\
             · `pinned=true` with `one_operator=false` is the defect he reported, returned. \
             `Pass 256.0`: *a pinned request never spans*, so a `find` sent beside a pin is \
             confined to one show operator, and his producer writes one glyph per operator. \
             `canvas::textedit::plan` must drop the pin when `occurrences == 1`.\n\
             · `occurrences=` greater than 1 is the AMBIGUITY GUARD firing, and that is \
             correct behaviour on a page holding the same words twice — but not on this one. \
             Re-aim, or check `canvas::textedit::page_occurrences`.\n\
             · no `edit-text-pin` line means the plan read no provenance at all, so nothing \
             was measured and the pin was never even a decision.\n\
             ★★ `canvas::textedit::glyphwall` holds this as unit tests over two authored \
             fixtures; run it first, because if it is GREEN the planner is fine and the \
             problem is between the caret and the plan. Trace: {}.",
            session.trace_path().display()
        )));
    }

    let Some(applied) = trace.events(APPLIED_EVENT).last() else {
        // Neither applied nor refused. Not this check's subject and not a
        // verdict about the build's editing: the gesture never reached the
        // engine at all.
        return Err(Error::new(format!(
            "the commit produced NEITHER `{APPLIED_EVENT}` nor `{REFUSED_EVENT}`, so nothing \
             reached the engine and there is no edit to judge. Either the caret was abandoned \
             before `Ctrl+Enter` or the chord did not arrive. This is reported as SKIPPED \
             rather than failed because a check that did not run has learned nothing. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    let applied_at = applied.lineno;
    let applied_raw = applied.raw.clone();
    report.note(format!(
        "★★★ THE CORRECTION LANDED ON HIS OWN FILE: `{applied_raw}` — the run he reported, in \
         text the document arrived with, committed from a caret in one gesture"
    ));

    // ★★★ THE DECISION THAT MADE IT POSSIBLE, asserted separately from the
    // outcome — because an edit that landed for the WRONG reason is a build
    // waiting to edit the wrong occurrence on a page that has two.
    //
    // A build that dropped the pin unconditionally passes the assertion above
    // for ever and is exactly the dangerous one: on a page holding the same
    // words twice it would silently correct whichever the engine reached first,
    // on a signed quotation. So the plan's own line must show that the pin came
    // off *because the text was counted and found unique*, not by default.
    let pin_line = trace.events("edit-text-pin").last();
    let counted = pin_line
        .and_then(|l| l.get("occurrences"))
        .map(str::to_owned);
    let pinned = pin_line.and_then(|l| l.get("pinned")).map(str::to_owned);
    match (counted.as_deref(), pinned.as_deref()) {
        (Some("1"), Some("false")) => {
            report.note(
                "★★★ and it landed for the RIGHT reason: `edit-text-pin` reports \
                 `occurrences=1 pinned=false` — the run spans show operators, the text occurs \
                 once on the page, and the pin was dropped BECAUSE it was counted unique. On a \
                 page with two candidates the same code keeps the pin and refuses",
            );
        }
        (Some(n), Some(p)) => {
            return Ok(Some(format!(
                "★★★ THE EDIT LANDED BUT THE GUARD DID NOT DECIDE IT. `edit-text-pin` reports \
                 `occurrences={n} pinned={p}`, and the only combination that licenses an \
                 unpinned request is `occurrences=1 pinned=false`.\n\
                 ⚠ `pinned=false` with any other count is the dangerous build: the pin is the \
                 ONLY disambiguator `EditRequest` carries — there is no occurrence index — so \
                 dropping it on a page holding the same words twice hands the choice to the \
                 engine's scan order. This document is a signed quotation. A wrong edit here \
                 is one he finds later, in a file he has already sent.\n\
                 See `canvas::textedit::Plan::occurrences` and the guard test \
                 `glyphwall::a_typo_that_appears_twice_on_the_page_is_refused_rather_than_guessed`. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        }
        _ => {
            return Ok(Some(format!(
                "★★ THE EDIT LANDED AND THE PLAN SAID NOTHING ABOUT WHY. No `edit-text-pin` \
                 line carrying both `occurrences=` and `pinned=`, so this check cannot tell a \
                 build that counted the occurrences from one that drops the pin \
                 unconditionally — and those two are a working program and a silent \
                 wrong-edit waiting to happen. The trace field is the whole instrument here; \
                 `canvas::textedit::plan` writes it. Trace: {}.",
                session.trace_path().display()
            )));
        }
    }

    // ★★ AND THE OPERATOR IS NOT TOLD IT FAILED. A build that applied the edit
    // and left a decline in the slot would be reporting a failure over a
    // document that is fine, which is its own defect — and this is the region
    // the check's previous incarnation existed to see drawn.
    if trace.lines.iter().any(|r| {
        r.event == "ui-rect" && r.lineno > applied_at && r.get("name") == Some(DECLINE_REGION)
    }) {
        let shot = ctx.out("typo-declined-a-success.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★★ THE CORRECTION LANDED AND THE STATUS BAR STILL DECLINED IT. `{applied_raw}` \
             was followed by a `{DECLINE_REGION}` region. Either the slot is not retiring a \
             stale sentence or a success is being recorded as a refusal; from his chair both \
             read as the program telling him his correction did not take, over a document \
             where it did. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(
        "★★ and nothing declined it: no `⊗` slot drew on any frame after the commit, so the \
         program does not report a failure over a document it just corrected",
    );

    // --- 3: the NEGATIVE CONTROL — the contrast the operator noticed --------
    //
    // ★★★ Without this the check has no dynamic range. The oracle above is
    // "a region was published", and a build that published it unconditionally
    // — or that never retires a stale one — satisfies the positive arm
    // permanently. What has to be shown is that the SAME instrument is silent
    // when the same gesture succeeds.
    //
    // ★★★ IT ARMS **ADD TEXT** AND CLICKS THE SAME POINT, and three cheaper
    // ideas were driven and discarded first. All three were trying to find
    // *bare paper* for the `edit.text` tool to turn into a new-text draft.
    //
    // 1. A fixed point on the sheet — the far corner, on the reasoning that a
    //    page of prose leaves its foot empty. **Off the window**: this check
    //    scrolls to reach the aim page, so what is on screen is a band of that
    //    page. The run said `the point (1386, 945) is OUTSIDE the application's
    //    window, which is 1100x800`.
    // 2. The aim's own `y`, stepped 250 pt sideways, then to the right margin.
    //    Both resolved **the same run** — `run=12` — from 215 pt past the end
    //    of its bounding box.
    // 3. Vertical offsets of ±55 and ±110 pt. All four resolved runs, one of
    //    them `run=12` again from 99 pt below its box.
    //
    // ★★★ The cause of (2) and (3) is in `pdfcer-core`'s
    // `EditableTextModel::hit_test` and it is worth knowing far beyond this
    // check: its last clause is *"fall back to the nearest line by baseline
    // distance"* with **no distance bound at all**. So on a page carrying any
    // text, every point resolves to a run, `Refusal::NoRun` is unreachable, and
    // `textedit::click`'s *"a click that names no run starts a new one"* — the
    // 2026-08-19 answer to the operator's *"How do I make new text when I click
    // on the canvas?"* — cannot fire. Filed; see the module header.
    //
    // ⇒ `TextEditKind::Add` never asks the hit test: `place::click`'s `Add` arm
    // builds an `Anchor::Origin` from the click point directly. So the control
    // arms Add text from the ribbon and clicks **the same coordinate the
    // subject used**, which is a better baseline than bare paper ever was: same
    // point, same page, same process, same instrument — one gesture refused and
    // explained, the other accepted and silent.
    //
    // ★★★ NO ESCAPE BEFORE IT, and the first version of this check pressed one.
    //
    // The reasoning was that clicking elsewhere with a live draft COMMITS it
    // (`textedit::click`'s `commit_into`), which would raise a second refusal
    // and make the "control" a second copy of the subject. Sound — and the
    // draft is **already gone**: `Ctrl+Enter` committed it, and the trace shows
    // `text-edit-abandon` immediately before `edit-text-refused`. So the Escape
    // had nothing to discard and did the next thing on its ladder instead:
    // `canvas-escape outcome=DisarmedTool`. The click that followed found no
    // armed text tool, opened no draft, and the check reported its own baseline
    // as unreachable.
    //
    // ⇒ **A key with a ladder does the rung it reaches, not the rung you meant.**
    // `canvas-escape` publishes which one, which is how this was five minutes
    // rather than an afternoon.
    crate::checks::ocr::click_tab(&session, &driver, ui_rect, EDIT_TAB)?;
    session.settle(12);
    crate::checks::ocr::click_command(&session, &driver, ui_rect, ADD_TEXT)?;
    session.settle(14);

    let at = aim(
        ctx,
        &session,
        page,
        DocPoint::new(target.page, target.x, target.y),
    )?;
    driver.click_at(at)?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(origin) = trace
        .events(CARET_EVENT)
        .filter(|l| l.lineno > applied_at && l.get("origin").is_some())
        .last()
    else {
        return Err(Error::new(format!(
            "the negative control opened no NEW-TEXT draft: no `{CARET_EVENT} … origin=…` after \
             the refusal, with `{ADD_TEXT}` armed from the ribbon and the click at the same \
             point the subject used. Without a successful edit through the same instrument this \
             check has measured a region with no baseline, and its positive arm is not a \
             verdict. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the control armed Add text and opened a new-text draft at the SAME point: `{}`",
        origin.raw
    ));

    let before_commit = session.trace()?.lines.len();
    driver.press_chord(&[vk::CONTROL], vk::ENTER)?;
    session.settle(34);

    let trace = session.trace()?;
    let Some(added) = trace
        .events(ADD_EVENT)
        .filter(|l| l.lineno > applied_at)
        .last()
    else {
        return Err(Error::new(format!(
            "the negative control's commit did not reach the document: no `{ADD_EVENT}` after \
             the refusal. New page text is written in the engine's own bundled Helvetica — the \
             very resource the operator's editable lines use — so a build where this fails has a \
             second, larger problem, and this check cannot use it as a baseline. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let added_at = added.lineno;
    report.note(format!(
        "★★ the control's edit REACHED the document: `{}` — this is his *\"the lines I added \
         below price) are editable\"*, driven",
        added.raw
    ));

    // ★★★ THE BASELINE ASSERTION. A decline published after a SUCCESSFUL edit
    // means the slot is not gated on failure, and every reading above is an
    // artefact of the frame count rather than of the program.
    if let Some(stray) = trace.lines.iter().find(|r| {
        r.event == "ui-rect" && r.lineno > added_at && r.get("name") == Some(DECLINE_REGION)
    }) {
        return Ok(Some(format!(
            "★★★ THE DECLINE SLOT DREW AFTER AN EDIT THAT SUCCEEDED, so the positive half of \
             this check is NOT a verdict. `{DECLINE_REGION}` was published at line {} — after \
             `{}` — which means the slot is either published unconditionally or is still \
             holding the refusal from the first gesture. In the first case a silent build would \
             pass this check for ever; in the second the operator is reading a sentence about a \
             gesture two gestures ago, which `app::status::decline`'s retirement rule exists to \
             forbid. The line: `{}`. Trace: {}.",
            stray.lineno,
            added.raw,
            stray.raw,
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★★★ and the slot stayed SILENT for it — {} trace lines were captured across the \
         control's commit, none of them a `{DECLINE_REGION}`. The instrument has dynamic range: \
         it speaks for the refusal and not for the success",
        trace.lines.len().saturating_sub(before_commit)
    ));

    Ok(None)
}

/// **Roll the wheel until the canvas is showing `want`.**
///
/// ## ★★★ Why this exists rather than a page-number box or a thumbnail
///
/// `text_selection::aim` refuses to convert a point whose page is not the page
/// the application says it is drawing, and the refusal is one of this harness's
/// best guards — mapping page 2's coordinates through page 1's rect produces a
/// click that is plausible, precise and in the wrong place, which is
/// indistinguishable from a broken feature and costs an investigation to
/// disprove. So a check aiming at anything but the first page must **move the
/// document**, not work around the guard.
///
/// The wheel is chosen over the Pages panel and over the status bar's page box
/// because it needs neither of them to be mounted, and this check has just
/// reset the dock. It is chosen over a keyboard chord because there is none:
/// `sys::vk` carries no `PAGE_DOWN`, deliberately.
///
/// ## ★★ It reads the application's own answer, not its own count
///
/// The loop does not scroll "the right number of times". It scrolls, then asks
/// the `canvas` line which page is on screen, and stops when that number is the
/// one wanted — because how far one notch travels depends on the zoom, the page
/// display mode and the platform, and a count derived from any of those is a
/// proxy for the thing that actually matters.
///
/// ★ It also refuses to loop for ever on a document that cannot reach the page:
/// a `--doc-point` naming page 9 of a three-page file would otherwise scroll to
/// the end and spin. The failure says how far it got.
fn scroll_to_page(
    ctx: &CheckContext,
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    want: usize,
    report: &mut CheckReport,
) -> Result<()> {
    let shown = |s: &Session| -> Result<Option<usize>> {
        Ok(s.trace()?
            .events(ctx.profile.vocab.canvas_event)
            .last()
            .and_then(|l| l.get_usize("page")))
    };
    if shown(session)? == Some(want) {
        return Ok(());
    }
    let trace = session.trace()?;
    let canvas = declared(&trace, ui_rect, CANVAS_REGION).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{CANVAS_REGION}` region, so there is nowhere to roll \
             the wheel and page {} cannot be reached. Regions beginning `canvas`: {}.",
            want + 1,
            list(&declared_names(&trace, ui_rect, "canvas"))
        ))
    })?;
    let over = session.frame()?.declared_at(canvas, 0.5, 0.5);
    let mut seen = shown(session)?;
    for step in 0..MAX_SCROLL_STEPS {
        driver.scroll_at(over, -NOTCHES)?;
        session.settle(10);
        let now = shown(session)?;
        if now == Some(want) {
            report.note(format!(
                "★ rolled the wheel {} step(s) to bring page {} onto the canvas",
                step + 1,
                want + 1
            ));
            return Ok(());
        }
        // ★ A page index that has stopped moving means the end of the document,
        // and continuing would spend forty steps learning the same thing.
        if step > 2 && now == seen {
            break;
        }
        seen = now;
    }
    Err(Error::new(format!(
        "page {} could not be brought onto the canvas: after rolling the wheel the application \
         is still reporting page {}. Either the document has fewer pages than the `--doc-point` \
         names — ★ PAGE IS 0-BASED — or the wheel is paging rather than scrolling and the \
         threshold was not met. Nothing below this would mean anything, so this is SKIPPED \
         rather than failed.",
        want + 1,
        seen.map_or_else(|| "an unreported index".to_owned(), |p| (p + 1).to_string()),
    )))
}

// (The blank-paper search that used to live here is gone. See the negative
// control's own comment: `hit_test` has no distance bound, so on a page that
// carries any text at all there is no point that resolves to no run, and the
// control arms **Add text** instead — which never asks the hit test.)
