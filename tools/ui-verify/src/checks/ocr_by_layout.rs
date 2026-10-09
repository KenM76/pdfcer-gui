//! `reading_by_layout_layers_each_region_apart` — File ▸ Recognise text…
//! with PaddleOCR-VL and *Read by layout* ticked writes the words read from
//! titles, tables and other regions on a layer per region, each nested in the
//! recognised-text layer, as the same one undo step; and Remove OCR text
//! deletes every one of those layers along with the main one. The window draws
//! neither word-list choice for this model, which can take neither. Run with the
//! scripted pointer in a window placed off the desktop, on a copy of
//! `fixtures/synthetic-image-only.pdf`, against a build whose `paddle-vl`
//! add-on holds the layout model.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_by_layout.md`.

use crate::checks::ocr_layer_group::{launch, remove_all};
use crate::checks::ocr_scripted::{SAVED, recognise_choosing};
use crate::checks::{Check, CheckContext};
use crate::error::{Error, Result};
use crate::report::CheckReport;

const ENGINE: &str = "paddle-vl";
/// The dialog's *Read by layout* checkbox.
const BY_LAYOUT: &str = "ocr-by-layout"; // ui-text-exempt: a trace region name, never displayed
/// Word-list choices the vision model cannot take, so must not be drawn.
const NO_LISTS: &str = "ocr-no-word-lists"; // ui-text-exempt: a trace region name, never displayed
const ADD_WORDS: &str = "ocr-add-words"; // ui-text-exempt: a trace region name, never displayed
const STARTED: &str = "ocr-started"; // ui-text-exempt: a trace event name, never displayed
const REGIONS: &str = "ocr-layer-regions"; // ui-text-exempt: a trace event name, never displayed
const NESTED: &str = "ocr-layer-nested"; // ui-text-exempt: a trace event name, never displayed
const GROUP: &str = "ocr-layer-group"; // ui-text-exempt: a trace event name, never displayed
const APPLIED: &str = "remove-ocr-layers-applied"; // ui-text-exempt: a trace event name, never displayed

/// See the module documentation.
pub struct ReadingByLayoutLayersEachRegionApart;

impl Check for ReadingByLayoutLayersEachRegionApart {
    fn name(&self) -> &'static str {
        "reading_by_layout_layers_each_region_apart"
    }

    fn defect(&self) -> &'static str {
        "reading by layout puts every word on the one recognised-text layer, leaves the region \
         layers loose at the top of the Layers panel, or Remove OCR text leaves them behind empty"
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
    let copy = ctx.out("ocr-by-layout.pdf");
    std::fs::copy(&fixture, &copy).map_err(|e| Error::new(format!("copying the fixture: {e}")))?;

    // --- 1: recognise by layout and save ------------------------------------
    let (trace, path) =
        recognise_choosing(ctx, report, &copy, ENGINE, "layout", true, &[BY_LAYOUT])?;
    let Some(started) = trace.last(STARTED) else {
        return Ok(Some(format!("Run traced no `{STARTED}`. Trace: {path}.")));
    };
    report.note(format!("run: `{}`", started.raw));
    let ui_rect = ctx.profile.vocab.ui_rect_event.unwrap_or("ui-rect");
    if let Some(r) = [NO_LISTS, ADD_WORDS]
        .into_iter()
        .find(|r| crate::checks::driving::declared(&trace, ui_rect, r).is_some())
    {
        return Ok(Some(format!(
            "★ PaddleOCR-VL can neither drop its word lists nor take a word file, and the \
             window drew `{r}`. Trace: {path}."
        )));
    }
    if started.get("by-layout") != Some("true") {
        return Ok(Some(format!(
            "★ *Read by layout* was ticked and the run did not ask for it: `{}`. Trace: {path}.",
            started.raw
        )));
    }
    let Some(regions) = trace.last(REGIONS) else {
        return Ok(Some(format!(
            "★ no `{REGIONS}` line: the recognition wrote no layer. Trace: {path}."
        )));
    };
    report.note(format!("regions: `{}`", regions.raw));
    let n = regions
        .get("groups")
        .map_or(0, |g| g.split(',').filter(|s| !s.is_empty()).count());
    if n == 0 {
        return Ok(Some(format!(
            "★★ read by layout, and no region other than body text got a layer: `{}`. The \
             layout model found none, or its regions did not reach the writer. Trace: {path}.",
            regions.raw
        )));
    }
    let want = n.to_string();
    let nested = trace.last(NESTED);
    let all = nested.is_some_and(|l| l.get("n") == Some(&want) && l.get("of") == Some(&want));
    if !all {
        return Ok(Some(format!(
            "★★ {n} region layer(s) made and `{}`: not all of them are sublayers of the \
             recognised-text layer. Trace: {path}.",
            nested.map_or("no nesting line", |l| l.raw.as_str())
        )));
    }
    let group = trace.last(GROUP).map(|l| l.raw.clone()).unwrap_or_default();
    if !group.contains("made=true") || !group.contains(&format!("groups-made={}", n + 1)) {
        return Ok(Some(format!(
            "★ a document with no layers must get the recognised-text layer and {n} region \
             layers from this write: `{group}`. Trace: {path}."
        )));
    }
    if !trace
        .last(SAVED)
        .is_some_and(|l| l.raw.contains("outcome=ok"))
    {
        return Ok(Some(format!("Ctrl+S did not save. Trace: {path}.")));
    }

    // --- 2: Remove OCR text deletes every layer it empties ------------------
    let (session, pointer, ui_rect) = launch(ctx, &copy, "ocr-by-layout")?;
    report.artifact(session.trace_path().to_path_buf());
    report.artifact(pointer.path().to_path_buf());
    session.settle(45);
    let removed = remove_all(&session, &pointer, ui_rect);
    let parked = pointer.gone(&session);
    removed?;
    parked?;
    let trace = session.trace()?;
    let path = session.trace_path().display().to_string();
    let Some(applied) = trace.last(APPLIED) else {
        return Ok(Some(format!(
            "Remove OCR text traced no `{APPLIED}`. Trace: {path}."
        )));
    };
    report.note(format!("remove: `{}`", applied.raw));
    let want = (n + 1).to_string();
    if applied.get("groups-deleted") != Some(want.as_str()) {
        return Ok(Some(format!(
            "★★★ Remove OCR text left region layers behind: {want} layers were emptied and \
             `{}`. Trace: {path}.",
            applied.raw
        )));
    }
    Ok(None)
}
