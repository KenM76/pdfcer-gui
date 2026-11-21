//! The in-place text editor's glyphs, caret and selection, set where the
//! field's own appearance will draw them: the engine's layout of the draft
//! (`WidgetAppearances::text_geometry` — `/DA` font and size with auto-size,
//! `/Q`, Comb with `/MaxLen`, Multiline wrap, Password mask), not egui's.
//!
//! egui's `TextEdit` keeps the draft, the keys and the cursor (character
//! indices); [`lay`] makes its text, caret and selection transparent and
//! paints them from the engine's layout, and a press or drag inside the box
//! is mapped through `caret_at_point`. Up and Down still walk egui's lines.
//! A field the engine cannot lay out (a `/DA` it refuses) keeps egui's own
//! drawing.

use std::path::PathBuf;
use std::sync::Arc;

use egui::text::{CCursor, CCursorRange};
use egui::{Color32, Id, Pos2, Rect, Stroke, Ui};
use pdfcer_core::page_tree::Page;
use pdfcer_core::widget_draw::WidgetAppearances;
use pdfcer_core::widget_draw::{FieldTextGeometry, PagePoint};
use pdfcer_gui_base::formtextbox as textbox;

use crate::app::state::OpenDoc;
use crate::canvas::forms::boxes::WidgetBox;
use crate::canvas::mapping::PageMapping;

/// `egui::Memory` key for the parsed form, per revision.
const FORM_KEY: &str = "pdfcer-canvas-form-caret-form"; // ui-text-exempt: internal memory id, never displayed
/// `egui::Memory` key for where the click that opened an editor landed.
const OPENED_KEY: &str = "pdfcer-canvas-form-caret-opened"; // ui-text-exempt: internal memory id, never displayed
/// `egui::Memory` key for a drag's anchor, in characters.
const ANCHOR_KEY: &str = "pdfcer-canvas-form-caret-anchor"; // ui-text-exempt: internal memory id, never displayed

/// The engine's layout of one draft, and how to put it on screen.
struct Placed<'a> {
    geom: FieldTextGeometry,
    page: &'a Page,
    map: PageMapping,
}

impl Placed<'_> {
    fn screen(&self, [x, y]: PagePoint) -> Option<Pos2> {
        let canvas = crate::viewer::pdf_space_to_canvas(Pos2::new(x as f32, y as f32), self.page)?;
        Some(self.map.to_screen(canvas))
    }

    fn offset_at(&self, pos: Pos2) -> Option<usize> {
        let pdf = crate::viewer::canvas_to_pdf_space(self.map.to_page(pos), self.page)?;
        Some(self.geom.caret_at_point(f64::from(pdf.x), f64::from(pdf.y)))
    }
}

/// Record where the click that opens `widget`'s editor landed, in canvas
/// space, so the caret is seated there rather than at the end.
pub(super) fn opened_at(ctx: &egui::Context, editor: Id, canvas: Pos2) {
    ctx.data_mut(|d| d.insert_temp(Id::new(OPENED_KEY), Some((editor, canvas))));
}

/// Lay the editor over `widget_box` and draw its text from the engine.
///
/// `seating` on the editor's first frame: the caret goes where the opening
/// click landed, or to the end when a Tab opened it.
pub(super) fn lay(
    ui: &mut Ui,
    doc: &OpenDoc,
    (widget_box, map): (&WidgetBox, PageMapping),
    draft: &mut String,
    (spec, seating): (&textbox::Spec<'_>, bool),
) -> egui::Response {
    let ctx = ui.ctx().clone();
    let page = doc.pages.get(widget_box.page);
    let form = form(&ctx, doc);
    let probe = form
        .as_ref()
        .and_then(|f| f.text_geometry(widget_box.id, draft));
    let (Some(page), Some(form), Some(_)) = (page, form, probe) else {
        let response = textbox::lay(ui, draft, spec);
        if seating {
            ctx.data_mut(|d| d.remove_temp::<Option<(Id, Pos2)>>(Id::new(OPENED_KEY)));
            textbox::seat(&ctx, spec.id, draft, false);
        }
        return response;
    };
    let response = ui
        .scope(|ui| {
            let v = ui.visuals_mut();
            v.text_cursor.stroke = Stroke::NONE;
            v.selection.bg_fill = Color32::TRANSPARENT;
            textbox::lay(
                ui,
                draft,
                &textbox::Spec {
                    engine_drawn: true,
                    ..*spec
                },
            )
        })
        .inner;
    let Some(geom) = form.text_geometry(widget_box.id, draft) else {
        return response;
    };
    let placed = Placed { geom, page, map };
    if seating {
        if !seat(&ctx, spec.id, &placed, draft) {
            textbox::seat(&ctx, spec.id, draft, false);
        }
    } else {
        point(&ctx, spec.id, &response, &placed, draft);
    }
    paint(ui, spec, &placed, draft);
    response
}

/// The form, parsed once per revision.
fn form(ctx: &egui::Context, doc: &OpenDoc) -> Option<Arc<WidgetAppearances>> {
    type Cached = ((PathBuf, u64), Option<Arc<WidgetAppearances>>);
    let id = Id::new(FORM_KEY);
    let key = (doc.path.clone(), doc.edit_epoch);
    if let Some((k, form)) = ctx.data(|d| d.get_temp::<Cached>(id))
        && k == key
    {
        return form;
    }
    let form = WidgetAppearances::read(&doc.session.view()).map(Arc::new);
    ctx.data_mut(|d| d.insert_temp::<Cached>(id, (key, form.clone())));
    form
}

/// Characters before byte `offset` of `draft`.
fn chars(draft: &str, offset: usize) -> usize {
    draft.get(..offset).map_or(0, |s| s.chars().count())
}

/// Put the caret where the click that opened this editor landed; `false`
/// when no click opened it.
fn seat(ctx: &egui::Context, id: Id, placed: &Placed<'_>, draft: &str) -> bool {
    let opened = ctx
        .data_mut(|d| d.remove_temp::<Option<(Id, Pos2)>>(Id::new(OPENED_KEY)))
        .flatten();
    let Some(at) = opened
        .filter(|(editor, _)| *editor == id)
        .and_then(|(_, canvas)| placed.offset_at(placed.map.to_screen(canvas)))
    else {
        return false;
    };
    set(ctx, id, CCursorRange::one(CCursor::new(chars(draft, at))));
    true
}

/// Move the caret under a press, and the selection's far end under a drag,
/// inside the open editor.
fn point(ctx: &egui::Context, id: Id, response: &egui::Response, placed: &Placed<'_>, draft: &str) {
    let anchor_id = Id::new((ANCHOR_KEY, id));
    let (pressed, down, pos) = ctx.input(|i| {
        (
            i.pointer.primary_pressed(),
            i.pointer.primary_down(),
            i.pointer.interact_pos(),
        )
    });
    if !down {
        ctx.data_mut(|d| d.remove_temp::<usize>(anchor_id));
        return;
    }
    if response.double_clicked() || response.triple_clicked() {
        return;
    }
    let Some(at) = pos
        .and_then(|p| placed.offset_at(p))
        .map(|o| chars(draft, o))
    else {
        return;
    };
    let anchor = if pressed && response.contains_pointer() {
        ctx.data_mut(|d| d.insert_temp(anchor_id, at));
        Some(at)
    } else {
        ctx.data(|d| d.get_temp::<usize>(anchor_id))
    };
    if let Some(anchor) = anchor {
        set(
            ctx,
            id,
            CCursorRange::two(CCursor::new(anchor), CCursor::new(at)),
        );
    }
}

fn set(ctx: &egui::Context, id: Id, range: CCursorRange) {
    if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
        state.cursor.set_char_range(Some(range));
        egui::TextEdit::store_state(ctx, id, state);
    }
}

