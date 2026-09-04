//! The **trailing controls** — the far right of the tab-strip row, past the
//! mode selector.
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────┐
//! │ [Open][Save] │ File View Pages ⏷ 3 more  ( Read │ Review │ Edit ) [ ] │
//! └──────────────────────────────────────────────────────────────────────┘
//!   └── QAT ───┘  └───── tabs ────────────┘  └── mode selector ──┘  └ here
//! ```
//!
//! # What this region is for, and what it is emphatically not
//!
//! [`crate::manifest::Trailing`] carries the argument in full. In one line:
//! it is the seam for a control that belongs **beside the mode selector** and
//! whose presence is a property of the machine rather than of the program —
//! and it carries [`crate::manifest::Item`]s, not bare ids, so that
//! `visible_when` can make such a control *absent* rather than greyed.
//!
//! It is **not** a second QAT. The QAT is the handful of verbs an operator
//! uses continuously, on the left where reading starts. This end of the row is
//! the last thing read, and a command that belongs here is one that ends the
//! current activity rather than one that advances it.
//!
//! # ★★ Why almost every line of this file is borrowed from [`super::qat`]
//!
//! Deliberately, and the borrowing is the point rather than an accident of
//! copy-and-paste. Both regions are:
//!
//! - a **short flat list of controls** with no menu behind them, so a control
//!   that does not fit must be dropped rather than moved somewhere;
//! - **reserved before the tabs**, so their width is granted by
//!   [`super::plan::plan_strip_row`] rather than discovered while drawing;
//! - drawn into a `Ui` whose `max_rect` is exactly the granted width.
//!
//! So the measurement functions are [`super::qat`]'s own — imported, not
//! reimplemented. That is [`super::band::measure_item`]'s standing rule
//! applied across a module boundary: **measure what the renderer draws.** Two
//! measurement functions for one button shape is how the QAT once came out
//! narrower than it drew and landed on top of the first tab.
//!
//! # ★ The one behavioural difference from the QAT: no trailing separator
//!
//! The QAT ends in a `ui.separator()`, because it has the tabs on its right
//! and needs a rule between two unlike things. This region ends at the edge of
//! the window. A rule there would be a vertical line down the right-hand side
//! of the ribbon, which reads as a panel border rather than as a divider, and
//! there is nothing on the far side of it to divide from.
//!
//! # ★ Hidden items are removed BEFORE measurement, not while drawing
//!
//! [`measure`] and [`render`] both filter on
//! [`super::sizing::visible`] first. If measurement counted a hidden control
//! the region would reserve space for it, the row would grant that space, and
//! the tabs would be narrowed by the width of a button nobody can see — the
//! same reflow rule [`crate::manifest::Item::Command`]'s `visible_when`
//! documents for a band group, applied to a region whose whole reason for
//! existing is that its contents come and go.

use crate::commands::Command;
use crate::manifest::{Item, Trailing};

use super::a11y;
use super::band;
use super::ctx::Ctx;
use super::qat::{control_width, min_control_width, shows_label};
use super::report;
use super::sizing;

/// The items that will actually be drawn this frame.
///
/// One helper rather than the same `filter` written twice, because
/// [`measure`] and [`render`] disagreeing about which items exist is exactly
/// the class of defect this region's reservation is supposed to make
/// impossible.
fn shown<'a>(trailing: Option<&'a Trailing>, ctx: &Ctx<'_>) -> Vec<(&'a str, Command)> {
    let Some(trailing) = trailing else {
        return Vec::new();
    };
    trailing
        .items()
        .iter()
        .filter(|item| sizing::visible(item, ctx.conditions))
        .filter_map(|item| match item {
            Item::Command { id, .. } => ctx
                .registry
                .get(id)
                .map(|command| (id.as_str(), command.clone())),
            // A separator or a custom item in this region draws nothing. The
            // variants exist on `Item` for the band's sake; giving them
            // meaning here would mean this region grew a layout of its own,
            // and a region three buttons wide does not need one.
            Item::Separator | Item::Custom { .. } => None,
        })
        .collect()
}

/// The width this region asks for.
///
/// `0.0` when there is nothing to draw — no trailing list, an empty one, or
/// one whose every item is hidden or names a command that is not registered.
/// A zero width is what makes the region **disappear** rather than leave a
/// gap, which is R9 in the layout rather than in the painting.
pub(crate) fn measure(ui: &egui::Ui, ctx: &Ctx<'_>, trailing: Option<&Trailing>) -> f32 {
    let shown = shown(trailing, ctx);
    if shown.is_empty() {
        return 0.0;
    }
    let gap = ui.spacing().item_spacing.x;
    let total: f32 = shown
        .iter()
        .map(|(_, command)| control_width(ui, ctx, command))
        .sum();
    // A gap before the first control as well as between them: this region
    // butts directly against the mode selector, and two controls of different
    // kinds sharing an edge read as one control with a seam in it.
    total + gap * shown.len() as f32
}

/// The narrowest this region can be and still be worth drawing.
///
/// The first control's own floor, exactly as [`super::qat::min_width`]
/// computes it — see [`super::plan::row`]'s header on why a region granted
/// less than a control's floor gets a control drawn *outside* its rectangle
/// rather than a smaller one.
pub(crate) fn min_width(ui: &egui::Ui, ctx: &Ctx<'_>, trailing: Option<&Trailing>) -> f32 {
    let shown = shown(trailing, ctx);
    shown
        .first()
        .map_or(0.0, |(_, command)| min_control_width(ui, ctx, command))
}

/// Draw the trailing controls.
///
/// Dropping rule, disclosure and containment check are [`super::qat::render`]'s
/// verbatim, and the reasoning there applies here unchanged: a control below
/// its floor is drawn outside the rectangle it was given, so the loop stops
/// rather than truncating, and what it dropped is announced.
pub(crate) fn render(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, trailing: Option<&Trailing>) {
    let shown = shown(trailing, ctx);
    if shown.is_empty() {
        return;
    }
    let total = shown.len();
    let mut dropped = 0_usize;

    for (_, command) in shown {
        if ui.available_width() < min_control_width(ui, ctx, &command) {
            dropped += 1;
            continue;
        }
        let enabled = command.is_enabled(ctx.conditions);
        let selected = ctx
            .conditions
            .is_set(&band::selected_condition(&command.id));
        let with_label = shows_label(&command, ctx.icons.is_some());

        let response =
            super::control::command_button(ui, ctx, &command, with_label, selected, enabled, true);

        a11y::describe_command(&response, &command, with_label, enabled);
        // ★ R9's second half — *"greying … is always explained on hover"*.
        // A control in this region that is present but disabled is
        // temporarily unavailable, and the tooltip is the only place the
        // operator can find out why, so it is shown on the disabled control
        // as well as on the live one.
        let response = match (&command.tooltip, enabled) {
            (Some(tip), true) => response.on_hover_text(tip),
            (Some(tip), false) => response.on_disabled_hover_text(tip),
            (None, _) => response,
        };

        ctx.reporter
            .report(response.rect, || report::trailing_item(&command.id));

        if response.clicked() {
            ctx.invoke(command.handler);
            crate::verify::event("ribbon-command-invoked")
                .kv("id", &command.id)
                .kv("handler", command.handler.get())
                .kv("surface", "trailing")
                .emit();
        }
    }

    if dropped > 0 {
        crate::verify::event("ribbon-trailing-controls-dropped")
            .kv("dropped", dropped.to_string())
            .kv("of", total.to_string())
            .emit();
    }
}
