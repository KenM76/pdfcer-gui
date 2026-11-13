//! # `panels::docprops::box_sizes` — every page's five box sizes
//!
//! A collapsed section of Document properties. Pages whose media, crop,
//! bleed, trim and art boxes have the same sizes and resolve the same way
//! share one entry, so a long drawing set is a few entries. Sizes are the
//! boxes as the engine resolved them (`Page::{media,crop,bleed,trim,art}_box`
//! and their `BoxResolution`), before `/Rotate`. The engine does not record
//! whether a written box is the page's own entry or inherited from a parent
//! `Pages` node, so both read as written.

use crate::app::state::OpenDoc;
use crate::text::panels::docprops as t;
use pdfcer_core::dimension::Unit;
use pdfcer_core::page_tree::{BoxResolution, Page};
use pdfcer_gui_base::entry::LengthUnit;
use std::collections::HashMap;
use std::fmt::Write as _;

/// The section header's region.
pub const HEADER: &str = "docprops.box-sizes.header"; // ui-text-exempt: trace region name, never displayed
/// The open section's body.
pub const REGION: &str = "docprops.box-sizes"; // ui-text-exempt: trace region name, never displayed
/// Prefix of the unit choices, followed by the unit's label.
pub const UNIT_PREFIX: &str = "docprops.box-sizes.unit."; // ui-text-exempt: trace region name, never displayed

/// The units offered, in the order shown.
const UNITS: [LengthUnit; 3] = [
    LengthUnit::Of(Unit::Millimeter),
    LengthUnit::Of(Unit::Inch),
    LengthUnit::Point,
];

/// Decimals shown per entry of [`UNITS`].
const DECIMALS: [usize; 3] = [1, 2, 1];

/// One box: its size in points, hundredths rounded, and how it resolved.
type Cell = (i64, i64, BoxResolution);

/// The five boxes of a page in the catalogue's order (media, crop, bleed,
/// trim, art). Media is always written.
fn cells(page: &Page) -> [Cell; 5] {
    let hundredths = |v: f64| (v * 100.0).round() as i64;
    let cell = |r: &pdfcer_core::page_tree::Rect, res| {
        (hundredths(r.width()), hundredths(r.height()), res)
    };
    [
        cell(&page.media_box, BoxResolution::AsWritten),
        cell(&page.crop_box, page.crop_box_resolution),
        cell(&page.bleed_box, page.bleed_box_resolution),
        cell(&page.trim_box, page.trim_box_resolution),
        cell(&page.art_box, page.art_box_resolution),
    ]
}

/// Groups of identical pages, in order of each group's first page; pages
/// are 1-based.
fn groups(pages: &[Page]) -> Vec<([Cell; 5], Vec<u32>)> {
    let mut out: Vec<([Cell; 5], Vec<u32>)> = Vec::new();
    let mut at: HashMap<[Cell; 5], usize> = HashMap::new();
    for (n, page) in (1..).zip(pages) {
        let key = cells(page);
        let slot = *at.entry(key).or_insert_with(|| {
            out.push((key, Vec::new()));
            out.len() - 1
        });
        if let Some((_, list)) = out.get_mut(slot) {
            list.push(n);
        }
    }
    out
}

/// The trace word for a resolution.
fn word(res: BoxResolution) -> &'static str {
    match res {
        BoxResolution::AsWritten => "written",
        BoxResolution::Clipped => "clipped",
        BoxResolution::Unusable => "unusable",
        _ => "default",
    }
}

/// Draws the section and, while it is open, traces
/// `box-sizes unit= groups= g1=<pages> g1_media=WxH:written ...` with sizes
/// in points.
pub(super) fn section(ui: &mut egui::Ui, doc: &OpenDoc) {
    let id = ui.make_persistent_id("docprops.box-sizes.unit");
    let mut chosen: usize = ui.data(|d| d.get_temp(id)).unwrap_or(0);
    let response = egui::CollapsingHeader::new(t::box_sizes_heading())
        .id_salt("docprops.box-sizes")
        .default_open(false)
        .show(ui, |ui| body(ui, doc, &mut chosen));
    ui.data_mut(|d| d.insert_temp(id, chosen));
    crate::diag::ui_rect_visible(HEADER, response.header_response.rect, ui.clip_rect());
    if let Some(inner) = response.body_response {
        crate::diag::ui_rect_visible(REGION, inner.rect, ui.clip_rect());
    }
}

fn body(ui: &mut egui::Ui, doc: &OpenDoc, chosen: &mut usize) {
    ui.horizontal(|ui| {
        for (i, unit) in UNITS.into_iter().enumerate() {
            let r = ui
                .radio_value(chosen, i, unit.label())
                .on_hover_text(t::box_sizes_unit_tooltip());
            // ui-text-exempt: trace region name, never displayed
            crate::diag::ui_rect_visible(
                &format!("{UNIT_PREFIX}{}", unit.label()),
                r.rect,
                ui.clip_rect(),
            );
        }
    });
    let index = (*chosen).min(UNITS.len() - 1);
    let (unit, places) = (UNITS[index], DECIMALS[index]);
    let all = groups(&doc.pages);
    let mut traced = format!("box-sizes unit={} groups={}", unit.label(), all.len()); // ui-text-exempt: trace line, never displayed
    for (g, (key, pages)) in (1..).zip(&all) {
        let ranges = pdfcer_core::fontinfo::format_page_ranges(pages);
        let _ = write!(traced, " g{g}={ranges}"); // ui-text-exempt: trace line, never displayed
        ui.label(t::box_sizes_pages(&ranges));
        egui::Grid::new(("docprops.box-sizes.grid", g))
            .num_columns(2)
            .show(ui, |ui| {
                for (which, &(w, h, res)) in key.iter().enumerate() {
                    let _ = write!(
                        traced,
                        " g{g}_{}={}x{}:{}", // ui-text-exempt: trace line, never displayed
                        t::BOX_SIZE_KEYS[which],
                        w as f64 / 100.0,
                        h as f64 / 100.0,
                        word(res)
                    );
                    ui.label(t::box_size_label(which));
                    let size = |v: i64| convert(v as f64 / 100.0, unit);
                    ui.label(t::box_size(
                        size(w),
                        size(h),
                        unit.label(),
                        places,
                        which,
                        res,
                    ));
                    ui.end_row();
                }
            });
        ui.add_space(4.0);
    }
    ui.weak(t::box_sizes_note());
    crate::diag::trace(|| traced);
}

/// `points` in `unit`.
fn convert(points: f64, unit: LengthUnit) -> f64 {
    match unit {
        LengthUnit::Point => points,
        LengthUnit::Of(u) => crate::units::from_points(points, u),
    }
}
