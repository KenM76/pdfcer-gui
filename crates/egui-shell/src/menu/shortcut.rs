//! Keyboard-chord hints — **derived from the keymap, never written twice**.
//!
//! # Why a context menu shows chords at all
//!
//! A context menu is where a user who has not yet learned the keyboard
//! goes. Showing `Ctrl+C` beside *Copy* is the one moment in an interface
//! where the mouse path and the keyboard path are visible side by side,
//! and it is how a menu teaches itself out of a job. Every desktop shell
//! since the 1980s does this, and the reason it works is that the user did
//! not have to go looking.
//!
//! # The rule that makes it maintainable: one source, derived
//!
//! The chord shown here is **computed from
//! [`crate::manifest::Keymap`]** — the same map the application consults
//! when it handles input. There is no second field on a menu item, no
//! `shortcut: "Ctrl+C"` in the menu document, and no table in the
//! application.
//!
//! That is not tidiness. A hand-written second copy of a key binding is
//! wrong the first time an operator rebinds anything, and it is wrong
//! *silently*: the menu says `Ctrl+C`, the key does something else, and
//! the interface is now actively lying to the person it was supposed to be
//! teaching. `SHELL_FRAMEWORK.md` §5 lets an operator rebind keys; a menu
//! that did not follow them would make that permission a trap.
//!
//! The keymap is stored chord → command, because that is the direction
//! *input* is resolved in (a key arrives; which command is it?). A menu
//! needs the opposite direction (a command is drawn; which key is it?), so
//! [`Shortcuts`] is a built-once reverse index rather than a linear scan
//! per item per frame.
//!
//! # When one command has several chords
//!
//! A keymap may bind two chords to one command — `Del` and `Ctrl+D` both
//! deleting, say. A menu row has space for one, and *"show them all"* is
//! not an option: the column would jump about and the row would read as
//! two commands.
//!
//! So [`prefer`] picks one, by a rule that is deterministic and
//! defensible rather than merely arbitrary:
//!
//! 1. **Fewest modifiers.** `Del` beats `Ctrl+D`; `Ctrl+C` beats
//!    `Ctrl+Shift+C`. A menu should teach the chord that is easiest to
//!    reach, because the point is to get the user to stop using the menu.
//! 2. **Shortest text**, as a tie-break. Between `Del` and `Backspace` the
//!    shorter one is the one that fits the column and the one a user is
//!    more likely to remember.
//! 3. **Lexicographic**, as the final tie-break, so the answer never
//!    depends on `BTreeMap` iteration order changing, on a file being
//!    re-saved, or on anything else that is not the text itself.
//!
//! All three are *total* on any pair of distinct strings, so the choice is
//! a function of the keymap alone — which is what
//! `the_chosen_chord_does_not_depend_on_insertion_order` pins.
//!
//! # What this module deliberately does not do
//!
//! It does not **parse** a chord. `"Ctrl+E"` stays an opaque string, for
//! exactly the reason [`crate::manifest::Keymap`] gives: parsing it into
//! modifiers and a key would mean a manifest could not be read by a tool
//! that does not link `egui`, and the menu only ever needs to *draw* it.
//!
//! The one exception is [`modifier_count`], which counts `+` separators.
//! That is not parsing — it never decides what a key *is*, only how many
//! pieces the operator wrote — and it degrades to "0 modifiers" on
//! anything strange rather than refusing it.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::manifest::Keymap;

/// Command id → the chord a menu will show for it.
///
/// Built once per frame (or once per menu open) from a
/// [`Keymap`], because the keymap is indexed the other way round. See the
/// module header.
///
/// Ordered (`BTreeMap`) so enumerating it — in a trace, in a failing test
/// message — is deterministic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shortcuts {
    by_command: BTreeMap<String, String>,
}

