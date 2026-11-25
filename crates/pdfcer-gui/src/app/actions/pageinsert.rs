//! # `app::actions::pageinsert` — inserting another document's pages, and what comes with them
//!
//! Split from `pages`; design in `docs/modules/pdfcer-gui/app/actions/pages.md`.

use std::path::Path;

use pdfcer_gui_base::prefs::PageLabelPolicy;

use crate::app::state::OpenDoc;

pub(super) fn insert_from_file(
    doc: &mut OpenDoc,
    path: &Path,
    pages: &[usize],
    position: pdfcer_core::pageops::InsertPosition,
    labels: PageLabelPolicy,
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
    insert_from_view(doc, &source.view(), pages, position, labels);
}

/// What an insert brings besides the pages: the bookmarks that reach them,
/// after this document's own, and the named destinations naming them, as
/// Acrobat's Insert Pages does; and the labels the operator's policy gives
/// the inserted pages. `insert_pages_with_options` rather than the shorter
/// `insert_pages_with`, which sets the labels alone and would drop the
/// outline and destination carry.
fn carry(labels: PageLabelPolicy) -> pdfcer_core::pageops::InsertOptions {
    pdfcer_core::pageops::InsertOptions::default()
        .with_outline(pdfcer_core::pageops::InsertedOutline::Append)
        .with_named_destinations(true)
        .with_labels(labels.inserted())
}

/// **The half of an insert that does not care where the source came from.**
pub(super) fn insert_from_view(
    doc: &mut OpenDoc,
    view: &pdfcer_core::view::DocumentView<'_>,
    pages: &[usize],
    position: pdfcer_core::pageops::InsertPosition,
    labels: PageLabelPolicy,
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
    let landing = landing_of(doc, position);
    let before = doc.pages.len();

    super::apply::vector_edit(doc, "insert-pages", landing, count, |session| {
        session
            .insert_pages_with_options(view, pages, position, &carry(labels))
            // `orphaned_widgets` is exact, so the number goes in front of the
            // operator unhedged and a zero drops the clause entirely.
            //
            // `orphaned_widgets_unrecoverable` is beside it, and the two
            // numbers are two different pieces of news. The engine
            // measured its own output and found that of 13 orphans, 11 could be
            // registered and 2 had lost their identity permanently — and said
            // plainly that the old undifferentiated sentence *"is true of both
            // and useful for only one"*, because it describes a chore for the
            // 11 and a permanent loss for the 2, in the milder wording.
            .map(|outcome| notes_for(&outcome, landing))
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
        super::pages::trace_labels(doc, "insert");
        let target = landing.min(doc.pages.len().saturating_sub(1));
        doc.view.go_to_page(target, doc.pages.len());
        // `tracked_page` is deliberately left behind: under a continuous display
        // the canvas reads `page_index != tracked_page` as a navigation and
        // scrolls the strip to it. Matching them here would make the next
        // frame's scroll tracking put `page_index` back where the view sits.
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

/// The disclosures an insert owes: the pages, then each structure that came
/// across or was left behind.
fn notes_for(o: &pdfcer_core::edit::InsertOutcome, landing: usize) -> Vec<String> {
    let mut notes = vec![crate::text::pages::inserted(
        o.pages_inserted,
        o.orphaned_widgets,
        o.orphaned_widgets_unrecoverable,
        crate::text::pages::Structures {
            outline_dropped: o.source_outline_dropped,
            labels_dropped: o.source_page_labels_dropped,
            labels_stale: o.page_labels_stale,
        },
        landing.checked_sub(1),
    )];
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "insert-pages-carried outline_items={} dests={} dests_renamed={} \
                 outline_dropped={}",
            o.outline_items_carried,
            o.dests_carried,
            o.dests_renamed,
            u8::from(o.source_outline_dropped),
        )
    });
    notes.extend(crate::text::insertoutline::carried(
        o.outline_items_carried,
        o.dests_carried,
        o.dests_renamed,
    ));
    notes.extend(crate::text::insertfields::carried(
        o.fields_carried,
        o.fields_renamed,
        o.fields_split,
        o.acroform_created,
    ));
    notes.extend(crate::text::structlinks::repointed(
        o.structure_links_to_pages,
        o.structure_links_to_first_page,
        o.links_to_pages_left_behind,
        o.source_tags_dropped,
    ));
    notes.extend(crate::text::cedimcopy::copied(&o.into()));
    notes
}

/// Where the first inserted sheet will land, read before the edit changes
/// what `position` names.
fn landing_of(doc: &OpenDoc, position: pdfcer_core::pageops::InsertPosition) -> usize {
    match position {
        pdfcer_core::pageops::InsertPosition::Start => 0,
        pdfcer_core::pageops::InsertPosition::End => doc.pages.len(),
        pdfcer_core::pageops::InsertPosition::Before(n) => n,
        pdfcer_core::pageops::InsertPosition::After(n) => n.saturating_add(1),
        // `InsertPosition` is `#[non_exhaustive]`: a variant added upstream
        // lands the operator on the page they were already on, which is wrong
        // but harmless, where a panic would lose the insert they just made.
        _ => doc.view.page_index,
    }
}
