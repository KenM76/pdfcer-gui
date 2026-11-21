//! # `text::panels::placed` — how widely each drawing on the page is drawn
//!
//! One line per form XObject the page places, from the engine's
//! `InvocationSet`: where it is drawn across the whole document. A count the
//! engine could not complete is said as "at least".

/// One placed drawing: `object` is drawn in `places` places on `pages` pages.
#[must_use]
pub fn objects_dock_placed(object: u32, places: usize, pages: usize, lower_bound: bool) -> String {
    let at_least = if lower_bound { "at least " } else { "" };
    if places <= 1 && !lower_bound {
        return format!("Placed drawing {object}: drawn only here.");
    }
    format!(
        "Placed drawing {object}: drawn in {at_least}{places} place(s) on {at_least}{pages} \
         page(s)."
    )
}

/// Hover on the placed-drawing lines.
#[must_use]
pub fn objects_dock_placed_hover() -> &'static str {
    "A placed drawing is one drawing shown wherever it is placed, such as a title block on \
     every sheet. Editing what is inside it changes every place it is drawn. To change it on \
     this page alone, select it and choose Give this page its own copy."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shared_drawing_is_counted_and_a_partial_count_says_at_least() {
        assert_eq!(
            objects_dock_placed(4, 4, 3, false),
            "Placed drawing 4: drawn in 4 place(s) on 3 page(s)."
        );
        assert!(objects_dock_placed(4, 4, 3, true).contains("at least 4 place(s)"));
        assert_eq!(
            objects_dock_placed(5, 1, 1, false),
            "Placed drawing 5: drawn only here."
        );
    }
}
