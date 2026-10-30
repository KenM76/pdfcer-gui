//! # `dialogs::netprint` — **Send to network printer**
//!
//! `file.network_print`, File ▸ Print ▸ Send to network printer…. The
//! operator gives a printer's IPP address and connects; the printer's answer
//! decides which controls appear (R9: two-sided, grey, staple, punch, fold
//! and tray only when it lists them). Send hands the PDF itself to the
//! printer, which lays it out. Both exchanges run on a worker, so a slow or
//! absent printer never stalls the window.
//!
//! The document's bytes are taken when the window opens: the saved file, or
//! the current edits as an incremental save when there are any.

use std::sync::mpsc::{Receiver, channel};

use egui::Ui;
use pdfcer_gui_base::entry;
use pdfcer_gui_base::printspooler::finishing::{Edge, Staple};
use pdfcer_gui_base::printspooler::network::{self as net, NetJob, NetPrinter, NetReceipt};
use pdfcer_print::finishing::Fold;
use pdfcer_print::ipp::Sides;

use super::print::finishing::pick;
use crate::text::print::finishing as tf;
use crate::text::print::network as t;

/// Prefills the address field, for `ui-verify`.
pub const DIAG_NETPRINT_URI: &str = "PDFCER_DIAG_NETPRINT_URI"; // ui-text-exempt: an environment variable name, never displayed

const REGION_BODY: &str = "netprint.body"; // ui-text-exempt: region id
const REGION_CONNECT: &str = "netprint.connect"; // ui-text-exempt: region id
const REGION_SEND: &str = "netprint.send"; // ui-text-exempt: region id
const REGION_STAPLE: &str = "netprint.staple"; // ui-text-exempt: region id

/// The choices kept between openings: the address, and how many copies.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetForm {
    uri: String,
}

enum Reply {
    Printer(Result<NetPrinter, String>),
    Sent(Result<NetReceipt, String>),
}

/// The window's state.
pub struct NetPrintDialog {
    form: NetForm,
    name: String,
    pdf: std::sync::Arc<Vec<u8>>,
    pages: u32,
    printer: Option<NetPrinter>,
    pending: Option<Receiver<Reply>>,
    message: Option<Result<String, String>>,
    copies: u16,
    ranges: String,
    two_sided: bool,
    grey: bool,
    staple: Option<Staple>,
    punch: Option<Edge>,
    fold: Option<Fold>,
    bin: Option<String>,
    close_requested: bool,
}

impl NetPrintDialog {
    /// Open on `pdf`, the document named `name` of `pages` pages. An `Err`
    /// is why the bytes could not be produced; the window shows it and
    /// offers no Send.
    #[must_use]
    pub fn open(
        name: &str,
        pdf: Result<Vec<u8>, String>,
        pages: u32,
        form: Option<&NetForm>,
    ) -> Self {
        let mut form = form.cloned().unwrap_or_default();
        if let Ok(uri) = std::env::var(DIAG_NETPRINT_URI) {
            form.uri = uri;
        }
        let (pdf, message) = match pdf {
            Ok(bytes) => (bytes, None),
            Err(why) => (Vec::new(), Some(Err(why))),
        };
        crate::diag::trace(|| format!("netprint-opened bytes={} pages={pages}", pdf.len())); // ui-text-exempt: trace
        Self {
            form,
            name: name.to_owned(),
            pdf: std::sync::Arc::new(pdf),
            pages,
            printer: None,
            pending: None,
            message,
            copies: 1,
            ranges: String::new(),
            two_sided: false,
            grey: false,
            staple: None,
            punch: None,
            fold: None,
            bin: None,
            close_requested: false,
        }
    }

    /// The form to seed the next window with.
    #[must_use]
    pub fn remembered(&self) -> NetForm {
        self.form.clone()
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        self.receive(ctx);
        let (frame, ()) = crate::dialogs::host::Host::new(
            "netprint", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(440.0, 460.0),
            egui::vec2(340.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    /// Take a worker's answer, if one has arrived.
    fn receive(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.pending else { return };
        let Ok(reply) = rx.try_recv() else {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
            return;
        };
        self.pending = None;
        match reply {
            Reply::Printer(Ok(p)) => {
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed
                        "netprint-connected pdf={} staples={} punch={} folds={} bins={} two_sided={}",
                        u8::from(p.accepts_pdf()),
                        p.staples.len(),
                        p.punch.len(),
                        p.folds.len(),
                        p.bins.len(),
                        u8::from(p.two_sided)
                    )
                });
                self.message = Some(Ok(t::connected(p.model.as_deref())));
                self.printer = Some(p);
            }
            Reply::Printer(Err(e)) | Reply::Sent(Err(e)) => {
                crate::diag::trace(|| format!("netprint-refused why={e}")); // ui-text-exempt: trace
                self.message = Some(Err(e));
            }
            Reply::Sent(Ok(r)) => {
                self.message = Some(Ok(t::sent(r.job_id, r.bytes, &r.ignored)));
            }
        }
    }

