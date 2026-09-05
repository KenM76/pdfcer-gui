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
//! # The three rules this module holds
//!
//! ## 1. ★ A command that does not exist is *absent*, not greyed
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
//! ## 2. ★ A menu with no *enabled* item does not open
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
//! ## 3. Separators are punctuation, and punctuation collapses
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

use crate::commands::{Command, CommandRegistry, ConditionSet};
use crate::manifest::Item;
use crate::ribbon::selected_condition;

use super::shortcut::Shortcuts;

/// One resolved menu row: what the renderer will actually draw.
///
/// Borrows from the registry and the [`Shortcuts`] index rather than
/// copying, because this is built once per menu open and dropped at the
/// end of the frame.
///
/// `PartialEq` is deliberately **not** derived: a `Slot` borrows a
/// [`Command`], and `Command` holds an `Enable` that may be a closure,
/// which has no meaningful equality. Tests match on shape instead, which
/// is what they actually want to assert.
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
///
/// `context` is used only to name the menu in a disclosure, so a log line
/// says *which* menu referred to a command that is not there.
///
/// See rule 1 in the module header for the unregistered-versus-disabled
/// distinction, which is the whole reason this function exists rather than
/// the renderer walking `items` directly.
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
///
/// Rule 3 in the module header. Separate from [`resolve`] so it can be
/// asserted on hand-built input, including the shapes a real document
/// would have to be perverse to produce but a *stale* one produces
/// routinely.
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
///
/// Rule 2 in the module header. `true` iff at least one row is something
/// the operator could act on: an enabled command, or a custom item the
/// application owns.
#[must_use]
pub fn offers_anything(slots: &[Slot<'_>]) -> bool {
    slots.iter().any(Slot::is_actionable)
}

// ---------------------------------------------------------------------
// The icon column
// ---------------------------------------------------------------------

/// What one command row does with its icon slot.
///
/// Three states rather than two, because "this row has no icon" is not one
/// answer — it is two, and they differ by **what the row beside it is
/// doing**. See [`icon_slot`] for the rule and the argument.
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
    ///
    /// [`Self::Blank`] and [`Self::Glyph`] cost exactly the same width;
    /// that is the entire point of `Blank`, and it is why measurement and
    /// drawing must both ask this question rather than each asking
    /// "does the command have an icon?".
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
///
/// `true` iff at least one command row that survived [`resolve`] names an
/// icon key. Separators and [`Slot::Custom`] rows never decide it: a
/// separator has no columns, and a custom row is drawn by the application,
/// which is the only party that knows whether its widget has a glyph.
///
/// # ★ Why the decision is per-menu and not per-row
///
/// A menu is a list of words and reads as one. The eye scans the left edge
/// of the labels, and the single thing that makes that scan cheap is that
/// the edge is *straight*. Deciding the slot per row breaks it: the rows
/// with icons indent and the rows without do not, so the label column
/// zig-zags and the reader loses the one alignment they were using.
///
/// The opposite extreme is worse still. Forcing a glyph onto every row —
/// inventing art for `Reflow block`, reusing something approximate for
/// `Close other documents` — buys a straight edge with a column of
/// pictures that do not mean anything, and a picture that does not mean
/// anything is read as one that does and then mis-read. That is the
/// wrong-picture refusal this project records at each command's own
/// registration, and it is not weakened by the column wanting to be full.
///
/// So: the **column** is a property of the menu, the **glyph** is a
/// property of the command, and a row whose command has no icon spends the
/// width and paints nothing ([`IconSlot::Blank`]). An indent is not a
/// hole — it is the same left margin every other row has.
///
/// # ★★ Why the empty menu is the common case and must stay free
///
/// A menu where *no* command has an icon reserves nothing
/// ([`IconSlot::Absent`] everywhere) and is laid out exactly as it was
/// before this rule existed. That matters more than it sounds: it means
/// the rule cannot make a plain menu wider, indent it, or move a single
/// pixel of it. Only a menu that has something to show pays for the
/// column.
#[must_use]
pub fn reserves_icon_column(slots: &[Slot<'_>]) -> bool {
    slots.iter().any(|slot| match slot {
        Slot::Command { command, .. } => command.icon.is_some(),
        Slot::Separator | Slot::Custom { .. } => false,
    })
}

/// What one row does with its icon slot, given the menu-wide decision.
///
/// `reserved` is [`reserves_icon_column`] for the whole menu; `has_key` is
/// whether *this* command names an icon. The pair is the entire rule, and
/// it is a free function over two `bool`s so it can be swept exhaustively
/// by a test rather than inspected inside a draw call.
///
/// | `reserved` | `has_key` | Result | Why |
/// |---|---|---|---|
/// | `false` | `false` | [`IconSlot::Absent`] | No row in this menu has a glyph; there is no column. |
/// | `false` | `true` | [`IconSlot::Absent`] | **Unreachable** — a row with a key is what makes `reserved` true. Answered rather than asserted: a panic here would turn a caller's bookkeeping slip into a crash in a popup, and `Absent` degrades to the pre-column layout, which is the safe direction. |
/// | `true` | `false` | [`IconSlot::Blank`] | A sibling has a glyph; indent to keep the label column straight, and paint nothing. |
/// | `true` | `true` | [`IconSlot::Glyph`] | Draw it. |
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
///
/// # Why this number exists at all, and why `egui` will not supply it
///
/// A menu row is drawn as an `egui::Button` whose atoms are
/// `[icon?] [label] [grow] [chord]`. The `grow` atom absorbs whatever
/// width is left over, which is what right-aligns the chord — and on the
/// **widest** row there is nothing left over, so the label and the chord
/// end up separated by one atom gap (4 pt) and read as a single run of
/// text: `Save a copy…Ctrl+Shift+S`.
///
/// `egui` cannot fix this for us because it sizes each button
/// independently; the fact that these buttons form two columns is a fact
/// about the menu, not about any one row. So the menu computes its own
/// width, adds this gap to the widest row, and every row — including that
/// one — then has at least this much clear space between the two columns.
///
/// 24 pt is roughly two capital widths at the body size: enough that the
/// eye reads two columns rather than one sentence, and not so much that a
/// three-item menu becomes a banner.
pub const COLUMN_GAP: f32 = 24.0;

/// The narrowest a menu body may be.
///
/// A menu narrower than this reads as a tooltip that happens to be
/// clickable. It also gives the pointer somewhere to be that is not
/// on top of a row, which matters for dismissing without invoking.
pub const MIN_BODY_WIDTH: f32 = 96.0;

/// The widest a menu body may be before labels start truncating.
///
/// A menu is a list of verbs, and past roughly this width it stops being
/// scannable and starts covering the thing that was right-clicked — which
/// for a canvas selection is the one piece of context the operator needs
/// while choosing. Beyond it the **label** gives way rather than the
/// position, for the same reason the ribbon's overflow affordance
/// truncates rather than moves: spending the shortfall on characters is
/// recoverable (the tooltip carries the full text), spending it on
/// position is not.
pub const MAX_BODY_WIDTH: f32 = 420.0;

/// The measured pieces of one command row.
///
/// Pure numbers, supplied by the renderer, which is the only party that
/// can ask `egui` how wide a string is. Keeping the *arithmetic* here
/// means it can be swept across hundreds of inputs by a unit test while
/// the measurement stays in one place next to the drawing.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RowWidths {
    /// The icon slot, or `0.0` if this row lays none out.
    ///
    /// ★ Note the wording: **this row**, not *this command*. A row in a
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
    ///
    /// Always the label; plus the icon slot if there is one; plus **two**
    /// if there is a chord, because the chord is preceded by the zero-size
    /// `grow` atom that does the right-aligning.
    ///
    /// This matters because `AtomLayout` charges `gap × (atoms − 1)`
    /// unconditionally — a zero-size atom still buys its gap
    /// (`egui-0.35.0/src/atomics/atom_layout.rs`) — so a width estimate
    /// that ignored the `grow` atom would under-estimate by one gap on
    /// every row that has a chord. Under-estimating is the dangerous
    /// direction: it is the one where the chord column runs off the right
    /// edge of the menu.
    #[must_use]
    pub fn atom_count(&self) -> usize {
        1 + usize::from(self.has_icon()) + if self.has_shortcut() { 2 } else { 0 }
    }

    /// The full width this row wants, gaps and padding included.
    ///
    /// `atom_gap` is `egui`'s `spacing().icon_spacing`, which is what
    /// `AtomLayout` puts between atoms; `padding` is the button's
    /// horizontal padding, both sides. Both come from the live style
    /// rather than from constants here, because both are theme-dependent
    /// and an estimate that disagreed with the toolkit would be wrong in a
    /// way no test of this module could see.
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
///
/// The **widest** row decides, because the rows form two columns and a
/// column is as wide as its widest member. Then the result is clamped:
/// never thinner than [`MIN_BODY_WIDTH`], never wider than
/// [`MAX_BODY_WIDTH`].
///
/// An empty menu yields [`MIN_BODY_WIDTH`] rather than zero. It should
/// never be drawn at all (see [`offers_anything`]), and a zero-width popup
/// is a much worse thing to fall back to than a small one.
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

    /// **★ An unregistered command is absent; a disabled one is present
    /// and greyed.**
    ///
    /// The two halves of rule 1, asserted together because the whole
    /// difficulty is telling them apart. Getting either one wrong produces
    /// a menu that looks plausible: drop the disabled row and the menu
    /// silently changes shape as the selection changes; keep the
    /// unregistered row and the build ships a placeholder for a command it
    /// does not have.
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

    /// **★ A menu whose every command is disabled offers nothing.**
    ///
    /// The decision behind "right-clicking something with nothing to offer
    /// does nothing". Asserted here, before any drawing, because after
    /// drawing there is by definition nothing to observe.
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

    /// **★ Punctuation collapses to match what survived.**
    ///
    /// Rule 3. The input here is what a real document turns into once a
    /// build without the editing commands has had its way with it: leading
    /// rules, a doubled rule where a command used to be, and a trailing
    /// rule.
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

    /// **★ The atom count charges for the invisible `grow` atom.**
    ///
    /// `AtomLayout` bills `gap × (atoms − 1)` whether or not an atom has
    /// any size, so the zero-width atom that right-aligns the chord costs
    /// a real gap. Forgetting it under-estimates every row that has a
    /// chord — and under-estimating is the direction in which the chord
    /// column runs off the right edge.
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

    /// **★ ONE row with a glyph gives the whole menu a column; none gives
    /// it nothing.**
    ///
    /// The two halves of the rule, asserted together because the whole
    /// difficulty is that they are the same question asked of different
    /// scopes. Get the first wrong and a menu with a single icon draws it
    /// against a ragged label column; get the second wrong and every plain
    /// menu in the application silently gains an indent it has no use for.
    ///
    /// The third case is the one a per-row implementation would never
    /// think to check: punctuation and application-drawn rows must not
    /// vote. A separator has no columns, and a `Custom` row is drawn by the
    /// application, which is the only party that knows whether its widget
    /// has a glyph — so a menu of nothing but those reserves nothing.
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

    /// **★★ An icon-less row beside an icon row spends the width and paints
    /// nothing.**
    ///
    /// The per-row half of the rule, swept over every input pair, because
    /// three of the four answers are load-bearing in a different direction:
    ///
    /// * `Blank` must be *reserved* (or the label column zig-zags) and must
    ///   not *draw* (or R9 is broken with a placeholder mark);
    /// * `Glyph` must be both;
    /// * `Absent` must be neither, which is what keeps a plain menu free.
    ///
    /// The fourth pair — no column, but this row has a key — is
    /// unreachable, since a row with a key is exactly what makes a menu
    /// reserve. It is *answered* rather than asserted against, and the
    /// answer degrades to the pre-column layout rather than panicking
    /// inside a popup.
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

    /// **★★★ A blank slot costs exactly what a glyph slot costs, and an
    /// absent one costs nothing.**
    ///
    /// The geometry the whole rule rests on. If a blank row were measured
    /// as though it had no slot, the arithmetic would under-estimate every
    /// icon-less row in a menu that has icons — and under-estimating is the
    /// dangerous direction, because the widest row sets the body width and
    /// an under-measured widest row truncates its own label. So the
    /// assertion is *equality*, not "close enough".
    ///
    /// The second half asserts the other direction: a menu with no icons
    /// must be unchanged, so its rows are narrower by exactly the slot plus
    /// the one atom gap the slot buys — and the body follows the rows
    /// uniformly, which is what keeps the label column straight rather than
    /// letting the widest row grow alone.
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
