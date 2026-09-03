//! # `text::shortcuts` — the words the keyboard reference shows
//!
//! ## ★ The shortest catalog in this crate, and that is the design
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

/// The window's title.
#[must_use]
pub const fn window_title() -> &'static str {
    "Keyboard shortcuts"
}

/// The paragraph under the title.
///
/// Says the reference is **live** rather than written down, because that is a
/// fact an operator can act on: if a key is not here, it is not bound, and
/// there is no third possibility involving a list somebody forgot to update.
#[must_use]
pub const fn intro() -> &'static str {
    "Every key pdfcer responds to, read from the same table that dispatches \
     them. A key that is not on this list is not bound to anything."
}

/// Joins the chords of a command bound to more than one.
///
/// ★ A comma and a space rather than a slash or a pipe: `Ctrl+Y` and
/// `Ctrl+Shift+Z` are two *alternatives*, not a sequence, and a slash between
/// keys reads as "press these together" to anyone who has met `Ctrl+Alt+Del`.
#[must_use]
pub const fn chord_separator() -> &'static str {
    ", "
}

/// How many shortcuts are listed, and where the number came from.
///
/// ★ The count is here **because it is checkable**. An operator who suspects a
/// key is missing can compare it against nothing useful — but a *future* build
/// whose count drops has told them something, and the number is the cheapest
/// form that fact can take.
#[must_use]
pub fn derived_note(commands: usize) -> String {
    format!("{commands} commands have a keyboard shortcut in this build.")
}

/// Chords bound to a command this build does not have.
///
/// ★ **Disclosed rather than absorbed.** R8's convention is that a capability's
/// absence is expressed by its command not being registered, and a customized
/// or stripped build can therefore carry a keymap naming commands that are not
/// there. Those keys do nothing, so they are not listed — and *"this build has
/// fewer shortcuts than its keymap declares"* is a true, surprising fact that
/// an operator comparing two installations needs.
///
/// Worded as a fact about **this build**, not as an error: a stripped build is
/// a supported thing to be, and the eventual exe-to-DLL move makes it the
/// ordinary case.
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
///
/// Reachable when the manifest failed to load, in which case **no chord works
/// either** — so an empty list would be accurate and unhelpfully so. Saying
/// which of the two states this is turns a puzzling window into a diagnosis.
#[must_use]
pub const fn no_keymap() -> &'static str {
    "pdfcer could not read its own shortcut table, so no keys are bound in this \
     session. Everything is still reachable from the ribbon."
}

/// The keymap loaded and is empty.
///
/// A different sentence from [`no_keymap`], because the two are different
/// situations and only one of them is a fault. A manifest that deliberately
/// binds nothing is a legitimate customization.
#[must_use]
pub const fn none_bound() -> &'static str {
    "No keys are bound in this build."
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ★ **No string in this catalog names a key or a command.**
    ///
    /// The rule this module exists to hold, asserted rather than trusted. A
    /// hand-written `"Ctrl+S — Save a copy"` added here for convenience would
    /// re-create D5 in the one file whose whole purpose is that D5 cannot
    /// happen, and it would look perfectly reasonable in review.
    ///
    /// The probe list is chord *shapes* rather than every chord, because the
    /// point is to catch the habit, not to enumerate the keymap — enumerating
    /// it here would itself be the second copy.
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
    ///
    /// *"pdfcer could not read its table"* and *"nothing is bound"* are a fault
    /// and a customization, and an operator seeing one message for both cannot
    /// tell which they have.
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