impl Shortcuts {
    /// No chord for anything.
    ///
    /// What a menu gets when the manifest has no keymap: every row draws
    /// its label alone, which is a correct menu rather than a degraded
    /// one.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Invert a keymap into command → chord, resolving multiple bindings
    /// with [`prefer`].
    #[must_use]
    pub fn from_keymap(keymap: &Keymap) -> Self {
        let mut by_command: BTreeMap<String, String> = BTreeMap::new();
        for (chord, command) in keymap.iter() {
            match by_command.get(command) {
                // `prefer` decides; an equal comparison keeps the
                // incumbent, which only happens for identical strings and
                // therefore cannot make the result order-dependent.
                Some(existing) if prefer(chord, existing) != Ordering::Less => {}
                _ => {
                    by_command.insert(command.to_owned(), chord.to_owned());
                }
            }
        }
        Self { by_command }
    }

    /// Invert the keymap of a manifest, if it has one.
    ///
    /// The convenience the renderer actually uses: a [`crate::Shell`] with
    /// no `keymap` key yields [`Self::none`] rather than requiring every
    /// call site to unwrap an `Option`.
    #[must_use]
    pub fn of(shell: &crate::manifest::Shell) -> Self {
        shell
            .keymap
            .as_ref()
            .map_or_else(Self::none, Self::from_keymap)
    }

    /// The chord to show beside this command, if it has one.
    #[must_use]
    pub fn get(&self, command_id: &str) -> Option<&str> {
        self.by_command.get(command_id).map(String::as_str)
    }

    /// Every command that has a chord, in command-id order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.by_command
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// How many commands carry a chord.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_command.len()
    }

    /// Whether no command carries a chord.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_command.is_empty()
    }
}

/// Which of two chords a menu should teach: `Less` means `a` wins.
///
/// Fewest modifiers, then shortest, then lexicographic. See the module
/// header for why each step is there. Total and antisymmetric, so the
/// result is a function of the two strings and of nothing else.
#[must_use]
pub fn prefer(a: &str, b: &str) -> Ordering {
    modifier_count(a)
        .cmp(&modifier_count(b))
        .then_with(|| a.chars().count().cmp(&b.chars().count()))
        .then_with(|| a.cmp(b))
}

