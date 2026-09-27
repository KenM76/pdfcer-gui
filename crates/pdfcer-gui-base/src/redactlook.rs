//! # `redactlook` — what a redaction looks like once applied
//!
//! The operator's **one** choice of fill colour and overlay caption, held for
//! the whole redaction panel and applied to every mark authored from it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/redact/appearance.md`.

use pdfcer_core::annot_author::{Color, RedactAppearance};
use pdfcer_core::vartext::Quadding;

/// The longest overlay caption offered.
pub const MAX_OVERLAY_CHARS: usize = 64;

/// The operator's choice of how an applied redaction looks.
#[derive(Debug, Clone, PartialEq)]
pub struct Appearance {
    /// What fills the redacted region on apply.
    pub fill: Fill,
    /// The caption drawn over the fill, or empty for none.
    ///
    /// Stored as a `String` rather than an `Option<String>` because that is
    /// what a text field edits; [`Self::to_core`] is the single place the
    /// empty-means-none rule is applied, so a caller cannot get it wrong.
    pub overlay_text: String,
    /// How the caption is justified inside the box.
    pub quadding: Quadding,
}

impl Default for Appearance {
    /// **Black, no caption, left-justified** — what every mark this shell has
    /// ever authored applied as, before the choice existed.
    fn default() -> Self {
        Self {
            fill: Fill::Black,
            overlay_text: String::new(),
            quadding: Quadding::Left,
        }
    }
}

/// What fills a redacted region once the redaction is applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    /// A solid black box. The convention, and what pdfcer has always drawn.
    Black,
    /// A solid white box, for a redaction that should read as blank paper.
    White,
    /// A solid box in a colour the operator picked.
    ///
    /// Carried as RGB components in `0.0..=1.0`, matching
    /// `pdfcer_core::annot_author::Color::Rgb`, so no conversion happens
    /// anywhere but [`Fill::to_core`].
    Custom(f64, f64, f64),
    /// **No box at all.** The content is removed and nothing marks where it
    /// was.
    ///
    /// Offered because ISO 32000-1 Table 192 describes it and because there is
    /// a real use — removing content without advertising its position — but
    /// never the default, and the panel says out loud what it does.
    Transparent,
}

impl Fill {
    /// The engine's form.
    #[must_use]
    pub const fn to_core(self) -> Option<Color> {
        match self {
            Self::Black => Some(Color::Gray(0.0)),
            Self::White => Some(Color::Gray(1.0)),
            Self::Custom(r, g, b) => Some(Color::Rgb(r, g, b)),
            Self::Transparent => None,
        }
    }

    /// Whether a caption drawn over this fill would be legible.
    #[must_use]
    pub fn caption_would_be_legible(self) -> bool {
        let Some(colour) = self.to_core() else {
            // Transparent: the caption is drawn over whatever the page had,
            // which this shell cannot know. Not flagged — an unknown backdrop
            // is not a known-bad one, and a warning that fires on every
            // transparent redaction would be noise.
            return true;
        };
        let luminance = match colour {
            Color::Gray(g) => g,
            // Rec. 709 luma. Approximate on purpose: the question is "is this
            // dark?", not "what is its exact perceived lightness?".
            Color::Rgb(r, g, b) => 0.2126f64.mul_add(r, 0.7152f64.mul_add(g, 0.0722 * b)),
            // Unreachable from `Fill`, and answered rather than panicking: a
            // future fill that is CMYK should degrade to "assume legible"
            // rather than crash a panel.
            Color::Cmyk(..) => 1.0,
        };
        // 0.5 rather than a tuned threshold. The engine draws black text; the
        // question is whether the backdrop is nearer white than black.
        luminance > 0.5
    }
}

