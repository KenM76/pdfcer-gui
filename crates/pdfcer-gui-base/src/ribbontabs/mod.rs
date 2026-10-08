//! # `ribbontabs` — eight of the ribbon's tabs, as `egui_shell` values
//!
//! File, Pages, Edit, Markup, Measure, Security, Tools and the contextual Format tab,
//! with the item helpers every tab module writes its lists in and the
//! condition names and `Item::Custom` kinds those tabs emit. The app's
//! `shell::manifest::built_in` assembles them with the View tab, which names
//! an app action and so stays there. A renderer for each custom kind lives in
//! the app; `check-custom-kind-drawn.sh` holds the two sides together.

use egui_shell::manifest::{Group, Item, ItemSize};

pub mod edit;
pub mod file;
pub mod format;
pub mod markup;
pub mod measure;
pub mod pages;
pub mod security;
pub mod tools;

/// **The condition, published by the application each frame, under which an
/// OBJECT is selected on the page.**
pub const SELECTION_ANY: &str = "selection.any"; // ui-text-exempt: a condition name, never displayed

/// **Something Delete and Properties can act on** — wider than
/// [`SELECTION_ANY`], and named separately for the reason `app::conditions`
/// gives at the site that publishes it: a selected **form field** lives in
/// `doc.selected_field`, not in `SelectionState`, so `selection.any` is false
/// while one is selected.
pub const SELECTION_ACTIONABLE: &str = "selection.actionable"; // ui-text-exempt: a condition name, never displayed

/// **An Acrobat was found on this machine, or the operator has pointed
/// pdfcer at one** — the condition under which `file.open_in_acrobat` is
/// DRAWN AT ALL. `OPERATOR_REQUESTS.md` O122.
pub const ACROBAT_AVAILABLE: &str = "acrobat.available"; // ui-text-exempt: a condition name, never displayed

/// **The engine would not refuse a delete of what is selected** — the
/// condition under which `format.delete` is DRAWN AT ALL.
pub const DELETE_PERMITTED: &str = "selection.delete_permitted"; // ui-text-exempt: a condition name, never displayed

/// **The `Item::Custom` kinds of the Format ▸ Font controls.**
pub const FONT_FACE: &str = "font_face"; // ui-text-exempt: a custom-item kind, never displayed
/// See [`FONT_FACE`].
pub const FONT_SIZE: &str = "font_size"; // ui-text-exempt: a custom-item kind, never displayed
/// See [`FONT_FACE`].
pub const FONT_COLOUR: &str = "font_colour"; // ui-text-exempt: a custom-item kind, never displayed

/// **The `Item::Custom` kind of the Recent-documents control.**
pub const RECENT_FILES: &str = "recent_files"; // ui-text-exempt: a custom-item kind, never displayed

/// **The `Item::Custom` kind of the Markup ▸ Style controls.**
pub const COLOUR_SWATCH: &str = "colour_swatch"; // ui-text-exempt: a custom-item kind, never displayed

/// **The `Item::Custom` kind of the View ▸ Display OCR blend control.**
pub const OCR_BLEND: &str = "ocr_blend"; // ui-text-exempt: a custom-item kind, never displayed

/// **The `Item::Custom` kinds of the Format ▸ Markup controls** — the six
/// that restyle a mark already on the page.
pub const MARKUP_STROKE: &str = "markup_stroke"; // ui-text-exempt: a custom-item kind, never displayed
/// See [`MARKUP_STROKE`].
pub const MARKUP_FILL: &str = "markup_fill"; // ui-text-exempt: a custom-item kind, never displayed
/// See [`MARKUP_STROKE`].
pub const MARKUP_WIDTH: &str = "markup_width"; // ui-text-exempt: a custom-item kind, never displayed
/// See [`MARKUP_STROKE`].
pub const MARKUP_OPACITY: &str = "markup_opacity"; // ui-text-exempt: a custom-item kind, never displayed
/// See [`MARKUP_STROKE`].
pub const MARKUP_ENDINGS: &str = "markup_endings"; // ui-text-exempt: a custom-item kind, never displayed
/// See [`MARKUP_STROKE`].
pub const MARKUP_DASH: &str = "markup_dash"; // ui-text-exempt: a custom-item kind, never displayed

/// A captioned band of items.
pub fn group(id: &str, caption: &str, items: impl IntoIterator<Item = Item>) -> Group {
    Group::new(id, caption).with_items(items)
}

/// The same, laid out on **two rows** even when one would fit.
pub fn group_two_rows(id: &str, caption: &str, items: impl IntoIterator<Item = Item>) -> Group {
    group(id, caption, items).with_prefer_rows(2)
}

/// A command reference, by id.
///
/// Named `command` rather than used as `Item::command` so that a tab
/// module's item lists read as a list of commands, which is what they are.
pub fn command(id: &str) -> Item {
    Item::command(id)
}

/// A command drawn **icon-only** — `RIBBON_SCALING.md` §5.1.
pub fn icon_only(id: &str) -> Item {
    Item::command(id).sized(ItemSize::Small)
}

/// A command drawn **large** — icon above label, spanning the band's rows.
pub fn large(id: &str) -> Item {
    Item::command(id).sized(ItemSize::Large)
}
