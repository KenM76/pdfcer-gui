//! # `dialogs::print::spooler::device` — what a printer IS, and how it is configured
//!
//! ## The seam this file is on the other side of
//!
//! [`super`] is the adapter for **the job**: which pages, at what size, in
//! what order, placed where on a sheet. This file is the adapter for **the
//! device**: which printers exist, what each one can do, which sheets it
//! offers, and what its driver currently holds.
//!
//! The two change for different reasons, which is the test R2 asks for when
//! a file comes due for splitting. A change to how a job is *laid out* — a
//! new scale mode, an imposition tab, a different resolution ceiling — never
//! touches this file. A change to how a device is *interrogated* — a paper
//! list, a properties dialog, a tray capability — never touches the
//! placement arithmetic next door. They were one file until 2026-08-18, and
//! the split happened because the paper work would have carried the total
//! past the 1,500-line limit; the seam was already there.
//!
//! ## What is still true of both halves
//!
//! **This module and its parent are the only files in the crate that name
//! `pdfcer_print`.** Everything else in [`crate::dialogs::print`] — the three
//! tabs, the preview, the footer — is written against the mirrored types
//! here. That is what confined "make printing work" to one module in
//! August 2026, and it is worth keeping: see [`super`]'s header for the full
//! reasoning, including why no arithmetic is ever mirrored.
//!
//! ## ★ The rule that governs every capability query here
//!
//! **A query that answers "I do not know" is not a query that answered
//! "no".** `pdfcer-print` was explicit about this when it declined this
//! project's proposal to gate the tray control on a `bool`:
//!
//! > *"`DC_BINS` on Microsoft Print to PDF returns nothing at all, while
//! > that same device's `dmDefaultSource` is already `DMBIN_FORMSOURCE` — it
//! > picks by form by default. A bool would have collapsed 'the driver said
//! > nothing' into 'no', and told the operator a device cannot do the thing
//! > it was already doing."*
//!
//! So [`FormSourceSupport`] has three states and not two, and the shell's
//! reading of them is the inverse of R83's usual direction: **`NotListed`
//! and `Unknown` still get the control**, with the disclosure. R83 forbids
//! offering an affordance the hardware *cannot* honour; it does not forbid
//! offering one the driver merely declined to advertise.
//!
//! Contrast [`DeviceFeatures::supports_duplex`], which is a genuine
//! capability answer and *is* gated: `DC_DUPLEX` returning zero means the
//! device is simplex, and no setting in the dialog will change that.

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
///
/// Maps to `pdfcer_print::DeviceFeatures`. Read **once**, when the dialog
/// opens: asking a driver this question sixty times a second while a dialog
/// sits open would be rude to a service other applications share.
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
    /// ★ Read the three states before writing a gate against this. Unlike
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
///
/// Called **once**, when the dialog opens — enumerating printers touches the
/// spooler, and doing it per frame while a dialog sits open would be rude to
/// a service other applications share. [`super::PrintDialog::new`] is the
/// only caller and it stores the result.
///
/// # Errors
///
/// [`Unavailable::Spooler`] when the spooler could not be queried at all,
/// which on a non-Windows target is always (`PrintError::Unsupported`).
///
/// **An empty `Vec` is `Ok`, not an error.** A machine with no printers
/// installed is a normal machine; see [`Unavailable`]'s own documentation for
/// why the type has nowhere to put that case.
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
///
/// Consulted **before** offering the duplex control at all (R83), never
/// after. [`crate::dialogs::print::PrintDialog::refresh_device`] calls it once per change
/// of the selected printer — which is the fix for a defect the old shell
/// still carries: it read features only for the *initially* selected device
/// and never again, so switching printers left the duplex control gated on
/// the previous one's capabilities.
///
/// # Errors
///
/// [`Unavailable::Spooler`] when the driver would not answer. The caller
/// falls back to [`DeviceFeatures::default`] — `supports_duplex: false` —
/// which is the safe direction: a device that cannot describe itself gets no
/// duplex control, rather than a control that may silently do nothing.
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
///
/// # Why the id and the name are both carried
///
/// [`Self::id`] is what a job is addressed with — `dmPaperSize`, an integer
/// the driver defined — and [`Self::name`] is what the operator recognises.
/// Neither substitutes for the other: two drivers can use different names for
/// the same standard id (`"A4"` and `"A4 210 x 297 mm"`), and a *vendor*
/// driver can use the same name for different ids across models. The combo
/// shows the name and sends the id, which is the only pairing that survives
/// both.
///
/// # Why the size is carried as well as the name
///
/// Because a driver's name for a roll or a custom form is frequently not a
/// size at all — `"Roll Paper 24in"`, `"User Defined"`, `"Custom"` — and the
/// operator choosing between two of those needs the dimensions.
///
/// **This is the PHYSICAL sheet, not the printable area.** The engine makes
/// the same distinction on `PrinterCaps` and for the same reason: fitting a
/// page to the physical size produces a page whose edges the hardware crops.
/// Nothing in this shell plans against this value — planning reads the
/// geometry [`super::plan`] gets back from `printer_caps_for`, which is the
/// printable area for *this* sheet. This one is for the label only.
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
///
/// Maps to `pdfcer_print::FormSourceSupport`. **Three states, and the third is
/// the whole point** — see this module's header for the measurement that
/// killed the `bool` version of this field.
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
///
/// # ★ What this actually is, and why the shell must not look inside
///
/// A Windows `DEVMODE`: a public header pdfcer understands, followed by a
/// **driver-private tail** in a format only that one driver knows. The
/// engine measured the tail on this machine — 5,208 bytes for Microsoft
/// Print to PDF, 7,972 for both EPSONs, 920 for the XPS writer. On the
/// EPSONs *97 % of a real `DEVMODE` is data pdfcer cannot interpret*, and it
/// carries media type, print quality, colour handling, stapling, output bin
/// and everything else the vendor's own dialog offers.
///
/// So this type has no fields the shell reads and no way to construct one:
/// it comes from the driver, it goes back to the driver, and the only thing
/// the shell may know about it is [`Self::summary`]. A shell that unpacked it
/// would be re-implementing a format it does not have.
///
/// # Why it exists at all rather than the dialog just holding the bytes
///
/// Because a `DEVMODE` belongs to **one device**. Handing one driver's
/// configuration to another is not a degraded result, it is an undefined one,
/// and the engine refuses it by name (`PrintError::Configuration`). Wrapping
/// it keeps that fact visible at the seam, and the dialog clears the field
/// with the rest of its per-device cache whenever the selection changes.
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
    ///
    /// `pub(super)` and not `pub(crate)`: [`super::plan`] and [`super::spool`]
    /// are the only callers, and widening this would let a `pdfcer_print` type
    /// escape into a third file — which is the property this module exists to
    /// hold.
    pub(super) const fn engine(&self) -> &pdfcer_print::PrinterConfiguration {
        &self.inner
    }
}

