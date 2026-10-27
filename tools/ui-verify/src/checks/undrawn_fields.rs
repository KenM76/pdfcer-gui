//! `a_form_without_drawn_values_draws_and_clicks` — a form that asks the
//! viewer to draw its values (`/NeedAppearances`, no `/AP`) shows each value
//! and its border on the page, opens each field on a click, and is not
//! modified by being opened — `OPERATOR_REQUESTS.md` O327.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/undrawn_fields.md`.

use super::dimdrive::launch_path;
use super::savedannots::read;
use crate::checks::driving;
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::{Image, Rgb};
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

/// The Forms panel open, so its canvas note is drawn and traced.
const INVOKE: &str = "mode.review,view.panel_forms"; // ui-text-exempt: command ids
const FIXTURE: &str = "need-appearances-fields.pdf";
const METHOD: &str = "Rebuild it with `python fixtures/need-appearances-fields.PROVENANCE.py`.";
const BOX_LINE: &str = "form-box"; // ui-text-exempt: a trace event name
const FOCUS: &str = "form-focus"; // ui-text-exempt: a trace event name
const SAVED: &str = "save-in-place"; // ui-text-exempt: a trace event name
const NOTE: &str = "forms-canvas-note"; // ui-text-exempt: a trace event name
/// A claim the panel must not make: the fields are drawn and clickable.
const STALE: &str = "not drawn on the page";
/// The disclosure the panel owes: the file stores no drawing for them.
const OWED: &str = "store no drawn value in the file"; // ui-text-exempt: a phrase the check searches for
/// Fewest dark pixels a drawn "Hello" leaves inside its box.
const MIN_INK: usize = 20;
/// Fewest saturated-blue pixels a drawn `/MK /BC [0 0 1]` border leaves.
const MIN_BLUE: usize = 40;

/// See the module documentation.
pub struct AFormWithoutDrawnValuesDrawsAndClicks;

impl Check for AFormWithoutDrawnValuesDrawsAndClicks {
    fn name(&self) -> &'static str {
        "a_form_without_drawn_values_draws_and_clicks"
    }

    fn defect(&self) -> &'static str {
        "a form that asks the viewer to draw its values shows empty boxes that cannot be \
         clicked, and the Forms panel says they are not drawn, where Acrobat and other readers \
         draw them and let them be filled"
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

/// One `form-box kind=text` line: its page and canvas rectangle.
struct TextBox {
    page: usize,
    field: String,
    rect: (f64, f64, f64, f64),
}

fn text_boxes(trace: &crate::trace::Trace) -> Vec<TextBox> {
    trace
        .events(BOX_LINE)
        .filter(|l| l.get("kind") == Some("text"))
        .filter_map(|l| {
            let raw = l.get("rect")?;
            let (min, size) = raw.split_once(")+(")?;
            let (x, y) = min.trim_start_matches('(').split_once(',')?;
            let (w, h) = size.trim_end_matches(')').split_once(',')?;
            Some(TextBox {
                page: l.get("page")?.parse().ok()?,
                field: l.get("field")?.to_owned(),
                rect: (
                    x.trim().parse().ok()?,
                    y.trim().parse().ok()?,
                    w.trim().parse().ok()?,
                    h.trim().parse().ok()?,
                ),
            })
        })
        .collect()
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let doc = ctx.out("undrawn_fields.pdf");
    let source = match &ctx.pdf {
        Some(pdf) => pdf.clone(),
        None => driving::repo_fixture(FIXTURE, METHOD)?,
    };
    std::fs::copy(&source, &doc)
        .map_err(|e| Error::new(format!("copying {}: {e}", source.display())))?;
    report.note(format!("document: {}", source.display()));
    let original = read(&doc)?;
    let page = crate::fixture::page_geometry(&doc)
        .ok_or_else(|| Error::new(format!("no page size in {}. SKIPPED.", source.display())))?;
    let (session, pointer) = launch_path(ctx, report, doc.clone(), INVOKE, "undrawn_fields", &[])?;
    report.artifact(session.trace_path().to_path_buf());
    let outcome = measure(ctx, report, (&session, &pointer), page, (&doc, &original));
    let parked = pointer.gone(&session);
    match outcome? {
        Some(failure) => Ok(Some(failure)),
        None => parked.map(|_| None),
    }
}

fn measure(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
    (doc, original): (&std::path::Path, &[u8]),
) -> Result<Option<String>> {
    session.settle(45);
    let trace = session.trace()?;
    let path = session.trace_path().display().to_string();
    let boxes = text_boxes(&trace);
    if boxes.len() < 3 {
        return Ok(Some(format!(
            "★★★ the page offers {} clickable text field(s) of the form's three: a field with \
             no stored drawing is not on the canvas. Trace: {path}.",
            boxes.len()
        )));
    }
    if let Some(failure) = drawn(ctx, report, (session, pointer), page, &boxes)? {
        return Ok(Some(failure));
    }
    if let Some(failure) = noted(report, &trace, &path) {
        return Ok(Some(failure));
    }
    if let Some(failure) = unmodified(report, (session, pointer), doc, original)? {
        return Ok(Some(failure));
    }
    clicked(ctx, report, (session, pointer), page, &boxes[0])
}

