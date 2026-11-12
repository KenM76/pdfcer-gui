//! # `panels::tags` — the document's structure tree, read-only, with the
//! picked element outlined on the page
//!
//! The tree is flattened each frame into the rows its open set shows and
//! drawn through `ScrollArea::show_rows`, so a document of ten thousand
//! paragraphs costs the rows on screen. Picking a row raises
//! [`Action::PickTag`] with the element's boxes and turns to the first; the
//! canvas draws them as a selection outline.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/tags.md`.

use std::collections::HashSet;

use pdfcer_core::structure_tree::{StructElement, StructKid, StructureTree};
use pdfcer_gui_base::tagtree::{TagPick, TagRead, no_box_reason};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::panels::PanelsState;
use crate::text::panels::tags as t;

/// Rows at a depth below this start open, which shows a table's cells.
const OPEN_BELOW: usize = 3;
/// The deepest level drawn; a deeper tree ends in one note row.
const MAX_DEPTH: usize = 200;
/// `RevealRect`'s requester.
const WHY: &str = "tags-pick"; // ui-text-exempt: a trace token, never displayed

/// Which rows the operator has turned from their default, per document.
#[derive(Default)]
pub struct TagsUi {
    serial: Option<u64>,
    toggled: HashSet<usize>,
}

impl TagsUi {
    fn for_doc(&mut self, serial: u64) -> &mut Self {
        if self.serial != Some(serial) {
            *self = Self {
                serial: Some(serial),
                toggled: HashSet::new(),
            };
        }
        self
    }

    fn is_open(&self, index: usize, depth: usize) -> bool {
        (depth < OPEN_BELOW) != self.toggled.contains(&index)
    }

    fn toggle(&mut self, index: usize) {
        if !self.toggled.remove(&index) {
            self.toggled.insert(index);
        }
    }
}

/// One drawn row.
struct Row {
    index: usize,
    depth: usize,
    parent_of: bool,
    open: bool,
}

/// Draw the Tags panel.
pub fn body(ui: &mut egui::Ui, doc: &OpenDoc, state: &mut PanelsState, actions: &mut Vec<Action>) {
    let read = doc.tag_tree();
    let tree = match &*read {
        TagRead::Untagged => {
            trace_state("untagged", 0);
            ui.label(t::untagged());
            return;
        }
        TagRead::Failed(why) => {
            trace_state("failed", 0);
            ui.label(t::failed(why));
            return;
        }
        TagRead::Read(tree) => tree,
    };
    trace_state("read", tree.elements.len());
    if tree.elements.is_empty() {
        ui.label(t::empty());
        return;
    }
    summary(ui, tree);
    let picked = doc
        .tag_pick
        .as_ref()
        .filter(|p| p.epoch == doc.edit_epoch)
        .map(|p| p.element);
    let tags = state.tags_mut().for_doc(doc.serial);
    let rows = flatten(tree, tags);
    let clicked = draw_rows(ui, tree, &rows, picked, tags);
    ui.separator();
    match picked {
        Some(i) => facts(ui, doc, tree, i),
        None => {
            ui.label(egui::RichText::new(t::pick_hint()).small().weak());
        }
    }
    if let Some(i) = clicked {
        pick(doc, tree, i, actions);
    }
}

fn trace_state(state: &str, elements: usize) {
    crate::diag::trace_changed("tags-panel", || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("tags-panel state={state} elements={elements}")
    });
}

/// One diagnostic sentence, given its count.
type Line = fn(usize) -> String;

/// The count and the engine's diagnostics that change what the tree means.
fn summary(ui: &mut egui::Ui, tree: &StructureTree) {
    let d = &tree.diagnostics;
    ui.label(t::count(tree.elements.len()));
    let lines: [(usize, Line); 3] = [
        (d.elements_revisited, t::revisited),
        (d.page_unresolved, t::unresolved),
        (d.named_not_declared, t::not_declared),
    ];
    for (n, line) in lines {
        if n > 0 {
            ui.label(egui::RichText::new(line(n)).small().weak());
        }
    }
    for note in &d.notes {
        ui.label(egui::RichText::new(note).small().weak());
    }
    ui.separator();
}

/// The rows the open set shows, in pre-order. Kids are followed only to a
/// larger index, the engine's own termination rule, so a cyclic `/K` ends.
fn flatten(tree: &StructureTree, tags: &TagsUi) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut stack: Vec<(usize, usize)> = tree.roots.iter().rev().map(|&r| (r, 0)).collect();
    while let Some((i, depth)) = stack.pop() {
        let Some(e) = tree.elements.get(i) else {
            continue;
        };
        let kids: Vec<usize> = element_kids(e, i).collect();
        let open = !kids.is_empty() && tags.is_open(i, depth);
        rows.push(Row {
            index: i,
            depth,
            parent_of: !kids.is_empty(),
            open,
        });
        if open && depth < MAX_DEPTH {
            stack.extend(kids.into_iter().rev().map(|c| (c, depth + 1)));
        }
    }
    rows
}

fn element_kids(e: &StructElement, i: usize) -> impl Iterator<Item = usize> + '_ {
    e.kids.iter().filter_map(move |k| match k {
        StructKid::Element(c) if *c > i => Some(*c),
        _ => None,
    })
}

