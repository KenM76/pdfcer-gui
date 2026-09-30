//! # `panels::align::arrange` — the Grid and Circular tabs.
//!
//! Both measure the selection exactly as the Align tab does (canvas boxes,
//! selection order) and hand it to `alignlayout::arrange`. Grid and a
//! Circular arrange without *Rotate objects* commit as one
//! [`VectorAction::MoveEach`]; with *Rotate objects* each object also turns,
//! so the commit is one [`VectorAction::TransformEach`] whose per-object
//! page matrix is the canvas placement conjugated by the page→canvas map.

use egui::{Pos2, Ui};
use pdfcer_core::page_tree::Page;
use pdfcer_core::vector::{Matrix, Point, Segment, VectorObject};
use pdfcer_gui_base::alignlayout::arrange::{
    self, CircleAnchor, Ellipse, GridParams, GridSpacing, Placement,
};

use super::{Measured, labelled_button, measure, moves_action};
use crate::app::actions::{Action, VectorAction};
use crate::app::state::OpenDoc;
use crate::icons::Icon;
use crate::text::panels::align as t;

/// The Grid tab's Arrange button.
pub const GRID_ARRANGE_REGION: &str = "grid.arrange"; // ui-text-exempt: diagnostic region name
/// The Circular tab's Arrange button.
pub const CIRCLE_ARRANGE_REGION: &str = "circular.arrange"; // ui-text-exempt: diagnostic region name
/// The Circular tab's ellipse sources: Parameterized, first, last selected.
pub const CIRCLE_SOURCE_REGIONS: [&str; 3] = [
    "circular.source.0",
    "circular.source.1",
    "circular.source.2", // ui-text-exempt: diagnostic region names
];
/// The Circular tab's *Rotate objects* toggle.
pub const CIRCLE_ROTATE_REGION: &str = "circular.rotate"; // ui-text-exempt: diagnostic region name

/// The Grid tab's settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridUi {
    /// Requested rows; 0 until the operator sets one, meaning `⌈√n⌉`.
    pub rows: usize,
    pub equal_height: bool,
    pub equal_width: bool,
    /// `(column, row)` of the anchor in each cell, `0..=2` each.
    pub anchor: (u8, u8),
    /// Fit into the selection box, or use `gap_x`/`gap_y`.
    pub fit: bool,
    /// Column and row gaps in points; Inkscape's default is 15 px each.
    pub gap_x: f64,
    pub gap_y: f64,
}

impl Default for GridUi {
    fn default() -> Self {
        Self {
            rows: 0,
            equal_height: false,
            equal_width: false,
            anchor: (1, 1),
            fit: false,
            gap_x: 15.0,
            gap_y: 15.0,
        }
    }
}

/// Where the Circular tab's ellipse comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleSource {
    Parameterized,
    First,
    Last,
}

impl CircleSource {
    const ALL: [Self; 3] = [Self::Parameterized, Self::First, Self::Last];
}

/// The Circular tab's settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleUi {
    pub source: CircleSource,
    /// The Parameterized ellipse, in canvas points (sheet top-left, y down).
    pub ellipse: Ellipse,
    pub anchor: CircleAnchor,
    pub rotate: bool,
}

impl Default for CircleUi {
    fn default() -> Self {
        Self {
            source: CircleSource::First,
            ellipse: Ellipse::DEFAULT,
            anchor: CircleAnchor::Centre,
            rotate: false,
        }
    }
}

/// The grid parameters in force for `n` objects.
fn grid_params(grid: GridUi, n: usize) -> GridParams {
    let rows = if grid.rows == 0 {
        arrange::default_rows_cols(n).0
    } else {
        grid.rows
    };
    let (rows, cols) = arrange::settle_rows_cols(n, rows, 0);
    GridParams {
        rows,
        cols,
        equal_height: grid.equal_height,
        equal_width: grid.equal_width,
        anchor: grid.anchor,
        spacing: if grid.fit {
            GridSpacing::Fit
        } else {
            GridSpacing::Set {
                x: grid.gap_x,
                y: grid.gap_y,
            }
        },
    }
}

