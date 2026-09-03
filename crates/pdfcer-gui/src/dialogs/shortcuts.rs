//! # `dialogs::shortcuts` — every keyboard chord, derived from the keymap that
//! dispatches them
//!
//! ## The gap this closes, and the instruction it followed
//!
//! `file.shortcuts` was registered, drawn on File ▸ pdfcer, and inert. Its
//! scaffold entry did not merely say *blocked* — it carried the design, from
//! `SALVAGE.md`'s row for the old shell's 7,912-line `ui_text.rs`:
//!
//! > Fix `shortcuts_reference()` — it **omits six live bindings**
//! > (`DEFECTS.md` D5) — and **derive it from the keyboard map so it cannot
//! > drift again.** Salvaging it unfixed would import D5.
//!
//! So nothing was salvaged. This window has **no list in it**.
//!
//! ## ★ D5 is not fixed here; it is made unrepresentable
//!
//! `DEFECTS.md` D5 is a *hand-maintained reference* disagreeing with the actual
//! bindings, and the reason it happened is the reason it would happen again:
//! two places state the same fact, one of them dispatches and the other is
//! prose, and only the first is exercised by using the program.
//!
//! Every row below is produced by iterating **the same `Keymap` that
//! `app::keyboard::commands` resolves a keystroke against**. A binding that
//! exists is listed because listing is a fold over the bindings; a listing that
//! is wrong is not a thing this window can produce. There is no second copy to
//! drift.
//!
//! That is the difference between fixing a defect and closing the class of it,
//! and it is the same move `canvas::snap`'s tolerance and
//! `dialogs::scale`'s unit list both made after their own copies drifted.
//!
//! ## ★ An unregistered command is DROPPED, not shown greyed — R8
//!
//! A chord whose command is not in the registry names a capability this build
//! does not have — the strippable-capability convention, where a feature's
//! absence is expressed by its command not being registered. Listing it would
//! promise a key that does nothing, which is the placeholder rule applied to
//! prose.
//!
//! **The count of dropped chords is disclosed** rather than silently absorbed,
//! because *"this build has fewer shortcuts than the manifest declares"* is a
//! true and surprising fact about a stripped build, and an operator comparing
//! two installations needs it. That is `SHELL_FRAMEWORK.md` §5b's
//! `CapabilityAbsent` posture, arriving in a window rather than in a log.
//!
//! ## Why it is application-scoped
//!
//! A keyboard reference is meaningful with nothing open — it is one of the two
//! things a new operator reaches for before opening a file, the other being
//! About. It therefore sits beside [`super::about`] in the group
//! [`super::DialogsState`] does not close when a document closes.

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
///
/// ★ **It holds nothing.** Every row is derived from the keymap and the
/// registry on each frame, which is the whole point — see the module header.
/// A cached list would be a second copy, and a second copy is D5.
///
/// The unit struct exists because [`super::DialogsState`]'s idiom is one
/// `Option<T>` per dialog, whose `Some` *is* the open state. A `bool` would
/// work and would be the one dialog here shaped differently.
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
        // ★ ITS OWN OS WINDOW as of 2026-08-21. A shortcut list is read
        // *while working*, which is the one thing a window trapped inside the
        // application frame makes impossible: it covers the surface whose
        // shortcuts are being looked up.
        //
        // ★★ THE HEIGHT NOTE THAT STOOD HERE IS ANSWERED BY CONSTRUCTION NOW,
        // and it is worth saying why rather than deleting it. It read: *"a
        // window with no declared height sizes itself to its content, and this
        // body is a vertical `ScrollArea` … the scroll area asks for its full
        // content height, the window grows to fit, the scroll area gets more
        // room and asks for more."* That loop needs a container whose size is
        // decided by its content. An **OS window always has a finite size** —
        // the platform gives it one — so the scroll area is bounded on every
        // frame and the loop has nowhere to start. The declared size below is
        // an opening bid, not a fix.
        let (frame, ()) = crate::dialogs::host::Host::new(
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
///
/// ★ **Chords are plural**, and that is not a nicety: `edit.redo` is bound to
/// both `Ctrl+Y` and `Ctrl+Shift+Z`, deliberately, and a reference showing one
/// of them would be a reference that is *incomplete in exactly the way D5 was*
/// — quietly, on the binding an operator's other application taught them.
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
    // `crate::dialogs::about`'s header for why the floor is not optional: a
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
            // ★★ Published AFTER the grid, which is the fix for a region that
            // always reported zero height.
            //
            // It used to be the first statement in this closure, over
            // `ui.min_rect()` — and at that moment nothing had been laid out,
            // so the rect was empty. A driven check asserting the list had
            // drawn read `0.0 pt high with 20 commands folded into it` and
            // reported the window as a title over an empty band. The window was
            // fine; the instrumentation was measuring a `Ui` before its
            // contents existed.
            //
            // Worth stating as a rule rather than a fix: **a region published
            // at the top of a closure describes the closure's starting point,
            // not its content.** A region that can only ever report zero cannot
            // detect the thing it was added to detect, and it is worse than no
            // region at all, because it produces a confident false failure.
            crate::diag::ui_rect(REGION_LIST, ui.min_rect());
        });

    // ★ Traced so a driven check can assert the two numbers rather than the
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
        // ★ Disclosed, not absorbed. See the module header — a stripped build
        // genuinely has fewer shortcuts, and an operator comparing two
        // installations is entitled to know which.
        ui.weak(t::dropped_note(dropped));
    }
}

/// Fold the keymap into one row per command, and count the chords dropped.
///
/// ## ★ Grouped by command, not by chord
///
/// A keymap is `chord → id`, and rendering it directly would give `Ctrl+Y` and
/// `Ctrl+Shift+Z` two rows saying the same thing — which reads as two features
/// rather than as one with two keys. Inverting it is what makes the *plural*
/// case legible, and the plural case is the one D5 got wrong.
///
/// ## Why the order is the command id's
///
/// `BTreeMap` over the id, so `edit.*` sorts together, `file.*` together, and
/// the list is stable across runs and machines. Sorting by chord would
/// interleave every tab's bindings and put `[` next to `]` next to `Alt+Down`,
/// which is alphabetical and useless — an operator looking for *"the shortcut
/// for rotating"* is thinking about the verb, not the key.
///
/// It is deliberately **not** the ribbon's tab order, which would be truer to
/// the operator's mental model and would require this window to know the
/// manifest's tab list. A window that reads the keymap and the registry and
/// nothing else is a window that cannot disagree with either.
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

    /// ★ **Every bound chord is listed.** This is D5, asserted.
    ///
    /// The defect was a hand-maintained reference omitting six live bindings.
    /// The listing is now a fold over the bindings, so the property is
    /// structural — and this test is what says so out loud, because a future
    /// reader looking at a window full of shortcuts has no way to tell a
    /// derived list from a copied one.
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

    /// ★ **Two chords on one command are ONE row.**
    ///
    /// `edit.redo` really is bound twice, deliberately, and a reference showing
    /// one of them would be incomplete in exactly D5's way — quietly, on the
    /// binding the operator's other application taught them.
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

    /// ★ **A chord for an unregistered command is dropped AND counted.**
    ///
    /// R8: a command that is not registered is a capability this build does not
    /// have, so listing its key would promise a keystroke that does nothing.
    /// The count is what stops the omission being silent — a stripped build
    /// genuinely has fewer shortcuts, and that is worth a sentence rather than
    /// a shrug.
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
    ///
    /// The second half of the same argument: a hand-written label would drift
    /// from the ribbon's the day one of them was reworded, and an operator
    /// reading two different names for one command has to work out that they
    /// are one command.
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
