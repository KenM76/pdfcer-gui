//! A hand signature already in a signature box: click it to select it, then
//! drag its body to move it or a grip to resize it, inside the room its box
//! allows (`handsign::place::clamp`). Release raises
//! `FieldAction::AdjustHandSign`; Escape or a click elsewhere drops the
//! selection. The outline, grips and the dragged rectangle are the cursor:
//! the signature itself is drawn by the page, as it saves.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/forms/sigadjust.md`.

use egui::{Id, Pos2, Rect, Sense, Stroke, Ui, pos2};
use egui_shell::theme::Theme;
use pdfcer_core::page_tree::{Page, Rect as PageRect};
use pdfcer_gui_base::handles::{Grip, GripSet, grip_at};
use pdfcer_gui_base::handsign::{HandSigned, place};

use super::boxes::FieldTarget;
use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::canvas::strip::{DrawnPage, PageView};
use crate::viewer;

/// Where the selection lives between frames.
const STATE: &str = "pdfcer-hand-sig-adjust"; // ui-text-exempt: internal memory id, never displayed

/// Screen points around a selected signature that still reach its grips.
const GRIP_REACH: f32 = 8.0;

/// The selected signature, and the grip being dragged with the rectangle it
/// started from and the one last asked for (canvas space). The last is kept
/// because the release frame carries no pointer position to recompute it.
#[derive(Clone, Debug, Default)]
struct Selection {
    field: String,
    page: usize,
    drag: Option<(Grip, Rect, Rect)>,
}

/// A signature on a drawn page, with its box.
struct OnPage<'a> {
    field: &'a str,
    page: usize,
    /// The mark, in canvas space.
    canvas: Rect,
    /// The box it signs, in canvas space.
    target: Rect,
    view: &'a PageView,
    sheet: &'a Page,
}

/// Draw and drive the selection. Returns whether this frame's click was
/// taken.
pub(super) fn overlay(
    ui: &Ui,
    sheets: &[Page],
    pages: &[PageView],
    drawn: &[DrawnPage],
    targets: &[FieldTarget],
    signed: &HandSigned,
    actions: &mut Vec<Action>,
) -> bool {
    let ctx = ui.ctx().clone();
    let marks = on_pages(sheets, pages, targets, signed);
    let mut selected =
        load(&ctx).filter(|s| marks.iter().any(|m| m.field == s.field && m.page == s.page));
    if let Some(first) = marks.first() {
        // ui-text-exempt: a diagnostic region name, never displayed.
        crate::diag::ui_rect("form.hand-sig", first.view.map.rect_to_screen(first.canvas));
    }
    if selected.is_some() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        selected = None;
    }
    let mut taken = false;
    for mark in &marks {
        taken |= drive(ui, mark, &mut selected, actions);
    }
    // A click on the page that no signature took drops the selection.
    if !taken
        && drawn
            .iter()
            .any(|d| d.response.clicked_by(egui::PointerButton::Primary))
    {
        selected = None;
    }
    store(&ctx, selected);
    taken
}

/// The signed boxes' marks on the drawn pages.
fn on_pages<'a>(
    sheets: &'a [Page],
    pages: &'a [PageView],
    targets: &'a [FieldTarget],
    signed: &'a HandSigned,
) -> Vec<OnPage<'a>> {
    signed
        .marks()
        .iter()
        .filter_map(|m| {
            let view = pages.iter().find(|v| v.page == m.page)?;
            let target = targets
                .iter()
                .find(|t| t.page == m.page && t.field == m.field)?;
            let sheet = sheets.get(m.page)?;
            let canvas = to_canvas(m.bounds, sheet)?;
            Some(OnPage {
                field: &m.field,
                page: m.page,
                canvas,
                target: target.rect,
                view,
                sheet,
            })
        })
        .collect()
}

