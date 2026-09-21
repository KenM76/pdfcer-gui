//! # `canvas::forms::choosing::typing` — the **editable** combo box
//!
//! `/Ff` bit 18 `Combo` **and** bit 19 `Edit`: a drop-down whose value need not
//! be one of its options. `FORMS_PARITY.md` §8.1 row 15.
//!
//! ## Contract
//!
//! One entry point, [`type_into`], called from [`super::choose`] before any of
//! its own work. It owns the whole frame for an editable combo and answers the
//! same thing `choose` does — whether this frame's primary press belonged here.
//!
//! Every outcome still leaves as [`FormEdit::SetChoice`], the same command a
//! pick raises, because `pdfcer-core`'s `set_choice_value` already resolves a
//! value against `/Opt` first and falls to a free-text branch only when it
//! matches nothing (`edit.rs`, `editable_combo`). So a typed string and a
//! picked row are one verb, and nothing here has to decide which the operator
//! meant — the engine decides, from the file.
//!
//! ## ★★★ Why this is a separate surface rather than a flag on [`super::choose`]
//!
//! A plain combo box is a **focus ring plus a popup**: it draws nothing over
//! the widget, reads the vertical arrows to move a highlight, and every value
//! it can hold is already in the list. An editable one is a **live text box
//! with a drop button**: it covers the widget, owns the vertical arrows for
//! its caret, and its value may be a string that exists nowhere in the file.
//!
//! Those differ in what has focus, in which keys mean what, in what is painted
//! over the page, and in when a write happens. Threading a boolean through
//! `choose` would have put a two-armed `if` at each of those four points and
//! called it one function.
//!
//! What *is* shared is shared by call: [`super::list`] draws the popup,
//! [`super::wanted`] resolves a pick, and [`super::super::textbox`] dresses the
//! text box — so an editable combo's list looks exactly like a plain one's and
//! its text box honours `/Q` and `/MK` `/BG` exactly as a `/Tx` field does.
//!
//! ## ★★ The measured behaviour this reproduces
//!
//! Photographed in Acrobat Pro on `fixtures/all-field-kinds.pdf`, which carries
//! `ComboEdit` for the purpose (`tools/acrobat-form-study.ps1`):
//!
//! * **Unfocused, it is indistinguishable from a plain combo box** — the same
//!   `/AP`, the same square drop arrow. Nothing here paints until it is
//!   focused, which is rule 4 and also what the photograph shows.
//! * **A click in the text area gives a caret and selects the whole value.**
//!   It does **not** open the list. The value came from a list, so replacing
//!   it wholesale is the common act.
//! * **A click on the drop button opens the list**, flush under the field,
//!   field-width, square-cornered, unshadowed — [`super::list`]'s geometry
//!   already, because that was measured from the same photographs.
//! * **A focused editable combo shows a chevron where the `/AP` drew a square
//!   button.** That is not decoration: the text box covers the appearance
//!   stream, so a field that drew no arrow of its own would lose the only mark
//!   that says it is a drop-down at the moment the operator starts using it.

use egui::{Key, Rect, Ui};

use super::{ListState, forget, list, load, pointer_in_list, store, wanted};
use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::canvas::forms::boxes::WidgetBox;
use crate::canvas::forms::{Focus, note_escape, store_focus, textbox};
use crate::canvas::tabnav;
use crate::panels::forms::edit::FormEdit;

/// The drop button's width, from a rectangle in whichever space it is in.
///
/// ★ Taken from the **height**, so the button is the square Acrobat draws and
/// scales with the field rather than with the zoom. Capped at half the width
/// so a wide-and-short field does not end up all button; floored at one unit
/// so the arithmetic below never produces an inverted rectangle.
///
/// Called on the **page** rectangle to decide what a click meant and on the
/// **screen** rectangle to decide where to draw. Those two disagree slightly
/// for a widget small enough that [`crate::canvas::forms::boxes::editor_rect`]
/// grew it to the legible minimum, and that is accepted: the alternative is a
/// hit region derived from a rectangle the operator cannot see.
pub(super) fn arrow_strip(rect: Rect) -> f32 {
    rect.height().min(rect.width() * 0.5).max(1.0)
}

