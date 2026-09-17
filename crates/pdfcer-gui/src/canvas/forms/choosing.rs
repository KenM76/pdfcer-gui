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

/// The **editable** combo box, `/Ff` bit 19 — a drop-down whose value need
/// not be one of its options, and therefore a live text box rather than a
/// ring. It reaches this module's list, row and pick machinery through
/// `super::`, so its popup is the same popup; what it does not share is the
/// lifecycle. See its header.
mod typing;

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

/// The allowance the popup's own border needs when its scroll height is
/// derived from the room on the chosen side.
///
/// It is not a gap. A combo's list is drawn **flush** against the widget, with
/// no rounding, no shadow and no frame margin, because that is what Acrobat
/// draws and because a floating card hovering a few points below a field reads
/// as a tooltip rather than as the field's own options.
const LIST_PAD: f32 = 4.0;

/// The least height the list is given, in screen points — about one row.
///
/// The floor is deliberately this low rather than a comfortable minimum: a
/// larger one would exceed the room on the chosen side for a widget near the
/// edge of the screen, and `constrain_to` would then slide the popup back over
/// the widget, which is the failure the module header's ★ is about.
const LIST_MIN_H: f32 = 24.0;

/// Space above and below a row's text, in screen points.
///
/// Small on purpose. `egui`'s own `selectable_label` sets a button's padding,
/// which gives rows roughly half again the height Acrobat draws — so a list of
/// eight options needed a scroll bar where Acrobat needed none, and the two
/// surfaces could not be compared row for row.
const ROW_VPAD: f32 = 1.0;

