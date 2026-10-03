//! # `settingspages::keys` — Settings ▸ Keyboard shortcuts
//!
//! Every command in the [`Catalog`], grouped by ribbon tab, with its keys and
//! three buttons: Change waits for the next key pressed, Off switches the
//! command's keys off, Reset puts the program's keys back. A key another
//! command holds becomes a clash the operator must Reassign or Cancel; a key
//! the program keeps is refused. Edits land in `Draft::working_prefs`, so one
//! Save writes them with every other setting and one Cancel drops them.
//!
//! Contract for the harness: the row buttons publish
//! `settings.keys.<change|off|reset>.<command>` (typing rows insert `typing.`
//! after `keys.`), the filter box `settings.keys.filter`, the clash buttons
//! `settings.keys.reassign` and `settings.keys.cancel`. Traces:
//! `shortcut-capture command=`, `shortcut-set command= chords=`,
//! `shortcut-clash command= chord= holder=`, `shortcut-refused chord= why=`.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settingspages/keys.md`.

use std::collections::BTreeMap;

use egui::{Event, Id, Key, Modifiers, RichText, Ui};

use super::keyscatalog::{Catalog, CatalogGroup, KeptFor, chords_of, holder};
use super::{Draft, widgets};
use crate::keychord::{format_chord, parse_chord};
use crate::prefs::shortcuts::{ShortcutPrefs, effective};
use crate::text::settings::keys as t;

// ui-text-exempt: an egui memory key, never displayed.
const STATE: &str = "settings.keys.state";

// ui-text-exempt: trace region prefix, never displayed.
const REGION: &str = "settings.keys.";

/// What the page is in the middle of, kept between frames.
#[derive(Debug, Clone, Default)]
struct PageState {
    filter: String,
    /// The command waiting for its new key.
    capturing: Option<String>,
    /// `(command, chord, holder)`: a key another command already has.
    clash: Option<(String, String, String)>,
    /// `(chord, why)`: the last key refused.
    refused: Option<(String, KeptFor)>,
}

/// Draw the page over `draft`.
pub fn page(ui: &mut Ui, draft: &mut Draft) {
    let catalog = draft.shortcuts.clone();
    let id = Id::new(STATE);
    let mut state: PageState = ui.data(|d| d.get_temp(id)).unwrap_or_default();
    let prefs = &mut draft.working_prefs.shortcuts;
    // The search index draws every page invisibly; only the page on show may
    // take the frame's keys.
    if ui.is_visible() {
        capture(ui, &catalog, prefs, &mut state);
    }

    widgets::header(ui, t::title(), t::silence(), t::radius());
    if catalog.defaults.is_empty() {
        ui.label(t::unavailable());
        return;
    }
    toolbar(ui, prefs, &mut state);
    notice(ui, &catalog, prefs, &mut state);
    ui.add_space(6.0);

    let keymap = effective(&catalog.defaults, prefs);
    for group in &catalog.groups {
        group_rows(ui, group, "", &keymap, prefs, &mut state);
    }
    if group_rows(ui, &catalog.typing, "typing.", &keymap, prefs, &mut state) {
        ui.label(RichText::new(t::typing_fixed()).small().weak());
    }
    ui.data_mut(|d| d.insert_temp(id, state));
}

fn toolbar(ui: &mut Ui, prefs: &mut ShortcutPrefs, state: &mut PageState) {
    ui.horizontal(|ui| {
        let filter = ui.add(
            // escape-disposition: not-content — a filter query over the list.
            egui::TextEdit::singleline(&mut state.filter)
                .hint_text(t::filter_hint())
                .desired_width(220.0),
        );
        crate::diag::ui_rect(&format!("{REGION}filter"), filter.rect);
        let all = ui.add_enabled(!prefs.is_empty(), egui::Button::new(t::reset_all()));
        crate::diag::ui_rect(&format!("{REGION}reset_all"), all.rect);
        if all.clicked() {
            prefs.reset_all();
            state.clash = None;
            trace_reset("*");
        }
    });
}

/// The clash question or the last refusal, above the list.
fn notice(ui: &mut Ui, catalog: &Catalog, prefs: &mut ShortcutPrefs, state: &mut PageState) {
    if let Some((command, chord, held_by)) = state.clash.clone() {
        let notice = egui_shell::theme::Theme::of(ui.ctx()).palette.notice;
        ui.label(
            RichText::new(t::clash(
                &chord,
                catalog.label(&held_by),
                catalog.label(&command),
            ))
            .color(notice),
        );
        ui.horizontal(|ui| {
            let yes = ui.button(t::reassign());
            crate::diag::ui_rect(&format!("{REGION}reassign"), yes.rect);
            let no = ui.button(t::cancel());
            crate::diag::ui_rect(&format!("{REGION}cancel"), no.rect);
            if yes.clicked() {
                reassign(catalog, prefs, &command, &chord, &held_by);
                state.clash = None;
            } else if no.clicked() {
                state.clash = None;
            }
        });
    } else if let Some((chord, why)) = &state.refused {
        let line = match why {
            KeptFor::Viewing => t::kept_for_viewing(chord),
            KeptFor::Canvas => t::kept_for_canvas(chord),
        };
        ui.label(RichText::new(line).small().weak());
    }
}