/// Whether a click at `point` — in the same space as `rect` — asked for the
/// list rather than for a caret.
pub(super) fn hit_arrow(rect: Rect, point: egui::Pos2) -> bool {
    point.x >= rect.max.x - arrow_strip(rect)
}

/// Draw the text box and its drop button, and answer the frame.
///
/// Returns whether this frame's primary press belonged here.
#[expect(
    clippy::too_many_arguments,
    // ui-text-exempt: a clippy lint reason, read by the compiler and by the
    // next reader of this file, never by an operator.
    reason = "the same shape `super::list` carries, and for the same reason: \
              every argument is a distinct fact about one control -- where it \
              sits, which field it belongs to, what it lists, what is \
              selected, how its text is quadded, and where its commands go. \
              Re-deriving the three that came out of `super::choose`'s \
              destructure would put a second unreachable `BoxKind` arm in the \
              crate, which is a worse trade than a long signature."
)]
pub(super) fn type_into(
    ui: &mut Ui,
    focus: Focus,
    widget_box: &WidgetBox,
    rect: Rect,
    options: &[(String, String)],
    selected: &[String],
    align: pdfcer_core::vartext::Quadding,
    actions: &mut Vec<Action>,
) -> bool {
    let ctx = ui.ctx().clone();
    let id = focus.editor_id();
    let strip = arrow_strip(rect);
    let text_rect = Rect::from_min_max(rect.min, egui::pos2(rect.max.x - strip, rect.max.y));
    let arrow_rect = Rect::from_min_max(egui::pos2(rect.max.x - strip, rect.min.y), rect.max);

    let mut state = load(&ctx, id);
    state.hl = state.hl.min(options.len().saturating_sub(1));
    let mut draft = focus.draft.clone();
    let mut chosen: Option<usize> = None;

    // ★★ The drop button is interacted with BEFORE the text box is laid, so
    // its press is consumed here rather than reaching the `TextEdit` beneath
    // the pointer — `ui.put` allocates the text rectangle only, but an
    // `Area`-hosted popup from a previous frame can still be over this one.
    let button = ui.interact(arrow_rect, id.with(ARROW), egui::Sense::click());
    chevron(ui, arrow_rect, state.open);

    let response = textbox::lay(
        ui,
        &mut draft,
        &textbox::Spec {
            id,
            rect: text_rect,
            align,
            fill: widget_box.fill,
            multiline: false,
            password: false,
            trace: (!focus.seated).then_some(focus.field.as_str()),
        },
    );
    tabnav::publish(&ctx, tabnav::Scope::Field, id);

    if !focus.seated {
        response.request_focus();
        // Select-all, not caret-at-end, and the difference is measured — see
        // [`textbox::seat`].
        textbox::seat(&ctx, id, &draft, true);
    } else if !response.has_focus() && pointer_in_list(&ctx, id) {
        // ★★★ The same surrender `super::choose` documents at length: egui's
        // `SurrenderFocusOn::Presses` takes focus from a focused widget on any
        // press where it is not hovered, and a row of the popup is not this
        // text box. Without this the press frame drops the focus, the release
        // frame draws no popup, and the row the operator aimed at does not
        // exist to be clicked.
        response.request_focus();
    }

    // ★ Escape's two rungs, innermost first — `super::choose`'s rule, and it
    // has to be read BEFORE the commit branch because egui's own `TextEdit`
    // surrenders focus on Escape. Without that ordering the two are one event
    // and which runs is an accident of ordering rather than a decision.
    //
    // ★★★ **The outer rung writes the typed string on its way out** —
    // `OPERATOR_REQUESTS.md` **O223**. The inner one does not, and the
    // difference is not an inconsistency: closing the popup leaves the operator
    // in the box they were typing in with their text still in front of them,
    // so there is nothing to lose yet. The rung that takes the text off the
    // screen is the rung that has to write it.
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        note_escape(&ctx);
        if state.open {
            state.open = false;
            store(&ctx, id, state);
            store_focus(
                &ctx,
                Some(Focus {
                    seated: true,
                    waiting: 0,
                    ..focus
                }),
            );
        } else {
            commit_typed(&widget_box.field, options, selected, &draft, actions);
            trace_leave(&widget_box.field, "escape");
            store_focus(&ctx, None);
            forget(&ctx, id);
        }
        return true;
    }

    // Alt+Down is the conventional opener for an editable combo box on every
    // platform this shell runs on, and F4 is the Windows alias. A bare Down is
    // deliberately NOT one: the caret is in a text box, and the vertical
    // arrows belong to it.
    let opener =
        ctx.input(|i| (i.modifiers.alt && i.key_pressed(Key::ArrowDown)) || i.key_pressed(Key::F4));
    if button.clicked() || opener {
        state.open = !state.open;
        if state.open {
            state.hl = super::first_selected(options, selected).unwrap_or(0);
        }
    }

    if state.open {
        if let Some(backwards) = super::arrow(&ctx) {
            state.hl = super::step(state.hl, options.len(), backwards);
        } else if ctx.input(|i| i.key_pressed(Key::Enter)) {
            chosen = Some(state.hl);
        }
    }

    if state.open && !options.is_empty() {
        // `combo = true` always: an editable field is a combo box by
        // definition, bit 19 being meaningless without bit 18, so the popup
        // drops outside the widget rather than sitting in place over it.
        let moved = chosen.is_none() && super::arrow(&ctx).is_some();
        if let Some(index) = list(&ctx, id, rect, options, selected, true, state.hl, moved) {
            chosen = Some(index);
        }
    }

    if let Some((index, export, display)) =
        chosen.and_then(|i| options.get(i).map(|(e, d)| (i, e, d)))
    {
        // The draft follows the pick so the text box shows what was chosen on
        // the very frame it was chosen, rather than a frame later when the
        // epoch moves and `Focus::sync` re-seeds it. Display text, not the
        // export: the box is what the operator reads.
        draft = if display.is_empty() {
            export.clone()
        } else {
            display.clone()
        };
        textbox::seat(&ctx, id, &draft, true);
        state.open = false;
        crate::diag::trace(|| {
            // The row and the COUNT, never the value — `super`'s §8 rule.
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "form-choice-pick field={} row={index} selected=1 multi=false",
                widget_box.field
            )
        });
        send(
            &widget_box.field,
            wanted(options, selected, export, false),
            actions,
        );
    } else if ctx.input(|i| i.key_pressed(Key::Enter)) || response.lost_focus() {
        // ★★ Enter and focus loss both commit **the typed string**, and this
        // is the whole of what bit 19 buys: `set_choice_value` matches it
        // against `/Opt` first, so a typed "Large" is the same command as a
        // picked "Large", and a typed "Extra large" is a free-text value the
        // engine stores as its own export. Nothing here has to tell the two
        // apart.
        commit_typed(&widget_box.field, options, selected, &draft, actions);
        if response.lost_focus() {
            trace_leave(&widget_box.field, "lost-focus");
            store_focus(&ctx, None);
            forget(&ctx, id);
            // The press that took focus away may have landed on another field.
            // Not claimed, so `super::super::overlay` reads it.
            return false;
        }
        state.open = false;
    }

    crate::diag::trace_changed(super::CHOICE_STATE, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "form-choice-state field={} open={} hl={}",
            widget_box.field, state.open, state.hl
        )
    });
    store(&ctx, id, state);
    store_focus(
        &ctx,
        Some(Focus {
            draft,
            seated: true,
            waiting: 0,
            ..focus
        }),
    );
    button.clicked()
        || super::pressed_in_list(&ctx, id)
        || (response.contains_pointer() && ctx.input(|i| i.pointer.any_pressed()))
}

