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
//! # ★ Why the active-tab fallback is a correctness rule, not a nicety
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
//! # ★ R84: colour is never the only cue
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
//! So a design that relied on `.strong()` for its redundant cue had, in
//! fact, two colour cues and no others — which is precisely the failure
//! R84 names, arrived at through a reasonable-sounding sentence about a
//! toolkit behaving the way a word processor does.
//!
//! This module therefore uses **geometry** for its redundancy. [`TabCues`]
//! is that redundancy, expressed as data so it can be asserted:
//!
//! | Cue | Inactive | Active | Kind |
//! |---|---|---|---|
//! | Accent rule under the tab | absent | present | **shape** |
//! | Border stroke around the tab | absent | present | **shape** |
//! | Fill behind the label | none | accent | colour |
//! | Text emphasis | plain | `strong()` | colour — see above |
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
//! The split is not cosmetic. Until 2026-08-13 this module carried a
//! `with_right_island` helper that laid the mode selector out from the
//! right edge and handed the remainder to the tabs. It protected the
//! *right* island and nothing else, and `egui` does not clip a `Ui`'s
//! children to its `max_rect`, so the QAT and the tabs simply ran through
//! the selector and off the window. Measured against the synthetic face of
//! `super::width_tests`, with the two-control QAT and two tabs of the test
//! manifest:
//!
//! ```text
//! window  QAT             tabs                selector      verdict
//!  500    0..166          188..265            322..500      correct
//!  320    0..166          188..265            142..320      tabs UNDER the selector
//!  180   -6..160          182..259              2..180      both tabs off screen
//! ```
//!
//! That is `MODES_AND_PANELS.md` failure mode #8 one row up. Fixing it
//! meant planning the row rather than nesting layouts in it, and a planner
//! is not a thing a "which tabs are visible" module should contain — so
//! the row moved out and this module kept the tabs.

use egui::{RichText, Stroke, TextStyle, vec2};

use crate::commands::ConditionSet;
use crate::manifest::{Shell, Tab};

use super::a11y;
use super::band;
use super::ctx::{Ctx, condition_holds};
use super::plan::ItemWidths;
use super::report;

/// The cues that distinguish an active tab from an inactive one.
///
/// A struct rather than three `if active` expressions scattered through
/// the drawing code, because R84 is a property of the *set* of cues and a
/// property of a set cannot be asserted about three separate expressions.
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
    /// The accent fill behind the label. A **colour** cue, and
    /// deliberately not the only one.
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
        filled: active,
        emphasised_text: active,
    }
}

/// The tabs to show this frame, in display order: the active mode's
/// ordinary tabs, then every contextual tab whose condition holds.
///
/// # The four rules, and why each is what it is
///
/// - **A mode names its tabs, and the mode's order wins.** A mode is a
///   *workspace*, and a workspace that reordered itself to match the
///   underlying manifest would not be one.
/// - **No modes, or an unknown mode id, means every ordinary tab.** The
///   manifest is allowed to have no modes at all — a small application
///   does not need them — and an unknown id is a stale customization,
///   which per `SHELL_FRAMEWORK.md` must lose one thing and not the
///   layout. Showing everything is the safe direction: it can never hide
///   a capability.
/// - **Hidden tabs are skipped.** [`Tab::hidden`] is the operator's own
///   choice and outranks the mode's list.
/// - **Contextual tabs come last and are never mode members.** Their
///   presence is decided by application state rather than by
///   configuration, which is exactly why [`Shell::contextual_tabs`] is a
///   separate field.
/// - **★★★ A tab with nothing left to show is not shown.** The fifth rule,
///   added with [`crate::manifest::Item`]'s `visible_when`, and it is the
///   symmetric completion of the band's own — *a group with nothing left is
///   not drawn at all*. Without it the two halves disagree: hide every item on
///   a tab and the groups all vanish, leaving a tab an operator can click and
///   an empty band beneath it.
///
///   ★ It is also what makes a **generous tab list** safe, which is the point.
///   A mode can name a tab it only sometimes needs, hide the items that do not
///   apply, and the tab appears exactly when it has something to offer. That
///   turns `Mode::tabs` from *"which tabs exist here"* into *"which tabs may
///   appear here"*, and it is the mechanism by which a command can live on the
///   tab it belongs on rather than the tab a mode happened to be granted —
///   `RIBBON_IA.md` records three commands that moved for want of it.
///
///   ★★ A tab whose items carry **no conditions at all** is never affected:
///   the question asked is *"is every item conditioned away?"*, and an
///   unconditioned item answers no. So this cannot hide a tab that a manifest
///   written before conditions existed would have shown.
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
///
/// ★ **A tab with no groups, or groups with no items, is `true`.** That is
/// deliberate and is not the case this rule is about: an empty tab is a
/// manifest under construction, and hiding it would make a half-written
/// manifest look like a working one with a missing feature. What this
/// suppresses is a tab whose items exist and are all **conditioned away**,
/// which is a statement the manifest made on purpose.
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
///
/// # Why a tab is measured at all
///
/// [`super::strip`] must decide which tabs fit **before** any of them is
/// drawn, for the reason [`super::plan`]'s header gives at length: an
/// affordance emitted into whatever the content left is an affordance the
/// content can take. So a tab is estimated the same analytic way a band
/// item is — the galley `egui` will lay out, plus the padding `egui` will
/// add — and the estimate is floored at
/// [`super::plan::MIN_ITEM_WIDTH`] by [`ItemWidths::total`].
///
/// A tab has no icon slot and no gap, so this is text plus padding. The
/// active tab is *not* measured wider even though it is drawn with a
/// stroke and `RichText::strong()`: an `egui` stroke is painted inside the
/// button's own rect, and `strong()` in `egui` 0.35 changes the colour and
/// not the face (see this module's header). Neither costs a point of
/// width, so a strip does not reflow when the operator changes tab — which
/// it very visibly would if the estimate said otherwise.
pub(crate) fn measure_tab(ui: &egui::Ui, tab: &Tab) -> f32 {
    ItemWidths {
        icon: 0.0,
        text: band::text_width(ui, tab_label(tab), &TextStyle::Button),
        gap: 0.0,
        padding: band::button_padding(ui),
    }
    .total()
}

