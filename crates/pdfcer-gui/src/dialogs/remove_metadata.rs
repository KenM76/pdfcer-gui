//! # `dialogs::remove_metadata` — Security ▸ Protect ▸ Remove metadata…
//!
//! Contract: lists every item `EditSession::metadata_inventory` finds, grouped
//! by kind in the engine's order, each with where it is, a short look at it
//! and its size, none ticked at open. A kind's checkbox ticks or clears its
//! items; *Select all* ticks every one. *Remove and save a copy…* pushes one
//! [`WriteAction::RemoveMetadata`] naming the ticked ids, which
//! `app::actions::remove_metadata` writes as a one-version copy. The open
//! document is not changed: this shell's Save appends a revision, which would
//! keep every removed value in the one before it.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/remove_metadata.md`.

use pdfcer_core::doc_metadata::{MetadataItem, MetadataKind};

use crate::app::actions::Action;
use crate::app::actions::write::WriteAction;
use crate::app::state::{OpenDoc, Status};
use crate::text::metadata as t;

/// The window body.
pub const REGION_BODY: &str = "remove-metadata.body"; // ui-text-exempt: trace region name, never displayed
/// One item's checkbox; the item's id follows the dot.
pub const REGION_ITEM: &str = "remove-metadata.item"; // ui-text-exempt: trace region name, never displayed
/// One kind's checkbox; `MetadataKind::as_str` follows the dot.
pub const REGION_KIND: &str = "remove-metadata.kind"; // ui-text-exempt: trace region name, never displayed
/// Select all.
pub const REGION_ALL: &str = "remove-metadata.all"; // ui-text-exempt: trace region name, never displayed
/// The button that removes.
pub const REGION_COMMIT: &str = "remove-metadata.commit"; // ui-text-exempt: trace region name, never displayed

/// One listed item and whether it is ticked.
#[derive(Debug)]
struct Row {
    item: MetadataItem,
    chosen: bool,
}

/// The window.
#[derive(Debug)]
pub struct RemoveMetadataDialog {
    rows: Vec<Row>,
    truncated: bool,
    commit_requested: bool,
    close_requested: bool,
}

impl RemoveMetadataDialog {
    fn open(doc: &OpenDoc) -> Self {
        let inventory = doc.session.metadata_inventory();
        let rows: Vec<Row> = inventory
            .items
            .into_iter()
            .map(|item| Row {
                item,
                chosen: false,
            })
            .collect();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed. Ids, not
            // previews: the previews are the operator's own content.
            format!(
                "remove-metadata-listed items={} truncated={} ids={}",
                rows.len(),
                inventory.truncated,
                ids(rows.iter())
            )
        });
        Self {
            rows,
            truncated: inventory.truncated,
            commit_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "remove-metadata", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(620.0, 520.0),
            egui::vec2(420.0, 300.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if std::mem::take(&mut self.commit_requested) {
            let chosen: Vec<&Row> = self.rows.iter().filter(|r| r.chosen).collect();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "remove-metadata-requested ids={}",
                    ids(chosen.iter().copied())
                )
            });
            actions.push(Action::Write(WriteAction::RemoveMetadata {
                ids: chosen
                    .iter()
                    .map(|r| r.item.id.as_str().to_owned())
                    .collect(),
            }));
            return false;
        }
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        if self.rows.is_empty() {
            ui.label(t::none_present());
        } else {
            ui.label(t::intro());
            if self.truncated {
                ui.label(egui::RichText::new(t::truncated()).weak());
            }
            ui.add_space(6.0);
            let all = ui.button(t::select_all());
            crate::diag::ui_rect(REGION_ALL, all.rect);
            if all.clicked() {
                self.rows.iter_mut().for_each(|r| r.chosen = true);
            }
            ui.add_space(6.0);
            let height = (ui.available_height() - 48.0).max(120.0);
            egui::ScrollArea::vertical()
                .max_height(height)
                .auto_shrink([false, true])
                .show(ui, |ui| self.list(ui));
        }
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            let any = self.rows.iter().any(|r| r.chosen);
            let remove = ui
                .add_enabled(any, egui::Button::new(t::remove_button()))
                .on_disabled_hover_text(t::names_nothing());
            crate::diag::ui_rect_visible(REGION_COMMIT, remove.rect, ui.clip_rect());
            if remove.clicked() {
                self.commit_requested = true;
            }
            if ui.button(t::cancel_button()).clicked() {
                self.close_requested = true;
            }
        });
    }

    /// Every kind present, its checkbox, then its items.
    fn list(&mut self, ui: &mut egui::Ui) {
        let clip = ui.clip_rect();
        for kind in MetadataKind::ALL {
            let mut members = self
                .rows
                .iter_mut()
                .filter(|r| r.item.kind == kind)
                .peekable();
            if members.peek().is_none() {
                continue;
            }
            let members: Vec<&mut Row> = members.collect();
            let mut all = members.iter().all(|r| r.chosen);
            let heading = format!("{} ({})", t::kind(kind), members.len()); // ui-text-exempt: a catalog phrase and a count
            let r = ui.checkbox(&mut all, heading);
            // ui-text-exempt: trace region name, never displayed
            crate::diag::ui_rect_visible(&format!("{REGION_KIND}.{}", kind.as_str()), r.rect, clip);
            if r.changed() {
                for row in members {
                    row.chosen = all;
                }
            } else {
                ui.indent(kind.as_str(), |ui| {
                    for row in members {
                        item_row(ui, row, clip);
                    }
                });
            }
            if kind == MetadataKind::DocumentId {
                ui.label(egui::RichText::new(t::document_id_note()).small().weak());
            }
            ui.add_space(4.0);
        }
    }
}

/// One item: its checkbox naming where it is, then its preview and size.
fn item_row(ui: &mut egui::Ui, row: &mut Row, clip: egui::Rect) {
    ui.horizontal(|ui| {
        let r = ui.checkbox(&mut row.chosen, label(&row.item));
        crate::diag::ui_rect_visible(
            // ui-text-exempt: trace region name, never displayed
            &format!("{REGION_ITEM}.{}", row.item.id),
            r.rect,
            clip,
        );
        if !row.item.preview.is_empty() {
            ui.label(egui::RichText::new(&row.item.preview).weak());
        }
        ui.label(egui::RichText::new(t::size(row.item.bytes)).small().weak());
    });
}

/// What names one item in its kind's list: a description entry by its key
/// (the id's documented `info/<Key>` form), anything else by where it is.
fn label(item: &MetadataItem) -> &str {
    match item.kind {
        MetadataKind::InfoEntry => item
            .id
            .as_str()
            .strip_prefix("info/") // ui-text-exempt: the engine's id prefix
            .unwrap_or(&item.location),
        _ => &item.location,
    }
}

/// The rows' ids, comma-joined, for a trace.
fn ids<'a>(rows: impl Iterator<Item = &'a Row>) -> String {
    rows.map(|r| r.item.id.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

/// Build it for the current document. `None` when no document is open.
pub fn open_for(status: &Status) -> Option<RemoveMetadataDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(RemoveMetadataDialog::open(doc))
}
