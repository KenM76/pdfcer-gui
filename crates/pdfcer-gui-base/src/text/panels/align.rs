//! # `text::panels::align` — every string the Align and Distribute panel shows
//!
//! Tooltips follow Inkscape 1.4's wording, which is what an operator who knows
//! that panel will look for; labels are shortened to fit a dock column.

/// The panel's title on its tab.
#[must_use]
pub const fn title() -> &'static str {
    "Align and Distribute"
}

/// The Align section's heading.
#[must_use]
pub const fn align_heading() -> &'static str {
    "Align"
}

/// The Distribute section's heading.
#[must_use]
pub const fn distribute_heading() -> &'static str {
    "Distribute"
}

/// The group toggle.
#[must_use]
pub const fn as_group() -> &'static str {
    "Move/align selection as group"
}

/// The group toggle's hover.
#[must_use]
pub const fn as_group_tip() -> &'static str {
    "Treat the selection as one object: every object moves by the same amount, \
     so their arrangement is kept."
}

/// The Relative-to combo's label.
#[must_use]
pub const fn relative_to() -> &'static str {
    "Relative to:"
}

/// The row labels.
#[must_use]
pub const fn horizontal() -> &'static str {
    "Horizontal"
}

/// The row labels.
#[must_use]
pub const fn vertical() -> &'static str {
    "Vertical"
}

/// Align button `index` (0‥6) of the horizontal (`true`) or vertical row, as
/// `(label, tooltip)`.
#[must_use]
pub const fn align_button(horizontal: bool, index: usize) -> (&'static str, &'static str) {
    match (horizontal, index) {
        (true, 0) => (
            "Before",
            "Align right edges of objects to the left edge of anchor",
        ),
        (true, 1) => ("Left", "Align left edges"),
        (true, 2) => ("Centre", "Center on vertical axis"),
        (true, 3) => ("Right", "Align right edges"),
        (true, 4) => (
            "After",
            "Align left edges of objects to the right edge of anchor",
        ),
        (true, _) => ("Text", "Align baseline anchors of texts horizontally"),
        (false, 0) => (
            "Above",
            "Align bottom edges of objects to the top edge of anchor",
        ),
        (false, 1) => ("Top", "Align top edges"),
        (false, 2) => ("Middle", "Center on horizontal axis"),
        (false, 3) => ("Bottom", "Align bottom edges"),
        (false, 4) => (
            "Below",
            "Align top edges of objects to the bottom edge of anchor",
        ),
        (false, _) => ("Text", "Align baselines of texts"),
    }
}

/// Distribute button `index` (0‥5) of the horizontal (`true`) or vertical
/// row, as `(label, tooltip)`.
#[must_use]
pub const fn distribute_button(horizontal: bool, index: usize) -> (&'static str, &'static str) {
    match (horizontal, index) {
        (true, 0) => ("Left", "Distribute left edges equidistantly"),
        (true, 1) => ("Centres", "Distribute centers equidistantly horizontally"),
        (true, 2) => ("Right", "Distribute right edges equidistantly"),
        (true, 3) => ("Gaps", "Make horizontal gaps between objects equal"),
        (true, _) => ("Text", "Distribute baseline anchors of texts horizontally"),
        (false, 0) => ("Top", "Distribute top edges equidistantly"),
        (false, 1) => ("Middles", "Distribute centers equidistantly vertically"),
        (false, 2) => ("Bottom", "Distribute bottom edges equidistantly"),
        (false, 3) => ("Gaps", "Make vertical gaps between objects equal"),
        (false, _) => ("Text", "Distribute baselines of texts vertically"),
    }
}

/// Shown when nothing on this page is selected at the object level.
#[must_use]
pub const fn nothing_selected() -> &'static str {
    "Select objects on the page to align them. Click one, Shift-click the rest."
}

/// Shown when a markup or form field is selected instead of page content.
#[must_use]
pub const fn not_content() -> &'static str {
    "Align and Distribute moves page content. A markup or form field is \
     selected — there is nothing to align it against."
}

/// Hover on a greyed button: it needs more objects.
#[must_use]
pub const fn needs_two() -> &'static str {
    "Select at least two objects."
}

/// How many objects are selected, above the buttons.
#[must_use]
pub fn selected_count(n: usize) -> String {
    if n == 1 {
        "1 object selected.".to_owned()
    } else {
        format!("{n} objects selected.")
    }
}

/// Said off-canvas when a button moved nothing.
#[must_use]
pub const fn already_aligned() -> &'static str {
    "Nothing moved: the objects are already where that button would put them."
}

/// The *Relative to* list's words for `r`.
#[must_use]
pub fn relative_choice(r: crate::alignlayout::RelativeTo) -> &'static str {
    use crate::alignlayout::RelativeTo;
    match r {
        RelativeTo::Last => "Last selected",
        RelativeTo::First => "First selected",
        RelativeTo::Biggest => "Biggest object",
        RelativeTo::Smallest => "Smallest object",
        RelativeTo::Page => "Page",
        RelativeTo::Drawing => "Drawing",
        RelativeTo::Selection => "Selection area",
    }
}