/// Raise a fill for the typed string, if it says anything the field does not
/// already hold.
///
/// ★★ The whole of what bit 19 buys: `set_choice_value` matches the string
/// against `/Opt` first, so a typed *"Large"* is the same command as a picked
/// *"Large"*, and a typed *"Extra large"* is a free-text value the engine
/// stores as its own export. Nothing here has to tell the two apart.
///
/// One function because it is reached from three exits — Enter, focus loss and
/// Escape — and *"tabbing through a field writes nothing"* has to mean the same
/// thing at all three. [`display_matches`] is the half that makes it true when
/// the field's `/V` is an export whose display text is what the box shows.
fn commit_typed(
    field: &str,
    options: &[(String, String)],
    selected: &[String],
    draft: &str,
    actions: &mut Vec<Action>,
) {
    let stored = selected.first().map_or("", String::as_str);
    let settled = draft.trim();
    if settled == stored || display_matches(options, selected, settled) {
        return;
    }
    crate::diag::trace(|| {
        // The field and the COUNT, never the value — `canvas::forms`' §8 rule.
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "form-choice-typed field={field} chars={}",
            settled.chars().count()
        )
    });
    send(field, vec![settled.to_owned()], actions);
}

/// Whether `text` is already the display or export of the selected option.
///
/// ★ Without this, opening an editable combo whose `/V` is an export that
/// differs from its display would commit the *display* on the way out — a
/// write the operator did not ask for, on every field they merely looked at.
fn display_matches(options: &[(String, String)], selected: &[String], text: &str) -> bool {
    selected.iter().any(|v| {
        options
            .iter()
            .any(|(e, d)| (v == e || v == d) && (text == e || text == d))
    })
}

