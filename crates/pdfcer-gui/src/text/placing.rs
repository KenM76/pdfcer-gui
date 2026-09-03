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

/// The button inside the dialog.
///
/// ★ The **ellipsis** means *this window steps aside*, which is the same
/// promise `Save a copy…` and `Open…` make about a picker. Without it the
/// button reads as one that does something immediately, and what it actually
/// does is make the window disappear — the most alarming thing on this surface
/// if it is unannounced.
///
/// His verb, not ours: he wrote *"place it with the mouse"*.
#[must_use]
pub fn place_button() -> &'static str {
    "Place it on the page…"
}

/// The tooltip, and the only place the RETURN is promised before it is needed.
///
/// ★★ The last clause is the one that matters. An operator about to press a
/// button that makes their window vanish needs to know it is coming back
/// *before* they press it, not afterwards — and afterwards the tooltip is off
/// screen with the window.
#[must_use]
pub fn place_tooltip() -> &'static str {
    "Close this window and click where it goes, or drag a box for its size. \
     pdfcer fills these numbers in and brings this window back."
}

/// The note under the button, saying when the pointer beats the keyboard.
///
/// ★ It ends by saying the numbers are still editable, because the button
/// otherwise reads as a mode you commit to. Both routes stay live and neither
/// is the real one.
#[must_use]
pub fn place_note() -> &'static str {
    "Easier than typing coordinates when you can see where it belongs. You can \
     still correct the numbers here afterwards."
}

/// ★★★ **The instruction on the Tool panel while a placement is armed, and it
/// is not optional.**
///
/// Every other armed-tool sentence in this shell is a convenience: the ribbon
/// control is still pressed and the tooltip is still hoverable, so the panel is
/// repeating something findable elsewhere.
///
/// This one is the **only** statement of the gesture and of the way out,
/// because `crate::dialogs::placing` hides the requesting window for exactly as
/// long as the placement is pending. The button's tooltip went with it.
///
/// ⇒ So it carries three things and cannot lose any of them: what a click does,
/// what a drag does, and that Escape brings the window back.
#[must_use]
pub fn armed_instruction() -> &'static str {
    "Click where it goes, or drag a box for its size. Escape brings the window \
     back."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★★ The return is promised in BOTH places an operator can be.
    ///
    /// Before they press, in the tooltip; and after they press — when the
    /// tooltip is off screen with the window — on the Tool panel. A version
    /// that promised it only once would be correct in the source and useless in
    /// whichever of the two moments it was missing from.
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

    /// ★★★ The armed instruction names the way OUT, which is the sentence that
    /// stops an operator being stranded.
    ///
    /// `canvas::placing`'s header records that the precedent this arm
    /// generalises — the Set-scale calibration — strands an operator on Escape
    /// today, with no window and no route back. The mechanism here makes that
    /// unrepresentable; this assertion makes sure he is also TOLD.
    #[test]
    fn the_armed_instruction_names_escape() {
        assert!(
            armed_instruction().contains("Escape"),
            "{}",
            armed_instruction()
        );
    }

    /// Both gestures are offered, in both sentences.
    ///
    /// A click and a drag do different things — a corner versus a box — and an
    /// operator who is told only about the click concludes the drag is not
    /// offered, which is the exact failure `panels::tool::armed` records for
    /// the form-field instruction.
    #[test]
    fn both_gestures_are_offered_wherever_the_gesture_is_described() {
        // ★ Case-insensitively: one of the two sentences begins with the
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
