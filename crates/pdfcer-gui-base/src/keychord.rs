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
