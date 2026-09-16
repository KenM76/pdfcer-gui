//! # `canvas::forms::choosing` — picking an option **where the field is drawn**
//!
//! The choice half of [`super`]: a `/Ch` combo box or list box, filled by
//! clicking it on the page rather than by finding its row in the side panel.
//! `FORMS_PARITY.md` §8.1 row 2.
//!
//! ## Contract
//!
//! Two entry points, both called from [`super`]:
//!
//! * [`focus_choice`] takes a click on a choice widget — it stores the
//!   [`super::Focus`] and opens the list, and is [`super::focus_button`]'s
//!   twin.
//! * [`choose`] is the third arm of [`super::editor`]: a choice field holds no
//!   caret and toggles no state, so it holds a **focus ring plus an anchored
//!   option list**, and reads Space, Enter, the arrow keys and Escape.
//!
//! Every outcome leaves as [`FormEdit::SetChoice`] on `actions` — the panel's
//! own command, carrying **export** values, reaching `set_choice_value`. There
//! is no fill path here.
//!
//! ## Why the list is a separate open/closed state from the focus
//!
//! Choosing a value must **not** end the ring. O204's complaint is that Tab
//! inside a form escapes to the ribbon, and dropping focus on a pick would
//! reproduce it from the commonest gesture on the surface: pick a country,
//! press Tab, land in the ribbon. So a pick closes the *list* and keeps the
//! *focus*, which is also what every program that fills forms does.
//!
//! That needs a second bit of state, and it lives in `egui` memory keyed on
//! the editor id rather than on [`super::Focus`], because nothing outside this
//! module has any use for it — [`super::commit`], [`super::tabbing`] and the
//! panel's `live_draft` mirror all ask about a *draft*, and a choice field
//! keeps none.
//!
//! A Tab arrival deliberately leaves the list **closed**: tabbing through a
//! form would otherwise spray open dropdowns over the sheet, and no program
//! behaves that way. Space, Enter or either vertical arrow opens it.
//!
//! ## ★ The popup is constrained to the half-plane it chose, not to the screen
//!
//! A page-anchored popup constrained to the viewport slides **back over its own
//! anchor** when it does not fit — and then it takes that anchor's clicks,
//! because it is the topmost layer. `D:/dev/rag/egui/` carries the general
//! finding; here it would mean an option list covering the very box the
//! operator is trying to fill.
//!
//! So the side is chosen first (below when there is room, above otherwise) and
//! the constraint rectangle is the half-plane on that side of the widget. An
//! overlap is then arithmetically impossible rather than merely unlikely, and
//! the scroll height is taken from the room actually available on the chosen
//! side.
//!
//! ## What it does not draw
//!
//! No facsimile of the widget, no in-place list, and nothing at all over an
//! **unfocused** choice field — rule 4's one-line test, exactly as [`super`]'s
//! §3 applies it to the text editor. A closed list leaves the page's own
//! appearance stream showing, which is the value the document holds.

use egui::{Align2, Id, Key, Rect, Ui};

use super::{Focus, note_escape, store_focus, stored_value};
use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::app::state::OpenDoc;
use crate::canvas::forms::boxes::{BoxKind, WidgetBox};
use crate::canvas::tabnav;
use crate::panels::forms::edit::FormEdit;

/// Memory key for the list's open/closed state and keyboard highlight.
const LIST_KEY: &str = "pdfcer-canvas-form-choice-list"; // ui-text-exempt: internal memory id, never displayed

/// `egui` id suffix for the popup's own `Area`.
const LIST_AREA: &str = "pdfcer-canvas-form-choice-area"; // ui-text-exempt: internal widget id, never displayed

/// The tallest the option list is drawn, in screen points.
///
/// `/Opt` is unbounded — a country list is two hundred entries — so the list
/// scrolls rather than growing to fit, and this is the height it scrolls
/// within. Above it the popup stops looking like a dropdown and starts looking
/// like a page of its own.
const LIST_MAX_H: f32 = 240.0;

