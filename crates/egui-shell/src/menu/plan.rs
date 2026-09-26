//! Everything a context menu decides **before it touches `egui`** — which
//! items survive, whether the menu opens at all, and how wide it is.
//!
//! # Why this is a separate module with no `egui` in it
//!
//! The same reason [`crate::ribbon::plan`] is: the invariants that matter
//! here are arithmetic and set logic, and a test that has to open a window
//! to assert one is a test that will be skipped on CI, run slowly, and
//! measure the toolkit rather than the rule.
//!
//! There is a sharper reason for menus specifically. The rule *"a menu
//! with nothing to offer does not open"* is a claim about a **decision
//! taken before any drawing happens** — it cannot be observed after the
//! fact, because the whole point is that nothing was drawn. Putting that
//! decision in a pure function ([`offers_anything`]) makes it assertable
//! directly, exhaustively, and without simulating a right-click.
//!
//! # The four rules this module holds
//!
//! ## 1. A command that does not exist is *absent*, not greyed
//!
//! `GUI_ROADMAP.md`'s no-placeholders rule (P3) and
//! `SHELL_FRAMEWORK.md` §4's *disclosed skip* meet here, and they say
//! opposite-sounding things about two different situations that are easy
//! to confuse:
//!
//! | Situation | What the operator sees | Why |
//! |---|---|---|
//! | The command **is registered** and its [`crate::commands::Enable`] predicate is false | the row, **greyed**, with its tooltip | It exists, it is simply not applicable *right now*. Removing it would make the menu's shape change under the operator's hand and hide the fact that the action is possible at all. |
//! | The command **is not registered** in this build | nothing at all | It does not exist. A greyed row for a command that will never be enabled is a placeholder, and a placeholder is a promise the build cannot keep. |
//!
//! [`resolve`] implements exactly that: an unregistered id is dropped and
//! disclosed through [`crate::verify`]; a registered-but-disabled command
//! survives as a [`Slot::Command`] with `enabled: false`.
//!
//! ## 2. A menu with no *enabled* item does not open
//!
//! Right-clicking something that has nothing to offer must do **nothing**
//! — not flash an empty box, and not open a menu of five greyed rows.
//!
//! The second half of that is the interesting one, and it is why
//! [`offers_anything`] tests `enabled` rather than mere presence. A menu
//! of nothing but disabled rows is strictly worse than no menu: it costs a
//! click to dismiss, it moves the pointer, and it teaches the operator
//! that right-clicking here is useless — the exact lesson that then
//! prevents them discovering the menu when it *does* have something. A
//! menu that simply does not appear says the same thing in no time at all.
//!
//! (A [`Slot::Custom`] counts as an offer. The shell cannot evaluate an
//! application's own control, and refusing to open a menu whose only item
//! is one would silently delete a control the application asked for. The
//! application decides; the shell does not guess.)
//!
//! ## 3. `visible_when` HIDES a row; `Enable` only greys it
//!
//! [`crate::manifest::Item`] is the shared vocabulary of ribbon groups and
//! menus, and its `visible_when` field means *"the item is drawn **only**
//! while the condition holds"* — R9's disappearing half, as against
//! [`crate::commands::Enable`]'s greying half. [`resolve`] honours it, and
//! must: a row greyed where the document said hidden is the R9 inversion the
//! field exists to prevent, and it is invisible to any test that asks the
//! model (`command_ids()`) rather than the resolution.
//!
//! The predicate is the ribbon's own — [`crate::ribbon::sizing::visible`],
//! called rather than restated, so a menu and a band can never disagree about
//! what `visible_when` means. That one shared reader is the point: the same
//! `Item` type serves both surfaces, so a field honoured on one and ignored on
//! the other reads, at every call site, exactly like a field that is honoured.
//!
//! Rule 4 below then does the rest: a row removed here can leave a separator
//! with nothing above it, and that is exactly the stale-document shape
//! [`collapse`] already handles.
//!
//! ## 4. Separators are punctuation, and punctuation collapses
//!
//! A separator's meaning is entirely relational — it says *"the things
//! above and the things below are different kinds"*. Once rule 1 has
//! removed some commands, a document that read
//!
//! ```text
//! Cut · Copy · ── · Rasterize · ── · Delete
//! ```
//!
//! can become `── · ── · Delete` in a build without the editing commands,
//! which draws two rules above one item and looks like a rendering fault.
//! [`collapse`] therefore drops leading and trailing separators and
//! collapses runs, which makes a menu's punctuation a *consequence* of
//! what survived rather than of what was written.
//!
//! Design and rationale: `docs/modules/egui-shell/menu/plan.md`.

