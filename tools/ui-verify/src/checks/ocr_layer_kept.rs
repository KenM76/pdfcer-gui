//! `an_edited_ocr_layer_stays_a_layer` — editing a recognised word inside an
//! OCR layer pdfcer wrote keeps the layer's marked section: the saved word is
//! still inside `/pdfc_OCR`, and Remove OCR text afterwards still finds both of
//! the document's layers. Run with the scripted pointer in a window placed off
//! the desktop, on a copy of `fixtures/ocr-layers.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_layer_kept.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, declared, declared_in, declared_names, list, repo_fixture,
};
use crate::checks::invisible_text_scripted::streams;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

/// Off every monitor, unfocused: the drive needs neither mouse nor keyboard.
const OFFSCREEN: &str = "-4200,-4200,1400,900";
const FIXTURE: &str = "ocr-layers.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/ocr-layers.PROVENANCE.py`.";
/// Edit mode, the OCR layer shown, the Edit Text tool armed.
const INVOKE: &str = "mode.edit,view.ocr_layer,edit.text"; // ui-text-exempt: command ids, never displayed
/// Inside `recognised one`, page 1's layer word: 12 pt Helvetica from
/// (72, 700), in PDF points.
const CLICK: (f64, f64) = (90.0, 703.0);
/// Letters typed at the end of the line; none of them is in the fixture.
const TYPED: &str = "XQ";
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.remove_ocr"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const APPLIED: &str = "remove-ocr-layers-applied"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct AnEditedOcrLayerStaysALayer;

impl Check for AnEditedOcrLayerStaysALayer {
    fn name(&self) -> &'static str {
        "an_edited_ocr_layer_stays_a_layer"
    }

    fn defect(&self) -> &'static str {
        "editing a word of an OCR layer pdfcer wrote folds the layer into the page's content: \
         the edited word is no longer inside the layer's marked section, and Remove OCR text \
         no longer finds that page's layer"
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
    let source = repo_fixture(FIXTURE, METHOD)?;
    let doc = ctx.out("ocr-layer-kept.pdf");
    std::fs::copy(&source, &doc).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("ocr-layer-kept.trace.txt"));
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
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("ocr-layer-kept.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, doc))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let (session, pointer, doc) = launch(ctx)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    report.note(format!("launched as pid {}", session.pid()));
    session.settle(45);
    let outcome = edit_save_remove(ctx, report, &session, &pointer, &doc);
    let parked = pointer.gone(&session);
    let outcome = outcome?;
    parked?;
    Ok(outcome)
}

fn edit_save_remove(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    doc: &std::path::Path,
) -> Result<Option<String>> {
    let page = crate::fixture::page_geometry(doc)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, CLICK.0, CLICK.1))?,
    )?;
    session.settle(20);
    pointer.key(session, None, "End", None)?;
    pointer.type_text(session, None, TYPED)?;
    session.settle(30);
    pointer.key(session, None, "Escape", None)?;
    session.settle(30);
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    let path = session.trace_path().display().to_string();
    if !session
        .trace()?
        .events("save-in-place")
        .any(|l| l.get("outcome") == Some("ok"))
    {
        return Ok(Some(format!(
            "Ctrl+S traced no `save-in-place outcome=ok`. Trace: {path}."
        )));
    }
    if let Some(failure) = judge_save(report, doc)? {
        return Ok(Some(failure));
    }
    remove(ctx, report, session, pointer)
}

/// The saved stream holding the typed letters must be a whole pdfcer
/// `/pdfc_OCR` section.
fn judge_save(report: &mut CheckReport, doc: &std::path::Path) -> Result<Option<String>> {
    let bytes = std::fs::read(doc).map_err(|e| Error::new(format!("reading the save: {e}")))?;
    let holding: Vec<String> = streams(&bytes)
        .iter()
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .filter(|t| t.contains(TYPED))
        .collect();
    let Some(text) = holding.first() else {
        return Ok(Some(format!(
            "★ the saved file holds `{TYPED}` in no content stream: the edit was not written."
        )));
    };
    // A layer is a whole stream: `read_marker` finds it only when the
    // stream opens with the marker and closes with its `EMC`.
    let body = text.trim();
    let opens = body.starts_with("/pdfc_OCR");
    let producer = body
        .lines()
        .next()
        .is_some_and(|l| l.contains("/Producer (pdfcer)"));
    let closes = body.ends_with("EMC");
    report.note(format!(
        "★ `{TYPED}` saved in {} stream(s); it opens with the marker: {opens}, pdfcer's: \
         {producer}, closes with EMC: {closes}",
        holding.len()
    ));
    Ok((!(opens && producer && closes)).then(|| {
        "★ the edited word is saved in a stream that is not a whole pdfcer `/pdfc_OCR` \
         section: the edit folded the layer into the page's content, so the layer lost its \
         boundary."
            .to_owned()
    }))
}

/// File ▸ Recognise ▸ Remove OCR text must still find both pdfcer layers.
fn remove(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            let prefix = region.rsplit_once('.').map_or(region, |(head, _)| head);
            Error::new(format!(
                "no `{region}` region. Declared under `{prefix}`: {}.",
                list(&declared_names(&trace, ui_rect, prefix))
            ))
        })?;
        pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };
    click(TAB)?;
    let trace = session.trace()?;
    if declared(&trace, ui_rect, ITEM).is_none() && declared(&trace, ui_rect, COLLAPSED).is_some() {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(applied) = trace.events(APPLIED).last() else {
        return Ok(Some(format!(
            "★★ Remove OCR text after the edit traced no `{APPLIED}`: no pdfcer layer was \
             found. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("★★ {}", applied.raw));
    Ok(
        (applied.get("removed") != Some("2") || applied.get("pages") != Some("2")).then(|| {
            format!(
                "★★ after the edit Remove OCR text found another count than the fixture's two \
                 layers on two pages: `{}`. The edited page's layer was folded away.",
                applied.raw
            )
        }),
    )
}
