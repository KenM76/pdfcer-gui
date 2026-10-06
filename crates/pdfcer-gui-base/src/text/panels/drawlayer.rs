//! # `text::panels::drawlayer` — what the Layers panel says about the layer
//! new content goes on, and the receipt an add on it leaves
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/panels/drawlayer.md`.

/// Hover on a layer's name that is not the current layer.
#[must_use]
pub const fn choose_hover() -> &'static str {
    "Click to put the text, pictures, pasted objects and markups you add next on this layer."
}

/// Hover on the current layer's name and its pencil.
#[must_use]
pub const fn current_hover() -> &'static str {
    "New content goes on this layer. Click its name again to stop."
}

/// The note an add on the current layer leaves in the status line.
#[must_use]
pub fn receipt(layer: &str) -> String {
    format!("Put on layer \u{201c}{layer}\u{201d}.")
}

/// The note an SVG or EMF drawing placed as page content leaves while a
/// current layer is chosen: the engine places those on no layer.
#[must_use]
pub fn drawing_not_on_layer(layer: &str) -> String {
    format!(
        "This drawing was not put on layer \u{201c}{layer}\u{201d}: pdfcer cannot yet place an \
         SVG or EMF drawing on a layer. It is on no layer; use Move to layer in Properties."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_receipt_names_the_layer() {
        assert!(receipt("Walls").contains("\u{201c}Walls\u{201d}"));
    }

    #[test]
    fn the_current_hover_says_how_to_stop() {
        assert!(current_hover().contains("again to stop"));
    }
}
