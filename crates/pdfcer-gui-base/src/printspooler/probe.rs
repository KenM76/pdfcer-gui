//! # `printspooler::probe` — every printer read off the UI thread
//!
//! An installed printer that cannot be reached makes the spooler take about
//! 45 to 50 s to answer any per-device query, so no such query runs on the UI
//! thread. Two caches stand between the dialog and the spooler:
//!
//! - **The probe**, process-wide: [`start`] lists printers with the
//!   engine's `list_printer_names` (registry only, returns at once) and
//!   `pdfcer_print::PrinterProbe` reads every printer's detail, features and
//!   forms on its own thread. Started when the application opens, so the
//!   answers are usually in before the Print dialog is.
//! - **[`CapsCache`]**, one per dialog: `printer_caps_for` for a (printer,
//!   driver configuration, paper) triple, read on a worker.
//!
//! Every reader returns "not yet" rather than waiting; the dialog repaints
//! while anything it shows is outstanding.
//!
//! `PDFCER_DIAG_SLOW_PRINTER=<name>=<seconds>` (tracing on only) makes every
//! read of that printer sleep first, the stand-in for an unreachable device.
//! The name `<default>` means whichever printer the spooler calls the default.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use super::{DeviceFeatures, DriverConfig, PaperChoice, PaperForm, Printer, Unavailable};

static PROBE: Mutex<Option<pdfcer_print::PrinterProbe>> = Mutex::new(None);

/// The spooler's default printer at the moment the probe started.
static DEFAULT: OnceLock<String> = OnceLock::new();

/// The diagnostic spelling of "the default printer, whatever it is called".
const DEFAULT_TOKEN: &str = "<default>"; // ui-text-exempt: diagnostic token, never displayed

/// The diagnostic delay for `printer`, if one was asked for.
fn slow(printer: &str) -> Option<Duration> {
    if !crate::diag::enabled() {
        return None;
    }
    // ui-text-exempt: environment variable name, never displayed
    let spec = std::env::var("PDFCER_DIAG_SLOW_PRINTER").ok()?;
    let (name, secs) = spec.rsplit_once('=')?;
    let named =
        name == printer || (name == DEFAULT_TOKEN && DEFAULT.get().is_some_and(|d| d == printer));
    named.then(|| Duration::from_secs_f64(secs.trim().parse().unwrap_or(0.0)))
}

fn sleep_if_slow(printer: &str) {
    if let Some(d) = slow(printer) {
        std::thread::sleep(d);
    }
}

fn probe() -> MutexGuard<'static, Option<pdfcer_print::PrinterProbe>> {
    PROBE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Start reading every printer in the background. Idempotent; a failure to
/// list leaves nothing started, and [`printers`] reports it.
pub fn start() {
    let mut slot = probe();
    if slot.is_some() {
        return;
    }
    let Ok(names) = pdfcer_print::list_printer_names() else {
        return;
    };
    if let Some(d) = names.iter().find(|p| p.is_default) {
        drop(DEFAULT.set(d.name.clone()));
    }
    let reader: pdfcer_print::ReportReader = Arc::new(|name: &str| {
        sleep_if_slow(name);
        pdfcer_print::PrinterReport::read(name)
    });
    crate::diag::trace(|| format!("printer-probe-start printers={}", names.len()));
    *slot = Some(pdfcer_print::PrinterProbe::start_with(names, reader));
}

/// Every installed printer, without contacting any. Driver and port are
/// filled once that printer's read has answered.
///
/// # Errors
///
/// [`Unavailable::Spooler`] when the spooler would not list its printers.
pub fn printers() -> Result<Vec<Printer>, Unavailable> {
    start();
    let mut slot = probe();
    let Some(probe) = slot.as_mut() else {
        return pdfcer_print::list_printer_names()
            .map(|_| Vec::new())
            .map_err(|e| Unavailable::Spooler(e.to_string()));
    };
    probe.poll();
    Ok(probe
        .printers()
        .iter()
        .map(|p| {
            let detail = probe.report(&p.name).and_then(|r| r.detail.as_ref().ok());
            Printer {
                name: p.name.clone(),
                driver: detail.map(|d| d.driver.clone()).unwrap_or_default(),
                port: detail.map(|d| d.port.clone()).unwrap_or_default(),
                is_default: p.is_default,
            }
        })
        .collect())
}

