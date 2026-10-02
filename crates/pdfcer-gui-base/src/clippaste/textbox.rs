//! Where another program's text lands when it is pasted with no text box
//! open: a box whose top-left corner is the pointer.
//!
//! Contract: all rectangles are PDF user space on the target page. The box is
//! [`WIDTH_PT`] wide, narrowed to the page's right edge, and slid left only
//! when less than [`MIN_WIDTH_PT`] would remain. [`content_box`] runs to the
//! page's bottom because the engine lays boxed text out from the top and
//! discloses an overflow; [`comment_box`] is sized to an estimate of the text,
//! because a comment's rectangle is what a reader draws.

use pdfcer_core::page_tree::Rect;

/// The wrap width a pasted text box takes when the page allows it.
pub const WIDTH_PT: f64 = 240.0;
/// The narrowest box worth wrapping into before sliding it left.
pub const MIN_WIDTH_PT: f64 = 72.0;
/// Line spacing as a multiple of the type size.
const LEADING: f64 = 1.2;
/// An average glyph's advance as a multiple of the type size.
const ADVANCE: f64 = 0.5;
/// The space a comment keeps between its border and its words.
const PAD_PT: f64 = 4.0;

/// Pasted text in the form both text routes type: line breaks as `\n`, a tab
/// as one space.
#[must_use]
pub fn normalise(pasted: &str) -> String {
    pasted
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\t', " ") // ui-text-exempt: a typed character, not prose
}

/// The box's left and right edges for a pointer at `x`.
fn span(x: f64, page: Rect) -> (f64, f64) {
    let narrowest = MIN_WIDTH_PT.min(page.width());
    let llx = x.clamp(page.llx, page.urx - narrowest);
    (llx, (llx + WIDTH_PT).min(page.urx))
}

/// The wrap box for page text pasted with the pointer at `at`.
#[must_use]
pub fn content_box(at: (f64, f64), page: Rect) -> Rect {
    let (llx, urx) = span(at.0, page);
    let lowest = MIN_WIDTH_PT.min(page.height());
    let top = at.1.clamp(page.lly + lowest, page.ury);
    Rect::from_corners(llx, page.lly, urx, top)
}

/// The rectangle for a comment holding `text` at `size_pt`, pasted with the
/// pointer at `at`; slid up if it would leave the page's bottom.
#[must_use]
pub fn comment_box(at: (f64, f64), text: &str, size_pt: f64, page: Rect) -> Rect {
    let (llx, urx) = span(at.0, page);
    let inner = (urx - llx - 2.0 * PAD_PT).max(size_pt);
    let per_line = (inner / (ADVANCE * size_pt)).floor().max(1.0);
    let lines: f64 = text
        .split('\n')
        .map(|p| {
            #[allow(
                clippy::cast_precision_loss,
                reason = "a character count, far below 2^52" // ui-text-exempt: lint reason
            )]
            let chars = p.chars().count() as f64;
            (chars / per_line).ceil().max(1.0)
        })
        .sum();
    let height = (lines * LEADING * size_pt + 2.0 * PAD_PT).min(page.height());
    let top = at.1.clamp(page.lly + height, page.ury);
    Rect::from_corners(llx, top - height, urx, top)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Rect {
        Rect::from_corners(0.0, 0.0, 612.0, 792.0)
    }

    #[test]
    fn line_breaks_of_every_platform_become_one_kind() {
        assert_eq!(normalise("a\r\nb\rc\nd\te"), "a\nb\nc\nd e");
    }

    #[test]
    fn a_content_box_hangs_from_the_pointer_to_the_page_bottom() {
        let r = content_box((100.0, 700.0), page());
        assert_eq!((r.llx, r.lly, r.urx, r.ury), (100.0, 0.0, 340.0, 700.0));
    }

    #[test]
    fn near_the_right_edge_the_box_narrows_then_slides() {
        let r = content_box((500.0, 700.0), page());
        assert_eq!((r.llx, r.urx), (500.0, 612.0));
        let r = content_box((600.0, 700.0), page());
        assert_eq!((r.llx, r.urx), (540.0, 612.0));
    }

    #[test]
    fn a_comment_box_grows_a_line_per_paragraph() {
        let one = comment_box((100.0, 700.0), "Hello", 11.0, page());
        let two = comment_box((100.0, 700.0), "Hello\nWorld", 11.0, page());
        assert!(
            (one.ury - 700.0).abs() < 1e-9,
            "its top is the pointer: {one:?}"
        );
        let step = (one.lly - two.lly) - LEADING * 11.0;
        assert!(
            step.abs() < 1e-9,
            "one more line is one more leading: {one:?} {two:?}"
        );
    }

    #[test]
    fn a_long_paragraph_wraps_into_more_lines() {
        let short = comment_box((100.0, 700.0), "word", 11.0, page());
        let long = comment_box((100.0, 700.0), &"word ".repeat(40), 11.0, page());
        assert!(
            long.height() > 3.0 * short.height() - 1.0,
            "{short:?} {long:?}"
        );
    }

    #[test]
    fn a_comment_near_the_bottom_is_slid_onto_the_page() {
        let r = comment_box((100.0, 5.0), "Hello\nWorld", 11.0, page());
        assert!(r.lly >= 0.0 && r.ury > 5.0, "{r:?}");
    }
}
