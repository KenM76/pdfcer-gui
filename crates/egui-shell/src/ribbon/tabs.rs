//! The tab strip — which tabs exist right now, which one is active, and
//! how the active one is made distinguishable.
//!
//! # Three sources, one strip
//!
//! What appears in the strip is the answer to three separate questions,
//! and keeping them separate is what makes each one testable:
//!
//! 1. **Which ordinary tabs does the current mode contain?**
//!    `MODES_AND_PANELS.md` Part 1: a mode names a fixed set of tabs, in
//!    an order the mode chooses. Read is *File · View*; Edit is all
//!    seven. Nothing in this crate knows those names —
//!    [`crate::manifest::Mode`] carries them and this module reads them.
//! 2. **Which contextual tabs are showing?** A contextual tab's
//!    `visible_when` is evaluated against the frame's
//!    [`crate::commands::ConditionSet`]. `RIBBON_IA.md` §4: *Format
//!    appears when a markup, dimension, image or vector object is
//!    selected.*
//! 3. **Which of those is active?** The operator's last choice, if it is
//!    still on screen; otherwise the first tab.
//!
//! `visible_tabs` answers 1 and 2, `resolve_active` answers 3, and
//! both are pure functions over a manifest — no `Ui`, no window, fully
//! testable.
//!
//! # Why the active-tab fallback is a correctness rule, not a nicety
//!
//! Switching from Edit to Read removes five tabs. If the operator was on
//! Measure, the active tab no longer exists. Three things could happen:
//!
//! - **Panic.** Obviously not.
//! - **Render an empty band.** The ribbon appears broken; the operator's
//!   next move is to click a tab, which fixes it, and they learn that the
//!   application sometimes goes blank.
//! - **Fall back to the first visible tab.** The strip is always
//!   coherent.
//!
//! `resolve_active` is the third, and the same rule covers a contextual
//! tab that stops being visible while it is active — deselect a markup
//! object while the Format tab is open and the strip must recover in the
//! same frame. `an_active_tab_that_disappears_falls_back_to_the_first`
//! pins both cases.
//!
//! # R84: colour is never the only cue
//!
//! The project's standing rule is that state is never carried by colour
//! alone. An active tab that differs only by fill is invisible to a
//! colour-blind operator, invisible on a projector, and invisible in a
//! greyscale screenshot — which is also how it becomes invisible in a
//! bug report.
//!
//! ## A correction worth recording: `RichText::strong()` is *not* a
//! weight cue in `egui`
//!
//! `D:\Dev\pdfcer\UI_PREFERENCES.md` §9 cites the old dock's active-tab
//! treatment as already R84-compliant on the grounds that it *"bolds the
//! active tab's label text — a weight cue, not a fill-color-only cue."*
//!
//! That is not what `egui` does. `RichText::strong()` sets a flag whose
//! only effect is `visuals.strong_text_color()` — **a different colour,
//! at the same weight** (`egui-0.35.0/src/widget_text.rs:484`). There is
//! no bold face involved, and with `default-features = false` this crate
//! does not even have a bold face available to switch to.
//!
//! So a design that relies on `.strong()` for its redundant cue has, in
//! fact, two colour cues and no others — precisely the failure R84 names,
//! arrived at through a reasonable-sounding sentence about a toolkit
//! behaving the way a word processor does.
//!
//! This module therefore uses **geometry** for its redundancy. [`TabCues`]
//! is that redundancy, expressed as data so it can be asserted:
//!
//! | Cue | Inactive | Active | Kind |
//! |---|---|---|---|
//! | Accent rule along the tab's top | absent | present | **shape** |
//! | Outline on top and sides, open into the band | absent | present | **shape** |
//! | Fill behind the label | none | none | — |
//! | Text emphasis | `text_muted` | `strong()` + `text` | colour — see above |
//!
//! Two of the four are the *presence or absence of a shape*, which
//! survives greyscale, colour blindness and a bad projector, and either
//! one alone is sufficient to read the strip.
//! `the_active_tab_is_distinguished_by_more_than_colour` asserts that,
//! and it counts `emphasised_text` as a **colour** cue on purpose, so
//! that the mistake above cannot be re-made by a future edit that
//! simplifies the cues back down to fill plus `.strong()`.
//!
//! # What this module does *not* own: where the tabs go
//!
//! This module answers *which* tabs and *how one looks*. It does not
//! decide how many of them fit, where the row's QAT and mode selector sit,
//! or what happens to the ones that do not fit — that is
//! [`super::strip`], which owns the whole tab-strip **row** and its
//! reservation order.
//!
//! The split is not cosmetic. A module that answered *which tabs* and also
//! placed the row would place it the only way one `Ui` allows: nest the
//! islands and hope. A nested right-hand island protects the right-hand
//! island and nothing else, and `egui` does not clip a `Ui`'s children to
//! its `max_rect`, so the QAT and the tabs run through the mode selector
//! and off the window — `MODES_AND_PANELS.md` failure mode #8, one row up.
//! [`super::strip`]'s header carries the measurement of what that costs.
//! Planning a row is not a thing a "which tabs are visible" module should
//! contain, so the row lives there and the tabs live here.
//!
//! Design and rationale: `docs/modules/egui-shell/ribbon/tabs.md`.

