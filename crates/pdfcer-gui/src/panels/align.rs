//! # `panels::align` — Inkscape's Align and Distribute, as a dock panel.
//!
//! Acts on page content selected at the Object rung on the current page.
//! Boxes are measured in canvas space (what the operator sees, `/Rotate`
//! applied), handed to `pdfcer_gui_base::alignlayout`, and the per-object
//! canvas deltas come back as page-space moves in one
//! [`VectorAction::MoveEach`]. `ALIGN_AND_DISTRIBUTE.md` maps each Inkscape
//! control; `docs/reference/inkscape-align-and-distribute.md` is the spec.

use egui::{Pos2, Ui, Vec2};
use pdfcer_core::page_tree::Page;
use pdfcer_core::vector::{Bounds, Point, VectorObject};
use pdfcer_gui_base::alignlayout::{
    self as layout, Axis, Bx, Edge, Frame, RelativeTo, Spacing, rearrange,
};

use crate::app::actions::{Action, VectorAction};
use crate::app::state::OpenDoc;
use crate::canvas::selection::SelectionLevel;
use crate::text::panels::align as t;

/// The panel's whole extent.
pub const REGION: &str = "align.panel"; // ui-text-exempt: diagnostic region name
/// The horizontal Align row's buttons, left to right.
pub const ALIGN_H_REGIONS: [&str; 6] = [
    "align.h0", "align.h1", "align.h2", "align.h3", "align.h4",
    "align.h5", // ui-text-exempt: diagnostic region names
];
/// The vertical Align row's buttons, top edge first.
pub const ALIGN_V_REGIONS: [&str; 6] = [
    "align.v0", "align.v1", "align.v2", "align.v3", "align.v4",
    "align.v5", // ui-text-exempt: diagnostic region names
];
/// The horizontal Distribute row's buttons.
pub const DISTRIBUTE_H_REGIONS: [&str; 5] = [
    "distribute.h0",
    "distribute.h1",
    "distribute.h2",
    "distribute.h3",
    "distribute.h4", // ui-text-exempt: diagnostic region names
];
/// The vertical Distribute row's buttons.
pub const DISTRIBUTE_V_REGIONS: [&str; 5] = [
    "distribute.v0",
    "distribute.v1",
    "distribute.v2",
    "distribute.v3",
    "distribute.v4", // ui-text-exempt: diagnostic region names
];

/// The Rearrange frame's buttons: Exchange by selection, stacking and
/// clockwise, then Randomize and Unclump.
pub const REARRANGE_REGIONS: [&str; 5] = [
    "rearrange.0",
    "rearrange.1",
    "rearrange.2",
    "rearrange.3",
    "rearrange.4", // ui-text-exempt: diagnostic region names
];
/// The Remove overlaps button.
pub const REMOVE_OVERLAPS_REGION: &str = "overlaps.remove"; // ui-text-exempt: diagnostic region name

/// The operator's settings, kept across documents as Inkscape keeps them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignUi {
    /// *Relative to* with two or more objects selected.
    pub multi: RelativeTo,
    /// *Relative to* with one object selected: Page or Drawing only.
    pub single: RelativeTo,
    /// *Move/align selection as group*.
    pub as_group: bool,
    /// Remove overlaps' horizontal and vertical gaps, in points.
    pub gap_x: f64,
    pub gap_y: f64,
    /// Presses of Randomize so far: each press's seed, so every press lands
    /// somewhere new and a replay of the same presses lands the same way.
    pub randomized: u64,
}

impl Default for AlignUi {
    fn default() -> Self {
        Self {
            multi: RelativeTo::Selection,
            single: RelativeTo::Page,
            as_group: false,
            gap_x: 0.0,
            gap_y: 0.0,
            randomized: 0,
        }
    }
}

impl AlignUi {
    /// The choice in force for `n` selected objects.
    #[must_use]
    pub fn relative(self, n: usize) -> RelativeTo {
        if n == 1 { self.single } else { self.multi }
    }
}

/// One button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Align(Axis, Edge),
    AlignText(Axis),
    Distribute(Axis, Spacing),
    DistributeText(Axis),
    /// Rearrange › Exchange positions, walking the selection in this order.
    Exchange(ExchangeOrder),
    /// Rearrange › Randomize, with this press's seed.
    Randomize(u64),
    /// Rearrange › Unclump.
    Unclump,
    /// Remove overlaps, with the panel's gaps.
    RemoveOverlaps,
}

/// The order *Exchange positions* walks the selection in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeOrder {
    /// The order the objects were selected in.
    Selection,
    /// Paint order, bottom-most first.
    Stacking,
    /// Clockwise around the selection's centre.
    Clockwise,
}

