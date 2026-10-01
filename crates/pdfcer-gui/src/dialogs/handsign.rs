//! # `dialogs::handsign` — the *Sign here* window: draw a signature, place it in the box
//!
//! Opened by a click on an unsigned signature box. Contract: Place pushes one
//! [`FieldAction::HandSign`] carrying the normalised mark; the digital-ID link
//! pushes [`FieldAction::SignWithId`] and is drawn only when `file.sign` is
//! registered. The window writes nothing to the document itself.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/handsign.md`.

use egui::{Pos2, Sense, Stroke, Ui, vec2};
use egui_shell::theme::Theme;
use pdfcer_gui_base::handsign::{self, Mark};

use crate::app::actions::Action;
use crate::app::actions::forms::FieldAction;
use crate::text::handsign as t;

/// The window body, for `ui-verify`.
// ui-text-exempt: trace region name, never displayed
pub const REGION_BODY: &str = "handsign.body";
/// The drawing area.
// ui-text-exempt: trace region name, never displayed
pub const REGION_PAD: &str = "handsign.pad";
/// The Place signature button.
// ui-text-exempt: trace region name, never displayed
pub const REGION_PLACE: &str = "handsign.place";
/// The digital-ID link.
// ui-text-exempt: trace region name, never displayed
pub const REGION_DIGITAL_ID: &str = "handsign.digital-id";

/// The drawing area's height, in points; its width is the window's.
const PAD_HEIGHT: f32 = 160.0;
/// The on-screen pen width in the drawing area, in points.
const PAD_PEN: f32 = 2.0;
/// A new point is kept only this far from the last, in points.
const MIN_STEP: f32 = 0.75;

/// The window, and the box it signs.
pub struct HandSignDialog {
    field: String,
    page: usize,
    rect: egui::Rect,
    /// Strokes in pad-local points, y-down.
    mark: Mark,
    /// Whether the pointer is down and drawing into the last stroke.
    drawing: bool,
    /// The signature *Use my last signature* restores, normalised.
    last: Option<Mark>,
    remember: bool,
    place_requested: bool,
    id_requested: bool,
    close_requested: bool,
}

impl HandSignDialog {
    /// Open on the box `rect` (canvas space) of `field` on `page`. `last` is
    /// this session's last placed signature, if any; the remembered one on
    /// disk is the fallback.
    #[must_use]
    pub fn open(field: &str, page: usize, rect: egui::Rect, last: Option<Mark>) -> Self {
        let saved = handsign::load_saved();
        let remember = saved.is_some();
        Self {
            field: field.to_owned(),
            page,
            rect,
            mark: Mark::default(),
            drawing: false,
            last: last.or(saved),
            remember,
            place_requested: false,
            id_requested: false,
            close_requested: false,
        }
    }

