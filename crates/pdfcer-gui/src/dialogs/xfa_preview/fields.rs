//! Form preview's fields: where each field lies on a drawn page, and the
//! in-place editor a click opens over it.
//!
//! The hover outline is the cursor, not content: it is drawn only while the
//! pointer is over a field. Leaving the field by Enter, Esc or a click
//! elsewhere writes a changed value, as one undo step: typed words are never
//! thrown away, and Ctrl+Z takes them back.

use pdfcer_core::xfa::{ContainerKind, LayoutPage, MergedForm};
use pdfcer_gui_base::formpreview::FormPreviewAction;

use crate::app::actions::Action;
use crate::text::xfapreview as t;

/// One field on a drawn page; `rect` is in page points from the top-left.
pub(super) struct FieldBox {
    path: Vec<usize>,
    name: Option<String>,
    rect: egui::Rect,
    value: String,
}

/// The visible fields of `page`, with the values `merged` binds to them.
pub(super) fn of_page(page: &LayoutPage, merged: &MergedForm) -> Vec<FieldBox> {
    page.items
        .iter()
        .filter(|i| i.kind == ContainerKind::Field && i.visible)
        .map(|i| FieldBox {
            path: i.path.clone(),
            name: i.name.clone(),
            rect: egui::Rect::from_min_size(
                egui::pos2(i.rect.x as f32, i.rect.y as f32),
                egui::vec2(i.rect.w as f32, i.rect.h as f32),
            ),
            value: merged
                .bound
                .iter()
                .find(|b| b.path == i.path)
                .map(|b| b.value.clone())
                .unwrap_or_default(),
        })
        .collect()
}

/// The field being typed into: its page, its index in that page's fields,
/// and the text so far.
pub(super) struct Editing {
    page: usize,
    index: usize,
    text: String,
    focused: bool,
}

/// What the overlay of one page needs from the window.
pub(super) struct Overlay<'a> {
    /// Zero-based page index.
    pub page: usize,
    /// Where the page's picture is on screen.
    pub image: egui::Rect,
    /// Screen points per page point.
    pub scale: f32,
    /// `false` while a redraw is in flight: a click then opens nothing.
    pub enabled: bool,
    pub choices: [bool; 5],
    pub actions: &'a mut Vec<Action>,
}

/// Draw `fields` over their page's picture: the hover outline, and the
/// editor for the field being typed into.
pub(super) fn overlay(
    ui: &mut egui::Ui,
    o: Overlay<'_>,
    fields: &[FieldBox],
    editing: &mut Option<Editing>,
) {
    for (n, f) in fields.iter().enumerate() {
        let r = egui::Rect::from_min_size(
            o.image.min + f.rect.min.to_vec2() * o.scale,
            f.rect.size() * o.scale,
        );
        // ui-text-exempt: trace region name, never displayed
        crate::diag::ui_rect(&format!("form-preview.field.{}.{n}", o.page), r);
        if let Some(e) = editing
            .as_mut()
            .filter(|e| e.page == o.page && e.index == n)
        {
            // escape-disposition: commits — Esc ends focus, and `lost_focus` writes.
            let edit = ui.put(r, egui::TextEdit::singleline(&mut e.text));
            if !e.focused {
                edit.request_focus();
                e.focused = true;
            }
            if edit.lost_focus() {
                if e.text != f.value {
                    o.actions.push(Action::FormPreview(FormPreviewAction::Fill {
                        path: f.path.clone(),
                        value: std::mem::take(&mut e.text),
                        name: f.name.clone(),
                        choices: o.choices,
                    }));
                }
                *editing = None;
            }
            continue;
        }
        if !o.enabled {
            continue;
        }
        let id = ui.id().with(("form-preview-field", o.page, n));
        let hit = ui
            .interact(r, id, egui::Sense::click())
            .on_hover_text(t::fill_hover(f.name.as_deref()));
        if hit.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
            ui.painter().rect_stroke(
                r,
                0.0,
                egui::Stroke::new(
                    1.5,
                    egui_shell::theme::Theme::canvas_selection_ink(ui.ctx()),
                ),
                egui::StrokeKind::Outside,
            );
        }
        if hit.clicked() {
            *editing = Some(Editing {
                page: o.page,
                index: n,
                text: f.value.clone(),
                focused: false,
            });
        }
    }
}
