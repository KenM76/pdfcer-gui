//! `highlight_shapes_draw_as_chosen` — **the highlighter draws the shape Tool
//! Options names**: an Alt-drag over text is a box where a plain drag follows
//! the text, Ellipse authors a washed ellipse, and Polygon takes its corners
//! by click, Backspace and Enter and authors a washed polygon.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/highlight_shapes.md`.

use super::snapshot_box::{Rig, launch_with};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowPoint};
use crate::error::Result;
use crate::input::scripted::xy;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review,markup.highlight"; // ui-text-exempt: command ids, never displayed
/// Review mode opens on Comments; Tool Options is on the Properties tab.
const PROPERTIES_TAB: &str = "dock.tab.file.properties"; // ui-text-exempt: a trace region name, never displayed
const HIGHLIGHT_BLOCK: &str = "properties.tool.highlight"; // ui-text-exempt: a trace region name, never displayed
const SHAPE_PREFIX: &str = "properties.tool.highlight.shape."; // ui-text-exempt: a trace region name, never displayed
const BAND_COMMIT: &str = "markup-commit"; // ui-text-exempt: a trace event name, never displayed
const TEXT_COMMIT: &str = "text-markup-commit"; // ui-text-exempt: a trace event name, never displayed
const AUTHOR: &str = "markup-author"; // ui-text-exempt: a trace event name, never displayed
const VERTEX_KEY: &str = "markup-vertex-key"; // ui-text-exempt: a trace event name, never displayed
/// `Shape::ALL` indices.
const ELLIPSE: usize = 2;
const POLYGON: usize = 4;

/// Two text rows of `blank-overhang.pdf` (1000 x 800 pt), measured with
/// `pdfcer.exe find-text`: "DRAWN INSIDE…" at y 703–716 and the title
/// block's "…TEST SHEET" at y 459–469. The Alt drag is a box over the first
/// row, because a flat area highlight is refused as having no extent.
const ROW_ALT: ((f64, f64), (f64, f64)) = ((0.10, 0.875), (0.35, 0.90));
const ROW_PLAIN: ((f64, f64), (f64, f64)) = ((0.31, 0.575), (0.43, 0.585));
/// Blank paper, clear of the drawing (x 6–50 %, y 50–92.5 %).
const ELLIPSE_BOX: ((f64, f64), (f64, f64)) = ((0.60, 0.15), (0.70, 0.25));
const CORNERS: [(f64, f64); 4] = [(0.75, 0.15), (0.85, 0.15), (0.88, 0.22), (0.78, 0.28)];

/// See the module documentation.
pub struct HighlightShapes;

impl Check for HighlightShapes {
    fn name(&self) -> &'static str {
        "highlight_shapes_draw_as_chosen"
    }

    fn defect(&self) -> &'static str {
        "the highlighter ignores its Tool Options shape: Alt-dragging over text still follows the \
         text, Ellipse or Polygon author an outlined comment shape rather than a multiplied wash, \
         or a polygon's Backspace and Enter do nothing"
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

/// Window point of page-0 fractions.
fn at(ctx: &CheckContext, rig: &Rig, page: PageGeometry, f: (f64, f64)) -> Result<WindowPoint> {
    let mapping = CanvasMapping::from_trace(&rig.session.trace()?, &ctx.profile.vocab, page, 0)?;
    mapping.doc_to_window(DocPoint::new(0, f.0 * page.width_pt, f.1 * page.height_pt))
}

/// Drag between page-0 fractions, holding `mods` if any.
fn drag(
    ctx: &CheckContext,
    rig: &Rig,
    page: PageGeometry,
    span: ((f64, f64), (f64, f64)),
    mods: Option<&str>,
) -> Result<()> {
    let (a, b) = (at(ctx, rig, page, span.0)?, at(ctx, rig, page, span.1)?);
    let mods = mods.map_or(String::new(), |m| format!(" mods={m}"));
    rig.pointer.send(
        &rig.session,
        &format!("drag {} {} steps=12{mods}", xy(a), xy(b)),
    )?;
    rig.session.settle(25);
    Ok(())
}

fn count(rig: &Rig, event: &str) -> Result<usize> {
    Ok(rig.session.trace()?.events(event).count())
}

