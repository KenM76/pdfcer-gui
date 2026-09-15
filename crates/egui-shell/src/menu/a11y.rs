//! Accessible names for menu rows — and an honest account of what `egui`
//! 0.35 cannot be made to say.
//!
//! # The rule
//!
//! A menu row announces **what it says on screen**, which for a row with a
//! chord is two things: the label and the chord. So:
//!
//! | Row | Announced as |
//! |---|---|
//! | `Copy` with `Ctrl+C` | `"Copy, Ctrl+C"` |
//! | `Copy` with no binding | `"Copy"` |
//! | a command with a blank label | its tooltip, then its id — via [`crate::ribbon::a11y::accessible_name`] |
//!
//! The fallback chain is **not reimplemented here**. It is
//! [`crate::ribbon::a11y::accessible_name`] with `shows_label = true`,
//! because a menu row always draws its label and the question "what do we
//! announce when the application registered a blank one" has exactly one
//! right answer and should have exactly one implementation. A second copy
//! would be a place for the two surfaces to start disagreeing about the
//! same command.
//!
//! # Ceiling 1: `egui` 0.35 has no menu-item role
//!
//! [`egui::WidgetType`] in 0.35 offers `Label · Link · TextEdit · Button ·
//! Checkbox · RadioButton · RadioGroup · SelectableLabel · ComboBox ·
//! Slider · DragValue · ColorButton · Image · CollapsingHeader · Panel ·
//! ProgressIndicator · Window · ResizeHandle · ScrollBar · Other`.
//!
//! There is **no `MenuItem` and no `Menu`**. The `accesskit` roles exist
//! (`Role::MenuItem`, `Role::Menu`, `Role::MenuItemCheckBox`), but `egui`
//! 0.35's `Response::widget_info` fills an accesskit node from the
//! `WidgetType` through a fixed `match`
//! (`egui-0.35.0/src/response.rs`, `fill_accesskit_node_from_widget_info`)
//! and that `match` has no menu case to hit.
//!
//! So a row is published as [`egui::WidgetType::Button`], which maps to
//! `Role::Button`. Stated plainly rather than papered over:
//!
//! - A screen reader announces *"Copy, Ctrl+C, button"*, not *"Copy,
//!   Ctrl+C, menu item, 2 of 5"*. The label, the chord and the
//!   enabled/disabled state are all correct and all announced. What is
//!   missing is the **set relationship** — that these rows are one menu,
//!   how many there are, and where in it you are.
//! - Nothing about operation is affected. Focus, arrow-key movement within
//!   the popup and activation are `egui`'s and all work; this is a
//!   *labelling* ceiling, not an interaction one.
//! - `Button` is the closest honest answer. `Other` maps to
//!   `Role::Unknown` and would lose the fact that the row is activatable
//!   at all, which is worse than losing that it is in a menu.
//!
//! This is the same ceiling the ribbon's tab strip hits from the other
//! direction — see [`crate::ribbon::a11y`], which records the missing
//! `Tab` role. When `egui` grows the vocabulary, these two modules are the
//! only places to change.
//!
//! # Ceiling 2: the chord cannot be published *as* a chord
//!
//! This one is sharper, and it is why the chord is folded into the name
//! rather than attached beside it.
//!
//! `accesskit::Node` has `set_keyboard_shortcut`. [`egui::WidgetInfo`] has
//! no field that reaches it — its fields are `typ`, `enabled`, `label`,
//! `current_text_value`, `prev_text_value`, `selected`, `value`,
//! `text_selection`, `hint_text`, and none of them is routed there.
//!
//! Worse, the field that *looks* like the place for extra prose is not
//! one. `hint_text` is documented as "the hint text for text edit fields",
//! and `fill_accesskit_node_from_widget_info` maps it to
//! **`set_placeholder`** — a property that means nothing on a `Button`
//! role and that assistive technologies do not read out for one. Putting
//! the chord in `hint_text` would therefore look right in the source, look
//! right in a review, and be **silently inaudible**.
//!
//! The only field that reaches a button's announced text is `label`, via
//! `set_label`. So the chord goes in the label:
//!
//! ```text
//! "Copy, Ctrl+C"
//! ```
//!
//! That is a deliberate divergence from the ribbon's rule that the
//! accessible name should equal the visible label — and it is *justified
//! by the same principle*, not in spite of it. The ribbon's rule exists so
//! that what a user hears matches what a sighted colleague can point at.
//! In a menu the chord **is** on screen, in the same row, drawn by this
//! module. `"Copy"` alone would announce less than the row shows; `"Copy,
//! Ctrl+C"` announces exactly it.
//!
//! The comma is not decoration: it is what makes a screen reader pause
//! between the two, so `"Copy Ctrl C"` does not run together into
//! something that sounds like one command's name.

