//! # `dialogs::weblink` — the Security Warning a web link raises
//!
//! Acrobat's prompt, word for word: the address in full, Allow or Block, and a
//! checkbox remembering the answer for the site. Enter allows; Escape and the
//! close button block, so dismissing the prompt never opens anything. The
//! answer is raised as [`OutwardAction::WebAnswer`]; remembering and opening
//! happen in the apply phase, beside the preferences they write.
//!
//! Design and rationale: `DESIGNS.md` § O320.

use pdfcer_gui_base::outwardaction::OutwardAction;

use crate::app::actions::Action;
use crate::text::outward as t;

/// The window body's published region, for `ui-verify`.
const REGION_BODY: &str = "weblink.body";
/// The remember checkbox's region.
pub const REGION_REMEMBER: &str = "weblink.remember";

/// The prompt for one address.
pub struct WebLinkDialog {
    uri: String,
    remember: bool,
    answer: Option<bool>,
}

impl WebLinkDialog {
    /// The prompt for `uri`.
    #[must_use]
    pub const fn new(uri: String) -> Self {
        Self {
            uri,
            remember: false,
            answer: None,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = pdfcer_gui_base::dialoghost::Host::new(
            "weblink", // ui-text-exempt: a viewport key, never displayed.
            t::web_title(),
            egui::vec2(480.0, 240.0),
            egui::vec2(380.0, 200.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        // Closing the window is Block, as it is in Acrobat.
        let allow = match self.answer {
            Some(allow) => allow,
            None if frame.closed => false,
            None => return true,
        };
        actions.push(self.answered(allow));
        false
    }

    /// The action an answer raises.
    fn answered(&self, allow: bool) -> Action {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("weblink-answer allow={allow} remember={}", self.remember)
        });
        Action::Outward(OutwardAction::WebAnswer {
            uri: self.uri.clone(),
            allow,
            remember: self.remember,
        })
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        ui.label(t::web_question(&self.uri));
        if !pdfcer_gui_base::outward::is_web(&self.uri) {
            let scheme = pdfcer_gui_base::outward::scheme(&self.uri).unwrap_or_default();
            ui.label(t::web_not_a_site(&scheme));
        }
        ui.add_space(6.0);
        let check = ui
            .checkbox(&mut self.remember, t::web_remember())
            .on_hover_text(t::web_remember_hover());
        crate::diag::ui_rect(REGION_REMEMBER, check.rect);
        ui.separator();
        let (allow, block, _) = pdfcer_gui_base::dialoghost::Host::footer(
            ui,
            (t::web_allow(), ""),
            (t::web_block(), ""),
            None,
        );
        if allow {
            self.answer = Some(true);
        } else if block {
            self.answer = Some(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer carries the address and the checkbox exactly.
    #[test]
    fn the_answer_carries_the_address_and_the_checkbox() {
        let mut dialog = WebLinkDialog::new("https://example.com/x".to_owned());
        dialog.remember = true;
        assert_eq!(
            dialog.answered(false),
            Action::Outward(OutwardAction::WebAnswer {
                uri: "https://example.com/x".to_owned(),
                allow: false,
                remember: true,
            })
        );
    }
}
