//! # `panels::align` — Inkscape's Align and Distribute, as a dock panel.
//!
//! Acts on page content selected at the Object rung on the current page.
//! Boxes are measured in canvas space (what the operator sees, `/Rotate`
//! applied), handed to `pdfcer_gui_base::alignlayout`, and the per-object
//! canvas deltas come back as page-space moves in one
//! [`VectorAction::MoveEach`] — or, when every selected item is a part of
//! one placed drawing, [`VectorAction::MoveEachInForm`]. Circular with
//! *Rotate objects* on parts is declined: no per-part transform verb (G180).
//! `ALIGN_AND_DISTRIBUTE.md` maps each Inkscape
//! control; `docs/reference/inkscape-align-and-distribute.md` is the spec.

use egui::{Pos2, Ui, Vec2};
use pdfcer_core::page_tree::Page;
use pdfcer_core::vector::{Bounds, PageObjects, Point, VectorObject};
use pdfcer_gui_base::alignlayout::{
    self as layout, Axis, Bx, Edge, Frame, RelativeTo, Spacing, rearrange,
};

use crate::app::actions::{Action, VectorAction};
use crate::app::state::OpenDoc;
use crate::canvas::selection::SelectionLevel;
use crate::icons::Icon;
use crate::text::panels::align as t;

mod arrange;
mod handles;
mod leftout;
/// The *On-canvas alignment* toggle.
pub const ON_CANVAS_REGION: &str = "align.on_canvas"; // ui-text-exempt: diagnostic region name
mod nodes;
pub use arrange::{CircleSource, CircleUi, GridUi};
pub use handles::{HANDLE_REGIONS, publish_box, show as show_handles};
pub use leftout::LeftOut;
pub use nodes::NODE_REGIONS;

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
/// The tab strip: Align, Grid, Circular.
pub const TAB_REGIONS: [&str; 3] = [
    "align.tab.align",
    "align.tab.grid",
    "align.tab.circular", // ui-text-exempt: diagnostic region names
];
pub use arrange::{
    CIRCLE_ARRANGE_REGION, CIRCLE_ROTATE_REGION, CIRCLE_SOURCE_REGIONS, GRID_ARRANGE_REGION,
};

/// The panel's three tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Align,
    Grid,
    Circular,
}

impl Tab {
    const ALL: [Tab; 3] = [Tab::Align, Tab::Grid, Tab::Circular];
}

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
    /// The tab showing.
    pub tab: Tab,
    pub grid: GridUi,
    pub circle: CircleUi,
    /// Node mode's *Relative to*.
    pub node_rel: rearrange::NodeRelative,
    /// *On-canvas alignment*: the nine handles inside the selection box.
    pub on_canvas: bool,
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
            tab: Tab::Align,
            grid: GridUi::default(),
            circle: CircleUi::default(),
            node_rel: rearrange::NodeRelative::default(),
            on_canvas: false,
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
    /// Paint-order indices, or leaf indices when [`Self::in_form`].
    pub objects: Vec<usize>,
    /// The selection is parts of a placed drawing and no page object.
    pub in_form: bool,
    /// Parts of placed drawings selected beside page objects (never set when
    /// [`Self::in_form`]). Their boxes follow the objects' in [`Self::boxes`].
    pub leaves: Vec<usize>,
    pub boxes: Vec<Bx>,
    pub anchors: Vec<Option<(f64, f64)>>,
    pub frame: Frame,
    /// What the selection holds that this measure does not move.
    pub left_out: LeftOut,
    /// Selected form-field widgets on the page. Their boxes follow the
    /// leaves' in [`Self::boxes`], with no text anchor.
    pub widgets: Vec<pdfcer_gui_base::docidentity::SelectedField>,
    /// Selected markups, links and ce dimensions on the page. Their boxes
    /// follow the widgets' in [`Self::boxes`], with no text anchor.
    pub annots: Vec<pdfcer_gui_base::annotselection::AnnotTarget>,
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

/// Object `i` of the selection's own list: a part's object when `in_form`.
#[must_use]
pub fn object_in(objects: &PageObjects, in_form: bool, i: usize) -> Option<&VectorObject> {
    if in_form {
        objects.leaves.get(i).map(|leaf| &leaf.object)
    } else {
        objects.objects.get(i)
    }
}

