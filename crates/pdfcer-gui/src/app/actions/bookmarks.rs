//! # `app::actions::bookmarks` — the verbs whose subject is one entry in the
//! document's outline
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/bookmarks.md`.

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;

pub use pdfcer_gui_base::subactions::BookmarkAction;

/// Apply one bookmark verb.
pub(super) fn apply(doc: &mut OpenDoc, action: BookmarkAction) {
    match action {
        // One bookmark, one undo entry, and NO count reported.
        //
        // See the variant: `/Count` is two quantities and its sign is the
        // open/closed flag, so a bookmark added under a collapsed ancestor
        // leaves the document's total unchanged. A disclosure built by diffing
        // it would say "0" for a correct save.
        //
        // The destination is an explicit page at `Fit`, which is the only form
        // `add_outline_item` authors without refusing — named and remote
        // destinations are refused by name, and `DestView::Unknown` is refused
        // because the reader keeps an extension's fit NAME and discards its
        // parameters, so re-emitting it would write a view that is not the one
        // the source had.
        BookmarkAction::Add {
            parent,
            title,
            page,
        } => {
            super::apply::vector_edit(doc, "add-bookmark", page, 1, |session| {
                session
                    .add_outline_item(
                        parent,
                        &title,
                        Some(pdfcer_core::outline::Destination::Page {
                            page_index: page,
                            view: pdfcer_core::outline::DestView::Fit,
                        }),
                    )
                    .map(|_| Vec::new())
            });
        }
        BookmarkAction::Rename { item, title } => rename(doc, item, &title),
        BookmarkAction::Delete { item } => delete(doc, item),
        BookmarkAction::Move { item, to } => move_to(doc, item, to),
        BookmarkAction::Paste { clip, to } => paste(doc, &clip, to),
        BookmarkAction::SetOpen { item, open } => set_open(doc, item, open),
    }
}

/// **Put a copied bookmark subtree into the outline**, as one undoable command.
fn paste(
    doc: &mut OpenDoc,
    clip: &pdfcer_core::outline::OutlineClip,
    to: pdfcer_core::edit::OutlinePlacement,
) {
    // Page 0: an outline is a document-level structure reached from the
    // catalogue's `/Outlines` and never from a page, so there is no page this
    // edit is "on". `vector_edit` wants one for its trace and its invalidation;
    // zero is the honest answer and is what `super::bookmarks`' other arms pass.
    super::apply::vector_edit(doc, "paste-bookmark", 0, clip.len(), |session| {
        session.paste_outline_item(clip, to).map(|outcome| {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "bookmark-paste-applied items={} dropped={}",
                    outcome.items_pasted, outcome.destinations_dropped
                )
            });
            if outcome.destinations_dropped == 0 {
                Vec::new()
            } else {
                vec![crate::text::panels::bookmark_paste_dropped(
                    outcome.destinations_dropped,
                )]
            }
        })
    });
}

/// **Rename one bookmark**, as one undoable command, disclosing nothing.
fn rename(doc: &mut OpenDoc, item: ObjId, title: &str) {
    super::apply::vector_edit(doc, "rename-bookmark", 0, 1, |session| {
        session.set_outline_title(item, title).map(|()| Vec::new())
    });
}

/// **Delete one bookmark and its whole subtree**, as one undoable command,
/// disclosing how many items went.
fn delete(doc: &mut OpenDoc, item: ObjId) {
    super::apply::vector_edit(doc, "delete-bookmark", 0, 1, |session| {
        session
            .delete_outline_item(item)
            .map(|removed| vec![crate::text::panels::bookmark_deleted(removed)])
    });
}

/// **Move one bookmark and its whole subtree**, as one undoable command,
/// disclosing what the operator could not watch.
fn move_to(doc: &mut OpenDoc, item: ObjId, to: pdfcer_core::edit::OutlinePlacement) {
    super::apply::vector_edit(doc, "move-bookmark", 0, 1, |session| {
        // Read BEFORE the move: after it, the item is somewhere else and its
        // own `/Count` sign has been carried along with it, which is fine — but
        // the tree walk that finds it would be walking the arrangement the
        // operator did not press the button on.
        let before = pdfcer_core::outline::read_outline(&session.view());
        // `None` unless the bookmark was **closed and not empty**, which is the
        // only case where the engine's count and the branch size differ. A leaf
        // and an open branch both need no second sentence: for the leaf there
        // is nothing hidden, and for the open one `visible_items` already
        // counted it.
        let hidden = crate::panels::bookmarks::tree::find(&before.items, item)
            .filter(|found| !found.open)
            .map(crate::panels::bookmarks::tree::descendants)
            .filter(|count| *count > 0);

        let report = match session.move_outline_item(item, to) {
            Ok(report) => report,
            Err(error) => {
                // Which sentence, decided here and nowhere else. The panel
                // raises a drop it has already forecast as impossible —
                // deliberately, see `panels::bookmarks::reorder::settle` — so
                // this arm is the one place that tells the operator's own
                // mistake apart from the document's refusal.
                crate::app::status::decline::record_bookmark_move_refused(matches!(
                    error,
                    pdfcer_core::edit::EditError::OutlineMoveIntoOwnSubtree { .. }
                ));
                return Err(error);
            }
        };
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            //
            // Every field of the engine's report, plus the shell's own branch
            // size. A line saying only "a move applied" would be identical for
            // a build that reordered where it should have re-parented, or that
            // reported the branch size where it should have reported what moved
            // on screen — and those are the two things a wrong build gets wrong
            // here, because they are two numbers that agree on every ordinary
            // document and part company on exactly the collapsed one.
            format!(
                "bookmark-move-report item={} moved={} reparented={} visible={} hidden={}",
                item.num,
                u8::from(report.moved),
                u8::from(report.reparented),
                report.visible_items,
                hidden.unwrap_or(0),
            )
        });
        if !report.moved {
            // The engine wrote nothing and created no undo entry. The panel
            // dims its caret over a landing it can see is a no-op, so reaching
            // this means the shell's forecast and the engine's answer
            // disagreed — which is worth one line rather than a shrug.
            return Ok(vec![
                crate::text::panels::bookmarks::bookmark_move_no_change().to_owned(),
            ]);
        }
        let mut notes = vec![crate::text::panels::bookmarks::bookmark_moved(
            report.visible_items,
            report.reparented,
        )];
        if let Some(count) = hidden {
            notes.push(crate::text::panels::bookmarks::bookmark_move_took_hidden(
                count,
            ));
        }
        let after = pdfcer_core::outline::read_outline(&session.view());
        if crate::panels::bookmarks::tree::find(&after.items, report.to_parent)
            .is_some_and(|parent| !parent.open)
        {
            notes.push(crate::text::panels::bookmarks::bookmark_move_into_collapsed().to_owned());
        }
        Ok(notes)
    });
}

/// **Expand or collapse one bookmark**, as one undoable command, disclosing
/// nothing.
fn set_open(doc: &mut OpenDoc, item: ObjId, open: bool) {
    super::apply::vector_edit(doc, "set-bookmark-open", 0, 1, |session| {
        session.set_outline_open(item, open).map(|_| Vec::new())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The three verbs are three distinct values**, so a match on them
    /// cannot silently collapse.
    #[test]
    fn the_three_verbs_are_distinguishable() {
        let id = ObjId::new(7, 0);
        let add = BookmarkAction::Add {
            parent: Some(id),
            title: "Chapter 3".to_owned(),
            page: 4,
        };
        let rename = BookmarkAction::Rename {
            item: id,
            title: "Chapter 3".to_owned(),
        };
        let delete = BookmarkAction::Delete { item: id };
        assert_ne!(add, rename);
        assert_ne!(rename, delete);
        assert_ne!(add, delete);
    }

    /// **A rename of the same item to two different titles is two different
    /// actions**, and a rename of two different items to the same title is
    /// too.
    #[test]
    fn a_rename_is_identified_by_both_its_item_and_its_title() {
        let a = ObjId::new(7, 0);
        let b = ObjId::new(8, 0);
        let same_item_new_title = (
            BookmarkAction::Rename {
                item: a,
                title: "one".to_owned(),
            },
            BookmarkAction::Rename {
                item: a,
                title: "two".to_owned(),
            },
        );
        assert_ne!(same_item_new_title.0, same_item_new_title.1);

        let same_title_new_item = (
            BookmarkAction::Rename {
                item: a,
                title: "one".to_owned(),
            },
            BookmarkAction::Rename {
                item: b,
                title: "one".to_owned(),
            },
        );
        assert_ne!(same_title_new_item.0, same_title_new_item.1);
    }

    /// **The generation number is part of the identity.**
    #[test]
    fn an_objid_generation_distinguishes_two_deletes() {
        assert_ne!(
            BookmarkAction::Delete {
                item: ObjId::new(7, 0)
            },
            BookmarkAction::Delete {
                item: ObjId::new(7, 1)
            },
        );
    }
}
