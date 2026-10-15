//! # `dialogs::ocr_upscale` — Recognise text's enlarge-before-reading option
//!
//! Opt-in and off by default: enlarging helps a scan whose text falls below
//! the recogniser's size window and can hurt one that already reads. The
//! group shows the factor, the kernel and the sizes the first chosen page is
//! drawn and read at, and warns before a size the engine refuses
//! (`MAX_OUTPUT_PIXELS`); the refusal itself is the engine's sentence at Run.
//!
//! R9: a model that cannot read through an enlargement
//! ([`crate::ocr::can_upscale`]) draws nothing and is sent nothing.

use pdfcer_core::ocr::addons::OcrModel;
use pdfcer_core::ocr::upscale::{MAX_FACTOR, MAX_OUTPUT_PIXELS, Upscale, UpscaleFilter};

use crate::text::ocrupscale as t;

/// The checkbox, for a driven check.
const REGION_ENLARGE: &str = "ocr-upscale"; // ui-text-exempt: trace region name
/// The Lanczos choice.
const REGION_LANCZOS: &str = "ocr-upscale-lanczos"; // ui-text-exempt: trace region name
/// The sizes line.
const REGION_SIZES: &str = "ocr-upscale-sizes"; // ui-text-exempt: trace region name

/// The smallest factor offered: anything less is not worth a pass.
pub(super) const MIN_FACTOR: f32 = 1.5;
/// The factor first offered.
pub(super) const DEFAULT_FACTOR: f32 = 2.0;

/// The operator's enlargement choice for one Recognise-text transaction.
#[derive(Debug, Clone, Copy)]
pub(super) struct Enlarge {
    on: bool,
    factor: f32,
    filter: UpscaleFilter,
}

impl Default for Enlarge {
    fn default() -> Self {
        Self {
            on: false,
            factor: DEFAULT_FACTOR,
            filter: UpscaleFilter::default(),
        }
    }
}

impl Enlarge {
    /// The enlargement the run asks of `model`: only when turned on and the
    /// model can take one.
    #[must_use]
    pub(super) fn upscale(&self, model: &OcrModel) -> Option<Upscale> {
        if !self.on || !crate::ocr::can_upscale(model) {
            return None;
        }
        Upscale::new(self.factor, self.filter).ok()
    }

    /// Draw the group; `raster` is the first chosen page's size as drawn.
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        model: Option<&OcrModel>,
        raster: Option<(u32, u32)>,
    ) {
        let Some(model) = model.filter(|m| crate::ocr::can_upscale(m)) else {
            return;
        };
        let check = ui
            .checkbox(&mut self.on, t::enlarge())
            .on_hover_text(t::enlarge_tooltip());
        crate::diag::ui_rect(REGION_ENLARGE, check.rect);
        if !self.on {
            return;
        }
        ui.indent(REGION_ENLARGE, |ui| {
            factor_and_filter(ui, &mut self.factor, &mut self.filter, REGION_LANCZOS);
            if let (Some(from), Some(up)) = (raster, self.upscale(model)) {
                let to = up.output_size(from.0, from.1);
                let sizes = ui.label(t::sizes(from, to));
                crate::diag::ui_rect(REGION_SIZES, sizes.rect);
                if u64::from(to.0) * u64::from(to.1) > MAX_OUTPUT_PIXELS as u64 {
                    ui.label(t::too_large(MAX_OUTPUT_PIXELS));
                }
            }
        });
    }
}

/// The factor slider and the kernel choice, also File ▸ Enlarge scans'.
/// `lanczos_region` names the Lanczos choice in the trace.
pub(super) fn factor_and_filter(
    ui: &mut egui::Ui,
    factor: &mut f32,
    filter: &mut UpscaleFilter,
    lanczos_region: &str,
) {
    ui.horizontal(|ui| {
        ui.label(t::factor());
        ui.add(
            egui::Slider::new(factor, MIN_FACTOR..=MAX_FACTOR)
                .step_by(0.5)
                .suffix(t::times()),
        );
    });
    ui.radio_value(filter, UpscaleFilter::Bicubic, t::bicubic());
    let lanczos = ui.radio_value(filter, UpscaleFilter::Lanczos, t::lanczos());
    crate::diag::ui_rect(lanczos_region, lanczos.rect);
}

/// The trace tokens for the run's `ocr-started` line: `factor filter`, or
/// `off -`.
#[must_use]
pub(super) fn trace_tokens(upscale: Option<Upscale>) -> (String, &'static str) {
    upscale.map_or_else(
        // ui-text-exempt: trace tokens, never displayed.
        || ("off".to_owned(), "-"),
        |u| (t::factor_text(u.factor()), u.filter().as_str()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(engine: &str) -> OcrModel {
        OcrModel {
            name: engine.to_owned(),
            engine: engine.to_owned(),
            folder: std::path::PathBuf::new(),
            root: std::path::PathBuf::new(),
            manifest: None,
        }
    }

    #[test]
    fn off_by_default_sends_nothing() {
        assert_eq!(Enlarge::default().upscale(&model("ocrs")), None);
    }

    #[test]
    fn paddle_vl_is_never_sent_an_enlargement() {
        let on = Enlarge {
            on: true,
            ..Enlarge::default()
        };
        assert!(on.upscale(&model("ocrs")).is_some());
        assert_eq!(on.upscale(&model("paddle-vl")), None);
    }

    #[test]
    fn every_offered_factor_is_one_the_engine_accepts() {
        let mut f = MIN_FACTOR;
        while f <= MAX_FACTOR {
            assert!(Upscale::new(f, UpscaleFilter::Lanczos).is_ok(), "{f}");
            f += 0.5;
        }
    }
}