/// Grid's action, or `None` when nothing would move.
#[must_use]
pub fn grid_plan(doc: &OpenDoc, m: &Measured, grid: GridUi) -> Option<Action> {
    let deltas = arrange::grid(&m.boxes, &grid_params(grid, m.boxes.len()));
    moves_action(doc, m, deltas, "grid")
}

/// Why a Circular arrange cannot run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleRefusal {
    /// The reference object is not a full axis-aligned ellipse.
    NotAnEllipse,
    /// Nothing would move.
    NothingMoves,
}

/// Circular's action.
///
/// # Errors
/// [`CircleRefusal`] when the reference object is not an ellipse, or when
/// every placement is negligible.
pub fn circular_plan(
    doc: &OpenDoc,
    m: &Measured,
    circle: CircleUi,
) -> Result<Action, CircleRefusal> {
    let sheet = doc.pages.get(m.page).ok_or(CircleRefusal::NothingMoves)?;
    let (ellipse, skip) = match circle.source {
        CircleSource::Parameterized => (circle.ellipse, None),
        CircleSource::First | CircleSource::Last => {
            let pos = if circle.source == CircleSource::First {
                0
            } else {
                m.objects.len().saturating_sub(1)
            };
            let provider = doc.page_objects().ok_or(CircleRefusal::NotAnEllipse)?;
            let object = m
                .objects
                .get(pos)
                .and_then(|&i| provider.page_objects().objects.get(i))
                .ok_or(CircleRefusal::NotAnEllipse)?;
            let points = ellipse_chain(object, sheet).ok_or(CircleRefusal::NotAnEllipse)?;
            let found = arrange::ellipse_from_points(&points).ok_or(CircleRefusal::NotAnEllipse)?;
            (found, Some(pos))
        }
    };
    let placements = arrange::circular(&m.boxes, ellipse, circle.anchor, circle.rotate, skip);
    if !circle.rotate {
        let deltas = placements.iter().map(|p| (p.dx, p.dy)).collect();
        return moves_action(doc, m, deltas, "circular").ok_or(CircleRefusal::NothingMoves);
    }
    let to_canvas = canvas_map(sheet).ok_or(CircleRefusal::NothingMoves)?;
    let to_page = to_canvas.inverse().ok_or(CircleRefusal::NothingMoves)?;
    let transforms: Vec<(usize, Matrix)> = m
        .objects
        .iter()
        .zip(&m.boxes)
        .zip(placements)
        .filter(|(_, p)| p.dx.hypot(p.dy) > 1e-6 || p.rotate_rad.abs() > 1e-9)
        .map(|((&i, b), p)| {
            let canvas = placement_matrix(p, b.x0, b.y0, b.x1, b.y1);
            (i, to_canvas.post_concat(canvas).post_concat(to_page))
        })
        .collect();
    if transforms.is_empty() {
        return Err(CircleRefusal::NothingMoves);
    }
    Ok(VectorAction::TransformEach {
        page: m.page,
        transforms,
        gesture: "circular",
    }
    .into())
}

/// A [`Placement`] as a canvas-space matrix: translate, then rotate about
/// the translated box centre.
fn placement_matrix(p: Placement, x0: f64, y0: f64, x1: f64, y1: f64) -> Matrix {
    let centre = Point::new(f64::midpoint(x0, x1) + p.dx, f64::midpoint(y0, y1) + p.dy);
    Matrix::translate(p.dx, p.dy).post_concat(Matrix::rotate(p.rotate_rad).about(centre))
}

/// The page→canvas map as a matrix, sampled from the viewer's own mapping
/// so `/Rotate` and the crop origin are whatever the canvas uses.
fn canvas_map(sheet: &Page) -> Option<Matrix> {
    const SPAN: f32 = 1000.0;
    let at = |x: f32, y: f32| crate::viewer::pdf_space_to_canvas(Pos2::new(x, y), sheet);
    let (o, ex, ey) = (at(0.0, 0.0)?, at(SPAN, 0.0)?, at(0.0, SPAN)?);
    let s = f64::from(SPAN);
    Some(Matrix::new(
        f64::from(ex.x - o.x) / s,
        f64::from(ex.y - o.y) / s,
        f64::from(ey.x - o.x) / s,
        f64::from(ey.y - o.y) / s,
        f64::from(o.x),
        f64::from(o.y),
    ))
}

