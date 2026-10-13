//! # `text::insertblank` — the copy of the Insert-blank-pages dialog
//!
//! Every operator-facing string `pdfcer_gui::dialogs::insert_blank` draws
//! that no neighbouring dialog already owns, and the one sentence an insert
//! owes when a page list stops part-way. The size names, orientation pair and
//! sheet summary are `text::new_document`'s; the four "where" radios are
//! `text::pages`'.

/// The dialog's window title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Insert blank pages"
}

/// Heading over the size list.
#[must_use]
pub const fn size_heading() -> &'static str {
    "Page size"
}

/// The size entry that copies the page the operator was on, shown as it
/// displays (rotation applied).
#[must_use]
pub fn same_as_page(page_number: usize, width_pt: f64, height_pt: f64) -> String {
    use crate::units::whole_mm_from_points as mm;
    format!(
        "Same as page {page_number} — {} × {} mm",
        mm(width_pt),
        mm(height_pt)
    )
}

/// Label beside the count field.
#[must_use]
pub const fn count_label() -> &'static str {
    "How many"
}

/// The commit button.
#[must_use]
pub fn commit(count: usize) -> String {
    if count == 1 {
        "Insert 1 blank page".to_owned()
    } else {
        format!("Insert {count} blank pages")
    }
}

/// An insert whose page list stopped part-way: `landed` of `asked` pages
/// arrived before the engine refused the rest.
#[must_use]
pub fn insert_stopped(landed: usize, asked: usize, reason: &str) -> String {
    format!("Only {landed} of {asked} pages were inserted; the rest were refused: {reason}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_commit_button_counts_in_english() {
        assert_eq!(commit(1), "Insert 1 blank page");
        assert_eq!(commit(3), "Insert 3 blank pages");
    }

    #[test]
    fn same_as_page_names_the_page_and_its_size() {
        assert_eq!(
            same_as_page(4, 595.276, 841.89),
            "Same as page 4 — 210 × 297 mm"
        );
    }
}