/// Measure what the panel acts on, or `None` when nothing on the current
/// page is selected at the Object rung and no form field or annotation is
/// selected there.
/// A selection of parts of a placed drawing and no page object is measured
/// from the parts, whose geometry is already in page space.
#[must_use]
pub fn measure(ctx: &egui::Context, doc: &OpenDoc) -> Option<Measured> {
    let page = doc.view.page_index;
    let picked: Vec<_> = if doc.selection.level() == SelectionLevel::Object {
        doc.selection
            .in_selection_order()
            .into_iter()
            .filter(|e| e.page == page)
            .map(|e| e.object)
            .collect()
    } else {
        Vec::new()
    };
    let in_form = !picked.is_empty() && picked.iter().all(|t| t.is_leaf());
    let order: Vec<usize> = picked
        .iter()
        .filter_map(|t| {
            if in_form {
                t.leaf_index()
            } else {
                t.page_object_index()
            }
        })
        .collect();
    let fields = field_boxes(ctx, doc, page);
    let marks = annot_boxes(doc, page);
    let provider = doc.page_objects()?;
    let (beside, carried) = if in_form {
        (Vec::new(), 0)
    } else {
        let leaf_order: Vec<usize> = picked.iter().filter_map(|t| t.leaf_index()).collect();
        leftout::uncarried(provider.page_objects(), &order, leaf_order)
    };
    if order.is_empty() && beside.is_empty() && fields.is_empty() && marks.is_empty() {
        return None;
    }
    let sheet = doc.pages.get(page)?;
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
        let Some(object) = object_in(provider.page_objects(), in_form, i) else {
            continue;
        };
        let Some(b) = canvas_box(object.page_bbox(), sheet) else {
            continue;
        };
        objects.push(i);
        boxes.push(b);
        anchors.push(text_anchor(object, sheet));
    }
    let mut leaves = Vec::with_capacity(beside.len());
    for i in beside {
        let Some(object) = object_in(provider.page_objects(), true, i) else {
            continue;
        };
        let Some(b) = canvas_box(object.page_bbox(), sheet) else {
            continue;
        };
        leaves.push(i);
        boxes.push(b);
        anchors.push(text_anchor(object, sheet));
    }
    let mut widgets = Vec::with_capacity(fields.len());
    for (field, b) in fields {
        widgets.push(field);
        boxes.push(b);
        anchors.push(None);
    }
    let mut annots = Vec::with_capacity(marks.len());
    for (target, b) in marks {
        annots.push(target);
        boxes.push(b);
        anchors.push(None);
    }
    Some(Measured {
        page,
        objects,
        in_form,
        leaves,
        boxes,
        anchors,
        left_out: LeftOut {
            carried,
            elsewhere: leftout::elsewhere(doc, page),
        },
        widgets,
        annots,
        frame: Frame {
            page: page_box,
            drawing,
        },
    })
}

/// The selected form fields' widget boxes on `page`, in canvas space, in
/// selection order. Read from the same placement census the canvas hit-tests,
/// so a field is measured exactly where it is drawn.
fn field_boxes(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
) -> Vec<(pdfcer_gui_base::docidentity::SelectedField, Bx)> {
    if !doc.annotations_visible() {
        return Vec::new();
    }
    let selected = doc.field_set.all(doc.selected_field.as_ref());
    let on_page = selected.into_iter().filter(|f| f.page == page);
    crate::canvas::itembounds::fields(ctx, doc, on_page)
        .into_iter()
        .map(|(f, r)| (f, rect_box(r)))
        .collect()
}

/// The selected annotations' outlines on `page`, in canvas space, in
/// selection order.
fn annot_boxes(
    doc: &OpenDoc,
    page: usize,
) -> Vec<(pdfcer_gui_base::annotselection::AnnotTarget, Bx)> {
    if !doc.annotations_visible() {
        return Vec::new();
    }
    crate::canvas::itembounds::annots(&doc.selection, Some(page))
        .into_iter()
        .map(|(t, r)| (t, rect_box(r)))
        .collect()
}

/// A canvas-space rectangle as a layout box.
fn rect_box(r: egui::Rect) -> Bx {
    Bx::new(
        r.min.x.into(),
        r.min.y.into(),
        r.max.x.into(),
        r.max.y.into(),
    )
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
                // Widgets are drawn over all content, so they stack last.
                ExchangeOrder::Stacking => {
                    let mut order = rearrange::stacking_order(&m.objects);
                    order.extend(m.objects.len()..m.boxes.len());
                    order
                }
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
    moves_action(doc, m, sum, first.gesture())
}

