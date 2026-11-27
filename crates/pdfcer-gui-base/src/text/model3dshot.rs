//! Strings for the 3D viewer's options tabs and its Picture tab: the size,
//! resolution and finish *Save picture…* draws at. Consumed by
//! `pdfcer_gui::dialogs::model3d`.

/// The options menu's tabs, in order: the view, its look, moving about,
/// the saved picture, and measuring.
#[must_use]
pub fn tabs() -> [&'static str; 6] {
    ["View", "Look", "Moving", "Picture", "Measure", "Controls"]
}

/// The panorama switch.
pub const SPHERE: &str = "Panorama (photo sphere)";
/// Its hover.
pub const SPHERE_TOOLTIP: &str = "Save the view round the eye as a JPEG that photo viewers open as a picture to look around in: 360° across by 180° up and down is the whole sphere. It turns about the model's up, from where this view stands: walk into the model for a view from inside.";
/// The panorama's longer side drop-down's label.
pub const SPHERE_SIZE: &str = "Longer side";
/// Its hover.
pub const SPHERE_SIZE_TOOLTIP: &str =
    "The picture's longer side. The other follows the angles, so the pixels are square.";
/// Why a panorama cannot have a clear background.
pub const SPHERE_NOT_CLEAR: &str =
    "A panorama is a JPEG, which has no clear background: it is drawn on white.";

/// The longer sides, smallest first, as `dialogs::model3d::panorama` holds them.
#[must_use]
pub fn sphere_size(which: usize) -> &'static str {
    match which {
        0 => "2048 pixels",
        1 => "4096 pixels",
        _ => "8192 pixels",
    }
}

/// The angle across's label.
pub const PANO_ACROSS: &str = "Across";
/// Its hover.
pub const PANO_ACROSS_TOOLTIP: &str = "How far round the picture reaches, centred on where the view looks: 360° is the whole way round.";
/// The angle up and down's label.
pub const PANO_UP_DOWN: &str = "Up and down";
/// Its hover.
pub const PANO_UP_DOWN_TOOLTIP: &str =
    "How far up and down the picture reaches: 180° is from straight down to straight up.";
/// The switch taking the angle up and down from the view.
pub const PANO_FROM_VIEW: &str = "As the view";
/// Its hover.
pub const PANO_FROM_VIEW_TOOLTIP: &str = "Reach up and down as far as this view does on screen.";
/// The switch tilting the band with the view.
pub const PANO_TILT: &str = "Tilt with the view";
/// Its hover.
pub const PANO_TILT_TOOLTIP: &str = "Off: the horizon is the picture's middle row. On: the middle is where the view looks. Either way the picture turns level with the ground, so the horizon stays a straight line.";
/// The switch laying the band out cylindrically.
pub const PANO_CYLINDRICAL: &str = "Keep verticals straight (cylindrical)";
/// Its hover.
pub const PANO_CYLINDRICAL_TOOLTIP: &str = "Space the rows so upright edges stay straight and nothing near the horizon is squashed: suits a wide, short strip. It cannot reach straight up or down.";
/// Said when a cylindrical panorama is saved without a photo-sphere tag.
pub const PANO_UNTAGGED: &str = "A cylindrical picture is saved without the photo-sphere tag: photo viewers have no cylindrical type, so they show it as a flat picture.";
/// Said when the band was moved to stop at the top or bottom of the sphere.
#[must_use]
pub fn pano_at_pole(up: bool) -> &'static str {
    if up {
        "Tilted this far the picture would run past straight up, so it is moved down to stop there."
    } else {
        "Tilted this far the picture would run past straight down, so it is moved up to stop there."
    }
}
/// Where a tilted band's middle row looks.
#[must_use]
pub fn pano_centre(pitch: f64) -> String {
    match pitch {
        p if p > 0.05 => format!("Its middle row looks {p:.0}° above the horizon."),
        p if p < -0.05 => format!("Its middle row looks {:.0}° below the horizon.", -p),
        _ => "Its middle row is the horizon.".to_owned(),
    }
}
/// The panorama's size and angles.
#[must_use]
pub fn pano_size(w: u32, h: u32, across: f64, up_down: f64) -> String {
    format!("{w} × {h} pixels, {across:.0}° across by {up_down:.0}° up and down.")
}
/// Why the panorama cannot be drawn from this view.
#[must_use]
pub fn pano_refused(why: &str) -> String {
    format!("This view cannot be drawn as a panorama: {why}.")
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
