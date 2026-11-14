//! What the vector print route may keep as vector graphics, and its lines in
//! `preferences.txt`. A group of [`super::printing::PrintPrefs`]; its parser
//! and writer are called from that group's.
//!
//! Both switches default on, where the engine's `EmfOptions` defaults them
//! off: the engine's defaults keep a library caller's output unchanged
//! between versions, while an operator printing a drawing wants it sharp.
//! Either can be turned off for a printer whose driver draws gradient
//! triangles or many small regions wrongly.

use super::printing::{KeyOutcome, PrintPrefs};

/// The vector route's switches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VectorRoute {
    /// Smooth shadings go as triangles (`EmfOptions::vector_shadings`).
    pub gradients: bool,
    /// Flat transparency goes as opaque regions, with pixels only where it
    /// cannot be resolved (`EmfOptions::vector_transparency`).
    pub transparency: bool,
    /// Those pixels cover only where a soft mask paints
    /// (`EmfOptions::tight_mask_patches`). Off by default: text freed from a
    /// patch is drawn by the driver, which does not smooth it.
    pub tight_patches: bool,
}

impl Default for VectorRoute {
    fn default() -> Self {
        Self {
            gradients: true,
            transparency: true,
            tight_patches: false,
        }
    }
}

// ui-text-exempt: file KEYS, parsed out of preferences.txt.
const GRADIENTS: &str = "print_vector_gradients";
// ui-text-exempt: file KEYS, as above.
const TRANSPARENCY: &str = "print_vector_transparency";
// ui-text-exempt: file KEYS, as above.
const TIGHT: &str = "print_tight_patches";

/// Read one line into [`PrintPrefs::vector`], if it is this group's.
pub(super) fn parse_key(prefs: &mut PrintPrefs, key: &str, value: &str) -> KeyOutcome {
    let slot = match key {
        GRADIENTS => &mut prefs.vector.gradients,
        TRANSPARENCY => &mut prefs.vector.transparency,
        TIGHT => &mut prefs.vector.tight_patches,
        _ => return KeyOutcome::NotMine,
    };
    match super::opening::bool_from_key(value) {
        Some(on) => {
            *slot = on;
            KeyOutcome::Accepted
        }
        None => KeyOutcome::BadValue,
    }
}

/// Append this group's lines.
pub(super) fn write_block(prefs: &PrintPrefs, out: &mut String) {
    out.push_str(
        "# print_vector_gradients: true | false. Send smooth colour gradients\n\
         # as vector graphics; false sends a page with one as a picture.\n\
         # print_vector_transparency: true | false. Send see-through and\n\
         # blended artwork as vector graphics, with small pictures only where\n\
         # it cannot be; false sends such a page as a picture.\n\
         # print_tight_patches: true | false. Make those pictures smaller; the\n\
         # text they no longer cover may print less smooth.\n",
    );
    for (key, on) in [
        (GRADIENTS, prefs.vector.gradients),
        (TRANSPARENCY, prefs.vector.transparency),
        (TIGHT, prefs.vector.tight_patches),
    ] {
        out.push_str(key);
        out.push_str(" = ");
        out.push_str(super::opening::bool_key(on));
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_keep_gradients_and_transparency_vector() {
        let route = VectorRoute::default();
        assert!(route.gradients && route.transparency && !route.tight_patches);
    }

    #[test]
    fn every_switch_round_trips() {
        let prefs = PrintPrefs {
            vector: VectorRoute {
                gradients: false,
                transparency: false,
                tight_patches: true,
            },
            ..PrintPrefs::default()
        };
        let mut out = String::new();
        write_block(&prefs, &mut out);
        let mut back = PrintPrefs::default();
        for line in out.lines().filter(|l| !l.starts_with('#')) {
            let (key, value) = line.split_once(" = ").expect("key = value");
            assert_eq!(parse_key(&mut back, key, value), KeyOutcome::Accepted);
        }
        assert_eq!(back.vector, prefs.vector);
    }

    #[test]
    fn a_bad_value_is_reported_and_leaves_the_default() {
        let mut prefs = PrintPrefs::default();
        assert_eq!(
            parse_key(&mut prefs, GRADIENTS, "maybe"),
            KeyOutcome::BadValue
        );
        assert!(prefs.vector.gradients);
        assert_eq!(parse_key(&mut prefs, "other", "true"), KeyOutcome::NotMine);
    }
}
