//! # `panels::docprops::stamps` — **the read half of O169**
//!
//! The operator asked for Acrobat's custom stamps *"with the same
//! import/export"*. The finding that shaped the whole feature is that there is
//! no import and no export to match: **a stamp collection is an ordinary PDF**
//! — one file per category, one page per stamp, the category in `/Info`
//! `/Title`, the names in the catalog's `/Names` → `/Pages` name tree. Handing
//! somebody the file *is* the export; `file.open` *is* the import.
//!
//! ⇒ Which means an operator can open one and pdfcer will show him a document
//! of unrelated pictures with nothing to say that it is anything else. That is
//! what this section exists to end. [`crate::dialogs::stamp_collection`]
//! authors a collection; this one **discloses** one.
//!
//! ## ★★ Why this is disclosure and not decoration
//!
//! **R8b rule 4 — "fuzzy, never sneaky".** Everything pdfcer infers about a
//! file it did not write is reported **off-canvas**, and this is as off-canvas
//! as it gets: a section in a properties panel, on the far side of the window
//! from the page. Nothing here draws on the document, tints a page, badges a
//! thumbnail, or marks a collection's pages as special in the page list. A
//! screenshot of the canvas with this panel closed is identical to a screenshot
//! of the same file opened by a build with no stamp support at all.
//!
//! ★ The half of rule 4 that is easy to drop is the half that binds here: *the
//! inferences the operator cannot see still owe a report.* Every fact in this
//! section is invisible in the page view — the category is a metadata string,
//! the names are name-tree keys, and *dynamic* is a `#` on an identifier no
//! rendering will ever show. Precisely because none of it is visible, all of it
//! is owed.
//!
//! ## ★ Why nothing here is editable
//!
//! The rows are labels, not fields. Renaming a stamp in place would need
//! `EditSession` verbs against the name tree that do not exist — the engine
//! writes a whole tree (`stamp_file::name_stamp_pages`) and does not amend one
//! — and **R9** says an unavailable capability renders **nothing**, not a
//! greyed stub. The route that does exist is the real one: *Save as stamp
//! collection…* reopens this file with its own names already filled into the
//! dialog, and writes a new collection.
//!
//! ⇒ So there is deliberately no *Edit names* button here, and no sentence
//! explaining its absence either. A sentence explaining an absent control is
//! the nagging rule 4 exists to prevent.
//!
//! ## What is read, and what it costs per frame
//!
//! [`pdfcer_core::stamp_file::read`] is called on **every frame this panel
//! draws**, with no cache, and that is a decision rather than an oversight.
//!
//! - **Staleness first, because cost is the wrong question to answer with.**
//!   A cache here would have to be invalidated on the edit epoch, on a re-read
//!   under the other duplicate-key policy, and on document replacement — three
//!   invalidations for a value that must never be wrong, in a panel whose whole
//!   job is to state facts about the open file. This project has already
//!   shipped one cache whose comment argued *cost* while the question in front
//!   of it was *staleness*.
//! - **And then cost, measured against the engine's actual code path.** `read`
//!   looks up `/Info` `/Title`, then the catalog's `/Names`, then its `/Pages`.
//!   On an ordinary PDF the second lookup fails and it returns — two dictionary
//!   probes and an empty `Vec`. It walks the page tree **only** when a `/Pages`
//!   name tree exists, which is to say only on a document that really is a
//!   collection, and Adobe's largest ships twelve stamps in one flat node.
//!
//! ⚠ The read is against `session.document()` — the **base revision**, exactly
//! as [`super::facts`] reads. That is right rather than merely convenient: no
//! verb in this shell edits a name tree, so the base revision's tree *is* the
//! current one, and there is no session state that could disagree with it.

use egui::Ui;

use crate::app::state::OpenDoc;
use crate::text::stamps as t;

