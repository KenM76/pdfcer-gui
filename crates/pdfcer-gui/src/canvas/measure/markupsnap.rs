//! # `canvas::measure::markupsnap` — a page's markups as snap targets
//!
//! `EditSession::markup_snap_geometry`, kept for one document, page, edit and
//! layer state: it is asked for on every hover frame and changes only when the
//! page's annotations or the hidden layers do. ce dimensions are left out by
//! the engine; they are placed by their own points.

use std::cell::RefCell;
use std::rc::Rc;

use pdfcer_core::vector::MarkupSnapGeometry;

use crate::app::state::OpenDoc;

/// Document serial, page, edit epoch, layer generation.
type Key = (u64, usize, u64, u64);

thread_local! {
    static KEPT: RefCell<Option<(Key, Rc<[MarkupSnapGeometry]>)>> = const { RefCell::new(None) };
}

/// The snap targets `page`'s visible markups offer, page space. Empty when the
/// page cannot be read.
#[must_use]
pub(in crate::canvas) fn on_page(doc: &OpenDoc, page: usize) -> Rc<[MarkupSnapGeometry]> {
    let key = (doc.serial, page, doc.edit_epoch, doc.layers.generation);
    KEPT.with(|kept| {
        let mut kept = kept.borrow_mut();
        if let Some((built_for, found)) = kept.as_ref()
            && *built_for == key
        {
            return Rc::clone(found);
        }
        let found: Rc<[MarkupSnapGeometry]> = doc
            .session
            .markup_snap_geometry(page, &doc.hidden_layers())
            .unwrap_or_default()
            .into();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("markup-snap-built page={page} markups={}", found.len())
        });
        *kept = Some((key, Rc::clone(&found)));
        found
    })
}

/// [`on_page`] without the markup `own`: a markup being reshaped does not
/// snap to where it was.
#[must_use]
pub(in crate::canvas) fn on_page_but(
    doc: &OpenDoc,
    page: usize,
    own: pdfcer_core::object::ObjId,
) -> Vec<MarkupSnapGeometry> {
    on_page(doc, page)
        .iter()
        .filter(|m| m.annotation() != Some(own))
        .cloned()
        .collect()
}
