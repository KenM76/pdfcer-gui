//! # `dialogs::remove_metadata` — Security ▸ Protect ▸ Remove metadata…
//!
//! Contract: lists the document-information entries the engine can read
//! (`InfoField::all()`) that the document holds, each with its value and a
//! checkbox, none ticked at open. *Select all* ticks every one; *Remove
//! chosen* pushes one [`Action::SetInfoField`] with `value: None` per ticked
//! entry, which the engine records as one Undo step each. The window says
//! off-canvas that dates, producer and XMP are not listed.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/remove_metadata.md`.

use pdfcer_core::edit::InfoField;

use crate::app::actions::Action;
use crate::app::state::{OpenDoc, Status};
use crate::text::metadata as t;

/// The window body.
pub const REGION_BODY: &str = "remove-metadata.body"; // ui-text-exempt: trace region name, never displayed
/// One entry's checkbox; the entry's PDF key follows the dot.
pub const REGION_FIELD: &str = "remove-metadata.field"; // ui-text-exempt: trace region name, never displayed
/// Select all.
pub const REGION_ALL: &str = "remove-metadata.all"; // ui-text-exempt: trace region name, never displayed
/// The button that removes.
pub const REGION_COMMIT: &str = "remove-metadata.commit"; // ui-text-exempt: trace region name, never displayed

/// One entry the document holds.
#[derive(Debug)]
struct Entry {
    field: InfoField,
    /// Its text as the engine decoded it.
    value: String,
    chosen: bool,
}

/// The window.
#[derive(Debug)]
pub struct RemoveMetadataDialog {
    entries: Vec<Entry>,
    commit_requested: bool,
    close_requested: bool,
}

impl RemoveMetadataDialog {
    fn open(doc: &OpenDoc) -> Self {
        let entries: Vec<Entry> = InfoField::all()
            .iter()
            .filter_map(|field| {
                doc.session.info_text(*field).map(|info| Entry {
                    field: *field,
                    value: info.text,
                    chosen: false,
                })
            })
            .collect();
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed. Keys, not
            // values: the values are the operator's own document metadata.
            format!("remove-metadata-listed fields={}", keys(entries.iter()))
        });
        Self {
            entries,
            commit_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. Returns whether it stays open.
    pub fn show(&mut self, ctx: &egui::Context, actions: &mut Vec<Action>) -> bool {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "remove-metadata", // ui-text-exempt: a viewport key, never displayed.
            t::title(),
            egui::vec2(460.0, 340.0),
            egui::vec2(340.0, 240.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui);
        });
        if std::mem::take(&mut self.commit_requested) {
            let chosen: Vec<&Entry> = self.entries.iter().filter(|e| e.chosen).collect();
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed
                format!(
                    "remove-metadata-requested fields={}",
                    keys(chosen.iter().copied())
                )
            });
            for entry in chosen {
                actions.push(Action::SetInfoField {
                    field: entry.field,
                    value: None,
                });
            }
            return false;
        }
        !frame.closed && !std::mem::take(&mut self.close_requested)
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        if self.entries.is_empty() {
            ui.label(t::none_present());
        } else {
            ui.label(t::intro());
            ui.add_space(8.0);
            egui::Grid::new("remove-metadata-grid") // ui-text-exempt: an egui id, never displayed
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    for entry in &mut self.entries {
                        let r = ui.checkbox(
                            &mut entry.chosen,
                            crate::text::panels::docprops::info_label(entry.field),
                        );
                        crate::diag::ui_rect(
                            // ui-text-exempt: trace region name, never displayed
                            &format!("{REGION_FIELD}.{}", key(entry.field)),
                            r.rect,
                        );
                        ui.label(&entry.value);
                        ui.end_row();
                    }
                });
            ui.add_space(6.0);
            let all = ui.button(t::select_all());
            crate::diag::ui_rect(REGION_ALL, all.rect);
            if all.clicked() {
                for entry in &mut self.entries {
                    entry.chosen = true;
                }
            }
        }
        ui.add_space(8.0);
        ui.label(egui::RichText::new(t::not_listed()).small().weak());
        ui.add_space(10.0);
        ui.separator();
        ui.horizontal(|ui| {
            let any = self.entries.iter().any(|e| e.chosen);
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
}

/// The entries' PDF keys, comma-joined, for a trace.
fn keys<'a>(entries: impl Iterator<Item = &'a Entry>) -> String {
    entries.map(|e| key(e.field)).collect::<Vec<_>>().join(",")
}

/// The entry's PDF key, `Title` … `Keywords`.
fn key(field: InfoField) -> String {
    String::from_utf8_lossy(field.key()).into_owned()
}

/// Build it for the current document. `None` when no document is open.
pub fn open_for(status: &Status) -> Option<RemoveMetadataDialog> {
    let Status::Open(doc) = status else {
        return None;
    };
    Some(RemoveMetadataDialog::open(doc))
}
