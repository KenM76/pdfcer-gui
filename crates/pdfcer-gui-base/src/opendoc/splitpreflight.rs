//! The engine's answer to *would this text-object split be refused*.
//!
//! `EditSession::text_object_split_refusal` takes `&mut self`, so it is asked
//! on the frame-level `&mut` (`canvas::runsplit::refresh`) and stored here; the
//! menus and conditions, which hold the document shared, read the stored
//! answer. An answer is used only for the page, object, cuts and edit epoch it
//! was measured at.

use pdfcer_core::vector::VectorEditError;

/// One measured preflight.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitPreflight {
    pub page: usize,
    pub object: usize,
    pub edit_epoch: u64,
    /// The `before_runs` handed to the engine.
    pub cuts: Vec<usize>,
    /// The first refusal the press would give; `None` when it will perform.
    pub refusal: Option<VectorEditError>,
}

impl SplitPreflight {
    /// The engine's answer for `(page, object, cuts)` at `edit_epoch`, or
    /// `None` when this measurement is about something else or is stale.
    #[must_use]
    pub fn answer(
        &self,
        (page, object): (usize, usize),
        cuts: &[usize],
        edit_epoch: u64,
    ) -> Option<Option<&VectorEditError>> {
        (self.page == page
            && self.object == object
            && self.edit_epoch == edit_epoch
            && self.cuts == cuts)
            .then_some(self.refusal.as_ref())
    }
}
