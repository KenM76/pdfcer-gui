//! # `dialogs::compact_classes` — picture settings per kind of picture
//!
//! Contract: [`Classes`] holds one [`Pictures`] each for colour, grey and
//! black-and-white pictures, the split Acrobat's PDF Optimizer makes. While
//! [`Classes::separate`] is on, [`Classes::rules`] is the engine's per-class
//! `Rule` list and replaces the window's single set of picture choices.
//! Palette pictures count as colour. Under JPEG they are stored the smallest
//! exact way instead, as the engine's own presets do, because the engine
//! stores no palette picture as JPEG.

use egui::Ui;
use pdfcer_core::optimize::{ImageClass, ImageEncode, ImagePatch, Rule, Selector, StreamAction};

use crate::dialogs::compact_images::{self, Format, Pictures, REGION_FOLD};
use crate::text::compact_classes as t;

/// The split switch.
pub const REGION_SPLIT: &str = "compact.pictures.split";

/// One kind of picture, as the engine classes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// More than one colour component, and palette pictures.
    Colour,
    /// One component, more than 1 bit.
    Grey,
    /// 1 bit, or an image mask.
    Mono,
}

impl Kind {
    /// Every kind, in the order the window shows them.
    pub const ALL: [Self; 3] = [Self::Colour, Self::Grey, Self::Mono];

    /// The kind's name in a region (`compact.pictures.<key>.…`), never displayed.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Colour => "colour",
            Self::Grey => "grey",
            Self::Mono => "mono",
        }
    }

    /// The group's heading.
    const fn heading(self) -> &'static str {
        match self {
            Self::Colour => t::colour_heading(),
            Self::Grey => t::grey_heading(),
            Self::Mono => t::mono_heading(),
        }
    }
}

/// Separate picture settings for each [`Kind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Classes {
    /// Whether the three below replace the window's one set of choices.
    pub separate: bool,
    /// Colour and palette pictures.
    pub colour: Pictures,
    /// Greyscale pictures.
    pub grey: Pictures,
    /// Black-and-white pictures.
    pub mono: Pictures,
}

impl Classes {
    /// Not split, opening on the engine's Standard resolutions: colour and
    /// grey at 150 ppi, black and white at 300 stored the smallest exact way.
    pub const OFF: Self = Self {
        separate: false,
        colour: Pictures {
            on: true,
            ..Pictures::OFF
        },
        grey: Pictures {
            on: true,
            ..Pictures::OFF
        },
        mono: Pictures {
            on: true,
            ppi: Some(300),
            format: Format::Smallest,
            ..Pictures::OFF
        },
    };

    /// The settings for `kind`.
    #[must_use]
    pub const fn of(self, kind: Kind) -> Pictures {
        match kind {
            Kind::Colour => self.colour,
            Kind::Grey => self.grey,
            Kind::Mono => self.mono,
        }
    }

    const fn of_mut(&mut self, kind: Kind) -> &mut Pictures {
        match kind {
            Kind::Colour => &mut self.colour,
            Kind::Grey => &mut self.grey,
            Kind::Mono => &mut self.mono,
        }
    }

    /// The engine's per-class rules; `None` while not split. A kind switched
    /// off gets no rule, so its pictures are left as they are.
    ///
    /// # Errors
    ///
    /// The engine's sentence when a kind's choices are refused.
    pub fn rules(self) -> Result<Option<Vec<Rule>>, String> {
        if !self.separate {
            return Ok(None);
        }
        let mut rules = Vec::new();
        for kind in Kind::ALL {
            let Some(patch) = self.of(kind).patch() else {
                continue;
            };
            patch.validate()?;
            match kind {
                Kind::Colour => {
                    rules.push(rule(ImageClass::Color, patch));
                    let mut palette = patch;
                    if self.colour.format == Format::Jpeg {
                        palette.encode = Some(ImageEncode::Auto);
                        palette.quality = None;
                    }
                    rules.push(rule(ImageClass::Indexed, palette));
                }
                Kind::Grey => rules.push(rule(ImageClass::Gray, patch)),
                Kind::Mono => rules.push(rule(ImageClass::Mono, patch)),
            }
        }
        Ok(Some(rules))
    }
}