/// Space between a row's left edge and its text, in screen points.
const ROW_HPAD: f32 = 3.0;

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
    point: egui::Pos2,
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
    let (hl, editable) = match &widget_box.kind {
        BoxKind::Choice {
            options,
            selected,
            editable,
            ..
        } => (first_selected(options, selected).unwrap_or(0), *editable),
        _ => (0, false),
    };
    // ★ An **editable** combo opens its list only when the click landed on the
    // drop button; a click in its text area asks for a caret instead. Every
    // other choice widget opens unconditionally, because a click on one has no
    // second meaning to tell apart. `point` and `widget_box.rect` are both in
    // page space, which is the space [`typing::arrow_strip`] has to be asked
    // in for the answer to be about where the operator actually pressed.
    let state = if editable {
        typing::arrival(widget_box.rect, point, hl)
    } else {
        ListState { open: true, hl }
    };
    store(ctx, focus.editor_id(), state);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "form-choice-open page={page} field={} widget={} row={hl} open={}",
            widget_box.field, widget_box.widget, state.open
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
        combo,
        editable,
        align,
    } = &widget_box.kind
    else {
        // Unreachable: `super::editor` routes by the same match. Answering
        // "not claimed" rather than panicking keeps a future fourth kind from
        // taking the window down.
        return false;
    };

    // ★★★ An editable combo is a live text box with a drop button, not a ring
    // over the appearance stream, and everything below this line assumes the
    // second: it locks the arrow keys to a highlight the text box needs for
    // its caret, draws no box over the widget, and treats Enter as a pick
    // rather than as a commit of typed text. [`typing`]'s header carries the
    // argument for why that is a separate surface rather than a flag.
    if *editable {
        return typing::type_into(
            ui, focus, widget_box, rect, options, selected, *align, actions,
        );
    }

    let id = focus.editor_id();
    // The arrow keys and Escape below are only reachable because this locks
    // them to the field; see [`super::keyboard_box`].
    let response = super::keyboard_box(ui, id, rect);
    if !focus.seated {
        response.request_focus();
    } else if !response.has_focus() {
        // ★★★ **A PRESS ON THE POPUP'S OWN ROW IS NOT A CLICK ELSEWHERE** —
        // O209, *"the drop-down options don't remember what I've clicked on."*
        //
        // `egui`'s default `SurrenderFocusOn::Presses` takes focus away from a
        // focused widget on **any** press where that widget is not hovered, and
        // the option list is a foreground `Area` sitting over the page — so the
        // press that lands on a row is, to `keyboard_box`, a press somewhere
        // else. `Response::has_focus` reads memory live, so the surrender is
        // already visible on the very frame of the press.
        //
        // ★★ What that cost: this branch then forgot the focus and the list
        // state, so on the RELEASE frame — which is the frame a row's
        // `clicked()` would fire on — there was no focused field, no popup was
        // drawn, and the row the operator was pointing at did not exist. The
        // pick could never be made by mouse at all. The trace shows it exactly:
        // `form-choice-open`, then `form-choice-unfocused`, and no
        // `form-choice-pick` line.
        //
        // ★ The question asked is deliberately *where the pointer is*, not
        // *what was pressed*: the `Area`'s rectangle is read from memory, which
        // holds last frame's geometry, so it is available on the press frame
        // before this frame's popup has been laid out. Re-requesting focus
        // restores it for the rest of this frame and for the next one, where
        // there is no press to surrender it again.
        if pointer_in_list(&ctx, id) {
            response.request_focus();
        } else {
            // A panel field, a dialog or a click elsewhere took the keyboard. A
            // choice field keeps no draft, so there is nothing to commit.
            //
            // Traced because this exit is otherwise indistinguishable from the
            // keyboard never arriving at all: both produce a field that stops
            // answering keys and writes nothing. A driven check that presses
            // Enter on an open list and gets no pick needs to know which of the
            // two it measured.
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!("form-choice-unfocused field={}", widget_box.field)
            });
            store_focus(&ctx, None);
            forget(&ctx, id);
            return false;
        }
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
        if let Some(index) = list(&ctx, id, rect, options, selected, *combo, state.hl, moved) {
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
        // ★ A single-select **combo** pick is finished, and closes. Everything
        // else stays open, for two different reasons that happen to agree:
        //
        // * a multi-select pick is the first of several, and closing after
        //   each tick would make choosing three options three gestures;
        // * a **list box** is not a thing that opens and shuts at all. Its
        //   rows are what a focused list box looks like, so collapsing them
        //   back to the appearance stream on the first pick would answer the
        //   operator's click by taking the control away.
        state.open = *multi || !*combo;
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
/// # ★★★ Two presentations, and they are not styling variants
///
/// `/Ff` `Combo` decides **where the options are drawn**, and the two answers
/// are structurally different surfaces:
///
/// * a **combo box** drops its list *outside* the widget, below it when there
///   is room and above it otherwise — the module header's ★ governs the side,
///   the constraint and the scroll height;
/// * a **list box** draws its options **inside its own rectangle**, opaquely
///   covering the appearance stream, with a scroll bar the moment they do not
///   fit. It is anchored to the widget and deliberately overlaps it, so none
///   of the half-plane arithmetic applies.
///
/// ### The opacity is load-bearing, and Acrobat is the counter-example
///
/// An in-place list that lets the page show through is unreadable over the one
/// document type this program exists for. O209, on a SolidWorks-exported
/// drawing: *"I tested some on the SW drawing so I guess the background on the
/// drawing interferes with the list box in Acrobat."* Acrobat's own list is
/// legible over a blank form and not over dense vector line work, so the fill
/// here is the theme's opaque text-entry background rather than anything
/// derived from the widget or blended with what is underneath.
///
/// That is measured against Acrobat rather than assumed, and it is what O209
/// reports: *"the list option is somehow hidden from view in Acrobat until I
/// click on it, then I can select options and if the box is too small for all
/// of the options it gives a scroll bar."*
///
/// # ★★ Why the frame is built here instead of using [`egui::Frame::popup`]
///
/// `Frame::popup` is a rounded, shadowed, generously padded card — correct for
/// a menu floating over an application's own chrome, wrong for a control
/// belonging to a rectangle on a page. Acrobat's list is a square 1 px box
/// flush against the field, no shadow and no margin, and the difference is
/// most of what *"look different than they do in Acrobat when they are clicked
/// on"* names. The border takes `canvas_selection_ink` so it matches the focus
/// ring [`choose`] has already drawn around the same widget.
#[expect(
    clippy::too_many_arguments,
    reason = "every argument is a distinct fact about one popup: where it \
              anchors, what it lists, what is selected, whether several may \
              be, whether it drops or sits in place, which row the keyboard \
              is on, and whether that row just moved. Grouping them into a \
              struct would name the same eight things one indirection away."
)]
fn list(
    ctx: &egui::Context,
    id: Id,
    rect: Rect,
    options: &[(String, String)],
    selected: &[String],
    combo: bool,
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
    let (pivot, anchor, limit, height) = if !combo {
        // In place. The anchor is the widget's own top-left and the list is
        // the widget's own height, so the options land exactly where the value
        // was — which is the whole behaviour a list box has.
        (
            Align2::LEFT_TOP,
            rect.left_top(),
            screen,
            rect.height().max(LIST_MIN_H),
        )
    } else if down {
        (
            Align2::LEFT_TOP,
            rect.left_bottom(),
            Rect::from_min_max(egui::pos2(screen.left(), rect.bottom()), screen.max),
            (below - LIST_PAD).clamp(LIST_MIN_H, LIST_MAX_H),
        )
    } else {
        (
            Align2::LEFT_BOTTOM,
            rect.left_top(),
            Rect::from_min_max(screen.min, egui::pos2(screen.right(), rect.top())),
            (above - LIST_PAD).clamp(LIST_MIN_H, LIST_MAX_H),
        )
    };
    // `set_width` below, and the reason is the doc on [`LIST_MIN_W`]: a
    // *maximum* lets the frame shrink to its longest label, which measured
    // 57 px under a 370 px field — the defect that reads as "it looks nothing
    // like Acrobat's".
    let width = rect.width().max(LIST_MIN_W);

    let mut clicked = None;
    egui::Area::new(id.with(LIST_AREA))
        .order(egui::Order::Foreground)
        .pivot(pivot)
        .fixed_pos(anchor)
        .constrain_to(limit)
        .show(ctx, |ui| {
            let frame = egui::Frame::NONE
                .fill(ui.visuals().extreme_bg_color)
                .stroke(egui::Stroke::new(
                    1.0,
                    egui_shell::theme::Theme::canvas_selection_ink(ctx),
                ))
                .corner_radius(egui::CornerRadius::ZERO);
            frame.show(ui, |ui| {
                ui.set_width(width);
                // Rows butt against one another, as a list's rows do. The
                // default spacing draws a stripe of window background between
                // every pair, which is what made the surface read as a stack of
                // buttons.
                ui.spacing_mut().item_spacing.y = 0.0;
                egui::ScrollArea::vertical()
                    // x: always fill the width that was just set.
                    //
                    // y: shrink to the rows for a **combo**, so a two-option
                    // drop is two rows tall and only an overflowing one grows
                    // a scroll bar — but never for a **list box**, whose whole
                    // height is the widget's own. Left shrinking, the list
                    // box's last rows would let the appearance stream show
                    // through below the options, which is the very bleed-
                    // through O209 reports Acrobat having.
                    .auto_shrink([false, combo])
                    .max_height(height)
                    .min_scrolled_height(height)
                    .show(ui, |ui| {
                        let full = ui.available_width();
                        for (index, (export, display)) in options.iter().enumerate() {
                            let on = selected.iter().any(|v| v == export || v == display);
                            // An `/Opt` entry may carry an explicitly empty
                            // display string, and a row with no label is a row
                            // nobody can aim at. The export is what the file
                            // has left to identify it by.
                            let label = if display.is_empty() { export } else { display };
                            let response = row(ui, full, label, on, index == hl);
                            if index == hl && moved {
                                response.scroll_to_me(None);
                            }
                            if response.clicked() {
                                clicked = Some(index);
                            }
                        }
                    });
            });
        });
    clicked
}