/// Each box's window rectangle in capture pixels, widened by two pixels so a
/// border on the edge is inside it.
fn capture_rect(
    ctx: &CheckContext,
    session: &Session,
    page: PageGeometry,
    b: &TextBox,
) -> Result<PixRect> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, b.page)?;
    let (x, y, w, h) = b.rect;
    let corner =
        |cx: f64, cy: f64| mapping.doc_to_window(DocPoint::new(b.page, cx, page.height_pt - cy));
    let (a, z) = (corner(x, y)?, corner(x + w, y + h)?);
    let rect = LRect::new(
        Pt::new(a.x().min(z.x()) - 2.0, a.y().min(z.y()) - 2.0),
        Pt::new(a.x().max(z.x()) + 2.0, a.y().max(z.y()) + 2.0),
    );
    Ok(session.frame()?.logical_to_capture_pixels(rect))
}

fn is_ink(p: Rgb) -> bool {
    p.r < 90 && p.g < 90 && p.b < 90
}

fn is_blue(p: Rgb) -> bool {
    p.r < 110 && p.g < 110 && p.b > 170
}

/// Every box holds dark ink (the value) and saturated blue (the border).
fn drawn(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
    boxes: &[TextBox],
) -> Result<Option<String>> {
    let png = ctx.out("undrawn-fields.png");
    pointer.screenshot(session, &png)?;
    let image = Image::load_png(&png)?;
    report.artifact(png);
    for b in boxes {
        let px = capture_rect(ctx, session, page, b)?;
        let (ink, blue) = image.pixels_in(px).fold((0, 0), |(i, u), p| {
            (i + usize::from(is_ink(p)), u + usize::from(is_blue(p)))
        });
        report.note(format!(
            "{}: {ink} ink and {blue} blue pixel(s) in {px:?}",
            b.field
        ));
        if ink < MIN_INK || blue < MIN_BLUE {
            return Ok(Some(format!(
                "★★★ {} is not drawn on the page: {ink} dark pixel(s) (needs {MIN_INK}) and \
                 {blue} border-blue pixel(s) (needs {MIN_BLUE}) inside its box {px:?}.",
                b.field
            )));
        }
    }
    Ok(None)
}

/// The Forms panel's note says the canvas drew the fields, not that it
/// could not.
fn noted(report: &mut CheckReport, trace: &crate::trace::Trace, path: &str) -> Option<String> {
    let Some(line) = trace.events(NOTE).last() else {
        return Some(format!(
            "★★ the Forms panel traced no `{NOTE}` line: it did not say how the page shows the \
             form's fields. Trace: {path}."
        ));
    };
    report.note(format!("`{}`", line.raw));
    if line.raw.contains(STALE) {
        return Some(format!(
            "★★ the Forms panel still says the fields are {STALE:?}: `{}`.",
            line.raw
        ));
    }
    (!line.raw.contains(OWED)).then(|| {
        format!(
            "★★ the Forms panel does not say the fields {OWED:?}, so the operator is not told \
             another reader may show them empty: `{}`.",
            line.raw
        )
    })
}

/// Ctrl+S on the just-opened document writes the bytes it read.
fn unmodified(
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    doc: &std::path::Path,
    original: &[u8],
) -> Result<Option<String>> {
    pointer.key(session, None, "S", Some("ctrl"))?;
    session.settle(40);
    if !session
        .trace()?
        .events(SAVED)
        .any(|l| l.get("outcome") == Some("ok"))
    {
        return Ok(Some(format!(
            "Ctrl+S traced no `{SAVED} outcome=ok`. Trace: {}.",
            session.trace_path().display()
        )));
    }
    let saved = read(doc)?;
    report.note(format!(
        "saved {} byte(s); opened {}",
        saved.len(),
        original.len()
    ));
    Ok((saved != original).then(|| {
        format!(
            "★★★ opening the form modified it: Save wrote {} byte(s) where {} were opened, so \
             the drawing went into the document rather than onto the screen.",
            saved.len(),
            original.len()
        )
    }))
}

/// A click on the box opens it for filling.
fn clicked(
    ctx: &CheckContext,
    report: &mut CheckReport,
    (session, pointer): (&Session, &ScriptedPointer),
    page: PageGeometry,
    b: &TextBox,
) -> Result<Option<String>> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, b.page)?;
    let (x, y, w, h) = b.rect;
    let point = mapping.doc_to_window(DocPoint::new(
        b.page,
        x + w / 2.0,
        page.height_pt - (y + h / 2.0),
    ))?;
    pointer.click(session, point)?;
    session.settle(25);
    let trace = session.trace()?;
    let Some(focus) = trace.events(FOCUS).last() else {
        return Ok(Some(format!(
            "★★★ clicking {} opened nothing for filling: no `{FOCUS}` line. Trace: {}.",
            b.field,
            session.trace_path().display()
        )));
    };
    report.note(format!("`{}`", focus.raw));
    Ok((focus.get("field") != Some(b.field.as_str())).then(|| {
        format!(
            "clicking {} opened a different field: `{}`.",
            b.field, focus.raw
        )
    }))
}
