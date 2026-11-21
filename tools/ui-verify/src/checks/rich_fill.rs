//! `rich_fill` — **a rich-text form field is typed into on the page, stays rich
//! text, draws what was typed and says which formatting the typing replaced**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/rich_fill.md`.

use super::dimdrive::launch_path;
use super::print_shop::await_line;
use crate::checks::driving;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const INVOKE: &str = "mode.review"; // ui-text-exempt: a command id
const FIXTURE: &str = "rich-field.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/rich-field.PROVENANCE.py`.";
const FOCUS: &str = "form-focus"; // ui-text-exempt: a trace event name
const FILL: &str = "form-fill-rich-text"; // ui-text-exempt: a trace event name
const REPLACED: &str = "replaced the formatting it held (bold)"; // ui-text-exempt: matched
/// The field's `/Rect`, PDF user space.
const RECT: (f64, f64, f64, f64) = (72.0, 600.0, 372.0, 640.0);

/// See the module documentation.
pub struct ARichTextFieldFillsOnThePage;

impl Check for ARichTextFieldFillsOnThePage {
    fn name(&self) -> &'static str {
        "a_rich_text_field_fills_on_the_page"
    }

    fn defect(&self) -> &'static str {
        "a rich-text form field cannot be typed into on the page, or typing into it writes a \
         plain value the engine refuses, or the fill draws nothing, or it replaces the field's \
         stored bold without saying so"
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
    let doc = ctx.out("rich_fill.pdf");
    let source = driving::repo_fixture(FIXTURE, METHOD)?;
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("no page size in {}.", source.display())))?;
    let (session, pointer) = launch_path(ctx, report, doc, INVOKE, "rich_fill", &[])?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(45);
    let outcome = fill(ctx, report, (&session, &pointer), page);
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

/// Picture the empty box, click it, type, commit, and read the fill.
fn fill(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
) -> Result<Option<String>> {
    let region = pixels(ctx, session, page)?;
    let before = picture(ctx, session, pointer, "before")?;
    if let Some(i) = ink(&before, region) {
        return Ok(Some(format!(
            "the fixture's box already holds ink {i:?} before any typing; the check cannot \
             tell a fill from the stored drawing."
        )));
    }
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let mark = session.trace()?.mark();
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, 360.0, 620.0))?,
    )?;
    if await_line(session, FOCUS, mark, |l| l.get("field") == Some("Note"))?.is_none() {
        return Ok(Some(format!(
            "★★★ a click on the rich-text field opened no editor (no `{FOCUS}` line): the \
             canvas does not offer it."
        )));
    }
    session.settle(15);
    pointer.type_text(session, None, " WWW")?;
    session.settle(15);
    let mark = session.trace()?.mark();
    pointer.click(
        session,
        mapping.doc_to_window(DocPoint::new(0, 480.0, 250.0))?,
    )?;
    let Some(line) = await_line(session, FILL, mark, |_| true)? else {
        return Ok(Some(format!(
            "★★★ leaving the rich-text field committed no rich fill (no `{FILL}` line)."
        )));
    };
    report.note(line.raw.clone());
    if line.get_usize("commands") != Some(1) {
        return Ok(Some(format!(
            "★★★ the rich fill pushed no command: {}",
            line.raw
        )));
    }
    if !line.raw.contains(REPLACED) {
        return Ok(Some(format!(
            "★★★ the fill replaced the field's stored bold without saying so (wanted \
             `{REPLACED}` in `redrawn=`): {}",
            line.raw
        )));
    }
    session.settle(45);
    let after = picture(ctx, session, pointer, "after")?;
    let Some(i) = ink(&after, region) else {
        return Ok(Some(
            "★★★ the rich fill was applied but the field draws nothing on the page.".to_owned(),
        ));
    };
    report.note(format!("ink after the fill {i:?}"));
    Ok(None)
}

fn picture(
    ctx: &CheckContext,
    session: &Session,
    pointer: &ScriptedPointer,
    stem: &str,
) -> Result<Image> {
    let png = ctx.out(&format!("rich-fill-{stem}.png"));
    pointer.screenshot(session, &png)?;
    Image::load_png(&png)
}

/// The field's box, less 1.5 pt, in capture pixels.
fn pixels(ctx: &CheckContext, session: &Session, page: PageGeometry) -> Result<PixRect> {
    let (x0, y0, x1, y1) = RECT;
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let a = mapping.doc_to_window(DocPoint::new(0, x0 + 1.5, y0 + 1.5))?;
    let z = mapping.doc_to_window(DocPoint::new(0, x1 - 1.5, y1 - 1.5))?;
    let rect = LRect::new(
        Pt::new(a.x().min(z.x()), a.y().min(z.y())),
        Pt::new(a.x().max(z.x()), a.y().max(z.y())),
    );
    Ok(session.frame()?.logical_to_capture_pixels(rect))
}

/// The bounding box `(x0, y0, x1, y1)` of the pixels darker than mid-grey.
fn ink(image: &Image, px: PixRect) -> Option<(u32, u32, u32, u32)> {
    let mut found: Option<(u32, u32, u32, u32)> = None;
    for y in px.y..px.y + px.h {
        for x in px.x..px.x + px.w {
            let Some(p) = image.pixel(x, y) else { continue };
            let luma = (u32::from(p.r) * 3 + u32::from(p.g) * 6 + u32::from(p.b)) / 10;
            if luma < 110 {
                found = Some(found.map_or((x, y, x, y), |(a, b, c, d)| {
                    (a.min(x), b.min(y), c.max(x), d.max(y))
                }));
            }
        }
    }
    found
}