/// A single all-cubic subpath as the `[a0, c1, c2, a1, …]` chain in canvas
/// space, or `None` for anything else.
fn ellipse_chain(object: &VectorObject, sheet: &Page) -> Option<Vec<(f64, f64)>> {
    let VectorObject::Path(path) = object else {
        return None;
    };
    let subpaths = path.page_subpaths();
    let [subpath] = subpaths.as_slice() else {
        return None;
    };
    let canvas = |p: Point| {
        // Canvas space is f32.
        #[allow(clippy::cast_possible_truncation)]
        let c = crate::viewer::pdf_space_to_canvas(Pos2::new(p.x as f32, p.y as f32), sheet)?;
        Some((f64::from(c.x), f64::from(c.y)))
    };
    let mut out = vec![canvas(subpath.start)?];
    for segment in &subpath.segments {
        let Segment::Cubic { c1, c2, to } = *segment else {
            return None;
        };
        out.extend([canvas(c1)?, canvas(c2)?, canvas(to)?]);
    }
    Some(out)
}

/// One of the three anchor combos: `h` picks the horizontal wording.
fn anchor_combo(ui: &mut Ui, salt: &str, value: &mut u8, h: bool) {
    let name = |i: u8| if h { t::anchor_h(i) } else { t::anchor_v(i) };
    egui::ComboBox::from_id_salt(salt)
        .selected_text(name(*value))
        .show_ui(ui, |ui| {
            for i in 0..3 {
                ui.selectable_value(value, i, name(i));
            }
        });
}

/// Draw the Grid tab.
pub fn grid_tab(
    ui: &mut Ui,
    doc: &OpenDoc,
    grid: &mut GridUi,
    n: usize,
    actions: &mut Vec<Action>,
) {
    let count = n.max(1);
    let params = grid_params(*grid, count);
    let (mut rows, mut cols) = (params.rows, params.cols);
    egui::Grid::new("align.grid.fields") // ui-text-exempt: widget id
        .num_columns(2)
        .show(ui, |ui| {
            ui.label(t::rows());
            if ui
                .add(egui::DragValue::new(&mut rows).range(1..=count))
                .changed()
            {
                grid.rows = rows;
            }
            ui.end_row();
            ui.label(t::columns());
            if ui
                .add(egui::DragValue::new(&mut cols).range(1..=count))
                .on_hover_text(t::columns_tip())
                .changed()
            {
                grid.rows = arrange::rows_for_cols(count, cols);
            }
            ui.end_row();
        });
    ui.checkbox(&mut grid.equal_height, t::equal_height());
    ui.checkbox(&mut grid.equal_width, t::equal_width());
    ui.horizontal(|ui| {
        ui.label(t::anchor());
        anchor_combo(ui, "align.grid.anchor.h", &mut grid.anchor.0, true); // ui-text-exempt: widget id
        anchor_combo(ui, "align.grid.anchor.v", &mut grid.anchor.1, false); // ui-text-exempt: widget id
    });
    ui.radio_value(&mut grid.fit, true, t::fit_spacing());
    ui.horizontal_wrapped(|ui| {
        ui.radio_value(&mut grid.fit, false, t::set_spacing());
        ui.add_enabled_ui(!grid.fit, |ui| {
            ui.label(t::gap_h());
            ui.add(egui::DragValue::new(&mut grid.gap_x).suffix(t::pt_suffix()));
            ui.label(t::gap_v());
            ui.add(egui::DragValue::new(&mut grid.gap_y).suffix(t::pt_suffix()));
        });
    });
    let (label, tip) = t::arrange_button(false);
    if labelled_button(
        ui,
        n >= 2,
        Icon::ArrangeGrid,
        label,
        tip,
        GRID_ARRANGE_REGION,
    ) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("align-pressed op=Grid n={n} rows={rows} cols={cols} grid={grid:?}")
        });
        let Some(m) = measure(doc) else { return };
        match grid_plan(doc, &m, *grid) {
            Some(action) => actions.push(action),
            None => decline(doc, "Grid", "nothing-moves", t::already_aligned()),
        }
    }
}

