//! # `canvas::region` — what the snapshot region takes from its page
//!
//! The region is the snapshot box. Its objects are found by the same hit test
//! the Select tool's marquee uses, under one standing choice — *Delete takes:
//! Inside only | Touching* — rather than by drag direction, because the box is
//! moved and resized after it is drawn and its drag direction stops meaning
//! anything.
//!
//! Only the page on screen has its objects loaded, so a region on another page
//! takes nothing until that page is shown.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/region.md`.

use egui::{Pos2, Rect};
use pdfcer_gui_base::snapshotbox::SnapshotBox;

use crate::app::state::OpenDoc;
use crate::canvas::marquee::{self, Combine};
use crate::canvas::selection::SelectionState;
use crate::canvas::target::CanvasTargetProvider;

/// The `egui::Memory` key of the Inside only / Touching choice.
const TOUCHING_KEY: &str = "pdfcer-region-touching"; // ui-text-exempt: internal memory id, never displayed

/// Whether Delete takes what the region touches, not only what it encloses.
/// Off by default: the narrower answer is the one that cannot surprise.
#[must_use]
pub fn touching(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::new(TOUCHING_KEY)))
        .unwrap_or(false)
}

/// Store the Inside only / Touching choice.
pub fn set_touching(ctx: &egui::Context, touching: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(TOUCHING_KEY), touching));
}

/// What the region takes under the current choice.
#[derive(Debug, Clone, PartialEq)]
pub struct Taken {
    /// The objects, as a selection on the region's page.
    pub selection: SelectionState,
    /// Objects the region touches but does not enclose, which Inside only
    /// leaves; always 0 under Touching.
    pub crossing_kept: usize,
}

/// The region's rect in canvas space; `None` when its page is not there.
#[must_use]
pub fn canvas_rect(doc: &OpenDoc, region: SnapshotBox) -> Option<Rect> {
    let page = doc.pages.get(region.page)?;
    let r = region.rect;
    #[allow(clippy::cast_possible_truncation)]
    // ui-text-exempt: a clippy lint name, never displayed
    let corner =
        |x: f64, y: f64| crate::viewer::pdf_space_to_canvas(Pos2::new(x as f32, y as f32), page);
    Some(Rect::from_two_pos(
        corner(r.llx, r.lly)?,
        corner(r.urx, r.ury)?,
    ))
}

/// What the region takes, or `None` when there is no region, it is on a page
/// not on screen, or the page's objects cannot be read.
#[must_use]
pub fn take(doc: &OpenDoc, touching: bool) -> Option<Taken> {
    let region = doc.snapshot.filter(|r| r.page == doc.view.page_index)?;
    let rect = canvas_rect(doc, region)?;
    let targets = doc.page_objects()?;
    let provider = Some(&*targets as &dyn CanvasTargetProvider);
    let band = |crossing: bool| {
        let mut taken = SelectionState::default();
        marquee::select_with(
            provider,
            region.page,
            rect,
            crossing,
            Combine::Replace,
            &mut taken,
        );
        taken
    };
    let inside = band(false);
    let touched = band(true);
    let crossing = touched.len().saturating_sub(inside.len());
    Some(if touching {
        Taken {
            selection: touched,
            crossing_kept: 0,
        }
    } else {
        Taken {
            selection: inside,
            crossing_kept: crossing,
        }
    })
}

/// Make what the region takes the canvas selection, so Delete, Copy and the
/// properties panel act on it. Called when a box is drawn, moved or resized.
pub fn fill_selection(ctx: &egui::Context, doc: &mut OpenDoc) {
    let Some(taken) = take(doc, touching(ctx)) else {
        return;
    };
    let objects = taken.selection.len();
    doc.selection = taken.selection;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!(
            "region-selection objects={objects} crossing_kept={}",
            taken.crossing_kept
        )
    });
}

/// The region the outside hatching publishes while Redact outside is open.
pub const HATCH_REGION: &str = "canvas.region.outside_hatch"; // ui-text-exempt: trace region name, never displayed

/// Spacing of the hatch lines, in screen points.
const HATCH_STEP: f32 = 8.0;

/// While Redact outside the region is open, hatch the part of the page that
/// would be marked. A pre-commit preview: nothing is marked until the window's
/// Mark button, and the hatching goes when the window does.
pub fn paint_outside(
    painter: &egui::Painter,
    doc: &OpenDoc,
    page_index: usize,
    map: &crate::canvas::mapping::PageMapping,
) {
    if !crate::dialogs::redact_outside::preview_active(painter.ctx()) {
        return;
    }
    let Some(keep) = crate::canvas::snapshot::screen_rect(doc, page_index, map) else {
        return;
    };
    let page = map.image_rect();
    let ink = egui_shell::theme::Theme::canvas_selection_ink(painter.ctx()).gamma_multiply(0.6);
    let stroke = egui::Stroke::new(1.0, ink);
    let strips = [
        Rect::from_min_max(page.min, Pos2::new(page.max.x, keep.min.y)),
        Rect::from_min_max(Pos2::new(page.min.x, keep.max.y), page.max),
        Rect::from_min_max(
            Pos2::new(page.min.x, keep.min.y),
            Pos2::new(keep.min.x, keep.max.y),
        ),
        Rect::from_min_max(
            Pos2::new(keep.max.x, keep.min.y),
            Pos2::new(page.max.x, keep.max.y),
        ),
    ];
    let span = page.width() + page.height();
    for strip in strips.into_iter().filter(|s| s.is_positive()) {
        let clipped = painter.with_clip_rect(strip.intersect(painter.clip_rect()));
        let mut x = page.min.x - page.height();
        while x < page.max.x {
            clipped.line_segment(
                [
                    Pos2::new(x, page.max.y),
                    Pos2::new(x + span, page.max.y - span),
                ],
                stroke,
            );
            x += HATCH_STEP;
        }
    }
    crate::diag::ui_rect(HATCH_REGION, page);
}
