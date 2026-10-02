//! # `text::panels::models` — every string the 3D models section shows
//!
//! Covers `pdfcer_gui::panels::attachments::models` and the insert and save verbs in
//! `pdfcer_gui::app::actions::models`.

use pdfcer_core::threed::{
    MAX_3D_ARTWORKS, ThreeDArtwork, ThreeDEmbedError, ThreeDFormat, ThreeDNotes, ThreeDSource,
};

/// The section's heading.
#[must_use]
pub fn heading() -> &'static str {
    "3D models"
}

/// How many models the document carries.
#[must_use]
pub fn count(total: usize) -> String {
    match total {
        0 => "No 3D model pdfcer can read.".to_owned(),
        1 => "1 3D model.".to_owned(),
        n => format!("{n} 3D models."),
    }
}

/// One model: its page, its format, and what a viewer would show with it.
#[must_use]
pub fn row(artwork: &ThreeDArtwork) -> String {
    let format = artwork
        .declared
        .as_ref()
        .map_or_else(|| "3D".to_owned(), ThreeDFormat::label);
    let mut said = format!("Page {} — {format} model", artwork.page_index + 1);
    match artwork.view_count {
        0 => {}
        1 => said.push_str(", 1 saved view"),
        n => said.push_str(&format!(", {n} saved views")),
    }
    if !artwork.has_poster {
        said.push_str(", no preview picture");
    }
    said
}

/// Where the model is kept, when that is not simply its own 3D annotation.
#[must_use]
pub fn source(artwork: &ThreeDArtwork) -> Option<String> {
    match &artwork.source {
        ThreeDSource::Stream { shared: true } => {
            Some("Its data is shared with another 3D annotation.".to_owned())
        }
        ThreeDSource::RichMediaAsset {
            name: Some(name), ..
        } => {
            // The name is the document's; control characters are not shown.
            let name: String = name.chars().filter(|c| !c.is_control()).collect();
            Some(format!("A rich-media asset named \"{name}\"."))
        }
        ThreeDSource::RichMediaAsset { name: None, .. } => {
            Some("A rich-media asset with no name.".to_owned())
        }
        _ => None,
    }
}

/// What the listing could not do.
#[must_use]
pub fn listing_notes(notes: &ThreeDNotes) -> Vec<String> {
    let mut said = Vec::new();
    if notes.page_tree_unwalkable {
        said.push(
            "pdfcer could not walk this document's pages, so models may be missing.".to_owned(),
        );
    }
    if notes.truncated {
        said.push(format!(
            "The list stops at {MAX_3D_ARTWORKS} models; the document has more."
        ));
    }
    match notes.annotations_without_stream {
        0 => {}
        1 => said.push("One 3D annotation carries no model data pdfcer can find.".to_owned()),
        n => said.push(format!(
            "{n} 3D annotations carry no model data pdfcer can find."
        )),
    }
    said
}

/// The row's button.
#[must_use]
pub fn save_button() -> &'static str {
    "Save model…"
}

/// The button's tip.
#[must_use]
pub fn save_tooltip() -> &'static str {
    "Write this model's data to a file exactly as it is stored — U3D, PRC or STEP — for a 3D viewer or CAD program."
}

/// The model the button named is no longer in the document.
#[must_use]
pub fn gone() -> &'static str {
    "That 3D model is no longer in the document as listed; nothing was saved."
}

/// The model's data could not be read out.
#[must_use]
pub fn extract_failed(detail: &str) -> String {
    format!("pdfcer could not read that 3D model out of the document: {detail}")
}

/// Written.
#[must_use]
pub fn saved(path: &str) -> String {
    format!("Saved the 3D model to {path}.")
}

/// The document names one format and the data is another.
#[must_use]
pub fn mismatch(declared: &str, found: &str) -> String {
    format!(
        "The document calls this model {declared}, but its data is {found}; it was saved with the {found} extension."
    )
}

/// The data matches no format pdfcer knows.
#[must_use]
pub fn unrecognised() -> &'static str {
    "pdfcer does not recognise this model's data as U3D, PRC or STEP; it was saved as it is."
}

/// The file could not be written.
#[must_use]
pub fn save_failed(detail: &str) -> String {
    format!("The 3D model could not be written: {detail}")
}

/// The file name the save dialog suggests, without extension.
#[must_use]
pub fn suggested_stem(document_stem: &str, page_index: usize) -> String {
    format!("{document_stem} - page {} model", page_index + 1)
}