/// What one printer said about itself. A failed read falls back to the
/// defaults, the same answer an unanswered query used to give.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeviceReport {
    pub features: DeviceFeatures,
    pub forms: Vec<PaperForm>,
}

/// `printer`'s report, or `None` while its read is outstanding.
pub fn report(printer: &str) -> Option<DeviceReport> {
    let mut slot = probe();
    let probe = slot.as_mut()?;
    probe.poll();
    let r = probe.report(printer)?;
    Some(DeviceReport {
        features: r
            .features
            .as_ref()
            .map(super::device::features_from_engine)
            .unwrap_or_default(),
        forms: r
            .forms
            .as_ref()
            .map(|forms| forms.iter().map(super::device::form_from_engine).collect())
            .unwrap_or_default(),
    })
}

/// Read `printer` again, keeping its old report until the new one lands.
pub fn refresh(printer: &str) {
    if let Some(probe) = probe().as_ref() {
        probe.refresh(printer);
    }
}

/// The geometry a job will be planned against.
pub type Caps = pdfcer_print::PrinterCaps;

#[derive(Clone, PartialEq)]
struct CapsKey {
    printer: String,
    config: Option<DriverConfig>,
    paper: PaperChoice,
}

/// What [`CapsCache::get`] knows.
#[derive(Debug, Clone, PartialEq)]
pub enum CapsState {
    Pending,
    Ready(Caps),
    Failed(Unavailable),
}

/// `printer_caps_for`, answered on a worker and remembered for the life of
/// one dialog. A late answer for a printer no longer selected is kept.
pub struct CapsCache {
    known: Vec<(CapsKey, CapsState)>,
    tx: Sender<(CapsKey, CapsState)>,
    rx: Receiver<(CapsKey, CapsState)>,
}

impl std::fmt::Debug for CapsCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapsCache")
            .field("known", &self.known.len())
            .finish_non_exhaustive()
    }
}

impl Default for CapsCache {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            known: Vec::new(),
            tx,
            rx,
        }
    }
}

impl CapsCache {
    /// The caps for this triple, starting the read on first ask.
    pub fn get(
        &mut self,
        printer: &str,
        config: Option<&DriverConfig>,
        paper: PaperChoice,
    ) -> CapsState {
        while let Ok((key, state)) = self.rx.try_recv() {
            if let Some(slot) = self.known.iter_mut().find(|(k, _)| *k == key) {
                slot.1 = state;
            }
        }
        let key = CapsKey {
            printer: printer.to_owned(),
            config: config.cloned(),
            paper,
        };
        if let Some((_, state)) = self.known.iter().find(|(k, _)| *k == key) {
            return state.clone();
        }
        self.known.push((key.clone(), CapsState::Pending));
        let tx = self.tx.clone();
        let spawned = std::thread::Builder::new()
            .name(format!("printer-caps {printer}")) // ui-text-exempt: thread name, never displayed
            .spawn(move || {
                sleep_if_slow(&key.printer);
                let state = match super::read_caps(&key.printer, key.config.as_ref(), key.paper) {
                    Ok(caps) => CapsState::Ready(caps),
                    Err(e) => CapsState::Failed(e),
                };
                drop(tx.send((key, state)));
            });
        if let Err(e) = spawned {
            let failed = CapsState::Failed(Unavailable::Device(e.to_string()));
            if let Some(last) = self.known.last_mut() {
                last.1 = failed.clone();
            }
            return failed;
        }
        CapsState::Pending
    }
}
