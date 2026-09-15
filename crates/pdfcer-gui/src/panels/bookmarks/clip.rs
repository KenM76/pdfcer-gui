//! # `panels::bookmarks::clip` — **cut, copy and paste a bookmark and everything under it**
//!
//! `OPERATOR_REQUESTS.md` **O59**, item 3, and the last of the three.
//!
//! ## ★★★ This is the one Acrobat cannot do
//!
//! `pdfcer-core`, 2026-08-29: *"Acrobat cannot do this between two files at all;
//! Adobe's own documentation says so by name."* Copying a chapter's bookmark
//! subtree out of one drawing set and into another is a thing an operator with
//! a template has always had to do by hand, one bookmark at a time.
//!
//! Worth stating plainly because it changes how the controls should read: this
//! is not a parity feature catching up with a reference implementation, so
//! there is no established wording to borrow and the sentences below are
//! written from what the operation actually does.
//!
//! ## Why the controls are in this panel and not on the ribbon
//!
//! Every other bookmark verb is here — Add, Rename, Remove, the reorder arrows
//! and the disclosure triangles. A bookmark is edited where it is *seen*,
//! because the tree is the only thing that says which one is selected and what
//! is filed under it.
//!
//! ⇒ So Copy and Paste join them, and there is no ribbon entry and no chord.
//! `app::dispatch::pageclip`'s header carries the general form of that argument
//! for pages; here it is simpler, because there was never a competing claimant:
//! `Ctrl+C` belongs to the canvas and no bookmark has ever wanted it.
//!
//! ## ★★ The one question that must be asked BEFORE the press
//!
//! **Does the destination document have the pages these bookmarks point at?**
//!
//! The engine flagged this itself and it is the third of its three
//! *"produces a document that looks right and is not"* cases:
//!
//! > A destination naming a page this document does not have is **DROPPED, not
//! > clamped**. A dropped-destination bookmark still shows, still has its
//! > title, and does nothing when clicked, with nothing on screen to
//! > distinguish it.
//!
//! `OutlineClip::deepest_page()` against this document's page count answers it,
//! and it is asked **while the operator can still choose** — before the paste,
//! beside the button, rather than as a report afterwards.
//!
//! ★ Dropped rather than clamped is the right engine behaviour and worth
//! understanding before writing the sentence: clamping would send the operator
//! to *some* page, confidently and wrongly, which is worse than a bookmark that
//! plainly does nothing. §12.3.3 permits an item with no destination — a pure
//! grouping entry — so a destination-less bookmark is a legal, honest shape.
//!
//! ## Where a paste lands
//!
//! **As the last child of the selected bookmark**, or at the top level when
//! nothing is selected. That is `add`'s rule exactly, and reusing it is the
//! point: an operator who has learned where a new bookmark appears already
//! knows where a pasted one will.

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
///
/// # ★ Why cut is copy-then-`Delete` and not `cut_outline_item`
///
/// `app::dispatch::pageclip`'s reason, unchanged: the clipboard lives in
/// `egui::Memory` and the action applier has no `egui::Context`, so a
/// single-call cut could not put its own clip anywhere. The engine's own
/// `cut_outline_item` is literally `copy_outline_item` followed by
/// `delete_outline_item`, so this is the same two steps in the same order —
/// and `BookmarkAction::Delete` already drops the selection, warns about the
/// subtree and is one undo entry.
pub fn copy_row(ui: &mut Ui, doc: &OpenDoc, selected: &OutlineItem, actions: &mut Vec<Action>) {
    let descendants = super::tree::descendants(selected);
    if descendants > 0 {
        ui.weak(t::bookmark_copy_takes_subtree(descendants));
    }

    ui.horizontal(|ui| {
        let copy = ui.button(t::bookmark_copy_button());
        crate::diag::ui_rect(REGION_COPY, copy.rect);
        let cut = ui.button(t::bookmark_cut_button());
        // ★ ONE take for both buttons, and the cut's delete is conditional on
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
///
/// Returns whether it worked, so a cut can call off its own delete half — the
/// ordering rule `canvas::clipboard::cut` established: *a cut that silently
/// becomes a delete is a different verb wearing the operator's control, and
/// they would find out by pasting.*
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
///
/// ★ Drawn only then — R9: an unavailable capability renders **nothing**, and a
/// Paste button on a program that has never had a bookmark copied is a control
/// whose only possible outcome is a refusal.
///
/// # ★★★ The warning is beside the button, not after the press
///
/// See the module header. A pasted bookmark whose page does not exist here is
/// **silently dead** — it shows, it has its title, and clicking does nothing —
/// so the count of how many would land that way is computed from the clip and
/// the page count and drawn where the operator is already looking.
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

    // ★★ The pre-press disclosure. `deepest_page` is 0-based, so a clip whose
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
        // ★ `LastChild` of the selection, or of the root when nothing is
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
