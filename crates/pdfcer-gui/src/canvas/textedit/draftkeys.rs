//! # `canvas::textedit::draftkeys` — the commands a text edit routes itself
//!
//! While a draft is open the global chords yield (`app::keyboard::commands`),
//! so the draft answers [`TYPING`] on its own — on the chords the live keymap
//! gives them, which the operator may have changed. [`publish`] copies those
//! chords into egui memory each frame; [`bound`] reads them, falling back to
//! the program's keys when nothing was published.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/draftkeys.md`.

use std::sync::Arc;

use egui::{Context, Id, Key, Modifiers};
use egui_shell::manifest::Keymap;
use pdfcer_gui_base::keychord::parse_chord;
use pdfcer_gui_base::settingspages::keyscatalog::same_modifiers;

/// The commands a draft answers, by registered id.
// ui-text-exempt: registered command ids, never displayed.
pub const TYPING: [&str; 7] = [
    "edit.select_all",
    "edit.undo",
    "edit.redo",
    "file.save",
    "format.bold",
    "format.italic",
    "format.underline",
];

/// The program's own chords for [`TYPING`], used before a keymap is published.
// ui-text-exempt: chord spellings and command ids, never displayed.
const BUILT_IN: [(&str, &str); 8] = [
    ("Ctrl+A", "edit.select_all"),
    ("Ctrl+Z", "edit.undo"),
    ("Ctrl+Y", "edit.redo"),
    ("Ctrl+Shift+Z", "edit.redo"),
    ("Ctrl+S", "file.save"),
    ("Ctrl+B", "format.bold"),
    ("Ctrl+I", "format.italic"),
    ("Ctrl+U", "format.underline"),
];

type Bindings = Arc<Vec<(Modifiers, Key, &'static str)>>;

fn memory_id() -> Id {
    Id::new("pdfcer-textedit-draft-keys")
}

/// Store the chords `keymap` gives [`TYPING`], when they differ from what is
/// stored.
pub fn publish(ctx: &Context, keymap: &Keymap) {
    let fresh: Vec<(Modifiers, Key, &'static str)> = keymap
        .iter()
        .filter_map(|(chord, command)| {
            let id = TYPING.iter().find(|t| **t == command)?;
            let (m, key) = parse_chord(chord)?;
            Some((m, key, *id))
        })
        .collect();
    let id = memory_id();
    let same = ctx.data(|d| d.get_temp::<Bindings>(id).is_some_and(|b| *b == fresh));
    if !same {
        ctx.data_mut(|d| d.insert_temp(id, Arc::new(fresh)));
    }
}

/// The [`TYPING`] command `key` under `modifiers` reaches. Only a chord with
/// Ctrl or Alt counts: anything else is typed as text.
#[must_use]
pub fn bound(ctx: &Context, key: Key, modifiers: Modifiers) -> Option<&'static str> {
    if !(modifiers.command || modifiers.ctrl || modifiers.alt) {
        return None;
    }
    let matches = |m: Modifiers, k: Key| k == key && same_modifiers(m, modifiers);
    match ctx.data(|d| d.get_temp::<Bindings>(memory_id())) {
        Some(bindings) => bindings
            .iter()
            .find(|(m, k, _)| matches(*m, *k))
            .map(|(_, _, id)| *id),
        None => BUILT_IN.iter().find_map(|(chord, id)| {
            let (m, k) = parse_chord(chord)?;
            matches(m, k).then_some(*id)
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keymap(pairs: &[(&str, &str)]) -> Keymap {
        Keymap(
            pairs
                .iter()
                .map(|(c, i)| ((*c).to_owned(), (*i).to_owned()))
                .collect(),
        )
    }

    #[test]
    fn the_program_keys_answer_before_anything_is_published() {
        let ctx = Context::default();
        assert_eq!(bound(&ctx, Key::B, Modifiers::COMMAND), Some("format.bold"));
        assert_eq!(
            bound(&ctx, Key::Z, Modifiers::COMMAND | Modifiers::SHIFT),
            Some("edit.redo")
        );
    }

    #[test]
    fn a_published_rebinding_moves_the_command_and_frees_the_old_key() {
        let ctx = Context::default();
        publish(
            &ctx,
            &keymap(&[("Ctrl+M", "format.bold"), ("Ctrl+F", "edit.find")]),
        );
        assert_eq!(bound(&ctx, Key::M, Modifiers::COMMAND), Some("format.bold"));
        assert_eq!(bound(&ctx, Key::B, Modifiers::COMMAND), None);
        assert_eq!(
            bound(&ctx, Key::F, Modifiers::COMMAND),
            None,
            "not a typing command"
        );
    }

    #[test]
    fn a_key_without_ctrl_or_alt_is_text() {
        let ctx = Context::default();
        publish(&ctx, &keymap(&[("B", "format.bold")]));
        assert_eq!(bound(&ctx, Key::B, Modifiers::NONE), None);
    }

    #[test]
    fn the_built_in_table_matches_the_shell_keymap() {
        let shell = crate::shell::manifest::built_in();
        let live = shell.keymap.expect("the built-in shell has a keymap");
        for (chord, id) in BUILT_IN {
            assert_eq!(
                live.get(chord),
                Some(id),
                "{chord} is not `{id}` in the shell"
            );
        }
    }
}
