//! # `app::actions::drawlayer` — the layer an add puts new content on
//!
//! Contract: [`for_add`] reads `OpenDoc::draw_layer_now` once per add. No
//! current layer gives `Ok(None)`. A visible current layer gives its id and
//! the receipt the add appends to its notes. A hidden one is refused before
//! the engine is called, with `LayerRefusal::DrawLayerHidden` recorded and a
//! `{label}-refused` trace, and gives `Err(())`.
//!
//! Design: `DESIGNS.md`, "Row 74 — a current layer that new content goes on".

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::app::status::decline;
use crate::text::panels::drawlayer as t;
use crate::text::panels::layeredit::LayerRefusal;

/// The current layer, as one add uses it.
pub(super) struct DrawLayer {
    /// The optional-content group the engine is handed.
    pub id: ObjId,
    /// The note naming the layer, for the add's notes.
    pub receipt: String,
}

/// See the module documentation.
pub(super) fn for_add(doc: &OpenDoc, label: &str) -> Result<Option<DrawLayer>, ()> {
    let Some((layer, hidden)) = doc.draw_layer_now() else {
        return Ok(None);
    };
    if hidden {
        crate::diag::trace(|| {
            format!(
                "{label}-refused reason=DrawLayerHidden layer={:?}",
                layer.id
            ) // ui-text-exempt: diagnostic trace, never displayed
        });
        decline::record_layer(LayerRefusal::DrawLayerHidden);
        return Err(());
    }
    crate::diag::trace(|| format!("{label}-on-layer layer={:?}", layer.id)); // ui-text-exempt: diagnostic trace, never displayed
    Ok(Some(DrawLayer {
        id: layer.id,
        receipt: t::receipt(&layer.name),
    }))
}

/// The id alone, for an engine call taking `Option<ObjId>`.
pub(super) fn id(layer: Option<&DrawLayer>) -> Option<ObjId> {
    layer.map(|l| l.id)
}
