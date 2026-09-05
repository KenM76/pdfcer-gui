//! # `app::actions::redact` — the three arms that MARK content for removal,
//! and the one that removes it
//!
//! Split out of [`super::apply`] on 2026-08-18 under rule R2, and the seam is a
//! real one rather than a line count.
//!
//! These are the only arms whose subject is **marking content for removal**.
//! They share a vocabulary nothing else in the funnel uses — `RedactAppearance`,
//! the mark census, the annotation ids a review surface addresses a mark by —
//! and their comments carry the argument for the one operation pdfcer cannot
//! undo. Moving the arms and leaving the reasoning behind would have been
//! exactly the split this project's own R2 note warns against.
//!
//! ## ★★★ CORRECTED 2026-09-04 (evening), AND AGAIN 2026-09-05 — the removal
//! is armed here, and the old section said it could never be here at all
//!
//! What stood here, verbatim, until the afternoon of 2026-09-04:
//!
//! > *"## ★ What is NOT here, and that is the point. **Nothing in this file
//! > removes anything.** Marking is the reversible half of redaction; the
//! > irreversible half is `crate::dialogs::redact`, which reaches no arm in
//! > this funnel at all — it changes no document through the queue, so it has
//! > nothing to order against and no epoch to bump. Routing the one operation
//! > that cannot be undone through a queue that replays would be the defect,
//! > not the tidiness."*
//!
//! The middle clause was a **fact about the engine**, not a principle, and it
//! expired that day: `EditSession::apply_redactions` (`Pass 250.1`) applied a
//! redaction into the open session, so the operation *did* change a document
//! through the queue and *did* have an epoch to bump.
//!
//! ★★★ **And a day later the engine changed again, in the direction that makes
//! the ORIGINAL paragraph's instinct look better than its conclusion.**
//! `Pass 250.2`'s [`crate::redact::stage_into_session`] does not remove
//! anything and does not touch the session's content: it arms the next save.
//! So *"nothing in this file removes anything"* is true once more — and it is
//! still not a principle, which is the whole lesson of having written it as
//! one. It is a dated property of an engine this project does not build.
//!
//! ⇒ The arm below is [`Action::PendingRedaction`], and it goes through the
//! identical `vector_edit` funnel as the three marking arms, for a reason that
//! is worth stating because the funnel does more than this verb needs: the
//! engine's staging and cancelling verbs both take `&mut EditSession`, and
//! `Arc::get_mut` — which is the funnel's second step and cannot be had any
//! other way here — is what makes them reachable at all. The epoch bump comes
//! with it and is wanted for its own reason (below).
//!
//! ★ The old paragraph's last sentence — *"routing the one operation that
//! cannot be undone through a queue that replays would be the defect"* —
//! deserves an answer rather than a deletion, because it names a real hazard.
//! **This queue does not replay.** `crate::app::actions` drains it once per
//! frame in order and discards it. And on `Pass 250.2` the hazard is smaller
//! still: the arming is not irreversible — [`crate::redact::Staging::Cancel`]
//! is the second half of this very arm.
//!
//! ## ★★★ Why the epoch is bumped for an edit that changes no pixel
//!
//! Staging alters nothing a rasteriser would draw (rule 4: no badge, no tint,
//! no provisional layer — see `crate::redact` §1.0.3), so on the face of it a
//! funnel that drops every page texture and rebuilds the decomposition is pure
//! waste. It is bumped anyway, and the reason is one specific consumer:
//!
//! `crate::app::save::has_unsaved_edits` is
//! `(is_modified() || has_pending_redaction()) && edit_epoch != saved_epoch`.
//! **Without the bump the second term is false**, and a document whose marks
//! were already in the file when it was opened — arm the removal, change
//! nothing else — answers *clean*, closes with no prompt, and loses the arming
//! in silence. The waste is one re-raster of an identical picture. The
//! alternative is the exact silent loss the third term was added to close.
//!
//! ## What is still NOT here
//!
//! **The write.** This arm arms the save and stops. Where the bytes go, and
//! when, is `file.save` / `file.save_as` / `file.save_copy`'s decision, exactly
//! as it is for every other edit — which is what the operator asked for in
//! `OPERATOR_REQUESTS.md` O125. `crate::app::save::write_copy` is what routes
//! them through the removal. The two *write-now* destinations still live in
//! `crate::dialogs::redact` and still reach no arm in this file.

use super::Action;
use super::apply::vector_edit;
use crate::app::state::OpenDoc;
use crate::redact::Staging;

