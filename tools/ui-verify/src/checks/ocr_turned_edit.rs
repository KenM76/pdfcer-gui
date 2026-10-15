//! `a_recognised_line_on_a_turned_page_edits_upright` — on a page stored
//! sideways with `/Rotate 90`, editing a recognised line with the text layer
//! shown previews it upright, along the words, at the layer's own size. See
//! `docs/modules/ui-verify/checks/ocr_turned_edit.md`.

use crate::checks::dimdrive::launch_path;
use crate::checks::invisible_text_scripted::streams;
use crate::checks::ocr_edit_preview::layer_hued;
use crate::checks::ocr_scripted::recognise;
use crate::checks::ocr_turned_page::{contains, text_matrices};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::report::CheckReport;

const ENGINE: &str = "ocrs";
const FIXTURE: &str = "scan-rotate-90.pdf";
/// Edit mode, the OCR layer shown, the Edit Text tool armed.
const INVOKE: &str = "mode.edit,view.ocr_layer,edit.text"; // ui-text-exempt: command ids, never displayed
const SHAPED: &str = "text-edit-shaped"; // ui-text-exempt: a trace event name, never displayed
const FALLBACK: &str = "text-edit-preview-fallback"; // ui-text-exempt: a trace event name, never displayed
/// Fewest layer-coloured pixels that count as a drawn line.
const MIN_INK: usize = 20;

/// See the module documentation.
pub struct ARecognisedLineOnATurnedPageEditsUpright;

impl Check for ARecognisedLineOnATurnedPageEditsUpright {
    fn name(&self) -> &'static str {
        "a_recognised_line_on_a_turned_page_edits_upright"
    }

    fn defect(&self) -> &'static str {
        "editing recognised text on a page with /Rotate previews it across the words, at another \
         size, or in a stand-in (OPERATOR_REQUESTS.md O294)"
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

/// The first recognised line: its text matrix and `Tf` size.
struct Line {
    m: [f64; 6],
    size: f64,
}

impl Line {
    /// The page point `along` and `up` from the line's origin, in units of
    /// its size, in text space.
    fn at(&self, along: f64, up: f64) -> (f64, f64) {
        let m = self.m;
        let (a, u) = (along * self.size, up * self.size);
        (m[4] + a * m[0] + u * m[2], m[5] + a * m[1] + u * m[3])
    }
}

/// The first `Tm` in the first invisible stream, and the last `Tf` size
/// before it.
fn first_line(saved: &[u8]) -> Option<Line> {
    let body = streams(saved).into_iter().find(|b| contains(b, b"3 Tr"))?;
    let m = *text_matrices(&body).first()?;
    let text = String::from_utf8_lossy(&body);
    let tokens: Vec<&str> = text.split_ascii_whitespace().collect();
    let tm = tokens.iter().position(|t| *t == "Tm")?;
    let tf = tokens[..tm].iter().rposition(|t| *t == "Tf")?;
    let size = tokens.get(tf.checked_sub(1)?)?.parse().ok()?;
    Some(Line { m, size })
}