/// Draw one group's rows; whether any passed the filter.
fn group_rows(
    ui: &mut Ui,
    group: &CatalogGroup,
    prefix: &str,
    keymap: &BTreeMap<String, String>,
    prefs: &mut ShortcutPrefs,
    state: &mut PageState,
) -> bool {
    let needle = state.filter.trim().to_lowercase();
    let shown: Vec<&(String, String)> = group
        .commands
        .iter()
        .filter(|(_, label)| needle.is_empty() || label.to_lowercase().contains(&needle))
        .collect();
    if shown.is_empty() {
        return false;
    }
    ui.add_space(8.0);
    // An explicit colour: bare `.strong()` is pale on pale (DEFECTS.md D11).
    ui.label(
        RichText::new(&group.title)
            .strong()
            .color(ui.visuals().text_color()),
    );
    egui::Grid::new(("settings-keys", prefix, &group.title))
        .num_columns(3)
        .striped(true)
        .show(ui, |ui| {
            for (command, label) in shown {
                row(ui, command, label, prefix, keymap, prefs, state);
                ui.end_row();
            }
        });
    true
}

fn row(
    ui: &mut Ui,
    command: &str,
    label: &str,
    prefix: &str,
    keymap: &BTreeMap<String, String>,
    prefs: &mut ShortcutPrefs,
    state: &mut PageState,
) {
    widgets::label(ui, label);
    let chords = chords_of(keymap, command);
    let shown = if chords.is_empty() {
        t::no_key().to_owned()
    } else {
        chords.join(crate::text::shortcuts::chord_separator())
    };
    ui.label(shown);
    ui.horizontal(|ui| {
        let waiting = state.capturing.as_deref() == Some(command);
        let text = if waiting {
            t::press_a_key()
        } else {
            t::change()
        };
        let change = ui.selectable_label(waiting, text);
        publish(ui, prefix, "change", command, &change);
        if change.clicked() {
            state.capturing = (!waiting).then(|| command.to_owned());
            state.clash = None;
            state.refused = None;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("shortcut-capture command={command} on={}", !waiting)
            });
        }
        let off = ui.add_enabled(!chords.is_empty(), egui::Button::new(t::off()));
        publish(ui, prefix, "off", command, &off);
        if off.clicked() {
            prefs.set(command, &[]);
            trace_set(command, &[]);
        }
        let reset = ui.add_enabled(prefs.get(command).is_some(), egui::Button::new(t::reset()));
        publish(ui, prefix, "reset", command, &reset);
        if reset.clicked() {
            prefs.reset(command);
            trace_reset(command);
        }
    });
}

fn publish(ui: &Ui, prefix: &str, what: &str, command: &str, response: &egui::Response) {
    crate::diag::ui_rect_visible(
        &format!("{REGION}{prefix}{what}.{command}"),
        response.rect,
        ui.clip_rect(),
    );
}

/// While a command waits, take the next key pressed and decide what it does.
fn capture(ui: &mut Ui, catalog: &Catalog, prefs: &mut ShortcutPrefs, state: &mut PageState) {
    let Some(command) = state.capturing.clone() else {
        return;
    };
    let Some((modifiers, key)) = ui.input_mut(take_chord) else {
        return;
    };
    if key == Key::Escape && !modifiers.any() {
        state.capturing = None;
        return;
    }
    let chord = format_chord(modifiers, key);
    if let Some(why) = catalog.kept_for(modifiers, key) {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("shortcut-refused chord={chord} why={}", why.word())
        });
        state.refused = Some((chord, why));
        return;
    }
    state.capturing = None;
    state.refused = None;
    let keymap = effective(&catalog.defaults, prefs);
    match holder(&keymap, &chord).filter(|h| *h != command) {
        Some(held_by) => {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("shortcut-clash command={command} chord={chord} holder={held_by}")
            });
            state.clash = Some((command, chord, held_by.to_owned()));
        }
        None => {
            let chords = [chord];
            prefs.set(&command, &chords);
            trace_set(&command, &chords);
        }
    }
}

