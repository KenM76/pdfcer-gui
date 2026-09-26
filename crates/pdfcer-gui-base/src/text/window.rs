//! # `text::window` — the way back out of a mode that hides its own control
//!
//! Six strings, and **not one of them names a key.** Every chord below arrives
//! as a parameter, resolved by `pdfcer_gui::app::window::chord_for` from the same
//! keymap `app::keyboard` dispatches from. That is the rule
//! [`crate::text::shortcuts`] states for the keyboard reference, applied here
//! for a harder reason: the reference is read by somebody browsing, and these
//! sentences are read by somebody **stuck**.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/text/window.md`.

/// **The window title's read-mode prefix**, when a chord turns the mode off.
#[must_use]
pub fn title_read_mode(chord: &str) -> String {
    format!("Read mode — {chord} to exit")
}

/// The window title's read-mode prefix when **nothing is bound**.
#[must_use]
pub const fn title_read_mode_unbound() -> &'static str {
    "Read mode — see the bar at the bottom"
}

/// **The status bar's read-mode line**, when a chord turns the mode off.
#[must_use]
pub fn status_read_mode(chord: &str) -> String {
    format!("Read mode — press {chord} to bring the ribbon and the panels back.")
}

/// The status bar's line when read mode and **full screen** are both on.
#[must_use]
pub fn status_read_mode_and_fullscreen(read_chord: &str, fullscreen_chord: &str) -> String {
    format!(
        "Read mode and full screen — press {read_chord} to bring the ribbon and the panels \
         back, {fullscreen_chord} to leave full screen."
    )
}

/// The status bar's line when read mode is on and **nothing is bound to it**.
#[must_use]
pub const fn status_read_mode_unbound() -> &'static str {
    "Read mode — no key in this build turns it off."
}

/// **The escape hatch**, drawn only when no chord is bound.
#[must_use]
pub const fn leave_read_mode_button() -> &'static str {
    "Bring the ribbon and the panels back"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **No string in this catalog names a key.**
    #[test]
    fn no_string_here_names_a_key() {
        let strings = [
            title_read_mode_unbound().to_owned(),
            status_read_mode_unbound().to_owned(),
            leave_read_mode_button().to_owned(),
            // The chord-bearing entries with a chord the probes cannot match,
            // so what is tested is the FIXED part of each format string.
            title_read_mode("<k>"),
            status_read_mode("<k>"),
            status_read_mode_and_fullscreen("<k>", "<j>"),
        ];
        for text in strings {
            for probe in ["Ctrl", "Alt", "Shift", "F11", "F1", "Esc", "view.", "edit."] {
                assert!(
                    !text.contains(probe),
                    "`{probe}` appears in a catalog string — the chord must come from the \
                     keymap that dispatches, never from here: {text:?}"
                );
            }
        }
    }

    /// **Each chord reaches the sentence, exactly once.**
    #[test]
    fn every_chord_handed_in_reaches_the_sentence() {
        assert_eq!(title_read_mode("Ctrl+H").matches("Ctrl+H").count(), 1);
        assert_eq!(status_read_mode("Ctrl+H").matches("Ctrl+H").count(), 1);
        let both = status_read_mode_and_fullscreen("Ctrl+H", "F11");
        assert_eq!(both.matches("Ctrl+H").count(), 1);
        assert_eq!(both.matches("F11").count(), 1);
        assert!(
            both.find("Ctrl+H") < both.find("F11"),
            "read mode leads: it is the mode that took the ribbon away, and the \
             full-screen control went with it"
        );
    }

    /// **The status line names what comes back**, in the command's own words.
    #[test]
    fn the_status_line_names_the_ribbon_and_the_panels() {
        for text in [
            status_read_mode("Ctrl+H"),
            status_read_mode_and_fullscreen("Ctrl+H", "F11"),
            leave_read_mode_button().to_owned(),
        ] {
            assert!(text.contains("ribbon"), "{text}");
            assert!(text.contains("panels"), "{text}");
        }
    }

    /// The title prefix is **short**, because it competes with four other facts
    /// in a strip the taskbar truncates.
    ///
    /// Bounded rather than exact, so rewording is allowed and sprawl is not.
    #[test]
    fn the_title_prefix_stays_short() {
        let title = title_read_mode("Ctrl+Shift+H");
        assert!(
            title.chars().count() <= 40,
            "the title prefix shares the strip with a file name, a document count, the \
             product name and a build stamp: {title:?} is {} characters",
            title.chars().count()
        );
        assert!(title_read_mode_unbound().chars().count() <= 40);
    }

    /// The unbound wordings never promise a key, and the bound ones never
    /// suggest there is not one.
    #[test]
    fn the_bound_and_unbound_wordings_are_different_sentences() {
        assert_ne!(status_read_mode("Ctrl+H"), status_read_mode_unbound());
        assert!(status_read_mode_unbound().contains("no key"));
        assert!(title_read_mode_unbound().contains("bottom"));
    }
}
