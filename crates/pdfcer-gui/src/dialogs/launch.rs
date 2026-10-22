//! # `dialogs::launch` — the Launch File prompt a `/Launch` link raises
//!
//! Acrobat's prompt: the file in full, *Do not show this message again*, and
//! Open / Do Not Open. Escape and the close button are Do Not Open. The answer
//! is raised as [`OutwardAction::LaunchAnswer`]; opening and the preference
//! write happen in the apply phase.
//!
//! Design and rationale: `DESIGNS.md` § O320.

use std::path::PathBuf;

use pdfcer_gui_base::outwardaction::OutwardAction;

use crate::app::actions::Action;
use crate::text::outward as t;

/// The window body's published region, for `ui-verify`.
const REGION_BODY: &str = "launch.body";
/// The *Do not show this message again* checkbox's region.
pub const REGION_DONT_ASK: &str = "launch.dont_ask";

/// The prompt for one file.
pub struct LaunchDialog {
    file: PathBuf,
    dont_ask: bool,
    answer: Option<bool>,
}

impl LaunchDialog {
    /// The prompt for `file`.
    #[must_use]
    pub const fn new(file: PathBuf) -> Self {
        Self {
            file,
            dont_ask: false,
            answer: None,
        }
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = pdfcer_gui_base::dialoghost::Host::new(
            "launch", // ui-text-exempt: a viewport key, never displayed.
            t::launch_title(),
            egui::vec2(500.0, 280.0),
            egui::vec2(400.0, 230.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        let open = match self.answer {
            Some(open) => open,
            None if frame.closed => false,
            None => return true,
        };
        actions.push(self.answered(open));
        false
    }

    /// The action an answer raises.
    fn answered(&self, open: bool) -> Action {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("launch-answer open={open} dont_ask={}", self.dont_ask)
        });
        Action::Outward(OutwardAction::LaunchAnswer {
            file: self.file.clone(),
            open,
            dont_ask: self.dont_ask,
        })
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        ui.label(t::launch_question(&self.file.display().to_string()));
        if pdfcer_gui_base::outward::is_pdf(&self.file) {
            ui.label(t::launch_is_pdf());
        }
        ui.add_space(6.0);
        let check = ui
            .checkbox(&mut self.dont_ask, t::launch_dont_ask())
            .on_hover_text(t::launch_dont_ask_hover());
        crate::diag::ui_rect(REGION_DONT_ASK, check.rect);
        ui.separator();
        let (open, refuse, _) = pdfcer_gui_base::dialoghost::Host::footer(
            ui,
            (t::launch_open(), ""),
            (t::launch_refuse(), ""),
            None,
        );
        if open {
            self.answer = Some(true);
        } else if refuse {
            self.answer = Some(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer carries the file and the checkbox exactly.
    #[test]
    fn the_answer_carries_the_file_and_the_checkbox() {
        let mut dialog = LaunchDialog::new(PathBuf::from("D:/x/payload.exe"));
        dialog.dont_ask = true;
        assert_eq!(
            dialog.answered(false),
            Action::Outward(OutwardAction::LaunchAnswer {
                file: PathBuf::from("D:/x/payload.exe"),
                open: false,
                dont_ask: true,
            })
        );
    }
}
