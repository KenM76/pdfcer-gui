//! `run_width` — **select one line of text, type a width under Properties ›
//! Fit to width, and the file changes.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/run_width.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

/// Edit mode, then the Properties panel, one per frame.
const INVOKE: &str = "mode.edit,file.properties";
/// The mode whose canvas selects page content.
const MODE: &str = "edit";
/// Shared with `move_line_of_text`: its second line is one run that states
/// its own position.
const FIXTURE: &str = "inherited-runs.pdf";
/// That line, PDF user space (y up).
const AIM: (f64, f64) = (128.0, 664.0);
/// The width field's region.
const FIELD_REGION: &str = "properties.text.run-width";
/// The width typed, in points.
const TYPED: &str = "150";
/// The ladder line and the rung a Points-tool click on a line lands on.
const SELECTION_EVENT: &str = "canvas-selection";
const PART_LEVEL: &str = "Part";
/// `text-run-width page=… object=… run=… width=… detail=applied|<refusal>`,
/// written by the apply arm after the engine call. The funnel's own success
/// line shares the head and carries no `detail=`.
const RESULT_EVENT: &str = "text-run-width";
/// Scroll notches to spend looking for the field below the panel's fold.
const SCROLL_ATTEMPTS: usize = 6;

/// See the module documentation.
pub struct TypingARunWidthReachesTheDocument;

impl Check for TypingARunWidthReachesTheDocument {
    fn name(&self) -> &'static str {
        "run_width"
    }

    fn defect(&self) -> &'static str {
        "Properties › Fit to width shows a line's width and accepts a new one, and typing it \
         commits nothing to the document"
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

/// The apply arm's own line, told from the funnel's by its `detail=`.
fn verdict(trace: &crate::trace::Trace) -> Option<crate::trace::TraceLine> {
    trace
        .events(RESULT_EVENT)
        .filter(|l| l.get("detail").is_some())
        .last()
        .cloned()
}

/// Poll until the apply arm reports; answer the elapsed time.
fn wait_for_verdict(session: &Session) -> Result<u128> {
    const CEILING_MS: u128 = 20_000;
    let started = std::time::Instant::now();
    loop {
        session.settle(4);
        if verdict(&session.trace()?).is_some() || started.elapsed().as_millis() > CEILING_MS {
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
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the committed fixture is not at {}: a broken checkout.",
            pdf.display()
        )));
    }
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks a line and types a width.",
        ));
    }
    let ui_rect = vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new(format!("cannot read a page size from {}.", pdf.display())))?;
    let page = PageGeometry {
        width_pt: page.width_pt,
        height_pt: page.height_pt,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("run_width.trace.txt"));
    spec.pdf = Some(pdf);
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
    session.settle(40);
    let driver = Driver::new(session.window());

    // --- 1: Edit mode; the Points tool; click the line ---------------------
    // The Points tool is what lands a click on a text line at the Part rung;
    // the arrow tool selects the whole block.
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, 0)?;
    let frame = session.frame()?;
    driver.press(vk::A)?;
    session.settle(10);
    driver.click_at(frame.to_screen(mapping.doc_to_window(DocPoint::new(0, AIM.0, AIM.1))?))?;
    session.settle(20);
    let trace = session.trace()?;
    let level = trace
        .events(SELECTION_EVENT)
        .last()
        .and_then(|l| l.get("level").map(str::to_owned));
    if level.as_deref() != Some(PART_LEVEL) {
        return Err(Error::new(format!(
            "the click on the line left the ladder at {level:?}, not `{PART_LEVEL}`, so Fit to \
             width has nothing to address. `move_line_of_text` owns this link. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // --- 2: find the field, scrolling Properties as an operator would ------
    let mut field = None;
    for _ in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, FIELD_REGION) {
            field = Some(rect);
            break;
        }
        let Some(body) = driving::declared(&trace, ui_rect, "dock.body.file.properties") else {
            break;
        };
        driver.scroll_at(session.frame()?.declared_center(body), -1)?;
        session.settle(12);
    }
    let Some(field) = field else {
        let shot = ctx.out("run_width.no-field.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "one line is selected at the Part rung and no `{FIELD_REGION}` region is on \
             screen: Fit to width is missing or unreachable. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- 3: type a width ---------------------------------------------------
    // A click on a DragValue without dragging opens it for typing; Enter
    // commits by losing focus.
    driver.click_at(session.frame()?.declared_center(field))?;
    session.settle(10);
    driver.press_chord(&[vk::CONTROL], vk::A)?;
    driver.type_ascii(TYPED)?;
    driver.press(vk::ENTER)?;
    let waited = wait_for_verdict(&session)?;
    report.note(format!("typed {TYPED} pt; the verdict took {waited} ms"));

    // --- 4: the verdict ----------------------------------------------------
    let trace = session.trace()?;
    let Some(line) = verdict(&trace) else {
        let shot = ctx.out("run_width.no-effect.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "a width was typed and Enter pressed, and no `{RESULT_EVENT}` line followed: the \
             field raised no action. The screenshot beside this report shows the field. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the apply arm reported: `{}`", line.raw));
    if line.get("detail") != Some("applied") {
        return Ok(Some(format!(
            "the width reached the engine and was refused: `{}`. Trace: {}.",
            line.raw,
            session.trace_path().display()
        )));
    }
    let width: f64 = line
        .get("width")
        .and_then(|v| v.parse().ok())
        .unwrap_or(f64::NAN);
    if (width - 150.0).abs() > 1e-6 {
        return Ok(Some(format!(
            "the engine was asked for width {width}, not the {TYPED} typed: `{}`.",
            line.raw
        )));
    }
    report.note("the line was fitted to the typed width, through the engine");
    Ok(None)
}