    /// Draw it. `offer_id` is whether the certificate route is registered.
    /// Returns whether it stays open, and the signature just placed.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        actions: &mut Vec<Action>,
        offer_id: bool,
    ) -> (bool, Option<Mark>) {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "hand-sign", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(560.0, 400.0),
            egui::vec2(420.0, 340.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui, offer_id);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.place_requested) {
            let mark = self
                .mark
                .simplified(handsign::SIMPLIFY_TOLERANCE)
                .normalised();
            let kept = if self.remember {
                handsign::save(&mark)
            } else {
                handsign::forget();
                false
            };
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "hand-sign-requested page={} strokes={} points={} remembered={}",
                    self.page,
                    mark.strokes.len(),
                    mark.point_count(),
                    u8::from(kept)
                )
            });
            actions.push(Action::Field(FieldAction::HandSign {
                field: self.field.clone(),
                page: self.page,
                rect: self.rect,
                mark: mark.clone(),
            }));
            return (false, Some(mark));
        }
        if std::mem::take(&mut self.id_requested) {
            actions.push(Action::Field(FieldAction::SignWithId {
                field: self.field.clone(),
            }));
            return (false, None);
        }
        (open && !std::mem::take(&mut self.close_requested), None)
    }

    fn body(&mut self, ui: &mut Ui, offer_id: bool) {
        ui.label(t::intro());
        ui.add_space(6.0);
        self.pad(ui);
        ui.horizontal(|ui| {
            if ui.button(t::clear()).clicked() {
                self.mark = Mark::default();
                self.drawing = false;
            }
            if ui
                .add_enabled(!self.mark.is_empty(), egui::Button::new(t::undo_stroke()))
                .clicked()
            {
                self.mark.strokes.pop();
                self.drawing = false;
            }
            if let Some(last) = &self.last
                && ui.button(t::use_last()).clicked()
            {
                self.mark = restore(last, self.pad_size(ui));
                self.drawing = false;
            }
        });
        ui.checkbox(&mut self.remember, t::remember())
            .on_hover_text(t::remember_hover());
        ui.add_space(4.0);
        ui.small(t::what_it_is());
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            let ready = self.mark.has_extent();
            let place = ui
                .add_enabled(ready, egui::Button::new(t::place()))
                .on_disabled_hover_text(t::place_needs_drawing());
            crate::diag::ui_rect_visible(REGION_PLACE, place.rect, ui.clip_rect());
            if place.clicked() {
                self.place_requested = true;
            }
            if ui.button(t::cancel()).clicked() {
                self.close_requested = true;
            }
        });
        if offer_id {
            ui.add_space(6.0);
            let link = ui
                .link(t::digital_id())
                .on_hover_text(t::digital_id_hover());
            crate::diag::ui_rect_visible(REGION_DIGITAL_ID, link.rect, ui.clip_rect());
            if link.clicked() {
                self.id_requested = true;
            }
        }
    }

    fn pad_size(&self, ui: &Ui) -> egui::Vec2 {
        vec2(ui.available_width(), PAD_HEIGHT)
    }

    /// The drawing area: drag draws a stroke, a click leaves a dot.
    fn pad(&mut self, ui: &mut Ui) {
        let palette = Theme::of(ui.ctx()).palette;
        let (rect, response) = ui.allocate_exact_size(self.pad_size(ui), Sense::click_and_drag());
        crate::diag::ui_rect_visible(REGION_PAD, rect, ui.clip_rect());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, palette.surface);
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, palette.outline),
            egui::StrokeKind::Inside,
        );
        let baseline = rect.bottom() - rect.height() * 0.25;
        painter.line_segment(
            [
                Pos2::new(rect.left() + 16.0, baseline),
                Pos2::new(rect.right() - 16.0, baseline),
            ],
            Stroke::new(1.0, palette.text_muted),
        );
        if self.mark.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                t::pad_hint(),
                egui::FontId::proportional(15.0),
                palette.text_muted,
            );
        }

        let local = |p: Pos2| (p - rect.min).to_pos2();
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(p) = ui
                .input(|i| i.pointer.press_origin())
                .or_else(|| response.interact_pointer_pos())
        {
            self.mark.strokes.push(vec![local(p)]);
            self.drawing = true;
        } else if self.drawing
            && response.dragged_by(egui::PointerButton::Primary)
            && let Some(p) = response.interact_pointer_pos()
            && let Some(stroke) = self.mark.strokes.last_mut()
        {
            let p = local(p.clamp(rect.min, rect.max));
            if stroke.last().is_none_or(|q| (p - *q).length() >= MIN_STEP) {
                stroke.push(p);
            }
        } else if response.clicked_by(egui::PointerButton::Primary)
            && let Some(p) = response.interact_pointer_pos()
        {
            self.mark.strokes.push(vec![local(p)]);
        }
        if response.drag_stopped() {
            self.drawing = false;
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "hand-sign-stroke strokes={} points={}",
                    self.mark.strokes.len(),
                    self.mark.point_count()
                )
            });
        }

        let pen = Stroke::new(PAD_PEN, palette.text);
        for stroke in &self.mark.strokes {
            let points: Vec<Pos2> = stroke.iter().map(|p| rect.min + p.to_vec2()).collect();
            match points.as_slice() {
                [] => {}
                [dot] => {
                    painter.circle_filled(*dot, PAD_PEN / 2.0, palette.text);
                }
                _ => {
                    painter.add(egui::Shape::line(points, pen));
                }
            }
        }
    }
}

/// A normalised mark scaled back into a pad of `size`, centred, with margin.
fn restore(mark: &Mark, size: egui::Vec2) -> Mark {
    let Some(b) = mark.bounds() else {
        return Mark::default();
    };
    let margin = 16.0;
    let scale = ((size.x - 2.0 * margin) / b.width().max(f32::EPSILON))
        .min((size.y - 2.0 * margin) / b.height().max(f32::EPSILON));
    let offset = vec2(
        (size.x - b.width() * scale) / 2.0,
        (size.y - b.height() * scale) / 2.0,
    );
    Mark {
        strokes: mark
            .strokes
            .iter()
            .map(|s| {
                s.iter()
                    .map(|p| ((p.to_vec2() - b.min.to_vec2()) * scale + offset).to_pos2())
                    .collect()
            })
            .collect(),
    }
}