/// Raise the one command this module has.
fn send(field: &str, values: Vec<String>, actions: &mut Vec<Action>) {
    actions.push(
        FieldAction::Edit(FormEdit::SetChoice {
            field: field.to_owned(),
            values,
        })
        .into(),
    );
}

/// Say which way the field was left, because the two are indistinguishable
/// from outside: both produce a box that stops answering keys and writes
/// nothing more.
fn trace_leave(field: &str, how: &str) {
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!("form-choice-leave field={field} how={how}")
    });
}

/// `egui` id salt for the drop button, so it does not collide with the text
/// box sharing the field's editor id.
const ARROW: &str = "pdfcer-canvas-form-combo-arrow"; // ui-text-exempt: internal widget id, never displayed

/// Paint the drop button: a chevron on the theme's own plate.
///
/// ★ `accent_pair`, never a named colour — `tools/gates/check-theme-colors.sh`
/// forbids the second and `check-plate-colour.sh` requires that an `on_accent`
/// ink state the plate it is drawn on. Both are satisfied by taking the pair
/// together, which is also the only way the contrast is gated.
fn chevron(ui: &Ui, rect: Rect, open: bool) {
    let (plate, ink) = egui_shell::theme::Theme::accent_pair(ui.ctx());
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::ZERO, plate);
    // A chevron rather than a filled triangle, which is what the photograph
    // shows and what every combo box drawn in the last decade uses.
    let c = rect.center();
    let w = (rect.width() * 0.28).min(rect.height() * 0.28);
    let (a, b) = if open { (w, -w) } else { (-w, w) };
    painter.add(egui::Shape::line(
        vec![
            egui::pos2(c.x - w * 1.4, c.y + a * 0.7),
            egui::pos2(c.x, c.y + b * 0.7),
            egui::pos2(c.x + w * 1.4, c.y + a * 0.7),
        ],
        egui::Stroke::new(1.5, ink),
    ));
}

/// The list state an arriving click should leave behind.
///
/// Called by [`super::focus_choice`] so the decision lives beside the
/// arithmetic that defines the button, rather than being re-derived at the
/// focus site.
pub(super) fn arrival(rect: Rect, point: egui::Pos2, hl: usize) -> ListState {
    ListState {
        open: hit_arrow(rect, point),
        hl,
    }
}

/// See `canvas/forms/choosing/typing/tests.rs` — the pure halves, under **R2**.
#[cfg(test)]
mod tests;
