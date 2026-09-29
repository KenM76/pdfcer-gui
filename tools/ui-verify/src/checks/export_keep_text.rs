//! `export_keep_text` — **the Keep-text-as-text box in Export image decides
//! whether an SVG carries `<text>` elements or outlines, in both directions.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_keep_text.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, declared_or_in_overflow,
    frame_of, list,
};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const MODE: &str = "read";
/// Four pages whose text is set in embedded TrueType, which SVG can keep.
const FIXTURE: &str = "four-pages.pdf";
const WINDOW: &str = "dialog:export-image";
const SVG_RADIO: &str = "export-image.format.svg";
const KEEP_TEXT: &str = "export-image.keep-text";
const EXPORT: &str = "export-image.export";
/// `export-image-requested … format=… keep_text=0|1`, when Export is pressed.
const REQUESTED: &str = "export-image-requested";
/// `export-image-svg-text kept=N …`, written only when text was kept.
const SVG_TEXT: &str = "export-image-svg-text";
/// `export-image …`, after the file was written.
const WROTE: &str = "export-image";
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
/// The SVG element a kept run becomes.
const TEXT_ELEMENT: &str = "<text";

/// See the module documentation.
pub struct KeepTextDecidesTheSvg;

impl Check for KeepTextDecidesTheSvg {
    fn name(&self) -> &'static str {
        "export_keep_text"
    }

    fn defect(&self) -> &'static str {
        "Export image offers Keep text as text for SVG and the box does not reach the file: \
         ticked still writes outlines, or unticked still writes <text>"
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
            "input is disabled (--no-input). This check clicks the ribbon and a window.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;

    let target = ctx.out("export_keep_text.svg");
    let mut spec = LaunchSpec::new(&exe, ctx.out("export_keep_text.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), target.display().to_string()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    // The box is remembered across runs, so its starting state is whatever
    // the last export chose. Two exports, each flipping it, cover both
    // directions and leave the remembered choice where it was found.
    let mut previous = None;
    for round in 1..=2 {
        let _ = std::fs::remove_file(&target);
        if target.exists() {
            return Err(Error::new(format!(
                "cannot clear {} before export {round}.",
                target.display()
            )));
        }
        let mark = session.trace()?.events(REQUESTED).count();
        if let Some(failure) = export_once(&session, &driver, ui_rect, report, round)? {
            return Ok(Some(failure));
        }
        let trace = session.trace()?;
        let Some(requested) = trace.events(REQUESTED).nth(mark) else {
            return Ok(Some(format!(
                "export {round}: Export was pressed and no `{REQUESTED}` line followed."
            )));
        };
        report.note(format!("export {round}: `{}`", requested.raw));
        if requested.get("format") != Some("svg") {
            return Ok(Some(format!(
                "export {round}: the SVG radio was clicked and the plan says `{}`.",
                requested.raw
            )));
        }
        let keep = match requested.get("keep_text") {
            Some("1") => true,
            Some("0") => false,
            other => {
                return Ok(Some(format!(
                    "export {round}: the plan carries keep_text={other:?}: `{}`.",
                    requested.raw
                )));
            }
        };
        if previous == Some(keep) {
            return Ok(Some(format!(
                "export {round}: the box was clicked and keep_text stayed {}: the checkbox \
                 drew and did not bind.",
                u8::from(keep)
            )));
        }
        previous = Some(keep);
        if trace.last(WROTE).is_none() {
            return Ok(Some(format!(
                "export {round}: no `{WROTE}` line followed the request. Trace: {}.",
                session.trace_path().display()
            )));
        }
        let svg = std::fs::read_to_string(&target).map_err(|e| {
            Error::new(format!(
                "export {round}: the shell traced a write and {} cannot be read: {e}.",
                target.display()
            ))
        })?;
        let elements = svg.matches(TEXT_ELEMENT).count();
        report.note(format!(
            "export {round}: keep_text={} wrote {} bytes with {elements} `{TEXT_ELEMENT}` \
             element(s)",
            u8::from(keep),
            svg.len()
        ));
        if keep {
            let kept = trace
                .last(SVG_TEXT)
                .and_then(|l| l.get("kept").and_then(|v| v.parse::<usize>().ok()))
                .unwrap_or(0);
            if kept == 0 || elements == 0 {
                return Ok(Some(format!(
                    "export {round}: Keep text was ticked and the SVG holds {elements} \
                     `{TEXT_ELEMENT}` element(s), the engine reporting kept={kept}. The \
                     fixture's text is embedded TrueType, which SVG can keep."
                )));
            }
        } else if elements != 0 {
            return Ok(Some(format!(
                "export {round}: Keep text was unticked and the SVG still holds {elements} \
                 `{TEXT_ELEMENT}` element(s)."
            )));
        }
    }
    report
        .note("the box reached the file both ways, and the remembered choice is back where it was");
    Ok(None)
}

/// Open Export image, choose SVG, flip Keep text, press Export.
fn export_once(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    report: &mut CheckReport,
    round: u32,
) -> Result<Option<String>> {
    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, "ribbon.tab.file").ok_or_else(|| {
        Error::new(format!(
            "no `ribbon.tab.file` region in {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(14);
    let Some(item) =
        declared_or_in_overflow(session, driver, ui_rect, "ribbon.item.file.export_image")?
    else {
        return Ok(Some(
            "the File tab declares no `ribbon.item.file.export_image`.".to_owned(),
        ));
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(20);

    for (name, what) in [
        (SVG_RADIO, "the SVG radio"),
        (KEEP_TEXT, "the Keep text box"),
    ] {
        let trace = session.trace()?;
        if declared(&trace, ui_rect, WINDOW).is_none() {
            return Ok(Some(format!(
                "export {round}: `file.export_image` was clicked and no `{WINDOW}` appeared."
            )));
        }
        let Some(rect) = declared(&trace, ui_rect, name) else {
            return Ok(Some(format!(
                "export {round}: the window declares no `{name}` region, so {what} is not on \
                 offer. Regions declared: {}.",
                list(&declared_names(&trace, ui_rect, "export-image."))
            )));
        };
        driver.click_at(frame_of(session, &trace, ui_rect, name)?.declared_center(rect))?;
        session.settle(14);
    }
    let trace = session.trace()?;
    let Some(button) = declared(&trace, ui_rect, EXPORT) else {
        return Ok(Some(format!(
            "export {round}: the window declares no `{EXPORT}` region."
        )));
    };
    driver.click_at(frame_of(session, &trace, ui_rect, EXPORT)?.declared_center(button))?;
    session.settle(60);
    report.note(format!("export {round}: pressed"));
    Ok(None)
}
