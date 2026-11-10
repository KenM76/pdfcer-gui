//! # `app::actions::whitemargins` — Pages ▸ Crop ▸ Remove white margins
//!
//! Contract: [`apply`] measures each page in scope with
//! `pdfcer_render::white_margin_crop`, then sets every measured crop through
//! the edit funnel as one `set_page_boxes` undo step. A page that paints
//! nothing, or that does not render, keeps its box and is counted in the
//! status sentence. The box is inferred, so the sentence says how (R8b).
//!
//! Measuring renders each page at 144 dpi on the UI thread, as image export
//! does; `DocumentView` borrows the session, so it cannot leave the thread.
//!
//! Design: `docs/modules/pdfcer-gui/app/actions/whitemargins.md`.

use pdfcer_core::edit::{CropBoxEdit, PageBox, PageBoxEdit};

use crate::app::state::OpenDoc;
use crate::text::page_crop as t;

/// What measuring the pages found.
#[derive(Debug, Default)]
struct Measured {
    edits: Vec<PageBoxEdit>,
    blank: usize,
    failed: usize,
}

/// Measure `pages` and crop the ones that paint something, keeping
/// `margin_mm` of paper around the ink.
pub(super) fn apply(doc: &mut OpenDoc, pages: &[usize], margin_mm: i64) {
    #[allow(
        clippy::cast_precision_loss,
        reason = "a millimetre count is exact in f64" // ui-text-exempt: lint justification
    )]
    let margin_pt = crate::units::points_from_mm(margin_mm.max(0) as f64);
    let found = measure(doc, pages, margin_pt);
    crate::diag::trace(|| {
        let first = found.edits.first().and_then(|e| match e.edit {
            CropBoxEdit::Set(r) => Some(r),
            _ => None,
        });
        let r = first.unwrap_or(pdfcer_core::page_tree::Rect::from_corners(
            0.0, 0.0, 0.0, 0.0,
        ));
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "white-margins-measured pages={} cropped={} blank={} failed={} margin_mm={margin_mm} \
             llx={:.2} lly={:.2} urx={:.2} ury={:.2}",
            pages.len(),
            found.edits.len(),
            found.blank,
            found.failed,
            r.llx,
            r.lly,
            r.urx,
            r.ury,
        )
    });
    let Some(first) = found.edits.first().map(|e| e.page_index) else {
        super::record_note(doc.edit_epoch, t::white_nothing(found.blank, found.failed));
        return;
    };
    let (cropped, blank, failed) = (found.edits.len(), found.blank, found.failed);
    super::apply::vector_edit(doc, "page-crop-changed", first, cropped, |session| {
        let mut notes = vec![t::white_cropped(cropped, blank, failed)];
        notes.extend(super::pagesize::crop(session, &found.edits)?);
        Ok::<_, pdfcer_core::edit::EditError>(notes)
    });
}

/// One crop edit per page that paints something.
fn measure(doc: &OpenDoc, pages: &[usize], margin_pt: f64) -> Measured {
    let view = doc.session.view();
    let mut found = Measured::default();
    for &index in pages {
        let Some(page) = doc.pages.get(index) else {
            found.failed += 1;
            continue;
        };
        match pdfcer_render::white_margin_crop(&view, page, margin_pt) {
            Ok(Some(crop)) => found.edits.push(PageBoxEdit::new(
                index,
                PageBox::Crop,
                CropBoxEdit::Set(crop.crop),
            )),
            Ok(None) => found.blank += 1,
            Err(why) => {
                crate::diag::trace(|| {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("white-margins-refused page={index} why={why}")
                });
                found.failed += 1;
            }
        }
    }
    found
}
