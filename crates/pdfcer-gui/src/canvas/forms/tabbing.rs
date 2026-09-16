//! # `canvas::forms::tabbing` — Tab walks the form, and a focused button waits
//!
//! `OPERATOR_REQUESTS.md` O204, the field half:
//!
//! > *"when I press tab while in a form I end up tabbing through the menus
//! > instead of the form items. The tab should tab through whatever space I
//! > have clicked on … if I've clicked on a form item it should tab forward and
//! > shift-tab backwards to the next one."*
//!
//! ## Contract
//!
//! Two entry points, both called from [`super::overlay`]:
//!
//! * [`advance`] spends a press [`crate::canvas::tabnav`] took off egui, moves
//!   [`super::Focus`] to the next stop of [`super::ring`]'s table, commits what
//!   was being typed, and asks for the least scroll that brings the new stop
//!   into view.
//! * [`button_focus`] is the other half of [`super::editor`]: a check box or a
//!   radio button cannot hold a caret, so it holds a **focus ring** instead and
//!   reads Space, Enter and the arrow keys.
//!
//! ## Why the ring must advance before the editor draws
//!
//! [`advance`] runs first in the frame, so the editor [`super::editor`] draws
//! is the one Tab has just arrived at. Running it after would draw the *old*
//! field, request focus for it, and move the caret a frame late — which on a
//! held Tab is a ring that lags one stop behind the key and never catches up.
//!
//! ## Why a cross-page ring
//!
//! O204 decision 3. A form is a document, not a page: tabbing off the last
//! field of sheet one lands on the first field of sheet two, because that is
//! what every program that fills forms does and because the alternative — a
//! ring that traps the operator on a page — makes Tab useless on the multi-page
//! forms it is most needed for. The object ring wraps within its page instead,
//! for the reason its own caller carries.

use egui::Ui;

use super::ring;
use super::{Focus, load_focus, note_escape, raise_button, store_focus, stored_value};
use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::forms::boxes::{BoxKind, WidgetBox};
use crate::canvas::tabnav;

/// O204 decision 3: the field ring crosses pages.
const CROSS_PAGES: bool = true;

/// The `why` a Tab-driven reveal carries into the trace.
const WHY_TAB: &str = "tab-field"; // ui-text-exempt: diagnostic token, never displayed

/// **Spend this frame's Tab press**, if one was claimed for the field ring.
///
/// Called before anything else this module draws — see the header. Does
/// nothing at all on the overwhelming majority of frames: `tabnav::take`
/// answers `None` unless the hook claimed a press, which it does only while the
/// canvas holds egui's keyboard focus.
pub(super) fn advance(
    ctx: &egui::Context,
    doc: &OpenDoc,
    list: &[WidgetBox],
    actions: &mut Vec<Action>,
) {
    let Some(request) = tabnav::take(ctx, tabnav::Scope::Field) else {
        return;
    };
    let table = ring::rings(ctx, doc, list);
    let Some(focus) = load_focus(ctx) else {
        // The hook claims only for a published id that still holds focus, and
        // only a drawn focus publishes — so reaching here means the focus was
        // dropped between the hook and this call. Traced rather than ignored:
        // the symptom is a Tab that does nothing, which nobody can debug from
        // outside the process.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "tab-field-unfocused".to_owned()
        });
        return;
    };

    let Some(at) = locate(&table, list, &focus) else {
        // The focused box is on no ring: an undo removed it, or the engine
        // says a reader does not visit it. Nothing to step from.
        tabnav::discard(ctx);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!(
                "tab-field-unringed page={} field={}",
                focus.page, focus.field
            )
        });
        return;
    };

    let Some((page, pos)) = tabnav::step(&table.lens(), at, request.backwards, CROSS_PAGES) else {
        return;
    };
    let Some(target) = table.at(page, pos).and_then(|index| list.get(index)) else {
        return;
    };
    if target.page == focus.page && target.field == focus.field && target.widget == focus.widget {
        // A ring of one. The press is spent and the focus does not move, which
        // is the right answer and is not the same as the press being ignored.
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("tab-field-alone page={page} field={}", focus.field)
        });
        return;
    }
    move_focus(ctx, doc, Some(&focus), target, actions);
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "tab-field backwards={} from_page={} from={} to_page={} to={} widget={}",
            request.backwards, focus.page, focus.field, target.page, target.field, target.widget
        )
    });
}

