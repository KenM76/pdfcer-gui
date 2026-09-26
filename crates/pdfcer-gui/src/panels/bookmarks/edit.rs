//! # `panels::bookmarks::edit` — renaming a bookmark, and removing one with
//! everything under it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/bookmarks/edit.md`.

use egui::Ui;
use pdfcer_core::outline::OutlineItem;

use crate::app::actions::Action;
use crate::app::actions::bookmarks::BookmarkAction;
use crate::text::panels as t;

use super::BookmarksUi;
use super::tree;

/// The region the rename field publishes.
pub const REGION_RENAME: &str = "bookmarks.rename"; // ui-text-exempt: trace region name, never displayed
/// The region the Remove button publishes.
pub const REGION_DELETE: &str = "bookmarks.delete"; // ui-text-exempt: trace region name, never displayed

/// Draw the rename-and-remove block for the selected bookmark.
pub fn show(
    ui: &mut Ui,
    selected: &OutlineItem,
    ui_state: &mut BookmarksUi,
    actions: &mut Vec<Action>,
) {
    ui.separator();
    ui.label(t::bookmark_edit_heading());
    ui.weak(t::bookmark_edit_selected(&tree::display_title(
        &selected.title,
    )));

    rename_row(ui, selected, ui_state, actions);
    delete_row(ui, selected, ui_state, actions);
}

/// The name field and its Rename button.
fn rename_row(
    ui: &mut Ui,
    selected: &OutlineItem,
    ui_state: &mut BookmarksUi,
    actions: &mut Vec<Action>,
) {
    let mut typed = ui_state.rename_draft_for(selected);
    let mut commit = false;
    ui.horizontal_wrapped(|ui| {
        ui.label(t::bookmark_rename_label());
        // escape-disposition: keeps-draft — written back below whatever
        // happened, so the key leaves the box and leaves the words in it.
        let response = ui.add(egui::TextEdit::singleline(&mut typed).desired_width(160.0));
        crate::diag::ui_rect(REGION_RENAME, response.rect);

        let trimmed = typed.trim();
        if trimmed.is_empty() {
            // Said on the field, not on an absent button. A name typed down to
            // nothing is the one state where "no button" could read as "this is
            // broken" rather than as "there is nothing to do".
            let _ = response.on_hover_text(t::bookmark_rename_needs_a_title());
        } else if trimmed != selected.title {
            commit = ui.button(t::bookmark_rename_button()).clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
        }
    });
    // Written back whatever happened, so the next frame redraws what the
    // operator sees rather than what they last committed.
    ui_state.set_rename_draft(selected.id, typed.clone());
    if commit {
        let title = typed.trim().to_owned();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed. The LENGTH,
            // not the text — a bookmark's name is the operator's own words
            // about their drawing, and the trace is a file a harness keeps.
            format!(
                "bookmark-rename id={} chars={}",
                selected.id.num,
                title.chars().count()
            )
        });
        actions.push(Action::Bookmark(BookmarkAction::Rename {
            item: selected.id,
            title,
        }));
        // Cleared so the next frame re-seeds from the document. Without this
        // the field would hold the typed name against a bookmark that now has
        // it, and the Rename button would correctly vanish — which is the right
        // end state reached by luck rather than by design.
        ui_state.clear_rename_draft();
    }
}

/// The Remove button, and the blast radius stated before it is pressed.
fn delete_row(
    ui: &mut Ui,
    selected: &OutlineItem,
    ui_state: &mut BookmarksUi,
    actions: &mut Vec<Action>,
) {
    let descendants = tree::descendants(selected);
    if descendants > 0 {
        ui.weak(t::bookmark_delete_takes_subtree(descendants));
    }
    ui.weak(t::bookmark_delete_keeps_pages());

    let response = ui.button(t::bookmark_delete_button());
    crate::diag::ui_rect(REGION_DELETE, response.rect);
    if response.clicked() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed. The id and the
            // shell's own subtree count, so a driven check can compare them
            // against the engine's returned count in the disclosure.
            format!(
                "bookmark-delete id={} descendants={descendants}",
                selected.id.num
            )
        });
        actions.push(Action::Bookmark(BookmarkAction::Delete {
            item: selected.id,
        }));
        // The selection is dropped HERE rather than being left to next
        // frame's fallback, which would also work and would work one frame
        // late. For that frame this block would draw against a bookmark the
        // document no longer has — a name, a subtree count and a live Remove
        // button belonging to something already gone. `super::add`'s
        // resolve-or-clear stays as the guard it is for every path that is not
        // this one, notably undo.
        ui_state.clear_selection();
    }
}