/// The narrowest the option list is drawn, in screen points.
///
/// A choice widget may be 30 pt wide on the sheet and its options may be
/// words. The list is at least this wide whatever the box measures, which is
/// the same trade `boxes::MIN_EDITOR` makes for the text editor: a
/// control too small to read is a worse lie than one a little wider than the
/// field it belongs to.
const LIST_MIN_W: f32 = 140.0;

/// Breathing room between the widget and the popup, and the allowance for
/// [`egui::Frame::popup`]'s own margins when the scroll height is derived.
const LIST_PAD: f32 = 8.0;

/// The least height the list is given, in screen points — about one row.
///
/// The floor is deliberately this low rather than a comfortable minimum: a
/// larger one would exceed the room on the chosen side for a widget near the
/// edge of the screen, and `constrain_to` would then slide the popup back over
/// the widget, which is the failure the module header's ★ is about.
const LIST_MIN_H: f32 = 24.0;

/// Whether the list is open, and which row the keyboard is on.
///
/// `Copy`, and stored in `egui`'s frame-to-frame temp store rather than on
/// [`Focus`] — see the module header.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ListState {
    /// Whether the option list is drawn this frame.
    open: bool,
    /// The row Enter and Space would pick. Meaningless while closed.
    hl: usize,
}

/// Read the list state for the field whose editor id is `id`.
fn load(ctx: &egui::Context, id: Id) -> ListState {
    ctx.data(|d| d.get_temp::<ListState>(id.with(LIST_KEY)))
        .unwrap_or_default()
}

/// Store the list state.
fn store(ctx: &egui::Context, id: Id, state: ListState) {
    ctx.data_mut(|d| d.insert_temp(id.with(LIST_KEY), state));
}

/// Forget the list state, because the field has lost the keyboard.
///
/// Called on every path that clears the focus, so that clicking the same
/// field again a minute later opens a list positioned and highlighted from
/// scratch rather than from wherever the last visit left it.
fn forget(ctx: &egui::Context, id: Id) {
    ctx.data_mut(|d| d.remove::<ListState>(id.with(LIST_KEY)));
}

/// [`crate::diag::trace_changed`]'s slot for the open list's state.
const CHOICE_STATE: &str = "form-choice-state";

/// **Take a click on a choice widget**: focus the field and open its list.
///
/// [`super::focus_button`]'s twin, and `draft` is seeded the same way and for
/// the same reason — equal to what the document holds, so that
/// [`super::commit`] on the way out finds nothing changed and writes nothing.
/// A choice field's value is never a draft: every pick is a complete command
/// the instant it is made.
pub(super) fn focus_choice(
    ctx: &egui::Context,
    doc: &OpenDoc,
    page: usize,
    widget_box: &WidgetBox,
) {
    let focus = Focus {
        path: doc.path.clone(),
        epoch: doc.edit_epoch,
        page,
        field: widget_box.field.clone(),
        widget: widget_box.widget,
        draft: stored_value(doc, &widget_box.field).unwrap_or_default(),
        seated: false,
        waiting: 0,
    };
    let hl = match &widget_box.kind {
        BoxKind::Choice {
            options, selected, ..
        } => first_selected(options, selected).unwrap_or(0),
        _ => 0,
    };
    store(ctx, focus.editor_id(), ListState { open: true, hl });
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "form-choice-open page={page} field={} widget={} row={hl}",
            widget_box.field, widget_box.widget
        )
    });
    store_focus(ctx, Some(focus));
}

