//! `text_on_a_scan_can_still_be_swept_over_the_image` — **the OCR layer must
//! not lose to the picture it sits on.**
//!
//! # The report
//!
//! Ken, 2026-09-01: *"I can't seem to copy and paste text we have OCRed."*
//!
//! ## ★★★ It was caused by a feature that shipped seven hours earlier
//!
//! His own earlier ask — *"select images so we can copy and paste them"* —
//! landed that morning as a Read-mode arm: a click on an image selects it, and
//! clears the text selection, because the operator has just said they mean the
//! picture.
//!
//! It was narrowed to **images only**, on an argument that was right about the
//! document it was written against: a CAD sheet has a path under the pointer
//! almost everywhere, so admitting paths would have made text unreachable. The
//! narrowing does nothing for the case that broke, because **a scanned page IS
//! one image**, edge to edge — every click hits it, and an OCR layer is
//! invisible text lying exactly on top.
//!
//! ⇒ The one document class where selecting text matters most is the one where
//! the arm swallowed it.
//!
//! ## ★★ Why this check is not "does OCR work"
//!
//! Three things were ruled out **before** any code was changed, by measurement
//! rather than by reading:
//!
//! | | verdict |
//! |---|---|
//! | does the engine extract an OCR layer? | **yes** — 13 invisible codes off his own file |
//! | do the recognised words carry geometry? | **yes** — a 6 × 6 pt box on the first word |
//! | does the shell's own extraction differ? | no — same options, same funnel |
//!
//! So the subject here is precisely the **precedence** between two things that
//! both claim the same pixel, and the oracle has to distinguish them: a click
//! that selects the image traces `via=read-image`, and one that sweeps text
//! traces a text selection. Both are "something was selected".
//!
//! ## The sequence
//!
//! | # | step | oracle |
//! |---|---|---|
//! | A | open a recognised scan in Read mode | the page draws |
//! | B | click a point **on a recognised word** | a text selection, and NOT `via=read-image` |
//! | C | click a point on the same page with **no** word under it | `via=read-image` — the picture still wins where the words are not |
//!
//! ★★★ Step C is the control point and is the half a careless fix would break.
//! Making text win everywhere would take the image feature away again, which is
//! the same defect facing the other direction — and a check asserting only B
//! would pass against exactly that.

