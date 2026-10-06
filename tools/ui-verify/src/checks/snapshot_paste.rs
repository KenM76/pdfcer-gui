//! `a_snapshot_pastes_back_as_a_drawing` — in Review, a snapshot box laid
//! round a drawing and copied puts a one-page PDF on the clipboard beside the
//! picture, and Ctrl+V places that PDF back as a stamp the box's size; in Edit
//! the same paste draws it into the page's content, one undo; in Read it is
//! refused. Driven through the scripted pointer on a window
//! placed off the desktop; the clipboard is restored afterwards.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snapshot_paste.md`.

use super::os_image_paste::{self as osp, ClipGuard};
use super::snapshot_box::{GROUPS, ITEM, Rig, TAB, laid_box, launch};
use super::snapshot_copy::{box_size, clear};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::Result;
use crate::report::CheckReport;
use crate::sys;

const COPIED: &str = "clipboard-snapshot-copy"; // ui-text-exempt: a trace event name, never displayed
const PLACED: &str = "custom-stamp-placed"; // ui-text-exempt: a trace event name, never displayed
const CONTENT: &str = "page-content-placed"; // ui-text-exempt: a trace event name, never displayed
const UNDONE: &str = "undo-applied"; // ui-text-exempt: a trace event name, never displayed
const DECLINED: &str = "command-declined"; // ui-text-exempt: a trace event name, never displayed
const PDF: &str = "application/pdf"; // ui-text-exempt: a clipboard format name, never displayed
/// How far the stamp's sides may sit from the box's: the box line rounds its
/// corners to 0.1 pt and the paste line to 0.01 pt.
const TOLERANCE_PT: f64 = 0.5;
/// The box's corners as fractions of the page: round the whole of the
/// fixture's drawing, which lies inside x 6–50 %, y 50–92.5 %, so nothing
/// crosses the box's edge and the vectors are cut rather than withheld.
const FROM: (f64, f64) = (0.04, 0.48);
const TO: (f64, f64) = (0.52, 0.95);
/// Where the paste is aimed, as fractions of the page: blank paper.
const PASTE_AT: (f64, f64) = (0.75, 0.30);

/// See the module documentation.
pub struct ASnapshotPastesBackAsADrawing;

