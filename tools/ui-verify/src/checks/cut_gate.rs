//! `cutting_a_redaction_mark_is_refused_before_anything_is_removed` — the
//! driven proof of `OPERATOR_REQUESTS.md` **O59**'s first item.
//!
//! # What this is about
//!
//! `pdfcer-core`, unprompted, 2026-08-29:
//!
//! > **Do not offer Cut as enabled and let it fail.** A **copy** of something
//! > pdfcer cannot carry costs nothing — the original stays. A **cut** of the
//! > same thing is a deletion wearing a clipboard's clothes.
//!
//! A `/Redact` annotation is a **pending destructive operation**. Pasting one
//! arms a redaction nobody reviewed, so the engine refuses to carry it — and
//! therefore refuses to cut it, before anything is removed.
//!
//! # ★★★ Why greying the button is not enough, and this check is about the gap
//!
//! `edit.cut` is greyed on `selection.cut_permitted`, so a **pointer** cannot
//! reach the verb. **A chord can.** `Ctrl+X` is dispatched through the keymap
//! and the keymap does not consult command enablement — so the handler runs
//! whatever the ribbon is showing.
//!
//! That gap is the whole subject here. A build that greyed the button and left
//! the chord unguarded would look completely correct in every screenshot, pass
//! every unit test of the gate, and delete a redaction mark on `Ctrl+X` while
//! putting nothing on the clipboard.
//!
//! ⇒ So the assertion is not *"the button is grey"*. It is **the keystroke was
//! refused and the mark is still there.**
//!
//! # The oracle, and why it is two lines rather than one
//!
//! | line | question |
//! |---|---|
//! | `clipboard-cut-refused reason=would-not-survive subtype=Redact` | did the gate fire, and did it name the right thing? |
//! | the **absence** of `clipboard-copy` | did it refuse *before* the copy, or after? |
//!
//! ★ The second is the one that matters and it is easy to leave out. A cut that
//! refused *after* copying would leave the mark on the page **and a copy of it
//! on the clipboard** — so the next `Ctrl+V` arms a redaction somewhere else,
//! which is precisely the outcome the refusal exists to prevent. The order is
//! asserted, not assumed.
//!
//! # Why this fixture and not `--pdf`
//!
//! `demo-marked-output.pdf` from the engine's own corpus is a document with
//! **redaction marks already on it** — three of them. A check whose subject is
//! *"what happens when you cut a redaction mark"* cannot take an arbitrary
//! drawing: on the operator's own CAD sheets there are none, and the honest
//! answer there is *"there was nothing to try it on"*, which is neither a pass
//! nor a defect.
//!
//! Marking a page from the ribbon instead was the alternative and was rejected:
//! it would make a check about the clipboard fail whenever the redaction panel
//! moved, and `checks::redaction` already owns that sequence.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// The engine's own redaction fixture, with marks already applied.
const FIXTURE: &str = "redact/demo-marked-output.pdf";

/// The refusal this check reads.
const REFUSED: &str = "clipboard-cut-refused";

/// The line that must **not** appear before it.
const COPIED: &str = "clipboard-copy";

/// What the canvas says when an ANNOTATION is selected.
///
/// ★★ `annot-select`, not `canvas-selection`. The general selection line
/// carries `sel=`, `level=` and `first=` — all about the **content** index
/// spaces — and an annotation selection is not in either of them. Reaching for
/// the familiar line would have made the hunt below click twenty-five times and
/// conclude there were no marks on a document with three.
///
/// It also carries `subtype=`, which turns the hunt from *"did something get
/// selected?"* into *"did a REDACTION MARK get selected?"* — the difference
/// between a check that proves its own subject and one that proves a click
/// landed on some annotation and then blames the cut gate.
const ANNOT_SELECT: &str = "annot-select";

/// See the module documentation.
pub struct CuttingARedactionMarkIsRefusedBeforeAnythingIsRemoved;

impl Check for CuttingARedactionMarkIsRefusedBeforeAnythingIsRemoved {
    fn name(&self) -> &'static str {
        "cutting_a_redaction_mark_is_refused_before_anything_is_removed"
    }

    fn defect(&self) -> &'static str {
        "Ctrl+X over a redaction mark deletes it and puts nothing on the clipboard — greying the \
         ribbon button does not help, because a chord is dispatched through the keymap without \
         consulting command enablement, so the handler runs whatever the ribbon is showing"
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

