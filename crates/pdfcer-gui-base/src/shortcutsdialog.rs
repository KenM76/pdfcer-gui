//! # `shortcutsdialog` — every keyboard chord, derived from the keymap that
//! dispatches them
//!
//! Design and rationale: `docs/modules/pdfcer-gui-base/shortcutsdialog.md`.

use std::collections::BTreeMap;

use egui::Ui;
use egui_shell::CommandRegistry;
use egui_shell::manifest::Keymap;

use crate::text::shortcuts as t;

/// The region this dialog publishes for its body.
pub const REGION_BODY: &str = "dialog:shortcuts"; // ui-text-exempt: trace region name, never displayed
/// The region the list publishes, so a driven check can assert it drew rows
/// rather than an empty window.
pub const REGION_LIST: &str = "shortcuts.list"; // ui-text-exempt: trace region name, never displayed

/// The Shortcuts window's live state.
pub struct ShortcutsDialog;

impl ShortcutsDialog {
    /// Open it.
    #[must_use]
    pub const fn open() -> Self {
        Self
    }

    /// Draw it. Returns `false` when it should close.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        keymap: Option<&Keymap>,
        registry: &CommandRegistry,
    ) -> bool {
        // ITS OWN OS WINDOW. A shortcut list is read *while working*, which
        // is the one thing a window trapped inside the application frame makes
        // impossible: it covers the surface whose shortcuts are being looked
        // up.
        //
        // It is also what bounds the body's vertical `ScrollArea`. A scroll
        // area inside a container whose size is decided by its content has no
        // fixed height to fill, so it asks for its full content height, the
        // container grows to fit, and it asks for more. An **OS window always
        // has a finite size** — the platform gives it one — so the scroll area
        // is bounded on every frame and that loop has nowhere to start. The
        // declared size below is an opening bid, not a fix.
        let (frame, ()) = crate::dialoghost::Host::new(
            "shortcuts", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(420.0, 480.0),
            egui::vec2(320.0, 220.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            body(ui, keymap, registry);
        });
        !frame.closed
    }
}

/// One command, and every chord bound to it.
struct Row {
    /// The command's own label, from the registry.
    label: String,
    /// Every chord that resolves to it, in the keymap's own order.
    chords: Vec<String>,
}

/// The window body.
fn body(ui: &mut Ui, keymap: Option<&Keymap>, registry: &CommandRegistry) {
    ui.label(t::intro());
    ui.add_space(8.0);

    let Some(keymap) = keymap else {
        // Reachable: `PdfcerApp::shell` is an `Option`, and a build whose
        // manifest failed to load has no keymap — in which case no chord works
        // either, so an empty list would be *accurate* and unhelpfully so.
        ui.label(t::no_keymap());
        return;
    };

    let (rows, dropped) = rows_from(keymap, registry);
    if rows.is_empty() {
        ui.label(t::none_bound());
        return;
    }

    // Capped and floored, the idiom five dialogs now share. See
    // `pdfcer_gui::dialogs::about`'s header for why the floor is not optional: a
    // negative `max_height` lays the rows out into nothing, silently.
    const FOOTER_RESERVE: f32 = 40.0;
    const LIST_FLOOR: f32 = 48.0;
    egui::ScrollArea::vertical()
        .id_salt("shortcuts-list")
        .max_height((ui.available_height() - FOOTER_RESERVE).max(LIST_FLOOR))
        .show(ui, |ui| {
            egui::Grid::new("shortcuts-grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for row in &rows {
                        // The chord first: it is what the operator is scanning
                        // for, and a left column of keys is the shape every
                        // reference of this kind uses.
                        ui.label(row.chords.join(t::chord_separator()));
                        ui.label(&row.label);
                        ui.end_row();
                    }
                });
            // Published AFTER the grid, so `ui.min_rect()` covers what was
            // laid out.
            //
            // **A region published at the top of a closure describes the
            // closure's starting point, not its content.** Over a `Ui` whose
            // contents do not exist yet the rect is empty, so the region
            // reports zero height however many rows follow it, and a driven
            // check reads a correctly drawn window as a title over an empty
            // band. A region that can only ever report zero cannot detect the
            // thing it was added to detect, and is worse than no region at all,
            // because it produces a confident false failure.
            crate::diag::ui_rect(REGION_LIST, ui.min_rect());
        });

    // Traced so a driven check can assert the two numbers rather than the
    // pixels — and the SECOND one is the assertion worth having.
    //
    // `dropped` counts chords naming a command this build did not register. On
    // a full build it must be **zero**: every chord in the manifest's keymap
    // names a command the registry has, which is the manifest-versus-registry
    // agreement nothing else checks end to end. A non-zero here on the shipped
    // build means the two have drifted, and the symptom in the application is a
    // key that silently does nothing.
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed
        format!("shortcuts-listed commands={} dropped={dropped}", rows.len())
    });

    ui.add_space(4.0);
    ui.weak(t::derived_note(rows.len()));
    if dropped > 0 {
        // Disclosed, not absorbed. See the module header — a stripped build
        // genuinely has fewer shortcuts, and an operator comparing two
        // installations is entitled to know which.
        ui.weak(t::dropped_note(dropped));
    }
}

