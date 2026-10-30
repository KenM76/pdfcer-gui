//! Strings for Insert 3D model's options window, the lighting a written view
//! carries, and the viewer's part visibility. Consumed by
//! `pdfcer_gui::dialogs::insert_model`, `pdfcer_gui::app::actions::model_insert`
//! and `pdfcer_gui::dialogs::model3d`.

use pdfcer_core::threed::{ThreeDActivation, ThreeDLighting};

/// The window's title.
#[must_use]
pub fn insert_title() -> &'static str {
    "Insert 3D model"
}

/// The first row: the chosen file.
#[must_use]
pub fn insert_file(name: &str) -> String {
    format!("Model: {name}")
}

/// Where the model goes.
#[must_use]
pub fn insert_where(page: usize) -> String {
    format!(
        "Placed in the middle of page {}; move or resize it afterwards.",
        page + 1
    )
}

/// The activation row's label.
#[must_use]
pub fn activation_label() -> &'static str {
    "Turn the model on"
}

/// Its tip.
#[must_use]
pub fn activation_tooltip() -> &'static str {
    "When a PDF reader starts the 3D model in place of its picture. Until then, and on paper when it starts on a click, the reader shows the picture."
}

/// One activation choice.
#[must_use]
pub fn activation(a: ThreeDActivation) -> &'static str {
    match a {
        ThreeDActivation::PageOpen => "When the page opens",
        ThreeDActivation::PageVisible => "When the page comes into view",
        _ => "When it is clicked",
    }
}

/// The lighting row's label.
#[must_use]
pub fn lighting_label() -> &'static str {
    "Lighting"
}

/// Its tip, on the Insert window.
#[must_use]
pub fn lighting_tooltip() -> &'static str {
    "How a PDF reader lights the model. Automatic lights a converted glTF model as CAD, because it carries no lights of its own and would otherwise draw dark, and leaves a PRC or U3D model's own lighting alone."
}

/// Its tip, beside Save views in the file.
#[must_use]
pub fn views_lighting_tooltip() -> &'static str {
    "The lighting every saved view asks a PDF reader for. As the reader chooses writes none."
}

/// The automatic choice on the Insert window.
#[must_use]
pub fn lighting_auto() -> &'static str {
    "Automatic"
}

/// No scheme written, beside Save views in the file.
#[must_use]
pub fn lighting_unset() -> &'static str {
    "As the reader chooses"
}

/// One lighting scheme (ISO 32000-1 Table 309), as Acrobat lists them.
#[must_use]
pub fn lighting(scheme: ThreeDLighting) -> &'static str {
    match scheme {
        ThreeDLighting::Artwork => "The model's own lights",
        ThreeDLighting::None => "No lights",
        ThreeDLighting::White => "White lights",
        ThreeDLighting::Day => "Day lights",
        ThreeDLighting::Night => "Night lights",
        ThreeDLighting::Hard => "Bright lights",
        ThreeDLighting::Primary => "Primary color lights",
        ThreeDLighting::Blue => "Blue lights",
        ThreeDLighting::Red => "Red lights",
        ThreeDLighting::Cube => "Cube lights",
        ThreeDLighting::Cad => "CAD optimized lights",
        ThreeDLighting::Headlamp => "Headlamp",
        _ => "Other lights",
    }
}

/// Every scheme, in the order the drop-downs list them.
#[must_use]
pub fn lighting_schemes() -> [ThreeDLighting; 12] {
    [
        ThreeDLighting::Cad,
        ThreeDLighting::Artwork,
        ThreeDLighting::White,
        ThreeDLighting::Day,
        ThreeDLighting::Night,
        ThreeDLighting::Hard,
        ThreeDLighting::Primary,
        ThreeDLighting::Blue,
        ThreeDLighting::Red,
        ThreeDLighting::Cube,
        ThreeDLighting::Headlamp,
        ThreeDLighting::None,
    ]
}

/// The picture row's label.
#[must_use]
pub fn poster_label() -> &'static str {
    "Picture on the page"
}

/// Its tip.
#[must_use]
pub fn poster_tooltip() -> &'static str {
    "What the page shows before the model starts, and what prints. A drawn picture is pdfcer's rendering of the model; the plain box draws nothing of it."
}

/// Draw a picture of the model.
#[must_use]
pub fn poster_rendered() -> &'static str {
    "A picture of the model"
}

/// The plain placeholder box.
#[must_use]
pub fn poster_plain() -> &'static str {
    "A plain box"
}

/// The glTF section's heading.
#[must_use]
pub fn gltf_heading() -> &'static str {
    "glTF conversion"
}

/// The colours switch.
#[must_use]
pub fn gltf_linear() -> &'static str {
    "Read colours as linear (the glTF standard)"
}

/// Its tip.
#[must_use]
pub fn gltf_linear_tooltip() -> &'static str {
    "glTF stores base colours as linear light, which pdfcer encodes to sRGB. Turn this off for a model whose exporter wrote sRGB values there anyway, which otherwise come out washed out."
}

/// The up-axis switch.
#[must_use]
pub fn gltf_z_up() -> &'static str {
    "Turn the model so +Z is up"
}

/// Its tip.
#[must_use]
pub fn gltf_z_up_tooltip() -> &'static str {
    "glTF models stand on +Y; CAD and PDF 3D viewers expect +Z. Turn this off to keep the axes as stored."
}

/// The commit button.
#[must_use]
pub fn place_button() -> &'static str {
    "Place"
}

/// Cancel.
#[must_use]
pub fn cancel_button() -> &'static str {
    "Cancel"
}

/// The opening view's name, as a reader lists it.
#[must_use]
pub fn default_view_name() -> &'static str {
    "Default"
}

/// The receipt for a written opening view.
#[must_use]
pub fn lit(scheme: ThreeDLighting, framed: bool) -> String {
    let camera = if framed {
        "seen as its picture shows it"
    } else {
        "the camera left to the reader"
    };
    format!(
        "The model opens on the view \"Default\", {camera}, lit with {}.",
        lighting(scheme).to_lowercase()
    )
}

/// glTF textures carried into the PRC.
#[must_use]
pub fn gltf_textured(primitives: usize) -> String {
    format!("{primitives} textured surfaces keep their textures.")
}

/// A part's show switch tip.
#[must_use]
pub fn part_show_tooltip() -> &'static str {
    "Show or hide this part in this window. The file is not changed."
}

/// A part's name tip.
#[must_use]
pub fn part_highlight_tooltip() -> &'static str {
    "Click to highlight this part in this window; right-click for more."
}

/// Show only one part.
#[must_use]
pub fn part_isolate() -> &'static str {
    "Show only this part"
}

/// Show every part.
#[must_use]
pub fn parts_show_all() -> &'static str {
    "Show all parts"
}

/// Back to what the file stores.
#[must_use]
pub fn parts_as_stored() -> &'static str {
    "Show parts as the file stores them"
}

/// A part's markup (PMI) and view counts.
#[must_use]
pub fn part_extras(markups: usize, views: usize) -> String {
    match (markups, views) {
        (0, v) => format!("{v} views"),
        (m, 0) => format!("{m} markups"),
        (m, v) => format!("{m} markups, {v} views"),
    }
}

/// The part list's note that hiding is this window's only.
#[must_use]
pub fn parts_note() -> &'static str {
    "Showing and hiding parts changes this window, its pictures and saved pictures, not the file."
}
