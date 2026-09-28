//! # `app::actions::pages` — the four page verbs, and the resync a structural
//! edit owes
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/pages.md`.

use std::path::Path;

use pdfcer_core::edit::EditSession;
use pdfcer_core::object::ObjId;

use crate::app::state::OpenDoc;
use crate::text::pages as t;

pub use pdfcer_gui_base::subactions::PageAction;

/// **Bring everything stated in page indices back into agreement with the
/// session.**
pub(super) fn resync(doc: &mut OpenDoc) {
    let before: Vec<(ObjId, u16)> = doc.pages.iter().map(|p| (p.id, p.rotate)).collect();
    let mut after = match doc.session.pages() {
        Ok(pages) => pages,
        Err(error) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("pages-resync-failed detail={error} kept={}", before.len())
            });
            return;
        }
    };
    let now: Vec<(ObjId, u16)> = after.iter().map(|p| (p.id, p.rotate)).collect();

    // The identity sequence, which is the fact that decides whether an INDEX
    // changed meaning. A rotation leaves it alone; a delete and a reorder do
    // not. Compared before `doc.pages` is overwritten, because afterwards
    // there is nothing left to compare against.
    let renumbered = before
        .iter()
        .map(|(id, _)| *id)
        .ne(now.iter().map(|(id, _)| *id));
    let structure_changed = now != before;

    // **THE PAGE VECTOR IS REPLACED ON EVERY EDIT, NOT ONLY A STRUCTURAL
    // ONE.**
    //
    // An early return on `now == before` is the tempting shape and it is
    // wrong: `page_tree::Page` is not an id, it is a RESOLVED page, with its
    // `/Contents` and its `/Resources` in it. An edit that rewrites the page
    // dictionary without changing the page's object id compares equal and
    // leaves `doc.pages` describing the document as it was.
    //
    // `EditSession::add_image` is the verb that shows it: it turns `/Contents`
    // from a stream into an array and adds an `/XObject` to `/Resources`, and
    // the page's id does not move. Skip the replacement and the canvas and the
    // Objects panel go on reading a `Page` whose `/Contents` names the old
    // stream alone — an inserted image that appears on neither, while the
    // bytes on disk are right, so saving and reopening shows it.
    //
    // # Why the other edits would look fine anyway
    //
    // * A **markup** is an annotation. `/Annots` is read from the session, not
    //   from this vector.
    // * A **move** rewrites a content stream **in place** — same stream object
    //   — so a stale `Page`'s `/Contents` reference still resolves to the
    //   right object and re-reading it gets the new bytes.
    //
    // `add_image` is the first verb that changes what `/Contents` *is*, which
    // is why the class of defect is invisible until one exists.
    //
    // # What it costs to replace it every time: nothing
    //
    // `after` has already been walked, three lines above. The early return
    // saved one `Vec` assignment and bought a class of stale-view defect.
    // The visible area, as every reader computes it; see `pagebox`.
    pdfcer_gui_base::pagebox::clip_crop_to_media(&mut after);
    doc.pages = after;

    // Keep the per-page revision vector the same length as the document
    // (O74). Growth fills with the document-wide floor, so a page that has
    // just arrived reports the most conservative number available and no cache
    // mistakes it for one it has a picture of. This is deliberately NOT a
    // bump: a document gaining a page at the end has not changed page 0, and a
    // rail that redrew page 0 for it would be `pageepoch`'s own defect.
    doc.page_epochs.resize(doc.pages.len());

    if !structure_changed {
        // No page was added, removed, reordered or turned. The vector above is
        // now current, every cached raster is still a picture of the right
        // sheet, and none of the heavier invalidation below applies — that is
        // what this comparison is genuinely for, and it is all it is for.
        return;
    }

    let page_count = doc.pages.len();

    // Every cached raster in the strip is keyed on a page **index**, and an
    // index that has changed meaning — or a sheet that has been turned — makes
    // every one of them a picture of something else. Cleared wholesale rather
    // than selectively: working out which strip entries survive a permutation
    // is a second statement of the permutation, and the cache refills from the
    // visible set on the next frame anyway.
    doc.strip_rasters.clear();

    // …and the CURRENT page's raster, for the same reason and only for that
    // reason.
    //
    // `vector_edit` keeps the raster and signals staleness through
    // `page_texture_epoch`, so the drop belongs where its REASON is — dropping
    // it there instead, on every edit, blanks the page on every ordinary one.
    // The
    // distinction is the whole of it: after a content edit the old raster is
    // an older picture of the same sheet, and showing it for two frames is
    // right. After a delete or a reorder it is a picture of a **different
    // sheet** — the index resolves elsewhere — and showing it would be wrong
    // rather than merely late.
    doc.page_texture = None;

    if renumbered {
        // **Every per-page revision is now meaningless** (O74). Page *n*
        // is a different sheet, so a thumbnail whose stored epoch matches page
        // n's counter is a picture of the WRONG DRAWING — the one failure mode
        // that outranks the slowness the counter exists to fix, because it is
        // rule 4's "sneaky" rather than merely late.
        //
        // Raised HERE, after `vector_edit_scoped` has already applied whatever
        // the verb asked for, which is what makes narrowing a verb a bounded
        // mistake: a caller can be wrong about which page's CONTENT changed
        // and still cannot be wrong about which sheet an index names.
        doc.page_epochs.bump_all();
        // The canvas selection names objects by paint-order index **on a page
        // index**, and that index now resolves to a different sheet. Cleared
        // rather than remapped: `SelectionState` exposes no way to rewrite an
        // entry's page, and adding one would put a page-remapping rule inside
        // the module that owns object identity. See the module header's table.
        doc.selection.clear();
        // …and the text selection, for the same reason — except that this one
        // is already keyed on the epoch `vector_edit` has bumped, so dropping
        // it is belt to that braces. It is dropped anyway because "already
        // stale by its key" and "gone" read differently in a debugger, and the
        // wash it paints is the most visible piece of state in the canvas.
        doc.text_selection = None;
        // The operator may have been on a page that no longer exists.
        // `go_to_page` clamps, which is the only defined answer — there is no
        // page to return to, and refusing to move would leave the canvas
        // pointed past the end of the document.
        doc.view.go_to_page(doc.view.page_index, page_count);
        // `tracked_page` follows, exactly as it does for a page-display change:
        // leaving it behind makes `canvas::strip` read the clamp as a
        // *navigation* and scroll to it on the next frame, so a delete would
        // jump the view for a reason the operator did not cause.
        doc.tracked_page = doc.view.page_index;
    }

    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "pages-resync was={} now={page_count} renumbered={} page={} epoch={}",
            before.len(),
            u8::from(renumbered),
            doc.view.page_index + 1,
            doc.edit_epoch,
        )
    });
}

// ---------------------------------------------------------------------------
// The disclosures a delete owes
// ---------------------------------------------------------------------------

/// **What removing these pages broke, as operator-facing sentences.**
fn delete_disclosures(
    dangling: &pdfcer_core::pageops::DanglingReport,
    sets_split: usize,
) -> Vec<String> {
    let mut notes = Vec::new();
    if dangling.outline_items > 0 {
        notes.push(t::deleted_dangling_bookmarks(dangling.outline_items));
    }
    if dangling.links > 0 {
        notes.push(t::deleted_dangling_links(dangling.links));
    }
    if dangling.named_destinations > 0 {
        notes.push(t::deleted_dangling_destinations(
            dangling.named_destinations,
        ));
    }
    if dangling.page_labels_stale {
        notes.push(t::deleted_page_labels_stale().to_owned());
    }
    if sets_split > 0 {
        notes.push(t::deleted_separations_repaired(sets_split));
    }
    notes
}

// ---------------------------------------------------------------------------
// The four verbs
// ---------------------------------------------------------------------------

/// **Insert another document's pages after `after_page`.**
pub(super) fn merge_into(doc: &mut OpenDoc, path: &Path) {
    // Loaded OUTSIDE the edit closure and borrowed inside it, for
    // `insert_from_file`'s stated reason: `merge_document` takes a
    // `DocumentView` over it, so it must outlive the call, and loading it
    // inside would mean reporting a *load* failure from a context that can only
    // report an *edit* failure.
    let source = match pdfcer_core::document::Document::load(path) {
        Ok(source) => source,
        Err(error) => {
            let detail = error.to_string();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("merge-refused path={path:?} reason={detail}")
            });
            super::record_note(doc.edit_epoch, crate::text::pages::merge_failed(&detail));
            return;
        }
    };
    let view = source.view();
    super::apply::vector_edit(doc, "merge-document", 0, 1, move |session| {
        session
            .merge_document(&view, pdfcer_core::pageops::InsertPosition::End)
            .map(|outcome| {
                crate::diag::trace(|| {
                    // `-applied`, not the bare `merge-document` `vector_edit`
                    // writes. Two lines sharing a trace name is how a driven
                    // check reads the wrong one and reports that a verb did
                    // nothing — this project has made that mistake twice, and
                    // the convention is what stops the third.
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!(
                        "merge-document-applied pages={} fields={} renamed={} bookmarks={}",
                        outcome.pages_merged,
                        outcome.fields_merged,
                        outcome.fields_renamed,
                        outcome.outline_items_carried
                    )
                });
                crate::text::pages::merged(&outcome)
            })
    });
}

pub(super) fn insert_from_file(
    doc: &mut OpenDoc,
    path: &Path,
    pages: &[usize],
    position: pdfcer_core::pageops::InsertPosition,
) {
    // Loaded OUTSIDE the edit closure and borrowed inside it: `insert_pages`
    // takes a `DocumentView` over it, so it has to outlive the call — and
    // loading it inside would mean deciding what to do about a *load* failure
    // from a context that can only report an *edit* failure.
    let source = match pdfcer_core::document::Document::load(path) {
        Ok(source) => source,
        Err(error) => {
            let detail = error.to_string();
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "insert-pages-refused path={path:?} reason={detail}"
                )
            });
            super::record_note(doc.edit_epoch, crate::text::pages::insert_failed(&detail));
            return;
        }
    };
    insert_from_view(doc, &source.view(), pages, position);
}

/// **The half of an insert that does not care where the source came from.**
pub(super) fn insert_from_view(
    doc: &mut OpenDoc,
    view: &pdfcer_core::view::DocumentView<'_>,
    pages: &[usize],
    position: pdfcer_core::pageops::InsertPosition,
) -> usize {
    if pages.is_empty() {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "insert-pages-refused reason=no-pages".to_owned()
        });
        super::record_note(
            doc.edit_epoch,
            crate::text::pages::insert_empty().to_owned(),
        );
        return 0;
    }
    let count = pages.len();

    // Where the first inserted sheet will land, computed BEFORE the edit.
    //
    // Afterwards the document has more pages and `position` no longer names a
    // slot in it — `End` in particular means something different once the
    // pages have arrived. Working it out here is also what lets this be a
    // plain number rather than a second interpretation of `InsertPosition`.
    let landing = match position {
        pdfcer_core::pageops::InsertPosition::Start => 0,
        pdfcer_core::pageops::InsertPosition::End => doc.pages.len(),
        pdfcer_core::pageops::InsertPosition::Before(n) => n,
        pdfcer_core::pageops::InsertPosition::After(n) => n.saturating_add(1),
        // `InsertPosition` is `#[non_exhaustive]`: a variant added upstream
        // lands the operator on the page they were already on, which is wrong
        // but harmless, where a panic would lose the insert they just made.
        _ => doc.view.page_index,
    };
    let before = doc.pages.len();

    super::apply::vector_edit(doc, "insert-pages", landing, count, |session| {
        session
            .insert_pages(view, pages, position)
            // `InsertOutcome`, not a `usize`, and the second field is the
            // one this shell needs. `orphaned_widgets`
            // is EXACT rather than an upper bound (the engine's reply: no field
            // in the target can be claiming a widget that just arrived, because
            // `/AcroForm` is not merged and every object number is remapped), so
            // the number goes in front of the operator unhedged and a zero drops
            // the clause entirely.
            //
            // `orphaned_widgets_unrecoverable` is beside it, and the two
            // numbers are two different pieces of news. The engine
            // measured its own output and found that of 13 orphans, 11 could be
            // registered and 2 had lost their identity permanently — and said
            // plainly that the old undifferentiated sentence *"is true of both
            // and useful for only one"*, because it describes a chore for the
            // 11 and a permanent loss for the 2, in the milder wording.
            .map(|outcome| {
                vec![crate::text::pages::inserted(
                    outcome.pages_inserted,
                    outcome.orphaned_widgets,
                    outcome.orphaned_widgets_unrecoverable,
                    crate::text::pages::Structures {
                        outline_dropped: outcome.source_outline_dropped,
                        labels_dropped: outcome.source_page_labels_dropped,
                        labels_stale: outcome.page_labels_stale,
                    },
                    landing,
                )]
            })
    });

    // GO TO WHAT WAS INSERTED — the half that makes this a feature rather
    // than a verb.
    //
    // An operator who inserts four sheets wants to see them; leaving the view
    // on the page they were reading means the only evidence anything happened
    // is a sentence in the status bar. The question a verb has to answer is
    // *what would a competent user reach for next, within this same gesture?*
    // — and the answer is "look at them".
    //
    // Guarded on the page count actually having grown, so a refused insert
    // does not navigate: `vector_edit` reports a refusal to the trace and the
    // disclosure, and moving the view on a failure would be a second, wordless
    // claim that something landed.
    if doc.pages.len() > before {
        let target = landing.min(doc.pages.len().saturating_sub(1));
        doc.view.go_to_page(target, doc.pages.len());
        doc.tracked_page = doc.view.page_index;
        crate::diag::trace(|| {
            format!(
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                "insert-pages-landed at={target} pages={} was={before}",
                doc.pages.len()
            )
        });
    }
    // The page vector is the oracle, not the operand count: `insert_pages` is
    // free to insert fewer than it was asked for, and a caller about to delete
    // the originals must be told what actually arrived rather than what was
    // requested.
    doc.pages.len().saturating_sub(before)
}

