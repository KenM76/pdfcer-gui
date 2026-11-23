//! # `canvas::measure::markupsnap` — every object on a page as a snap target
//!
//! `EditSession::page_snap_geometry` (markups, images, text runs, form fields,
//! ce dimensions, stamps, notes, links) and `vector::hidden_objects` (the
//! paths on layers switched off), kept for one document, page, edit, layer
//! state and model presence: asked for on every hover frame, changed only by
//! an edit, a layer toggle or the page's decomposition arriving.
//!
//! The decomposition is the caller's, never built here: building it is
//! synchronous, and a hover must not pay for it. Without one the page's
//! images and text runs are absent, as its paths already are.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use pdfcer_core::object::ObjId;
use pdfcer_core::vector::SnapGeometry;

use crate::app::state::OpenDoc;
use crate::canvas::target::CanvasTargetProvider;

/// What a page offers a snap query beside its paths, and the paths a hidden
/// layer withholds, as `PageObjects` indices.
#[derive(Clone, Default)]
pub(in crate::canvas) struct PageSnap {
    pub geometry: Rc<[SnapGeometry]>,
    pub skip: Rc<BTreeSet<usize>>,
}

/// Document serial, page, edit epoch, layer generation, model present.
type Key = (u64, usize, u64, u64, bool);

thread_local! {
    static KEPT: RefCell<Option<(Key, PageSnap)>> = const { RefCell::new(None) };
}

/// The snap targets `page` offers, page space. Empty when the page cannot be
/// read.
#[must_use]
pub(in crate::canvas) fn on_page(
    doc: &OpenDoc,
    page: usize,
    targets: Option<&dyn CanvasTargetProvider>,
) -> PageSnap {
    let model = targets.and_then(|t| t.page_objects_model(page));
    let key = (
        doc.serial,
        page,
        doc.edit_epoch,
        doc.layers.generation,
        model.is_some(),
    );
    KEPT.with(|kept| {
        let mut kept = kept.borrow_mut();
        if let Some((built_for, found)) = kept.as_ref()
            && *built_for == key
        {
            return found.clone();
        }
        let hidden = doc.hidden_layers();
        let geometry: Rc<[SnapGeometry]> = doc
            .session
            .page_snap_geometry(page, model, &hidden)
            .unwrap_or_default()
            .into();
        let skip = model.map_or_else(BTreeSet::new, |m| {
            pdfcer_core::vector::hidden_objects(&doc.session.view(), m, &hidden)
        });
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "page-snap-built page={page} targets={} hidden_paths={} model={}",
                geometry.len(),
                skip.len(),
                model.is_some()
            )
        });
        let found = PageSnap {
            geometry,
            skip: Rc::new(skip),
        };
        *kept = Some((key, found.clone()));
        found
    })
}

/// [`on_page`] without the annotation `own`: a markup or ce dimension being
/// reshaped does not snap to where it was.
#[must_use]
pub(in crate::canvas) fn on_page_but(
    doc: &OpenDoc,
    page: usize,
    targets: Option<&dyn CanvasTargetProvider>,
    own: ObjId,
) -> PageSnap {
    let all = on_page(doc, page, targets);
    PageSnap {
        geometry: all
            .geometry
            .iter()
            .filter(|m| m.annotation() != Some(own))
            .cloned()
            .collect(),
        skip: all.skip,
    }
}
