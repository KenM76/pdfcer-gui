//! # `dialogs::settings::redaction` — the one control that decides what is destroyed
//!
//! One setting: how far applying a redaction may reach beyond the marked
//! regions. It files here, and not under *Saving files*, because this window
//! files by the **symptom that brings an operator looking** (`super`'s header)
//! and the symptom is *"redacting a word took it off a sheet I never
//! touched"* — or its mirror, *"I redacted it and pdfcer says a copy is still
//! in the file"*. Both are facts about redaction, and neither reads as a fact
//! about writing a file.
//!
//! ## Why it is last
//!
//! Group order in this window runs from what an operator changes often to what
//! they change once. This is changed once or never, and it is the only control
//! in the window whose wrong answer cannot be undone by reopening the document
//! with the right one — so it sits where a reader arrives deliberately rather
//! than where a reader scrolling past can click it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/settings/redaction.md`.

use egui::Ui;

use super::widgets;
use crate::app::prefs::{Prefs, RedactionReach};
use crate::text::settings as t;

/// How far a redaction reaches beyond the marked regions.
pub fn residual_reach(ui: &mut Ui, prefs: &mut Prefs) {
    widgets::header(ui, t::reach_title(), t::reach_silence(), t::reach_radius());
    for option in RedactionReach::ALL {
        widgets::option(
            ui,
            &mut prefs.redaction_reach,
            *option,
            t::reach_label(*option),
            Some(t::reach_note(*option)),
        );
    }
}