impl Check for ASnapshotPastesBackAsADrawing {
    fn name(&self) -> &'static str {
        "a_snapshot_pastes_back_as_a_drawing"
    }

    fn defect(&self) -> &'static str {
        "A snapshot copy puts no PDF on the clipboard, or pasting it back into pdfcer places the \
         flat picture instead of the drawing, at a size other than the box's, or places it in \
         Read"
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
    let (rig, page) = launch(ctx, report, "snapshot-paste")?;
    switch(&rig, "2")?;
    rig.click(TAB)?;
    rig.click_item(ITEM, GROUPS)?;
    let (w, h) = (page.width_pt, page.height_pt);
    let corners = ((FROM.0 * w, FROM.1 * h), (TO.0 * w, TO.1 * h));
    let window = |x: f64, y: f64| -> Result<WindowPoint> {
        CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?
            .doc_to_window(DocPoint::new(0, x, y))
    };
    let (a, b) = (
        window(corners.0.0, corners.0.1)?,
        window(corners.1.0, corners.1.1)?,
    );
    rig.pointer.drag(&rig.session, a, b, 12)?;
    rig.session.settle(20);
    if let Err(why) = laid_box(&rig, corners)? {
        return Ok(Some(why));
    }
    let size = box_size(&rig)?;

    clear(guard)?;
    rig.pointer.copy(&rig.session, None)?;
    rig.session.settle(60);
    guard.adopt();
    if let Some(why) = copied(&rig, report)? {
        return Ok(Some(why));
    }

    let at = window(PASTE_AT.0 * w, PASTE_AT.1 * h)?;
    let mut failure = pasted_in_review(&rig, report, at, size)?;
    if failure.is_none() {
        switch(&rig, "3")?;
        failure = pasted_in_edit(&rig, report, at, size)?;
    }
    if failure.is_none() {
        switch(&rig, "1")?;
        failure = read_refuses(&rig, at)?;
    }
    let parked = rig.pointer.gone(&rig.session);
    match failure {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Change mode with `Ctrl+digit`.
fn switch(rig: &Rig, digit: &str) -> Result<()> {
    rig.pointer.key(&rig.session, None, digit, Some("ctrl"))?;
    rig.session.settle(20);
    Ok(())
}

/// The copy line naming the PDF, and PDF bytes on the clipboard under it.
fn copied(rig: &Rig, report: &mut CheckReport) -> Result<Option<String>> {
    let trace = rig.session.trace()?;
    let Some(line) = trace.last(COPIED) else {
        return Ok(Some(format!("★ the Copy chord wrote no `{COPIED}` line.")));
    };
    report.note(format!("copy: `{}`", line.raw));
    let names_pdf = line
        .get("formats")
        .is_some_and(|f| f.split(',').any(|name| name == PDF));
    if line.get("vectors") != Some("cut") || !names_pdf {
        return Ok(Some(format!(
            "★★ the copy placed `{}`, not the cut vectors with `{PDF}`.",
            line.raw
        )));
    }
    let bytes = sys::clipboard_bytes(sys::register_format(PDF)).unwrap_or_default();
    report.note(format!("clipboard `{PDF}`: {} bytes", bytes.len()));
    Ok((!bytes.starts_with(b"%PDF-")).then(|| {
        format!(
            "★★ the clipboard's `{PDF}` entry is not a PDF ({} bytes).",
            bytes.len()
        )
    }))
}

/// Ctrl+V at `at` in Review places the drawing as a stamp the box's size.
fn pasted_in_review(
    rig: &Rig,
    report: &mut CheckReport,
    at: WindowPoint,
    (w, h): (f64, f64),
) -> Result<Option<String>> {
    let (pasted, placed) = (
        osp::count(&rig.session, osp::PASTED)?,
        osp::count(&rig.session, PLACED)?,
    );
    rig.pointer.hover(&rig.session, at)?;
    rig.pointer.paste(&rig.session, None, "x")?;
    rig.session.settle(30);
    let trace = rig.session.trace()?;
    let Some(line) = trace.events(osp::PASTED).nth(pasted) else {
        return Ok(Some(format!(
            "★ Ctrl+V in Review traced no `{}` line.",
            osp::PASTED
        )));
    };
    report.note(format!("Review paste: `{}`", line.raw));
    if line.get("kind") != Some("pdf") || line.get("as") != Some("stamp") {
        return Ok(Some(format!(
            "★★★ Ctrl+V in Review placed kind={} as={}, not the drawing as a stamp.",
            line.get("kind").unwrap_or("-"),
            line.get("as").unwrap_or("-")
        )));
    }
    let Some(r) = osp::rect(line) else {
        return Ok(Some(format!(
            "★ a `{}` line without its rectangle.",
            osp::PASTED
        )));
    };
    let (rw, rh) = (r[2] - r[0], r[3] - r[1]);
    if (rw - w).abs() > TOLERANCE_PT || (rh - h).abs() > TOLERANCE_PT {
        return Ok(Some(format!(
            "★★★ the stamp is {rw:.2} x {rh:.2} pt; the box was {w:.2} x {h:.2}."
        )));
    }
    Ok((trace.events(PLACED).count() == placed)
        .then(|| format!("★★ the stamp was asked for but no `{PLACED}` line followed.")))
}

/// In Read the same paste places nothing and is declined.
/// Ctrl+V at `at` in Edit draws the drawing into the page's content at the
/// box's size, through `place_page_content` and not the stamp verb, and one
/// Ctrl+Z takes it back.
fn pasted_in_edit(
    rig: &Rig,
    report: &mut CheckReport,
    at: WindowPoint,
    (w, h): (f64, f64),
) -> Result<Option<String>> {
    let s = &rig.session;
    let (pasted, content, stamped, undone) = (
        osp::count(s, osp::PASTED)?,
        osp::count(s, CONTENT)?,
        osp::count(s, PLACED)?,
        osp::count(s, UNDONE)?,
    );
    rig.pointer.hover(s, at)?;
    rig.pointer.paste(s, None, "x")?;
    rig.session.settle(30);
    let trace = s.trace()?;
    let Some(line) = trace.events(osp::PASTED).nth(pasted) else {
        return Ok(Some(format!(
            "Ctrl+V in Edit traced no `{}` line.",
            osp::PASTED
        )));
    };
    report.note(format!("Edit paste: `{}`", line.raw));
    if line.get("kind") != Some("pdf") || line.get("as") != Some("content") {
        return Ok(Some(format!(
            "Ctrl+V in Edit placed kind={} as={}, not the drawing as page content.",
            line.get("kind").unwrap_or("-"),
            line.get("as").unwrap_or("-")
        )));
    }
    let Some(placed) = trace.events(CONTENT).nth(content) else {
        return Ok(Some(format!(
            "the drawing was asked for as content but no `{CONTENT}` line followed."
        )));
    };
    report.note(format!("placed: `{}`", placed.raw));
    if trace.events(PLACED).count() != stamped {
        return Ok(Some(format!(
            "Ctrl+V in Edit traced `{PLACED}`: the drawing went in as a stamp."
        )));
    }
    let scale = |k: &str| placed.get(k).and_then(|v| v.parse::<f64>().ok());
    let natural = |v: Option<f64>| v.is_some_and(|v| (v - 1.0).abs() < 0.01);
    if !natural(scale("scale-x")) || !natural(scale("scale-y")) {
        return Ok(Some(format!(
            "the drawing went in at scale {:?} x {:?}; the box's own size, {w:.2} x {h:.2} pt, \
             was expected.",
            scale("scale-x"),
            scale("scale-y")
        )));
    }
    rig.pointer.gone(s)?;
    rig.pointer.key(s, None, "Z", Some("ctrl"))?;
    rig.session.settle(20);
    Ok((osp::count(s, UNDONE)? == undone)
        .then(|| "Ctrl+Z did not undo the placed drawing.".to_owned()))
}

fn read_refuses(rig: &Rig, at: WindowPoint) -> Result<Option<String>> {
    let (pasted, declined) = (
        osp::count(&rig.session, osp::PASTED)?,
        osp::count(&rig.session, DECLINED)?,
    );
    rig.pointer.hover(&rig.session, at)?;
    rig.pointer.paste(&rig.session, None, "x")?;
    rig.session.settle(20);
    if osp::count(&rig.session, osp::PASTED)? != pasted {
        return Ok(Some(
            "★★★ Read pasted the drawing onto the page.".to_owned(),
        ));
    }
    let trace = rig.session.trace()?;
    let refused = trace
        .events(DECLINED)
        .nth(declined)
        .is_some_and(|l| l.get("id") == Some("edit.paste"));
    Ok((!refused).then(|| format!("★★ Read traced no `{DECLINED} id=edit.paste` line.")))
}