/// Canvas `deltas`, one per measured box, as page-space moves: one
/// [`VectorAction::MoveEach`] or [`VectorAction::MoveEachInForm`] when one
/// engine verb covers them and nothing was left out, else one
/// [`VectorAction::MoveMixed`]. `None` when every delta is negligible.
fn moves_action(
    doc: &OpenDoc,
    m: &Measured,
    deltas: Vec<(f64, f64)>,
    gesture: &'static str,
) -> Option<Action> {
    let sheet = doc.pages.get(m.page)?;
    // Canvas space is f32.
    #[allow(clippy::cast_possible_truncation)]
    let to_page = |(dx, dy): (f64, f64)| {
        if layout::negligible((dx, dy)) {
            return None;
        }
        let d = crate::canvas::moving::page_delta(Vec2::new(dx as f32, dy as f32), sheet)?;
        Some((d.dx, d.dy))
    };
    let (object_deltas, rest) = deltas.split_at(m.objects.len().min(deltas.len()));
    let (leaf_deltas, rest) = rest.split_at(m.leaves.len().min(rest.len()));
    let (widget_deltas, annot_deltas) = rest.split_at(m.widgets.len().min(rest.len()));
    let moves: Vec<(usize, f64, f64)> = m
        .objects
        .iter()
        .zip(object_deltas)
        .filter_map(|(&i, &d)| to_page(d).map(|(dx, dy)| (i, dx, dy)))
        .collect();
    let leaf_moves: Vec<(usize, f64, f64)> = m
        .leaves
        .iter()
        .zip(leaf_deltas)
        .filter_map(|(&i, &d)| to_page(d).map(|(dx, dy)| (i, dx, dy)))
        .collect();
    let widgets: Vec<_> = m
        .widgets
        .iter()
        .zip(widget_deltas)
        .filter_map(|(f, &d)| to_page(d).map(|(dx, dy)| (f.clone(), dx, dy)))
        .collect();
    let annots: Vec<_> = m
        .annots
        .iter()
        .zip(annot_deltas)
        .filter_map(|(a, &d)| to_page(d).map(|(dx, dy)| (a.clone(), dx, dy)))
        .collect();
    let (objects, leaves) = if m.in_form {
        (Vec::new(), moves)
    } else {
        (moves, leaf_moves)
    };
    let provider = doc.page_objects()?;
    let mut leaves = leftout::by_drawing(provider.page_objects(), leaves);
    let notes = m.left_out.notes();
    // One engine verb covers page objects, or the parts of ONE drawing; any
    // other mix, or a sentence to say, goes through the folding commit.
    let verbs = usize::from(!objects.is_empty())
        + leaves.len()
        + usize::from(!widgets.is_empty())
        + usize::from(!annots.is_empty());
    if verbs == 0 {
        return None;
    }
    if verbs > 1 || !widgets.is_empty() || !annots.is_empty() || !notes.is_empty() {
        return Some(
            VectorAction::MoveMixed {
                page: m.page,
                objects,
                leaves,
                widgets,
                annots,
                notes,
                gesture,
            }
            .into(),
        );
    }
    Some(
        match leaves.pop() {
            Some(moves) => VectorAction::MoveEachInForm {
                page: m.page,
                moves,
                gesture,
            },
            None => VectorAction::MoveEach {
                page: m.page,
                moves: objects,
                gesture,
            },
        }
        .into(),
    )
}

/// Run `ops` against the current selection as one step: push the action, or
/// say why nothing moved.
pub fn run(
    ctx: &egui::Context,
    doc: &OpenDoc,
    ops: &[Op],
    settings: AlignUi,
    actions: &mut Vec<Action>,
) {
    let Some(m) = measure(ctx, doc) else {
        let (reason, said) = leftout::unmeasured(doc);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("align-declined op={ops:?} reason={reason}")
        });
        crate::app::actions::record_note(doc.edit_epoch, said);
        return;
    };
    match plan(doc, &m, ops, settings) {
        Some(action) => actions.push(action),
        None => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "align-declined op={ops:?} reason=nothing-moves n={}",
                    m.boxes.len()
                )
            });
            let mut said = vec![t::already_aligned().to_owned()];
            said.extend(m.left_out.notes());
            crate::app::actions::record_notes(doc.edit_epoch, said);
        }
    }
}

