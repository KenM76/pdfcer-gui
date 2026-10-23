//! `app::dispatch::compactopen` — the three commands that open *Save a
//! compacted copy*, each at its own starting point.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/dispatch/compactopen.md`.

use crate::app::PdfcerApp;
use crate::app::state::Status;
use crate::dialogs::compact_target::Start;

#[cfg(test)]
const PLAIN: &str = "file.save_compacted";
const REDUCE: &str = "file.reduce_size";
const FIT: &str = "file.fit_size";

/// The starting point `id` names.
fn start_of(id: &str) -> Start {
    match id {
        REDUCE => Start::Reduce,
        FIT => Start::Fit,
        _ => Start::Plain,
    }
}

impl PdfcerApp {
    /// Open the compact window at `id`'s starting point, on the tab that was
    /// right-clicked when a tab menu asked. A refusal (a file the engine
    /// cannot rewrite in full) is recorded, not swallowed.
    pub(in crate::app) fn dispatch_compact(&mut self, id: &str) {
        if let Some(slot) = self.tab_menu_target {
            self.activate_slot(slot);
        }
        let epoch = match &self.status {
            Status::Open(doc) => doc.edit_epoch,
            _ => return,
        };
        if let Some(refusal) = self.dialogs.open_compact(&self.status, start_of(id)) {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("compact-refused detail={refusal}")
            });
            crate::app::actions::record_note(epoch, refusal);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_command_names_its_own_starting_point() {
        assert_eq!(start_of(PLAIN), Start::Plain);
        assert_eq!(start_of(REDUCE), Start::Reduce);
        assert_eq!(start_of(FIT), Start::Fit);
    }
}
