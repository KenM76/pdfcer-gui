//! # `app::prefs::offpage` — applying the remembered off-page answer to open documents
//!
//! The answer and its file format are `pdfcer_gui_base::prefs::offpage`.

pub use pdfcer_gui_base::prefs::offpage::*;

use crate::app::state::Status;

/// **Put the mode's answer into every open document's view state.**
pub fn apply_mode(prefs: &super::Prefs, mode: &str, status: &mut Status, parked: &mut [Status]) {
    let on = prefs.off_page.for_mode(mode);
    for status in std::iter::once(status).chain(parked.iter_mut()) {
        if let Status::Open(doc) = status {
            doc.view.off_page = on;
        }
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("off-page-mode mode={mode} on={on}")
    });
}
