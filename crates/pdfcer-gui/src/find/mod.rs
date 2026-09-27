//! # `find` — the Find bar
//!
//! The search itself — its state, the engine call and bringing a hit on screen —
//! is `pdfcer_gui_base::find`, re-exported here; the bar's widgets are [`bar`].

pub use pdfcer_gui_base::find::*;

/// The Find bar's widgets.
pub mod bar;