/// The readable part of a [`DriverConfig`].
///
/// Maps to the three fields of `pdfcer_print::ConfigurationSummary` this shell
/// has a use for. The engine's version carries five more — orientation,
/// duplex, tray, form name, device name — and they are deliberately **not**
/// mirrored: a field nothing reads is a field that can quietly acquire the
/// wrong units or stop being filled, and the dialog's own controls are
/// authoritative for every one of them (see [`super::to_engine_settings`] on
/// which members pdfcer asserts over whatever the configuration held).
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
///
/// Called on a change of the selected printer, alongside [`device_features`],
/// and stored. The list is what the paper combo is drawn from; an empty list
/// or a refusal leaves the combo showing only "from the printer's own
/// settings", which is honest — pdfcer cannot name a sheet the driver would
/// not enumerate.
///
/// # Errors
///
/// [`Unavailable::Spooler`] when the driver would not answer. The caller
/// falls back to an empty list.
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
///
/// # Why there is no silent "read the current settings" call beside this
///
/// There was one, briefly, on the theory that the FIRST press of this button
/// should resume from the device's current configuration. It should not, and
/// it already does: `DocumentProperties` with `DM_IN_PROMPT` and no input
/// buffer starts the dialog from the printer's own settings, which is
/// precisely what an operator expects the first time they open it. Passing a
/// separately-fetched copy would have been the same value by a longer route.
///
/// `start_from` earns its place on the SECOND press: it resumes from what the
/// first press produced, so an operator reopening the dialog to change one
/// thing does not silently lose the rest.
///
/// # ★ `Ok(None)` is Cancel, and it is not a failure
///
/// The engine is explicit: *"that is the operator declining, and a shell that
/// showed an error for it would be scolding them for using the dialog
/// correctly."* The caller keeps whatever configuration it already had and
/// says nothing.
///
/// # The parent handle
///
/// `parent` is this application's own top-level window, as a raw `HWND` cast
/// to `isize`. Passing `None` is legal and produces an **unowned** modal
/// dialog, which can fall behind the main window — a modal the operator
/// cannot see and cannot dismiss, with the application apparently frozen
/// behind it. So the shell passes its handle.
///
/// # ★ This call BLOCKS the frame, for as long as the operator takes
///
/// It is a nested modal message loop belonging to the driver, run from inside
/// our own event loop. egui stops painting until it returns. That is
/// acceptable here and would not be for anything on the canvas: it is one
/// button, pressed deliberately, whose entire purpose is a window the
/// operator is about to interact with, and the alternative — running it on
/// another thread — hands a foreign modal a parent it does not own.
///
/// # Errors
///
/// [`Unavailable::Device`], for the same reason as
/// [`printer_configuration`].
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
