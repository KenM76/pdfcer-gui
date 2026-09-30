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
        "Placed a {} model on page {}. pdfcer shows a placeholder picture; the model itself opens in a 3D-capable reader such as Acrobat.",
        format.label(),
        page_index + 1
    )
}

/// The document's version predates the format.
#[must_use]
pub fn below_version(required: &str, document: &str) -> String {
    format!(
        "This document says it is PDF {document}, and a 3D model of this kind needs PDF {required}; some readers may ignore it."
    )
}
