//! `a_click_inside_a_form_selects_what_is_drawn_there` — the driven proof of
//! the operator's headline defect.
//!
//! # The defect
//!
//! Reported 2026-08-26, and it is the largest single complaint in
//! `OPERATOR_REQUESTS.md`:
//!
//! > *"There are obviously more than one item on the page, but when I click on
//! > one of the objects all I get is the page selected. When I double click on
//! > an object it doesn't select — it still only has the whole page selected."*
//!
//! He was reporting the truth, precisely. His file wraps its visible body in a
//! **form XObject**, `pdfcer-core` did not descend into one, and the form's
//! `/BBox` is a clipping extent (§8.10.1) rather than a claim about ink — so a
//! page-sized form sat in paint order above everything drawn before it and won
//! every click at every point. He was selecting a real object. It was the
//! wrapper.
//!
//! # Why a unit test is not enough here, and this is R1's own argument
//!
//! `panels::objects::provider::tests` proves the hit test against a real
//! decomposition, and those tests are good: they were falsified by putting the
//! shallow query back, which turns three of them red. They still cannot see
//! four things that stand between the engine's answer and the operator's
//! screen, and every one of them has been a real defect on this project:
//!
//! | | the failure it would hide |
//! |---|---|
//! | the **mode gate** | Edit is required for content selection, and a check that did not switch modes once reported the gate as a selection defect |
//! | the **coordinate hop** | canvas space, window points, DPI scaling and the page raster's own rect. A wrong page height mirrors every click about the page centre and hit-tests something plausible |
//! | the **dock geometry** | a panel width changes the canvas rect, which has silently invalidated harness coordinates before |
//! | the **click actually reaching egui** | in-process injection would not exercise the focus machinery a person's click does |
//!
//! ★ *"The tests pass"* is not a report of working software. That is the rule
//! this project was founded on, and this file is its discharge for the form
//! work.
//!
//! # The oracle
//!
//! `canvas-selection` gained a **`first=`** field on 2026-08-27, for this
//! check. Before it the line carried `sel=` (a count) and `level=` (a rung),
//! and neither can distinguish the defect from the fix: selecting the
//! page-sized form and selecting the square inside it both produce
//! `sel=1 level=Object`. A check reading that line would have passed against
//! the broken build — which is this harness's own stated worst outcome, a
//! green result reporting nothing.
//!
//! `first=` is `object:N`, `leaf:N` or `none`. The kind is spelled out rather
//! than implied, because `objects[7]` and `leaves[7]` are different things in
//! the same document.
//!
//! # The sequence
//!
//! 1. open the pinned fixture — a 200 × 200 pt page whose **only** page object
//!    is a page-sized form holding three 40 × 40 squares;
//! 2. click the Edit mode segment, because a Read-mode canvas click on content
//!    is refused by design (`DEFECTS.md` D6);
//! 3. click the **centre of the middle square**, through the OS;
//! 4. assert `first=leaf:` — the object painted inside the form;
//! 5. click a **gap between two squares**, still inside the form's page-sized
//!    box;
//! 6. assert the selection is **empty**.
//!
//! ★★ **Step 6 is the half that is easy to lose and expensive to lose.** It
//! forbids the tempting "fall back to the shallow hit test when the deep one
//! finds nothing" repair, which would answer a click on blank paper inside a
//! page-sized form with the form — the operator's original complaint, restored,
//! for the case that produces it most often. A check that asserted only step 4
//! would stay green through that regression.
//!
//! # Why this check pins its own fixture and ignores `--pdf`
//!
//! The same reason `ocr` does, learned the same week: a check whose subject is
//! *"what does a click inside a form select"* cannot take an arbitrary
//! document. On a drawing with no forms — the operator's own SolidWorks export
//! has **zero** — the honest answer is *"there was nothing to descend into"*,
//! which is neither a pass nor a defect. A suite-wide `--pdf` is a convenience
//! for the checks that need *some* drawing; this one needs a specific shape.
//!
//! The fixture is the engine's `forms-xobject/page-sized-form.pdf`, read from
//! the read-only corpus at `D:\Dev\pdfcer`. It is the file the engine built to
//! reproduce this operator's report, so the check and the fix are aimed at the
//! same target by construction.
//!
//! # Where the aim comes from
//!
//! The squares' page-space boxes are known from the fixture and stated as
//! constants below rather than discovered at run time, and the geometry hop is
//! `crate::coords::CanvasMapping` as every other driven check uses. A literal
//! screen coordinate is never written: `crate::coords`'s header records what
//! that cost the last time somebody tried.

use crate::checks::driving::{SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, ScreenPoint, WindowFrame};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// The fixture, under the engine's read-only synthetic corpus.
///
/// One page-sized form holding three separate 40 × 40 squares: **one** page
/// object, **three** leaves.
const FIXTURE: &str = "forms-xobject/page-sized-form.pdf";

