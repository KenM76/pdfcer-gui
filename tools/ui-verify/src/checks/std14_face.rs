//! `the_face_chooser_offers_a_face_the_document_does_not_contain` — **open the
//! font list, pick a face that is not in the file, and the file changes.**
//!
//! # What this is for
//!
//! `pdfcer-core` v0.15.0 (`Pass 162.0`) closed the last of the four things the
//! operator named as not fully editable:
//!
//! > **FONTS** — text can be restyled to a face the document **DOES NOT
//! > CONTAIN**, for the fourteen faces every PDF reader is required to have.
//! > pdfcer authors the font resource on demand, with widths, embedding nothing.
//!
//! The engine shipped it and **the shell could not reach it**: the chooser built
//! its list from `preview_font_resources`, which by construction enumerates the
//! *page's own* `/Font` resources, so the one thing the release note is about
//! was absent from every surface in the program. That is the specific defect
//! this check exists to detect — a capability that is present, tested, released
//! and unreachable.
//!
//! ## ★★ The way this feature can pass every unit test and be dead
//!
//! `panels::properties::face::choices` is unit-tested: give it a page carrying
//! `ArialMT` and it answers fourteen addable faces. **Every one of those tests
//! would still pass on a build where the popup never draws them** — where the
//! two group headings are never reached, where the disclosure is drawn off the
//! bottom of a 78-point-wide popup, where the row is drawn and the click parks
//! nothing, or where the selector sent is the label of the row above.
//!
//! The chain this drives:
//!
//! | # | link | its own test |
//! |---|---|---|
//! | 1 | a sweep produces a `TextSelection` and the panel draws a Text section | `restyle_text` |
//! | 2 | the chooser's combo is on screen and opens | ★ **nothing** |
//! | 3 | the popup separates the two kinds of row and draws the standard-14 heading | ★ **nothing** |
//! | 4 | ★★★ **the disclosure is on screen where the choice is made** | ★ **nothing** — and no unit test can see it, because what is under test is that a string reached a rectangle |
//! | 5 | a standard-14 row is clickable | ★ **nothing** |
//! | 6 | the click reaches `format_text` and the document changes | unit-tested |
//!
//! Link 4 is the one this check would be worth writing for on its own. pdfcer
//! *"authors the font resource on demand, with widths, embedding nothing"*, so
//! the restyled text is drawn with **the reader's own copy** of that face —
//! invisible on this screen and visible on somebody else's machine. Rule 4's
//! surviving half is that an inference the operator cannot see still owes an
//! off-canvas report, and a disclosure that is written, catalogued, unit-tested
//! for its three clauses and then never painted has discharged nothing.
//!
//! ## ★ The control point, and why it is not optional
//!
//! `properties.text.face.new` must be **absent** before the combo is clicked.
//! Without that, a build whose popup was somehow always open would pass this
//! check on regions that were never opened by the gesture — the defect
//! `max_zoom` names in its own header, wearing the same green tick.
//!
//! ## ★★ It has been seen to FAIL, which is the acceptance criterion
//!
//! [`crate::checks`]' founding rule: *"it must fail against a build where the
//! wiring is absent, and the wiring must be something no unit test in the
//! workspace can observe."* A check that has only ever been seen to pass is
//! indistinguishable from one that cannot fail.
//!
//! Falsified on 2026-08-29 against `D:/Dev/pdfTests/SW41177/SW41177.pdf` at
//! `--doc-point 0,1140,62`, by planting the **pre-change behaviour** —
//! `panels::properties::face::choices` returning only the page's own accepted
//! faces, which is exactly what the chooser did before this work — and
//! rebuilding `--release`:
//!
//! | build | verdict |
//! |---|---|
//! | page faces only (the planted build, and the shell as it shipped) | **FAIL** — *"the chooser opened and declared no `properties.text.face.addable` region"*, with `properties.text.face` as the only region under that prefix and a screenshot beside it |
//! | page faces + the standard fourteen | **PASS** — `text-style-applied page=0 change=face applied=19 runs=14`, followed by `format-text` |
//!
//! ★ Note what the planted build's failure text shows: the popup **opened**,
//! the page's own faces were **listed**, and every unit test in the workspace
//! was green — including `choices`' own, which asserts the filter and not the
//! painting. The whole defect was one absent group in one popup, and this is
//! the only instrument in the project that can see it.
//!
//! ## What this deliberately does not assert
//!
//! **Which** of the fourteen was applied. The trace's `text-style-applied` line
//! carries `change=face` and not the selector, and adding the face name to it to
//! satisfy a check would be the harness dictating a diagnostic's contents. The
//! row that is clicked is the *first* addable row on a fixture whose page fonts
//! are known, which is deterministic enough; and `format-text` landing is the
//! claim that matters, because it is the one that says a `/Font` object was
//! written into the operator's document.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The mode whose canvas may select page content and whose panels carry
/// Properties.
const MODE: &str = "edit";
/// The commands run at startup, in order, before this check touches anything.
///
/// ★★ The same two `restyle_text` learned to need, and for the same reason it
/// learned it: **the dock arrangement is persisted per machine**, so the
/// Properties panel being on screen is a coincidence rather than a
/// precondition. On 2026-08-29 that check spent a red run reporting a panel as
/// silent when the panel was not mounted at all. `file.properties` mounts and
/// activates it from any arrangement and is idempotent; `mode.edit` is first
/// because the dock follows the ribbon mode on the same frame, so a panel
/// mounted before the mode moved would be mounted into the workspace this check
/// is about to leave.
const INVOKE: &str = "mode.edit,file.properties";
/// The Text section's own region.
const SECTION_REGION: &str = "properties.text";
/// The face chooser's combo — the control that opens the popup.
const FACE_REGION: &str = "properties.text.face";
/// The heading over the rows pdfcer would ADD to the document.
const ADDABLE_REGION: &str = "properties.text.face.addable";
/// ★★★ The standard-14 disclosure, drawn once, where the choice is made.
const DISCLOSURE_REGION: &str = "properties.text.face.disclosure";
/// The first row offering a face the document does not contain.
const NEW_FACE_REGION: &str = "properties.text.face.new";
/// The `text-style-applied page=… change=… applied=… of=…` line.
///
/// ★ Named `-applied` rather than plain `text-style` because `vector_edit`'s
/// label for the same edit is a sibling event and trace matching is on the exact
/// event name — the mistake `tools/gates/check-trace-names.py` was written after
/// this project made three times in three days.
const STYLE_EVENT: &str = "text-style-applied";
/// The `text-style-declined applied=…` line.
const DECLINED_EVENT: &str = "text-style-declined";
/// The label `vector_edit` writes when the restyle reached the engine.
const APPLIED: &str = "format-text";
/// The sweep's own oracle, shared with `clipboard_text` and `restyle_text`.
const SELECTION_EVENT: &str = "canvas-text-selection";
/// How far to sweep along the baseline, in PDF points.
const SWEEP_PT: f64 = 60.0;
/// How many scroll notches to spend looking for the chooser below the fold.
const SCROLL_ATTEMPTS: usize = 6;
/// `T`, as a Windows virtual key — the text-sweep tool.
///
/// ★★ Not optional. In Edit mode `textsel::gate::takes_the_press` reads
/// `tool.is_text() || (Select && !caps.edit_content)`, and the second disjunct
/// is false there — so a drag with the Select tool is an object marquee and the
/// check would report a working panel as broken.
const VK_T: u16 = 0x54;
/// `Esc`, to shut the popup if this check leaves early.
const VK_ESCAPE: u16 = 0x1B;

