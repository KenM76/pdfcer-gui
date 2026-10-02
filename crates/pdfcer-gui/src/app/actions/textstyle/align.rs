//! Setting paragraph alignment (`OPERATOR_REQUESTS.md` O271): each paragraph
//! is re-laid by the engine's block reflow with the alignment asked for.
//!
//! Paragraphs are applied last-first, so a reflow that adds or removes lines
//! never renumbers a paragraph still to come. Each is its own undo step; the
//! gesture stops at the first refusal, whose sentence `reflow` has already put
//! on the bar.

use pdfcer_core::text_edit::BlockAlignment;

use crate::app::state::OpenDoc;

/// Align every paragraph in `blocks` (relaxed-recognition numbering).
pub(in crate::app::actions) fn align(
    doc: &mut OpenDoc,
    page: usize,
    blocks: &[usize],
    alignment: BlockAlignment,
) {
    let mut ordered = blocks.to_vec();
    ordered.sort_unstable();
    ordered.dedup();
    let total = ordered.len();
    let mut done = 0_usize;
    for block in ordered.into_iter().rev() {
        if !super::reflow(doc, page, block, Some(alignment)) {
            break;
        }
        done += 1;
    }
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("text-align-applied page={page} alignment={alignment:?} blocks={done}/{total}")
    });
}
