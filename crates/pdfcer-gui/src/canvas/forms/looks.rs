//! A button's rollover and down looks while filling (ISO 32000-1 §12.5.5
//! Table 168): the push button, check box or radio under the pointer is drawn
//! from its `/AP /R` while hovered and its `/AP /D` while pressed, as Acrobat
//! does. A cursor affordance (R8b): only the canvas render of that one page
//! asks for it, through `OpenDoc::widget_look`; print, export and save never.
//!
//! The overlay decides the look each frame and parks it in `egui::Memory`;
//! the next frame's `canvas::present` [`take`]s it into the document before
//! the render key is built, so a frame on which filling is not offered leaves
//! no look behind.

use std::path::PathBuf;
use std::sync::Arc;

use egui::{Id, Rect};
use pdfcer_core::annot::AppearanceKind;
use pdfcer_core::forms::FieldType;
use pdfcer_core::graph::ObjectGraph;
use pdfcer_core::object::{ObjId, Object};

use crate::app::state::OpenDoc;
use crate::canvas::strip::PageView;

/// `egui::Memory` key for the widgets that carry a look, per revision.
const BOXES_KEY: &str = "pdfcer-canvas-form-looks"; // ui-text-exempt: internal memory id, never displayed
/// `egui::Memory` key for the look this frame decided.
const WANTED_KEY: &str = "pdfcer-canvas-form-look"; // ui-text-exempt: internal memory id, never displayed

/// One button widget whose `/AP` carries `/R` or `/D`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LookBox {
    /// 0-based page index.
    pub page: usize,
    /// The widget annotation's object id — what the render's
    /// `appearance_states` is keyed on.
    pub id: ObjId,
    /// Where it is, in canvas space.
    pub rect: Rect,
    /// `/AP` has `/R`.
    pub rollover: bool,
    /// `/AP` has `/D`.
    pub down: bool,
}

/// A look the render is asked for: `(page, widget, kind)`.
pub(crate) type Look = (usize, ObjId, AppearanceKind);

/// The look the last frame's overlay decided, removed so a frame that does
/// not reach the overlay asks for none.
pub(crate) fn take(ctx: &egui::Context) -> Option<Look> {
    ctx.data_mut(|d| d.remove_temp::<Option<Look>>(Id::new(WANTED_KEY)))
        .flatten()
}

/// Decide this frame's look from the pointer and park it for [`take`].
pub(super) fn track(ctx: &egui::Context, doc: &OpenDoc, pages: &[PageView]) {
    let list = boxes(ctx, doc);
    let wanted = wanted(ctx, pages, &list);
    if wanted != doc.widget_look {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            format!("form-look {}", describe(wanted))
        });
        ctx.request_repaint();
    }
    ctx.data_mut(|d| d.insert_temp(Id::new(WANTED_KEY), wanted));
}

/// The trace's words for a look.
pub(crate) fn describe(look: Option<Look>) -> String {
    match look {
        None => "kind=none".to_owned(),
        Some((page, id, kind)) => format!(
            "kind={} page={page} widget={}",
            match kind {
                AppearanceKind::Rollover => "rollover",
                AppearanceKind::Down => "down",
                _ => "normal",
            },
            id.num
        ),
    }
}

/// The widget under the pointer and the look it has for the button's state:
/// `/D` while the primary button is held after a press inside it, else `/R`.
fn wanted(ctx: &egui::Context, pages: &[PageView], list: &[LookBox]) -> Option<Look> {
    let (pos, held, origin) = ctx.input(|i| {
        (
            i.pointer.latest_pos(),
            i.pointer.primary_down(),
            i.pointer.press_origin(),
        )
    });
    let pos = pos?;
    let view = pages.iter().find(|v| v.map.image_rect().contains(pos))?;
    let point = view.map.to_page(pos);
    let hit = list
        .iter()
        .rfind(|b| b.page == view.page && b.rect.contains(point))?;
    let pressed = held
        && origin.is_some_and(|o| {
            view.map.image_rect().contains(o) && hit.rect.contains(view.map.to_page(o))
        });
    let kind = if pressed {
        if !hit.down {
            return None;
        }
        AppearanceKind::Down
    } else {
        if !hit.rollover {
            return None;
        }
        AppearanceKind::Rollover
    };
    Some((hit.page, hit.id, kind))
}

/// Every button widget with a rollover or down look, built once per
/// `(document, revision)`.
fn boxes(ctx: &egui::Context, doc: &OpenDoc) -> Arc<Vec<LookBox>> {
    let id = Id::new(BOXES_KEY);
    let key = (doc.path.clone(), doc.edit_epoch);
    if let Some((cached, list)) = ctx.data(|d| d.get_temp::<(Key, Arc<Vec<LookBox>>)>(id))
        && cached == key
    {
        return list;
    }
    let list = Arc::new(build(doc));
    ctx.data_mut(|d| d.insert_temp(id, (key, Arc::clone(&list))));
    list
}

/// The cache key: which document, at which revision.
type Key = (PathBuf, u64);

/// Walk the form's button widgets and keep those whose `/AP` has `/R` or `/D`.
fn build(doc: &OpenDoc) -> Vec<LookBox> {
    let view = doc.session.view();
    let Some(form) = pdfcer_core::forms::parse_acroform(&view) else {
        return Vec::new();
    };
    let buttons: std::collections::HashSet<ObjId> = form
        .fields
        .iter()
        .filter(|f| f.field_type == Some(FieldType::Button))
        .flat_map(|f| f.widgets.iter().map(|w| w.id))
        .collect();
    let mut out = Vec::new();
    for (page_index, page) in doc.pages.iter().enumerate() {
        for (id, rect) in doc.session.widget_rects(page_index) {
            if !buttons.contains(&id) {
                continue;
            }
            let (rollover, down) = ap_states(&view, id);
            if !(rollover || down) {
                continue;
            }
            if let Some(rect) = crate::canvas::mapping::annot_canvas_rect(rect, page) {
                out.push(LookBox {
                    page: page_index,
                    id,
                    rect,
                    rollover,
                    down,
                });
            }
        }
    }
    out
}

/// Whether widget `id`'s `/AP` has an `/R` and a `/D` entry.
fn ap_states<G: ObjectGraph + ?Sized>(graph: &G, id: ObjId) -> (bool, bool) {
    let ap = graph
        .value(id)
        .and_then(Object::as_dict)
        .and_then(|d| d.get(b"AP"))
        .map(|o| graph.resolve(o))
        .and_then(Object::as_dict);
    ap.map_or((false, false), |ap| {
        (ap.get(b"R").is_some(), ap.get(b"D").is_some())
    })
}