/// See the module documentation.
pub struct TheFaceChooserOffersAFaceTheDocumentDoesNotContain;

impl Check for TheFaceChooserOffersAFaceTheDocumentDoesNotContain {
    fn name(&self) -> &'static str {
        "the_face_chooser_offers_a_face_the_document_does_not_contain"
    }

    fn defect(&self) -> &'static str {
        "pdfcer-core can restyle text to any of the fourteen standard faces without embedding \
         anything, and the shell's font chooser offers only the fonts the page already carries — \
         so the capability is present, released and unreachable; or it is offered without the \
         disclosure that the text will then be drawn with the reader's own copy of the face"
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

/// Poll until the restyle reports one way or the other, and answer how long it
/// took.
///
/// ★ A bounded poll rather than a fixed sleep, for `restyle_text`'s reason: a
/// restyle re-resolves the pin per run from a fresh provenance extraction, a
/// sweep across a title-block label is a dozen runs, and a fixed sleep long
/// enough for that makes every run slow while a pleasant one fails on the
/// operator's own drawings. Reaching the ceiling is not an error — the caller
/// then reads a trace with neither line in it and reports the "nothing
/// happened" verdict, which is the right answer if a restyle really can take
/// twenty seconds.
fn wait_for_verdict(session: &Session) -> Result<u128> {
    const CEILING_MS: u128 = 20_000;
    let started = std::time::Instant::now();
    loop {
        session.settle(4);
        let trace = session.trace()?;
        if trace.last(STYLE_EVENT).is_some() || trace.last(DECLINED_EVENT).is_some() {
            return Ok(started.elapsed().as_millis());
        }
        if started.elapsed().as_millis() > CEILING_MS {
            return Ok(started.elapsed().as_millis());
        }
    }
}

