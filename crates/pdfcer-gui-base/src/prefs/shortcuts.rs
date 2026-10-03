//! # `prefs::shortcuts` — the operator's own keys, one line per command he changed
//!
//! `shortcut.<command id> = <chord> <chord>…` gives a command exactly those
//! chords; `none` switches its keys off. A command with no line keeps the
//! program's keys. [`ShortcutPrefs::apply`] lays the lines over the built-in
//! keymap; the Settings ▸ Keyboard shortcuts page writes them.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/prefs/shortcuts.md`.

use std::collections::BTreeMap;

use super::printing::KeyOutcome;
use crate::keychord::{canonical, parse_chord};

// ui-text-exempt: a file KEY prefix, written into preferences.txt and parsed
// back out of it. Never displayed.
const PREFIX: &str = "shortcut.";

// ui-text-exempt: the file's word for a command whose keys are switched off.
const OFF: &str = "none";

/// The commands whose keys the operator has set, each with its chords in the
/// manifest's spelling. An empty list means switched off.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShortcutPrefs {
    keys: BTreeMap<String, Vec<String>>,
}

impl ShortcutPrefs {
    /// The chords set for `command`: `None` when it keeps the program's own,
    /// `Some(&[])` when its keys are off.
    #[must_use]
    pub fn get(&self, command: &str) -> Option<&[String]> {
        self.keys.get(command).map(Vec::as_slice)
    }

    /// Give `command` exactly `chords`, respelled canonically; an unreadable
    /// chord is dropped.
    pub fn set(&mut self, command: &str, chords: &[String]) {
        let mut kept: Vec<String> = chords.iter().filter_map(|c| canonical(c)).collect();
        kept.dedup();
        self.keys.insert(command.to_owned(), kept);
    }

    /// Return `command` to the program's own keys.
    pub fn reset(&mut self, command: &str) {
        self.keys.remove(command);
    }

    /// Return every command to the program's own keys.
    pub fn reset_all(&mut self) {
        self.keys.clear();
    }

    /// Whether every command keeps the program's own keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Rewrite `keymap` (chord → command) so each command named here answers
    /// exactly its chords. A chord taken this way leaves the command that held
    /// it, whatever spelling the keymap used for it.
    pub fn apply(&self, keymap: &mut BTreeMap<String, String>) {
        for command in self.keys.keys() {
            keymap.retain(|_, bound| bound != command);
        }
        for (command, chords) in &self.keys {
            for chord in chords {
                let wanted = parse_chord(chord);
                keymap.retain(|k, _| parse_chord(k) != wanted);
                keymap.insert(chord.clone(), command.clone());
            }
        }
    }
}

/// `defaults` with `prefs` laid over it: the keymap a Save would produce.
#[must_use]
pub fn effective(
    defaults: &BTreeMap<String, String>,
    prefs: &ShortcutPrefs,
) -> BTreeMap<String, String> {
    let mut map = defaults.clone();
    prefs.apply(&mut map);
    map
}

pub(super) fn parse_key(prefs: &mut ShortcutPrefs, key: &str, value: &str) -> KeyOutcome {
    let Some(command) = key.strip_prefix(PREFIX) else {
        return KeyOutcome::NotMine;
    };
    if command.is_empty() {
        return KeyOutcome::BadValue;
    }
    if value.is_empty() || value.eq_ignore_ascii_case(OFF) {
        prefs.keys.insert(command.to_owned(), Vec::new());
        return KeyOutcome::Accepted;
    }
    let chords: Option<Vec<String>> = value.split_whitespace().map(canonical).collect();
    match chords {
        Some(chords) => {
            prefs.keys.insert(command.to_owned(), chords);
            KeyOutcome::Accepted
        }
        None => KeyOutcome::BadValue,
    }
}