    fn connect(&mut self) {
        let (tx, rx) = channel();
        let uri = self.form.uri.trim().to_owned();
        self.printer = None;
        self.message = None;
        let spawned = std::thread::Builder::new()
            .name("netprint-query".to_owned()) // ui-text-exempt: thread name
            .spawn(move || drop(tx.send(Reply::Printer(net::query(&uri)))));
        match spawned {
            Ok(_) => self.pending = Some(rx),
            Err(e) => self.message = Some(Err(e.to_string())),
        }
    }

    fn job(&self) -> Result<NetJob, String> {
        let mut finishing = pdfcer_gui_base::printspooler::finishing::request(
            self.staple,
            self.punch,
            self.bin.as_deref(),
        );
        finishing.fold = self.fold;
        Ok(NetJob {
            copies: self.copies,
            sides: self.two_sided.then_some(Sides::TwoSidedLongEdge),
            ranges: net::parse_ranges(&self.ranges, self.pages)?,
            monochrome: self.grey,
            finishing,
        })
    }

    fn send(&mut self, job: NetJob, printer: NetPrinter) {
        let (tx, rx) = channel();
        let (uri, name, pdf) = (
            self.form.uri.trim().to_owned(),
            self.name.clone(),
            self.pdf.clone(),
        );
        self.message = None;
        let spawned = std::thread::Builder::new()
            .name("netprint-send".to_owned()) // ui-text-exempt: thread name
            .spawn(move || {
                drop(tx.send(Reply::Sent(net::send(&uri, &name, &pdf, &job, &printer))))
            });
        match spawned {
            Ok(_) => self.pending = Some(rx),
            Err(e) => self.message = Some(Err(e.to_string())),
        }
    }

    fn body(&mut self, ui: &mut Ui) {
        ui.label(t::address_label());
        ui.horizontal(|ui| {
            // escape-disposition: dialog-cancels
            ui.add(egui::TextEdit::singleline(&mut self.form.uri).hint_text(t::address_hint()));
            if self.pending.is_none() && !self.form.uri.trim().is_empty() {
                let r = ui.button(t::connect());
                crate::diag::ui_rect(REGION_CONNECT, r.rect);
                if r.clicked() {
                    self.connect();
                }
            }
        });
        ui.label(egui::RichText::new(t::lays_out_note()).small().weak());
        ui.add_space(6.0);
        if let Some(printer) = self.printer.clone() {
            self.options(ui, &printer);
        }
        if self.pending.is_some() {
            ui.label(t::waiting());
        }
        match &self.message {
            Some(Ok(m)) => {
                ui.label(m);
            }
            Some(Err(e)) => {
                ui.label(egui::RichText::new(e).color(ui.visuals().error_fg_color));
            }
            None => {}
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
            let job = self.job();
            if let (Some(printer), Ok(job), None) = (self.printer.clone(), &job, &self.pending)
                && printer.accepts_pdf()
                && !self.pdf.is_empty()
            {
                let r = ui.button(t::send());
                crate::diag::ui_rect(REGION_SEND, r.rect);
                if r.clicked() {
                    self.send(job.clone(), printer);
                }
            }
            if let Err(why) = job {
                ui.label(egui::RichText::new(why).color(ui.visuals().error_fg_color));
            }
        });
    }

    /// The controls the printer's answer allows.
    fn options(&mut self, ui: &mut Ui, p: &NetPrinter) {
        egui::Grid::new("netprint.grid") // ui-text-exempt: widget id
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(t::copies_label());
                let (w, refusal) = entry::drag_value(ui, &mut self.copies, entry::Kind::Count);
                refusal.show(ui.add(w.range(1..=999)));
                ui.end_row();
                ui.label(t::pages_label());
                ui.text_edit_singleline(&mut self.ranges);
                ui.end_row();
            });
        if p.two_sided {
            ui.checkbox(&mut self.two_sided, t::two_sided());
        }
        if p.monochrome {
            ui.checkbox(&mut self.grey, t::grey());
        }
        if !p.staples.is_empty() {
            let r = pick(
                ui,
                tf::staple_label(),
                "netprint-staple",
                &mut self.staple,
                &p.staples,
                tf::staple,
            );
            crate::diag::ui_rect(REGION_STAPLE, r);
        }
        if !p.punch.is_empty() {
            pick(
                ui,
                tf::punch_label(),
                "netprint-punch",
                &mut self.punch,
                &p.punch,
                tf::edge,
            );
        }
        if !p.folds.is_empty() {
            pick(
                ui,
                t::fold_label(),
                "netprint-fold",
                &mut self.fold,
                &p.folds,
                t::fold,
            );
        }
        if !p.bins.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.label(tf::bin_label());
                let shown = self
                    .bin
                    .clone()
                    .unwrap_or_else(|| tf::bin_default().to_owned());
                egui::ComboBox::from_id_salt("netprint-bin")
                    .selected_text(shown)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.bin, None, tf::bin_default());
                        for bin in &p.bins {
                            ui.selectable_value(&mut self.bin, Some(bin.clone()), bin);
                        }
                    });
            });
        }
        ui.add_space(6.0);
    }
}
