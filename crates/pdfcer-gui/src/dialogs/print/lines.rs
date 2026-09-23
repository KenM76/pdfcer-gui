//! # `dialogs::print::lines` — print every line at one fixed width (O233)
//!
//! Off by default: a print carries the document's real line weights (O137).
//! On, every stroke is drawn at one width on paper, through the engine's
//! `StrokeDisplay::Fixed`. The operator asked for it by name, so this module
//! is the one exemption to O137's guard outside the canvas worker, and it
//! reads only the print dialog's own setting — never the canvas's
//! `view.line_weights`.
//!
//! # Contract
//!
//! - A width is resolved **on paper**, in points ([`paper_pt`]), then turned
//!   into device pixels for each render ([`apply`]): a render at `scale` pixels
//!   per page point of a page printed at `placement` paper points per page
//!   point draws `paper_pt / placement × scale` pixels. The preview, the
//!   zoomed detail and the spool call the same two functions, so the preview
//!   shows the width that prints.
//! - **Auto** is the thinnest line a printer renders and a reader still sees:
//!   [`AUTO_MM`], but never under one printer pixel at the job's resolution.

use egui::Ui;

use super::PrintDialog;
use crate::app::prefs::printing::{LINE_WIDTH_MM_RANGE, LineWidthPrefs};
use crate::text::print as t;

/// The published region of the fixed-line-width checkbox.
pub(super) const REGION_LINES_FIXED: &str = "print.lines.fixed";

/// Auto's width on paper, millimetres. 0.1 mm is the thin pen of CAD plot
/// styles, finer than ISO 128's thinnest standard line (0.13 mm) and still
/// plainly visible on a laser or inkjet print.
pub(crate) const AUTO_MM: f64 = 0.1;

/// The fixed width on paper, points, for a job printed at `dpi`; `None` when
/// the operator has not asked for one.
#[must_use]
pub(super) fn paper_pt(prefs: LineWidthPrefs, dpi: u32) -> Option<f64> {
    if !prefs.fixed {
        return None;
    }
    let pt = if prefs.auto {
        let one_pixel = 72.0 / f64::from(dpi.max(1));
        crate::units::points_from_mm(AUTO_MM).max(one_pixel)
    } else {
        crate::units::points_from_mm(prefs.width_mm)
    };
    (pt.is_finite() && pt > 0.0).then_some(pt)
}

/// Draw every stroke of a render at `paper_pt`. `scale` is the render's pixels
/// per page point, `placement` the page's paper points per page point. Leaves
/// `options` alone when `paper_pt` is `None`.
pub(super) fn apply(
    options: &mut pdfcer_render::RenderOptions,
    paper_pt: Option<f64>,
    scale: f64,
    placement: f64,
) {
    let Some(pt) = paper_pt else {
        return;
    };
    if !(placement > 0.0 && scale > 0.0) {
        return;
    }
    let device_px = (pt / placement * scale) as f32;
    options.stroke_display = pdfcer_render::font::StrokeDisplay::Fixed { device_px };
}

/// The Pages tab's line-width controls: *Fixed line width*, then *Auto*, then
/// the width itself once Auto is cleared.
pub(super) fn row(ui: &mut Ui, dialog: &mut PrintDialog) {
    let prefs = &mut dialog.lines;
    ui.horizontal_wrapped(|ui| {
        let fixed = ui
            .checkbox(&mut prefs.fixed, t::lines_fixed())
            .on_hover_text(t::lines_fixed_tooltip());
        crate::diag::ui_rect(REGION_LINES_FIXED, fixed.rect);
        if !prefs.fixed {
            return;
        }
        ui.checkbox(&mut prefs.auto, t::lines_auto())
            .on_hover_text(t::lines_auto_tooltip());
        if !prefs.auto {
            ui.add(
                egui::DragValue::new(&mut prefs.width_mm)
                    .range(LINE_WIDTH_MM_RANGE)
                    .speed(0.01)
                    .max_decimals(2)
                    .suffix(t::mm_suffix()),
            )
            .on_hover_text(t::lines_width_tooltip());
        }
    });
    if prefs.fixed {
        ui.label(egui::RichText::new(t::lines_caption(prefs.auto)).small());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefs(fixed: bool, auto: bool, width_mm: f64) -> LineWidthPrefs {
        LineWidthPrefs {
            fixed,
            auto,
            width_mm,
        }
    }

    #[test]
    fn off_changes_nothing() {
        assert_eq!(paper_pt(prefs(false, true, 0.5), 300), None);
        let mut o = pdfcer_render::RenderOptions::default();
        let before = format!("{:?}", o.stroke_display);
        apply(&mut o, None, 4.0, 1.0);
        assert_eq!(format!("{:?}", o.stroke_display), before);
    }

    #[test]
    fn auto_is_the_thin_pen_until_a_pixel_is_wider() {
        let pen = crate::units::points_from_mm(AUTO_MM);
        assert_eq!(paper_pt(prefs(true, true, 9.0), 600), Some(pen));
        // At 150 dpi one pixel is 0.48 pt, wider than 0.1 mm (0.28 pt).
        assert_eq!(paper_pt(prefs(true, true, 9.0), 150), Some(72.0 / 150.0));
    }

    #[test]
    fn a_typed_width_is_used_as_typed() {
        let pt = paper_pt(prefs(true, false, 0.5), 300).unwrap();
        assert!((pt - crate::units::points_from_mm(0.5)).abs() < 1e-12);
        assert_eq!(paper_pt(prefs(true, false, 0.0), 300), None);
    }

    /// A page printed at half size keeps its paper width: the render must
    /// draw twice as many page-space pixels.
    #[test]
    fn the_width_is_on_paper_whatever_the_placement() {
        let mut o = pdfcer_render::RenderOptions::default();
        apply(&mut o, Some(1.0), 300.0 / 72.0 * 0.5, 0.5);
        match o.stroke_display {
            pdfcer_render::font::StrokeDisplay::Fixed { device_px } => {
                assert!((device_px - 300.0 / 72.0).abs() < 1e-4);
            }
            other => panic!("expected Fixed, got {other:?}"),
        }
    }
}