/// Selection, glyphs and caret, in that order, from the engine's layout.
fn paint(ui: &Ui, spec: &textbox::Spec<'_>, placed: &Placed<'_>, draft: &str) {
    let painter = ui.painter_at(spec.rect);
    let range = egui::TextEdit::load_state(ui.ctx(), spec.id)
        .and_then(|s| s.cursor.char_range())
        .map(|r| {
            let byte = |c: CCursor| {
                draft
                    .char_indices()
                    .nth(c.index.into())
                    .map_or(draft.len(), |(b, _)| b)
            };
            let (a, z) = (byte(r.primary), byte(r.secondary));
            (a, a.min(z), a.max(z))
        });
    let Some(scale) = scale(placed) else {
        return;
    };
    let font = egui::FontId::proportional((placed.geom.font_size as f32 * scale).max(1.0));
    // The `/DA` colour is not in the layout; a field's text is black on a page
    // far more often than not, and a `/MK /BG` tint brings its own ink.
    let ink = textbox::ink(ui.ctx(), spec).unwrap_or(Color32::BLACK);
    if let Some((_, lo, hi)) = range.filter(|(_, lo, hi)| lo < hi) {
        select(
            &painter,
            placed,
            (lo, hi),
            ui.visuals().selection.bg_fill.gamma_multiply(0.6),
        );
    }
    for c in &placed.geom.chars {
        if c.start == c.stop {
            continue;
        }
        let shown = if spec.password {
            "*"
        } else {
            draft.get(c.offset..c.end).unwrap_or("?")
        };
        let Some(at) = placed.screen(c.start) else {
            continue;
        };
        let galley = painter.layout_no_wrap(shown.to_owned(), font.clone(), ink);
        let base = galley
            .rows
            .first()
            .and_then(|r| r.glyphs.first().map(|g| r.pos.y + g.pos.y))
            .unwrap_or(0.0);
        painter.galley(Pos2::new(at.x, at.y - base), galley, ink);
    }
    if let Some((caret, ..)) = range
        && let Some([foot, head]) = placed.geom.caret_segment(caret)
        && let (Some(a), Some(b)) = (placed.screen(foot), placed.screen(head))
    {
        painter.line_segment([a, b], Stroke::new(1.5, ink));
    }
}

/// Screen pixels per page point at this zoom.
fn scale(placed: &Placed<'_>) -> Option<f32> {
    let o = placed.screen([0.0, 0.0])?;
    let up = placed.screen([0.0, 1.0])?;
    Some(o.distance(up)).filter(|s| s.is_finite() && *s > 0.0)
}

/// Shade the characters in `lo..hi` (byte offsets) over their line's height.
fn select(painter: &egui::Painter, placed: &Placed<'_>, (lo, hi): (usize, usize), fill: Color32) {
    for c in placed
        .geom
        .chars
        .iter()
        .filter(|c| c.offset >= lo && c.end <= hi)
    {
        let Some(line) = placed.geom.lines.get(c.line) else {
            continue;
        };
        let foot = line.corners[0][1];
        let head = line.corners[3][1];
        let (Some(a), Some(b)) = (
            placed.screen([c.start[0], foot]),
            placed.screen([c.stop[0], head]),
        ) else {
            continue;
        };
        painter.rect_filled(Rect::from_two_pos(a, b), 0.0, fill);
    }
}
