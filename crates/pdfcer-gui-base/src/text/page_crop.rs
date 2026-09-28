//! # `text::page_crop` — the words for **the visible area of a sheet**
//!
//! Pages ▸ Transform ▸ Crop…: its window and the disclosures its commit
//! raises. Design: `docs/modules/pdfcer-gui/dialogs/page_crop.md`.

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Crop pages"
}

/// The standing rule, first line of the window.
#[must_use]
pub const fn intro() -> &'static str {
    "This changes what is shown and printed, not the paper. Nothing is deleted: \
     the part outside the crop is hidden, and Show whole sheet brings it back."
}

/// Heading above the four margin boxes.
#[must_use]
pub const fn margins_heading() -> &'static str {
    "Crop from each edge (mm)"
}

/// Label of the left margin box.
#[must_use]
pub const fn left() -> &'static str {
    "Left"
}

/// Label of the right margin box.
#[must_use]
pub const fn right() -> &'static str {
    "Right"
}

/// Label of the top margin box.
#[must_use]
pub const fn top() -> &'static str {
    "Top"
}

/// Label of the bottom margin box.
#[must_use]
pub const fn bottom() -> &'static str {
    "Bottom"
}

/// The button that zeroes every margin.
#[must_use]
pub const fn whole_sheet() -> &'static str {
    "Show whole sheet"
}

/// Tooltip of [`whole_sheet`].
#[must_use]
pub const fn whole_sheet_tooltip() -> &'static str {
    "Remove the crop, so every reader shows the whole paper."
}

/// What will be visible after the commit, in whole millimetres from
/// `units::whole_mm_from_points`.
#[must_use]
pub fn visible_summary(w_mm: i64, h_mm: i64) -> String {
    format!("Visible area: {w_mm} × {h_mm} mm.")
}

/// The margins leave nothing to show.
#[must_use]
pub const fn nothing_left() -> &'static str {
    "These margins leave nothing visible. Make them smaller."
}

/// The picked sheets are different sizes, so only the whole sheet is offered.
#[must_use]
pub fn mixed_sizes(count: usize) -> String {
    format!(
        "These {count} sheets are different sizes, so one set of margins would crop \
         them differently. Pick sheets of one size to crop them, or show every sheet whole."
    )
}

/// The commit button.
#[must_use]
pub const fn apply() -> &'static str {
    "Crop"
}

/// Tooltip of [`apply`].
#[must_use]
pub const fn apply_tooltip() -> &'static str {
    "Crop the picked sheets as one step; Undo reverses it."
}

/// The cancel button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// `n` sheets were cropped by a rectangle hanging past their paper, so what
/// shows is the part on the sheet.
#[must_use]
pub fn disclosure_overhang(n: usize) -> String {
    format!(
        "{n} {} cropped past the edge of the paper; what shows is the part on the paper.",
        if n == 1 { "sheet was" } else { "sheets were" }
    )
}

/// `n` sheets had their own crop removed because the document's shared
/// setting already says the same thing.
#[must_use]
pub fn disclosure_inherited(n: usize) -> String {
    format!(
        "{n} {} now take their crop from the document's shared setting, which says the same.",
        if n == 1 { "sheet" } else { "sheets" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every sentence here reaches the operator; none may be empty.
    #[test]
    fn every_sentence_is_worded() {
        for s in [
            window_title(),
            intro(),
            margins_heading(),
            whole_sheet(),
            nothing_left(),
            apply(),
        ] {
            assert!(!s.trim().is_empty());
        }
        assert!(mixed_sizes(3).contains('3'));
        assert!(disclosure_overhang(1).contains("sheet was"));
        assert!(disclosure_inherited(2).contains("sheets"));
    }
}
