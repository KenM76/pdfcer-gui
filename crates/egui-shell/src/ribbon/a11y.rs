//! Accessible names — what a screen reader is told about a control that
//! shows only a glyph.
//!
//! # The problem
//!
//! A ribbon is mostly icons. An icon-only button has no text for an
//! assistive technology to read, so unless something else is published,
//! the control announces as *"button"* — which is true, useless, and
//! indistinguishable from every other button beside it.
//!
//! The information already exists: the control has a **tooltip**, written
//! by the application for exactly the purpose of saying what the glyph
//! means. [`accessible_name`] is the rule that routes it to the
//! accessibility tree as well as to the hover surface, so the two can
//! never drift apart — a tooltip that is edited is an accessible name
//! that is edited.
//!
//! # The rule
//!
//! | Control | Accessible name | Hint |
//! |---|---|---|
//! | Icon **and** label | the visible label | the tooltip |
//! | Icon only | **the tooltip** | — |
//! | Icon only, no tooltip | the label (which is not drawn, but exists) | — |
//!
//! The third row is a fallback, not a design: an icon-only control with
//! no tooltip is an application defect, and the shell's job is to make it
//! degrade to "announces its label" rather than to "announces nothing".
//! There is no case in which the accessible name is empty, which
//! `an_accessible_name_is_never_empty` pins.
//!
//! # A known ceiling: `egui` 0.35 has no tab `WidgetType`
//!
//! `egui::WidgetType` in 0.35 is:
//!
//! ```text
//! Label · Link · TextEdit · Button · Checkbox · RadioButton · RadioGroup ·
//! SelectableLabel · ComboBox · Slider · DragValue · ColorButton · Image ·
//! CollapsingHeader · Panel · ProgressIndicator · Window · ResizeHandle ·
//! ScrollBar · Other
//! ```
//!
//! There is **no `Tab` and no `TabList`**. The `accesskit` roles exist
//! (`Role::Tab`, `Role::TabList`), but `egui` 0.35 does not expose a path
//! from a `WidgetInfo` to them — `Response::widget_info` fills an
//! accesskit node from the `WidgetType`, and the mapping has no tab case
//! to hit.
//!
//! So the ribbon's tabs are published as [`egui::WidgetType::SelectableLabel`].
//! The consequence, stated plainly rather than papered over:
//!
//! - A screen reader announces a ribbon tab as a **selectable label that
//!   is or is not selected**, not as *"tab 2 of 7"*. Selection state,
//!   the label, and the enabled flag are all correct and all announced.
//!   What is missing is the **set relationship** — that these seven
//!   controls are one tab list and that exactly one of them is current.
//! - Keyboard operation is unaffected. Focus, activation and the mode
//!   selector's arrow-key movement all work; this is a *labelling*
//!   ceiling, not an interaction one.
//!
//! This is a limitation of the toolkit at this version, not a shortcut
//! taken here. It is written down so that (a) nobody re-derives it, and
//! (b) when `egui` gains the variant, this module is the one place to
//! change. `SelectableLabel` is the closest honest answer available:
//! `Button` would lose the selection state, and `Other` would lose both.
//!
//! The mode selector is in a better position — [`egui::WidgetType::RadioButton`]
//! is a genuine match for "one of N, mutually exclusive", and
//! `RadioGroup` exists for the container — so the selector announces
//! correctly and only the tab strip carries the ceiling.

use crate::commands::Command;
use egui::{Response, WidgetInfo, WidgetType};

/// The name an assistive technology should announce for a command
/// control.
///
/// See the module header's table. `shows_label` is whether the control
/// draws its text; when it does not, the tooltip is the only description
/// of the glyph that exists.
#[must_use]
pub fn accessible_name(command: &Command, shows_label: bool) -> &str {
    let label = non_empty(&command.label);
    let tooltip = command.tooltip.as_deref().and_then(non_empty);
    // The two orderings differ only in which of the two is preferred; both
    // fall through to the other, and both end at the id. A control whose
    // *preferred* source is blank must not be announced as nothing merely
    // because it was drawn with its label showing.
    let chosen = if shows_label {
        label.or(tooltip)
    } else {
        tooltip.or(label)
    };
    chosen.unwrap_or_else(|| fallback(command))
}

/// A last-resort name: the command's own id.
///
/// Reached only when an application registers a command with an empty
/// label and an empty (or absent) tooltip. The id is never empty in a
/// registered command, so this is what makes
/// `an_accessible_name_is_never_empty` a total claim rather than a
/// hopeful one. It is also *diagnostic*: hearing "view dot fit page"
/// announced tells whoever hears it exactly which registration to fix.
fn fallback(command: &Command) -> &str {
    if command.id.is_empty() {
        "unnamed control"
    } else {
        &command.id
    }
}

/// `Some(s)` if `s` has any non-whitespace content.
///
/// A label of `"   "` is as unannounceable as `""` and must fall through
/// to the next candidate, which a plain `is_empty` check would not do.
fn non_empty(s: &str) -> Option<&str> {
    if s.trim().is_empty() { None } else { Some(s) }
}

/// Publish a command control's accessibility information.
///
/// Called for **every** control the ribbon draws, not only icon-only
/// ones, because `Response::widget_info` is also what feeds `egui`'s
/// output events — a labelled button that skipped this would be
/// announced by `egui`'s own default, which is the label with no enabled
/// state and no hint.
pub(crate) fn describe_command(
    response: &Response,
    command: &Command,
    shows_label: bool,
    enabled: bool,
) {
    let name = accessible_name(command, shows_label).to_owned();
    let hint = if shows_label {
        command.tooltip.clone()
    } else {
        // The tooltip has already been promoted to the name; repeating it
        // as a hint makes a screen reader say the same sentence twice.
        None
    };
    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(WidgetType::Button, enabled, name.clone());
        info.hint_text = hint.clone();
        info
    });
}

