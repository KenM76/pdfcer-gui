//! A click in a recognised (invisible) line opens its recognised paragraph,
//! found by `EditSession::recognised_block_at_point` and committed by
//! `edit_recognised_block_text`: unchanged words keep their place over the
//! scan, so the rewrite never re-spaces the layer. Recognised paragraphs are
//! numbered among the page's recognised text only, so the anchor says so.

use pdfcer_core::text_edit::LineEndSource;
use pdfcer_gui_base::text::draftnote::DraftNote;

use super::{Anchor, Declined, Draft, here};
use crate::app::state::OpenDoc;

/// Re-anchor a clicked recognised run draft on its recognised paragraph,
/// its text the engine's spelling of the paragraph. `Err(None)` leaves the
/// draft as it was: the run is the whole paragraph, the paragraph answers
/// no, or the engine found none.
///
/// # Errors
///
/// `Err(None)`: the draft stays a run draft, with nothing owed.
pub(super) fn widen(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &mut Draft,
    default: bool,
) -> Result<LineEndSource, Option<DraftNote>> {
    let Anchor::Run { run, original } = &draft.anchor else {
        return Err(None);
    };
    let run = *run;
    let result = opened(ctx, doc, draft, run, original, default);
    match result {
        Ok(opened) => {
            *draft = opened;
            crate::diag::trace(|| {
                let block = match draft.anchor {
                    Anchor::Block { block, .. } => block,
                    _ => usize::MAX,
                };
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "text-edit-widened page={} run={run} block={block} recognised=true len={} \
                     caret={}",
                    draft.page,
                    draft.text.chars().count(),
                    draft.caret
                )
            });
            Ok(LineEndSource::Marked)
        }
        Err(d) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "text-edit-widen-declined page={} run={run} reason={} recognised=true",
                    draft.page, d.token
                )
            });
            Err(None)
        }
    }
}

/// The draft re-anchored on the recognised paragraph under `run`'s centre,
/// the caret where it was within the run.
fn opened(
    ctx: &egui::Context,
    doc: &OpenDoc,
    draft: &Draft,
    run: usize,
    original: &str,
    default: bool,
) -> Result<Draft, Declined> {
    let text = doc
        .provenance_page_text(draft.page)
        .ok_or(Declined::no_paragraph("no-text"))?;
    let at = text
        .runs
        .get(run)
        .and_then(|r| r.bbox)
        .ok_or(Declined::no_paragraph("no-box"))?;
    let (x, y) = (f64::midpoint(at.llx, at.urx), f64::midpoint(at.lly, at.ury));
    let hit = doc
        .session
        .recognised_block_at_point(draft.page, x, y)
        .ok()
        .flatten()
        .ok_or(Declined::no_paragraph("no-paragraph"))?;
    if hit.words.len() < 2 || hit.text.trim() == original.trim() {
        return Err(Declined::no_paragraph("one-line"));
    }
    let bbox = (hit.bbox.llx, hit.bbox.lly, hit.bbox.urx, hit.bbox.ury);
    if !here::opens(ctx, doc.serial, draft.page, bbox, default) {
        return Err(Declined::no_paragraph("switched-off"));
    }
    let shift = hit
        .text
        .find(original)
        .map_or(0, |byte| hit.text[..byte].chars().count());
    Ok(Draft {
        anchor: Anchor::Block {
            block: hit.block_index,
            run,
            llx: bbox.0,
            lly: bbox.1,
            urx: bbox.2,
            ury: bbox.3,
            original: hit.text.clone(),
            places: Vec::new(),
            recognised: true,
        },
        text: hit.text,
        caret: draft.caret + shift,
        mark: draft.mark.map(|m| m + shift),
        ..draft.clone()
    })
}
