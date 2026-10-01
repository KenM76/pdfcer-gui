//! Unsigned signature boxes on the page: a red corner tag, a *Click to sign*
//! tip, and a click that opens the *Sign here* window on that box — Acrobat's
//! behaviour. A box this session has signed by hand is not passed in.

use egui::{Pos2, Ui, vec2};
use egui_shell::theme::Theme;

use super::boxes::{FieldTarget, hit_target};
use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::canvas::strip::{DrawnPage, PageView};
use crate::text::forms as t;

/// The tag's leg on screen, in points; a smaller box caps it.
const TAG_LEG: f32 = 12.0;
/// How far the tip sits from the pointer — `notepopup`'s offset.
const TIP_OFFSET: f32 = 16.0;

/// Draw every tag, show the tip under the pointer, and raise a click.
/// Returns whether this frame's click was taken.
pub(super) fn overlay(
    ui: &Ui,
    pages: &[PageView],
    drawn: &[DrawnPage],
    unsigned: &[FieldTarget],
    signed: &pdfcer_gui_base::handsign::Ledger,
    actions: &mut Vec<Action>,
) -> bool {
    // Each kept box carries its index in the full list: the signing strip's
    // Next numbers boxes the same way, and the trace names them by it.
    let (index, unsigned): (Vec<usize>, Vec<FieldTarget>) = unsigned
        .iter()
        .enumerate()
        .filter(|(_, t)| !signed.is_signed(&t.field))
        .map(|(i, t)| (i, t.clone()))
        .unzip();
    let unsigned = unsigned.as_slice();
    if unsigned.is_empty() {
        return false;
    }
    let painter = ui.painter().clone();
    // The signature-needed red of Acrobat's tag is the palette's warning red.
    let tag = Theme::of(ui.ctx()).palette.danger;
    for (k, target) in unsigned.iter().enumerate() {
        let Some(view) = pages.iter().find(|v| v.page == target.page) else {
            continue;
        };
        let screen = view.map.rect_to_screen(target.rect);
        if k == 0 {
            // ui-text-exempt: a diagnostic region name, never displayed.
            crate::diag::ui_rect("form.sign-box", screen);
        }
        // ui-text-exempt: a diagnostic region name, never displayed.
        crate::diag::ui_rect(&format!("form.sign-box.{}", index[k]), screen);
        let leg = TAG_LEG.min(screen.width()).min(screen.height());
        let corner = screen.left_top();
        painter.add(egui::Shape::convex_polygon(
            vec![corner, corner + vec2(leg, 0.0), corner + vec2(0.0, leg)],
            tag,
            egui::Stroke::NONE,
        ));
    }
    let ctx = ui.ctx().clone();
    if let Some(pos) = ctx.pointer_latest_pos()
        && under(&ctx, pages, unsigned, pos).is_some()
    {
        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
        let tip = t::sign_box_tooltip();
        egui::Area::new(egui::Id::new("pdfcer-sign-box-tooltip")) // ui-text-exempt: internal widget id, never displayed
            .order(egui::Order::Tooltip)
            .fixed_pos(pos + vec2(TIP_OFFSET, TIP_OFFSET))
            .interactable(false)
            .show(&ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| ui.label(tip));
            });
    }
    click(&ctx, pages, drawn, unsigned, actions)
}

/// The unsigned box under `pos`, unless a window is over it.
fn under<'a>(
    ctx: &egui::Context,
    pages: &[PageView],
    unsigned: &'a [FieldTarget],
    pos: Pos2,
) -> Option<&'a FieldTarget> {
    if ctx
        .layer_id_at(pos)
        .is_some_and(|layer| layer.order > egui::Order::Background)
    {
        return None;
    }
    pages
        .iter()
        .filter(|v| v.map.image_rect().contains(pos))
        .find_map(|v| hit_target(unsigned, v.page, v.map.to_page(pos)))
}

/// A primary click on an unsigned box asks for the *Sign here* window on it.
fn click(
    ctx: &egui::Context,
    pages: &[PageView],
    drawn: &[DrawnPage],
    unsigned: &[FieldTarget],
    actions: &mut Vec<Action>,
) -> bool {
    if !drawn
        .iter()
        .any(|d| d.response.clicked_by(egui::PointerButton::Primary))
    {
        return false;
    }
    let Some(pos) = ctx.pointer_interact_pos() else {
        return false;
    };
    let Some(target) = under(ctx, pages, unsigned, pos) else {
        return false;
    };
    // ui-text-exempt: diagnostic trace, never displayed. No field name: it is
    // text from the operator's own document.
    crate::diag::trace(|| format!("sign-box-click page={}", target.page));
    actions.push(Action::Field(FieldAction::Sign {
        field: target.field.clone(),
        page: target.page,
        rect: target.rect,
    }));
    true
}