/// Draw the Circular tab.
pub fn circular_tab(
    ui: &mut Ui,
    doc: &OpenDoc,
    circle: &mut CircleUi,
    n: usize,
    actions: &mut Vec<Action>,
) {
    for (index, source) in CircleSource::ALL.into_iter().enumerate() {
        let response = ui.radio_value(&mut circle.source, source, t::circle_source(index));
        crate::diag::ui_rect_visible(CIRCLE_SOURCE_REGIONS[index], response.rect, ui.clip_rect());
    }
    ui.add_enabled_ui(circle.source == CircleSource::Parameterized, |ui| {
        let e = &mut circle.ellipse;
        egui::Grid::new("align.circle.fields") // ui-text-exempt: widget id
            .num_columns(3)
            .show(ui, |ui| {
                for (label, a, b, suffix) in [
                    (t::centre_xy(), &mut e.cx, &mut e.cy, t::pt_suffix()),
                    (t::radius_xy(), &mut e.rx, &mut e.ry, t::pt_suffix()),
                    (
                        t::angles(),
                        &mut e.start_deg,
                        &mut e.end_deg,
                        t::deg_suffix(),
                    ),
                ] {
                    ui.label(label);
                    ui.add(egui::DragValue::new(a).suffix(suffix));
                    ui.add(egui::DragValue::new(b).suffix(suffix));
                    ui.end_row();
                }
            });
        ui.label(egui::RichText::new(t::circle_coords_note()).small().weak());
    });
    ui.label(t::anchor());
    ui.radio_value(&mut circle.anchor, CircleAnchor::Centre, t::anchor_centre());
    let (mut h, mut v) = match circle.anchor {
        CircleAnchor::BoxPoint(h, v) => (h, v),
        CircleAnchor::Centre => (1, 1),
    };
    ui.horizontal(|ui| {
        let on_box = matches!(circle.anchor, CircleAnchor::BoxPoint(..));
        if ui.radio(on_box, t::anchor_box()).clicked() {
            circle.anchor = CircleAnchor::BoxPoint(h, v);
        }
        ui.add_enabled_ui(on_box, |ui| {
            anchor_combo(ui, "align.circle.anchor.h", &mut h, true); // ui-text-exempt: widget id
            anchor_combo(ui, "align.circle.anchor.v", &mut v, false); // ui-text-exempt: widget id
        });
        if on_box {
            circle.anchor = CircleAnchor::BoxPoint(h, v);
        }
    });
    let rotate = ui.checkbox(&mut circle.rotate, t::rotate_objects());
    crate::diag::ui_rect_visible(CIRCLE_ROTATE_REGION, rotate.rect, ui.clip_rect());
    let needs = if circle.source == CircleSource::Parameterized {
        1
    } else {
        2
    };
    let (label, tip) = t::arrange_button(true);
    if labelled_button(
        ui,
        n >= needs,
        Icon::ArrangeCircular,
        label,
        tip,
        CIRCLE_ARRANGE_REGION,
    ) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("align-pressed op=Circular n={n} circle={circle:?}")
        });
        let Some(m) = measure(doc) else { return };
        match circular_plan(doc, &m, *circle) {
            Ok(action) => actions.push(action),
            Err(CircleRefusal::NotAnEllipse) => decline(
                doc,
                "Circular",
                "not-an-ellipse",
                t::not_an_ellipse(circle.source == CircleSource::First),
            ),
            Err(CircleRefusal::NothingMoves) => {
                decline(doc, "Circular", "nothing-moves", t::already_aligned());
            }
        }
    }
}

/// Trace and say off-canvas why a press did nothing.
fn decline(doc: &OpenDoc, op: &str, reason: &str, note: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("align-declined op={op} reason={reason}")
    });
    crate::app::actions::record_note(doc.edit_epoch, note.to_owned());
}
