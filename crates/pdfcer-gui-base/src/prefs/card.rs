//! # `prefs::card` — whether the page view draws pages on the graphics card
//!
//! Key: `draw_pages_on_graphics_card` (`true` | `false`, default `false`). On,
//! each page the canvas draws is packed by the engine for the graphics card
//! and drawn there, the same picture byte for byte; a page the card cannot
//! draw yet is drawn by the processor, and the reason is said off the page.
//! Screen only: print and every export draw on the processor as before.

use std::fmt::Write as _;

use super::printing::KeyOutcome;

/// The file key.
const KEY: &str = "draw_pages_on_graphics_card";

/// The page view's graphics-card preference.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CardPrefs {
    /// Draw the canvas's pages on the graphics card where it can.
    pub draw_pages: bool,
}

/// Claim `key = value` if it is this group's.
pub(super) fn parse_key(prefs: &mut CardPrefs, key: &str, value: &str) -> KeyOutcome {
    if key != KEY {
        return KeyOutcome::NotMine;
    }
    match value.trim().parse::<bool>() {
        Ok(on) => {
            prefs.draw_pages = on;
            KeyOutcome::Accepted
        }
        Err(_) => KeyOutcome::BadValue,
    }
}

/// Append this group's block to a preferences file.
pub(super) fn write_block(prefs: &CardPrefs, out: &mut String) {
    out.push_str(
        "\n\
         # draw_pages_on_graphics_card: true or false (default false). Draws\n\
         # the pages on screen with the graphics card where it can, the same\n\
         # picture. Screen only; printing and exports are not affected.\n",
    );
    let _ = writeln!(out, "{KEY} = {}", prefs.draw_pages); // ui-text-exempt: a preferences-file line, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_round_trips_and_a_bad_value_is_refused() {
        let mut prefs = CardPrefs::default();
        assert!(!prefs.draw_pages);
        let mut out = String::new();
        write_block(&CardPrefs { draw_pages: true }, &mut out);
        let line = out.lines().last().unwrap();
        let (key, value) = line.split_once(" = ").unwrap();
        assert!(matches!(
            parse_key(&mut prefs, key, value),
            KeyOutcome::Accepted
        ));
        assert!(prefs.draw_pages);
        assert!(matches!(
            parse_key(&mut prefs, key, "maybe"),
            KeyOutcome::BadValue
        ));
        assert!(matches!(
            parse_key(&mut prefs, "other", "true"),
            KeyOutcome::NotMine
        ));
    }
}
