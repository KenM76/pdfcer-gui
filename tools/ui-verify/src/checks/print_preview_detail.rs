//! `the_zoomed_print_preview_is_as_sharp_as_the_print` — O235, with O236/O237
//! on the way in.
//!
//! # What it asserts
//!
//! 1. **The resolution field is on the Pages tab when the dialog opens**
//!    (`print.resolution` declared). The field used to be drawn only when the
//!    job was capped, so after the operator set a value the printer could meet
//!    it vanished on the next open (O237), and it lived under Comments (O236).
//! 2. **Zoomed in, the preview re-renders the visible region finer than its
//!    base raster, and never finer than the print** — the
//!    `print-preview-detail` line reaches `shown_scale > base_scale` and
//!    `shown_scale <= print_scale`.
//!
//! The first half is what makes the second meaningful: a detail raster
//! appearing at all proves the zoom reached the preview; the ceiling proves it
//! shows the print's pixels and not invented ones.
//!
//! # What it does NOT establish
//!
//! That it looks sharp. Two captures are written, before and after, for a
//! person to compare; no pixel is asserted.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_names, frame_of, list, stable_rect,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::sys::vk;

const INVOKE: &str = "file.print";
const INVOKE_ENV: &str = "PDFCER_DIAG_INVOKE";
const RESOLUTION: &str = "print.resolution";
const PAGE: &str = "print.preview.page";
const PAPER: &str = "print.paper";
const DETAIL: &str = "print-preview-detail";
/// Ctrl+wheel notches toward the page. The preview's zoom step is ×1.25, so
/// twelve notches is ×14 — past the base raster on any sheet size.
const NOTCHES: usize = 12;
/// How many settle rounds to wait for the off-thread render, 30 frames each.
const WAIT_ROUNDS: usize = 20;

/// See the module documentation.
pub struct TheZoomedPrintPreviewIsAsSharpAsThePrint;

impl Check for TheZoomedPrintPreviewIsAsSharpAsThePrint {
    fn name(&self) -> &'static str {
        "the_zoomed_print_preview_is_as_sharp_as_the_print"
    }

    fn defect(&self) -> &'static str {
        "zoomed in, the print preview magnifies a fixed 150-dpi raster and goes blurry, so the \
         operator cannot tell what the print will look like; and the resolution field is missing \
         whenever the job is not capped"
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

fn number(line: &crate::trace::TraceLine, key: &str) -> f32 {
    line.get(key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(f32::NAN)
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check's subject is a Ctrl+wheel zoom.",
        ));
    }
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. The preview has nothing to zoom without a page."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("print-preview-detail.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env.push((INVOKE_ENV.to_owned(), INVOKE.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(60);
    let driver = Driver::new(session.window());

    let trace = session.trace()?;
    let Some(page) = stable_rect(&session, ui_rect, PAGE, 8)? else {
        return Err(Error::new(format!(
            "the print dialog drew no preview page, so there is nothing to zoom. `print_dialog` \
             diagnoses a dialog that did not open. Regions beginning `print`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "print")),
            session.trace_path().display()
        )));
    };
    let before = ctx.out("print-preview-detail-before.png");
    if crate::capture::window_to_png(&session, &before).is_ok() {
        report.artifact(before);
    }

    // The Pages tab is taller than the dialog's default height, so the field
    // sits below the fold of the options scroll area and is published only
    // once scrolled into view. Scroll the way the operator would: the wheel
    // over the options, aimed at the paper combo, which ignores the wheel.
    let mut trace = trace;
    if declared(&trace, ui_rect, RESOLUTION).is_none()
        && let Some(paper) = declared(&trace, ui_rect, PAPER)
    {
        let at = frame_of(&session, &trace, ui_rect, PAPER)?.declared_center(paper);
        driver.scroll_at(at, -10)?;
        session.settle(30);
        trace = session.trace()?;
        report.note("· scrolled the options down to reach the resolution field");
    }
    if declared(&trace, ui_rect, RESOLUTION).is_none() {
        return Ok(Some(format!(
            "the print dialog's Pages tab has NO resolution field, even scrolled to its end: no `{RESOLUTION}` \
             region. `tabs::resolution` must draw the field on every open, capped or not. \
             Regions beginning `print`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "print")),
            session.trace_path().display()
        )));
    }
    report.note("· the resolution field is on the Pages tab");

    let frame = frame_of(&session, &trace, ui_rect, PAGE)?;
    driver.scroll_at_held(frame.declared_center(page), &[vk::CONTROL], 1, NOTCHES)?;

    let mut last = None;
    for _ in 0..WAIT_ROUNDS {
        session.settle(30);
        let trace = session.trace()?;
        last = trace.events(DETAIL).last().map(|l| l.raw.clone());
        if let Some(line) = trace.events(DETAIL).last()
            && number(line, "shown_scale") > 0.0
        {
            let (base, print, shown) = (
                number(line, "base_scale"),
                number(line, "print_scale"),
                number(line, "shown_scale"),
            );
            let after = ctx.out("print-preview-detail-zoomed.png");
            if crate::capture::window_to_png(&session, &after).is_ok() {
                report.artifact(after);
            }
            if shown.is_nan() || shown <= base {
                return Ok(Some(format!(
                    "a detail raster was drawn no finer than the base one: `{}`. Trace: {}.",
                    line.raw,
                    session.trace_path().display()
                )));
            }
            if shown > print * 1.001 {
                return Ok(Some(format!(
                    "the zoomed preview was rendered FINER than the print will be: `{}` — it shows \
                     detail the paper will not get. `detail::wanted_scale` caps at print_scale. \
                     Trace: {}.",
                    line.raw,
                    session.trace_path().display()
                )));
            }
            report.note(format!(
                "★ zoomed in, the preview re-rendered at {shown:.2} px/pt against a base of \
                 {base:.2} and a print density of {print:.2}: `{}`",
                line.raw
            ));
            return Ok(None);
        }
    }
    Ok(Some(format!(
        "{NOTCHES} Ctrl+wheel notches over the preview page and no detail raster arrived within \
         {WAIT_ROUNDS} settles. Last `{DETAIL}` line: {}. Trace: {}.",
        last.unwrap_or_else(|| "none".to_owned()),
        session.trace_path().display()
    )))
}
