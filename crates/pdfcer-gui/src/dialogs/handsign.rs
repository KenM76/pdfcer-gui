//! # `dialogs::handsign` — the *Sign here* window: draw, type or choose a picture of a signature, place it in the box
//!
//! Opened by a click on an unsigned signature box. Contract: Place pushes one
//! [`FieldAction::HandSign`] carrying the drawn mark (normalised), the typed
//! name and face, or the picture, and where in the box the operator put it
//! (`None` for the fit rule); the Type tab is drawn only when a handwriting
//! face is usable; the Type and Picture tabs are usable only on a page shown
//! upright; the digital-ID link pushes [`FieldAction::SignWithId`] and is
//! drawn only when `file.sign` is registered. The window writes nothing to the
//! document itself.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/dialogs/handsign.md`.

use egui::{FontFamily, FontId, Painter, Pos2, Sense, Stroke, Ui, Vec2, vec2};
use egui_shell::theme::Theme;
use pdfcer_gui_base::handsign::picture::{self as sigpicture, SigPicture};
use pdfcer_gui_base::handsign::typed::{self, Typed};
use pdfcer_gui_base::handsign::{self, Mark, Signature};

mod picture;
mod placing;

use placing::Ink;

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
/// The Type tab.
// ui-text-exempt: trace region name, never displayed
pub const REGION_TAB_TYPE: &str = "handsign.tab-type";
/// The Picture tab.
// ui-text-exempt: trace region name, never displayed
pub const REGION_TAB_PICTURE: &str = "handsign.tab-picture";
pub use picture::{REGION_CHOOSE, REGION_CLEAR_WHITE};
pub use placing::{REGION_INK, REGION_PLACEMENT, REGION_RESET};
/// The name field on the Type tab.
// ui-text-exempt: trace region name, never displayed
pub const REGION_NAME: &str = "handsign.name";
/// The typed signature's preview.
// ui-text-exempt: trace region name, never displayed
pub const REGION_PREVIEW: &str = "handsign.preview";

/// The drawing area's height, in points; its width is the window's.
const PAD_HEIGHT: f32 = 160.0;
/// The on-screen pen width in the drawing area, in points.
const PAD_PEN: f32 = 2.0;
/// A new point is kept only this far from the last, in points.
const MIN_STEP: f32 = 0.75;
/// The preview's largest font size, in points.
const PREVIEW_MAX: f32 = 56.0;

/// The signatures placed this run, which *Use my last signature* and the
/// Type tab's starting name restore. Application-scoped: it is the
/// operator's, not the document's.
#[derive(Clone, Debug, Default)]
pub struct LastSignature {
    /// The last drawn signature, normalised.
    pub drawn: Option<Mark>,
    /// The last typed signature.
    pub typed: Option<Typed>,
    /// The last picture signature.
    pub picture: Option<SigPicture>,
    /// The tab the last placed came from, so the window opens on it.
    was: Tab,
}

