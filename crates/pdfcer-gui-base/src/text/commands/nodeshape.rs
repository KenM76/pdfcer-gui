//! # `text::commands::nodeshape` — the six labels of **Format ▸ Nodes**, the
//! controls that add a node to a page path and change the kind of its nodes
//! and segments
//!
//! The words are Inkscape's node toolbar's, the drawing program whose node
//! tool these six mirror. Each acts on every selected node.

use super::CommandText;

/// `format.node_insert`
#[must_use]
pub const fn format_node_insert() -> CommandText {
    CommandText::new(
        "Insert node",
        "Add a node halfway along the segment after each selected node. The shape does not change.",
    )
}

/// `format.node_corner`
#[must_use]
pub const fn format_node_corner() -> CommandText {
    CommandText::new(
        "Corner node",
        "Pull both handles of each selected node into it, so the line meets there at a sharp \
         point. A straight side is left alone.",
    )
}

/// `format.node_smooth`
#[must_use]
pub const fn format_node_smooth() -> CommandText {
    CommandText::new(
        "Smooth node",
        "Turn the two handles of each selected node onto one line, so the line passes through \
         it without a corner. Each handle keeps its length.",
    )
}

/// `format.node_symmetric`
#[must_use]
pub const fn format_node_symmetric() -> CommandText {
    CommandText::new(
        "Symmetric node",
        "As Smooth node, and give both handles the same length.",
    )
}

/// `format.segment_line`
#[must_use]
pub const fn format_segment_line() -> CommandText {
    CommandText::new(
        "Segment to line",
        "Make the segment after each selected node a straight line.",
    )
}

/// `format.segment_curve`
#[must_use]
pub const fn format_segment_curve() -> CommandText {
    CommandText::new(
        "Segment to curve",
        "Make the segment after each selected node a curve with handles you can drag. It keeps \
         its shape until you do.",
    )
}
