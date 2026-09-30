//! `ocr_layer_view` — **View ▸ Display's OCR switch paints the invisible text,
//! and its slider's two ends are the page alone and the text alone.**
//!
//! Design and rationale: `docs/modules/ui-verify/checks/ocr_layer_view.md`.

use crate::checks::driving::{
    SHELL_DIAG_ENV, click_mode_segment, declared, declared_names, declared_or_in_overflow, list,
};
use crate::checks::{Check, CheckContext};
use crate::coords::{CanvasMapping, DocPoint, PageGeometry, WindowFrame};
use crate::error::{Error, Result};
use crate::geom::{LRect, PixRect, Pt};
use crate::input::Driver;
use crate::launch::{LaunchSpec, Session};
use crate::pixels;
use crate::report::CheckReport;

const MODE: &str = "read";
/// Page 1 holds `Visible page one` drawn, and `recognised one` at render mode 3.
const FIXTURE: &str = "ocr-layers.pdf";
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
        let mut report = CheckReport::new(self.name(), self.defect());
        match drive(ctx, &mut report) {
            Ok(Some(failure)) => report.fail(failure),
            Ok(None) => report.pass(),
            Err(why) => report.from_error(&why),
        }
    }
}

/// Ink pixels in the drawn band and in the invisible band, from one capture.
struct Ink {
    drawn: usize,
    ocr: usize,
}

/// A launched window's canvas, mapped, so a band in PDF points becomes capture pixels.
pub(super) struct Scene<'a> {
    pub(super) session: &'a Session,
    pub(super) frame: WindowFrame,
    pub(super) mapping: CanvasMapping,
    pub(super) canvas: PixRect,
}

impl Scene<'_> {
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

    fn ink(&self, shot: &str, ctx: &CheckContext, report: &mut CheckReport) -> Result<Ink> {
        let path = ctx.out(&format!("ocr_layer_view.{shot}.png"));
        let img = crate::capture::window_to_png(self.session, &path)?;
        report.artifact(path);
        let ink = Ink {
            drawn: pixels::ink_run_into(&img, self.band(DRAWN_BAND)?).ink,
            ocr: pixels::ink_run_into(&img, self.band(OCR_BAND)?).ink,
        };
        report.note(format!(
            "{shot}: ink {} px on the drawn line, {} px on the recognised line",
            ink.drawn, ink.ocr
        ));
        Ok(ink)
    }
}

fn drive(ctx: &CheckContext, report: &mut CheckReport) -> Result<Option<String>> {
    let exe = ctx.resolve_exe().ok_or_else(|| {
        Error::new(format!(
            "no binary to drive. Pass --exe, or build the profile's default at {}.",
            ctx.profile.default_exe
        ))
    })?;
    let pdf = crate::fixture::workspace_root()
        .join("fixtures")
        .join(FIXTURE);
    if !pdf.is_file() {
        return Ok(Some(format!(
            "the committed fixture is not at {}: a broken checkout.",
            pdf.display()
        )));
    }
    if !ctx.allow_input {
        return Err(Error::new(
            "input is disabled (--no-input). This check clicks the ribbon and drags a slider.",
        ));
    }
    let ui_rect = ctx
        .profile
        .vocab
        .ui_rect_event
        .ok_or_else(|| Error::new("the profile declares no ui-rect trace event."))?;
    let page: PageGeometry = crate::fixture::page_geometry(&pdf)
        .ok_or_else(|| Error::new("could not read a page size from the fixture."))?;

    let mut spec = LaunchSpec::new(&exe, ctx.out("ocr_layer_view.trace.txt"));
    spec.pdf = Some(pdf);
    spec.env.push((
        ctx.profile.diag_env.0.to_owned(),
        ctx.profile.diag_env.1.to_owned(),
    ));
    spec.env
        .push((SHELL_DIAG_ENV.0.to_owned(), SHELL_DIAG_ENV.1.to_owned()));
    spec.allow_stale = ctx.allow_stale;
    spec.source_root = ctx.source_root.clone();
    let session = Session::launch(&spec, ctx.profile.trace_prefix)?;
    report.artifact(session.trace_path().to_path_buf());
    session.settle(40);
    let driver = Driver::new(session.window());
    click_mode_segment(&session, &driver, ui_rect, MODE)?;
    session.settle(20);

    let trace = session.trace()?;
    let tab = declared(&trace, ui_rect, TAB).ok_or_else(|| {
        Error::new(format!(
            "no `{TAB}` region in {MODE}. Tabs declared: {}.",
            list(&declared_names(&trace, ui_rect, "ribbon.tab."))
        ))
    })?;
    driver.click_at(session.frame()?.declared_center(tab))?;
    session.settle(20);

    let frame = session.frame()?;
    let trace = session.trace()?;
    let canvas =
        frame.logical_to_capture_pixels(declared(&trace, ui_rect, VIEWPORT).ok_or_else(|| {
            Error::new(format!(
                "no `{VIEWPORT}` region, so no canvas to measure. SKIPPED."
            ))
        })?);
    let scene = Scene {
        session: &session,
        frame,
        mapping: CanvasMapping::from_trace(&trace, &ctx.profile.vocab, page, 0)?,
        canvas,
    };

    // The switch is not remembered across launches, so it starts off.
    if declared(&trace, ui_rect, SLIDER).is_some() {
        return Ok(Some(format!(
            "`{SLIDER}` is declared before the switch was pressed: the blend is drawn with the \
             layer off, or the layer came up on by itself."
        )));
    }
    let off = scene.ink("off", ctx, report)?;
    if off.drawn == 0 {
        return Err(Error::new(
            "the drawn line has no ink in the capture, so the bands are not where the fixture \
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
    if !press_toggle(&session, &driver, ui_rect)? {
        return Ok(Some(format!(
            "no `{TOGGLE}` on the View tab or in its overflow. Items declared: {}.",
            list(&declared_names(
                &session.trace()?,
                ui_rect,
                "ribbon.item.view."
            ))
        )));
    }
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
        let frame = session.frame()?;
        // Press on the rail's first tenth, which is rail whatever the label
        // and value box take, and drag well past the control's edge.
        let from = frame.declared_at(slider, 0.05, 0.5);
        let to = frame.offset_from(from, slider.width() * 0.5 + overshoot, 0.0);
        driver.drag(from, to)?;
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
            (ink.ocr == 0 || ink.drawn != 0).then(|| {
                format!(
                    "at the right end the page should be gone and the text alone: the drawn \
                     line keeps {} ink pixels and the recognised line has {}.",
                    ink.drawn, ink.ocr
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
    if !press_toggle(&session, &driver, ui_rect)? {
        return Err(Error::new(format!(
            "`{TOGGLE}` was pressed once and could not be found again."
        )));
    }
    if declared(&session.trace()?, ui_rect, SLIDER).is_some() {
        return Ok(Some(format!(
            "the switch was turned off and `{SLIDER}` is still declared."
        )));
    }
    report.note("the switch paints the layer, both slider ends are exact, and off is off");
    Ok(None)
}

fn press_toggle(session: &Session, driver: &Driver, ui_rect: &str) -> Result<bool> {
    let Some(item) = declared_or_in_overflow(session, driver, ui_rect, TOGGLE)? else {
        return Ok(false);
    };
    driver.click_at(session.frame()?.declared_center(item))?;
    session.settle(30);
    Ok(true)
}
