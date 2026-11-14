//! # `dialogs::print::direct` — a plain job sent to the printer as the PDF
//!
//! With *Send the PDF itself when the printer reads it* on, the selected
//! queue's Windows port is resolved to IPP addresses
//! (`pdfcer_print::queue_port`, `pdfcer_print::ipp::resolve_queue_port`) and
//! each is asked Get-Printer-Attributes on a worker; the first that reads
//! `application/pdf` is kept. Print then sends the print-ready copy
//! (`OpenDoc::print_ready_copy`) with Print-Job instead of spooling sheets.
//!
//! Only a job the printer can lay out from IPP attributes alone goes this
//! way: copies, collation, sides, the page subset as `page-ranges`,
//! grey as `print-color-mode`, Fit or Actual size as `print-scaling`, a
//! forced orientation. Anything pdfcer composes itself ([`PrintDialog::
//! direct_blocker`]) prints through the driver and the dialog names why.
//! Nothing touches the network while the switch is off.
//!
//! [`DIAG_URI`] replaces the port resolution with one address, for
//! `ui-verify`'s local printer; under the spooler's dry run nothing is sent
//! unless it is set.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};

use pdfcer_gui_base::printspooler::network::{self as net, NetJob, NetPrinter};
use pdfcer_gui_base::printspooler::{
    DIAG_DRY_RUN, DirectSent, Duplex, Orientation, PaperChoice, ScaleMode, SettingsSource,
    SpoolReport,
};
use pdfcer_print::ipp::Sides;
use pdfcer_print::output_color::OutputColor;

use super::PrintDialog;
use crate::app::state::OpenDoc;
use crate::dialogs::print::spooler::Job;
use crate::text::print::direct as t;

/// Ask this address instead of resolving the queue's port, for `ui-verify`.
pub const DIAG_URI: &str = "PDFCER_DIAG_PRINT_DIRECT_URI"; // ui-text-exempt: an environment variable name, never displayed

/// A printer that reads PDF.
#[derive(Debug, Clone)]
pub(super) struct Found {
    uri: String,
    /// Where the address came from, in plain words.
    from: &'static str,
    printer: NetPrinter,
}

#[derive(Default)]
struct Probe {
    /// The queue the answer or the question is about.
    queue: Option<String>,
    pending: Option<Receiver<Result<Found, String>>>,
    answer: Option<Result<Found, String>>,
}

/// The printer question, for the selected queue. Behind a lock so the
/// commit, which holds the dialog shared, can wait for an answer in flight.
pub(super) struct Direct {
    probe: Mutex<Probe>,
    /// Whether any page carries an annotation that prints (anything but a
    /// link). A printer handed the PDF prints those by their own Print flag,
    /// which is the default scope, so only then does another scope matter.
    annotated: bool,
}

impl Direct {
    pub(super) fn new(doc: &OpenDoc) -> Self {
        let graph = doc.session.graph();
        let annotated = doc.pages.iter().any(|page| {
            pdfcer_core::annot::page_annotations(&graph, page.id)
                .iter()
                .any(|a| a.subtype != b"Link")
        });
        Self {
            probe: Mutex::new(Probe::default()),
            annotated,
        }
    }

    /// Ask `queue`'s printer when `on` and it has not been asked; take an
    /// answer that has arrived. Off, or no queue, forgets everything.
    pub(super) fn poll(&self, on: bool, queue: Option<&str>, ctx: &egui::Context) {
        let Ok(mut probe) = self.probe.lock() else {
            return;
        };
        let Some(queue) = queue.filter(|_| on) else {
            *probe = Probe::default();
            return;
        };
        if probe.queue.as_deref() != Some(queue) {
            *probe = Probe {
                queue: Some(queue.to_owned()),
                pending: Some(ask(queue)),
                answer: None,
            };
        }
        if let Some(rx) = &probe.pending {
            match rx.try_recv() {
                Ok(answer) => {
                    trace_answer(queue, &answer);
                    probe.answer = Some(answer);
                    probe.pending = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(200));
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => probe.pending = None,
            }
        }
    }

    /// The answer, waiting for one in flight. `None` when nothing was asked.
    fn answer(&self) -> Option<Result<Found, String>> {
        let mut probe = self.probe.lock().ok()?;
        if let Some(rx) = probe.pending.take() {
            let answer = rx.recv().unwrap_or_else(|e| Err(e.to_string()));
            if let Some(queue) = probe.queue.clone() {
                trace_answer(&queue, &answer);
            }
            probe.answer = Some(answer);
        }
        probe.answer.clone()
    }

