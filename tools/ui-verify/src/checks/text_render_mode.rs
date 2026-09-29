//! `text_render_mode` — **sweep text, choose "Outlined" under Properties ›
//! Drawn as, and the file changes.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/text_render_mode.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, then the Properties panel, one per frame.
const INVOKE: &str = "mode.edit,file.properties";
/// The mode whose canvas selects page content.
const MODE: &str = "edit";
/// The Text section's region.
const SECTION_REGION: &str = "properties.text";
/// The "Drawn as" combo.
const COMBO_REGION: &str = "properties.text.render";
/// The popup entry for render mode 1, "Outlined".
const ENTRY_REGION: &str = "properties.text.render.1";
/// The `text-style-applied page=… change=… applied=… runs=…` summary line.
const STYLE_EVENT: &str = "text-style-applied";
/// The refusal line.
const DECLINED_EVENT: &str = "text-style-declined";
/// `StyleChange::RenderMode`'s trace label.
const CHANGE: &str = "render-mode";
/// Traced by `vector_edit` when the engine verb ran.
const APPLIED: &str = "format-text";
/// The sweep's own oracle.
const SELECTION_EVENT: &str = "canvas-text-selection";
/// How far to sweep along the baseline, in PDF points.
const SWEEP_PT: f64 = 60.0;
/// Scroll notches to spend looking for the combo below the panel's fold.
const SCROLL_ATTEMPTS: usize = 6;
/// `T`, the text-sweep tool.
const VK_T: u16 = 0x54;

/// See the module documentation.
pub struct ChoosingARenderModeReachesTheDocument;

impl Check for ChoosingARenderModeReachesTheDocument {
    fn name(&self) -> &'static str {
        "text_render_mode"
    }

    fn defect(&self) -> &'static str {
        "Properties › Drawn as offers eight ways to draw the swept text, and choosing one \
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

/// Poll until the restyle reports either way; answer the elapsed time.
fn wait_for_verdict(session: &Session) -> Result<u128> {
    const CEILING_MS: u128 = 20_000;
    let started = std::time::Instant::now();
    loop {
        session.settle(4);
        let trace = session.trace()?;
        if trace.last(STYLE_EVENT).is_some()
            || trace.last(DECLINED_EVENT).is_some()
            || started.elapsed().as_millis() > CEILING_MS
        {
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
    // `--pdf` and `--doc-point` are ignored: the gesture needs a point IN
    // text, which `fixture::text_point_target` owns.
    let (pdf, target) = crate::fixture::text_point_target();
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the committed text fixture is not at {}: a broken checkout.",
            pdf.display()
        )));
    }
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check sweeps text and picks from a list.",
        ));
    }
    let ui_rect = vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!("cannot read a page size from {}.", pdf.display()))
        })?,
    };

    let mut spec = LaunchSpec::new(&exe, ctx.out("text_render_mode.trace.txt"));
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

    // --- 1: Edit mode, then sweep along the baseline ---------------------
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);
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
            "the sweep selected no text, so there is nothing to restyle. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!("the sweep selected {swept} character(s)"));

    // --- 2: find the combo, scrolling Properties as an operator would ----
    let mut combo = None;
    for _ in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, COMBO_REGION) {
            combo = Some(rect);
            break;
        }
        let Some(section) = driving::declared(&trace, ui_rect, SECTION_REGION) else {
            return Ok(Some(format!(
                "{swept} character(s) are selected and no `{SECTION_REGION}` region is on \
                 screen, so Drawn as cannot be reached. `restyle_text` diagnoses this link in \
                 detail; run it. Trace: {}.",
                session.trace_path().display()
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(section), -1)?;
        session.settle(12);
    }
    let Some(combo) = combo else {
        return Ok(Some(format!(
            "the Text section drew and no `{COMBO_REGION}` region appeared after \
             {SCROLL_ATTEMPTS} scroll notches: Drawn as is missing or unreachable. Trace: {}.",
            session.trace_path().display()
        )));
    };

    // --- 3: open it, then pick "Outlined" ---------------------------------
    driver.click_at(session.frame()?.declared_center(combo))?;
    session.settle(16);
    let trace = session.trace()?;
    let Some(entry) = driving::declared(&trace, ui_rect, ENTRY_REGION) else {
        let shot = ctx.out("text_render_mode.no-popup.png");
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "Drawn as was clicked and no `{ENTRY_REGION}` region followed: the list did not \
             open, or opened off screen. The screenshot beside this report shows which. \
             Trace: {}.",
            session.trace_path().display()
        )));
    };
    driver.click_at(session.frame()?.declared_center(entry))?;
    let waited = wait_for_verdict(&session)?;
    report.note(format!("the restyle took {waited} ms"));

    // --- 4: the verdict -----------------------------------------------------
    let trace = session.trace()?;
    if let Some(declined) = trace.events(DECLINED_EVENT).last() {
        return Ok(Some(format!(
            "\"Outlined\" was chosen and the restyle declined: `{}`. Outlining needs no face \
             the page lacks, so a refusal here is the pin or the engine. Trace: {}.",
            declined.raw,
            session.trace_path().display()
        )));
    }
    let Some(applied) = trace.events(STYLE_EVENT).last() else {
        return Ok(Some(format!(
            "\"Outlined\" was clicked and neither `{STYLE_EVENT}` nor `{DECLINED_EVENT}` \
             followed: the pick raised no action, or the click missed the entry. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the pick committed: `{}`", applied.raw));
    if applied.get("change") != Some(CHANGE) {
        return Ok(Some(format!(
            "the restyle that ran was not a render-mode change: `{}`. The entry raised the \
             wrong action.",
            applied.raw
        )));
    }
    let n: usize = applied
        .get("applied")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        return Ok(Some(format!(
            "the render-mode change reported `applied=0`: every run refused silently. `{}`.",
            applied.raw
        )));
    }
    if trace.last(APPLIED).is_none() {
        return Ok(Some(format!(
            "`{}` was computed and no `{APPLIED}` line followed, so nothing reached the \
             document. Trace: {}.",
            applied.raw,
            session.trace_path().display()
        )));
    }
    report.note("the swept text is now drawn outlined, through `format_text`");
    Ok(None)
}
