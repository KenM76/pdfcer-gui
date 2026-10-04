//! # `text::tool` — the words the tools say, wherever they are said
//!
//! ## This file OUTLIVED the panel it was written for
//!
//!
//! | what | who says it now |
//! |---|---|
//! | the per-tool instructions and live stages | `pdfcer_gui::app::toolstatus` — the right dock's permanent one-line strip |
//! | the second sentence of the stages that had one | the same strip, in its hover |
//! | the text pen's labels, the measure pick list, the resize switches | `pdfcer_gui::panels::properties::tool` |
//! | the disclosure heading | `pdfcer_gui::panels::properties::disclose` |
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/tool.md`.
use crate::editmodel::kind::TextEditKind;
use crate::markupkind::MarkupKind;
use crate::measure::kind::MeasureKind;
use crate::wordmarkup::TextAnnotKind;

// ===========================================================================
// What the pointer does right now — the resting tool's sentence
// ===========================================================================

/// What a press means in a mode that can select page content — Edit.
#[must_use]
pub const fn pointer_edit() -> &'static str {
    "Drag marquees objects on the page; click selects one. Hold Space to move \
     the paper."
}

/// What a press means in a mode that cannot select page content — Read and
/// Review.
#[must_use]
pub const fn pointer_reading() -> &'static str {
    "Drag selects text on the page; click puts the cursor in it. Hold Space to \
     move the paper."
}

// ===========================================================================
// Block B — the tools this mode has
// ===========================================================================

// ===========================================================================
// The armed frame
// ===========================================================================

/// The put-the-tool-down button.
#[must_use]
pub const fn put_down_button() -> &'static str {
    "Put this tool down"
}

/// The hint on that button, naming the key that does the same thing.
#[must_use]
pub const fn put_down_hint() -> &'static str {
    "Esc does the same."
}

/// What the **Node tool** does, in the Tool panel's live stage.
#[must_use]
pub const fn node_instruction() -> &'static str {
    "Click a shape to show its points. Click a point to select it, then drag to \
     move it. On a measurement you have drawn, drag a corner to reshape it. On \
     a block of text, clicking picks out just the line you clicked."
}

/// The line under it: how to take more than one, and how to change how many
/// corners a measurement has.
#[must_use]
pub const fn node_shift() -> &'static str {
    "Shift-click to add more points. A point on a curve also shows its handles. \
     On a measurement: Ctrl-drag a corner to add one after it, Ctrl+Shift-drag \
     to take it away."
}

/// The Hand tool's instruction.
#[must_use]
pub const fn hand_instruction() -> &'static str {
    "Drag to move the paper. Nothing on the page changes."
}

/// The Hand tool's second line — the borrow every other tool can do.
#[must_use]
pub const fn hand_borrow() -> &'static str {
    "Holding Space borrows this from any other tool, so you rarely need to arm it."
}

/// The Snapshot tool's instruction.
#[must_use]
pub const fn snapshot_instruction() -> &'static str {
    "Drag a box around the part of the page you want. Drag again to replace it."
}

/// The Snapshot tool's second line.
#[must_use]
pub const fn snapshot_stays() -> &'static str {
    "The box stays on the page while you zoom and scroll."
}

/// The text-sweep tool's instruction.
#[must_use]
pub const fn text_select_instruction() -> &'static str {
    "Drag across the words you want. Click once to put the cursor in a word."
}

/// What arming the sweep takes away, in the mode where it takes something.
#[must_use]
pub const fn text_select_takes_the_press() -> &'static str {
    "While this is armed a drag selects text instead of marqueeing objects."
}

/// One markup kind's instruction. The gesture, and how it ends.
#[must_use]
pub const fn markup_instruction(kind: MarkupKind) -> &'static str {
    match kind {
        MarkupKind::Rectangle => "Drag from one corner to the other.",
        MarkupKind::Ellipse => "Drag out the box it fits inside.",
        MarkupKind::Arrow => "Drag from the tail to the head.",
        MarkupKind::PolyLine | MarkupKind::Polygon | MarkupKind::Cloud => {
            "Click each corner in turn, and double-click the last one."
        }
        MarkupKind::Ink => "Press and draw. Let go when you are done.",
        MarkupKind::Highlight => "Drag across what you want marked.",
    }
}

/// How many corners are down, and what ends the run.
#[must_use]
pub fn vertices_placed(n: usize) -> String {
    match n {
        0 => "No corners placed yet.".to_owned(),
        1 => "1 corner placed. Double-click the last one to finish.".to_owned(),
        _ => format!("{n} corners placed. Double-click the last one to finish."),
    }
}

/// One text-annotation kind's instruction.
#[must_use]
pub const fn text_annot_instruction(kind: TextAnnotKind) -> &'static str {
    match kind {
        TextAnnotKind::TextBox => "Drag out the box, then type into it.",
        TextAnnotKind::Sticky => "Click where the note should sit, then type into it.",
        TextAnnotKind::Stamp => "Drag out the area the stamp should cover.",
        TextAnnotKind::Attachment => {
            "Click where the file's marker should sit, then choose the file."
        }
        TextAnnotKind::Caret => "Click where the words should go, then type them.",
        TextAnnotKind::Sound => {
            "Click where the sound's icon should sit, then choose the recording."
        }
    }
}

/// What a release does for a text-bearing annotation, which is NOT what it
/// does for a shape.
#[must_use]
pub const fn text_annot_release() -> &'static str {
    "Nothing is added to the page until you accept what you have typed."
}

/// Edit-text's instruction, before there is a caret.
#[must_use]
pub const fn text_edit_instruction(kind: TextEditKind) -> &'static str {
    match kind {
        TextEditKind::Edit => "Click a word already on the page to put the cursor in it.",
        TextEditKind::Add => "Click an empty spot to start typing new text there.",
    }
}

/// Edit-text's instruction while a caret is live.
#[must_use]
pub const fn text_edit_live() -> &'static str {
    "Enter commits what you have typed. Esc abandons it."
}

/// The heading over the refusal, when a click was declined.
#[must_use]
pub const fn refusal_heading() -> &'static str {
    "That click was declined"
}

/// The perimeter tool's LIVE sentence - vertices so far, and the running total
/// in the authoring group's own units.
#[must_use]
pub fn measure_perimeter_live(vertices: usize, length: &str) -> String {
    format!("{vertices} points so far, {length} around. Click the first point to close it.")
}

/// The area tool's live sentence: how many corners, and the area they close.
#[must_use]
pub fn measure_area_live(vertices: usize, area: &str) -> String {
    format!("{vertices} points so far, {area} enclosed. Click the first point to close it.")
}

/// The radius/diameter tool's LIVE sentence — how many points are in the fit,
/// and what circle they currently make.
#[must_use]
pub fn measure_circular_live(points: usize, measurement: &str) -> String {
    format!("{points} points, {measurement}. Add more, or finish it.")
}

/// The radius/diameter tool's sentence while the fit is still degenerate.
#[must_use]
pub fn measure_circular_needs_more(points: usize) -> String {
    match points {
        0 => "No points yet. Click around the arc — three or more.".to_owned(),
        1 => "1 point. Two more at least, spread around the arc.".to_owned(),
        n => format!("{n} points, and no circle through them yet — spread them around the arc."),
    }
}

/// The heading over the list of points in the circular fit.
#[must_use]
pub const fn measure_points_heading() -> &'static str {
    "Points in this measurement"
}

/// One row in that list: its position in the set, where it came from, and where
/// it is.
#[must_use]
pub fn measure_point_row(ordinal: usize, origin: &str, x: f64, y: f64) -> String {
    format!("{ordinal}. {origin} — {x:.1}, {y:.1}")
}

/// What a point's row does when it is clicked.
#[must_use]
pub const fn measure_point_remove_hint() -> &'static str {
    "Click to take this point out of the measurement"
}

/// The line drawn in place of the list when nothing has been picked.
#[must_use]
pub const fn measure_points_empty() -> &'static str {
    "Nothing picked yet."
}

/// The operator-facing name for where a picked point came from.
#[must_use]
pub const fn measure_point_origin(origin: crate::measure::pick::PickOrigin) -> &'static str {
    use pdfcer_core::vector::snap::SnapKind;

    use crate::measure::pick::PickOrigin;
    match origin {
        PickOrigin::Free => "Free position",
        PickOrigin::Snapped(kind) => match kind {
            SnapKind::Node => "Node",
            SnapKind::Endpoint => "Endpoint",
            SnapKind::Center => "Centre",
            SnapKind::Midpoint => "Midpoint",
            SnapKind::Intersection => "Intersection",
            SnapKind::SegmentCenterline => "On a line",
            SnapKind::DerivedCenterline => "Centreline",
            SnapKind::Axis => "Axis",
        },
    }
}

/// One measure kind's instruction, before any pick.
#[must_use]
pub const fn measure_instruction(kind: MeasureKind) -> &'static str {
    match kind {
        MeasureKind::Linear => {
            "Click the first point, then the second, then where the \
                                dimension line should sit."
        }
        MeasureKind::Circular => "Click three or more points around the arc, then finish it.",
        // All three endings, in one sentence, in the order an operator meets
        // them. A tool with three ways to stop needs to say so before the first
        // click - discovering the closing convention by accident works, and
        // discovering it AFTER tracing thirty vertices the wrong way does not.
        MeasureKind::Perimeter => {
            "Click around the shape. Click the first point again to close it, or double-click to finish an open path."
        }
        // Two endings, not three - and the sentence says so, because the
        // difference between this tool and Perimeter IS the missing ending.
        MeasureKind::PathLength => {
            "Click along what you are measuring. Double-click the last point to finish."
        }
        MeasureKind::Area => {
            "Click the corners of the region. Click the first point again, or double-click the last one, to close it."
        }
        MeasureKind::TwoLine => "Click one line, then the other.",
        // The calibration pick, which is armed from inside the Set-scale
        // window rather than from the Measure tab — it is deliberately absent
        // from `MeasureKind::ALL` for that reason.
        //
        // It gets its own sentence rather than borrowing Linear's, even though
        // it reuses `LinearPick` verbatim, because the two picks mean opposite
        // things: Linear AUTHORS a ce dimension onto the page and this one
        // authors nothing at all — it measures a length the operator is about
        // to tell pdfcer the real-world value of. An operator who read
        // "then where the dimension line should sit" would wait for a third
        // click that never comes.
        //
        MeasureKind::Scale => {
            "Click each end of something whose real length you know. \
             The Set-scale window comes back with your entries still in it."
        }
    }
}

/// The label over the group the next dimension will join.
#[must_use]
pub const fn draw_into_label() -> &'static str {
    "Drawing into"
}

/// The button that opens the panel which owns the group picker.
#[must_use]
pub const fn manage_groups_button() -> &'static str {
    "Groups…"
}

// ===========================================================================
// The Select tool — what rides along with a resize
// ===========================================================================

/// The heading over the Select tool's three scale switches.
#[must_use]
pub const fn scale_heading() -> &'static str {
    "When you resize something"
}

/// The stroke-width switch.
#[must_use]
pub const fn scale_stroke_label() -> &'static str {
    "Scale line weight"
}

/// The `/RD` switch.
#[must_use]
pub const fn scale_insets_label() -> &'static str {
    "Keep the inner margins the same size"
}

/// The distortion escape.
#[must_use]
pub const fn scale_distort_label() -> &'static str {
    "Allow the artwork to distort (borders may come out uneven)"
}

/// The note under the three switches.
#[must_use]
pub const fn scale_note() -> &'static str {
    "These apply to the next resize. Line weight stays put by default, because on a drawing it is a drafting standard rather than decoration — the same default Acrobat, Illustrator and Inkscape all ship."
}

// ===========================================================================
// The text pen — what NEW page text is written in
// ===========================================================================

/// The heading over the Add-text options.
#[must_use]
pub const fn text_pen_heading() -> &'static str {
    "New text"
}

/// The font combo's label.
#[must_use]
pub const fn text_pen_font_label() -> &'static str {
    "Font"
}

/// One bundled face's name, as an operator would say it.
#[must_use]
pub const fn text_pen_font_name(face: pdfcer_core::fontdata::Std14) -> &'static str {
    use pdfcer_core::fontdata::Std14 as F;
    match face {
        F::Helvetica => "Helvetica",
        F::HelveticaBold => "Helvetica Bold",
        F::HelveticaOblique => "Helvetica Oblique",
        F::HelveticaBoldOblique => "Helvetica Bold Oblique",
        F::TimesRoman => "Times Roman",
        F::TimesBold => "Times Bold",
        F::TimesItalic => "Times Italic",
        F::TimesBoldItalic => "Times Bold Italic",
        F::Courier => "Courier",
        F::CourierBold => "Courier Bold",
        F::CourierOblique => "Courier Oblique",
        F::CourierBoldOblique => "Courier Bold Oblique",
        F::Symbol => "Symbol",
        F::ZapfDingbats => "Zapf Dingbats",
        // NO wildcard, and its absence is deliberate. `Std14` is not
        // `#[non_exhaustive]` — checked, rather than assumed from its
        // neighbours in that module, several of which are — so this match is
        // exhaustive by the compiler's own count and a fifteenth face would be
        // a build error here rather than a combo entry reading "Another
        // bundled face". That is the stronger arrangement and it is available,
        // so it is taken.
    }
}

