//! Whether printed sheets are drawn with smoothed edges, and its line in
//! `preferences.txt`. A group of [`super::printing::PrintPrefs`]; its parser
//! and writer are called from that group's.

use pdfcer_render::EdgeAntialiasing;

use super::printing::{KeyOutcome, PrintPrefs};

/// The print resolution at and above which [`EdgeChoice::Auto`] draws hard
/// edges: the threshold `pdfcer print --edge-antialiasing auto` uses.
pub const HARD_EDGE_DPI: u32 = 300;

/// How the edges of shapes are drawn on a printed sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeChoice {
    /// Hard edges at [`HARD_EDGE_DPI`] and finer, smoothed below it.
    #[default]
    Auto,
    /// Always smoothed.
    On,
    /// Always hard, so shapes that touch leave no seam.
    Off,
}

/// Every choice, in the order offered, with its file token.
// ui-text-exempt: file tokens, never displayed.
pub const EDGE_CHOICES: [(EdgeChoice, &str); 3] = [
    (EdgeChoice::Auto, "auto"),
    (EdgeChoice::On, "on"),
    (EdgeChoice::Off, "off"),
];

impl EdgeChoice {
    /// The engine's setting for a sheet rendered at `dpi`.
    #[must_use]
    pub const fn at_dpi(self, dpi: u32) -> EdgeAntialiasing {
        match self {
            Self::On => EdgeAntialiasing::On,
            Self::Off => EdgeAntialiasing::Off,
            Self::Auto if dpi >= HARD_EDGE_DPI => EdgeAntialiasing::Off,
            Self::Auto => EdgeAntialiasing::On,
        }
    }

    /// The file token.
    #[must_use]
    pub fn key(self) -> &'static str {
        EDGE_CHOICES
            .iter()
            .find(|(c, _)| *c == self)
            .map_or("auto", |(_, t)| t) // ui-text-exempt: file token
    }
}

/// Read one line into [`PrintPrefs::edges`], if it is this group's.
pub(super) fn parse_key(prefs: &mut PrintPrefs, key: &str, value: &str) -> KeyOutcome {
    // ui-text-exempt: file KEY, parsed out of preferences.txt.
    if key != "print_edge_smoothing" {
        return KeyOutcome::NotMine;
    }
    match EDGE_CHOICES.iter().find(|(_, t)| *t == value) {
        Some((choice, _)) => {
            prefs.edges = *choice;
            KeyOutcome::Accepted
        }
        None => KeyOutcome::BadValue,
    }
}

/// Append this group's line.
pub(super) fn write_block(prefs: &PrintPrefs, out: &mut String) {
    out.push_str(
        "# print_edge_smoothing: auto | on | off. Smoothed edges on printed\n\
         # sheets; auto draws them hard at 300 dpi and finer.\n",
    );
    out.push_str("print_edge_smoothing = "); // ui-text-exempt: file key
    out.push_str(prefs.edges.key());
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_turns_smoothing_off_from_300_dpi() {
        assert!(EdgeChoice::Auto.at_dpi(299).is_on());
        assert!(!EdgeChoice::Auto.at_dpi(300).is_on());
        assert!(EdgeChoice::On.at_dpi(1200).is_on());
        assert!(!EdgeChoice::Off.at_dpi(72).is_on());
    }

    #[test]
    fn every_choice_round_trips_through_its_token() {
        for (choice, token) in EDGE_CHOICES {
            let mut prefs = PrintPrefs::default();
            assert_eq!(
                parse_key(&mut prefs, "print_edge_smoothing", token),
                KeyOutcome::Accepted
            );
            assert_eq!(prefs.edges, choice);
            let mut out = String::new();
            write_block(&prefs, &mut out);
            assert!(out.contains(&format!("print_edge_smoothing = {token}\n")));
        }
    }
}