/// Draw a run of tabs and report which one the operator wants active.
///
/// `visible` is the subset [`super::plan::plan_tab_strip`] decided to
/// draw, already in display order and already including the pinned active
/// tab. The tabs that did not fit are drawn by
/// [`render_overflow_menu`] instead; both paths call [`draw_tab`], so a
/// tab in the menu carries the same cues, the same accessible name and the
/// same tooltip as one in the strip. (That is the same rule
/// [`super::band`] follows for overflowed groups, and for the same reason:
/// a second, simpler drawing path for the menu is how the salvage
/// source's caption-less groups happened.)
///
/// Returns the newly clicked tab's id, if any. The caller writes it into
/// [`super::RibbonState`]; this function mutates nothing, so a test can
/// call it without owning shell state.
pub(crate) fn render_tabs(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    visible: &[&Tab],
    active_id: Option<&str>,
) -> Option<String> {
    let mut clicked = None;
    for tab in visible {
        let is_active = active_id == Some(tab.id.as_str());
        if draw_tab(ui, ctx, tab, is_active) {
            clicked = Some(tab.id.clone());
        }
    }
    clicked
}

/// Draw one tab button, wherever it is. Returns whether it was clicked.
///
/// `truncate()` is unconditional, and it is what makes the pinned active
/// tab's promise keepable: [`super::plan::plan_tab_strip`] guarantees the
/// active tab a slot but cannot guarantee that slot is as wide as its
/// label, so when the row is narrow the tab loses characters rather than
/// losing its place. See [`super::band::command_button`]'s `truncate`
/// section on why the band makes the opposite choice.
fn draw_tab(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, tab: &Tab, is_active: bool) -> bool {
    let accent = ctx.theme.palette.accent;
    let cues = tab_cues(is_active);
    let label = tab_label(tab);

    // ★ The active tab's plate and label are painted from the CHROME roles,
    // not from `egui`'s selection visuals. See `DEFECTS.md` D10's second
    // half.
    //
    // `Button::selectable(true, …)` fills from `visuals.selection.bg_fill`
    // and colours its label from the same family — and this theme points
    // `selection.bg_fill` at `palette.selection_fill`, which is the
    // **canvas object-selection tint**: a deliberately translucent blue
    // (alpha 70/255) designed to sit *over page content* without hiding it.
    // Used as a chrome plate it is a 27 % wash, and the label keeps its
    // ordinary foreground — so the active tab rendered as pale blue-grey
    // text on pale blue, while the inactive tab beside it stayed crisp.
    // That is `DEFECTS.md` D2's failure — a label invisible in the default
    // theme — in the one place D2's fix did not reach.
    //
    // The palette has the right pair and has had it all along:
    // `accent` is *"the single accent — selection, focus, the active tab"*
    // and `on_accent` is *"text and icons drawn ON accent"*. Two roles
    // exist here on purpose, and the bug was borrowing a third from a
    // different concept.
    //
    // `super::mode_selector` already does exactly this and looks correct on
    // screen — it paints `palette.accent` and picks `palette.on_accent` for
    // its label. This is that, applied to the tab that sits beside it.
    // Keeping `Button::selectable` rather than hand-painting preserves the
    // inactive tab's *unplated* look, the truncation promise, the sizing
    // and the `Response`; only the two colours are taken back.
    // ★ WEIGHT AND COLOUR ARE ONE DECISION, and `.strong()` is unreachable
    // without the colour that makes it legible — 2026-08-17.
    //
    // This was two independent `if`s, one on `emphasised_text` and one on
    // `filled`, and it was a latent repeat of `DEFECTS.md` **D11**: a tab with
    // `emphasised_text` and not `filled` got a bare `.strong()`, which `egui`
    // resolves to `widgets.active.fg_stroke` — the foreground chosen for the
    // accent-FILLED state — on a background that is not the accent. Pale text
    // on a pale plate, exactly the six labels D11 records.
    //
    // It was not reachable through this crate's own constructor, because
    // `tab_cues` derives all four cues from one `active` flag. It was reachable
    // by anyone building a `TabCues` by hand — every field is `pub`, and this
    // module's own tests do it — so the guarantee rested on a coincidence
    // between two lines rather than on anything structural.
    //
    // Nesting is what makes it structural: the weight can now only be applied
    // inside the branch that has already stated the colour, so the two cannot
    // be separated by an edit that only looks at one of them. Behaviour is
    // unchanged for every caller that uses `tab_cues`.
    //
    // Found by `tools/gates/check-strong-text.sh`, on the run that introduced
    // it — which is the argument for that gate existing: D11 wrote the rule
    // down on 2026-08-14 and it was broken again on 2026-08-17, in a different
    // crate, by someone who had read it.
    let mut text = RichText::new(label);
    if cues.filled {
        text = text.color(ctx.theme.palette.on_accent);
        if cues.emphasised_text {
            // R84's non-colour cue: weight survives greyscale and
            // colour-vision deficiency, which the fill alone does not. Safe
            // here and only here, because the line above has already said what
            // colour the text is.
            text = text.strong();
        }
    }

    let mut button = egui::Button::selectable(cues.filled, text).truncate();
    if cues.filled {
        button = button.fill(accent);
    }
    if cues.outlined {
        // R84's second non-colour cue. A stroke's *presence* is a
        // shape difference and reads with no colour information; the
        // colour it happens to be drawn in is a bonus, not the cue.
        button = button.stroke(Stroke::new(1.0, accent));
    }
    let response = ui.add(button);

    // ★ R84's non-colour cue. Painted rather than themed, because a
    // fill is a colour and a rule under the label is a shape — the
    // whole reason both exist. Drawn at the bottom of the tab's own
    // rect so it moves with the tab and cannot drift out of
    // alignment when the strip's height changes.
    if cues.underline {
        let y = response.rect.bottom() - 1.0;
        ui.painter().line_segment(
            [
                egui::pos2(response.rect.left(), y),
                egui::pos2(response.rect.right(), y),
            ],
            Stroke::new(2.0, accent),
        );
    }

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
///
/// The menu is a vertical run of the *same* tab buttons the strip draws —
/// see [`render_tabs`] on why there is no second drawing path. Clicking
/// one activates it, and because
/// [`super::plan::plan_tab_strip`] pins the active tab, the tab the
/// operator just picked out of the menu is guaranteed to be in the strip
/// on the next frame. That is the property that makes the menu a *route*
/// to a hidden tab rather than a place a tab can be looked at.
///
/// `active_id` is passed even though the active tab is never in `hidden`:
/// it costs nothing, and it means a future change that unpins the active
/// tab cannot accidentally draw it in the menu without its cues.
pub(crate) fn render_overflow_menu(
    ui: &mut egui::Ui,
    ctx: &mut Ctx<'_>,
    hidden: &[&Tab],
    active_id: Option<&str>,
) -> Option<String> {
    let mut clicked = None;
    for tab in hidden {
        let is_active = active_id == Some(tab.id.as_str());
        if draw_tab(ui, ctx, tab, is_active) {
            clicked = Some(tab.id.clone());
        }
    }
    clicked
}

/// Height reserved for one tab-strip row, used to align the mode selector
/// with the tabs.
pub(crate) fn strip_height(ctx: &Ctx<'_>) -> f32 {
    ctx.theme.metrics.control_height
}

/// A thin rule under the tab strip, separating it from the band.
pub(crate) fn strip_underline(ui: &mut egui::Ui, ctx: &Ctx<'_>) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().line_segment(
        [rect.left_center(), rect.right_center()],
        Stroke::new(1.0, ctx.theme.palette.outline),
    );
}