/// Fold the keymap into one row per command, and count the chords dropped.
fn rows_from(keymap: &Keymap, registry: &CommandRegistry) -> (Vec<Row>, usize) {
    let mut by_command: BTreeMap<&str, Row> = BTreeMap::new();
    let mut dropped = 0usize;

    for (chord, id) in &keymap.0 {
        // R8: a chord naming a command this build did not register describes a
        // capability that is not compiled in. Counted, never listed.
        let Some(command) = registry.get(id) else {
            dropped = dropped.saturating_add(1);
            continue;
        };
        by_command
            .entry(id.as_str())
            .or_insert_with(|| Row {
                label: command.label.clone(),
                chords: Vec::new(),
            })
            .chords
            .push(chord.clone());
    }

    (by_command.into_values().collect(), dropped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::commands::{Command, HandlerToken};

    /// A registry holding exactly the named ids.
    fn registry(ids: &[&str]) -> CommandRegistry {
        let mut reg = CommandRegistry::new();
        // The token is irrelevant to every assertion here — this window
        // invokes nothing — so it is the index, which keeps each command
        // distinct without inventing a meaning for the number.
        for (n, id) in ids.iter().enumerate() {
            reg.register(Command::new(*id, *id, HandlerToken::new(n as u64)))
                .expect("a fresh id");
        }
        reg
    }

    /// A keymap from `(chord, id)` pairs.
    fn keymap(pairs: &[(&str, &str)]) -> Keymap {
        Keymap(
            pairs
                .iter()
                .map(|(c, i)| ((*c).to_owned(), (*i).to_owned()))
                .collect(),
        )
    }

    /// **Every bound chord is listed.** `DEFECTS.md` D5, asserted.
    #[test]
    fn every_bound_chord_appears() {
        let map = keymap(&[
            ("Ctrl+O", "file.open"),
            ("Ctrl+S", "file.save_copy"),
            ("Ctrl+Z", "edit.undo"),
            ("[", "pages.rotate_left"),
        ]);
        let reg = registry(&[
            "file.open",
            "file.save_copy",
            "edit.undo",
            "pages.rotate_left",
        ]);
        let (rows, dropped) = rows_from(&map, &reg);

        let listed: Vec<&str> = rows
            .iter()
            .flat_map(|r| r.chords.iter().map(String::as_str))
            .collect();
        for chord in map.0.keys() {
            assert!(
                listed.contains(&chord.as_str()),
                "{chord} is bound and is not in the reference — which is D5"
            );
        }
        assert_eq!(dropped, 0);
    }

    /// **Two chords on one command are ONE row.**
    #[test]
    fn a_command_with_two_chords_is_one_row_naming_both() {
        let map = keymap(&[("Ctrl+Y", "edit.redo"), ("Ctrl+Shift+Z", "edit.redo")]);
        let reg = registry(&["edit.redo"]);
        let (rows, _) = rows_from(&map, &reg);

        assert_eq!(
            rows.len(),
            1,
            "one command, one row: {rows:?}",
            rows = rows.len()
        );
        assert_eq!(rows[0].chords.len(), 2, "both chords must be named");
        assert!(rows[0].chords.iter().any(|c| c == "Ctrl+Y"));
        assert!(rows[0].chords.iter().any(|c| c == "Ctrl+Shift+Z"));
    }

    /// **A chord for an unregistered command is dropped AND counted.**
    #[test]
    fn a_chord_for_a_missing_command_is_dropped_and_counted() {
        let map = keymap(&[
            ("Ctrl+O", "file.open"),
            ("Ctrl+K", "tools.not_in_this_build"),
        ]);
        let reg = registry(&["file.open"]);
        let (rows, dropped) = rows_from(&map, &reg);

        assert_eq!(rows.len(), 1, "only the registered command is listed");
        assert_eq!(dropped, 1, "and the other is counted rather than forgotten");
        assert!(
            !rows.iter().any(|r| r.chords.iter().any(|c| c == "Ctrl+K")),
            "a key that does nothing must not be promised"
        );
    }

    /// The label comes from the registry, not from a table here.
    #[test]
    fn the_label_is_the_registrys_own() {
        let map = keymap(&[("Ctrl+O", "file.open")]);
        let mut reg = CommandRegistry::new();
        reg.register(Command::new(
            "file.open",
            "Open a drawing",
            HandlerToken::new(0),
        ))
        .expect("a fresh id");
        let (rows, _) = rows_from(&map, &reg);
        assert_eq!(rows[0].label, "Open a drawing");
    }
}
