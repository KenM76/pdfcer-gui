//! # `dialogs::print::finishing` — staple, punch and output tray on Copies & finishing
//!
//! Each control is drawn only when the selected printer's driver offers at
//! least one option for it (R9); a printer offering none shows nothing. The
//! choices reset when the printer changes, since one driver's bin names mean
//! nothing to another. At commit [`Choice::config`] has the driver merge the
//! request into the job's settings; a refusal stops the job with the
//! driver's reason in the dialog.

use egui::Ui;
use pdfcer_gui_base::printspooler::{self as spooler, DriverConfig, finishing as fin};
use pdfcer_gui_base::text::print::finishing as t;

/// Region of the staple combo, for driven checks.
const REGION_STAPLE: &str = "print.finishing.staple"; // ui-text-exempt: trace region name
/// Region of the punch combo.
const REGION_PUNCH: &str = "print.finishing.punch"; // ui-text-exempt: trace region name
/// Region of the output-tray combo.
const REGION_BIN: &str = "print.finishing.bin"; // ui-text-exempt: trace region name

/// The offer cache and what the operator chose from it.
#[derive(Debug, Default)]
pub(super) struct Choice {
    cache: fin::OfferCache,
    staple: Option<fin::Staple>,
    punch: Option<fin::Edge>,
    bin: Option<String>,
}

impl Choice {
    /// Forget the choices: they named the previous printer's options.
    pub(super) fn printer_changed(&mut self) {
        self.staple = None;
        self.punch = None;
        self.bin = None;
    }

    /// The driver settings to spool with on `printer`.
    ///
    /// # Errors
    ///
    /// The driver's refusal, worded by the engine.
    pub(super) fn config(
        &self,
        printer: &str,
        config: Option<&DriverConfig>,
    ) -> Result<Option<DriverConfig>, String> {
        let request = fin::request(self.staple, self.punch, self.bin.as_deref());
        fin::with_finishing(printer, config, &request)
    }

    /// The controls the driver's offer allows, under the tab's other settings.
    pub(super) fn controls(&mut self, ui: &mut Ui, printer: Option<&str>) {
        let Some(printer) = printer else { return };
        let spooler::finishing::OfferState::Ready(offer) = self.cache.get(printer) else {
            return;
        };
        if offer.is_empty() {
            return;
        }
        if !offer.staples.is_empty() {
            let r = pick(
                ui,
                t::staple_label(),
                "print-staple",
                &mut self.staple,
                &offer.staples,
                t::staple,
            );
            crate::diag::ui_rect(REGION_STAPLE, r);
        }
        if !offer.punch.is_empty() {
            let r = pick(
                ui,
                t::punch_label(),
                "print-punch",
                &mut self.punch,
                &offer.punch,
                t::edge,
            );
            crate::diag::ui_rect(REGION_PUNCH, r);
        }
        if !offer.bins.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.label(t::bin_label());
                let shown = self
                    .bin
                    .clone()
                    .unwrap_or_else(|| t::bin_default().to_owned());
                let r = egui::ComboBox::from_id_salt("print-bin")
                    .selected_text(shown)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.bin, None, t::bin_default());
                        for bin in &offer.bins {
                            ui.selectable_value(&mut self.bin, Some(bin.clone()), bin);
                        }
                    })
                    .response
                    .rect;
                crate::diag::ui_rect(REGION_BIN, r);
            });
        }
        ui.label(egui::RichText::new(t::finishing_note()).small().weak());
        ui.add_space(8.0);
    }
}

/// One labelled combo over `options`, with "None" first.
pub(crate) fn pick<T: Copy + PartialEq>(
    ui: &mut Ui,
    label: &str,
    salt: &str,
    value: &mut Option<T>,
    options: &[T],
    name: fn(T) -> &'static str,
) -> egui::Rect {
    ui.horizontal_wrapped(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(salt)
            .selected_text(value.map_or(t::none(), name))
            .show_ui(ui, |ui| {
                ui.selectable_value(value, None, t::none());
                for &o in options {
                    ui.selectable_value(value, Some(o), name(o));
                }
            })
            .response
            .rect
    })
    .inner
}
