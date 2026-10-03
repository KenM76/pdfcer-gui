//! # `keychord` — spell a manifest chord into the modifiers and key that fire it
//!
//! The grammar and its reasons: `docs/modules/pdfcer-gui/app/keyboard.md`, `fn parse_chord`.

use egui::{Key, Modifiers};

/// **Spell a manifest chord into the modifiers and key that fire it.**
pub fn parse_chord(chord: &str) -> Option<(Modifiers, Key)> {
    let mut modifiers = Modifiers::NONE;
    let mut key = None;
    for part in chord.split('+') {
        match part {
            "Ctrl" | "Cmd" | "Command" => modifiers.command = true,
            "Shift" => modifiers.shift = true,
            "Alt" => modifiers.alt = true,
            // A trailing empty segment is the literal `+` of a chord spelled
            // `Ctrl++`. Splitting on the separator cannot tell the two apart,
            // so the empty string is read as the key it can only have been.
            "" => key = Some(Key::Plus),
            other => key = Some(Key::from_name(other)?),
        }
    }
    Some((modifiers, key?))
}

/// Spell `key` under `modifiers` the way the manifest does: `Ctrl+`, `Shift+`,
/// `Alt+`, then the key's symbol when [`parse_chord`] reads it back, else its
/// name. The result always parses to the same chord.
#[must_use]
pub fn format_chord(modifiers: Modifiers, key: Key) -> String {
    let mut out = String::new();
    // ui-text-exempt: manifest chord spelling, parsed back by parse_chord.
    for (on, word) in [
        (modifiers.command || modifiers.ctrl, "Ctrl+"),
        (modifiers.shift, "Shift+"),
        (modifiers.alt, "Alt+"),
    ] {
        if on {
            out.push_str(word);
        }
    }
    let symbol = key.symbol_or_name();
    let readable = symbol.len() == 1
        && symbol != "+"
        && symbol.chars().all(|c| c.is_ascii_punctuation())
        && Key::from_name(symbol) == Some(key);
    out.push_str(if readable { symbol } else { key.name() });
    out
}

/// `chord` respelled by [`format_chord`], or `None` when it does not parse.
#[must_use]
pub fn canonical(chord: &str) -> Option<String> {
    let (modifiers, key) = parse_chord(chord)?;
    Some(format_chord(modifiers, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_round_trips_under_four_modifier_sets() {
        let sets = [
            Modifiers::NONE,
            Modifiers::COMMAND,
            Modifiers::COMMAND | Modifiers::SHIFT,
            Modifiers::ALT | Modifiers::SHIFT,
        ];
        for key in Key::ALL {
            for m in sets {
                let spelled = format_chord(m, *key);
                assert_eq!(
                    parse_chord(&spelled),
                    Some((m, *key)),
                    "`{spelled}` does not read back as {key:?}"
                );
            }
        }
    }

    #[test]
    fn a_manifest_spelling_is_kept_where_it_is_already_canonical() {
        assert_eq!(canonical("Ctrl+[").as_deref(), Some("Ctrl+["));
        assert_eq!(canonical("Ctrl+OpenBracket").as_deref(), Some("Ctrl+["));
        assert_eq!(canonical("Ctrl+Minus").as_deref(), Some("Ctrl+Minus"));
        assert_eq!(canonical("Shift+Ctrl+Z").as_deref(), Some("Ctrl+Shift+Z"));
        assert_eq!(canonical("Ctrl+Nope"), None);
    }
}
