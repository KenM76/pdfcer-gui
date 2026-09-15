//! The per-frame render context every ribbon surface is handed.
//!
//! # Why a context struct rather than a parameter list
//!
//! The tab strip, the band, the mode selector and the QAT all need the
//! same things: the registry, the conditions, the theme, the rect
//! reporter, the icon painter and somewhere to collect the tokens the
//! operator invoked. Threading those through as parameters means every
//! new capability is a signature change in every file that renders a
//! ribbon surface, and — worse — makes it possible for one surface to be
//! handed a *different* condition set than another, which would produce a
//! ribbon whose tabs and whose controls disagreed about what is enabled.
//!
//! One value, constructed once per frame, removes both.
//!
//! # The two callbacks, and why they take what they take
//!
//! Both exist because the shell must render something it is forbidden to
//! understand. `SHELL_FRAMEWORK.md` §2's diagnostic form of the purity
//! rule applies directly: *if `egui-shell` needs to know about pages, the
//! abstraction is wrong* — and an icon set is exactly that kind of
//! knowledge. An icon is a licensing decision, a rasterization decision
//! and a look; none of those are a shell's business.
//!
//! ## [`IconPainter`] receives a [`egui::Painter`], not a `&mut Ui`
//!
//! This is deliberate and it is not a matter of taste. The icon is
//! painted into a slot **inside a button that is already being laid
//! out** — an `egui` "custom atom" whose rectangle the button's own
//! layout computed. Handing the application a `&mut Ui` there would let
//! it allocate layout space inside a widget that has already decided its
//! size, which corrupts the button's geometry in ways that show up as
//! text overlapping its own frame. A `Painter` can draw and cannot
//! allocate, so the seam is safe by type rather than by instruction.
//!
//! ## [`CustomItemRenderer`] receives a `&mut Ui`, and must
//!
//! The opposite case. [`crate::manifest::Item::Custom`] exists precisely
//! for controls that are *not* a button — a colour swatch, a zoom
//! slider, a scale picker with a gallery. Those are widgets, they need to
//! allocate, and the shell's job is to reserve a slot in the band's
//! horizontal flow and get out of the way. It returns an optional
//! [`HandlerToken`] so a custom control can report an invocation through
//! the same channel as everything else — see [`super`]'s "the shell
//! reports, the application dispatches" section.

use crate::commands::{Command, CommandRegistry, ConditionSet, HandlerToken};
use crate::theme::Theme;

use super::report::Reporter;

/// A request to paint one icon into a slot the ribbon has reserved.
///
/// The application resolves [`Self::key`] against its own icon set —
/// `Command::icon` is a `String` key for the reasons given on that field
/// — and paints into [`Self::rect`].
#[derive(Debug, Clone, Copy)]
pub struct IconRequest<'a> {
    /// The application-defined icon key from [`Command::icon`].
    pub key: &'a str,
    /// The rectangle reserved for the glyph, in screen coordinates.
    ///
    /// Square, sized from [`crate::theme::Metrics::icon_pts`], and
    /// already positioned inside the control by the button's own layout.
    pub rect: egui::Rect,
    /// The colour the glyph should take, so a monochrome icon set follows
    /// the theme and the widget state (hover, active, disabled) without
    /// the application tracking either.
    pub tint: egui::Color32,
    /// Whether the control this icon belongs to is enabled.
    ///
    /// Supplied in addition to the tint because an application whose
    /// icons are multi-coloured cannot express "disabled" by tinting and
    /// needs to know.
    pub enabled: bool,
    /// Whether the control is currently **selected** — an armed tool, an
    /// active page-display mode, a panel that is open.
    ///
    /// Supplied for the same reason as [`Self::enabled`], and the reason
    /// is stronger here. The shell already shows selection with the
    /// button's frame, so an application could ignore this field and look
    /// correct. But a frame is one cue, and an icon set that can render a
    /// heavier weight when selected gives a second one that survives a
    /// theme whose selected and unselected frames are close in value.
    ///
    /// The rule this exists to let an application keep is **selected
    /// state is never colour alone**: the shell cannot honour that on the
    /// application's behalf, because the second cue lives in the glyph,
    /// which only the application can draw.
    ///
    /// For a control with no selected/unselected distinction — an
    /// ordinary command button — this is `false`, which is the same thing
    /// it means for a two-state control that is off. The two are not
    /// distinguished, because an icon set has nothing different to draw
    /// for them.
    pub selected: bool,
}

