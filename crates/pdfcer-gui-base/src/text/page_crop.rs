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
    "This changes what is shown and printed, not the paper. Crop hides content \
     outside the box; it does not remove it, and Show whole sheet brings it back. \
     To remove it, use Redact outside the region."
}

/// Heading above the page choice.
#[must_use]
pub const fn scope_heading() -> &'static str {
    "Pages"
}

/// The pages the command was raised on.
#[must_use]
pub fn scope_operands(n: usize) -> String {
    if n == 1 {
        "This page".to_owned()
    } else {
        format!("The {n} picked pages")
    }
}

/// Every page.
#[must_use]
pub const fn scope_all() -> &'static str {
    "All pages"
}

/// Page numbers 1, 3, 5….
#[must_use]
pub const fn scope_odd() -> &'static str {
    "Odd pages"
}

/// Page numbers 2, 4, 6….
#[must_use]
pub const fn scope_even() -> &'static str {
    "Even pages"
}

/// A typed range.
#[must_use]
pub const fn scope_range() -> &'static str {
    "Pages:"
}

/// Placeholder in the range field.
#[must_use]
pub const fn range_hint() -> &'static str {
    "e.g. 1-3, 7"
}

/// How many pages the crop will change.
#[must_use]
pub fn scope_count(n: usize) -> String {
    match n {
        1 => "1 page will be cropped.".to_owned(),
        n => format!("{n} pages will be cropped."),
    }
}

/// The range names no page of this document.
#[must_use]
pub const fn no_pages() -> &'static str {
    "No page of this document is named. Type page numbers such as 1-3, 7."
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

/// The commit button.
#[must_use]
pub const fn apply() -> &'static str {
    "Crop"
}

/// Tooltip of [`apply`].
#[must_use]
pub const fn apply_tooltip() -> &'static str {
    "Crop the pages counted above as one step, each from its own edges; Undo \
     reverses it."
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
        assert!(scope_count(3).contains('3'));
        assert_eq!(scope_operands(1), "This page");
        assert!(disclosure_overhang(1).contains("sheet was"));
        assert!(disclosure_inherited(2).contains("sheets"));
    }
}