impl ExchangeOrder {
    /// The Rearrange frame's three, left to right.
    pub const ALL: [ExchangeOrder; 3] = [Self::Selection, Self::Stacking, Self::Clockwise];
}

impl Op {
    /// The horizontal (`Axis::X`) or vertical row's Align buttons, in order.
    #[must_use]
    pub const fn align_row(axis: Axis) -> [Op; 6] {
        [
            Op::Align(axis, Edge::MaxToMin),
            Op::Align(axis, Edge::Min),
            Op::Align(axis, Edge::Centre),
            Op::Align(axis, Edge::Max),
            Op::Align(axis, Edge::MinToMax),
            Op::AlignText(axis),
        ]
    }

    /// The row's Distribute buttons, in order.
    #[must_use]
    pub const fn distribute_row(axis: Axis) -> [Op; 5] {
        [
            Op::Distribute(axis, Spacing::Min),
            Op::Distribute(axis, Spacing::Centre),
            Op::Distribute(axis, Spacing::Max),
            Op::Distribute(axis, Spacing::Gaps),
            Op::DistributeText(axis),
        ]
    }

    /// How many selected objects the button needs.
    #[must_use]
    pub const fn needs(self) -> usize {
        match self {
            Op::Align(..) | Op::AlignText(_) => 1,
            Op::Distribute(..)
            | Op::DistributeText(_)
            | Op::Exchange(_)
            | Op::Randomize(_)
            | Op::Unclump
            | Op::RemoveOverlaps => 2,
        }
    }

    /// The trace word and the gesture [`VectorAction::MoveEach`] carries.
    #[must_use]
    pub const fn gesture(self) -> &'static str {
        match self {
            Op::Align(..) | Op::AlignText(_) => "align",
            Op::Distribute(..) | Op::DistributeText(_) => "distribute",
            Op::Exchange(_) | Op::Randomize(_) | Op::Unclump => "rearrange",
            Op::RemoveOverlaps => "remove-overlaps",
        }
    }
}

/// The selection, measured: paint indices in selection order, their canvas
/// boxes, text baseline origins, and the page and drawing frames.
pub struct Measured {
    pub page: usize,
    pub objects: Vec<usize>,
    pub boxes: Vec<Bx>,
    pub anchors: Vec<Option<(f64, f64)>>,
    pub frame: Frame,
}

/// A page-space bounds as a canvas-space box.
fn canvas_box(b: Bounds, page: &Page) -> Option<Bx> {
    let corners = [
        (b.min.x, b.min.y),
        (b.max.x, b.min.y),
        (b.min.x, b.max.y),
        (b.max.x, b.max.y),
    ];
    let mut out: Option<Bx> = None;
    for (x, y) in corners {
        // Canvas space is f32.
        #[allow(clippy::cast_possible_truncation)]
        let p = crate::viewer::pdf_space_to_canvas(Pos2::new(x as f32, y as f32), page)?;
        let c = Bx::new(p.x.into(), p.y.into(), p.x.into(), p.y.into());
        out = Some(out.map_or(c, |u| u.union(c)));
    }
    out
}

/// A text object's baseline origin in canvas space: its first run's text
/// matrix through the object's CTM.
fn text_anchor(object: &VectorObject, page: &Page) -> Option<(f64, f64)> {
    let VectorObject::Text(text) = object else {
        return None;
    };
    let run = text.runs.first()?;
    let origin = text
        .ctm
        .map_point(run.text_matrix.map_point(Point::new(0.0, 0.0)));
    // Canvas space is f32.
    #[allow(clippy::cast_possible_truncation)]
    let p = crate::viewer::pdf_space_to_canvas(Pos2::new(origin.x as f32, origin.y as f32), page)?;
    Some((p.x.into(), p.y.into()))
}

/// Measure what the panel acts on, or `None` when nothing on the current
/// page is selected at the Object rung.
#[must_use]
pub fn measure(doc: &OpenDoc) -> Option<Measured> {
    let page = doc.view.page_index;
    if doc.selection.level() != SelectionLevel::Object {
        return None;
    }
    let order: Vec<usize> = doc
        .selection
        .in_selection_order()
        .into_iter()
        .filter(|e| e.page == page)
        .filter_map(|e| e.object.page_object_index())
        .collect();
    if order.is_empty() {
        return None;
    }
    let sheet = doc.pages.get(page)?;
    let provider = doc.page_objects()?;
    let all = &provider.page_objects().objects;
    // The sheet as the canvas draws it: its device box at scale 1.
    let (w, h, _) = pdfcer_render::page_device_geometry(sheet, 1.0);
    let page_box = Bx::new(0.0, 0.0, f64::from(w), f64::from(h));
    let drawing = all
        .iter()
        .filter_map(|o| canvas_box(o.page_bbox(), sheet))
        .reduce(Bx::union)
        .unwrap_or(page_box);
    let mut objects = Vec::with_capacity(order.len());
    let mut boxes = Vec::with_capacity(order.len());
    let mut anchors = Vec::with_capacity(order.len());
    for i in order {
        let Some(object) = all.get(i) else { continue };
        let Some(b) = canvas_box(object.page_bbox(), sheet) else {
            continue;
        };
        objects.push(i);
        boxes.push(b);
        anchors.push(text_anchor(object, sheet));
    }
    Some(Measured {
        page,
        objects,
        boxes,
        anchors,
        frame: Frame {
            page: page_box,
            drawing,
        },
    })
}

