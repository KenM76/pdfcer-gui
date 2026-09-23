//! # `theme::icon_accents` — the optional second colour of a toolbar glyph
//!
//! An application that offers "colourful" icons, in the manner of Microsoft
//! Office and SOLIDWORKS toolbars, draws each glyph's outline in the ordinary
//! foreground and one meaningful part of it — the plus on *New*, the cross on
//! *Close*, the arrow on *Undo* — in one of four muted hues. This module owns
//! those four hues. Which part of which glyph takes which one is the
//! application's decision; the shell knows only the roles.
//!
//! ## Contract
//!
//! - Four **roles**, never colour names, for the reason [`super::Palette`]
//!   gives: a preset must be free to move a role without its field lying.
//! - Every role clears [`super::contrast::READABLE_LUMA_GAP`] against both
//!   [`super::Palette::surface`] and [`super::Palette::panel`] in every
//!   preset — the grounds a toolbar glyph is drawn on. The test below holds
//!   it, so a preset cannot ship an accent that vanishes into its own chrome.
//! - Nothing here is on by default. An application that never asks for these
//!   draws exactly what it drew before.

use egui::Color32;

use super::{Preset, Theme};

/// The four hues an icon accent may take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconAccents {
    /// The ordinary accent of a tool: arrows, handles, the working part.
    /// Blue in every shipped preset.
    pub primary: Color32,
    /// Something is added, created or confirmed. Green.
    pub affirm: Color32,
    /// Something is removed, closed or struck out. Red.
    pub remove: Color32,
    /// Something is marked or kept: a highlighter, a note, a folder, a lock.
    /// Amber.
    pub mark: Color32,
}

impl IconAccents {
    /// The accents for `preset`.
    ///
    /// The light presets share one set, darkened from the Office hues until
    /// each clears the contrast floor on a near-white panel; the dark preset
    /// lifts them by the same measure so they read on a dark one.
    #[must_use]
    pub fn for_preset(preset: Preset) -> Self {
        match preset {
            Preset::Quiet | Preset::Airy => Self {
                primary: Color32::from_rgb(0x2B, 0x6C, 0xC4),
                affirm: Color32::from_rgb(0x2E, 0x8B, 0x45),
                remove: Color32::from_rgb(0xC8, 0x37, 0x2F),
                mark: Color32::from_rgb(0xB0, 0x72, 0x16),
            },
            Preset::Dark => Self {
                primary: Color32::from_rgb(0x6F, 0xA8, 0xF0),
                affirm: Color32::from_rgb(0x62, 0xC2, 0x7A),
                remove: Color32::from_rgb(0xF0, 0x7A, 0x70),
                mark: Color32::from_rgb(0xE3, 0xB0, 0x4A),
            },
        }
    }

    /// The accents of the theme in force on `ctx`.
    #[must_use]
    pub fn of(ctx: &egui::Context) -> Self {
        Self::for_preset(Theme::of(ctx).preset)
    }

    /// All four, for tests and for anything that must treat them alike.
    #[must_use]
    pub const fn all(self) -> [Color32; 4] {
        [self.primary, self.affirm, self.remove, self.mark]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::contrast::{READABLE_LUMA_GAP, gap};

    #[test]
    fn every_accent_reads_on_the_chrome_it_is_drawn_on() {
        for &preset in Preset::ALL {
            let palette = Theme::new(preset).palette;
            for (i, hue) in IconAccents::for_preset(preset)
                .all()
                .into_iter()
                .enumerate()
            {
                for (ground, bg) in [("surface", palette.surface), ("panel", palette.panel)] {
                    let g = gap(hue, bg);
                    assert!(
                        g >= READABLE_LUMA_GAP,
                        "{preset:?} accent {i} on {ground}: gap {g:.1} < {READABLE_LUMA_GAP}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_four_roles_are_distinct_in_every_preset() {
        for &preset in Preset::ALL {
            let hues = IconAccents::for_preset(preset).all();
            for a in 0..hues.len() {
                for b in a + 1..hues.len() {
                    assert_ne!(
                        hues[a], hues[b],
                        "{preset:?}: roles {a} and {b} share a colour"
                    );
                }
            }
        }
    }
}
