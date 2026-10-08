//! # `app::actions::blocktext` — committing a paragraph's whole text
//!
//! The body of `TextAction::CommitBlock`: one `EditSession::edit_block_text`,
//! one undo entry, through the edit funnel. A refusal is worded by the
//! family it belongs to — a text refusal as a caret commit's, a block refusal
//! as a reflow's.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/blocktext.md`.

use pdfcer_core::text_edit::BlockEditError;

use crate::app::state::OpenDoc;

/// Replace paragraph `block` on `page` with `text`, opened from line `run`.
pub(super) fn commit(
    doc: &mut OpenDoc,
    page: usize,
    (run, block): (usize, usize),
    wrap: Option<f64>,
    text: &str,
) {
    let options = crate::canvas::textedit::promote::options(doc, wrap);
    super::apply::vector_edit(doc, "edit-block-text", page, 1, |session| {
        session
            .edit_block_text(page, block, text, &options)
            .inspect_err(|error| match error {
                BlockEditError::Text(e) => {
                    crate::app::status::decline::record_edit_text_refusal(page, run, true, e);
                }
                BlockEditError::Block(e) => {
                    crate::app::status::decline::record_reflow(super::textstyle::reflow_refusal(e));
                }
                // The funnel's generic refusal stands.
                _ => {}
            })
            .map(|report| {
                let paragraphs = text.split('\n').count();
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed
                    format!(
                        "edit-block-text-applied page={page} block={block} lines={}->{} \
                         paragraphs={paragraphs} glyphs_added={}",
                        report.lines_before,
                        report.lines_after,
                        report.glyphs_added.len()
                    )
                });
                report.disclosures
            })
    });
}