/// The engine call behind [`Action::RotatePages`].
///
/// Handed to `vector_edit` as a closure rather than run here, so the whole
/// four-step protocol — cancel the worker, mutate through `Arc::get_mut`, bump
/// the epoch, drop the texture — is the one in `apply.rs` and not a fifth copy.
///
/// The `usize` `rotate_pages` returns is how many pages **actually changed**,
/// and it is discarded: a page already at the requested rotation contributes
/// nothing and records no command, which is the engine's business and not a
/// disclosure. The empty list is the statement that a rotation rewrites no
/// operator's form and therefore owes rule 4 nothing.
///
/// [`Action::RotatePages`]: super::Action::RotatePages
pub(super) fn rotate(
    session: &mut EditSession,
    pages: &[usize],
    delta: i32,
) -> Result<Vec<String>, pdfcer_core::edit::EditError> {
    session.rotate_pages(pages, delta).map(|_| Vec::new())
}

/// The engine call behind [`Action::DeletePages`], with its disclosures.
///
/// The one verb in this file whose return value carries sentences — see
/// [`delete_disclosures`].
///
/// [`Action::DeletePages`]: super::Action::DeletePages
pub(super) fn delete(
    session: &mut EditSession,
    pages: &[usize],
    separations: pdfcer_core::pageops::SeparationPolicy,
) -> Result<Vec<String>, pdfcer_core::edit::EditError> {
    // `delete_pages_with`, not `delete_pages` — the latter is what makes
    // the operator's separation policy a broken promise.
    //
    // `delete_pages` delegates to this verb with `SeparationPolicy::Repair`
    // hard-coded, so Settings > Pages > "what to do when deleting pages splits
    // a preseparated set" was persisted, validated, drawn in a window, and
    // consulted by nothing. An operator who chose **Refuse** — the choice that
    // exists so a print-production file cannot be quietly half-separated — got
    // Repair.
    //
    // The engine named this call as the fix in advance: *"Exists so the
    // policy is reachable before there is a settings store to reach it from …
    // when operator settings land this is the entry point they drive."* The
    // store landed months ago and the entry point was never taken. Found by
    // `tools/verb-coverage.py`, which lists the engine verbs nothing here
    // calls; this one was a MISS sitting next to a HIT for the same act, which
    // is the shape a reader skims past.
    //
    // Under `Refuse` the engine returns `EditError::SeparationSplit` and
    // `vector_edit` traces the refusal. That is the operator's own instruction
    // being honoured, not a fault.
    session
        .delete_pages_with(pages, separations)
        .map(|outcome| {
            crate::diag::trace(|| {
                format!(
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "pages-deleted removed={} freed={} bookmarks={} links={} \
                 destinations={} labels_stale={}",
                    outcome.pages_removed,
                    outcome.objects_freed,
                    outcome.dangling.outline_items,
                    outcome.dangling.links,
                    outcome.dangling.named_destinations,
                    u8::from(outcome.dangling.page_labels_stale),
                )
            });
            delete_disclosures(&outcome.dangling, outcome.separations.sets_split)
        })
}