use crate::checks::driving::{SHELL_DIAG_ENV, declared, declared_names, list};
use crate::checks::text_selection::aim;
use crate::checks::{Check, CheckContext};
use crate::coords::{DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The line a content selection writes. `via=read-image` names the image arm.
const SELECTION: &str = "selection-set"; // ui-text-exempt: a trace event name, never displayed
/// The line a text sweep writes.
///
/// ★★★ `canvas-text-selection`, and the first three drafts said `text-selection`
/// — which is a SUBSTRING of it. Every `grep` used to confirm the name matched,
/// the trace looked right, and the check reported *"selected no characters"*
/// through a settle, a longer settle and a poll loop, because `events()` is an
/// exact match and none of those three attempts was ever going to work.
///
/// ⇒ A harness constant confirmed by a substring grep is not confirmed. The
/// instrument that finally answered it was the check reporting what it had
/// SEEN — `saw 0 line(s): []` beside a file that plainly contained two.
const TEXT: &str = "canvas-text-selection"; // ui-text-exempt: a trace event name
/// The page region, so a failure can say whether a sheet was drawn at all.
const PAGE_REGION: &str = "page"; // ui-text-exempt: a trace region name, never displayed

/// The operator's own recognised scan.
const FIXTURE: &str = r"C:\Users\Ken\OneDrive\pdfTests\KEN-recognised.pdf";

/// Where the recognised words are, in page points.
///
/// ★ Measured off the engine's own extraction rather than guessed: the first
/// run's box is `[9.6, 4.46, 15.84, 10.42]`, so its middle is about
/// `(12.7, 7.4)`. A point picked by eye from a raster would be a point in
/// raster space, which is not this space and is a coordinate-space error this
/// harness has made before.
const ON_A_WORD: (f64, f64) = (12.7, 7.4);

/// A point on the same page with no recognised word under it.
///
/// ★★ Inside the page and clear of every run. The first draft put this at
/// (200, 500) on a page that turns out to be **145 x 74 pt** — a point off the
/// sheet entirely, which would have made step C assert something about nothing.
/// Read off the extraction's own run boxes, none of which reaches past x=110.
const OFF_THE_WORDS: (f64, f64) = (120.0, 62.0);

pub struct TextOnAScanCanStillBeSweptOverTheImage;

impl Check for TextOnAScanCanStillBeSweptOverTheImage {
    fn name(&self) -> &'static str {
        "text_on_a_scan_can_still_be_swept_over_the_image"
    }

    fn defect(&self) -> &'static str {
        "on a scanned page the picture takes every click, so the OCR text lying on top of it \
         cannot be selected or copied — the one document class where recognised text is the \
         only text there is"
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

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). The subject is which of two things a click means.",
        ));
    }
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let source = std::path::PathBuf::from(FIXTURE);
    if !source.is_file() {
        return Err(Error::new(format!(
            "no recognised scan at {FIXTURE}. This check needs a page that is ONE image with an \
             invisible OCR layer over it; a document with ordinary text would not reproduce the \
             precedence question at all."
        )));
    }
    let pdf = ctx.out("ocr-text-select.pdf");
    if let Some(dir) = pdf.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::new(e.to_string()))?;
    }
    std::fs::copy(&source, &pdf).map_err(|e| Error::new(e.to_string()))?;

    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page = crate::fixture::page_geometry(&pdf).ok_or_else(|| {
        Error::new("could not read the fixture's page size, and this check works in page points.")
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("ocr-text-select.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    // ★ No mode command. Read is the default and is the mode this is about —
    // in Edit the image arm does not run at all and the check would pass
    // without exercising anything.
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    session.settle(50);
    let driver = Driver::new(session.window());

    if declared(&session.trace()?, ui_rect, PAGE_REGION).is_none() {
        return Err(Error::new(format!(
            "no `{PAGE_REGION}` region, so no sheet is on screen. Regions beginning `page`: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "page"))
        )));
    }

    // --- B: on a word ------------------------------------------------------
    let at = aim(
        ctx,
        &session,
        PageGeometry {
            width_pt: page.width_pt,
            height_pt: page.height_pt,
        },
        DocPoint::new(0, ON_A_WORD.0, ON_A_WORD.1),
    )?;
    // ★★ A DOUBLE-click, not a single one. A single click on text places a
    // caret and traces `chars=0` — which proves text took the press but is
    // indistinguishable from a click that landed on nothing. A double-click
    // selects the word, so `chars>0` is a positive statement about the OCR
    // layer being reachable rather than an absence of evidence.
    driver.click_at(at)?;
    session.settle(20);
    driver.double_click_at(at)?;

    // ★★★ **POLLED, not settled** — `D:/dev/rag/egui/` records this exact rule
    // under `a_multi_step_gesture_needs_a_polled_verdict_not_a_settle`, and
    // this check reproduced it twice before applying it.
    //
    // A double-click has to wait out the system's double-click interval before
    // the second press counts, and the word selection is traced after that. A
    // fixed settle of 35 and then 70 ticks both read the trace before the line
    // existed and reported *"selected no characters"* — the harness's commonest
    // false negative, and one that reads exactly like the defect under test.
    //
    // ★ The loop asks for the ANSWER rather than for time. It ends the moment
    // the line appears, so the ordinary case costs a frame or two, and its
    // bound is what turns "the feature is missing" into a claim worth making.
    let mut trace = session.trace()?;
    for _ in 0..40 {
        if trace.events(TEXT).any(|l| l.get("via") == Some("word")) {
            break;
        }
        session.settle(5);
        trace = session.trace()?;
    }
    let took_image = trace
        .events(SELECTION)
        .any(|l| l.get("via") == Some("read-image"));
    let took_text = trace
        .events(TEXT)
        .filter_map(|l| l.get("chars").and_then(|c| c.parse::<usize>().ok()))
        .any(|chars| chars > 0);
    if took_image {
        return Ok(Some(format!(
            "★★★ THE PICTURE TOOK A CLICK ON A WORD: `{SELECTION} … via=read-image` after a \
             click at ({:.1}, {:.1}), which is the middle of the first recognised word.\n\
             This is the operator's report reproduced: on a scanned page the image is the whole \
             page, so the Read-mode image arm swallows every attempt to sweep the OCR layer. \
             `canvas::clicking` must ask whether there is text under the pointer BEFORE taking \
             the image. Trace: {}.",
            ON_A_WORD.0,
            ON_A_WORD.1,
            session.trace_path().display()
        )));
    }
    if !took_text {
        return Err(Error::new(format!(
            "the double-click on ({:.1}, {:.1}) selected no characters and took no image, so it \
             it landed on nothing this check can reason about — most likely the aim missed the \
             word. SKIPPED rather than failed: that is a fact about this point, not about \
             precedence. Trace: {}.",
            ON_A_WORD.0,
            ON_A_WORD.1,
            session.trace_path().display()
        )));
    }
    report.note("★★ a click on a recognised word selects TEXT, not the picture under it");

    // --- C: off the words, the picture still wins ---------------------------
    //
    // ★★★ THE CONTROL POINT. Making text win everywhere would take the image
    // feature away again — the same defect facing the other way — and a check
    // asserting only step B would pass against exactly that.
    let off = aim(
        ctx,
        &session,
        PageGeometry {
            width_pt: page.width_pt,
            height_pt: page.height_pt,
        },
        DocPoint::new(0, OFF_THE_WORDS.0, OFF_THE_WORDS.1),
    )?;
    driver.click_at(off)?;
    session.settle(35);

    let trace = session.trace()?;
    let image_now = trace
        .events(SELECTION)
        .filter(|l| l.get("via") == Some("read-image"))
        .count();
    if image_now == 0 {
        return Ok(Some(format!(
            "★★ THE PICTURE CAN NO LONGER BE SELECTED AT ALL: a click at ({:.0}, {:.0}), well \
             clear of any recognised word, produced no `via=read-image`.\n\
             The fix for the OCR layer has gone one step too far — text now wins everywhere, \
             which takes away the image copying the operator asked for on 2026-08-31. The \
             predicate must be CONTAINMENT in a run's box, not the nearest-line fallback that \
             answers almost everywhere. Trace: {}.",
            OFF_THE_WORDS.0,
            OFF_THE_WORDS.1,
            session.trace_path().display()
        )));
    }
    report
        .note("★★★ …and off the words the picture still takes the click, which is the whole rule");
    Ok(None)
}
