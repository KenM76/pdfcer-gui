//! # `dialogs::print::spooler::device` — what a printer IS, and how it is configured
//!
//! ## The seam this file is on the other side of
//!
//! [`super`] is the adapter for **the job**: which pages, at what size, in
//! what order, placed where on a sheet. This file is the adapter for **the
//! device**: which printers exist, what each one can do, which sheets it
//! offers, and what its driver currently holds.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/print/spooler/device.md`.

use super::Unavailable;

// ---------------------------------------------------------------------------
// The device, and what it says about itself
// ---------------------------------------------------------------------------

/// One printer the system knows about. Maps to `pdfcer_print::Printer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Printer {
    /// The name the spooler reports, and the one a job is addressed to.
    pub(crate) name: String,
    /// The driver's name.
    ///
    /// Carried because two printers can share a human-readable name closely
    /// enough that an operator cannot tell them apart, and the driver usually
    /// distinguishes them. Traced rather than shown today: the selector is a
    /// combo of names, and a two-line row is a change to make on evidence
    /// that the ambiguity actually bites.
    pub(crate) driver: String,
    /// The port, for the same reason as [`Self::driver`].
    pub(crate) port: String,
    /// Whether this is the system default — the dialog's initial selection.
    pub(crate) is_default: bool,
}

/// What a device says it can do, beyond geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DeviceFeatures {
    /// The driver reports duplex support. The dialog draws no duplex control
    /// without it (R83).
    pub(crate) supports_duplex: bool,
    /// How many copies the driver can produce itself.
    ///
    /// **Reported, not used.** pdfcer sends its own sequence today, so this is
    /// carried to the trace so a later decision about hardware collation can
    /// be made on evidence rather than on assumption.
    pub(crate) max_copies: u16,
    /// Whether the driver advertises tray-selection-by-sheet-size.
    ///
    /// Read the three states before writing a gate against this. Unlike
    /// [`Self::supports_duplex`] it is **not** a capability answer, and the
    /// control it governs is drawn in all three states. See
    /// [`FormSourceSupport`] and this module's header.
    pub(crate) form_source: FormSourceSupport,
}
// ---------------------------------------------------------------------------
// The queries into the engine
// ---------------------------------------------------------------------------
//
// Each one is called on a CHANGE — the dialog opening, or the selected
// printer changing — and never per frame. Asking a driver these questions
// sixty times a second while a dialog sits open would be rude to a service
// other applications share, and two of them (`printer_configuration`,
// `printer_forms`) open a device context to do it.

/// Enumerate the system's printers.
pub(crate) fn list_printers() -> Result<Vec<Printer>, Unavailable> {
    match pdfcer_print::list_printers() {
        Ok(found) => Ok(found
            .into_iter()
            .map(|printer| Printer {
                name: printer.name,
                driver: printer.driver,
                port: printer.port,
                is_default: printer.is_default,
            })
            .collect()),
        Err(error) => Err(Unavailable::Spooler(error.to_string())),
    }
}

/// Read one device's non-geometric capabilities.
pub(crate) fn device_features(printer: &str) -> Result<DeviceFeatures, Unavailable> {
    match pdfcer_print::device_features(printer) {
        Ok(features) => Ok(DeviceFeatures {
            supports_duplex: features.supports_duplex,
            max_copies: features.max_copies,
            form_source: match features.form_source_bin {
                pdfcer_print::FormSourceSupport::Listed => FormSourceSupport::Listed,
                pdfcer_print::FormSourceSupport::NotListed => FormSourceSupport::NotListed,
                pdfcer_print::FormSourceSupport::Unknown => FormSourceSupport::Unknown,
            },
        }),
        Err(error) => Err(Unavailable::Spooler(error.to_string())),
    }
}

/// One sheet size the driver offers. Maps to `pdfcer_print::PaperForm`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PaperForm {
    /// The `dmPaperSize` value that selects this form.
    pub(crate) id: u16,
    /// The driver's own name for it. Operator-facing; not stable across
    /// drivers, which is why it is never used as an identity.
    pub(crate) name: String,
    /// The physical sheet in points.
    pub(crate) size_pt: (f64, f64),
}

/// Whether the driver advertises "choose the tray from the sheet size".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FormSourceSupport {
    /// `DC_BINS` includes `DMBIN_FORMSOURCE`. Offer the control plainly.
    Listed,
    /// `DC_BINS` answered and did not include it. **Not a refusal.** Offer
    /// the control with the disclosure.
    NotListed,
    /// `DC_BINS` did not answer — nothing was learned either way. Same
    /// treatment as [`Self::NotListed`], different sentence.
    #[default]
    Unknown,
}

/// A driver's own settings, carried opaquely.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DriverConfig {
    /// The engine's own value. Private: see the type's docs.
    inner: pdfcer_print::PrinterConfiguration,
}

impl DriverConfig {
    /// What this configuration asks for, as far as the dialog needs to know.
    pub(crate) fn summary(&self) -> ConfigSummary {
        let summary = self.inner.summary();
        ConfigSummary {
            paper_form_id: summary.paper_form_id,
            custom_paper_pt: summary.custom_paper_pt,
            driver_extra: summary.driver_extra,
        }
    }

    /// The engine's value, for the two calls that take one.
    pub(super) const fn engine(&self) -> &pdfcer_print::PrinterConfiguration {
        &self.inner
    }
}

/// The readable part of a [`DriverConfig`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ConfigSummary {
    /// `dmPaperSize`, when the configuration asserts one.
    ///
    /// Read after the driver's own properties dialog closes, so the paper
    /// combo can follow a sheet the operator chose *in that dialog* rather
    /// than sitting on "from the printer's own settings" while the driver
    /// holds A3. The two would otherwise describe the same job differently.
    pub(crate) paper_form_id: Option<u16>,
    /// `dmPaperWidth`/`dmPaperLength` in points, when the configuration names
    /// a sheet by size rather than by form.
    ///
    /// Not selectable from this shell — there is no size-entry surface — but
    /// reachable *through* the driver's dialog, which is why it is read: an
    /// operator who typed a custom size there gets it disclosed rather than
    /// silently reported as "from the printer's own settings".
    pub(crate) custom_paper_pt: Option<(f64, f64)>,
    /// Bytes of driver-private data being carried through untouched.
    ///
    /// **Traced, not shown.** It is the evidence that a configuration is
    /// doing something — a properties dialog that returned 7,972 bytes of
    /// tail carried settings pdfcer cannot name — and it is meaningless as
    /// operator copy.
    pub(crate) driver_extra: usize,
}

/// Every sheet size this device offers.
pub(crate) fn printer_forms(printer: &str) -> Result<Vec<PaperForm>, Unavailable> {
    match pdfcer_print::printer_forms(printer) {
        Ok(forms) => Ok(forms
            .into_iter()
            .map(|form| PaperForm {
                id: form.id,
                name: form.name,
                size_pt: form.size_pt,
            })
            .collect()),
        Err(error) => Err(Unavailable::Spooler(error.to_string())),
    }
}

/// Open the driver's **own** properties dialog.
pub(crate) fn edit_printer_configuration(
    printer: &str,
    parent: Option<isize>,
    start_from: Option<&DriverConfig>,
) -> Result<Option<DriverConfig>, Unavailable> {
    match pdfcer_print::edit_printer_configuration(
        printer,
        parent,
        start_from.map(DriverConfig::engine),
    ) {
        Ok(edited) => Ok(edited.map(|inner| DriverConfig { inner })),
        Err(error) => Err(Unavailable::Device(error.to_string())),
    }
}