/// Resolve a fixture under the engine repository's synthetic corpus.
///
/// `None` rather than a panic turns a missing corpus into a SKIP with a reason
/// instead of a crash mid-suite. `D:\Dev\pdfcer` is READ-ONLY to this project;
/// this reads from it and writes nowhere near it.
fn engine_fixture(rel: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(rel);
    path.is_file().then_some(path)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let vocab = &ctx.profile.vocab;

    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check cannot be performed without clicking \
             and typing. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let fixture = engine_fixture(FIXTURE).ok_or_else(|| {
        Error::new(format!(
            "the engine's marked redaction fixture is not at \
             D:/Dev/pdfcer/fixtures/synthetic/{FIXTURE}. This check pins it and ignores --pdf: its \
             subject is what happens when you cut a REDACTION MARK, and a drawing with none is \
             neither a pass nor a defect."
        ))
    })?;
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(
            "the profile declares no ui-rect trace event, so this check cannot leave Read mode — \
             where a canvas click on an annotation is refused by design.",
        )
    })?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("cut-gate.trace.txt"));
    spec.pdf = Some(fixture.clone());
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
        "launched {} as pid {} on {} — three redaction marks already on it",
        exe.display(),
        session.pid(),
        fixture.display()
    ));
    session.settle(45);

    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, "edit")?;
    report.note("clicked the Edit mode segment — Read refuses a canvas click on content by design");

    // --- find a mark and click it ------------------------------------------
    //
    // The page geometry comes from the fixture rather than from `--page-size`,
    // because this check pins its document and therefore knows it.
    let page: PageGeometry = crate::fixture::page_geometry(&fixture).ok_or_else(|| {
        Error::new(format!(
            "cannot read a page size from {}, so the coordinate hop has no page box.",
            fixture.display()
        ))
    })?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, vocab, page, 0)?;
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}",
        mapping.image_rect, mapping.zoom
    ));

    // ★ A sweep of candidate points rather than one. The fixture's marks cover
    // the redacted words, and this check does not know where those words are —
    // pinning a coordinate would make it fail the day the fixture is
    // regenerated with a different sentence. It clicks a coarse grid and stops
    // at the first point that selects an annotation, which is what an operator
    // does too.
    let mut selected = false;
    'hunt: for fy in [0.20_f64, 0.35, 0.50, 0.65, 0.80] {
        for fx in [0.20_f64, 0.35, 0.50, 0.65, 0.80] {
            let at = DocPoint::new(0, fx * page.width_pt, fy * page.height_pt);
            let window = mapping.doc_to_window(at)?;
            driver.click_at(session.frame()?.to_screen(window))?;
            session.settle(10);
            let trace = session.trace()?;
            // ★ The SUBTYPE, not merely "something was selected". A click that
            // landed on some other annotation would otherwise send this check
            // on to press Ctrl+X on a square and report the cut gate broken.
            if trace
                .last(ANNOT_SELECT)
                .and_then(|l| l.get("subtype"))
                .is_some_and(|st| st == "Redact")
            {
                report.note(format!(
                    "★ selected a /Redact mark at page fractions ({fx:.2}, {fy:.2})"
                ));
                selected = true;
                break 'hunt;
            }
        }
    }
    if !selected {
        return Err(Error::new(format!(
            "no point on a 5x5 grid over page 1 selected a /Redact annotation, so this check \
             never got a redaction mark under the cursor and has nothing to cut. Reported as a \
             SKIP rather than a failure: that is a fact about where this fixture's marks are, not \
             about the cut gate. ★ If the trace holds `annot-select` lines with other subtypes, \
             the clicks landed and the marks are elsewhere; if it holds none at all, the mode \
             click or the coordinate hop is the suspect. Trace: {}",
            session.trace_path().display()
        )));
    }

    // --- the gesture under test --------------------------------------------
    driver.press_chord(&[vk::CONTROL], vk::X)?;
    session.settle(20);
    let after = session.trace()?;

    let Some(line) = after.events(REFUSED).last() else {
        return Ok(Some(format!(
            "★★★ Ctrl+X OVER A REDACTION MARK WAS NOT REFUSED. No `{REFUSED}` line. A /Redact \
             annotation is a pending destructive operation and the clipboard cannot carry it, so \
             cutting it is a deletion that puts nothing anywhere. Greying `edit.cut` does not \
             cover this: a chord is dispatched through the keymap without consulting command \
             enablement. Trace: {}",
            session.trace_path().display()
        )));
    };
    let reason = line.get("reason").unwrap_or("");
    let subtype = line.get("subtype").unwrap_or("");
    report.note(format!(
        "★ Ctrl+X refused: reason={reason} subtype={subtype}"
    ));

    if reason != "would-not-survive" {
        return Ok(Some(format!(
            "the cut was refused for `{reason}`, not for `would-not-survive`. Something else \
             declined first — most likely the delete gate — so this check proved a different \
             thing than its name. Trace: {}",
            session.trace_path().display()
        )));
    }
    if subtype != "Redact" {
        return Ok(Some(format!(
            "the refusal names subtype `{subtype}`, not `Redact`. The subtype travels into the \
             operator's sentence, so a wrong one is a wrong explanation. Trace: {}",
            session.trace_path().display()
        )));
    }

    // ★★★ AND IT REFUSED **BEFORE** THE COPY. The half that is easy to omit:
    // a cut that refused after copying would leave the mark on the page AND a
    // copy of it on the clipboard, so the next Ctrl+V arms a redaction
    // elsewhere — the exact outcome the refusal exists to prevent.
    if after.events(COPIED).count() > 0 {
        return Ok(Some(format!(
            "★★★ THE CUT REFUSED, AND COPIED ANYWAY. A `{COPIED}` line is in the trace beside \
             the refusal, so the redaction mark is on the page AND on the clipboard — and the \
             next Ctrl+V arms a redaction nobody reviewed somewhere else. The gate must run \
             BEFORE the copy, not after it. Trace: {}",
            session.trace_path().display()
        )));
    }
    report.note("★★ and nothing was copied — the gate ran before the copy, not after it");

    Ok(None)
}