/// One signature's hover, selection, grips and drag.
fn drive(
    ui: &Ui,
    mark: &OnPage<'_>,
    selected: &mut Option<Selection>,
    actions: &mut Vec<Action>,
) -> bool {
    let map = &mark.view.map;
    let screen = map.rect_to_screen(mark.canvas);
    let is_selected = selected
        .as_ref()
        .is_some_and(|s| s.field == mark.field && s.page == mark.page);
    let reach = if is_selected {
        screen.expand(GRIP_REACH)
    } else {
        screen
    };
    let id = Id::new((STATE, mark.field, mark.page));
    let response = ui.interact(reach, id, Sense::click_and_drag());
    let painter = ui.painter();
    let accent = Theme::of(ui.ctx()).palette.accent;
    let offer = GripSet::scale_only();
    if let Some(p) = response.hover_pos() {
        let grip = if is_selected {
            grip_at(screen, p, offer)
        } else {
            None
        };
        ui.ctx()
            .set_cursor_icon(grip.map_or(egui::CursorIcon::PointingHand, Grip::cursor));
        if !is_selected {
            painter.rect_stroke(
                screen,
                0.0,
                Stroke::new(1.0, accent),
                egui::StrokeKind::Outside,
            );
        }
    }
    let press = ui.input(|i| i.pointer.press_origin());
    if response.clicked() || response.drag_started_by(egui::PointerButton::Primary) {
        let grip = press
            .filter(|_| is_selected)
            .and_then(|p| grip_at(screen, p, offer))
            .unwrap_or(Grip::Move);
        let starting = response.drag_started_by(egui::PointerButton::Primary);
        *selected = Some(Selection {
            field: mark.field.to_owned(),
            page: mark.page,
            drag: starting.then_some((grip, mark.canvas, mark.canvas)),
        });
        // ui-text-exempt: diagnostic trace, never displayed. No field name.
        crate::diag::trace(|| format!("hand-sig-selected page={} grip={}", mark.page, grip.name()));
    }
    let Some(sel) = selected
        .as_mut()
        .filter(|s| s.field == mark.field && s.page == mark.page)
    else {
        return response.clicked();
    };
    // ui-text-exempt: a diagnostic region name, never displayed.
    crate::diag::ui_rect("form.hand-sig.selected", screen);
    painter.rect_stroke(
        screen,
        0.0,
        Stroke::new(1.0, accent),
        egui::StrokeKind::Outside,
    );
    crate::canvas::overlay::draw_grips(painter, ui.visuals(), screen, offer);
    let Some((grip, original, last)) = sel.drag else {
        return response.clicked();
    };
    let shift = ui.input(|i| i.modifiers.shift);
    let wanted = match (press, response.interact_pointer_pos()) {
        (Some(from), Some(to)) => {
            let delta = map.to_page(to) - map.to_page(from);
            place::clamp(
                place::reshape(grip, original, delta, shift, true),
                mark.target,
            )
        }
        _ => last,
    };
    sel.drag = Some((grip, original, wanted));
    if response.dragged_by(egui::PointerButton::Primary) {
        painter.rect_stroke(
            map.rect_to_screen(wanted),
            0.0,
            Stroke::new(1.5, accent),
            egui::StrokeKind::Middle,
        );
    }
    if response.drag_stopped() {
        sel.drag = None;
        raise(mark, original, wanted, grip, actions);
    }
    true
}

/// Ask for the signature to move from `from` to `to` (canvas space).
fn raise(mark: &OnPage<'_>, from: Rect, to: Rect, grip: Grip, actions: &mut Vec<Action>) {
    if (from.min - to.min).length() + (from.max - to.max).length() < 0.05 {
        return;
    }
    let (Some(a), Some(b)) = (to_page(from, mark.sheet), to_page(to, mark.sheet)) else {
        return;
    };
    crate::diag::trace(|| {
        // ui-text-exempt: diagnostic trace, never displayed. No field name.
        format!(
            "hand-sig-adjust page={} grip={} x0={:.2} y0={:.2} x1={:.2} y1={:.2}",
            mark.page,
            grip.name(),
            b.llx,
            b.lly,
            b.urx,
            b.ury
        )
    });
    actions.push(Action::Field(FieldAction::AdjustHandSign {
        field: mark.field.to_owned(),
        page: mark.page,
        from: a,
        to: b,
    }));
}

/// A PDF user-space rectangle on `sheet`, in canvas space.
fn to_canvas(r: PageRect, sheet: &Page) -> Option<Rect> {
    #[allow(clippy::cast_possible_truncation)]
    let corner = |x: f64, y: f64| viewer::pdf_space_to_canvas(pos2(x as f32, y as f32), sheet);
    Some(Rect::from_two_pos(
        corner(r.llx, r.lly)?,
        corner(r.urx, r.ury)?,
    ))
}

/// A canvas-space rectangle on `sheet`, in PDF user space.
fn to_page(r: Rect, sheet: &Page) -> Option<PageRect> {
    let a: Pos2 = viewer::canvas_to_pdf_space(r.min, sheet)?;
    let b: Pos2 = viewer::canvas_to_pdf_space(r.max, sheet)?;
    Some(PageRect {
        llx: f64::from(a.x.min(b.x)),
        lly: f64::from(a.y.min(b.y)),
        urx: f64::from(a.x.max(b.x)),
        ury: f64::from(a.y.max(b.y)),
    })
}

fn load(ctx: &egui::Context) -> Option<Selection> {
    ctx.data(|d| d.get_temp::<Selection>(Id::new(STATE)))
        .filter(|s| !s.field.is_empty())
}

fn store(ctx: &egui::Context, selection: Option<Selection>) {
    ctx.data_mut(|d| d.insert_temp(Id::new(STATE), selection.unwrap_or_default()));
}
