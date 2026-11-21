//! Each field's display notes: what pdfcer's drawing of it formatted,
//! declined or ignored, read from the engine's synthesized appearances
//! (`WidgetAppearances::appearance(..).notes`) once per document revision.
//!
//! A row with notes declares `forms.fill.note.<row>`, and each read traces
//! one `forms-field-note row=… field=… kind=… reason="…"` line per note.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use pdfcer_core::forms::AcroForm;
use pdfcer_core::view::DocumentView;
use pdfcer_core::widget_draw::WidgetAppearances;

use crate::app::state::OpenDoc;
use crate::text::forms as t;

/// Region prefix of a row's notes; the row index is appended.
// ui-text-exempt: a diagnostic region name, never displayed.
const REGION_PREFIX: &str = "forms.fill.note.";

/// Each row's note sentences, for one `(document, epoch)`.
#[derive(Clone, Default)]
pub(super) struct FieldNotes {
    key: Option<(PathBuf, u64)>,
    by_row: Arc<HashMap<usize, Vec<String>>>,
}

impl FieldNotes {
    fn id() -> egui::Id {
        egui::Id::new("pdfcer-forms-notes")
    }

    /// This revision's notes, read from the engine when the revision is new.
    pub(super) fn load(
        ui: &egui::Ui,
        doc: &OpenDoc,
        view: &DocumentView<'_>,
        form: &AcroForm,
    ) -> Self {
        let key = (doc.path.clone(), doc.edit_epoch);
        let held: Option<Self> = ui.data(|d| d.get_temp(Self::id()));
        if let Some(held) = held.filter(|h| h.key.as_ref() == Some(&key)) {
            return held;
        }
        let fresh = Self {
            key: Some(key),
            by_row: Arc::new(read(view, form)),
        };
        ui.data_mut(|d| d.insert_temp(Self::id(), fresh.clone()));
        fresh
    }

    /// Draw row `row`'s notes, if it has any.
    pub(super) fn show(&self, ui: &mut egui::Ui, row: usize) {
        let Some(lines) = self.by_row.get(&row) else {
            return;
        };
        let r = ui
            .vertical(|ui| {
                for line in lines {
                    ui.small(line);
                }
            })
            .response
            .on_hover_text(t::field_display_notes_tooltip());
        crate::diag::ui_rect_visible(&format!("{REGION_PREFIX}{row}"), r.rect, ui.clip_rect());
    }
}

/// Every field's notes by row, each sentence once though several widgets
/// drew it.
fn read(view: &DocumentView<'_>, form: &AcroForm) -> HashMap<usize, Vec<String>> {
    let Some(drawn) = WidgetAppearances::read(view) else {
        return HashMap::new();
    };
    let mut by_row = HashMap::new();
    for (row, field) in form.fields.iter().enumerate() {
        let mut lines: Vec<String> = Vec::new();
        for widget in &field.widgets {
            let Some(look) = drawn.appearance(view, widget.id) else {
                continue;
            };
            for (kind, line) in t::field_display_notes(&look.notes) {
                if lines.contains(&line) {
                    continue;
                }
                crate::diag::trace(|| {
                    format!(
                        "forms-field-note row={row} field=\"{}\" kind={} reason=\"{}\"",
                        field.fully_qualified_name.replace('"', "'"),
                        kind.slug(),
                        line.replace('"', "'")
                    )
                });
                lines.push(line);
            }
        }
        if !lines.is_empty() {
            by_row.insert(row, lines);
        }
    }
    by_row
}