/// Paints one icon. See [`IconRequest`] and this module's header.
pub type IconPainter<'a> = dyn FnMut(&egui::Painter, &IconRequest<'_>) + 'a;

/// One [`crate::manifest::Item::Custom`], handed back to the application
/// to draw.
#[derive(Debug, Clone, Copy)]
pub struct CustomItem<'a> {
    /// The application-defined kind, e.g. `"colour_swatch"`.
    pub kind: &'a str,
    /// The application-defined payload, if the manifest carried one.
    pub payload: Option<&'a str>,
    /// The id of the tab this item is on, so one renderer can serve a
    /// kind that appears in more than one place.
    pub tab: &'a str,
    /// The id of the group this item is in.
    pub group: &'a str,
}

/// Draws one custom item. Returns a token if the operator invoked
/// something, so custom controls report through the same channel as
/// commands.
pub type CustomItemRenderer<'a> =
    dyn FnMut(&mut egui::Ui, &CustomItem<'_>) -> Option<HandlerToken> + 'a;

/// Everything a ribbon surface needs for one frame.
///
/// Not `Clone`, not `Copy`, and never stored: it borrows the
/// application's callbacks for the duration of one
/// [`super::Ribbon::render`] call and is dropped at the end of it.
pub(crate) struct Ctx<'a> {
    /// The commands that exist. A manifest may only reference these.
    pub registry: &'a CommandRegistry,
    /// What is true this frame, for enable predicates and for contextual
    /// tab visibility.
    pub conditions: &'a ConditionSet,
    /// The look in force, read from the `egui` context once per frame.
    pub theme: Theme,
    /// Where drawn rectangles are published, if anywhere.
    pub reporter: Reporter<'a>,
    /// How to paint an icon, if the application supplied a painter.
    pub icons: Option<&'a mut IconPainter<'a>>,
    /// How to draw a custom item, if the application supplied a renderer.
    pub custom: Option<&'a mut CustomItemRenderer<'a>>,
    /// The base `egui::Id` every ribbon widget id is derived from.
    pub base_id: egui::Id,
    /// The tokens the operator invoked this frame, in the order the
    /// controls were drawn.
    pub invoked: Vec<HandlerToken>,
}

impl Ctx<'_> {
    /// Look a command id up, tracing a disclosure if it is unknown.
    ///
    /// # Why an unknown id is a skip and not a panic
    ///
    /// [`crate::manifest::Shell::validate_against`] is supposed to have
    /// caught this at load, and [`crate::manifest::merge`] is supposed to
    /// have turned an operator's stale reference into a disclosed
    /// [`crate::manifest::Skip`] before that. Reaching here means an
    /// application rendered a manifest it did not validate — a
    /// programming error, but one whose correct penalty is *one missing
    /// control*, not a crash in the paint loop with a document open.
    ///
    /// The trace is what stops it being silent. `SHELL_FRAMEWORK.md` §4
    /// calls an unknown id a **disclosed skip**, and an undisclosed skip
    /// is indistinguishable from a rendering fault — the lesson
    /// [`crate::verify`]'s header records about a step that was dropped
    /// without saying so.
    pub(crate) fn command(&self, id: &str) -> Option<&Command> {
        match self.registry.get(id) {
            Some(c) => Some(c),
            None => {
                crate::verify::event("ribbon-skipped-unknown-command")
                    .kv("id", id)
                    .emit();
                None
            }
        }
    }

    /// Record that the operator invoked a command.
    pub(crate) fn invoke(&mut self, token: HandlerToken) {
        self.invoked.push(token);
    }

    /// An `egui::Id` for a ribbon widget, derived from the base id.
    ///
    /// Derived rather than auto-generated so that ids are **stable across
    /// frames even as the layout changes**. `egui` keeps focus, hover and
    /// popup state per id; an auto-generated id shifts when a group is
    /// collapsed or scrolled out of the band, and the symptom is a control
    /// that loses keyboard focus when the window is resized — which reads
    /// as a focus bug rather than as an id bug and is very hard to
    /// attribute.
    pub(crate) fn id(&self, kind: &str, key: &str) -> egui::Id {
        self.base_id.with(kind).with(key)
    }
}

