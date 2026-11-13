//! # `canvas::pageboxes` — outline each page's written trim, bleed and art box
//!
//! `view.page_boxes`, Acrobat's *Show Art, Trim & Bleed Boxes*. A view aid
//! like the grid: off by default, canvas only, never printed or saved, and it
//! marks the page's geometry rather than any content's state. Only boxes the
//! file writes (`BoxResolution::AsWritten` or `Clipped`) are drawn — a
//! defaulted box equals the crop box, which is the page edge already.
//!
//! Each box has its own colour role and its own stroke pattern (trim solid,
//! bleed dashed, art dotted), so colour is never the only cue.
//!
//! Traces `page-boxes on=1 pages=<n>:<which>+<which>,...` while on and
//! `page-boxes on=0` while off, each only when it changes, and a
//! `canvas.page-box.<page>.<which>` region per outline, pages 1-based.

use egui::{Pos2, Rect, Shape, Stroke, Ui};
use pdfcer_core::page_tree::{BoxResolution, Page};
use std::fmt::Write as _;

use crate::app::state::OpenDoc;
use crate::canvas::strip::PageView;

use pdfcer_gui_base::overlayroles::{
    PAGE_BOX_ART_ROLE as ART_ROLE, PAGE_BOX_BLEED_ROLE as BLEED_ROLE,
    PAGE_BOX_TRIM_ROLE as TRIM_ROLE,
};

/// Region prefix; followed by `<page>.<which>`.
pub const REGION_PREFIX: &str = "canvas.page-box."; // ui-text-exempt: trace region name, never displayed

/// How a box's outline is stroked.
#[derive(Clone, Copy)]
enum Pattern {
    Solid,
    Dashed,
    Dotted,
}

/// The boxes drawn, in drawing order: (trace key, role, pattern).
const BOXES: [(&str, &str, Pattern); 3] = [
    ("bleed", BLEED_ROLE, Pattern::Dashed), // ui-text-exempt: trace keys, never displayed
    ("trim", TRIM_ROLE, Pattern::Solid),    // ui-text-exempt: trace keys, never displayed
    ("art", ART_ROLE, Pattern::Dotted),     // ui-text-exempt: trace keys, never displayed
];

/// The page's box for a [`BOXES`] key, if the file writes it.
fn written(page: &Page, key: &str) -> Option<pdfcer_core::page_tree::Rect> {
    let (rect, res) = match key {
        "bleed" => (page.bleed_box, page.bleed_box_resolution),
        "trim" => (page.trim_box, page.trim_box_resolution),
        _ => (page.art_box, page.art_box_resolution),
    };
    matches!(res, BoxResolution::AsWritten | BoxResolution::Clipped).then_some(rect)
}

/// A PDF-space rectangle on screen, through the page's rotation and the
/// frame's map.
fn to_screen(r: pdfcer_core::page_tree::Rect, page: &Page, view: &PageView) -> Option<Rect> {
    #[allow(clippy::cast_possible_truncation)]
    let corner = |x: f64, y: f64| {
        crate::viewer::pdf_space_to_canvas(Pos2::new(x as f32, y as f32), page)
            .map(|c| view.map.to_screen(c))
    };
    Some(Rect::from_two_pos(
        corner(r.llx, r.lly)?,
        corner(r.urx, r.ury)?,
    ))
}

/// Draw the outlines on every page the frame shows.
pub(super) fn draw(ui: &Ui, doc: &OpenDoc, pages: &[PageView], clip: Rect) {
    let roles = egui_shell::theme::Overlays::of(ui.ctx());
    let fallback = egui_shell::theme::Theme::accent_pair(ui.ctx()).0;
    let painter = ui.painter().with_clip_rect(clip);
    let mut traced = String::from("page-boxes on=1 pages="); // ui-text-exempt: trace line, never displayed
    let mut first = true;
    for view in pages {
        let Some(page) = doc.pages.get(view.page) else {
            continue;
        };
        let mut drawn: Vec<&str> = Vec::new();
        for (key, role, pattern) in BOXES {
            let Some(screen) = written(page, key).and_then(|r| to_screen(r, page, view)) else {
                continue;
            };
            let stroke = Stroke::new(1.5, roles.get(role).unwrap_or(fallback));
            outline(&painter, screen, stroke, pattern);
            crate::diag::ui_rect_visible(
                &format!("{REGION_PREFIX}{}.{key}", view.page + 1), // ui-text-exempt: trace region name, never displayed
                screen,
                clip,
            );
            drawn.push(key);
        }
        if !drawn.is_empty() {
            let sep = if first { "" } else { "," };
            let _ = write!(traced, "{sep}{}:{}", view.page + 1, drawn.join("+"));
            first = false;
        }
    }
    crate::diag::trace_changed(EVENT, || traced);
}

/// The trace slot and event name.
const EVENT: &str = "page-boxes"; // ui-text-exempt: a trace event name, never displayed

/// Trace that the switch is off; see the module header.
pub(super) fn trace_off() {
    crate::diag::trace_changed(EVENT, || format!("{EVENT} on=0")); // ui-text-exempt: trace line, never displayed
}

fn outline(painter: &egui::Painter, r: Rect, stroke: Stroke, pattern: Pattern) {
    let ring = [
        r.left_top(),
        r.right_top(),
        r.right_bottom(),
        r.left_bottom(),
        r.left_top(),
    ];
    match pattern {
        Pattern::Solid => {
            painter.add(Shape::line(ring.to_vec(), stroke));
        }
        Pattern::Dashed => {
            painter.extend(Shape::dashed_line(&ring, stroke, 6.0, 4.0));
        }
        Pattern::Dotted => {
            painter.extend(Shape::dashed_line(&ring, stroke, 1.5, 3.0));
        }
    }
}
