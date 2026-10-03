//! `a_snapshot_copy_is_cropped_at_the_set_dpi` — with a snapshot box laid
//! over blank paper beside a drawing, Copy (the platform chord) and the box's
//! own right-click menu each put the box on the clipboard: vectors with none of
//! the drawing in them, and a picture exactly the box's size at the resolution
//! the app reports. Driven through the scripted pointer on a window placed off
//! the desktop; the clipboard is restored afterwards.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/snapshot_copy.md`.

use super::os_image_paste::ClipGuard;
use super::snapshot_box::{BOX_EVENT, BOX_REGION, GROUPS, ITEM, Rig, TAB, laid_box, launch};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, WindowPoint};
use crate::error::Result;
use crate::report::CheckReport;
use crate::sys;

const COPIED: &str = "clipboard-snapshot-copy"; // ui-text-exempt: a trace event name, never displayed
const REFUSED: &str = "clipboard-snapshot-copy-refused"; // ui-text-exempt: a trace event name, never displayed
const MENU: &str = "canvas-menu"; // ui-text-exempt: a trace event name, never displayed
const CONTEXT: &str = "canvas.snapshot"; // ui-text-exempt: a menu context id, never displayed
const MENU_COPY: &str = "menu.item.canvas.snapshot.edit.copy"; // ui-text-exempt: a trace region name, never displayed
/// The placement order a page copy uses, as the app names the formats.
const ORDER: &str = "image/svg+xml,CF_ENHMETAFILE,PNG,CF_DIBV5"; // ui-text-exempt: clipboard format names, never displayed
const PNG: &str = "PNG"; // ui-text-exempt: a clipboard format name, never displayed
const SVG: &str = "image/svg+xml"; // ui-text-exempt: a clipboard format name, never displayed
/// How far a measured size may sit from the box's: the trace rounds the
/// corners to 0.1 pt, which moves a ceiling by at most one pixel.
const PX_TOLERANCE: f64 = 1.0;
const PT_TOLERANCE: f64 = 0.5;
/// The box's corners as fractions of the page: blank paper, clear of the
/// fixture's drawing, which lies inside x 6–50 %, y 50–92.5 %.
const FROM: (f64, f64) = (0.70, 0.10);
const TO: (f64, f64) = (0.90, 0.30);
/// SVG elements that paint.
const PAINTING: [&str; 10] = [
    "path", "use", "image", "text", "rect", "line", "polyline", "polygon", "circle", "ellipse",
];
/// SVG elements whose children paint nothing by themselves.
const NON_PAINTING: [&str; 6] = ["defs", "clipPath", "mask", "pattern", "symbol", "marker"];

/// See the module documentation.
pub struct ASnapshotCopyIsCroppedAtTheSetDpi;

impl Check for ASnapshotCopyIsCroppedAtTheSetDpi {
    fn name(&self) -> &'static str {
        "a_snapshot_copy_is_cropped_at_the_set_dpi"
    }

    fn defect(&self) -> &'static str {
        "Copy with a snapshot box laid copies nothing, copies the whole page or the selection \
         instead of the box, carries drawing from outside the box in its vectors, makes the \
         picture at a resolution other than the one it reports, or the box's right-click menu \
         offers no Copy"
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
    let (rig, page) = launch(ctx, report, "snapshot-copy")?;
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
    let size_pt = box_size(&rig)?;

    clear(guard)?;
    rig.pointer.copy(&rig.session, None)?;
    rig.session.settle(60);
    guard.adopt();
    if let Some(why) = judge(&rig, report, size_pt, 1, "the Copy chord")? {
        return Ok(Some(why));
    }

    clear(guard)?;
    let inside = WindowPoint::centre_of(rig.region(BOX_REGION)?);
    rig.pointer.right_click(&rig.session, inside)?;
    rig.session.settle(15);
    let context = rig
        .session
        .trace()?
        .last(MENU)
        .and_then(|l| l.get("context").map(str::to_owned));
    if context.as_deref() != Some(CONTEXT) {
        return Ok(Some(format!(
            "★★★ a right-click inside the box opened `{}`, not `{CONTEXT}`.",
            context.as_deref().unwrap_or("no menu")
        )));
    }
    rig.click(MENU_COPY)?;
    rig.session.settle(60);
    guard.adopt();
    judge(&rig, report, size_pt, 2, "the box's menu")
}

/// Empty the clipboard, so what is read next was placed by this run.
fn clear(guard: &mut ClipGuard) -> Result<()> {
    if !sys::clear_clipboard() {
        return Err(crate::error::Error::new(
            "could not clear the clipboard, so a later read could be an earlier run's.",
        ));
    }
    guard.adopt();
    Ok(())
}

/// The laid box's width and height in points, from its trace line.
fn box_size(rig: &Rig) -> Result<(f64, f64)> {
    let trace = rig.session.trace()?;
    let line = trace
        .last(BOX_EVENT)
        .ok_or_else(|| crate::error::Error::new("the box line vanished."))?;
    let f = |k: &str| {
        line.get(k)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(f64::NAN)
    };
    Ok((f("urx") - f("llx"), f("ury") - f("lly")))
}

