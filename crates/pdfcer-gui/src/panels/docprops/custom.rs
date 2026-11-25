//! # `panels::docprops::custom` — the document's other `/Info` entries
//!
//! Lists every `/Info` entry the five standard rows above do not show — a
//! title block's drawing number, revision or project — with an editor for each
//! text value, a Remove button, and an Add row (`OPERATOR_REQUESTS.md` O381).
//! Every change is one `Action::SetInfoEntry`, so one undo step; the engine
//! keeps the XMP copy in step and the action reports any copy it skipped.
//!
//! The keys pdfcer keeps itself (`ReservedInfoKey`: Producer and the two
//! dates, plus Trapped) are listed read-only. The list is mirrored in
//! [`RESERVED`] because the engine's own predicate is private; the engine's
//! refusal stays the authority and is worded if the two ever disagree.

use egui::Ui;
use pdfcer_core::edit::{InfoField, InfoValue};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::panels::docprops as t;

/// The section's region.
pub const REGION: &str = "properties.custom"; // ui-text-exempt: trace region name, never displayed
/// The new entry's name box.
pub const REGION_NEW_KEY: &str = "properties.custom.key"; // ui-text-exempt: trace region name, never displayed
/// The new entry's value box.
pub const REGION_NEW_VALUE: &str = "properties.custom.value"; // ui-text-exempt: trace region name, never displayed
/// The Add button.
pub const REGION_ADD: &str = "properties.custom.add"; // ui-text-exempt: trace region name, never displayed
/// An entry's value box; the key follows the prefix.
pub const REGION_ROW_PREFIX: &str = "properties.custom.row."; // ui-text-exempt: trace region name, never displayed
/// An entry's Remove button; the key follows the prefix.
pub const REGION_REMOVE_PREFIX: &str = "properties.custom.remove."; // ui-text-exempt: trace region name, never displayed

/// Keys the engine refuses to write (`pdfcer_core::edit::ReservedInfoKey`).
const RESERVED: &[&[u8]] = &[b"Producer", b"CreationDate", b"ModDate", b"Trapped"];

/// One listed entry.
struct Entry {
    /// The key, byte-for-byte as stored.
    key: Vec<u8>,
    /// The text value as stored, or `None` for a value that is not text.
    stored: Option<String>,
    /// The operator's draft of the value.
    draft: String,
    /// Shown, never written: reserved, not text, or not decoded exactly.
    read_only: bool,
}

/// The section's drafts, re-seeded whenever the document's revision moves.
#[derive(Default)]
pub struct CustomDrafts {
    /// The `edit_epoch` the rows were seeded from.
    seeded_at: Option<u64>,
    /// The entries, in the document's order.
    entries: Vec<Entry>,
    /// The Add row's name.
    new_key: String,
    /// The Add row's value.
    new_value: String,
}

impl std::fmt::Debug for CustomDrafts {
    /// Counts, not contents.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomDrafts")
            .field("entries", &self.entries.len())
            .field("seeded_at", &self.seeded_at)
            .finish_non_exhaustive()
    }
}

impl CustomDrafts {
    /// Re-seed from the document when the revision has moved.
    fn sync(&mut self, doc: &OpenDoc) {
        if self.seeded_at == Some(doc.edit_epoch) {
            return;
        }
        self.seeded_at = Some(doc.edit_epoch);
        let standard: Vec<&[u8]> = InfoField::all().iter().map(|f| f.key()).collect();
        self.entries = doc
            .session
            .info_entries()
            .into_iter()
            .filter(|e| !standard.contains(&e.key.as_slice()))
            .map(|e| {
                let (stored, exact) = match &e.value {
                    InfoValue::Text(text) => (Some(text.text.clone()), text.exact),
                    _ => (None, true),
                };
                let read_only = !exact || stored.is_none() || RESERVED.contains(&e.key.as_slice());
                Entry {
                    draft: stored.clone().unwrap_or_default(),
                    key: e.key,
                    stored,
                    read_only,
                }
            })
            .collect();
        trace_entries(&self.entries);
    }
}

/// The listed keys and value lengths, for a driven check's readback.
fn trace_entries(entries: &[Entry]) {
    crate::diag::trace(|| {
        let list: Vec<String> = entries
            .iter()
            .map(|e| {
                let len = e.stored.as_ref().map_or(0, |s| s.chars().count());
                format!("{}:{len}", String::from_utf8_lossy(&e.key))
            })
            .collect();
        // ui-text-exempt: diagnostic trace, never displayed. Keys and value
        // LENGTHS: the values are the operator's document metadata.
        format!(
            "info-entries-now count={} keys={}",
            entries.len(),
            list.join("|")
        )
    });
}