use egui::{RichText, Stroke, TextStyle, vec2};

use crate::commands::ConditionSet;
use crate::manifest::{Shell, Tab};

use super::a11y;
use super::ctx::{Ctx, condition_holds};
use super::measure;
use super::plan::ItemWidths;
use super::report;

/// The cues that distinguish an active tab from an inactive one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabCues {
    /// An accent rule drawn under the tab. A **shape** cue: its presence
    /// or absence reads without any colour information at all, and it is
    /// the convention every ribbon uses.
    pub underline: bool,
    /// A border stroke around the tab. A second **shape** cue, so that
    /// losing the underline to a clipping edge does not leave the strip
    /// unreadable.
    pub outlined: bool,
    /// The accent fill behind the label. A **colour** cue. No tab in the
    /// strip is filled any more: a filled tab reads as a button.
    pub filled: bool,
    /// `RichText::strong()` on the label.
    ///
    /// **This is a colour cue**, not a weight cue — see the module
    /// header. It is kept because a stronger text colour is a genuine
    /// improvement for a sighted operator; it is *counted* as colour so
    /// that it can never be mistaken for the redundancy R84 asks for.
    pub emphasised_text: bool,
}

impl TabCues {
    /// How many of this tab's cues do not depend on colour.
    ///
    /// The number R84 cares about. Used by the test rather than by the
    /// renderer, which is the point: the rule is checkable.
    pub fn non_colour_cues(self) -> usize {
        usize::from(self.underline) + usize::from(self.outlined)
    }
}

/// The cues for a tab in a given state.
pub fn tab_cues(active: bool) -> TabCues {
    TabCues {
        underline: active,
        outlined: active,
        filled: false,
        emphasised_text: active,
    }
}

/// The tabs to show this frame, in display order: the active mode's
/// ordinary tabs, then every contextual tab whose condition holds.
pub(crate) fn visible_tabs<'a>(
    shell: &'a Shell,
    mode_id: Option<&str>,
    conditions: &ConditionSet,
) -> Vec<&'a Tab> {
    let mut out: Vec<&Tab> = Vec::new();

    let mode = mode_id.and_then(|id| shell.modes().iter().find(|m| m.id == id));
    match mode {
        Some(mode) if !mode.tabs().is_empty() => {
            for wanted in mode.tabs() {
                match shell.tabs().iter().find(|t| &t.id == wanted) {
                    Some(tab) if !tab.is_hidden() && has_something_to_show(tab, conditions) => {
                        out.push(tab);
                    }
                    Some(_) => {}
                    None => {
                        crate::verify::event("ribbon-mode-names-unknown-tab")
                            .kv("mode", &mode.id)
                            .kv("tab", wanted)
                            .emit();
                    }
                }
            }
        }
        _ => out.extend(
            shell
                .tabs()
                .iter()
                .filter(|t| !t.is_hidden() && has_something_to_show(t, conditions)),
        ),
    }

    out.extend(shell.contextual_tabs().iter().filter(|t| {
        !t.is_hidden()
            && has_something_to_show(t, conditions)
            && t.visible_when
                .as_deref()
                .is_some_and(|cond| condition_holds(cond, conditions))
    }));

    out
}