impl LastSignature {
    /// Record `placed` as the latest.
    pub fn record(&mut self, placed: Signature) {
        match placed {
            Signature::Drawn(mark) => {
                self.drawn = Some(mark);
                self.was = Tab::Draw;
            }
            Signature::Typed(typed) => {
                self.typed = Some(typed);
                self.was = Tab::Type;
            }
            Signature::Picture(picture) => {
                self.picture = Some(picture);
                self.was = Tab::Picture;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Tab {
    #[default]
    Draw,
    Type,
    Picture,
}

impl Tab {
    /// The tab as the placement preview tells tabs apart.
    const fn key(self) -> u8 {
        match self {
            Self::Draw => 0,
            Self::Type => 1,
            Self::Picture => 2,
        }
    }
}

/// The window, and the box it signs.
pub struct HandSignDialog {
    field: String,
    page: usize,
    rect: egui::Rect,
    tab: Tab,
    /// Strokes in pad-local points, y-down.
    mark: Mark,
    /// Whether the pointer is down and drawing into the last stroke.
    drawing: bool,
    /// The signature *Use my last signature* restores, normalised.
    last: Option<Mark>,
    /// The name on the Type tab.
    name: String,
    /// The index into [`typed::faces`] the name is shown in.
    face: usize,
    /// Whether the page is shown upright, which a typed or picture
    /// signature needs.
    upright: bool,
    /// Whether the name field still needs keyboard focus.
    focus_name: bool,
    /// The name and face last checked against the face's characters, and
    /// either the name's proportions (advance by full height, per point of
    /// size) or the reason the face cannot write it.
    coverage: Option<(String, usize, Result<Vec2, String>)>,
    /// The Picture tab.
    picture: picture::PictureTab,
    /// Where in the box the operator put the signature.
    placing: placing::Placing,
    remember: bool,
    place_requested: bool,
    id_requested: bool,
    close_requested: bool,
}

impl HandSignDialog {
    /// Open on the box `rect` (canvas space) of `field` on `page`. This run's
    /// last signatures come first; the copies remembered on disk are the
    /// fallback. `upright` is whether the page is shown upright, which a
    /// typed or picture signature needs.
    #[must_use]
    pub fn open(
        field: &str,
        page: usize,
        rect: egui::Rect,
        last: &LastSignature,
        upright: bool,
        render: pdfcer_render::RenderOptions,
    ) -> Self {
        let saved = handsign::load_saved();
        let saved_typed = typed::load();
        let saved_picture = sigpicture::load();
        let remember = saved.is_some() || saved_typed.is_some() || saved_picture.is_some();
        let typed_last = last.typed.clone().or(saved_typed);
        let picture_last = last.picture.clone().or(saved_picture);
        let faces = typed::faces();
        let face = typed_last
            .as_ref()
            .and_then(|t| faces.iter().position(|f| f.label == t.face))
            .unwrap_or(0);
        let placed_this_run =
            last.drawn.is_some() || last.typed.is_some() || last.picture.is_some();
        let wanted = if placed_this_run {
            last.was
        } else if saved.is_some() {
            Tab::Draw
        } else if typed_last.is_some() {
            Tab::Type
        } else if picture_last.is_some() {
            Tab::Picture
        } else {
            Tab::Draw
        };
        let tab = match wanted {
            Tab::Type if upright && !faces.is_empty() => Tab::Type,
            Tab::Picture if upright => Tab::Picture,
            _ => Tab::Draw,
        };
        Self {
            field: field.to_owned(),
            page,
            rect,
            tab,
            mark: Mark::default(),
            drawing: false,
            last: last.drawn.clone().or(saved),
            name: typed_last.map(|t| t.name).unwrap_or_default(),
            face,
            upright,
            focus_name: tab == Tab::Type,
            coverage: None,
            picture: picture::PictureTab::with(picture_last, render),
            placing: placing::Placing::default(),
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
    ) -> (bool, Option<Signature>) {
        let (frame, ()) = crate::dialogs::host::Host::new(
            "hand-sign", // ui-text-exempt: a viewport key, never displayed.
            t::window_title(),
            egui::vec2(560.0, 640.0),
            egui::vec2(420.0, 520.0),
        )
        .show(ctx, |ui| {
            crate::diag::ui_rect(REGION_BODY, ui.max_rect());
            self.body(ui, offer_id);
        });
        let open = !frame.closed;

        if std::mem::take(&mut self.place_requested)
            && let Some(signature) = self.signature()
        {
            let kept = self.keep(&signature);
            let placement = self.placing.chosen();
            let placed = if placement.is_some() { "chosen" } else { "fit" }; // ui-text-exempt: diagnostic trace value
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                match &signature {
                    Signature::Drawn(mark) => format!(
                        "hand-sign-requested via=draw page={} strokes={} points={} remembered={} placement={placed}",
                        self.page,
                        mark.strokes.len(),
                        mark.point_count(),
                        u8::from(kept)
                    ),
                    // No name: it is the operator's own.
                    Signature::Typed(typed) => format!(
                        "hand-sign-requested via=type page={} face={} chars={} remembered={} placement={placed}",
                        self.page,
                        typed.face.replace(' ', "_"),
                        typed.name.chars().count(),
                        u8::from(kept)
                    ),
                    Signature::Picture(picture) => format!(
                        "hand-sign-requested via=picture page={} bytes={} clear_white={} remembered={} placement={placed}",
                        self.page,
                        picture.bytes.len(),
                        u8::from(picture.clear_white),
                        u8::from(kept)
                    ),
                }
            });
            actions.push(Action::Field(FieldAction::HandSign {
                field: self.field.clone(),
                page: self.page,
                rect: self.rect,
                signature: signature.clone(),
                placement,
            }));
            return (false, Some(signature));
        }
        if std::mem::take(&mut self.id_requested) {
            actions.push(Action::Field(FieldAction::SignWithId {
                field: self.field.clone(),
            }));
            return (false, None);
        }
        (open && !std::mem::take(&mut self.close_requested), None)
    }

    /// What Place would place on the current tab, or `None` when it is not
    /// ready.
    fn signature(&self) -> Option<Signature> {
        match self.tab {
            Tab::Draw => self.mark.has_extent().then(|| {
                Signature::Drawn(
                    self.mark
                        .simplified(handsign::SIMPLIFY_TOLERANCE)
                        .normalised(),
                )
            }),
            Tab::Type => {
                let name = self.name.trim();
                let face = typed::faces().get(self.face)?;
                let covered = self
                    .coverage
                    .as_ref()
                    .is_some_and(|(n, f, ink)| n == name && *f == self.face && ink.is_ok());
                (!name.is_empty() && covered).then(|| {
                    Signature::Typed(Typed {
                        face: face.label.to_owned(),
                        name: name.to_owned(),
                    })
                })
            }
            Tab::Picture => self.picture.signature().map(Signature::Picture),
        }
    }

    /// Keep or delete the copies on this computer per the checkbox. Returns
    /// whether `signature` was kept.
    fn keep(&self, signature: &Signature) -> bool {
        if !self.remember {
            handsign::forget();
            typed::forget();
            sigpicture::forget();
            return false;
        }
        match signature {
            Signature::Drawn(mark) => handsign::save(mark),
            Signature::Typed(typed) => typed::save(typed),
            Signature::Picture(picture) => sigpicture::save(picture),
        }
    }

    fn body(&mut self, ui: &mut Ui, offer_id: bool) {
        self.tabs(ui);
        match self.tab {
            Tab::Draw => self.draw_tab(ui),
            Tab::Type => self.type_tab(ui),
            Tab::Picture => self.picture.show(ui),
        }
        ui.add_space(6.0);
        self.placement(ui);
        ui.add_space(6.0);
        ui.checkbox(&mut self.remember, t::remember())
            .on_hover_text(t::remember_hover());
        ui.add_space(4.0);
        ui.small(t::what_it_is());
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            let ready = self.signature().is_some();
            let why = match self.tab {
                Tab::Draw => t::place_needs_drawing(),
                Tab::Type => t::place_needs_name(),
                Tab::Picture => t::place_needs_picture(),
            };
            let place = ui
                .add_enabled(ready, egui::Button::new(t::place()))
                .on_disabled_hover_text(why);
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

    /// Draw, Type when a handwriting face is usable, and Picture; the last
    /// two only on an upright page.
    fn tabs(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, Tab::Draw, t::tab_draw());
            if !typed::faces().is_empty() {
                let typing = ui
                    .add_enabled_ui(self.upright, |ui| {
                        ui.selectable_value(&mut self.tab, Tab::Type, t::tab_type())
                    })
                    .inner
                    .on_disabled_hover_text(t::type_needs_upright_page());
                crate::diag::ui_rect_visible(REGION_TAB_TYPE, typing.rect, ui.clip_rect());
                if typing.clicked() {
                    self.focus_name = true;
                }
            }
            let picture = ui
                .add_enabled_ui(self.upright, |ui| {
                    ui.selectable_value(&mut self.tab, Tab::Picture, t::tab_picture())
                })
                .inner
                .on_disabled_hover_text(t::picture_needs_upright_page());
            crate::diag::ui_rect_visible(REGION_TAB_PICTURE, picture.rect, ui.clip_rect());
        });
        ui.add_space(4.0);
    }

    /// The signature's proportions on the current tab, `None` while there is
    /// nothing to place, and whether they may change.
    fn ink(&self) -> (Option<Vec2>, bool) {
        match self.tab {
            Tab::Draw => (
                self.mark
                    .has_extent()
                    .then(|| self.mark.bounds())
                    .flatten()
                    .map(|b| b.size()),
                true,
            ),
            Tab::Type => {
                let name = self.name.trim();
                let ink = self.coverage.as_ref().and_then(|(n, f, ink)| {
                    (n == name && *f == self.face)
                        .then(|| ink.as_ref().ok().copied())
                        .flatten()
                });
                (ink, false)
            }
            Tab::Picture => (self.picture.chosen.as_ref().map(|c| c.size), true),
        }
    }

    /// Where in the box the signature lands, drawn to scale and adjustable.
    fn placement(&mut self, ui: &mut Ui) {
        let (ink, stretchable) = self.ink();
        let ctx = ui.ctx().clone();
        let colour = Theme::of(&ctx).palette.text;
        let tab = self.tab;
        let mark = &self.mark;
        let name = self.name.trim();
        let face = self.face;
        let picture = &mut self.picture;
        self.placing.show(
            ui,
            self.rect,
            tab.key(),
            ink,
            stretchable,
            &mut |painter, at| match tab {
                Tab::Draw => paint_mark(painter, mark, at, colour),
                Tab::Type => paint_name(&ctx, painter, name, face, ink, at, colour),
                Tab::Picture => picture.paint(&ctx, painter, at),
            },
        );
    }

    fn draw_tab(&mut self, ui: &mut Ui) {
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
    }

    fn type_tab(&mut self, ui: &mut Ui) {
        let faces = typed::faces();
        ui.label(t::type_intro());
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let field = ui.add(
                // escape-disposition: dialog-cancels — `dialogs::host` owns the key for
                // every field in this window: the first press leaves the box, the second
                // cancels.
                egui::TextEdit::singleline(&mut self.name)
                    .hint_text(t::name_hint())
                    .desired_width(240.0),
            );
            crate::diag::ui_rect_visible(REGION_NAME, field.rect, ui.clip_rect());
            if std::mem::take(&mut self.focus_name) {
                field.request_focus();
            }
            if faces.len() > 1 {
                ui.label(t::style());
                egui::ComboBox::from_id_salt("hand-sign-style") // ui-text-exempt: a widget id, never displayed.
                    .selected_text(faces.get(self.face).map_or("", |f| f.label))
                    .show_ui(ui, |ui| {
                        for (i, face) in faces.iter().enumerate() {
                            ui.selectable_value(&mut self.face, i, face.label);
                        }
                    });
            }
        });
        self.check_coverage();
        if let Some((_, _, Err(why))) = &self.coverage {
            let danger = Theme::of(ui.ctx()).palette.danger;
            ui.colored_label(danger, t::style_cannot_write(why));
        }
        ui.add_space(6.0);
        self.preview(ui);
    }

    /// Re-check, when the name or face changed, that the face has every
    /// character of the name, so Place is never offered for one it would
    /// refuse.
    fn check_coverage(&mut self) {
        let name = self.name.trim();
        if name.is_empty()
            || self
                .coverage
                .as_ref()
                .is_some_and(|(n, f, _)| n == name && *f == self.face)
        {
            return;
        }
        let ink = typed::faces()
            .get(self.face)
            .ok_or_else(String::new)
            .and_then(|face| {
                let plan = typed::plan(face, name)?;
                let advance = typed::advance(&plan, name).ok_or_else(String::new)?;
                let height = (plan.metrics.ascent as f32 - plan.metrics.descent as f32) / 1000.0;
                Ok(vec2(advance, height))
            });
        self.coverage = Some((name.to_owned(), self.face, ink));
    }

    /// The typed name in the face that will be embedded, sized to the area.
    fn preview(&self, ui: &mut Ui) {
        let palette = Theme::of(ui.ctx()).palette;
        let (rect, _) = ui.allocate_exact_size(self.pad_size(ui), Sense::hover());
        crate::diag::ui_rect_visible(REGION_PREVIEW, rect, ui.clip_rect());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, palette.surface);
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, palette.outline),
            egui::StrokeKind::Inside,
        );
        let name = self.name.trim();
        let Some(face) = typed::faces().get(self.face) else {
            return;
        };
        let Some(family) = preview_family(ui.ctx(), self.face, face) else {
            return;
        };
        if name.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                t::preview_hint(),
                egui::FontId::proportional(15.0),
                palette.text_muted,
            );
            return;
        }
        // Lay out once at a reference size, then scale to fit the area.
        let reference = 32.0;
        let width = ui.ctx().fonts_mut(|f| {
            f.layout_no_wrap(
                name.to_owned(),
                FontId::new(reference, family.clone()),
                palette.text,
            )
            .size()
            .x
        });
        let fit = (0.9 * rect.width() / width.max(1.0)) * reference;
        let size = fit.min(PREVIEW_MAX);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            name,
            FontId::new(size, family),
            palette.text,
        );
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

