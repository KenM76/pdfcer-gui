//! # `panels::docprops::boxes` — page boxes not used as the file wrote them
//!
//! ISO 32000-2 §14.11.2.1 reads a crop, bleed, trim or art box that runs
//! past the media box as its intersection with it; one that misses the sheet
//! or is malformed falls back to its Table 30 default. The engine records
//! which per page (`Page::{crop,bleed,trim,art}_box_resolution`) and does not
//! rewrite the file, so the difference is said here, never drawn on the page.

use crate::app::state::OpenDoc;
use crate::text::panels::docprops as t;
use pdfcer_core::page_tree::{BoxResolution, Page};
use std::fmt::Write as _;

/// The block of page-box lines, published only when one is drawn.
pub const REGION: &str = "docprops.page-boxes"; // ui-text-exempt: trace region name, never displayed

/// Reads one box's resolution off a page.
type Resolution = fn(&Page) -> BoxResolution;

/// Each box's trace key and resolution, in the order the text catalogue
/// numbers them (0 crop, 1 bleed, 2 trim, 3 art).
const BOXES: [(&str, Resolution); 4] = [
    ("crop", |p| p.crop_box_resolution),
    ("bleed", |p| p.bleed_box_resolution),
    ("trim", |p| p.trim_box_resolution),
    ("art", |p| p.art_box_resolution),
];

/// The 1-based pages on which `resolution(page)` is `want`.
fn pages_where(pages: &[Page], resolution: Resolution, want: BoxResolution) -> Vec<u32> {
    (1..)
        .zip(pages)
        .filter(|(_, p)| resolution(p) == want)
        .map(|(n, _)| n)
        .collect()
}

/// Draws one line per box and outcome that occurred, and traces them every
/// frame (`page-boxes pages=N` alone when the file's boxes are all used as
/// written or defaulted).
pub(super) fn lines(ui: &mut egui::Ui, doc: &OpenDoc) {
    let mut said: Vec<String> = Vec::new();
    let mut traced = format!("page-boxes pages={}", doc.pages.len()); // ui-text-exempt: trace line, never displayed
    for (which, (key, resolution)) in BOXES.into_iter().enumerate() {
        for (outcome, want, say) in [
            (
                "clipped",
                BoxResolution::Clipped,
                t::page_box_clipped as fn(usize, &str) -> String,
            ),
            ("unusable", BoxResolution::Unusable, t::page_box_unusable),
        ] {
            let on = pages_where(&doc.pages, resolution, want);
            if on.is_empty() {
                continue;
            }
            let ranges = pdfcer_core::fontinfo::format_page_ranges(&on);
            let _ = write!(traced, " {key}_{outcome}={ranges}"); // ui-text-exempt: trace line, never displayed
            said.push(say(which, &ranges));
        }
    }
    crate::diag::trace(|| traced);
    if said.is_empty() {
        return;
    }
    let block = ui
        .vertical(|ui| {
            for line in said {
                ui.weak(line);
            }
        })
        .response
        .rect;
    crate::diag::ui_rect_visible(REGION, block, ui.clip_rect());
}
