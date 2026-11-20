//! Which parts of the live picture the graphics card draws, and the Look
//! tab's switches for them. A frame showing a part switched off is declined
//! to the processor, which draws the same picture more slowly; the whole
//! picture switched off is drawn by the processor every frame. Saved
//! pictures are the processor's whatever these say.

use egui::Ui;

use pdfcer_gui_base::cardstatus::{Outcome, Path, note};

use super::ModelView;
use crate::text::cardstatus as cs;
use crate::text::model3dlook::{self as k, CardPart, ProcessorReason};

/// `PDFCER_3D_CPU=1` keeps the 3D viewer off the card for the session.
const CPU_VAR: &str = "PDFCER_3D_CPU";

/// Whether [`CPU_VAR`] keeps the 3D viewer off the card.
pub(crate) fn forced_by_environment() -> bool {
    std::env::var_os(CPU_VAR).is_some_and(|v| v == "1")
}

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
        ui.label(k::CARD_HEADING);
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
    /// The card's switches as they stand.
    pub(crate) fn card(&self) -> CardParts {
        self.gpu.card
    }

    /// Set the card's switches from outside the Look tab.
    pub(crate) fn set_card(&mut self, card: CardParts) {
        let before = self.gpu.card;
        self.gpu.card = card;
        self.card_changed(before);
    }

    /// The Look tab's graphics card switches.
    pub(super) fn card_control(&mut self, ui: &mut Ui) {
        let forced = self.gpu.forced_off();
        let before = self.gpu.card;
        self.gpu.card.controls(ui, forced);
        self.card_changed(before);
    }

    /// Record what the picture and each part did this frame, `gpu` being
    /// whether the frame was offered to the card.
    pub(super) fn note_card_status(&self, gpu: bool) {
        let whole = if self.gpu.forced_off() {
            Some(Outcome::ForcedOff(CPU_VAR))
        } else if self.gpu.turned_off() {
            Some(Outcome::Off)
        } else if let Some(why) = self.gpu.failure() {
            Some(Outcome::Processor(why))
        } else if !gpu {
            Some(Outcome::Processor(cs::light_map_shown().to_owned()))
        } else {
            None
        };
        let declined = self.gpu.declined();
        let picture = whole.clone().unwrap_or_else(|| {
            declined.map_or(Outcome::InUse, |r| {
                Outcome::Processor(k::processor_reason(r).to_owned())
            })
        });
        for (i, part) in CardPart::ALL.into_iter().enumerate() {
            let outcome = whole.clone().unwrap_or_else(|| match declined {
                Some(ProcessorReason::TurnedOff(p)) if p == part => {
                    Outcome::Processor(cs::part_refused().to_owned())
                }
                Some(ProcessorReason::WidePoints) if part == CardPart::Lines => {
                    Outcome::Processor(cs::wide_points(self.gpu.widest_point()))
                }
                _ if !self.gpu.card.parts[i] => Outcome::Off,
                Some(r) => Outcome::Processor(k::processor_reason(r).to_owned()),
                None => Outcome::InUse,
            });
            note(Path::Part(i), outcome);
        }
        note(Path::Picture, picture);
    }

    /// Trace a change of the card's switches from `before`.
    fn card_changed(&self, before: CardParts) {
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
