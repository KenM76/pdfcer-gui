//! # `text::panels::layeredit` — what the Layers panel says when it creates,
//! renames, changes or deletes a layer

use pdfcer_core::edit::{LayerDeleteOutcome, LayerIntent, LayerOutputState};

/// The name field beside the New layer button, when empty.
#[must_use]
pub const fn new_name_hint() -> &'static str {
    "New layer name"
}

/// The button that creates a layer.
#[must_use]
pub const fn new_layer() -> &'static str {
    "New layer"
}

/// Its tooltip: what it makes, and where the new layer goes.
#[must_use]
pub const fn new_layer_tooltip() -> &'static str {
    "Add an empty layer at the bottom of the list, shown when the document opens. Leave the name box empty to call it \u{201c}New layer\u{201d}."
}

/// The name a layer gets when the box is empty; `n` makes it unique.
#[must_use]
pub fn default_name(n: usize) -> String {
    if n <= 1 {
        "New layer".to_owned()
    } else {
        format!("New layer {n}")
    }
}

/// Row menu: open the properties window.
#[must_use]
pub const fn menu_properties() -> &'static str {
    "Layer properties\u{2026}"
}

/// Row menu: open the delete dialog.
#[must_use]
pub const fn menu_delete() -> &'static str {
    "Delete layer\u{2026}"
}

/// The properties window's title.
#[must_use]
pub fn properties_title(name: &str) -> String {
    format!("Layer \u{201c}{name}\u{201d}")
}

/// Field label: the layer's name.
#[must_use]
pub const fn field_name() -> &'static str {
    "Name"
}

/// Field label: shown when the document opens.
#[must_use]
pub const fn field_visible() -> &'static str {
    "Shown when the document opens"
}

/// Field label: locked against being shown or hidden in a reader.
#[must_use]
pub const fn field_locked() -> &'static str {
    "Locked (readers cannot show or hide it)"
}

/// Field label: when the layer prints.
#[must_use]
pub const fn field_print() -> &'static str {
    "Printing"
}

/// Field label: when the layer is exported.
#[must_use]
pub const fn field_export() -> &'static str {
    "Exporting"
}

/// Field label: the layer's intent.
#[must_use]
pub const fn field_intent() -> &'static str {
    "Purpose"
}

/// The choice that leaves a setting the panel cannot read as it is.
#[must_use]
pub const fn unchanged() -> &'static str {
    "Leave as it is"
}

/// Why print, export and purpose start at "Leave as it is".
#[must_use]
pub const fn unread_settings_tooltip() -> &'static str {
    "pdfcer cannot yet read this setting back from the file, so it starts at \u{201c}Leave as it is\u{201d}. Choosing anything else overwrites whatever the file holds."
}

/// The print and export choices the window offers, in menu order.
pub const OUTPUT_CHOICES: [(LayerOutputState, &str); 3] = [
    (LayerOutputState::WhenVisible, "Only when shown"),
    (LayerOutputState::Always, "Always"),
    (LayerOutputState::Never, "Never"),
];

/// The purpose choices the window offers, in menu order.
pub const INTENT_CHOICES: [(LayerIntent, &str); 3] = [
    (LayerIntent::View, "Viewing"),
    (LayerIntent::Design, "Design (hidden from ordinary viewing)"),
    (LayerIntent::Both, "Viewing and design"),
];

/// The properties window's apply button.
#[must_use]
pub const fn apply() -> &'static str {
    "Apply"
}

/// Cancel, in either window.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// The delete dialog's question.
#[must_use]
pub fn delete_question(name: &str) -> String {
    format!("Delete the layer \u{201c}{name}\u{201d}? What should happen to what it draws?")
}

/// Delete, keeping the drawing on no layer.
#[must_use]
pub const fn delete_keep() -> &'static str {
    "Keep the drawing, always shown"
}

/// Its tooltip.
#[must_use]
pub const fn delete_keep_tooltip() -> &'static str {
    "The layer goes; everything it drew stays on the page and is no longer on any layer, so it can no longer be hidden."
}

/// Delete, removing the drawing too.
#[must_use]
pub const fn delete_remove() -> &'static str {
    "Remove the drawing too"
}

/// Its tooltip.
#[must_use]
pub const fn delete_remove_tooltip() -> &'static str {
    "The layer and everything it drew are removed from every page. Ctrl+Z puts both back."
}

/// The disclosure after a layer was created.
#[must_use]
pub fn added(name: &str) -> String {
    format!("Added the layer \u{201c}{name}\u{201d}. It draws nothing yet.")
}

/// The disclosure after a layer delete, from the engine's counts.
#[must_use]
pub fn deleted(name: &str, removed: bool, o: &LayerDeleteOutcome) -> String {
    let places = o.sections + o.annotations + o.xobject_calls + o.paints;
    if places == 0 {
        return format!("Deleted the layer \u{201c}{name}\u{201d}. It drew nothing.");
    }
    let what = format!(
        "{} drawing section(s) in {} content stream(s), {} markup(s) and {} placed object(s)",
        o.sections,
        o.streams,
        o.annotations,
        o.xobject_calls + o.xobjects + o.paints
    );
    if removed {
        format!("Deleted the layer \u{201c}{name}\u{201d} and removed what it drew: {what}.")
    } else {
        format!(
            "Deleted the layer \u{201c}{name}\u{201d}. What it drew stays, now on no layer: {what}."
        )
    }
}

/// Why a layer edit wrote nothing — the status-bar decline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerRefusal {
    /// `EditError::EmptyLayerName`.
    EmptyName,
    /// `EditError::LayerNotFound`: the layer is not in the document's list.
    NotFound,
    /// `EditError::LayerInMembership`: a visibility rule names the layer.
    InMembership,
    /// `EditError::LayerHasWidget`: a form field's box is on the layer.
    HasWidget,
    /// `EditError::LayerContentNotRewritable`.
    ContentNotRewritable,
}

impl LayerRefusal {
    /// The sentence.
    #[must_use]
    pub const fn line(self) -> &'static str {
        match self {
            Self::EmptyName => "A layer needs a name, so nothing was changed.",
            Self::NotFound => {
                "That layer is no longer in the document's layer list, so nothing was changed."
            }
            Self::InMembership => {
                "Another rule in the document decides visibility using this layer, so deleting it would change what that rule shows. Nothing was deleted."
            }
            Self::HasWidget => {
                "A form field sits on this layer. Delete the field first, or keep the drawing when deleting the layer. Nothing was deleted."
            }
            Self::ContentNotRewritable => {
                "pdfcer could not rewrite a page that draws on this layer, so nothing was deleted."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_names_are_unique_and_the_first_is_plain() {
        assert_eq!(default_name(1), "New layer");
        assert_eq!(default_name(3), "New layer 3");
    }

    #[test]
    fn a_delete_that_drew_nothing_says_so() {
        let o = LayerDeleteOutcome::default();
        assert!(deleted("A", false, &o).contains("drew nothing"));
    }
}
