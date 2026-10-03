//! `export_image_draws_the_chosen_background_and_standard` — Export image with a
//! typed background colour and a rendering standard writes a file drawn on that
//! colour, under that standard.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/export_image_standard.md`.

use crate::checks::driving::{self, SHELL_DIAG_ENV};
use crate::checks::{Check, CheckContext, CheckReport};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};

/// Off the desktop, so the check runs while the operator uses the machine.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
/// The colour typed into the background field.
const COLOUR: &str = "#3366cc";
/// The pure-K square under the calibrated default, and under every PDF/X preset.
const CALIBRATED_K: &str = "rgb(35,31,31)";
const NEUTRAL_K: &str = "rgb(0,0,0)";
const SAVE_PATH_ENV: &str = "PDFCER_DIAG_SAVE_PATH"; // ui-text-exempt: an environment variable name
const FILE_TAB: &str = "file"; // ui-text-exempt: a ribbon tab id
const COMMAND: &str = "ribbon.item.file.export_image"; // ui-text-exempt: a trace region name
const SVG: &str = "export-image.format.svg"; // ui-text-exempt: a trace region name
const TRANSPARENT: &str = "export-image.transparent"; // ui-text-exempt: a trace region name
const BACKGROUND: &str = "export-image.background"; // ui-text-exempt: a trace region name
const STANDARD: &str = "export-image.standard"; // ui-text-exempt: a trace region name
const PDF_X4: &str = "export-image.standard.pdf-x4"; // ui-text-exempt: a trace region name
const EXPORT: &str = "export-image.export"; // ui-text-exempt: a trace region name
const BODY: &str = "dialog:export-image"; // ui-text-exempt: a trace region name
const OPENED: &str = "export-image-open"; // ui-text-exempt: a trace event name
const REQUESTED: &str = "export-image-requested"; // ui-text-exempt: a trace event name
const WROTE: &str = "export-image"; // ui-text-exempt: a trace event name

/// See the module documentation.
pub struct ExportImageDrawsTheChosenBackgroundAndStandard;

/// A check's three answers.
enum Verdict {
    Pass,
    Fail(String),
    Skip(String),
}

impl Check for ExportImageDrawsTheChosenBackgroundAndStandard {
    fn name(&self) -> &'static str {
        "export_image_draws_the_chosen_background_and_standard"
    }

    fn defect(&self) -> &'static str {
        "Export image ignores the background colour typed or the rendering standard chosen"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        match assess(ctx, &mut report) {
            Ok(Verdict::Pass) => report.pass(),
            Ok(Verdict::Fail(why)) => report.fail(why),
            Ok(Verdict::Skip(why)) => report.skip(why),
            Err(why) => report.from_error(&why),
        }
    }
}

