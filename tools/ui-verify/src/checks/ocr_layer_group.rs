//! `recognised_text_is_a_layers_row` — File ▸ Recognise text… writes the
//! words on a layer named *Recognised text*, which the Layers panel lists;
//! Remove OCR text takes the words off and deletes the layer they leave
//! empty, and one Ctrl+Z puts both back. Run with the scripted pointer in a
//! window placed off the desktop, on a copy of
//! `fixtures/synthetic-image-only.pdf`.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_layer_group.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_in, declared_names, list,
};
use crate::checks::forms_spotlight::open_from_tab;
use crate::checks::ocr_scripted::{SAVED, recognise};
use crate::checks::{Check, CheckContext};
use crate::coords::WindowPoint;
use crate::error::{Error, Result};
use crate::input::scripted::ScriptedPointer;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;

const OFFSCREEN: &str = "-4200,-4200,1400,900";
const ENGINE: &str = "ocrs";
const GROUP: &str = "ocr-layer-group"; // ui-text-exempt: a trace event name, never displayed
const ROW: &str = "panel.layers.row.Recognised_text"; // ui-text-exempt: a trace region name, never displayed
const PANEL_ITEM: &str = "ribbon.item.view.panel_layers"; // ui-text-exempt: a trace region name, never displayed
const TAB: &str = "ribbon.tab.file"; // ui-text-exempt: a trace region name, never displayed
const ITEM: &str = "ribbon.item.file.remove_ocr"; // ui-text-exempt: a trace region name, never displayed
const COLLAPSED: &str = "ribbon.group.file.recognise.collapsed"; // ui-text-exempt: a trace region name, never displayed
const APPLIED: &str = "remove-ocr-layers-applied"; // ui-text-exempt: a trace event name, never displayed
const UNDO: &str = "undo"; // ui-text-exempt: a trace event name, never displayed
const UNDONE_KIND: &str = "kind=RemoveOcrLayer"; // ui-text-exempt: a trace token, never displayed

/// See the module documentation.
pub struct RecognisedTextIsALayersRow;

impl Check for RecognisedTextIsALayersRow {
    fn name(&self) -> &'static str {
        "recognised_text_is_a_layers_row"
    }

    fn defect(&self) -> &'static str {
        "recognised text is written on no layer, so the Layers panel cannot show or hide it, \
         or Remove OCR text leaves an empty layer behind"
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
    let fixture = crate::fixture::workspace_root()
        .join("fixtures")
        .join("synthetic-image-only.pdf");
    let copy = ctx.out("ocr-layer-group.pdf");
    std::fs::copy(&fixture, &copy).map_err(|e| Error::new(format!("copying the fixture: {e}")))?;

    // --- 1: recognise and save: the layer is made ---------------------------
    let (trace, path) = recognise(ctx, report, &copy, ENGINE, "group", true)?;
    let Some(group) = trace.last(GROUP) else {
        return Ok(Some(format!(
            "★ no `{GROUP}` line: the recognised text was written on no layer. Trace: {path}."
        )));
    };
    report.note(format!("write: `{}`", group.raw));
    if group.get("made") != Some("true") {
        return Ok(Some(format!(
            "★ a document with no layers traced `{}`: the layer was not made.",
            group.raw
        )));
    }
    if !trace
        .last(SAVED)
        .is_some_and(|l| l.raw.contains("outcome=ok"))
    {
        return Ok(Some(format!("Ctrl+S did not save. Trace: {path}.")));
    }

    // --- 2..4: the row, its removal and the undo, on the saved file ---------
    let (session, pointer, ui_rect) = launch(ctx, &copy)?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let outcome = row_remove_undo(report, &session, &pointer, ui_rect);
    let parked = pointer.gone(&session);
    let outcome = outcome?;
    parked?;
    Ok(outcome)
}