/// Whether any item on `tab` is visible under `conditions`.
fn has_something_to_show(tab: &Tab, conditions: &ConditionSet) -> bool {
    let mut saw_item = false;
    for group in tab.groups() {
        for item in group.items() {
            saw_item = true;
            match item.visible_condition() {
                None => return true,
                Some(cond) if condition_holds(cond, conditions) => return true,
                Some(_) => {}
            }
        }
    }
    !saw_item
}

/// Which tab is active: the requested one if it is still on screen,
/// otherwise the first.
///
/// See the module header on why the fallback is a correctness rule.
pub(crate) fn resolve_active<'a>(visible: &[&'a Tab], requested: Option<&str>) -> Option<&'a Tab> {
    if let Some(id) = requested
        && let Some(tab) = visible.iter().find(|t| t.id == id)
    {
        return Some(tab);
    }
    visible.first().copied()
}

/// The label one tab draws. Never empty in a well-formed manifest, and
/// diagnostic when it is not — the same fallback rule as
/// [`super::band::caption_text`] and [`super::mode_selector::mode_label`].
pub(crate) fn tab_label(tab: &Tab) -> &str {
    match tab.label.as_deref() {
        Some(l) if !l.trim().is_empty() => l,
        _ => &tab.id,
    }
}

/// The width one tab will occupy, before the row's budget is applied.
pub(crate) fn measure_tab(ui: &egui::Ui, tab: &Tab) -> f32 {
    ItemWidths {
        icon: 0.0,
        text: measure::text_width(ui, tab_label(tab), &TextStyle::Button),
        gap: 0.0,
        padding: measure::button_padding(ui),
    }
    .total()
}

/// Draw a run of tabs and report which one the operator wants active.
pub(crate) fn render_tabs(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    visible: &[&Tab],
    active_id: Option<&str>,
) -> Option<String> {
    let mut clicked = None;
    for tab in visible {
        let is_active = active_id == Some(tab.id.as_str());
        if draw_tab(ui, ctx, tab, is_active, false) {
            clicked = Some(tab.id.clone());
        }
    }
    clicked
}

/// Draw one tab button, wherever it is. Returns whether it was clicked.
fn draw_tab(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    tab: &Tab,
    is_active: bool,
    in_menu: bool,
) -> bool {
    let palette = ctx.theme.palette;
    let cues = tab_cues(is_active);
    let label = tab_label(tab);

    let text = if cues.emphasised_text {
        RichText::new(label).strong().color(palette.text)
    } else {
        RichText::new(label).color(palette.text_muted)
    };

    let response = if in_menu {
        ui.add(egui::Button::selectable(is_active, text).truncate())
    } else {
        // Frameless, at the width `measure_tab` planned for (or what is left
        // of the row); the tab's shape is painted into `body` once hover is known.
        let width = measure_tab(ui, tab).min(ui.available_width());
        let body = ui.painter().add(egui::Shape::Noop);
        let response = ui.add(
            egui::Button::new(text)
                .frame(false)
                .truncate()
                .min_size(vec2(width, strip_height(ctx)))
                .selected(is_active),
        );
        ui.painter().set(
            body,
            crate::tabshape::body(
                &palette,
                ctx.theme.metrics.corner_radius,
                palette.surface,
                response.rect,
                cues.outlined,
                response.hovered(),
            ),
        );
        if is_active {
            let pass = ui.ctx().cumulative_pass_nr();
            ui.ctx()
                .data_mut(|d| d.insert_temp(active_tab_key(), (pass, response.rect)));
        }
        response
    };

    a11y::describe_tab(&response, label, is_active);

    let response = match tab.question.as_deref() {
        // The tab's one-line question is the best tooltip it could
        // have: `RIBBON_IA.md` §4 keeps it as the test of whether a
        // tab is coherent, and it answers "what is this tab for"
        // better than any separate string an application would write.
        Some(q) => response.on_hover_text(q),
        None => response,
    };

    ctx.reporter.report(response.rect, || report::tab(&tab.id));

    if response.clicked() {
        crate::verify::event("ribbon-tab-activated")
            .kv("tab", &tab.id)
            .emit();
        return true;
    }
    false
}

