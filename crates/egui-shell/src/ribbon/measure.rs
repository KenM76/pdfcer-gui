//! # `ribbon::measure` — the primitives every ribbon row measures with
//!
//! The seam [`super::control`]'s header describes from the other side:
//! [`super::band`] answers *"how are the groups arranged?"*, and everything
//! here answers a question that has nothing to do with groups at all — **"how
//! wide is this, in this theme, in this font?"**
//!
//! ## Why these belong together, and why none of them belongs to the band
//!
//! Not one of them is about the band. They are used by:
//!
//! | caller | what it measures |
//! |---|---|
//! | [`super::band`] | a group's width, and the rule between two groups |
//! | [`super::strip`] | the tab row and its overflow affordance |
//! | [`super::qat`] | the quick-access toolbar and its own `…` |
//! | [`super::tabs`] | one tab button |
//! | [`super::sizing`] | one control, at each of the three sizes |
//! | [`super::collapsed`] | a collapsed group's button |
//! | [`crate::menu::render`] | a menu row — a different surface entirely |
//!
//! **That list is the whole argument for one copy.** Every row of the
//! ribbon plans its own width, and a row that measured a button differently
//! from the row above it would disagree about whether the window is wide
//! enough — which is not a wrong number, it is *two* numbers, and the
//! symptom is a control drawn outside its own rectangle on top of its
//! neighbour. A file named after any one of those callers would be the wrong
//! home for it.
//!
//! ## What is deliberately NOT here
//!
//! The band's **vertical** geometry — `CAPTION_GAP`, `BAND_PADDING_BOTTOM`,
//! `rows_height`, `band_height`, `GroupBox`. Those really are facts about the
//! band: R128 makes the band's height a promise to the canvas below it, and
//! that promise belongs in the file that keeps it.
//!
//! Design and rationale: `docs/modules/egui-shell/ribbon/measure.md`.

use egui::TextStyle;

/// The horizontal padding `egui` will add inside a button, both sides.
pub(crate) fn button_padding(ui: &egui::Ui) -> f32 {
    ui.spacing().button_padding.x * 2.0
}

/// **The narrowest an `egui::Button` can be drawn — the floor `truncate()`
/// cannot go below.**
///
/// # Why this number decides the whole tab-strip row
///
/// `Button::truncate()` shortens a label to the room available, which
/// sounds like it can shrink to nothing and cannot. `egui` lays a
/// truncated label out as *the ellipsis* plus the button's own padding,
/// and stops there. Measured against the synthetic face of
/// [`super::testfont`], asking a `"Save a copy…"` button to lay itself out
/// in rooms from 0 to 80 pt:
///
/// ```text
/// room     0     2     6    10    14    20    26    40    80
/// width  19.7  19.7  19.7  19.7  19.7  19.7  19.7  34.7  74.7
///        └──────────── the floor ────────────┘ └─ room − 5.3 ─┘
/// ```
///
/// 19.6875 = 4 + 4 of `button_padding` plus 11.6875 of `…`. Below about
/// 25 pt of room the button simply **overflows the space it was given**,
/// silently, because `egui` does not clip children to a `Ui`'s `max_rect`.
///
/// The consequence is the one rule the tab-strip row is built on: a region
/// gets either **at least this much width, or none at all**. Granting a
/// sliver produces a control drawn outside its own rectangle, on top of
/// its neighbour — which is exactly the class of defect
/// [`super::strip`] exists to retire, arrived at by trying to be
/// accommodating. See [`super::plan::plan_strip_row`], which takes this as
/// its `button_floor`.
///
/// Measured from the live style rather than written down as a constant,
/// because both terms are theme- and font-dependent: `button_padding` is
/// the theme's, and the ellipsis's advance is the face's.
pub(crate) fn min_button_width(ui: &egui::Ui) -> f32 {
    button_padding(ui) + text_width(ui, "…", &TextStyle::Button)
}

/// The space a `ui.separator()` allocates for itself in a horizontal
/// layout, excluding the layout gaps around it.
pub(crate) const SEPARATOR_LINE: f32 = 6.0;

/// The full cost of putting a `ui.separator()` **between two things** in a
/// horizontal layout: its own width plus the `item_spacing` `egui` puts on
/// each side of it.
pub(crate) fn separator_width(ui: &egui::Ui) -> f32 {
    SEPARATOR_LINE + ui.spacing().item_spacing.x * 2.0
}

/// Measure a string in the font `egui` will draw it in.
pub(crate) fn text_width(ui: &egui::Ui, text: &str, style: &TextStyle) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let font_id = style.resolve(ui.style());
    ui.ctx().fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font_id, egui::Color32::PLACEHOLDER)
            .size()
            .x
    })
}