fn last_raw(rig: &Rig, event: &str) -> Result<Option<String>> {
    Ok(rig.session.trace()?.last(event).map(|l| l.raw.clone()))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let invoke = [("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned())];
    let (rig, page) = launch_with(ctx, report, "highlight-shapes", &invoke)?;
    rig.session.settle(30);
    for step in [text_steps, ellipse_step, polygon_step] {
        if let Some(failure) = step(ctx, &rig, page, report)? {
            return Ok(Some(failure));
        }
    }
    let shot = ctx.out("highlight-shapes.png");
    rig.pointer.screenshot(&rig.session, &shot)?;
    report.artifact(shot);
    Ok(None)
}

/// Alt over text draws a box; the same drag without Alt follows the text.
fn text_steps(
    ctx: &CheckContext,
    rig: &Rig,
    page: PageGeometry,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    // Alt over text: a box, and no text sweep.
    let (bands, texts) = (count(rig, BAND_COMMIT)?, count(rig, TEXT_COMMIT)?);
    drag(ctx, rig, page, ROW_ALT, Some("alt"))?;
    let alt_band = count(rig, BAND_COMMIT)? - bands;
    let alt_text = count(rig, TEXT_COMMIT)? - texts;
    report.note(format!(
        "Alt-drag over a row: band commits {alt_band}, text commits {alt_text}"
    ));
    if alt_band != 1 || alt_text != 0 {
        return Ok(Some(format!(
            "★★★ an Alt-drag over a text row made {alt_band} box commits and {alt_text} \
             text commits; a box (1, 0) was expected."
        )));
    }
    // The control: the same gesture without Alt follows the text.
    let bands = count(rig, BAND_COMMIT)?;
    drag(ctx, rig, page, ROW_PLAIN, None)?;
    let plain_text = count(rig, TEXT_COMMIT)? - texts;
    let plain_band = count(rig, BAND_COMMIT)? - bands;
    report.note(format!(
        "plain drag over a row: text commits {plain_text}, band commits {plain_band}"
    ));
    if plain_text != 1 {
        return Ok(Some(format!(
            "★ a plain drag over a text row made {plain_text} text commits, so the row \
             holds no text here and the Alt step proved nothing."
        )));
    }
    if plain_band != 0 {
        return Ok(Some(format!(
            "★★★ a text highlight also made {plain_band} box commits from the same release."
        )));
    }
    Ok(None)
}

/// The Properties block is reachable, and its Ellipse authors a washed ellipse.
fn ellipse_step(
    ctx: &CheckContext,
    rig: &Rig,
    page: PageGeometry,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    rig.click(PROPERTIES_TAB)?;
    rig.session.settle(20);
    if rig.region(HIGHLIGHT_BLOCK).is_err() {
        return Ok(Some(format!(
            "★★ with the highlighter armed the Properties tab drew no `{HIGHLIGHT_BLOCK}` block, \
             so its shapes cannot be reached."
        )));
    }
    rig.click(&format!("{SHAPE_PREFIX}{ELLIPSE}"))?;
    drag(ctx, rig, page, ELLIPSE_BOX, None)?;
    let ellipse = last_raw(rig, AUTHOR)?;
    report.note(format!("ellipse: {ellipse:?}"));
    if !ellipse.as_deref().is_some_and(|l| {
        l.contains("kind=Ellipse") && l.contains("wash=Shape") && l.contains("highlighter=true")
    }) {
        return Ok(Some(format!(
            "★★★ with Ellipse chosen a drag authored {ellipse:?}; a washed, multiplied ellipse \
             was expected."
        )));
    }
    Ok(None)
}

/// Polygon: Backspace takes back a corner, Enter authors a washed polygon.
fn polygon_step(
    ctx: &CheckContext,
    rig: &Rig,
    page: PageGeometry,
    report: &mut CheckReport,
) -> Result<Option<String>> {
    rig.click(&format!("{SHAPE_PREFIX}{POLYGON}"))?;
    for corner in CORNERS {
        rig.pointer
            .click(&rig.session, at(ctx, rig, page, corner)?)?;
        rig.session.settle(10);
    }
    rig.pointer.key(&rig.session, None, "Backspace", None)?;
    rig.session.settle(15);
    let back = last_raw(rig, VERTEX_KEY)?;
    report.note(format!("after Backspace: {back:?}"));
    if !back
        .as_deref()
        .is_some_and(|l| l.contains("key=backspace vertices=3"))
    {
        return Ok(Some(format!(
            "★★★ Backspace after four corners traced {back:?}; three corners left was expected."
        )));
    }
    let authored = count(rig, AUTHOR)?;
    rig.pointer.key(&rig.session, None, "Enter", None)?;
    rig.session.settle(25);
    let polygon = last_raw(rig, AUTHOR)?;
    report.note(format!("after Enter: {polygon:?}"));
    if count(rig, AUTHOR)? != authored + 1
        || !polygon.as_deref().is_some_and(|l| {
            l.contains("kind=Polygon") && l.contains("wash=Shape") && l.contains("highlighter=true")
        })
    {
        return Ok(Some(format!(
            "★★★ Enter on a three-corner polygon authored {polygon:?}; one washed, multiplied \
             polygon was expected."
        )));
    }
    Ok(None)
}
