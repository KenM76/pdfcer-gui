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

/// The switch for lights that turn with the view, in the 3D viewer.
#[must_use]
pub fn lights_turn() -> &'static str {
    "Lights turn with the view"
}

/// Its tip.
#[must_use]
pub fn lights_turn_tooltip() -> &'static str {
    "On, the lights stay where you look from, so the side facing you is always lit. Off, they stay fixed to the model, so turning it shows its shaded sides. The PDF standard does not say which a reader does."
}

/// Which lighting draws the 3D viewer's picture and why: `file` when the
/// view on screen asks for it, `chosen` when picked in the viewer.
#[must_use]
pub fn lit_by(scheme: &str, file: bool, chosen: bool) -> String {
    if chosen {
        format!("Drawn with {scheme}, as you chose.")
    } else if file {
        format!("Drawn with {scheme}, as this view in the file asks.")
    } else {
        "Drawn lit from the camera; the file asks for no lighting.".to_owned()
    }
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

/// The shadows switch, in the 3D viewer.
#[must_use]
pub fn shadows() -> &'static str {
    "Shadows"
}

/// Its tip.
#[must_use]
pub fn shadows_tooltip() -> &'static str {
    "Lights cast shadows onto the model, so a surface facing away from a light gets none of it. Off, a surface is lit from both sides. The PDF standard defines no shadows, so another reader may draw this model without them."
}

/// The label of the drop-down for which light casts shadows.
#[must_use]
pub fn shadow_light_label() -> &'static str {
    "Shadows cast by"
}

/// One caster: `sun` a light fixed above the model, else the lighting's.
#[must_use]
pub fn shadow_light(sun: bool) -> &'static str {
    if sun {
        "A sun above the model"
    } else {
        "The lighting"
    }
}

/// Its tip.
#[must_use]
pub fn shadow_light_tooltip() -> &'static str {
    "A sun above the model stays where it is as you orbit and walk, so shadows keep their place on the model; above is the Up axis. The lighting casts from its brightest light on your side, or a light above and to the left of the view, so shadows turn as you move."
}

/// The ambient occlusion switch.
#[must_use]
pub fn occlusion() -> &'static str {
    "Shade corners and creases"
}

/// Its tip.
#[must_use]
pub fn occlusion_tooltip() -> &'static str {
    "Ambient occlusion: darkens creases, inside corners and where parts meet, as soft room light would. The PDF standard does not define it, so another reader may draw this model without it."
}

/// The label of the drop-down for which way a lighting scheme's
/// directions point.
#[must_use]
pub fn light_sense_label() -> &'static str {
    "Scheme lights point"
}

/// One way they point: `emitted` the way the light travels, else towards
/// the light.
#[must_use]
pub fn light_sense(emitted: bool) -> &'static str {
    if emitted {
        "The way the light travels"
    } else {
        "Towards the light"
    }
}

/// Its tip.
#[must_use]
pub fn light_sense_tooltip() -> &'static str {
    "The PDF standard does not say which way a lighting scheme's directions point. It changes only the side shadows fall on; the shading is the same either way."
}

/// Which light casts the viewer's shadows, for [`effects_said`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowCast {
    /// Shadows switched off, or nothing casts.
    None,
    /// The lighting's brightest light on the viewer's side.
    SchemeLight,
    /// A light above and to the left of the view.
    ViewKey,
    /// A sun fixed above the model.
    Sun,
}

/// What the viewer's picture shows of shadows and ambient occlusion:
/// which light casts, and `drawn` when either effect is drawn.
#[must_use]
pub fn effects_said(cast: ShadowCast, drawn: bool) -> String {
    if !drawn {
        return "This lighting is unlit, so neither shadows nor shaded corners are drawn."
            .to_owned();
    }
    let cast = match cast {
        ShadowCast::SchemeLight => {
            "Shadows fall from the lighting's lights, the brightest on your side first. "
        }
        ShadowCast::ViewKey => {
            "This lighting is all at the camera, so shadows fall from a light above and to the left of the view. "
        }
        ShadowCast::Sun => "Shadows fall from a sun fixed above the model. ",
        ShadowCast::None => "",
    };
    format!("{cast}Other PDF readers may not draw shadows or shaded corners.")
}
