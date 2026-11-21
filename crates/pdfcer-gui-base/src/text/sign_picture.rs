//! What the Sign window says about the picture in a visible signature.

/// The heading over the choice.
#[must_use]
pub const fn heading() -> &'static str {
    "Picture in the box"
}

/// The control that picks a PNG or JPEG.
#[must_use]
pub const fn choose() -> &'static str {
    "Choose picture…"
}

/// The picker's title bar.
#[must_use]
pub const fn picker_title() -> &'static str {
    "Choose a picture for the signature"
}

/// Shown while no picture is chosen.
#[must_use]
pub const fn none() -> &'static str {
    "None: the box shows its text only."
}

/// The chosen picture.
#[must_use]
pub fn chosen(name: &str, width: u32, height: u32) -> String {
    format!(
        "{name}, {width} × {height} pixels, drawn at the left of the box, which \
         widens to hold it."
    )
}

/// The control that drops the chosen picture.
#[must_use]
pub const fn remove() -> &'static str {
    "Remove"
}

/// A file that could not be read as a picture.
#[must_use]
pub fn unreadable(why: &str) -> String {
    format!("That file cannot be used as the picture: {why}.")
}

/// The receipt's line about the picture: `None` when none was asked for.
#[must_use]
pub const fn receipt(requested: bool, drawn: bool) -> Option<&'static str> {
    match (requested, drawn) {
        (false, _) => None,
        (true, true) => Some("The picture is drawn in the box beside its text."),
        (true, false) => Some("The picture was not drawn: the box shows its text only."),
    }
}

#[cfg(test)]
mod tests {
    use super::receipt;

    #[test]
    fn the_receipt_speaks_only_of_a_picture_asked_for() {
        assert_eq!(receipt(false, false), None);
        assert!(receipt(true, true).is_some_and(|s| s.contains("is drawn")));
        assert!(receipt(true, false).is_some_and(|s| s.contains("not drawn")));
    }
}
