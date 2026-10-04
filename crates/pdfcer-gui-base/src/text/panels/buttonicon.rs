//! Words for a push button's icon rows in the Properties panel and for the
//! outcome of choosing its picture.

use pdfcer_core::annot_author::CaptionPosition;

/// The row's label.
#[must_use]
pub const fn label_icon() -> &'static str {
    "Picture"
}

/// The state shown beside the label when the button has no icon.
#[must_use]
pub const fn no_icon() -> &'static str {
    "None — the button shows its caption"
}

/// The state shown beside the label when the button has an icon.
#[must_use]
pub const fn has_icon() -> &'static str {
    "Set"
}

/// The button that opens the picker, with no icon yet.
#[must_use]
pub const fn choose() -> &'static str {
    "Choose picture…"
}

/// The same button, over an icon it would replace.
#[must_use]
pub const fn replace() -> &'static str {
    "Replace picture…"
}

/// The button that clears `/MK /I`.
#[must_use]
pub const fn remove() -> &'static str {
    "Remove picture"
}

/// The hover text on [`remove`].
#[must_use]
pub const fn remove_hint() -> &'static str {
    "The button goes back to showing its caption alone."
}

/// The caption-position combo's label.
#[must_use]
pub const fn label_caption_position() -> &'static str {
    "Caption"
}

/// One `/TP` layout, in the operator's words.
#[must_use]
pub const fn caption_position(position: CaptionPosition) -> &'static str {
    match position {
        CaptionPosition::CaptionOnly => "Caption only, no picture",
        CaptionPosition::IconOnly => "Picture only, no caption",
        CaptionPosition::CaptionBelow => "Below the picture",
        CaptionPosition::CaptionAbove => "Above the picture",
        CaptionPosition::CaptionRight => "Right of the picture",
        CaptionPosition::CaptionLeft => "Left of the picture",
        CaptionPosition::Overlaid => "Over the picture",
    }
}

/// What the operator touched, for a refused icon edit.
#[must_use]
pub const fn touched_icon() -> &'static str {
    "the button's picture"
}

/// What the operator touched, for a refused caption-position edit.
#[must_use]
pub const fn touched_caption_position() -> &'static str {
    "where the caption sits beside the picture"
}

/// A drawing chosen as a button icon: the engine's icon verb takes pixels.
#[must_use]
pub fn icon_not_a_drawing(kind: &str) -> String {
    format!(
        "A button's picture is made of pixels, and that file is an {} drawing. Nothing was changed — choose a PNG, JPEG, BMP, GIF or TIFF.",
        kind.to_ascii_uppercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Seven positions, seven distinct sentences: the combo shows each once.
    #[test]
    fn every_caption_position_has_its_own_words() {
        let mut all: Vec<&str> = (0..=6)
            .filter_map(CaptionPosition::from_tp)
            .map(caption_position)
            .collect();
        assert_eq!(all.len(), 7);
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), 7);
    }
}