use crate::commands::{Command, CommandRegistry, ConditionSet};
use crate::manifest::Item;
use crate::ribbon::selected_condition;

use super::shortcut::Shortcuts;

/// One resolved menu row: what the renderer will actually draw.
#[derive(Debug, Clone)]
pub enum Slot<'a> {
    /// A command that exists in this build.
    Command {
        /// The registration: label, tooltip, icon key, handler token.
        command: &'a Command,
        /// Whether its [`crate::commands::Enable`] predicate holds right
        /// now. `false` draws the row greyed — see rule 1.
        enabled: bool,
        /// Whether the command is currently *on*, via the ribbon's
        /// [`selected_condition`] convention. A checkable menu item and a
        /// toggled band control are the same state, expressed the same
        /// way, so a toggle cannot disagree between the two surfaces.
        selected: bool,
        /// The chord to show, right-aligned, if the keymap binds one.
        shortcut: Option<&'a str>,
    },
    /// A horizontal rule. Presentation only; never counts as an offer.
    Separator,
    /// Something the application draws itself.
    Custom {
        /// The application-defined kind.
        kind: &'a str,
        /// The application-defined payload, if the document carried one.
        payload: Option<&'a str>,
    },
}

impl Slot<'_> {
    /// Whether this row is something the operator could act on.
    ///
    /// A disabled command is **not** — see rule 2 in the module header.
    #[must_use]
    pub fn is_actionable(&self) -> bool {
        match self {
            Slot::Command { enabled, .. } => *enabled,
            Slot::Custom { .. } => true,
            Slot::Separator => false,
        }
    }

    /// Whether this row is a separator.
    #[must_use]
    pub fn is_separator(&self) -> bool {
        matches!(self, Slot::Separator)
    }
}

/// Resolve a menu's items against the registry, the conditions and the
/// keymap, then collapse the punctuation.
#[must_use]
pub fn resolve<'a>(
    items: &'a [Item],
    registry: &'a CommandRegistry,
    conditions: &ConditionSet,
    shortcuts: &'a Shortcuts,
    context: &str,
) -> Vec<Slot<'a>> {
    let mut slots = Vec::with_capacity(items.len());
    for item in items {
        // Rule 3. **Before** the registry lookup, deliberately: an item the
        // conditions hide is not a row at all, so an unregistered id behind a
        // false `visible_when` should not also emit a `menu-skipped-unknown-command`
        // disclosure. Hidden is hidden; the skip channel is for a command the
        // document asked for and this build does not have.
        if !crate::ribbon::sizing::visible(item, conditions) {
            continue;
        }
        match item {
            Item::Separator => slots.push(Slot::Separator),
            Item::Custom { kind, payload, .. } => slots.push(Slot::Custom {
                kind,
                payload: payload.as_deref(),
            }),
            Item::Command { id, .. } => match registry.get(id) {
                Some(command) => slots.push(Slot::Command {
                    command,
                    enabled: command.is_enabled(conditions),
                    selected: conditions.is_set(&selected_condition(&command.id)),
                    shortcut: shortcuts.get(id),
                }),
                None => {
                    // Absent, not greyed — rule 1. Disclosed, because an
                    // undisclosed skip is indistinguishable from a
                    // rendering fault, which is the lesson
                    // `crate::verify`'s header records.
                    crate::verify::event("menu-skipped-unknown-command")
                        .kv("context", context)
                        .kv("id", id)
                        .emit();
                }
            },
        }
    }
    collapse(slots)
}

