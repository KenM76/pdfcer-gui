//! # `shell::manifest::keys` — the keymap the operator's shortcut lines make
//!
//! [`apply`] rebuilds the live keymap from the one the shell loaded: the
//! paste-order preference first, then the operator's `shortcut.` lines. It
//! starts from that snapshot every time, so a line removed by Reset gives the
//! program's key back. [`catalog`] is what Settings ▸ Keyboard shortcuts lists.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/shell/manifest/keys.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use egui::{Key, Modifiers};
use egui_shell::CommandRegistry;
use egui_shell::manifest::{Item, Keymap, Shell};
use pdfcer_gui_base::keychord::parse_chord;
use pdfcer_gui_base::settingspages::keyscatalog::{Catalog, CatalogGroup, Kept, KeptFor};

use crate::app::prefs::{PasteChords, ShortcutPrefs};

/// The keymap as loaded, before any preference touched it.
static LOADED: OnceLock<Keymap> = OnceLock::new();

/// Rebuild `shell`'s keymap from the loaded one, the paste `order` and the
/// operator's `shortcuts`.
pub fn apply(shell: &mut Shell, order: PasteChords, shortcuts: &ShortcutPrefs) {
    let Some(keymap) = shell.keymap.as_mut() else {
        return;
    };
    *keymap = LOADED.get_or_init(|| keymap.clone()).clone();
    super::apply_paste_chords(shell, order);
    if let Some(keymap) = shell.keymap.as_mut() {
        shortcuts.apply(&mut keymap.0);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("keymap-applied chords={}", keymap.len())
        });
    }
}

/// The program's keymap for `order`: the loaded one with only the paste
/// preference applied.
fn defaults(shell: &Shell, order: PasteChords) -> BTreeMap<String, String> {
    let mut copy = Shell::new();
    copy.keymap = Some(
        LOADED
            .get()
            .cloned()
            .or_else(|| shell.keymap.clone())
            .unwrap_or_default(),
    );
    super::apply_paste_chords(&mut copy, order);
    copy.keymap.map(|k| k.0).unwrap_or_default()
}

/// What Settings ▸ Keyboard shortcuts lists: every registered command under
/// the first ribbon tab that shows it, the rest under "Other".
#[must_use]
pub fn catalog(shell: &Shell, registry: &CommandRegistry, order: PasteChords) -> Catalog {
    let mut placed = BTreeSet::new();
    let mut groups = Vec::new();
    for tab in shell.all_tabs() {
        let mut commands = Vec::new();
        let ids = tab
            .groups()
            .iter()
            .flat_map(|g| g.items())
            .filter_map(|item| match item {
                Item::Command { id, .. } => Some(id.as_str()),
                _ => None,
            });
        for id in ids {
            if let Some(command) = registry.get(id)
                && placed.insert(id.to_owned())
            {
                commands.push((id.to_owned(), command.label.clone()));
            }
        }
        if !commands.is_empty() {
            let title = tab.label.clone().unwrap_or_else(|| tab.id.clone());
            groups.push(CatalogGroup { title, commands });
        }
    }
    let mut rest: Vec<(String, String)> = registry
        .iter()
        .filter(|c| !placed.contains(c.id.as_str()))
        .map(|c| (c.id.clone(), c.label.clone()))
        .collect();
    rest.sort_by(|a, b| a.1.cmp(&b.1));
    if !rest.is_empty() {
        groups.push(CatalogGroup {
            title: crate::text::settings::keys::group_other().to_owned(),
            commands: rest,
        });
    }
    let typing = CatalogGroup {
        title: crate::text::settings::keys::group_typing().to_owned(),
        commands: crate::canvas::textedit::draftkeys::TYPING
            .iter()
            .filter_map(|id| {
                registry
                    .get(id)
                    .map(|c| ((*id).to_owned(), c.label.clone()))
            })
            .collect(),
    };
    Catalog {
        groups,
        typing,
        defaults: defaults(shell, order),
        kept: kept(),
    }
}

/// The keys bound outside the keymap: the viewer's, and the canvas's.
fn kept() -> Vec<Kept> {
    let mut out: Vec<Kept> = crate::app::keyboard::OWNED
        .iter()
        .filter_map(|(key, spellings)| {
            let (m, _) = parse_chord(spellings.first()?)?;
            Some(Kept {
                key: *key,
                modifiers: m.any().then_some(m),
                why: KeptFor::Viewing,
            })
        })
        .collect();
    for key in [Key::Escape, Key::Delete, Key::Backspace] {
        out.push(Kept {
            key,
            modifiers: None,
            why: KeptFor::Canvas,
        });
    }
    let arrows = [
        Key::ArrowUp,
        Key::ArrowDown,
        Key::ArrowLeft,
        Key::ArrowRight,
    ];
    let unaltered = [
        Modifiers::NONE,
        Modifiers::SHIFT,
        Modifiers::COMMAND,
        Modifiers::COMMAND | Modifiers::SHIFT,
    ];
    for key in arrows {
        for m in unaltered {
            out.push(Kept {
                key,
                modifiers: Some(m),
                why: KeptFor::Canvas,
            });
        }
    }
    for m in [Modifiers::NONE, Modifiers::SHIFT] {
        out.push(Kept {
            key: Key::Tab,
            modifiers: Some(m),
            why: KeptFor::Canvas,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No key the program keeps is one the built-in keymap also binds.
    #[test]
    fn no_kept_key_is_a_built_in_binding() {
        let shell = super::super::built_in();
        let catalog = catalog(&shell, &CommandRegistry::new(), PasteChords::default());
        for (chord, command) in &catalog.defaults {
            let Some((m, key)) = parse_chord(chord) else {
                continue;
            };
            assert!(
                catalog.kept_for(m, key).is_none(),
                "`{chord}` is bound to `{command}` and also kept, so the page would refuse \
                 the program's own key"
            );
        }
    }

    /// Every chord the built-in keymap binds survives an empty set of lines.
    #[test]
    fn no_lines_leave_the_built_in_keymap_alone() {
        let mut shell = super::super::built_in();
        let before = shell.keymap.clone();
        let mut expected = shell.clone();
        super::super::apply_paste_chords(&mut expected, PasteChords::default());
        apply(
            &mut shell,
            PasteChords::default(),
            &ShortcutPrefs::default(),
        );
        assert!(before.is_some());
        assert_eq!(shell.keymap, expected.keymap);
    }
}
