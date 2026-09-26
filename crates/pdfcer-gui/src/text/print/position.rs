//! # `text::print::position` — where the page sits on the paper
//!
//! Operator request O208: *"when we are printing at a scale that will lose
//! content we have the option to drag the drawing to a new position on the
//! print page — that way we can choose what gets cropped."*
//!
//! Split out of the print catalog at R2's ceiling, and the seam is a real one:
//! every sentence here is about one quantity — a displacement from the
//! placement pdfcer chose — and the two conventions that quantity needs
//! stated before it means anything. See [`position_frame`] for the frame and
//! the sign, and [`position_extends_past`] for why the per-edge readout is
//! worded as geometry and never as loss.
//!
//! The implementation these words label is
//! [`crate::dialogs::print::position`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/print/position.md`.

/// The heading over the position controls.
#[must_use]
pub const fn position_heading() -> &'static str {
    "Position on the sheet"
}

/// Label for the horizontal displacement entry.
#[must_use]
pub const fn position_across() -> &'static str {
    "Across"
}

/// Label for the vertical displacement entry.
#[must_use]
pub const fn position_down() -> &'static str {
    "Down"
}

/// The suffix both displacement entries carry.
#[must_use]
pub const fn position_mm_suffix() -> &'static str {
    " mm"
}

/// **The frame the two numbers are measured in**, which is the half a
/// bare offset cannot state.
#[must_use]
pub const fn position_frame() -> &'static str {
    "Measured from where pdfcer places the page. Positive moves it right and down."
}

/// Put this page back where pdfcer placed it.
#[must_use]
pub const fn position_reset() -> &'static str {
    "Reset position"
}

/// Hover text for a Reset button on a page nobody has moved.
///
/// R9's distinction: greying is for *temporarily* unavailable, and it always
/// says why. A page at its planned position has nothing to reset.
#[must_use]
pub const fn position_reset_unmoved() -> &'static str {
    "This page is already where pdfcer placed it"
}

/// Centre the page on both axes.
#[must_use]
pub const fn position_centre() -> &'static str {
    "Centre"
}

/// Hover text for Centre, which says what it does to the crop.
#[must_use]
pub const fn position_centre_tooltip() -> &'static str {
    "Crops the drawing evenly on all four edges"
}

/// Centre the page left-to-right only, leaving the vertical position alone.
#[must_use]
pub const fn position_centre_horizontally() -> &'static str {
    "Centre horizontally"
}

/// Centre the page top-to-bottom only, leaving the horizontal position alone.
#[must_use]
pub const fn position_centre_vertically() -> &'static str {
    "Centre vertically"
}

/// Put **every** page back where pdfcer placed it.
#[must_use]
pub const fn position_reset_all() -> &'static str {
    "Reset all pages"
}

/// Hover text for Reset all pages when no page in the job has been moved.
#[must_use]
pub const fn position_reset_all_none() -> &'static str {
    "No page in this job has been moved"
}

/// How many pages of this job carry a position the operator chose.
#[must_use]
pub fn position_moved_count(pages: usize) -> String {
    if pages == 1 {
        "1 page in this job has been moved".to_owned()
    } else {
        format!("{pages} pages in this job have been moved")
    }
}

/// **How much of the page falls outside the printable area, edge
/// by edge.**
#[must_use]
pub fn position_extends_past(left_mm: i64, right_mm: i64, top_mm: i64, bottom_mm: i64) -> String {
    let mut edges: Vec<String> = Vec::new();
    for (name, amount) in [
        ("left", left_mm),
        ("right", right_mm),
        ("top", top_mm),
        ("bottom", bottom_mm),
    ] {
        if amount > 0 {
            edges.push(format!("{name} {amount} mm"));
        }
    }
    format!("Extends past the printable area — {}", edges.join(", "))
}

/// The same measurement when the page is wholly inside the printable area.
#[must_use]
pub const fn position_fits_entirely() -> &'static str {
    "The whole page falls inside the printable area"
}

/// How to move the page, said once, beside the controls that do it.
#[must_use]
pub const fn position_drag_hint() -> &'static str {
    "Drag the page in the preview to choose what gets cropped"
}

/// Which page the position controls are acting on.
#[must_use]
pub fn position_page_label(page_number: usize, page_size_pt: (f64, f64)) -> String {
    use crate::units::whole_mm_from_points as mm;
    format!(
        "Page {page_number} — {} × {} mm",
        mm(page_size_pt.0),
        mm(page_size_pt.1)
    )
}
