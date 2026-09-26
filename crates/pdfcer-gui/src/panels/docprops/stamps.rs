//! # `panels::docprops::stamps` — **the read half of O169**
//!
//! The operator asked for Acrobat's custom stamps *"with the same
//! import/export"*. The finding that shaped the whole feature is that there is
//! no import and no export to match: **a stamp collection is an ordinary PDF**
//! — one file per category, one page per stamp, the category in `/Info`
//! `/Title`, the names in the catalog's `/Names` → `/Pages` name tree. Handing
//! somebody the file *is* the export; `file.open` *is* the import.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/docprops/stamps.md`.

use egui::Ui;

use crate::app::state::OpenDoc;
use crate::text::stamps as t;

/// **The section's own region**, published only when the open document really
/// is a stamp collection.
pub const REGION: &str = "properties.stamp-collection"; // ui-text-exempt: trace region name, never displayed

/// The prefix of the per-stamp row regions; the stamp's **index in the name
/// tree** is appended.
pub const REGION_ROW_PREFIX: &str = "properties.stamp-collection."; // ui-text-exempt: trace region name, never displayed

/// Draw the stamp-collection section, or draw nothing at all.
pub(super) fn section(ui: &mut Ui, doc: &OpenDoc) {
    let collection = pdfcer_core::stamp_file::read(doc.session.document());
    if !crate::stamps::is_collection(&collection) {
        return;
    }

    // Scoped so the section's rect is a value egui computed rather than a
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

            // Why no page number is shown, said ONCE and above the list.
            //
            let unreadable = crate::stamps::page_tree_unreadable(&collection);
            if let Some(why) = unreadable {
                ui.label(
                    egui::RichText::new(t::properties_page_tree_unreadable(why))
                        .small()
                        .weak(),
                );
            }

            ui.add_space(2.0);
            for (index, stamp) in collection.stamps.iter().enumerate() {
                let row = ui.horizontal_wrapped(|ui| {
                    ui.label(t::properties_stamp(&stamp.display, &stamp.internal));
                    // ⚠ The order of these three cases is the whole point. An
                    // unreadable page tree is checked FIRST, because in that
                    // state `page_index` is `None` for every stamp and means
                    // nothing about any of them — printing *"names no page in
                    // this document"* there is exactly the sentence that told
                    // the operator both of his signatures were broken when
                    // neither was.
                    let page = match (unreadable, stamp.page_index) {
                        (Some(_), _) => t::properties_stamp_page_unreadable().to_owned(),
                        (None, Some(i)) => t::properties_stamp_page(i + 1),
                        (None, None) => t::properties_stamp_no_page().to_owned(),
                    };
                    ui.label(egui::RichText::new(page).small().weak());
                });
                // Per-row region. The section region below says *"this file is
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

    // `ui_rect_visible` rather than `ui_rect`, for the reason `info_body`
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
        // Not "Unknown". The category is genuinely ABSENT from the file,
        // which is a fact about the file; Acrobat lists such a set with no
        // heading. Saying so beats saying pdfcer does not know.
        let sentence = t::properties_no_category().to_lowercase();
        assert!(!sentence.contains("unknown"), "got {sentence:?}");
    }

    #[test]
    fn a_stamp_pointing_nowhere_is_described_and_not_diagnosed() {
        // THE SENTENCE THIS FEATURE'S ENGINE REQUEST EXISTS ABOUT.
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
        // Not pedantry: several driven checks dump every declared region
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