    /// The answer so far, without waiting: `None` while asking.
    fn shown(&self) -> Option<Result<Found, String>> {
        self.probe.lock().ok()?.answer.clone()
    }
}

/// Ask `queue`'s printer on a worker.
fn ask(queue: &str) -> Receiver<Result<Found, String>> {
    let (tx, rx) = channel();
    let queue = queue.to_owned();
    let spawned = std::thread::Builder::new()
        .name("print-direct-probe".to_owned()) // ui-text-exempt: thread name
        .spawn(move || {
            let _ = tx.send(find(&queue));
        });
    if let Err(e) = spawned {
        let (tx, failed) = channel();
        let _ = tx.send(Err(e.to_string()));
        return failed;
    }
    rx
}

/// The first of `queue`'s addresses whose printer reads PDF.
fn find(queue: &str) -> Result<Found, String> {
    let (uris, from) = match std::env::var(DIAG_URI) {
        Ok(uri) => (vec![uri], "the diagnostic address"), // ui-text-exempt: harness-only source
        Err(_) => {
            let port = pdfcer_print::queue_port(queue).map_err(|e| e.to_string())?;
            let found = pdfcer_print::ipp::resolve_queue_port(&port).map_err(|e| e.to_string())?;
            (found.uris, found.source.describe())
        }
    };
    let mut last = t::no_pdf().to_owned();
    for uri in uris {
        match net::query(&uri) {
            Ok(printer) if printer.accepts_pdf() => {
                return Ok(Found { uri, from, printer });
            }
            Ok(_) => last = t::no_pdf().to_owned(),
            Err(why) => last = why,
        }
    }
    Err(last)
}

fn trace_answer(queue: &str, answer: &Result<Found, String>) {
    crate::diag::trace(|| match answer {
        // ui-text-exempt: diagnostic trace, never displayed
        Ok(f) => format!(
            "print-direct-found queue={queue:?} uri={} from={:?}",
            f.uri, f.from
        ),
        // ui-text-exempt: diagnostic trace, never displayed
        Err(why) => format!("print-direct-found queue={queue:?} uri=none why={why:?}"),
    });
}

impl PrintDialog {
    /// What in this job only pdfcer's own layout can print, or `None` when
    /// the printer can be handed the PDF.
    pub(super) fn direct_blocker(&self) -> Option<&'static str> {
        use pdfcer_gui_base::prefs::print_impose::ImposeMode;
        use t::needs;
        let blockers = [
            (self.print_as_image, needs::image()),
            (self.separations, needs::separations()),
            (
                self.output_color == OutputColor::BlackOnly,
                needs::black_only(),
            ),
            (self.icc.managed(), needs::profile()),
            (self.impose.prefs.mode != ImposeMode::Off, needs::impose()),
            (self.poster.on, needs::poster()),
            (
                !matches!(self.scale, ScaleMode::Fit | ScaleMode::ActualSize),
                needs::scale(),
            ),
            (
                self.device.paper != PaperChoice::DeviceDefault,
                needs::paper(),
            ),
            (self.page_positions.any(), needs::moved()),
            (self.lines.fixed, needs::lines()),
            (self.running.copy().is_some(), needs::running()),
            (self.reverse, needs::reverse()),
            (
                self.direct.annotated && self.scope != pdfcer_render::AnnotationScope::default(),
                needs::scope(),
            ),
            (self.finishing.chosen(), needs::finishing()),
        ];
        blockers.into_iter().find(|(on, _)| *on).map(|(_, why)| why)
    }

    /// Send this job to the printer as the PDF, when the switch is on, the
    /// job allows it and the printer reads PDF; `None` to spool it instead.
    pub(super) fn commit_direct(
        &self,
        doc: &OpenDoc,
        job: &Job,
    ) -> Option<Result<SpoolReport, String>> {
        if !self.vector_route.pdf_direct {
            return None;
        }
        if let Some(why) = self.direct_blocker() {
            trace_route(&format!("driver why={why:?}")); // ui-text-exempt: diagnostic trace
            return None;
        }
        let found = match self.direct.answer()? {
            Ok(found) => found,
            Err(why) => {
                trace_route(&format!("driver why={why:?}")); // ui-text-exempt: diagnostic trace // ui-text-exempt: diagnostic trace
                return None;
            }
        };
        Some(self.send(doc, job, &found))
    }

    fn send(&self, doc: &OpenDoc, job: &Job, found: &Found) -> Result<SpoolReport, String> {
        let copy = doc.print_ready_copy().map_err(|e| e.to_string())?;
        let count = u32::try_from(doc.pages.len()).unwrap_or(u32::MAX);
        let indices: Vec<usize> = job.plans.iter().map(|p| p.index).collect();
        let request = NetJob {
            copies: self.copies,
            sides: Some(match self.device.duplex {
                Duplex::Simplex => Sides::OneSided,
                Duplex::LongEdge => Sides::TwoSidedLongEdge,
                Duplex::ShortEdge => Sides::TwoSidedShortEdge,
            }),
            ranges: ranges(&indices, count),
            monochrome: self.output_color == OutputColor::Grayscale,
            finishing: pdfcer_print::finishing::Finishing::default(),
            scaling: Some(if self.scale == ScaleMode::ActualSize {
                "none"
            } else {
                "fit"
            }),
            orientation: match self.device.orientation {
                Orientation::Auto => None,
                Orientation::Portrait => Some(3),
                Orientation::Landscape => Some(4),
            },
            uncollated: self.uncollated,
        };
        let dry =
            std::env::var(DIAG_DRY_RUN).is_ok_and(|v| v == "1") && std::env::var(DIAG_URI).is_err();
        trace_route(&format!(
            "direct uri={} bytes={} pages={} ranges={:?} copies={} dry_run={dry}", // ui-text-exempt: diagnostic trace
            found.uri,
            copy.bytes.len(),
            indices.len(),
            request.ranges,
            request.copies
        ));
        let receipt = if dry {
            net::NetReceipt {
                job_id: None,
                ignored: Vec::new(),
                bytes: copy.bytes.len(),
            }
        } else {
            net::send(
                &found.uri,
                &super::poster::document_name(doc),
                &copy.bytes,
                &request,
                &found.printer,
            )?
        };
        Ok(SpoolReport {
            pages: indices.len(),
            printed: !dry,
            dpi: (0, 0),
            clipped_pages: 0,
            job_id: None,
            settings_source: SettingsSource::DeviceDefault,
            vector_sheets: 0,
            fell_back: 0,
            image_reasons: Vec::new(),
            tally: crate::dialogs::print::spooler::VectorTally::default(),
            plates: crate::dialogs::print::spooler::PlateTally::default(),
            direct: Some(DirectSent {
                uri: found.uri.clone(),
                receipt,
                notes: copy.report.disclosures,
            }),
        })
    }
}

