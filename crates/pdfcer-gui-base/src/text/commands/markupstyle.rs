//! # `text::commands::markupstyle` — the labels and tooltips of **Format ▸
//! Markup**, the six controls that restyle a mark that is already on the page
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/commands/markupstyle.md`.

use super::CommandText;

/// `format.colour` — the mark's **stroke** colour, `/C`.
#[must_use]
pub const fn format_colour() -> CommandText {
    CommandText::new(
        "Line colour",
        "Set the colour of the selected mark's outline. A mark drawn in CMYK or a spot colour \
         shows the default swatch instead, so its ink is not converted to screen colour behind \
         your back.",
    )
}

/// `format.fill` — the mark's **interior** colour, `/IC`.
#[must_use]
pub const fn format_fill() -> CommandText {
    CommandText::new(
        "Fill colour",
        "Fill the inside of the selected mark, for the shapes that have an interior. Marks are \
         drawn with no fill so that they do not hide the drawing underneath, and No fill puts \
         one back the way it was.",
    )
}

/// `format.line_width` — `/BS` `/W`, in points.
#[must_use]
pub const fn format_line_width() -> CommandText {
    CommandText::new(
        "Line width",
        "Set how thick the selected mark is drawn, in points. A thicker line needs more room, so \
         for every shape but a rectangle or an ellipse the mark's box grows with it.",
    )
}

/// `format.opacity` — `/CA`, shown as a percentage.
#[must_use]
pub const fn format_opacity() -> CommandText {
    CommandText::new(
        "Opacity",
        "Set how solid the selected mark is, from clear to fully opaque, so the drawing \
         underneath can show through it.",
    )
}

/// `format.arrowheads` — `/LE`, the pair of line endings. `/Line` only.
#[must_use]
pub const fn format_arrowheads() -> CommandText {
    CommandText::new(
        "Arrowheads",
        "Choose which ends of the selected line carry an arrowhead. Only a line or an arrow has \
         ends to put one on, so this is not offered for the other marks.",
    )
}

/// `format.line_style` — `/BS` `/S` and `/D`, the border's line style.
#[must_use]
pub const fn format_line_style() -> CommandText {
    CommandText::new(
        "Line style",
        "Draw the selected mark's outline solid or dashed. A dash the file already carries is \
         kept when you change anything else about the mark, so this is the only control that \
         changes it. Highlights, underlines and strikeouts have no outline, so it is not \
         offered for those.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The six hold the catalog's two copy rules — a label is a name and
    /// takes no trailing period, a tooltip is prose and ends in one.
    #[test]
    fn the_six_markup_style_strings_hold_the_catalog_conventions() {
        let all = [
            format_colour(),
            format_fill(),
            format_line_width(),
            format_opacity(),
            format_arrowheads(),
            format_line_style(),
        ];
        for t in all {
            assert!(!t.label.trim().is_empty(), "empty label: {t:?}");
            assert!(!t.tooltip.trim().is_empty(), "empty tooltip: {t:?}");
            assert!(
                t.tooltip.ends_with('.'),
                "a tooltip is prose and ends in a full stop: {:?}",
                t.tooltip
            );
            assert!(
                !t.label.ends_with('.'),
                "a label is a name and takes no trailing period: {:?}",
                t.label
            );
        }
        let mut labels: Vec<&str> = all.iter().map(|t| t.label).collect();
        let total = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), total, "two of these six share a label");
    }

    /// **None of the five reuses a label the Font group already carries.**
    #[test]
    fn no_markup_style_label_collides_with_the_font_group_on_the_same_tab() {
        let font = [
            super::super::format_font().label,
            super::super::format_font_size().label,
            super::super::format_font_colour().label,
            super::super::format_bold().label,
            super::super::format_italic().label,
        ];
        for t in [
            format_colour(),
            format_fill(),
            format_line_width(),
            format_opacity(),
            format_arrowheads(),
            format_line_style(),
        ] {
            assert!(
                !font.contains(&t.label),
                "`{}` is already a Format ▸ Font label, and the two groups are on one tab",
                t.label
            );
        }
    }
}