/// Where the focused box sits on its page's ring.
///
/// Falls back to *the ring stop belonging to the same field* when the box
/// itself is not a stop, which is the radio group: [`ring::assemble`] collapses
/// a group to its first widget, and an arrow key can leave the focus on one of
/// the others. Without the fallback the next Tab would find no position and
/// stop dead in the middle of a form.
fn locate(table: &ring::TabRings, list: &[WidgetBox], focus: &Focus) -> Option<(usize, usize)> {
    let here = list
        .iter()
        .position(|b| b.page == focus.page && b.field == focus.field && b.widget == focus.widget);
    if let Some(at) = here.and_then(|index| table.locate(focus.page, index)) {
        return Some(at);
    }
    let (_, stops) = table.rings.iter().find(|(p, _)| *p == focus.page)?;
    let pos = stops
        .iter()
        .position(|index| list.get(*index).is_some_and(|b| b.field == focus.field))?;
    Some((focus.page, pos))
}

/// **Move the focus to `target`**, committing whatever `leaving` held and
/// asking for the smallest scroll that brings the new box into view.
///
/// `leaving` is `None` for a move that had no previous focus. The commit is
/// guarded exactly as [`super::settle`] guards its own: a draft describing a
/// document or a revision that is no longer on screen is dropped rather than
/// written, because writing it would write a value against a document the
/// operator has not seen since they typed it.
fn move_focus(
    ctx: &egui::Context,
    doc: &OpenDoc,
    leaving: Option<&Focus>,
    target: &WidgetBox,
    actions: &mut Vec<Action>,
) {
    if let Some(leaving) = leaving
        && leaving.path == doc.path
        && leaving.epoch == doc.edit_epoch
    {
        super::commit(leaving, doc, actions);
    }
    store_focus(
        ctx,
        Some(Focus {
            path: doc.path.clone(),
            epoch: doc.edit_epoch,
            page: target.page,
            field: target.field.clone(),
            widget: target.widget,
            // Seeded from the document, which is also what makes a button's
            // focus harmless: its draft is never edited, so it always equals
            // the stored value and `commit` writes nothing.
            draft: stored_value(doc, &target.field).unwrap_or_default(),
            seated: false,
            // The new box may be below the fold or on another page, and the
            // scroll that reveals it lands a frame or more later. See
            // `Focus::waiting`.
            waiting: crate::canvas::minreveal::REVEAL_GRACE_FRAMES,
        }),
    );

    // The page turn FIRST, because the reveal below is solved only on a frame
    // that is drawing the page it names, and under a paged display mode that
    // frame does not arrive until this command has been applied.
    if leaving.is_none_or(|f| f.page != target.page) {
        actions.push(Action::GoToPage(target.page));
    }
    // …then the reveal, which outranks the page command's own scroll in
    // `canvas::offset` precisely so that the page turn cannot park the view at
    // the top of the sheet and leave the field unvisited.
    if let Some(page) = doc.pages.get(target.page) {
        let (min, max) = crate::canvas::minreveal::fracs_for_canvas_rect(target.rect, page);
        actions.push(Action::RevealRect {
            page: target.page,
            min,
            max,
            why: WHY_TAB,
        });
    }
}

/// **Hold a focus that this frame cannot draw, or settle it.**
///
/// [`super::editor`]'s two undrawable branches. A focus whose page is not in
/// the strip, or whose box is outside the clip rect, is normally a focus the
/// operator has scrolled away from — and committing it is the old spec's rule
/// and the right one, because a half-typed value is something they typed on
/// purpose.
///
/// A focus a **Tab** put there is the exception, and [`Focus::waiting`] is how
/// the two are told apart: it names a box the ring chose, whose reveal is still
/// in flight. Settling it on the frame the reveal was asked for would make Tab
/// appear to do nothing whenever the next field was off screen, which on a form
/// worth tabbing through is most of the time.
///
/// Returns `false` in both cases, because neither drew an editor and so neither
/// claimed the frame's click.
pub(super) fn hold_or_settle(
    ctx: &egui::Context,
    doc: &OpenDoc,
    focus: Focus,
    actions: &mut Vec<Action>,
) -> bool {
    if focus.waiting == 0 {
        super::settle(ctx, doc, actions);
        return false;
    }
    let waiting = focus.waiting - 1;
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed in the UI
        format!(
            "tab-field-waiting page={} field={} left={waiting}",
            focus.page, focus.field
        )
    });
    store_focus(ctx, Some(Focus { waiting, ..focus }));
    false
}

