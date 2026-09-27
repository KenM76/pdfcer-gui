//! # `app::actions::redact` — the three arms that MARK content for removal,
//! and the one that removes it
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/redact.md`.

// ===========================================================================
// THE ACTION FAMILY
// ===========================================================================

pub use pdfcer_gui_base::editactions::RedactAction;

use super::apply::vector_edit;
use crate::app::state::OpenDoc;
use crate::redact::Staging;

/// Apply one of the three marking actions, or the staging one.
pub fn apply(
    doc: &mut OpenDoc,
    action: RedactAction,
    settings: &pdfcer_core::settings::Settings,
    reach: crate::app::prefs::RedactionReach,
) {
    match action {
        // ===============================================================
        // THE REDACTION MARKING VERBS
        //
        // Three arms, each one call, through the same `vector_edit` funnel
        // every other document change uses — which is the whole reason they
        // are one line each. Marking is an ordinary edit: it authors an
        // annotation, the engine records it as an undoable command, and the
        // page has to re-raster because a `/Redact` mark draws a red
        // outline the operator needs to see.
        //
        // **Nothing in THESE THREE removes anything** — the scope of that
        // claim is these arms, not the file. See the module header for why it
        // is a statement about the verbs that exist rather than a principle.
        //
        // `.map(|_| Vec::new())` on the first two adapts the engine's
        // `Vec<ObjId>`/`ObjId` to the disclosure list `vector_edit` traces,
        // and the empty vec is a statement rather than a placeholder —
        // authoring an annotation rewrites no existing operator, so nothing
        // changed form and rule 4 owes the operator nothing. It is the same
        // adaptation `CommitMarkup` makes one screen up.
        // ===============================================================
        RedactAction::BySearch {
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
                    // Case-INSENSITIVE, always, and it is not a missing
                    // control. Over-marking is the safe direction of error
                    // on this verb and under-marking is not: a mark the
                    // operator did not want is one row and one click in the
                    // review list, and a mark they did want and did not get
                    // is a name shipped in a document they believe is
                    // redacted.
                    //
                    // The `_styled` verb, because the unstyled one builds its
                    // spec internally with `fill: None`: a fill the operator
                    // chose would be discarded on this path and honoured on
                    // the whole-page one, and a control honoured on some marks
                    // and silently dropped on others is worse than no control
                    // on the one operation that cannot be undone.
                    //
                    // `search_and_mark_redactions_styled`, NOT
                    // `mark_redactions_by_search_styled`.
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
                    // Both populations RENDER PERFECTLY, which is what makes
                    // it invisible. Nothing on the page looks unredacted.
                    //
                    // **Both routes disclose**, and the pattern one is the
                    // easier to get wrong: `mark_redactions_by_pattern_styled`
                    // returns the marks and nothing else, so a shell calling it
                    // writes `(created, None)` and discards a census the engine
                    // already computed on that exact scan.
                    //
                    // `None` there does NOT mean *unknown*. It collapses into
                    // `unreadable = 0` two statements below, which is the
                    // shell asserting *"no font in this document hides text
                    // from a search"* — on the route least able to know it.
                    // Worse: a pattern pass run after a literal one would
                    // **overwrite a true warning already on screen**, so the
                    // operator watches an honest disclosure disappear.
                    //
                    // And the pattern route is the MORE exposed of the two,
                    // not the less. What people reach for wildcards for is
                    // structured confidential material — account numbers,
                    // dates of birth, case references — which is exactly the
                    // content a redaction is run for.
                    //
                    // `search_and_mark_redactions_by_pattern_styled` is the
                    // same scan with the diagnostics attached, and it is the
                    // single implementation the other three pattern entry
                    // points delegate to, so the four cannot disagree about
                    // what was marked.
                    let marked = if pattern {
                        session
                            .search_and_mark_redactions_by_pattern_styled(&query, true, &appearance)
                            .map(|m| (m.created, Some(m.diagnostics)))
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
                // Reported AFTER the edit, from the same census the panel
                // lists from, so the number on the trace and the number of
                // rows on screen cannot disagree. `created=0` is the
                // interesting value: it is a search that found nothing,
                // which on a scanned page is the named real-world failure
                // — `crate::text::redact::search_hint` is the sentence that
                // warns about it, and this is how a reader of a trace sees
                // it happen.
                // Recorded on the document BEFORE the trace, so a reader of
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
        RedactAction::WholePage { page, appearance } => {
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
                // Asked before the edit, for the same reason the selection
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
        // ===============================================================
        // THE FOURTH MARKING ROUTE — content off the sheet.
        //
        // The only arm here that authors marks on MORE THAN ONE PAGE, and
        // the only one whose geometry the operator never saw on screen.
        // Everything structural about it is on the variant; what is below is
        // the loop and the two things the loop has to be honest about.
        // ===============================================================
        RedactAction::OffPage {
            bands,
            unreadable,
            appearance,
        } => {
            // `page` for the trace is the first sheet touched, which is this
            // funnel's convention everywhere; the count is what makes the
            // line useful — and here it counts BANDS, because that is what
            // will appear in the operator's review list.
            let first = bands.first().map_or(0, |(page, _)| *page);
            let total: usize = bands.iter().map(|(_, rects)| rects.len()).sum();
            if total == 0 {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("redact-mark-offpage-declined reason=no-bands unreadable={unreadable}")
                });
                return;
            }
            vector_edit(doc, "redact-mark-offpage", first, total, |session| {
                let mut marked_pages = 0usize;
                let mut marked_bands = 0usize;
                let mut refused_pages = 0usize;
                // The engine's own words about the first refusal, kept for the
                // trace only. `vector_edit`'s Err arm documents why an
                // `EditError`'s `Display` may not become UI text; this is the
                // same rule applied to a PARTIAL failure, which that arm cannot
                // see because a partial failure returns `Ok`.
                let mut first_refusal: Option<String> = None;
                for (page, rects) in &bands {
                    if rects.is_empty() {
                        continue;
                    }
                    // One annotation per PAGE carrying every band, not one
                    // per band — §12.5.6.23 makes `/QuadPoints` a list
                    // precisely so a single mark can cover a disjoint region,
                    // and `actions::redactsel` made the same call for the same
                    // clause. Four annotations per sheet would be four rows in
                    // the review list for one gesture on one page.
                    let quads: Vec<pdfcer_core::annot_author::Quad> = rects
                        .iter()
                        .copied()
                        .map(pdfcer_core::annot_author::Quad::from_rect)
                        .collect();
                    let spec = appearance.to_spec(quads);
                    match session.add_redaction(*page, &spec) {
                        Ok(_) => {
                            marked_pages += 1;
                            marked_bands += rects.len();
                        }
                        Err(error) => {
                            refused_pages += 1;
                            let _ = first_refusal.get_or_insert_with(|| error.to_string());
                        }
                    }
                }
                if marked_pages == 0 {
                    // NOTHING landed, so this is a decline and must travel
                    // as one. Returning `Ok` with a sad sentence would bump the
                    // epoch, invalidate every page cache and write a disclosure
                    // about an edit that did not happen — and the operator
                    // would be told to go and review marks that are not there.
                    return Err(first_refusal.unwrap_or_else(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "no page accepted a mark".to_owned()
                    }));
                }
                // From here the edit SUCCEEDED, partially or wholly, and
                // every remaining sentence is rule 4's surviving half: the
                // marks are visible, so nothing below is about what the
                // operator can see. It is about what they cannot —
                //
                //   • that Undo will take these back one sheet at a time,
                //     because the engine records one command per page and
                //     there is no range verb (see `text::offpage`);
                //   • that some sheets were never checked;
                //   • that some sheets were checked, found dirty, and refused.
                //
                // All three are invisible on a canvas that now shows red
                // outlines exactly where they were asked for.
                let mut notes = vec![crate::text::offpage::marked_disclosure(
                    marked_bands,
                    marked_pages,
                )];
                if marked_pages > 1 {
                    notes.push(crate::text::offpage::marked_undo_note(marked_pages));
                }
                if unreadable > 0 {
                    notes.push(crate::text::offpage::marked_skipped(unreadable));
                }
                if refused_pages > 0 {
                    notes.push(crate::text::offpage::marked_refused(refused_pages));
                    crate::diag::trace(|| {
                        format!(
                            // ui-text-exempt: diagnostic trace, never displayed in the UI
                            "redact-mark-offpage-partial refused={refused_pages} detail={}",
                            first_refusal.as_deref().unwrap_or("none")
                        )
                    });
                }
                Ok::<_, String>(notes)
            });
        }
        RedactAction::RemoveMark { annot_id } => {
            let page = doc.view.page_index;
            vector_edit(doc, "redact-unmark", page, 1, |session| {
                session.delete_redaction_mark(annot_id).map(|()| Vec::new())
            });
        }
        // ===============================================================
        // THE ONE THAT ARMS A REMOVAL — `OPERATOR_REQUESTS.md` O125.
        //
        // Raised by `crate::dialogs::redact` on its default destination,
        // after the operator has read a measured report, ticked the
        // permanence box and been told that the page will not change.
        // Nothing is written and nothing is removed: the next save carries
        // the removal out, which is the whole of what he asked for.
        //
        // `vector_edit`, NOT `vector_edit_on_page`. A redaction is a
        // whole-document change by construction — the removal at save is a
        // full rewrite, every page of it — so the page-scoped funnel would
        // be a claim this verb cannot make. The `page` argument below is the
        // trace's, as it is everywhere.
        // ===============================================================
        RedactAction::Pending(Staging::Stage) => {
            let page = doc.view.page_index;
            // Set inside the closure and read after it, the same shape the
            // search arm uses for its unreadable-font count and for the same
            // reason: the closure holds the only `&mut EditSession`, and
            // `doc` is borrowed for the whole of the call.
            //
            // `absence_claims` is the load-bearing one. It is
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
                // `{:?}` rather than a `Display` impl on the refusal, and
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
                crate::redact::stage_into_session(session, reach)
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
                        // `None` for the verification, and it is a statement:
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
            // Recorded only on success. An empty claim list means
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
        // …AND THE ONE THAT DISARMS IT.
        //
        // It exists because a stageable operation that cannot be un-staged
        // is a trap, and this one has teeth: while a removal is armed the
        // engine refuses BOTH ordinary save modes by name, so an operator who
        // changed his mind with no way to say so could not save at all.
        //
        // Through the same funnel, and it is not symmetry: the engine's
        // verb takes `&mut EditSession` and `Arc::get_mut` is the funnel's
        // second step. The epoch bump that comes with it is wanted for the
        // module header's reason — `has_unsaved_edits` reads it — and the
        // re-raster it costs draws an identical picture, because nothing
        // about the page ever changed.
        // ===============================================================
        // ===============================================================
        // APPLY NOW — the removal happens and the page changes.
        //
        // The operator, on a build where arming was the only destination:
        // *"the redaction feature regressed back to just giving me the 'don't
        // apply yet' button."* Arming costs nothing and changes nothing on the
        // page, which is its whole design — and an operator who presses it and
        // watches nothing happen reads that as a broken feature. This arm is
        // the destination that does change the page.
        //
        // THIS ARM REPLACES THE WHOLE `OpenDoc`, which nothing else here
        // does, and that is deliberate rather than convenient. `OpenDoc::new`'s
        // own doc argues against a `reset()`: *"opening a document constructs a
        // whole new `OpenDoc`, so a cached texture or a page index can never
        // refer to a page from a previous file."* A redaction that removes
        // content is exactly that case — every cached raster, extraction and
        // selection describes bytes that no longer exist.
        //
        // **The undo log goes with it**, and that is the price his own ruling
        // accepted: *"finalizing the document and can't be undone is ok for
        // now."* It is stated at the control, in
        // `destination_open_document_now_tooltip`, rather than discovered.
        // ===============================================================
        RedactAction::ApplyNow { marks, pages } => {
            let Some(document) = crate::redact::take_applied_document() else {
                // The dialog parks the document before pushing this action, so
                // an empty slot means the action ran twice. Doing nothing is
                // the correct second outcome — see `take_applied_document`.
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "redact-apply-now-applied installed=0 reason=nothing-parked".to_owned()
                });
                return;
            };
            let Ok(page_tree) = pdfcer_core::page_tree::pages(&document) else {
                // Refused rather than unwrapped. A redacted document whose
                // page tree will not read is an engine defect worth a request,
                // and the operator's own document is still open and intact —
                // which is the outcome to protect.
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "redact-apply-now-applied installed=0 reason=no-page-tree".to_owned()
                });
                return;
            };
            // The page he was looking at is carried over by hand, and it is
            // the only thing that is. A redaction removes content, never pages,
            // so the index still names the same sheet — and being thrown back
            // to page 1 of a hundred-page set after every redaction would be
            // its own defect.
            let was_on = doc.view.page_index.min(page_tree.len().saturating_sub(1));
            let path = doc.path.clone();
            // `settings.open_session`, NEVER `EditSession::new`. The
            // funnel's own doc calls its absence *"a live defect for the whole
            // life of this shell"*: a session built raw discards the extraction
            // and write options the operator chose — and on THIS path that is
            // sharper than usual, because `unmappable_code` changes character
            // offsets and therefore changes which runs a later
            // redaction-by-text matches (`pdfcer-core` R35).
            //
            // ⇒ `no_call_site_builds_its_own_options` catches a call site
            // that builds its own, which is why that test scans call sites
            // rather than trusting a convention.
            use crate::app::settings::SettingsExt as _;
            *doc = OpenDoc::new(path, settings.open_session(document), page_tree);
            doc.view.page_index = was_on;
            crate::diag::trace(move || {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "redact-apply-now-applied installed=1 marks={marks} pages={pages} \
                     page={was_on}"
                )
            });
        }
        RedactAction::Pending(Staging::Cancel) => {
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
            // The claims go with it, and this line is the one that must
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