/// **The section's own region**, published only when the open document really
/// is a stamp collection.
///
/// That conditional publication is the contract, and it is the half a driven
/// check can only test with a **second launch**: a region declared on
/// `fixtures/stamps-standard-business.pdf` *and* declared on an ordinary
/// drawing would be a heading that is always there — a different defect wearing
/// the same green tick. [`super::REGION_ANOMALIES`] carries the same shape and
/// the same warning.
///
/// ★ Named under the `properties.` prefix like its neighbours, so that the
/// `declared_names(&trace, "properties")` dump several checks print when they
/// cannot find a region lists it. A region under a prefix nobody enumerates is
/// discoverable only by whoever wrote it.
pub const REGION: &str = "properties.stamp-collection"; // ui-text-exempt: trace region name, never displayed

/// The prefix of the per-stamp row regions; the stamp's **index in the name
/// tree** is appended.
///
/// ⚠ Indexed by tree position, **not** by page number, and the difference is
/// the whole subject of [`crate::stamps`]' re-opening logic: the name tree is
/// sorted lexicographically (§7.9.6) and the pages are not, so tree entry 1 is
/// routinely not page 2. A check that wants a particular stamp must read the
/// row it finds, never compute an index from a page.
pub const REGION_ROW_PREFIX: &str = "properties.stamp-collection."; // ui-text-exempt: trace region name, never displayed

/// Draw the stamp-collection section, or draw nothing at all.
///
/// # ★ The test is the name tree, never the title
///
/// [`crate::stamps::is_collection`] asks whether the document has **named
/// pages**. A PDF with a `/Title` and no name tree is just a PDF with a title,
/// and every drawing the operator opens has one of those. Testing the title
/// would put this section on most of his files, which is the failure mode a
/// conditional section has: shown too often, it stops being information and
/// becomes furniture.
pub(super) fn section(ui: &mut Ui, doc: &OpenDoc) {
    let collection = pdfcer_core::stamp_file::read(doc.session.document());
    if !crate::stamps::is_collection(&collection) {
        return;
    }

    // ★★ Scoped so the section's rect is a value egui computed rather than a
    // difference between two cursor readings — `load_anomalies_note`'s finding,
    // copied rather than re-derived: a before/after `ui.cursor()` pair is right
    // today and wrong the first time somebody wraps this in a horizontal
    // layout.
    let block = ui
        .scope(|ui| {
            ui.add_space(6.0);
            // No `.strong()` — R84 / DEFECTS.md D11: egui resolves it to the
            // accent-filled widget foreground, which on an ordinary panel is
            // pale text on pale ground.
            ui.label(t::properties_heading());

            // The category is `/Info` `/Title`, which this same panel offers
            // further down as an EDITABLE field. That is not a duplicate: the
            // field says *"this document's title"* and this row says *"the
            // heading Acrobat will file this set under"*, and an operator who
            // does not know those are one string cannot be expected to guess
            // that renaming the title renames the menu. Saying it twice, under
            // each of its two meanings, is the disclosure.
            // ⚠ A `match` and not `unwrap_or_else`, and the compiler is the
            // one that insisted: `Option<&'a str>::unwrap_or_else` unifies
            // its default's lifetime with the option's, so handing it a
            // function returning `&'static str` demands that `collection`
            // itself live for `'static`. The arms of a `match` coerce the
            // other way round, which is the direction that is true here — the
            // literal outlives the borrow, not the reverse.
            let category = match collection.category.as_deref() {
                Some(named) => named,
                None => t::properties_no_category(),
            };
            super::fact(ui, t::properties_category_label(), category);

            let dynamic = crate::stamps::dynamic_count(&collection);
            super::fact(
                ui,
                t::properties_count_label(),
                &t::properties_count(collection.stamps.len(), dynamic),
            );
            if dynamic > 0 {
                ui.label(
                    egui::RichText::new(t::properties_dynamic_note())
                        .small()
                        .weak(),
                );
            }

            ui.add_space(2.0);
            for (index, stamp) in collection.stamps.iter().enumerate() {
                let row = ui.horizontal_wrapped(|ui| {
                    ui.label(t::properties_stamp(&stamp.display, &stamp.internal));
                    // ⚠ `page_index` is `None` in TWO different situations and
                    // pdfcer cannot currently tell them apart — see
                    // [`crate::stamps::unresolved_count`] for the engine defect
                    // and the request filed against it. The wording therefore
                    // says what pdfcer OBSERVED rather than what it concluded,
                    // which is the only reason the row can be drawn at all:
                    // *"names no page in this document"* is true both when the
                    // collection is broken and when the page tree failed to
                    // read, and *"this stamp is broken"* is true in one of them.
                    ui.label(
                        egui::RichText::new(stamp.page_index.map_or_else(
                            || t::properties_stamp_no_page().to_owned(),
                            |i| t::properties_stamp_page(i + 1),
                        ))
                        .small()
                        .weak(),
                    );
                });
                // ★ Per-row region. The section region below says *"this file is
                // a stamp collection"*; only these say *"and here is what is in
                // it"* — a regression that drew the heading and the count with
                // no rows under them would be invisible to the section region
                // alone. That is exactly the shape `REGION_ANOMALY_ROW_PREFIX`
                // was added to catch, one panel over.
                crate::diag::ui_rect_visible(
                    // ui-text-exempt: trace region name, never displayed
                    &format!("{REGION_ROW_PREFIX}{index}"),
                    row.response.rect,
                    ui.clip_rect(),
                );
            }
        })
        .response
        .rect;

    // ★ `ui_rect_visible` rather than `ui_rect`, for the reason `info_body`
    // states at its own publication: this draws inside `body`'s `ScrollArea`,
    // and a rect published for a scrolled-out control gets clicked by the
    // harness at a coordinate the operator can never reach.
    crate::diag::ui_rect_visible(REGION, block, ui.clip_rect());
}

