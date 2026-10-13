//! `ocr_layer_view` — **View ▸ Display's OCR switch paints the invisible text,
//! and its slider's two ends are the page alone and the text alone.** Driven
//! with the scripted pointer in a window placed off the desktop; pixels come
//! from the window's own frame.
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_layer_view.md`.

use super::dimdrive::{Fixture, press_on_tab, run_on, ui_rect_event};
use crate::checks::driving::{declared, declared_names, list};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowFrame, WindowPoint};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::image::Image;
use crate::input::scripted::ScriptedPointer;
use crate::launch::Session;
use crate::pixels;
use crate::report::CheckReport;

/// Page 1 holds `Visible page one` drawn, and `recognised one` at render mode 3.
pub(super) const FIXTURE: Fixture = Fixture {
    file: "ocr-layers.pdf",
    method: "Rebuild it with `python fixtures/ocr-layers.PROVENANCE.py`.",
    page: PageGeometry {
        width_pt: 612.0,
        height_pt: 792.0,
    },
};
const TAB: &str = "ribbon.tab.view";
const TOGGLE: &str = "ribbon.item.view.ocr_layer";
/// The slider, declared only while the switch is on.
const SLIDER: &str = "ribbon.item.ocr_blend";
const VIEWPORT: &str = "canvas-viewport"; // ui-text-exempt: a trace region name
/// `ocr-blend set=F`, whenever the slider moves.
const BLEND_EVENT: &str = "ocr-blend";
/// The drawn line's glyph band, `(x0, y_bottom, x1, y_top)` in PDF points.
const DRAWN_BAND: (f64, f64, f64, f64) = (72.0, 719.0, 150.0, 729.0);
/// The invisible line's glyph band.
pub(super) const OCR_BAND: (f64, f64, f64, f64) = (72.0, 699.0, 150.0, 709.0);
/// How far the slider is dragged past its own rect, in logical points, so the
/// pointer ends beyond the rail's end whatever the rail's length.
const OVERSHOOT_PT: f32 = 220.0;

/// See the module documentation.
pub struct OcrLayerIsShownAndBlended;

impl Check for OcrLayerIsShownAndBlended {
    fn name(&self) -> &'static str {
        "ocr_layer_view"
    }

    fn defect(&self) -> &'static str {
        "View ▸ Display's OCR text switch is offered and pressing it paints no recognised text, \
         or its blend slider's ends do not show the page alone and the text alone"
    }

    fn run(&self, ctx: &CheckContext) -> CheckReport {
        run_on(self, ctx, &FIXTURE, "mode.read", "ocr_layer_view", drive)
    }
}

/// Ink pixels in the drawn band and in the invisible band, from one picture.
struct Ink {
    drawn: usize,
    ocr: usize,
    /// Pixels on the drawn line farther than [`PAPER_TOLERANCE`] from paper.
    /// `drawn` needs strong contrast to count, so a veil short of opaque
    /// leaves a faded line it reads as no ink; this reads it.
    residue: usize,
}

/// How far, per channel, a pixel may sit from white paper and still be paper.
const PAPER_TOLERANCE: u8 = 3;

/// A launched window's canvas, mapped, so a band in PDF points becomes
/// screenshot pixels.
pub(super) struct Scene<'a> {
    pub(super) session: &'a Session,
    pub(super) pointer: &'a ScriptedPointer,
    pub(super) frame: WindowFrame,
    pub(super) mapping: CanvasMapping,
    pub(super) canvas: PixRect,
}

impl<'a> Scene<'a> {
    /// Map page 1 of [`FIXTURE`] and the canvas viewport as they are now.
    pub(super) fn now(
        ctx: &CheckContext,
        session: &'a Session,
        pointer: &'a ScriptedPointer,
    ) -> Result<Self> {
        let trace = session.trace()?;
        let frame = session.frame()?;
        let canvas = frame.logical_to_capture_pixels(
            declared(&trace, ui_rect_event(ctx)?, VIEWPORT).ok_or_else(|| {
                Error::new(format!(
                    "no `{VIEWPORT}` region, so no canvas to measure. SKIPPED."
                ))
            })?,
        );
        Ok(Self {
            session,
            pointer,
            frame,
            mapping: CanvasMapping::from_trace(&trace, &ctx.profile.vocab, FIXTURE.page, 0)?,
            canvas,
        })
    }

    /// `band`, `(x0, y_bottom, x1, y_top)` in page-1 PDF points, clipped to the canvas.
    pub(super) fn band(&self, band: (f64, f64, f64, f64)) -> Result<PixRect> {
        let (x0, y0, x1, y1) = band;
        let tl = self.mapping.doc_to_window(DocPoint::new(0, x0, y1))?;
        let br = self.mapping.doc_to_window(DocPoint::new(0, x1, y0))?;
        let r = self.frame.logical_to_capture_pixels(LRect::new(
            Pt::new(tl.x(), tl.y()),
            Pt::new(br.x(), br.y()),
        ));
        let c = self.canvas;
        let (x, y) = (r.x.max(c.x), r.y.max(c.y));
        let (x2, y2) = ((r.x + r.w).min(c.x + c.w), (r.y + r.h).min(c.y + c.h));
        if x2 <= x || y2 <= y {
            return Err(Error::new(format!(
                "the band {band:?} lies outside the canvas {c:?}. SKIPPED: the sheet is not \
                 laid out where this check expects."
            )));
        }
        Ok(PixRect::new(x, y, x2 - x, y2 - y))
    }

    /// The window's own frame, written to `<stem>.<shot>.png`.
    pub(super) fn shot(
        &self,
        ctx: &CheckContext,
        report: &mut CheckReport,
        stem: &str,
        shot: &str,
    ) -> Result<Image> {
        let path = ctx.out(&format!("{stem}.{shot}.png"));
        self.pointer.screenshot(self.session, &path)?;
        let img = Image::load_png(&path)?;
        report.artifact(path);
        Ok(img)
    }

    fn ink(&self, shot: &str, ctx: &CheckContext, report: &mut CheckReport) -> Result<Ink> {
        let img = self.shot(ctx, report, "ocr_layer_view", shot)?;
        let ink = Ink {
            drawn: pixels::ink_run_into(&img, self.band(DRAWN_BAND)?).ink,
            ocr: pixels::ink_run_into(&img, self.band(OCR_BAND)?).ink,
            residue: img
                .pixels_in(self.band(DRAWN_BAND)?)
                .filter(|px| px.r.min(px.g).min(px.b) < 255 - PAPER_TOLERANCE)
                .count(),
        };
        report.note(format!(
            "{shot}: ink {} px on the drawn line ({} off paper), {} px on the recognised line",
            ink.drawn, ink.residue, ink.ocr
        ));
        Ok(ink)
    }
}

/// The window point `(x, y)`, in logical points.
fn point(x: f32, y: f32) -> WindowPoint {
    WindowPoint::centre_of(LRect::new(Pt::new(x, y), Pt::new(x, y)))
}

fn drive(
    ctx: &CheckContext,
    report: &mut CheckReport,
    session: &Session,
    pointer: &ScriptedPointer,
) -> Result<Option<String>> {
    let ui_rect = ui_rect_event(ctx)?;
    super::dimdrive::press(ctx, session, pointer, TAB)?;
    let scene = Scene::now(ctx, session, pointer)?;

    // The switch is not remembered across launches, so it starts off.
    if declared(&session.trace()?, ui_rect, SLIDER).is_some() {
        return Ok(Some(format!(
            "`{SLIDER}` is declared before the switch was pressed: the blend is drawn with the \
             layer off, or the layer came up on by itself."
        )));
    }
    let off = scene.ink("off", ctx, report)?;
    if off.drawn == 0 {
        return Err(Error::new(
            "the drawn line has no ink in the picture, so the bands are not where the fixture \
             puts its text. SKIPPED: the control arm failed, and nothing below would mean \
             anything.",
        ));
    }
    if off.ocr != 0 {
        return Ok(Some(format!(
            "with the switch off the recognised line already shows {} ink pixels: the \
             invisible text is being drawn, or the fixture's layer is not at render mode 3.",
            off.ocr
        )));
    }

    // --- the switch ------------------------------------------------------
    press_on_tab(ctx, session, pointer, TOGGLE, TAB)?;
    let on = scene.ink("on", ctx, report)?;
    if on.ocr == 0 {
        return Ok(Some(
            "the OCR text switch was pressed and the recognised line is still blank.".to_owned(),
        ));
    }
    let Some(slider) = declared(&session.trace()?, ui_rect, SLIDER) else {
        return Ok(Some(format!(
            "the switch is on and the text shows, and no `{SLIDER}` is declared: the blend is \
             not offered. Items declared: {}.",
            list(&declared_names(&session.trace()?, ui_rect, "ribbon.item."))
        )));
    };

    // --- the two ends of the slider --------------------------------------
    for (overshoot, want, name) in [
        (OVERSHOOT_PT, "1.000", "right"),
        (-OVERSHOOT_PT, "0.000", "left"),
    ] {
        // Press on the rail's first twentieth, which is rail whatever the
        // label and value box take, and drag well past the control's edge.
        let (x, y) = (
            slider.min.x + slider.width() * 0.05,
            (slider.min.y + slider.max.y) / 2.0,
        );
        let to = x + slider.width() * 0.5 + overshoot;
        pointer.drag(session, point(x, y), point(to, y), 10)?;
        session.settle(20);
        let trace = session.trace()?;
        let Some(set) = trace.last(BLEND_EVENT) else {
            return Ok(Some(format!(
                "the slider was dragged {name} and no `{BLEND_EVENT}` line followed."
            )));
        };
        if set.get("set") != Some(want) {
            return Ok(Some(format!(
                "the slider was dragged past its {name} end and the blend reads `{}`, not \
                 set={want}.",
                set.raw
            )));
        }
        let ink = scene.ink(name, ctx, report)?;
        let failure = if want == "1.000" {
            (ink.ocr == 0 || ink.residue != 0).then(|| {
                format!(
                    "at the right end the page should be gone and the text alone: the drawn \
                     line keeps {} pixels off paper and the recognised line has {} ink pixels.",
                    ink.residue, ink.ocr
                )
            })
        } else {
            (ink.ocr != 0 || ink.drawn == 0).then(|| {
                format!(
                    "at the left end the page should be alone: the recognised line keeps {} ink \
                     pixels and the drawn line has {}.",
                    ink.ocr, ink.drawn
                )
            })
        };
        if let Some(failure) = failure {
            return Ok(Some(failure));
        }
    }

    // --- off again -------------------------------------------------------
    press_on_tab(ctx, session, pointer, TOGGLE, TAB)?;
    if declared(&session.trace()?, ui_rect, SLIDER).is_some() {
        return Ok(Some(format!(
            "the switch was turned off and `{SLIDER}` is still declared."
        )));
    }
    report.note("the switch paints the layer, both slider ends are exact, and off is off");
    Ok(None)
}
