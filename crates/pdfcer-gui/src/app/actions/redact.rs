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
//! ## ★★★ CORRECTED 2026-09-04 (evening) — the removal IS here now, and the
//! old section said it never could be
//!
//! What stood here, verbatim, until this afternoon:
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
//! expired the same day: `EditSession::apply_redactions` (`Pass 250.1`) applies
//! a redaction into the open session, so the operation now *does* change a
//! document through the queue and *does* have an epoch to bump. It is
//! [`Action::ApplyRedactionsIntoDocument`], the fourth arm below, and it goes
//! through the identical `vector_edit` funnel as the three marking arms.
//!
//! ★ The last sentence — *"routing the one operation that cannot be undone
//! through a queue that replays would be the defect"* — deserves an answer
//! rather than a deletion, because it names a real hazard. **This queue does
//! not replay.** `crate::app::actions` drains it once per frame in order and
//! discards it; the thing that replays is the engine's *undo log*, and the
//! apply is not in it — the engine clears the log rather than recording a
//! command, which is precisely why it cannot be replayed backwards into an
//! un-redaction. The hazard the sentence feared is structurally absent, and the
//! epoch bump, the texture invalidation and the page resync that the funnel
//! performs are all things this verb genuinely needs.
//!
//! ## What is still NOT here
//!
//! **The write.** This arm puts the removal in the session and stops. Where the
//! bytes go, and when, is `file.save` / `file.save_as` / `file.save_copy`'s
//! decision, exactly as it is for every other edit — which is what the operator
//! asked for in `OPERATOR_REQUESTS.md` O125. The two *write-now* destinations
//! still live in `crate::dialogs::redact` and still reach no arm in this file.

use super::Action;
use super::apply::vector_edit;
use crate::app::state::OpenDoc;

/// Apply one of the three marking actions, or the apply-into-document one.
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
        // ★★★ THE ONE THAT REMOVES — `OPERATOR_REQUESTS.md` O125,
        // 2026-09-04.
        //
        // Raised by `crate::dialogs::redact` on its default destination,
        // after the operator has read a measured report, ticked the
        // permanence box and been told how many undo steps this discards.
        // Nothing is written: the redaction lands in the session and Save
        // decides where it goes, which is the whole of what he asked for.
        //
        // ★★ `vector_edit`, NOT `vector_edit_on_page`. A redaction is a
        // whole-document change by construction — the engine collapses the
        // session onto a completely new base, every page of it — so the
        // page-scoped funnel would be a claim this verb cannot make. The
        // `page` argument below is the trace's, as it is everywhere.
        // ===============================================================
        Action::ApplyRedactionsIntoDocument => {
            let page = doc.view.page_index;
            // Set inside the closure and read after it, the same shape the
            // search arm uses for its unreadable-font count and for the same
            // reason: the closure holds the only `&mut EditSession`, and
            // `doc` is borrowed for the whole of the call.
            //
            // ★★★ `absence_claims` is the load-bearing one. It is
            // `RedactionReport::redacted_text` — the exact strings the engine
            // says it removed — and it is what `crate::app::save` greps the
            // saved bytes for on every subsequent save of this document. Our
            // engine request promised to keep that proof and to move it to
            // save time; this is where the promise is kept, and dropping this
            // assignment would silently retire the shell's only independent
            // check that a redacted document reaches disk redacted.
            let mut absence_claims: Vec<String> = Vec::new();
            let mut disclosed: Option<String> = None;
            vector_edit(doc, "redact-apply-into-document", page, 1, |session| {
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
                crate::redact::apply_into_session(session)
                    .map_err(|refusal| format!("{refusal:?}"))
                    .map(|applied| {
                        absence_claims = applied.report.redacted_text.clone();
                        // The residual list the DIALOG showed is built from more
                        // sources than the report alone (retained marks, uncut
                        // vector geometry, kept clips, raw-byte residuals). The
                        // sentence here counts the same way, so the number the
                        // operator acknowledged and the number he is told about
                        // afterwards cannot disagree.
                        let residuals =
                            crate::redact::residual_count(&applied.report, &applied.verification);
                        let line = crate::text::redact::applied_into_document(
                            applied.report.marks_applied,
                            applied.report.pages_redacted,
                            residuals,
                            applied.undo_steps_cleared,
                        );
                        disclosed = Some(line.clone());
                        vec![line]
                    })
            });
            // ★ Recorded only on success. `disclosed` is `None` on every
            // refusal path, and an empty claim list means `crate::app::save`
            // proves nothing — which is correct, because on a refusal the
            // session was never redacted and there is nothing to prove about
            // it. Assigning unconditionally would have armed the save-time
            // proof with the strings from a removal that did not happen.
            if disclosed.is_some() {
                doc.redaction_absence_claims = absence_claims;
            }
        }
        _ => {}
    }
}