/// Draw the panel, scrolling: it is taller than a docked pane usually is.
pub fn body(ui: &mut Ui, doc: &OpenDoc, settings: &mut AlignUi, actions: &mut Vec<Action>) {
    egui::ScrollArea::vertical()
        .id_salt("align-panel") // ui-text-exempt: widget id
        .show(ui, |ui| contents(ui, doc, settings, actions));
    // The pane as it stands, not the content scrolled inside it: this is
    // where a wheel notch lands, however tall the tab's contents are.
    crate::diag::ui_rect_visible(REGION, ui.min_rect(), ui.clip_rect());
}

/// The panel's frames, top to bottom.
fn contents(ui: &mut Ui, doc: &OpenDoc, settings: &mut AlignUi, actions: &mut Vec<Action>) {
    if doc.selection.level() == SelectionLevel::Node {
        ui.label(t::nodes_heading());
        nodes::body(ui, doc, &mut settings.node_rel, actions);
        return;
    }
    let measured = measure(ui.ctx(), doc);
    let n = measured.as_ref().map_or(0, |m| m.boxes.len());
    ui.label(if n == 0 {
        leftout::unmeasured(doc).1
    } else {
        t::selected_count(n)
    });
    for line in measured.iter().flat_map(|m| m.left_out.notes()) {
        ui.label(line);
    }
    ui.horizontal(|ui| {
        for (index, tab) in Tab::ALL.into_iter().enumerate() {
            let response = egui_shell::tabshape::underline(ui, settings.tab == tab, t::tab(index));
            if response.clicked() {
                settings.tab = tab;
            }
            crate::diag::ui_rect_visible(TAB_REGIONS[index], response.rect, ui.clip_rect());
        }
    });
    ui.separator();
    match settings.tab {
        Tab::Align => align_tab(ui, doc, settings, n, actions),
        Tab::Grid => arrange::grid_tab(ui, doc, &mut settings.grid, n, actions),
        Tab::Circular => arrange::circular_tab(ui, doc, &mut settings.circle, n, actions),
    }
}

/// The Align tab: Align, Distribute, Rearrange, Remove overlaps.
fn align_tab(
    ui: &mut Ui,
    doc: &OpenDoc,
    settings: &mut AlignUi,
    n: usize,
    actions: &mut Vec<Action>,
) {
    ui.label(t::align_heading());
    ui.checkbox(&mut settings.as_group, t::as_group())
        .on_hover_text(t::as_group_tip());
    let toggle = ui
        .checkbox(&mut settings.on_canvas, t::on_canvas())
        .on_hover_text(t::on_canvas_tip());
    crate::diag::ui_rect_visible(ON_CANVAS_REGION, toggle.rect, ui.clip_rect());
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
                let icon = align_icon(axis, index);
                if button(ui, n >= op.needs(), icon, label, tip, regions[index]) {
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
                let icon = distribute_icon(axis, index);
                if button(ui, n >= op.needs(), icon, label, tip, regions[index]) {
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
            if button(
                ui,
                n >= 2,
                REARRANGE_ICONS[index],
                label,
                tip,
                REARRANGE_REGIONS[index],
            ) {
                pressed = Some(Op::Exchange(order));
            }
        }
        let (label, tip) = t::rearrange_button(3);
        if button(
            ui,
            n >= 2,
            REARRANGE_ICONS[3],
            label,
            tip,
            REARRANGE_REGIONS[3],
        ) {
            settings.randomized = settings.randomized.wrapping_add(1);
            pressed = Some(Op::Randomize(settings.randomized));
        }
        let (label, tip) = t::rearrange_button(4);
        if button(
            ui,
            n >= 2,
            REARRANGE_ICONS[4],
            label,
            tip,
            REARRANGE_REGIONS[4],
        ) {
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
        if button(
            ui,
            n >= 2,
            Icon::RemoveOverlaps,
            label,
            tip,
            REMOVE_OVERLAPS_REGION,
        ) {
            pressed = Some(Op::RemoveOverlaps);
        }
    });

    if let Some(op) = pressed {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!(
                "align-pressed op={op:?} n={n} relative={:?} group={}",
                settings.relative(n),
                settings.as_group
            )
        });
        run(ui.ctx(), doc, &[op], *settings, actions);
    }
}