/// The fixture's page, in PDF points. Stated rather than read, because the
/// whole file is fourteen objects of hand-written syntax and a page size that
/// changed would change what every constant below means.
const PAGE: PageGeometry = PageGeometry {
    width_pt: 200.0,
    height_pt: 200.0,
};

/// The centre of the middle square (PDF user space, 80,80 → 120,120).
///
/// The **middle** one deliberately: it is furthest from every page edge, so a
/// small error in the coordinate hop lands on paper rather than off-window,
/// and the check fails with "selected nothing" rather than with "the click
/// went outside the client area", which are different diagnoses.
const ON_A_SQUARE: (f64, f64) = (100.0, 100.0);

/// A point inside the form and outside every square (the gap between the first
/// square, which ends at 50, and the second, which starts at 80).
const IN_A_GAP: (f64, f64) = (65.0, 65.0);

/// The trace line this check reads.
const SELECTION_EVENT: &str = "canvas-selection";

/// The field added for this check — `object:N`, `leaf:N` or `none`.
const FIRST_FIELD: &str = "first";

/// See the module documentation.
pub struct AClickInsideAFormSelectsWhatIsDrawnThere;

impl Check for AClickInsideAFormSelectsWhatIsDrawnThere {
    fn name(&self) -> &'static str {
        "a_click_inside_a_form_selects_what_is_drawn_there"
    }

    fn defect(&self) -> &'static str {
        "a click inside a form XObject selects the form — a page-sized /BBox wins every click \
         at every point — so on a wrapped drawing nothing the operator can see is selectable, \
         and the outline hugging the page edge reads as 'the page is selected'"
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

/// Resolve a fixture under the engine repository's synthetic corpus.
///
/// ★ The path is derived, not configured. `D:\Dev\pdfcer` is READ-ONLY to this
/// project and its corpus is the only place this shape exists, so the check
/// reads from it and writes nowhere near it. `None` rather than a panic turns
/// a missing corpus into a SKIP with a reason instead of a crash mid-suite.
fn engine_fixture(rel: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new("D:/Dev/pdfcer/fixtures/synthetic").join(rel);
    path.is_file().then_some(path)
}

