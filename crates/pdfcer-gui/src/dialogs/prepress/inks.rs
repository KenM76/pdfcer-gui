//! # `dialogs::prepress::inks` — the Ink Manager's list and choices
//!
//! Lists `EditSession::inks` once per edit epoch, so a run's result is the
//! next list the window shows, and holds one choice per spot ink: print as
//! itself, as another ink (`alias_ink`), or as process (`spot_to_process`).
//! Alias targets are every other listed ink plus the four process inks.
//!
//! Traces `ink-manager-listed epoch= inks= drawn= spots=` on every re-list,
//! ink names `|`-joined with spaces written `_`. `drawn` is the inks some
//! page draws with: a converted spot stays in `inks`, because
//! `spot_to_process` leaves the space's own array in place, and leaves
//! `drawn`.

use std::collections::BTreeMap;

use pdfcer_core::prepress::{InkKind, InkUse};
use pdfcer_gui_base::prepress::PrepressVerb;

use crate::app::state::OpenDoc;
use crate::text::inks as t;

/// One spot ink's choice combo box, suffixed `:<ink>`.
pub const REGION_CHOICE: &str = "prepress.ink-choice"; // ui-text-exempt: trace region name, never displayed
/// One ink's name cell, suffixed `:<ink>`.
pub const REGION_ROW: &str = "prepress.ink-row"; // ui-text-exempt: trace region name, never displayed

/// The process inks, in plate order: always offered as alias targets.
const PROCESS: [&str; 4] = ["Cyan", "Magenta", "Yellow", "Black"]; // ui-text-exempt: PDF colorant names

/// How one spot ink prints.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Choice {
    Keep,
    Process,
    As(String),
}

/// An ink name as a region or trace field: no spaces.
fn token(name: &str) -> String {
    name.replace(' ', "_")
}

/// The Ink Manager half of a prepress window.
#[derive(Default)]
pub(super) struct InkForm {
    /// The edit epoch [`Self::inks`] was read at.
    listed_at: Option<u64>,
    inks: Vec<InkUse>,
    /// Spot ink name → its choice; absent means [`Choice::Keep`].
    choice: BTreeMap<String, Choice>,
}

