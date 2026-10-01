//! `a_stand_in_preview_is_the_texts_size_and_says_why` — when an edit to
//! existing text cannot be previewed in the text's own font, the stand-in is
//! set at the text's own size times the zoom, and the status bar says why.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/preview_fallback.md`.

use crate::checks::driving::{SHELL_DIAG_ENV, repo_fixture};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "ocr-layer.pdf";
/// Edit mode with the Edit Text tool armed, so one click opens a caret.
const INVOKE: &str = "mode.edit,edit.text";
/// Inside the fixture's invisible run `HIDDEN RUN IN A VISIBLE STREAM`, set at
/// 10 pt from (72, 200), in PDF points.
const CLICK: (f64, f64) = (110.0, 203.0);
const RUN_PT: f64 = 10.0;
const FALLBACK: &str = "text-edit-preview-fallback"; // ui-text-exempt: a trace event name, never displayed
const REGION: &str = "status-group:preview-fallback"; // ui-text-exempt: a trace region name, never displayed

/// See the module documentation.
pub struct AStandInPreviewIsTheTextsSize;

impl Check for AStandInPreviewIsTheTextsSize {
    fn name(&self) -> &'static str {
        "a_stand_in_preview_is_the_texts_size_and_says_why"
    }

    fn defect(&self) -> &'static str {
        "an edit previewed in a stand-in font is drawn at a size of its own rather than the \
         text's, and nothing off the page says the font is a stand-in or why"
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

fn launch(ctx: &CheckContext) -> Result<(Session, ScriptedPointer, std::path::PathBuf)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let source = repo_fixture(FIXTURE, "Run tools/gen-ocr-layer-fixture.py.")?;
    let doc = ctx.out("preview-fallback-source.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("preview-fallback.trace.txt"));
    spec.pdf = Some(doc.clone());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("preview-fallback.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer, doc) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let at = mapping.doc_to_window(DocPoint::new(0, CLICK.0, CLICK.1))?;
    pointer.click(&session, at)?;
    session.settle(20);
    pointer.key(&session, None, "End", None)?;
    pointer.type_text(&session, None, "_")?;
    session.settle(25);
    let trace = session.trace()?;
    let fell_back = trace
        .events(FALLBACK)
        .filter(|l| l.get("reason") != Some("none"))
        .last()
        .map(|l| l.raw.clone());
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile names no ui-rect trace event."))?;
    let disclosed = trace.events(ui_rect).any(|l| l.get("name") == Some(REGION));
    pointer.key(&session, None, "Escape", None)?;
    session.settle(20);
    pointer.gone(&session)?;
    let path = session.trace_path().display().to_string();
    let Some(line) = fell_back else {
        return Ok(Some(format!(
            "★ typing into the invisible run produced no `{FALLBACK}` line naming a reason: \
             no caret opened, or the preview's fallback is not recorded. Trace: {path}."
        )));
    };
    if !line.contains("reason=invisible") {
        return Ok(Some(format!(
            "★ the stand-in preview gave the wrong reason for an invisible run: `{line}`. \
             Trace: {path}."
        )));
    }
    report.note(format!(
        "★ the fallback is recorded with its reason: `{line}`"
    ));
    let font_pt = line
        .split_whitespace()
        .find_map(|w| w.strip_prefix("font_pt="))
        .and_then(|v| v.parse::<f64>().ok());
    let expected = RUN_PT * f64::from(mapping.zoom);
    match font_pt {
        Some(f) if (f - expected).abs() <= expected * 0.03 => {
            report.note(format!(
                "★★ the stand-in is {f:.2} pt on screen, the run's {RUN_PT} pt at zoom {:.3}",
                mapping.zoom
            ));
        }
        other => {
            return Ok(Some(format!(
                "★★ the stand-in font is {other:?} screen points; the run's {RUN_PT} pt at zoom \
                 {:.3} is {expected:.2}. Trace: {path}.",
                mapping.zoom
            )));
        }
    }
    if !disclosed {
        return Ok(Some(format!(
            "★★★ the status bar drew no `{REGION}` while the stand-in was on screen, so \
             nothing says why the font differs. Trace: {path}."
        )));
    }
    report.note(format!("★★★ the status bar said why, in `{REGION}`"));
    Ok(None)
}