#[cfg(test)]
mod tests {
    use crate::manifest::Item;
    /// ★★★ **A tab every one of whose items is conditioned away is not
    /// shown at all.**
    ///
    /// The symmetric completion of the band's *"a group with nothing left is
    /// not drawn"*. Without it the two disagree, and the operator gets a tab
    /// they can click with an empty band beneath it.
    ///
    /// ★ This is also the rule that makes a **generous tab list** safe, which
    /// is the point of having it: a mode can name a tab it only sometimes
    /// needs and the tab appears exactly when it has something to offer. It is
    /// what would let a command live on the tab it belongs on rather than the
    /// tab a mode happened to be granted — `RIBBON_IA.md` records three
    /// commands that moved for want of exactly this.
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

    /// ★★ A manifest that never uses conditions is untouched by the rule.
    ///
    /// The guard that makes it safe to add to a shipped shell: the question
    /// asked is *"is every item conditioned away?"*, and an unconditioned item
    /// answers no. An **empty** tab is also still shown — that is a manifest
    /// under construction, and hiding it would make a half-written manifest
    /// look like a working one with a feature missing.
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
    ///
    /// `MODES_AND_PANELS.md` Part 1's whole premise: Read is *File ·
    /// View*, Edit is everything. Nothing in this crate names those
    /// modes — they come out of the manifest — which is the "Read/Review/
    /// Edit is configuration, not a built-in" requirement made concrete.
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
    ///
    /// A mode is a workspace. A workspace that silently reordered itself
    /// to match the underlying tab list would not be one, and the
    /// operator who put Tools first would find it back in fourth place
    /// with no explanation.
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
    ///
    /// Both are fail-soft in the safe direction: showing everything can
    /// never hide a capability, and `SHELL_FRAMEWORK.md`'s rule for a
    /// stale customization is that it loses one thing rather than the
    /// layout.
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
    ///
    /// `RIBBON_IA.md` §4: Format appears when something is selected. It
    /// is appended after the mode's tabs rather than inserted into them,
    /// because a tab that changed the *position* of the others as it came
    /// and went would move every target under the operator's cursor.
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
    ///
    /// The opposite of the empty-string case in
    /// [`super::ctx::condition_holds`], and deliberately so: a tab placed
    /// in `contextual_tabs` with **no** `visible_when` key at all has not
    /// said when it appears, and a contextual tab that is always present
    /// is an ordinary tab in the wrong list.
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
    ///
    /// The operator's own hide outranks the mode's list — hiding is a
    /// customization, and `SHELL_FRAMEWORK.md` §5 puts "hide them" in the
    /// allowed column.
    #[test]
    fn a_hidden_tab_is_skipped_even_when_a_mode_names_it() {
        let mut shell = shell();
        shell.tabs.as_mut().expect("has tabs")[1].hidden = Some(true);
        assert_eq!(
            ids(&visible_tabs(&shell, Some("edit"), &ConditionSet::new())),
            ["file", "pages", "tools"]
        );
    }

    /// **★ An active tab that disappears falls back to the first visible
    /// one, in the same frame.**
    ///
    /// Two ways a tab disappears while active: the operator switches to a
    /// mode that does not contain it, or a contextual tab's condition
    /// stops holding. Both must recover without a blank band and without
    /// a click. See the module header on why the alternatives are worse.
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

    /// **★ R84: the active tab differs from an inactive one by more than
    /// colour.**
    ///
    /// A fill-only cue is invisible to a colour-blind operator, invisible
    /// on a projector, and invisible in a greyscale screenshot — which is
    /// also how it becomes invisible in a bug report. Two of the four
    /// cues here are the presence or absence of a *shape*, and either
    /// alone is sufficient to read the strip.
    ///
    /// Written against the *count* of non-colour cues rather than against
    /// the specific ones, so a future redesign may swap an underline for
    /// a top rule or a border for a notch — but may not quietly reduce
    /// the set to the fill.
    ///
    /// **`emphasised_text` is deliberately not counted.** See the module
    /// header: `RichText::strong()` in `egui` 0.35 is a stronger *colour*
    /// at the same weight, and treating it as a weight cue is exactly the
    /// mistake this project's own preferences document made.
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