/// The engine call behind [`Action::ReorderPages`].
///
/// `order` is passed through untouched: `crate::panels::pages::ops::move_order`
/// builds it to `reorder_pages`' contract and the engine re-checks that it is a
/// permutation, so there is nothing for this layer to add and one more place
/// for a rule to be restated if it did.
///
/// [`Action::ReorderPages`]: super::Action::ReorderPages
pub(super) fn reorder(
    session: &mut EditSession,
    order: &[usize],
) -> Result<Vec<String>, pdfcer_core::edit::EditError> {
    session.reorder_pages(order).map(|()| Vec::new())
}

// ---------------------------------------------------------------------------
// Extract — the verb that changes no document
// ---------------------------------------------------------------------------

/// Apply one page verb, and do the invalidation it owes the shell.
pub(super) fn apply(
    doc: &mut OpenDoc,
    panels: &mut crate::panels::PanelsState,
    action: PageAction,
    // The one setting any page verb consults, threaded in rather than read
    // here: this function is handed `&mut OpenDoc`, and the configuration
    // belongs to the application rather than to the document. See `delete`.
    separations: pdfcer_core::pageops::SeparationPolicy,
) {
    match action {
        // ===============================================================
        // THE PAGE VERBS
        //
        // Four arms, each one call, because everything that could be a
        // rule lives elsewhere: the operand list and the permutation in
        // `crate::panels::pages::ops` (pure, unit-tested), the engine call
        // and the disclosures in `super::pages`, and the four-step
        // protocol in `vector_edit` — which now carries a fifth step that
        // brings the page vector, the strip rasters, the canvas selection
        // and the view back into agreement with the session.
        //
        // `page=` on the trace line is the FIRST operand rather than "the
        // page", and `n=` is how many were named. There is no single page
        // a multi-page verb is about; the first one is the honest answer
        // and `n=` is the field that actually says what happened, exactly
        // as `history_step`'s own docs argue for the undo case.
        // ===============================================================
        PageAction::RotatePages { pages, delta } => {
            if !pages.is_empty() {
                let first = pages.first().copied().unwrap_or(0);
                super::apply::vector_edit(doc, "rotate-pages", first, pages.len(), |session| {
                    rotate(session, &pages, delta)
                });
            }
        }
        // **The paper changes and the drawing does not.** The body, the
        // measurement behind that sentence and the rule-4 disclosures are all
        // in `super::pagesize`; this arm only routes, exactly as the rotate arm
        // above it does.
        //
        // The funnel label is `page-size-changed`, and the suffix is not
        // decoration: `vector_edit` publishes `<label> page= n= epoch=
        // disclosures=`, `ui-verify` matches a trace line on its FIRST token,
        // and `super::pagesize` publishes `page-size-sheet` and
        // `page-size-applied` of its own. Three distinct first tokens, which is
        // what `tools/gates/check-trace-names.py` exists to keep true — the
        // convention was broken three times in three days before it existed.
        //
        // Guarded on a non-empty operand list for the rotate arm's reason: the
        // engine would accept an empty selection and record nothing, producing
        // a control the operator pressed that changed nothing and said nothing.
        PageAction::SetPageSize { pages, rect } => {
            if !pages.is_empty() {
                let first = pages.first().copied().unwrap_or(0);
                super::apply::vector_edit(
                    doc,
                    "page-size-changed",
                    first,
                    pages.len(),
                    |session| super::pagesize::set(session, &pages, rect),
                );
            }
        }
        // **The destructive one**, and the one that renumbers.
        //
        // Two things happen here that no other arm needs, and both are
        // about a *position* ceasing to mean what it meant:
        //
        // 1. `vector_edit`'s resync clears the **canvas** selection and
        //    clamps the view — see `super::pages::resync`;
        // 2. the **Pages panel's** picks are cleared here, because they
        //    live on `self.panels` rather than on the document and
        //    `vector_edit` cannot reach them.
        //
        // The panel's own `retain_below` would drop the picks that fell
        // off the end on the next frame, and that is NOT sufficient: the
        // pages that were deleted are exactly the ones that were picked,
        // so the survivors of a clamp would be picks pointing at sheets
        // that have shuffled down into their indices. Clearing is both
        // correct and provable — every picked sheet is gone.
        //
        // Guarded on the epoch rather than on a return value, so the
        // clear happens only for an edit that actually applied: a refused
        // delete (the engine refuses removing every page, §7.7.3.3) must
        // leave the operator's selection exactly as they built it.
        //
        // **No confirmation dialog.** `crate::app::save::save_pending` is
        // the one predicate this application consults before a destructive
        // path and it is about a save being in flight, not about unsaved
        // work; the engine records this as an undoable command; and
        // nothing reaches disk — the operator's file is untouched until
        // they choose to save a copy. A modal here would be the only one
        // in the application and would be asking about the one destructive
        // act that is already reversible in the session.
        // **The page paste** — O59 item 2 — and it is three lines because
        // it is a THIRD SOURCE for a path that already exists.
        //
        // `insert_from_view` is the shared half of `insert_from_file`, reached
        // also by a page dragged between two open documents. A clipboard paste
        // is the third route, and it wants every single
        // thing that function already does: the landing calculation done before
        // the edit, `orphaned_widgets` and `orphaned_widgets_unrecoverable`
        // reported as two different pieces of news, the dropped outline and page
        // labels, and the jump to what was inserted.
        //
        // ⇒ Writing a fourth of those would have been a second wording of the
        // most consequential disclosure in this file — the orphaned widgets the
        // engine flagged as *"the one that produces a document that looks right
        // and is not"*.
        PageAction::PastePages { bytes, after } => {
            // Loaded OUTSIDE the call for `insert_from_file`'s reason: the view
            // borrows it and must outlive the insert.
            let source = match pdfcer_core::document::Document::from_bytes(bytes) {
                Ok(source) => source,
                Err(error) => {
                    let detail = error.to_string();
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!("paste-pages-refused reason={detail}")
                    });
                    super::record_note(doc.edit_epoch, crate::text::pages::insert_failed(&detail));
                    return;
                }
            };
            let view = source.view();
            // EVERY page of the clip, because the clip is exactly what was
            // copied — the operator already chose which sheets when they pressed
            // Copy, and asking again at the paste would be a second selection
            // for one decision.
            // `pages_in` returns the page list or a tree error, and a clip
            // whose page tree will not walk is a clip that cannot be pasted at
            // all -- reported rather than silently pasting nothing, because the
            // operator pressed Paste and is owed an answer either way.
            let count = match pdfcer_core::page_tree::pages_in(&source) {
                Ok(pages) => pages.len(),
                Err(error) => {
                    let detail = error.to_string();
                    crate::diag::trace(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        format!("paste-pages-refused reason=page-tree detail={detail}")
                    });
                    super::record_note(doc.edit_epoch, crate::text::pages::insert_failed(&detail));
                    return;
                }
            };
            let pages: Vec<usize> = (0..count).collect();
            insert_from_view(
                doc,
                &view,
                &pages,
                pdfcer_core::pageops::InsertPosition::After(after),
            );
        }
        PageAction::DeletePages { pages } => {
            if !pages.is_empty() {
                let first = pages.first().copied().unwrap_or(0);
                let before = doc.edit_epoch;
                super::apply::vector_edit(doc, "delete-pages", first, pages.len(), |session| {
                    delete(session, &pages, separations)
                });
                if doc.edit_epoch != before {
                    panels.pages_mut().selection.clear();
                }
            }
        }
        // **The middle case**: every page survives, and every index
        // means a different sheet.
        //
        // The canvas selection is cleared by the resync; the panel's picks
        // are **remapped** rather than cleared, because the permutation
        // states exactly where each picked sheet went. See
        // `crate::panels::pages::select::PageSelection::remap` for why the
        // two selections get different answers to the same edit — and for
        // why clearing here would make the reorder arrows unusable twice
        // in a row, which is the one gesture they exist for.
        PageAction::ReorderPages { order } => {
            if !order.is_empty() {
                let before = doc.edit_epoch;
                super::apply::vector_edit(doc, "reorder-pages", 0, order.len(), |session| {
                    reorder(session, &order)
                });
                if doc.edit_epoch != before {
                    let landed = crate::panels::pages::ops::inverse(&order);
                    panels.pages_mut().selection.remap(&landed);
                }
            }
        }
        // The one page verb that goes nowhere near `vector_edit`: it
        // changes no document, it opens a native save dialog, and it is an
        // `Action` for `Action::SaveCopy`'s frame-timing reason and only
        // that one. See `super::pages::extract`.
        // The one verb that reads a SECOND document, and the only page
        // action whose consequence is a navigation rather than an
        // invalidation: `insert_from_file` goes to what it inserted, because
        // an operator who inserts four sheets wants to see them.
        PageAction::MergeIntoDocument { path } => merge_into(doc, &path),
        PageAction::InsertPagesFromFile {
            path,
            pages,
            position,
        } => insert_from_file(doc, &path, &pages, position),
        PageAction::ExtractPages { pages } => super::extract::extract(doc, &pages),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{FOUR_PAGES, open_fixture};

    /// Apply one engine verb to a fixture, the way `vector_edit` does.
    fn edit(doc: &mut OpenDoc, verb: impl FnOnce(&mut EditSession)) {
        let session = std::sync::Arc::get_mut(&mut doc.session)
            .expect("nothing else holds the session in a test");
        verb(session);
        doc.edit_epoch = doc.edit_epoch.wrapping_add(1);
        resync(doc);
    }

    /// Put one object on `page` into the canvas selection.
    fn select_object_on(doc: &mut OpenDoc, page: usize) {
        use crate::panels::objects::provider::TargetId;
        doc.selection.marquee(page, &[TargetId::Object(0)], false);
    }

    /// **A delete shortens the page vector, and the view follows it.**
    #[test]
    fn a_delete_shortens_the_page_vector_and_clamps_the_view() {
        let mut doc = open_fixture(FOUR_PAGES);
        assert_eq!(doc.pages.len(), 4, "the fixture must have four pages");
        doc.view.page_index = 3;

        edit(&mut doc, |s| {
            s.delete_pages(&[2, 3]).expect("two of four pages may go");
        });

        assert_eq!(doc.pages.len(), 2, "the page vector must follow the delete");
        assert_eq!(
            doc.view.page_index, 1,
            "the operator was on page 4, which no longer exists; the view must land on the \
             last page that does rather than pointing past the end"
        );
        assert_eq!(
            doc.tracked_page, doc.view.page_index,
            "a clamp the canvas reads as a NAVIGATION scrolls the strip for a reason the \
             operator did not cause"
        );
    }

    /// **A delete clears the canvas selection, because its page index now
    /// names a different sheet.**
    #[test]
    fn a_delete_clears_the_canvas_selection() {
        let mut doc = open_fixture(FOUR_PAGES);
        select_object_on(&mut doc, 2);
        assert!(!doc.selection.is_empty(), "the fixture selection must take");

        edit(&mut doc, |s| {
            s.delete_pages(&[0]).expect("one of four pages may go");
        });

        assert!(
            doc.selection.is_empty(),
            "the selection named an object on page 3, and page 3 is now the sheet that was \
             page 4 — the entry survives as a pointer at something nobody chose"
        );
    }

    /// **A reorder renumbers without shortening, and is treated as such.**
    #[test]
    fn a_reorder_renumbers_and_clears_the_canvas_selection() {
        let mut doc = open_fixture(FOUR_PAGES);
        let before: Vec<ObjId> = doc.pages.iter().map(|p| p.id).collect();
        select_object_on(&mut doc, 0);

        edit(&mut doc, |s| {
            s.reorder_pages(&[1, 0, 2, 3]).expect("a legal permutation");
        });

        assert_eq!(doc.pages.len(), 4, "a reorder removes nothing");
        let after: Vec<ObjId> = doc.pages.iter().map(|p| p.id).collect();
        assert_ne!(
            before, after,
            "the page vector still describes the old order, so every index in the application \
             now names the wrong sheet"
        );
        assert_eq!(after[0], before[1], "page 2 must have moved to position 1");
        assert!(
            doc.selection.is_empty(),
            "the selection's page index survived a permutation of the pages"
        );
    }

    /// **A rotation is NOT a renumbering, and the selection survives it.**
    #[test]
    fn a_rotation_refreshes_the_pages_without_clearing_the_selection() {
        let mut doc = open_fixture(FOUR_PAGES);
        let before = doc.pages[1].rotate;
        select_object_on(&mut doc, 1);

        edit(&mut doc, |s| {
            s.rotate_pages(&[1], 90).expect("a quarter turn is legal");
        });

        assert_eq!(
            doc.pages[1].rotate,
            (before + 90) % 360,
            "the page vector still carries the old rotation, so the canvas would keep drawing \
             the sheet the way it was"
        );
        assert!(
            !doc.selection.is_empty(),
            "a rotation adds and removes no operator, so nothing renumbered and there was \
             nothing to clear"
        );
        assert_eq!(doc.pages.len(), 4);
    }

    /// An edit that touches no page leaves the vector alone and the selection
    /// with it.
    #[test]
    fn an_edit_that_changes_no_page_resyncs_nothing() {
        let mut doc = open_fixture(FOUR_PAGES);
        select_object_on(&mut doc, 0);
        let before: Vec<ObjId> = doc.pages.iter().map(|p| p.id).collect();

        edit(&mut doc, |_| {});

        assert_eq!(doc.pages.iter().map(|p| p.id).collect::<Vec<_>>(), before);
        assert!(!doc.selection.is_empty());
    }

    /// A delete with nothing to disclose produces **no** sentence, and one with
    /// something to disclose produces one per fact.
    #[test]
    fn a_delete_discloses_one_sentence_per_broken_thing() {
        let mut dangling = pdfcer_core::pageops::DanglingReport::default();
        assert!(
            delete_disclosures(&dangling, 0).is_empty(),
            "a clean delete owes rule 4 nothing"
        );

        dangling.outline_items = 3;
        dangling.page_labels_stale = true;
        let notes = delete_disclosures(&dangling, 0);
        assert_eq!(
            notes.len(),
            2,
            "one sentence per fact, and no more: {notes:?}"
        );
        assert!(
            notes[0].contains('3'),
            "the count must reach the operator: {notes:?}"
        );
        assert!(
            notes.iter().all(|n| n.ends_with('.')),
            "these are prose and take a full stop, per the catalog's conventions: {notes:?}"
        );
    }
}
