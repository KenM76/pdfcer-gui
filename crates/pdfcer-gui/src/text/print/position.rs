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

/// The heading over the position controls.
///
/// It names the *sheet*, not the preview. The operator is not arranging a
/// picture on screen; they are choosing which part of an oversized drawing
/// reaches paper, and the preview is only how they see it. A heading reading
/// "Preview position" would describe the wrong thing and invite the reading
/// that this is a view control like the zoom beside it.
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
///
/// Whole millimetres are this dialog's unit everywhere — see
/// [`sheet_from_driver`] — but the entry keeps a decimal, because an
/// arrow-key nudge steps by a millimetre and a control whose number does not
/// move when the operator presses a key reads as a control that is not
/// listening.
#[must_use]
pub const fn position_mm_suffix() -> &'static str {
    " mm"
}

/// ★ **The frame the two numbers are measured in**, which is the half a
/// bare offset cannot state.
///
/// A displacement is meaningless without an origin, and this one's origin is
/// not the sheet corner — it is *wherever pdfcer put the page*, which is
/// centred for a page that fits and flush to the top-left corner for one that
/// does not. So `0, 0` does not mean "at the corner", it means "where pdfcer
/// chose", and that is the sentence the operator needs in order to read the
/// Reset button as anything other than a synonym for Centre.
///
/// The sign convention is stated for the same reason: positive-is-down is the
/// device's sense and the preview's, and it is the opposite of the sense a PDF
/// page uses, so leaving it to be inferred invites exactly one wrong guess.
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
///
/// ★ This is **not** the same command as [`position_reset`], and the whole
/// feature turns on the difference. pdfcer places an oversized page flush to
/// the top-left corner of the printable area, so that as little of it as
/// possible falls off the sheet. Reset returns to that corner; Centre moves it
/// to the middle, which crops the drawing evenly on all four edges. Both are
/// wanted, and an operator choosing what to lose off a big drawing wants the
/// second one far more often.
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
///
/// Its own control rather than a modifier on Reset, because the two have
/// different scopes and a job may have a hundred sheets: an operator who has
/// nudged nine drawings and wants the tenth back needs the narrow one, and an
/// operator who wants to start over needs the wide one. A single button that
/// did whichever the modifier key said would make the wide, unrecoverable act
/// the one nobody can see.
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
///
/// Drawn beside [`position_reset_all`] so the wide button's scope is visible
/// before it is pressed. One is the commonest count and reads oddly in a
/// plural, so it gets its own sentence.
#[must_use]
pub fn position_moved_count(pages: usize) -> String {
    if pages == 1 {
        "1 page in this job has been moved".to_owned()
    } else {
        format!("{pages} pages in this job have been moved")
    }
}

/// ★★ **How much of the page falls outside the printable area, edge
/// by edge.**
///
/// Operator request O208, his second clause: *"the hash lines we use to show
/// what won't be printed should have a line for each edge of the page."* The
/// hatch answers that on the picture; this answers it as a number, because a
/// hatched band tells an operator that something is over the edge and not by
/// how much — and "how much" is the quantity they are adjusting.
///
/// # Why this is worded as geometry and never as loss
///
/// It says the page *extends past* the printable area. It does not say content
/// will be lost, because on a 1:1 CAD drawing the overhang is usually empty
/// paper — which is the whole of operator request O113, and the ink verdict
/// beside it is what gets to make the claim about content. Two surfaces making
/// overlapping claims about the same risk is how a dialog comes to contradict
/// itself, so this one keeps to the measurement it can make honestly.
///
/// For the same reason it is never drawn in the warning colour. It is a
/// readout of a number the operator is steering, not an alarm.
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
///
/// A control with a line under it in every other state and a blank in one
/// reads as a control that failed, which is the same argument
/// [`paper_auto_nothing_to_measure`] makes.
#[must_use]
pub const fn position_fits_entirely() -> &'static str {
    "The whole page falls inside the printable area"
}

/// How to move the page, said once, beside the controls that do it.
///
/// The drag is the primary gesture and the buttons are the shortcuts, so the
/// gesture is what this names. It is here rather than only under the preview
/// because the preview can be popped into a window of its own, and a hint that
/// only exists on a surface the operator has moved elsewhere is a hint that is
/// not there.
#[must_use]
pub const fn position_drag_hint() -> &'static str {
    "Drag the page in the preview to choose what gets cropped"
}

/// Which page the position controls are acting on.
///
/// ★ The controls act on the sheet the preview is showing, not on "the
/// current page" of the document — the job may be a narrowed range,
/// odd/even filtered or reversed, so those two are different numbers. Naming
/// it removes the one ambiguity that would make a per-page setting
/// untrustworthy: an operator who cannot tell which page a button applies to
/// will not press it twice.
#[must_use]
pub fn position_page_label(page_number: usize, page_size_pt: (f64, f64)) -> String {
    use crate::units::whole_mm_from_points as mm;
    format!(
        "Page {page_number} — {} × {} mm",
        mm(page_size_pt.0),
        mm(page_size_pt.1)
    )
}
