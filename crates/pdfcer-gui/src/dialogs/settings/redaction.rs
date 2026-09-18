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

use egui::Ui;

use super::widgets;
use crate::app::prefs::{Prefs, RedactionReach};
use crate::text::settings as t;

/// How far a redaction reaches beyond the marked regions.
///
/// # Why the choice is here and not in the apply dialog
///
/// The apply dialog performs the removal the moment it opens — it exists to
/// show a completed rewrite and ask whether to keep it — so a control inside it
/// would be answering a question that has already been decided. The value has
/// to be in force before that window appears, which makes it a preference.
/// [`crate::app::prefs::redaction`] carries the argument in full.
///
/// # What this does not change
///
/// It does not change what is **reported**. Every value runs the same search
/// and names everything it finds; they differ only in what the search is
/// permitted to edit. A match the narrowest value declines to act on is still
/// counted and still disclosed off-canvas, which is the only reason offering
/// the narrowest value is safe at all.
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
