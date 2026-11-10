//! # `text::region` — the words of the snapshot region's commands
//!
//! The region is the snapshot box: one rectangle on one page. These strings
//! belong to the Tool Options block that offers its commands, to the status
//! sentences those commands leave, and to the Redact outside window.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/region.md`.

/// The block's heading in Tool Options.
#[must_use]
pub const fn heading() -> &'static str {
    "Region"
}

/// The chip: where the region is, how big, and how many objects it holds.
/// `objects` is `None` when the region is on a page other than the one on
/// screen, whose objects are not loaded.
#[must_use]
pub fn chip(page_index: usize, w_pt: f64, h_pt: f64, objects: Option<usize>) -> String {
    use crate::units::{inches_from_points, whole_mm_from_points};
    let (w_in, h_in) = (inches_from_points(w_pt), inches_from_points(h_pt));
    let (w_mm, h_mm) = (whole_mm_from_points(w_pt), whole_mm_from_points(h_pt));
    let page = page_index + 1;
    let size = format!("{w_in:.1} \u{d7} {h_in:.1} in ({w_mm} \u{d7} {h_mm} mm)");
    match objects {
        Some(1) => format!("Region: page {page}, {size}, 1 object"),
        Some(n) => format!("Region: page {page}, {size}, {n} objects"),
        None => format!("Region: page {page}, {size}"),
    }
}

/// Shown in place of the commands while no box is drawn.
#[must_use]
pub const fn none_yet() -> &'static str {
    "Draw a marquee first: drag a box on the page. Esc clears it."
}

/// Label before the inside/touching choice.
#[must_use]
pub const fn selects_label() -> &'static str {
    "Delete takes"
}

/// Objects wholly inside the box.
#[must_use]
pub const fn selects_inside() -> &'static str {
    "Inside only"
}

/// Objects the box touches.
#[must_use]
pub const fn selects_touching() -> &'static str {
    "Touching"
}

/// Under the choice: what it governs and what it does not.
#[must_use]
pub const fn selects_note() -> &'static str {
    "This decides what Delete in region removes. Redact always removes anything the \
     region crosses, and the Select tool's own marquee still picks by drag direction."
}

/// Label before the redact inside/outside choice.
#[must_use]
pub const fn redact_label() -> &'static str {
    "Redact"
}

/// Redact what is inside the box.
#[must_use]
pub const fn redact_inside() -> &'static str {
    "Inside region"
}

/// Redact everything except what is inside the box.
#[must_use]
pub const fn redact_outside() -> &'static str {
    "Outside region"
}

/// The redact button, for the current choice.
#[must_use]
pub const fn redact_button(outside: bool) -> &'static str {
    if outside {
        "Redact outside\u{2026}"
    } else {
        "Redact region"
    }
}

/// The status sentence after Delete in region.
#[must_use]
pub fn deleted(deleted: usize, crossing_kept: usize) -> String {
    let objects = if deleted == 1 { "object" } else { "objects" };
    match crossing_kept {
        0 => format!("Deleted {deleted} {objects} inside the region."),
        1 => format!("Deleted {deleted} {objects} inside the region; 1 crossing its edge kept."),
        n => format!("Deleted {deleted} {objects} inside the region; {n} crossing its edge kept."),
    }
}

/// Delete in region found nothing to delete.
#[must_use]
pub fn nothing_to_delete(crossing: usize) -> String {
    match crossing {
        0 => "Nothing is inside the region to delete.".to_owned(),
        n => format!(
            "Nothing is wholly inside the region; {n} crossing its edge were left. \
             Choose Touching to delete them too."
        ),
    }
}

/// The region is on a page that is not on screen.
#[must_use]
pub fn other_page(page_index: usize) -> String {
    format!(
        "The region is on page {}. Go to that page to delete in it.",
        page_index + 1
    )
}

/// The status sentence after Redact region.
#[must_use]
pub const fn marked_region() -> &'static str {
    "Region marked for redaction. Nothing has been removed yet \u{2014} press Apply \
     redactions when you have checked the marks."
}

/// The Redact outside window's title.
#[must_use]
pub const fn outside_title() -> &'static str {
    "Redact outside the region"
}

/// The window's warning, for `pages` pages.
#[must_use]
pub fn outside_warning(pages: usize) -> String {
    let pages = if pages == 1 {
        "1 page".to_owned()
    } else {
        format!("{pages} pages")
    };
    format!(
        "This permanently removes all text, images, vector graphics and annotations \
         OUTSIDE the selected area on {pages}. Only the selected area remains."
    )
}

/// When the removal happens.
#[must_use]
pub const fn outside_when() -> &'static str {
    "Mark adds the redaction marks now; the content goes when you apply redactions, \
     and Undo takes the marks back until then."
}

/// Pages left out because they are not the region page's size.
#[must_use]
pub fn outside_skipped(skipped: usize, page_index: usize) -> String {
    let page = page_index + 1;
    match skipped {
        0 => String::new(),
        1 => format!(
            "1 page is a different size from page {page}, so the region would not sit on \
             it in the same place. It is left alone."
        ),
        n => format!(
            "{n} pages are a different size from page {page}, so the region would not sit \
             on them in the same place. They are left alone."
        ),
    }
}

/// The scope names no page this region can be used on.
#[must_use]
pub const fn outside_no_pages() -> &'static str {
    "No page in this choice is the region page's size."
}

/// The window's commit button.
#[must_use]
pub const fn outside_mark() -> &'static str {
    "Mark outside"
}

/// The window's cancel button.
#[must_use]
pub const fn outside_cancel() -> &'static str {
    "Cancel"
}

/// The status sentence after Redact outside.
#[must_use]
pub fn marked_outside(marked: usize, nothing: usize, skipped: usize) -> String {
    let mut s = match marked {
        1 => "Marked everything outside the region on 1 page for redaction.".to_owned(),
        n => format!("Marked everything outside the region on {n} pages for redaction."),
    };
    if nothing > 0 {
        s.push_str(&format!(
            " {nothing} had nothing outside the region and got no mark."
        ));
    }
    if skipped > 0 {
        s.push_str(&format!(" {skipped} of another size were left alone."));
    }
    s.push_str(
        " Nothing has been removed yet \u{2014} press Apply redactions when you have checked.",
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chip_names_page_size_and_count() {
        let s = chip(2, 302.4, 223.2, Some(7));
        assert_eq!(
            s,
            "Region: page 3, 4.2 \u{d7} 3.1 in (107 \u{d7} 79 mm), 7 objects"
        );
        assert!(!chip(0, 72.0, 72.0, None).contains("object"));
    }

    #[test]
    fn the_delete_sentence_counts_both_sides() {
        assert_eq!(
            deleted(7, 2),
            "Deleted 7 objects inside the region; 2 crossing its edge kept."
        );
        assert_eq!(deleted(1, 0), "Deleted 1 object inside the region.");
    }

    #[test]
    fn the_warning_names_the_page_count() {
        assert!(outside_warning(3).contains("on 3 pages"));
        assert!(outside_warning(1).contains("on 1 page."));
        assert!(outside_skipped(0, 0).is_empty());
        assert!(marked_outside(2, 1, 1).contains("1 had nothing"));
    }
}
