//! The words for another PDF's page placed into a page's content.

/// A placed page drawn at other than its own size, to fit where it went.
#[must_use]
pub fn scaled(scale: f64) -> String {
    format!(
        "The placed page was drawn at {:.0}% of its own size.",
        scale * 100.0
    )
}

/// A placed page drawn at a different scale across than down.
#[must_use]
pub fn stretched(scale_x: f64, scale_y: f64) -> String {
    format!(
        "The placed page was stretched to fit: {:.0}% of its size across and {:.0}% down.",
        scale_x * 100.0,
        scale_y * 100.0
    )
}

/// Comments on the placed page, which are not part of its drawing.
#[must_use]
pub fn comments_left(count: usize) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!(
        "The placed page carried {count} comment{plural}, which are not part of its drawing and \
         were not placed."
    )
}

/// Form fields on the placed page, which are not part of its drawing.
#[must_use]
pub fn fields_left(count: usize) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!(
        "The placed page carried {count} form field{plural}, which are not part of its drawing \
         and were not placed."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_is_singular_and_more_are_plural() {
        assert!(comments_left(1).contains("1 comment,"));
        assert!(fields_left(2).contains("2 form fields,"));
    }

    #[test]
    fn scales_are_said_as_percentages() {
        assert!(scaled(0.5).contains("50%"));
        assert!(stretched(0.5, 0.25).contains("50% of its size across and 25% down"));
    }
}
