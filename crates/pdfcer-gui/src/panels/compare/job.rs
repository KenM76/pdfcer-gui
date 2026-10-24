//! # `panels::compare::job` — one comparison, off the UI thread
//!
//! Loads the older version, reads the newer one (the open document's file,
//! or its unsaved state as bytes taken on the UI thread), and runs
//! `pdfcer_core::compare::compare_documents` and
//! `pdfcer_render::visual_compare` as the options ask. The worker never
//! holds the open document's `Arc<EditSession>`: an edit needs to be its sole
//! owner, and a comparison of a large drawing can take seconds.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use pdfcer_core::compare::{
    CompareError, CompareOptions, CompareReport, RegionMark, Side, compare_documents,
};
use pdfcer_core::document::Document;
use pdfcer_gui_base::pageselection::parse_page_range;
use pdfcer_render::{VisualCompareOptions, VisualCompareReport, visual_compare};

use super::Options;
use crate::text::panels::compare as t;

/// Where the newer version's bytes come from.
pub(super) enum Newer {
    /// The open document is saved: read its file in the worker.
    File(PathBuf),
    /// The open document has edits not on disk: its incremental save.
    Bytes(Vec<u8>),
}

/// Everything the worker needs, owned.
pub(super) struct Request {
    pub older: PathBuf,
    pub newer: Newer,
    pub options: Options,
}

/// What the worker found.
pub(super) struct Found {
    pub text: Option<CompareReport>,
    pub visual: Option<VisualCompareReport>,
    /// The appearance marks, in the newer document's page indices.
    pub regions: Vec<RegionMark>,
    /// Sentences about what the comparison could not do.
    pub notes: Vec<String>,
}

/// Run `request` on a worker thread; the answer arrives on the receiver and
/// `ctx` is asked to repaint.
pub(super) fn spawn(request: Request, ctx: egui::Context) -> Receiver<Result<Found, String>> {
    let (tx, rx) = channel();
    // A thread that never started drops the sender, which `poll` reads as a
    // lost worker and says so.
    if let Err(e) = std::thread::Builder::new()
        // ui-text-exempt: a thread name, never displayed
        .name("pdfcer-compare".to_owned())
        .spawn(move || deliver(&tx, &ctx, run(request)))
    {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("compare-spawn-failed error={e}")
        });
    }
    rx
}

fn deliver(tx: &Sender<Result<Found, String>>, ctx: &egui::Context, found: Result<Found, String>) {
    let _ = tx.send(found);
    ctx.request_repaint();
}

/// Load both sides and compare them.
fn run(request: Request) -> Result<Found, String> {
    let began = std::time::Instant::now();
    let old = Document::load(&request.older).map_err(|e| t::older_unreadable(&e.to_string()))?;
    let new = match request.newer {
        Newer::File(path) => Document::load(&path),
        Newer::Bytes(bytes) => Document::from_bytes(bytes),
    }
    .map_err(|e| t::newer_unreadable(&e.to_string()))?;
    let pages = pages_of(&old, &new, &request.options)?;
    let mut found = Found {
        text: None,
        visual: None,
        regions: Vec::new(),
        notes: Vec::new(),
    };
    if request.options.text {
        compare_text(&old, &new, &pages, &request.options, &mut found);
    }
    if request.options.appearance {
        compare_looks(&old, &new, &pages, &mut found);
    }
    let ms = began.elapsed().as_millis();
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "compare-ran text_changes={} regions={} notes={} ms={ms}",
            found.text.as_ref().map_or(0, |r| r.changes.len()),
            found.regions.len(),
            found.notes.len(),
        )
    });
    Ok(found)
}

/// The two page selections, zero-based; `None` is every page.
type Pages = (Option<Vec<usize>>, Option<Vec<usize>>);

fn pages_of(old: &Document, new: &Document, options: &Options) -> Result<Pages, String> {
    let count = |doc: &Document| pdfcer_core::page_tree::pages(doc).map_or(0, |p| p.len());
    let parse = |spec: &str, n: usize, which: &str| -> Result<Option<Vec<usize>>, String> {
        if spec.trim().is_empty() {
            return Ok(None);
        }
        parse_page_range(spec, n)
            .map(Some)
            .ok_or_else(|| t::pages_invalid(which))
    };
    Ok((
        parse(&options.old_pages, count(old), t::old_pages())?,
        parse(&options.new_pages, count(new), t::new_pages())?,
    ))
}

fn compare_text(
    old: &Document,
    new: &Document,
    pages: &Pages,
    options: &Options,
    found: &mut Found,
) {
    let mut opts = CompareOptions::default().with_formatting(options.formatting);
    if let Some(p) = &pages.0 {
        opts = opts.with_pages(Side::Old, p.clone());
    }
    if let Some(p) = &pages.1 {
        opts = opts.with_pages(Side::New, p.clone());
    }
    match compare_documents(old, new, &opts) {
        Ok(report) => {
            let (o, n) = report.unreadable_pages;
            if o + n > 0 {
                found.notes.push(t::unreadable(o, n));
            }
            if report.truncated {
                found.notes.push(t::truncated().to_owned());
            }
            found.text = Some(report);
        }
        Err(CompareError::CopyNotPermitted { side }) => {
            found
                .notes
                .push(t::copy_forbidden(side == Side::Old).to_owned());
        }
        Err(e) => found.notes.push(t::text_failed(&e.to_string())),
    }
}

fn compare_looks(old: &Document, new: &Document, pages: &Pages, found: &mut Found) {
    let opts = VisualCompareOptions::default().with_pages(pages.0.clone(), pages.1.clone());
    match visual_compare(old, new, &opts) {
        Ok(report) => {
            found.regions = super::rows::region_marks(&report, new);
            found.notes.extend(super::rows::visual_notes(&report));
            found.visual = Some(report);
        }
        Err(e) => found.notes.push(t::appearance_failed(&e.to_string())),
    }
}