impl InkForm {
    /// Re-read the inks when the document has changed since the last list.
    pub(super) fn refresh(&mut self, doc: &OpenDoc) {
        if self.listed_at == Some(doc.edit_epoch) {
            return;
        }
        self.listed_at = Some(doc.edit_epoch);
        self.inks = doc.session.inks();
        let names: Vec<&str> = self.inks.iter().map(|i| i.name.as_str()).collect();
        self.choice.retain(|k, _| names.contains(&k.as_str()));
        let spots = self.inks.iter().filter(|i| i.kind == InkKind::Spot).count();
        let join = |only_drawn: bool| {
            let kept = self
                .inks
                .iter()
                .filter(|i| !only_drawn || !i.pages.is_empty());
            kept.map(|i| token(&i.name)).collect::<Vec<_>>().join("|") // ui-text-exempt: a trace list separator
        };
        let (all, drawn) = (join(false), join(true));
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed
            format!(
                "ink-manager-listed epoch={} inks={all} drawn={drawn} spots={spots}",
                doc.edit_epoch
            )
        });
    }

    /// The ink table.
    pub(super) fn options(&mut self, ui: &mut egui::Ui) {
        if self.inks.is_empty() {
            ui.label(t::none());
            return;
        }
        egui::Grid::new("prepress-inks") // ui-text-exempt: an egui id
            .striped(true)
            .num_columns(5)
            .show(ui, |ui| {
                for heading in [
                    t::col_ink(),
                    t::col_kind(),
                    t::col_alternate(),
                    t::col_pages(),
                    t::col_print_as(),
                ] {
                    ui.label(heading);
                }
                ui.end_row();
                for index in 0..self.inks.len() {
                    self.row(ui, index);
                }
            });
    }

    fn row(&mut self, ui: &mut egui::Ui, index: usize) {
        let ink = &self.inks[index];
        let named = ui.label(&ink.name);
        crate::diag::ui_rect(&format!("{REGION_ROW}:{}", token(&ink.name)), named.rect);
        let spot = ink.kind == InkKind::Spot;
        ui.label(if spot {
            t::kind_spot()
        } else {
            t::kind_process()
        });
        let alternates = ink.alternates.join(", "); // ui-text-exempt: a list separator
        let alt = ui.label(alternates);
        if ink.alternates.len() > 1 {
            alt.on_hover_text(t::alternates_disagree_hover());
        }
        if ink.pages.is_empty() {
            ui.label(t::not_drawn());
        } else {
            let list: Vec<String> = ink.pages.iter().map(usize::to_string).collect();
            ui.label(t::pages(&list.join(", "), ink.uses)); // ui-text-exempt: a list separator
        }
        let name = ink.name.clone();
        if spot {
            self.choice_box(ui, &name);
        } else {
            ui.allocate_space(egui::Vec2::ZERO);
        }
        ui.end_row();
    }

    /// The combo box choosing how spot `name` prints.
    fn choice_box(&mut self, ui: &mut egui::Ui, name: &str) {
        let region = format!("{REGION_CHOICE}:{}", token(name));
        let mut targets: Vec<String> = self
            .inks
            .iter()
            .map(|i| i.name.clone())
            .filter(|n| n != name)
            .collect();
        for p in PROCESS {
            if !targets.iter().any(|n| n == p) {
                targets.push(p.to_owned());
            }
        }
        let current = self.choice.get(name).cloned().unwrap_or(Choice::Keep);
        let mut picked = current.clone();
        let combo = egui::ComboBox::from_id_salt(&region)
            .selected_text(label(&current))
            .height(f32::INFINITY)
            .show_ui(ui, |ui| {
                let keep = ui.selectable_value(&mut picked, Choice::Keep, t::keep());
                crate::diag::ui_rect(&format!("{region}:keep"), keep.rect); // ui-text-exempt: a region suffix
                let process = ui
                    .selectable_value(&mut picked, Choice::Process, t::process())
                    .on_hover_text(t::process_hover());
                crate::diag::ui_rect(&format!("{region}:process"), process.rect); // ui-text-exempt: a region suffix
                for to in &targets {
                    let entry = ui
                        .selectable_value(&mut picked, Choice::As(to.clone()), t::alias(to))
                        .on_hover_text(t::alias_hover());
                    crate::diag::ui_rect(&format!("{region}:as:{}", token(to)), entry.rect); // ui-text-exempt: a region suffix
                }
            })
            .response;
        crate::diag::ui_rect(&region, combo.rect);
        if picked != current {
            if picked == Choice::Keep {
                self.choice.remove(name);
            } else {
                self.choice.insert(name.to_owned(), picked);
            }
        }
    }

    /// Why Apply is greyed, or `None` when it is not.
    pub(super) fn blocked(&self) -> Option<String> {
        if self.choice.is_empty() {
            return Some(t::nothing_chosen().to_owned());
        }
        self.choice.iter().find_map(|(from, c)| match c {
            Choice::As(to) if matches!(self.choice.get(to), Some(Choice::As(_))) => {
                Some(t::chained(from, to))
            }
            _ => None,
        })
    }

    /// The verb the choices make, and the choices cleared for the next run.
    pub(super) fn take_verb(&mut self) -> PrepressVerb {
        let mut aliases = Vec::new();
        let mut to_process = Vec::new();
        for (name, c) in std::mem::take(&mut self.choice) {
            match c {
                Choice::As(to) => aliases.push((name, to)),
                Choice::Process => to_process.push(name),
                Choice::Keep => {}
            }
        }
        PrepressVerb::Inks {
            aliases,
            to_process,
        }
    }
}

fn label(choice: &Choice) -> String {
    match choice {
        Choice::Keep => t::keep().to_owned(),
        Choice::Process => t::process().to_owned(),
        Choice::As(to) => t::alias(to),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(choices: &[(&str, Choice)]) -> InkForm {
        InkForm {
            choice: choices
                .iter()
                .map(|(n, c)| ((*n).to_owned(), c.clone()))
                .collect(),
            ..InkForm::default()
        }
    }

    #[test]
    fn nothing_chosen_blocks_apply() {
        assert!(form(&[]).blocked().is_some());
    }

    #[test]
    fn an_alias_onto_an_aliased_ink_blocks_apply() {
        let f = form(&[("A", Choice::As("B".into())), ("B", Choice::As("C".into()))]);
        assert!(f.blocked().is_some());
    }

    #[test]
    fn an_alias_onto_a_converted_ink_is_allowed() {
        let f = form(&[("A", Choice::As("B".into())), ("B", Choice::Process)]);
        assert!(f.blocked().is_none());
    }

    #[test]
    fn the_verb_carries_every_choice_and_clears_them() {
        let mut f = form(&[("A", Choice::As("Black".into())), ("G", Choice::Process)]);
        let verb = f.take_verb();
        assert_eq!(
            verb,
            PrepressVerb::Inks {
                aliases: vec![("A".into(), "Black".into())],
                to_process: vec!["G".into()],
            }
        );
        assert!(f.choice.is_empty());
    }
}
