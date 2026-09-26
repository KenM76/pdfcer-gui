//! # `text::shortcuts` — the words the keyboard reference shows
//!
//! ## The shortest catalog in this crate, and that is the design
//!
//! Six strings, and **none of them is a shortcut**. Every chord and every
//! command name in that window comes from the live keymap and the command
//! registry; if a key or a label appeared here it would be a second statement
//! of a fact that already has one, which is `DEFECTS.md` D5 exactly:
//!
//! > The keyboard-shortcuts reference omits six live bindings.
//!
//! The old shell's reference was a hand-maintained list in a 7,912-line
//! catalog. Six bindings existed and were not in it, and nobody noticed because
//! nothing exercised the list — a reference is read by operators and by no
//! test.
//!
//! So the rule for this file is narrow and worth stating: **a string may
//! describe the reference; it may not be part of it.**
//!
//! Design and rationale: `docs/modules/pdfcer-gui/text/shortcuts.md`.

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Keyboard shortcuts"
}

/// The paragraph under the title.
#[must_use]
pub const fn intro() -> &'static str {
    "Every key pdfcer responds to, read from the same table that dispatches \
     them. A key that is not on this list is not bound to anything."
}

/// Joins the chords of a command bound to more than one.
#[must_use]
pub const fn chord_separator() -> &'static str {
    ", "
}

/// How many shortcuts are listed, and where the number came from.
#[must_use]
pub fn derived_note(commands: usize) -> String {
    format!("{commands} commands have a keyboard shortcut in this build.")
}

/// Chords bound to a command this build does not have.
#[must_use]
pub fn dropped_note(dropped: usize) -> String {
    if dropped == 1 {
        "1 more key is set up but does nothing here — the feature it belongs to \
         is not part of this build."
            .to_owned()
    } else {
        format!(
            "{dropped} more keys are set up but do nothing here — the features \
             they belong to are not part of this build."
        )
    }
}

/// The build has no keymap at all.
#[must_use]
pub const fn no_keymap() -> &'static str {
    "pdfcer could not read its own shortcut table, so no keys are bound in this \
     session. Everything is still reachable from the ribbon."
}

/// The keymap loaded and is empty.
#[must_use]
pub const fn none_bound() -> &'static str {
    "No keys are bound in this build."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **No string in this catalog names a key or a command.**
    #[test]
    fn no_string_here_is_part_of_the_reference() {
        let strings = [
            window_title(),
            intro(),
            chord_separator(),
            no_keymap(),
            none_bound(),
        ];
        for text in strings {
            for probe in ["Ctrl+", "Alt+", "Shift+", "F11", "file.", "edit.", "view."] {
                assert!(
                    !text.contains(probe),
                    "`{probe}` appears in a catalog string — the reference must come from \
                     the keymap, never from here: {text:?}"
                );
            }
        }
        // The two counted sentences are built from a number and must not name
        // anything either.
        assert!(!derived_note(7).contains("Ctrl"));
        assert!(!dropped_note(2).contains("Ctrl"));
    }

    /// The two empty states are different sentences.
    #[test]
    fn a_missing_table_and_an_empty_one_read_differently() {
        assert_ne!(no_keymap(), none_bound());
        assert!(no_keymap().contains("could not read"));
        assert!(
            no_keymap().contains("ribbon"),
            "it must say what still works"
        );
    }

    /// The dropped-key sentence agrees in number and blames the build, not the
    /// operator.
    #[test]
    fn the_dropped_note_agrees_in_number_and_states_a_fact() {
        assert!(dropped_note(1).starts_with("1 more key is"));
        assert!(dropped_note(3).starts_with("3 more keys are"));
        for n in [1, 3] {
            let text = dropped_note(n);
            assert!(
                text.contains("not part of this build"),
                "a stripped build is a supported thing to be: {text}"
            );
            for alarm in ["error", "failed", "missing feature", "broken"] {
                assert!(!text.to_lowercase().contains(alarm), "{text}");
            }
        }
    }
}
