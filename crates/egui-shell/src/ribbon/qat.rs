//! The Quick Access Toolbar — the handful of controls that must never
//! sit behind a tab switch.
//!
//! # Why it exists, in the words of the defect it prevents
//!
//! The salvage source's own doc comment records what happens without one,
//! and it is the clearest possible statement of the requirement:
//!
//! > The consequence, confirmed by capturing each tab of the running
//! > application on 2026-08-08: an operator on **Measure** — this
//! > operator's own stated primary activity — had **no undo, no zoom and
//! > no page control** without first leaving the tab they were working
//! > in.
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
//! # ★ Icon-only is earned, not assumed
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
//! There is a second, human reason to be reluctant about icon-only, and
//! the salvage source states it about one specific control:
//!
//! > A bare disk glyph says "Save" — i.e. "overwrite what I opened" —
//! > which is the small lie `save_button`'s own doc comment was written
//! > to forbid. Convention loses to not misleading anyone.
//!
//! That judgement is the **application's**, not the shell's: only the
//! application knows that its save command does something a disk glyph
//! misdescribes. The seam is already there — an application that wants a
//! label on a QAT control registers the command without an icon key, and
//! gets one.

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
///
/// `true` unless **all three** hold: the command names an icon, it has a
/// tooltip to serve as that icon's accessible name, and the application
/// actually supplied a painter that can draw the icon.
///
/// The first two are the accessibility rule described in the module
/// header. The third is a *rendering* rule and it was learned the
/// embarrassing way: an application that registers icon keys but supplies
/// no [`super::Ribbon::with_icon_painter`] used to get a row of blank
/// boxes — a control with no label, no glyph and no explanation. That is
/// precisely the placeholder the shell's no-placeholders rule forbids,
/// and it looks to an operator exactly like the application is broken.
///
/// Declaring an icon is an intention; being able to paint one is a
/// capability, and only the second may be traded against the label.
/// Falling back to text is always safe — it is what a command with no
/// icon does anyway — so the degraded state is a slightly wider button
/// rather than an invisible one.
#[must_use]
pub(crate) fn shows_label(command: &Command, can_paint_icons: bool) -> bool {
    !can_paint_icons || command.icon.is_none() || command.tooltip.is_none()
}