/// Apply one of the three marking actions, or the staging one.
///
/// Takes the whole `Action` rather than destructured fields, so the match here
/// is the same shape as the one it was lifted out of and a reader comparing
/// them sees one dispatch rather than two spellings of it.
///
/// # Panics
///
/// Never. The `_` arm is unreachable — `super::apply` routes only the four
/// redaction variants here — and it is spelled rather than `unreachable!()`
/// because a future fifth variant sent here by mistake should do nothing
/// visible rather than end the process an operator is mid-edit in.
pub fn apply(doc: &mut OpenDoc, action: Action) {
    match action {
        // ===============================================================
        // ★ THE REDACTION MARKING VERBS
        //
        // Three arms, each one call, through the same `vector_edit` funnel
        // every other document change uses — which is the whole reason they
        // are one line each. Marking is an ordinary edit: it authors an
        // annotation, the engine records it as an undoable command, and the
        // page has to re-raster because a `/Redact` mark draws a red
        // outline the operator needs to see.
        //
        // ★ **Nothing in THESE THREE removes anything** — corrected
        // 2026-09-04, when the fourth arm arrived. This comment used to say
        // "nothing here removes anything" of the whole file and to argue
        // that the irreversible half could never reach this funnel. See the
        // module header for why that expired the day
        // `EditSession::apply_redactions` shipped. Marking is still the
        // reversible half and these three are still all of it.
        //
        // `.map(|_| Vec::new())` on the first two adapts the engine's
        // `Vec<ObjId>`/`ObjId` to the disclosure list `vector_edit` traces,
        // and the empty vec is a statement rather than a placeholder —
        // authoring an annotation rewrites no existing operator, so nothing
        // changed form and rule 4 owes the operator nothing. It is the same
        // adaptation `CommitMarkup` makes one screen up.
        // ===============================================================
        Action::MarkRedactionsBySearch {
            query,
            pattern,
            appearance,
        } => {
            if !query.is_empty() {
                let page = doc.view.page_index;
                // The label distinguishes the two marking modes on the
                // trace, because a pattern that marked nothing and a
                // literal that marked nothing are different diagnoses:
                // one is a query the document does not contain, the other
                // is very often a `#` the operator meant literally.
                let label = if pattern {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "redact-mark-pattern"
                } else {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    "redact-mark-search"
                };
                let before = crate::panels::redact::mark_ids(&doc.session).len();
                // How many fonts in this document store text no search could
                // reach. Set inside the closure below; see the note there for
                // why a redaction owes this disclosure and a search merely
                // benefits from it.
                let mut unreadable: u64 = 0;
                vector_edit(doc, label, page, 1, |session| {
                    // ★ Case-INSENSITIVE, always, and it is not a missing
                    // control. Over-marking is the safe direction of error
                    // on this verb and under-marking is not: a mark the
                    // operator did not want is one row and one click in the
                    // review list, and a mark they did want and did not get
                    // is a name shipped in a document they believe is
                    // redacted. The old shell made the same ruling in the
                    // same words.
                    //  The `_styled` verbs, which arrived on 2026-08-17
                    // (`a7210a4`) in answer to this shell's filing: before
                    // them `author_text_matches` built its spec internally
                    // with `fill: None`, so a fill the operator chose was
                    // discarded on this path and honoured on the whole-page
                    // one. A control honoured on some marks and silently
                    // dropped on others is worse than no control, on the
                    // one operation that cannot be undone.
                    // ★★★ `search_and_mark_redactions_styled`, NOT
                    // `mark_redactions_by_search_styled` — Pass 127.1, wired
                    // 2026-08-25.
                    //
                    // The two run the identical scan and author the identical
                    // marks. The difference is that this one also hands back
                    // the extraction diagnostics, and on THIS operation that is
                    // not a nicety.
                    //
                    // `Vec<ObjId>` coming back empty has two causes with one
                    // appearance: the term is not in the document, or the
                    // document's text was never recoverable as Unicode so no
                    // term could ever have matched it. For a search that
                    // ambiguity wastes a minute. **For a redaction it fails in
                    // the direction nobody catches**: the operator asked for
                    // every occurrence of a name to be removed, the run
                    // reported success, the file still contains it — and then
                    // they send it.
                    //
                    // ★ Both populations RENDER PERFECTLY, which is what makes
                    // it invisible. Nothing on the page looks unredacted.
                    let marked = if pattern {
                        session
                            .mark_redactions_by_pattern_styled(&query, true, &appearance)
                            .map(|created| (created, None))
                    } else {
                        session
                            .search_and_mark_redactions_styled(
                                &query,
                                &pdfcer_core::edit::TextSearchOptions::default()
                                    .with_case_insensitive(true),
                                &appearance,
                            )
                            .map(|m| (m.created, Some(m.diagnostics)))
                    };
                    marked.map(|(_, diagnostics)| {
                        unreadable = diagnostics.map_or(0, |d| {
                            d.type3_fonts_without_to_unicode + d.identity_fonts_without_to_unicode
                        });
                        Vec::new()
                    })
                });
                // ★ Reported AFTER the edit, from the same census the panel
                // lists from, so the number on the trace and the number of
                // rows on screen cannot disagree. `created=0` is the
                // interesting value: it is a search that found nothing,
                // which on a scanned page is the named real-world failure
                // — `crate::text::redact::search_hint` is the sentence that
                // warns about it, and this is how a reader of a trace sees
                // it happen.
                // ★ Recorded on the document BEFORE the trace, so a reader of
                // the trace and a reader of the panel see the same number.
                doc.last_redaction_unreadable_fonts = unreadable;
                let after = crate::panels::redact::mark_ids(&doc.session).len();
                crate::diag::trace(|| {
                    format!(
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "redact-marked mode={} created={} total={} unreadable_fonts={}",
                        if pattern { "pattern" } else { "literal" },
                        after.saturating_sub(before),
                        after,
                        unreadable
                    )
                });
            }
        }
        Action::MarkPageForRedaction { page, appearance } => {
            // Resolved here rather than carried on the action because the
            // rectangle is the page's, not the operator's — see the
            // variant's docs. A page index past the end is unreachable from
            // the panel and is answered rather than indexed, because an
            // action is plain data a test can build.
            if let Some(spec) = doc
                .pages
                .get(page)
                .map(|p| crate::panels::redact::whole_page_spec(p, &appearance))
            {
                // ★ Asked before the edit, for the same reason the selection
                // route asks early: `vector_edit` bumps the epoch and the
                // object model is rebuilt underneath. See `super::redactimg`.
                //
                // A whole-page mark covers every image on the page by
                // definition, so this is the one route where the question needs
                // no geometry — only "are there any?".
                let images = crate::app::actions::redactimg::images_on_page(doc, page);
                vector_edit(doc, "redact-mark-page", page, 1, |session| {
                    session.add_redaction(page, &spec).map(|_| {
                        if images > 0 {
                            vec![crate::text::redact::mark_covers_image(images)]
                        } else {
                            Vec::new()
                        }
                    })
                });
            } else {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("redact-mark-page-declined page={page} reason=no-such-page")
                });
            }
        }
        Action::RemoveRedactionMark { annot_id } => {
            let page = doc.view.page_index;
            vector_edit(doc, "redact-unmark", page, 1, |session| {
                session.delete_redaction_mark(annot_id).map(|()| Vec::new())
            });
        }
        // ===============================================================
        // ★★★ THE ONE THAT ARMS A REMOVAL — `OPERATOR_REQUESTS.md` O125,
        // 2026-09-04, rebuilt on `Pass 250.2` 2026-09-05.
        //
        // Raised by `crate::dialogs::redact` on its default destination,
        // after the operator has read a measured report, ticked the
        // permanence box and been told that the page will not change.
        // Nothing is written and nothing is removed: the next save carries
        // the removal out, which is the whole of what he asked for.
        //
        // ★★ `vector_edit`, NOT `vector_edit_on_page`. A redaction is a
        // whole-document change by construction — the removal at save is a
        // full rewrite, every page of it — so the page-scoped funnel would
        // be a claim this verb cannot make. The `page` argument below is the
        // trace's, as it is everywhere.
        // ===============================================================
        Action::PendingRedaction(Staging::Stage) => {
            let page = doc.view.page_index;
            // Set inside the closure and read after it, the same shape the
            // search arm uses for its unreadable-font count and for the same
            // reason: the closure holds the only `&mut EditSession`, and
            // `doc` is borrowed for the whole of the call.
            //
            // ★★★ `absence_claims` is the load-bearing one. It is
            // `RedactionReport::redacted_text` — the exact strings the engine
            // says the save will remove — and it is the standing claim
            // `crate::app::save` greps the bytes for. On the staged path the
            // save proves against the report the REMOVAL returned rather than
            // against this list, because an edit in between changes what comes
            // out; this list is the document's own record, and what it does is
            // stop an ordinary save from ever being clean-by-omission if the
            // arming is somehow lost without being cancelled.
            let mut absence_claims: Vec<String> = Vec::new();
            let mut armed = false;
            vector_edit(doc, "redact-stage", page, 1, |session| {
                // ★ `{:?}` rather than a `Display` impl on the refusal, and
                // deliberately so. `vector_edit` needs `E: Display` to put the
                // cause on the trace, and `RedactApplyRefusal` has no `Display`
                // ON PURPOSE — `check-ui-strings.sh`'s exclusion 3 permits a
                // diagnostic `Display`, but this type is rendered to the
                // operator by `crate::text::redact::refusal_message` and giving
                // it a second, uncatalogued rendering is how the two drift.
                // Debug is unambiguously diagnostic and cannot be mistaken for
                // copy. The operator-facing half of a refusal here is the
                // funnel's own worded decline; the dialog has already refused
                // by name for every cause reachable in practice.
                crate::redact::stage_into_session(session)
                    .map_err(|refusal| format!("{refusal:?}"))
                    .map(|staged| {
                        absence_claims = staged.report.redacted_text.clone();
                        // The residual list the DIALOG showed is built from more
                        // sources than the report alone (retained marks, uncut
                        // vector geometry, kept clips, raw-byte residuals). The
                        // sentence here counts the same way, so the number the
                        // operator acknowledged and the number he is told about
                        // afterwards cannot disagree.
                        //
                        // ★ `None` for the verification, and it is a statement:
                        // the staging verb discards its bytes, so no absence
                        // sweep has run and the raw-byte residuals the dialog
                        // could list are not among these. Passing a default
                        // `AbsenceVerification` would have compiled and told
                        // him a sweep found nothing.
                        let residuals = crate::redact::residual_count(&staged.report, None);
                        let line = crate::text::redact::staged_into_document(
                            staged.report.marks_applied,
                            staged.report.pages_redacted,
                            residuals,
                        );
                        armed = true;
                        vec![line]
                    })
            });
            // ★ Recorded only on success. An empty claim list means
            // `crate::app::save` proves nothing on the ordinary path — which is
            // correct, because on a refusal nothing is armed and there is
            // nothing to prove about it. Assigning unconditionally would have
            // armed the save-time proof with the strings from a removal that
            // was never set up.
            if armed {
                doc.redaction_absence_claims = absence_claims;
            }
        }
        // ===============================================================
        // ★★★ …AND THE ONE THAT DISARMS IT — new 2026-09-05.
        //
        // It exists because a stageable operation that cannot be un-staged
        // is a trap, and this one has teeth: while a removal is armed the
        // engine refuses BOTH ordinary save modes by name, so an operator
        // who changed his mind and had no way to say so could not save his
        // document at all.
        //
        // ★ Through the same funnel, and it is not symmetry: the engine's
        // verb takes `&mut EditSession` and `Arc::get_mut` is the funnel's
        // second step. The epoch bump that comes with it is wanted for the
        // module header's reason — `has_unsaved_edits` reads it — and the
        // re-raster it costs draws an identical picture, because nothing
        // about the page ever changed.
        // ===============================================================
        Action::PendingRedaction(Staging::Cancel) => {
            let page = doc.view.page_index;
            let marks = crate::panels::redact::mark_ids(&doc.session).len();
            vector_edit(doc, "redact-stage-cancel", page, 1, |session| {
                crate::redact::cancel_staged_redaction(session);
                // `Ok` unconditionally, and the engine's verb is why rather
                // than optimism: `cancel_pending_redaction` is a `const fn`
                // that clears a `bool` and is documented idempotent, so a
                // cancel on a document with nothing armed is a no-op rather
                // than an error. There is no failure to model and inventing an
                // error type for one would be a branch nothing can take.
                //
                // The type annotation is needed because nothing in the closure
                // constrains `E`.
                Ok::<_, String>(vec![crate::text::redact::staging_cancelled(marks)])
            });
            // ★★★ The claims go with it, and this line is the one that must
            // not be dropped. They are this shell's statement that *every file
            // it writes for this document has this text removed from it*, and
            // after a cancel that statement is false — the content is
            // deliberately still there. Leaving them set would make the next
            // ordinary save refuse itself, correctly, over a removal the
            // operator called off on purpose, and there would be no way out of
            // it but to close the document.
            doc.redaction_absence_claims.clear();
        }
        _ => {}
    }
}
