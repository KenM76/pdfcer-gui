//! Recording where each document was left, and returning there when it is
//! opened again — `OPERATOR_REQUESTS.md` O464, under
//! `Prefs::reopen_where_left`.
//!
//! Recorded when a tab closes and when the program exits, from the centre the
//! canvas last wrote (`OpenDoc::view_centre`). Restored in `adopt`, after the
//! opening preferences, so a remembered view outranks the opening fit; the
//! canvas places the remembered point on its first laid-out frame.

use crate::app::PdfcerApp;
use crate::app::state::{OpenDoc, Status};
use crate::viewer::FitMode;
use crate::viewer::lastview::{self, LastView};

/// The view `doc` is showing, if it has a file and its canvas has drawn.
fn showing(doc: &OpenDoc) -> Option<(&std::path::Path, LastView)> {
    let path = doc.stored_under()?;
    let (page, frac) = doc.view_centre?;
    let view = LastView {
        page,
        frac,
        zoom: doc.view.zoom,
        fit: doc.view.fit,
    };
    Some((path, view))
}

impl PdfcerApp {
    /// Record where the document in `slot` is, before it closes.
    pub(super) fn remember_view_of(&self, slot: usize) {
        if !self.prefs.reopen_where_left {
            return;
        }
        if let Some(Status::Open(doc)) = self.slot(slot)
            && let Some((path, view)) = showing(doc)
        {
            lastview::remember(path, view);
        }
    }

    /// Record every open document, as the program exits.
    pub(super) fn remember_every_view(&self) {
        for slot in 0..self.document_count() {
            self.remember_view_of(slot);
        }
    }

    /// Return the just-opened document to where it was left.
    pub(super) fn restore_last_view(&mut self) {
        if !self.prefs.reopen_where_left {
            return;
        }
        let ceiling = self.prefs.max_zoom_percent / 100.0;
        let Status::Open(doc) = &mut self.status else {
            return;
        };
        let Some(view) = doc.stored_under().and_then(lastview::recall) else {
            return;
        };
        let count = doc.pages.len();
        match view.fit {
            FitMode::None => doc.view.set_zoom(view.zoom, ceiling * doc.current_unit()),
            fit => doc.view.set_fit(fit),
        }
        doc.frame.observed_zoom = doc.view.zoom;
        doc.view.go_to_page(view.page, count);
        doc.reopen_at = Some(view.frac);
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "last-view-restored page={} at={:.4},{:.4} zoom={:.4} fit={:?}",
                doc.view.page_index, view.frac.0, view.frac.1, doc.view.zoom, doc.view.fit
            )
        });
    }
}