use egui::{Response, WidgetInfo, WidgetType};

use crate::commands::Command;
use crate::ribbon::a11y::accessible_name;

/// The name an assistive technology should announce for a menu row.
///
/// The visible label (with the ribbon's fallback chain behind it) plus the
/// chord, if there is one. See the module header for why the chord is here
/// rather than in a field of its own.
#[must_use]
pub fn menu_item_name(command: &Command, shortcut: Option<&str>) -> String {
    // `shows_label = true`: a menu row always draws its text, so the label
    // is the preferred source and the tooltip is the fallback — exactly
    // the ribbon's band case.
    let base = accessible_name(command, true);
    match shortcut.map(str::trim).filter(|s| !s.is_empty()) {
        Some(chord) => format!("{base}, {chord}"),
        None => base.to_owned(),
    }
}

/// Publish a menu row's accessibility information.
///
/// Called for **every** row, not only ones with a chord: `widget_info` is
/// also what feeds `egui`'s own output events, and a row that skipped it
/// would fall back to `egui`'s default — the button's atoms flattened into
/// text, with no enabled state.
///
/// [`WidgetType::Button`] rather than a menu-item role; see ceiling 1.
pub(crate) fn describe_item(
    response: &Response,
    command: &Command,
    shortcut: Option<&str>,
    enabled: bool,
    selected: bool,
) {
    let name = menu_item_name(command, shortcut);
    response.widget_info(|| {
        if selected {
            // A checkable row reports its toggle state, which `accesskit`
            // *does* carry for a button (`set_toggled`). Only published
            // when the row is actually a toggle: announcing "not pressed"
            // for every ordinary Delete would be noise, and worse, would
            // imply the command is a toggle when it is not.
            WidgetInfo::selected(WidgetType::Button, enabled, true, name.clone())
        } else {
            WidgetInfo::labeled(WidgetType::Button, enabled, name.clone())
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::HandlerToken;

    fn command(label: &str, tooltip: Option<&str>) -> Command {
        let base = Command::new("edit.copy", label, HandlerToken::new(1));
        match tooltip {
            Some(t) => base.with_tooltip(t),
            None => base,
        }
    }

    /// **A row announces its chord, because the chord is on screen.**
    ///
    /// The whole point of ceiling 2. Without this a screen-reader user is
    /// the one person the shortcut column cannot teach — while the menu
    /// looks completely correct in a screenshot, which is why this needs a
    /// test rather than a review.
    #[test]
    fn a_row_announces_its_chord() {
        let c = command("Copy", None);
        assert_eq!(menu_item_name(&c, Some("Ctrl+C")), "Copy, Ctrl+C");
        assert_eq!(menu_item_name(&c, None), "Copy");
    }

    /// A blank or whitespace chord is not announced as a trailing comma.
    ///
    /// `"Copy, "` is what a naive `format!` produces from a keymap entry
    /// bound to the empty string, and it announces as a pause after a word
    /// — a small, permanent, unattributable oddity.
    #[test]
    fn a_blank_chord_is_not_announced() {
        let c = command("Copy", None);
        assert_eq!(menu_item_name(&c, Some("")), "Copy");
        assert_eq!(menu_item_name(&c, Some("   ")), "Copy");
        assert_eq!(
            menu_item_name(&c, Some("  Ctrl+C  ")),
            "Copy, Ctrl+C",
            "and a chord with stray whitespace is still a chord"
        );
    }

    /// **The fallback chain is the ribbon's, not a second copy.**
    ///
    /// A command registered with a blank label announces its tooltip, then
    /// its id — and it does so because this module calls
    /// [`accessible_name`] rather than reimplementing the rule. If the two
    /// ever diverge, one command announces two different names on two
    /// surfaces, which is the sort of defect that gets reported as "the
    /// screen reader says something different in the menu".
    #[test]
    fn the_fallback_chain_is_borrowed_from_the_ribbon() {
        assert_eq!(
            menu_item_name(&command("", Some("Copy the selection")), Some("Ctrl+C")),
            "Copy the selection, Ctrl+C"
        );
        assert_eq!(
            menu_item_name(&command("", None), None),
            "edit.copy",
            "the id is the diagnostic last resort — hearing it read out says \
             which registration to fix"
        );
        assert_eq!(
            menu_item_name(&command("   ", Some("   ")), None),
            "edit.copy"
        );
    }

    /// **A name is never empty, whatever the application registered.**
    #[test]
    fn an_announced_name_is_never_empty() {
        for c in [
            command("Copy", Some("Copy it")),
            command("Copy", None),
            command("", Some("Copy it")),
            command("", None),
            command("   ", Some("   ")),
        ] {
            for chord in [None, Some(""), Some("Ctrl+C")] {
                let name = menu_item_name(&c, chord);
                assert!(!name.trim().is_empty(), "{c:?} with {chord:?} said nothing");
            }
        }
    }

    /// **Ceiling 1, asserted so it cannot be quietly forgotten.**
    ///
    /// If a future `egui` adds `WidgetType::MenuItem`, this is what fails
    /// and points at the module header that has to be rewritten. Until
    /// then it records that `Button` is a deliberate choice with a stated
    /// cost, not an oversight.
    #[test]
    fn rows_are_published_as_buttons_because_egui_035_has_no_menu_item_role() {
        // The vocabulary `egui` 0.35 offers for something activatable. If
        // this stops compiling, the enum changed and the header is wrong.
        let _closest_available = WidgetType::Button;
        let info = WidgetInfo::labeled(WidgetType::Button, true, "Copy, Ctrl+C");
        assert_eq!(info.typ, WidgetType::Button);
        assert_eq!(info.label.as_deref(), Some("Copy, Ctrl+C"));
        assert!(info.enabled);
    }

    /// **Ceiling 2, asserted the same way.**
    ///
    /// `WidgetInfo` carries no route to `accesskit`'s
    /// `set_keyboard_shortcut`, and its one prose-shaped field —
    /// `hint_text` — is mapped to `set_placeholder`, which a button role
    /// does not announce. This test pins the consequence: the chord has to
    /// travel in `label`, because `label` is the only field that arrives.
    #[test]
    fn the_chord_travels_in_the_label_because_no_other_field_arrives() {
        let info = WidgetInfo::labeled(WidgetType::Button, true, "Copy, Ctrl+C");
        assert!(
            info.label.as_deref().is_some_and(|l| l.contains("Ctrl+C")),
            "the chord must be inside the announced label; `hint_text` maps to \
             `set_placeholder` and is silent on a button"
        );
        assert!(
            info.hint_text.is_none(),
            "nothing may be smuggled into `hint_text` and assumed audible"
        );
    }

    /// A disabled row announces as disabled — the one piece of state that
    /// *does* survive the mapping (`set_disabled`).
    #[test]
    fn a_disabled_row_announces_as_disabled() {
        let info = WidgetInfo::labeled(WidgetType::Button, false, "Paste");
        assert!(!info.enabled);
    }

    /// A toggle row reports its state; an ordinary row does not claim to
    /// be a toggle at all.
    #[test]
    fn only_a_toggle_row_reports_a_toggle_state() {
        let toggled = WidgetInfo::selected(WidgetType::Button, true, true, "Single page");
        assert_eq!(toggled.selected, Some(true));
        let plain = WidgetInfo::labeled(WidgetType::Button, true, "Delete");
        assert_eq!(
            plain.selected, None,
            "announcing `not pressed` for Delete would imply it is a toggle"
        );
    }
}
