//! # `settingspages::keyscatalog` — what the Keyboard shortcuts page lists
//!
//! The app builds a [`Catalog`] when the Settings window opens: every
//! registered command grouped by ribbon tab, the commands a text edit routes,
//! the program's own keymap, and the keys the program keeps for itself. The
//! page reads only this, so it needs no shell and no registry.
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/settingspages/keys.md`.

use std::collections::BTreeMap;

use egui::{Key, Modifiers};

use crate::keychord::parse_chord;

/// One titled run of commands, each `(command id, label)`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogGroup {
    pub title: String,
    pub commands: Vec<(String, String)>,
}

/// Why a key cannot be given to a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeptFor {
    /// Page navigation and zoom, bound outside the keymap.
    Viewing,
    /// Moving, deleting and leaving the selection on the page.
    Canvas,
}

impl KeptFor {
    /// The trace's word for this reason.
    #[must_use]
    pub const fn word(self) -> &'static str {
        // ui-text-exempt: trace tokens, never displayed.
        match self {
            Self::Viewing => "viewing",
            Self::Canvas => "canvas",
        }
    }
}

/// A key the program keeps. `modifiers: None` keeps it under every modifier;
/// otherwise Ctrl, Shift and Alt must match exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept {
    pub key: Key,
    pub modifiers: Option<Modifiers>,
    pub why: KeptFor,
}

/// Everything the page shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalog {
    /// Commands by ribbon tab, in tab order, then the rest.
    pub groups: Vec<CatalogGroup>,
    /// The commands a text edit routes itself, shown again under their own
    /// heading.
    pub typing: CatalogGroup,
    /// The program's keymap, chord → command, before the operator's lines.
    pub defaults: BTreeMap<String, String>,
    /// Keys no command may take.
    pub kept: Vec<Kept>,
}

impl Catalog {
    /// Why `key` under `modifiers` cannot be assigned, if it cannot.
    #[must_use]
    pub fn kept_for(&self, modifiers: Modifiers, key: Key) -> Option<KeptFor> {
        self.kept
            .iter()
            .find(|k| k.key == key && k.modifiers.is_none_or(|m| same_modifiers(m, modifiers)))
            .map(|k| k.why)
    }

    /// The label of `command`, or its id when the catalog does not list it.
    #[must_use]
    pub fn label<'a>(&'a self, command: &'a str) -> &'a str {
        self.groups
            .iter()
            .chain(std::iter::once(&self.typing))
            .flat_map(|g| g.commands.iter())
            .find(|(id, _)| id == command)
            .map_or(command, |(_, label)| label.as_str())
    }
}

/// Whether two modifier sets mean the same chord: Ctrl, Shift and Alt.
#[must_use]
pub fn same_modifiers(a: Modifiers, b: Modifiers) -> bool {
    (a.command || a.ctrl) == (b.command || b.ctrl) && a.shift == b.shift && a.alt == b.alt
}

/// The command `keymap` sends `chord` to, whatever spelling it uses.
#[must_use]
pub fn holder<'a>(keymap: &'a BTreeMap<String, String>, chord: &str) -> Option<&'a str> {
    let wanted = parse_chord(chord)?;
    keymap
        .iter()
        .find(|(k, _)| {
            parse_chord(k).is_some_and(|(m, key)| key == wanted.1 && same_modifiers(m, wanted.0))
        })
        .map(|(_, c)| c.as_str())
}

/// Every chord `keymap` sends to `command`, in chord order.
#[must_use]
pub fn chords_of(keymap: &BTreeMap<String, String>, command: &str) -> Vec<String> {
    keymap
        .iter()
        .filter(|(_, c)| *c == command)
        .map(|(k, _)| k.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Catalog {
        Catalog {
            kept: vec![
                Kept {
                    key: Key::Home,
                    modifiers: None,
                    why: KeptFor::Viewing,
                },
                Kept {
                    key: Key::Minus,
                    modifiers: Some(Modifiers::COMMAND),
                    why: KeptFor::Viewing,
                },
            ],
            ..Catalog::default()
        }
    }

    #[test]
    fn a_key_kept_under_every_modifier_is_kept_under_any() {
        let c = catalog();
        assert_eq!(
            c.kept_for(Modifiers::SHIFT, Key::Home),
            Some(KeptFor::Viewing)
        );
        assert_eq!(
            c.kept_for(Modifiers::NONE, Key::Home),
            Some(KeptFor::Viewing)
        );
    }

    #[test]
    fn a_key_kept_under_one_chord_is_free_under_another() {
        let c = catalog();
        assert_eq!(
            c.kept_for(Modifiers::COMMAND, Key::Minus),
            Some(KeptFor::Viewing)
        );
        assert_eq!(c.kept_for(Modifiers::NONE, Key::Minus), None);
        assert_eq!(
            c.kept_for(Modifiers::COMMAND | Modifiers::SHIFT, Key::Minus),
            None
        );
    }

    #[test]
    fn the_holder_is_found_through_another_spelling() {
        let keymap: BTreeMap<String, String> =
            [("Ctrl+[".to_owned(), "markup.send_backward".to_owned())].into();
        assert_eq!(
            holder(&keymap, "Ctrl+OpenBracket"),
            Some("markup.send_backward")
        );
        assert_eq!(holder(&keymap, "Ctrl+Shift+OpenBracket"), None);
    }
}