/// Draw one option row and answer whether it was clicked.
///
/// # ★ Why the row is painted rather than assembled from `selectable_label`
///
/// Three things have to be true at once and no stock widget delivers them
/// together: the row is the **full width** of the list (a click anywhere along
/// it picks, as in every list the operator has used), it is **compact** enough
/// to compare against Acrobat row for row, and a selected row is a **solid
/// plate with legible ink** rather than a tinted button.
///
/// The plate is `Theme::accent_pair` — the sanctioned spelling of *"paint this
/// as the emphasised thing"*, contrast-gated at the theme, and the only legal
/// route to a solid emphasis colour outside the theme module.
/// `tools/gates/check-selection-channel.sh` forbids reading `visuals.selection`
/// here and `check-theme-colors.sh` forbids naming a colour outright; both are
/// satisfied, and the result is Acrobat's solid-row treatment expressed in this
/// shell's own palette instead of copied out of a screenshot.
///
/// A **multi-select** row gets the same plate and no check box. That is
/// Acrobat's answer too, measured: a multi-select list marks its chosen rows by
/// filling them, and adding a check box would invent an affordance the product
/// class does not have — which is why this takes no `multi` argument.
fn row(ui: &mut Ui, width: f32, label: &str, on: bool, hl: bool) -> egui::Response {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let height = ui.text_style_height(&egui::TextStyle::Body) + 2.0 * ROW_VPAD;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());
    let (plate, on_plate) = egui_shell::theme::Theme::accent_pair(ui.ctx());
    let hovered = ui.visuals().widgets.hovered.bg_fill;
    let plain = ui.visuals().text_color();
    let painter = ui.painter();
    if on {
        painter.rect_filled(rect, egui::CornerRadius::ZERO, plate);
    } else if response.hovered() {
        painter.rect_filled(rect, egui::CornerRadius::ZERO, hovered);
    }
    if hl {
        // Where Enter would land, drawn as an outline so it never looks like a
        // second selected state. Inside the row, and in whichever of the pair
        // the row is not already filled with, so it survives on both.
        painter.rect_stroke(
            rect.shrink(0.5),
            egui::CornerRadius::ZERO,
            egui::Stroke::new(1.0, if on { on_plate } else { plate }),
            egui::StrokeKind::Inside,
        );
    }
    painter.text(
        rect.left_center() + egui::vec2(ROW_HPAD, 0.0),
        Align2::LEFT_CENTER,
        label,
        font,
        if on { on_plate } else { plain },
    );
    response
}

/// Whether the pointer is inside the popup's own rectangle.
///
/// Read from the `Area`'s rectangle rather than from a row's response, because
/// a point on the frame's padding is still inside the popup — and because the
/// `Area`'s rect is in `egui` **memory**, carried over from the pass that drew
/// it, so this answers before this frame's popup has been laid out. That is
/// what makes it usable by [`choose`]'s focus guard, which runs above the call
/// to [`list`].
fn pointer_in_list(ctx: &egui::Context, id: Id) -> bool {
    let Some(pos) = ctx.pointer_interact_pos() else {
        return false;
    };
    ctx.memory(|m| m.area_rect(id.with(LIST_AREA)))
        .is_some_and(|r| r.contains(pos))
}

/// Whether this frame's press landed inside the popup.
///
/// [`pointer_in_list`] plus *"and a button went down this frame"* — the extra
/// half that makes it a claim on **this** press, which is what
/// [`super::overlay`] needs to know before reading the same press as a request
/// to focus some other field.
fn pressed_in_list(ctx: &egui::Context, id: Id) -> bool {
    pointer_in_list(ctx, id) && ctx.input(|i| i.pointer.any_pressed())
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