/// Require the `nth` copy line, the order, a PNG of the box's size at the
/// reported resolution, and an SVG the box's size in points.
fn judge(
    rig: &Rig,
    report: &mut CheckReport,
    (w_pt, h_pt): (f64, f64),
    nth: usize,
    route: &str,
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
    if line.get("vectors") != Some("cut") || line.get("formats") != Some(ORDER) {
        return Ok(Some(format!(
            "★★ {route} placed `{}`, not the cut vectors and `{ORDER}`.",
            line.raw
        )));
    }
    let num = |k: &str| {
        line.get(k)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(f64::NAN)
    };
    let dpi = num("dpi");
    let want = ((w_pt * dpi / 72.0).ceil(), (h_pt * dpi / 72.0).ceil());
    let Some((pw, ph)) = sys::clipboard_bytes(sys::register_format(PNG)).and_then(|b| ihdr(&b))
    else {
        return Ok(Some(format!(
            "★★ {route} left no readable PNG on the clipboard."
        )));
    };
    report.note(format!(
        "{route}: PNG {pw} x {ph}; a {w_pt:.1} x {h_pt:.1} pt box at {dpi} dpi is {} x {}",
        want.0, want.1
    ));
    let off = |got: u32, want: f64| (f64::from(got) - want).abs() > PX_TOLERANCE;
    if off(pw, want.0) || off(ph, want.1) || num("w") != f64::from(pw) {
        return Ok(Some(format!(
            "★★★ {route} placed a {pw} x {ph} picture; the box at {dpi} dpi is {} x {}.",
            want.0, want.1
        )));
    }
    let svg = sys::clipboard_bytes(sys::register_format(SVG)).unwrap_or_default();
    let svg = String::from_utf8_lossy(&svg);
    let svg_w = svg_width_pt(&svg);
    if svg_w.is_none_or(|got| (got - w_pt).abs() > PT_TOLERANCE) {
        return Ok(Some(format!(
            "★★★ {route} placed an SVG {svg_w:?} pt wide; the box is {w_pt:.1} pt."
        )));
    }
    let painted = painting_elements(&svg);
    report.note(format!(
        "{route}: the SVG paints {painted} elements over blank paper"
    ));
    if painted > 0 {
        return Ok(Some(format!(
            "★★★ {route} placed an SVG that paints {painted} elements for a box over blank \
             paper: drawing from outside the box went with it."
        )));
    }
    Ok(None)
}

/// A PNG's width and height from its header.
fn ihdr(bytes: &[u8]) -> Option<(u32, u32)> {
    let word = |at: usize| Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
    (bytes.get(12..16) == Some(b"IHDR")).then_some(())?;
    Some((word(16)?, word(20)?))
}

/// How many painting elements an SVG has outside definitions, clips and masks.
fn painting_elements(svg: &str) -> usize {
    let mut hidden = 0usize;
    let mut painted = 0;
    for tag in svg.split('<').skip(1) {
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect();
        let closing = tag.starts_with('/');
        let empty = tag.split('>').next().is_some_and(|t| t.ends_with('/'));
        if NON_PAINTING.contains(&name.as_str()) {
            if closing {
                hidden = hidden.saturating_sub(1);
            } else if !empty {
                hidden += 1;
            }
        } else if !closing && hidden == 0 && PAINTING.contains(&name.as_str()) {
            painted += 1;
        }
    }
    painted
}

/// The root element's `width="…pt"`.
fn svg_width_pt(svg: &str) -> Option<f64> {
    let rest = &svg[svg.find("<svg")?..];
    let start = rest.find("width=\"")? + 7;
    let end = rest[start..].find("pt\"")?;
    rest[start..start + end].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_and_the_width_are_read() {
        let mut png = vec![0u8; 24];
        png[12..16].copy_from_slice(b"IHDR");
        png[16..20].copy_from_slice(&300u32.to_be_bytes());
        png[20..24].copy_from_slice(&150u32.to_be_bytes());
        assert_eq!(ihdr(&png), Some((300, 150)));
        assert_eq!(ihdr(&png[..20]), None);
        let svg = r#"<?xml?><svg xmlns="x" width="95.25pt" height="10pt">"#;
        assert_eq!(svg_width_pt(svg), Some(95.25));
    }

    #[test]
    fn only_elements_that_paint_are_counted() {
        let blank = r#"<svg><defs><clipPath id="c"><path d="M0 0"/></clipPath></defs><g/></svg>"#;
        assert_eq!(painting_elements(blank), 0);
        let drawn =
            r##"<svg><defs><path id="g"/></defs><g><use href="#g"/><path d="M0"/></g></svg>"##;
        assert_eq!(painting_elements(drawn), 2);
    }
}