/// The egui family the preview of face `index` is drawn in, registering the
/// face on first use. `None` until egui has rebuilt its fonts with it, which
/// is the next frame: naming an unknown family panics inside egui.
fn preview_family(
    ctx: &egui::Context,
    index: usize,
    face: &'static typed::Face,
) -> Option<FontFamily> {
    // ui-text-exempt: an egui font key, never displayed.
    let key = format!("hand-sign-face-{index}");
    let family = FontFamily::Name(key.clone().into());
    if ctx.fonts(|f| f.definitions().families.contains_key(&family)) {
        return Some(family);
    }
    ctx.add_font(egui::epaint::text::FontInsert::new(
        &key,
        egui::FontData::from_static(face.bytes.as_slice()),
        vec![egui::epaint::text::InsertFontFamily {
            family,
            priority: egui::epaint::text::FontPriority::Highest,
        }],
    ));
    ctx.request_repaint();
    None
}

/// `mark` drawn into `at` as it would be placed.
fn paint_mark(painter: &Painter, mark: &Mark, at: Ink, colour: egui::Color32) {
    let Some(fitted) = handsign::fit_into(mark, at.canvas) else {
        return;
    };
    let pen = Stroke::new(fitted.width * at.scale, colour);
    for stroke in fitted.strokes {
        let points: Vec<Pos2> = stroke.into_iter().map(|p| at.to_screen(p)).collect();
        painter.add(egui::Shape::line(points, pen));
    }
}

/// `name` in face `face` drawn into `at`, its full height filling it; `ink`
/// is its proportions per point of size.
fn paint_name(
    ctx: &egui::Context,
    painter: &Painter,
    name: &str,
    face: usize,
    ink: Option<Vec2>,
    at: Ink,
    colour: egui::Color32,
) {
    let (Some(ink), Some(found)) = (ink, typed::faces().get(face)) else {
        return;
    };
    let Some(family) = preview_family(ctx, face, found) else {
        return;
    };
    let size = at.screen.height() / ink.y.max(f32::EPSILON);
    painter.text(
        at.screen.left_center(),
        egui::Align2::LEFT_CENTER,
        name,
        FontId::new(size, family),
        colour,
    );
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