/// The insert picker's title.
#[must_use]
pub fn insert_dialog_title() -> &'static str {
    "Place a 3D model"
}

/// The insert picker's filter name.
#[must_use]
pub fn insert_filter() -> &'static str {
    "3D models (U3D, PRC)"
}

/// The chosen file could not be read.
#[must_use]
pub fn insert_unreadable(detail: &str) -> String {
    format!("The 3D model file could not be read: {detail}")
}

/// The file is not U3D or PRC; STEP gets its own answer.
#[must_use]
pub fn insert_refused(error: &ThreeDEmbedError) -> String {
    match error {
        ThreeDEmbedError::Empty => "That 3D model file is empty; nothing was placed.".to_owned(),
        ThreeDEmbedError::NotEmbeddable { format } => format!(
            "A PDF can carry only U3D or PRC models, and that file is {format}. Convert it to PRC or U3D in your CAD program, then place it."
        ),
        _ => "That file is not a U3D or PRC model; nothing was placed.".to_owned(),
    }
}

/// Placed.
#[must_use]
pub fn inserted(format: &ThreeDFormat, page_index: usize) -> String {
    format!(
        "Placed a {} model on page {}. The model itself opens in a 3D-capable reader such as Acrobat.",
        format.label(),
        page_index + 1
    )
}

/// The page's picture of a placed model is pdfcer's own drawing of it.
#[must_use]
pub fn poster_rendered(parts_missing: bool) -> &'static str {
    if parts_missing {
        "The picture on the page is pdfcer's drawing of the model from above its front-right corner, not one of the model's own saved views. Some parts could not be drawn and are missing from it."
    } else {
        "The picture on the page is pdfcer's drawing of the model from above its front-right corner, not one of the model's own saved views."
    }
}

/// The page shows a placeholder for a placed model, and the engine's reason.
#[must_use]
pub fn poster_placeholder(reason: &str) -> String {
    format!(
        "The page shows a placeholder box for this model, because {reason}. The model itself is unchanged."
    )
}

/// The row's second button, on a PRC model.
#[must_use]
pub fn mesh_button() -> &'static str {
    "Save as mesh…"
}

/// Its tip.
#[must_use]
pub fn mesh_tooltip() -> &'static str {
    "Write this PRC model's triangles as an STL or OBJ file, for a 3D printer, a mesh editor or a CAD import. Give the file an .obj ending for OBJ."
}

/// The mesh save dialog's title.
#[must_use]
pub fn mesh_dialog_title() -> &'static str {
    "Save the 3D model as a mesh"
}

/// The STL filter name.
#[must_use]
pub fn mesh_filter_stl() -> &'static str {
    "STL mesh"
}

/// The OBJ filter name.
#[must_use]
pub fn mesh_filter_obj() -> &'static str {
    "Wavefront OBJ mesh"
}

/// Only PRC is decoded.
#[must_use]
pub fn mesh_not_prc() -> &'static str {
    "Only a PRC model can be saved as a mesh; nothing was written. Save model… writes this one as it is stored."
}

/// The model could not be decoded.
#[must_use]
pub fn mesh_unreadable(detail: &str) -> String {
    format!("pdfcer could not read this PRC model's shape: {detail}")
}

/// The model holds no triangles pdfcer can decode.
#[must_use]
pub fn mesh_empty(compressed: usize) -> String {
    match compressed {
        0 => "This model holds no triangle mesh; nothing was written.".to_owned(),
        1 => "This model's mesh is stored in a compressed form pdfcer cannot rebuild; nothing was written.".to_owned(),
        n => format!(
            "This model's {n} meshes are stored in a compressed form pdfcer cannot rebuild; nothing was written."
        ),
    }
}

/// Written.
#[must_use]
pub fn mesh_saved(path: &str, meshes: usize, triangles: usize) -> String {
    let parts = if meshes == 1 {
        "1 part".to_owned()
    } else {
        format!("{meshes} parts")
    };
    format!("Saved {triangles} triangles in {parts} to {path}.")
}

/// Parts are not moved into place: the assembly could not be read.
#[must_use]
pub fn mesh_placement_note() -> &'static str {
    "pdfcer could not read how this model's parts are assembled, so each part is where the model file stores it and parts may overlap."
}