/// One button's canvas deltas, per measured object.
fn deltas(m: &Measured, op: Op, settings: AlignUi) -> Vec<(f64, f64)> {
    let rel = settings.relative(m.boxes.len());
    match op {
        Op::Align(axis, edge) => {
            layout::align(&m.boxes, axis, edge, rel, m.frame, settings.as_group)
        }
        Op::AlignText(axis) => layout::align_text(&m.boxes, &m.anchors, axis, rel, m.frame),
        Op::Distribute(axis, spacing) => layout::distribute(&m.boxes, axis, spacing),
        Op::DistributeText(axis) => layout::distribute_text(&m.anchors, axis),
        Op::Exchange(order) => {
            let order: Vec<usize> = match order {
                ExchangeOrder::Selection => (0..m.boxes.len()).collect(),
                ExchangeOrder::Stacking => rearrange::stacking_order(&m.objects),
                ExchangeOrder::Clockwise => rearrange::clockwise_order(&m.boxes),
            };
            rearrange::exchange(&m.boxes, &order)
        }
        Op::Randomize(seed) => rearrange::randomize(&m.boxes, seed),
        Op::Unclump => rearrange::unclump(&m.boxes),
        Op::RemoveOverlaps => rearrange::remove_overlaps(&m.boxes, settings.gap_x, settings.gap_y),
    }
}

/// The action `ops` ask for together — their deltas summed, so presses on
/// different axes land as one undo step — or `None` when nothing would move.
#[must_use]
pub fn plan(doc: &OpenDoc, m: &Measured, ops: &[Op], settings: AlignUi) -> Option<Action> {
    let first = *ops.first()?;
    let mut sum = vec![(0.0, 0.0); m.boxes.len()];
    for &op in ops {
        for (acc, (dx, dy)) in sum.iter_mut().zip(deltas(m, op, settings)) {
            acc.0 += dx;
            acc.1 += dy;
        }
    }
    let sheet = doc.pages.get(m.page)?;
    // Canvas space is f32.
    #[allow(clippy::cast_possible_truncation)]
    let moves: Vec<(usize, f64, f64)> = m
        .objects
        .iter()
        .zip(sum)
        .filter(|(_, d)| !layout::negligible(*d))
        .filter_map(|(&i, (dx, dy))| {
            let d = crate::canvas::moving::page_delta(Vec2::new(dx as f32, dy as f32), sheet)?;
            Some((i, d.dx, d.dy))
        })
        .collect();
    (!moves.is_empty()).then(|| {
        VectorAction::MoveEach {
            page: m.page,
            moves,
            gesture: first.gesture(),
        }
        .into()
    })
}

/// Run `ops` against the current selection as one step: push the action, or
/// say why nothing moved.
pub fn run(doc: &OpenDoc, ops: &[Op], settings: AlignUi, actions: &mut Vec<Action>) {
    let Some(m) = measure(doc) else {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("align-declined op={ops:?} reason=nothing-selected")
        });
        crate::app::actions::record_note(doc.edit_epoch, t::nothing_selected().to_owned());
        return;
    };
    match plan(doc, &m, ops, settings) {
        Some(action) => actions.push(action),
        None => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "align-declined op={ops:?} reason=nothing-moves n={}",
                    m.objects.len()
                )
            });
            crate::app::actions::record_note(doc.edit_epoch, t::already_aligned().to_owned());
        }
    }
}

/// Draw the panel, scrolling: it is taller than a docked pane usually is.
pub fn body(ui: &mut Ui, doc: &OpenDoc, settings: &mut AlignUi, actions: &mut Vec<Action>) {
    egui::ScrollArea::vertical()
        .id_salt("align-panel") // ui-text-exempt: widget id
        .show(ui, |ui| contents(ui, doc, settings, actions));
}

