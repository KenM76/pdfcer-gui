//! `text_render_mode` — **sweep text, choose "Outlined" under Properties ›
//! Drawn as, and the file changes.** Also the driver `text_spacing` runs with
//! its own [`Restyle`].
//!
//! Design and rationale: `docs/modules/ui-verify/checks/text_render_mode.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV, click_mode_segment};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::LRect;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Edit mode, then the Properties panel, one per frame.
const INVOKE: &str = "mode.edit,file.properties";
/// The mode whose canvas selects page content.
const MODE: &str = "edit";
/// The Text section's region.
const SECTION_REGION: &str = "properties.text";
/// The `text-style-applied page=… change=… applied=… runs=…` summary line.
const STYLE_EVENT: &str = "text-style-applied";
/// The refusal line.
const DECLINED_EVENT: &str = "text-style-declined";
/// Traced by `vector_edit` when the engine verb ran.
const APPLIED: &str = "format-text";
/// The sweep's own oracle.
const SELECTION_EVENT: &str = "canvas-text-selection";
/// How far to sweep along the baseline, in PDF points.
const SWEEP_PT: f64 = 60.0;
/// Scroll notches to spend looking for the control below the panel's fold.
const SCROLL_ATTEMPTS: usize = 6;
/// `T`, the text-sweep tool.
const VK_T: u16 = 0x54;

/// One Properties › Text control to reach and press on swept text.
pub(crate) struct Restyle {
    /// Output-file stem.
    pub stem: &'static str,
    /// The control that opens (a combo or a fold header).
    pub opener: &'static str,
    /// The entry or button inside it.
    pub entry: &'static str,
    /// The `change=` label the restyle must report.
    pub change: &'static str,
    /// The control's name, as failure messages say it.
    pub control: &'static str,
    /// What the check picks, as failure messages say it.
    pub pick: &'static str,
    /// Why a refusal of this pick on the fixture indicts the pin or engine.
    pub refusal: &'static str,
    /// The pass note.
    pub done: &'static str,
}

const OUTLINED: Restyle = Restyle {
    stem: "text_render_mode",
    opener: "properties.text.render",
    entry: "properties.text.render.1",
    change: "render-mode",
    control: "Drawn as",
    pick: "\"Outlined\"",
    refusal: "Outlining needs no face the page lacks",
    done: "the swept text is now drawn outlined, through `format_text`",
};

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
        match drive(ctx, &mut report, &OUTLINED) {
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

/// Sweep the fixture's text, press `r`, and judge what reached the document.
pub(crate) fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    r: &Restyle,
) -> Result<Option<String>> {
    let pdf = crate::fixture::text_point_target().0;
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the committed text fixture is not at {}: a broken checkout.",
            pdf.display()
        )));
    }
    let (session, page, target) = launch(ctx, report, r)?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let driver = Driver::new(session.window());
    let swept = sweep(ctx, &session, &driver, ui_rect, page, &target)?;
    report.note(format!("the sweep selected {swept} character(s)"));
    let opener = match find_opener(&session, &driver, ui_rect, r, swept)? {
        Ok(rect) => rect,
        Err(failure) => return Ok(Some(failure)),
    };
    driver.click_at(session.frame()?.declared_center(opener))?;
    session.settle(16);
    let trace = session.trace()?;
    let Some(entry) = driving::declared(&trace, ui_rect, r.entry) else {
        let shot = ctx.out(&format!("{}.no-entry.png", r.stem));
        if crate::capture::window_to_png(&session, &shot).is_ok() {
            report.artifact(shot);
        }
        return Ok(Some(format!(
            "{} was clicked and no `{}` region followed: it did not open, or opened off \
             screen. The screenshot beside this report shows which. Trace: {}.",
            r.control,
            r.entry,
            session.trace_path().display()
        )));
    };
    driver.click_at(session.frame()?.declared_center(entry))?;
    let waited = wait_for_verdict(&session)?;
    report.note(format!("the restyle took {waited} ms"));
    verdict(&session, report, r)
}

/// Launch on the committed text fixture in Edit mode with Properties open.
fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    r: &Restyle,
) -> Result<(Session, PageGeometry, DocPoint)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    // `--pdf` and `--doc-point` are ignored: the gesture needs a point IN
    // text, which `fixture::text_point_target` owns.
    let (pdf, target) = crate::fixture::text_point_target();
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check sweeps text and presses a control.",
        ));
    }
    let page: PageGeometry = match ctx.page_size {
        Some((w, h)) => PageGeometry {
            width_pt: w,
            height_pt: h,
        },
        None => crate::fixture::page_geometry(&pdf).ok_or_else(|| {
            Error::new(format!("cannot read a page size from {}.", pdf.display()))
        })?,
    };
    let mut spec = LaunchSpec::new(&exe, ctx.out(&format!("{}.trace.txt", r.stem)));
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
    Ok((session, page, target))
}

/// Edit mode, then sweep along the baseline; answer the characters selected.
fn sweep(
    ctx: &CheckContext,
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    page: PageGeometry,
    target: &DocPoint,
) -> Result<usize> {
    click_mode_segment(session, driver, ui_rect, MODE)?;
    session.settle(20);
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, target.page)?;
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
    let swept = session
        .trace()?
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
    Ok(swept)
}

/// Find the opener, scrolling Properties as an operator would.
fn find_opener(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    r: &Restyle,
    swept: usize,
) -> Result<std::result::Result<LRect, String>> {
    for _ in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, r.opener) {
            return Ok(Ok(rect));
        }
        let Some(section) = driving::declared(&trace, ui_rect, SECTION_REGION) else {
            return Ok(Err(format!(
                "{swept} character(s) are selected and no `{SECTION_REGION}` region is on \
                 screen, so {} cannot be reached. `restyle_text` diagnoses this link in \
                 detail; run it. Trace: {}.",
                r.control,
                session.trace_path().display()
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(section), -1)?;
        session.settle(12);
    }
    Ok(Err(format!(
        "the Text section drew and no `{}` region appeared after {SCROLL_ATTEMPTS} scroll \
         notches: {} is missing or unreachable. Trace: {}.",
        r.opener,
        r.control,
        session.trace_path().display()
    )))
}

/// The restyle ran, as `r.change`, on at least one run, and reached the engine.
fn verdict(session: &Session, report: &mut CheckReport, r: &Restyle) -> Result<Option<String>> {
    let trace = session.trace()?;
    if let Some(declined) = trace.events(DECLINED_EVENT).last() {
        return Ok(Some(format!(
            "{} was pressed and the restyle declined: `{}`. {}, so a refusal here is the pin \
             or the engine. Trace: {}.",
            r.pick,
            declined.raw,
            r.refusal,
            session.trace_path().display()
        )));
    }
    let Some(applied) = trace.events(STYLE_EVENT).last() else {
        return Ok(Some(format!(
            "{} was clicked and neither `{STYLE_EVENT}` nor `{DECLINED_EVENT}` followed: the \
             press raised no action, or the click missed it. Trace: {}.",
            r.pick,
            session.trace_path().display()
        )));
    };
    report.note(format!("the press committed: `{}`", applied.raw));
    if applied.get("change") != Some(r.change) {
        return Ok(Some(format!(
            "the restyle that ran was not a {} change: `{}`. The control raised the wrong \
             action.",
            r.change, applied.raw
        )));
    }
    let n: usize = applied
        .get("applied")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        return Ok(Some(format!(
            "the {} change reported `applied=0`: every run refused silently. `{}`.",
            r.change, applied.raw
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
    report.note(r.done);
    Ok(None)
}