fn launch(
    ctx: &CheckContext,
    report: &mut CheckReport,
    chosen: &std::path::Path,
) -> Result<(Session, ScriptedPointer)> {
    let exe = ctx
        .resolve_exe()
        .ok_or_else(|| Error::new("no binary to drive. Pass --exe."))?;
    let viewport_env = ctx
        .profile
        .viewport_env
        .ok_or_else(|| Error::new("the profile has no viewport variable."))?;
    let pdf = driving::repo_fixture(
        "pure-k-square.pdf",
        "The oracle is that fixture's pure-K square, so --pdf is ignored.",
    )?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("export_image_standard.trace.txt"));
    spec.pdf = Some(pdf.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((SAVE_PATH_ENV.to_owned(), chosen.display().to_string()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("export_image_standard.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(pointer.path().to_path_buf());
    report.artifact(session.trace_path().to_path_buf());
    report.note(format!(
        "launched on {} as pid {}",
        pdf.display(),
        session.pid()
    ));
    session.settle(30);
    Ok((session, pointer))
}

/// Wheel the window's scrolling body until `name` lies between its top and
/// the pinned button row, so a click lands on it rather than on the clip.
fn reveal(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, name: &str) -> Result<()> {
    for _ in 0..12 {
        let trace = session.trace()?;
        let (Some((rect, viewport)), Some(body), Some(footer)) = (
            driving::declared_in(&trace, ui_rect, name),
            driving::declared(&trace, ui_rect, BODY),
            driving::declared(&trace, ui_rect, EXPORT),
        ) else {
            return Ok(());
        };
        let dy = if rect.max.y > footer.min.y {
            -3.0
        } else if rect.min.y < body.min.y {
            3.0
        } else {
            return Ok(());
        };
        pointer.wheel_in(
            session,
            viewport.as_deref(),
            WindowPoint::centre_of(body),
            dy,
        )?;
        session.settle(10);
    }
    Err(Error::new(format!(
        "`{name}` stayed outside the window's visible body after twelve wheel steps."
    )))
}

/// Click a declared region in whichever viewport declared it; returns that viewport.
fn click(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    name: &str,
) -> Result<Option<String>> {
    if name != EXPORT {
        reveal(session, pointer, ui_rect, name)?;
    }
    let trace = session.trace()?;
    let (rect, viewport) = driving::declared_in(&trace, ui_rect, name).ok_or_else(|| {
        Error::new(format!(
            "the application declared no `{name}` region. Regions beginning `export-image.`: {}.",
            driving::list(&driving::declared_names(&trace, ui_rect, "export-image."))
        ))
    })?;
    if !rect.is_substantial() {
        return Err(Error::new(format!(
            "`{name}` was declared at {rect:?}, which has no usable area to click."
        )));
    }
    pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
    session.settle(15);
    Ok(viewport)
}

/// Open File > Export image as SVG with transparency off; returns the
/// background the window opened with.
fn open_as_svg(session: &Session, pointer: &ScriptedPointer, ui_rect: &str) -> Result<String> {
    let opened_before = session.trace()?.events(OPENED).count();
    crate::checks::ocr::click_tab(session, pointer, ui_rect, FILE_TAB)?;
    let Some(item) = driving::declared_or_in_overflow(session, pointer, ui_rect, COMMAND)? else {
        return Err(Error::new(format!("the File tab declares no `{COMMAND}`.")));
    };
    crate::input::Click::click_rect(pointer, session, item)?;
    session.settle(20);
    let trace = session.trace()?;
    if trace.events(OPENED).count() == opened_before {
        return Err(Error::new(format!(
            "`{COMMAND}` was clicked and no new `{OPENED}` followed."
        )));
    }
    let opened = trace
        .last(OPENED)
        .map(|l| l.raw.clone())
        .unwrap_or_default();
    let field = |key: &str| {
        trace
            .last(OPENED)
            .and_then(|l| l.get(key))
            .map(str::to_owned)
    };
    click(session, pointer, ui_rect, SVG)?;
    if field("transparent").as_deref() == Some("1") {
        click(session, pointer, ui_rect, TRANSPARENT)?;
    }
    field("background").ok_or_else(|| {
        Error::new(format!(
            "`{OPENED}` carries no background field: `{opened}`."
        ))
    })
}

/// Press Export and return the SVG it wrote, with the `export-image` line.
fn export(
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
    chosen: &std::path::Path,
) -> Result<(String, String)> {
    let wrote_before = session.trace()?.events(WROTE).count();
    click(session, pointer, ui_rect, EXPORT)?;
    session.settle(40);
    let trace = session.trace()?;
    if trace.events(WROTE).count() == wrote_before {
        return Err(Error::new(format!(
            "Export was clicked and no new `{WROTE}` line followed. Requested: `{}`.",
            trace.last(REQUESTED).map_or("none", |l| l.raw.as_str())
        )));
    }
    let line = trace.last(WROTE).map(|l| l.raw.clone()).unwrap_or_default();
    let svg = std::fs::read_to_string(chosen)
        .map_err(|e| Error::new(format!("{} was not readable: {e}.", chosen.display())))?;
    Ok((svg, line))
}

/// Replace the background field's text with [`COLOUR`] and pick PDF/X-4.
fn choose(session: &Session, pointer: &ScriptedPointer, ui_rect: &str, typed: &str) -> Result<()> {
    let viewport = click(session, pointer, ui_rect, BACKGROUND)?;
    pointer.key(session, viewport.as_deref(), "End", None)?;
    for _ in 0..typed.chars().count() {
        pointer.key(session, viewport.as_deref(), "Backspace", None)?;
    }
    pointer.type_text(session, viewport.as_deref(), COLOUR)?;
    session.settle(10);
    for _ in 0..3 {
        click(session, pointer, ui_rect, STANDARD)?;
        if driving::declared(&session.trace()?, ui_rect, PDF_X4).is_some() {
            click(session, pointer, ui_rect, PDF_X4)?;
            return Ok(());
        }
    }
    Err(Error::new(format!(
        "the rendering list did not open with `{PDF_X4}`."
    )))
}

fn assess(ctx: &CheckContext, report: &mut CheckReport) -> Result<Verdict> {
    let chosen = ctx.out("image-standard.svg");
    let _ = std::fs::remove_file(&chosen);
    let (session, pointer) = launch(ctx, report, &chosen)?;
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");

    // The control: the operator's own settings must draw pure K as something
    // other than neutral black, or the standard cannot be told apart.
    open_as_svg(&session, &pointer, ui_rect)?;
    let (control, line) = export(&session, &pointer, ui_rect, &chosen)?;
    report.note(format!("control: `{line}`"));
    if !control.contains(CALIBRATED_K) {
        return Ok(Verdict::Skip(format!(
            "under the settings this profile carries, the pure-K square is not {CALIBRATED_K}, \
             so drawing it under PDF/X-4 changes nothing measurable. The control file: {}.",
            chosen.display()
        )));
    }
    let _ = std::fs::remove_file(&chosen);

    let typed = open_as_svg(&session, &pointer, ui_rect)?;
    choose(&session, &pointer, ui_rect, &typed)?;
    let (svg, line) = export(&session, &pointer, ui_rect, &chosen)?;
    report.note(format!("export: `{line}`"));
    report.artifact(chosen.clone());
    Ok(judge(&svg, &line))
}

/// The file sits on the typed colour and draws pure K as neutral black.
fn judge(svg: &str, line: &str) -> Verdict {
    let fill = format!("fill=\"{COLOUR}\"");
    if !svg.contains(&fill) {
        return Verdict::Fail(format!(
            "★★ the SVG carries no `{fill}` background, so the typed colour did not reach \
             the file. Its line: `{line}`."
        ));
    }
    if svg.contains(CALIBRATED_K) || !svg.contains(NEUTRAL_K) {
        return Verdict::Fail(format!(
            "★★★ PDF/X-4 was chosen and the pure-K square is still drawn under your own \
             settings (expected {NEUTRAL_K}, not {CALIBRATED_K}). Its line: `{line}`."
        ));
    }
    Verdict::Pass
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each arm of the judgement is reachable, and only the right file passes.
    #[test]
    fn only_a_file_on_the_colour_under_the_standard_passes() {
        let bg = format!("<rect fill=\"{COLOUR}\"/>");
        let good = format!("{bg}<path fill=\"{NEUTRAL_K}\"/>");
        assert!(matches!(judge(&good, ""), Verdict::Pass));
        let white = format!("<rect fill=\"#ffffff\"/><path fill=\"{NEUTRAL_K}\"/>");
        assert!(matches!(judge(&white, ""), Verdict::Fail(w) if w.contains("typed colour")));
        let own = format!("{bg}<path fill=\"{CALIBRATED_K}\"/>");
        assert!(matches!(judge(&own, ""), Verdict::Fail(w) if w.contains("PDF/X-4")));
    }
}
