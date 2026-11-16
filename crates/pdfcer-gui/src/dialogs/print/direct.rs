//! # `dialogs::print::direct` — a job sent to the printer as a PDF
//!
//! With *Send the PDF itself when the printer reads it* on, the selected
//! queue's Windows port is resolved to IPP addresses
//! (`pdfcer_print::queue_port`, `pdfcer_print::ipp::resolve_queue_port`) and
//! each is asked Get-Printer-Attributes on a worker; the first that reads
//! `application/pdf` is kept. Print then sends a PDF with Print-Job instead of
//! spooling sheets.
//!
//! Which PDF is [`compose::Layout`]'s decision: the print-ready copy
//! (`OpenDoc::print_ready_copy`) as it is, with copies, collation, sides,
//! `page-ranges`, grey, Fit / Actual size / Shrink as `print-scaling` and a
//! forced orientation as IPP attributes; or the sheets the engine composed
//! from it, sent at actual size with no orientation. What only pdfcer's own
//! drawing can print ([`PrintDialog::direct_blocker`]) goes through the
//! driver, and so does a printer that cannot be reached or does not read PDF,
//! and a layout the engine cannot compose. Each fallback is a
//! `pdfcer_print::pdf_direct::FallbackReason`, traced and told in the receipt.
//! Nothing touches the network while the switch is off.
//!
//! [`DIAG_URI`] replaces the port resolution with one address, for
//! `ui-verify`'s local printer; under the spooler's dry run nothing is sent
//! unless it is set.

mod compose;

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};

use pdfcer_gui_base::printspooler::network::{self as net, NetJob, NetPrinter};
use pdfcer_gui_base::printspooler::{
    DIAG_DRY_RUN, DirectSent, Duplex, Orientation, ScaleMode, SettingsSource, SpoolReport,
};
use pdfcer_print::ipp::Sides;
use pdfcer_print::output_color::OutputColor;
use pdfcer_print::pdf_direct::FallbackReason;

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

/// Why the printer cannot be handed a PDF, and the particulars.
type Refused = (FallbackReason, String);

#[derive(Default)]
struct Probe {
    /// The queue the answer or the question is about.
    queue: Option<String>,
    pending: Option<Receiver<Result<Found, Refused>>>,
    answer: Option<Result<Found, Refused>>,
}

/// The printer question, for the selected queue. Behind a lock so the
/// commit, which holds the dialog shared, can wait for an answer in flight.
pub(super) struct Direct {
    probe: Mutex<Probe>,
    /// Whether any page carries an annotation that prints (anything but a
    /// link). A printer handed the PDF prints those by their own Print flag,
    /// which is the default scope, so only then does another scope matter.
    annotated: bool,
    /// The sheet this job would be composed for, points, portrait; `None`
    /// when the document's own pages would be sent.
    composed_for: Option<(f64, f64)>,
    /// The last commit's fallback sentence, for its receipt.
    fallback: Mutex<Option<String>>,
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
            composed_for: None,
            fallback: Mutex::new(None),
        }
    }

    /// Ask `queue`'s printer when `on` and it has not been asked; take an
    /// answer that has arrived. Off, or no queue, forgets everything.
    /// `composed_for` is this frame's [`PrintDialog::direct_composed_for`].
    pub(super) fn poll(
        &mut self,
        on: bool,
        queue: Option<&str>,
        composed_for: Option<(f64, f64)>,
        ctx: &egui::Context,
    ) {
        self.composed_for = composed_for;
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
    fn answer(&self) -> Option<Result<Found, Refused>> {
        let mut probe = self.probe.lock().ok()?;
        if let Some(rx) = probe.pending.take() {
            let answer = rx
                .recv()
                .unwrap_or_else(|e| Err((FallbackReason::Unreachable, e.to_string())));
            if let Some(queue) = probe.queue.clone() {
                trace_answer(&queue, &answer);
            }
            probe.answer = Some(answer);
        }
        probe.answer.clone()
    }

    /// The answer so far, without waiting: `None` while asking.
    fn shown(&self) -> Option<Result<Found, Refused>> {
        self.probe.lock().ok()?.answer.clone()
    }

    /// The last commit's fallback sentence, once.
    pub(super) fn take_fallback(&self) -> Option<String> {
        self.fallback.lock().ok()?.take()
    }

    /// Record that this commit goes through the driver, and why.
    fn fall_back(&self, reason: FallbackReason, why: &str) {
        trace_route(&format!("driver fallback={} why={why:?}", reason.keyword())); // ui-text-exempt: diagnostic trace
        if let Ok(mut slot) = self.fallback.lock() {
            *slot = Some(t::fell_back(reason, why));
        }
    }
}

