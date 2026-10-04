//! # `panels::properties::installedletter` — a refused letter typed in a face
//! from the font folders
//!
//! Contract: [`offer`] draws one button under a refused-letter block when the
//! letter was refused at the keystroke, the draft is still open on that run,
//! and there are font folders to pick from. The press commits the draft with
//! the letter at the caret, and the engine's replacement-face ladder picks the
//! face; nothing is drawn otherwise.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/installedletter.md`.

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::workaround as t;

/// The button, published on the frames it draws.
pub const REGION: &str = "properties.refusedchar.installed"; // ui-text-exempt: trace region name, never displayed

/// Draw the offer for `character` refused on `page`'s `run`.
pub(super) fn offer(
    ui: &mut egui::Ui,
    doc: &OpenDoc,
    page: usize,
    run: usize,
    character: char,
    actions: &mut Vec<Action>,
) {
    let ctx = ui.ctx().clone();
    let Some(commit) =
        crate::canvas::textedit::installed::letter_commit(&ctx, doc, page, run, character)
    else {
        return;
    };
    ui.label(egui::RichText::new(t::installed_letter_note()).small());
    let button = ui
        .button(t::installed_letter_button(character))
        .on_hover_text(t::installed_letter_hover());
    crate::diag::ui_rect(REGION, button.rect);
    crate::diag::trace(|| {
        format!("installed-letter-offer page={page} run={run} character={character:?}") // ui-text-exempt: diagnostic trace
    });
    if button.clicked() {
        actions.push(commit);
        crate::canvas::textedit::abandon(&ctx);
    }
}