/// Draw the focused choice field's ring and, while it is open, its option
/// list.
///
/// Returns whether this frame's primary press belonged to the popup, so
/// [`super::overlay`] does not also read it as a request to focus something
/// else.
///
/// The `rect` is [`super::boxes::editor_rect`]'s — the widget's own rectangle,
/// grown to a legible minimum — so the ring and the popup's anchor are the
/// same rectangle the text editor would have used on the same widget.
pub(super) fn choose(
    ui: &mut Ui,
    focus: Focus,
    widget_box: &WidgetBox,
    rect: Rect,
    actions: &mut Vec<Action>,
) -> bool {
    let ctx = ui.ctx().clone();
    let BoxKind::Choice {
        options,
        selected,
        multi,
    } = &widget_box.kind
    else {
        // Unreachable: `super::editor` routes by the same match. Answering
        // "not claimed" rather than panicking keeps a future fourth kind from
        // taking the window down.
        return false;
    };

    let id = focus.editor_id();
    // The arrow keys and Escape below are only reachable because this locks
    // them to the field; see [`super::keyboard_box`].
    let response = super::keyboard_box(ui, id, rect);
    if !focus.seated {
        response.request_focus();
    } else if !response.has_focus() {
        // A panel field, a dialog or a click elsewhere took the keyboard. A
        // choice field keeps no draft, so there is nothing to commit.
        //
        // Traced because this exit is otherwise indistinguishable from the
        // keyboard never arriving at all: both produce a field that stops
        // answering keys and writes nothing. A driven check that presses Enter
        // on an open list and gets no pick needs to know which of the two it
        // measured.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("form-choice-unfocused field={}", widget_box.field)
        });
        store_focus(&ctx, None);
        forget(&ctx, id);
        return false;
    }
    tabnav::publish(&ctx, tabnav::Scope::Field, id);
    // `canvas_selection_ink`, never `visuals.selection.stroke` — the same
    // channel rule `tools/gates/check-selection-channel.sh` holds the button
    // ring to.
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(1.5, egui_shell::theme::Theme::canvas_selection_ink(&ctx)),
        egui::StrokeKind::Outside,
    );

    let mut state = load(&ctx, id);
    state.hl = state.hl.min(options.len().saturating_sub(1));
    let mut chosen: Option<usize> = None;
    let mut claimed = false;
    let mut leaving = false;

    // ★ Escape's two rungs, innermost first. An open list is the most
    // transient thing on the surface, so Escape closes it and leaves the ring;
    // a second press gives up the field. Collapsing the two would make one key
    // do both, which is decision 025's L1 in miniature.
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        if state.open {
            state.open = false;
        } else {
            leaving = true;
        }
        note_escape(&ctx);
        claimed = true;
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "form-choice-escape field={} closed={} left={leaving}",
                widget_box.field, !state.open
            )
        });
    } else if state.open {
        if let Some(backwards) = arrow(&ctx) {
            state.hl = step(state.hl, options.len(), backwards);
        } else if ctx.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)) {
            chosen = Some(state.hl);
        }
    } else if ctx.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space))
        || arrow(&ctx).is_some()
    {
        // ★ Opening rather than selecting, and that is the conservative half
        // of the keyboard story. Windows changes a closed combo's value on an
        // arrow press; doing that here would write a document edit — and an
        // undo entry — for a key the operator pressed to *look* at the
        // options. So every opening key opens, and only Enter, Space or a
        // click on a row writes.
        state.open = true;
        state.hl = first_selected(options, selected).unwrap_or(0);
    }

    if state.open && !options.is_empty() {
        let moved = chosen.is_none() && arrow(&ctx).is_some();
        if let Some(index) = list(&ctx, id, rect, options, selected, *multi, state.hl, moved) {
            chosen = Some(index);
        }
        claimed = claimed || pressed_in_list(&ctx, id);
    }

    // Resolved to the option's own export here, so [`wanted`] has no
    // out-of-range case to answer for and no branch nothing can reach.
    if let Some((index, export)) = chosen.and_then(|i| options.get(i).map(|(e, _)| (i, e))) {
        let values = wanted(options, selected, export, *multi);
        crate::diag::trace(|| {
            // The row and the COUNT, never the value — the module header §8
            // rule, applied to a selection: an option's display text is the
            // document's, but which of them an operator picked is their answer
            // to the form.
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "form-choice-pick field={} row={index} selected={} multi={multi}",
                widget_box.field,
                values.len()
            )
        });
        actions.push(
            FieldAction::Edit(FormEdit::SetChoice {
                field: widget_box.field.clone(),
                values,
            })
            .into(),
        );
        // A single-select pick is finished; a multi-select one is the first of
        // several, and closing the list after each tick would make choosing
        // three options three separate gestures.
        state.open = *multi;
    }

    // The list's whole state, de-duplicated, so a keyboard gesture that moves
    // the highlight is observable from outside without waiting for a pick.
    // `trace_changed` and not `trace`: this would otherwise be one line per
    // frame for as long as the list is open.
    crate::diag::trace_changed(CHOICE_STATE, || {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "form-choice-state field={} open={} hl={}",
            widget_box.field, state.open, state.hl
        )
    });

    if leaving {
        store_focus(&ctx, None);
        forget(&ctx, id);
    } else {
        store(&ctx, id, state);
        store_focus(
            &ctx,
            Some(Focus {
                seated: true,
                waiting: 0,
                ..focus
            }),
        );
    }
    claimed
}

