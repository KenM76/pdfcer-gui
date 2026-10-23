//! # `dialogs::compact_compare` — each picture, before and after
//!
//! Contract: [`rows`] reads a recompression's plan into one [`Row`] per
//! picture it changes, and [`show`] draws them as a folded table under the
//! finished recompression, before the copy is saved: pixel size, resolution,
//! colours, storage, JPEG quality asked and written, closeness reached and
//! bytes, each as `before to after`. At most [`LISTED`] rows are drawn; the
//! rest are counted in a line under the table, never dropped silently.

use egui::Ui;
use pdfcer_core::optimize::{ImageEncode, OptimizePlan};

use crate::text::compact_compare as t;

/// The fold's header.
pub const REGION: &str = "compact.compare";
/// The table, drawn only while the fold is open.
pub const REGION_TABLE: &str = "compact.compare.table";
/// The most rows the table draws.
const LISTED: usize = 200;

/// What the copy does to one picture.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Object number.
    pub number: u32,
    /// Pixel size before and after.
    pub pixels: ((u32, u32), (u32, u32)),
    /// The largest placement's resolution after resizing; `None` when not resized.
    pub ppi_after: Option<f64>,
    /// Colour space before and after, by family name.
    pub colours: (String, String),
    /// Storage before and after, in the window's words.
    pub stored: (String, String),
    /// JPEG quality asked and written; `None` unless written as JPEG.
    pub quality: Option<(u8, u8)>,
    /// SSIM reached in ten-thousandths, and whether it met the target.
    pub closeness: Option<(u16, bool)>,
    /// Stored bytes before and after.
    pub bytes: (u64, u64),
}

/// One row per picture `plan` changes, in plan order.
#[must_use]
pub fn rows(plan: &OptimizePlan) -> Vec<Row> {
    plan.changes
        .iter()
        .filter_map(|c| {
            let image = c.image.as_ref()?;
            let before = c.filters_before.last().map_or("", String::as_str);
            Some(Row {
                number: c.id.num,
                pixels: (image.size_before, image.size_after),
                ppi_after: image
                    .binding_ppi_after
                    .filter(|_| image.size_after != image.size_before),
                colours: image.color_space.clone(),
                stored: (t::filter(before), encoded(image.encoded_as)),
                quality: image.jpeg.map(|j| (j.asked, j.used)),
                closeness: image
                    .jpeg
                    .and_then(|j| j.ssim)
                    .map(|s| (s.achieved.ten_thousandths(), s.met)),
                bytes: (c.before_bytes, c.after_bytes),
            })
        })
        .collect()
}

/// The storage written, in the words [`t::filter`] gives its filter.
fn encoded(encode: ImageEncode) -> String {
    let filter = match encode {
        ImageEncode::Jpeg => "DCTDecode",
        ImageEncode::Flate { .. } => "FlateDecode",
        ImageEncode::Lzw => "LZWDecode",
        ImageEncode::RunLength => "RunLengthDecode",
        ImageEncode::Ccitt => "CCITTFaxDecode",
        ImageEncode::Jbig2 => "JBIG2Decode",
        ImageEncode::Brotli { .. } => "BrotliDecode",
        _ => "FlateDecode",
    };
    t::filter(filter)
}

/// The folded table; nothing when no picture changes.
pub fn show(ui: &mut Ui, rows: &[Row]) {
    if rows.is_empty() {
        return;
    }
    let fold = egui::CollapsingHeader::new(t::heading(rows.len()))
        .id_salt("compact-compare") // ui-text-exempt: an id, never displayed
        .show(ui, |ui| {
            // Eight columns are wider than the window; they scroll sideways
            // inside it rather than widen it.
            egui::ScrollArea::horizontal()
                .id_salt("compact-compare-scroll") // ui-text-exempt: an id, never displayed
                .show(ui, |ui| {
                    let grid = egui::Grid::new("compact-compare-grid") // ui-text-exempt: an id, never displayed
                        .striped(true)
                        .show(ui, |ui| {
                            for heading in t::columns() {
                                ui.strong(heading);
                            }
                            ui.end_row();
                            for row in rows.iter().take(LISTED) {
                                cells(ui, row);
                            }
                        });
                    crate::diag::ui_rect_visible(REGION_TABLE, grid.response.rect, ui.clip_rect());
                });
            if rows.len() > LISTED {
                ui.small(t::more(rows.len() - LISTED));
            }
        });
    let header = fold.header_response.on_hover_text(t::heading_tooltip());
    crate::diag::ui_rect_visible(REGION, header.rect, ui.clip_rect());
}

/// One row's cells, in [`t::columns`] order.
fn cells(ui: &mut Ui, row: &Row) {
    ui.label(t::picture(row.number));
    ui.label(t::change(
        &t::pixels(row.pixels.0),
        &t::pixels(row.pixels.1),
    ));
    ui.label(t::resolution(row.ppi_after));
    ui.label(t::change(&row.colours.0, &row.colours.1));
    ui.label(t::change(&row.stored.0, &row.stored.1));
    ui.label(t::quality(row.quality));
    ui.label(t::closeness(row.closeness));
    ui.label(t::size(row.bytes.0, row.bytes.1));
    ui.end_row();
}