/// The glyph side of a button, in points.
const GLYPH: f32 = 22.0;

/// The Rearrange row's glyphs, in [`REARRANGE_REGIONS`] order.
const REARRANGE_ICONS: [Icon; 5] = [
    Icon::ExchangeSelection,
    Icon::ExchangeStacking,
    Icon::ExchangeClockwise,
    Icon::Randomize,
    Icon::Unclump,
];

/// Align button `index` of `axis`'s row, in [`t::align_button`] order.
const fn align_icon(axis: Axis, index: usize) -> Icon {
    const X: [Icon; 6] = [
        Icon::AlignBefore,
        Icon::AlignLeft,
        Icon::AlignCentreH,
        Icon::AlignRight,
        Icon::AlignAfter,
        Icon::AlignTextH,
    ];
    const Y: [Icon; 6] = [
        Icon::AlignAbove,
        Icon::AlignTop,
        Icon::AlignCentreV,
        Icon::AlignBottom,
        Icon::AlignBelow,
        Icon::AlignTextV,
    ];
    match axis {
        Axis::X => X[index],
        Axis::Y => Y[index],
    }
}

/// Distribute button `index` of `axis`'s row, in [`t::distribute_button`] order.
const fn distribute_icon(axis: Axis, index: usize) -> Icon {
    const X: [Icon; 5] = [
        Icon::DistributeLeft,
        Icon::DistributeCentreH,
        Icon::DistributeRight,
        Icon::DistributeGapsH,
        Icon::DistributeTextH,
    ];
    const Y: [Icon; 5] = [
        Icon::DistributeTop,
        Icon::DistributeCentreV,
        Icon::DistributeBottom,
        Icon::DistributeGapsV,
        Icon::DistributeTextV,
    ];
    match axis {
        Axis::X => X[index],
        Axis::Y => Y[index],
    }
}

/// One glyph button, as Inkscape draws the panel: the words are its hover and
/// its accessible name. Greyed with a reason when too few objects are selected.
fn button(
    ui: &mut Ui,
    enabled: bool,
    icon: Icon,
    label: &str,
    tip: &str,
    region: &'static str,
) -> bool {
    let size = Vec2::splat(GLYPH + 2.0 * ui.spacing().button_padding.y);
    let response = ui.add_enabled(enabled, egui::Button::new("").min_size(size));
    paint_glyph(
        ui,
        &response,
        icon,
        response
            .rect
            .shrink2((response.rect.size() - Vec2::splat(GLYPH)) / 2.0),
    );
    finish(ui, response, enabled, label, tip, region)
}

/// A glyph-and-words button, for a control Inkscape labels in words.
fn labelled_button(
    ui: &mut Ui,
    enabled: bool,
    icon: Icon,
    label: &str,
    tip: &str,
    region: &'static str,
) -> bool {
    let pad = ui.spacing().button_padding;
    let text = egui::RichText::new(label);
    let response = ui.scope(|ui| {
        ui.spacing_mut().button_padding.x = pad.x + (GLYPH + pad.x) / 2.0;
        ui.add_enabled(
            enabled,
            egui::Button::new(text).min_size(Vec2::new(0.0, GLYPH + 2.0 * pad.y)),
        )
    });
    let response = response.inner;
    let r = response.rect;
    let glyph = egui::Rect::from_center_size(
        egui::pos2(r.left() + pad.x + GLYPH / 2.0, r.center().y),
        Vec2::splat(GLYPH),
    );
    paint_glyph(ui, &response, icon, glyph);
    finish(ui, response, enabled, label, tip, region)
}

fn paint_glyph(ui: &Ui, response: &egui::Response, icon: Icon, rect: egui::Rect) {
    let tint = ui.style().interact(response).fg_stroke.color;
    let tint = if response.enabled() {
        tint
    } else {
        ui.visuals().weak_text_color()
    };
    crate::icons::paint_icon_accented(ui.painter(), icon, rect, tint, response.enabled());
}

fn finish(
    ui: &Ui,
    response: egui::Response,
    enabled: bool,
    label: &str,
    tip: &str,
    region: &'static str,
) -> bool {
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    let response = response
        .on_hover_text(tip)
        .on_disabled_hover_text(t::needs_two());
    crate::diag::ui_rect_visible(region, response.rect, ui.clip_rect());
    response.clicked()
}
