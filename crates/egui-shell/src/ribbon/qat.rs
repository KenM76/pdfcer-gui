//! The Quick Access Toolbar — the handful of controls that must never
//! sit behind a tab switch.
//!
//! # Why it exists — the defect it prevents
//!
//! Without one, an operator working in the tab their job actually lives in
//! has **no undo, no zoom and no navigation control** without first leaving
//! that tab. Capturing each tab of a running application shows it directly:
//! the controls used continuously are the ones that are absent from every
//! tab but the one that happens to own them.
//!
//! A ribbon emits only the *active* tab's band. Anything an operator uses
//! continuously therefore cannot live in a band, because using it means
//! leaving the tab they are working in and coming back. The QAT is drawn
//! on every frame regardless of the active tab, which is the whole
//! feature.
//!
//! # One command, one *tab* — and the QAT is not a tab
//!
//! `SHELL_FRAMEWORK.md` §5 amends the one-command-one-place rule
//! specifically to permit this: *"a command may appear on exactly one
//! **tab**; the QAT and status bar may mirror it."*
//!
//! The amendment matters. Without it, putting Open on the QAT would mean
//! removing it from the File tab, and an operator looking for "open" on
//! the tab called File would not find it. With it, the QAT is a shortcut
//! to a known place rather than a second place to hunt.
//!
//! The uniqueness check lives in [`crate::manifest::Shell::validate`] and
//! already counts tabs only, so nothing here has to enforce it.
//!
//! # Icon-only is earned, not assumed
//!
//! A QAT is conventionally icon-only. This module will not draw an
//! icon-only control unless the command supplies **both** an icon key and
//! a tooltip — see `shows_label`.
//!
//! The reason is that the tooltip is the icon's accessible name (see
//! [`super::a11y`]). A command with an icon and no tooltip, rendered
//! icon-only, would be a control that announces nothing to a screen
//! reader and explains nothing on hover. Making the label reappear in
//! that case turns an accessibility failure into a *cosmetic* one, and
//! makes the failure impossible to ship rather than merely discouraged.
//!
//! There is a second, human reason to be reluctant about icon-only, and it
//! is easiest to see on one control: a bare disk glyph says "Save" — i.e.
//! "overwrite what I opened" — so an application whose save command does
//! something else has drawn a small lie. Convention loses to not misleading
//! anyone.
//!
//! That judgement is the **application's**, not the shell's: only the
//! application knows that its save command does something a disk glyph
//! misdescribes. The seam is already there — an application that wants a
//! label on a QAT control registers the command without an icon key, and
//! gets one.
//!
//! Design and rationale: `docs/modules/egui-shell/ribbon/qat.md`.

use egui::TextStyle;

use crate::commands::Command;
use crate::manifest::Qat;

use super::a11y;
use super::band;
use super::ctx::Ctx;
use super::measure;
use super::plan::ItemWidths;
use super::report;

/// Whether a QAT control draws its text label.
#[must_use]
pub(crate) fn shows_label(command: &Command, can_paint_icons: bool) -> bool {
    !can_paint_icons || command.icon.is_none() || command.tooltip.is_none()
}

/// The width the QAT will occupy, measured **before** it is drawn.
pub(crate) fn measure(ui: &egui::Ui, ctx: &Ctx<'_>, qat: Option<&Qat>) -> f32 {
    let Some(qat) = qat else {
        return 0.0;
    };
    if qat.ids().is_empty() {
        return 0.0;
    }

    let gap = ui.spacing().item_spacing.x;
    let mut total = 0.0_f32;
    let mut drawn = 0_usize;

    for id in qat.ids() {
        // `registry.get` rather than `ctx.command`, so measuring does not
        // emit the unknown-id disclosure a second time — `render` will
        // emit it on the same frame and one defect deserves one line.
        let Some(command) = ctx.registry.get(id) else {
            continue;
        };
        total += control_width(ui, ctx, command);
        drawn += 1;
    }

    if drawn == 0 {
        return 0.0;
    }
    // (drawn − 1) inter-control gaps, plus the trailing separator with a
    // gap on each side — the same figure `measure::separator_width` computes
    // for the band's inter-group rule.
    total + gap * (drawn as f32 - 1.0) + measure::separator_width(ui)
}

/// The pieces of one QAT control's width, in the shape
/// [`super::band::command_button`] will actually draw them.
fn control_pieces(ui: &egui::Ui, ctx: &Ctx<'_>, command: &Command) -> ItemWidths {
    let with_label = shows_label(command, ctx.icons.is_some());
    ItemWidths {
        icon: if command.icon.is_some() {
            ctx.theme.metrics.icon_pts
        } else {
            0.0
        },
        text: if with_label {
            measure::text_width(ui, &command.label, &TextStyle::Button)
        } else {
            0.0
        },
        // `icon_spacing`, not the theme's gutter — see
        // [`super::band::measure_item`] on why the two disagree at the
        // comfortable density and why under-estimating is the dangerous
        // direction.
        gap: ui.spacing().icon_spacing,
        padding: measure::button_padding(ui),
    }
}

/// The width one QAT control wants.
pub(super) fn control_width(ui: &egui::Ui, ctx: &Ctx<'_>, command: &Command) -> f32 {
    control_pieces(ui, ctx, command).total()
}

/// The narrowest one QAT control can be drawn, with its label truncated
/// all the way down to the ellipsis.
pub(super) fn min_control_width(ui: &egui::Ui, ctx: &Ctx<'_>, command: &Command) -> f32 {
    let pieces = control_pieces(ui, ctx, command);
    ItemWidths {
        text: if pieces.text > 0.0 {
            measure::text_width(ui, "…", &TextStyle::Button)
        } else {
            0.0
        },
        ..pieces
    }
    .total()
}

