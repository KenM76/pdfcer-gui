//! Where a Review mark lands: an annotation, or the page's own content.
//!
//! The pen's `on_page` switch picks the verb. Both take the same `MarkupSpec`
//! and `MarkupOptions` and both are one undo entry. The page-content verb
//! writes no note — the engine says so in `paste.disclosures`, which the
//! funnel carries to the status line — and is refused on a certified file
//! whose permissions allow comments but not content changes; that refusal
//! reaches the status line through the funnel too.

use super::funnel::{vector_edit, vector_edit_on_page};
use crate::app::state::OpenDoc;
use pdfcer_core::annot_author::MarkupSpec;
use pdfcer_core::edit::{MarkupContentOutcome, MarkupOptions};

/// Author `spec` on `page`, as an annotation or, with `on_page`, as content.
///
/// `label` names the annotation route's funnel line; the content route's is
/// `add-markup-as-content`, followed by a `markup-as-content-applied` line
/// carrying the object range the page gained.
pub(super) fn author(
    doc: &mut OpenDoc,
    label: &str,
    page: usize,
    spec: &MarkupSpec,
    options: &MarkupOptions,
    on_page: bool,
) {
    let Ok((options, receipt)) = super::drawlayer::onto(doc, label, options.clone()) else {
        return;
    };
    let options = &options;
    let receipt: Vec<String> = receipt.into_iter().collect();
    if !on_page {
        // An annotation changes only its own page's `/Annots`.
        vector_edit_on_page(doc, label, page, 1, |session| {
            session
                .add_markup_with(page, spec, options)
                .map(|_| receipt)
        });
        return;
    }
    let mut applied = None;
    vector_edit(doc, "add-markup-as-content", page, 1, |session| {
        session
            .add_markup_as_content(page, spec, options)
            .map(|outcome| {
                let MarkupContentOutcome { objects, paste, .. } = outcome;
                applied = Some((objects, paste.resources_added));
                let mut notes = paste.disclosures;
                notes.extend(receipt);
                notes
            })
    });
    if let Some((objects, added)) = applied {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "markup-as-content-applied page={page} objects={}..{} resources_added={added}",
                objects.start, objects.end
            )
        });
    }
}
