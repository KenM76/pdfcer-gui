//! # `panels::properties::annotflags` — whether the selected annotation shows
//! on screen, prints, and is locked
//!
//! Drawn for an annotation of any kind. Each switch raises
//! `AnnotAction::SetFlag`, resolved against the document's `/F` when applied
//! (`pdfcer_gui_base::annotflagswitch`). Drawn for a locked mark too: this is
//! the one place a lock can be undone, and the engine permits it by design.
//! Read from the session every frame, because the action that changes these
//! values applies after the frame that raised it.

use egui::Ui;
use pdfcer_gui_base::annotflagswitch::{FlagSwitch, is_on};

use crate::app::actions::Action;
use crate::app::actions::annot::AnnotAction;
use crate::app::state::OpenDoc;
use crate::text::panels::annotflags as t;

/// The section's rect, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION: &str = "properties.annot_flags";

/// Draw the three switches, or nothing.
pub fn section(ui: &mut Ui, doc: &OpenDoc, actions: &mut Vec<Action>) -> bool {
    let Some(selection) = doc.selection.annot() else {
        return false;
    };
    let target = &selection.target;
    let Some(page) = doc.pages.get(target.page) else {
        return false;
    };
    let Some(word) = pdfcer_core::annot::page_annotations(&doc.session.graph(), page.id)
        .into_iter()
        .find(|a| a.id == Some(target.id))
        .map(|a| a.flags.0)
    else {
        return false;
    };
    crate::diag::trace_changed(REGION, || {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed
            "annot-flags-shown id={} on_screen={} prints={} locked={}",
            target.id.num,
            u8::from(is_on(word, FlagSwitch::OnScreen)),
            u8::from(is_on(word, FlagSwitch::Prints)),
            u8::from(is_on(word, FlagSwitch::Locked)),
        )
    });
    let top = ui.min_rect().bottom();
    ui.label(t::heading());
    let rows = [
        (FlagSwitch::OnScreen, t::on_screen(), t::on_screen_hover()),
        (FlagSwitch::Prints, t::prints(), t::prints_hover()),
        (FlagSwitch::Locked, t::locked(), t::locked_hover()),
    ];
    for (switch, label, hover) in rows {
        let mut on = is_on(word, switch);
        let response = ui.checkbox(&mut on, label).on_hover_text(hover);
        crate::diag::ui_rect_visible(&region_of(switch), response.rect, ui.clip_rect());
        if response.changed() {
            actions.push(Action::Annot(AnnotAction::SetFlag {
                page: target.page,
                id: target.id,
                switch,
                on,
            }));
        }
    }
    let mut rect = ui.min_rect();
    rect.min.y = top;
    crate::diag::ui_rect(REGION, rect);
    ui.separator();
    true
}

/// One switch's region name.
fn region_of(switch: FlagSwitch) -> String {
    let leaf = match switch {
        FlagSwitch::OnScreen => "on_screen", // ui-text-exempt: trace region name
        FlagSwitch::Prints => "prints",      // ui-text-exempt: trace region name
        FlagSwitch::Locked => "locked",      // ui-text-exempt: trace region name
    };
    format!("{REGION}.{leaf}")
}
