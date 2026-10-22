//! # `dialogs::model_trust` — the prompt a 3D model raises while Play 3D content is off
//!
//! Acrobat's document message bar as a window: *3D content has been disabled.
//! Enable this feature if you trust this document.*, with *Trust this document
//! one time only* (the default), *Trust this document always* and Cancel.
//! Escape and the close button are Cancel. A trusting answer raises
//! [`OutwardAction::ModelAnswer`] and then the `ViewModel` that raised the
//! prompt, so the model shows through the same choke point it was stopped at.
//!
//! Design and rationale: `DESIGNS.md` § O320.

use pdfcer_core::threed::ThreeDArtwork;
use pdfcer_gui_base::outwardaction::{ModelTrust, OutwardAction};

use crate::app::actions::Action;
use crate::app::actions::attachments::AttachmentAction;
use crate::text::outward as t;

/// The window body's published region, for `ui-verify`.
const REGION_BODY: &str = "model_trust.body";

/// The prompt for one model.
pub struct ModelTrustDialog {
    artwork: ThreeDArtwork,
    answer: Option<ModelTrust>,
}

impl ModelTrustDialog {
    /// The prompt for `artwork`.
    #[must_use]
    pub const fn new(artwork: ThreeDArtwork) -> Self {
        Self {
            artwork,
            answer: None,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = pdfcer_gui_base::dialoghost::Host::new(
            "model_trust", // ui-text-exempt: a viewport key, never displayed.
            t::model_title(),
            egui::vec2(520.0, 160.0),
            egui::vec2(420.0, 140.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let trust = match self.answer {
            Some(trust) => trust,
            None if frame.closed => ModelTrust::No,
            None => return true,
        };
        actions.extend(self.answered(trust));
        false
    }

    /// The actions an answer raises.
    fn answered(&self, trust: ModelTrust) -> Vec<Action> {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("model3d-answer trust={}", trust_word(trust))
        });
        let mut raised = vec![Action::Outward(OutwardAction::ModelAnswer { trust })];
        if trust != ModelTrust::No {
            raised.push(Action::Attachment(AttachmentAction::ViewModel {
                artwork: self.artwork.clone(),
            }));
        }
        raised
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        ui.label(t::model_disabled());
        ui.separator();
        let (once, refuse, always) = pdfcer_gui_base::dialoghost::Host::footer(
            ui,
            (t::model_trust_once(), t::model_trust_once_hover()),
            (t::model_refuse(), ""),
            Some((t::model_trust_always(), t::model_trust_always_hover())),
        );
        if once {
            self.answer = Some(ModelTrust::Once);
        } else if always {
            self.answer = Some(ModelTrust::Always);
        } else if refuse {
            self.answer = Some(ModelTrust::No);
        }
    }
}

/// The trace's word for an answer.
const fn trust_word(trust: ModelTrust) -> &'static str {
    match trust {
        ModelTrust::No => "no",         // ui-text-exempt: trace token
        ModelTrust::Once => "once",     // ui-text-exempt: trace token
        ModelTrust::Always => "always", // ui-text-exempt: trace token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words `ui-verify` matches on; `trust_word` is an exhaustive match.
    #[test]
    fn the_answers_trace_the_words_the_harness_reads() {
        let words = [ModelTrust::No, ModelTrust::Once, ModelTrust::Always].map(trust_word);
        assert_eq!(words, ["no", "once", "always"]);
    }
}