/// Ask `queue`'s printer on a worker.
fn ask(queue: &str) -> Receiver<Result<Found, Refused>> {
    let (tx, rx) = channel();
    let queue = queue.to_owned();
    let spawned = std::thread::Builder::new()
        .name("print-direct-probe".to_owned()) // ui-text-exempt: thread name
        .spawn(move || {
            let _ = tx.send(find(&queue));
        });
    if let Err(e) = spawned {
        let (tx, failed) = channel();
        let _ = tx.send(Err((FallbackReason::Unreachable, e.to_string())));
        return failed;
    }
    rx
}

/// The first of `queue`'s addresses whose printer reads PDF. Refused as
/// `NoNetworkAddress` when the port names none, `NoPdf` when a printer
/// answered without PDF, else `Unreachable`.
fn find(queue: &str) -> Result<Found, Refused> {
    let no_address = |e: String| (FallbackReason::NoNetworkAddress, e);
    let (uris, from) = match std::env::var(DIAG_URI) {
        Ok(uri) => (vec![uri], "the diagnostic address"), // ui-text-exempt: harness-only source
        Err(_) => {
            let port = pdfcer_print::queue_port(queue).map_err(|e| no_address(e.to_string()))?;
            let found = pdfcer_print::ipp::resolve_queue_port(&port)
                .map_err(|e| no_address(e.to_string()))?;
            (found.uris, found.source.describe())
        }
    };
    let mut refused = (FallbackReason::Unreachable, t::no_pdf().to_owned());
    for uri in uris {
        match net::query(&uri) {
            Ok(printer) if printer.accepts_pdf() => {
                return Ok(Found { uri, from, printer });
            }
            Ok(_) => refused = (FallbackReason::NoPdf, t::no_pdf().to_owned()),
            Err(why) if refused.0 != FallbackReason::NoPdf => {
                refused = (FallbackReason::Unreachable, why);
            }
            Err(_) => {}
        }
    }
    Err(refused)
}

fn trace_answer(queue: &str, answer: &Result<Found, Refused>) {
    crate::diag::trace(|| match answer {
        // ui-text-exempt: diagnostic trace, never displayed
        Ok(f) => format!(
            "print-direct-found queue={queue:?} uri={} from={:?}",
            f.uri, f.from
        ),
        // ui-text-exempt: diagnostic trace, never displayed
        Err((reason, why)) => format!(
            "print-direct-found queue={queue:?} uri=none fallback={} why={why:?}",
            reason.keyword()
        ),
    });
}

