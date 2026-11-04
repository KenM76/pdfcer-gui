//! Strings for the 3D viewer's options tabs and its Picture tab: the size,
//! resolution and finish *Save picture…* draws at. Consumed by
//! `pdfcer_gui::dialogs::model3d`.

/// The options menu's tabs, in order: the view, its look, moving about,
/// and the saved picture.
#[must_use]
pub fn tabs() -> [&'static str; 4] {
    ["View", "Look", "Moving", "Picture"]
}

/// What the Picture tab sets, above its rows.
#[must_use]
pub fn note() -> &'static str {
    "How Save picture… draws this view. The page's picture keeps its own size."
}

/// The size row's label.
#[must_use]
pub fn size_label() -> &'static str {
    "Size"
}

/// The unit after a width or height.
#[must_use]
pub fn pixels() -> &'static str {
    " px"
}

/// The width and height entries' tip.
#[must_use]
pub fn size_tooltip() -> &'static str {
    "The picture's width and height in pixels. It keeps the shape of the picture on screen, \
     so changing one changes the other."
}

/// The scale slider's label.
#[must_use]
pub fn scale_label() -> &'static str {
    "times the screen"
}

/// A scale, as the slider shows it.
#[must_use]
pub fn scale(times: f64) -> String {
    format!("{times:.2}×")
}

/// The scale slider's tip: what 1× is.
#[must_use]
pub fn scale_tooltip(width: u32, height: u32) -> String {
    format!(
        "1× is the picture as it is on screen, {width} × {height} pixels; 8× is eight times as wide and as tall."
    )
}

/// Said when the renderer's pixel ceiling stops the size short of 8×.
#[must_use]
pub fn limited(width: u32, height: u32) -> String {
    format!("Largest here: {width} × {height} pixels, as many as pdfcer draws in one picture.")
}

/// The resolution row's label.
#[must_use]
pub fn dpi_label() -> &'static str {
    "Resolution"
}

/// The unit after a resolution.
#[must_use]
pub fn dpi_unit() -> &'static str {
    " dpi"
}

/// The resolution entry's tip.
#[must_use]
pub fn dpi_tooltip() -> &'static str {
    "The pixels per inch written into the PNG, so a program that prints or places it uses that \
     size. The pixels are the same at any resolution."
}

/// Inches as millimetres, through the one conversion.
fn mm(inches: f64) -> f64 {
    crate::units::mm_from_points(crate::units::points_from_inches(inches))
}

/// The size the picture prints at, at its resolution.
#[must_use]
pub fn print_size(inches: [f64; 2]) -> String {
    let [w, h] = inches;
    format!(
        "Prints {w:.1} × {h:.1} in ({:.0} × {:.0} mm).",
        mm(w),
        mm(h)
    )
}

/// The background drop-down's label.
#[must_use]
pub fn background_label() -> &'static str {
    "Background"
}

/// One background: white, the screen's, or none.
#[must_use]
pub fn background(which: usize) -> &'static str {
    match which {
        0 => "White",
        1 => "As on screen",
        _ => "None (see-through)",
    }
}

/// The background drop-down's tip.
#[must_use]
pub fn background_tooltip() -> &'static str {
    "What is behind the model. None leaves it see-through, for placing the picture on \
     something else."
}

/// The shadow detail drop-down's label.
#[must_use]
pub fn detail_label() -> &'static str {
    "Shadow detail"
}

/// One shadow detail: the screen's, twice it, four times it.
#[must_use]
pub fn detail(which: usize) -> &'static str {
    match which {
        0 => "As on screen",
        1 => "Fine",
        _ => "Finest",
    }
}

/// The shadow detail drop-down's tip.
#[must_use]
pub fn detail_tooltip() -> &'static str {
    "Sharper shadow edges, for a large picture. Fine and Finest take longer to draw."
}

/// The switch that widens lines with the picture.
#[must_use]
pub fn wide_lines() -> &'static str {
    "Widen lines with the size"
}

/// Its tip.
#[must_use]
pub fn wide_lines_tooltip() -> &'static str {
    "Edges and wires grow with the picture, so a large picture looks like the screen. Off, \
     they stay one pixel wide."
}

/// Shown in place of *Save picture…* while the picture is drawn.
#[must_use]
pub fn drawing() -> &'static str {
    "Drawing the picture…"
}

/// Why a picture being drawn never arrived.
#[must_use]
pub fn lost() -> &'static str {
    "the drawing stopped before it finished"
}