/// Drop leading and trailing separators and collapse runs of them.
#[must_use]
pub fn collapse(slots: Vec<Slot<'_>>) -> Vec<Slot<'_>> {
    let mut out: Vec<Slot<'_>> = Vec::with_capacity(slots.len());
    for slot in slots {
        if slot.is_separator() {
            // A separator is kept only if something real precedes it and
            // it is not repeating the previous rule. Whether anything
            // *follows* it is not knowable here, so a trailing run is
            // removed afterwards.
            if out.last().is_some_and(|s| !s.is_separator()) {
                out.push(slot);
            }
            continue;
        }
        out.push(slot);
    }
    while out.last().is_some_and(Slot::is_separator) {
        out.pop();
    }
    out
}

/// **Whether this menu should open at all.**
#[must_use]
pub fn offers_anything(slots: &[Slot<'_>]) -> bool {
    slots.iter().any(Slot::is_actionable)
}

// ---------------------------------------------------------------------
// The icon column
// ---------------------------------------------------------------------

/// What one command row does with its icon slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSlot {
    /// No slot is laid out at all: the row is `[label] [grow] [chord]` and
    /// its label starts at the button's left padding.
    ///
    /// The state of **every** row in a menu where no command names an
    /// icon, which is most of them.
    Absent,
    /// A slot the width of one icon is laid out and **nothing is drawn
    /// into it**.
    ///
    /// The row's own command has no icon key, but a sibling in the same
    /// menu does, so the label indents to keep the label column straight.
    /// Nothing is painted — no placeholder, no outline, no dimmed
    /// stand-in — which is R9 in the small: a command with no icon must
    /// not leave a mark that reads as a *missing* picture.
    Blank,
    /// A slot is laid out and the application's icon painter is asked to
    /// fill it with the command's key.
    Glyph,
}

impl IconSlot {
    /// Whether a slot is laid out at all — i.e. whether this row spends
    /// the width.
    #[must_use]
    pub fn is_reserved(self) -> bool {
        !matches!(self, Self::Absent)
    }

    /// Whether the painter is called for this row.
    #[must_use]
    pub fn draws(self) -> bool {
        matches!(self, Self::Glyph)
    }
}

/// **Whether this menu reserves an icon column.**
#[must_use]
pub fn reserves_icon_column(slots: &[Slot<'_>]) -> bool {
    slots.iter().any(|slot| match slot {
        Slot::Command { command, .. } => command.icon.is_some(),
        Slot::Separator | Slot::Custom { .. } => false,
    })
}

/// What one row does with its icon slot, given the menu-wide decision.
#[must_use]
pub fn icon_slot(reserved: bool, has_key: bool) -> IconSlot {
    match (reserved, has_key) {
        (false, _) => IconSlot::Absent,
        (true, false) => IconSlot::Blank,
        (true, true) => IconSlot::Glyph,
    }
}

// ---------------------------------------------------------------------
// Width
// ---------------------------------------------------------------------

/// **The minimum gap between the label column and the chord column.**
pub const COLUMN_GAP: f32 = 24.0;

/// The narrowest a menu body may be.
pub const MIN_BODY_WIDTH: f32 = 96.0;

/// The widest a menu body may be before labels start truncating.
pub const MAX_BODY_WIDTH: f32 = 420.0;

/// The measured pieces of one command row.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RowWidths {
    /// The icon slot, or `0.0` if this row lays none out.
    ///
    /// Note the wording: **this row**, not *this command*. A row in a
    /// menu that reserves the column spends the width whether or not its
    /// own command has a key ([`IconSlot::Blank`]), and a measurement that
    /// asked about the command instead would under-estimate every
    /// icon-less row in a menu that has icons — which is the dangerous
    /// direction, because the widest row decides the body width and an
    /// under-measured widest row truncates its own label. Callers get the
    /// answer from [`icon_slot`], which is the same function the renderer
    /// draws from.
    pub icon: f32,
    /// The label.
    pub label: f32,
    /// The chord, or `0.0` if the command has no binding.
    pub shortcut: f32,
}

impl RowWidths {
    /// Whether this row draws an icon.
    #[must_use]
    pub fn has_icon(&self) -> bool {
        self.icon > 0.0
    }

    /// Whether this row draws a chord.
    ///
    /// A chord that measures nothing is treated as absent, which is also
    /// the right answer for a keymap entry bound to the empty string.
    #[must_use]
    pub fn has_shortcut(&self) -> bool {
        self.shortcut > 0.0
    }

    /// How many `egui` atoms the row is built from.
    #[must_use]
    pub fn atom_count(&self) -> usize {
        1 + usize::from(self.has_icon()) + if self.has_shortcut() { 2 } else { 0 }
    }

    /// The full width this row wants, gaps and padding included.
    #[must_use]
    pub fn total(&self, atom_gap: f32, padding: f32) -> f32 {
        let gaps = atom_gap * (self.atom_count().saturating_sub(1)) as f32;
        let column = if self.has_shortcut() { COLUMN_GAP } else { 0.0 };
        padding + self.icon + self.label + self.shortcut + gaps + column
    }
}

/// The width a menu body will be laid out at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyWidth {
    /// The width, in points, clamped into
    /// [`MIN_BODY_WIDTH`]..=[`MAX_BODY_WIDTH`].
    pub points: f32,
    /// Whether at least one row wanted more than [`MAX_BODY_WIDTH`], and
    /// will therefore have its label truncated.
    ///
    /// Carried out rather than left implicit so the renderer can disclose
    /// it: a control silently rendering at less than the size it asked for
    /// is exactly the kind of degradation that stays invisible until
    /// somebody screenshots it.
    pub truncating: bool,
}