#[allow(clippy::too_many_lines)]
fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. This check needs a page carrying real text."))?;
    let target = ctx.target.ok_or_else(|| {
        Error::new(
            "no --doc-point. Pass PAGE,X,Y in PDF user space naming the LEFT END of a piece of \
             text's baseline. `pdfcer extract-text --json` gives the first glyph's x and y of \
             every run; use those. A point on blank paper sweeps nothing and the check would \
             report the chooser as broken.",
        )
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check sweeps the pointer across text, opens a \
             combo box and clicks a row in it, and none of the three can be simulated from the \
             trace.",
        ));
    }
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event.",
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

    let mut spec = LaunchSpec::new(&exe, ctx.out("std14-face.trace.txt"));
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

    // --- 1: Edit mode -------------------------------------------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // --- 2: sweep across the text ------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, target.page)?;
    let frame = session.frame()?;
    let start =
        frame.to_screen(mapping.doc_to_window(DocPoint::new(target.page, target.x, target.y))?);
    let end = frame.to_screen(mapping.doc_to_window(DocPoint::new(
        target.page,
        target.x + SWEEP_PT,
        target.y,
    ))?);
    driver.press(VK_T)?;
    session.settle(16);
    driver.drag(start, end)?;
    session.settle(24);

    let trace = session.trace()?;
    let swept = trace
        .events(SELECTION_EVENT)
        .last()
        .and_then(|l| l.get("chars"))
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or(0);
    if swept == 0 {
        return Err(Error::new(format!(
            "the drag from (page {}, {:.1}, {:.1}) rightwards {SWEEP_PT} pt selected no text, so \
             there was no run for the chooser to be about. SKIPPED rather than failed: this says \
             the --doc-point is not on text, which is the harness's aim and not the program's \
             behaviour. Trace: {}.",
            target.page,
            target.x,
            target.y,
            session.trace_path().display()
        )));
    }
    report.note(format!("the sweep selected {swept} character(s)"));

    // --- 3: the section, then the chooser, scrolling as an operator would ---
    let trace = session.trace()?;
    if driving::declared(&trace, ui_rect, SECTION_REGION).is_none() {
        return Err(Error::new(format!(
            "{swept} character(s) are selected and there is no `{SECTION_REGION}` region, so the \
             Properties panel drew no Text section at all. SKIPPED rather than failed: that is \
             `restyle_text`'s subject and it reports it with the three candidates and a \
             screenshot. Fix it there; this check has nothing to say until the section draws. \
             Trace: {}.",
            session.trace_path().display()
        )));
    }

    let mut combo = None;
    for attempt in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, FACE_REGION) {
            combo = Some(rect);
            if attempt > 0 {
                report.note(format!(
                    "the face chooser was below the panel's fold; {attempt} scroll notch(es) \
                     brought it into view"
                ));
            }
            break;
        }
        let Some(section) = driving::declared(&trace, ui_rect, SECTION_REGION) else {
            return Err(Error::new(format!(
                "the Text section stopped being visible while scrolling for the face chooser. \
                 Trace: {}.",
                session.trace_path().display()
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(section), -1)?;
        session.settle(12);
    }
    let Some(combo) = combo else {
        // ★ The skip reason names what WAS on screen, per this crate's rule 5: a
        // reason that says "I did not find X" and does not say what it *did*
        // find sends its reader to guess.
        let seen = driving::live_names(&session.trace()?, ui_rect, SECTION_REGION);
        return Err(Error::new(format!(
            "no `{FACE_REGION}` region after scrolling the Properties panel {SCROLL_ATTEMPTS} \
             times, so the chooser was never on screen. SKIPPED rather than failed: a control \
             that was never opened proves nothing about what it offers. Regions beginning \
             `{SECTION_REGION}`: {seen:?}. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- 4: the control point ----------------------------------------------
    //
    // ★ See the module header. Without this, every region below could be one
    // that was declared from the first frame, and the check would be green on a
    // popup no gesture ever opened.
    let trace = session.trace()?;
    if driving::declared(&trace, ui_rect, NEW_FACE_REGION).is_some() {
        return Ok(Some(format!(
            "`{NEW_FACE_REGION}` is declared before the chooser was clicked, so the popup's rows \
             are on screen with the popup shut. That is a layer or a visibility defect rather \
             than a chooser one, and it would make every assertion below vacuous. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 5: open the chooser ------------------------------------------------
    let frame = session.frame()?;
    driver.click_at(frame.declared_center(combo))?;
    session.settle(24);

    let trace = session.trace()?;
    let Some(_addable) = driving::declared(&trace, ui_rect, ADDABLE_REGION) else {
        let shot = ctx.out("std14_face.no-addable-group.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        driver.press(VK_ESCAPE)?;
        return Ok(Some(format!(
            "★ THE FONT LIST OFFERS NOTHING THE DOCUMENT DOES NOT ALREADY CONTAIN: the chooser \
             opened and declared no `{ADDABLE_REGION}` region.\n\
             `pdfcer-core` v0.15.0 authors a standard-14 `/Font` resource on demand, so a page \
             built from one embedded face should offer thirteen or fourteen more. Three \
             candidates. (1) **`panels::properties::face::choices` returned no addable rows** — \
             it filters `Std14::ALL` against every `/BaseFont` in the pre-flight's `entries`, so \
             a fixture whose page genuinely carries all fourteen would legitimately produce \
             this; the Fonts panel says which fonts this page has. (2) **The popup never reached \
             the heading**, which is a draw-order defect in `popup_body`. (3) **The pre-flight \
             was absent** — `choices(None)` is an empty list by design, and the chooser then \
             shows only its no-faces sentence. The screenshot beside this report separates (2) \
             from (1) and (3): a popup showing page fonts and no second group is (2). Regions \
             beginning `{FACE_REGION}`: {:?}. Trace: {}.",
            driving::live_names(&trace, ui_rect, FACE_REGION),
            session.trace_path().display()
        )));
    };
    report.note("★ the chooser offers a second group — faces pdfcer would add to the document");

    // --- 6: ★★★ the disclosure is ON SCREEN, where the choice is made -------
    if driving::declared(&trace, ui_rect, DISCLOSURE_REGION).is_none() {
        let shot = ctx.out("std14_face.no-disclosure.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        driver.press(VK_ESCAPE)?;
        return Ok(Some(format!(
            "★★★ THE STANDARD-14 FACES ARE OFFERED AND THE DISCLOSURE IS NOT ON SCREEN: \
             `{ADDABLE_REGION}` is declared and `{DISCLOSURE_REGION}` is not.\n\
             pdfcer authors these faces *\"with widths, embedding nothing\"*, so the restyled text \
             is drawn with the READER'S OWN COPY of the face — which looks correct on this \
             machine and can look different on somebody else's. That is an inference the \
             operator cannot see, and rule 4 requires an off-canvas report for exactly that \
             case. A sentence that is catalogued and unit-tested and never painted has \
             discharged nothing.\n\
             Two candidates. (1) **It is drawn and clipped away** — `ui_rect_visible` publishes \
             nothing for a rect outside the clip, and the ribbon's copy of this popup is only 78 \
             points wide before `POPUP_MIN_WIDTH` widens it. (2) **It is not drawn** — \
             `popup_body` draws it once, immediately after the addable heading, so a heading \
             without a disclosure is a two-line edit that dropped one. The screenshot beside \
             this report tells them apart. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note("★★★ the disclosure is on screen, in the popup, before any addable row is clicked");

    // --- 7: click a face the document does not contain -----------------------
    let Some(row) = driving::declared(&trace, ui_rect, NEW_FACE_REGION) else {
        driver.press(VK_ESCAPE)?;
        return Ok(Some(format!(
            "the chooser drew the `{ADDABLE_REGION}` heading and no `{NEW_FACE_REGION}` row \
             under it, so the group is a caption over an empty band — the exact defect \
             `app::fontband`'s own tests were written after, where a manifest declared a custom \
             kind no renderer matched and the symptom was a gap. Trace: {}.",
            session.trace_path().display()
        )));
    };
    let frame = session.frame()?;
    driver.click_at(frame.declared_center(row))?;

    let waited = wait_for_verdict(&session)?;
    report.note(format!(
        "the restyle took {waited} ms of wall clock — one provenance extraction per run"
    ));

    // --- 8: the verdict ------------------------------------------------------
    let trace = session.trace()?;
    if let Some(declined) = trace.events(DECLINED_EVENT).last() {
        return Ok(Some(format!(
            "a standard-14 face was chosen and the restyle DECLINED: `{}`.\n\
             The program answered rather than staying silent, so the chain works — but the \
             answer is worth reading, and there is one honest reason for it. pdfcer runs its own \
             coverage test against the SYNTHESIZED dictionary too, so a run containing a \
             character that face cannot encode is refused exactly like a page font that cannot; \
             the shell cannot pre-test that without re-deriving the encoding rule, so these rows \
             are offered and the refusal is a sentence. If the refusal here is \
             `FaceLacksCharacters` on a symbolic face, that is the designed behaviour and the \
             fixture picked the wrong first row. Anything else — a `FaceNotOnPage` above all — \
             means the selector sent was not a standard-14 spelling, which is a chooser defect. \
             Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    let Some(applied) = trace.events(STYLE_EVENT).last() else {
        let shot = ctx.out("std14_face.no-effect.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "★ A STANDARD-14 ROW WAS CLICKED AND NOTHING HAPPENED AND NOTHING WAS DECLINED: no \
             `{STYLE_EVENT}` line and no `{DECLINED_EVENT}` line.\n\
             Two candidates. (1) **The click missed the row** — the region was declared, so this \
             is a coordinate problem inside a popup layer and the screenshot settles it. (2) \
             **The press raised no action**: `popup_body` returns the chosen selector and \
             `face_row` turns it into one `Action::TextStyle` OUTSIDE the popup closure, because \
             nothing mutates from a widget. A return value dropped between those two is exactly \
             this symptom, and an operator would report it as 'the font list does nothing'. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "★ the chosen face committed a restyle: `{}`",
        applied.raw
    ));

    let n: usize = applied
        .get("applied")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        return Ok(Some(format!(
            "the restyle reported `applied=0`, so the module decided to act and every run it \
             tried refused without saying so. That is worse than a decline: `{}`. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "the chooser computed `{}` and no `{APPLIED}` line followed, so the action was \
             raised and its apply arm never ran. Nothing reached the document, which from a \
             chair is indistinguishable from the list doing nothing. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note(
        "★★★ the swept text was restyled to a face the document did not contain, through \
         `format_text` — which wrote the `/Font` resource itself, in the same undo command",
    );
    Ok(None)
}