/// 0-based `indices` as 1-based inclusive runs; empty when they are every
/// page of `count` in order.
fn ranges(indices: &[usize], count: u32) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for &i in indices {
        let n = u32::try_from(i + 1).unwrap_or(u32::MAX);
        match out.last_mut() {
            Some((_, end)) if *end + 1 == n => *end = n,
            _ => out.push((n, n)),
        }
    }
    if out == [(1, count)] { Vec::new() } else { out }
}

fn trace_route(what: &str) {
    crate::diag::trace(|| format!("print-direct route={what}")); // ui-text-exempt: diagnostic trace
}

/// The switch and the line saying where this job will go.
pub(super) fn controls(ui: &mut egui::Ui, dialog: &mut PrintDialog) {
    let switch = ui
        .checkbox(&mut dialog.vector_route.pdf_direct, t::switch())
        .on_hover_text(t::hover());
    crate::diag::ui_rect_visible(REGION, switch.rect, ui.clip_rect());
    if !dialog.vector_route.pdf_direct {
        return;
    }
    let said = match (dialog.direct.shown(), dialog.direct_blocker()) {
        (None, _) => t::checking().to_owned(),
        (Some(Err(why)), _) => t::unavailable(&why),
        (Some(Ok(_)), Some(why)) => t::needs_layout(why),
        (Some(Ok(found)), None) => t::ready(&found.uri, found.from),
    };
    ui.label(
        egui::RichText::new(said)
            .small()
            .color(egui_shell::theme::Theme::of(ui.ctx()).palette.text_muted),
    );
}

/// The switch's region, for `ui-verify`.
pub const REGION: &str = "print.direct"; // ui-text-exempt: region id

#[cfg(test)]
mod tests {
    use super::ranges;

    #[test]
    fn every_page_in_order_is_no_range() {
        assert!(ranges(&[0, 1, 2], 3).is_empty());
    }

    #[test]
    fn a_subset_becomes_runs() {
        assert_eq!(ranges(&[0, 2, 3, 5], 6), vec![(1, 1), (3, 4), (6, 6)]);
        assert_eq!(ranges(&[0, 1], 3), vec![(1, 2)]);
    }
}