/// Give `command` the clashing `chord` and take it from `held_by`, which keeps
/// its other keys.
fn reassign(
    catalog: &Catalog,
    prefs: &mut ShortcutPrefs,
    command: &str,
    chord: &str,
    held_by: &str,
) {
    let keymap = effective(&catalog.defaults, prefs);
    let wanted = parse_chord(chord);
    let rest: Vec<String> = chords_of(&keymap, held_by)
        .into_iter()
        .filter(|c| parse_chord(c) != wanted)
        .collect();
    prefs.set(held_by, &rest);
    trace_set(held_by, &rest);
    let chords = [chord.to_owned()];
    prefs.set(command, &chords);
    trace_set(command, &chords);
}

/// The first key pressed this frame, as a chord, with every key and text
/// event removed so nothing else in the window acts on it. Clipboard keys
/// arrive as clipboard events rather than key presses.
fn take_chord(input: &mut egui::InputState) -> Option<(Modifiers, Key)> {
    let shift = input.modifiers.shift;
    let found = input.events.iter().find_map(|e| match e {
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => Some((*modifiers, *key)),
        Event::Copy => Some((Modifiers::COMMAND, Key::C)),
        Event::Cut => Some((Modifiers::COMMAND, Key::X)),
        Event::Paste(_) if shift => Some((Modifiers::COMMAND | Modifiers::SHIFT, Key::V)),
        Event::Paste(_) => Some((Modifiers::COMMAND, Key::V)),
        _ => None,
    })?;
    input.events.retain(|e| {
        !matches!(
            e,
            Event::Key { .. } | Event::Text(_) | Event::Copy | Event::Cut | Event::Paste(_)
        )
    });
    Some(found)
}

fn trace_reset(command: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("shortcut-reset command={command}")
    });
}

fn trace_set(command: &str, chords: &[String]) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed.
        format!("shortcut-set command={command} chords={}", chords.join(","))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(key: Key, modifiers: Modifiers) -> egui::RawInput {
        egui::RawInput {
            events: vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }],
            modifiers,
            ..Default::default()
        }
    }

    fn catalog() -> Catalog {
        Catalog {
            defaults: [
                ("Ctrl+F".to_owned(), "edit.find".to_owned()),
                ("Ctrl+K".to_owned(), "tools.other".to_owned()),
            ]
            .into(),
            kept: vec![super::super::keyscatalog::Kept {
                key: Key::Home,
                modifiers: None,
                why: KeptFor::Viewing,
            }],
            ..Catalog::default()
        }
    }

    fn captured(input: egui::RawInput) -> (ShortcutPrefs, PageState) {
        let ctx = egui::Context::default();
        let mut prefs = ShortcutPrefs::default();
        let mut state = PageState {
            capturing: Some("edit.find".to_owned()),
            ..PageState::default()
        };
        let _ = ctx.run_ui(input, |ui| capture(ui, &catalog(), &mut prefs, &mut state));
        (prefs, state)
    }

    #[test]
    fn a_free_key_is_assigned_at_once() {
        let (prefs, state) = captured(press(Key::J, Modifiers::COMMAND));
        assert_eq!(prefs.get("edit.find"), Some(&["Ctrl+J".to_owned()][..]));
        assert!(state.capturing.is_none() && state.clash.is_none());
    }

    #[test]
    fn a_held_key_waits_for_reassign_and_changes_nothing() {
        let (prefs, state) = captured(press(Key::K, Modifiers::COMMAND));
        assert!(prefs.is_empty(), "a clash must not assign before Reassign");
        assert_eq!(
            state.clash,
            Some((
                "edit.find".to_owned(),
                "Ctrl+K".to_owned(),
                "tools.other".to_owned()
            ))
        );
    }

    #[test]
    fn reassign_takes_the_key_from_its_holder() {
        let mut prefs = ShortcutPrefs::default();
        reassign(&catalog(), &mut prefs, "edit.find", "Ctrl+K", "tools.other");
        let keymap = effective(&catalog().defaults, &prefs);
        assert_eq!(holder(&keymap, "Ctrl+K"), Some("edit.find"));
        assert!(chords_of(&keymap, "tools.other").is_empty());
    }

    #[test]
    fn a_kept_key_is_refused_and_capture_continues() {
        let (prefs, state) = captured(press(Key::Home, Modifiers::NONE));
        assert!(prefs.is_empty());
        assert_eq!(state.capturing.as_deref(), Some("edit.find"));
        assert_eq!(state.refused.map(|(_, why)| why), Some(KeptFor::Viewing));
    }

    #[test]
    fn escape_cancels_without_assigning() {
        let (prefs, state) = captured(press(Key::Escape, Modifiers::NONE));
        assert!(prefs.is_empty() && state.capturing.is_none());
    }
}