/// **The focus a check box or radio button holds**, and the keys it reads.
///
/// The button half of [`super::editor`]. There is no caret and no draft to
/// edit: the whole of the state is *this box has the keyboard*, drawn as a ring
/// and spent by Space or Enter.
///
/// # Why the ring is a cursor and not a mark on the content
///
/// pdfcer's rule 4 forbids styling applied content as provisional and admits
/// the cursor in full. A focus ring says where the next keystroke goes; it
/// states nothing about the document, disappears the moment focus leaves, and
/// is drawn over the finished raster so it reaches no print, no export and no
/// save. It is the same affordance as the I-beam this module already sets over
/// a fillable field.
pub(super) fn button_focus(
    ui: &mut Ui,
    doc: &OpenDoc,
    list: &[WidgetBox],
    focus: Focus,
    widget_box: &WidgetBox,
    rect: egui::Rect,
    actions: &mut Vec<Action>,
) -> bool {
    let ctx = ui.ctx().clone();
    let id = focus.editor_id();
    // `focusable_noninteractive`: the box takes the keyboard and nothing else.
    // Sensing a click here would take the press away from the page response
    // `super::click` reads, which is the input layering the module header §4
    // settles — and a button that both claimed the press and toggled on it
    // would toggle twice.
    let response = ui.interact(rect, id, egui::Sense::focusable_noninteractive());
    if !focus.seated {
        response.request_focus();
    } else if !response.has_focus() {
        // Something else took the keyboard — a panel field, a dialog, a click
        // on the page. The ring is over; nothing to commit, because a button
        // keeps no draft.
        store_focus(&ctx, None);
        return false;
    }

    tabnav::publish(&ctx, tabnav::Scope::Field, id);
    // ★ `canvas_selection_ink`, whose own doc names "the selected form field's
    // box" as its role. Never `visuals.selection.stroke` — that is egui's
    // CHROME channel and `tools/gates/check-selection-channel.sh` keeps it out
    // of the content area.
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::ZERO,
        egui::Stroke::new(1.5, egui_shell::theme::Theme::canvas_selection_ink(&ctx)),
        egui::StrokeKind::Outside,
    );

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        store_focus(&ctx, None);
        note_escape(&ctx);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("form-button-abandon field={}", widget_box.field)
        });
        return true;
    }

    // Space and Enter both, because the two conventions disagree by platform
    // and by program and an operator who tries the wrong one concludes the ring
    // is broken rather than that they used the wrong key.
    if ctx.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter)) {
        activate(widget_box, actions);
    } else if let Some(direction) = arrow(&ctx) {
        // Within a radio group only: the arrow keys move between the widgets of
        // ONE field and Tab moves past the whole group, which is the split
        // `EditSession::page_tab_sequence`'s own doc asks a caller to make.
        if let Some(target) = sibling(list, &focus, direction) {
            let target = target.clone();
            move_focus(&ctx, doc, Some(&focus), &target, actions);
            // Arrowing onto a radio selects it, which is the convention and is
            // also the only thing that makes the gesture useful: a group where
            // the arrows moved a ring but chose nothing would need a Space
            // after every press.
            activate(&target, actions);
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed in the UI
                format!(
                    "form-radio-arrow field={} widget={}",
                    target.field, target.widget
                )
            });
            return false;
        }
    }

    store_focus(
        &ctx,
        Some(Focus {
            seated: true,
            waiting: 0,
            ..focus
        }),
    );
    false
}

/// Raise the state change a press on this button means.
///
/// The same rule [`super::click`] applies to a pointer press, called rather
/// than restated so a keyboard activation and a click cannot come to mean
/// different things.
fn activate(widget_box: &WidgetBox, actions: &mut Vec<Action>) {
    match &widget_box.kind {
        BoxKind::Check { on_state, on } => {
            let state = if *on {
                "Off".to_owned()
            } else {
                on_state.clone()
            };
            raise_button(&widget_box.field, state, actions);
        }
        // Choosing the chosen one is not a change — see `BoxKind::Radio`.
        BoxKind::Radio { on_state, on } => {
            if !*on {
                raise_button(&widget_box.field, on_state.clone(), actions);
            }
        }
        // A caret's keys do not reach here: `super::editor` routes a text box
        // to the `TextEdit` before this function can be called.
        BoxKind::Text { .. } => {}
    }
}

/// Which way an arrow key points, or `None` when none was pressed.
///
/// Both axes, because a radio group may be laid out in a row or a column and
/// nothing in the file says which.
fn arrow(ctx: &egui::Context) -> Option<bool> {
    ctx.input(|i| {
        if i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::ArrowRight) {
            Some(false)
        } else if i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::ArrowLeft) {
            Some(true)
        } else {
            None
        }
    })
}

/// The next widget of the focused radio group, wrapping.
///
/// `None` for anything that is not a radio group, which is what keeps the arrow
/// keys out of a check box's way: a lone check box has no siblings and the
/// arrows should go on meaning whatever the canvas means by them.
fn sibling<'a>(list: &'a [WidgetBox], focus: &Focus, backwards: bool) -> Option<&'a WidgetBox> {
    let group: Vec<&WidgetBox> = list
        .iter()
        .filter(|b| {
            b.page == focus.page
                && b.field == focus.field
                && matches!(b.kind, BoxKind::Radio { .. })
        })
        .collect();
    if group.len() < 2 {
        return None;
    }
    let here = group.iter().position(|b| b.widget == focus.widget)?;
    let next = if backwards {
        (here + group.len() - 1) % group.len()
    } else {
        (here + 1) % group.len()
    };
    group.get(next).copied()
}