/// The narrowest a QAT worth drawing can be: its **first** control's
/// floor, plus the divider that separates it from the tab strip.
pub(crate) fn min_width(ui: &egui::Ui, ctx: &Ctx<'_>, qat: Option<&Qat>) -> f32 {
    let Some(qat) = qat else {
        return 0.0;
    };
    qat.ids()
        .iter()
        .find_map(|id| ctx.registry.get(id))
        .map_or(0.0, |command| min_control_width(ui, ctx, command))
}

/// Draw the quick-access toolbar.
pub(crate) fn render(ui: &mut egui::Ui, ctx: &mut Ctx<'_>, qat: Option<&Qat>) {
    let Some(qat) = qat else {
        return;
    };
    if qat.ids().is_empty() {
        return;
    }

    let mut dropped = 0_usize;

    for id in qat.ids() {
        let Some(command) = ctx.command(id).cloned() else {
            continue;
        };
        // The containment rule. `available_width()` is honest here
        // because the caller gave this `Ui` an explicit `max_rect`; what
        // it cannot tell us is that a button below its floor overflows
        // rather than shrinking, which is why the check is against
        // `min_control_width` and not against zero.
        if ui.available_width() < min_control_width(ui, ctx, &command) {
            dropped += 1;
            continue;
        }
        let enabled = command.is_enabled(ctx.conditions);
        let selected = ctx
            .conditions
            .is_set(&band::selected_condition(&command.id));
        let with_label = shows_label(&command, ctx.icons.is_some());

        // `truncate: true` — the QAT is a fixed cost with no menu behind
        // it, so a control that does not fit must lose characters rather
        // than lose its place. See `band::command_button`.
        let response =
            super::control::command_button(ui, ctx, &command, with_label, selected, enabled, true);

        a11y::describe_command(&response, &command, with_label, enabled);
        let response = match (&command.tooltip, enabled) {
            (Some(tip), true) => response.on_hover_text(tip),
            (Some(tip), false) => response.on_disabled_hover_text(tip),
            (None, _) => response,
        };

        ctx.reporter
            .report(response.rect, || report::qat_item(&command.id));

        if response.clicked() {
            ctx.invoke(command.handler);
            crate::verify::event("ribbon-command-invoked")
                .kv("id", &command.id)
                .kv("handler", command.handler.get())
                .kv("surface", "qat")
                .emit();
        }
    }

    if dropped > 0 {
        crate::verify::event("ribbon-qat-controls-dropped")
            .kv("dropped", dropped.to_string())
            .kv("of", qat.ids().len().to_string())
            .emit();
    }

    // The divider only if there is room for it; see the header.
    if ui.available_width() >= measure::separator_width(ui) {
        ui.separator();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::HandlerToken;

    /// **A control is icon-only only when it has an accessible name to
    /// go with the icon.**
    #[test]
    fn a_control_goes_icon_only_only_when_it_has_a_tooltip() {
        let full = Command::new("file.open", "Open…", HandlerToken::new(1))
            .with_icon("open")
            .with_tooltip("Open a document");
        assert!(
            !shows_label(&full, true),
            "icon plus tooltip may be icon-only"
        );

        let no_tooltip = Command::new("file.open", "Open…", HandlerToken::new(1)).with_icon("open");
        assert!(
            shows_label(&no_tooltip, true),
            "an icon with no tooltip has no accessible name, so the label must stay"
        );

        let no_icon = Command::new("file.open", "Open…", HandlerToken::new(1))
            .with_tooltip("Open a document");
        assert!(
            shows_label(&no_icon, true),
            "nothing to draw instead of the label"
        );

        let bare = Command::new("file.open", "Open…", HandlerToken::new(1));
        assert!(shows_label(&bare, true));
    }

    /// **An application that cannot paint icons still gets labels.**
    #[test]
    fn a_control_keeps_its_label_when_the_application_cannot_paint_icons() {
        let full = Command::new("file.open", "Open…", HandlerToken::new(1))
            .with_icon("open")
            .with_tooltip("Open a document");

        assert!(
            !shows_label(&full, true),
            "with a painter available, icon plus tooltip may go icon-only"
        );
        assert!(
            shows_label(&full, false),
            "with no painter there is nothing to draw, so the label must stay \
             — otherwise the control is a blank box"
        );
    }

    /// The rule composes with [`super::a11y::accessible_name`]: whatever
    /// [`shows_label`] decides, the control has a name to announce.
    #[test]
    fn every_qat_control_has_an_accessible_name_whatever_the_rule_decides() {
        let commands = [
            Command::new("a", "Open…", HandlerToken::new(1))
                .with_icon("open")
                .with_tooltip("Open a document"),
            Command::new("b", "Open…", HandlerToken::new(2)).with_icon("open"),
            Command::new("c", "Open…", HandlerToken::new(3)).with_tooltip("Open a document"),
            Command::new("d", "Open…", HandlerToken::new(4)),
            Command::new("e", "", HandlerToken::new(5)).with_icon("open"),
        ];
        for command in &commands {
            let with_label = shows_label(command, true);
            let name = a11y::accessible_name(command, with_label);
            assert!(
                !name.trim().is_empty(),
                "{} would be announced as nothing",
                command.id
            );
            if !with_label {
                assert_eq!(
                    Some(name),
                    command.tooltip.as_deref(),
                    "an icon-only control must announce its tooltip"
                );
            }
        }
    }
}
