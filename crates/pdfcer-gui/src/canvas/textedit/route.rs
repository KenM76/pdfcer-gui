//! # `canvas::textedit::route` — where a text-tool click goes
//!
//! Contract: [`click`] is the one route from a text-tool click to
//! [`super::click`]. A click on a note or on a fillable form field settles any
//! open draft and opens no caret, leaving the click to that surface. Otherwise
//! the caret is placed; a refusal is raised as
//! [`CanvasDecline::TextClick`] for the status bar and traced as
//! `text-edit-declined reason=…`, with `via` appended.
//!
//! Design and rationale: `docs/modules/pdfcer-gui/canvas/textedit/route.md`.

use egui::Pos2;

use super::{Click, TextEditKind};
use crate::app::actions::Action;
use crate::app::state::OpenDoc;
use crate::canvas::mapping::PageMapping;
use crate::canvas::pick::{PickClass, PickFilter};
use crate::canvas::tool::CanvasTool;
use crate::panels::objects::provider::ObjectModelProvider;
use pdfcer_gui_base::subactions::CanvasDecline;

/// One text-tool click, as `canvas::clicking` resolved it.
pub struct Routed<'a> {
    /// The egui context, for the draft and the form overlay's cache.
    pub ctx: &'a egui::Context,
    /// The document clicked.
    pub doc: &'a OpenDoc,
    /// Which page was clicked.
    pub page_index: usize,
    /// Which verb the caret opens with.
    pub kind: TextEditKind,
    /// The armed tool, for whether it fills forms.
    pub tool: CanvasTool,
    /// Where, in canvas space.
    pub point: Pos2,
    /// The page's canvas mapping, for the image pick.
    pub map: &'a PageMapping,
    /// The page's objects, for the image pick.
    pub targets: Option<&'a ObjectModelProvider>,
    /// Whether this click opened or closed a note's pop-up.
    pub on_note: bool,
    /// Appended to the refusal's trace line: empty or ` via=double-click`.
    pub via: &'static str,
}

/// Route the click; `true` when a caret was placed or moved.
pub fn click(r: &Routed<'_>, actions: &mut Vec<Action>) -> bool {
    if let Some(to) = elsewhere(r) {
        super::settle(r.ctx, actions);
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("text-click-routed to={to}")
        });
        return false;
    }
    let click = Click {
        doc: r.doc,
        page_index: r.page_index,
        kind: r.kind,
        canvas_point: r.point,
        on_image: on_image(r),
    };
    match super::click(r.ctx, &click, actions) {
        Ok(()) => true,
        Err(refusal) => {
            actions.push(Action::DeclineOnCanvas(CanvasDecline::TextClick(refusal)));
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("text-edit-declined reason={refusal:?}{}", r.via)
            });
            false
        }
    }
}

/// The surface other than the caret that owns this click, if any.
fn elsewhere(r: &Routed<'_>) -> Option<&'static str> {
    if r.on_note {
        return Some("note"); // ui-text-exempt: trace token, never displayed
    }
    let field = crate::canvas::forms::offer(r.doc, r.tool)
        && crate::canvas::forms::boxes::hit(
            &crate::canvas::forms::placed(r.ctx, r.doc).boxes,
            r.page_index,
            r.point,
        )
        .is_some();
    field.then_some("field") // ui-text-exempt: trace token, never displayed
}

/// Whether an image is the topmost picture under the click.
fn on_image(r: &Routed<'_>) -> bool {
    r.targets.is_some_and(|t| {
        crate::canvas::input::topmost(
            t,
            r.page_index,
            r.point,
            r.map,
            PickFilter::none().with(PickClass::Image, true),
            crate::canvas::smart::scope(r.ctx, r.page_index),
        )
        .is_some()
    })
}
