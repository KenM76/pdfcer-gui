//! # `text::page_crop` — the words for **a page box of a sheet**
//!
//! Pages ▸ Transform ▸ Crop…: its window and the disclosures its commit
//! raises. A `box_` index counts crop 0, bleed 1, trim 2, art 3. Design:
//! `docs/modules/pdfcer-gui/dialogs/page_crop.md`.

/// The box names, by index.
const BOX_NAMES: [&str; 4] = ["crop box", "bleed box", "trim box", "art box"];

/// Box `i`'s name in a sentence, e.g. "trim box".
#[must_use]
pub fn box_name(i: usize) -> &'static str {
    BOX_NAMES.get(i).copied().unwrap_or(BOX_NAMES[0])
}

/// Label before the box radios.
#[must_use]
pub const fn box_heading() -> &'static str {
    "Box"
}

/// Box `i`'s radio.
#[must_use]
pub fn box_label(i: usize) -> &'static str {
    ["Crop", "Bleed", "Trim", "Art"]
        .get(i)
        .copied()
        .unwrap_or("Crop")
}

/// Box `i`'s radio hover text: what the box is for.
#[must_use]
pub fn box_tooltip(i: usize) -> &'static str {
    match i {
        1 => "The bleed box: how far past the trim a print shop keeps the artwork.",
        2 => "The trim box: the finished page after the sheet is cut.",
        3 => "The art box: the extent of the page's meaningful content.",
        _ => "The crop box: what every reader shows and prints.",
    }
}

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Crop pages"
}

/// The standing rule under the box choice, for box `i`.
#[must_use]
pub const fn intro(i: usize) -> &'static str {
    match i {
        0 => {
            "This changes what is shown and printed, not the paper. Crop hides content \
             outside the box; it does not remove it, and Show whole sheet brings it back. \
             To remove it, use Redact outside the region."
        }
        _ => {
            "This box is read by print shops and production tools; readers do not show \
             it and it hides nothing. Without one of its own a page uses its crop box."
        }
    }
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

/// How many pages the commit will change; `crop` when it is the crop box.
#[must_use]
pub fn scope_count(n: usize, crop: bool) -> String {
    let done = if crop { "cropped" } else { "changed" };
    match n {
        1 => format!("1 page will be {done}."),
        n => format!("{n} pages will be {done}."),
    }
}

/// The range names no page of this document.
#[must_use]
pub const fn no_pages() -> &'static str {
    "No page of this document is named. Type page numbers such as 1-3, 7."
}

/// Heading above the four margin boxes; `crop` when it is the crop box.
#[must_use]
pub const fn margins_heading(crop: bool) -> &'static str {
    if crop {
        "Crop from each edge (mm)"
    } else {
        "In from each edge of the paper (mm)"
    }
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

/// Box `i`'s size after the commit, in whole millimetres from
/// `units::whole_mm_from_points`, as the sheet is seen.
#[must_use]
pub fn box_summary(i: usize, w_mm: i64, h_mm: i64) -> String {
    if i == 0 {
        format!("Visible area: {w_mm} × {h_mm} mm.")
    } else {
        format!("New {}: {w_mm} × {h_mm} mm.", box_name(i))
    }
}

/// The margins leave nothing; `crop` when it is the crop box.
#[must_use]
pub const fn nothing_left(crop: bool) -> &'static str {
    if crop {
        "These margins leave nothing visible. Make them smaller."
    } else {
        "These margins leave no box. Make them smaller."
    }
}

/// The commit button for box `i`.
#[must_use]
pub fn apply(i: usize) -> &'static str {
    ["Crop", "Set bleed box", "Set trim box", "Set art box"]
        .get(i)
        .copied()
        .unwrap_or("Crop")
}

/// Tooltip of [`apply`]; `crop` when it is the crop box.
#[must_use]
pub const fn apply_tooltip(crop: bool) -> &'static str {
    if crop {
        "Crop the pages counted above as one step, each from its own edges; Undo \
         reverses it."
    } else {
        "Set this box on the pages counted above as one step, each from its own \
         edges; Undo reverses it."
    }
}

/// The button that zeroes every margin of a bleed, trim or art box.
#[must_use]
pub const fn set_zero() -> &'static str {
    "Set to zero"
}

/// Tooltip of [`set_zero`].
#[must_use]
pub const fn set_zero_tooltip() -> &'static str {
    "Make the box the whole paper."
}

/// The button that removes box `i` from the pages.
#[must_use]
pub fn remove_box(i: usize) -> String {
    format!("Remove {}", box_name(i))
}

/// Tooltip of [`remove_box`].
#[must_use]
pub fn remove_box_tooltip(i: usize) -> String {
    format!(
        "Take the {} off the pages counted above, so each uses its crop box instead.",
        box_name(i)
    )
}

/// The switch that keeps all four margins the same.
#[must_use]
pub const fn equal() -> &'static str {
    "Same margin on every edge"
}

/// Tooltip of [`equal`].
#[must_use]
pub const fn equal_tooltip() -> &'static str {
    "Changing one margin changes all four to match."
}

/// The button that puts the margins back to the marked region.
#[must_use]
pub const fn back_to_region() -> &'static str {
    "Back to the marked region"
}

/// Tooltip of [`back_to_region`].
#[must_use]
pub const fn back_to_region_tooltip() -> &'static str {
    "Set the margins to the region this window was opened on."
}

/// The cancel button.
#[must_use]
pub const fn cancel() -> &'static str {
    "Cancel"
}

/// `n` sheets got box `i` hanging past their paper, so what counts is the
/// part on the sheet.
#[must_use]
pub fn disclosure_overhang(n: usize, i: usize) -> String {
    if i == 0 {
        return format!(
            "{n} {} cropped past the edge of the paper; what shows is the part on the paper.",
            if n == 1 { "sheet was" } else { "sheets were" }
        );
    }
    format!(
        "{n} {} a {} past the edge of the paper; what counts is the part on the paper.",
        if n == 1 { "sheet got" } else { "sheets got" },
        box_name(i)
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
        for i in 0..4 {
            for s in [
                window_title(),
                intro(i),
                margins_heading(i == 0),
                whole_sheet(),
                nothing_left(i == 0),
                apply(i),
                box_label(i),
                box_tooltip(i),
            ] {
                assert!(!s.trim().is_empty());
            }
        }
        assert!(scope_count(3, true).contains("3 pages will be cropped"));
        assert!(scope_count(1, false).contains("changed"));
        assert_eq!(scope_operands(1), "This page");
        assert!(disclosure_overhang(1, 0).contains("sheet was"));
        assert!(disclosure_overhang(2, 2).contains("trim box"));
        assert!(box_summary(2, 210, 297).contains("trim box: 210"));
        assert!(remove_box(1).contains("bleed box"));
        assert!(disclosure_inherited(2).contains("sheets"));
        assert!(white_cropped(3, 1, 0).contains("1 page drew nothing"));
        assert!(!white_cropped(1, 0, 0).contains("left alone"));
        assert!(white_nothing(2, 0).contains("draws anything"));
    }
}
