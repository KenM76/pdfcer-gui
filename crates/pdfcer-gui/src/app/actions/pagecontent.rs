//! # `app::actions::pagecontent` — another PDF's page drawn into a page's content
//!
//! Contract: [`place`] runs `EditSession::place_page_content` once through the
//! funnel: one undo entry (`CommandKind::PlacePageContent`), the placed form
//! appended last in paint order and selected, so it moves, resizes and deletes
//! as the page's own artwork rather than as a stamp. A scale other than the
//! natural size, a stretch, and source comments or form fields not carried are
//! said on the status line. A `/Rotate` page is not compensated: the verb
//! authors in unrotated user space, as `add_image` does.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/app/actions/pagecontent.md`.

use std::path::Path;

use pdfcer_core::edit::PlacedPageContent;
use pdfcer_core::page_tree::Rect;

use super::apply::vector_edit;
use crate::app::state::OpenDoc;
use crate::text::ospaste::OsPasteRefusal;
use crate::text::pagecontent as t;

/// A factor this close to 1 is the natural size, not a scaling to report.
const NATURAL: f64 = 5e-4;

/// Draw page `source_page` of the PDF at `file` into `page`'s content,
/// filling `rect`, then select it.
pub(super) fn place(doc: &mut OpenDoc, file: &Path, source_page: usize, page: usize, rect: Rect) {
    let source = match pdfcer_core::document::Document::load(file) {
        Ok(source) => source,
        Err(error) => {
            let why = error.to_string();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("page-content-refused reason=unreadable detail={why:?}")
            });
            crate::app::status::decline::record_os_paste(OsPasteRefusal::SourceUnreadable(why));
            return;
        }
    };
    let view = source.view();
    let before = doc.edit_epoch;
    vector_edit(doc, "place-page-content", page, 1, move |session| {
        let placed = session.place_page_content(&view, source_page, page, rect)?;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "page-content-placed page={page} form={} content={} scale-x={:.4} \
                 scale-y={:.4} distorted={} imported={} annots-ignored={} widgets-ignored={} \
                 group={}",
                placed.form_id.num,
                placed.content_id.num,
                placed.scale_x,
                placed.scale_y,
                placed.distorted,
                placed.objects_imported,
                placed.source_annotations_ignored,
                placed.source_widgets_ignored,
                placed.transparency_group_carried
            )
        });
        Ok::<_, pdfcer_core::edit::EditError>(disclosures(&placed))
    });
    if doc.edit_epoch != before {
        super::picture::select_newest(doc, page);
    }
}

/// The words owed for one placement, in the order they are read.
fn disclosures(placed: &PlacedPageContent) -> Vec<String> {
    let mut said = Vec::new();
    if placed.distorted {
        said.push(t::stretched(placed.scale_x, placed.scale_y));
    } else if (placed.scale_x - 1.0).abs() > NATURAL {
        said.push(t::scaled(placed.scale_x));
    }
    if placed.source_annotations_ignored > 0 {
        said.push(t::comments_left(placed.source_annotations_ignored));
    }
    if placed.source_widgets_ignored > 0 {
        said.push(t::fields_left(placed.source_widgets_ignored));
    }
    said
}
