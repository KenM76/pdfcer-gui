//! # `app::dispatch::replacetext` — `markup.replace_text`: strike the selected
//! text and ask for the words that replace it
//!
//! The selection is read now, through the same eligibility rule the
//! strike-out command uses (`canvas::markup::text::mark`), and its boxes
//! travel with the window; nothing is authored until Add.

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::state::Status;
use crate::canvas::markup::text::{Refusal, TextMarkKind, mark};

/// The one id this module owns.
pub(crate) const ID: &str = "markup.replace_text"; // ui-text-exempt: a command id, never displayed

/// Open the replace-text window over the selection, or trace why not.
pub(super) fn dispatch(app: &mut PdfcerApp) {
    if !app.capabilities().author_markup {
        decline("mode-cannot-author-markup");
        return;
    }
    let selected = if let Status::Open(doc) = &app.status {
        mark(
            TextMarkKind::StrikeOut,
            doc.text_selection.as_ref(),
            doc.edit_epoch,
            app.pen,
        )
    } else {
        Err(Refusal::NoSelection)
    };
    match selected {
        Ok(Action::CommitTextMarkup { page, quads, .. }) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("replace-text-open page={page} quads={}", quads.len())
            });
            app.dialogs.open_replace_text(&app.status, page, quads);
        }
        Ok(_) => decline("not-a-text-mark"),
        Err(reason) => decline(&format!("{reason:?}")),
    }
}

fn decline(reason: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("command-declined id={ID} reason={reason}")
    });
}
