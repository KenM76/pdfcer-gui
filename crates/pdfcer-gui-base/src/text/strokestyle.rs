//! # `text::strokestyle` — the words of the line-width, dash and opacity rows
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/strokestyle.md`.

/// The section heading.
#[must_use]
pub fn heading() -> String {
    "Line and opacity".to_owned()
}

/// The line-width row.
#[must_use]
pub fn width_label() -> String {
    "Width".to_owned()
}

/// Beside a width, dash or opacity control whose members disagree.
#[must_use]
pub fn mixed() -> String {
    "(mixed — a new value sets them all)".to_owned()
}

/// The dash row.
#[must_use]
pub fn dash_label() -> String {
    "Style".to_owned()
}

/// The stroking-alpha row of a shape.
#[must_use]
pub fn line_opacity_label() -> String {
    "Line opacity".to_owned()
}

/// The non-stroking-alpha row of a shape.
#[must_use]
pub fn fill_opacity_label() -> String {
    "Fill opacity".to_owned()
}

/// The one opacity row of a picture or a placed drawing.
#[must_use]
pub fn picture_opacity_label() -> String {
    "Picture opacity".to_owned()
}

/// The status line after a restyle that changed everything sent.
#[must_use]
pub fn restyled(changed: usize) -> String {
    format!("Restyled {changed} object(s).")
}

/// The status line when some objects were left alone.
#[must_use]
pub fn restyled_partly(changed: usize, refused: usize) -> String {
    format!(
        "Restyled {changed} object(s). {refused} were left alone — text, or a picture the change \
         does not apply to."
    )
}

/// The status line when the gesture could not be folded into one undo step.
#[must_use]
pub fn several_undos(steps: usize) -> String {
    format!("The change was applied; undoing it takes {steps} steps.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_string_is_non_empty() {
        for s in [
            heading(),
            width_label(),
            mixed(),
            dash_label(),
            line_opacity_label(),
            fill_opacity_label(),
            picture_opacity_label(),
            restyled(2),
            restyled_partly(2, 1),
            several_undos(2),
        ] {
            assert!(!s.trim().is_empty());
        }
    }
}
