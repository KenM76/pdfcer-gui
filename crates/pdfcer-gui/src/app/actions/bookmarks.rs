//! # `app::actions::bookmarks` — the verbs whose subject is one entry in the
//! document's outline
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/bookmarks.md`.

use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;

/// The verbs whose subject is one entry in the document's outline.
///
/// See the module header for what makes them a family: every one of them names
/// its operand by `ObjId`, because an outline is a tree that every edit to it
/// renumbers.
/// **`PartialEq` and not `Eq`**, and the bound cannot be restored.
/// `pdfcer_core::outline::OutlineClip`, which [`BookmarkAction::Paste`]
/// carries, is `PartialEq` only — a bookmark's colour is three `f64`s and
/// floats have no total equality — and an enum holding one cannot be `Eq`.
/// Nothing needs it: `Eq` over `PartialEq` buys a `HashMap` key, and no action
/// is ever one.
#[derive(Debug, Clone, PartialEq)]
pub enum BookmarkAction {
    /// **Add a bookmark to the document's outline.**
    ///
    /// Raised by `crate::panels::bookmarks::add` and by nothing else.
    ///
    /// # Why nothing here counts anything
    ///
    /// `EditSession::add_outline_item` maintains `/Count`, and `/Count` is two
    /// different quantities — see the module header's table. The consequence
    /// the engine flagged as *"the entire difficulty of the feature"*: **adding
    /// a bookmark under a collapsed ancestor does not change the document's
    /// total**, because the new item is not visible. A surface reporting
    /// *"added N"* by diffing the root count therefore reports **zero for a
    /// correct save**.
    ///
    /// So this variant carries one bookmark, the apply arm adds one bookmark,
    /// and the panel says one bookmark. There is no number to get wrong.
    ///
    /// # Why the parent is an `ObjId` and not a position
    ///
    /// Because a position is invalidated by the very edit this performs. The
    /// engine hit that in its own CLI — *"the indices shift after every add …
    /// I got this wrong myself while driving the command and nested something
    /// two levels deeper than intended, and the output looked entirely
    /// plausible."* `OutlineItem::id` exists for this.
    ///
    /// `None` is the top level, which is `add_outline_item`'s own spelling.
    Add {
        /// The item it goes under, or `None` for the top level.
        parent: Option<ObjId>,
        /// The title. Trimmed and non-empty by the time it gets here.
        title: String,
        /// The 0-based page it points at — the one the operator is looking at.
        page: usize,
    },
    /// **Rename a bookmark** — write a new `/Title` onto one outline item.
    ///
    /// Raised by `crate::panels::bookmarks::edit` and by nothing else. The
    /// commonest bookmark edit there is, which is why it is the verb the panel
    /// puts first once a row is selected.
    ///
    /// # The verb with no structural risk, and saying so is load-bearing
    ///
    /// `set_outline_title`'s own doc comment is unusually reassuring, and the
    /// reassurance is a fact a reader of *this* file needs:
    ///
    /// > *"a title is a text string (§7.9.2) on one dictionary, and nothing in
    /// > the `/First`/`/Last`/`/Next`/`/Prev`/`/Count` machinery depends on
    /// > it."*
    ///
    /// **A rename cannot move, orphan, hide or renumber anything.** That is
    /// why this arm reports no disclosure at all: there is no consequence the
    /// operator cannot see. The new title appears in the row they are looking
    /// at, on the next frame, and that is the whole of what happened. Every
    /// other verb in this enum owes a sentence to `app::status`; this one owes
    /// none, and inventing one — *"Bookmark renamed."* under a row that now
    /// visibly reads the new name — would be noise standing where a real
    /// disclosure belongs.
    ///
    /// # Why the title travels by value
    ///
    /// The panel holds a **draft** that the operator is still typing into, and
    /// the queue drains after the frame. Borrowing it would tie the action's
    /// lifetime to the panel state, which `PdfcerApp::apply` cannot reach — it
    /// has no `egui::Context` and deliberately does not — so the operand comes
    /// with it, which is what an action *is*: a complete statement of intent,
    /// resolvable after the frame that raised it.
    ///
    /// Encoding is the engine's problem and is documented as deliberately not
    /// ours: `set_outline_title` routes through *"the same `crate::textstring`
    /// path every other text string uses"*, one path so that two cannot
    /// disagree about PDFDocEncoding. So an em dash or an accented name in this
    /// `String` needs nothing from this crate.
    Rename {
        /// The outline item whose `/Title` is being replaced.
        item: ObjId,
        /// The new title. Trimmed and non-empty by the time it gets here — a
        /// bookmark with a blank title is legal and is an invisible row, which
        /// is the same defect as no row.
        title: String,
    },
    /// **Delete a bookmark AND everything under it.**
    ///
    /// Raised by `crate::panels::bookmarks::edit` and by nothing else.
    ///
    /// # The subtree goes too, and that is a decision with a reason
    ///
    /// The engine takes Acrobat's behaviour and states the alternative it
    /// rejected, which is the part worth carrying here because it is the part
    /// an operator would otherwise discover:
    ///
    /// > *"promoting orphaned children to the deleted item's parent silently
    /// > **reorganises** a document's navigation, and an operator who deleted
    /// > one chapter heading would find its ten sections spliced into the top
    /// > level. Deleting what was asked for is the predictable act."*
    ///
    /// This is therefore a verb whose blast radius is **larger than the thing
    /// the operator clicked**, and the whole of the UI obligation follows from
    /// that one sentence. It is stated before the press by
    /// `crate::panels::bookmarks::edit`, from the tree the panel already drew,
    /// and it is stated again after the press from the engine's own count. See
    /// [`delete`] for why the answer is given twice and why the two numbers are
    /// allowed to differ.
    ///
    /// # Why there is no confirmation dialog, and it IS a choice
    ///
    /// A destructive act must be **confirmed or clearly undoable**, and this
    /// is the second. One press produces **one** `EditSession` command, so one
    /// `Ctrl+Z` puts the entire subtree back — the engine plans every relink
    /// (`/Prev`, `/Next`, the parent's `/First`/`/Last`, every open ancestor's
    /// `/Count`) inside that one command, so there is no half-undone state to
    /// reach. A modal would buy nothing that the undo does not already buy, and
    /// it would cost the thing modals always cost: an operator who has answered
    /// *"are you sure?"* four times stops reading it, and the fifth one is the
    /// one that mattered.
    ///
    /// The consequence the operator actually needs is **not** *"are you
    /// sure?"* — it is *"this takes the eleven bookmarks underneath as well"*,
    /// which a confirmation dialog is a bad place to put because it arrives
    /// after the decision. It is on the panel, beside the button, before the
    /// press.
    ///
    /// # No page index
    ///
    /// An outline is a document-level structure (§12.3.3) reached from the
    /// catalogue's `/Outlines`, not from any page. The item's own destination
    /// may name a page, and it is irrelevant here: this deletes the bookmark,
    /// never the page it points at, and nothing on any page changes.
    Delete {
        /// The outline item to remove, together with its whole subtree.
        item: ObjId,
    },
    /// **Move a bookmark — reorder it among its siblings, or re-parent it
    /// under a different one — carrying its whole subtree.**
    ///
    /// Raised by `crate::panels::bookmarks::reorder` and by nothing else.
    ///
    /// Without it an outline in the wrong **order** could only be fixed by
    /// deleting a branch and re-authoring it, which loses every destination,
    /// colour and style on it — not an edit any operator would call a
    /// reorganisation.
    ///
    /// # The subtree travels, and the destination does not move
    ///
    /// `move_outline_item`'s own words: *"A chapter dragged under a different
    /// part takes its sections with it."* That matches
    /// [`Self::Delete`]'s subtree semantics and Acrobat's model — its
    /// `PDBookmark` unlink/add-child pair operates on the node, which owns its
    /// children wherever `/Parent` points, and there is no API path that leaves
    /// them behind.
    ///
    /// So this verb, like the delete, has a **blast radius larger than the
    /// row the operator clicked** — and unlike the delete, the size of it is
    /// reported by the engine rather than counted by the panel. See [`move_to`]
    /// for the two numbers and why both are needed.
    ///
    /// # Why the placement is an anchor and NEVER an index
    ///
    /// `OutlinePlacement`'s own doc comment states the rule and names the
    /// failure this shell would otherwise walk into:
    ///
    /// > *"An outline's siblings are a **doubly-linked list** (§12.3.3 Table
    /// > 153: `/Prev`, `/Next`), not an array — there is no stored index, so an
    /// > index parameter would have to be *counted* by walking the chain, and
    /// > every caller holding one would be holding a number that silently goes
    /// > stale the moment any sibling is added or removed. **A shell that reads
    /// > a panel, lets the operator drag a row, and then calls with the index
    /// > it read has a race with its own undo stack.**"*
    ///
    /// That is this panel, described from the other side of the API. It is the
    /// same rule the whole of this module is built on — every variant here
    /// addresses its operand by `ObjId` — applied to the *destination* as well
    /// as to the subject.
    ///
    /// # Why there is no separate promote or demote verb
    ///
    /// Because they are this variant with a different anchor, and the engine
    /// refuses to spell one operation twice: *"a second spelling of one
    /// operation is exactly how two implementations of one rule come to
    /// disagree (`R171`)."* Re-parenting to the top level is
    /// `FirstChild { parent: None }` or `After` a top-level sibling; nesting is
    /// `LastChild { parent: Some(..) }`. The panel's three drop bands produce
    /// all of them.
    ///
    /// # The expansion of the destination is NOT folded in here
    ///
    /// The engine shipped [`Self::SetOpen`] alongside this verb and said why in
    /// a sentence that binds this shell:
    ///
    /// > *"Expand/collapse ships alongside, as a separate verb, because whether
    /// > a move should reveal a collapsed destination has two defensible
    /// > answers and both now exist."*
    ///
    /// pdfcer takes *"leave it as the operator set it"*, which is
    /// `move_outline_item`'s own default — a destination parent that already
    /// has children keeps its `/Count` sign — and discloses the consequence
    /// instead. A `reveal: bool` on this variant would bury a second state
    /// change inside an unrelated command and would produce **one** undo entry
    /// **Put a copied bookmark subtree into this document's outline.**
    ///
    /// `OPERATOR_REQUESTS.md` **O59** item 3. Raised by
    /// `panels::bookmarks::clip::paste_row` and by nothing else.
    ///
    /// **Acrobat cannot do this between two files at all**, by Adobe's own
    /// documentation. There is therefore no established behaviour to match and
    /// no borrowed wording — which is why the disclosure below is written from
    /// what the operation does rather than from what a reference implementation
    /// says about it.
    ///
    /// # The disclosure this arm owes
    ///
    /// `OutlinePasteOutcome::destinations_dropped`. A destination naming a page
    /// this document does not have is **dropped, not clamped** — so the
    /// bookmark arrives, shows, keeps its title, and does nothing when clicked.
    /// Nothing on screen distinguishes it from one that works.
    ///
    /// The panel warns about this **before** the press as well, from
    /// `OutlineClip::deepest_page()` against the page count. The two are not
    /// duplicates: the panel's is a prediction the operator can act on, and
    /// this one is what actually happened. A prediction alone would be a guess
    /// nobody confirmed; a report alone would arrive too late to choose
    /// differently.
    Paste {
        /// The copied roots and their children.
        clip: Box<pdfcer_core::outline::OutlineClip>,
        /// Where they go, as an anchor. Never a position — `Move`'s rule, and
        /// its documentation carries why.
        to: pdfcer_core::edit::OutlinePlacement,
    },
    /// for two acts.
    Move {
        /// The bookmark being moved, together with everything filed under it.
        item: ObjId,
        /// Where it is going, as an anchor. Never a position.
        to: pdfcer_core::edit::OutlinePlacement,
    },
    /// **Expand or collapse a bookmark** — flip the sign on its `/Count`.
    ///
    /// Raised by `crate::panels::bookmarks::reorder`'s disclosure triangle and
    /// by nothing else.
    ///
    /// # This is a document edit, and every other program makes it a view
    /// setting
    ///
    /// The single most surprising thing about this verb, and the reason the
    /// triangle's hover text says it out loud. §12.3.3 Table 153 carries
    /// open-or-closed as the **sign** on `/Count` and defines no `/Open` key,
    /// so there is nowhere in the file to record a per-viewer answer. Expanding
    /// a bookmark therefore:
    ///
    /// * writes objects, and marks the document modified;
    /// * lands on the undo stack as one entry;
    /// * is **seen by everybody who opens the file afterwards**.
    ///
    /// An operator who collapses three chapters to find their place, saves, and
    /// sends the drawing out has changed what the recipient sees. That is not a
    /// defect — it is what the format is — and it is why the disclosure is on
    /// the control rather than in a release note.
    ///
    /// # The magnitude is the engine's problem, and getting it wrong is
    /// silent
    ///
    /// `set_outline_open` propagates the flip up the ancestor chain by the
    /// `/Count` **magnitude**, not by one, and its doc comment is emphatic:
    ///
    /// > *"a closed node contributes 1 (itself); an open one contributes
    /// > `1 + magnitude`. So expanding a node with magnitude 7 adds **7** to
    /// > every ancestor up to the first closed one — not 1, and not 8."*
    ///
    /// Nothing in this shell computes that, and nothing in this shell may. A
    /// wrong `/Count` is invisible: the file opens, the outline draws, and the
    /// only symptom is another reader's panel disagreeing about what is there.
    ///
    /// # A leaf is never asked
    ///
    /// An item with no descendants carries no `/Count` at all — Table 153 makes
    /// it *"required if the item has any descendants"* — so there is nothing to
    /// flip. `set_outline_open` answers `Ok(false)` rather than refusing,
    /// because *"asking a leaf to expand is what a 'collapse all' sweep does to
    /// every row it walks"*, and the panel simply draws no triangle on one.
    /// R83: never offer a control for something that cannot work.
    SetOpen {
        /// The bookmark whose `/Count` sign is being flipped.
        item: ObjId,
        /// `true` to expand it, `false` to collapse it.
        open: bool,
    },
}

/// Apply one bookmark verb.
///
/// The dispatch half of this module, reached from `PdfcerApp::apply`'s single
/// [`super::action::Action::Bookmark`] arm. It is a free function taking
/// `&mut OpenDoc` rather than a method, exactly like [`super::dimensions::apply`]
/// and [`super::pages::apply`], because the caller is the one place that owns
/// the borrow and the arm should be one line.
///
/// **Every arm goes through [`super::apply::vector_edit`]** — the
/// cancel–mutate–bump–invalidate protocol — and none of them may hand-roll it.
/// Its doc comment carries the argument: four hand-written copies of a
/// four-step protocol are four chances to omit a step, and the two steps most
/// easily omitted (the epoch bump and the structural resync) fail *silently*,
/// leaving an edit that happened in the document and did not happen on screen.
///
/// The `page` argument passed to `vector_edit` is **`0` for all three**, and
/// that is honest rather than lazy: an outline is document-level, no page is
/// being edited, and the parameter exists only so the diagnostic trace can say
/// which sheet a geometry edit touched. [`super::dimensions::apply`] passes `0`
/// for its group verbs for the identical reason. The one exception is
/// [`BookmarkAction::Add`], which passes the destination page — not because a
/// page is being changed, but because the page is the operand that decides what
/// the bookmark points at, and a trace that could not say which one would be
/// unable to check the commonest thing to get wrong.
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
