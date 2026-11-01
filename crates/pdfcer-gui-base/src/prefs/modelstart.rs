//! # `prefs::modelstart` — how the 3D viewer opens
//!
//! Key: `model_view_start` (`walk` | `orbit`, default `walk`). `walk` opens
//! the viewer walking with ghost on, from the opening view's eye; `orbit`
//! opens it orbiting the model.

use super::printing::KeyOutcome;

/// The file key.
const KEY: &str = "model_view_start";

/// Claim `key = value` if it is this group's; `walk` sets `true`.
pub(super) fn parse_key(walk: &mut bool, key: &str, value: &str) -> KeyOutcome {
    if key != KEY {
        return KeyOutcome::NotMine;
    }
    match value.trim() {
        "walk" => *walk = true,
        "orbit" => *walk = false,
        _ => return KeyOutcome::BadValue,
    }
    KeyOutcome::Accepted
}

/// Append this group's block to a preferences file.
pub(super) fn write_block(walk: bool, out: &mut String) {
    out.push_str(
        "\n\
         # model_view_start: walk or orbit (default walk). How the 3D viewer\n\
         # opens: walking with ghost on, or orbiting the model.\n",
    );
    let value = if walk { "walk" } else { "orbit" };
    out.push_str(&format!("{KEY} = {value}\n")); // ui-text-exempt: a preferences-file line, never displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_starts_round_trip_and_others_are_refused() {
        for choice in [true, false] {
            let mut out = String::new();
            write_block(choice, &mut out);
            let (key, value) = out.lines().last().unwrap().split_once(" = ").unwrap();
            let mut back = !choice;
            assert!(matches!(
                parse_key(&mut back, key, value),
                KeyOutcome::Accepted
            ));
            assert_eq!(back, choice);
        }
        let mut walk = true;
        assert!(matches!(
            parse_key(&mut walk, KEY, "fly"),
            KeyOutcome::BadValue
        ));
        assert!(matches!(
            parse_key(&mut walk, "other", "walk"),
            KeyOutcome::NotMine
        ));
        assert!(walk);
    }

    #[test]
    fn the_viewer_opens_walking_by_default() {
        assert!(crate::prefs::Prefs::default().model_start_walk);
    }
}