/// Draw the option list and answer which row was clicked.
///
/// The side, the constraint and the scroll height are the module header's ★.
#[expect(
    clippy::too_many_arguments,
    reason = "every argument is a distinct fact about one popup: where it \
              anchors, what it lists, what is selected, whether several may \
              be, which row the keyboard is on, and whether that row just \
              moved. Grouping them into a struct would name the same seven \
              things one indirection away."
)]
fn list(
    ctx: &egui::Context,
    id: Id,
    rect: Rect,
    options: &[(String, String)],
    selected: &[String],
    multi: bool,
    hl: usize,
    moved: bool,
) -> Option<usize> {
    // `content_rect`, not `viewport_rect`: the second includes the strip an OS
    // notch or status bar may cover, and a list that ends under one is a list
    // whose last row cannot be read.
    let screen = ctx.content_rect();
    let below = screen.bottom() - rect.bottom();
    let above = rect.top() - screen.top();
    // Below when there is room for a useful list, otherwise whichever side has
    // more. A dropdown that drops down is what the operator expects; a
    // dropdown that covers the field is a defect.
    let down = below >= LIST_MAX_H || below >= above;
    let (pivot, anchor, limit, room) = if down {
        (
            Align2::LEFT_TOP,
            rect.left_bottom(),
            Rect::from_min_max(egui::pos2(screen.left(), rect.bottom()), screen.max),
            below,
        )
    } else {
        (
            Align2::LEFT_BOTTOM,
            rect.left_top(),
            Rect::from_min_max(screen.min, egui::pos2(screen.right(), rect.top())),
            above,
        )
    };
    // Two pads: one for the gap and one for the frame's own margins, so the
    // drawn popup — not merely its scroll area — fits inside `limit`.
    let height = (room - 2.0 * LIST_PAD).clamp(LIST_MIN_H, LIST_MAX_H);
    let width = rect.width().max(LIST_MIN_W);

    let mut clicked = None;
    egui::Area::new(id.with(LIST_AREA))
        .order(egui::Order::Foreground)
        .pivot(pivot)
        .fixed_pos(anchor)
        .constrain_to(limit)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(width);
                egui::ScrollArea::vertical()
                    .max_height(height)
                    .show(ui, |ui| {
                        for (index, (export, display)) in options.iter().enumerate() {
                            let on = selected.iter().any(|v| v == export || v == display);
                            // An `/Opt` entry may carry an explicitly empty
                            // display string, and a row with no label is a row
                            // nobody can aim at. The export is what the file
                            // has left to identify it by.
                            let label = if display.is_empty() { export } else { display };
                            let response = if multi {
                                let mut checked = on;
                                ui.checkbox(&mut checked, label)
                            } else {
                                ui.selectable_label(on, label)
                            };
                            let hit = response.clicked();
                            if index == hl {
                                // The keyboard's row, shown as a hover so that
                                // "where Enter will land" is visible without
                                // inventing a second selected-looking state.
                                //
                                // The repaint is not optional. `highlight()`
                                // is read from the PREVIOUS pass's state, so
                                // asking for it after the row has drawn lands
                                // one pass later, and egui asks for no pass of
                                // its own. Without this the arrow key would
                                // move a highlight nobody sees until some
                                // unrelated event happened to redraw.
                                if !response.highlighted() {
                                    ctx.request_repaint();
                                }
                                let response = response.highlight();
                                if moved {
                                    response.scroll_to_me(None);
                                }
                            }
                            if hit {
                                clicked = Some(index);
                            }
                        }
                    });
            });
        });
    clicked
}

