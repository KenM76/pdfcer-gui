//! # `app::dispatch::dimdisplay` — show a circular ce dimension as a radius or
//! a diameter, from the canvas menu

use pdfcer_core::dimension::{DimensionId, DimensionKind};

use crate::app::PdfcerApp;
use crate::app::actions::Action;
use crate::app::actions::dimensions::DimensionAction;
use crate::app::state::{OpenDoc, Status};
use crate::canvas::selection::{AnnotKind, SelectionState};

/// Switch the selected circular ce dimension to its diameter.
pub(crate) const DIAMETER: &str = "format.dimension_diameter"; // ui-text-exempt: a command id, never displayed

/// Switch the selected circular ce dimension to its radius.
pub(crate) const RADIUS: &str = "format.dimension_radius"; // ui-text-exempt: a command id, never displayed

/// Switch the selected closed perimeter ce dimension to its enclosed area.
pub(crate) const AREA: &str = "format.dimension_area"; // ui-text-exempt: a command id, never displayed

/// Switch the selected perimeter ce dimension back to its perimeter.
pub(crate) const PERIMETER: &str = "format.dimension_perimeter"; // ui-text-exempt: a command id, never displayed

/// Whether `id` is one of the switches this module dispatches.
#[must_use]
pub(crate) fn claims(id: &str) -> bool {
    matches!(id, DIAMETER | RADIUS | AREA | PERIMETER)
}

/// The selected ce dimension and whether it shows its diameter, when the
/// selection is one circular ce dimension. Read from the session's model, so
/// it includes unsaved edits.
///
/// `selection` is passed rather than read from `doc`: while the canvas runs its
/// frame it holds the selection taken off the document, and `doc.selection` is
/// then empty.
#[must_use]
pub(crate) fn circular(doc: &OpenDoc, selection: &SelectionState) -> Option<(DimensionId, bool)> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::CeDimension {
        return None;
    }
    let model = doc.session.dimension_model();
    let record = model
        .dimensions()
        .iter()
        .find(|r| r.annot == Some(annot.target.id))?;
    match record.kind {
        DimensionKind::Circular { show_diameter, .. } => Some((record.id, show_diameter)),
        _ => None,
    }
}

/// The selected ce dimension and whether it shows its area, when the selection
/// is one closed perimeter ce dimension: the engine refuses an area on an open
/// path, so an open one is not offered the switch.
#[must_use]
pub(crate) fn perimeter(doc: &OpenDoc, selection: &SelectionState) -> Option<(DimensionId, bool)> {
    let annot = selection.annot()?;
    if annot.target.kind != AnnotKind::CeDimension {
        return None;
    }
    let model = doc.session.dimension_model();
    let record = model
        .dimensions()
        .iter()
        .find(|r| r.annot == Some(annot.target.id))?;
    match &record.kind {
        DimensionKind::Perimeter {
            closed: true,
            area,
            points,
            ..
        } if points.len() >= 3 => Some((record.id, *area)),
        _ => None,
    }
}

/// The action `id` asks of the selection, or `None` when the selection cannot
/// take it or already shows the asked-for measure.
fn switch(doc: &OpenDoc, id: &str) -> Option<Action> {
    let action = match id {
        DIAMETER | RADIUS => {
            let want = id == DIAMETER;
            let (dimension, shown) = circular(doc, &doc.selection)?;
            if shown == want {
                return None;
            }
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!(
                    "dimension-display id={} diameter={}",
                    dimension.0,
                    u8::from(want)
                )
            });
            DimensionAction::SetDisplay {
                dimension,
                show_diameter: want,
            }
        }
        AREA | PERIMETER => {
            let want = id == AREA;
            let (dimension, shown) = perimeter(doc, &doc.selection)?;
            if shown == want {
                return None;
            }
            crate::diag::trace(|| {
                // ui-text-exempt: diagnostic trace, never displayed.
                format!("dimension-area id={} area={}", dimension.0, u8::from(want))
            });
            DimensionAction::SetArea {
                dimension,
                area: want,
            }
        }
        _ => return None,
    };
    Some(Action::Dimension(action))
}

/// Raise the switch. A chord or a stale menu can arrive with no matching ce
/// dimension selected, or with it already showing the asked-for measure; both
/// are traced and raise nothing.
pub(super) fn dispatch(app: &mut PdfcerApp, id: &str, actions: &mut Vec<Action>) {
    if !app.capabilities().author_measure {
        crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=mode-cannot-author-measure")
        });
        return;
    }
    let Status::Open(doc) = &app.status else {
        return;
    };
    match switch(doc, id) {
        Some(action) => actions.push(action),
        None => crate::diag::trace(|| {
            // ui-text-exempt: diagnostic trace, never displayed.
            format!("command-declined id={id} reason=no-dimension-to-switch")
        }),
    }
}
