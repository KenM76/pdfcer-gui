//! A 3D model on the page opens in the viewer when clicked in Read or Review,
//! through the same action as the Attachments panel's *View…*. In Edit a
//! click selects it like any annotation, to move or delete it.

use egui::Pos2;
use pdfcer_core::threed::{ThreeDArtwork, list_3d_with_notes};

use crate::app::state::OpenDoc;

/// The 3D artwork whose annotation is under `point` on `page_index`.
///
/// One annotation hit test, and a 3D listing only when that hit is a `/3D`
/// or `/RichMedia` annotation, so a click elsewhere costs no listing.
#[must_use]
pub fn under_pointer(
    doc: &OpenDoc,
    page_index: usize,
    point: Pos2,
    map: &crate::canvas::mapping::PageMapping,
) -> Option<ThreeDArtwork> {
    let hit = crate::canvas::selection::annot::under_pointer(doc, page_index, point, map)?;
    if !matches!(hit.target.subtype.as_str(), "3D" | "RichMedia") {
        return None;
    }
    list_3d_with_notes(&*doc.session)
        .0
        .into_iter()
        .find(|a| a.page_index == page_index && a.annot_id == Some(hit.target.id))
}
