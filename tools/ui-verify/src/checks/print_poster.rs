//! `poster_printing_tiles_the_page_across_sheets` — O238.
//!
//! # What it asserts
//!
//! 1. **The Pages tab offers Poster** — a `print.poster.mode` region.
//! 2. **Choosing it tiles the page** — the `print-plan` line's `poster_tiles`
//!    becomes non-zero and `sheets` becomes at least that.
//! 3. **The tile scale drives the tiling** — typing a larger scale into
//!    `print.poster.scale` raises `poster_tiles`.
//! 4. **The Enter that commits the scale does not print.** A field gives up
//!    focus on that Enter before the footer runs, so a guard asking only
//!    whether a field is focused let it through and this check spooled 42
//!    sheets to a real printer. No `print-spool` line may follow it.
//!
//! # What it does NOT establish
//!
//! That the sheets assemble into the drawing. Two captures are written, before
//! and after the scale change, for a person to look at; no pixel is asserted.

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
const PAGE: &str = "print.preview.page";
const MODE: &str = "print.poster.mode";
const SCALE: &str = "print.poster.scale";
const PLAN: &str = "print-plan";
const SPOOL: &str = "print-spool";
/// The tile scale typed in step 3, percent.
const BIGGER: &str = "400";

/// See the module documentation.
pub struct PosterPrintingTilesThePageAcrossSheets;

impl Check for PosterPrintingTilesThePageAcrossSheets {
    fn name(&self) -> &'static str {
        "poster_printing_tiles_the_page_across_sheets"
    }

    fn defect(&self) -> &'static str {
        "the print dialog has no poster mode, or choosing it does not split the page across \
         sheets, or its tile scale does not change how many"
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

/// `poster_tiles` and `sheets` from the last `print-plan` line.
fn counts(session: &Session) -> Result<(usize, usize, String)> {
    let trace = session.trace()?;
    let Some(line) = trace.last(PLAN) else {
        return Err(Error::new("the print dialog emitted no `print-plan` line."));
    };
    let tiles = line
        .get("poster_tiles")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let sheets = line
        .get("sheets")
        .and_then(|v| {
            v.trim_start_matches("Some(")
                .trim_end_matches(')')
                .parse()
                .ok()
        })
        .unwrap_or(0);
    Ok((tiles, sheets, line.raw.clone()))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input), and this check's subject is a click and a typed scale.",
        ));
    }
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let pdf = ctx
        .pdf
        .clone()
        .ok_or_else(|| Error::new("no --pdf. A poster needs a page to tile."))?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("print-poster.trace.txt"));
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

    if stable_rect(&session, ui_rect, PAGE, 8)?.is_none() {
        let trace = session.trace()?;
        return Err(Error::new(format!(
            "the print dialog drew no preview page. `print_dialog` diagnoses a dialog that did \
             not open. Regions beginning `print`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "print")),
            session.trace_path().display()
        )));
    }

    let trace = session.trace()?;
    let Some(mode) = declared(&trace, ui_rect, MODE) else {
        return Ok(Some(format!(
            "the Pages tab offers no Poster choice: no `{MODE}` region. Regions beginning \
             `print`: {}. Trace: {}.",
            list(&declared_names(&trace, ui_rect, "print")),
            session.trace_path().display()
        )));
    };
    driver.click_at(frame_of(&session, &trace, ui_rect, MODE)?.declared_center(mode))?;
    session.settle(30);
    let (tiles, sheets, line) = counts(&session)?;
    if tiles == 0 || sheets < tiles {
        return Ok(Some(format!(
            "Poster was chosen and the job was not tiled: `{line}`. `dialogs::print::poster::apply` \
             replaces each tiled page by its sheets. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "· Poster at 100 %: {tiles} tile sheets of {sheets}"
    ));
    let before = ctx.out("print-poster-100.png");
    if crate::capture::window_to_png(&session, &before).is_ok() {
        report.artifact(before);
    }

    let trace = session.trace()?;
    let Some(scale) = declared(&trace, ui_rect, SCALE) else {
        return Ok(Some(format!(
            "Poster mode is on and draws no tile-scale field: no `{SCALE}` region. Trace: {}.",
            session.trace_path().display()
        )));
    };
    driver.click_at(frame_of(&session, &trace, ui_rect, SCALE)?.declared_center(scale))?;
    session.settle(8);
    driver.press_chord(&[vk::CONTROL], vk::A)?;
    driver.type_ascii(BIGGER)?;
    driver.press(vk::ENTER)?;
    session.settle(30);
    if let Some(spooled) = session.trace()?.last(SPOOL) {
        return Ok(Some(format!(
            "the Enter that committed the tile scale also pressed Print: `{}`. \
             `dialogs::host::Host::footer` must refuse the Enter a field surrendered focus on. \
             Trace: {}.",
            spooled.raw,
            session.trace_path().display()
        )));
    }
    let (bigger, _, line) = counts(&session)?;
    let after = ctx.out("print-poster-400.png");
    if crate::capture::window_to_png(&session, &after).is_ok() {
        report.artifact(after);
    }
    if bigger <= tiles {
        return Ok(Some(format!(
            "the tile scale was set to {BIGGER} % and the poster did not grow: {tiles} tile sheets \
             before, {bigger} after: `{line}`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    report.note(format!(
        "★ Poster tiles the page, and {BIGGER} % takes {bigger} sheets where 100 % took {tiles}; \
         the Enter that committed it did not print"
    ));
    Ok(None)
}
