//! `field_outline` — **a text field placed with the dialog's defaults draws a
//! visible outline on the page.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/field_outline.md`.

use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::report::CheckReport;
use crate::trace::Trace;

/// Edit mode, then the text-field tool, one per frame.
const INVOKE: &str = "mode.edit,edit.form_text_field";
/// The seam that accepts the placement dialog with its defaults.
const ACCEPT_ENV: (&str, &str) = ("PDFCER_DIAG_FORM_ACCEPT", "1");
/// Traced by the `vector_edit` funnel for the authoring verb.
const AUTHORED: &str = "add-form-field";
/// The census of selectable widgets: `rect=(x,y)+(w,h)` in page points, y down.
const BOX_LINE: &str = "form-target";
/// The fixture's page, in points.
const PAGE: (f64, f64) = (300.0, 300.0);
/// Where the field is placed, PDF user space (y up).
const PLACE: (f64, f64) = (80.0, 200.0);
/// Blank paper for the deselecting click, PDF user space.
const BLANK: (f64, f64) = (250.0, 30.0);
/// A channel at or below this, on all three, is ink.
const INK: u8 = 110;
/// Pixels either side of the field's edge that count as the edge.
const BAND: u32 = 3;

/// See the module documentation.
pub struct DefaultFieldIsOutlined;

impl Check for DefaultFieldIsOutlined {
    fn name(&self) -> &'static str {
        "field_outline"
    }

    fn defect(&self) -> &'static str {
        "a text field placed with the dialog's defaults states no border colour, so it draws no \
         box: empty, it is invisible on the page and in a reader that does not highlight fields"
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

/// A blank one-page PDF with no form.
fn blank_pdf() -> Vec<u8> {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        format!(
            "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 {} {}] >>",
            PAGE.0, PAGE.1
        ),
        "<< /Type /Page /Parent 2 0 R /Resources << >> >>".to_owned(),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// The placed field's rect, `(x, y, w, h)` in page points with y down.
fn placed_rect(trace: &Trace) -> Option<(f64, f64, f64, f64)> {
    let raw = trace.last(BOX_LINE)?.get("rect")?.to_owned();
    let (min, size) = raw.split_once(")+(")?;
    let (x, y) = min.trim_start_matches('(').split_once(',')?;
    let (w, h) = size.trim_end_matches(')').split_once(',')?;
    Some((
        x.trim().parse().ok()?,
        y.trim().parse().ok()?,
        w.trim().parse().ok()?,
        h.trim().parse().ok()?,
    ))
}

/// Ink pixels in `region`.
fn ink_in(image: &Image, region: PixRect) -> usize {
    image
        .pixels_in(region)
        .filter(|p| p.r <= INK && p.g <= INK && p.b <= INK)
        .count()
}

/// The field's box in capture pixels.
fn box_pixels(
    session: &Session,
    mapping: &CanvasMapping,
    rect: (f64, f64, f64, f64),
) -> Result<PixRect> {
    let (x, y, w, h) = rect;
    // The census is y-down page points; `DocPoint` is PDF user space, y up.
    let tl = mapping.doc_to_window(DocPoint::new(0, x, PAGE.1 - y))?;
    let br = mapping.doc_to_window(DocPoint::new(0, x + w, PAGE.1 - (y + h)))?;
    let frame = session.frame()?;
    Ok(frame.logical_to_capture_pixels(LRect::new(
        Pt {
            x: tl.x(),
            y: tl.y(),
        },
        Pt {
            x: br.x(),
            y: br.y(),
        },
    )))
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check is two clicks and a keystroke.",
        ));
    }
    let vocab = &ctx.profile.vocab;
    let pdf = ctx.out("field_outline.blank.pdf");
    std::fs::write(&pdf, blank_pdf())
        .map_err(|why| Error::new(format!("could not write {}: {why}", pdf.display())))?;
    report.artifact(pdf.clone());

    let mut spec = LaunchSpec::new(&exe, ctx.out("field_outline.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push(("PDFCER_DIAG_INVOKE".to_owned(), INVOKE.to_owned()));
    spec.env
        .push((ACCEPT_ENV.0.to_owned(), ACCEPT_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);

    let page = PageGeometry {
        width_pt: PAGE.0,
        height_pt: PAGE.1,
    };
    let trace = session.trace()?;
    let mapping = CanvasMapping::from_trace(&trace, vocab, page, 0)?;
    let frame = session.frame()?;
    let driver = Driver::new(session.window());
    let place = mapping.doc_to_window(DocPoint::new(0, PLACE.0, PLACE.1))?;
    driver.click_at(frame.to_screen(place))?;
    session.settle(30);

    let trace = session.trace()?;
    if !trace
        .events(AUTHORED)
        .any(|l| !l.raw.contains("refused") && !l.raw.contains("failed"))
    {
        return Err(Error::new(format!(
            "no clean `{AUTHORED}` line after the placing click, so no field exists to look at. \
             `form_field` owns placement; run it. Trace: {}.",
            session.trace_path().display()
        )));
    }

    // Disarm, then clear the selection the placement left, so no selection
    // chrome is drawn over the field's own appearance.
    driver.press(crate::sys::vk::ESCAPE)?;
    session.settle(12);
    let blank = mapping.doc_to_window(DocPoint::new(0, BLANK.0, BLANK.1))?;
    driver.click_at(frame.to_screen(blank))?;
    session.settle(30);

    let trace = session.trace()?;
    let Some(rect) = placed_rect(&trace) else {
        return Err(Error::new(format!(
            "no `{BOX_LINE}` line after placement, so the field's position is unknown. Trace: {}.",
            session.trace_path().display()
        )));
    };
    report.note(format!("the field sits at page points {rect:?} (y down)"));

    let shot = ctx.out("field_outline.png");
    let image = crate::capture::window_to_png(&session, &shot)?;
    report.artifact(shot);
    let bx = box_pixels(&session, &mapping, rect)?;
    if bx.w <= 4 * BAND || bx.h <= 4 * BAND {
        return Err(Error::new(format!(
            "the field is {}x{} px on screen, too small to tell its edge from its inside.",
            bx.w, bx.h
        )));
    }
    let outer = PixRect {
        x: bx.x.saturating_sub(BAND),
        y: bx.y.saturating_sub(BAND),
        w: bx.w + 2 * BAND,
        h: bx.h + 2 * BAND,
    };
    let inner = PixRect {
        x: bx.x + BAND,
        y: bx.y + BAND,
        w: bx.w - 2 * BAND,
        h: bx.h - 2 * BAND,
    };
    let edge = ink_in(&image, outer).saturating_sub(ink_in(&image, inner));
    // A one-pixel frame round the box, as a count; half of it must be ink.
    let perimeter = 2 * (bx.w + bx.h) as usize;
    report.note(format!(
        "ink on the field's edge band: {edge} px, against a perimeter of {perimeter} px"
    ));
    if edge * 2 < perimeter {
        return Ok(Some(format!(
            "the field's edge carries {edge} ink pixels against a {perimeter}-pixel perimeter, \
             so no outline is drawn. Either the dialog's default border colour is unset, or the \
             engine did not stroke the frame. The screenshot beside this report shows which."
        )));
    }
    report.note("★ the placed field draws its outline");
    Ok(None)
}
