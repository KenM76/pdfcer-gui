//! # `dialogs::handsign::placing` — where the signature lands in its box, chosen before it is placed
//!
//! Contract: [`Placing::show`] draws the box to scale with the room above it a
//! signature may rise into, the signature where it would land, and grips. A
//! drag on the body moves it and a drag on a grip resizes it
//! (`handsign::place::reshape`, kept inside `handsign::place::allowed`).
//! [`Placing::chosen`] is `None` until the operator moves it, so an untouched
//! signature lands by the fit rule; a signature whose proportions change goes
//! back to the fit, because a placement made for one shape would stretch
//! another.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/handsign.md`.

use egui::{Painter, Pos2, Rect, Sense, Stroke, Ui, Vec2, pos2, vec2};
use egui_shell::theme::Theme;
use pdfcer_gui_base::handles::{Grip, GripSet, grip_at};
use pdfcer_gui_base::handsign::place::{self, Placement};

use crate::text::handsign as t;

/// The placement preview.
// ui-text-exempt: trace region name, never displayed
pub const REGION_PLACEMENT: &str = "handsign.placement";
/// The signature inside the placement preview.
// ui-text-exempt: trace region name, never displayed
pub const REGION_INK: &str = "handsign.ink";
/// The Reset to fit button.
// ui-text-exempt: trace region name, never displayed
pub const REGION_RESET: &str = "handsign.reset-fit";

/// The preview's tallest drawing, in points.
const MAX_HEIGHT: f32 = 120.0;

/// Two proportions closer than this are the same shape.
const SAME_SHAPE: f32 = 1e-3;

/// The signature's rectangle in the box's space and on screen, and the
/// screen points per box unit, for a tab to paint into.
#[derive(Clone, Copy, Debug)]
pub(super) struct Ink {
    /// In the box's space (the canvas's).
    pub canvas: Rect,
    /// On screen, in the preview.
    pub screen: Rect,
    /// Screen points per box unit.
    pub scale: f32,
}

impl Ink {
    /// A point in the box's space, on screen.
    pub fn to_screen(self, p: Pos2) -> Pos2 {
        self.screen.min + (p - self.canvas.min) * self.scale
    }
}

/// The operator's choice of where the signature goes, and the drag making it.
#[derive(Debug, Default)]
pub(super) struct Placing {
    chosen: Option<Placement>,
    /// The tab and the ink proportions `chosen` was made for.
    made_for: Option<(u8, Vec2)>,
    /// The grip held and the rectangle when it was taken, in box space.
    drag: Option<(Grip, Rect)>,
}

impl Placing {
    /// The placement the operator chose; `None` lands by the fit rule.
    pub fn chosen(&self) -> Option<Placement> {
        self.chosen
    }

    /// Draw the preview for the box `target` (canvas space). `ink` is the
    /// signature's proportions on tab `tab`, `None` when there is nothing to
    /// place yet; `stretchable` is whether its proportions may change;
    /// `paint` draws the signature into an [`Ink`].
    pub fn show(
        &mut self,
        ui: &mut Ui,
        target: Rect,
        tab: u8,
        ink: Option<Vec2>,
        stretchable: bool,
        paint: &mut dyn FnMut(&Painter, Ink),
    ) {
        if let (Some((was_tab, was)), Some(now)) = (self.made_for, ink)
            && (was_tab != tab || !same_shape(was, now))
        {
            *self = Self::default();
        }
        ui.small(t::placement_hint(stretchable));
        let region = place::allowed(target);
        let width = ui.available_width();
        let scale = (width / region.width().max(1.0)).min(MAX_HEIGHT / region.height().max(1.0));
        let (area, response) = ui.allocate_exact_size(
            vec2(width, region.height() * scale),
            Sense::click_and_drag(),
        );
        crate::diag::ui_rect_visible(REGION_PLACEMENT, area, ui.clip_rect());
        let origin = pos2(area.center().x - region.width() * scale / 2.0, area.min.y);
        let to_screen = |r: Rect| {
            Rect::from_min_max(
                origin + (r.min - region.min) * scale,
                origin + (r.max - region.min) * scale,
            )
        };
        let painter = ui.painter_at(area);
        backdrop(&painter, ui, to_screen(region), to_screen(target));
        let Some(canvas) = ink.and_then(|ink| place::ink_rect(ink, target, self.chosen)) else {
            return;
        };
        let at = Ink {
            canvas,
            screen: to_screen(canvas),
            scale,
        };
        crate::diag::ui_rect_visible(REGION_INK, at.screen, ui.clip_rect());
        paint(&painter, at);
        let accent = Theme::of(ui.ctx()).palette.accent;
        painter.rect_stroke(
            at.screen,
            0.0,
            Stroke::new(1.0, accent),
            egui::StrokeKind::Outside,
        );
        crate::canvas::overlay::draw_grips(
            &painter,
            ui.visuals(),
            at.screen,
            GripSet::scale_only(),
        );
        self.drag_on(ui, &response, at, target, stretchable);
        if let Some(ink) = ink {
            self.made_for = self.chosen.map(|_| (tab, ink));
        }
        self.reset_button(ui);
    }