/// The tabs that did not fit, as a menu behind the strip's "⏷ N more"
/// affordance.
pub(crate) fn render_overflow_menu(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    hidden: &[&Tab],
    active_id: Option<&str>,
) -> Option<String> {
    let mut clicked = None;
    for tab in hidden {
        let is_active = active_id == Some(tab.id.as_str());
        if draw_tab(ui, ctx, tab, is_active, true) {
            clicked = Some(tab.id.clone());
        }
    }
    clicked
}

/// Height reserved for one tab-strip row, used to align the mode selector
/// with the tabs.
/// Where the active tab was drawn this pass, so the strip's baseline can
/// open beneath it.
fn active_tab_key() -> egui::Id {
    egui::Id::new("egui-shell-ribbon-active-tab")
}

pub(crate) fn strip_height(ctx: &Ctx<'_>) -> f32 {
    ctx.theme.metrics.control_height
}

/// A thin rule under the tab strip, separating it from the band.
pub(crate) fn strip_underline(ui: &mut egui::Ui, ctx: &Ctx<'_>) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
    let stroke = Stroke::new(1.0, ctx.theme.palette.outline);
    let y = rect.center().y;
    let pass = ui.ctx().cumulative_pass_nr();
    let active = ui
        .ctx()
        .data(|d| d.get_temp::<(u64, egui::Rect)>(active_tab_key()))
        .filter(|(drawn, tab)| *drawn == pass && tab.x_range().intersects(rect.x_range()))
        .map(|(_, tab)| tab);
    let painter = ui.painter();
    match active {
        // The active tab's sides run down to the baseline, and the baseline
        // breaks under it, so the tab opens into the band below.
        Some(tab) => {
            let (l, r) = (tab.left() + 0.5, tab.right() - 0.5);
            painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(l, y)], stroke);
            painter.line_segment([egui::pos2(r, y), egui::pos2(rect.right(), y)], stroke);
            painter.line_segment([egui::pos2(l, tab.bottom()), egui::pos2(l, y)], stroke);
            painter.line_segment([egui::pos2(r, tab.bottom()), egui::pos2(r, y)], stroke);
        }
        None => {
            painter.line_segment([rect.left_center(), rect.right_center()], stroke);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::manifest::Item;
    /// **A tab every one of whose items is conditioned away is not
    /// shown at all.**
    #[test]
    fn a_tab_with_every_item_conditioned_away_is_not_shown() {
        let shell = Shell::new()
            .with_mode(Mode::new("m", "M", ["a", "b"]))
            .with_tab(Tab::new("a", "A").with_groups([
                Group::new("g", "G").with_items([Item::command("x").shown_when("never")]),
            ]))
            .with_tab(
                Tab::new("b", "B")
                    .with_groups([Group::new("g", "G").with_items([Item::command("y")])]),
            );

        let off = ConditionSet::new();
        let shown: Vec<&str> = visible_tabs(&shell, Some("m"), &off)
            .iter()
            .map(|t| t.id.as_str())
            .collect();
        assert_eq!(
            shown,
            vec!["b"],
            "a tab whose only item is conditioned away must not be offered"
        );

        let mut on = ConditionSet::new();
        on.set("never");
        let shown: Vec<&str> = visible_tabs(&shell, Some("m"), &on)
            .iter()
            .map(|t| t.id.as_str())
            .collect();
        assert_eq!(
            shown,
            vec!["a", "b"],
            "and must come back the moment its condition holds"
        );
    }

    /// A manifest that never uses conditions is untouched by the rule.
    #[test]
    fn an_unconditioned_or_empty_tab_is_always_shown() {
        let shell = Shell::new()
            .with_mode(Mode::new("m", "M", ["plain", "bare"]))
            .with_tab(
                Tab::new("plain", "Plain")
                    .with_groups([Group::new("g", "G").with_items([Item::command("x")])]),
            )
            .with_tab(Tab::new("bare", "Bare").with_groups([]));

        let shown: Vec<&str> = visible_tabs(&shell, Some("m"), &ConditionSet::new())
            .iter()
            .map(|t| t.id.as_str())
            .collect();
        assert_eq!(shown, vec!["plain", "bare"]);
    }

    use super::*;
    use crate::manifest::{Group, Mode};

    /// Two modes over four ordinary tabs, plus one contextual tab.
    fn shell() -> Shell {
        Shell::new()
            .with_mode(Mode::new("read", "Read", ["file", "view"]))
            .with_mode(Mode::new(
                "edit",
                "Edit",
                ["file", "view", "pages", "tools"],
            ))
            .with_tab(Tab::new("file", "File").with_groups([Group::new("file", "File")]))
            .with_tab(Tab::new("view", "View").with_groups([Group::new("display", "Display")]))
            .with_tab(Tab::new("pages", "Pages").with_groups([Group::new("pages", "Pages")]))
            .with_tab(Tab::new("tools", "Tools").with_groups([Group::new("run", "Run")]))
            .with_contextual_tab(
                Tab::new("format", "Format")
                    .with_visible_when("selection.any")
                    .with_groups([Group::new("style", "Style")]),
            )
    }

    fn ids(tabs: &[&Tab]) -> Vec<String> {
        tabs.iter().map(|t| t.id.clone()).collect()
    }

    /// **A mode shows its own tabs, in its own order.**
    #[test]
    fn a_mode_shows_only_its_own_tabs() {
        let shell = shell();
        let none = ConditionSet::new();
        assert_eq!(
            ids(&visible_tabs(&shell, Some("read"), &none)),
            ["file", "view"]
        );
        assert_eq!(
            ids(&visible_tabs(&shell, Some("edit"), &none)),
            ["file", "view", "pages", "tools"]
        );
    }

    /// **A mode's order wins over the manifest's.**
    #[test]
    fn a_modes_order_wins_over_the_manifests() {
        let shell = shell().with_mode(Mode::new("backwards", "Backwards", ["tools", "file"]));
        assert_eq!(
            ids(&visible_tabs(
                &shell,
                Some("backwards"),
                &ConditionSet::new()
            )),
            ["tools", "file"]
        );
    }

    /// No modes, or an unknown mode, shows every ordinary tab.
    #[test]
    fn an_absent_or_unknown_mode_shows_everything() {
        let shell = shell();
        let none = ConditionSet::new();
        assert_eq!(
            ids(&visible_tabs(&shell, None, &none)),
            ["file", "view", "pages", "tools"]
        );
        assert_eq!(
            ids(&visible_tabs(&shell, Some("no-such-mode"), &none)),
            ["file", "view", "pages", "tools"]
        );

        let modeless = Shell::new().with_tab(Tab::new("only", "Only"));
        assert_eq!(ids(&visible_tabs(&modeless, Some("read"), &none)), ["only"]);
    }

    /// **A contextual tab appears exactly while its condition holds.**
    #[test]
    fn a_contextual_tab_appears_only_while_its_condition_holds() {
        let shell = shell();
        assert_eq!(
            ids(&visible_tabs(&shell, Some("read"), &ConditionSet::new())),
            ["file", "view"]
        );
        assert_eq!(
            ids(&visible_tabs(
                &shell,
                Some("read"),
                &ConditionSet::new().with("selection.any")
            )),
            ["file", "view", "format"],
            "a contextual tab is appended, never inserted"
        );
    }

    /// A contextual tab with no condition never appears.
    #[test]
    fn a_contextual_tab_with_no_condition_never_appears() {
        let shell = Shell::new()
            .with_tab(Tab::new("a", "A"))
            .with_contextual_tab(Tab::new("ctx", "Ctx"));
        assert_eq!(
            ids(&visible_tabs(
                &shell,
                None,
                &ConditionSet::new().with("anything")
            )),
            ["a"]
        );
    }

    /// A hidden tab is skipped even when a mode names it.
    #[test]
    fn a_hidden_tab_is_skipped_even_when_a_mode_names_it() {
        let mut shell = shell();
        shell.tabs.as_mut().expect("has tabs")[1].hidden = Some(true);
        assert_eq!(
            ids(&visible_tabs(&shell, Some("edit"), &ConditionSet::new())),
            ["file", "pages", "tools"]
        );
    }

    /// **An active tab that disappears falls back to the first visible
    /// one, in the same frame.**
    #[test]
    fn an_active_tab_that_disappears_falls_back_to_the_first() {
        let shell = shell();
        let none = ConditionSet::new();

        let edit = visible_tabs(&shell, Some("edit"), &none);
        assert_eq!(
            resolve_active(&edit, Some("tools")).map(|t| t.id.as_str()),
            Some("tools"),
            "a tab that is still on screen stays active"
        );

        let read = visible_tabs(&shell, Some("read"), &none);
        assert_eq!(
            resolve_active(&read, Some("tools")).map(|t| t.id.as_str()),
            Some("file"),
            "switching to a mode without the active tab must not leave a blank band"
        );

        let with_format = visible_tabs(
            &shell,
            Some("read"),
            &ConditionSet::new().with("selection.any"),
        );
        assert_eq!(
            resolve_active(&with_format, Some("format")).map(|t| t.id.as_str()),
            Some("format")
        );
        assert_eq!(
            resolve_active(&read, Some("format")).map(|t| t.id.as_str()),
            Some("file"),
            "deselecting must retire the Format tab without blanking the ribbon"
        );

        assert_eq!(
            resolve_active(&[], Some("file")),
            None,
            "an empty strip has no active tab"
        );
        assert_eq!(
            resolve_active(&read, None).map(|t| t.id.as_str()),
            Some("file"),
            "with nothing requested the first tab is active"
        );
    }

    /// **R84: the active tab differs from an inactive one by more than
    /// colour.**
    #[test]
    fn the_active_tab_is_distinguished_by_more_than_colour() {
        let active = tab_cues(true);
        let inactive = tab_cues(false);
        assert_ne!(active, inactive);
        assert!(
            active.non_colour_cues() >= 2,
            "R84: an active tab must carry at least two cues that survive \
             greyscale; it carries {}",
            active.non_colour_cues()
        );
        assert_eq!(
            inactive.non_colour_cues(),
            0,
            "an inactive tab must carry none of them, or the cue says nothing"
        );
        assert!(
            active.underline && active.outlined,
            "two independent shape cues, so either alone reads"
        );
        // The tripwire on the correction: if someone re-counts
        // `emphasised_text` as non-colour, the cue budget silently drops
        // to one real cue and this stops being true.
        assert_eq!(
            TabCues {
                underline: false,
                outlined: false,
                filled: true,
                emphasised_text: true,
            }
            .non_colour_cues(),
            0,
            "fill plus `strong()` is two COLOUR cues and no shape cue at all"
        );
    }
}
