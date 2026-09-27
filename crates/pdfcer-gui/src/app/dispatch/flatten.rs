//! # `app::dispatch::flatten` — `markup.flatten`: make the selected markup part
//! of the page

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::state::Status;
use crate::canvas::selection::AnnotKind;

/// The one id this module owns.
pub(crate) const ID: &str = "markup.flatten"; // ui-text-exempt: a command id, never displayed

/// The page-wide command this module also owns.
pub(crate) const PAGE_ID: &str = "markup.flatten_page"; // ui-text-exempt: a command id, never displayed

/// Raise the burn of every markup on the page in view.
pub(super) fn dispatch_page(app: &mut PdfcerApp, actions: &mut Vec<Action>) {
    if !app.capabilities().author_markup {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={PAGE_ID} reason=mode-cannot-author-markup")
        });
        return;
    }
    if let Status::Open(doc) = &app.status
        && doc.view.page_index < doc.pages.len()
    {
        actions.push(Action::Annot(AnnotAction::FlattenPage {
            page: doc.view.page_index,
        }));
    }
}

/// Raise the flatten of the selected markup. A chord or a stale menu reaches
/// the verb too, and the verb's own refusal is worded on the status bar.
pub(super) fn dispatch(app: &mut PdfcerApp, actions: &mut Vec<Action>) {
    if !app.capabilities().author_markup {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={ID} reason=mode-cannot-author-markup")
        });
        return;
    }
    let Status::Open(doc) = &app.status else {
        return;
    };
    match doc.selection.annot() {
        Some(a) if a.target.kind == AnnotKind::Markup => {
            actions.push(Action::Annot(AnnotAction::Flatten {
                page: a.target.page,
                id: a.target.id,
            }))
        }
        _ => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={ID} reason=no-markup-selected")
        }),
    }
}