    /// Take, follow and let go of a grip.
    fn drag_on(
        &mut self,
        ui: &Ui,
        response: &egui::Response,
        at: Ink,
        target: Rect,
        stretchable: bool,
    ) {
        let press = ui.input(|i| i.pointer.press_origin());
        if let Some(p) = response.hover_pos()
            && let Some(grip) = grip_at(at.screen, p, GripSet::scale_only())
        {
            ui.ctx().set_cursor_icon(grip.cursor());
        }
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(p) = press
        {
            self.drag = grip_at(at.screen, p, GripSet::scale_only()).map(|g| (g, at.canvas));
        }
        if let Some((grip, original)) = self.drag
            && response.dragged_by(egui::PointerButton::Primary)
            && let (Some(from), Some(to)) = (press, response.interact_pointer_pos())
        {
            let free = ui.input(|i| i.modifiers.shift);
            let moved = place::reshape(grip, original, (to - from) / at.scale, free, stretchable);
            self.chosen = Some(Placement::of(place::clamp(moved, target), target));
        }
        if response.drag_stopped()
            && let Some((grip, _)) = self.drag.take()
            && let Some(Placement(r)) = self.chosen
        {
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "hand-sign-placement grip={} x0={:.3} y0={:.3} x1={:.3} y1={:.3}",
                    grip.name(),
                    r.min.x,
                    r.min.y,
                    r.max.x,
                    r.max.y
                )
            });
        }
    }

    fn reset_button(&mut self, ui: &mut Ui) {
        let reset = ui
            .add_enabled(self.chosen.is_some(), egui::Button::new(t::reset_fit()))
            .on_hover_text(t::reset_fit_hover());
        crate::diag::ui_rect_visible(REGION_RESET, reset.rect, ui.clip_rect());
        if reset.clicked() {
            *self = Self::default();
            // ui-text-exempt: diagnostic trace, never displayed.
            crate::diag::trace(|| "hand-sign-placement reset=1".to_owned());
        }
    }
}

/// The room a signature may rise into, faint, and the box, outlined.
fn backdrop(painter: &Painter, ui: &Ui, region: Rect, target: Rect) {
    let palette = Theme::of(ui.ctx()).palette;
    painter.rect_filled(region, 2.0, palette.panel);
    painter.rect_filled(target, 0.0, palette.surface);
    painter.rect_stroke(
        target,
        0.0,
        Stroke::new(1.0, palette.outline),
        egui::StrokeKind::Inside,
    );
}

/// Whether two proportions are the same shape.
fn same_shape(a: Vec2, b: Vec2) -> bool {
    let ratio = |v: Vec2| v.x / (v.x + v.y).max(f32::EPSILON);
    (ratio(a) - ratio(b)).abs() < SAME_SHAPE
}