pub(super) fn write_block(prefs: &ShortcutPrefs, out: &mut String) {
    out.push_str(
        "\n\
         # Keyboard shortcuts you have changed, one line per command:\n\
         #   shortcut.edit.find = Ctrl+K\n\
         #   shortcut.edit.redo = Ctrl+Y Ctrl+Shift+Z\n\
         #   shortcut.format.bold = none\n\
         #\n\
         # A line gives that command exactly the keys listed, separated by\n\
         # spaces, and takes each of them from whichever command had it.\n\
         # `none` switches the command's keys off. A command with no line keeps\n\
         # pdfcer's own keys. Settings > Keyboard shortcuts writes these lines.\n",
    );
    for (command, chords) in &prefs.keys {
        // ui-text-exempt: a file KEY, written into preferences.txt and parsed
        // back out of it. Never displayed.
        out.push_str(PREFIX);
        out.push_str(command);
        // ui-text-exempt: the file's key/value separator. Never displayed.
        out.push_str(" = ");
        if chords.is_empty() {
            out.push_str(OFF);
        } else {
            out.push_str(&chords.join(" ")); // ui-text-exempt: the chord separator in a preferences line
        }
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(c, i)| ((*c).to_owned(), (*i).to_owned()))
            .collect()
    }

    fn parsed(text: &str) -> ShortcutPrefs {
        let mut prefs = ShortcutPrefs::default();
        for line in text.lines().filter(|l| !l.trim_start().starts_with('#')) {
            if let Some((k, v)) = line.split_once('=') {
                let _ = parse_key(&mut prefs, k.trim(), v.trim());
            }
        }
        prefs
    }

    #[test]
    fn a_new_chord_replaces_every_old_one() {
        let mut keymap = map(&[("Ctrl+Y", "edit.redo"), ("Ctrl+Shift+Z", "edit.redo")]);
        let mut prefs = ShortcutPrefs::default();
        prefs.set("edit.redo", &["Ctrl+R".to_owned()]);
        prefs.apply(&mut keymap);
        assert_eq!(keymap, map(&[("Ctrl+R", "edit.redo")]));
    }

    #[test]
    fn a_taken_chord_leaves_its_holder_whatever_its_spelling() {
        let mut keymap = map(&[("Ctrl+[", "markup.send_backward"), ("Ctrl+F", "edit.find")]);
        let mut prefs = ShortcutPrefs::default();
        prefs.set("edit.find", &["Ctrl+OpenBracket".to_owned()]);
        prefs.apply(&mut keymap);
        assert_eq!(keymap, map(&[("Ctrl+[", "edit.find")]));
    }

    #[test]
    fn switched_off_leaves_no_chord() {
        let mut keymap = map(&[("Ctrl+B", "format.bold"), ("Ctrl+I", "format.italic")]);
        let mut prefs = ShortcutPrefs::default();
        prefs.set("format.bold", &[]);
        prefs.apply(&mut keymap);
        assert_eq!(keymap, map(&[("Ctrl+I", "format.italic")]));
    }

    #[test]
    fn the_file_round_trips_set_and_off() {
        let mut prefs = ShortcutPrefs::default();
        prefs.set("edit.find", &["Ctrl+K".to_owned()]);
        prefs.set(
            "edit.redo",
            &["Ctrl+Y".to_owned(), "Ctrl+Shift+Z".to_owned()],
        );
        prefs.set("format.bold", &[]);
        let mut text = String::new();
        write_block(&prefs, &mut text);
        assert!(text.contains("shortcut.format.bold = none"), "{text}");
        assert_eq!(parsed(&text), prefs);
    }

    #[test]
    fn an_unreadable_chord_is_a_bad_value_and_sets_nothing() {
        let mut prefs = ShortcutPrefs::default();
        assert_eq!(
            parse_key(&mut prefs, "shortcut.edit.find", "Ctrl+NoSuchKey"),
            KeyOutcome::BadValue
        );
        assert!(prefs.is_empty());
        assert_eq!(
            parse_key(&mut prefs, "off_page.read", "true"),
            KeyOutcome::NotMine
        );
    }
}
