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

/// The white-margin group's heading.
#[must_use]
pub const fn white_heading() -> &'static str {
    "Remove white margins"
}

/// Label before the margin kept around the ink.
#[must_use]
pub const fn white_keep() -> &'static str {
    "Keep around the ink"
}

/// The button that measures and crops.
#[must_use]
pub const fn white_button() -> &'static str {
    "Remove white margins"
}

/// The button's hover text: what it does and its three limits.
#[must_use]
pub const fn white_tooltip() -> &'static str {
    "Crops each page in scope to what it draws, plus the margin above, in one step. The \
     edge is found by drawing the page, to within half a point; a grey or tinted \
     background counts as drawing, so such a page may not shrink; and a page is never \
     cropped larger than it already shows."
}

/// The status sentence after Remove white margins.
#[must_use]
pub fn white_cropped(cropped: usize, blank: usize, failed: usize) -> String {
    let pages = |n: usize| {
        if n == 1 {
            "1 page".to_owned()
        } else {
            format!("{n} pages")
        }
    };
    let mut s = format!(
        "Cropped {} to what {} draw, found by drawing each page: the edge is within \
         half a point, and a tinted background counts as drawing.",
        pages(cropped),
        if cropped == 1 { "it" } else { "they" },
    );
    if blank > 0 {
        s.push_str(&format!(
            " {} drew nothing and were left alone.",
            pages(blank)
        ));
    }
    if failed > 0 {
        s.push_str(&format!(
            " {} could not be drawn and were left alone.",
            pages(failed)
        ));
    }
    s
}

/// Remove white margins found nothing to crop.
#[must_use]
pub fn white_nothing(blank: usize, failed: usize) -> String {
    match (blank, failed) {
        (_, 0) => "No page in scope draws anything, so nothing was cropped.".to_owned(),
        (0, n) => format!("No page in scope could be drawn ({n}), so nothing was cropped."),
        (b, n) => format!("Nothing was cropped: {b} drew nothing and {n} could not be drawn."),
    }
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
        assert!(white_cropped(3, 1, 0).contains("1 page drew nothing"));
        assert!(!white_cropped(1, 0, 0).contains("left alone"));
        assert!(white_nothing(2, 0).contains("draws anything"));
    }
}
