//! Strings for the 3D viewer's render mode, shadow-casting and highlight
//! choices, and for the lights a model carries. Consumed by
//! `pdfcer_gui::dialogs::model3d`.

/// The render mode drop-down's label.
#[must_use]
pub fn mode_label() -> &'static str {
    "Draw as"
}

/// The render mode item that follows the view on screen.
#[must_use]
pub fn mode_from_view(name: &str) -> String {
    format!("As the view says ({})", mode(name))
}

/// One render mode, by its `/Subtype` name (ISO 32000-1 Table 308).
#[must_use]
pub fn mode(name: &str) -> &'static str {
    match name {
        "Solid" => "Solid",
        "SolidWireframe" => "Solid with edges",
        "Transparent" => "Transparent",
        "TransparentWireframe" => "Transparent with edges",
        "BoundingBox" => "Bounding boxes",
        "TransparentBoundingBox" => "Transparent bounding boxes",
        "TransparentBoundingBoxOutline" => "Transparent bounding boxes with edges",
        "Wireframe" => "Wireframe",
        "ShadedWireframe" => "Shaded wireframe",
        "HiddenWireframe" => "Hidden-line wireframe",
        "Vertices" => "Vertices",
        "ShadedVertices" => "Shaded vertices",
        "Illustration" => "Illustration",
        "SolidOutline" => "Solid with outline",
        "ShadedIllustration" => "Shaded illustration",
        _ => "Another mode",
    }
}

/// The render mode drop-down's tip.
#[must_use]
pub fn mode_tooltip() -> &'static str {
    "How faces, edges and corners are drawn. A saved view can ask for one; picking one here overrides it until you pick \"As the view says\"."
}

/// The drop-down for which lights cast shadows.
#[must_use]
pub fn casters_label() -> &'static str {
    "Lights casting"
}

/// One choice of casters.
#[must_use]
pub fn casters(every: bool) -> &'static str {
    if every {
        "Every light"
    } else {
        "The brightest only"
    }
}

/// Its tip.
#[must_use]
pub fn casters_tooltip() -> &'static str {
    "With the lighting casting, every directional light can cast its own shadow, each darkening only that light's share; or only the brightest casts. A sun above the model is one light either way."
}

/// The drop-down for how see-through parts cast.
#[must_use]
pub fn alpha_label() -> &'static str {
    "See-through parts"
}

/// One way see-through parts cast.
#[must_use]
pub fn alpha(alpha: SeeThrough) -> &'static str {
    match alpha {
        SeeThrough::Tinted => "Cast a tinted, lighter shadow",
        SeeThrough::Cutout => "Cast fully where at least half opaque",
        SeeThrough::None => "Cast no shadow",
    }
}

/// How see-through parts cast, for [`alpha`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeeThrough {
    /// As light and as coloured as the part.
    Tinted,
    /// Fully where at least half opaque, else not at all.
    Cutout,
    /// Not at all.
    None,
}

/// Its tip.
#[must_use]
pub fn alpha_tooltip() -> &'static str {
    "How glass and other see-through parts cast: a shadow as light and as coloured as the part, a full shadow only where the part is mostly opaque, or none."
}

/// The lamp shadow switch.
#[must_use]
pub fn lamp_shadows() -> &'static str {
    "Lamps cast shadows"
}

/// Its tip.
#[must_use]
pub fn lamp_shadows_tooltip() -> &'static str {
    "The model's own point and spot lights cast shadows in every direction around them. A lamp inside a closed part, such as a bulb in its shade, is shadowed by it."
}

/// The camera light switch beside a model's own lights.
#[must_use]
pub fn headlight() -> &'static str {
    "Add a light at the camera"
}

/// Its tip.
#[must_use]
pub fn headlight_tooltip() -> &'static str {
    "Under Artwork lighting the model is lit by its own lights only. This adds a light at the camera, for a model whose lights leave the side facing you dark."
}

/// The highlight scale drop-down's label.
#[must_use]
pub fn shininess_label() -> &'static str {
    "Highlights read as"
}

/// One reading of stored shininess.
#[must_use]
pub fn shininess(scale: Highlights) -> &'static str {
    match scale {
        Highlights::Auto => "Whichever fits the value",
        Highlights::Exponent => "An exponent",
        Highlights::Fraction => "A fraction of 128",
    }
}

/// How stored shininess is read, for [`shininess`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlights {
    /// A fraction when at most 1, else an exponent.
    Auto,
    /// Always an exponent.
    Exponent,
    /// Always a fraction of 128.
    Fraction,
}

/// Its tip.
#[must_use]
pub fn shininess_tooltip() -> &'static str {
    "Model files store a material's shininess either as a 0-1 fraction or as an exponent, and do not say which. A value of 1 or less is read as a fraction unless you pick otherwise."
}

/// The lights a model carries, under the picture: `lamps` point or spot
/// lights, `directional` and `ambient` the rest, and whether they light
/// the picture (only Artwork lighting draws them).
#[must_use]
pub fn model_lights(lamps: usize, directional: usize, ambient: usize, drawn: bool) -> String {
    let mut kinds = Vec::new();
    if lamps > 0 {
        kinds.push(format!("{lamps} lamp(s)"));
    }
    if directional > 0 {
        kinds.push(format!("{directional} directional"));
    }
    if ambient > 0 {
        kinds.push(format!("{ambient} ambient"));
    }
    let lit = if drawn {
        "they light it."
    } else {
        "pick Artwork lighting to draw them."
    };
    format!(
        "The model carries its own lights ({}); {lit}",
        kinds.join(", ")
    )
}

/// Lights past the `limit` the picture draws.
#[must_use]
pub fn lights_dropped(count: usize, limit: usize) -> String {
    format!("{count} light(s) past the {limit} drawn are left out.")
}

/// Directional lights past the `limit` that cast shadows, lit without one.
#[must_use]
pub fn casters_dropped(count: usize, limit: usize) -> String {
    format!("{count} light(s) past the {limit} casting shadows light the model without one.")
}

/// Point and spot lights past the `limit` that cast shadows, lit without one.
#[must_use]
pub fn lamp_casters_dropped(count: usize, limit: usize) -> String {
    format!("{count} lamp(s) past the {limit} casting shadows light the model without one.")
}

/// The live picture is drawn by the processor, not the graphics card,
/// because it shows `what`.
#[must_use]
pub fn drawn_on_processor(what: &str) -> String {
    format!("Drawn without the graphics card while it shows {what}, so moving it is slower.")
}

/// What the graphics card does not draw, for [`drawn_on_processor`].
#[must_use]
pub fn processor_reason(reason: ProcessorReason) -> &'static str {
    match reason {
        ProcessorReason::WidePoints => "vertices larger than the card draws",
    }
}

/// Why the live picture is drawn by the processor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorReason {
    /// Points wider than the card's widest.
    WidePoints,
}