/// Whether a `visible_when` / enable-style condition holds.
///
/// Mirrors [`crate::commands::Enable::When`]'s language exactly — a bare
/// name is "this condition is set", a leading `!` negates — but without
/// allocating a `String` per contextual tab per frame, which is what
/// building an [`crate::commands::Enable`] to evaluate it would cost.
///
/// An **empty** condition is `true`: a manifest that says
/// `visible_when: ""` has said nothing, and the tab is not usefully made
/// permanently invisible by an empty string.
///
/// `the_local_condition_evaluator_agrees_with_enable` pins this against
/// the real implementation, because two copies of a rule that can drift
/// is exactly how a contextual tab ends up appearing under conditions its
/// author's enable predicate would have refused.
pub(crate) fn condition_holds(expr: &str, conditions: &ConditionSet) -> bool {
    let expr = expr.trim();
    if expr.is_empty() {
        return true;
    }
    match expr.strip_prefix('!') {
        Some(negated) => !conditions.is_set(negated),
        None => conditions.is_set(expr),
    }
}

impl std::fmt::Debug for Ctx<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ctx")
            .field("commands", &self.registry.len())
            .field("conditions", &self.conditions)
            .field("preset", &self.theme.preset)
            .field("reporter", &self.reporter)
            .field("icons", &self.icons.is_some())
            .field("custom", &self.custom.is_some())
            .field("invoked", &self.invoked)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Enable;

    /// **The local condition evaluator agrees with
    /// [`crate::commands::Enable::When`], case for case.**
    ///
    /// There are two implementations of one rule: the canonical one in
    /// `commands`, and this allocation-free copy used for contextual
    /// tab visibility. That is a drift hazard with a nasty failure mode —
    /// a `Format` tab that appears under conditions its author's enable
    /// predicate would have refused, so the tab is present and every
    /// control on it is disabled.
    ///
    /// The copy is justified (it runs once per contextual tab per frame
    /// and `Enable::When` would allocate a `String` to be dropped), but
    /// it is only safe while something checks the two agree. This is that
    /// something.
    #[test]
    fn the_local_condition_evaluator_agrees_with_enable() {
        let conditions = ConditionSet::new().with("selection.any").with("doc.open");
        for expr in [
            "selection.any",
            "doc.open",
            "doc.readonly",
            "!selection.any",
            "!doc.readonly",
            "!",
            "nothing.at.all",
        ] {
            let canonical = Enable::When(expr.to_owned()).evaluate(&conditions);
            assert_eq!(
                condition_holds(expr, &conditions),
                canonical,
                "the two implementations of the condition language disagree on `{expr}`"
            );
        }
    }

    /// An unstated condition is not a permanently invisible tab.
    ///
    /// The one deliberate divergence from `Enable::When`, which has no
    /// empty case because a command always carries a real predicate. A
    /// manifest that spells `visible_when: ""` has said nothing, and
    /// reading "nothing" as "never" would silently delete a tab from the
    /// interface with no message anywhere.
    #[test]
    fn an_empty_condition_says_nothing_rather_than_never() {
        let conditions = ConditionSet::new();
        assert!(condition_holds("", &conditions));
        assert!(condition_holds("   ", &conditions));
    }

    /// Whitespace around a condition name does not change its meaning —
    /// a hand-edited manifest is the primary source of these strings.
    #[test]
    fn a_condition_tolerates_surrounding_whitespace() {
        let conditions = ConditionSet::new().with("selection.any");
        assert!(condition_holds("  selection.any  ", &conditions));
        assert!(!condition_holds("  !selection.any ", &conditions));
    }
}
