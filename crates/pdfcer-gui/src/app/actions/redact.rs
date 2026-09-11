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
//! ⇒ The arm below is [`RedactAction::Pending`], and it goes through the
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

// ===========================================================================
// THE ACTION FAMILY
// ===========================================================================

/// ★★★ **Everything whose subject is a redaction**, as one family.
///
/// Moved out of [`super::Action`] under **R2** on 2026-09-06, when document
/// signing needed a variant that file could not afford — it was at exactly
/// 1,500 of 1,500 lines, which is a state its own header had predicted and
/// named the remedy for: *"the next family of variants to grow is the one that
/// will have to become a sub-enum beside `PageAction` and `DimensionAction`."*
///
/// ## Why redaction, when the written plan nominated markup
///
/// `super`'s declaration of `action` carries a 2026-08-20 measurement naming
/// **markup** as the candidate, *"written down here so the next person does not
/// have to re-measure it under deadline."* It was not taken, and departing from
/// a plan that was written down deserves its reason in the same place:
///
/// | family | lines in `action.rs` | call sites | destination module |
/// |---|---|---|---|
/// | markup | ~370 | **48** | `canvas::markup` + `actions::annots` — two |
/// | redaction | 114 | **19** | `actions::redact` — this one, already holding every body |
///
/// The deciding term is the third column, and it is
/// [`super::pages::PageAction`]'s own third argument verbatim: *"the
/// destination already existed. This module has held the five verbs' bodies
/// since page operations shipped, and `apply` already routed every one of them
/// here. The enum was the only half still living elsewhere."* That is exactly
/// true of this file and is not true of markup, whose bodies are split across
/// two modules.
///
/// ⇒ **Markup remains the next candidate and its measurement stands.** This is
/// a cheaper move taken first, not a revision of that judgement.
///
/// ## ★★★ The property that puts all five in an `Action` at all: they are
/// REVERSIBLE
///
/// Marking authors a `/Redact` annotation and removes nothing — the engine
/// records each as an undoable command, so every one goes through `vector_edit`
/// exactly as a markup does and `Ctrl+Z` takes it back. [`Self::Pending`] arms
/// the removal at the next save (`Pass 250.2`) and can be called off.
///
/// **The verb that actually destroys content is not in this family and is not
/// an `Action`.** Routing the one operation that cannot be undone through a
/// queue that replays would be the defect, not the tidiness.
///
/// ## ★★ The names lost their `Redaction` stutter, and nothing else changed
///
/// `MarkRedactionsBySearch` became `RedactAction::BySearch`, and its four
/// siblings likewise: the family name is now carried by the enum, so repeating
/// it in every variant would spell it twice at every call site. The **fields
/// are untouched**, deliberately — a rename and a change of shape in one commit
/// is a diff nobody can review, and the fields are what the engine sees.
#[derive(Debug, Clone, PartialEq)]
pub enum RedactAction {
    /// **Mark every occurrence of some text for redaction.**
    ///
    /// Raised by [`crate::panels::redact`]'s Find & mark control. Applied
    /// through `vector_edit`, so it is one undoable command however many marks
    /// it creates — which is the right granularity: the operator asked one
    /// question, and taking back "mark every occurrence of this name" one
    /// annotation at a time would be unusable.
    ///
    /// # ★ The query is carried, not a hit list
    ///
    /// The panel could resolve the matches itself and push the quads, the way
    /// [`super::super::actions::Action::CommitTextMarkup`] carries the selection's boxes. It must not,
    /// for a reason specific to this verb: `pdfcer-core`'s own
    /// `mark_redactions_by_search_with` documents the trap — a front end whose
    /// search and whose marking disagree about *which hits exist* produces
    /// "three highlights and eleven redaction marks", and *"on the one
    /// operation whose whole purpose is removing content irreversibly, 'the
    /// mark set is a superset of the highlight set' is not a cosmetic
    /// difference."* Handing the engine the query lets the engine answer both
    /// halves with one scan.
    BySearch {
        /// The text, already trimmed by the panel.
        query: String,
        /// Whether to read the query as a pattern (`#` any digit, `?` any
        /// character) rather than as literal text.
        ///
        /// A `bool` here rather than an enum, unlike
        /// `crate::redact::ResidualAcknowledgement` — because this one is
        /// *named at its field* and reads as a sentence at the one call site
        /// that builds it, while that one is a positional argument at a call
        /// site where a transposition would write a file.
        pattern: bool,
        /// How the marks this creates will look once applied.
        ///
        /// ★ Carried on the action, not read at apply time, and it is the same
        /// rule the pen follows for markup: the operator's choice is the one
        /// they had **when they pressed the control**. Reading it in the
        /// dispatcher would let a frame in which they also changed the fill
        /// swatch author marks they did not choose — and on this verb the
        /// difference is not cosmetic, because the appearance is baked into
        /// each `/Redact` annotation at creation and there is no verb that
        /// modifies one afterwards.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// **Mark the whole of one page for redaction.**
    ///
    /// Raised by [`crate::panels::redact`]'s Mark whole page control. The page
    /// is carried rather than read from `doc.view` at apply time, on
    /// [`super::super::actions::Action::CommitTextMarkup`]'s rule: the operator marked the sheet they
    /// were looking at, and an action applied after a frame in which they also
    /// paged away must mark the sheet they meant.
    ///
    /// The rectangle is not carried, because it is not the operator's choice —
    /// it is the page's crop box, and `crate::panels::redact::whole_page_spec`
    /// is the one place that decision is made and tested.
    WholePage {
        /// The 0-based page to cover.
        page: usize,
        /// How the mark will look once applied. See
        /// [`Self::BySearch`]'s field of the same name.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// ★★★ **Mark what is SELECTED on the page for redaction** — the third
    /// marking route, and the first that does not go through text.
    ///
    /// **Ken, 2026-08-30:** *"am I able to select objects on the canvas and
    /// redact them that way yet? … it just told me it couldn't."* It could not:
    /// [`Self::BySearch`] reaches text pdfcer can read as text and
    /// [`Self::WholePage`] reaches everything, and on a CAD drawing
    /// most of what wants redacting is in between.
    ///
    /// `super::redactsel`'s header carries the argument in full, including why
    /// neither a page nor a rectangle is carried here.
    Selection {
        /// How the mark will look once applied. See
        /// [`Self::BySearch`]'s field of the same name.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// ★★★ **Mark everything drawn outside the page boundary, on every sheet
    /// that has any** — the fourth marking route, added 2026-09-11.
    ///
    /// Raised by [`crate::dialogs::offpage`], and it is the only one of the
    /// four raised by a **window**. That is not an accident of where the button
    /// ended up: the other three mark something the operator is already looking
    /// at (a search hit, the current sheet, a selection), and this one marks
    /// content that **is not on screen and cannot be** — it does not render, it
    /// does not print, and the whole reason the window exists is that nothing in
    /// an ordinary reading of the document discloses it.
    ///
    /// # ★★★ Why the BANDS travel and not the objects
    ///
    /// The census this comes from lists objects, and it would be natural to
    /// carry their boxes. It would also be wrong twice over:
    ///
    /// - **A box per object is a mark per object.** A CAD sheet with a
    ///   superseded revision block off its left edge decomposes into hundreds
    ///   of stroked paths, and marking each would author hundreds of `/Redact`
    ///   annotations for one operator gesture. `pdfcer_core::offpage::offpage_bands`
    ///   answers with at most four **non-overlapping** rectangles per page that
    ///   cover the same area, which is what §12.5.6.23's `/QuadPoints` list is
    ///   for.
    /// - **An object's box is not the area to remove.** Removing "that path"
    ///   leaves whatever else happens to sit beside it, and the operator's
    ///   request is *"take off what is outside the sheet"* — an area, not an
    ///   inventory.
    ///
    /// ⇒ So the window runs the census, asks the engine for the bands, and
    /// sends the bands. The objects stay in the window, where they are the
    /// **disclosure** — which is exactly rule 4's split: what is removed is a
    /// geometry, what is reported is the words that were found in it.
    ///
    /// # ★★ Why the page indices travel with them
    ///
    /// One press covers many sheets, so there is no "current page" to resolve
    /// against — and resolving against one would silently mark one sheet of a
    /// thirty-six-sheet set. The pairing is carried whole for
    /// [`Self::WholePage`]'s reason taken further: the operator marked the
    /// sheets the **census** named, and a frame in which they also paged away
    /// must not change which.
    OffPage {
        /// One entry per sheet with content outside its own boundary: the
        /// 0-based page index, and the bands the engine computed for it.
        ///
        /// A sheet with an empty band list is not expected here — the window
        /// filters clean pages out — but an empty list is skipped rather than
        /// treated as an error, because an action is plain data a test can
        /// build and four zero-area marks would be four annotations that do
        /// nothing.
        bands: Vec<(usize, Vec<pdfcer_core::page_tree::Rect>)>,
        /// How many sheets the census could not read at all.
        ///
        /// ★★ Carried so the status line can say so, and it is the sentence
        /// that keeps this window from issuing a clean bill it has not earned:
        /// a page whose content streams will not decode was **not checked**,
        /// and "marked everything outside the sheet" said over such a document
        /// is a claim about pages nobody looked at. `pdfcer_core::offpage`
        /// returns the two separately for precisely this reason.
        unreadable: usize,
        /// How the marks will look once applied. See [`Self::BySearch`]'s field
        /// of the same name — and note that this route reads the **panel's**
        /// chosen appearance like the other three, so a window on the Edit tab
        /// cannot produce a differently-coloured mark from the panel beside it.
        appearance: pdfcer_core::annot_author::RedactAppearance,
    },
    /// **Take one redaction mark off.**
    ///
    /// Raised by a row's Remove control. The engine's
    /// `EditSession::delete_redaction_mark` rather than its general annotation
    /// delete, deliberately and on core's own instruction: the two record
    /// different `CommandKind`s so that an undo tooltip can say *"remove a
    /// redaction mark"* rather than *"delete annotation"*, and — as that
    /// method's docs put it — *"I decided not to redact that"* is a different
    /// claim from *"delete annotation"*.
    ///
    /// The **annotation id**, not a row index: a list position is a position in
    /// a census rebuilt every frame, and by the time the apply phase runs the
    /// same index may name a different mark. `crate::app` §10's rule —
    /// *selection is an identity, not a position* — applied to a list.
    RemoveMark {
        /// The `/Redact` annotation to delete.
        annot_id: pdfcer_core::object::ObjId,
    },
    /// ★★★ **Arm the removal of every redaction mark at the next save, or
    /// disarm it** — O125, corrected 2026-09-05 for `Pass 250.2`. **The whole
    /// argument — undo-preserving, why Cancel had to exist — is on the apply
    /// arm** in `app::actions::redact`, on this file's own R2 rule.
    Pending(crate::redact::Staging),
    /// **Apply the removal into the open document, now** — the operator's
    /// 2026-09-08 report.
    ///
    /// # ★★★ Why this carries COUNTS and not the document
    ///
    /// `PreparedRedaction::bytes` is private *"deliberately and
    /// load-bearingly"*, because a public accessor would restore the surface
    /// `pdfcer`'s own `redact-apply` used to write an **unverified** file. So
    /// the whole value travels, and the only thing this arm may do with it is
    /// call [`crate::redact::PreparedRedaction::into_verified_document`], which
    /// re-proves the removal before handing back anything.
    ///
    /// ★★ The acknowledgement travels with it for the same reason it is an
    /// argument to `write_to` rather than a field: consent belongs to the
    /// press, not to the preparation. A `PreparedRedaction` sitting in a queue
    /// carries no permission of its own.
    ///
    /// ⚠ Boxed. `PreparedRedaction` holds a whole redacted document, and an
    /// unboxed variant would make every `Action` in the program that size.
    ApplyNow {
        /// How many marked regions the removal covered, for the trace and the
        /// operator's receipt.
        marks: usize,
        /// How many pages it touched.
        pages: usize,
    },
}

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
pub fn apply(doc: &mut OpenDoc, action: RedactAction, settings: &pdfcer_core::settings::Settings) {
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
        // ===============================================================
        // ★★★ THE FOURTH MARKING ROUTE — content off the sheet, 2026-09-11.
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
                    // ★ One annotation per PAGE carrying every band, not one
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
                    // ★★★ NOTHING landed, so this is a decline and must travel
                    // as one. Returning `Ok` with a sad sentence would bump the
                    // epoch, invalidate every page cache and write a disclosure
                    // about an edit that did not happen — and the operator
                    // would be told to go and review marks that are not there.
                    return Err(first_refusal.unwrap_or_else(|| {
                        // ui-text-exempt: diagnostic trace, never displayed in the UI
                        "no page accepted a mark".to_owned()
                    }));
                }
                // ★★★ From here the edit SUCCEEDED, partially or wholly, and
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
        RedactAction::Pending(Staging::Stage) => {
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
        // ===============================================================
        // ★★★ APPLY NOW — the removal happens and the page changes.
        //
        // The operator, 2026-09-08: *"the redaction feature regressed back to
        // just giving me the 'don't apply yet' button."*
        //
        // Nothing had regressed. `Destination::OpenDocument` became the default
        // on 2026-09-04 because he asked for it, and `Pass 250.2` made it cost
        // nothing on 2026-09-05 — at the price stated in its own doc: **the
        // page does not change**. He pressed the only button the default
        // offered and watched nothing happen.
        //
        // ★★★ THIS ARM REPLACES THE WHOLE `OpenDoc`, which nothing else here
        // does, and that is deliberate rather than convenient. `OpenDoc::new`'s
        // own doc argues against a `reset()`: *"opening a document constructs a
        // whole new `OpenDoc`, so a cached texture or a page index can never
        // refer to a page from a previous file."* A redaction that removes
        // content is exactly that case — every cached raster, extraction and
        // selection describes bytes that no longer exist.
        //
        // ⚠ **The undo log goes with it**, and that is the price his own ruling
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
                // ★ Refused rather than unwrapped. A redacted document whose
                // page tree will not read is an engine defect worth a request,
                // and the operator's own document is still open and intact —
                // which is the outcome to protect.
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed.
                    "redact-apply-now-applied installed=0 reason=no-page-tree".to_owned()
                });
                return;
            };
            // ★★ The page he was looking at is carried over by hand, and it is
            // the only thing that is. A redaction removes content, never pages,
            // so the index still names the same sheet — and being thrown back
            // to page 1 of a hundred-page set after every redaction would be
            // its own defect.
            let was_on = doc.view.page_index.min(page_tree.len().saturating_sub(1));
            let path = doc.path.clone();
            // ★★★ `settings.open_session`, NEVER `EditSession::new`. The
            // funnel's own doc calls its absence *"a live defect for the whole
            // life of this shell"*: a session built raw discards the extraction
            // and write options the operator chose — and on THIS path that is
            // sharper than usual, because `unmappable_code` changes character
            // offsets and therefore changes which runs a later
            // redaction-by-text matches (`pdfcer-core` R35).
            //
            // ⇒ Caught by `no_call_site_builds_its_own_options` on the day this
            // arm was written, which is the whole reason that test scans call
            // sites rather than trusting a convention.
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