/// The width a menu body should be laid out at, given what every row
/// wants.
#[must_use]
pub fn body_width(row_totals: &[f32]) -> BodyWidth {
    let widest = row_totals.iter().copied().fold(0.0_f32, f32::max);
    BodyWidth {
        points: widest.clamp(MIN_BODY_WIDTH, MAX_BODY_WIDTH),
        truncating: widest > MAX_BODY_WIDTH,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Enable, HandlerToken};
    use crate::manifest::Keymap;

    fn registry() -> CommandRegistry {
        let mut r = CommandRegistry::new();
        r.register_all([
            Command::new("edit.cut", "Cut", HandlerToken::new(1))
                .enabled_when("selection.any")
                .with_tooltip("Move the selection to the clipboard"),
            Command::new("edit.copy", "Copy", HandlerToken::new(2)).enabled_when("selection.any"),
            Command::new("edit.paste", "Paste", HandlerToken::new(3)).enabled_when("clipboard.any"),
            Command::new("view.single", "Single page", HandlerToken::new(4))
                .with_enable(Enable::Always),
        ])
        .expect("distinct ids");
        r
    }

    fn shortcuts() -> Shortcuts {
        Shortcuts::from_keymap(&Keymap(
            [
                ("Ctrl+X".to_owned(), "edit.cut".to_owned()),
                ("Ctrl+C".to_owned(), "edit.copy".to_owned()),
            ]
            .into_iter()
            .collect(),
        ))
    }

    /// **`visible_when` takes the row AWAY, and `Enable` only greys it.**
    #[test]
    fn a_hidden_item_leaves_no_row_while_a_disabled_one_leaves_a_greyed_one() {
        let items = [
            // Hidden: its condition is not in the set.
            Item::command("edit.copy").shown_when("selection.delete_permitted"),
            // Visible (no condition) and disabled (`selection.any` is unset).
            Item::command("edit.cut"),
            // Visible and enabled.
            Item::command("view.single"),
        ];
        let registry = registry();
        let shortcuts = shortcuts();
        let slots = resolve(
            &items,
            &registry,
            &ConditionSet::new(),
            &shortcuts,
            "canvas.markup",
        );

        assert_eq!(
            slots.len(),
            2,
            "a `visible_when` that does not hold must remove the row, not grey it: \
             greying says `not right now`, and this field is how a menu says `never here`"
        );
        assert!(matches!(
            &slots[0],
            Slot::Command { command, enabled, .. } if command.id == "edit.cut" && !enabled
        ));
        assert!(matches!(
            &slots[1],
            Slot::Command { command, enabled, .. } if command.id == "view.single" && *enabled
        ));

        // …and the same document with the condition SET draws all three.
        let showing = ConditionSet::new().with("selection.delete_permitted");
        let slots = resolve(&items, &registry, &showing, &shortcuts, "canvas.markup");
        assert_eq!(slots.len(), 3);
        assert!(matches!(
            &slots[0],
            Slot::Command { command, .. } if command.id == "edit.copy"
        ));
    }

    /// **A hidden row takes its separator with it**, through rule 4.
    #[test]
    fn hiding_a_group_collapses_the_rule_that_introduced_it() {
        let items = [
            Item::command("edit.copy").shown_when("never.set"),
            Item::Separator,
            Item::command("view.single"),
        ];
        let registry = registry();
        let shortcuts = shortcuts();
        let slots = resolve(
            &items,
            &registry,
            &ConditionSet::new(),
            &shortcuts,
            "canvas.markup",
        );
        assert_eq!(slots.len(), 1, "a leading rule must not survive the hiding");
        assert!(!slots[0].is_separator());
    }

    /// **An unregistered command is absent; a disabled one is present and
    /// greyed.**
    #[test]
    fn an_unknown_command_is_absent_and_a_disabled_one_is_greyed() {
        let items = [
            Item::command("edit.cut"),       // registered, disabled here
            Item::command("edit.telepathy"), // not registered at all
            Item::command("view.single"),    // registered, enabled
        ];
        let registry = registry();
        let shortcuts = shortcuts();
        let slots = resolve(
            &items,
            &registry,
            &ConditionSet::new(),
            &shortcuts,
            "canvas.object",
        );

        assert_eq!(slots.len(), 2, "the unregistered id must leave no row");
        match &slots[0] {
            Slot::Command {
                command, enabled, ..
            } => {
                assert_eq!(command.id, "edit.cut");
                assert!(
                    !enabled,
                    "a registered command whose predicate is false is DRAWN, greyed — \
                     removing it would make the menu change shape under the operator's hand"
                );
            }
            other => panic!("expected a command row, got {other:?}"),
        }
        match &slots[1] {
            Slot::Command {
                command, enabled, ..
            } => {
                assert_eq!(command.id, "view.single");
                assert!(enabled);
            }
            other => panic!("expected a command row, got {other:?}"),
        }
        assert!(
            !slots.iter().any(|s| matches!(
                s,
                Slot::Command { command, .. } if command.id == "edit.telepathy"
            )),
            "no row may name a command this build does not have"
        );
    }

    /// **A menu whose every command is disabled offers nothing.**
    #[test]
    fn a_menu_of_only_disabled_commands_offers_nothing() {
        let items = [
            Item::command("edit.cut"),
            Item::Separator,
            Item::command("edit.copy"),
            Item::command("edit.paste"),
        ];
        let registry = registry();
        let shortcuts = shortcuts();

        let nothing_true = resolve(
            &items,
            &registry,
            &ConditionSet::new(),
            &shortcuts,
            "canvas.object",
        );
        assert_eq!(
            nothing_true.len(),
            4,
            "all three commands still resolve, and the rule between them survives because it still separates two real rows"
        );
        assert!(
            !offers_anything(&nothing_true),
            "three greyed rows are worse than no menu: they cost a click to \
             dismiss and teach the operator that right-clicking here is useless"
        );

        let something_true = resolve(
            &items,
            &registry,
            &ConditionSet::new().with("selection.any"),
            &shortcuts,
            "canvas.object",
        );
        assert!(
            offers_anything(&something_true),
            "one enabled command is enough to open the menu"
        );
    }

    /// An empty menu, and a menu of nothing but separators, both offer
    /// nothing. The second is the shape a stale document degenerates into.
    #[test]
    fn separators_alone_are_not_an_offer() {
        let registry = registry();
        let shortcuts = shortcuts();
        let conditions = ConditionSet::new().with("selection.any");

        let nothing: [Item; 0] = [];
        let rules_only = [Item::Separator, Item::Separator];
        let stale = [Item::command("edit.telepathy"), Item::Separator];
        for (items, context) in [
            (&nothing[..], "empty"),
            (&rules_only[..], "rules.only"),
            (&stale[..], "stale"),
        ] {
            let slots = resolve(items, &registry, &conditions, &shortcuts, context);
            assert!(
                !offers_anything(&slots),
                "`{context}` must offer nothing; got {slots:?}"
            );
        }
    }

    /// A custom item counts as an offer: the shell cannot evaluate an
    /// application's own control and must not silently delete it.
    #[test]
    fn a_custom_item_is_an_offer_the_shell_cannot_second_guess() {
        let registry = registry();
        let shortcuts = shortcuts();
        let items = [Item::command("edit.cut"), Item::custom("colour_swatch")];
        let slots = resolve(
            &items,
            &registry,
            &ConditionSet::new(), // `edit.cut` is disabled
            &shortcuts,
            "canvas.object",
        );
        assert!(
            offers_anything(&slots),
            "the only actionable row is the application's own, and the shell \
             has no way to know it is not actionable"
        );
    }

    /// **Punctuation collapses to match what survived.**
    ///
    #[test]
    fn separators_collapse_around_what_is_left() {
        let registry = registry();
        let shortcuts = shortcuts();
        let items = [
            Item::Separator,                 // leading
            Item::command("edit.telepathy"), // gone
            Item::Separator,                 // now also leading
            Item::command("view.single"),
            Item::Separator,
            Item::command("edit.astrology"), // gone
            Item::Separator,                 // now a doubled rule
            Item::command("edit.cut"),
            Item::Separator, // trailing
            Item::Separator, // trailing
        ];
        let slots = resolve(
            &items,
            &registry,
            &ConditionSet::new().with("selection.any"),
            &shortcuts,
            "canvas.object",
        );

        let shape: Vec<bool> = slots.iter().map(Slot::is_separator).collect();
        assert_eq!(
            shape,
            [false, true, false],
            "expected `view.single · ── · edit.cut`; got {slots:?}"
        );
    }

    /// The collapse rule stated as a property: no result ever begins with,
    /// ends with, or contains two adjacent separators — whatever it was
    /// handed.
    #[test]
    fn a_collapsed_list_never_begins_ends_or_doubles_on_a_rule() {
        let cmd = Command::new("x", "X", HandlerToken::new(1));
        let real = || Slot::Command {
            command: &cmd,
            enabled: true,
            selected: false,
            shortcut: None,
        };
        let cases: Vec<Vec<Slot<'_>>> = vec![
            vec![],
            vec![Slot::Separator],
            vec![Slot::Separator, Slot::Separator],
            vec![Slot::Separator, real()],
            vec![real(), Slot::Separator],
            vec![real(), Slot::Separator, Slot::Separator, real()],
            vec![
                Slot::Separator,
                real(),
                Slot::Separator,
                real(),
                Slot::Separator,
            ],
        ];
        for case in cases {
            let out = collapse(case.clone());
            assert!(
                !out.first().is_some_and(Slot::is_separator),
                "leading rule survived: {case:?}"
            );
            assert!(
                !out.last().is_some_and(Slot::is_separator),
                "trailing rule survived: {case:?}"
            );
            assert!(
                !out.windows(2)
                    .any(|w| w[0].is_separator() && w[1].is_separator()),
                "doubled rule survived: {case:?}"
            );
        }
    }

    /// The chord comes from the keymap and lands on the right row.
    #[test]
    fn a_row_carries_the_chord_the_keymap_binds() {
        let registry = registry();
        let shortcuts = shortcuts();
        let items = [Item::command("edit.cut"), Item::command("view.single")];
        let slots = resolve(
            &items,
            &registry,
            &ConditionSet::new().with("selection.any"),
            &shortcuts,
            "canvas.object",
        );
        assert!(matches!(
            slots[0],
            Slot::Command {
                shortcut: Some("Ctrl+X"),
                ..
            }
        ));
        assert!(matches!(slots[1], Slot::Command { shortcut: None, .. }));
    }

    /// A toggle reads its state through the ribbon's own convention, so a
    /// checkable menu item and a toggled band control cannot disagree.
    #[test]
    fn a_toggle_uses_the_ribbons_selected_condition() {
        let registry = registry();
        let shortcuts = shortcuts();
        let conditions = ConditionSet::new().with(selected_condition("view.single"));
        let items = [Item::command("view.single")];
        let slots = resolve(&items, &registry, &conditions, &shortcuts, "canvas");
        assert!(matches!(slots[0], Slot::Command { selected: true, .. }));
    }

    /// **The atom count charges for the invisible `grow` atom.**
    #[test]
    fn the_atom_count_charges_for_the_invisible_grow_atom() {
        let plain = RowWidths {
            icon: 0.0,
            label: 40.0,
            shortcut: 0.0,
        };
        assert_eq!(plain.atom_count(), 1, "label alone");

        let with_icon = RowWidths {
            icon: 16.0,
            ..plain
        };
        assert_eq!(with_icon.atom_count(), 2);

        let with_chord = RowWidths {
            shortcut: 30.0,
            ..plain
        };
        assert_eq!(
            with_chord.atom_count(),
            3,
            "label + grow + chord — the grow atom is invisible and still billed"
        );

        let both = RowWidths {
            icon: 16.0,
            label: 40.0,
            shortcut: 30.0,
        };
        assert_eq!(both.atom_count(), 4);
    }

    /// A row with a chord reserves [`COLUMN_GAP`]; a row without one does
    /// not pay for a column it has not got.
    #[test]
    fn only_a_row_with_a_chord_pays_for_the_column_gap() {
        let gap = 4.0;
        let padding = 8.0;

        let plain = RowWidths {
            icon: 0.0,
            label: 40.0,
            shortcut: 0.0,
        };
        assert!((plain.total(gap, padding) - 48.0).abs() < f32::EPSILON);

        let chorded = RowWidths {
            shortcut: 30.0,
            ..plain
        };
        // 8 padding + 40 label + 30 chord + 2 gaps + 24 column gap.
        assert!(
            (chorded.total(gap, padding) - (8.0 + 40.0 + 30.0 + 8.0 + COLUMN_GAP)).abs() < 0.01
        );
        assert!(
            chorded.total(gap, padding) > plain.total(gap, padding) + COLUMN_GAP,
            "the chord column must be paid for, or it lands on top of the label"
        );
    }

    /// **The widest row decides the menu's width, and it is clamped both
    /// ways.**
    #[test]
    fn the_widest_row_decides_the_width_within_the_clamp() {
        let w = body_width(&[120.0, 240.0, 80.0]);
        assert!((w.points - 240.0).abs() < f32::EPSILON);
        assert!(!w.truncating);

        let narrow = body_width(&[10.0, 12.0]);
        assert!(
            (narrow.points - MIN_BODY_WIDTH).abs() < f32::EPSILON,
            "a menu thinner than the floor reads as a clickable tooltip"
        );
        assert!(!narrow.truncating);

        let huge = body_width(&[100.0, MAX_BODY_WIDTH + 1.0]);
        assert!((huge.points - MAX_BODY_WIDTH).abs() < f32::EPSILON);
        assert!(
            huge.truncating,
            "a clamped menu must say so, or the truncation is invisible until \
             somebody screenshots it"
        );

        // Never zero, even with nothing to measure.
        assert!((body_width(&[]).points - MIN_BODY_WIDTH).abs() < f32::EPSILON);
    }

    // -----------------------------------------------------------------
    // The icon column
    // -----------------------------------------------------------------

    /// A registry with one command that names an icon and two that do not.
    fn icon_registry() -> CommandRegistry {
        let mut r = CommandRegistry::new();
        r.register_all([
            Command::new("view.zoom_fit", "Fit page", HandlerToken::new(10))
                .with_enable(Enable::Always)
                .with_icon("fit-page"),
            Command::new("view.zoom_actual", "Actual size", HandlerToken::new(11))
                .with_enable(Enable::Always),
            Command::new("edit.reflow", "Reflow block", HandlerToken::new(12))
                .with_enable(Enable::Always),
        ])
        .expect("distinct ids");
        r
    }

    /// Build slots for `ids` against `registry`, all enabled, no chords.
    fn slots_for<'a>(registry: &'a CommandRegistry, ids: &[&str]) -> Vec<Slot<'a>> {
        ids.iter()
            .map(|id| Slot::Command {
                command: registry.get(id).expect("registered"),
                enabled: true,
                selected: false,
                shortcut: None,
            })
            .collect()
    }

    /// **ONE row with a glyph gives the whole menu a column; none gives it
    /// nothing.**
    #[test]
    fn one_row_with_a_glyph_gives_the_whole_menu_a_column() {
        let registry = icon_registry();

        let mixed = slots_for(&registry, &["view.zoom_actual", "view.zoom_fit"]);
        assert!(
            reserves_icon_column(&mixed),
            "a menu where any row names an icon has a column, or its labels zig-zag"
        );

        let bare = slots_for(&registry, &["view.zoom_actual", "edit.reflow"]);
        assert!(
            !reserves_icon_column(&bare),
            "a menu whose commands have no icons must lay out exactly as it did \
             before the column existed — no slot, no indent, no extra width"
        );

        let punctuation = vec![
            Slot::Separator,
            Slot::Custom {
                kind: "colour-swatch",
                payload: None,
            },
        ];
        assert!(
            !reserves_icon_column(&punctuation),
            "a separator has no columns and a custom row is the application's; \
             neither may vote a column into existence"
        );
    }

    /// **An icon-less row beside an icon row spends the width and paints
    /// nothing.**
    #[test]
    fn an_icon_less_row_in_a_reserving_menu_spends_the_width_and_paints_nothing() {
        assert_eq!(icon_slot(true, true), IconSlot::Glyph);
        assert_eq!(icon_slot(true, false), IconSlot::Blank);
        assert_eq!(icon_slot(false, false), IconSlot::Absent);
        assert_eq!(
            icon_slot(false, true),
            IconSlot::Absent,
            "unreachable, and it must degrade rather than panic"
        );

        assert!(
            IconSlot::Blank.is_reserved(),
            "the blank is what keeps the label column straight"
        );
        assert!(
            !IconSlot::Blank.draws(),
            "a row with no icon key must leave NO mark — not a box, not an outline, \
             not a dimmed stand-in. R9: an absent capability renders nothing."
        );
        assert!(IconSlot::Glyph.is_reserved() && IconSlot::Glyph.draws());
        assert!(!IconSlot::Absent.is_reserved() && !IconSlot::Absent.draws());
    }

    /// **A blank slot costs exactly what a glyph slot costs, and an absent
    /// one costs nothing.**
    #[test]
    fn a_blank_icon_slot_costs_exactly_what_a_glyph_costs() {
        const SLOT: f32 = 16.0;
        const GAP: f32 = 4.0;
        const PADDING: f32 = 12.0;

        let glyph = RowWidths {
            icon: SLOT,
            label: 100.0,
            shortcut: 0.0,
        };
        // Same label, no icon key, but the menu reserves — so `icon_slot`
        // says `Blank` and the measurement spends the slot width anyway.
        let blank = RowWidths {
            icon: if icon_slot(true, false).is_reserved() {
                SLOT
            } else {
                0.0
            },
            ..glyph
        };
        assert_eq!(
            blank.total(GAP, PADDING),
            glyph.total(GAP, PADDING),
            "a blank slot and a glyph slot are the same width; measuring them \
             differently is how the widest row comes to truncate its own label"
        );
        assert_eq!(blank.atom_count(), glyph.atom_count());

        let absent = RowWidths {
            icon: if icon_slot(false, false).is_reserved() {
                SLOT
            } else {
                0.0
            },
            ..glyph
        };
        assert_eq!(
            glyph.total(GAP, PADDING) - absent.total(GAP, PADDING),
            SLOT + GAP,
            "a menu with no icons must cost exactly nothing: the slot and the one \
             atom gap it buys are the entire difference"
        );

        let reserved_body = body_width(&[glyph.total(GAP, PADDING), blank.total(GAP, PADDING)]);
        let plain_body = body_width(&[absent.total(GAP, PADDING), absent.total(GAP, PADDING)]);
        assert_eq!(reserved_body.points - plain_body.points, SLOT + GAP);
        assert!(!reserved_body.truncating && !plain_body.truncating);
    }
}
