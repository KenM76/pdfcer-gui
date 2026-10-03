//! New PDF from Clipboard and Insert Pages from Clipboard: what another
//! program copied becomes a document of its own, or pages after the one on
//! screen.
//!
//! Contract: a picture becomes one page its natural size holding only the
//! picture; text is set as File ▸ Import text as pages sets it with its
//! controls untouched. A clipboard holding neither, or a picture that cannot
//! be read, is declined with the reason and nothing changes. An insert goes
//! through the same actions as Insert from file and Import text as pages, so
//! it is one undo step with their receipts.

use super::{decline, read};
use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::importtext;
use crate::app::actions::pages::PageAction;
use crate::app::state::Status;
use crate::dialogs::import_text::ImportTextDialog;
use crate::text::ospaste::OsPasteRefusal;
use pdfcer_core::pageops::InsertPosition;
use pdfcer_gui_base::blank;
use pdfcer_gui_base::clippaste::{self, Incoming, page};

/// Whether this module owns `id`.
#[must_use]
pub fn handles(id: &str) -> bool {
    matches!(
        id,
        "file.new_from_clipboard" | "pages.insert_from_clipboard"
    )
}

/// Route one of the two commands.
pub fn dispatch(app: &mut PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    match id {
        "file.new_from_clipboard" => new_document(app, read()),
        "pages.insert_from_clipboard" => insert(app, id, read(), actions),
        _ => {}
    }
}

/// The clipboard as a new document, in a tab of its own.
fn new_document(app: &mut PdfcerApp, incoming: Incoming) {
    let (kind, made, notes) = match incoming {
        Incoming::Image { image, .. } => (
            "image",
            blank::picture_page(&image).and_then(page::opened),
            Vec::new(),
        ),
        Incoming::Text(text) => {
            match blank::text_pages(&text, &ImportTextDialog::default_template()) {
                Ok((bytes, report)) => {
                    ("text", page::opened(bytes), importtext::judgements(&report))
                }
                Err(blank::TextPagesError::Refused(e)) => {
                    return decline(
                        "refused",
                        OsPasteRefusal::PagesRefused(importtext::refusal_for(&e)),
                    );
                }
                Err(blank::TextPagesError::Failed(why)) => ("text", Err(why), Vec::new()),
            }
        }
        other => return nothing(other),
    };
    app.adopt_created(made);
    if let Status::Open(doc) = &app.status {
        let first = doc.pages.first().map(|p| p.media_box);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "pages-from-clipboard to=new kind={kind} pages={} w={:.2} h={:.2}",
                doc.pages.len(),
                first.map_or(0.0, |m| m.width()),
                first.map_or(0.0, |m| m.height()),
            )
        });
        if !notes.is_empty() {
            crate::app::actions::record_notes(doc.edit_epoch, notes);
        }
    }
}

/// The clipboard as pages directly after the one on screen.
fn insert(app: &PdfcerApp, id: &str, incoming: Incoming, actions: &mut Vec<Action>) {
    if !app.capabilities().edit_content {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=mode-cannot-edit-content")
        });
        return;
    }
    let Status::Open(doc) = &app.status else {
        return;
    };
    let current = doc.view.page_index;
    let seq = clippaste::sequence();
    match incoming {
        Incoming::Image { image, .. } => {
            // ui-text-exempt: a file name, never displayed
            let name = format!("clipboard-page-{seq}.pdf");
            match blank::picture_page(&image).and_then(|bytes| page::scratch(&name, &bytes)) {
                Ok(path) => actions.push(Action::Page(PageAction::InsertPagesFromFile {
                    path,
                    pages: vec![0],
                    position: InsertPosition::After(current),
                })),
                Err(why) => decline("unplaceable", OsPasteRefusal::NotAPage(why)),
            }
        }
        Incoming::Text(text) => {
            // ui-text-exempt: a file name, never displayed
            let name = format!("clipboard-text-{seq}.txt");
            match page::scratch(&name, text.as_bytes()) {
                Ok(path) => actions.push(ImportTextDialog::dropped(path, current)),
                Err(why) => decline("unplaceable", OsPasteRefusal::NotAPage(why)),
            }
        }
        other => nothing(other),
    }
}

/// A clipboard with no picture or text to make pages from.
fn nothing(incoming: Incoming) {
    match incoming {
        Incoming::Unreadable(why) => decline("unreadable", OsPasteRefusal::Unreadable(why)),
        _ => decline("nothing", OsPasteRefusal::NothingForPages),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_owns_the_two_commands_and_no_paste() {
        assert!(handles("file.new_from_clipboard"));
        assert!(handles("pages.insert_from_clipboard"));
        assert!(!handles("edit.paste"));
    }
}
