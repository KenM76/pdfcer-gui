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

/// The Rearrange section's heading.
#[must_use]
pub const fn rearrange_heading() -> &'static str {
    "Rearrange"
}

/// Rearrange button `index` (0‥5), as `(label, tooltip)`: Exchange by
/// selection order, stacking order and clockwise, then Randomize and Unclump.
#[must_use]
pub const fn rearrange_button(index: usize) -> (&'static str, &'static str) {
    match index {
        0 => (
            "Exchange",
            "Exchange positions of selected objects - selection order",
        ),
        1 => (
            "By stacking",
            "Exchange positions of selected objects - stacking order",
        ),
        2 => (
            "Clockwise",
            "Exchange positions of selected objects - rotate around center point",
        ),
        3 => ("Randomize", "Randomize centers in both dimensions"),
        _ => (
            "Unclump",
            "Unclump objects: try to equalize edge-to-edge distances",
        ),
    }
}

/// The Remove overlaps section's heading.
#[must_use]
pub const fn overlaps_heading() -> &'static str {
    "Remove overlaps"
}

/// The horizontal gap's label.
#[must_use]
pub const fn gap_h() -> &'static str {
    "H:"
}

/// The vertical gap's label.
#[must_use]
pub const fn gap_v() -> &'static str {
    "V:"
}

/// The unit after a gap.
#[must_use]
pub const fn pt_suffix() -> &'static str {
    " pt"
}

/// The Remove overlaps button, as `(label, tooltip)`.
#[must_use]
pub const fn remove_overlaps_button() -> (&'static str, &'static str) {
    (
        "Remove",
        "Move objects as little as possible so that their bounding boxes do not overlap",
    )
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

/// The panel's three tabs, as Inkscape names them.
#[must_use]
pub const fn tab(index: usize) -> &'static str {
    match index {
        0 => "Align",
        1 => "Grid",
        _ => "Circular",
    }
}

/// Grid: the rows and columns labels.
#[must_use]
pub const fn rows() -> &'static str {
    "Rows:"
}

/// Grid: the columns label.
#[must_use]
pub const fn columns() -> &'static str {
    "Columns:"
}

/// Grid: the columns field's hover, since rows decide the shape.
#[must_use]
pub const fn columns_tip() -> &'static str {
    "Follows from Rows and the number of objects selected, as in Inkscape."
}

/// Grid: equal-height rows.
#[must_use]
pub const fn equal_height() -> &'static str {
    "Equal height"
}

/// Grid: equal-width columns.
#[must_use]
pub const fn equal_width() -> &'static str {
    "Equal width"
}

/// The anchor-in-cell (Grid) or anchor-on-object (Circular) label.
#[must_use]
pub const fn anchor() -> &'static str {
    "Anchor:"
}

/// Horizontal anchor choice `i` (0‥3).
#[must_use]
pub const fn anchor_h(i: u8) -> &'static str {
    match i {
        0 => "Left",
        1 => "Centre",
        _ => "Right",
    }
}

/// Vertical anchor choice `i` (0‥3).
#[must_use]
pub const fn anchor_v(i: u8) -> &'static str {
    match i {
        0 => "Top",
        1 => "Middle",
        _ => "Bottom",
    }
}

/// Grid: fit the grid into the selection box.
#[must_use]
pub const fn fit_spacing() -> &'static str {
    "Fit into selection box"
}

/// Grid: set the gaps explicitly.
#[must_use]
pub const fn set_spacing() -> &'static str {
    "Set spacing:"
}

/// The Arrange button on the Grid and Circular tabs, as `(label, tooltip)`.
#[must_use]
pub const fn arrange_button(circular: bool) -> (&'static str, &'static str) {
    if circular {
        ("Arrange", "Arrange selected objects on an ellipse or arc")
    } else {
        ("Arrange", "Arrange selected objects in a table")
    }
}