/// The panel's frames, top to bottom.
fn contents(ui: &mut Ui, doc: &OpenDoc, settings: &mut AlignUi, actions: &mut Vec<Action>) {
    let top = ui.min_rect().min;
    if doc.selection.annot().is_some() || doc.selected_field.is_some() {
        ui.label(t::not_content());
    }
    let measured = measure(doc);
    let n = measured.as_ref().map_or(0, |m| m.objects.len());
    ui.label(if n == 0 {
        t::nothing_selected().to_owned()
    } else {
        t::selected_count(n)
    });

    ui.separator();
    ui.label(t::align_heading());
    ui.checkbox(&mut settings.as_group, t::as_group())
        .on_hover_text(t::as_group_tip());
    ui.horizontal(|ui| {
        ui.label(t::relative_to());
        let single = n == 1;
        let (current, choices): (&mut RelativeTo, &[RelativeTo]) = if single {
            (&mut settings.single, &RelativeTo::SINGLE)
        } else {
            (&mut settings.multi, &RelativeTo::ALL)
        };
        egui::ComboBox::from_id_salt("align.relative") // ui-text-exempt: widget id
            .selected_text(t::relative_choice(*current))
            .show_ui(ui, |ui| {
                for &choice in choices {
                    ui.selectable_value(current, choice, t::relative_choice(choice));
                }
            });
    });
    let mut pressed = None;
    for (axis, row_label, regions) in [
        (Axis::X, t::horizontal(), &ALIGN_H_REGIONS),
        (Axis::Y, t::vertical(), &ALIGN_V_REGIONS),
    ] {
        ui.label(egui::RichText::new(row_label).small().weak());
        ui.horizontal_wrapped(|ui| {
            for (index, op) in Op::align_row(axis).into_iter().enumerate() {
                let (label, tip) = t::align_button(axis == Axis::X, index);
                if button(ui, n >= op.needs(), label, tip, regions[index]) {
                    pressed = Some(op);
                }
            }
        });
    }

    ui.separator();
    ui.label(t::distribute_heading());
    for (axis, row_label, regions) in [
        (Axis::X, t::horizontal(), &DISTRIBUTE_H_REGIONS),
        (Axis::Y, t::vertical(), &DISTRIBUTE_V_REGIONS),
    ] {
        ui.label(egui::RichText::new(row_label).small().weak());
        ui.horizontal_wrapped(|ui| {
            for (index, op) in Op::distribute_row(axis).into_iter().enumerate() {
                let (label, tip) = t::distribute_button(axis == Axis::X, index);
                if button(ui, n >= op.needs(), label, tip, regions[index]) {
                    pressed = Some(op);
                }
            }
        });
    }
    ui.separator();
    ui.label(t::rearrange_heading());
    ui.horizontal_wrapped(|ui| {
        for (index, order) in ExchangeOrder::ALL.into_iter().enumerate() {
            let (label, tip) = t::rearrange_button(index);
            if button(ui, n >= 2, label, tip, REARRANGE_REGIONS[index]) {
                pressed = Some(Op::Exchange(order));
            }
        }
        let (label, tip) = t::rearrange_button(3);
        if button(ui, n >= 2, label, tip, REARRANGE_REGIONS[3]) {
            settings.randomized = settings.randomized.wrapping_add(1);
            pressed = Some(Op::Randomize(settings.randomized));
        }
        let (label, tip) = t::rearrange_button(4);
        if button(ui, n >= 2, label, tip, REARRANGE_REGIONS[4]) {
            pressed = Some(Op::Unclump);
        }
    });

    ui.separator();
    ui.label(t::overlaps_heading());
    ui.horizontal_wrapped(|ui| {
        ui.label(t::gap_h());
        ui.add(
            egui::DragValue::new(&mut settings.gap_x)
                .range(0.0..=1000.0)
                .suffix(t::pt_suffix()),
        );
        ui.label(t::gap_v());
        ui.add(
            egui::DragValue::new(&mut settings.gap_y)
                .range(0.0..=1000.0)
                .suffix(t::pt_suffix()),
        );
        let (label, tip) = t::remove_overlaps_button();
        if button(ui, n >= 2, label, tip, REMOVE_OVERLAPS_REGION) {
            pressed = Some(Op::RemoveOverlaps);
        }
    });

    crate::diag::ui_rect_visible(
        REGION,
        egui::Rect::from_min_max(top, ui.min_rect().max),
        ui.clip_rect(),
    );

    if let Some(op) = pressed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "align-pressed op={op:?} n={n} relative={:?} group={}",
                settings.relative(n),
                settings.as_group
            )
        });
        run(doc, &[op], *settings, actions);
    }
}

/// One button, greyed with a reason when too few objects are selected.
fn button(ui: &mut Ui, enabled: bool, label: &str, tip: &str, region: &'static str) -> bool {
    let response = ui
        .add_enabled(enabled, egui::Button::new(label))
        .on_hover_text(tip)
        .on_disabled_hover_text(t::needs_two());
    crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
    response.clicked()
}
