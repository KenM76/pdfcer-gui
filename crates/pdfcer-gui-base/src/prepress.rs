//! # `prepress` — a print-production verb, on its way to the edit funnel
//!
//! File ▸ Print's Fix hairlines, Flatten transparency and Convert colours
//! dialogs, and the Preflight panel's Convert to PDF/X, cannot edit: they
//! raise [`PrepressRun`] as an action. The handler runs it through the edit
//! funnel as one undo step and parks the engine's account in [`PrepressNote`],
//! keyed by the run's number so the window that asked is the one that shows it.

use std::sync::atomic::{AtomicU64, Ordering};

use pdfcer_core::preflight::PdfxLevel;
use pdfcer_core::prepress::{ColourConversion, PdfxConversion};
use pdfcer_core::transparency::FlattenOptions;

/// One press of a prepress window's Run.
#[derive(Debug, Clone, PartialEq)]
pub struct PrepressRun {
    /// [`next_run`]'s number; the note this run leaves carries it back.
    pub run: u64,
    /// The pages, zero-based. Ignored by [`PrepressVerb::Pdfx`], which is
    /// about the whole file.
    pub pages: Vec<usize>,
    /// The verb and its options.
    pub verb: PrepressVerb,
}

/// The engine verb a run calls, with the options it passes.
#[derive(Debug, Clone, PartialEq)]
pub enum PrepressVerb {
    /// `EditSession::fix_hairlines`, minimum printed width in points.
    Hairlines { min_width_pt: f64 },
    /// `EditSession::flatten_transparency`.
    Flatten(FlattenOptions),
    /// `EditSession::convert_colours`.
    ConvertColours(ColourConversion),
    /// `EditSession::convert_to_pdfx`.
    Pdfx {
        level: PdfxLevel,
        conversion: Box<PdfxConversion>,
    },
}

impl PrepressVerb {
    /// The trace token naming the verb.
    #[must_use]
    pub const fn token(&self) -> &'static str {
        match self {
            Self::Hairlines { .. } => "hairlines",
            Self::Flatten(_) => "flatten",
            Self::ConvertColours(_) => "convert-colours",
            Self::Pdfx { .. } => "pdfx",
        }
    }
}

/// What the last run reported, for the window that raised it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepressNote {
    /// The [`PrepressRun::run`] this answers.
    pub run: u64,
    /// `true` when the engine refused and nothing changed.
    pub refused: bool,
    /// The account: what changed, what could not, and every disclosure.
    pub text: String,
}

/// A fresh run number, unique for the process.
pub fn next_run() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
