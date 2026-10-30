//! Printing several files with one setup: Tools ▸ Batch ▸ Batch print….
//!
//! The Print window opens on the first file that loads and lists every file
//! above its tabs. On Print, each file is planned with the window's settings
//! (range, scale, layout, header and footer) and spooled as its own job, in
//! the listed order. A file that prints is taken off the list, so after a
//! failure the window stays open and a second press sends only the rest. A
//! file that cannot be opened is listed as left out, with the reason, and
//! never prints.
//!
//! The finished sentence goes to the status row of the document open when
//! the batch started. With nothing open there is no row to carry it, nor the
//! sentence for a batch in which no file opened, which opens no window: both
//! reach only the trace, as Merge files' sentences do.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use egui::Ui;
use pdfcer_core::document::{DocError, Document, LoadOptions};
use pdfcer_gui_base::settings::SettingsExt as _;

use super::PrintDialog;
use crate::app::state::OpenDoc;
use crate::dialogs::sign::file_name_of as name_of;
use crate::text::print::batch as t;

/// The files a batch print is working through.
pub(crate) struct Batch {
    /// Still to print, in order. The window previews the first.
    docs: Vec<Rc<OpenDoc>>,
    /// Files left out, as (name, why).
    skipped: Vec<(String, String)>,
    /// How many files the batch started with, for the partial-run sentence.
    total: usize,
    /// The open document's revision when the batch started, for the note.
    report_epoch: Option<u64>,
}

impl Batch {
    /// Open each of `paths`; a file that cannot be opened is left out.
    pub(crate) fn load(
        paths: &[PathBuf],
        settings: &pdfcer_core::settings::Settings,
        report_epoch: Option<u64>,
    ) -> Self {
        let mut docs = Vec::new();
        let mut skipped = Vec::new();
        for path in paths {
            match load_one(path, settings) {
                Ok(doc) => docs.push(Rc::new(doc)),
                Err(why) => skipped.push((name_of(path), why)),
            }
        }
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "print-batch loaded={} skipped={} first={:?}",
                docs.len(),
                skipped.len(),
                docs.first().map(|d| name_of(&d.path)).unwrap_or_default()
            )
        });
        Self {
            total: docs.len(),
            docs,
            skipped,
            report_epoch,
        }
    }

    /// The file the window previews.
    pub(crate) fn first(&self) -> Option<Rc<OpenDoc>> {
        self.docs.first().cloned()
    }
}

/// Open one file as the app opens a document, without a password.
fn load_one(path: &Path, settings: &pdfcer_core::settings::Settings) -> Result<OpenDoc, String> {
    let document = match Document::load_with_options(path, None, LoadOptions::new()) {
        Ok(d) => d,
        Err(DocError::PasswordRequired | DocError::PasswordRequiresNormalisation) => {
            return Err(t::needs_password().to_owned());
        }
        Err(e) => return Err(e.to_string()),
    };
    let pages = pdfcer_core::page_tree::pages(&document).map_err(|e| e.to_string())?;
    let mut doc = OpenDoc::new(path.to_path_buf(), settings.open_session(document), pages);
    doc.settings = settings.clone();
    Ok(doc)
}

/// The file list, above the tabs.
pub(super) fn controls(ui: &mut Ui, batch: &Batch) {
    ui.label(egui::RichText::new(t::heading(batch.docs.len())).strong())
        .on_hover_text(t::heading_tooltip());
    egui::ScrollArea::vertical()
        .id_salt("print.batch.files") // ui-text-exempt: a widget id, never displayed
        .max_height(96.0)
        .show(ui, |ui| {
            for doc in &batch.docs {
                ui.label(t::file_row(&name_of(&doc.path), doc.pages.len()));
            }
            let colour = ui.visuals().warn_fg_color;
            for (name, why) in &batch.skipped {
                ui.colored_label(colour, t::skipped_row(name, why));
            }
        });
    ui.separator();
}

