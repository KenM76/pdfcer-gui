//! The graphics-card window's words. Consumed by
//! `pdfcer_gui::dialogs::cardswitches`.

use crate::cardstatus::Outcome;

/// The window's title.
#[must_use]
pub fn title() -> &'static str {
    "Graphics card"
}

/// The button in Settings ▸ Display that opens it.
#[must_use]
pub fn open_button() -> &'static str {
    "Every graphics-card switch, and what each did here…"
}

/// The button's tip.
#[must_use]
pub fn open_tooltip() -> &'static str {
    "One window with every part of pdfcer that can draw on the graphics card, its switch, and what it actually did on this computer this session."
}

/// The header's first line: the renderer and the card's largest texture.
#[must_use]
pub fn renderer(side: usize) -> String {
    format!("Drawn with OpenGL; the card's largest picture side is {side} pixels.")
}

/// The header's second line: whether the card has run out of memory.
#[must_use]
pub const fn memory(seen: bool) -> &'static str {
    if seen {
        "The graphics card has run out of memory this session; some pictures may be missing until pdfcer is restarted."
    } else {
        "The graphics card has not run out of memory this session."
    }
}

/// The grid's column headings.
#[must_use]
pub fn columns() -> [&'static str; 2] {
    ["Use the graphics card for", "What it did on this computer"]
}

/// The page view's row.
#[must_use]
pub fn pages() -> &'static str {
    "Pages in the page view"
}

/// The 3D viewer's whole-picture row.
#[must_use]
pub fn picture() -> &'static str {
    "The 3D viewer's picture"
}

/// The bounce-light row.
#[must_use]
pub fn bounce() -> &'static str {
    "3D bounce light"
}

/// The Measure light row.
#[must_use]
pub fn meter() -> &'static str {
    "3D Measure light"
}

/// The light map row.
#[must_use]
pub fn light_map() -> &'static str {
    "3D Colour by light level"
}

/// The rows with no switch: they follow the 3D picture's.
#[must_use]
pub fn follows_picture() -> &'static str {
    "Follows the 3D viewer's picture switch; no switch of its own."
}

/// The closing line: what never uses the card.
#[must_use]
pub fn processor_only() -> &'static str {
    "Always on the processor: thumbnails, printing, every export, saved 3D pictures, and the picture of Colour by light level (its figures are measured on the card)."
}

/// The tip on the 3D rows: where else they are set, and what changing one here does.
#[must_use]
pub fn model_tooltip() -> &'static str {
    "The same switch as the 3D viewer's Look tab. Changing it here changes the open viewer and the default every new viewer opens with."
}

/// The tip on the pages row.
#[must_use]
pub fn pages_tooltip() -> &'static str {
    "The same switch as Settings > Display > Draw pages on the graphics card, saved at once."
}

/// The tip on a 3D part greyed because the whole picture is off.
#[must_use]
pub fn part_greyed() -> &'static str {
    "The 3D viewer's picture is drawn by the processor, so every part is."
}

/// A status cell.
#[must_use]
pub fn outcome(o: &Outcome) -> String {
    match o {
        Outcome::Untried => "Not tried this session".into(),
        Outcome::InUse => "Drawing on the graphics card".into(),
        Outcome::Processor(why) => format!("On the processor: {why}"),
        Outcome::Limit(what) => format!("On the graphics card, within its limit: {what}"),
        Outcome::ForcedOff(var) => format!("On the processor: {var}=1 is set"),
        Outcome::Off => "Switched off; on the processor".into(),
    }
}

/// The picture's reason while Colour by light level is shown.
#[must_use]
pub fn light_map_shown() -> &'static str {
    "Colour by light level is shown, which only the processor draws"
}

/// The bounce build's reason when the card cannot run it.
#[must_use]
pub fn bounce_unsupported() -> &'static str {
    "this graphics card cannot build it"
}

/// Measure light's reason when the viewer is not drawing on the card.
#[must_use]
pub fn meter_not_drawing() -> &'static str {
    "the 3D viewer is not drawing on the graphics card"
}

/// Measure light's reason when the card could not run the build.
#[must_use]
pub fn meter_refused() -> &'static str {
    "the graphics card could not build it"
}

/// Measure light's reason when the card did not finish in time.
#[must_use]
pub fn meter_timeout(seconds: u64) -> String {
    format!("the graphics card had not finished it in {seconds} s")
}

/// The light map's reason when its points need a texture wider than the card's largest.
#[must_use]
pub fn map_too_large() -> &'static str {
    "too many points for the card's largest picture"
}

/// A part's reason when the frame shows it with its switch off.
#[must_use]
pub fn part_refused() -> &'static str {
    "switched off, and the picture shows it"
}

/// Lines and points: points wider than the card draws.
#[must_use]
pub fn wide_points(widest: f32) -> String {
    format!("points wider than the card's widest ({widest:.0} px)")
}
