//! # `panels::properties::link` — where a selected link goes, and its border
//!
//! Drawn for a selected `/Link` and nothing else. The form is seeded from the
//! file and re-seeded whenever the file changes under it (an undo, another
//! edit), so it never shows a value the link no longer has. Apply writes the
//! destination or the border as one undo entry each.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/panels/properties/link.md`.

use egui::Ui;
use pdfcer_core::object::ObjId;
use pdfcer_gui_base::linkaction::LinkAction;
use pdfcer_gui_base::linkform::{LinkForm, Regions};
use pdfcer_gui_base::linkread::{self, Goes, LinkReading};

use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::text::linkedit as t;

/// The region this section publishes.
pub const REGION: &str = "properties.link"; // ui-text-exempt: trace region name, never displayed
/// The Apply-destination button.
pub const REGION_APPLY_TARGET: &str = "properties.link.apply-target"; // ui-text-exempt: trace region name, never displayed
/// The Apply-border button.
pub const REGION_APPLY_BORDER: &str = "properties.link.apply-border"; // ui-text-exempt: trace region name, never displayed

/// The regions the form publishes here.
pub const REGIONS: Regions = Regions {
    page: "properties.link.page", // ui-text-exempt: trace region name, never displayed
    web: "properties.link.web",   // ui-text-exempt: trace region name, never displayed
    named: "properties.link.named", // ui-text-exempt: trace region name, never displayed
    field: "properties.link.field", // ui-text-exempt: trace region name, never displayed
    border: "properties.link.border", // ui-text-exempt: trace region name, never displayed
    colour: "properties.link.colour", // ui-text-exempt: trace region name, never displayed
};

/// The form, and the reading it was seeded from.
#[derive(Debug, Clone)]
struct Draft {
    seed: LinkReading,
    form: LinkForm,
}

/// **Draw the selected link's controls, or nothing.** Returns whether it
/// drew.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    let Some(selection) = doc.selection.annot() else {
        return false;
    };
    if selection.target.subtype != "Link" {
        return false;
    }
    let id = selection.target.id;
    crate::diag::ui_rect(REGION, ui.max_rect());
    ui.label(t::section_heading());
    // Locked is a property of this link, not of the build: it greys with its
    // reason (R9).
    if selection.target.locked {
        ui.label(egui::RichText::new(t::locked()).small().weak());
        ui.separator();
        return true;
    }
    let pages: Vec<ObjId> = doc.pages.iter().map(|p| p.id).collect();
    let Some(reading) = linkread::read(&doc.session, id, &pages) else {
        return false;
    };
    match &reading.goes {
        Goes::Other(kind) => {
            ui.label(egui::RichText::new(t::other_action(kind)).small().weak());
        }
        Goes::Nowhere => {
            ui.label(egui::RichText::new(t::nowhere()).small().weak());
        }
        Goes::Page(_) | Goes::Named(_) | Goes::Uri(_) => {}
    }

    let key = ui.make_persistent_id(("properties-link-draft", id)); // ui-text-exempt: an egui id salt, never displayed
    let mut draft = ui
        .data(|d| d.get_temp::<Draft>(key))
        .filter(|d| d.seed == reading)
        .unwrap_or_else(|| Draft {
            form: LinkForm::from_reading(&reading),
            seed: reading.clone(),
        });

    draft.form.goes_ui(ui, pages.len(), REGIONS);
    match draft.form.target(pages.len()) {
        Err(reason) => {
            ui.colored_label(ui.visuals().error_fg_color, reason);
        }
        Ok(target) => {
            let r = ui.button(t::apply_target());
            crate::diag::ui_rect(REGION_APPLY_TARGET, r.rect);
            if r.clicked() {
                actions.push(Action::Link(LinkAction::SetTarget { id, target }));
            }
        }
    }
    ui.add_space(6.0);
    draft.form.border_ui(ui, "properties-link-colour", REGIONS); // ui-text-exempt: an egui id salt, never displayed
    let r = ui.button(t::apply_border());
    crate::diag::ui_rect(REGION_APPLY_BORDER, r.rect);
    if r.clicked() {
        actions.push(Action::Link(LinkAction::SetBorder {
            id,
            border: draft.form.border(),
        }));
    }
    ui.data_mut(|d| d.insert_temp(key, draft));
    ui.separator();
    true
}
