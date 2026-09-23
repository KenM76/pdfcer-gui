//! # `dialogs::print::poster` — poster mode over a planned job
//!
//! Poster mode transforms the planned [`Job`]. The job is planned as an
//! ordinary one at the tile scale, then every page
//! [`pdfcer_gui_base::poster::impose`] tiles is replaced by one [`PagePlan`] per
//! sheet, carrying its [`Tile`]. Everything downstream walks sheets, so the
//! stepper, the sheet count, the commit and the spool need no second path.
//!
//! # Contract
//!
//! - A tile's [`Placement`] is **the whole page's**: drawn at it and clipped to
//!   `content_pt`, the page shows exactly that tile's part. The preview
//!   therefore draws a tile with its ordinary page path plus one clip, and a
//!   tile never counts as clipped.
//! - Its `render_scale` is `dpi / 72 × tile_scale`, the density an ordinary
//!   page at that scale gets.
//! - The operator's page positions (O208) are not applied to a poster: a
//!   tile's position is the imposition's, and dragging a tile would tear the
//!   poster.
//! - A refusal from the engine (more tiles than its ceiling) leaves no job,
//!   so nothing can print, and the refusal is the Pages tab's caption.

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Ui};
use pdfcer_gui_base::poster::{self as base, Imposed, Options, Tile};

use super::PrintDialog;
use super::spooler::{Job, PagePlan, Placement};
use crate::app::prefs::printing::{POSTER_OVERLAP_MM_RANGE, POSTER_PERCENT_RANGE, PosterPrefs};
use crate::text::print as t;

/// The published region of the Size/Poster choice's Poster half.
pub(super) const REGION_POSTER_MODE: &str = "print.poster.mode";
/// The published region of the tile-scale field.
pub(super) const REGION_POSTER_SCALE: &str = "print.poster.scale";

/// The operator's settings in engine units.
fn options(prefs: PosterPrefs) -> Options {
    Options {
        tile_scale: f64::from(prefs.tile_percent) / 100.0,
        overlap_pt: crate::units::points_from_mm(prefs.overlap_mm),
        cut_marks: prefs.cut_marks,
        labels: prefs.labels,
        large_only: prefs.large_only,
    }
}

/// The scale the job is planned at while poster mode is on.
pub(super) fn plan_scale(prefs: PosterPrefs) -> f64 {
    f64::from(prefs.tile_percent) / 100.0
}

/// One sheet of a tiled page. See the module contract for the placement.
fn sheet(index: usize, tile: Tile, dpi: u32) -> PagePlan {
    let scale = tile.tile_scale;
    PagePlan {
        index,
        placement: Placement {
            scale,
            offset_x_pt: tile.content_pt.x - tile.source_pt.x * scale,
            offset_y_pt: tile.content_pt.y - tile.source_pt.y * scale,
            clipped: false,
        },
        render_scale: f64::from(dpi) / 72.0 * scale,
        tile: Some(tile),
    }
}

/// Tile `job` when poster mode is on, and set [`PrintDialog::poster_caption`]
/// for the sheet on the preview. Off, the job passes through untouched.
pub(super) fn apply(
    dialog: &mut PrintDialog,
    job: Option<Job>,
    page_sizes: &[(f64, f64)],
) -> Option<Job> {
    dialog.poster_caption = None;
    if !dialog.poster.on {
        return job;
    }
    let mut job = job?;
    let options = options(dialog.poster);
    let mut sheets = Vec::with_capacity(job.plans.len());
    for plan in &job.plans {
        let Some(&page) = page_sizes.get(plan.index) else {
            continue;
        };
        match base::impose(job.device.printable_pt, page, options) {
            Ok(Imposed::Untiled) => sheets.push(*plan),
            Ok(Imposed::Tiled { tiles, .. }) => sheets.extend(
                tiles
                    .into_iter()
                    .map(|tile| sheet(plan.index, tile, job.resolution.dpi)),
            ),
            Err(error) => {
                let reason = error.to_string();
                crate::diag::trace_changed("print-poster-refused", || {
                    // ui-text-exempt: diagnostic trace, never displayed in the UI
                    format!("print-poster-refused page={} reason={reason}", plan.index)
                });
                dialog.poster_caption = Some(t::poster_refused(&reason));
                return None;
            }
        }
    }
    job.plans = sheets;
    let shown = dialog.preview_page.min(job.plans.len().saturating_sub(1));
    dialog.poster_caption = job.plans.get(shown).map(|plan| match plan.tile {
        Some(tile) => {
            let (w, h) = page_sizes[plan.index];
            let mm = crate::units::whole_mm_from_points;
            t::poster_grid(
                tile.rows,
                tile.columns,
                (mm(w * tile.tile_scale), mm(h * tile.tile_scale)),
            )
        }
        None => t::poster_untiled().to_owned(),
    });
    Some(job)
}

/// The trace fields for `print-plan`: sheets that are tiles, and the first
/// tile's grid.
pub(super) fn trace_fields(job: Option<&Job>) -> String {
    let plans = job.map_or(&[][..], |j| &j.plans[..]);
    let tiles = plans.iter().filter(|p| p.tile.is_some()).count();
    let grid = plans
        .iter()
        .find_map(|p| p.tile)
        .map_or((0, 0), |tile| (tile.rows, tile.columns));
    format!(
        "poster_tiles={tiles} poster_rows={} poster_cols={}", // ui-text-exempt: diagnostic trace fields
        grid.0, grid.1
    )
}

