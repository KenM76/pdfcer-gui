//! Strings for the 3D viewer's options tabs and its Picture tab: the size,
//! resolution and finish *Save picture…* draws at. Consumed by
//! `pdfcer_gui::dialogs::model3d`.

/// The options menu's tabs, in order: the view, its look, moving about,
/// and the saved picture.
#[must_use]
pub fn tabs() -> [&'static str; 4] {
    ["View", "Look", "Moving", "Picture"]
}

/// The 360° picture switch.
pub const SPHERE: &str = "360° picture (photo sphere)";
/// Its hover.
pub const SPHERE_TOOLTIP: &str = "Save the whole view round the eye as a JPEG that photo viewers open as a picture to look around in. It is drawn level, from where this view stands: walk into the model for a view from inside.";
/// The 360° size drop-down's label.
pub const SPHERE_SIZE: &str = "360° size";
/// The 360° size drop-down's hover.
pub const SPHERE_SIZE_TOOLTIP: &str = "The picture's pixels. Its width is always twice its height, the whole way round the eye by half way over.";
/// Why a 360° picture cannot have a clear background.
pub const SPHERE_NOT_CLEAR: &str =
    "A 360° picture is a JPEG, which has no clear background: it is drawn on white.";

/// The 360° sizes, smallest first, as `dialogs::model3d::sphere` holds them.
#[must_use]
pub fn sphere_size(which: usize) -> &'static str {
    match which {
        0 => "2048 × 1024 pixels",
        1 => "4096 × 2048 pixels",
        _ => "8192 × 4096 pixels",
    }
}

/// The options menu's button writing every setting as the operator's own.
pub const SAVE_DEFAULTS: &str = "Save as default";
/// Its hover.
pub const SAVE_DEFAULTS_TOOLTIP: &str = "Open every 3D viewer with these settings, the eye height included. The Settings window's 3D viewer page changes them too.";
/// The options menu's button taking up the built-in settings.
pub const RESET_DEFAULTS: &str = "Reset to defaults";
/// Its hover.
pub const RESET_DEFAULTS_TOOLTIP: &str =
    "Use the built-in settings in this viewer. Your saved defaults are kept until you save again.";
/// Said after *Save as default*.
pub const DEFAULTS_SAVED: &str = "Saved as your defaults.";

/// The Settings window's 3D viewer page, in its list and over it.
pub const PAGE_TITLE: &str = "3D viewer";
/// What the page sets.
pub const PAGE_SILENCE: &str = "How every 3D viewer opens: the same settings its options menu has, saved there by Save as default.";
/// What it reaches.
pub const PAGE_RADIUS: &str =
    "A viewer already open keeps its own until it is opened again or reset.";
/// How a viewer starts: walking.
pub const START_WALKING: &str = "Open walking";
/// How a viewer starts: orbiting.
pub const START_ORBITING: &str = "Open orbiting";
/// The eye height left to the model's size.
pub const EYE_AUTO: &str = "Eye height from the model's size";
/// The eye height's unit.
pub const MM: &str = " mm";
/// A brightness's unit.
pub const PERCENT: &str = "%";
/// The render mode that follows each view.
pub const MODE_FROM_VIEW: &str = "As the view says";
/// The saved picture's scale on the page.
pub const PICTURE_SCALE: &str = "Picture size, times the screen";

/// One lighting scheme's brightness.
#[must_use]
pub fn brightness_of(scheme: &str) -> String {
    format!("Brightness: {scheme}")
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

/// The Picture tab's switch giving the picture its own lighting, shadows and
/// render mode.
#[must_use]
pub fn own_look() -> &'static str {
    "Its own look"
}

/// Its tip.
#[must_use]
pub fn own_look_tooltip() -> &'static str {
    "Light, shade and render the saved picture with its own choices, set below; they start at \
     the screen's. Off, it looks as the screen does."
}