#[cfg(test)]
mod tests {
    //! ⚠ **The floor, not the ceiling — R1.** These pin the sentences that are
    //! easy to get wrong and impossible to *see* wrong. What proves an operator
    //! can open somebody else's collection and learn what is in it is
    //! `tools/ui-verify`'s driven check against the release binary; a unit test
    //! calling `section` would be asserting that egui exists.

    use super::t;

    #[test]
    fn a_collection_with_no_title_says_what_acrobat_would_do() {
        // ★ Not "Unknown". The category is genuinely ABSENT from the file,
        // which is a fact about the file; Acrobat lists such a set with no
        // heading. Saying so beats saying pdfcer does not know.
        let sentence = t::properties_no_category().to_lowercase();
        assert!(!sentence.contains("unknown"), "got {sentence:?}");
    }

    #[test]
    fn a_stamp_pointing_nowhere_is_described_and_not_diagnosed() {
        // ★★ THE SENTENCE THIS FEATURE'S ENGINE REQUEST EXISTS ABOUT.
        // `stamp_file::read` swallows a page-tree failure and then reports
        // EVERY stamp as pointing at nothing, so a perfectly good collection
        // inside a document pdfcer could not walk counts as entirely broken.
        // Measured on the operator's own signature file. Until that is
        // answered, this string must not accuse the document.
        let sentence = t::properties_stamp_no_page().to_lowercase();
        for forbidden in ["broken", "corrupt", "invalid", "damaged", "missing"] {
            assert!(
                !sentence.contains(forbidden),
                "the no-page sentence must describe what pdfcer OBSERVED rather \
                 than diagnose the file — it reads {sentence:?} and contains \
                 {forbidden:?}"
            );
        }
    }

    #[test]
    fn the_count_sentence_distinguishes_all_dynamic_from_some() {
        // The operator's own signature collection is entirely dynamic, and a
        // set where every entry rewrites itself deserves a different sentence
        // from one with a single such entry among twelve.
        assert_eq!(t::properties_count(1, 0), "1 stamp");
        assert_eq!(t::properties_count(12, 0), "12 stamps");
        assert_eq!(t::properties_count(3, 3), "3 stamps, all of them dynamic");
        assert_eq!(t::properties_count(12, 1), "12 stamps, 1 of them dynamic");
    }

    #[test]
    fn the_row_regions_are_a_prefix_of_the_section_region() {
        // ★ Not pedantry: several driven checks dump every declared region
        // under a prefix when they cannot find the one they wanted, and a row
        // prefix that drifted out from under the section's would vanish from
        // that dump — leaving a check reporting "no such region" about rows
        // that were on screen.
        assert!(
            super::REGION_ROW_PREFIX.starts_with(super::REGION),
            "{} is not under {}",
            super::REGION_ROW_PREFIX,
            super::REGION
        );
    }
}