impl PrintDialog {
    /// What in this job only pdfcer's own drawing can print, or `None` when
    /// the printer can be handed a PDF. Layout — n-up, booklet, poster,
    /// paper, scale, order — is not here: [`compose`] lays it out.
    pub(super) fn direct_blocker(&self) -> Option<&'static str> {
        use t::needs;
        let blockers = [
            (self.print_as_image, needs::image()),
            (self.separations, needs::separations()),
            (
                self.output_color == OutputColor::BlackOnly,
                needs::black_only(),
            ),
            (self.icc.managed(), needs::profile()),
            (self.page_positions.any(), needs::moved()),
            (self.lines.fixed, needs::lines()),
            (self.running.copy().is_some(), needs::running()),
            (
                self.direct.annotated && self.scope != pdfcer_render::AnnotationScope::default(),
                needs::scope(),
            ),
            (self.finishing.chosen(), needs::finishing()),
        ];
        blockers.into_iter().find(|(on, _)| *on).map(|(_, why)| why)
    }

    /// Send this job to the printer as a PDF, when the switch is on, the job
    /// allows it and the printer reads PDF; `None` to spool it instead, with
    /// the reason kept for the receipt ([`Direct::take_fallback`]).
    pub(super) fn commit_direct(
        &self,
        doc: &OpenDoc,
        job: &Job,
        page_sizes: &[(f64, f64)],
    ) -> Option<Result<SpoolReport, String>> {
        self.direct.take_fallback();
        if !self.vector_route.pdf_direct {
            return None;
        }
        if let Some(why) = self.direct_blocker() {
            self.direct.fall_back(FallbackReason::DriverOnlyOption, why);
            return None;
        }
        let found = match self.direct.answer()? {
            Ok(found) => found,
            Err((reason, why)) => {
                self.direct.fall_back(reason, &why);
                return None;
            }
        };
        let layout = self.direct_layout(job, page_sizes, doc.view.page_index);
        let copy = match doc.print_ready_copy() {
            Ok(copy) => copy,
            Err(e) => return Some(Err(e.to_string())),
        };
        let mut notes = copy.report.disclosures;
        let payload = if layout.composed {
            match compose::sheets(
                doc,
                &copy.bytes,
                &layout,
                &super::poster::document_name(doc),
            ) {
                Ok(composed) => {
                    let (w, h) = compose::mm(layout.paper.physical_pt);
                    notes.push(t::composed(composed.pages, w, h));
                    notes.extend(composed.notes);
                    Payload::Composed {
                        pdf: composed.pdf,
                        sides: composed.pages,
                    }
                }
                Err(why) => {
                    self.direct.fall_back(FallbackReason::CannotCompose, &why);
                    return None;
                }
            }
        } else {
            notes.insert(0, t::printer_lays_out().to_owned());
            Payload::Document(copy.bytes)
        };
        Some(self.send(doc, &layout, &found, payload, notes))
    }

    fn send(
        &self,
        doc: &OpenDoc,
        layout: &compose::Layout,
        found: &Found,
        payload: Payload,
        notes: Vec<String>,
    ) -> Result<SpoolReport, String> {
        let device = self.effective_device();
        let count = u32::try_from(doc.pages.len()).unwrap_or(u32::MAX);
        let (bytes, pages, composed) = match payload {
            Payload::Composed { pdf, sides } => (pdf, sides, true),
            Payload::Document(bytes) => (bytes, layout.sequence.len(), false),
        };
        let request = NetJob {
            copies: self.copies,
            sides: Some(match device.duplex {
                Duplex::Simplex => Sides::OneSided,
                Duplex::LongEdge => Sides::TwoSidedLongEdge,
                Duplex::ShortEdge => Sides::TwoSidedShortEdge,
            }),
            ranges: if composed {
                Vec::new()
            } else {
                ranges(&layout.sequence, count)
            },
            monochrome: self.output_color == OutputColor::Grayscale,
            finishing: pdfcer_print::finishing::Finishing::default(),
            scaling: Some(scaling(composed, self.scale)),
            orientation: match device.orientation {
                _ if composed => None,
                Orientation::Auto => None,
                Orientation::Portrait => Some(3),
                Orientation::Landscape => Some(4),
            },
            uncollated: self.uncollated,
        };
        let dry =
            std::env::var(DIAG_DRY_RUN).is_ok_and(|v| v == "1") && std::env::var(DIAG_URI).is_err();
        trace_route(&format!(
            "direct uri={} bytes={} pages={pages} ranges={:?} copies={} composed={} dry_run={dry}", // ui-text-exempt: diagnostic trace
            found.uri,
            bytes.len(),
            request.ranges,
            request.copies,
            u8::from(composed),
        ));
        let receipt = if dry {
            net::NetReceipt {
                job_id: None,
                ignored: Vec::new(),
                bytes: bytes.len(),
            }
        } else {
            net::send(
                &found.uri,
                &super::poster::document_name(doc),
                &bytes,
                &request,
                &found.printer,
            )?
        };
        Ok(sent_report(pages, !dry, found, receipt, notes))
    }
}

/// What is handed to the printer.
enum Payload {
    /// The print-ready copy, laid out by the printer.
    Document(Vec<u8>),
    /// Sheets the engine composed, `sides` of them.
    Composed { pdf: Vec<u8>, sides: usize },
}

/// The IPP `print-scaling` keyword. Composed sheets are final: actual size.
const fn scaling(composed: bool, scale: ScaleMode) -> &'static str {
    match scale {
        _ if composed => "none",                  // ui-text-exempt: IPP keyword
        ScaleMode::ActualSize => "none",          // ui-text-exempt: IPP keyword
        ScaleMode::ShrinkOversized => "auto-fit", // ui-text-exempt: IPP keyword
        ScaleMode::Fit | ScaleMode::Custom(_) => "fit", // ui-text-exempt: IPP keyword
    }
}

fn sent_report(
    pages: usize,
    printed: bool,
    found: &Found,
    receipt: net::NetReceipt,
    notes: Vec<String>,
) -> SpoolReport {
    SpoolReport {
        pages,
        printed,
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
            notes,
        }),
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
        (Some(Err((_, why))), _) => t::unavailable(&why),
        (Some(Ok(_)), Some(why)) => t::needs_layout(why),
        (Some(Ok(found)), None) => match dialog.direct.composed_for {
            Some(paper) => {
                let (w, h) = compose::mm(paper);
                t::ready_composed(&found.uri, found.from, w, h)
            }
            None => t::ready(&found.uri, found.from),
        },
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
    use super::{ScaleMode, ranges, scaling};

    #[test]
    fn every_page_in_order_is_no_range() {
        assert!(ranges(&[0, 1, 2], 3).is_empty());
    }

    #[test]
    fn a_subset_becomes_runs() {
        assert_eq!(ranges(&[0, 2, 3, 5], 6), vec![(1, 1), (3, 4), (6, 6)]);
        assert_eq!(ranges(&[0, 1], 3), vec![(1, 2)]);
    }

    #[test]
    fn composed_sheets_print_at_actual_size() {
        assert_eq!(scaling(true, ScaleMode::Fit), "none");
        assert_eq!(scaling(false, ScaleMode::Fit), "fit");
        assert_eq!(scaling(false, ScaleMode::ShrinkOversized), "auto-fit");
    }
}