/// Parts are moved into place.
#[must_use]
pub fn mesh_placed_note() -> &'static str {
    "Each part is where the model's assembly puts it."
}

/// The row's third button, on a PRC model.
#[must_use]
pub fn view_button() -> &'static str {
    "View…"
}

/// Its tip.
#[must_use]
pub fn view_tooltip() -> &'static str {
    "Look at this PRC model from any side: drag to turn it, scroll to zoom."
}

/// The viewer's title.
#[must_use]
pub fn view_title(page_index: usize) -> String {
    format!("3D model on page {}", page_index + 1)
}

/// How to move the camera.
#[must_use]
pub fn view_hint() -> &'static str {
    "Drag to turn the model, drag with the right button to move it, scroll to zoom in on \
     the pointer. F11 fills the screen; Esc leaves it."
}

/// The button that minimises the viewer.
#[must_use]
pub fn view_minimize() -> &'static str {
    "Minimize"
}

/// Its tip.
#[must_use]
pub fn view_minimize_tooltip() -> &'static str {
    "Shrink the viewer to the taskbar. It comes back with the same view."
}

/// The button that fills the screen with the viewer.
#[must_use]
pub fn view_full_screen() -> &'static str {
    "Full screen"
}

/// Its tip.
#[must_use]
pub fn view_full_screen_tooltip() -> &'static str {
    "Fill the screen with this window (F11). Esc or F11 brings it back."
}

/// The same button while the viewer fills the screen.
#[must_use]
pub fn view_full_screen_leave() -> &'static str {
    "Exit full screen"
}

/// Its tip.
#[must_use]
pub fn view_full_screen_leave_tooltip() -> &'static str {
    "Put this window back to its size (Esc or F11)."
}

/// What the picture leaves out.
#[must_use]
pub fn view_flat_note() -> &'static str {
    "Drawn in one colour, lit from where you look. The model's own colours, textures, lights and saved views are not shown."
}

/// The size of what is shown.
#[must_use]
pub fn view_census(parts: usize, triangles: usize) -> String {
    let parts = if parts == 1 {
        "1 part".to_owned()
    } else {
        format!("{parts} parts")
    };
    format!("{parts}, {triangles} triangles")
}

/// The named views, in button order.
#[must_use]
pub fn view_names() -> [&'static str; 5] {
    ["Isometric", "Front", "Right", "Top", "Back"]
}

/// The fit-and-reset button.
#[must_use]
pub fn view_reset() -> &'static str {
    "Fit"
}

/// Its tip.
#[must_use]
pub fn view_reset_tooltip() -> &'static str {
    "Frame the whole model again, without changing the side you look from."
}

/// The projection switch.
#[must_use]
pub fn view_perspective() -> &'static str {
    "Perspective"
}

/// Its tip.
#[must_use]
pub fn view_perspective_tooltip() -> &'static str {
    "On, nearer parts look bigger, as to the eye. Off, sizes stay true at every depth, as in a drawing."
}

/// The picture could not be drawn.
#[must_use]
pub fn view_render_failed(detail: &str) -> String {
    format!("pdfcer could not draw this view: {detail}")
}

/// The close button.
#[must_use]
pub fn view_close() -> &'static str {
    "Close"
}

/// Only PRC is decoded.
#[must_use]
pub fn view_not_prc() -> &'static str {
    "Only a PRC model can be viewed in pdfcer. Save model… writes this one out for a 3D program."
}

/// The model holds no triangles pdfcer can decode.
#[must_use]
pub fn view_empty(compressed: usize) -> String {
    match compressed {
        0 => "This model holds no triangle mesh, so there is nothing to show.".to_owned(),
        _ => "This model's shape is stored in a compressed form pdfcer cannot rebuild, so there is nothing to show.".to_owned(),
    }
}

/// Parts left out.
#[must_use]
pub fn mesh_skipped(skipped: usize) -> String {
    match skipped {
        1 => "One part is not a triangle mesh pdfcer can rebuild (wire lines, markup or an unusual compressed mesh) and was left out.".to_owned(),
        n => format!(
            "{n} parts are not triangle meshes pdfcer can rebuild (wire lines, markup or unusual compressed meshes) and were left out."
        ),
    }
}

/// The document's version predates the format.
#[must_use]
pub fn below_version(required: &str, document: &str) -> String {
    format!(
        "This document says it is PDF {document}, and a 3D model of this kind needs PDF {required}; some readers may ignore it."
    )
}