/// The Size/Poster choice, and in Poster mode its controls in place of the
/// scale radios. Returns whether Size mode is on, so the caller draws those.
pub(super) fn mode_row(ui: &mut Ui, dialog: &mut PrintDialog) -> bool {
    ui.horizontal_wrapped(|ui| {
        ui.label(t::sizing_heading())
            .on_hover_text(t::sizing_tooltip());
        if ui.radio(!dialog.poster.on, t::sizing_mode_size()).clicked() {
            dialog.poster.on = false;
        }
        let poster = ui
            .radio(dialog.poster.on, t::sizing_mode_poster())
            .on_hover_text(t::sizing_mode_poster_tooltip());
        crate::diag::ui_rect(REGION_POSTER_MODE, poster.rect);
        if poster.clicked() {
            dialog.poster.on = true;
        }
    });
    if !dialog.poster.on {
        return true;
    }
    let prefs = &mut dialog.poster;
    ui.horizontal_wrapped(|ui| {
        ui.label(t::poster_tile_scale());
        let scale = ui
            .add(
                egui::DragValue::new(&mut prefs.tile_percent)
                    .range(POSTER_PERCENT_RANGE)
                    .suffix(t::percent_suffix()),
            )
            .on_hover_text(t::poster_tile_scale_tooltip());
        crate::diag::ui_rect(REGION_POSTER_SCALE, scale.rect);
        ui.label(t::poster_overlap());
        ui.add(
            egui::DragValue::new(&mut prefs.overlap_mm)
                .range(POSTER_OVERLAP_MM_RANGE)
                .speed(0.5)
                .max_decimals(1)
                .suffix(t::mm_suffix()),
        )
        .on_hover_text(t::poster_overlap_tooltip());
    });
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut prefs.cut_marks, t::poster_cut_marks())
            .on_hover_text(t::poster_cut_marks_tooltip());
        ui.checkbox(&mut prefs.labels, t::poster_labels())
            .on_hover_text(t::poster_labels_tooltip());
    });
    ui.checkbox(&mut prefs.large_only, t::poster_large_only())
        .on_hover_text(t::poster_large_only_tooltip());
    if let Some(caption) = &dialog.poster_caption {
        ui.label(egui::RichText::new(caption).small());
    }
    false
}

/// Where a sheet-space rectangle is on screen: `printable` is the printable
/// area's screen rect and `scale` screen points per paper point.
pub(super) fn on_screen(printable: Rect, r: pdfcer_print::imposition::Rect, scale: f32) -> Rect {
    let at = |x: f64, y: f64| {
        Pos2::new(
            printable.min.x + x as f32 * scale,
            printable.min.y + y as f32 * scale,
        )
    };
    Rect::from_min_max(at(r.x, r.y), at(r.right(), r.bottom()))
}

/// Draw a tile's cut marks and label in its band, as the sheet will carry
/// them. The label is in egui's monospace face, not the print's FoxitFixed:
/// same words, same place, a different typeface.
pub(super) fn paint_band(painter: &Painter, printable: Rect, tile: &Tile, scale: f32, doc: &str) {
    // NOT A THEME COLOUR: the marks and label are ink on the sheet, drawn as
    // `compose` prints them, so restyling the application must not restyle them.
    let ink = Color32::BLACK;
    let at = |x: f64, y: f64| {
        Pos2::new(
            printable.min.x + x as f32 * scale,
            printable.min.y + y as f32 * scale,
        )
    };
    if tile.cut_marks {
        let stroke = Stroke::new(1.0, ink);
        for (x0, y0, x1, y1) in tile.mark_segments() {
            painter.line_segment([at(x0, y0), at(x1, y1)], stroke);
        }
    }
    if let Some(r) = tile.label_rect {
        painter.text(
            at(r.x, r.y),
            Align2::LEFT_TOP,
            tile.label(doc),
            FontId::monospace(r.height as f32 * scale),
            ink,
        );
    }
}

/// The document's file name, for the label.
pub(super) fn document_name(doc: &crate::app::state::OpenDoc) -> String {
    doc.path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile() -> Tile {
        use pdfcer_print::imposition::Rect as R;
        Tile {
            row: 1,
            column: 2,
            rows: 3,
            columns: 4,
            source_pt: R {
                x: 100.0,
                y: 50.0,
                width: 200.0,
                height: 300.0,
            },
            content_pt: R {
                x: 18.0,
                y: 18.0,
                width: 400.0,
                height: 600.0,
            },
            trim_pt: R {
                x: 18.0,
                y: 18.0,
                width: 400.0,
                height: 600.0,
            },
            tile_scale: 2.0,
            band_pt: (18.0, 18.0),
            cut_marks: true,
            labels: true,
            marks: [None; 2],
            label_rect: None,
        }
    }

    /// Drawn at the sheet's placement, the tile's source corner lands on its
    /// content corner — which is what lets the preview draw a tile as a page.
    #[test]
    fn a_tiles_placement_puts_its_source_on_its_content() {
        let tile = tile();
        let plan = sheet(7, tile, 300);
        let p = plan.placement;
        assert_eq!(plan.index, 7);
        assert!(!p.clipped);
        assert!((tile.source_pt.x * p.scale + p.offset_x_pt - tile.content_pt.x).abs() < 1e-9);
        assert!((tile.source_pt.y * p.scale + p.offset_y_pt - tile.content_pt.y).abs() < 1e-9);
        assert!((plan.render_scale - 300.0 / 72.0 * 2.0).abs() < 1e-9);
    }

    /// The settings reach the engine in its units.
    #[test]
    fn the_prefs_convert_to_engine_units() {
        let o = options(PosterPrefs {
            on: true,
            tile_percent: 250,
            overlap_mm: 10.0,
            cut_marks: true,
            labels: false,
            large_only: true,
        });
        assert!((o.tile_scale - 2.5).abs() < 1e-12);
        assert!((o.overlap_pt - pdfcer_gui_base::units::points_from_mm(10.0)).abs() < 1e-9);
        assert!(o.cut_marks && !o.labels && o.large_only);
    }
}