/// How many modifiers a chord names, counted as `+`-separated pieces
/// minus the key itself.
///
/// `"Ctrl+Shift+P"` → 2, `"F11"` → 0, `"Ctrl+E"` → 1.
///
/// Degenerate inputs degrade to 0 rather than to a panic or a negative:
/// `"+"` names no pieces at all once empty segments are dropped, and
/// `""` names none either. A chord this strange is an operator's typo,
/// and the correct penalty is that it sorts first among equals — not that
/// the menu refuses to draw.
#[must_use]
pub fn modifier_count(chord: &str) -> usize {
    chord
        .split('+')
        .filter(|piece| !piece.trim().is_empty())
        .count()
        .saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keymap(pairs: &[(&str, &str)]) -> Keymap {
        Keymap(
            pairs
                .iter()
                .map(|(c, k)| ((*c).to_owned(), (*k).to_owned()))
                .collect(),
        )
    }

    /// The ordinary case: a chord reaches the command it is bound to.
    #[test]
    fn a_bound_command_gets_its_chord() {
        let s = Shortcuts::from_keymap(&keymap(&[
            ("Ctrl+C", "edit.copy"),
            ("Ctrl+V", "edit.paste"),
            ("F11", "view.fullscreen"),
        ]));
        assert_eq!(s.get("edit.copy"), Some("Ctrl+C"));
        assert_eq!(s.get("view.fullscreen"), Some("F11"));
        assert_eq!(s.get("edit.delete"), None, "unbound means no hint");
        assert_eq!(s.len(), 3);
    }

    /// **The chosen chord does not depend on insertion order.**
    ///
    /// The failure this prevents is nasty and would never be reported as a
    /// bug in this file: an operator adds an unrelated binding, the
    /// `BTreeMap`'s iteration order shifts, and *Delete* silently starts
    /// advertising a different key. The rule in [`prefer`] is total, so
    /// the answer is a function of the set of chords and of nothing else —
    /// which is what this asserts, by building the same set several ways.
    #[test]
    fn the_chosen_chord_does_not_depend_on_insertion_order() {
        let orders: [&[(&str, &str)]; 3] = [
            &[("Del", "edit.delete"), ("Ctrl+D", "edit.delete")],
            &[("Ctrl+D", "edit.delete"), ("Del", "edit.delete")],
            &[
                ("Ctrl+D", "edit.delete"),
                ("Alt+Backspace", "edit.delete"),
                ("Del", "edit.delete"),
            ],
        ];
        for pairs in orders {
            assert_eq!(
                Shortcuts::from_keymap(&keymap(pairs)).get("edit.delete"),
                Some("Del"),
                "the unmodified chord must win however the file was written: {pairs:?}"
            );
        }
    }

    /// The preference rule, step by step, so a change to it is a
    /// deliberate act rather than a side effect.
    #[test]
    fn the_preference_rule_is_modifiers_then_length_then_text() {
        // 1. Fewest modifiers.
        assert_eq!(prefer("Del", "Ctrl+D"), Ordering::Less);
        assert_eq!(prefer("Ctrl+C", "Ctrl+Shift+C"), Ordering::Less);
        // 2. Then shortest — both have zero modifiers.
        assert_eq!(prefer("Del", "Backspace"), Ordering::Less);
        // 3. Then lexicographic — same modifiers, same length.
        assert_eq!(prefer("Ctrl+A", "Ctrl+B"), Ordering::Less);
        // Antisymmetry and reflexivity, which `from_keymap` relies on to
        // keep its incumbent when the comparison is not `Less`.
        assert_eq!(prefer("Ctrl+A", "Ctrl+A"), Ordering::Equal);
        assert_eq!(prefer("Ctrl+B", "Ctrl+A"), Ordering::Greater);
    }

    /// Modifier counting is arithmetic on separators, not parsing, and
    /// every degenerate spelling lands on 0 rather than panicking.
    #[test]
    fn modifier_counting_degrades_rather_than_failing() {
        assert_eq!(modifier_count("F11"), 0);
        assert_eq!(modifier_count("Ctrl+E"), 1);
        assert_eq!(modifier_count("Ctrl+Shift+P"), 2);
        // The two spellings in which the key *is* the separator.
        //
        // `"+"` and `"Ctrl++"` both count 0, and that is a stated limit
        // rather than a bug: this module does not parse chords (see the
        // header), and separator counting cannot tell a `+` key from a `+`
        // separator without doing so. The consequence is bounded — such a
        // chord sorts earlier among equal candidates, which only matters
        // when one command has several bindings — and it is never that the
        // chord fails to draw.
        assert_eq!(modifier_count("+"), 0);
        assert_eq!(modifier_count("Ctrl++"), 0);
        assert_eq!(modifier_count(""), 0);
        assert_eq!(modifier_count("  "), 0);
    }

    /// A manifest with no keymap yields no hints — a correct menu, not a
    /// degraded one.
    #[test]
    fn a_manifest_without_a_keymap_yields_no_hints() {
        let shell = crate::manifest::Shell::new();
        let s = Shortcuts::of(&shell);
        assert!(s.is_empty());
        assert_eq!(s.get("edit.copy"), None);

        let bound = shell.with_binding("Ctrl+C", "edit.copy");
        assert_eq!(Shortcuts::of(&bound).get("edit.copy"), Some("Ctrl+C"));
    }

    /// Two commands may of course each have their own chord; the reverse
    /// index does not conflate them.
    #[test]
    fn distinct_commands_keep_distinct_chords() {
        let s = Shortcuts::from_keymap(&keymap(&[
            ("Ctrl+C", "edit.copy"),
            ("Ctrl+X", "edit.cut"),
            ("Ctrl+Shift+C", "edit.copy_special"),
        ]));
        assert_eq!(
            s.iter().collect::<Vec<_>>(),
            [
                ("edit.copy", "Ctrl+C"),
                ("edit.copy_special", "Ctrl+Shift+C"),
                ("edit.cut", "Ctrl+X"),
            ]
        );
    }
}