/// Run the sequence.
///
/// The three-way return is the SKIP/FAIL/PASS rule made structural: `Err` is a
/// precondition that was absent (SKIP), `Ok(Some(_))` is an assertion that did
/// not hold (FAIL), `Ok(None)` is a pass. An author who reaches for `?` gets a
/// SKIP, which is the safe default — the unsafe default would be a pass.
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
            "input is disabled (--no-input), and this check cannot be performed without \
             clicking. Reported as SKIPPED rather than passed: a check that did not run has \
             learned nothing.",
        ));
    }
    let fixture = engine_fixture(FIXTURE).ok_or_else(|| {
        Error::new(format!(
            "the engine's form fixture is not at D:/Dev/pdfcer/fixtures/synthetic/{FIXTURE}. \
             This check pins it and ignores --pdf: its subject is what a click inside a form \
             XObject selects, and on a document with no forms the honest answer is 'there was \
             nothing to descend into', which is neither a pass nor a defect."
        ))
    })?;
    let ui_rect = vocab.ui_rect_event.ok_or_else(|| {
        Error::new(format!(
            "the `{}` profile declares no ui-rect trace event, so the application cannot state \
             where its mode segments are and this check cannot leave Read mode — where a canvas \
             click on content is refused by design.",
            ctx.profile.name
        ))
    })?;
    report.note(format!(
        "fixture {} — one page object (a page-sized form) and three 40x40 squares painted from \
         inside it",
        fixture.display()
    ));

    // --- launch ------------------------------------------------------------
    let mut spec = LaunchSpec::new(&exe, ctx.out("form_selection.trace.txt"));
    spec.pdf = Some(fixture.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    // The shell's channel too: `click_mode_segment` reads `egui-shell`'s own
    // trace, and without this the mode click looks like a miss.
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();

    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.note(format!(
        "launched {} as pid {}",
        exe.display(),
        session.pid()
    ));
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let trace = session.trace()?;
    if !trace.started(vocab.start_event) {
        return Err(Error::new(format!(
            "the trace has no `{}` line, so the diagnostic switch did not reach the process and \
             this check has no oracle. Captured stderr is at {}.",
            vocab.start_event,
            session.trace_path().display()
        )));
    }

    // --- leave Read, where a content click is refused BY DESIGN ------------
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, "edit")?;
    report.note(
        "clicked the Edit mode segment first — the shell's default mode is Read, where a canvas \
         click on content is refused by design (DEFECTS.md D6). A check that skipped this step \
         once reported the mode gate as a selection defect",
    );

    // --- aim, in document space -------------------------------------------
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, PAGE, 0)?;
    report.note(format!(
        "canvas rect {:?} at zoom {:.3}",
        mapping.image_rect, mapping.zoom
    ));
    let frame = session.frame()?;

    // --- step 1: click a square inside the form ----------------------------
    let on_square = aim(&mapping, &frame, ON_A_SQUARE)?;
    report.note(format!(
        "the middle square's centre (page 0, {:.1}, {:.1}) -> screen ({}, {})",
        ON_A_SQUARE.0,
        ON_A_SQUARE.1,
        on_square.x(),
        on_square.y()
    ));
    driver.click_at(on_square)?;
    session.settle(12);

    let after = session.trace()?;
    let Some(first) = last_first(&after) else {
        return Err(Error::new(format!(
            "the click produced no `{SELECTION_EVENT} … {FIRST_FIELD}=` line, so the harness has \
             no oracle. Two readings it cannot distinguish: the click never reached the canvas, \
             or this build predates the `{FIRST_FIELD}=` field (added 2026-08-27 for this \
             check). Reported as SKIPPED rather than failed for exactly that reason. Trace: {}",
            session.trace_path().display()
        )));
    };
    report.note(format!(
        "after the click on the square: {FIRST_FIELD}={first}"
    ));

    if first.starts_with("object:") {
        return Ok(Some(format!(
            "THE DEFECT. The click landed on the middle square and the selection is \
             `{FIRST_FIELD}={first}` — a PAGE object. The only page object on this fixture is \
             the page-sized form, so this is the operator's report reproduced: 'when I click on \
             one of the objects all I get is the page selected'. A form's /BBox is a clipping \
             extent (8.10.1), not a claim about ink; `hit_test_point_deep` excludes forms as \
             candidates and answers with what is drawn inside them. Trace: {}",
            session.trace_path().display()
        )));
    }
    if !first.starts_with("leaf:") {
        return Ok(Some(format!(
            "the click on the middle square selected nothing (`{FIRST_FIELD}={first}`). The \
             square is 40x40 pt at the centre of a 200x200 pt page, so this is not a near-miss: \
             either the deep hit test is not reaching the leaf list, or the coordinate hop is \
             wrong. Read the `canvas rect` note above before assuming the first. Trace: {}",
            session.trace_path().display()
        )));
    }

    // --- step 2: click blank paper INSIDE the form -------------------------
    //
    // The half that forbids a fallback to the shallow query. See the module
    // docs: this is where the operator's original complaint would come back.
    let in_gap = aim(&mapping, &frame, IN_A_GAP)?;
    report.note(format!(
        "the gap between the first and second squares (page 0, {:.1}, {:.1}) -> screen ({}, {}) \
         — inside the form's page-sized /BBox and on no square",
        IN_A_GAP.0,
        IN_A_GAP.1,
        in_gap.x(),
        in_gap.y()
    ));
    driver.click_at(in_gap)?;
    session.settle(12);

    let after_gap = session.trace()?;
    let Some(gap_first) = last_first(&after_gap) else {
        return Err(Error::new(
            "the second click produced no selection line at all, so the harness cannot tell \
             'selected nothing' (which is the pass) from 'the click did not arrive'. The first \
             click DID produce one, so this is more likely a dropped event than a missing field.",
        ));
    };
    report.note(format!(
        "after the click in the gap: {FIRST_FIELD}={gap_first}"
    ));

    if gap_first != "none" {
        return Ok(Some(format!(
            "a click on blank paper INSIDE the page-sized form selected `{FIRST_FIELD}={gap_first}`, \
             and it must select nothing. This is the shape a 'fall back to the shallow hit test \
             when the deep one is empty' repair produces, and it is the operator's original \
             complaint restored for the case that produces it most often: on a page-sized form, \
             most of the page is gap. Trace: {}",
            session.trace_path().display()
        )));
    }

    Ok(None)
}

/// A page-space point, through the mapping and the window frame, to a desktop
/// point.
///
/// Its own function so the two call sites cannot hop differently — the class of
/// error `crate::coords` exists to prevent, and the one a literal screen
/// coordinate always is.
fn aim(mapping: &CanvasMapping, frame: &WindowFrame, point: (f64, f64)) -> Result<ScreenPoint> {
    let window = mapping.doc_to_window(DocPoint {
        page: 0,
        x: point.0,
        y: point.1,
    })?;
    Ok(frame.to_screen(window))
}

/// The `first=` value of the most recent `canvas-selection` line, if any.
///
/// ★ The **last** line rather than a count of new ones. `canvas-selection` is
/// emitted through `diag::trace_changed`, so a click producing the same
/// selection as the previous one emits nothing — a consumer that counted lines
/// would read a legitimate no-change as a dropped event. Reading the last line
/// asks the question this check actually has: *what is selected now?*
fn last_first(trace: &crate::trace::Trace) -> Option<String> {
    trace
        .last(SELECTION_EVENT)
        .and_then(|l| l.get(FIRST_FIELD))
        .map(str::to_owned)
}
