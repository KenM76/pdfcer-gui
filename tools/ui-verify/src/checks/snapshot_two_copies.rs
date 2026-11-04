//! `the_snapshot_boxs_two_copies_choose_its_text` — with a snapshot box laid
//! over the whole of a page that carries text, the Clipboard tab's
//! *Copy snapshot, text as drawing* places vectors holding no text element,
//! and *Copy snapshot, text as text* places vectors holding some, whatever the
//! Snapshot setting says. Driven through the scripted pointer on a window
//! placed off the desktop; the clipboard is restored afterwards.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snapshot_two_copies.md`.

use super::os_image_paste::ClipGuard;
use super::snapshot_box::{GROUPS, ITEM, Rig, TAB, laid_box, launch};
use super::snapshot_copy::clear;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint};
use crate::error::Result;
use crate::report::CheckReport;
use crate::sys;

const COPIED: &str = "clipboard-snapshot-copy"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "clipboard-snapshot-copy-refused"; // ui-text-exempt: a trace event name, never displayed
const CLIPBOARD_TAB: &str = "ribbon.tab.clipboard"; // ui-text-exempt: a trace region name, never displayed
const OUTLINES: &str = "ribbon.item.edit.copy_snapshot_outlines"; // ui-text-exempt: a trace region name, never displayed
const KEEP_TEXT: &str = "ribbon.item.edit.copy_snapshot_text"; // ui-text-exempt: a trace region name, never displayed
const SVG: &str = "image/svg+xml"; // ui-text-exempt: a clipboard format name, never displayed
/// The box's corners as fractions of the page: all of it, so the fixture's
/// labels and title block are inside.
const FROM: (f64, f64) = (0.02, 0.02);
const TO: (f64, f64) = (0.98, 0.98);

pub struct TheSnapshotBoxsTwoCopiesChooseItsText;

impl Check for TheSnapshotBoxsTwoCopiesChooseItsText {
    fn name(&self) -> &'static str {
        "the_snapshot_boxs_two_copies_choose_its_text"
    }

    fn defect(&self) -> &'static str {
        "The Clipboard tab's two snapshot copies are missing, copy nothing, or both copy the \
         same text form: text as drawing still carries text elements, or text as text carries \
         none over a page that has text"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        let mut report = CheckReport::new(self.name(), self.defect());
        let mut guard = ClipGuard::take();
        let outcome = drive(ctx, &mut report, &mut guard);
        report.note(guard.release());
        match outcome {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    guard: &mut ClipGuard,
) -> Result<Option<String>> {
    let (rig, page) = launch(ctx, report, "snapshot-two-copies")?;
    rig.click(TAB)?;
    rig.click_item(ITEM, GROUPS)?;
    let (w, h) = (page.width_pt, page.height_pt);
    let corners = ((FROM.0 * w, FROM.1 * h), (TO.0 * w, TO.1 * h));
    let mapping = CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?;
    let a = mapping.doc_to_window(DocPoint::new(0, corners.0.0, corners.0.1))?;
    let b = mapping.doc_to_window(DocPoint::new(0, corners.1.0, corners.1.1))?;
    rig.pointer.drag(&rig.session, a, b, 12)?;
    rig.session.settle(20);
    if let Err(why) = laid_box(&rig, corners)? {
        return Ok(Some(why));
    }
    rig.click(CLIPBOARD_TAB)?;

    clear(guard)?;
    rig.click(OUTLINES)?;
    rig.session.settle(60);
    guard.adopt();
    if let Some(why) = judge(&rig, report, 1, "text as drawing", false)? {
        return Ok(Some(why));
    }

    clear(guard)?;
    rig.click(KEEP_TEXT)?;
    rig.session.settle(60);
    guard.adopt();
    judge(&rig, report, 2, "text as text", true)
}

/// Require the `nth` copy line, and its text runs and the clipboard's SVG
/// `<text` elements both present when `want_text`, both absent when not.
fn judge(
    rig: &Rig,
    report: &mut CheckReport,
    nth: usize,
    route: &str,
    want_text: bool,
) -> Result<Option<String>> {
    let trace = rig.session.trace()?;
    if let Some(refused) = trace.last(REFUSED) {
        return Ok(Some(format!("★ {route} was refused: `{}`.", refused.raw)));
    }
    let lines: Vec<_> = trace.events(COPIED).collect();
    if lines.len() < nth {
        return Ok(Some(format!(
            "★ {route} copied nothing: {} `{COPIED}` lines where {nth} were due.",
            lines.len()
        )));
    }
    let line = lines[nth - 1];
    report.note(format!("{route}: `{}`", line.raw));
    if line.get("vectors") != Some("cut") {
        return Ok(Some(format!(
            "★★ {route} placed `{}`, not the cut vectors.",
            line.raw
        )));
    }
    let runs: u64 = line
        .get("text_runs")
        .and_then(|v| v.parse().ok())
        .unwrap_or(u64::MAX);
    let svg = sys::clipboard_bytes(sys::register_format(SVG)).unwrap_or_default();
    let svg = String::from_utf8_lossy(&svg);
    let elements = svg.matches("<text").count();
    report.note(format!(
        "{route}: text_runs={runs}, {elements} `<text` elements in {} SVG bytes",
        svg.len()
    ));
    if svg.is_empty() {
        return Ok(Some(format!("★★ {route} left no SVG on the clipboard.")));
    }
    let has = runs > 0 && runs != u64::MAX && elements > 0;
    let none = runs == 0 && elements == 0;
    if want_text && !has {
        return Ok(Some(format!(
            "★★★ {route} kept no text: text_runs={runs}, {elements} `<text` elements."
        )));
    }
    if !want_text && !none {
        return Ok(Some(format!(
            "★★★ {route} still carries text: text_runs={runs}, {elements} `<text` elements."
        )));
    }
    Ok(None)
}
