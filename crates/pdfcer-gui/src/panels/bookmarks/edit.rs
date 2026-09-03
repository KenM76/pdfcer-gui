//! # `panels::bookmarks::edit` — renaming a bookmark, and removing one with
//! everything under it
//!
//! ## What this closes
//!
//! The two verbs this panel shipped **without**. Until `pdfcer-core`
//! `Pass 156.0` a bookmark could be **created and never changed** — the
//! engine's own words — and the covering note is blunt about which half of that
//! hurt:
//!
//! > *"Renaming is the commonest bookmark edit there is."*
//!
//! `EditSession::set_outline_title` and `EditSession::delete_outline_item`
//! shipped together on 2026-08-28 and these are the controls that answer them.
//!
//! ## ★ The gesture is already there: the selected row
//!
//! No new selection mechanic is invented. Clicking a bookmark row **already**
//! meant two things — *"take me there"* and *"this is the parent for the next
//! add"* — and this makes it mean a third: *"this is the one I am editing."*
//! All three are true of the row the operator pointed at, which is what makes
//! the overload honest rather than a shortcut, and the alternative is a second
//! selection gesture (a checkbox column, a right-click menu) that would have to
//! be taught for two verbs.
//!
//! The consequence is stated rather than left to be discovered: pressing **Move
//! to top level** in the add row clears the selection, so these controls
//! disappear with it. That is correct — nothing is selected — and it is why
//! this block names the bookmark it acts on in words
//! (`bookmark_edit_selected`) rather than relying on a highlight in a list that
//! may be scrolled out of view.
//!
//! ## ★★ Delete is UNDOABLE, not confirmed — and that is the choice made here
//!
//! `HANDOFF.md`'s rule is *"confirmed or clearly undoable"*. This surface takes
//! the second, for three reasons, in order of weight:
//!
//! 1. **One press is one `EditSession` command.** The engine plans every relink
//!    — the previous sibling's `/Next`, the next one's `/Prev`, the parent's
//!    `/First`/`/Last`, every open ancestor's `/Count` — *inside* that single
//!    command. So `Ctrl+Z` restores the entire subtree and there is no
//!    half-undone state to land in. An undo that is total is a better safety
//!    net than a question, because it also covers the operator who *meant* to
//!    press it and changed their mind afterwards, which a confirmation does
//!    not.
//! 2. **The question a confirmation would ask is the wrong question.** *"Are
//!    you sure?"* carries no information. What the operator actually needs is
//!    *"this takes the eleven bookmarks filed underneath as well"* — and a
//!    modal is a bad place to put that, because it arrives after the decision
//!    has been made and is read by nobody by the fourth time. So it is on the
//!    panel, beside the button, **before** the press.
//! 3. **Modals train themselves away.** An operator who has dismissed the same
//!    dialog four times dismisses the fifth without reading it, and the fifth
//!    is the one that mattered. This is the same reasoning the ce-dimension
//!    group window applies to its own Delete, which asks a *substantive*
//!    question (where do the members go?) rather than a ceremonial one.
//!
//! ⇒ The obligation that replaces the modal: **the blast radius must be
//! visible before the press and reported after it.** Both are done, and the
//! two numbers are allowed to differ — see [`delete_row`].
//!
//! ## Why the encryption and certification refusals are not pre-empted here
//!
//! Both verbs refuse an encrypted document and refuse one whose certification
//! signature forbids the change. Neither is checked before drawing the buttons,
//! deliberately and consistently with every other authoring surface in this
//! shell — including [`super::add`], which has the same two refusals and the
//! same absence of a guard.
//!
//! R9 says an *unavailable capability* renders nothing, and the distinction it
//! turns on is whether the shell can know. Here it cannot know honestly: the
//! certification gate is a property of a signature the panel does not read, and
//! duplicating the engine's test at the widget would produce a second
//! implementation of a permission rule that can drift from the first. A refusal
//! is traced and the document is left alone, which is this shell's whole
//! response to any engine refusal today; wording declines is `FEATURES.md`'s
//! "Worded decline" row and belongs to `super::super::super::app::actions::apply`'s
//! `Err` arm, not to this file.
//!
//! ## ★ What was deliberately absent, and where it went
//!
//! This section read: *"**Reorder and re-parent.** The engine's note lists them
//! as not shipped … so there is no drag handle, no Move up, no Promote. R9: a
//! capability that does not exist renders **nothing**."* It was correct for one
//! day. `pdfcer-core` `Pass 161.0` shipped `move_outline_item` and
//! `set_outline_open`, and [`super::reorder`] is the surface for both.
//!
//! It is recorded rather than deleted because it is the rule working: nothing
//! was greyed, nothing was drawn as a promise, and the day the engine could
//! honour the gesture the panel grew it.
//!
//! ⇒ **And it did not grow it here.** The move is a *drag on the row*, not a
//! button in this block, which is the conventional gesture every outline panel
//! uses and the operator's standing tie-breaker — *"make it work the way other
//! programs do."* A pair of *Move up* / *Promote* buttons would have been the
//! easy thing to add to this block and would have been a second, worse idiom
//! beside the one [`crate::panels::pages`] already established for reordering.
//!
//! **Still absent:** a verb that deletes the whole outline.
//! `EditError::OutlineRootIsNotAnItem` refuses the root by name, because that
//! is *"a different act that gets its own verb when it is wanted"*.

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
/// `actions`, which is the invariant `app::actions`' `OVERVIEW.md` calls *"the
/// single best structural decision in the old GUI"*.
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
/// # ★ The draft carries its own `ObjId`, and that is not tidiness
///
/// A half-typed name must not follow the operator to a different bookmark.
/// Holding the id **with** the text makes a stale pair detectable, so clicking
/// another row re-seeds the field from the bookmark actually on screen rather
/// than offering to rename *it* to a name meant for the last one.
///
/// The same hazard `dialogs::scale` names for its captured group — *"a group
/// picker that moved underneath an open dialog would let them type a number for
/// one group and commit it to another"* — one control smaller, and here it is
/// worse than there because the row list is one click away rather than behind a
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
/// # ★★ The subtree count is the disclosure, and it is said twice on purpose
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
/// ★ **The two numbers are allowed to differ, and that is why both are said.**
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
        // ★ The selection is dropped HERE rather than being left to next
        // frame's fallback, which would also work and would work one frame
        // late. For that frame this block would draw against a bookmark the
        // document no longer has — a name, a subtree count and a live Remove
        // button belonging to something already gone. `super::add`'s
        // resolve-or-clear stays as the guard it is for every path that is not
        // this one, notably undo.
        ui_state.clear_selection();
    }
}