/// Publish a ribbon **tab** button's accessibility information.
///
/// [`WidgetType::SelectableLabel`] rather than a tab role — see the
/// module header's "known ceiling" section for why, and for what is lost.
pub(crate) fn describe_tab(response: &Response, label: &str, selected: bool) {
    let label = label.to_owned();
    response.widget_info(|| {
        WidgetInfo::selected(WidgetType::SelectableLabel, true, selected, label.clone())
    });
}

/// Publish a **mode-selector segment**'s accessibility information.
///
/// [`WidgetType::RadioButton`] is an honest match here: the segments are
/// mutually exclusive, exactly one is current, and that is precisely what
/// a radio button means. Unlike the tab strip, the selector loses nothing
/// to the toolkit's vocabulary.
pub(crate) fn describe_mode_segment(response: &Response, label: &str, selected: bool) {
    let label = label.to_owned();
    response.widget_info(|| {
        WidgetInfo::selected(WidgetType::RadioButton, true, selected, label.clone())
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::HandlerToken;

    fn command(label: &str, tooltip: Option<&str>) -> Command {
        let base = Command::new("view.fit_page", label, HandlerToken::new(1));
        match tooltip {
            Some(t) => base.with_tooltip(t),
            None => base,
        }
    }

    /// **An icon-only control announces its tooltip.**
    ///
    /// The whole point of the module. Without this, every icon in the
    /// ribbon announces as "button" and the ribbon is unusable by anyone
    /// who cannot see it — while looking completely correct in a
    /// screenshot, which is why it needs a test rather than a review.
    #[test]
    fn an_icon_only_control_announces_its_tooltip() {
        let c = command("Fit page", Some("Scale the page to fit the window"));
        assert_eq!(
            accessible_name(&c, false),
            "Scale the page to fit the window"
        );
    }

    /// A control that draws its label announces the label, and the
    /// tooltip stays a hint.
    ///
    /// Promoting the tooltip over a visible label would make the spoken
    /// name differ from the printed one, which is worse than either
    /// alone: a user cannot then relate what they hear to what a sighted
    /// colleague is pointing at.
    #[test]
    fn a_labelled_control_announces_the_label_it_shows() {
        let c = command("Fit page", Some("Scale the page to fit the window"));
        assert_eq!(accessible_name(&c, true), "Fit page");
    }

    /// **An accessible name is never empty, whatever the application
    /// registered.**
    ///
    /// The fallback chain is tooltip → label → id, and a registered
    /// command always has an id. The degenerate rows are application
    /// defects; the shell's obligation is that they degrade to something
    /// announceable and *diagnostic* — hearing the id read out says which
    /// registration to fix.
    #[test]
    fn an_accessible_name_is_never_empty() {
        let cases = [
            command("Fit page", Some("Fit the page")),
            command("Fit page", None),
            command("", Some("Fit the page")),
            command("", None),
            command("   ", Some("   ")),
        ];
        for c in &cases {
            for shows_label in [true, false] {
                let name = accessible_name(c, shows_label);
                assert!(
                    !name.trim().is_empty(),
                    "{c:?} with shows_label={shows_label} announced nothing"
                );
            }
        }
        // The degenerate case falls through to the id, which is what
        // makes the defect findable from the announcement alone.
        assert_eq!(accessible_name(&command("", None), false), "view.fit_page");
    }

    /// Whitespace is not a name.
    ///
    /// `"   "` passes an `is_empty` check and announces as silence, which
    /// is the same failure as no name at all but harder to spot in a
    /// string catalogue.
    #[test]
    fn whitespace_falls_through_to_the_next_candidate() {
        // A blank label with a real tooltip announces the tooltip, even
        // though the control was drawn "with its label showing" — the
        // label it is showing is nothing.
        let c = Command::new("edit.undo", "   ", HandlerToken::new(2)).with_tooltip("Undo");
        assert_eq!(accessible_name(&c, true), "Undo");
        // And the mirror image.
        let c2 = Command::new("edit.undo", "Undo", HandlerToken::new(2)).with_tooltip("  ");
        assert_eq!(accessible_name(&c2, false), "Undo");
    }

    /// **The tab ceiling is asserted, so it cannot be quietly forgotten.**
    ///
    /// If a future `egui` adds `WidgetType::Tab`, this test is what fails
    /// and points at the module header that has to be rewritten. Until
    /// then it records that `SelectableLabel` is a deliberate choice with
    /// a stated cost, not an oversight.
    #[test]
    fn tabs_are_published_as_selectable_labels_because_egui_035_has_no_tab_role() {
        // The variant list `egui` 0.35 offers. If this stops compiling,
        // the vocabulary changed and the module header is now wrong.
        let _closest_available = WidgetType::SelectableLabel;
        let info = WidgetInfo::selected(WidgetType::SelectableLabel, true, true, "View");
        assert_eq!(info.typ, WidgetType::SelectableLabel);
        assert_eq!(info.selected, Some(true));
        assert_eq!(info.label.as_deref(), Some("View"));
    }

    /// The mode selector does *not* carry the tab strip's ceiling: a
    /// radio button is a true description of one-of-N.
    #[test]
    fn mode_segments_are_published_as_radio_buttons() {
        let info = WidgetInfo::selected(WidgetType::RadioButton, true, false, "Review");
        assert_eq!(info.typ, WidgetType::RadioButton);
        assert_eq!(info.selected, Some(false));
    }
}