/// Draw the section.
pub(super) fn section(
    ui: &mut Ui,
    doc: &OpenDoc,
    drafts: &mut CustomDrafts,
    actions: &mut Vec<Action>,
) {
    drafts.sync(doc);
    let top = ui.cursor().min;
    ui.add_space(6.0);
    ui.label(t::custom_heading());
    ui.label(egui::RichText::new(t::custom_note()).small().weak());
    if drafts.entries.is_empty() {
        ui.weak(t::custom_none());
    }
    for entry in &mut drafts.entries {
        entry_row(ui, entry, actions);
    }
    add_row(ui, drafts, actions);
    let rect = egui::Rect::from_min_max(top, ui.min_rect().max);
    crate::diag::ui_rect_visible(REGION, rect, ui.clip_rect());
}

/// Room left beside an entry's field for its Remove button, points.
const REMOVE_ROOM: f32 = 80.0;
/// The narrowest a field is drawn, points.
const MIN_FIELD: f32 = 80.0;

/// One entry: its name, its value, and a Remove button.
fn entry_row(ui: &mut Ui, entry: &mut Entry, actions: &mut Vec<Action>) {
    let name = String::from_utf8_lossy(&entry.key).into_owned();
    if entry.read_only {
        ui.horizontal_wrapped(|ui| {
            ui.label(&name);
            ui.label(entry.stored.as_deref().unwrap_or(t::custom_not_text()));
        });
        return;
    }
    // The name on its own line and the field filling what the Remove button
    // leaves: a dock is narrower than a name, a field and a button side by side.
    ui.label(&name);
    ui.horizontal(|ui| {
        let width = (ui.available_width() - REMOVE_ROOM).max(MIN_FIELD);
        // escape-disposition: commits — on `lost_focus`, which Escape causes.
        let response = ui.add(egui::TextEdit::singleline(&mut entry.draft).desired_width(width));
        // ui-text-exempt: trace region name, never displayed
        crate::diag::ui_rect_visible(
            &format!("{REGION_ROW_PREFIX}{name}"),
            response.rect,
            ui.clip_rect(),
        );
        let stored = entry.stored.as_deref().unwrap_or("");
        if crate::panels::forms::rows::commit(response.lost_focus(), &entry.draft, stored).is_some()
        {
            let value = (!entry.draft.trim().is_empty()).then(|| entry.draft.clone());
            actions.push(Action::SetInfoEntry {
                key: entry.key.clone(),
                value,
                adding: false,
            });
        }
        let remove = ui
            .button(t::custom_remove())
            .on_hover_text(t::custom_remove_hover());
        // ui-text-exempt: trace region name, never displayed
        crate::diag::ui_rect_visible(
            &format!("{REGION_REMOVE_PREFIX}{name}"),
            remove.rect,
            ui.clip_rect(),
        );
        if remove.clicked() {
            actions.push(Action::SetInfoEntry {
                key: entry.key.clone(),
                value: None,
                adding: false,
            });
        }
    });
}

/// The Add row: a name, a value, and the button.
fn add_row(ui: &mut Ui, drafts: &mut CustomDrafts, actions: &mut Vec<Action>) {
    // One field per line, each the dock's width, and the button under them:
    // a dock is too narrow for both fields and the button on one line.
    let width = ui.available_width().max(MIN_FIELD);
    // escape-disposition: keeps-draft — the Add row lives in panel state; nothing commits until Add.
    let key = ui.add(
        egui::TextEdit::singleline(&mut drafts.new_key)
            .hint_text(t::custom_new_key_hint())
            .desired_width(width),
    );
    crate::diag::ui_rect_visible(REGION_NEW_KEY, key.rect, ui.clip_rect());
    // escape-disposition: keeps-draft — the Add row lives in panel state; nothing commits until Add.
    let value = ui.add(
        egui::TextEdit::singleline(&mut drafts.new_value)
            .hint_text(t::custom_new_value_hint())
            .desired_width(width),
    );
    crate::diag::ui_rect_visible(REGION_NEW_VALUE, value.rect, ui.clip_rect());
    let named = !drafts.new_key.trim().is_empty();
    let add = ui
        .add_enabled(named, egui::Button::new(t::custom_add()))
        .on_disabled_hover_text(t::custom_add_needs_name());
    crate::diag::ui_rect_visible(REGION_ADD, add.rect, ui.clip_rect());
    if add.clicked() {
        // An empty value is written as an empty entry: a title block's
        // blank revision is still a field the operator asked for.
        actions.push(Action::SetInfoEntry {
            key: drafts.new_key.trim().as_bytes().to_vec(),
            value: Some(drafts.new_value.clone()),
            adding: true,
        });
        drafts.new_key.clear();
        drafts.new_value.clear();
    }
}
