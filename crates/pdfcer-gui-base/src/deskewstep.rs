//! # `deskewstep` — one turn of a File ▸ Straighten scans or Enlarge scans run
//!
//! A run is paced one page per frame, so the window paints and Stop answers
//! between pages: the engine's measure and resample need the session
//! exclusively, and a page of a 300 dpi scan takes most of a second.

use pdfcer_core::ocr::upscale::UpscaleFilter;

/// What a run does to each picture it reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanJob {
    /// Measure the picture's tilt and, when it is trusted and large enough,
    /// turn the picture level.
    Straighten {
        /// Leave the page alone when it already draws text.
        skip_text: bool,
        /// The least confidence, in percent, at which a measured tilt is
        /// applied; 0 applies every measured tilt.
        min_confidence_percent: u8,
    },
    /// Enlarge the picture's samples; it keeps its place and size on the page.
    Enlarge {
        /// The factor in halves: 3 is 1.5×. Halves keep the step `Eq` and
        /// are the slider's step.
        factor_halves: u8,
        /// The resampling kernel.
        filter: UpscaleFilter,
    },
}

/// One turn of a run, raised by `pdfcer_gui::dialogs::deskew`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeskewStep {
    /// Do the run's job to one picture.
    Page {
        /// The run; outcomes are logged under it.
        run: u64,
        /// `OpenDoc::serial` of the document the run is for.
        doc: u64,
        /// 0-based page index.
        page: usize,
        /// A `page_objects` index, or `None` for the page's scan (its
        /// largest image, `EditSession::page_scan_image`).
        object: Option<usize>,
        /// What to do to it.
        job: ScanJob,
    },
    /// The run ended, finished or stopped: fold its edits into one undo
    /// entry.
    Finish {
        /// The run.
        run: u64,
        /// `OpenDoc::serial` of the document the run is for.
        doc: u64,
    },
}
