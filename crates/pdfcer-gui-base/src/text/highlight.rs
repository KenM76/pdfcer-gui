//! # `text::highlight` — the words of the highlighter's Tool Options block
//!
//! The block chooses what one highlighter drag draws; the status sentence is
//! the one a text highlight leaves when it drops lines outside the drag.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/highlight.md`.

/// The block's heading in Tool Options.
#[must_use]
pub const fn heading() -> &'static str {
    "Highlight"
}

/// The label before the shape choice.
#[must_use]
pub const fn draws() -> &'static str {
    "Draws"
}

/// The shape choices, in the order `canvas::markup::highlight::Shape::ALL`
/// lists them.
#[must_use]
pub const fn shapes() -> [&'static str; 5] {
    ["Text", "Area", "Ellipse", "Freehand", "Polygon"]
}

/// What each choice does, in the same order, shown on hover.
#[must_use]
pub const fn shape_hints() -> [&'static str; 5] {
    [
        "Follows the text you drag over; draws a box where there is none",
        "Draws a box, whatever is under it",
        "Draws a filled ellipse in the highlighter colour",
        "Draws a broad highlighter stroke where you drag",
        "Click each corner; double-click or press Enter to finish",
    ]
}

/// The line under the choice, for the two modes that draw a box.
#[must_use]
pub const fn alt_hint() -> &'static str {
    "Hold Alt while dragging to draw a box over text"
}

/// The line under the choice, for the polygon.
#[must_use]
pub const fn polygon_hint() -> &'static str {
    "Backspace removes the last corner; Esc abandons the polygon"
}

/// The status a text highlight leaves when it kept only the lines under the
/// drag. `dropped` is how many lines the sweep reached that it left out.
#[must_use]
pub fn clamped(dropped: usize) -> String {
    match dropped {
        1 => "Highlight: clamped to the lines under the drag (1 line left out)".to_owned(),
        n => format!("Highlight: clamped to the lines under the drag ({n} lines left out)"),
    }
}
