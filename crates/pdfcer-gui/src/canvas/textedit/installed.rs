//! # `canvas::textedit::installed` — every typed edit may extend the run's
//! embedded subset from the same face in the font folders
//!
//! Contract: [`augmented`] is applied to the options of every commit, preview
//! and keystroke query of a typed edit, so the three agree on which keys the
//! subset can take. The folders are this computer's font folder plus the
//! operator's own (`app::dispatch::fonts::augment_folders`).

use pdfcer_core::text_edit::EditOptions;
use pdfcer_gui_base::editmodel::installedfaces::{self, InstalledFaces};

use crate::app::prefs::Prefs;
use crate::app::state::OpenDoc;

/// `options`, able to extend the run's subset from a same-name installed face.
#[must_use]
pub fn augmented(doc: &OpenDoc, options: EditOptions) -> EditOptions {
    installedfaces::augmenting(options, warm(&doc.prefs))
}

/// The installed faces for `prefs`' folders, indexed in the background the
/// first time they are asked for.
#[must_use]
pub fn warm(prefs: &Prefs) -> &'static InstalledFaces {
    installedfaces::for_folders(&crate::app::dispatch::fonts::augment_folders(prefs))
}