impl Appearance {
    /// The engine's form, ready for `add_redaction` or either `_styled` verb.
    ///
    /// The single place `""` becomes `None`, so no call site has to remember
    /// that an empty field means no caption.
    #[must_use]
    pub fn to_core(&self) -> RedactAppearance {
        let trimmed = self.overlay_text.trim();
        RedactAppearance {
            fill: self.fill.to_core(),
            overlay_text: if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            },
            quadding: self.quadding,
        }
    }

    /// Whether the operator has asked for a caption at all.
    ///
    #[must_use]
    pub fn has_overlay(&self) -> bool {
        !self.overlay_text.trim().is_empty()
    }

    /// Whether the operator should be warned before applying.
    ///
    /// True only when there **is** a caption and the fill would make it
    /// illegible. See [`Fill::caption_would_be_legible`].
    #[must_use]
    pub fn caption_is_illegible(&self) -> bool {
        self.has_overlay() && !self.fill.caption_would_be_legible()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The shipped appearance is an explicit black box.**
    #[test]
    fn the_shipped_fill_is_an_explicit_black_box_not_a_none() {
        let core = Appearance::default().to_core();
        assert_eq!(
            core.fill,
            Some(Color::Gray(0.0)),
            "the default redaction fill must be EXPLICIT black — `None` now means \
             transparent, so a redaction would remove the content and draw nothing over it"
        );
        assert!(
            core.overlay_text.is_none(),
            "no caption unless the operator writes one — inventing a default would put \
             words on their page that they did not write"
        );
    }

    /// An empty or whitespace caption is no caption.
    #[test]
    fn a_blank_caption_is_no_caption() {
        for blank in ["", "   ", "\t\n "] {
            let a = Appearance {
                overlay_text: blank.to_owned(),
                ..Appearance::default()
            };
            assert!(a.to_core().overlay_text.is_none(), "{blank:?}");
            assert!(!a.has_overlay(), "{blank:?}");
        }
        let a = Appearance {
            overlay_text: "  REDACTED  ".to_owned(),
            ..Appearance::default()
        };
        assert_eq!(
            a.to_core().overlay_text.as_deref(),
            Some("REDACTED"),
            "a caption must be trimmed, or the justification applies to the spaces"
        );
    }

    /// Every fill maps to the engine colour it names.
    #[test]
    fn every_fill_maps_to_its_engine_colour() {
        assert_eq!(Fill::Black.to_core(), Some(Color::Gray(0.0)));
        assert_eq!(Fill::White.to_core(), Some(Color::Gray(1.0)));
        assert_eq!(
            Fill::Custom(0.8, 0.1, 0.1).to_core(),
            Some(Color::Rgb(0.8, 0.1, 0.1))
        );
        assert_eq!(
            Fill::Transparent.to_core(),
            None,
            "transparent is the ONE fill that is `None`, and it is chosen rather than \
             defaulted into"
        );
    }

    /// **A caption on a dark fill is flagged, including on the default.**
    #[test]
    fn a_caption_on_a_dark_fill_is_flagged() {
        let dark = Appearance {
            overlay_text: "REDACTED".to_owned(),
            ..Appearance::default()
        };
        assert!(
            dark.caption_is_illegible(),
            "black text on the default black box is invisible and must be warned about"
        );

        let dark_red = Appearance {
            fill: Fill::Custom(0.4, 0.0, 0.0),
            overlay_text: "REDACTED".to_owned(),
            quadding: Quadding::Left,
        };
        assert!(
            dark_red.caption_is_illegible(),
            "the engine's own reply names dark red as the case it saw go wrong"
        );

        let light = Appearance {
            fill: Fill::White,
            overlay_text: "REDACTED".to_owned(),
            quadding: Quadding::Left,
        };
        assert!(!light.caption_is_illegible(), "black on white is legible");
    }

    /// …and no caption is never flagged, whatever the fill.
    #[test]
    fn a_fill_with_no_caption_is_never_flagged() {
        for fill in [
            Fill::Black,
            Fill::White,
            Fill::Custom(0.0, 0.0, 0.0),
            Fill::Transparent,
        ] {
            let a = Appearance {
                fill,
                ..Appearance::default()
            };
            assert!(
                !a.caption_is_illegible(),
                "{fill:?} with no caption has nothing to be illegible"
            );
        }
    }
}
