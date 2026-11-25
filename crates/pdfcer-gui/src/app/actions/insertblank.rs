//! Pages ▸ Insert blank…: blank sheets as copies of a one-page blank document.

use pdfcer_core::pageops::InsertPosition;

/// Insert `count` blank sheets of `size_pt` (width, height) at `position`.
///
/// The engine has no blank-page verb, so the sheets are copies of a one-page
/// blank document of that size, made here and inserted as the list
/// `[0; count]` in one `insert_pages` call: one undo step, and a fresh page
/// object per repeat because the engine copies a repeated index afresh.
pub(super) fn insert_blank(
    doc: &mut crate::app::state::OpenDoc,
    size_pt: (f64, f64),
    count: usize,
    position: InsertPosition,
    labels: pdfcer_gui_base::prefs::PageLabelPolicy,
) {
    let rect = pdfcer_core::page_tree::Rect::from_corners(0.0, 0.0, size_pt.0, size_pt.1);
    let blank = match pdfcer_gui_base::blank::document_sized(rect) {
        Ok((blank, _)) => blank,
        Err(detail) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("insert-blank-refused reason={detail}")
            });
            super::record_note(doc.edit_epoch, crate::text::pages::insert_failed(&detail));
            return;
        }
    };
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "insert-blank-requested n={count} w_pt={:.2} h_pt={:.2} position={position:?}",
            size_pt.0, size_pt.1
        )
    });
    super::pages::insert_from_view(doc, &blank.view(), &vec![0; count], position, labels);
}