/// Whether this frame's press landed inside the popup.
///
/// Read from the `Area`'s own rectangle rather than from a row's response,
/// because a press on the frame's padding is still a press the page must not
/// also read as a click on some other field.
fn pressed_in_list(ctx: &egui::Context, id: Id) -> bool {
    let Some(pos) = ctx.pointer_interact_pos() else {
        return false;
    };
    ctx.memory(|m| m.area_rect(id.with(LIST_AREA)))
        .is_some_and(|r| r.contains(pos))
        && ctx.input(|i| i.pointer.any_pressed())
}

/// The selections a pick produces, as **export** values.
///
/// Single-select is the one option picked. Multi-select toggles it against
/// what is already selected.
///
/// # ★ Rebuilt from `/Opt`, which is not what the panel does
///
/// The current selection is recovered by asking each *option* whether it is
/// selected, rather than by copying `/V` and editing it. The difference shows
/// on a field whose `/V` holds a value matching no option — a real state, set
/// by another program or left behind when the option list changed. Carrying it
/// forward hands `set_choice_value` a value it must refuse
/// (`ChoiceValueNotInOptions`), so the operator's tick would fail with a
/// refusal naming a value they never touched. Rebuilding drops it instead,
/// which is the same thing picking a new value on any other surface does.
fn wanted(
    options: &[(String, String)],
    selected: &[String],
    export: &str,
    multi: bool,
) -> Vec<String> {
    if !multi {
        return vec![export.to_owned()];
    }
    let mut out: Vec<String> = options
        .iter()
        .filter(|(e, d)| selected.iter().any(|v| v == e || v == d))
        .map(|(e, _)| e.clone())
        .collect();
    if out.iter().any(|v| v == export) {
        out.retain(|v| v != export);
    } else {
        out.push(export.to_owned());
    }
    out
}

/// The first selected option's index, or `None` when nothing is selected.
///
/// Matched on export **or** display, because `/V` legally holds either and a
/// strict match on one opens the list on row 0 for a field that is answered.
fn first_selected(options: &[(String, String)], selected: &[String]) -> Option<usize> {
    options
        .iter()
        .position(|(e, d)| selected.iter().any(|v| v == e || v == d))
}

/// Move the highlight one row, wrapping.
fn step(hl: usize, len: usize, backwards: bool) -> usize {
    if len == 0 {
        return 0;
    }
    if backwards {
        (hl + len - 1) % len
    } else {
        (hl + 1) % len
    }
}

/// Which way a vertical arrow points, or `None` when neither was pressed.
///
/// Vertical only, which is where this differs from
/// [`super::tabbing`]'s `arrow`: a radio group may be laid out in a row, so
/// the horizontal arrows mean something there. A list is a list, and Left and
/// Right are left to whatever the canvas means by them.
fn arrow(ctx: &egui::Context) -> Option<bool> {
    ctx.input(|i| {
        if i.key_pressed(Key::ArrowDown) {
            Some(false)
        } else if i.key_pressed(Key::ArrowUp) {
            Some(true)
        } else {
            None
        }
    })
}

/// See `canvas/forms/choosing/tests.rs` — the pure halves, under **R2**.
#[cfg(test)]
mod tests;
