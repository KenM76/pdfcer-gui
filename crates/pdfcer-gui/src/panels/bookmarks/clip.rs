//! # `panels::bookmarks::clip` — **cut, copy and paste a bookmark and everything under it**
//!
//! `OPERATOR_REQUESTS.md` **O59**, item 3, and the last of the three.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/bookmarks/clip.md`.

use egui::Ui;

use crate::app::actions::Action;
use crate::app::actions::bookmarks::BookmarkAction;
use crate::app::state::OpenDoc;
use crate::canvas::clipboard::{Clipped, read, store};
use crate::text::panels as t;
use pdfcer_core::outline::OutlineItem;

/// The region a driven check aims at for Copy.
pub const REGION_COPY: &str = "bookmark-copy";

/// The region a driven check aims at for Paste.
pub const REGION_PASTE: &str = "bookmark-paste";

/// **Copy and Cut**, drawn only when a bookmark is selected.
pub fn copy_row(ui: &mut Ui, doc: &OpenDoc, selected: &OutlineItem, actions: &mut Vec<Action>) {
    let descendants = super::tree::descendants(selected);
    if descendants > 0 {
        ui.weak(t::bookmark_copy_takes_subtree(descendants));
    }

    ui.horizontal(|ui| {
        let copy = ui.button(t::bookmark_copy_button());
        crate::diag::ui_rect(REGION_COPY, copy.rect);
        let cut = ui.button(t::bookmark_cut_button());
        // ONE take for both buttons, and the cut's delete is conditional on
        // it succeeding. That is `canvas::clipboard::cut`'s ordering rule: a
        // cut whose copy half failed must not go on to delete, because a cut
        // that silently became a delete is a different verb wearing the
        // operator's control and they would find out by pasting.
        if (copy.clicked() || cut.clicked()) && take(ui, doc, selected) && cut.clicked() {
            actions.push(Action::Bookmark(BookmarkAction::Delete {
                item: selected.id,
            }));
        }
    });
}

/// Put the selected bookmark and its subtree on the clipboard.
fn take(ui: &Ui, doc: &OpenDoc, selected: &OutlineItem) -> bool {
    match doc.session.copy_outline_item(selected.id) {
        Ok(clip) => {
            let deepest = clip.deepest_page();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "bookmark-copy id={} items={} deepest_page={deepest:?}",
                    selected.id.num,
                    clip.len()
                )
            });
            store(
                ui.ctx(),
                Clipped::Outline {
                    clip: Box::new(clip),
                    deepest_page: deepest,
                },
            );
            true
        }
        Err(error) => {
            let detail = error.to_string();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("bookmark-copy-refused id={} err={detail}", selected.id.num)
            });
            crate::app::actions::record_note(doc.edit_epoch, t::bookmark_copy_refused(&detail));
            false
        }
    }
}

/// **Paste**, drawn whenever the clipboard holds bookmarks.
pub fn paste_row(
    ui: &mut Ui,
    doc: &OpenDoc,
    selected: Option<&OutlineItem>,
    actions: &mut Vec<Action>,
) {
    let Some(Clipped::Outline { clip, deepest_page }) = read(ui.ctx()) else {
        return;
    };

    ui.separator();
    ui.label(t::bookmark_paste_heading(clip.len()));

    // The pre-press disclosure. `deepest_page` is 0-based, so a clip whose
    // deepest destination is page index 11 needs twelve pages here.
    let short = deepest_page.is_some_and(|deepest| deepest >= doc.pages.len());
    if short {
        ui.weak(t::bookmark_paste_destinations_dropped(
            deepest_page.unwrap_or(0).saturating_add(1),
            doc.pages.len(),
        ));
    }

    ui.weak(match selected {
        Some(item) => t::bookmark_paste_under(&super::tree::display_title(&item.title)),
        None => t::bookmark_paste_at_top_level().to_owned(),
    });

    let response = ui.button(t::bookmark_paste_button());
    crate::diag::ui_rect(REGION_PASTE, response.rect);
    if response.clicked() {
        // `LastChild` of the selection, or of the root when nothing is
        // selected — `add`'s placement rule verbatim, so an operator who knows
        // where a new bookmark appears knows where a pasted one will.
        let to = pdfcer_core::edit::OutlinePlacement::LastChild {
            parent: selected.map(|item| item.id),
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "bookmark-paste items={} under={:?} short={short}",
                clip.len(),
                selected.map(|i| i.id.num)
            )
        });
        actions.push(Action::Bookmark(BookmarkAction::Paste { clip, to }));
    }
}
