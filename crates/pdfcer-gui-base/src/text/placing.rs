//! # `text::placing` — the words a window says when it steps aside
//!
//! `OPERATOR_REQUESTS.md` **O66**:
//!
//! > *"anything we are inserting like this should have an option in its
//! > dialogue box to place it with the mouse instead of by positional
//! > co-ordinates."*
//!
//! Four sentences, and one of them is load-bearing in a way the others are not
//! — see [`armed_instruction`].
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/placing.md`.

/// The button inside the dialog.
#[must_use]
pub fn place_button() -> &'static str {
    "Place it on the page…"
}

/// The tooltip, and the only place the RETURN is promised before it is needed.
#[must_use]
pub fn place_tooltip() -> &'static str {
    "Close this window and click where it goes, or drag a box for its size. \
     pdfcer fills these numbers in and brings this window back."
}

/// The note under the button, saying when the pointer beats the keyboard.
#[must_use]
pub fn place_note() -> &'static str {
    "Easier than typing coordinates when you can see where it belongs. You can \
     still correct the numbers here afterwards."
}

/// **The instruction on the Tool panel while a placement is armed, and it
/// is not optional.**
#[must_use]
pub fn armed_instruction() -> &'static str {
    "Click where it goes, or drag a box for its size. Escape brings the window \
     back."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The return is promised in BOTH places an operator can be.
    #[test]
    fn the_window_coming_back_is_promised_before_and_after() {
        assert!(
            place_tooltip().contains("brings this window back"),
            "before pressing: {}",
            place_tooltip()
        );
        assert!(
            armed_instruction().contains("brings the window back"),
            "after pressing, when the tooltip is gone: {}",
            armed_instruction()
        );
    }

    /// The armed instruction names the way OUT, which is the sentence that
    /// stops an operator being stranded.
    #[test]
    fn the_armed_instruction_names_escape() {
        assert!(
            armed_instruction().contains("Escape"),
            "{}",
            armed_instruction()
        );
    }

    /// Both gestures are offered, in both sentences.
    #[test]
    fn both_gestures_are_offered_wherever_the_gesture_is_described() {
        // Case-insensitively: one of the two sentences begins with the
        // word, and asserting the lower-case spelling would be testing
        // capitalisation rather than the property. Caught by this test's own
        // first run, which is the cheapest place to find it.
        for s in [place_tooltip(), armed_instruction()] {
            let lower = s.to_lowercase();
            assert!(lower.contains("click"), "{s}");
            assert!(lower.contains("drag"), "{s}");
        }
    }

    /// The button promises a picker-like disappearance with an ellipsis.
    #[test]
    fn the_button_says_the_window_steps_aside() {
        assert!(place_button().ends_with('…'), "{}", place_button());
    }
}