/// The size control's label.
#[must_use]
pub const fn text_pen_size_label() -> &'static str {
    "Size"
}

/// The colour swatch's label.
#[must_use]
pub const fn text_pen_colour_label() -> &'static str {
    "Colour"
}

/// The sentence under the three controls.
#[must_use]
pub const fn text_pen_note() -> &'static str {
    "These apply to the next text you add. They do not change text already on \
     the page — pdfcer cannot restyle a run it did not write."
}

// ===========================================================================
// Block C — what pdfcer last inferred
// ===========================================================================

/// The heading over the disclosures block.
#[must_use]
pub const fn disclosures_heading() -> &'static str {
    "What pdfcer worked out"
}

// ===========================================================================
// The empty case
// ===========================================================================

/// What the Tool panel says while a form-field tool is armed.
#[must_use]
pub const fn form_instruction() -> &'static str {
    "Click the page to place one at a standard size, or drag out the exact size you want."
}

/// The second line: what happens next, and what this kind needs.
#[must_use]
pub const fn form_kind_hint(kind: crate::formfieldkind::FormFieldKind) -> &'static str {
    use crate::formfieldkind::FormFieldKind as K;
    match kind {
        K::Radio => {
            "Nothing is added until you fill in the box that appears. Give buttons the same group name to make them alternatives."
        }
        _ => "Nothing is added until you fill in the box that appears. Escape cancels.",
    }
}

/// **The Points tool was asked for in a mode that cannot change page content.**
#[must_use]
pub fn node_tool_needs_edit_mode() -> &'static str {
    "Switch to Edit to work on points."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every markup kind has an instruction, and every instruction says how the
    /// gesture ends.
    #[test]
    fn every_markup_instruction_says_how_the_gesture_ends() {
        for kind in MarkupKind::ALL.iter().copied() {
            let s = markup_instruction(kind);
            assert!(!s.is_empty(), "{kind:?} has no instruction");
            let ends = s.contains("double-click")
                || s.contains("Let go")
                || s.contains("Drag from")
                || s.contains("Drag out")
                || s.contains("Drag across");
            assert!(
                ends,
                "{kind:?}'s instruction {s:?} never says how the gesture ends, so an \
                 operator following it has no way to know when to stop"
            );
        }
    }

    /// The corner count reads as English at one and at many.
    #[test]
    fn the_corner_count_reads_as_english() {
        assert!(vertices_placed(1).starts_with("1 corner placed"));
        assert!(vertices_placed(3).starts_with("3 corners placed"));
        assert!(!vertices_placed(0).contains('0'));
    }
}
