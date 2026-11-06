//! Which parts of the live picture the graphics card draws, and the Look
//! tab's switches for them. A frame showing a part switched off is declined
//! to the processor, which draws the same picture more slowly; the whole
//! picture switched off is drawn by the processor every frame. Saved
//! pictures are the processor's whatever these say.

use egui::Ui;

use super::ModelView;
use crate::text::model3dlook::{self as k, CardPart};

/// The whole-picture switch.
pub const REGION_CARD_PICTURE: &str = "model3d.card.picture"; // ui-text-exempt: trace region name, never displayed
/// A part's switch, after this prefix its index in [`CardPart::ALL`].
pub const REGION_CARD_PART: &str = "model3d.card.part."; // ui-text-exempt: trace region name, never displayed

/// The card's switches: the whole picture, and each part in
/// [`CardPart::ALL`]'s order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CardParts {
    pub(crate) picture: bool,
    pub(crate) parts: [bool; CardPart::ALL.len()],
}

impl Default for CardParts {
    fn default() -> Self {
        Self {
            picture: true,
            parts: [true; CardPart::ALL.len()],
        }
    }
}

impl CardParts {
    /// The first part `shown` says the frame draws that the card is switched
    /// off for.
    pub(super) fn refused(&self, shown: impl Fn(CardPart) -> bool) -> Option<CardPart> {
        CardPart::ALL
            .into_iter()
            .zip(self.parts)
            .find(|&(part, on)| !on && shown(part))
            .map(|(part, _)| part)
    }

    /// The switches; `forced` greys the whole picture's with its reason.
    pub(crate) fn controls(&mut self, ui: &mut Ui, forced: bool) {
        ui.label(egui::RichText::new(k::CARD_HEADING).strong());
        let whole = ui
            .add_enabled(
                !forced,
                egui::Checkbox::new(&mut self.picture, k::CARD_PICTURE),
            )
            .on_hover_text(k::CARD_PICTURE_TOOLTIP)
            .on_disabled_hover_text(k::CARD_FORCED_OFF_TOOLTIP);
        crate::diag::ui_rect_visible(REGION_CARD_PICTURE, whole.rect, ui.clip_rect());
        let live = self.picture && !forced;
        ui.label(k::CARD_PARTS);
        ui.indent("model3d-card-parts", |ui| {
            for (i, (part, on)) in CardPart::ALL.into_iter().zip(&mut self.parts).enumerate() {
                let r = ui
                    .add_enabled(live, egui::Checkbox::new(on, k::card_part(part)))
                    .on_hover_text(k::CARD_PART_TOOLTIP)
                    .on_disabled_hover_text(k::CARD_PARTS_OFF_TOOLTIP);
                // ui-text-exempt: trace region name, never displayed
                let region = format!("{REGION_CARD_PART}{i}");
                crate::diag::ui_rect_visible(&region, r.rect, ui.clip_rect());
            }
        });
    }
}

impl ModelView {
    /// The Look tab's graphics card switches.
    pub(super) fn card_control(&mut self, ui: &mut Ui) {
        let forced = self.gpu.forced_off();
        let before = self.gpu.card;
        self.gpu.card.controls(ui, forced);
        if self.gpu.card != before {
            let c = self.gpu.card;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "model-card picture={} parts={:?}",
                    u8::from(c.picture),
                    c.parts.map(u8::from)
                )
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_part_switched_off_is_refused_only_when_the_frame_shows_it() {
        let mut card = CardParts::default();
        assert_eq!(card.refused(|_| true), None);
        card.parts[3] = false;
        assert_eq!(card.refused(|p| p == CardPart::Shadows), None);
        assert_eq!(
            card.refused(|p| p == CardPart::Occlusion),
            Some(CardPart::Occlusion)
        );
        assert_eq!(CardPart::ALL[3], CardPart::Occlusion);
    }
}