/// What a Print press on a batch did, when it did not fail.
pub(super) enum Press {
    /// A file's printer settings are not known yet; press again next frame.
    Retry,
    /// Every file was sent: the note, and the revision it is stamped with.
    Sent(Option<u64>, Vec<String>),
}

impl PrintDialog {
    /// Spool every listed file to `printer`, in order. `Err` with the
    /// partial-run sentence when one was not sent, the files already sent
    /// taken off the list.
    pub(super) fn print_batch(
        &mut self,
        batch: &mut Batch,
        printer: &str,
    ) -> Result<Press, String> {
        // Plan every file before sending any, so a printer still reporting
        // one file's paper does not split the batch across two presses.
        let mut plans = Vec::with_capacity(batch.docs.len());
        for doc in &batch.docs {
            plans.push(self.plan(doc));
        }
        if self.device_pending {
            return Ok(Press::Retry);
        }
        let mut pages = 0;
        let mut sent = 0;
        for (doc, plan) in batch.docs.iter().zip(plans) {
            let outcome = self.print_one(doc, printer, plan);
            let name = name_of(&doc.path);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("print-batch-file name={name:?} outcome={outcome:?}")
            });
            match outcome {
                Ok(n) => {
                    pages += n;
                    sent += 1;
                }
                Err(why) => {
                    let printed_before = batch.total - batch.docs.len();
                    batch.docs.drain(..sent);
                    return Err(t::stopped(printed_before + sent, batch.total, &name, &why));
                }
            }
        }
        batch.docs.clear();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("print-batch-done files={sent} pages={pages}")
        });
        Ok(Press::Sent(
            batch.report_epoch,
            vec![t::sent(batch.total, pages)],
        ))
    }

    /// Stamp, render and spool one file; the pages sent.
    fn print_one(
        &mut self,
        doc: &OpenDoc,
        printer: &str,
        plan: super::plan::Planned,
    ) -> Result<usize, String> {
        let job = plan.job.ok_or_else(|| t::printer_not_ready().to_owned())?;
        if job.plans.is_empty() {
            return Err(t::no_pages_chosen().to_owned());
        }
        self.running.ensure(doc);
        if let Some(why) = self.running.error() {
            return Err(crate::text::print::running::refused(why));
        }
        self.commit(printer, doc, &job, &plan.page_sizes)
            .map(|report| report.pages)
    }
}

impl PrintDialog {
    /// Act on a batch press's outcome, then settle how the window closes.
    pub(super) fn batch_outcome(
        &mut self,
        ctx: &egui::Context,
        outcome: Result<Press, String>,
        prefs: &mut crate::app::prefs::Prefs,
        closed: bool,
    ) -> bool {
        match outcome {
            Ok(Press::Retry) => {
                self.commit_requested = true;
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
            Ok(Press::Sent(epoch, notes)) => {
                if let Some(epoch) = epoch {
                    crate::app::actions::record_notes(epoch, notes);
                }
                self.dismissal = Some(super::Dismissal::Printed);
            }
            Err(why) => self.outcome = Some(Err(why)),
        }
        self.dismiss(prefs, closed, true)
    }

    /// The Print window for a batch, opened on its first file; `None` when
    /// no file could be opened.
    pub(crate) fn open_batch(
        batch: Batch,
        remembered: &crate::app::prefs::PrintPrefs,
    ) -> Option<Self> {
        let first = batch.first()?;
        let mut dialog = Self::open(&first, remembered);
        dialog.batch = Some(batch);
        Some(dialog)
    }

    /// The file the window previews, when this is a batch.
    pub(crate) fn batch_first(&self) -> Option<Rc<OpenDoc>> {
        self.batch.as_ref().and_then(Batch::first)
    }

    /// Whether this window is printing a batch rather than the open document.
    pub(crate) fn is_batch(&self) -> bool {
        self.batch.is_some()
    }
}