/// Width and height of the bounding box of the layer-coloured pixels in
/// `region`, in capture pixels.
fn ink_extent(image: &Image, region: PixRect) -> (u32, u32) {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for y in region.y..region.y + region.h {
        for x in region.x..region.x + region.w {
            if layer_hued(image, PixRect::new(x, y, 1, 1)) == 1 {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
    }
    if x1 < x0 {
        (0, 0)
    } else {
        (x1 - x0 + 1, y1 - y0 + 1)
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let fixture = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    let copy = ctx.out("ocr-turned-edit.pdf");
    std::fs::copy(&fixture, &copy).map_err(|e| Error::new(format!("copying {FIXTURE}: {e}")))?;
    recognise(ctx, report, &copy, ENGINE, "turned-edit", true)?;
    let saved = std::fs::read(&copy).map_err(|e| Error::new(format!("reading the save: {e}")))?;
    let line = first_line(&saved)
        .ok_or_else(|| Error::new("the saved file holds no invisible line with a Tm and a Tf."))?;
    report.note(format!(
        "first line: matrix {:?}, size {}",
        line.m, line.size
    ));
    let m = line.m;
    if !(m[0].abs() < 1e-3 && m[3].abs() < 1e-3 && m[1] > 0.0 && m[2] < 0.0) {
        return Ok(Some(format!(
            "★ the recognised layer is not turned with the page: its first line's matrix is \
             {m:?}, where /Rotate 90 needs `0 b c 0` with b > 0 and c < 0, so no edit of it can \
             preview upright."
        )));
    }
    let page = crate::fixture::page_geometry(&copy)
        .ok_or_else(|| Error::new("could not read a page size from the recognised copy."))?;
    let (session, pointer) = launch_path(ctx, report, copy, INVOKE, "ocr-turned-edit", &[])?;
    let seen = edit(ctx, report, &session, &pointer, &line, page);
    let parked = pointer.gone(&session);
    let seen = seen?;
    parked?;
    Ok(seen.map(|why| format!("{why} Trace: {}.", session.trace_path().display())))
}

/// The window rectangle holding the line from one size before its origin to
/// sixteen sizes along it, and from 0.6 below its baseline to 1.6 above,
/// clipped to the page.
fn line_band(mapping: &CanvasMapping, line: &Line, page: PageGeometry) -> Result<LRect> {
    let (mut lo, mut hi) = (Pt::new(f32::MAX, f32::MAX), Pt::new(f32::MIN, f32::MIN));
    for (a, u) in [(-1.0, -0.6), (16.0, -0.6), (-1.0, 1.6), (16.0, 1.6)] {
        let (x, y) = line.at(a, u);
        let (x, y) = (x.clamp(0.0, page.width_pt), y.clamp(0.0, page.height_pt));
        let p = mapping.doc_to_window(DocPoint::new(0, x, y))?;
        lo = Pt::new(lo.x.min(p.x()), lo.y.min(p.y()));
        hi = Pt::new(hi.x.max(p.x()), hi.y.max(p.y()));
    }
    Ok(LRect::new(lo, hi))
}

/// What one screenshot shows in the line's band.
#[derive(Clone, Copy)]
struct Ink {
    w: u32,
    h: u32,
    count: usize,
}

fn edit(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
    line: &Line,
    page: PageGeometry,
) -> Result<Option<String>> {
    let mapping = CanvasMapping::from_trace(&session.trace()?, &ctx.profile.vocab, page, 0)?;
    let region = session
        .frame()?
        .logical_to_capture_pixels(line_band(&mapping, line, page)?);
    let shot = |name: &str| -> Result<Ink> {
        let png = ctx.out(&format!("ocr-turned-edit-{name}.png"));
        pointer.screenshot(session, &png)?;
        let image = Image::load_png(&png)?;
        let (w, h) = ink_extent(&image, region);
        Ok(Ink {
            w,
            h,
            count: layer_hued(&image, region),
        })
    };
    let before = shot("before")?;
    report.note(format!(
        "before: ink {}x{} px, {} layer pixels, in {region:?}",
        before.w, before.h, before.count
    ));
    if before.count < MIN_INK || before.w <= 3 * before.h {
        return Ok(Some(format!(
            "★ the layer does not draw the first line as a horizontal line before the edit: ink \
             {}x{} px, {} layer pixels, in {region:?}.",
            before.w, before.h, before.count
        )));
    }
    let mark = session.trace()?.mark();
    let (x, y) = line.at(2.0, 0.35);
    pointer.click(session, mapping.doc_to_window(DocPoint::new(0, x, y))?)?;
    session.settle(20);
    let clicked = shot("clicked")?;
    pointer.type_text(session, None, "XQ")?;
    session.settle(30);
    let typed = shot("typed")?;
    for (name, ink) in [("clicked", clicked), ("typed", typed)] {
        report.note(format!(
            "{name}: ink {}x{} px, {} layer pixels",
            ink.w, ink.h, ink.count
        ));
        if let Some(why) = judge_ink(name, before, ink) {
            return Ok(Some(why));
        }
    }
    let trace = session.trace()?;
    let shaped = trace.last_after(SHAPED, mark).map(|l| l.raw.clone());
    let fallback = trace.last_after(FALLBACK, mark).map(|l| l.raw.clone());
    report.note(format!("{shaped:?}; fallback {fallback:?}"));
    let fell_back = fallback
        .as_deref()
        .is_some_and(|l| !l.contains("reason=none"));
    Ok((shaped.is_none() || fell_back).then(|| {
        format!(
            "★★★ the draft was not laid out in the line's own font: last `{SHAPED}` {shaped:?}, \
             last `{FALLBACK}` {fallback:?}."
        )
    }))
}

/// `None` when the draft's ink is at least 0.8 of the layer's width and at
/// most 1.5 of its height: a draft laid across the words is tall and narrow.
fn judge_ink(name: &str, before: Ink, after: Ink) -> Option<String> {
    let narrow = f64::from(after.w) < 0.8 * f64::from(before.w);
    let tall = f64::from(after.h) > 1.5 * f64::from(before.h);
    (after.count < MIN_INK || narrow || tall).then(|| {
        format!(
            "★★ once {name}, the line's draft is not drawn where the layer drew it: ink {}x{} px \
             and {} layer pixels, where the layer drew {}x{} px.",
            after.w, after.h, after.count, before.w, before.h
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{Ink, first_line, judge_ink};

    #[test]
    fn the_first_line_reads_its_matrix_and_size() {
        let pdf = b"4 0 obj\n<< /Length 50 >>\nstream\nBT 3 Tr /F1 9 Tf 0 1 -1 0 40 50 Tm (a) Tj ET\nendstream\nendobj\n";
        let line = first_line(pdf).expect("a line");
        assert_eq!(line.m, [0.0, 1.0, -1.0, 0.0, 40.0, 50.0]);
        assert!((line.size - 9.0).abs() < 1e-9);
        assert_eq!(line.at(1.0, 0.0), (40.0, 59.0));
    }

    #[test]
    fn a_draft_across_the_words_fails() {
        let before = Ink {
            w: 200,
            h: 20,
            count: 900,
        };
        assert!(
            judge_ink(
                "typed",
                before,
                Ink {
                    w: 210,
                    h: 22,
                    count: 950
                }
            )
            .is_none()
        );
        assert!(
            judge_ink(
                "typed",
                before,
                Ink {
                    w: 20,
                    h: 200,
                    count: 900
                }
            )
            .is_some()
        );
        assert!(
            judge_ink(
                "typed",
                before,
                Ink {
                    w: 400,
                    h: 40,
                    count: 900
                }
            )
            .is_some()
        );
    }
}