fn launch(
    ctx: &CheckContext,
    pdf: &std::path::Path,
) -> Result<(Session, ScriptedPointer, &'static str)> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let viewport_env = ctx.profile.viewport_env.ok_or_else(|| {
        Error::new("the profile has no viewport variable to place the window off the desktop.")
    })?;
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let mut spec = LaunchSpec::new(&exe, ctx.out("ocr-layer-group.panel.trace.txt"));
    spec.pdf = Some(pdf.to_path_buf());
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.env
        .push((viewport_env.to_owned(), OFFSCREEN.to_owned()));
    spec.place = false;
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let pointer = ScriptedPointer::attach(&mut spec, ctx.out("ocr-layer-group.pointer.txt"))?;
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    Ok((session, pointer, ui_rect))
}

fn row_remove_undo(
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    ui_rect: &str,
) -> Result<Option<String>> {
    let path = session.trace_path().display().to_string();
    click_mode_segment(session, pointer, ui_rect, "edit")?;
    session.settle(20);
    if declared(&session.trace()?, ui_rect, ROW).is_none() {
        open_from_tab(session, pointer, ui_rect, "view", PANEL_ITEM)?;
        session.settle(24);
    }
    if declared(&session.trace()?, ui_rect, ROW).is_none() {
        return Ok(Some(format!(
            "★ the saved file's Layers panel has no `{ROW}` row. Rows: {}. Trace: {path}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "panel.layers.row."
            ))
        )));
    }
    report.note("the saved file lists the layer in the Layers panel");

    let click = |region: &str| -> Result<()> {
        let trace = session.trace()?;
        let (rect, viewport) = declared_in(&trace, ui_rect, region).ok_or_else(|| {
            Error::new(format!(
                "no `{region}` region. Declared under `ribbon.item.file`: {}.",
                list(&declared_names(&trace, ui_rect, "ribbon.item.file"))
            ))
        })?;
        pointer.click_in(session, viewport.as_deref(), WindowPoint::centre_of(rect))?;
        session.settle(15);
        Ok(())
    };
    click(TAB)?;
    if declared(&session.trace()?, ui_rect, ITEM).is_none()
        && declared(&session.trace()?, ui_rect, COLLAPSED).is_some()
    {
        click(COLLAPSED)?;
    }
    click(ITEM)?;
    session.settle(30);
    let trace = session.trace()?;
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "Remove OCR text traced no `{APPLIED}`. Trace: {path}."
        )));
    };
    report.note(format!("remove: `{}`", applied.raw));
    if applied.get("removed") != Some("1") || applied.get("groups-deleted") != Some("1") {
        return Ok(Some(format!(
            "★★ Remove OCR text must take the one layer of words off and delete the layer it \
             leaves empty: `{}`. Trace: {path}.",
            applied.raw
        )));
    }
    if declared(&trace, ui_rect, ROW).is_some() {
        return Ok(Some(format!(
            "★★ the removal reported the layer deleted and the Layers panel still lists \
             `{ROW}`. Trace: {path}."
        )));
    }

    let viewport = declared_in(&trace, ui_rect, TAB).and_then(|(_, vp)| vp);
    pointer.key(session, viewport.as_deref(), "Z", Some("ctrl"))?;
    session.settle(30);
    let trace = session.trace()?;
    // The removal is this launch's only edit, so folded it is the stack's one
    // entry (`undo_depth` is read before the pop). Unfolded, the stack is two
    // deep and the top entry is the layer's deletion alone, relabelled with
    // the removal's kind, which brings the row back and leaves the words off.
    let undone = trace.last(UNDO);
    let folded =
        undone.is_some_and(|l| l.raw.contains(UNDONE_KIND) && l.get("undo_depth") == Some("1"));
    let undone = undone.map(|l| l.raw.clone());
    if !folded {
        return Ok(Some(format!(
            "★★★ Ctrl+Z after Remove OCR text undid {undone:?}, not one `{UNDONE_KIND}` entry: \
             the words and the layer come back in separate steps. Trace: {path}."
        )));
    }
    if declared(&trace, ui_rect, ROW).is_none() {
        return Ok(Some(format!(
            "★★★ one Ctrl+Z after Remove OCR text did not bring the layer back: the removal \
             and the layer's deletion are not one undo step. Trace: {path}."
        )));
    }
    report.note("one Ctrl+Z put the words and the layer back");
    Ok(None)
}
