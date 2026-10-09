//! # `deskewstep` — one turn of a File ▸ Straighten scans run
//!
//! A run is paced one page per frame, so the window paints and Stop answers
//! between pages: the engine's measure and resample need the session
//! exclusively, and a page of a 300 dpi scan takes most of a second.

/// One turn of a run, raised by `pdfcer_gui::dialogs::deskew`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeskewStep {
    /// Measure one picture and, when the tilt is trusted and large enough,
    /// straighten it.
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
        /// Leave the page alone when it already draws text.
        skip_text: bool,
    },
    /// The run ended, finished or stopped: fold its corrections into one
    /// undo entry.
    Finish {
        /// The run.
        run: u64,
        /// `OpenDoc::serial` of the document the run is for.
        doc: u64,
    },
}
