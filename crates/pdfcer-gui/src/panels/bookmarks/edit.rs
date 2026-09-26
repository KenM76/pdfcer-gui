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
///
/// `selected` is the item the operator last clicked, already resolved against
/// the outline **as it stands this frame** by the caller. Resolving it there
/// rather than here is what lets the whole block be skipped when nothing is
/// selected — R9, one call site up — and it means this function never has to
/// consider an id that no longer names anything, which is the ordinary state
/// one frame after an undo.
///
/// Nothing is mutated except `ui_state`'s own draft. Both verbs leave through
/// `actions`: no code path runs from a widget to a document, which is what
/// keeps one gesture equal to one undo entry. See `app::actions`' `OVERVIEW.md`.
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
///
/// # The draft carries its own `ObjId`, and that is not tidiness
///
/// A half-typed name must not follow the operator to a different bookmark.
/// Holding the id **with** the text makes a stale pair detectable, so clicking
/// another row re-seeds the field from the bookmark actually on screen rather
/// than offering to rename *it* to a name meant for the last one.
///
/// It is the same hazard `dialogs::scale` names for its captured group — a
/// picker that moves underneath an open dialog lets the operator type a number
/// for one group and commit it to another — one control smaller, and worse here
/// than there because the row list is one click away rather than behind a
/// window.
///
/// # Why the button is ABSENT rather than greyed when there is nothing to do
///
/// A Rename button beside a field holding the bookmark's current name is a
/// control whose only possible effect is an undo entry the operator did not
/// earn. The field alone reads as *"this is what it is called"*, which is true.
/// That is the same call `panels::dimension_groups::identity` makes for the
/// identical control, and it is deliberately the **opposite** of the Add
/// button's one block up: that one is the whole of its feature, so it stays and
/// explains itself.
///
/// The blank-name case is the one worth a sentence rather than a silence, so it
/// is on the field's hover text.
///
/// # Enter commits
///
/// Because a name is a thing people type and then press Enter on. Checking only
/// the button would make that keystroke do nothing and send the operator back
/// to a mouse they had just put down.
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
///
/// # The subtree count is the disclosure, and it is said twice on purpose
///
/// **Before the press**, from the tree this panel already drew: *"Removing this
/// also removes the 11 bookmarks filed under it."* The operator cannot get that
/// number any other way — a collapsed heading shows nothing beneath it, and
/// §12.3.3 gives a closed item's ancestors a `/Count` contribution of exactly
/// one however large its subtree is.
///
/// **After the press**, from the count `EditSession::delete_outline_item`
/// returns, in the status line where every other verb's disclosure goes. See
/// `crate::app::actions::bookmarks::delete`.
///
/// **The two numbers are allowed to differ, and that is why both are said.**
/// `read_outline` gives up part-way on a cycle, on excessive depth, or on
/// exhausting its item budget — this panel draws a truncation notice above the
/// list when it does — so the number here counts *what pdfcer could read* and
/// the number afterwards counts *what the engine removed*. On any ordinary
/// document they agree. On a damaged one, an operator who was promised 3 and
/// told 47 has learned something real about their file, which is strictly
/// better than being shown one number and trusting it.
///
/// # Why the leaf case says nothing extra
///
/// A bookmark with no children removes exactly itself, and there is no hidden
/// consequence to disclose. *"Removing this also removes the 0 bookmarks filed
/// under it"* is the shape of sentence that makes a program look like it is
/// filling in a template, so the sentence is simply not drawn.
///
/// # The pages line is a fact, not reassurance
///
/// An outline is a document-level structure reached from the catalogue's
/// `/Outlines`, never from a page. Removing a bookmark removes a way of
/// *reaching* a page. It is worth saying beside a control that takes several
/// things at once, because the operator's reasonable fear at that moment is
/// that the pages are what is going.
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
