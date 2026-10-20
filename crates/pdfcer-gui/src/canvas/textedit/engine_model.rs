//! # `canvas::textedit::engine_model` — the paragraph numbering the engine's block verbs read
//!
//! Contract: [`recognise`] builds the block model exactly as
//! `EditSession::block_at_point`, `edit_block_text` and `reflow_block` build
//! theirs — cell-aware, reflow options, the page's line marks attached — so a
//! block index taken from it names the same paragraph when handed to one of
//! them. Every index this shell sends to a block verb comes from here; a
//! model built any other way numbers paragraphs differently on a page pdfcer
//! wrote a blank-line paragraph into (the marks merge it back into one block).

use pdfcer_core::content::ContentStream;
use pdfcer_core::text_edit::{
    EditableTextModel, LineMarks, detect_cell_regions, reflow_recognition_options,
};
use pdfcer_core::text_extract::PageText;
use pdfcer_core::view::DocumentView;

/// The engine's block model of `page`; `None` when its cells cannot be read.
#[must_use]
pub fn recognise<'t>(
    text: &'t PageText,
    view: &DocumentView<'_>,
    page: usize,
) -> Option<EditableTextModel<'t>> {
    let cells = detect_cell_regions(view, page).ok()?;
    Some(
        EditableTextModel::recognize_with_cells(text, &reflow_recognition_options(), &cells)
            .with_line_marks(marks(view, page)),
    )
}

/// The page's line-end marks, so a paragraph pdfcer wrote reads its breaks
/// exactly; empty when the page's content cannot be read.
fn marks(view: &DocumentView<'_>, page: usize) -> LineMarks {
    pdfcer_core::page_tree::pages_in(view)
        .ok()
        .and_then(|pages| ContentStream::from_page(view, pages.get(page)?).ok())
        .map(|content| LineMarks::scan(&content))
        .unwrap_or_default()
}
