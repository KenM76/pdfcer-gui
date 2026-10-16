//! The text objects one paragraph is drawn from, so a Select-tool click on
//! any of them can take the paragraph whole (O295 item 1).
//!
//! The paragraph is the one a click with the text tool would open: the same
//! recognition (`reflow_recognition_options`, table cells, line marks) and the
//! same switches (*Paragraphs*, the page's answer, the paragraph's own). Its
//! runs are joined to page objects through the engine's
//! `vector::text_locate::locate_text_run`, which answers `None` rather than
//! guessing.

use pdfcer_core::text_edit::{EditableTextModel, GlyphRef, TextPosition};

use super::{detect_cell_regions, enabled_on, here, marks, reflow_recognition_options};
use crate::app::state::OpenDoc;
use crate::canvas::textedit::pin;

/// The page objects (indices into `PageObjects::objects`, ascending) whose
/// text makes up the paragraph `object` belongs to; `None` when the
/// paragraph switch answers no for it, when `object` is not text, or when the
/// paragraph lies within `object` alone; the reason, as a trace token.
pub fn of(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    object: usize,
) -> Result<Vec<usize>, &'static str> {
    let run = pin::object_text(doc, page, object)
        .ok_or("not-text")?
        .first_run;
    let text = doc.provenance_page_text(page).ok_or("no-text")?;
    let view = doc.session.view();
    let cells = detect_cell_regions(&view, page).map_err(|_| "no-cells")?;
    let model =
        EditableTextModel::recognize_with_cells(&text, &reflow_recognition_options(), &cells)
            .with_line_marks(marks(&view, page));
    let block = model
        .blocks()
        .get(
            model
                .block_at(TextPosition::new(run, 0))
                .ok_or("no-paragraph")?,
        )
        .ok_or("no-paragraph")?;
    if block.line_indices.len() < 2 {
        return Err("one-line");
    }
    let corner = (
        block.bbox.llx,
        block.bbox.lly,
        block.bbox.urx,
        block.bbox.ury,
    );
    if !here::opens(
        ctx,
        doc.serial,
        page,
        corner,
        enabled_on(ctx, doc.serial, page),
    ) {
        return Err("switched-off");
    }
    let mut runs: Vec<usize> = block
        .line_indices
        .iter()
        .filter_map(|&l| model.lines().get(l))
        .flat_map(|line| line.glyphs.iter().map(|g| g.run))
        .collect();
    runs.sort_unstable();
    runs.dedup();

    use crate::canvas::target::CanvasTargetProvider as _;
    let provider = doc.page_objects().ok_or("no-objects")?;
    let objects = provider.page_objects_model(page).ok_or("no-objects")?;
    let mut found: Vec<usize> = runs
        .into_iter()
        .filter_map(|run| model.provenance(GlyphRef::new(run, 0)))
        .filter_map(
            |p| match pdfcer_core::vector::text_locate::locate_text_run(objects, p) {
                Some(pdfcer_core::vector::text_locate::TextRunRef::Page {
                    object_index, ..
                }) => Some(object_index),
                _ => None,
            },
        )
        .collect();
    found.sort_unstable();
    found.dedup();
    if !found.contains(&object) {
        return Err("not-joined");
    }
    if found.len() < 2 {
        return Err("one-object");
    }
    Ok(found)
}