/// A rule giving pictures of `class` the patch `patch`.
fn rule(class: ImageClass, patch: ImagePatch) -> Rule {
    let mut select = Selector::new();
    select.image_classes = vec![class];
    Rule {
        select,
        action: StreamAction::Image(patch),
    }
}

/// The split switch and, while it is on, one group per kind; whether the split
/// is on, so the caller draws its single set of rows only when it is not.
pub fn show(ui: &mut Ui, classes: &mut Classes) -> bool {
    let split = ui
        .checkbox(&mut classes.separate, t::split())
        .on_hover_text(t::split_tooltip());
    crate::diag::ui_rect_visible(REGION_SPLIT, split.rect, ui.clip_rect());
    if !classes.separate {
        return false;
    }
    for kind in Kind::ALL {
        let stem = format!("{REGION_FOLD}.{}", kind.key());
        let p = classes.of_mut(kind);
        ui.push_id(kind.key(), |ui| {
            let on = ui.checkbox(&mut p.on, kind.heading());
            crate::diag::ui_rect_visible(&format!("{stem}.on"), on.rect, ui.clip_rect());
            ui.add_enabled_ui(p.on, |ui| {
                egui::Grid::new("compact-class-grid") // ui-text-exempt: an id, never displayed
                    .show(ui, |ui| {
                        compact_images::rows_in(ui, p, &stem, kind == Kind::Mono);
                    });
            });
        });
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classes_of(rules: &[Rule]) -> Vec<ImageClass> {
        rules
            .iter()
            .flat_map(|r| r.select.image_classes.iter().copied())
            .collect()
    }

    /// Not split asks for no rules, whatever the kinds hold.
    #[test]
    fn not_split_builds_no_rules() {
        assert_eq!(Classes::OFF.rules(), Ok(None));
    }

    /// Each kind switched on gets a rule for its own class only; palette
    /// pictures ride with colour.
    #[test]
    fn each_kind_rules_its_own_class() {
        let split = Classes {
            separate: true,
            ..Classes::OFF
        };
        let rules = split.rules().expect("accepted").expect("split");
        assert_eq!(
            classes_of(&rules),
            [
                ImageClass::Color,
                ImageClass::Indexed,
                ImageClass::Gray,
                ImageClass::Mono
            ]
        );
        let StreamAction::Image(mono) = rules[3].action else {
            panic!("an image rule");
        };
        assert_eq!(
            (mono.ppi, mono.encode),
            (Some(300), Some(ImageEncode::Auto))
        );
    }

    /// A kind switched off is left alone: no rule names its class.
    #[test]
    fn a_kind_switched_off_gets_no_rule() {
        let mut split = Classes {
            separate: true,
            ..Classes::OFF
        };
        split.grey.on = false;
        let rules = split.rules().expect("accepted").expect("split");
        assert!(!classes_of(&rules).contains(&ImageClass::Gray));
        assert!(classes_of(&rules).contains(&ImageClass::Mono));
    }

    /// Colour as JPEG stores palette pictures exactly, with no quality.
    #[test]
    fn jpeg_colour_keeps_palettes_exact() {
        let mut split = Classes {
            separate: true,
            ..Classes::OFF
        };
        split.colour.format = Format::Jpeg;
        let rules = split.rules().expect("accepted").expect("split");
        let StreamAction::Image(colour) = rules[0].action else {
            panic!("an image rule");
        };
        let StreamAction::Image(palette) = rules[1].action else {
            panic!("an image rule");
        };
        assert_eq!(colour.encode, Some(ImageEncode::Jpeg));
        assert_eq!(
            (palette.encode, palette.quality),
            (Some(ImageEncode::Auto), None)
        );
    }
}
