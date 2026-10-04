//! # `text::panels::layerassign` — what Properties' Layer row and the Move to
//! layer window say, and the receipt for a move

/// The Properties row's label.
#[must_use]
pub const fn heading() -> &'static str {
    "Layer"
}

/// The choice that takes the selection off every layer.
#[must_use]
pub const fn no_layer() -> &'static str {
    "No layer"
}

/// The combo's text when the selection is on several layers.
#[must_use]
pub const fn mixed() -> &'static str {
    "Several layers"
}

/// The combo's text when the selection is under a visibility rule rather
/// than on one layer, or its layer cannot be read.
#[must_use]
pub const fn other() -> &'static str {
    "A visibility rule"
}

/// The combo's tooltip.
#[must_use]
pub const fn tooltip() -> &'static str {
    "The layer the selection shows, prints and exports with. Choosing another moves it there; Undo moves it back."
}

/// Why the combo is greyed for a selection of parts.
#[must_use]
pub const fn parts_tooltip() -> &'static str {
    "A part of an object is always on the object's layer. Select the whole object to move it."
}

/// Why the combo is greyed for a selection across pages.
#[must_use]
pub const fn pages_tooltip() -> &'static str {
    "The selection is on more than one page. Move one page's objects at a time."
}

/// The Move to layer window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Move to layer"
}

/// The label before the window's combo.
#[must_use]
pub const fn window_to() -> &'static str {
    "Move to"
}

/// The combo's text before a choice is made.
#[must_use]
pub const fn window_pick() -> &'static str {
    "Choose a layer"
}

/// The window's confirming button.
#[must_use]
pub const fn window_go() -> &'static str {
    "Move"
}

/// The window's cancel button.
#[must_use]
pub const fn window_cancel() -> &'static str {
    "Cancel"
}

/// Where the selection went: a layer's name, or no layer.
fn destination(layer: Option<&str>) -> String {
    layer.map_or_else(
        || "off every layer".to_owned(),
        |name| format!("to layer \u{201c}{name}\u{201d}"),
    )
}

/// The receipt for page content: `moved` objects went, `unchanged` were
/// already there.
#[must_use]
pub fn objects_moved(moved: usize, unchanged: usize, layer: Option<&str>) -> String {
    let to = destination(layer);
    let noun = if moved == 1 { "object" } else { "objects" };
    match (moved, unchanged) {
        (0, _) => format!("The selection is already {}; nothing was moved.", at(layer)),
        (_, 0) => format!("Moved {moved} {noun} {to}."),
        _ => format!("Moved {moved} {noun} {to}; {unchanged} already there were left alone."),
    }
}

/// The receipt for an annotation or widget. `replaced_rule` when it was under
/// a visibility rule that the move replaced.
#[must_use]
pub fn annotation_moved(changed: bool, layer: Option<&str>, replaced_rule: bool) -> String {
    if !changed {
        return format!("It is already {}; nothing was moved.", at(layer));
    }
    let mut out = format!("Moved it {}.", destination(layer));
    if replaced_rule {
        out.push_str(" It was shown by a visibility rule naming several layers; that rule no longer applies to it.");
    }
    out
}

/// Where the selection already is, for the nothing-moved receipt.
fn at(layer: Option<&str>) -> String {
    layer.map_or_else(
        || "on no layer".to_owned(),
        |name| format!("on layer \u{201c}{name}\u{201d}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipts_name_the_count_and_the_layer() {
        assert_eq!(
            objects_moved(2, 0, Some("Welds")),
            "Moved 2 objects to layer \u{201c}Welds\u{201d}."
        );
        assert_eq!(
            objects_moved(0, 1, None),
            "The selection is already on no layer; nothing was moved."
        );
        assert!(annotation_moved(true, Some("A"), true).contains("visibility rule"));
    }
}