/// Draw the visible rows; returns the one clicked.
fn draw_rows(
    ui: &mut egui::Ui,
    tree: &StructureTree,
    rows: &[Row],
    picked: Option<usize>,
    tags: &mut TagsUi,
) -> Option<usize> {
    let mut clicked = None;
    let height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
    egui::ScrollArea::vertical()
        .id_salt("tags-tree")
        .max_height(ui.available_height() * 0.6)
        .auto_shrink([false, true])
        .show_rows(ui, height, rows.len(), |ui, range| {
            for row in &rows[range] {
                let e = &tree.elements[row.index];
                ui.horizontal(|ui| {
                    ui.add_space(row.depth as f32 * 12.0);
                    if row.parent_of {
                        if ui.small_button(t::toggle(row.open)).clicked() {
                            tags.toggle(row.index);
                        }
                    } else {
                        ui.add_space(ui.spacing().interact_size.y);
                    }
                    let label = t::row(&e.resolved_type, e.title.as_deref());
                    let resp = ui.selectable_label(picked == Some(row.index), label);
                    crate::diag::ui_rect(&region(row.index, &e.resolved_type), resp.rect);
                    if resp.clicked() {
                        clicked = Some(row.index);
                    }
                    if row.open && row.depth == MAX_DEPTH {
                        ui.label(egui::RichText::new(t::too_deep()).small().weak());
                    }
                });
            }
        });
    clicked
}

/// The row's harness region: `tags.row.<index>.<type>`, the type kept to
/// letters and digits so a region name holds no space.
fn region(index: usize, kind: &str) -> String {
    let kind: String = kind.chars().filter(char::is_ascii_alphanumeric).collect();
    format!("tags.row.{index}.{kind}") // ui-text-exempt: a harness region name, never displayed
}

/// The picked element's attributes, and where it is outlined.
fn facts(ui: &mut egui::Ui, doc: &OpenDoc, tree: &StructureTree, i: usize) {
    use t::field as f;
    let Some(e) = tree.elements.get(i) else {
        return;
    };
    let lang = match (&e.lang, &e.effective_lang) {
        (Some(l), _) => Some(l.clone()),
        (None, Some(l)) => Some(t::inherited(l)),
        _ => None,
    };
    let rows = [
        (f::kind(), Some(e.resolved_type.clone())),
        (
            f::written(),
            (e.raw_type != e.resolved_type).then(|| e.raw_type.clone()),
        ),
        (f::title(), e.title.clone()),
        (f::alt(), e.alt.clone()),
        (f::actual(), e.actual_text.clone()),
        (f::lang(), lang),
        (f::scope(), e.scope.clone()),
        (
            f::headers(),
            (!e.headers.is_empty()).then(|| t::header_list(&e.headers)),
        ),
        (f::row_span(), e.row_span.map(|n| n.to_string())),
        (f::col_span(), e.col_span.map(|n| n.to_string())),
    ];
    let pick = doc.tag_pick.as_ref();
    egui::Grid::new("tags-facts").num_columns(2).show(ui, |ui| {
        for (name, value) in rows {
            if let Some(value) = value {
                ui.label(name);
                ui.label(value);
                ui.end_row();
            }
        }
        if let Some(pick) = pick.filter(|p| !p.boxes.is_empty()) {
            let pages: Vec<usize> = pick.boxes.iter().map(|(p, _)| *p).collect();
            ui.label(f::pages());
            ui.label(t::page_list(&pages));
            ui.end_row();
        }
    });
    if pick.is_some_and(|p| p.boxes.is_empty()) {
        ui.label(t::no_box(no_box_reason(tree, i)));
    }
}

/// Pick element `i`: record its boxes, turn to the first page, reveal it.
fn pick(doc: &OpenDoc, tree: &StructureTree, i: usize, actions: &mut Vec<Action>) {
    let boxes = tree.element_bbox(i);
    let kind = tree
        .elements
        .get(i)
        .map_or("", |e| e.resolved_type.as_str());
    crate::diag::trace(|| {
        let at: Vec<String> = boxes
            .iter()
            .map(|(p, r)| {
                format!(
                    "{}:{:.1},{:.1},{:.1},{:.1}",
                    p + 1,
                    r.llx,
                    r.lly,
                    r.urx,
                    r.ury
                )
            })
            .collect();
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "tags-pick element={i} type={kind} boxes={} at={}",
            boxes.len(),
            at.join(";")
        )
    });
    let first = boxes.first().copied();
    actions.push(Action::PickTag(Some(TagPick {
        element: i,
        epoch: doc.edit_epoch,
        boxes,
    })));
    let Some((page, r)) = first else {
        return;
    };
    if doc.view.page_index != page {
        actions.push(Action::GoToPage(page));
    }
    let Some(info) = doc.pages.get(page) else {
        return;
    };
    if let Some(canvas) =
        crate::canvas::geometry::pdf_rect_to_canvas((r.llx, r.lly, r.urx, r.ury), info)
    {
        let (min, max) = crate::canvas::minreveal::fracs_for_canvas_rect(canvas, info);
        actions.push(Action::RevealRect {
            page,
            min,
            max,
            why: WHY,
        });
    }
}