/// Circular: where the ellipse comes from, choice `i` (0‥3).
#[must_use]
pub const fn circle_source(i: usize) -> &'static str {
    match i {
        0 => "Parameterized",
        1 => "First selected circle/ellipse",
        _ => "Last selected circle/ellipse",
    }
}

/// Circular: the centre fields' label.
#[must_use]
pub const fn centre_xy() -> &'static str {
    "Centre X/Y:"
}

/// Circular: the radius fields' label.
#[must_use]
pub const fn radius_xy() -> &'static str {
    "Radius X/Y:"
}

/// Circular: the angle fields' label.
#[must_use]
pub const fn angles() -> &'static str {
    "Angle start/end:"
}

/// Circular: the unit after an angle.
#[must_use]
pub const fn deg_suffix() -> &'static str {
    "°"
}

/// Circular: the anchor choice that means the object's centre.
#[must_use]
pub const fn anchor_centre() -> &'static str {
    "Object centre"
}

/// Circular: the anchor choice that means a point on the object's box.
#[must_use]
pub const fn anchor_box() -> &'static str {
    "Point on bounding box"
}

/// Circular: rotate each object to face out.
#[must_use]
pub const fn rotate_objects() -> &'static str {
    "Rotate objects"
}

/// Circular: the coordinate note under the fields.
#[must_use]
pub const fn circle_coords_note() -> &'static str {
    "Points from the sheet's top-left corner, y down; 0° points right, 90° down."
}

/// Said off-canvas when the reference circle/ellipse is not one.
#[must_use]
pub const fn not_an_ellipse(first: bool) -> &'static str {
    if first {
        "The first selected object is not a circle or ellipse, so there is nothing to arrange on. \
         Select the ellipse first, or use Parameterized."
    } else {
        "The last selected object is not a circle or ellipse, so there is nothing to arrange on. \
         Select the ellipse last, or use Parameterized."
    }
}

/// Node mode: how many anchors the node rows act on.
#[must_use]
pub fn nodes_selected(n: usize) -> String {
    if n == 1 {
        "1 node selected.".to_owned()
    } else {
        format!("{n} nodes selected.")
    }
}

/// Node mode: anchors selected on other paths, which the node rows leave.
#[must_use]
pub fn nodes_elsewhere(n: usize) -> String {
    format!(
        "{n} more selected on another path; the node rows move the anchors of the path you \
         entered first, one path at a time."
    )
}

/// Node mode: what the anchors line up with.
#[must_use]
pub const fn node_relative(rel: crate::alignlayout::rearrange::NodeRelative) -> &'static str {
    use crate::alignlayout::rearrange::NodeRelative;
    match rel {
        NodeRelative::Last => "Last selected",
        NodeRelative::First => "First selected",
        NodeRelative::Middle => "Middle of selection",
        NodeRelative::Min => "Min value",
        NodeRelative::Max => "Max value",
    }
}

/// Node mode's four buttons, as `(label, tooltip)`, Inkscape's wording.
#[must_use]
pub const fn node_button(index: usize) -> (&'static str, &'static str) {
    match index {
        0 => (
            "Vertical line",
            "Align selected nodes to a common vertical line",
        ),
        1 => (
            "Horizontal line",
            "Align selected nodes to a common horizontal line",
        ),
        2 => ("Spread across", "Distribute selected nodes horizontally"),
        _ => ("Spread down", "Distribute selected nodes vertically"),
    }
}

/// Node mode's heading.
#[must_use]
pub const fn nodes_heading() -> &'static str {
    "Nodes"
}

/// The on-canvas alignment toggle.
#[must_use]
pub const fn on_canvas() -> &'static str {
    "On-canvas alignment"
}

/// The on-canvas alignment toggle's tooltip.
#[must_use]
pub const fn on_canvas_tip() -> &'static str {
    "Show nine alignment handles inside the selection box while two or more objects are selected"
}

/// An on-canvas alignment handle's tooltip.
#[must_use]
pub const fn handle_tip() -> &'static str {
    "Click: align the selected objects to this side of the selection. Shift+click: align them just outside it."
}