/// The width the QAT will occupy, measured **before** it is drawn.
///
/// # ★ Why this exists at all — the QAT used to be unmeasured
///
/// The tab-strip row reserves space outermost-first
/// ([`super::plan::plan_strip_row`]), and a reservation you cannot measure
/// is not a reservation. Before this function existed the QAT was simply
/// emitted into a left-to-right layout and whatever it took, it took —
/// which is the immediate-mode spelling that produces
/// `MODES_AND_PANELS.md` failure mode #8. Measured at a 180 pt viewport
/// with real font metrics, the two-control QAT of the test manifest ran
/// from **x = −6** to x = 160 and the tabs behind it were entirely off
/// screen.
///
/// The measurement mirrors [`render`] line for line, and it has to:
///
/// - the same `shows_label` decision, because an icon-only control is
///   much narrower than a labelled one and getting that backwards
///   mis-budgets every QAT that has a painter;
/// - the same skip for an unknown command id, because a control that is
///   not drawn must not be budgeted (the same rule
///   [`super::band::measure_item`] follows, for the same reason);
/// - the trailing `ui.separator()`, which is the visible divider between
///   the QAT and the tab strip and is part of the QAT's cost.
///
/// Returns `0.0` for an absent or empty QAT, so a manifest without one
/// costs the row nothing at all rather than a stray separator.
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
///
/// # ⚠ The icon slot is allocated whenever the command **names** an icon
///
/// Not whenever one can be painted. [`super::band::command_button`] pushes
/// an `egui::Atom::custom` of `icon_pts` for any command with an icon key,
/// and only *paints* into it if the application supplied a painter — so a
/// command with a key and no painter draws an empty square, and that
/// square has width.
///
/// Budgeting it on `ctx.icons.is_some()` instead was a real defect and a
/// subtle one: the measured QAT came out narrower than the drawn one by
/// `icon_pts + icon_spacing` per control, the row granted it that narrower
/// figure, and the controls then overflowed their own region and were
/// drawn across the first tab. `the_tab_strip_never_runs_under_the_mode_selector_or_the_qat`
/// caught it at 128 pt.
///
/// The rule this restates is [`super::band::measure_item`]'s, which has
/// always had it right: **measure what the renderer draws, not what the
/// application intended.**
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
///
/// [`super::measure::min_button_width`] is the text-only case; a QAT control
/// may also carry an icon slot, which `truncate()` cannot shrink at all.
/// So the floor is *per control*, and it is what [`render`] tests each
/// control's remaining room against before drawing it — see that
/// function's header on why drawing it anyway is not an option.
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
///
/// [`super::plan::plan_strip_row`] uses this as the QAT's `grant` floor:
/// below it the region cannot hold even one control, and granting a
/// sliver produces a button drawn outside its own rectangle rather than a
/// narrower one.
///
/// Zero for an absent or empty QAT, so a manifest without one is not
/// charged a floor it will never use.
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
///
/// Ordering is the manifest's; unknown ids are skipped with a disclosure
/// by [`Ctx::command`].
///
/// # ★ How a QAT that does not fit degrades, and why it is not "truncate"
/// alone
///
/// The caller lays this out inside a `Ui` whose `max_rect` is the width
/// [`measure`] asked for and [`super::plan::plan_strip_row`] granted. When
/// those differ — a QAT wider than the row can afford — the controls
/// truncate, *and the ones that still do not fit are not drawn at all*.
///
/// The second half is not belt-and-braces; it is required, and the reason
/// is measured in [`super::measure::min_button_width`]:
/// **`Button::truncate()` stops shrinking** at padding-plus-ellipsis. Ask
/// a button to lay itself out in 6 pt and it lays itself out in 19.7 and
/// overflows — silently, because `egui` does not clip children to a `Ui`'s
/// `max_rect`. A loop that merely truncated would therefore walk straight
/// out of the QAT's rectangle and draw over the tab strip, which is the
/// defect [`super::strip`] exists to retire, reached by trying to be
/// accommodating.
///
/// So each control is drawn only while a whole button still fits, and the
/// ones dropped are disclosed as `ribbon-qat-controls-dropped`. Dropping
/// is a real loss — the QAT is *"the handful of controls that must never
/// sit behind a tab switch"* — and it is why the row reserves the QAT
/// first and why the disclosure exists. It happens only at widths where
/// the alternative is a control drawn on top of another one.
///
/// The trailing `ui.separator()` is subject to the same rule: it is the
/// divider between the QAT and the tabs, and a divider drawn past the end
/// of the QAT would be a rule through the middle of the first tab.
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
        // ★ The containment rule. `available_width()` is honest here
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

    /// **★ A control is icon-only only when it has an accessible name to
    /// go with the icon.**
    ///
    /// The tooltip *is* the accessible name of an icon-only control. A
    /// command with an icon and no tooltip, drawn icon-only, would
    /// announce nothing to a screen reader and explain nothing on hover
    /// — and would look completely correct in a screenshot, which is why
    /// this needs a test rather than a review.
    ///
    /// Falling back to the label converts an accessibility failure into a
    /// cosmetic one. That is the trade this rule makes, deliberately.
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

    /// **★ An application that cannot paint icons still gets labels.**
    ///
    /// Registering an icon key is an *intention*; supplying an
    /// [`super::Ribbon::with_icon_painter`] is the *capability*. Only the
    /// capability may be traded against the label, because trading on the
    /// intention alone produces a control with no glyph, no text and no
    /// explanation — a blank box.
    ///
    /// This is not hypothetical. `pdfcer-gui` wired the ribbon at S2 with a
    /// full set of icon keys and no painter, and its QAT rendered as four
    /// empty rectangles. Every unit test passed; a screenshot caught it.
    /// The lesson is the same one `DEFECTS.md` D2 taught — a property that
    /// only exists once something is *rendered* cannot be asserted from
    /// the values that went into it.
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
    ///
    /// Stated as a joint property because the two functions are only
    /// correct *together* — either alone can be changed into something
    /// that leaves a control anonymous.
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
