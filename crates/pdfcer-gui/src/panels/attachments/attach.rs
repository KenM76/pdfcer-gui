//! # `panels::attachments::attach` — putting a file into the document
//!
//! ## The gap this closes
//!
//! `EditSession::attach_file` and `EditSession::detach_file` have existed in
//! `pdfcer-core` with **no GUI surface of any kind** — not a command, not a
//! panel, not a menu item. This row is the writing half of the surface that
//! closes it, and `super` is the reading half.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/attachments/attach.md`.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::attachments::AttachmentAction;
use crate::text::panels::attachments as t;

use super::AttachmentsUi;

/// The region the description field publishes.
pub const REGION_DESCRIPTION: &str = "attachments.description"; // ui-text-exempt: trace region name, never displayed
/// The region the Attach button publishes.
pub const REGION_ATTACH: &str = "attachments.attach"; // ui-text-exempt: trace region name, never displayed

/// Draw the attach-a-file row.
pub fn show(ui: &mut Ui, ui_state: &mut AttachmentsUi, actions: &mut Vec<Action>) {
    ui.label(t::attach_heading());

    let response = ui.add(
        // escape-disposition: keeps-draft — the description lives in panel
        // state and is committed by the Attach button. Nothing clears it.
        egui::TextEdit::singleline(&mut ui_state.description)
            .desired_width(f32::INFINITY)
            .hint_text(t::attach_description_hint()),
    );
    crate::diag::ui_rect(REGION_DESCRIPTION, response.rect);
    ui.label(
        egui::RichText::new(t::attach_description_note())
            .small()
            .weak(),
    );

    let button = ui
        .button(t::attach_button())
        .on_hover_text(t::attach_tooltip());
    crate::diag::ui_rect(REGION_ATTACH, button.rect);
    if button.clicked() {
        // Trimmed here rather than in the apply arm, because the trim is a
        // decision about what the operator *meant* and belongs where they are
        // looking. An all-whitespace description is no description: writing it
        // would put a `/Desc` key holding blanks into the file, which a later
        // reader has to interpret and which no operator intended.
        let description = ui_state.description.trim();
        let description = (!description.is_empty()).then(|| description.to_owned());
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed. The LENGTH,
            // not the text — a description is the operator's own words about
            // their own file, and this reaches a trace a harness keeps.
            format!(
                "attach-file-requested described={} chars={}",
                description.is_some(),
                description.as_ref().map_or(0, |d| d.chars().count())
            )
        });
        actions.push(Action::Attachment(AttachmentAction::Attach { description }));
        // Cleared on the press, for `panels::bookmarks::add`'s reason: the
        // queue drains after the frame, and a description left in the box would
        // silently be re-used by the next attach — which the engine would
        // accept, and which would describe one file with another's note.
        ui_state.description.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **An all-whitespace description is no description.**
    #[test]
    fn a_blank_description_becomes_no_description_at_all() {
        for blank in ["", " ", "\t", "\n  \t"] {
            let trimmed = blank.trim();
            assert!(
                !(!trimmed.is_empty()),
                "{blank:?} must not produce a /Desc key"
            );
        }
        let typed = "  the supplier's quote  ";
        let trimmed = typed.trim();
        assert_eq!(
            trimmed, "the supplier's quote",
            "surrounding space is not the operator's text"
        );
        assert!(!trimmed.is_empty());
    }

    /// **The two published regions are distinct names.**
    #[test]
    fn the_two_regions_are_named_apart() {
        assert_ne!(REGION_DESCRIPTION, REGION_ATTACH);
        assert!(REGION_DESCRIPTION.starts_with("attachments."));
        assert!(REGION_ATTACH.starts_with("attachments."));
    }
}
